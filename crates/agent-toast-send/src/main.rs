//! `agent-toast-send` — remote notification CLI.
//!
//! Invoked from Claude Code hooks on Linux servers to POST notifications
//! to the Windows desktop's HTTP receiver. Registers/removes its own hook
//! entries via `init` and `uninstall`.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "agent-toast-send",
    version,
    about = "Send remote notifications to Agent Toast desktop"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    send_args: SendArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Send a single notification (default when no subcommand is given)
    Send(SendArgs),
    /// Register default hooks in ~/.claude/settings.json
    Init(InitArgs),
    /// Remove agent-toast hooks from ~/.claude/settings.json
    Uninstall,
}

#[derive(clap::Args, Debug, Default, Clone)]
struct SendArgs {
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    event: Option<String>,
    #[arg(long)]
    message: Option<String>,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    hostname: Option<String>,
    #[arg(long, default_value = "claude")]
    source: String,
    #[arg(long, default_value_t = 2000)]
    timeout_ms: u64,
    #[arg(long)]
    quiet: bool,
}

#[derive(clap::Args, Debug)]
struct InitArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    hostname: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let exit = match cli.command {
        Some(Command::Init(args)) => run_init(args),
        Some(Command::Uninstall) => run_uninstall(),
        Some(Command::Send(args)) => run_send(args),
        None => run_send(cli.send_args),
    };
    std::process::exit(exit);
}

fn build_request(args: &SendArgs) -> agent_toast_core::NotifyRequest {
    let hostname_val = args
        .hostname
        .clone()
        .or_else(|| {
            hostname::get()
                .ok()
                .and_then(|s| s.into_string().ok())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| Some("unknown".into()));

    let title_hint = args.title.clone().or_else(|| {
        std::env::var("CLAUDE_PROJECT_DIR")
            .ok()
            .filter(|p| !p.is_empty())
            .map(|p| {
                std::path::Path::new(&p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(p)
            })
    });

    agent_toast_core::NotifyRequest {
        pid: 0,
        event: args.event.clone().unwrap_or_default(),
        message: args.message.clone(),
        title_hint,
        process_tree: None,
        source: args.source.clone(),
        hostname: hostname_val,
    }
}

fn run_send(args: SendArgs) -> i32 {
    let url = match &args.url {
        Some(u) if !u.is_empty() => u.clone(),
        _ => {
            if !args.quiet {
                eprintln!("error: --url is required");
            }
            return 2;
        }
    };
    if args.event.is_none() || args.event.as_deref() == Some("") {
        if !args.quiet {
            eprintln!("error: --event is required");
        }
        return 2;
    }

    let req = build_request(&args);
    let body = match serde_json::to_vec(&req) {
        Ok(b) => b,
        Err(e) => {
            if !args.quiet {
                eprintln!("error: serialize failed: {e}");
            }
            return 0;
        }
    };

    let endpoint = format!("{}/notify", url.trim_end_matches('/'));
    let result = ureq::post(&endpoint)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_millis(args.timeout_ms))
        .send_bytes(&body);

    match result {
        Ok(resp) if (200..300).contains(&resp.status()) => 0,
        Ok(resp) => {
            if !args.quiet {
                eprintln!("warn: server returned HTTP {}", resp.status());
            }
            0
        }
        Err(e) => {
            if !args.quiet {
                eprintln!("warn: send failed: {e}");
            }
            0
        }
    }
}

pub fn settings_path() -> std::path::PathBuf {
    // On Linux: $HOME is respected. On Windows: dirs uses SHGetKnownFolderPath
    // (ignores %USERPROFILE% env override), so we check the env var first to
    // allow integration tests to redirect the settings file via HOME/USERPROFILE.
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".claude").join("settings.json")
}

fn detect_locale() -> String {
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .map(|s| {
            if s.starts_with("ko") {
                "ko".into()
            } else {
                "en".into()
            }
        })
        .unwrap_or_else(|_| "en".into())
}

fn run_init(args: InitArgs) -> i32 {
    use agent_toast_core::hook_config::{merge_agent_toast_hooks, HookEntry};

    let path = settings_path();
    let root: serde_json::Value = match std::fs::read_to_string(&path) {
        Ok(s) if !s.trim().is_empty() => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: invalid JSON in {}: {e}", path.display());
                return 1;
            }
        },
        _ => serde_json::json!({}),
    };

    let host_flag = args
        .hostname
        .as_ref()
        .map(|h| format!(" --hostname {}", shell_escape::escape(h.into())))
        .unwrap_or_default();

    let url_esc = shell_escape::escape(args.url.clone().into());

    let locale = detect_locale();
    let (stop_msg, input_msg) = match locale.as_str() {
        "ko" => ("작업이 완료되었습니다", "권한 요청이 발생했습니다"),
        _ => ("Task completed", "Permission requested"),
    };

    let stop_cmd = format!(
        "agent-toast-send --url {} --event task_complete --message {}{}",
        url_esc,
        shell_escape::escape(stop_msg.into()),
        host_flag,
    );
    let input_cmd = format!(
        "agent-toast-send --url {} --event user_input_required --message {}{}",
        url_esc,
        shell_escape::escape(input_msg.into()),
        host_flag,
    );

    let entries = vec![
        HookEntry {
            event_key: "Stop",
            matcher: None,
            command: stop_cmd,
        },
        HookEntry {
            event_key: "Notification",
            matcher: Some("permission_prompt"),
            command: input_cmd,
        },
    ];

    let merged = merge_agent_toast_hooks(root, &entries);

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: create dir {}: {e}", parent.display());
            return 1;
        }
    }
    match std::fs::write(&path, serde_json::to_string_pretty(&merged).unwrap()) {
        Ok(()) => {
            eprintln!("registered: Stop, Notification → {}", path.display());
            0
        }
        Err(e) => {
            eprintln!("error: write {}: {e}", path.display());
            1
        }
    }
}

fn run_uninstall() -> i32 {
    use agent_toast_core::hook_config::remove_agent_toast_hooks;

    let path = settings_path();
    let Ok(s) = std::fs::read_to_string(&path) else {
        eprintln!("nothing to uninstall (no {})", path.display());
        return 0;
    };
    let root: serde_json::Value = match serde_json::from_str(&s) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: invalid JSON in {}: {e}", path.display());
            return 1;
        }
    };
    let cleaned = remove_agent_toast_hooks(root);
    match std::fs::write(&path, serde_json::to_string_pretty(&cleaned).unwrap()) {
        Ok(()) => {
            eprintln!("removed agent-toast hooks from {}", path.display());
            0
        }
        Err(e) => {
            eprintln!("error: write {}: {e}", path.display());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 환경변수를 수정하는 테스트 간의 경쟁 조건 방지용 락
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn mk_args(url: &str, event: &str) -> SendArgs {
        SendArgs {
            url: Some(url.into()),
            event: Some(event.into()),
            source: "claude".into(),
            ..SendArgs::default()
        }
    }

    #[test]
    fn payload_uses_cli_args() {
        let args = SendArgs {
            url: Some("http://x".into()),
            event: Some("task_complete".into()),
            message: Some("done".into()),
            title: Some("proj".into()),
            hostname: Some("box".into()),
            source: "claude".into(),
            timeout_ms: 2000,
            quiet: false,
        };
        let req = build_request(&args);
        assert_eq!(req.event, "task_complete");
        assert_eq!(req.message.as_deref(), Some("done"));
        assert_eq!(req.title_hint.as_deref(), Some("proj"));
        assert_eq!(req.hostname.as_deref(), Some("box"));
        assert_eq!(req.pid, 0);
        assert!(req.process_tree.is_none());
        assert_eq!(req.source, "claude");
    }

    #[test]
    fn payload_auto_detects_hostname_when_missing() {
        let args = mk_args("http://x", "test");
        let req = build_request(&args);
        assert!(req.hostname.is_some(), "auto-detect or 'unknown'");
    }

    #[test]
    fn payload_extracts_title_from_project_dir() {
        let _lock = ENV_LOCK.lock().unwrap();
        // SAFETY: 테스트 스레드 전용 락으로 보호됨
        unsafe { std::env::set_var("CLAUDE_PROJECT_DIR", "/home/user/claude-notify") };
        let args = mk_args("http://x", "test");
        let req = build_request(&args);
        assert_eq!(req.title_hint.as_deref(), Some("claude-notify"));
        unsafe { std::env::remove_var("CLAUDE_PROJECT_DIR") };
    }

    #[test]
    fn payload_without_project_dir_has_no_title() {
        let _lock = ENV_LOCK.lock().unwrap();
        // SAFETY: 테스트 스레드 전용 락으로 보호됨
        unsafe { std::env::remove_var("CLAUDE_PROJECT_DIR") };
        let args = mk_args("http://x", "test");
        let req = build_request(&args);
        // Either None or from explicit --title, but here --title wasn't set
        assert!(req.title_hint.is_none() || req.title_hint.as_deref() == Some(""));
    }
}
