use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "agent-toast", about = "Smart notification for AI coding agents")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRequest {
    pub pid: u32,
    pub event: String,
    pub message: Option<String>,
    pub title_hint: Option<String>,
    /// Pre-resolved process tree from CLI side (avoids race with dead process)
    #[serde(default)]
    pub process_tree: Option<Vec<u32>>,
    /// Source of the notification: "claude" or "codex"
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "claude".into()
}

impl NotifyRequest {
    pub fn event_display(&self) -> &str {
        // Return the event key as-is; frontend translates via i18n
        &self.event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(event: &str) -> NotifyRequest {
        NotifyRequest {
            pid: 1234,
            event: event.to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        }
    }

    #[test]
    fn event_display_returns_key_as_is() {
        assert_eq!(
            make_request("task_complete").event_display(),
            "task_complete"
        );
        assert_eq!(
            make_request("user_input_required").event_display(),
            "user_input_required"
        );
        assert_eq!(make_request("error").event_display(), "error");
        assert_eq!(make_request("custom_event").event_display(), "custom_event");
    }

    #[test]
    fn notify_request_serde_roundtrip() {
        let req = NotifyRequest {
            pid: 42,
            event: "task_complete".to_string(),
            message: Some("빌드 완료".to_string()),
            title_hint: Some("my-project".to_string()),
            process_tree: Some(vec![100, 200, 300]),
            source: "claude".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, 42);
        assert_eq!(deserialized.event, "task_complete");
        assert_eq!(deserialized.message.as_deref(), Some("빌드 완료"));
        assert_eq!(deserialized.title_hint.as_deref(), Some("my-project"));
        assert_eq!(deserialized.process_tree, Some(vec![100, 200, 300]));
    }

    #[test]
    fn notify_request_deserialize_without_optional_fields() {
        let json = r#"{"pid":1,"event":"error"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.pid, 1);
        assert_eq!(req.event, "error");
        assert!(req.message.is_none());
        assert!(req.title_hint.is_none());
        assert!(req.process_tree.is_none());
    }

    #[test]
    fn notify_request_default_source() {
        let json = r#"{"pid":1,"event":"test"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "claude");
    }

    #[test]
    fn notify_request_custom_source() {
        let json = r#"{"pid":1,"event":"test","source":"codex"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "codex");
    }

    #[test]
    fn notify_request_updater_source() {
        let json = r#"{"pid":0,"event":"update_available","source":"updater"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "updater");
        assert_eq!(req.pid, 0);
    }

    #[test]
    fn notify_request_empty_process_tree() {
        let json = r#"{"pid":1,"event":"test","process_tree":[]}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.process_tree, Some(vec![]));
    }

    #[test]
    fn notify_request_large_process_tree() {
        let tree: Vec<u32> = (1..=100).collect();
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: Some(tree.clone()),
            source: "claude".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.process_tree.unwrap().len(), 100);
    }

    #[test]
    fn notify_request_unicode_message() {
        let req = NotifyRequest {
            pid: 1,
            event: "task_complete".to_string(),
            message: Some("한글 메시지 🎉 日本語 العربية".to_string()),
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.message.as_deref(),
            Some("한글 메시지 🎉 日本語 العربية")
        );
    }

    #[test]
    fn notify_request_unicode_title_hint() {
        let req = NotifyRequest {
            pid: 1,
            event: "test".to_string(),
            message: None,
            title_hint: Some("프로젝트-이름".to_string()),
            process_tree: None,
            source: "claude".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title_hint.as_deref(), Some("프로젝트-이름"));
    }

    #[test]
    fn notify_request_max_pid() {
        let req = NotifyRequest {
            pid: u32::MAX,
            event: "test".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, u32::MAX);
    }

    #[test]
    fn notify_request_zero_pid() {
        let req = NotifyRequest {
            pid: 0,
            event: "internal".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "updater".into(),
        };
        assert_eq!(req.pid, 0);
    }

    #[test]
    fn notify_request_empty_event() {
        let req = NotifyRequest {
            pid: 1,
            event: "".to_string(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
        };
        assert_eq!(req.event, "");
        assert_eq!(req.event_display(), "");
    }

    #[test]
    fn notify_request_all_event_types() {
        let events = [
            "task_complete",
            "user_input_required",
            "error",
            "session_start",
            "session_end",
            "subagent_start",
            "subagent_stop",
            "update_available",
        ];
        for event in events {
            let req = make_request(event);
            assert_eq!(req.event_display(), event);
        }
    }

    #[test]
    fn notify_request_clone() {
        let req = NotifyRequest {
            pid: 42,
            event: "test".to_string(),
            message: Some("message".to_string()),
            title_hint: Some("hint".to_string()),
            process_tree: Some(vec![1, 2, 3]),
            source: "claude".into(),
        };
        let cloned = req.clone();
        assert_eq!(cloned.pid, req.pid);
        assert_eq!(cloned.event, req.event);
        assert_eq!(cloned.message, req.message);
        assert_eq!(cloned.title_hint, req.title_hint);
        assert_eq!(cloned.process_tree, req.process_tree);
        assert_eq!(cloned.source, req.source);
    }

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
        let cli =
            Cli::try_parse_from(["agent-toast", "--codex", r#"{"type":"test"}"#]).unwrap();
        assert!(cli.codex);
        assert_eq!(cli.codex_json.as_deref(), Some(r#"{"type":"test"}"#));
    }

    #[test]
    fn cli_parse_pid_and_event() {
        let cli = Cli::try_parse_from([
            "agent-toast",
            "--pid",
            "1234",
            "--event",
            "task_complete",
        ])
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
        let cli = Cli::try_parse_from([
            "agent-toast",
            "--message",
            "빌드 완료 🎉",
        ])
        .unwrap();
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
        let cli =
            Cli::try_parse_from(["agent-toast", "--pid", &u32::MAX.to_string()]).unwrap();
        assert_eq!(cli.pid, Some(u32::MAX));
    }

    #[test]
    fn cli_parse_zero_pid() {
        let cli = Cli::try_parse_from(["agent-toast", "--pid", "0"]).unwrap();
        assert_eq!(cli.pid, Some(0));
    }
}
