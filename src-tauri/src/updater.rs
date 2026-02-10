use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::cli::NotifyRequest;
use crate::notification::{show_notification, NotificationManagerState};

const CHECK_INTERVAL_HOURS: i64 = 12;

#[derive(Debug, Serialize, Deserialize, Default)]
struct UpdaterState {
    last_check: Option<String>,
    pending_version: Option<String>,
}

fn get_state_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("agent-toast").join("updater.json"))
}

fn load_state() -> UpdaterState {
    get_state_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &UpdaterState) {
    if let Some(path) = get_state_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, serde_json::to_string(state).unwrap_or_default());
    }
}

fn should_check() -> bool {
    let state = load_state();
    match state.last_check {
        None => true,
        Some(last) => {
            if let Ok(last_time) = last.parse::<DateTime<Utc>>() {
                let now = Utc::now();
                let diff = now.signed_duration_since(last_time);
                diff.num_hours() >= CHECK_INTERVAL_HOURS
            } else {
                true
            }
        }
    }
}

fn mark_checked() {
    let mut state = load_state();
    state.last_check = Some(Utc::now().to_rfc3339());
    save_state(&state);
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

pub fn check_for_updates(app: &AppHandle, state: &NotificationManagerState) {
    if !should_check() {
        debug!("Skipping check, not enough time passed");
        return;
    }

    let app = app.clone();
    let state = state.clone();
    std::thread::spawn(move || {
        info!("Checking for updates...");

        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create HTTP client: {}", e);
                return;
            }
        };

        let resp = client
            .get("https://api.github.com/repos/hopoduck/agent-toast/releases/latest")
            .header("User-Agent", "agent-toast-updater")
            .send();

        let release: GithubRelease = match resp {
            Ok(r) => match r.json() {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to parse response: {}", e);
                    return;
                }
            },
            Err(e) => {
                error!("Failed to fetch: {}", e);
                return;
            }
        };

        mark_checked();

        let current_version = app.package_info().version.to_string();
        info!("Current: {}, Latest: {}", current_version, release.tag_name);

        if is_newer(&current_version, &release.tag_name) {
            info!("New version available!");
            show_update_notification(&app, &state, &release.tag_name);
        } else {
            debug!("Already up to date");
        }
    });
}

fn show_update_notification(app: &AppHandle, state: &NotificationManagerState, version: &str) {
    let locale = crate::setup::read_locale();
    let message = match locale.as_str() {
        "en" => format!("Version {} is available. Click to update.", version),
        _ => format!(
            "{} 버전을 사용할 수 있습니다. 클릭하여 업데이트하세요.",
            version
        ),
    };

    let req = NotifyRequest {
        pid: 0,
        event: "update_available".to_string(),
        message: Some(message),
        title_hint: Some("Agent Toast".to_string()),
        process_tree: Some(vec![]),
        source: "updater".into(),
    };

    show_notification(app, state, req);
}

#[tauri::command]
pub fn mark_update_pending(version: String) {
    let mut state = load_state();
    state.pending_version = Some(version);
    save_state(&state);
    debug!("Marked update pending");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_version tests ──

    #[test]
    fn parse_version_standard() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn parse_version_with_v_prefix() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn parse_version_zero_components() {
        assert_eq!(parse_version("0.0.0"), Some((0, 0, 0)));
    }

    #[test]
    fn parse_version_large_numbers() {
        assert_eq!(parse_version("10.20.300"), Some((10, 20, 300)));
    }

    #[test]
    fn parse_version_two_parts_fails() {
        assert_eq!(parse_version("1.2"), None);
    }

    #[test]
    fn parse_version_single_part_fails() {
        assert_eq!(parse_version("1"), None);
    }

    #[test]
    fn parse_version_empty_fails() {
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn parse_version_non_numeric_fails() {
        assert_eq!(parse_version("a.b.c"), None);
    }

    #[test]
    fn parse_version_extra_parts_uses_first_three() {
        // "1.2.3.4" → parts = ["1", "2", "3", "4"], len >= 3
        assert_eq!(parse_version("1.2.3.4"), Some((1, 2, 3)));
    }

    // ── is_newer tests ──

    #[test]
    fn is_newer_patch_bump() {
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn is_newer_minor_bump() {
        assert!(is_newer("1.0.0", "1.1.0"));
    }

    #[test]
    fn is_newer_major_bump() {
        assert!(is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn is_newer_same_version() {
        assert!(!is_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn is_newer_older_version() {
        assert!(!is_newer("2.0.0", "1.0.0"));
    }

    #[test]
    fn is_newer_with_v_prefix() {
        assert!(is_newer("v1.0.0", "v1.0.1"));
        assert!(is_newer("1.0.0", "v1.0.1"));
        assert!(is_newer("v1.0.0", "1.0.1"));
    }

    #[test]
    fn is_newer_invalid_current() {
        assert!(!is_newer("invalid", "1.0.0"));
    }

    #[test]
    fn is_newer_invalid_latest() {
        assert!(!is_newer("1.0.0", "invalid"));
    }

    #[test]
    fn is_newer_both_invalid() {
        assert!(!is_newer("invalid", "also-invalid"));
    }

    // ── UpdaterState tests ──

    #[test]
    fn updater_state_default() {
        let state = UpdaterState::default();
        assert!(state.last_check.is_none());
        assert!(state.pending_version.is_none());
    }

    #[test]
    fn updater_state_serde_roundtrip() {
        let state = UpdaterState {
            last_check: Some("2024-01-01T12:00:00Z".to_string()),
            pending_version: Some("v1.2.3".to_string()),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: UpdaterState = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.last_check,
            Some("2024-01-01T12:00:00Z".to_string())
        );
        assert_eq!(deserialized.pending_version, Some("v1.2.3".to_string()));
    }

    #[test]
    fn updater_state_deserialize_empty_json() {
        let json = "{}";
        let state: UpdaterState = serde_json::from_str(json).unwrap();
        assert!(state.last_check.is_none());
        assert!(state.pending_version.is_none());
    }

    #[test]
    fn updater_state_deserialize_partial() {
        let json = r#"{"last_check": "2024-06-15T10:00:00Z"}"#;
        let state: UpdaterState = serde_json::from_str(json).unwrap();
        assert_eq!(state.last_check, Some("2024-06-15T10:00:00Z".to_string()));
        assert!(state.pending_version.is_none());
    }

    // ── Check interval tests ──

    #[test]
    fn check_interval_is_reasonable() {
        // 12시간 간격이 적절한 범위인지 확인 (1시간 ~ 24시간)
        assert!(CHECK_INTERVAL_HOURS >= 1);
        assert!(CHECK_INTERVAL_HOURS <= 24);
    }

    #[test]
    fn last_check_datetime_parsing() {
        // RFC3339 형식 파싱 테스트 (should_check 내부 로직)
        let timestamp = "2024-06-15T10:30:00Z";
        let parsed = timestamp.parse::<DateTime<Utc>>();
        assert!(parsed.is_ok());
    }

    #[test]
    fn last_check_invalid_datetime_format() {
        // 잘못된 형식은 파싱 실패
        let invalid = "not-a-date";
        let parsed = invalid.parse::<DateTime<Utc>>();
        assert!(parsed.is_err());
    }

    #[test]
    fn duration_calculation_logic() {
        // 시간 차이 계산 로직 검증
        let past = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let future = "2024-01-01T13:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let diff = future.signed_duration_since(past);
        assert_eq!(diff.num_hours(), 13);
        assert!(diff.num_hours() >= CHECK_INTERVAL_HOURS);
    }

    #[test]
    fn duration_under_interval() {
        // 간격 미달 케이스
        let past = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let future = "2024-01-01T06:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let diff = future.signed_duration_since(past);
        assert_eq!(diff.num_hours(), 6);
        assert!(diff.num_hours() < CHECK_INTERVAL_HOURS);
    }

    // ── GithubRelease tests ──

    #[test]
    fn github_release_deserialize() {
        let json = r#"{"tag_name": "v1.2.3"}"#;
        let release: GithubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v1.2.3");
    }

    #[test]
    fn github_release_with_extra_fields() {
        // API 응답에는 더 많은 필드가 있지만 무시됨
        let json = r#"{"tag_name": "v2.0.0", "name": "Release 2.0", "draft": false}"#;
        let release: GithubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "v2.0.0");
    }

    // ── Version edge cases ──

    #[test]
    fn is_newer_leading_zeros() {
        // 선행 0은 숫자로 파싱되므로 무시됨
        assert!(is_newer("01.02.03", "1.2.4"));
    }

    #[test]
    fn parse_version_whitespace_handling() {
        // trim_start_matches('v')만 하므로 공백은 처리 안 됨
        assert_eq!(parse_version(" 1.2.3"), None);
    }

    #[test]
    fn parse_version_negative_numbers_fail() {
        // u32 파싱이므로 음수는 실패
        assert_eq!(parse_version("-1.2.3"), None);
    }
}

pub fn check_update_completed(app: &AppHandle, state: &NotificationManagerState) {
    let mut updater_state = load_state();
    if let Some(pending_version) = updater_state.pending_version.take() {
        save_state(&updater_state);

        let current_version = app.package_info().version.to_string();
        // Only show completion if we're now on the pending version
        if format!("v{}", current_version) == pending_version
            || current_version == pending_version.trim_start_matches('v')
        {
            info!(
                "Update completed: {} -> {}",
                pending_version, current_version
            );
            show_update_completed_notification(app, state, &current_version);
        }
    }
}

fn show_update_completed_notification(
    app: &AppHandle,
    state: &NotificationManagerState,
    version: &str,
) {
    let locale = crate::setup::read_locale();
    let message = match locale.as_str() {
        "en" => format!("Updated to v{}!", version),
        _ => format!("v{}(으)로 업데이트되었습니다!", version),
    };

    let req = NotifyRequest {
        pid: 0,
        event: "task_complete".to_string(),
        message: Some(message),
        title_hint: Some("Agent Toast".to_string()),
        process_tree: Some(vec![]),
        source: "updater".into(),
    };

    show_notification(app, state, req);
}
