use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::cli::NotifyRequest;
use crate::notification::{show_notification, NotificationManagerState};

const CHECK_INTERVAL_HOURS: i64 = 12;

#[derive(Debug, Serialize, Deserialize, Default)]
struct UpdaterState {
    last_check: Option<String>,
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
    let state = UpdaterState {
        last_check: Some(Utc::now().to_rfc3339()),
    };
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
        eprintln!("[updater] Skipping check, not enough time passed");
        return;
    }

    let app = app.clone();
    let state = state.clone();
    std::thread::spawn(move || {
        eprintln!("[updater] Checking for updates...");

        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[updater] Failed to create HTTP client: {}", e);
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
                    eprintln!("[updater] Failed to parse response: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("[updater] Failed to fetch: {}", e);
                return;
            }
        };

        mark_checked();

        let current_version = app.package_info().version.to_string();
        eprintln!(
            "[updater] Current: {}, Latest: {}",
            current_version, release.tag_name
        );

        if is_newer(&current_version, &release.tag_name) {
            eprintln!("[updater] New version available!");
            show_update_notification(&app, &state, &release.tag_name);
        } else {
            eprintln!("[updater] Already up to date");
        }
    });
}

fn show_update_notification(app: &AppHandle, state: &NotificationManagerState, version: &str) {
    let locale = crate::setup::read_locale();
    let message = match locale.as_str() {
        "en" => format!("Version {} is available. Click to update.", version),
        _ => format!("{} 버전을 사용할 수 있습니다. 클릭하여 업데이트하세요.", version),
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
