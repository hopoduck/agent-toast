//! Orca runtime integration.
//!
//! Orca hosts every agent session as a tab inside a single window, and the
//! window belongs to a process that is not in the session's parent chain: the
//! shell's parent is `orca-terminal-daemon.exe`, a detached daemon whose own
//! parent is already gone. So the process-tree walk in [`crate::win32`] finds
//! no window at all, `source_hwnd` stays 0, and both the skip-if-focused and
//! the auto-close-on-focus paths silently stop working. Every tab also shares
//! the same window title (`Orca`), so title matching cannot separate them
//! either.
//!
//! Two files Orca owns carry what is needed. `%APPDATA%\orca\orca-runtime.json`
//! publishes the app PID and a named pipe speaking newline-delimited JSON:
//! write one request object, read frames until the one carrying the request id
//! arrives (`_keepalive` frames may be interleaved). That pipe is how a click
//! on a toast switches to its tab.
//!
//! Which tab is *on screen* is not on the pipe at all. `terminal.resolveActive`
//! reads like the method for it, but called without a worktree it walks the tab
//! map and returns the first entry, so it names whatever tab was created first
//! no matter what the UI is showing. The runtime reads the real value from its
//! own store and exposes it only to plugins, and the RPC envelope permits
//! exactly three frames (success, failure, keepalive), so there is no event to
//! subscribe to either. The active tab therefore comes from Orca's persisted
//! state, `profiles/<active profile>/orca-data.json`.
//!
//! Both are Orca internals rather than a published API, so every lookup here
//! fails soft: a missing file, an unreachable pipe, an unexpected frame or a
//! changed JSON shape all resolve to `None` and leave the caller on the
//! ordinary window-based behavior.

use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

/// Ceiling for one runtime round trip. The runtime answers in single-digit
/// milliseconds; this only exists so a wedged pipe cannot stall a notification.
const RPC_TIMEOUT: Duration = Duration::from_millis(400);

static REQUEST_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Deserialize)]
struct RuntimeTransport {
    kind: String,
    endpoint: String,
}

/// Contents of `orca-runtime.json`, rewritten by Orca on every app start.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeMeta {
    /// PID of the Orca app process that owns the window.
    pub pid: u32,
    #[serde(rename = "authToken")]
    auth_token: String,
    transports: Vec<RuntimeTransport>,
}

impl RuntimeMeta {
    /// The named-pipe endpoint. The file also advertises a websocket, which
    /// would need a handshake the pipe does not.
    fn pipe_endpoint(&self) -> Option<&str> {
        self.transports
            .iter()
            .find(|t| t.kind == "named-pipe")
            .map(|t| t.endpoint.as_str())
    }
}

/// Parse `orca-runtime.json` contents. Split from the file read so the shape
/// stays testable off-Windows.
fn parse_runtime_meta(text: &str) -> Option<RuntimeMeta> {
    serde_json::from_str(text).ok()
}

/// Read Orca's runtime metadata, or `None` when Orca has never run on this
/// machine. Re-read per call rather than cached: the file changes on every
/// Orca restart (new pid, pipe and token) and reading ~400 bytes is cheaper
/// than reasoning about staleness.
pub fn read_runtime_meta() -> Option<RuntimeMeta> {
    let path = dirs::config_dir()?.join("orca").join("orca-runtime.json");
    let text = std::fs::read_to_string(path).ok()?;
    parse_runtime_meta(&text)
}

/// Send one RPC and return its `result`, or `None` on any failure.
///
/// The blocking pipe work runs on a throwaway thread so a runtime that accepts
/// the connection and then never answers cannot hold up the caller past
/// [`RPC_TIMEOUT`].
fn call(method: &str, params: serde_json::Value) -> Option<serde_json::Value> {
    let meta = read_runtime_meta()?;
    let endpoint = meta.pipe_endpoint()?.to_string();
    let id = format!(
        "agent-toast-{}-{}",
        std::process::id(),
        REQUEST_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let request = serde_json::json!({
        "id": id,
        "authToken": meta.auth_token,
        "method": method,
        "params": params,
    });
    let mut line = serde_json::to_string(&request).ok()?;
    line.push('\n');

    let (tx, rx) = std::sync::mpsc::channel();
    let id_for_thread = id.clone();
    std::thread::spawn(move || {
        let _ = tx.send(exchange(&endpoint, &line, &id_for_thread));
    });

    match rx.recv_timeout(RPC_TIMEOUT) {
        Ok(result) => result,
        Err(_) => {
            log::debug!("[ORCA] {} timed out after {:?}", method, RPC_TIMEOUT);
            None
        }
    }
}

/// One blocking request/response exchange over the runtime pipe.
fn exchange(endpoint: &str, line: &str, id: &str) -> Option<serde_json::Value> {
    use std::io::{BufRead, BufReader, Write};

    let mut pipe = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(endpoint)
        .ok()?;
    pipe.write_all(line.as_bytes()).ok()?;
    pipe.flush().ok()?;

    let reader = BufReader::new(pipe);
    for frame in reader.lines() {
        let frame = frame.ok()?;
        if frame.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<serde_json::Value>(&frame).ok()?;
        if let Some(result) = read_frame(&value, id) {
            return result;
        }
    }
    None
}

/// Classify one response frame.
///
/// - `None`: not the frame we are waiting for (keepalive, or another id), keep reading.
/// - `Some(None)`: terminal failure frame.
/// - `Some(Some(result))`: the answer.
fn read_frame(frame: &serde_json::Value, id: &str) -> Option<Option<serde_json::Value>> {
    if frame.get("_keepalive").is_some() {
        return None;
    }
    if frame.get("id").and_then(|v| v.as_str()) != Some(id) {
        return None;
    }
    if frame.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        log::debug!("[ORCA] runtime returned an error frame: {}", frame);
        return Some(None);
    }
    Some(frame.get("result").cloned())
}

/// PID of the Orca app process that owns the window, when Orca is running.
pub fn app_pid() -> Option<u32> {
    read_runtime_meta().map(|m| m.pid)
}

/// The Orca window, found by the runtime's own app PID rather than by walking
/// the notifying process's parents (which never reaches it).
pub fn app_window() -> Option<isize> {
    let pid = app_pid()?;
    crate::win32::find_visible_window_for_pids(&[pid])
}

/// Orca keeps one profile's state per directory and names the live one here.
#[derive(Debug, Clone, Deserialize)]
struct ProfileIndex {
    #[serde(rename = "activeProfileId")]
    active_profile_id: String,
}

/// The slice of `orca-data.json` this module cares about. Everything else in
/// the file (repos, worktrees, terminal scrollback) is ignored, and unknown
/// fields are dropped, so added keys cannot break the read.
#[derive(Debug, Clone, Deserialize)]
struct AppData {
    #[serde(rename = "workspaceSession")]
    workspace_session: Option<WorkspaceSession>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkspaceSession {
    /// The tab the user is looking at, across every workspace. Matches the
    /// `ORCA_TAB_ID` Orca exports into that tab's terminal.
    #[serde(rename = "activeTabId")]
    active_tab_id: Option<String>,
}

fn parse_active_profile_id(text: &str) -> Option<String> {
    serde_json::from_str::<ProfileIndex>(text)
        .ok()
        .map(|i| i.active_profile_id)
        .filter(|id| !id.is_empty())
}

fn parse_active_tab_id(text: &str) -> Option<String> {
    serde_json::from_str::<AppData>(text)
        .ok()?
        .workspace_session?
        .active_tab_id
        .filter(|id| !id.is_empty())
}

/// Path of the live profile's persisted state.
fn data_file_path() -> Option<PathBuf> {
    let base = dirs::config_dir()?.join("orca");
    let index = std::fs::read_to_string(base.join("orca-profile-index.json")).ok()?;
    let profile = parse_active_profile_id(&index)?;
    Some(base.join("profiles").join(profile).join("orca-data.json"))
}

/// Last parse of `orca-data.json`, keyed by the file identity it came from.
///
/// The file runs to a megabyte and more, and the tab watcher asks several times
/// a second, so it is re-read only once Orca has written to it again.
static ACTIVE_TAB_CACHE: Mutex<Option<CachedActiveTab>> = Mutex::new(None);

#[derive(Debug)]
struct CachedActiveTab {
    modified: SystemTime,
    len: u64,
    tab_id: Option<String>,
}

/// Id of the Orca tab currently on screen.
///
/// Orca writes this on a debounce, so it can trail the UI by up to a second.
/// It is also UI state that survives Orca losing focus, and therefore only
/// means the user is reading that tab when the Orca window holds focus too.
pub fn active_tab_id() -> Option<String> {
    let path = data_file_path()?;
    let meta = std::fs::metadata(&path).ok()?;
    let modified = meta.modified().ok()?;
    let len = meta.len();

    let mut cache = ACTIVE_TAB_CACHE.lock().ok()?;
    if let Some(cached) = cache.as_ref() {
        if cached.modified == modified && cached.len == len {
            return cached.tab_id.clone();
        }
    }

    // A read that lands mid-write parses to `None` and is cached as such; the
    // completed write moves the mtime again, which retries it.
    let tab_id = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| parse_active_tab_id(&text));
    *cache = Some(CachedActiveTab {
        modified,
        len,
        tab_id: tab_id.clone(),
    });
    tab_id
}

/// Bring one terminal tab to the front. `navigation: "host"` asks Orca to
/// surface the tab's own window too, which matters when the notification came
/// from a workspace that is not the one on screen.
pub fn focus_terminal(handle: &str) -> bool {
    call(
        "terminal.focus",
        serde_json::json!({ "terminal": handle, "navigation": "host" }),
    )
    .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "runtimeId": "23c25aae-0b5d-4025-9d10-7313f18e88d9",
      "pid": 92008,
      "transports": [
        { "kind": "named-pipe", "endpoint": "\\\\.\\pipe\\orca-92008-23c2" },
        { "kind": "websocket", "endpoint": "ws://127.0.0.1:6768" }
      ],
      "authToken": "797ced982fad4f2ba21acdcd618f3cde942ad645f9639205",
      "startedAt": 1788166666375
    }"#;

    #[test]
    fn parses_runtime_metadata() {
        let meta = parse_runtime_meta(SAMPLE).unwrap();
        assert_eq!(meta.pid, 92008);
        assert_eq!(meta.pipe_endpoint(), Some(r"\\.\pipe\orca-92008-23c2"));
    }

    #[test]
    fn prefers_named_pipe_over_websocket() {
        let json = r#"{"pid":1,"authToken":"t","transports":[
            {"kind":"websocket","endpoint":"ws://127.0.0.1:6768"},
            {"kind":"named-pipe","endpoint":"\\\\.\\pipe\\orca-1-aaaa"}
        ]}"#;
        let meta = parse_runtime_meta(json).unwrap();
        assert_eq!(meta.pipe_endpoint(), Some(r"\\.\pipe\orca-1-aaaa"));
    }

    #[test]
    fn missing_named_pipe_transport_yields_none() {
        let json = r#"{"pid":1,"authToken":"t","transports":[
            {"kind":"websocket","endpoint":"ws://127.0.0.1:6768"}
        ]}"#;
        let meta = parse_runtime_meta(json).unwrap();
        assert!(meta.pipe_endpoint().is_none());
    }

    #[test]
    fn garbage_metadata_is_not_fatal() {
        assert!(parse_runtime_meta("not json").is_none());
        assert!(parse_runtime_meta("{}").is_none());
    }

    #[test]
    fn keepalive_frames_are_skipped() {
        let frame = serde_json::json!({ "_keepalive": true });
        assert!(read_frame(&frame, "req-1").is_none());
    }

    #[test]
    fn frames_for_another_request_are_skipped() {
        let frame = serde_json::json!({ "id": "req-2", "ok": true, "result": {} });
        assert!(read_frame(&frame, "req-1").is_none());
    }

    #[test]
    fn error_frame_resolves_to_failure() {
        let frame = serde_json::json!({
            "id": "req-1",
            "ok": false,
            "error": { "code": "method_not_found", "message": "Unknown method" }
        });
        assert_eq!(read_frame(&frame, "req-1"), Some(None));
    }

    #[test]
    fn success_frame_yields_result() {
        let frame = serde_json::json!({
            "id": "req-1",
            "ok": true,
            "result": { "handle": "term_58587268" }
        });
        let result = read_frame(&frame, "req-1").unwrap().unwrap();
        assert_eq!(result["handle"], "term_58587268");
    }

    const PROFILE_INDEX: &str = r#"{
      "schemaVersion": 1,
      "activeProfileId": "local-default",
      "profiles": [{ "id": "local-default", "name": "Personal" }]
    }"#;

    #[test]
    fn reads_the_live_profile_id() {
        assert_eq!(
            parse_active_profile_id(PROFILE_INDEX).as_deref(),
            Some("local-default")
        );
    }

    #[test]
    fn profile_index_without_an_active_id_yields_none() {
        assert!(parse_active_profile_id(r#"{"profiles":[]}"#).is_none());
        assert!(parse_active_profile_id(r#"{"activeProfileId":""}"#).is_none());
        assert!(parse_active_profile_id("not json").is_none());
    }

    #[test]
    fn reads_the_active_tab_id() {
        let data = r#"{
          "schemaVersion": 1,
          "repos": [{ "id": "r1" }],
          "workspaceSession": {
            "activeRepoId": "8a6cbd96",
            "activeWorktreeId": "8a6cbd96::C:/git/rust_project/claude-notify",
            "activeTabId": "62fb1a1d-81be-4eab-ba3b-72930068b1c7",
            "tabsByWorktree": {}
          }
        }"#;
        assert_eq!(
            parse_active_tab_id(data).as_deref(),
            Some("62fb1a1d-81be-4eab-ba3b-72930068b1c7")
        );
    }

    #[test]
    fn unknown_keys_do_not_break_the_read() {
        let data = r#"{
          "workspaceSession": { "activeTabId": "tab-1", "somethingNew": 42 },
          "featureAddedLater": { "nested": true }
        }"#;
        assert_eq!(parse_active_tab_id(data).as_deref(), Some("tab-1"));
    }

    #[test]
    fn missing_or_empty_active_tab_yields_none() {
        assert!(parse_active_tab_id(r#"{"workspaceSession":{}}"#).is_none());
        assert!(parse_active_tab_id(r#"{"workspaceSession":{"activeTabId":null}}"#).is_none());
        assert!(parse_active_tab_id(r#"{"workspaceSession":{"activeTabId":""}}"#).is_none());
        assert!(parse_active_tab_id("{}").is_none());
        // A read that lands mid-write sees truncated JSON.
        assert!(parse_active_tab_id(r#"{"workspaceSession":{"activeTa"#).is_none());
    }
}
