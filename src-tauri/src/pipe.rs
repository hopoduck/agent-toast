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
                eprintln!("Pipe error (attempt {fail_count}): {e}");
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
    unsafe { ConnectNamedPipe(handle, None) }
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

    // Read length prefix
    let mut len_buf = [0u8; 4];
    let mut bytes_read = 0u32;
    unsafe { ReadFile(handle, Some(&mut len_buf), Some(&mut bytes_read), None) }
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
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

    if let Ok(req) = serde_json::from_slice::<NotifyRequest>(&buf) {
        on_request(req);
    }

    unsafe {
        let _ = DisconnectNamedPipe(handle);
        let _ = CloseHandle(handle);
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
}
