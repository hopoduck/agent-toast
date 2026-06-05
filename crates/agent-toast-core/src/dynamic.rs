//! Dynamic notification message extraction from Claude Code hook stdin JSON.
//!
//! When a hook command carries `--dynamic`, the desktop CLI reads the hook's
//! stdin JSON and derives the toast body from it, in priority order:
//!   1. the tool action description (`tool_input.description`) — present on
//!      permission-style hooks for tools that carry one (e.g. Bash),
//!   2. the assistant's final message (`last_assistant_message`) — delivered
//!      directly on stdin for Stop and similar turn-end hooks,
//!   3. otherwise the caller falls back to the static `--message`.
//!
//! `last_assistant_message` is the just-finished turn (no transcript-flush
//! timing lag), so we never need to read `transcript_path`. These functions are
//! pure so they can be unit-tested.

use serde_json::Value;

/// Max characters shown in a notification body. Longer text is truncated here;
/// the frontend applies the final visual ellipsis via CSS `line-clamp`.
pub const MAX_MESSAGE_CHARS: usize = 200;

/// Truncate to at most `max` characters (NOT bytes).
///
/// Slicing by byte index (`&s[..max]`) panics when the boundary falls inside a
/// multi-byte UTF-8 sequence — common with Korean text and emoji. Counting
/// `chars()` avoids that. Surrounding whitespace is trimmed first.
pub fn truncate_chars(s: &str, max: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max {
        trimmed.to_string()
    } else {
        trimmed.chars().take(max).collect()
    }
}

/// Extract a human-readable tool action description from a hook payload's
/// `tool_input`. Claude Code's Bash tool carries a `description`; most other
/// tools (Edit/Write/Read/…) don't, so this returns `None` for them.
pub fn extract_tool_description(hook: &Value) -> Option<String> {
    let desc = hook
        .get("tool_input")
        .and_then(|ti| ti.get("description"))
        .and_then(|d| d.as_str())?
        .trim();
    if desc.is_empty() {
        None
    } else {
        Some(desc.to_string())
    }
}

/// Extract the assistant's final message, delivered directly on the hook's
/// stdin JSON as `last_assistant_message` (present on Stop and similar turn-end
/// hooks). This is the just-finished turn — unlike reading `transcript_path`,
/// there is no flush-timing lag, so the toast shows the actual last message.
pub fn extract_last_assistant_message(hook: &Value) -> Option<String> {
    let msg = hook
        .get("last_assistant_message")
        .and_then(|m| m.as_str())?
        .trim();
    if msg.is_empty() {
        None
    } else {
        Some(msg.to_string())
    }
}

/// Resolve the dynamic notification body from a parsed hook payload, in
/// priority order: tool action description → assistant's last message →
/// static `fallback`. The dynamic candidates are truncated to
/// [`MAX_MESSAGE_CHARS`]; the fallback is passed through as-is (it's the user's
/// own short fixed text).
pub fn resolve_message(hook: &Value, fallback: Option<&str>) -> Option<String> {
    extract_tool_description(hook)
        .or_else(|| extract_last_assistant_message(hook))
        .map(|s| truncate_chars(&s, MAX_MESSAGE_CHARS))
        .or_else(|| fallback.map(str::to_string))
}

/// Read the hook event JSON from stdin and [`resolve_message`] it. Shared by the
/// `--dynamic` path of both the desktop and remote (`agent-toast-send`) CLIs.
/// Any failure (no stdin, empty, bad JSON, no usable field) falls back to
/// `fallback` so the toast is never empty.
pub fn resolve_from_stdin(fallback: Option<&str>) -> Option<String> {
    use std::io::{IsTerminal, Read};
    // Manual terminal invocation: no hook JSON will ever arrive on stdin, and
    // read_to_string would block forever waiting for EOF — bail out to the
    // fallback immediately. Hook invocations pipe stdin, so they skip this.
    if std::io::stdin().is_terminal() {
        return fallback.map(str::to_string);
    }
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() || buf.trim().is_empty() {
        return fallback.map(str::to_string);
    }
    match serde_json::from_str::<Value>(&buf) {
        Ok(hook) => resolve_message(&hook, fallback),
        Err(_) => fallback.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_chars("hello", 200), "hello");
    }

    #[test]
    fn truncate_trims_whitespace() {
        assert_eq!(truncate_chars("  hi \n", 200), "hi");
    }

    #[test]
    fn truncate_counts_chars_not_bytes_no_panic() {
        // 300 Korean chars = 900 bytes. Byte-slicing at 200 would panic.
        let s = "가".repeat(300);
        let out = truncate_chars(&s, 200);
        assert_eq!(out.chars().count(), 200);
    }

    #[test]
    fn truncate_emoji_boundary_safe() {
        let s = "🎉".repeat(50);
        let out = truncate_chars(&s, 10);
        assert_eq!(out.chars().count(), 10);
    }

    #[test]
    fn description_extracted_when_present() {
        let hook = json!({
            "tool_name": "Bash",
            "tool_input": { "command": "ls", "description": "List files" }
        });
        assert_eq!(extract_tool_description(&hook), Some("List files".into()));
    }

    #[test]
    fn description_none_when_absent() {
        // Edit-style tool: no description field.
        let hook = json!({
            "tool_name": "Edit",
            "tool_input": { "file_path": "a.rs", "old_string": "x", "new_string": "y" }
        });
        assert_eq!(extract_tool_description(&hook), None);
    }

    #[test]
    fn description_none_when_empty() {
        let hook = json!({ "tool_input": { "description": "   " } });
        assert_eq!(extract_tool_description(&hook), None);
    }

    #[test]
    fn last_message_extracted_when_present() {
        let hook = json!({
            "hook_event_name": "Stop",
            "last_assistant_message": "Build completed!"
        });
        assert_eq!(
            extract_last_assistant_message(&hook),
            Some("Build completed!".into())
        );
    }

    #[test]
    fn last_message_none_when_absent() {
        let hook = json!({ "hook_event_name": "Stop" });
        assert_eq!(extract_last_assistant_message(&hook), None);
    }

    #[test]
    fn last_message_none_when_empty() {
        let hook = json!({ "last_assistant_message": "   " });
        assert_eq!(extract_last_assistant_message(&hook), None);
    }

    #[test]
    fn resolve_prefers_description_over_last_message() {
        let hook = json!({
            "tool_input": { "description": "List files" },
            "last_assistant_message": "some long reply"
        });
        assert_eq!(
            resolve_message(&hook, Some("fixed")),
            Some("List files".into())
        );
    }

    #[test]
    fn resolve_uses_last_message_when_no_description() {
        let hook = json!({ "last_assistant_message": "Done!" });
        assert_eq!(resolve_message(&hook, Some("fixed")), Some("Done!".into()));
    }

    #[test]
    fn resolve_falls_back_to_static() {
        let hook = json!({ "hook_event_name": "Stop" });
        assert_eq!(resolve_message(&hook, Some("fixed")), Some("fixed".into()));
    }

    #[test]
    fn resolve_none_when_nothing_available() {
        assert_eq!(resolve_message(&json!({}), None), None);
    }

    #[test]
    fn resolve_truncates_dynamic_candidate() {
        let hook = json!({ "last_assistant_message": "가".repeat(300) });
        let out = resolve_message(&hook, None).unwrap();
        assert_eq!(out.chars().count(), 200);
    }
}
