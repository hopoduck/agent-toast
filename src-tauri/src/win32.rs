use log::debug;
use std::sync::{Arc, Mutex};

/// (hwnd, pid) pair identifying a window candidate.
pub type WindowCandidate = (isize, u32);

#[cfg(windows)]
use windows::core::BOOL;
use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::Foundation::RECT;
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::System::Console::{AttachConsole, FreeConsole, GetConsoleWindow};
#[cfg(windows)]
use windows::Win32::UI::Accessibility::SetWinEventHook;
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EnumWindows, GetForegroundWindow, GetMessageW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow, ShowWindow,
    SystemParametersInfoW, EVENT_SYSTEM_FOREGROUND, MSG, SPI_GETWORKAREA, SW_RESTORE,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

/// Walk up the process tree from `start_pid`, collecting all ancestor PIDs.
/// Returns vec including start_pid itself.
#[cfg(windows)]
pub fn get_process_tree(start_pid: u32) -> Vec<u32> {
    let mut tree = vec![start_pid];
    let mut current = start_pid;

    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return tree;
    };

    const BLOCKED_EXES: &[&str] = &[
        "explorer.exe",
        "winlogon.exe",
        "csrss.exe",
        "services.exe",
        "svchost.exe",
    ];

    // Build a pid -> (parent_pid, exe_name) map
    let mut entries: Vec<(u32, u32, String)> = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    unsafe {
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            let exe = String::from_utf16_lossy(
                &entry.szExeFile[..entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len())],
            );
            entries.push((entry.th32ProcessID, entry.th32ParentProcessID, exe));
            while Process32NextW(snapshot, &mut entry).is_ok() {
                let exe = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );
                entries.push((entry.th32ProcessID, entry.th32ParentProcessID, exe));
            }
        }
    }

    // Walk up parent chain, stop at blocked shell/system processes
    for _ in 0..20 {
        if let Some((_, parent, _)) = entries.iter().find(|(pid, _, _)| *pid == current) {
            if *parent == 0 || *parent == current {
                break;
            }
            // Check if parent is a blocked process
            if let Some((_, _, ref parent_exe)) = entries.iter().find(|(pid, _, _)| *pid == *parent)
            {
                let parent_lower = parent_exe.to_lowercase();
                if BLOCKED_EXES.iter().any(|b| parent_lower == *b) {
                    debug!(
                        "get_process_tree: stopped at blocked process '{}' (pid={})",
                        parent_exe, parent
                    );
                    break;
                }
            }
            tree.push(*parent);
            current = *parent;
        } else {
            break;
        }
    }

    debug!("get_process_tree: start_pid={}, tree={:?}", start_pid, tree);

    tree
}

/// Find the best visible window owned by any PID in the process tree.
/// If title_hint is provided, prefer windows whose title contains it.
/// Otherwise prefer PIDs closer to the start PID (child-first).
/// Returns (all_candidates, best_match).
#[cfg(windows)]
pub fn find_source_window(
    process_tree: &[u32],
    title_hint: Option<&str>,
) -> (Vec<WindowCandidate>, Option<WindowCandidate>) {
    let candidates: Arc<Mutex<Vec<(isize, u32)>>> = Arc::new(Mutex::new(Vec::new()));
    let tree: Vec<u32> = process_tree.to_vec();
    let candidates_clone = candidates.clone();

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&(tree.clone(), candidates_clone) as *const _ as isize),
        );
    }

    let candidates = candidates.lock().unwrap();
    // If title_hint provided, prefer matching title first
    let best = if let Some(hint) = title_hint {
        let hint_lower = hint.to_lowercase();
        candidates
            .iter()
            .find(|(h, _)| get_window_title(*h).to_lowercase().contains(&hint_lower))
            .copied()
    } else {
        None
    }
    // Fallback: pick by closest PID in tree (child-first)
    .or_else(|| {
        candidates
            .iter()
            .min_by_key(|(_, pid)| {
                process_tree
                    .iter()
                    .position(|p| p == pid)
                    .unwrap_or(usize::MAX)
            })
            .copied()
    });
    (candidates.clone(), best)
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &*(lparam.0 as *const (Vec<u32>, Arc<Mutex<Vec<(isize, u32)>>>));
    let (tree, candidates) = data;

    if IsWindowVisible(hwnd).as_bool() {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if tree.contains(&pid) {
            let mut lock = candidates.lock().unwrap();
            lock.push((hwnd.0 as isize, pid));
        }
    }
    BOOL(1) // continue all
}

/// Get window title text
#[cfg(windows)]
pub fn get_window_title(hwnd: isize) -> String {
    let hwnd = HWND(hwnd as *mut _);
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..len])
}

/// Get the PID that owns a given HWND. Returns 0 if invalid.
#[cfg(windows)]
pub fn get_window_pid(hwnd: isize) -> u32 {
    if hwnd == 0 {
        return 0;
    }
    let hwnd = HWND(hwnd as *mut _);
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}

#[cfg(not(windows))]
pub fn get_window_pid(_hwnd: isize) -> u32 {
    0
}

/// Find the actual terminal window when the source window is a hidden conhost.
/// On Windows 11 with Windows Terminal as default terminal, the shell process tree
/// does NOT include WindowsTerminal.exe (they communicate via ConPTY, not parent-child).
///
/// Strategy:
/// 1. Try AttachConsole + GetConsoleWindow (works for legacy conhost)
/// 2. Fall back to scanning for WindowsTerminal.exe visible windows
///    (EnumWindows returns z-order, so the topmost/most-recent WT window comes first)
#[cfg(windows)]
pub fn find_console_window(pid: u32, exclude_hwnd: isize) -> Option<isize> {
    if pid == 0 {
        return None;
    }

    // Strategy 1: Console API
    unsafe {
        if AttachConsole(pid).is_ok() {
            let console_hwnd = GetConsoleWindow();
            let _ = FreeConsole();
            if !console_hwnd.0.is_null() {
                let hwnd_val = console_hwnd.0 as isize;
                if hwnd_val != exclude_hwnd && IsWindowVisible(console_hwnd).as_bool() {
                    let title = get_window_title(hwnd_val);
                    debug!(
                        "find_console_window: Console API found hwnd={}, title={:?}",
                        hwnd_val, title
                    );
                    return Some(hwnd_val);
                }
            }
        }
    }

    // Strategy 2: Find WindowsTerminal.exe windows directly
    find_windows_terminal_window()
}

/// Scan for visible WindowsTerminal.exe windows.
/// Returns the first (topmost in z-order) visible window with a non-empty title.
#[cfg(windows)]
fn find_windows_terminal_window() -> Option<isize> {
    // Step 1: Find all WindowsTerminal.exe PIDs
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return None;
    };

    let mut wt_pids: Vec<u32> = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    unsafe {
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let exe = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );
                if exe.eq_ignore_ascii_case("WindowsTerminal.exe") {
                    wt_pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }

    if wt_pids.is_empty() {
        debug!("find_windows_terminal_window: no WindowsTerminal.exe processes found");
        return None;
    }

    debug!(
        "find_windows_terminal_window: found WindowsTerminal.exe PIDs: {:?}",
        wt_pids
    );

    // Step 2: Find visible windows belonging to those PIDs
    // EnumWindows returns windows in z-order (topmost first)
    let result: Arc<Mutex<Option<isize>>> = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    unsafe {
        let _ = EnumWindows(
            Some(enum_wt_windows_callback),
            LPARAM(&(wt_pids, result_clone) as *const _ as isize),
        );
    }

    let found = result.lock().unwrap().take();
    if let Some(hwnd) = found {
        let title = get_window_title(hwnd);
        debug!(
            "find_windows_terminal_window: found hwnd={}, title={:?}",
            hwnd, title
        );
    }
    found
}

#[cfg(windows)]
unsafe extern "system" fn enum_wt_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &*(lparam.0 as *const (Vec<u32>, Arc<Mutex<Option<isize>>>));
    let (wt_pids, result) = data;

    if IsWindowVisible(hwnd).as_bool() {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if wt_pids.contains(&pid) {
            // Check non-empty title (skip hidden/helper windows)
            let title = get_window_title(hwnd.0 as isize);
            if !title.is_empty() {
                let mut lock = result.lock().unwrap();
                if lock.is_none() {
                    *lock = Some(hwnd.0 as isize);
                    return BOOL(0); // Stop enumeration — first (topmost) match is best
                }
            }
        }
    }
    BOOL(1) // Continue
}

#[cfg(not(windows))]
pub fn find_console_window(_pid: u32, _exclude_hwnd: isize) -> Option<isize> {
    None
}

/// Check if the given HWND is currently the foreground window.
#[cfg(windows)]
pub fn is_hwnd_focused(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    let fg = unsafe { GetForegroundWindow() };
    if fg.0.is_null() {
        return false;
    }
    fg.0 as isize == hwnd
}

/// Bring the source window to foreground.
#[cfg(windows)]
pub fn activate_window(hwnd: isize) {
    debug!("activate_window: hwnd={}", hwnd);
    if hwnd == 0 {
        debug!("activate_window: hwnd is 0, skipping");
        return;
    }
    let hwnd = HWND(hwnd as *mut _);
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
            return;
        }
        // Simulate Alt key press/release to bypass focus stealing prevention
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_MENU,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_MENU,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        // Only restore if minimized, to preserve maximized state
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Start foreground change listener using SetWinEventHook.
/// Calls `on_foreground_change(hwnd)` whenever the foreground window changes.
/// Must be called from a dedicated thread (spawns its own message loop).
#[cfg(windows)]
pub fn start_foreground_listener(on_foreground_change: impl Fn(isize) + Send + 'static) {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel::<isize>();

    // Thread 1: message loop with SetWinEventHook
    std::thread::spawn(move || {
        // Store sender in thread-local for the callback
        FOREGROUND_TX.with(|cell| {
            *cell.borrow_mut() = Some(tx);
        });

        let _hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(foreground_event_callback),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };

        // Run message loop (required for out-of-context hooks)
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }
        }
    });

    // Thread 2: receive HWND changes and invoke callback
    std::thread::spawn(move || {
        while let Ok(hwnd) = rx.recv() {
            on_foreground_change(hwnd);
        }
    });
}

thread_local! {
    static FOREGROUND_TX: std::cell::RefCell<Option<std::sync::mpsc::Sender<isize>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(windows)]
unsafe extern "system" fn foreground_event_callback(
    _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _event_time: u32,
) {
    FOREGROUND_TX.with(|cell| {
        if let Some(tx) = cell.borrow().as_ref() {
            let _ = tx.send(hwnd.0 as isize);
        }
    });
}

#[cfg(not(windows))]
pub fn start_foreground_listener(_on_foreground_change: impl Fn(isize) + Send + 'static) {}

/// Get work area (screen minus taskbar) in physical pixels: (x, y, width, height)
#[cfg(windows)]
pub fn get_work_area() -> (f64, f64, f64, f64) {
    let mut rect = RECT::default();
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rect as *mut _ as *mut _),
            Default::default(),
        )
    };
    if ok.is_ok() {
        (
            rect.left as f64,
            rect.top as f64,
            (rect.right - rect.left) as f64,
            (rect.bottom - rect.top) as f64,
        )
    } else {
        (0.0, 0.0, 1920.0, 1080.0)
    }
}

/// Monitor information for multi-monitor support
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorInfo {
    pub name: String,
    pub work_area: (f64, f64, f64, f64), // (x, y, w, h) physical pixels
    pub is_primary: bool,
}

/// Enumerate all monitors, primary first
#[cfg(windows)]
pub fn get_monitor_list() -> Vec<MonitorInfo> {
    let monitors: Arc<Mutex<Vec<MonitorInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let monitors_clone = monitors.clone();

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitors_callback),
            LPARAM(&monitors_clone as *const _ as isize),
        );
    }

    let mut list = monitors.lock().unwrap().clone();

    // QueryDisplayConfig으로 GDI device name → friendly name 맵 구축
    let friendly_map = get_friendly_monitor_names();
    for m in &mut list {
        if let Some(friendly) = friendly_map.get(&m.name) {
            m.name = friendly.clone();
        }
    }

    // Sort: primary first, then by name
    list.sort_by(|a, b| b.is_primary.cmp(&a.is_primary).then(a.name.cmp(&b.name)));
    list
}

/// QueryDisplayConfig API로 GDI 디바이스 이름 → 모니터 실제 이름 맵 생성
#[cfg(windows)]
fn get_friendly_monitor_names() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut map = HashMap::new();

    unsafe {
        let mut num_paths: u32 = 0;
        let mut num_modes: u32 = 0;
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
            != windows::Win32::Foundation::WIN32_ERROR(0)
        {
            return map;
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];

        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            None,
        ) != windows::Win32::Foundation::WIN32_ERROR(0)
        {
            return map;
        }
        paths.truncate(num_paths as usize);

        for path in &paths {
            // Get target friendly name
            let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
            target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target_name.header.size =
                std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            target_name.header.adapterId = path.targetInfo.adapterId;
            target_name.header.id = path.targetInfo.id;

            if DisplayConfigGetDeviceInfo(&mut target_name.header) != 0i32 {
                continue;
            }

            let friendly = {
                let raw = &target_name.monitorFriendlyDeviceName;
                let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                String::from_utf16_lossy(&raw[..len])
            };

            if friendly.is_empty() {
                continue;
            }

            // Get source GDI device name (e.g. "\\.\DISPLAY1")
            let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
            source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source_name.header.size =
                std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            source_name.header.adapterId = path.sourceInfo.adapterId;
            source_name.header.id = path.sourceInfo.id;

            if DisplayConfigGetDeviceInfo(&mut source_name.header) != 0i32 {
                continue;
            }

            let gdi_name = {
                let raw = &source_name.viewGdiDeviceName;
                let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                String::from_utf16_lossy(&raw[..len])
            };

            map.insert(gdi_name, friendly);
        }
    }

    map
}

#[cfg(windows)]
unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprc: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &*(lparam.0 as *const Arc<Mutex<Vec<MonitorInfo>>>);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
        let rc = info.monitorInfo.rcWork;
        let is_primary = (info.monitorInfo.dwFlags & 0x00000001) != 0;
        let device_name_raw = &info.szDevice[..info
            .szDevice
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(info.szDevice.len())];
        let device_path = String::from_utf16_lossy(device_name_raw);

        let mut lock = monitors.lock().unwrap();
        lock.push(MonitorInfo {
            name: device_path,
            work_area: (
                rc.left as f64,
                rc.top as f64,
                (rc.right - rc.left) as f64,
                (rc.bottom - rc.top) as f64,
            ),
            is_primary,
        });
    }

    BOOL(1)
}

/// Get work area for a specific monitor by value ("primary", "0", "1", ...)
#[cfg(windows)]
pub fn get_monitor_work_area(monitor_value: &str) -> (f64, f64, f64, f64) {
    if monitor_value == "primary" || monitor_value.is_empty() {
        return get_work_area();
    }

    if let Ok(index) = monitor_value.parse::<usize>() {
        let list = get_monitor_list();
        if let Some(m) = list.get(index) {
            return m.work_area;
        }
    }

    // Fallback to primary
    get_work_area()
}

#[cfg(not(windows))]
pub fn get_monitor_list() -> Vec<MonitorInfo> {
    vec![]
}

#[cfg(not(windows))]
pub fn get_monitor_work_area(_monitor_value: &str) -> (f64, f64, f64, f64) {
    (0.0, 0.0, 1920.0, 1080.0)
}

// Non-windows stubs
#[cfg(not(windows))]
pub fn get_process_tree(_start_pid: u32) -> Vec<u32> {
    vec![]
}
#[cfg(not(windows))]
pub fn find_source_window(
    _process_tree: &[u32],
    _title_hint: Option<&str>,
) -> (Vec<WindowCandidate>, Option<WindowCandidate>) {
    (vec![], None)
}
#[cfg(not(windows))]
pub fn get_window_title(_hwnd: isize) -> String {
    String::new()
}
#[cfg(not(windows))]
pub fn is_hwnd_focused(_hwnd: isize) -> bool {
    false
}
#[cfg(not(windows))]
pub fn activate_window(_hwnd: isize) {}
#[cfg(not(windows))]
pub fn get_work_area() -> (f64, f64, f64, f64) {
    (0.0, 0.0, 1920.0, 1080.0)
}
