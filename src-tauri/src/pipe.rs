use crate::cli::NotifyRequest;
use std::io::Write;

#[cfg(debug_assertions)]
const PIPE_NAME: &str = r"\\.\pipe\agent-toast-dev";

#[cfg(not(debug_assertions))]
const PIPE_NAME: &str = r"\\.\pipe\agent-toast";

/// Check if a pipe server is already running by attempting to open the pipe.
pub fn is_server_running() -> bool {
    use std::fs::OpenOptions;
    OpenOptions::new().write(true).open(PIPE_NAME).is_ok()
}

pub fn try_send(request: &NotifyRequest) -> Result<bool, Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;

    let file = OpenOptions::new().write(true).open(PIPE_NAME);
    match file {
        Ok(mut f) => {
            let data = serde_json::to_vec(request)?;
            let len = (data.len() as u32).to_le_bytes();
            f.write_all(&len)?;
            f.write_all(&data)?;
            f.flush()?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

pub fn start_server<F>(on_request: F)
where
    F: Fn(NotifyRequest) + Send + 'static,
{
    std::thread::spawn(move || {
        let mut fail_count: u32 = 0;
        loop {
            if let Err(e) = run_pipe_instance(&on_request) {
                fail_count += 1;
                let delay = std::cmp::min(100 * fail_count as u64, 5000);
                log::error!("Pipe error (attempt {fail_count}): {e}");
                std::thread::sleep(std::time::Duration::from_millis(delay));
            } else {
                fail_count = 0;
            }
        }
    });
}

#[cfg(windows)]
fn run_pipe_instance<F>(on_request: &F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(NotifyRequest),
{
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND};
    use windows::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
        PIPE_TYPE_BYTE, PIPE_WAIT,
    };

    let pipe_name = HSTRING::from(PIPE_NAME);
    let handle: HANDLE = unsafe {
        CreateNamedPipeW(
            &pipe_name,
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            255,
            4096,
            4096,
            0,
            None,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let err = unsafe { windows::Win32::Foundation::GetLastError() };
        return Err(format!("Failed to create named pipe (error {})", err.0).into());
    }

    // ConnectNamedPipe returns Result<()> in windows 0.58
    log::debug!("[PIPE] Waiting for client connection...");
    unsafe { ConnectNamedPipe(handle, None) }
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    log::debug!("[PIPE] Client connected");

    // Read length prefix
    let mut len_buf = [0u8; 4];
    let mut bytes_read = 0u32;
    if let Err(e) = unsafe { ReadFile(handle, Some(&mut len_buf), Some(&mut bytes_read), None) } {
        // Broken pipe = client connected and immediately disconnected
        // (e.g., is_server_running() probe). Not a real error.
        let is_broken_pipe = e.code() == windows::Win32::Foundation::ERROR_BROKEN_PIPE.to_hresult();
        unsafe {
            let _ = DisconnectNamedPipe(handle);
            let _ = CloseHandle(handle);
        }
        if is_broken_pipe {
            log::debug!("[PIPE] Client disconnected without sending data");
            return Ok(());
        }
        return Err(Box::new(e));
    }
    let len = u32::from_le_bytes(len_buf) as usize;

    // Read payload
    let mut buf = vec![0u8; len];
    let mut total_read = 0usize;
    while total_read < len {
        let mut br = 0u32;
        unsafe { ReadFile(handle, Some(&mut buf[total_read..]), Some(&mut br), None) }
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        total_read += br as usize;
    }

    match serde_json::from_slice::<NotifyRequest>(&buf) {
        Ok(req) => {
            log::debug!(
                "[PIPE] Received request: event={}, pid={}",
                req.event,
                req.pid
            );
            on_request(req);
            log::debug!("[PIPE] Callback completed");
        }
        Err(e) => {
            log::error!("[PIPE] JSON parse error: {}", e);
        }
    }

    unsafe {
        if let Err(e) = DisconnectNamedPipe(handle) {
            log::warn!("[PIPE] DisconnectNamedPipe failed: {:?}", e);
        }
        if let Err(e) = CloseHandle(handle) {
            log::warn!("[PIPE] CloseHandle failed: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn run_pipe_instance<F>(_on_request: &F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(NotifyRequest),
{
    Err("Named pipes are only supported on Windows".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// try_send가 사용하는 와이어 포맷 검증: [4바이트 LE 길이][JSON 페이로드]
    /// 실제 수신 측 디코딩 흐름을 시뮬레이션하여 프레임 단위로 검증
    #[test]
    fn wire_format_length_prefix_and_json() {
        let req = NotifyRequest {
            pid: 1234,
            event: "task_complete".to_string(),
            message: Some("빌드 완료".to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len_bytes = (data.len() as u32).to_le_bytes();

        // 프레임 조립: [4바이트 LE 길이][JSON 페이로드]
        let mut frame = Vec::new();
        frame.extend_from_slice(&len_bytes);
        frame.extend_from_slice(&data);

        // 수신 측 디코딩 시뮬레이션
        assert_eq!(frame.len(), 4 + data.len());
        let received_len = u32::from_le_bytes(frame[0..4].try_into().unwrap()) as usize;
        assert_eq!(received_len, data.len());
        let decoded: NotifyRequest = serde_json::from_slice(&frame[4..4 + received_len]).unwrap();
        assert_eq!(decoded.pid, 1234);
        assert_eq!(decoded.event, "task_complete");
        assert_eq!(decoded.message.as_deref(), Some("빌드 완료"));
    }

    #[test]
    fn wire_format_minimal_request() {
        let req = NotifyRequest {
            pid: 1,
            event: "error".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len = (data.len() as u32).to_le_bytes();

        // 프레임 조립
        let mut frame = Vec::new();
        frame.extend_from_slice(&len);
        frame.extend_from_slice(&data);

        // 수신 측 디코딩 시뮬레이션
        let received_len = u32::from_le_bytes(frame[0..4].try_into().unwrap()) as usize;
        let received_data = &frame[4..4 + received_len];
        let decoded: NotifyRequest = serde_json::from_slice(received_data).unwrap();
        assert_eq!(decoded.pid, 1);
        assert_eq!(decoded.event, "error");
        assert!(decoded.message.is_none());
    }

    #[test]
    fn wire_format_with_process_tree() {
        let req = NotifyRequest {
            pid: 5678,
            event: "user_input_required".to_string(),
            message: Some("입력 대기".to_string()),
            title_hint: Some("my-project".to_string()),
            process_tree: Some(vec![100, 200, 300, 400]),
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.process_tree, Some(vec![100, 200, 300, 400]));
        assert_eq!(decoded.title_hint.as_deref(), Some("my-project"));
    }

    #[test]
    fn wire_format_unicode_message() {
        let msg = "한글 메시지 🎉 テスト";
        let req = NotifyRequest {
            pid: 1,
            event: "task_complete".to_string(),
            message: Some(msg.to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len_bytes = (data.len() as u32).to_le_bytes();
        let decoded_len = u32::from_le_bytes(len_bytes) as usize;

        // UTF-8 멀티바이트 문자가 포함되므로 바이트 길이가 문자 수보다 커야 함
        assert!(decoded_len > msg.chars().count());
        // JSON 페이로드이므로 메시지 문자열의 UTF-8 바이트보다도 커야 함
        assert!(decoded_len > msg.len());

        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.message.as_deref(), Some(msg));
    }

    #[test]
    fn wire_format_large_payload() {
        let long_message = "A".repeat(10000);
        let req = NotifyRequest {
            pid: 1,
            event: "task_complete".to_string(),
            message: Some(long_message.clone()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len_bytes = (data.len() as u32).to_le_bytes();
        let received_len = u32::from_le_bytes(len_bytes) as usize;

        assert_eq!(received_len, data.len());
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.message.as_deref(), Some(long_message.as_str()));
    }

    #[test]
    fn wire_format_with_source_field() {
        for source in ["claude", "codex", "updater"] {
            let req = NotifyRequest {
                pid: 1,
                event: "test".to_string(),
                message: None,
                title_hint: None,
                process_tree: None,
                source: source.into(),
            };

            let data = serde_json::to_vec(&req).unwrap();
            let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
            assert_eq!(decoded.source, source);
        }
    }

    #[test]
    fn wire_format_all_fields_populated() {
        let req = NotifyRequest {
            pid: 99999,
            event: "user_input_required".to_string(),
            message: Some("권한 승인이 필요합니다".to_string()),
            title_hint: Some("my-awesome-project".to_string()),
            process_tree: Some(vec![1000, 2000, 3000, 4000, 5000]),
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len_bytes = (data.len() as u32).to_le_bytes();

        // 프레임 조립
        let mut frame = Vec::new();
        frame.extend_from_slice(&len_bytes);
        frame.extend_from_slice(&data);

        // 수신 측 디코딩
        let received_len = u32::from_le_bytes(frame[0..4].try_into().unwrap()) as usize;
        let decoded: NotifyRequest = serde_json::from_slice(&frame[4..4 + received_len]).unwrap();

        assert_eq!(decoded.pid, 99999);
        assert_eq!(decoded.event, "user_input_required");
        assert_eq!(decoded.message.as_deref(), Some("권한 승인이 필요합니다"));
        assert_eq!(decoded.title_hint.as_deref(), Some("my-awesome-project"));
        assert_eq!(
            decoded.process_tree,
            Some(vec![1000, 2000, 3000, 4000, 5000])
        );
        assert_eq!(decoded.source, "claude");
    }

    #[test]
    fn wire_format_empty_message_string() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: Some("".to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.message.as_deref(), Some(""));
    }

    #[test]
    fn wire_format_special_characters_in_message() {
        let special_msg = r#"Line1\nLine2\tTab "quoted" 'single' <tag> & symbol"#;
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: Some(special_msg.to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.message.as_deref(), Some(special_msg));
    }

    #[test]
    fn wire_format_length_prefix_byte_order() {
        // 리틀 엔디안 바이트 순서 확인
        let req = NotifyRequest {
            pid: 1,
            event: "x".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len = data.len() as u32;
        let len_bytes = len.to_le_bytes();

        // 리틀 엔디안: 최하위 바이트가 먼저
        assert_eq!(len_bytes[0], (len & 0xFF) as u8);
        assert_eq!(len_bytes[1], ((len >> 8) & 0xFF) as u8);
        assert_eq!(len_bytes[2], ((len >> 16) & 0xFF) as u8);
        assert_eq!(len_bytes[3], ((len >> 24) & 0xFF) as u8);
    }

    #[test]
    fn wire_format_max_reasonable_size() {
        // 100KB 페이로드 테스트
        let big_tree: Vec<u32> = (0..10000).collect();
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: Some("A".repeat(50000)),
            title_hint: Some("B".repeat(1000)),
            process_tree: Some(big_tree),
            source: "claude".into(),
        };

        let data = serde_json::to_vec(&req).unwrap();
        let len = data.len() as u32;

        // 길이가 u32 범위 내인지 확인
        assert!(len < u32::MAX);

        // 디코딩 가능한지 확인
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.pid, 1);
        assert!(decoded.process_tree.is_some());
        assert_eq!(decoded.process_tree.as_ref().unwrap().len(), 10000);
    }

    // ── Pipe name tests ──

    #[test]
    fn pipe_name_has_valid_format() {
        // Windows named pipe must start with \\.\pipe\
        assert!(PIPE_NAME.starts_with(r"\\.\pipe\"));
    }

    #[test]
    fn pipe_name_contains_app_identifier() {
        assert!(PIPE_NAME.contains("agent-toast"));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn pipe_name_debug_has_dev_suffix() {
        assert!(PIPE_NAME.ends_with("-dev"));
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn pipe_name_release_no_dev_suffix() {
        assert!(!PIPE_NAME.ends_with("-dev"));
    }

    // ── Frame encoding edge cases ──

    #[test]
    fn wire_format_zero_length_would_be_invalid_json() {
        // 길이가 0인 프레임은 빈 JSON이 아닌 파싱 실패를 야기함
        let empty_data: &[u8] = &[];
        let result = serde_json::from_slice::<NotifyRequest>(empty_data);
        assert!(result.is_err());
    }

    #[test]
    fn wire_format_partial_json_fails() {
        // 불완전한 JSON은 파싱 실패
        let partial = br#"{"pid":1,"event":"#;
        let result = serde_json::from_slice::<NotifyRequest>(partial);
        assert!(result.is_err());
    }

    #[test]
    fn wire_format_extra_fields_ignored() {
        // 알려지지 않은 필드는 무시됨 (forward compatibility)
        let json = r#"{"pid":1,"event":"test","unknown_field":"value","future_field":123}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.pid, 1);
        assert_eq!(req.event, "test");
    }

    #[test]
    fn wire_format_null_optional_fields() {
        // null 값은 None으로 처리됨
        let json = r#"{"pid":1,"event":"test","message":null,"title_hint":null}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert!(req.message.is_none());
        assert!(req.title_hint.is_none());
    }

    // ── is_server_running / try_send tests ──

    // is_server_running은 실제 파이프를 열어보는 동작이므로 단위 테스트에서 호출하지 않음

    // try_send는 실제 파이프 서버에 요청을 보내므로 단위 테스트에서 호출하지 않음
    // (실행 중인 앱에 알림이 뜨는 부작용 발생)

    #[test]
    fn try_send_serializes_request_correctly() {
        // try_send 내부에서 serde_json::to_vec 사용하는 것과 동일한 직렬화 검증
        let req = NotifyRequest {
            pid: 42,
            event: "task_complete".to_string(),
            message: Some("테스트 메시지".to_string()),
            title_hint: Some("project-dir".to_string()),
            process_tree: Some(vec![1, 2, 3]),
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let len = (data.len() as u32).to_le_bytes();

        // try_send가 보내는 형식 검증
        assert_eq!(len.len(), 4);
        assert!(!data.is_empty());

        // 역직렬화 가능 확인
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.pid, 42);
        assert_eq!(decoded.source, "claude");
    }

    // ── PIPE_NAME constant tests ──

    #[test]
    fn pipe_name_length() {
        assert!(PIPE_NAME.len() > r"\\.\pipe\".len());
    }

    #[test]
    fn pipe_name_is_valid_windows_path() {
        // Windows named pipe 경로는 \\.\pipe\ 접두사 필수
        assert!(PIPE_NAME.starts_with(r"\\.\pipe\"));
        // 접두사 이후에 이름이 있어야 함
        let name = &PIPE_NAME[r"\\.\pipe\".len()..];
        assert!(!name.is_empty());
    }

    // ── Boundary value tests ──

    #[test]
    fn wire_format_pid_zero() {
        let req = NotifyRequest {
            pid: 0,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.pid, 0);
    }

    #[test]
    fn wire_format_pid_u32_max() {
        let req = NotifyRequest {
            pid: u32::MAX,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.pid, u32::MAX);
    }

    #[test]
    fn wire_format_empty_event() {
        let req = NotifyRequest {
            pid: 1,
            event: "".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.event, "");
    }

    #[test]
    fn wire_format_empty_source() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.source, "");
    }

    #[test]
    fn wire_format_single_element_process_tree() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: Some(vec![42]),
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.process_tree, Some(vec![42]));
    }

    #[test]
    fn wire_format_empty_process_tree_vs_none() {
        // Some(vec![])와 None은 직렬화/역직렬화 시 구분되어야 함
        let req_empty = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: Some(vec![]),
            source: "claude".into(),
        };
        let req_none = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };

        let data_empty = serde_json::to_vec(&req_empty).unwrap();
        let data_none = serde_json::to_vec(&req_none).unwrap();

        let decoded_empty: NotifyRequest = serde_json::from_slice(&data_empty).unwrap();
        let decoded_none: NotifyRequest = serde_json::from_slice(&data_none).unwrap();

        assert_eq!(decoded_empty.process_tree, Some(vec![]));
        assert!(decoded_none.process_tree.is_none());
    }

    #[test]
    fn wire_format_process_tree_with_zero_pid() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: Some(vec![0, 0, 0]),
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.process_tree, Some(vec![0, 0, 0]));
    }

    #[test]
    fn wire_format_process_tree_with_u32_max() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: Some(vec![u32::MAX]),
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.process_tree, Some(vec![u32::MAX]));
    }

    #[test]
    fn wire_format_length_prefix_for_small_payload() {
        // 최소 페이로드의 길이 프리픽스가 4바이트 내에 들어가는지
        let req = NotifyRequest {
            pid: 0,
            event: "".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let len = data.len() as u32;
        // 최소 페이로드도 유효한 JSON이므로 2바이트 이상
        assert!(len >= 2);
        // u32 범위 내
        assert!(len < u32::MAX);
    }

    #[test]
    fn wire_format_message_with_only_whitespace() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: Some("   \t\n  ".to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.message.as_deref(), Some("   \t\n  "));
    }

    #[test]
    fn wire_format_title_hint_very_long() {
        let long_hint = "가".repeat(10000); // 유니코드 3바이트 × 10000
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: Some(long_hint.clone()),
            process_tree: None,
            source: "claude".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_slice(&data).unwrap();
        assert_eq!(decoded.title_hint.as_deref(), Some(long_hint.as_str()));
    }
}
