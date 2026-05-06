use crate::cli::NotifyRequest;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const MAX_BODY_BYTES: usize = 64 * 1024;
pub const HTTP_PATH: &str = "/notify";
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Handle to a running HTTP server. Drop or call `stop()` to terminate the
/// background thread within ~POLL_INTERVAL.
pub struct HttpHandle {
    stop: Arc<AtomicBool>,
    addr: String,
}

impl HttpHandle {
    pub fn stop(self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }
}

/// Bind synchronously and spawn the receive loop. Returns an error if the
/// address cannot be bound — caller can surface this to the UI immediately.
pub fn start_server<F>(bind_addr: &str, on_request: F) -> Result<HttpHandle, String>
where
    F: Fn(NotifyRequest) + Send + 'static,
{
    let server = tiny_http::Server::http(bind_addr).map_err(|e| e.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let addr = bind_addr.to_string();
    log::info!("[HTTP] listening on {}", addr);

    let wake_addrs = wake_targets(bind_addr);
    std::thread::spawn(move || {
        loop {
            if stop_thread.load(Ordering::SeqCst) {
                log::info!("[HTTP] stop requested, shutting down");
                break;
            }
            match server.recv_timeout(POLL_INTERVAL) {
                Ok(Some(req)) => handle_request(req, &on_request),
                Ok(None) => continue,
                Err(e) => {
                    log::warn!("[HTTP] recv error: {e}");
                }
            }
        }
        // Drop tiny_http::Server explicitly. Its Drop sets close=true and tries
        // to self-connect to its bind address to unblock the internal acceptor
        // thread — but that connect fails on Windows when bound to 0.0.0.0 / [::].
        // After Drop returns, we kick the acceptor ourselves by connecting to a
        // routable equivalent (127.0.0.1 / ::1). The acceptor wakes, observes
        // close=true (set during Drop), and exits, releasing the listener.
        drop(server);
        for target in wake_addrs {
            let _ = TcpStream::connect_timeout(&target, Duration::from_millis(500));
        }
    });

    Ok(HttpHandle { stop, addr })
}

/// Routable equivalents of the bind address used to wake tiny_http's acceptor
/// thread after Drop. For wildcard binds (0.0.0.0 / [::]) we substitute the
/// loopback. Returns an empty Vec if parsing fails — fallback is just slower
/// shutdown, not a leak (only matters if Drop's own self-connect also failed).
fn wake_targets(bind_addr: &str) -> Vec<SocketAddr> {
    let Ok(mut iter) = bind_addr.to_socket_addrs() else {
        return Vec::new();
    };
    let Some(parsed) = iter.next() else {
        return Vec::new();
    };
    let port = parsed.port();
    match parsed {
        SocketAddr::V4(v4) if v4.ip().is_unspecified() => {
            vec![SocketAddr::from(([127, 0, 0, 1], port))]
        }
        SocketAddr::V6(v6) if v6.ip().is_unspecified() => {
            vec![
                SocketAddr::from(([127, 0, 0, 1], port)),
                SocketAddr::new(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), port),
            ]
        }
        addr => vec![addr],
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

    fn bind_test_server() -> (u16, mpsc::Receiver<NotifyRequest>, HttpHandle) {
        let (tx, rx) = mpsc::channel();
        let port = pick_free_port();
        let addr = format!("127.0.0.1:{}", port);
        let handle = start_server(&addr, move |req| {
            tx.send(req).ok();
        })
        .expect("bind should succeed");
        std::thread::sleep(Duration::from_millis(100));
        (port, rx, handle)
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
        let (port, _rx, _h) = bind_test_server();
        let resp = ureq::get(&format!("http://127.0.0.1:{}/notify", port)).call();
        assert_eq!(status_of(resp), 404);
    }

    #[test]
    fn rejects_wrong_path() {
        let (port, _rx, _h) = bind_test_server();
        let resp = ureq::post(&format!("http://127.0.0.1:{}/wrong", port))
            .set("Content-Type", "application/json")
            .send_string("{}");
        assert_eq!(status_of(resp), 404);
    }

    #[test]
    fn accepts_valid_notify_and_forwards_to_callback() {
        let (port, rx, _h) = bind_test_server();
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
        let (port, _rx, _h) = bind_test_server();
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
        let (port, _rx, _h) = bind_test_server();
        let resp = ureq::post(&format!("http://127.0.0.1:{}/notify", port))
            .set("Content-Type", "application/json")
            .send_string("{ not json");
        assert_eq!(status_of(resp), 400);
    }

    #[test]
    fn stop_releases_port_for_rebind() {
        let port = pick_free_port();
        let addr = format!("127.0.0.1:{}", port);
        let h1 = start_server(&addr, |_req| {}).expect("first bind");
        std::thread::sleep(Duration::from_millis(100));
        h1.stop();
        // Wait long enough for the recv_timeout loop to observe the stop flag
        // and release the socket.
        std::thread::sleep(POLL_INTERVAL + Duration::from_millis(500));
        let h2 = start_server(&addr, |_req| {}).expect("rebind after stop");
        h2.stop();
    }

    /// Regression: tiny_http's Drop tries to self-connect to its bind address
    /// to unblock the internal acceptor; on Windows that fails when bound to
    /// 0.0.0.0, leaving the listener alive. We compensate by waking via 127.0.0.1.
    #[test]
    fn stop_releases_wildcard_bind() {
        let port = pick_free_port();
        let addr = format!("0.0.0.0:{}", port);
        let h1 = start_server(&addr, |_req| {}).expect("first wildcard bind");
        std::thread::sleep(Duration::from_millis(100));
        h1.stop();
        std::thread::sleep(POLL_INTERVAL + Duration::from_millis(500));
        let h2 = start_server(&addr, |_req| {}).expect("rebind 0.0.0.0 after stop");
        h2.stop();
    }

    #[test]
    fn wake_targets_substitutes_loopback_for_wildcard_v4() {
        let targets = wake_targets("0.0.0.0:9999");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], SocketAddr::from(([127, 0, 0, 1], 9999)));
    }

    #[test]
    fn wake_targets_keeps_specific_address() {
        let targets = wake_targets("127.0.0.1:9999");
        assert_eq!(targets, vec![SocketAddr::from(([127, 0, 0, 1], 9999))]);
    }

    #[test]
    fn wake_targets_substitutes_loopback_for_wildcard_v6() {
        let targets = wake_targets("[::]:9999");
        assert!(targets.iter().any(|a| a.is_ipv4() && a.port() == 9999));
        assert!(targets.iter().any(|a| a.is_ipv6() && a.port() == 9999));
    }

    #[test]
    fn duplicate_bind_returns_error() {
        let (port, _rx, _h) = bind_test_server();
        let addr = format!("127.0.0.1:{}", port);
        let result = start_server(&addr, |_req| {});
        assert!(result.is_err(), "second bind on same port must fail");
    }
}
