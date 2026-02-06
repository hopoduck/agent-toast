// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use agent_toast_lib::cli::{Cli, NotifyRequest};
use agent_toast_lib::pipe;
use agent_toast_lib::win32;

fn get_parent_pid() -> u32 {
    #[cfg(windows)]
    {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        let my_pid = std::process::id();
        let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
            return my_pid;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        unsafe {
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ProcessID == my_pid {
                        return entry.th32ParentProcessID;
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
        }
        my_pid
    }
    #[cfg(not(windows))]
    {
        std::process::id()
    }
}

/// Try to acquire a global named mutex. Returns the handle if this is the first instance.
/// The handle must be kept alive for the lifetime of the app.
#[cfg(windows)]
fn try_acquire_singleton() -> Option<windows::Win32::Foundation::HANDLE> {
    use windows::core::w;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Threading::CreateMutexW;

    #[cfg(debug_assertions)]
    const MUTEX_NAME: windows::core::PCWSTR = w!("agent-toast-singleton-dev");

    #[cfg(not(debug_assertions))]
    const MUTEX_NAME: windows::core::PCWSTR = w!("agent-toast-singleton");

    let handle: HANDLE =
        unsafe { CreateMutexW(None, true, MUTEX_NAME) }.unwrap_or_default();
    if handle.is_invalid() || handle == HANDLE::default() {
        return None;
    }
    // ERROR_ALREADY_EXISTS = 183
    let last_err = unsafe { windows::Win32::Foundation::GetLastError() };
    if last_err.0 == 183 {
        // Another instance already holds the mutex
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(handle);
        }
        return None;
    }
    Some(handle)
}

#[cfg(not(windows))]
fn try_acquire_singleton() -> Option<()> {
    Some(())
}

fn main() {
    let args = Cli::parse();

    // --codex mode: parse JSON from Codex CLI
    if args.codex {
        let json_str = args.codex_json.unwrap_or_default();
        let codex_payload: serde_json::Value =
            serde_json::from_str(&json_str).unwrap_or_else(|e| {
                eprintln!("[ERROR] Failed to parse Codex JSON: {}", e);
                std::process::exit(1);
            });

        let codex_type = codex_payload["type"]
            .as_str()
            .unwrap_or("agent-turn-complete");
        let event = codex_type.replace('-', "_");

        let message = codex_payload["last-assistant-message"].as_str().map(|s| {
            // Truncate long messages for notification display
            if s.len() > 200 {
                format!("{}...", &s[..200])
            } else {
                s.to_string()
            }
        });

        let title_hint = codex_payload["cwd"].as_str().map(|cwd| {
            std::path::Path::new(cwd)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| cwd.to_string())
        });

        let pid = get_parent_pid();
        let process_tree = win32::get_process_tree(pid);

        let request = NotifyRequest {
            pid,
            event,
            message,
            title_hint,
            process_tree: Some(process_tree),
            source: "codex".into(),
        };

        match pipe::try_send(&request) {
            Ok(true) => return,
            _ => {
                let _mutex = try_acquire_singleton();
                if _mutex.is_none() {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = pipe::try_send(&request);
                    return;
                }
                agent_toast_lib::run_app(Some(request), false);
            }
        }
        return;
    }

    if args.daemon {
        // If another instance is already running, exit silently
        if pipe::is_server_running() {
            return;
        }
        let _mutex = try_acquire_singleton();
        if _mutex.is_none() {
            eprintln!("[INFO] Another instance is already starting up, exiting.");
            return;
        }
        // Start as daemon: just launch the Tauri app with no initial notification
        agent_toast_lib::run_app(None, false);
        return;
    }

    if args.setup || args.event.is_none() {
        let _mutex = try_acquire_singleton();
        if _mutex.is_none() {
            // Another instance exists; try to signal it via pipe, then exit
            eprintln!("[INFO] Another instance is already running, exiting.");
            return;
        }
        // No args or --setup: launch app with setup GUI (daemon also runs)
        agent_toast_lib::run_app(None, true);
        return;
    }

    // --pid defaults to parent process PID (the shell that invoked us).
    // We use parent because this process may exit before the daemon reads the snapshot.
    let pid = args.pid.unwrap_or_else(|| {
        let ppid = get_parent_pid();
        eprintln!(
            "[DEBUG] auto pid: self={}, parent={}",
            std::process::id(),
            ppid
        );
        ppid
    });
    let event = args
        .event
        .expect("--event is required when not using --daemon");

    // Pre-resolve process tree while the process is still alive
    let process_tree = win32::get_process_tree(pid);

    // Title hint: use --title arg, or fall back to CLAUDE_PROJECT_DIR env var.
    // Extract folder name from path (e.g. "C:\foo\bar" -> "bar") for window title matching.
    let title_hint = args
        .title
        .or_else(|| std::env::var("CLAUDE_PROJECT_DIR").ok())
        .map(|t| {
            std::path::Path::new(&t)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(t)
        });

    let request = NotifyRequest {
        pid,
        event,
        message: args.message,
        title_hint,
        process_tree: Some(process_tree),
        source: "claude".into(),
    };

    // Try to send to existing instance
    match pipe::try_send(&request) {
        Ok(true) => {
            // Sent to existing instance, exit
        }
        _ => {
            // No daemon running — silently exit (user may have intentionally closed it)
        }
    }
}

