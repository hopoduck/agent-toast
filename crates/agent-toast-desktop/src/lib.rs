mod changelog;
pub mod cli;
mod fonts;
mod global_sync;
pub mod http_server;
mod notification;
pub mod pipe;
pub mod setup;
pub mod sound;
pub mod stats;
mod updater;
pub mod win32;

use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};

use cli::NotifyRequest;
use notification::{
    close_notification, get_notification_for_window, on_foreground_changed, show_notification,
    NotificationData, NotificationManagerState,
};

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Holds the running HTTP server handle so we can start/stop it at runtime
/// in response to settings changes.
pub struct HttpServerState(pub Arc<Mutex<Option<http_server::HttpHandle>>>);

/// Synchronize the HTTP server with the current `http_enabled` / `http_bind_addr`
/// settings. Called at boot and after `save_hook_config`. Returns an error if
/// the user wants the server enabled but binding fails — caller surfaces this
/// to the UI.
pub fn sync_http_server(app: &AppHandle) -> Result<(), String> {
    let http_state = app.state::<HttpServerState>();
    let mut guard = http_state.0.lock().unwrap();

    let want_enabled = setup::read_http_enabled();
    let want_addr = format!("0.0.0.0:{}", setup::read_http_port());
    let current_addr = guard.as_ref().map(|h| h.addr().to_string());

    let need_stop = match &current_addr {
        Some(addr) => !want_enabled || *addr != want_addr,
        None => false,
    };
    let need_start = want_enabled && (current_addr.is_none() || need_stop);

    if need_stop {
        if let Some(h) = guard.take() {
            h.stop();
        }
        // Give the recv_timeout loop a moment to release the socket before rebind.
        if need_start {
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
    }

    if need_start {
        let handle = app.clone();
        let mgr_state = app.state::<NotificationManagerState>().inner().clone();
        let new_handle = http_server::start_server(&want_addr, move |req| {
            show_notification(&handle, &mgr_state, req);
        })?;
        *guard = Some(new_handle);
        log::info!("[HTTP] started on {}", want_addr);
    } else if need_stop {
        log::info!("[HTTP] stopped");
    }

    Ok(())
}

/// Holds tray menu items so we can update their text at runtime.
pub struct TrayMenuState {
    pub settings_item: MenuItem<tauri::Wry>,
    pub restart_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
}

/// Update tray menu text to match the current locale.
pub fn update_tray_locale(app: &AppHandle) {
    let locale = setup::read_locale();
    let (label_settings, label_restart, label_quit) = match locale.as_str() {
        "en" => ("Settings", "Restart", "Quit"),
        _ => ("설정", "재시작", "종료"),
    };
    if let Some(state) = app.try_state::<TrayMenuState>() {
        let _ = state.settings_item.set_text(label_settings);
        let _ = state.restart_item.set_text(label_restart);
        let _ = state.quit_item.set_text(label_quit);
    }
}

#[tauri::command]
fn get_locale() -> String {
    setup::read_locale()
}

#[tauri::command]
fn get_theme() -> String {
    setup::read_theme()
}

#[tauri::command]
fn set_theme(theme: String) -> Result<(), String> {
    setup::write_theme(&theme)
}

#[tauri::command]
fn get_stats(app: AppHandle) -> stats::Stats {
    let state = app.state::<stats::StatsState>();
    let m = state.lock().unwrap();
    m.stats.clone()
}

#[tauri::command]
fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn is_portable() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return true;
    };
    let Some(dir) = exe.parent() else {
        return true;
    };
    !dir.join("uninstall.exe").exists()
}

#[tauri::command]
fn get_monitor_list() -> Vec<win32::MonitorInfo> {
    win32::get_monitor_list()
}

/// Detect this machine's Tailscale MagicDNS short hostname (e.g. `mypc`).
/// Returns `None` if Tailscale is not installed, not logged in, or the lookup fails.
#[tauri::command]
fn get_tailscale_hostname() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let candidates = [
            "tailscale",
            r"C:\Program Files\Tailscale\tailscale.exe",
            r"C:\Program Files (x86)\Tailscale\tailscale.exe",
        ];

        let output = candidates.iter().find_map(|path| {
            Command::new(path)
                .args(["status", "--json", "--self"])
                .output()
                .ok()
                .filter(|o| o.status.success())
        })?;

        let v: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
        // `DNSName` is the MagicDNS FQDN (e.g. `mypc.tailXXXX.ts.net.`); its
        // first label is the Tailscale node name, which differs from the OS
        // hostname stored in `HostName`.
        let dns = v.get("Self")?.get("DNSName")?.as_str()?;
        let short = dns.split('.').next()?.trim();
        if short.is_empty() {
            None
        } else {
            Some(short.to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[tauri::command]
fn get_notification_data(window: WebviewWindow) -> Option<NotificationData> {
    let state = window.app_handle().state::<NotificationManagerState>();
    get_notification_for_window(&state, window.label())
}

#[tauri::command]
fn close_notify(id: String, reason: Option<String>, app: AppHandle) {
    let close_reason = match reason.as_deref() {
        Some("timeout") => stats::CloseReason::Timeout,
        _ => stats::CloseReason::Manual,
    };
    let state = app.state::<NotificationManagerState>();
    close_notification(&app, &state, &id, close_reason);
}

#[tauri::command]
fn resize_notify(id: String, height: f64, app: AppHandle) {
    let state = app.state::<NotificationManagerState>();
    notification::resize_notification(&app, &state, &id, height);
}

#[tauri::command]
fn activate_source(hwnd: isize, id: String, app: AppHandle) {
    log::debug!("activate_source called: hwnd={}, id={}", hwnd, id);
    let state = app.state::<NotificationManagerState>();
    // 활성화보다 먼저 Activated로 닫아 알림을 매니저에서 제거한다. activate_window가
    // 소스 창을 포그라운드로 올리면 EVENT_SYSTEM_FOREGROUND가 발생하고, 포그라운드
    // 리스너 스레드가 이 토스트를 Focus 사유로 먼저 닫아버리는 경쟁이 생기는데
    // (그러면 "보기" 클릭이 closed_focus로 잘못 집계됨), 먼저 제거하면 리스너가
    // 일치하는 알림을 못 찾아 경쟁이 사라진다.
    close_notification(&app, &state, &id, stats::CloseReason::Activated);
    if hwnd != 0 {
        win32::activate_window(hwnd);
    } else {
        log::debug!("[ACTIVATE] hwnd=0, skipping window activation (likely remote)");
    }
}

#[tauri::command]
fn test_notification(app: AppHandle, title: Option<String>, message: Option<String>) {
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
        message: Some(message.unwrap_or_else(|| test_msg.to_string())),
        title_hint: Some(title.unwrap_or_else(|| test_title.to_string())),
        process_tree: Some(vec![]),
        source: "claude".into(),
        hostname: None,
    };
    log::debug!("[TEST] Spawning notification thread for event={}", event);
    std::thread::spawn(move || {
        log::debug!("[TEST] Thread started, calling show_notification");
        show_notification(&app, &state, req);
        log::debug!("[TEST] show_notification returned");
    });
}

#[tauri::command]
fn preview_notification_sound(app: AppHandle, path: Option<String>) {
    use tauri::Emitter;
    // 저장 전 선택도 미리 들을 수 있게 경로를 직접 받는다 (None = 시스템 기본음)
    // 재생이 끝까지 가면 setup 창의 토글 버튼을 ▶로 되돌리는 이벤트를 보낸다
    sound::preview(path.as_deref(), move || {
        let _ = app.emit_to("setup", "sound-preview-ended", ());
    });
}

#[tauri::command]
fn stop_notification_sound() {
    sound::stop_playback();
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
    let setup_title_base = match locale.as_str() {
        "en" => "Agent Toast Settings",
        _ => "Agent Toast 설정",
    };
    // 개발모드에서는 작업표시줄에서 구분할 수 있도록 [DEV] 표식을 붙인다.
    let setup_title = if is_dev_mode() {
        format!("{setup_title_base} [DEV]")
    } else {
        setup_title_base.to_string()
    };

    let url = match tab {
        Some(t) => format!("index.html#{}", t),
        None => "index.html".to_string(),
    };

    let mut builder = WebviewWindowBuilder::new(app, "setup", WebviewUrl::App(url.into()))
        .title(setup_title)
        .inner_size(560.0, 720.0)
        .resizable(true)
        .center();
    // 개발모드에서는 작업표시줄 아이콘도 DEV 배지 버전으로 교체
    if is_dev_mode() {
        if let Ok(icon) = Image::from_bytes(include_bytes!("../icons/icon-dev.ico")) {
            builder = builder.icon(icon).expect("failed to set dev window icon");
        }
    }
    let _ = builder.build();
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

    let mut log_builder = ConfigBuilder::new();
    log_builder.set_time_format_rfc3339();
    // Use local time if available, otherwise fall back to UTC
    let _ = log_builder.set_time_offset_to_local();
    let log_config = log_builder.build();

    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Debug,
        log_config.clone(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )];
    if let Some(file) = log_file {
        loggers.push(WriteLogger::new(LevelFilter::Debug, log_config, file));
    }
    let _ = CombinedLogger::init(loggers);

    // Capture panics from any thread into the log file before aborting
    let panic_log_path = log_path.clone();
    std::panic::set_hook(Box::new(move |info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let bt = std::backtrace::Backtrace::force_capture();
        let msg = format!(
            "[PANIC] {payload}\n  at {location}\n  thread: {:?}\n{bt}",
            std::thread::current().name().unwrap_or("unnamed")
        );
        log::error!("{msg}");
        // Also write directly to file in case the logger is broken
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open(&panic_log_path)
        {
            use std::io::Write;
            let _ = writeln!(f, "{msg}");
        }
    }));

    log::info!("=== Agent Toast Started === (log: {})", log_path.display());

    let mgr_state = notification::create_manager();
    let http_state = HttpServerState(Arc::new(Mutex::new(None)));
    let stats_state = stats::create_manager();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(mgr_state.clone())
        .manage(http_state)
        .manage(stats_state.clone())
        .on_window_event(|window, event| {
            // 설정 창이 닫히면 미리듣기 재생도 정지
            if window.label() == "setup" && matches!(event, tauri::WindowEvent::Destroyed) {
                sound::stop_playback();
            }
        })
        .invoke_handler(tauri::generate_handler![
            close_notify,
            resize_notify,
            activate_source,
            get_notification_data,
            test_notification,
            preview_notification_sound,
            stop_notification_sound,
            get_locale,
            get_theme,
            set_theme,
            get_stats,
            is_dev_mode,
            is_portable,
            open_settings,
            setup::get_hook_config,
            setup::save_hook_config,
            setup::get_exe_path,
            setup::get_saved_exe_path,
            setup::open_settings_file,
            setup::is_hook_config_saved,
            setup::get_toast_style,
            setup::copy_notification_sound_file,
            fonts::list_system_fonts,
            get_monitor_list,
            get_tailscale_hostname,
            updater::mark_update_pending,
            updater::snooze_update,
            changelog::get_releases,
            global_sync::get_global_stats,
            setup::get_global_stats_enabled,
            setup::set_global_stats_enabled
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = mgr_state.clone();

            // One-shot migration: append --dynamic to pre-existing agent-toast
            // hooks for users who never stored a dynamic_message_enabled choice
            // (idempotent — records the key, so subsequent starts no-op).
            setup::run_dynamic_message_migration();

            // System tray
            let tray_handle = handle.clone();
            let locale = setup::read_locale();
            let (label_settings, label_restart, label_quit) = match locale.as_str() {
                "en" => ("Settings", "Restart", "Quit"),
                _ => ("설정", "재시작", "종료"),
            };
            let settings_item = MenuItemBuilder::with_id("settings", label_settings).build(app)?;
            let restart_item = MenuItemBuilder::with_id("restart", label_restart).build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", label_quit).build(app)?;
            app.manage(TrayMenuState {
                settings_item: settings_item.clone(),
                restart_item: restart_item.clone(),
                quit_item: quit_item.clone(),
            });
            let menu = MenuBuilder::new(app)
                .item(&settings_item)
                .separator()
                .item(&restart_item)
                .item(&quit_item)
                .build()?;
            // TODO: Tauri ICO 파싱 버그로 인해 트레이 아이콘 별도 로드 필요
            // - Tauri가 ICO의 첫 번째 엔트리만 사용 (entries()[0])
            // - icon.ico는 작업표시줄용 (큰 사이즈 먼저), tray.ico는 트레이용 (작은 사이즈)
            // - 제목표시줄도 icon.ico 사용해서 해상도 깨짐 (Tauri 수정 필요)
            // - 관련 이슈: https://github.com/tauri-apps/tauri/issues/14596
            // 개발모드에서는 작업표시줄/트레이에서 한눈에 구분되도록 DEV 배지 아이콘 사용
            let tray_icon_bytes: &[u8] = if is_dev_mode() {
                include_bytes!("../icons/tray-dev.ico")
            } else {
                include_bytes!("../icons/tray.ico")
            };
            let tray_icon = Image::from_bytes(tray_icon_bytes).expect("failed to load tray icon");
            let click_handle = tray_handle.clone();
            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip(if is_dev_mode() {
                    "Agent Toast [DEV]"
                } else {
                    "Agent Toast"
                })
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "settings" => open_setup_window(app),
                    "restart" => {
                        stats::flush(&app.state::<stats::StatsState>().inner().clone());
                        app.restart()
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        open_setup_window(&click_handle);
                    }
                })
                .build(&tray_handle)?;

            // Start Named Pipe server for subsequent calls
            let pipe_handle = handle.clone();
            let pipe_state = state.clone();
            pipe::start_server(move |req| {
                show_notification(&pipe_handle, &pipe_state, req);
            });

            // Start HTTP receiver if enabled in settings
            if let Err(e) = sync_http_server(&handle) {
                log::error!("[HTTP] initial start failed: {e}");
            }

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

            // Check for updates in background (once at startup)
            updater::check_for_updates(&handle, &state);

            // 글로벌 통계: 시작 시 1회 무조건 업로드 (dev 빌드/설정 off면 내부에서 no-op)
            global_sync::sync(&handle.state::<stats::StatsState>().inner().clone(), true);

            // Re-check periodically so a long-running app still notices new releases
            {
                let timer_handle = handle.clone();
                let timer_state = state.clone();
                let timer_stats = handle.state::<stats::StatsState>().inner().clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(60 * 60));
                    updater::check_for_updates(&timer_handle, &timer_state);
                    global_sync::sync(&timer_stats, false);
                });
            }

            // Debounced stats flush: persist at most every 5s if changed.
            {
                let flush_state = handle.state::<stats::StatsState>().inner().clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    stats::flush(&flush_state);
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            RunEvent::ExitRequested { api, code, .. } => {
                log::warn!("[EXIT] ExitRequested: code={:?}", code);
                if code.is_none() {
                    api.prevent_exit();
                } else {
                    log::error!("[EXIT] App exiting with code={:?}", code);
                }
            }
            RunEvent::Exit => {
                log::warn!("[EXIT] App is shutting down (RunEvent::Exit)");
                stats::flush(&app.state::<stats::StatsState>().inner().clone());
            }
            _ => {}
        });
}
