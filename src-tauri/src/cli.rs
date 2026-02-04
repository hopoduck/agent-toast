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
}
