pub mod cli;
mod notification;
pub mod pipe;
pub mod setup;
pub mod sound;
mod updater;
pub mod win32;

use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;

use cli::NotifyRequest;
use notification::{
    close_notification, get_notification_for_window, on_foreground_changed, show_notification,
    NotificationData, NotificationManagerState,
};

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Holds tray menu items so we can update their text at runtime.
pub struct TrayMenuState {
    pub settings_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
}

/// Update tray menu text to match the current locale.
pub fn update_tray_locale(app: &AppHandle) {
    let locale = setup::read_locale();
    let (label_settings, label_quit) = match locale.as_str() {
        "en" => ("Settings", "Quit"),
        _ => ("설정", "종료"),
    };
    if let Some(state) = app.try_state::<TrayMenuState>() {
        let _ = state.settings_item.set_text(label_settings);
        let _ = state.quit_item.set_text(label_quit);
    }
}

#[tauri::command]
fn get_locale() -> String {
    setup::read_locale()
}

#[tauri::command]
fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn get_monitor_list() -> Vec<win32::MonitorInfo> {
    win32::get_monitor_list()
}

#[tauri::command]
fn get_notification_data(window: WebviewWindow) -> Option<NotificationData> {
    let state = window.app_handle().state::<NotificationManagerState>();
    get_notification_for_window(&state, window.label())
}

#[tauri::command]
fn close_notify(id: String, app: AppHandle) {
    let state = app.state::<NotificationManagerState>();
    close_notification(&app, &state, &id);
}

#[tauri::command]
fn activate_source(hwnd: isize, id: String, app: AppHandle) {
    log::debug!("activate_source called: hwnd={}, id={}", hwnd, id);
    win32::activate_window(hwnd);
    let state = app.state::<NotificationManagerState>();
    close_notification(&app, &state, &id);
}

#[tauri::command]
fn test_notification(app: AppHandle) {
    log::debug!("[TEST] test_notification command called");
    let state = app.state::<NotificationManagerState>().inner().clone();
    let locale = setup::read_locale();

    // Pick random event type using current time
    let events = ["task_complete", "user_input_required", "error"];
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let event = events[(nanos as usize) % events.len()];

    let (test_msg, test_title) = match locale.as_str() {
        "en" => ("This is a test notification", "Test"),
        _ => ("테스트 알림입니다", "테스트"),
    };
    let req = NotifyRequest {
        pid: 0,
        event: event.to_string(),
        message: Some(test_msg.to_string()),
        title_hint: Some(test_title.to_string()),
        process_tree: Some(vec![]),
        source: "claude".into(),
    };
    log::debug!("[TEST] Spawning notification thread for event={}", event);
    std::thread::spawn(move || {
        log::debug!("[TEST] Thread started, calling show_notification");
        show_notification(&app, &state, req);
        log::debug!("[TEST] show_notification returned");
    });
}

#[tauri::command]
async fn open_settings(app: AppHandle, tab: Option<String>) {
    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        open_setup_window_with_tab(&app_clone, tab.as_deref());
    });
}

pub fn open_setup_window(app: &AppHandle) {
    open_setup_window_with_tab(app, None);
}

pub fn open_setup_window_with_tab(app: &AppHandle, tab: Option<&str>) {
    // If setup window already exists, focus it (and optionally navigate to tab)
    if let Some(win) = app.get_webview_window("setup") {
        if let Some(t) = tab {
            let _ = win.eval(format!("window.location.hash = '{}';", t));
        }
        let _ = win.set_focus();
        return;
    }

    let locale = setup::read_locale();
    let setup_title = match locale.as_str() {
        "en" => "Agent Toast Settings",
        _ => "Agent Toast 설정",
    };

    let url = match tab {
        Some(t) => format!("index.html#{}", t),
        None => "index.html".to_string(),
    };

    let _ = WebviewWindowBuilder::new(app, "setup", WebviewUrl::App(url.into()))
        .title(setup_title)
        .inner_size(560.0, 720.0)
        .resizable(true)
        .center()
        .build();
}

pub fn run_app(initial_request: Option<NotifyRequest>, open_setup: bool) {
    // Initialize logging to temp file + terminal
    let log_path = std::env::temp_dir().join("agent-toast.log");

    // Keep only the last half (by line count) if the log exceeds 1 MB
    const MAX_LOG_SIZE: u64 = 1024 * 1024;
    if let Ok(meta) = std::fs::metadata(&log_path) {
        if meta.len() > MAX_LOG_SIZE {
            if let Ok(content) = std::fs::read_to_string(&log_path) {
                let lines: Vec<&str> = content.lines().collect();
                let kept = lines[lines.len() / 2..].join("\n");
                let _ = std::fs::write(&log_path, kept + "\n");
            }
        }
    }

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )];
    if let Some(file) = log_file {
        loggers.push(WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            file,
        ));
    }
    let _ = CombinedLogger::init(loggers);

    log::info!("=== Agent Toast Started === (log: {})", log_path.display());

    let mgr_state = notification::create_manager();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(mgr_state.clone())
        .invoke_handler(tauri::generate_handler![
            close_notify,
            activate_source,
            get_notification_data,
            test_notification,
            get_locale,
            is_dev_mode,
            open_settings,
            setup::get_hook_config,
            setup::save_hook_config,
            setup::get_exe_path,
            setup::get_saved_exe_path,
            setup::open_settings_file,
            setup::is_hook_config_saved,
            setup::get_codex_installed,
            get_monitor_list,
            updater::mark_update_pending
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = mgr_state.clone();

            // System tray
            let tray_handle = handle.clone();
            let locale = setup::read_locale();
            let (label_settings, label_quit) = match locale.as_str() {
                "en" => ("Settings", "Quit"),
                _ => ("설정", "종료"),
            };
            let settings_item = MenuItemBuilder::with_id("settings", label_settings).build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", label_quit).build(app)?;
            app.manage(TrayMenuState {
                settings_item: settings_item.clone(),
                quit_item: quit_item.clone(),
            });
            let menu = MenuBuilder::new(app)
                .item(&settings_item)
                .item(&quit_item)
                .build()?;
            // TODO: Tauri ICO 파싱 버그로 인해 트레이 아이콘 별도 로드 필요
            // - Tauri가 ICO의 첫 번째 엔트리만 사용 (entries()[0])
            // - icon.ico는 작업표시줄용 (큰 사이즈 먼저), tray.ico는 트레이용 (작은 사이즈)
            // - 제목표시줄도 icon.ico 사용해서 해상도 깨짐 (Tauri 수정 필요)
            // - 관련 이슈: https://github.com/tauri-apps/tauri/issues/14596
            let tray_icon_bytes = include_bytes!("../icons/tray.ico");
            let tray_icon = Image::from_bytes(tray_icon_bytes).expect("failed to load tray icon");
            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Agent Toast")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "settings" => open_setup_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(&tray_handle)?;

            // Start Named Pipe server for subsequent calls
            let pipe_handle = handle.clone();
            let pipe_state = state.clone();
            pipe::start_server(move |req| {
                show_notification(&pipe_handle, &pipe_state, req);
            });

            // FR-3: Event-based foreground change detection via SetWinEventHook
            let focus_handle = handle.clone();
            let focus_state = state.clone();
            win32::start_foreground_listener(move |hwnd| {
                on_foreground_changed(&focus_handle, &focus_state, hwnd);
            });

            // Open setup window if requested
            if open_setup {
                open_setup_window(&handle);
            }

            // Show initial notification if provided
            if let Some(req) = initial_request {
                let init_handle = handle.clone();
                let init_state = state.clone();
                // Delay slightly to ensure app is ready
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    show_notification(&init_handle, &init_state, req);
                });
            }

            // Check if update was just completed
            updater::check_update_completed(&handle, &state);

            // Check for updates in background
            updater::check_for_updates(&handle, &state);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Prevent app from exiting when all windows are closed (daemon mode)
            if let RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
