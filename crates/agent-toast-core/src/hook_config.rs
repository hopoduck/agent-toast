//! Pure JSON merge logic for Claude Code's `~/.claude/settings.json`.
//!
//! Both the desktop GUI (`setup.rs`) and the remote CLI (`agent-toast-send init`)
//! use this to add/remove agent-toast hook entries while preserving any unrelated
//! hooks the user has configured.

use serde_json::{json, Value};

/// A hook entry to register into `settings.json`.
///
/// `event_key` is the Claude Code event name ("Stop", "Notification",
/// "SessionStart", "SessionEnd", "SubagentStop", "PreCompact", "UserPromptSubmit",
/// "PreToolUse", "PostToolUse").
///
/// `matcher` applies only to events that support matchers (e.g. `Notification`
/// uses `"permission_prompt"` / `"elicitation_prompt"` / `"idle"`). Pass `None`
/// for events that don't take a matcher.
pub struct HookEntry {
    pub event_key: &'static str,
    pub matcher: Option<&'static str>,
    pub command: String,
}

/// Returns true if the command string belongs to agent-toast.
///
/// Matches the Windows local binary (`agent-toast.exe`), the Windows installer
/// product name (`Agent Toast`), the remote CLI (`agent-toast-send`), and the
/// underscore variant (`agent_toast`), all case-insensitively.
pub fn is_agent_toast_cmd(command: &str) -> bool {
    let lc = command.to_lowercase();
    lc.contains("agent-toast") || lc.contains("agent_toast") || lc.contains("agent toast")
}

/// Merge `entries` into `root`, replacing any existing agent-toast hooks and
/// preserving everything else the user has configured.
///
/// Semantics: "replace all agent-toast hooks with this set". Any agent-toast
/// entries on event_keys not present in `entries` are removed, so the result
/// exactly matches `entries` with all non-agent-toast hooks preserved.
///
/// `root` is the parsed `settings.json` as a `serde_json::Value`. The returned
/// `Value` is ready to be serialized back out.
pub fn merge_agent_toast_hooks(mut root: Value, entries: &[HookEntry]) -> Value {
    // First pass: clear ALL agent-toast hooks across every event, so stale
    // entries on events no longer in `entries` are removed too.
    root = remove_agent_toast_hooks(root);

    // Ensure root.hooks is an object (remove_agent_toast_hooks returns early
    // if the key is missing).
    if !root.get("hooks").is_some_and(|v| v.is_object()) {
        root["hooks"] = json!({});
    }

    // Group new entries by event_key.
    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<&'static str, Vec<&HookEntry>> = BTreeMap::new();
    for e in entries {
        grouped.entry(e.event_key).or_default().push(e);
    }

    for (event_key, event_entries) in grouped {
        // `root.hooks.<event>` may be absent; start from whatever remains
        // after the remove pass (all non-agent-toast entries).
        let existing = root["hooks"][event_key]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut combined = existing;

        // Append new agent-toast entries.
        for e in event_entries {
            combined.push(build_outer_entry(e));
        }

        root["hooks"][event_key] = Value::Array(combined);
    }

    root
}

/// Remove all agent-toast entries from `root.hooks.*`, preserving other hooks.
pub fn remove_agent_toast_hooks(mut root: Value) -> Value {
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return root;
    };
    for (_event_key, arr_val) in hooks.iter_mut() {
        if let Some(arr) = arr_val.as_array_mut() {
            arr.retain(|outer| !outer_contains_agent_toast_cmd(outer));
        }
    }
    root
}

// ────────── internal helpers ──────────

/// Claude Code settings.json nests hooks as:
///   "Stop": [ { "hooks": [ {"type":"command","command":"..."} ] } ]
/// or with a matcher:
///   "Notification": [ { "matcher": "permission_prompt", "hooks": [ {...} ] } ]
/// This helper checks if any of the inner command entries are agent-toast's.
fn outer_contains_agent_toast_cmd(outer: &Value) -> bool {
    outer
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|cmd_entry| {
                cmd_entry
                    .get("command")
                    .and_then(|c| c.as_str())
                    .is_some_and(is_agent_toast_cmd)
            })
        })
        .unwrap_or(false)
}

/// Builds the outer entry Claude Code expects, with or without a matcher.
fn build_outer_entry(e: &HookEntry) -> Value {
    let inner = json!({
        "type": "command",
        "command": e.command,
    });
    match e.matcher {
        Some(m) => json!({
            "matcher": m,
            "hooks": [inner],
        }),
        None => json!({
            "hooks": [inner],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_agent_toast_cmd_detects_variants() {
        assert!(is_agent_toast_cmd(
            "/usr/local/bin/agent-toast-send --url http://x --event test"
        ));
        assert!(is_agent_toast_cmd(
            r#""C:\tools\agent-toast.exe" --pid 1 --event task_complete"#
        ));
        assert!(is_agent_toast_cmd("AGENT-TOAST --event x"));
        assert!(is_agent_toast_cmd("my_agent_toast_wrapper"));
        assert!(!is_agent_toast_cmd("/usr/bin/other-tool --flag"));
        assert!(!is_agent_toast_cmd(""));
    }

    #[test]
    fn is_agent_toast_cmd_detects_product_name_with_space() {
        // Windows installer productName: "Agent Toast"
        assert!(is_agent_toast_cmd(
            "C:\\Users\\test\\Agent Toast\\Agent Toast.exe --daemon"
        ));
    }

    #[test]
    fn merge_into_empty_root_creates_hooks_object() {
        let root = json!({});
        let entries = vec![HookEntry {
            event_key: "Stop",
            matcher: None,
            command: "agent-toast-send --url http://x --event task_complete".to_string(),
        }];
        let merged = merge_agent_toast_hooks(root, &entries);
        let cmd = &merged["hooks"]["Stop"][0]["hooks"][0]["command"];
        assert_eq!(
            cmd.as_str().unwrap(),
            "agent-toast-send --url http://x --event task_complete"
        );
    }

    #[test]
    fn merge_preserves_unrelated_hooks() {
        let root = json!({
            "hooks": {
                "Stop": [
                    {"hooks":[{"type":"command","command":"/usr/bin/custom-tool --arg"}]}
                ],
                "SessionStart": [
                    {"hooks":[{"type":"command","command":"other-tool"}]}
                ]
            }
        });
        let entries = vec![HookEntry {
            event_key: "Stop",
            matcher: None,
            command: "agent-toast-send --url http://x --event task_complete".to_string(),
        }];
        let merged = merge_agent_toast_hooks(root, &entries);

        // Unrelated Stop hook preserved
        let stop_arr = merged["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop_arr.len(), 2);
        assert!(stop_arr
            .iter()
            .any(|e| e["hooks"][0]["command"].as_str() == Some("/usr/bin/custom-tool --arg")));
        // agent-toast added
        assert!(stop_arr.iter().any(|e| e["hooks"][0]["command"]
            .as_str()
            .is_some_and(|s| s.contains("agent-toast-send"))));
        // SessionStart untouched
        let sess = merged["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(sess.len(), 1);
        assert_eq!(sess[0]["hooks"][0]["command"].as_str(), Some("other-tool"));
    }

    #[test]
    fn merge_replaces_existing_agent_toast_entries() {
        let root = json!({
            "hooks": {
                "Stop": [
                    {"hooks":[{"type":"command","command":"/old/path/agent-toast.exe --event old"}]},
                    {"hooks":[{"type":"command","command":"/usr/bin/custom"}]}
                ]
            }
        });
        let entries = vec![HookEntry {
            event_key: "Stop",
            matcher: None,
            command: "agent-toast-send --url http://x --event task_complete".to_string(),
        }];
        let merged = merge_agent_toast_hooks(root, &entries);
        let stop_arr = merged["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(
            stop_arr.len(),
            2,
            "old agent-toast replaced, custom preserved, new added"
        );
        assert!(stop_arr
            .iter()
            .any(|e| e["hooks"][0]["command"].as_str() == Some("/usr/bin/custom")));
        assert!(stop_arr.iter().all(|e| !e["hooks"][0]["command"]
            .as_str()
            .is_some_and(|s| s.contains("/old/path"))));
        assert!(
            stop_arr.iter().any(|e| e["hooks"][0]["command"]
                .as_str()
                .is_some_and(|s| s.contains("agent-toast-send --url http://x"))),
            "The new agent-toast entry must be present after replacement"
        );
    }

    #[test]
    fn merge_with_matcher_entry() {
        let root = json!({});
        let entries = vec![HookEntry {
            event_key: "Notification",
            matcher: Some("permission_prompt"),
            command: "agent-toast-send --url http://x --event user_input_required".to_string(),
        }];
        let merged = merge_agent_toast_hooks(root, &entries);
        assert_eq!(
            merged["hooks"]["Notification"][0]["matcher"].as_str(),
            Some("permission_prompt")
        );
    }

    #[test]
    fn remove_clears_only_agent_toast() {
        let root = json!({
            "hooks": {
                "Stop": [
                    {"hooks":[{"type":"command","command":"agent-toast-send --event x"}]},
                    {"hooks":[{"type":"command","command":"/usr/bin/keep-me"}]}
                ],
                "SessionStart": [
                    {"hooks":[{"type":"command","command":"AGENT-TOAST"}]}
                ]
            }
        });
        let cleaned = remove_agent_toast_hooks(root);
        let stop_arr = cleaned["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop_arr.len(), 1);
        assert_eq!(
            stop_arr[0]["hooks"][0]["command"].as_str(),
            Some("/usr/bin/keep-me")
        );
        let sess = cleaned["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(sess.len(), 0);
    }

    #[test]
    fn remove_handles_missing_hooks_gracefully() {
        let root = json!({});
        let cleaned = remove_agent_toast_hooks(root);
        // Should not panic, should stay {}
        assert!(cleaned.get("hooks").is_none() || cleaned["hooks"].is_object());
    }

    #[test]
    fn merge_with_multiple_events_and_matchers() {
        let root = json!({});
        let entries = vec![
            HookEntry {
                event_key: "Stop",
                matcher: None,
                command: "agent-toast-send --event task_complete".into(),
            },
            HookEntry {
                event_key: "Notification",
                matcher: Some("permission_prompt"),
                command: "agent-toast-send --event user_input_required".into(),
            },
        ];
        let merged = merge_agent_toast_hooks(root, &entries);
        assert_eq!(merged["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(merged["hooks"]["Notification"].as_array().unwrap().len(), 1);
        assert_eq!(
            merged["hooks"]["Notification"][0]["matcher"].as_str(),
            Some("permission_prompt")
        );
    }

    #[test]
    fn merge_removes_agent_toast_from_events_not_in_new_entries() {
        let root = json!({
            "hooks": {
                "Stop": [
                    {"hooks":[{"type":"command","command":"agent-toast-send --event task_complete"}]}
                ],
                "SessionStart": [
                    {"hooks":[{"type":"command","command":"agent-toast-send --event session_start"}]}
                ]
            }
        });
        let entries = vec![HookEntry {
            event_key: "Stop",
            matcher: None,
            command: "agent-toast-send --url http://x --event task_complete".to_string(),
        }];
        let merged = merge_agent_toast_hooks(root, &entries);
        // Stop has the one new entry.
        assert_eq!(merged["hooks"]["Stop"].as_array().unwrap().len(), 1);
        // SessionStart's agent-toast entry should be cleared — it's not in the new set.
        let sess = merged["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(
            sess.len(),
            0,
            "SessionStart agent-toast must be cleared since not in new entries"
        );
    }

    #[test]
    fn merge_with_empty_entries_clears_all_agent_toast_but_keeps_others() {
        let root = json!({
            "hooks": {
                "Stop": [
                    {"hooks":[{"type":"command","command":"agent-toast-send --event x"}]},
                    {"hooks":[{"type":"command","command":"/usr/bin/custom"}]}
                ],
                "SessionStart": [
                    {"hooks":[{"type":"command","command":"agent-toast-send --event y"}]}
                ]
            }
        });
        let merged = merge_agent_toast_hooks(root, &[]);
        let stop = merged["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(
            stop.len(),
            1,
            "Only the non-agent-toast entry remains on Stop"
        );
        assert_eq!(
            stop[0]["hooks"][0]["command"].as_str(),
            Some("/usr/bin/custom")
        );
        let sess = merged["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(
            sess.len(),
            0,
            "SessionStart had only agent-toast — now empty"
        );
    }
}
