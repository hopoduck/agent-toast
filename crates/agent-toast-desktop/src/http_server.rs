use crate::cli::NotifyRequest;
use std::time::Duration;

pub const MAX_BODY_BYTES: usize = 64 * 1024;
pub const HTTP_PATH: &str = "/notify";

/// Start the HTTP receiver on a dedicated thread. Binding failure triggers
/// exponential backoff retry (same pattern as pipe::start_server).
pub fn start_server<F>(bind_addr: &str, on_request: F)
where
    F: Fn(NotifyRequest) + Send + 'static,
{
    let addr = bind_addr.to_string();
    std::thread::spawn(move || {
        let mut fail_count: u32 = 0;
        loop {
            match run_once(&addr, &on_request) {
                Ok(()) => fail_count = 0,
                Err(e) => {
                    fail_count += 1;
                    let delay = std::cmp::min(100 * fail_count as u64, 5000);
                    log::error!("[HTTP] server error (attempt {fail_count}): {e}");
                    std::thread::sleep(Duration::from_millis(delay));
                }
            }
        }
    });
}

fn run_once<F>(addr: &str, on_request: &F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(NotifyRequest),
{
    let server = tiny_http::Server::http(addr).map_err(|e| e.to_string())?;
    log::info!("[HTTP] listening on {}", addr);
    loop {
        let req = match server.recv() {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[HTTP] recv error: {e}");
                continue;
            }
        };
        handle_request(req, on_request);
    }
}

fn handle_request<F>(mut req: tiny_http::Request, on_request: &F)
where
    F: Fn(NotifyRequest),
{
    use tiny_http::{Method, Response, StatusCode};

    if req.method() != &Method::Post || req.url() != HTTP_PATH {
        let _ = req.respond(Response::empty(StatusCode(404)));
        return;
    }

    let body_len = req.body_length();
    match body_len {
        Some(n) if n > MAX_BODY_BYTES => {
            let _ = req.respond(Response::empty(StatusCode(413)));
            return;
        }
        Some(_) => {}
        None => {
            let _ = req.respond(Response::empty(StatusCode(411)));
            return;
        }
    }

    let mut buf = Vec::with_capacity(body_len.unwrap_or(0));
    use std::io::Read;
    // We need to read from the request body before consuming req.
    // Extract what we need first, then respond.
    let (read_ok, too_large) = {
        let reader = req.as_reader();
        let read_result = reader
            .take((MAX_BODY_BYTES as u64) + 1)
            .read_to_end(&mut buf);
        (read_result.is_ok(), buf.len() > MAX_BODY_BYTES)
    };

    if !read_ok || too_large {
        let _ = req.respond(Response::empty(StatusCode(400)));
        return;
    }

    match serde_json::from_slice::<NotifyRequest>(&buf) {
        Ok(parsed) => {
            on_request(parsed);
            let _ = req.respond(Response::empty(StatusCode(204)));
        }
        Err(e) => {
            log::warn!("[HTTP] JSON parse error: {e}");
            let _ = req.respond(Response::empty(StatusCode(400)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn pick_free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    fn bind_test_server() -> (u16, mpsc::Receiver<NotifyRequest>) {
        let (tx, rx) = mpsc::channel();
        let port = pick_free_port();
        let addr = format!("127.0.0.1:{}", port);
        start_server(&addr, move |req| {
            tx.send(req).ok();
        });
        // Brief wait for server to bind.
        std::thread::sleep(Duration::from_millis(200));
        (port, rx)
    }

    fn status_of(resp: Result<ureq::Response, ureq::Error>) -> u16 {
        match resp {
            Ok(r) => r.status(),
            Err(ureq::Error::Status(s, _)) => s,
            _ => 0,
        }
    }

    #[test]
    fn rejects_non_post_method() {
        let (port, _rx) = bind_test_server();
        let resp = ureq::get(&format!("http://127.0.0.1:{}/notify", port)).call();
        assert_eq!(status_of(resp), 404);
    }

    #[test]
    fn rejects_wrong_path() {
        let (port, _rx) = bind_test_server();
        let resp = ureq::post(&format!("http://127.0.0.1:{}/wrong", port))
            .set("Content-Type", "application/json")
            .send_string("{}");
        assert_eq!(status_of(resp), 404);
    }

    #[test]
    fn accepts_valid_notify_and_forwards_to_callback() {
        let (port, rx) = bind_test_server();
        let body = r#"{"pid":0,"event":"task_complete","message":"hi","title_hint":"p","source":"claude","hostname":"box1"}"#;
        let resp = ureq::post(&format!("http://127.0.0.1:{}/notify", port))
            .set("Content-Type", "application/json")
            .send_string(body);
        assert_eq!(status_of(resp), 204);
        let received = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("callback not called");
        assert_eq!(received.event, "task_complete");
        assert_eq!(received.hostname.as_deref(), Some("box1"));
    }

    #[test]
    fn rejects_oversize_payload() {
        let (port, _rx) = bind_test_server();
        let big = "x".repeat(MAX_BODY_BYTES + 1);
        let body = format!(
            r#"{{"pid":0,"event":"task_complete","message":"{}","source":"claude"}}"#,
            big
        );
        let resp = ureq::post(&format!("http://127.0.0.1:{}/notify", port))
            .set("Content-Type", "application/json")
            .send_string(&body);
        assert_eq!(status_of(resp), 413);
    }

    #[test]
    fn rejects_invalid_json() {
        let (port, _rx) = bind_test_server();
        let resp = ureq::post(&format!("http://127.0.0.1:{}/notify", port))
            .set("Content-Type", "application/json")
            .send_string("{ not json");
        assert_eq!(status_of(resp), 400);
    }
}
