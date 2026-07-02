//! 글로벌 통계 동기화: 누적 카운터를 Cloudflare Worker에 업로드하고,
//! 통계 탭에 보여줄 글로벌 집계를 가져온다. 업로드는 절대 알림 경로를
//! 블로킹하지 않는다(별도 스레드).

use std::time::Duration;

use log::{debug, info, warn};
use serde::Serialize;

use crate::setup;
use crate::stats::{self, Stats, StatsState};

pub const WORKER_BASE_URL: &str = "https://agent-toast-stats.hopoduck.com";

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize)]
struct SyncPayload<'a> {
    device_id: &'a str,
    version: u32,
    since: &'a str,
    counts:
        &'a std::collections::HashMap<String, std::collections::HashMap<String, stats::CounterSet>>,
    origin: &'a std::collections::HashMap<String, stats::CounterSet>,
}

/// 업로드 페이로드 JSON 직렬화 (순수 함수 — 테스트 대상).
fn build_payload_json(device_id: &str, s: &Stats) -> String {
    serde_json::to_string(&SyncPayload {
        device_id,
        version: s.version,
        since: &s.since,
        counts: &s.counts,
        origin: &s.origin,
    })
    .unwrap_or_default()
}

/// dev 빌드는 절대 업로드하지 않는다(프로덕션 수치 오염 방지, stats-dev와 동일 원칙).
fn upload_allowed() -> bool {
    !cfg!(debug_assertions) && setup::read_global_stats_enabled()
}

/// 앱 시작 시 `force=true`(무조건 1회), 1시간 틱에서 `force=false`(변경 시에만).
pub fn sync(state: &StatsState, force: bool) {
    if !upload_allowed() {
        debug!("[global-sync] upload disabled (dev build or setting off)");
        return;
    }
    if !force && !stats::is_sync_pending(state) {
        debug!("[global-sync] nothing changed since last sync, skipping");
        return;
    }
    let (device_id, snapshot) = stats::snapshot_for_sync(state);
    let state = state.clone();
    std::thread::spawn(move || {
        let body = build_payload_json(&device_id, &snapshot);
        let client = match reqwest::blocking::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("[global-sync] client build failed: {e}");
                return;
            }
        };
        let resp = client
            .post(format!("{WORKER_BASE_URL}/v1/sync"))
            .header("content-type", "application/json")
            .body(body)
            .send();
        match resp {
            Ok(r) if r.status().is_success() => {
                stats::mark_synced(&state, chrono::Utc::now().to_rfc3339());
                info!("[global-sync] uploaded");
            }
            Ok(r) => warn!("[global-sync] server rejected: {}", r.status()),
            Err(e) => warn!("[global-sync] upload failed: {e}"),
        }
        // 실패 시 아무것도 하지 않는다 — 누적값이라 다음 틱 재시도로 충분.
    });
}

/// 통계 탭의 글로벌 섹션 데이터. Worker 응답을 그대로 전달한다(프론트에서 타입 지정).
#[tauri::command]
pub async fn get_global_stats() -> Result<serde_json::Value, String> {
    if !setup::read_global_stats_enabled() {
        return Err("disabled".into());
    }
    tauri::async_runtime::spawn_blocking(|| {
        let client = reqwest::blocking::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client
            .get(format!("{WORKER_BASE_URL}/v1/global"))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("server error: {}", resp.status()));
        }
        resp.json::<serde_json::Value>().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_includes_device_id_and_counts() {
        let mut s = Stats::empty("2026-06-01T00:00:00Z".into());
        s.record_shown("task_complete", "claude", false);
        let json = build_payload_json("test-device-id", &s);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["device_id"], "test-device-id");
        assert_eq!(v["since"], "2026-06-01T00:00:00Z");
        assert_eq!(v["counts"]["task_complete"]["claude"]["shown"], 1);
        assert_eq!(v["origin"]["local"]["shown"], 1);
        assert!(v.get("synced").is_none(), "synced stays local-only");
    }

    #[test]
    fn payload_of_empty_stats_is_valid_json() {
        let s = Stats::empty("x".into());
        let v: serde_json::Value = serde_json::from_str(&build_payload_json("id", &s)).unwrap();
        assert_eq!(v["version"], 1);
    }
}
