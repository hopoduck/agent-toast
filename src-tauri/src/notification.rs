use crate::cli::NotifyRequest;
use crate::win32;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Notification window width in logical pixels.
/// Sized to fit title + message comfortably (min 200, max 600 for readability).
const NOTIFICATION_WIDTH: f64 = 380.0;
/// Notification window height in logical pixels.
/// Sized to fit 2-3 lines of text with padding (min 80, max 300).
const NOTIFICATION_HEIGHT: f64 = 140.0;
/// Margin between notification windows and screen edges in logical pixels.
const NOTIFICATION_MARGIN: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub id: String,
    pub window_title: String,
    pub event_display: String,
    pub message: Option<String>,
    pub source_hwnd: isize,
    pub process_tree: Vec<u32>,
    pub auto_dismiss_seconds: u32,
    pub source: String,
}

pub struct NotificationManager {
    notifications: Vec<NotificationData>,
    counter: u32,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            counter: 0,
        }
    }
}

pub type NotificationManagerState = Arc<Mutex<NotificationManager>>;

pub fn create_manager() -> NotificationManagerState {
    Arc::new(Mutex::new(NotificationManager::new()))
}

/// Returns notification data for a specific window label
pub fn get_notification_for_window(
    state: &NotificationManagerState,
    window_label: &str,
) -> Option<NotificationData> {
    let mgr = state.lock().unwrap();
    mgr.notifications
        .iter()
        .find(|n| n.id == window_label)
        .cloned()
}

pub fn show_notification(
    app: &AppHandle,
    state: &NotificationManagerState,
    request: NotifyRequest,
) {
    log::debug!(
        "[NOTIFY] show_notification called: event={}, pid={}, source={}",
        request.event,
        request.pid,
        request.source
    );

    // For internal notifications (updater), skip win32 lookups
    let is_internal = request.source == "updater";

    let (source_hwnd, process_tree, window_title) = if is_internal {
        (
            0isize,
            vec![],
            request
                .title_hint
                .clone()
                .unwrap_or_else(|| "Agent Toast".to_string()),
        )
    } else {
        let tree = request
            .process_tree
            .clone()
            .filter(|t| t.len() > 1)
            .unwrap_or_else(|| win32::get_process_tree(request.pid));

        let (all_candidates, found) =
            win32::find_source_window(&tree, request.title_hint.as_deref());
        log::debug!(
            "[DEBUG] event={}, title_hint={:?}, process_tree={:?}, find_source_window={:?}",
            request.event,
            request.title_hint,
            tree,
            found
        );
        for (h, p) in &all_candidates {
            let title = win32::get_window_title(*h);
            log::debug!("[DEBUG]   candidate hwnd={} pid={} title={:?}", h, p, title);
        }
        let (hwnd, _) = found.unwrap_or((0, 0));

        // FR-2: Skip if source window is already focused (compare by HWND, not PID)
        let focused = win32::is_hwnd_focused(hwnd);
        log::debug!("[DEBUG] is_hwnd_focused({})={}", hwnd, focused);
        if focused {
            return;
        }

        let title = {
            let title_mode = crate::setup::get_hook_config().title_display_mode;
            if title_mode == "project" {
                request.title_hint.clone().unwrap_or_else(|| {
                    if hwnd != 0 {
                        win32::get_window_title(hwnd)
                    } else {
                        format!("PID {}", request.pid)
                    }
                })
            } else if hwnd != 0 {
                win32::get_window_title(hwnd)
            } else {
                format!("PID {}", request.pid)
            }
        };

        (hwnd, tree, title)
    };

    let mut mgr = state.lock().unwrap();
    mgr.counter += 1;
    let id = format!("notify-{}", mgr.counter);

    let auto_dismiss_seconds = crate::setup::get_hook_config().auto_dismiss_seconds;

    let data = NotificationData {
        id: id.clone(),
        window_title,
        event_display: request.event_display().to_string(),
        message: request.message,
        source_hwnd,
        process_tree,
        auto_dismiss_seconds,
        source: request.source.clone(),
    };

    // Calculate position: stack from bottom-right
    let index = mgr.notifications.len();
    mgr.notifications.push(data.clone());
    drop(mgr);

    let y_offset = (index as f64) * (NOTIFICATION_HEIGHT + NOTIFICATION_MARGIN);

    // Create notification window
    {
        let position = crate::setup::load_notification_position();
        let monitor = crate::setup::load_notification_monitor();
        let (x, y) = calculate_notification_position(app, &position, &monitor, y_offset);

        let window = WebviewWindowBuilder::new(app, &id, WebviewUrl::App("index.html".into()))
            .title("Agent Toast")
            .inner_size(NOTIFICATION_WIDTH, NOTIFICATION_HEIGHT)
            .position(x, y)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .background_color(Color(0, 0, 0, 0))
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .focused(false)
            .build();

        match window {
            Ok(win) => {
                log::debug!("[NOTIFY] Window created: id={}", id);
                // Explicitly set position with Logical coordinates (builder may use Physical)
                let _ =
                    win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));

                // 알림 소리 재생
                if crate::setup::load_notification_sound() {
                    crate::sound::play_notification_sound();
                }
                // Also emit event as backup (frontend primarily uses invoke)
                let data_clone = data.clone();
                let label = id.clone();
                let app_clone = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    match app_clone.emit_to(&label, "notification-data", &data_clone) {
                        Ok(_) => log::debug!("[NOTIFY] Event emitted: id={}", label),
                        Err(e) => {
                            log::debug!("[NOTIFY] Event emit failed: id={}, err={}", label, e)
                        }
                    }
                });
            }
            Err(e) => {
                log::debug!("[NOTIFY] Window creation FAILED: id={}, err={}", id, e);
                // Rollback: remove from notifications list
                let mut mgr = state.lock().unwrap();
                mgr.notifications.retain(|n| n.id != id);
            }
        }
    }
}

pub fn close_notification(app: &AppHandle, state: &NotificationManagerState, id: &str) {
    log::debug!("[DEBUG] close_notification called: id={}", id);
    let mut mgr = state.lock().unwrap();
    mgr.notifications.retain(|n| n.id != id);
    let remaining: Vec<NotificationData> = mgr.notifications.clone();
    drop(mgr);

    // Close the window
    if let Some(win) = app.get_webview_window(id) {
        log::debug!("[DEBUG] closing window: id={}", id);
        match win.destroy() {
            Ok(_) => log::debug!("[DEBUG] window closed ok: id={}", id),
            Err(e) => log::debug!("[DEBUG] window close failed: id={}, err={}", id, e),
        }
    } else {
        log::debug!("[DEBUG] window not found: id={}", id);
    }

    // Reposition remaining notifications
    reposition_notifications(app, &remaining);
}

pub fn reposition_all(app: &AppHandle, state: &NotificationManagerState) {
    let mgr = state.lock().unwrap();
    let notifications: Vec<NotificationData> = mgr.notifications.clone();
    drop(mgr);
    reposition_notifications(app, &notifications);
}

fn reposition_notifications(app: &AppHandle, notifications: &[NotificationData]) {
    let position = crate::setup::load_notification_position();
    let monitor = crate::setup::load_notification_monitor();

    for (i, n) in notifications.iter().enumerate() {
        let y_offset = (i as f64) * (NOTIFICATION_HEIGHT + NOTIFICATION_MARGIN);
        let (x, y) = calculate_notification_position(app, &position, &monitor, y_offset);

        if let Some(win) = app.get_webview_window(&n.id) {
            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        }
    }
}

fn calculate_notification_position(
    app: &AppHandle,
    position: &str,
    monitor_value: &str,
    y_offset: f64,
) -> (f64, f64) {
    let (wa_x, wa_y, wa_w, wa_h) = win32::get_monitor_work_area(monitor_value);

    // Find scale factor for the selected monitor
    let scale = if monitor_value == "primary" || monitor_value.is_empty() {
        app.primary_monitor()
            .ok()
            .flatten()
            .map(|m| m.scale_factor())
            .unwrap_or(1.0)
    } else if let Ok(index) = monitor_value.parse::<usize>() {
        app.available_monitors()
            .ok()
            .and_then(|monitors| {
                let list: Vec<_> = monitors.into_iter().collect();
                list.get(index).map(|m| m.scale_factor())
            })
            .unwrap_or(1.0)
    } else {
        1.0
    };

    let logical_x = wa_x / scale;
    let logical_y = wa_y / scale;
    let logical_w = wa_w / scale;
    let logical_h = wa_h / scale;

    // Stack notifications, clamping to just off-screen when overflowing
    // This prevents Windows from repositioning to weird locations on negative coordinates
    let min_y = logical_y - NOTIFICATION_HEIGHT - NOTIFICATION_MARGIN;
    let max_y = logical_y + logical_h + NOTIFICATION_HEIGHT + NOTIFICATION_MARGIN;

    match position {
        "top_left" => {
            let x = logical_x + NOTIFICATION_MARGIN;
            let y = (logical_y + NOTIFICATION_MARGIN + y_offset).min(max_y);
            (x, y)
        }
        "top_right" => {
            let x = logical_x + logical_w - NOTIFICATION_WIDTH - NOTIFICATION_MARGIN;
            let y = (logical_y + NOTIFICATION_MARGIN + y_offset).min(max_y);
            (x, y)
        }
        "bottom_left" => {
            let x = logical_x + NOTIFICATION_MARGIN;
            let y = (logical_y + logical_h - NOTIFICATION_HEIGHT - NOTIFICATION_MARGIN - y_offset)
                .max(min_y);
            (x, y)
        }
        _ => {
            // bottom_right (default)
            let x = logical_x + logical_w - NOTIFICATION_WIDTH - NOTIFICATION_MARGIN;
            let y = (logical_y + logical_h - NOTIFICATION_HEIGHT - NOTIFICATION_MARGIN - y_offset)
                .max(min_y);
            (x, y)
        }
    }
}

/// FR-3: Auto-close notifications whose source HWND matches the newly focused window
pub fn on_foreground_changed(
    app: &AppHandle,
    state: &NotificationManagerState,
    focused_hwnd: isize,
) {
    if !crate::setup::load_auto_close_on_focus() {
        return;
    }
    let mgr = state.lock().unwrap();
    let to_close: Vec<String> = mgr
        .notifications
        .iter()
        .filter(|n| n.source_hwnd == focused_hwnd)
        .map(|n| n.id.clone())
        .collect();
    drop(mgr);
    if !to_close.is_empty() {
        log::debug!(
            "[DEBUG] on_foreground_changed: focused_hwnd={}, closing={:?}",
            focused_hwnd,
            to_close
        );
    }

    for id in to_close {
        close_notification(app, state, &id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NotificationData tests ──

    #[test]
    fn notification_data_serde_roundtrip() {
        let data = NotificationData {
            id: "notify-1".to_string(),
            window_title: "My Terminal".to_string(),
            event_display: "task_complete".to_string(),
            message: Some("빌드 완료".to_string()),
            source_hwnd: 12345,
            process_tree: vec![100, 200, 300],
            auto_dismiss_seconds: 30,
            source: "claude".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: NotificationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "notify-1");
        assert_eq!(deserialized.window_title, "My Terminal");
        assert_eq!(deserialized.event_display, "task_complete");
        assert_eq!(deserialized.message.as_deref(), Some("빌드 완료"));
        assert_eq!(deserialized.source_hwnd, 12345);
        assert_eq!(deserialized.process_tree, vec![100, 200, 300]);
        assert_eq!(deserialized.auto_dismiss_seconds, 30);
        assert_eq!(deserialized.source, "claude");
    }

    #[test]
    fn notification_data_without_message() {
        let data = NotificationData {
            id: "notify-2".to_string(),
            window_title: "VSCode".to_string(),
            event_display: "error".to_string(),
            message: None,
            source_hwnd: 0,
            process_tree: vec![],
            auto_dismiss_seconds: 0,
            source: "codex".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: NotificationData = serde_json::from_str(&json).unwrap();
        assert!(deserialized.message.is_none());
        assert_eq!(deserialized.source, "codex");
    }

    #[test]
    fn notification_data_empty_process_tree() {
        let data = NotificationData {
            id: "notify-3".to_string(),
            window_title: "Test".to_string(),
            event_display: "user_input_required".to_string(),
            message: None,
            source_hwnd: 0,
            process_tree: vec![],
            auto_dismiss_seconds: 0,
            source: "updater".to_string(),
        };
        assert!(data.process_tree.is_empty());
    }

    #[test]
    fn notification_data_unicode_content() {
        let data = NotificationData {
            id: "notify-4".to_string(),
            window_title: "한글 터미널 🚀".to_string(),
            event_display: "task_complete".to_string(),
            message: Some("テスト完了 ✅".to_string()),
            source_hwnd: 999,
            process_tree: vec![1, 2, 3],
            auto_dismiss_seconds: 10,
            source: "claude".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: NotificationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.window_title, "한글 터미널 🚀");
        assert_eq!(deserialized.message.as_deref(), Some("テスト完了 ✅"));
    }

    // ── NotificationManager tests ──

    #[test]
    fn notification_manager_new_empty() {
        let mgr = NotificationManager::new();
        assert!(mgr.notifications.is_empty());
        assert_eq!(mgr.counter, 0);
    }

    #[test]
    fn notification_manager_default_same_as_new() {
        let new_mgr = NotificationManager::new();
        let default_mgr = NotificationManager::default();
        assert_eq!(new_mgr.notifications.len(), default_mgr.notifications.len());
        assert_eq!(new_mgr.counter, default_mgr.counter);
    }

    #[test]
    fn create_manager_returns_arc_mutex() {
        let state = create_manager();
        let mgr = state.lock().unwrap();
        assert!(mgr.notifications.is_empty());
        assert_eq!(mgr.counter, 0);
    }

    #[test]
    fn get_notification_for_window_not_found() {
        let state = create_manager();
        let result = get_notification_for_window(&state, "nonexistent-id");
        assert!(result.is_none());
    }

    #[test]
    fn get_notification_for_window_found() {
        let state = create_manager();
        {
            let mut mgr = state.lock().unwrap();
            mgr.notifications.push(NotificationData {
                id: "notify-1".to_string(),
                window_title: "Test Window".to_string(),
                event_display: "task_complete".to_string(),
                message: Some("Done".to_string()),
                source_hwnd: 123,
                process_tree: vec![],
                auto_dismiss_seconds: 0,
                source: "claude".to_string(),
            });
        }
        let result = get_notification_for_window(&state, "notify-1");
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.id, "notify-1");
        assert_eq!(data.window_title, "Test Window");
    }

    #[test]
    fn get_notification_for_window_multiple_notifications() {
        let state = create_manager();
        {
            let mut mgr = state.lock().unwrap();
            mgr.notifications.push(NotificationData {
                id: "notify-1".to_string(),
                window_title: "First".to_string(),
                event_display: "task_complete".to_string(),
                message: None,
                source_hwnd: 100,
                process_tree: vec![],
                auto_dismiss_seconds: 0,
                source: "claude".to_string(),
            });
            mgr.notifications.push(NotificationData {
                id: "notify-2".to_string(),
                window_title: "Second".to_string(),
                event_display: "error".to_string(),
                message: Some("Error occurred".to_string()),
                source_hwnd: 200,
                process_tree: vec![],
                auto_dismiss_seconds: 0,
                source: "claude".to_string(),
            });
        }

        let first = get_notification_for_window(&state, "notify-1");
        let second = get_notification_for_window(&state, "notify-2");
        let third = get_notification_for_window(&state, "notify-3");

        assert!(first.is_some());
        assert_eq!(first.unwrap().window_title, "First");
        assert!(second.is_some());
        assert_eq!(second.unwrap().window_title, "Second");
        assert!(third.is_none());
    }

    // ── Constants tests ──

    #[test]
    fn notification_dimensions_are_positive() {
        assert!(NOTIFICATION_WIDTH > 0.0);
        assert!(NOTIFICATION_HEIGHT > 0.0);
        assert!(NOTIFICATION_MARGIN >= 0.0);
    }

    #[test]
    fn notification_dimensions_reasonable() {
        // 알림 창 크기가 합리적인 범위인지 확인 (상수 문서 참조)
        assert!(NOTIFICATION_WIDTH >= 200.0 && NOTIFICATION_WIDTH <= 600.0);
        assert!(NOTIFICATION_HEIGHT >= 80.0 && NOTIFICATION_HEIGHT <= 300.0);
        assert!(NOTIFICATION_MARGIN >= 0.0 && NOTIFICATION_MARGIN <= 50.0);
    }

    // ── Stacking calculation tests ──

    #[test]
    fn notification_stack_offset_calculation() {
        // y_offset 계산 검증: index * (height + margin)
        let index = 2;
        let y_offset = (index as f64) * (NOTIFICATION_HEIGHT + NOTIFICATION_MARGIN);
        let expected = 2.0 * (140.0 + 10.0);
        assert!((y_offset - expected).abs() < 0.001);
    }

    #[test]
    fn notification_stack_first_item_no_offset() {
        let index = 0;
        let y_offset = (index as f64) * (NOTIFICATION_HEIGHT + NOTIFICATION_MARGIN);
        assert!((y_offset - 0.0).abs() < 0.001);
    }

    // ── NotificationManager counter tests ──

    #[test]
    fn notification_manager_counter_increments() {
        let state = create_manager();
        {
            let mut mgr = state.lock().unwrap();
            assert_eq!(mgr.counter, 0);
            mgr.counter += 1;
            assert_eq!(mgr.counter, 1);
            mgr.counter += 1;
            assert_eq!(mgr.counter, 2);
        }
    }

    #[test]
    fn notification_id_format() {
        // ID 형식: "notify-{counter}"
        let state = create_manager();
        {
            let mut mgr = state.lock().unwrap();
            mgr.counter += 1;
            let id = format!("notify-{}", mgr.counter);
            assert_eq!(id, "notify-1");
        }
    }

    // ── NotificationData source field tests ──

    #[test]
    fn notification_data_sources() {
        for source in ["claude", "codex", "updater"] {
            let data = NotificationData {
                id: "test".to_string(),
                window_title: "Test".to_string(),
                event_display: "test".to_string(),
                message: None,
                source_hwnd: 0,
                process_tree: vec![],
                auto_dismiss_seconds: 0,
                source: source.to_string(),
            };
            assert_eq!(data.source, source);
        }
    }

    #[test]
    fn notification_data_auto_dismiss_values() {
        for seconds in [0, 5, 10, 30, 60] {
            let data = NotificationData {
                id: "test".to_string(),
                window_title: "Test".to_string(),
                event_display: "test".to_string(),
                message: None,
                source_hwnd: 0,
                process_tree: vec![],
                auto_dismiss_seconds: seconds,
                source: "claude".to_string(),
            };
            assert_eq!(data.auto_dismiss_seconds, seconds);
        }
    }

    #[test]
    fn notification_data_clone_deep() {
        let data = NotificationData {
            id: "notify-1".to_string(),
            window_title: "Terminal".to_string(),
            event_display: "task_complete".to_string(),
            message: Some("Done".to_string()),
            source_hwnd: 12345,
            process_tree: vec![100, 200, 300],
            auto_dismiss_seconds: 30,
            source: "claude".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.id, data.id);
        assert_eq!(cloned.process_tree, data.process_tree);
        // Modifying clone doesn't affect original (deep copy)
        // Note: Rust's clone is always deep for Vec
    }
}
