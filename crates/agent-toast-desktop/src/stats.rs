use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    Activated,
    Manual,
    Timeout,
    Focus,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterSet {
    pub shown: u64,
    pub activated: u64,
    pub closed_manual: u64,
    pub closed_timeout: u64,
    pub closed_focus: u64,
    pub skipped_focused: u64,
    pub skipped_ratelimit: u64,
}

/// 글로벌 통계 동기화 상태 — stats.json의 예약 필드 `synced`에 저장된다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncedInfo {
    pub device_id: String,
    #[serde(default)]
    pub last_sync: Option<String>,
}

/// synced 필드가 예상 못 한 모양이어도 파일 전체를 corrupt 처리하지 않도록
/// 파싱 실패 시 None으로 강등한다.
fn lenient_synced<'de, D>(d: D) -> Result<Option<SyncedInfo>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(v.and_then(|v| serde_json::from_value(v).ok()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub version: u32,
    pub since: String,
    pub counts: HashMap<String, HashMap<String, CounterSet>>,
    pub origin: HashMap<String, CounterSet>,
    #[serde(default, deserialize_with = "lenient_synced")]
    pub synced: Option<SyncedInfo>,
}

impl Stats {
    pub fn new() -> Self {
        Self::empty(chrono::Utc::now().to_rfc3339())
    }

    pub fn empty(since: String) -> Self {
        Self {
            version: 1,
            since,
            counts: HashMap::new(),
            origin: HashMap::new(),
            synced: None,
        }
    }

    /// Apply `f` to both the `(event, source)` cell and the local/remote origin
    /// cell. No-op for updater (app-internal, not user behavior).
    fn bump(&mut self, event: &str, source: &str, remote: bool, f: impl Fn(&mut CounterSet)) {
        if source == "updater" {
            return;
        }
        let cell = self
            .counts
            .entry(event.to_string())
            .or_default()
            .entry(source.to_string())
            .or_default();
        f(cell);
        let origin_key = if remote { "remote" } else { "local" };
        let origin_cell = self.origin.entry(origin_key.to_string()).or_default();
        f(origin_cell);
    }

    pub fn record_shown(&mut self, event: &str, source: &str, remote: bool) {
        self.bump(event, source, remote, |c| c.shown += 1);
    }

    pub fn record_skipped_focused(&mut self, event: &str, source: &str, remote: bool) {
        self.bump(event, source, remote, |c| c.skipped_focused += 1);
    }

    pub fn record_skipped_ratelimit(&mut self, event: &str, source: &str, remote: bool) {
        self.bump(event, source, remote, |c| c.skipped_ratelimit += 1);
    }

    pub fn record_terminal(
        &mut self,
        reason: CloseReason,
        event: &str,
        source: &str,
        remote: bool,
    ) {
        self.bump(event, source, remote, |c| match reason {
            CloseReason::Activated => c.activated += 1,
            CloseReason::Manual => c.closed_manual += 1,
            CloseReason::Timeout => c.closed_timeout += 1,
            CloseReason::Focus => c.closed_focus += 1,
        });
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// Append a suffix to a full path (keeps the original extension): `stats.json`
/// -> `stats.json.corrupt`. (`PathBuf::with_extension` would replace `.json`.)
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

pub fn load_stats_from(path: &Path) -> Stats {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Stats::new(); // missing/unreadable -> fresh
    };
    match serde_json::from_str::<Stats>(&content) {
        Ok(s) => s,
        Err(_) => {
            // Preserve the bad file for inspection instead of silently clobbering it.
            let _ = std::fs::rename(path, sibling(path, ".corrupt"));
            Stats::new()
        }
    }
}

pub fn save_stats_to(path: &Path, stats: &Stats) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(stats).map_err(std::io::Error::other)?;
    let tmp = sibling(path, ".tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?; // atomic replace on the same volume
    Ok(())
}

pub struct StatsManager {
    pub stats: Stats,
    pub dirty: bool,
    /// 마지막 글로벌 업로드 성공 이후 통계가 바뀌었는지 (디스크 flush와 별개)
    pub sync_pending: bool,
}

pub type StatsState = Arc<Mutex<StatsManager>>;

/// Dev and prod coexist (separate `-dev` mutex/pipe), so the stats file is
/// namespaced too — otherwise dev runs and test notifications would pollute the
/// user's real production counters and clobber each other's writes.
pub fn stats_path() -> PathBuf {
    let file = if cfg!(debug_assertions) {
        "stats-dev.json"
    } else {
        "stats.json"
    };
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agent-toast")
        .join(file)
}

pub fn create_manager() -> StatsState {
    create_manager_at(&stats_path())
}

pub fn create_manager_at(path: &Path) -> StatsState {
    Arc::new(Mutex::new(StatsManager {
        stats: load_stats_from(path),
        dirty: false,
        sync_pending: false,
    }))
}

/// Apply `f`, marking the manager dirty only if the model actually changed.
/// (updater no-ops leave the model untouched, so they never trigger a flush.)
fn with_dirty(state: &StatsState, f: impl FnOnce(&mut Stats)) {
    let mut m = state.lock().unwrap();
    let before = m.stats.clone();
    f(&mut m.stats);
    if m.stats != before {
        m.dirty = true;
        m.sync_pending = true;
    }
}

pub fn record_shown(state: &StatsState, event: &str, source: &str, remote: bool) {
    with_dirty(state, |s| s.record_shown(event, source, remote));
}

pub fn record_skipped_focused(state: &StatsState, event: &str, source: &str, remote: bool) {
    with_dirty(state, |s| s.record_skipped_focused(event, source, remote));
}

pub fn record_skipped_ratelimit(state: &StatsState, event: &str, source: &str, remote: bool) {
    with_dirty(state, |s| s.record_skipped_ratelimit(event, source, remote));
}

pub fn record_terminal(
    state: &StatsState,
    reason: CloseReason,
    event: &str,
    source: &str,
    remote: bool,
) {
    with_dirty(state, |s| s.record_terminal(reason, event, source, remote));
}

/// If dirty, clear the flag and return a snapshot to persist; else `None`.
pub fn take_if_dirty(state: &StatsState) -> Option<Stats> {
    let mut m = state.lock().unwrap();
    if !m.dirty {
        return None;
    }
    m.dirty = false;
    Some(m.stats.clone())
}

pub fn is_sync_pending(state: &StatsState) -> bool {
    state.lock().unwrap().sync_pending
}

/// 업로드용 스냅샷. device_id가 없으면 UUID v4를 생성해 저장(dirty 표시)한다.
pub fn snapshot_for_sync(state: &StatsState) -> (String, Stats) {
    let mut m = state.lock().unwrap();
    if m.stats.synced.is_none() {
        m.stats.synced = Some(SyncedInfo {
            device_id: uuid::Uuid::new_v4().to_string(),
            last_sync: None,
        });
        m.dirty = true;
    }
    let id = m.stats.synced.as_ref().unwrap().device_id.clone();
    (id, m.stats.clone())
}

/// 업로드 성공 기록: last_sync 갱신 + sync_pending 해제.
pub fn mark_synced(state: &StatsState, when: String) {
    let mut m = state.lock().unwrap();
    if let Some(s) = m.stats.synced.as_mut() {
        s.last_sync = Some(when);
    }
    m.sync_pending = false;
    m.dirty = true;
}

/// Persist the model to disk if it changed since the last flush.
pub fn flush(state: &StatsState) {
    if let Some(snapshot) = take_if_dirty(state) {
        if let Err(e) = save_stats_to(&stats_path(), &snapshot) {
            log::warn!("[stats] flush failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Stats {
        Stats::empty("2026-06-30T00:00:00Z".into())
    }

    #[test]
    fn record_shown_bumps_counts_and_origin() {
        let mut s = fresh();
        s.record_shown("task_complete", "claude", false);
        assert_eq!(s.counts["task_complete"]["claude"].shown, 1);
        assert_eq!(s.origin["local"].shown, 1);
    }

    #[test]
    fn remote_flag_routes_origin() {
        let mut s = fresh();
        s.record_shown("error", "claude", true);
        assert_eq!(s.origin["remote"].shown, 1);
        assert!(!s.origin.contains_key("local"));
    }

    #[test]
    fn updater_source_is_ignored_everywhere() {
        let mut s = fresh();
        s.record_shown("task_complete", "updater", false);
        s.record_terminal(CloseReason::Manual, "task_complete", "updater", false);
        s.record_skipped_focused("task_complete", "updater", false);
        assert!(s.counts.is_empty());
        assert!(s.origin.is_empty());
    }

    #[test]
    fn terminal_reasons_map_to_fields() {
        let mut s = fresh();
        s.record_terminal(CloseReason::Activated, "e", "claude", false);
        s.record_terminal(CloseReason::Manual, "e", "claude", false);
        s.record_terminal(CloseReason::Timeout, "e", "claude", false);
        s.record_terminal(CloseReason::Focus, "e", "claude", false);
        let c = &s.counts["e"]["claude"];
        assert_eq!(
            (
                c.activated,
                c.closed_manual,
                c.closed_timeout,
                c.closed_focus
            ),
            (1, 1, 1, 1)
        );
    }

    #[test]
    fn open_ended_keys_appear_on_demand() {
        let mut s = fresh();
        s.record_skipped_ratelimit("brand_new_event", "codex", false);
        assert_eq!(s.counts["brand_new_event"]["codex"].skipped_ratelimit, 1);
    }

    #[test]
    fn origin_total_equals_counts_total() {
        let mut s = fresh();
        for _ in 0..5 {
            s.record_shown("a", "claude", false);
        }
        for _ in 0..3 {
            s.record_shown("b", "codex", true);
        }
        let counts_total: u64 = s
            .counts
            .values()
            .flat_map(|m| m.values())
            .map(|c| c.shown)
            .sum();
        let origin_total: u64 = s.origin.values().map(|c| c.shown).sum();
        assert_eq!(counts_total, 8);
        assert_eq!(origin_total, 8);
    }

    // ── SyncedInfo / sync_pending tests ──

    #[test]
    fn synced_info_roundtrip() {
        let mut s = fresh();
        s.synced = Some(SyncedInfo {
            device_id: "abc".into(),
            last_sync: Some("2026-07-02T00:00:00Z".into()),
        });
        let json = serde_json::to_string(&s).unwrap();
        let back: Stats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.synced.as_ref().unwrap().device_id, "abc");
    }

    #[test]
    fn unknown_synced_shape_degrades_to_none() {
        // 예전/외부에서 온 이상한 synced 값이 있어도 파일 전체를 corrupt 처리하지 않는다.
        let json = r#"{"version":1,"since":"x","counts":{},"origin":{},"synced":{"foo":1}}"#;
        let s: Stats = serde_json::from_str(json).unwrap();
        assert!(s.synced.is_none());
    }

    #[test]
    fn record_sets_sync_pending() {
        let path = temp_path("sync-pending");
        let state = create_manager_at(&path);
        assert!(!is_sync_pending(&state));
        super::record_shown(&state, "task_complete", "claude", false);
        assert!(is_sync_pending(&state));
    }

    #[test]
    fn take_if_dirty_does_not_clear_sync_pending() {
        let path = temp_path("sync-pending-flush");
        let state = create_manager_at(&path);
        super::record_shown(&state, "task_complete", "claude", false);
        let _ = take_if_dirty(&state); // disk flush
        assert!(
            is_sync_pending(&state),
            "flush must not consume the upload flag"
        );
    }

    #[test]
    fn snapshot_for_sync_generates_stable_device_id() {
        let path = temp_path("device-id");
        let state = create_manager_at(&path);
        let (id1, snap) = snapshot_for_sync(&state);
        let (id2, _) = snapshot_for_sync(&state);
        assert_eq!(id1, id2, "id generated once, then reused");
        assert_eq!(snap.synced.as_ref().unwrap().device_id, id1);
        assert!(
            take_if_dirty(&state).is_some(),
            "new id marks dirty for persistence"
        );
    }

    #[test]
    fn mark_synced_stamps_and_clears_pending() {
        let path = temp_path("mark-synced");
        let state = create_manager_at(&path);
        super::record_shown(&state, "task_complete", "claude", false);
        let _ = snapshot_for_sync(&state);
        mark_synced(&state, "2026-07-02T01:00:00Z".into());
        assert!(!is_sync_pending(&state));
        let m = state.lock().unwrap();
        assert_eq!(
            m.stats.synced.as_ref().unwrap().last_sync.as_deref(),
            Some("2026-07-02T01:00:00Z")
        );
    }

    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "agent-toast-stats-test-{}-{}",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p.push("stats.json");
        p
    }

    #[test]
    fn load_missing_file_starts_fresh_with_since() {
        let path = temp_path("missing");
        let s = load_stats_from(&path);
        assert!(s.counts.is_empty());
        assert!(!s.since.is_empty());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let path = temp_path("roundtrip");
        let mut s = Stats::empty("2026-01-01T00:00:00Z".into());
        s.record_shown("task_complete", "claude", false);
        save_stats_to(&path, &s).unwrap();
        let loaded = load_stats_from(&path);
        assert_eq!(loaded, s);
    }

    #[test]
    fn corrupt_file_is_renamed_aside_not_overwritten() {
        let path = temp_path("corrupt");
        std::fs::write(&path, b"{ this is not json").unwrap();
        let s = load_stats_from(&path);
        assert!(s.counts.is_empty()); // fresh
        let mut aside = path.clone().into_os_string();
        aside.push(".corrupt");
        assert!(std::path::Path::new(&aside).exists(), "bad file preserved");
    }

    #[test]
    fn save_is_atomic_no_tmp_left_behind() {
        let path = temp_path("atomic");
        save_stats_to(&path, &Stats::empty("x".into())).unwrap();
        let mut tmp = path.clone().into_os_string();
        tmp.push(".tmp");
        assert!(
            !std::path::Path::new(&tmp).exists(),
            "tmp cleaned up by rename"
        );
        assert!(path.exists());
    }

    #[test]
    fn recording_sets_dirty_and_take_clears_it() {
        let path = temp_path("dirty");
        let state = create_manager_at(&path);
        assert!(take_if_dirty(&state).is_none(), "clean at start");
        super::record_shown(&state, "task_complete", "claude", false);
        let taken = take_if_dirty(&state).expect("dirty after record");
        assert_eq!(taken.counts["task_complete"]["claude"].shown, 1);
        assert!(take_if_dirty(&state).is_none(), "clean after take");
    }

    #[test]
    fn updater_record_does_not_set_dirty() {
        let path = temp_path("updater-clean");
        let state = create_manager_at(&path);
        super::record_terminal(&state, CloseReason::Manual, "e", "updater", false);
        assert!(take_if_dirty(&state).is_none(), "updater never dirties");
    }
}
