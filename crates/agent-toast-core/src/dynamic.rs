//! Dynamic notification message extraction from Claude Code / Codex hook stdin JSON.
//!
//! When a hook command carries `--dynamic`, the desktop CLI reads the hook's
//! stdin JSON and derives the toast body from it, in priority order:
//!   1. the tool action description (`tool_input.description`) — present on
//!      permission-style hooks for tools that carry one (e.g. Bash),
//!   2. the question text (`tool_input.questions[].question`) — Claude's
//!      `AskUserQuestion` (via PermissionRequest) and Codex's
//!      `request_user_input` (via PreToolUse) both carry it; Codex's
//!      `request_user_input_async` uses `questions[].title` instead,
//!   3. the assistant's final message (`last_assistant_message`) — delivered
//!      directly on stdin for Stop and similar turn-end hooks,
//!   4. otherwise the caller falls back to the static `--message`.
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

/// Extract the question text from a question tool's `tool_input.questions`.
/// Several questions in one call are joined line by line so none is dropped.
pub fn extract_questions(hook: &Value) -> Option<String> {
    let questions: Vec<&str> = hook
        .get("tool_input")
        .and_then(|ti| ti.get("questions"))
        .and_then(|q| q.as_array())?
        .iter()
        .filter_map(|q| {
            ["question", "title"].into_iter().find_map(|field| {
                q.get(field)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
            })
        })
        .collect();
    if questions.is_empty() {
        None
    } else {
        Some(questions.join("\n"))
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
/// priority order: tool action description → question text → assistant's last
/// message → static `fallback`. The dynamic candidates are truncated to
/// [`MAX_MESSAGE_CHARS`]; the fallback is passed through as-is (it's the user's
/// own short fixed text).
pub fn resolve_message(hook: &Value, fallback: Option<&str>) -> Option<String> {
    extract_tool_description(hook)
        .or_else(|| extract_questions(hook))
        .or_else(|| extract_last_assistant_message(hook))
        .map(|s| truncate_chars(&s, MAX_MESSAGE_CHARS))
        .or_else(|| fallback.map(str::to_string))
}

/// Agent session id from a hook payload. Claude Code and Codex both send
/// `session_id` on every hook event.
pub fn extract_session_id(hook: &Value) -> Option<String> {
    hook["session_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// How long [`read_stdin_json`] waits for EOF. Hook runners write the JSON and
/// close stdin right away; only a caller that leaves an inherited pipe open
/// (a script, a test harness) ever reaches this.
const STDIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// Read the hook event JSON from stdin. `None` when stdin is a terminal, empty,
/// unreadable, not JSON, or still open after [`STDIN_TIMEOUT`].
pub fn read_stdin_json() -> Option<Value> {
    use std::io::{IsTerminal, Read};
    // Manual terminal invocation: no hook JSON will ever arrive on stdin, and
    // read_to_string would block forever waiting for EOF — bail out
    // immediately. Hook invocations pipe stdin, so they skip this.
    if std::io::stdin().is_terminal() {
        return None;
    }
    // stdin is read on every invocation, not only with --dynamic, so a pipe
    // nobody closes must not hang the CLI. The reader thread is left behind on
    // timeout; the process exits shortly after anyway.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = String::new();
        let read = std::io::stdin().read_to_string(&mut buf).map(|_| buf);
        let _ = tx.send(read.ok());
    });
    let buf = rx.recv_timeout(STDIN_TIMEOUT).ok().flatten()?;
    if buf.trim().is_empty() {
        return None;
    }
    serde_json::from_str(&buf).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn session_id_extracted_when_present() {
        let hook = json!({ "session_id": "abc-123", "hook_event_name": "Stop" });
        assert_eq!(extract_session_id(&hook).as_deref(), Some("abc-123"));
    }

    #[test]
    fn session_id_none_when_absent_or_empty() {
        assert!(extract_session_id(&json!({ "hook_event_name": "Stop" })).is_none());
        assert!(extract_session_id(&json!({ "session_id": "" })).is_none());
        assert!(extract_session_id(&json!({ "session_id": 42 })).is_none());
    }

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
    fn questions_extracted_from_codex_request_user_input() {
        let hook = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "request_user_input",
            "tool_input": { "questions": [{
                "id": "continue_test",
                "header": "훅 필터 테스트",
                "question": "계속할까요?",
                "options": [{ "label": "계속" }, { "label": "중단" }]
            }]}
        });
        assert_eq!(extract_questions(&hook), Some("계속할까요?".into()));
    }

    #[test]
    fn resolve_uses_async_question_titles() {
        let hook = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "request_user_input_async",
            "tool_input": { "questions": [
                { "title": "  어느 것을 선택할까요?  ", "options": ["A", "B"] },
                { "title": " " },
                { "title": "추가 조건이 있나요?" }
            ]}
        });
        assert_eq!(
            resolve_message(&hook, Some("질문")),
            Some("어느 것을 선택할까요?\n추가 조건이 있나요?".into())
        );
    }

    #[test]
    fn questions_joined_line_by_line() {
        let hook = json!({ "tool_input": { "questions": [
            { "question": "First?" },
            { "question": "  " },
            { "question": "Second?" }
        ]}});
        assert_eq!(extract_questions(&hook), Some("First?\nSecond?".into()));
    }

    #[test]
    fn questions_none_when_absent_or_empty() {
        assert_eq!(extract_questions(&json!({ "tool_input": {} })), None);
        let hook = json!({ "tool_input": { "questions": [{ "question": "" }] } });
        assert_eq!(extract_questions(&hook), None);
    }

    #[test]
    fn resolve_uses_question_before_last_message() {
        let hook = json!({
            "tool_input": { "questions": [{ "question": "Apply now?" }] },
            "last_assistant_message": "some reply"
        });
        assert_eq!(
            resolve_message(&hook, Some("fixed")),
            Some("Apply now?".into())
        );
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
