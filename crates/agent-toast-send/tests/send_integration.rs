use std::time::Duration;

#[test]
fn send_posts_to_endpoint_with_all_fields() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr();
    let port = addr.to_ip().expect("expected IP addr").port();
    let url = format!("http://127.0.0.1:{}", port);

    let received = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
    let received_clone = received.clone();
    std::thread::spawn(move || {
        if let Ok(mut req) = server.recv() {
            let mut body = String::new();
            req.as_reader().read_to_string(&mut body).ok();
            *received_clone.lock().unwrap() = Some(body);
            let _ = req.respond(tiny_http::Response::empty(204));
        }
    });

    let exe = env!("CARGO_BIN_EXE_agent-toast-send");
    let status = std::process::Command::new(exe)
        .args([
            "--url",
            &url,
            "--event",
            "task_complete",
            "--message",
            "integration test",
            "--hostname",
            "test-box",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    std::thread::sleep(Duration::from_millis(200));
    let body = received.lock().unwrap().clone().expect("no body received");
    assert!(body.contains(r#""event":"task_complete""#));
    assert!(body.contains(r#""hostname":"test-box""#));
    assert!(body.contains(r#""message":"integration test""#));
}

#[test]
fn send_returns_zero_when_server_unreachable() {
    // Use a port that's certainly not listening
    let exe = env!("CARGO_BIN_EXE_agent-toast-send");
    let status = std::process::Command::new(exe)
        .args([
            "--url",
            "http://127.0.0.1:1", // reserved/unlikely port
            "--event",
            "task_complete",
            "--timeout-ms",
            "500",
            "--quiet",
        ])
        .status()
        .unwrap();
    // exit 0 per spec even on failure — hook must not be blocked
    assert!(status.success());
}
