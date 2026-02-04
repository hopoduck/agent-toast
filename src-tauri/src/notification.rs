use crate::cli::NotifyRequest;
use crate::win32;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const NOTIFICATION_WIDTH: f64 = 380.0;
const NOTIFICATION_HEIGHT: f64 = 140.0;
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
    // For internal notifications (updater), skip win32 lookups
    let is_internal = request.source == "updater";

    let (source_hwnd, process_tree, window_title) = if is_internal {
        (
            0isize,
            vec![],
            request.title_hint.clone().unwrap_or_else(|| "Agent Toast".to_string()),
        )
    } else {
        let tree = request
            .process_tree
            .clone()
            .filter(|t| t.len() > 1)
            .unwrap_or_else(|| win32::get_process_tree(request.pid));

        let (all_candidates, found) =
            win32::find_source_window(&tree, request.title_hint.as_deref());
        eprintln!(
            "[DEBUG] event={}, title_hint={:?}, process_tree={:?}, find_source_window={:?}",
            request.event, request.title_hint, tree, found
        );
        for (h, p) in &all_candidates {
            let title = win32::get_window_title(*h);
            eprintln!("[DEBUG]   candidate hwnd={} pid={} title={:?}", h, p, title);
        }
        let (hwnd, _) = found.unwrap_or((0, 0));

        // FR-2: Skip if source window is already focused (compare by HWND, not PID)
        let focused = win32::is_hwnd_focused(hwnd);
        eprintln!("[DEBUG] is_hwnd_focused({})={}", hwnd, focused);
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

        if let Ok(win) = window {
            // Explicitly set position with Logical coordinates (builder may use Physical)
            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));

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
                let _ = app_clone.emit_to(&label, "notification-data", &data_clone);
            });
        }
    }
}

pub fn close_notification(app: &AppHandle, state: &NotificationManagerState, id: &str) {
    eprintln!("[DEBUG] close_notification called: id={}", id);
    let mut mgr = state.lock().unwrap();
    mgr.notifications.retain(|n| n.id != id);
    let remaining: Vec<NotificationData> = mgr.notifications.clone();
    drop(mgr);

    // Close the window
    if let Some(win) = app.get_webview_window(id) {
        eprintln!("[DEBUG] closing window: id={}", id);
        match win.destroy() {
            Ok(_) => eprintln!("[DEBUG] window closed ok: id={}", id),
            Err(e) => eprintln!("[DEBUG] window close failed: id={}, err={}", id, e),
        }
    } else {
        eprintln!("[DEBUG] window not found: id={}", id);
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
        eprintln!(
            "[DEBUG] on_foreground_changed: focused_hwnd={}, closing={:?}",
            focused_hwnd, to_close
        );
    }

    for id in to_close {
        close_notification(app, state, &id);
    }
}
