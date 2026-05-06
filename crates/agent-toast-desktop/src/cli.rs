use clap::Parser;

pub use agent_toast_core::NotifyRequest;

#[derive(Parser, Debug)]
#[command(
    name = "agent-toast",
    about = "Smart notification for AI coding agents"
)]
pub struct Cli {
    /// PID of the source terminal/editor (defaults to current process)
    #[arg(long)]
    pub pid: Option<u32>,

    /// Event type
    #[arg(long)]
    pub event: Option<String>,

    /// Message text
    #[arg(long)]
    pub message: Option<String>,

    /// Window title hint for matching the correct source window
    #[arg(long)]
    pub title: Option<String>,

    /// Start as background daemon (no notification)
    #[arg(long)]
    pub daemon: bool,

    /// Internal: actually run the daemon process (spawned by --daemon)
    #[arg(long, hide = true)]
    pub daemon_run: bool,

    /// Open hook setup GUI
    #[arg(long)]
    pub setup: bool,

    /// Codex mode: receive JSON from Codex CLI notify hook
    #[arg(long)]
    pub codex: bool,

    /// Positional argument for Codex JSON payload
    #[arg(index = 1)]
    pub codex_json: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Cli parsing tests ──

    #[test]
    fn cli_parse_daemon_flag() {
        let cli = Cli::try_parse_from(["agent-toast", "--daemon"]).unwrap();
        assert!(cli.daemon);
        assert!(!cli.setup);
        assert!(!cli.codex);
        assert!(cli.pid.is_none());
        assert!(cli.event.is_none());
    }

    #[test]
    fn cli_parse_daemon_run_flag() {
        let cli = Cli::try_parse_from(["agent-toast", "--daemon-run"]).unwrap();
        assert!(cli.daemon_run);
        assert!(!cli.daemon);
        assert!(!cli.setup);
        assert!(!cli.codex);
    }

    #[test]
    fn cli_parse_daemon_and_daemon_run_independent() {
        // --daemon과 --daemon-run은 별개 플래그
        let cli = Cli::try_parse_from(["agent-toast", "--daemon"]).unwrap();
        assert!(cli.daemon);
        assert!(!cli.daemon_run);

        let cli = Cli::try_parse_from(["agent-toast", "--daemon-run"]).unwrap();
        assert!(!cli.daemon);
        assert!(cli.daemon_run);
    }

    #[test]
    fn cli_daemon_run_not_in_help() {
        // --daemon-run은 숨김 플래그이므로 help에 표시되지 않아야 함
        let mut cmd = <Cli as clap::CommandFactory>::command();
        let mut buf = Vec::new();
        cmd.write_help(&mut buf).unwrap();
        let help = String::from_utf8(buf).unwrap();
        assert!(help.contains("--daemon"));
        assert!(!help.contains("--daemon-run"));
    }

    #[test]
    fn cli_parse_no_args_daemon_run_false() {
        let cli = Cli::try_parse_from(["agent-toast"]).unwrap();
        assert!(!cli.daemon_run);
    }

    #[test]
    fn cli_parse_setup_flag() {
        let cli = Cli::try_parse_from(["agent-toast", "--setup"]).unwrap();
        assert!(cli.setup);
        assert!(!cli.daemon);
    }

    #[test]
    fn cli_parse_codex_flag() {
        let cli = Cli::try_parse_from(["agent-toast", "--codex"]).unwrap();
        assert!(cli.codex);
    }

    #[test]
    fn cli_parse_codex_with_json_payload() {
        let cli = Cli::try_parse_from(["agent-toast", "--codex", r#"{"type":"test"}"#]).unwrap();
        assert!(cli.codex);
        assert_eq!(cli.codex_json.as_deref(), Some(r#"{"type":"test"}"#));
    }

    #[test]
    fn cli_parse_pid_and_event() {
        let cli = Cli::try_parse_from(["agent-toast", "--pid", "1234", "--event", "task_complete"])
            .unwrap();
        assert_eq!(cli.pid, Some(1234));
        assert_eq!(cli.event.as_deref(), Some("task_complete"));
    }

    #[test]
    fn cli_parse_full_notification() {
        let cli = Cli::try_parse_from([
            "agent-toast",
            "--pid",
            "5678",
            "--event",
            "user_input_required",
            "--message",
            "입력이 필요합니다",
            "--title",
            "my-project",
        ])
        .unwrap();
        assert_eq!(cli.pid, Some(5678));
        assert_eq!(cli.event.as_deref(), Some("user_input_required"));
        assert_eq!(cli.message.as_deref(), Some("입력이 필요합니다"));
        assert_eq!(cli.title.as_deref(), Some("my-project"));
    }

    #[test]
    fn cli_parse_no_args() {
        let cli = Cli::try_parse_from(["agent-toast"]).unwrap();
        assert!(!cli.daemon);
        assert!(!cli.setup);
        assert!(!cli.codex);
        assert!(cli.pid.is_none());
        assert!(cli.event.is_none());
        assert!(cli.message.is_none());
        assert!(cli.title.is_none());
        assert!(cli.codex_json.is_none());
    }

    #[test]
    fn cli_parse_invalid_pid_fails() {
        let result = Cli::try_parse_from(["agent-toast", "--pid", "not-a-number"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_parse_message_with_spaces() {
        let cli = Cli::try_parse_from([
            "agent-toast",
            "--event",
            "task_complete",
            "--message",
            "Build completed successfully",
        ])
        .unwrap();
        assert_eq!(cli.message.as_deref(), Some("Build completed successfully"));
    }

    #[test]
    fn cli_parse_message_with_unicode() {
        let cli = Cli::try_parse_from(["agent-toast", "--message", "빌드 완료 🎉"]).unwrap();
        assert_eq!(cli.message.as_deref(), Some("빌드 완료 🎉"));
    }

    #[test]
    fn cli_parse_multiple_flags() {
        let cli = Cli::try_parse_from(["agent-toast", "--daemon", "--setup"]).unwrap();
        assert!(cli.daemon);
        assert!(cli.setup);
    }

    #[test]
    fn cli_parse_max_pid() {
        let cli = Cli::try_parse_from(["agent-toast", "--pid", &u32::MAX.to_string()]).unwrap();
        assert_eq!(cli.pid, Some(u32::MAX));
    }

    #[test]
    fn cli_parse_zero_pid() {
        let cli = Cli::try_parse_from(["agent-toast", "--pid", "0"]).unwrap();
        assert_eq!(cli.pid, Some(0));
    }
}
