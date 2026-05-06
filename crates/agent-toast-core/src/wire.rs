use serde::{Deserialize, Serialize};

/// Diagnostic version string reflecting the `agent-toast-core` crate version.
///
/// Surfaced by `agent-toast-send --version` so users can check whether the
/// sender and desktop share compatible build versions when debugging
/// cross-host notification issues.
///
/// This is NOT a stable wire-schema version identifier. The wire format
/// evolves via `#[serde(default)]` on new fields for forward/backward
/// compatibility, without bumping this constant's meaning.
pub const WIRE_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    /// Sender host identifier for remote notifications.
    ///
    /// - `None`: delivered via local named pipe (Windows-only path).
    /// - `Some(label)`: delivered via HTTP from a remote host. `label` is
    ///   either the result of `hostname::get()` or a user-provided override.
    /// - Used by the desktop UI to distinguish remote from local notifications
    ///   and render a host badge.
    #[serde(default)]
    pub hostname: Option<String>,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
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
            hostname: None,
        };
        let cloned = req.clone();
        assert_eq!(cloned.pid, req.pid);
        assert_eq!(cloned.event, req.event);
        assert_eq!(cloned.message, req.message);
        assert_eq!(cloned.title_hint, req.title_hint);
        assert_eq!(cloned.process_tree, req.process_tree);
        assert_eq!(cloned.source, req.source);
    }

    #[test]
    fn hostname_field_default_is_none() {
        let json = r#"{"pid":0,"event":"task_complete","source":"claude"}"#;
        let req: NotifyRequest = serde_json::from_str(json).unwrap();
        assert!(req.hostname.is_none());
    }

    #[test]
    fn hostname_field_roundtrip() {
        let req = NotifyRequest {
            pid: 0,
            event: "task_complete".into(),
            message: Some("done".into()),
            title_hint: Some("proj".into()),
            process_tree: None,
            source: "claude".into(),
            hostname: Some("prod-vps-01".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""hostname":"prod-vps-01""#));
        let decoded: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.hostname.as_deref(), Some("prod-vps-01"));
    }

    #[test]
    fn hostname_field_unicode() {
        let req = NotifyRequest {
            pid: 0,
            event: "task_complete".into(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
            hostname: Some("회사-서버-01".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.hostname.as_deref(), Some("회사-서버-01"));
    }

    #[test]
    fn hostname_field_empty_string_is_some_empty() {
        let req = NotifyRequest {
            pid: 0,
            event: "task_complete".into(),
            message: None,
            title_hint: None,
            process_tree: None,
            source: "claude".into(),
            hostname: Some(String::new()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: NotifyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.hostname.as_deref(), Some(""));
    }
}
