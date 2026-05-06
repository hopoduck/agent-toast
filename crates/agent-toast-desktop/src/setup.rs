use agent_toast_core::hook_config::{is_agent_toast_cmd, merge_agent_toast_hooks, HookEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Hook configuration as shown in the setup GUI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookConfig {
    // 권장 항목
    pub stop_enabled: bool,
    pub stop_message: String,
    pub permission_request_enabled: bool,
    pub permission_request_message: String,
    // Notification 훅
    pub notification_permission_enabled: bool,
    pub notification_permission_message: String,
    pub notification_elicitation_enabled: bool,
    pub notification_elicitation_message: String,
    // 세션 생명주기
    pub setup_enabled: bool,
    pub setup_message: String,
    pub session_start_enabled: bool,
    pub session_start_message: String,
    pub session_end_enabled: bool,
    pub session_end_message: String,
    // 서브에이전트 생명주기
    pub subagent_start_enabled: bool,
    pub subagent_start_message: String,
    pub subagent_stop_enabled: bool,
    pub subagent_stop_message: String,
    // 사용자 입력
    pub user_prompt_submit_enabled: bool,
    pub user_prompt_submit_message: String,
    // 도구 실행 흐름
    pub pre_tool_use_enabled: bool,
    pub pre_tool_use_message: String,
    pub post_tool_use_enabled: bool,
    pub post_tool_use_message: String,
    pub post_tool_use_failure_enabled: bool,
    pub post_tool_use_failure_message: String,
    // 기타
    pub pre_compact_enabled: bool,
    pub pre_compact_message: String,
    pub notification_idle_enabled: bool,
    pub notification_idle_message: String,
    /// "project" = title_hint(실행 폴더명) 우선, "window" = 윈도우 제목 그대로
    #[serde(default = "default_title_display_mode")]
    pub title_display_mode: String,
    /// 타겟 창에 포커스 시 알림 자동 닫기
    #[serde(default = "default_auto_close_on_focus")]
    pub auto_close_on_focus: bool,
    /// 알림 자동 소멸 시간 (초). 0이면 자동 소멸 안 함.
    #[serde(default = "default_auto_dismiss_seconds")]
    pub auto_dismiss_seconds: u32,
    /// 알림 표시 위치: "bottom_right", "bottom_left", "top_right", "top_left"
    #[serde(default = "default_notification_position")]
    pub notification_position: String,
    /// 알림 소리 재생 여부
    #[serde(default = "default_notification_sound")]
    pub notification_sound: bool,
    /// 알림 표시 모니터: "primary", "0", "1", ...
    #[serde(default = "default_notification_monitor")]
    pub notification_monitor: String,
    /// UI 언어: "ko", "en"
    #[serde(default = "default_locale")]
    pub locale: String,
    /// 세션 시작 시 앱 자동 실행 여부 (--daemon 훅)
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    /// Codex notify 훅 활성화 여부
    #[serde(default)]
    pub codex_enabled: bool,
    /// 설정을 저장한 앱 버전 (마이그레이션 판단용)
    #[serde(default)]
    pub version: String,
    /// v1 신규: 원격 알림 HTTP 수신 활성화 (기본 false — 옵트인)
    #[serde(default = "default_http_enabled")]
    pub http_enabled: bool,
    /// HTTP 서버 바인딩 포트 (기본 38787). 주소는 항상 0.0.0.0 으로 고정.
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    /// 원격 알림 UI 에 호스트명 표시 여부 (기본 true)
    #[serde(default = "default_show_hostname")]
    pub show_hostname: bool,
}

fn default_title_display_mode() -> String {
    "project".into()
}

fn default_auto_close_on_focus() -> bool {
    true
}

fn default_auto_dismiss_seconds() -> u32 {
    0
}

fn default_notification_position() -> String {
    "bottom_right".into()
}

fn default_notification_sound() -> bool {
    true
}

fn default_notification_monitor() -> String {
    "primary".into()
}

fn default_auto_start() -> bool {
    true
}

fn default_http_enabled() -> bool {
    false
}

fn default_http_port() -> u16 {
    38787
}

fn default_show_hostname() -> bool {
    true
}

fn default_locale() -> String {
    detect_system_locale()
}

/// 시스템 UI 언어를 감지하여 지원하는 로케일 코드 반환
fn detect_system_locale() -> String {
    #[cfg(windows)]
    {
        // LANGID 하위 10비트 = primary language
        let langid = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
        let primary = langid & 0x3FF;
        if primary == 0x12 {
            // LANG_KOREAN
            return "ko".into();
        }
        "en".into()
    }
    #[cfg(not(windows))]
    {
        "ko".into()
    }
}

impl Default for HookConfig {
    fn default() -> Self {
        let locale = default_locale();
        let is_ko = locale == "ko";
        Self {
            // 권장 항목
            stop_enabled: true,
            stop_message: if is_ko {
                "작업이 완료되었습니다"
            } else {
                "Task completed"
            }
            .into(),
            permission_request_enabled: true,
            permission_request_message: if is_ko {
                "권한 요청이 발생했습니다"
            } else {
                "Permission requested"
            }
            .into(),
            // Notification 훅
            notification_permission_enabled: false,
            notification_permission_message: if is_ko {
                "권한 승인이 필요합니다"
            } else {
                "Permission approval required"
            }
            .into(),
            notification_elicitation_enabled: false,
            notification_elicitation_message: if is_ko {
                "입력이 필요합니다"
            } else {
                "Input required"
            }
            .into(),
            // 세션 생명주기
            setup_enabled: false,
            setup_message: if is_ko {
                "초기화가 실행되었습니다"
            } else {
                "Setup executed"
            }
            .into(),
            session_start_enabled: false,
            session_start_message: if is_ko {
                "세션이 시작되었습니다"
            } else {
                "Session started"
            }
            .into(),
            session_end_enabled: false,
            session_end_message: if is_ko {
                "세션이 종료되었습니다"
            } else {
                "Session ended"
            }
            .into(),
            // 서브에이전트 생명주기
            subagent_start_enabled: false,
            subagent_start_message: if is_ko {
                "서브에이전트가 시작되었습니다"
            } else {
                "Subagent started"
            }
            .into(),
            subagent_stop_enabled: false,
            subagent_stop_message: if is_ko {
                "서브에이전트가 완료되었습니다"
            } else {
                "Subagent completed"
            }
            .into(),
            // 사용자 입력
            user_prompt_submit_enabled: false,
            user_prompt_submit_message: if is_ko {
                "프롬프트가 제출되었습니다"
            } else {
                "Prompt submitted"
            }
            .into(),
            // 도구 실행 흐름
            pre_tool_use_enabled: false,
            pre_tool_use_message: if is_ko {
                "도구 실행이 시작됩니다"
            } else {
                "Tool execution starting"
            }
            .into(),
            post_tool_use_enabled: false,
            post_tool_use_message: if is_ko {
                "도구 실행이 완료되었습니다"
            } else {
                "Tool execution completed"
            }
            .into(),
            post_tool_use_failure_enabled: false,
            post_tool_use_failure_message: if is_ko {
                "도구 실행이 실패했습니다"
            } else {
                "Tool execution failed"
            }
            .into(),
            // 기타
            pre_compact_enabled: false,
            pre_compact_message: if is_ko {
                "컨텍스트 압축이 시작됩니다"
            } else {
                "Context compaction starting"
            }
            .into(),
            notification_idle_enabled: false,
            notification_idle_message: if is_ko {
                "입력을 기다리고 있습니다"
            } else {
                "Waiting for input"
            }
            .into(),
            // 설정
            title_display_mode: "project".into(),
            auto_close_on_focus: true,
            auto_dismiss_seconds: 0,
            http_enabled: false,
            http_port: default_http_port(),
            show_hostname: true,
            notification_position: "bottom_right".into(),
            notification_sound: true,
            notification_monitor: "primary".into(),
            locale,
            auto_start: true,
            codex_enabled: false,
            version: String::new(),
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

/// Returns the exe path without quotes (for TOML array, display, etc.)
fn exe_path_unquoted() -> String {
    std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("agent-toast.exe"))
        .to_string_lossy()
        .to_string()
}

/// Returns the exe path always quoted (bash interprets backslashes as escape chars without quotes)
fn exe_path_for_shell() -> String {
    let path = exe_path_unquoted();
    format!("\"{}\"", path)
}

/// Read current hook config from ~/.claude/settings.json
#[tauri::command]
pub fn get_hook_config() -> HookConfig {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return HookConfig::default();
    };
    parse_hook_config_from_json(&content)
}

/// Parse hook config from raw JSON string. Separated for testability.
fn parse_hook_config_from_json(content: &str) -> HookConfig {
    let Ok(root) = serde_json::from_str::<Value>(content) else {
        return HookConfig::default();
    };

    let hooks = &root["hooks"];

    let mut config = HookConfig {
        // 모든 enabled는 false로 시작 (JSON에서 agent-toast 훅 발견 시 true로 설정)
        stop_enabled: false,
        notification_permission_enabled: false,
        notification_elicitation_enabled: false,
        setup_enabled: false,
        session_start_enabled: false,
        session_end_enabled: false,
        subagent_start_enabled: false,
        subagent_stop_enabled: false,
        user_prompt_submit_enabled: false,
        permission_request_enabled: false,
        pre_tool_use_enabled: false,
        post_tool_use_enabled: false,
        post_tool_use_failure_enabled: false,
        pre_compact_enabled: false,
        notification_idle_enabled: false,
        // agent_toast 설정은 JSON에서 읽기
        title_display_mode: root["agent_toast"]["title_display_mode"]
            .as_str()
            .unwrap_or("project")
            .to_string(),
        auto_close_on_focus: root["agent_toast"]["auto_close_on_focus"]
            .as_bool()
            .unwrap_or(true),
        auto_dismiss_seconds: root["agent_toast"]["auto_dismiss_seconds"]
            .as_u64()
            .unwrap_or(0) as u32,
        notification_position: root["agent_toast"]["notification_position"]
            .as_str()
            .unwrap_or("bottom_right")
            .to_string(),
        notification_sound: root["agent_toast"]["notification_sound"]
            .as_bool()
            .unwrap_or(true),
        notification_monitor: root["agent_toast"]["notification_monitor"]
            .as_str()
            .unwrap_or("primary")
            .to_string(),
        locale: root["agent_toast"]["locale"]
            .as_str()
            .unwrap_or("ko")
            .to_string(),
        auto_start: root["agent_toast"]["auto_start"].as_bool().unwrap_or(true),
        codex_enabled: root["agent_toast"]["codex_enabled"]
            .as_bool()
            .unwrap_or(false),
        version: root["agent_toast"]["version"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        http_enabled: root["agent_toast"]["http_enabled"]
            .as_bool()
            .unwrap_or_else(default_http_enabled),
        http_port: root["agent_toast"]["http_port"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .unwrap_or_else(default_http_port),
        show_hostname: root["agent_toast"]["show_hostname"]
            .as_bool()
            .unwrap_or_else(default_show_hostname),
        // 나머지는 Default에서 가져오기
        ..HookConfig::default()
    };

    // Check Stop hooks
    if let Some(stop_arr) = hooks["Stop"].as_array() {
        for entry in stop_arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.stop_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.stop_message = msg;
                }
            }
        }
    }

    // Check Notification hooks
    if let Some(notif_arr) = hooks["Notification"].as_array() {
        for entry in notif_arr {
            let matcher = entry["matcher"].as_str().unwrap_or("");
            let Some(cmd) = extract_agent_toast_cmd(entry) else {
                continue;
            };
            match matcher {
                "permission_prompt" => {
                    config.notification_permission_enabled = true;
                    if let Some(msg) = extract_message(cmd) {
                        config.notification_permission_message = msg;
                    }
                }
                "elicitation_dialog" => {
                    config.notification_elicitation_enabled = true;
                    if let Some(msg) = extract_message(cmd) {
                        config.notification_elicitation_message = msg;
                    }
                }
                "idle_prompt" => {
                    config.notification_idle_enabled = true;
                    if let Some(msg) = extract_message(cmd) {
                        config.notification_idle_message = msg;
                    }
                }
                _ => {}
            }
        }
    }

    // Check SessionStart hooks
    // --daemon only entries are infrastructure (always added), skip them
    // --message entries indicate notification enabled
    if let Some(ss_arr) = hooks["SessionStart"].as_array() {
        for entry in ss_arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                if extract_message(cmd).is_some() {
                    config.session_start_enabled = true;
                    if let Some(msg) = extract_message(cmd) {
                        config.session_start_message = msg;
                    }
                }
            }
        }
    }

    // Check SessionEnd hooks
    if let Some(arr) = hooks["SessionEnd"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.session_end_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.session_end_message = msg;
                }
            }
        }
    }

    // Check SubagentStop hooks
    if let Some(arr) = hooks["SubagentStop"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.subagent_stop_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.subagent_stop_message = msg;
                }
            }
        }
    }

    // Check PreCompact hooks
    if let Some(arr) = hooks["PreCompact"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.pre_compact_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.pre_compact_message = msg;
                }
            }
        }
    }

    // Check Setup hooks
    if let Some(arr) = hooks["Setup"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.setup_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.setup_message = msg;
                }
            }
        }
    }

    // Check UserPromptSubmit hooks
    if let Some(arr) = hooks["UserPromptSubmit"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.user_prompt_submit_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.user_prompt_submit_message = msg;
                }
            }
        }
    }

    // Check PreToolUse hooks
    if let Some(arr) = hooks["PreToolUse"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.pre_tool_use_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.pre_tool_use_message = msg;
                }
            }
        }
    }

    // Check PostToolUse hooks
    if let Some(arr) = hooks["PostToolUse"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.post_tool_use_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.post_tool_use_message = msg;
                }
            }
        }
    }

    // Check PostToolUseFailure hooks
    if let Some(arr) = hooks["PostToolUseFailure"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.post_tool_use_failure_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.post_tool_use_failure_message = msg;
                }
            }
        }
    }

    // Check PermissionRequest hooks
    if let Some(arr) = hooks["PermissionRequest"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.permission_request_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.permission_request_message = msg;
                }
            }
        }
    }

    // Check SubagentStart hooks
    if let Some(arr) = hooks["SubagentStart"].as_array() {
        for entry in arr {
            if let Some(cmd) = extract_agent_toast_cmd(entry) {
                config.subagent_start_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.subagent_start_message = msg;
                }
            }
        }
    }

    config
}

/// Check if settings.json contains any agent-toast hooks
#[tauri::command]
pub fn is_hook_config_saved() -> bool {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return false;
    };
    let hooks = &root["hooks"];
    // Check if agent_toast section exists (covers codex-only usage)
    if root
        .get("agent_toast")
        .and_then(|v| v.as_object())
        .is_some()
    {
        return true;
    }
    // Check if any hook event array contains an agent-toast command
    for key in [
        "Stop",
        "Notification",
        "SessionStart",
        "SessionEnd",
        "SubagentStop",
        "PreCompact",
        "Setup",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "PermissionRequest",
        "SubagentStart",
    ] {
        if let Some(arr) = hooks[key].as_array() {
            for entry in arr {
                if is_agent_toast_entry(entry) {
                    return true;
                }
            }
        }
    }
    false
}

/// Write `agent_toast` settings block into `root`, preserving any pre-existing
/// keys within `agent_toast` that are not managed by this struct.
fn write_agent_toast_settings(root: &mut Value, config: &HookConfig) {
    let mut cn = root["agent_toast"].as_object().cloned().unwrap_or_default();
    cn.insert(
        "title_display_mode".into(),
        Value::String(config.title_display_mode.clone()),
    );
    cn.insert(
        "auto_close_on_focus".into(),
        Value::Bool(config.auto_close_on_focus),
    );
    cn.insert(
        "auto_dismiss_seconds".into(),
        Value::Number(config.auto_dismiss_seconds.into()),
    );
    cn.insert(
        "notification_position".into(),
        Value::String(config.notification_position.clone()),
    );
    cn.insert(
        "notification_sound".into(),
        Value::Bool(config.notification_sound),
    );
    cn.insert(
        "notification_monitor".into(),
        Value::String(config.notification_monitor.clone()),
    );
    cn.insert("locale".into(), Value::String(config.locale.clone()));
    cn.insert("auto_start".into(), Value::Bool(config.auto_start));
    cn.insert("codex_enabled".into(), Value::Bool(config.codex_enabled));
    cn.insert("http_enabled".into(), Value::Bool(config.http_enabled));
    cn.insert("http_port".into(), Value::Number(config.http_port.into()));
    cn.insert("show_hostname".into(), Value::Bool(config.show_hostname));
    cn.insert(
        "version".into(),
        Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );
    root["agent_toast"] = Value::Object(cn);
}

/// Save hook config to ~/.claude/settings.json, preserving other fields
#[tauri::command]
pub fn save_hook_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::notification::NotificationManagerState>,
    config: HookConfig,
) -> Result<String, String> {
    let path = settings_path();

    // Read existing settings or create new object
    let mut root: Value = if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    let exe = exe_path_for_shell();

    // Build agent-toast hook entries.
    let mut entries: Vec<HookEntry> = Vec::new();

    // SessionStart: add --daemon entry if auto_start is enabled
    if config.auto_start {
        entries.push(HookEntry {
            event_key: "SessionStart",
            matcher: None,
            command: format!("{} --daemon", exe),
        });
    }
    // SessionStart: add notification entry if enabled
    if config.session_start_enabled {
        entries.push(HookEntry {
            event_key: "SessionStart",
            matcher: None,
            command: format!(
                "{} --event session_start --message \"{}\"",
                exe, config.session_start_message
            ),
        });
    }

    // CLI reads CLAUDE_PROJECT_DIR env var directly as title hint fallback,
    // so no --title arg needed in the hook command.

    if config.stop_enabled {
        entries.push(HookEntry {
            event_key: "Stop",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.stop_message
            ),
        });
    }

    if config.notification_permission_enabled {
        entries.push(HookEntry {
            event_key: "Notification",
            matcher: Some("permission_prompt"),
            command: format!(
                "{} --event user_input_required --message \"{}\"",
                exe, config.notification_permission_message
            ),
        });
    }

    if config.notification_elicitation_enabled {
        entries.push(HookEntry {
            event_key: "Notification",
            matcher: Some("elicitation_dialog"),
            command: format!(
                "{} --event user_input_required --message \"{}\"",
                exe, config.notification_elicitation_message
            ),
        });
    }

    if config.notification_idle_enabled {
        entries.push(HookEntry {
            event_key: "Notification",
            matcher: Some("idle_prompt"),
            command: format!(
                "{} --event user_input_required --message \"{}\"",
                exe, config.notification_idle_message
            ),
        });
    }

    if config.session_end_enabled {
        entries.push(HookEntry {
            event_key: "SessionEnd",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.session_end_message
            ),
        });
    }

    if config.subagent_stop_enabled {
        entries.push(HookEntry {
            event_key: "SubagentStop",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.subagent_stop_message
            ),
        });
    }

    if config.pre_compact_enabled {
        entries.push(HookEntry {
            event_key: "PreCompact",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.pre_compact_message
            ),
        });
    }

    if config.setup_enabled {
        entries.push(HookEntry {
            event_key: "Setup",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.setup_message
            ),
        });
    }

    if config.user_prompt_submit_enabled {
        entries.push(HookEntry {
            event_key: "UserPromptSubmit",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.user_prompt_submit_message
            ),
        });
    }

    if config.pre_tool_use_enabled {
        entries.push(HookEntry {
            event_key: "PreToolUse",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.pre_tool_use_message
            ),
        });
    }

    if config.post_tool_use_enabled {
        entries.push(HookEntry {
            event_key: "PostToolUse",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.post_tool_use_message
            ),
        });
    }

    if config.post_tool_use_failure_enabled {
        entries.push(HookEntry {
            event_key: "PostToolUseFailure",
            matcher: None,
            command: format!(
                "{} --event error --message \"{}\"",
                exe, config.post_tool_use_failure_message
            ),
        });
    }

    if config.permission_request_enabled {
        entries.push(HookEntry {
            event_key: "PermissionRequest",
            matcher: None,
            command: format!(
                "{} --event user_input_required --message \"{}\"",
                exe, config.permission_request_message
            ),
        });
    }

    if config.subagent_start_enabled {
        entries.push(HookEntry {
            event_key: "SubagentStart",
            matcher: None,
            command: format!(
                "{} --event task_complete --message \"{}\"",
                exe, config.subagent_start_message
            ),
        });
    }

    // Merge entries into root, preserving non-agent-toast hooks.
    root = merge_agent_toast_hooks(root, &entries);

    write_agent_toast_settings(&mut root, &config);

    // Ensure .claude directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())?;

    // Codex config.toml 업데이트
    save_codex_config(config.codex_enabled).map_err(|e| e.to_string())?;

    // HTTP 서버 상태를 새 설정과 동기화 (즉시 시작/중지/재시작)
    crate::sync_http_server(&app).map_err(|e| format!("HTTP 서버 시작 실패: {e}"))?;

    // 저장 후 이미 떠 있는 알림들의 위치를 즉시 반영
    crate::notification::reposition_all(&app, &state);

    // 트레이 메뉴 텍스트를 현재 locale에 맞게 갱신
    crate::update_tray_locale(&app);

    let mut result = path.to_string_lossy().to_string();
    if config.codex_enabled {
        result.push('\n');
        result.push_str(&codex_config_path().to_string_lossy());
    }
    Ok(result)
}

#[tauri::command]
pub fn get_exe_path() -> String {
    std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("agent-toast.exe"))
        .to_string_lossy()
        .to_string()
}

/// 설정 파일의 훅 커맨드에서 저장된 실행 파일 경로 추출
#[tauri::command]
pub fn get_saved_exe_path() -> Option<String> {
    let path = settings_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let root: Value = serde_json::from_str(&content).ok()?;
    let hooks = root.get("hooks")?.as_object()?;
    // 아무 훅 커맨드에서 exe 경로 추출
    for (_key, entries) in hooks {
        let arr = entries.as_array()?;
        for entry in arr {
            let inner = entry.get("hooks")?.as_array()?;
            for hook in inner {
                if hook.get("type")?.as_str()? == "command" {
                    let cmd = hook.get("command")?.as_str()?;
                    // "path\to\exe.exe --event ..." 형태에서 경로 추출
                    if let Some(idx) = cmd.find(" --") {
                        let exe = cmd[..idx].trim().trim_matches('"');
                        let lower = exe.to_lowercase();
                        if lower.contains("agent-toast") || lower.contains("agent toast") {
                            return Some(exe.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn open_settings_file() -> Result<(), String> {
    let path = settings_path();
    open::that(&path).map_err(|e| e.to_string())
}

pub fn read_http_enabled() -> bool {
    read_hook_config().http_enabled
}

pub fn read_http_port() -> u16 {
    read_hook_config().http_port
}

pub fn read_show_hostname() -> bool {
    read_hook_config().show_hostname
}

/// Read `HookConfig` from ~/.claude/settings.json — returns Default on any failure.
fn read_hook_config() -> HookConfig {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return HookConfig::default();
    };
    parse_hook_config_from_json(&content)
}

/// 설정 파일에서 auto_close_on_focus 값만 빠르게 읽기
pub fn load_auto_close_on_focus() -> bool {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return true;
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return true;
    };
    root["agent_toast"]["auto_close_on_focus"]
        .as_bool()
        .unwrap_or(true)
}

/// 설정 파일에서 notification_sound 값만 빠르게 읽기
pub fn load_notification_sound() -> bool {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return true;
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return true;
    };
    root["agent_toast"]["notification_sound"]
        .as_bool()
        .unwrap_or(true)
}

/// 설정 파일에서 notification_position 값만 빠르게 읽기
pub fn load_notification_position() -> String {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return "bottom_right".into();
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return "bottom_right".into();
    };
    root["agent_toast"]["notification_position"]
        .as_str()
        .unwrap_or("bottom_right")
        .to_string()
}

/// 설정 파일에서 notification_monitor 값만 빠르게 읽기
pub fn load_notification_monitor() -> String {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return "primary".into();
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return "primary".into();
    };
    root["agent_toast"]["notification_monitor"]
        .as_str()
        .unwrap_or("primary")
        .to_string()
}

/// 설정 파일에서 locale 값만 빠르게 읽기
pub fn read_locale() -> String {
    let fallback = detect_system_locale();
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return fallback;
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return fallback;
    };
    root["agent_toast"]["locale"]
        .as_str()
        .unwrap_or(&fallback)
        .to_string()
}

fn codex_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("config.toml")
}

fn save_codex_config(enabled: bool) -> Result<(), String> {
    let path = codex_config_path();

    // Read existing config or start fresh
    let mut doc: toml::Table = if let Ok(content) = std::fs::read_to_string(&path) {
        content
            .parse::<toml::Table>()
            .unwrap_or_else(|_| toml::Table::new())
    } else {
        toml::Table::new()
    };

    if enabled {
        let exe = exe_path_unquoted();
        let notify_value = toml::Value::Array(vec![
            toml::Value::String(exe),
            toml::Value::String("--codex".into()),
        ]);
        doc.insert("notify".into(), notify_value);
    } else {
        doc.remove("notify");
    }

    // Ensure .codex directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = toml::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Extract the command string from a hook entry if it belongs to agent-toast.
/// Delegates detection to `is_agent_toast_cmd` (core) so hyphen/underscore/space
/// variants and "Agent Toast" productName are all matched consistently.
fn extract_agent_toast_cmd(entry: &Value) -> Option<&str> {
    let hooks_arr = entry.get("hooks")?.as_array()?;
    for hook in hooks_arr {
        let cmd = hook.get("command")?.as_str()?;
        if is_agent_toast_cmd(cmd) {
            return Some(cmd);
        }
    }
    None
}

/// Check if a hook entry belongs to agent-toast.
fn is_agent_toast_entry(entry: &Value) -> bool {
    extract_agent_toast_cmd(entry).is_some()
}

#[cfg(test)]
fn build_hook_entry(matcher: Option<&str>, command: &str, timeout: Option<u32>) -> Value {
    let mut entry = serde_json::Map::new();
    if let Some(m) = matcher {
        entry.insert("matcher".into(), Value::String(m.into()));
    }
    let mut hook_obj = serde_json::json!({
        "type": "command",
        "command": command
    });
    if let Some(t) = timeout {
        hook_obj["timeout"] = Value::Number(t.into());
    }
    entry.insert("hooks".into(), Value::Array(vec![hook_obj]));
    Value::Object(entry)
}

/// Extract --message value from a command string.
/// Supports: --message="...", --message "...", --message=value
fn extract_message(cmd: &str) -> Option<String> {
    // Match --message="..." or --message "..."
    let patterns = ["--message=\"", "--message \""];
    for pat in patterns {
        if let Some(start) = cmd.find(pat) {
            let msg_start = start + pat.len();
            let rest = &cmd[msg_start..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    // Match --message=value (no quotes)
    if let Some(start) = cmd.find("--message=") {
        let msg_start = start + "--message=".len();
        let rest = &cmd[msg_start..];
        let end = rest.find(' ').unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_message tests ──

    #[test]
    fn extract_message_double_quoted_equals() {
        let cmd = r#"agent-toast --event task_complete --message="빌드 완료""#;
        assert_eq!(extract_message(cmd), Some("빌드 완료".to_string()));
    }

    #[test]
    fn extract_message_double_quoted_space() {
        let cmd = r#"agent-toast --event task_complete --message "빌드 완료""#;
        assert_eq!(extract_message(cmd), Some("빌드 완료".to_string()));
    }

    #[test]
    fn extract_message_unquoted_equals() {
        let cmd = "agent-toast --message=hello";
        assert_eq!(extract_message(cmd), Some("hello".to_string()));
    }

    #[test]
    fn extract_message_unquoted_equals_with_trailing_args() {
        let cmd = "agent-toast --message=hello --event task_complete";
        assert_eq!(extract_message(cmd), Some("hello".to_string()));
    }

    #[test]
    fn extract_message_none_when_missing() {
        let cmd = "agent-toast --event task_complete";
        assert_eq!(extract_message(cmd), None);
    }

    #[test]
    fn extract_message_empty_quoted() {
        let cmd = r#"agent-toast --message="""#;
        assert_eq!(extract_message(cmd), Some("".to_string()));
    }

    #[test]
    fn extract_message_flag_without_value() {
        // --message 뒤에 값 없이 끝나는 경우
        let cmd = "agent-toast --message";
        assert_eq!(extract_message(cmd), None);
    }

    #[test]
    fn extract_message_single_quoted_not_supported() {
        // 싱글 쿼트는 지원하지 않으므로 None 또는 쿼트 포함 문자열 반환
        let cmd = "agent-toast --message='hello'";
        // --message= 이후 'hello'가 unquoted로 파싱됨
        assert_eq!(extract_message(cmd), Some("'hello'".to_string()));
    }

    // ── HookConfig default tests ──

    #[test]
    fn hook_config_default_values() {
        let config = HookConfig::default();
        assert!(config.stop_enabled);
        assert!(config.permission_request_enabled);
        assert!(!config.notification_permission_enabled);
        assert!(!config.notification_elicitation_enabled);
        assert!(!config.notification_idle_enabled);
        assert!(!config.session_start_enabled);
        assert!(!config.session_end_enabled);
        assert_eq!(config.title_display_mode, "project");
        assert!(config.auto_close_on_focus);
        assert_eq!(config.auto_dismiss_seconds, 0);
        assert_eq!(config.notification_position, "bottom_right");
        assert!(config.notification_sound);
        assert_eq!(config.notification_monitor, "primary");
    }

    #[test]
    fn hook_config_serde_roundtrip() {
        let config = HookConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn hook_config_deserialize_with_defaults() {
        // serde default가 누락된 필드에 적용되는지 확인
        let json = r#"{
            "stop_enabled": false,
            "stop_message": "test",
            "notification_permission_enabled": false,
            "notification_permission_message": "test",
            "notification_elicitation_enabled": false,
            "notification_elicitation_message": "test",
            "notification_idle_enabled": false,
            "notification_idle_message": "test",
            "session_start_enabled": false,
            "session_start_message": "test",
            "session_end_enabled": false,
            "session_end_message": "test",
            "subagent_stop_enabled": false,
            "subagent_stop_message": "test",
            "pre_compact_enabled": false,
            "pre_compact_message": "test",
            "setup_enabled": false,
            "setup_message": "test",
            "user_prompt_submit_enabled": false,
            "user_prompt_submit_message": "test",
            "pre_tool_use_enabled": false,
            "pre_tool_use_message": "test",
            "post_tool_use_enabled": false,
            "post_tool_use_message": "test",
            "post_tool_use_failure_enabled": false,
            "post_tool_use_failure_message": "test",
            "permission_request_enabled": false,
            "permission_request_message": "test",
            "subagent_start_enabled": false,
            "subagent_start_message": "test"
        }"#;
        let config: HookConfig = serde_json::from_str(json).unwrap();
        // serde(default) 필드들이 기본값으로 채워져야 함
        assert_eq!(config.title_display_mode, "project");
        assert!(config.auto_close_on_focus);
        assert_eq!(config.auto_dismiss_seconds, 0);
        assert_eq!(config.notification_position, "bottom_right");
        assert!(config.notification_sound);
        assert_eq!(config.notification_monitor, "primary");
    }

    // ── build_hook_entry tests ──

    #[test]
    fn build_hook_entry_without_matcher() {
        let entry = build_hook_entry(None, "agent-toast --daemon", None);
        assert!(entry["matcher"].is_null());
        assert_eq!(
            entry["hooks"][0]["command"].as_str().unwrap(),
            "agent-toast --daemon"
        );
        assert_eq!(entry["hooks"][0]["type"].as_str().unwrap(), "command");
    }

    #[test]
    fn build_hook_entry_with_matcher() {
        let entry = build_hook_entry(Some("permission_prompt"), "agent-toast --event test", None);
        assert_eq!(entry["matcher"].as_str().unwrap(), "permission_prompt");
        assert_eq!(
            entry["hooks"][0]["command"].as_str().unwrap(),
            "agent-toast --event test"
        );
    }

    #[test]
    fn build_hook_entry_with_timeout() {
        let entry = build_hook_entry(None, "agent-toast --daemon", Some(5));
        // timeout이 hook 객체에 포함되어야 함
        assert_eq!(entry["hooks"][0]["timeout"].as_u64().unwrap(), 5);
        assert_eq!(
            entry["hooks"][0]["command"].as_str().unwrap(),
            "agent-toast --daemon"
        );
    }

    // ── parse_hook_config_from_json tests ──

    /// 빈 JSON → hooks 없으므로 enabled=false, agent_toast 섹션 없으므로 설정은 기본값.
    #[test]
    fn parse_empty_json_returns_all_hooks_disabled_with_parse_defaults() {
        let config = parse_hook_config_from_json("{}");
        // hooks가 없으므로 모든 enabled가 false
        assert!(!config.stop_enabled);
        assert!(!config.notification_permission_enabled);
        assert!(!config.session_start_enabled);
        // agent_toast 섹션도 없으므로 기본값 적용
        assert_eq!(config.title_display_mode, "project");
        assert!(config.auto_close_on_focus);
        assert_eq!(config.auto_dismiss_seconds, 0);
        assert_eq!(config.notification_position, "bottom_right");
    }

    /// 잘못된 JSON → HookConfig::default() 반환 (parse 경로 기본값과 다름)
    #[test]
    fn parse_invalid_json_returns_struct_default() {
        let config = parse_hook_config_from_json("not valid json");
        // 파싱 실패 시 HookConfig::default() 반환 — parse 경로와 다른 기본값
        assert_eq!(config, HookConfig::default());
    }

    #[test]
    fn parse_stop_hook_enabled() {
        let json = r#"{
            "hooks": {
                "Stop": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"작업 끝\""}]
                }]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.stop_enabled);
        assert_eq!(config.stop_message, "작업 끝");
    }

    #[test]
    fn parse_notification_hooks_by_matcher() {
        let json = r#"{
            "hooks": {
                "Notification": [
                    {
                        "matcher": "permission_prompt",
                        "hooks": [{"type": "command", "command": "agent-toast --event user_input_required --message=\"권한 필요\""}]
                    },
                    {
                        "matcher": "elicitation_dialog",
                        "hooks": [{"type": "command", "command": "agent-toast --event user_input_required --message=\"입력 요청\""}]
                    },
                    {
                        "matcher": "idle_prompt",
                        "hooks": [{"type": "command", "command": "agent-toast --event user_input_required --message=\"대기 중\""}]
                    }
                ]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.notification_permission_enabled);
        assert_eq!(config.notification_permission_message, "권한 필요");
        assert!(config.notification_elicitation_enabled);
        assert_eq!(config.notification_elicitation_message, "입력 요청");
        assert!(config.notification_idle_enabled);
        assert_eq!(config.notification_idle_message, "대기 중");
    }

    #[test]
    fn parse_ignores_non_agent_toast_hooks() {
        let json = r#"{
            "hooks": {
                "Stop": [{
                    "hooks": [{"type": "command", "command": "some-other-tool --done"}]
                }]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.stop_enabled);
    }

    #[test]
    fn parse_agent_toast_settings() {
        let json = r#"{
            "agent_toast": {
                "title_display_mode": "window",
                "auto_close_on_focus": false,
                "auto_dismiss_seconds": 30,
                "notification_position": "top_left",
                "notification_sound": false,
                "notification_monitor": "1"
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.title_display_mode, "window");
        assert!(!config.auto_close_on_focus);
        assert_eq!(config.auto_dismiss_seconds, 30);
        assert_eq!(config.notification_position, "top_left");
        assert!(!config.notification_sound);
        assert_eq!(config.notification_monitor, "1");
    }

    #[test]
    fn parse_all_hook_types() {
        // serde_json::json!으로 생성하여 이스케이프 문제 방지
        let json_value = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {"hooks": [{"type": "command", "command": "agent-toast --daemon"}]},
                    {"hooks": [{"type": "command", "command": "agent-toast --event session_start --message=\"세션 시작\""}]}
                ],
                "SessionEnd": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"세션 끝\""}]}],
                "SubagentStop": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"서브 완료\""}]}],
                "PreCompact": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"압축\""}]}],
                "Setup": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"초기화\""}]}],
                "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"제출\""}]}],
                "PreToolUse": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"도구 시작\""}]}],
                "PostToolUse": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"도구 끝\""}]}],
                "PostToolUseFailure": [{"hooks": [{"type": "command", "command": "agent-toast --event error --message=\"도구 실패\""}]}],
                "PermissionRequest": [{"hooks": [{"type": "command", "command": "agent-toast --event user_input_required --message=\"권한\""}]}],
                "SubagentStart": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"서브 시작\""}]}]
            }
        });
        let json = serde_json::to_string(&json_value).unwrap();

        let config = parse_hook_config_from_json(&json);
        assert!(config.session_start_enabled);
        assert_eq!(config.session_start_message, "세션 시작");
        assert!(config.session_end_enabled);
        assert_eq!(config.session_end_message, "세션 끝");
        assert!(config.subagent_stop_enabled);
        assert!(config.pre_compact_enabled);
        assert!(config.setup_enabled);
        assert!(config.user_prompt_submit_enabled);
        assert!(config.pre_tool_use_enabled);
        assert!(config.post_tool_use_enabled);
        assert!(config.post_tool_use_failure_enabled);
        assert!(config.permission_request_enabled);
        assert!(config.subagent_start_enabled);
    }

    #[test]
    fn parse_mixed_agent_toast_and_other_hooks() {
        let json = r#"{
            "hooks": {
                "Stop": [
                    {"hooks": [{"type": "command", "command": "other-tool --notify"}]},
                    {"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"완료\""}]}
                ]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.stop_enabled);
        assert_eq!(config.stop_message, "완료");
    }

    // ── HookConfig additional tests ──

    #[test]
    fn hook_config_all_hooks_disabled() {
        let config = HookConfig {
            stop_enabled: false,
            notification_permission_enabled: false,
            notification_elicitation_enabled: false,
            ..Default::default()
        };

        assert!(!config.stop_enabled);
        assert!(!config.notification_permission_enabled);
        assert!(!config.notification_elicitation_enabled);
    }

    #[test]
    fn hook_config_locale_values() {
        for locale in ["ko", "en"] {
            let config = HookConfig {
                locale: locale.to_string(),
                ..Default::default()
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.locale, locale);
        }
    }

    #[test]
    fn hook_config_notification_positions() {
        let positions = ["bottom_right", "bottom_left", "top_right", "top_left"];
        for pos in positions {
            let config = HookConfig {
                notification_position: pos.to_string(),
                ..Default::default()
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.notification_position, pos);
        }
    }

    #[test]
    fn hook_config_auto_dismiss_values() {
        for seconds in [0, 5, 10, 30, 60, 300] {
            let config = HookConfig {
                auto_dismiss_seconds: seconds,
                ..Default::default()
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.auto_dismiss_seconds, seconds);
        }
    }

    #[test]
    fn hook_config_title_display_modes() {
        for mode in ["project", "window"] {
            let config = HookConfig {
                title_display_mode: mode.to_string(),
                ..Default::default()
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.title_display_mode, mode);
        }
    }

    #[test]
    fn hook_config_monitor_values() {
        let monitors = ["primary", "0", "1", "2"];
        for monitor in monitors {
            let config = HookConfig {
                notification_monitor: monitor.to_string(),
                ..Default::default()
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.notification_monitor, monitor);
        }
    }

    #[test]
    fn hook_config_boolean_fields() {
        let config = HookConfig {
            auto_close_on_focus: false,
            notification_sound: false,
            codex_enabled: true,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HookConfig = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.auto_close_on_focus);
        assert!(!deserialized.notification_sound);
        assert!(deserialized.codex_enabled);
    }

    // ── extract_message edge cases ──

    #[test]
    fn extract_message_with_korean() {
        let cmd = r#"agent-toast --message="한글 메시지 테스트""#;
        assert_eq!(extract_message(cmd), Some("한글 메시지 테스트".to_string()));
    }

    #[test]
    fn extract_message_with_spaces() {
        let cmd = r#"agent-toast --message="message with multiple spaces""#;
        assert_eq!(
            extract_message(cmd),
            Some("message with multiple spaces".to_string())
        );
    }

    #[test]
    fn extract_message_at_command_start() {
        let cmd = r#"--message="first" agent-toast"#;
        assert_eq!(extract_message(cmd), Some("first".to_string()));
    }

    #[test]
    fn extract_message_multiple_equals() {
        // --message=a=b=c 형태
        let cmd = "agent-toast --message=a=b=c --other";
        assert_eq!(extract_message(cmd), Some("a=b=c".to_string()));
    }

    // ── parse_hook_config_from_json edge cases ──

    #[test]
    fn parse_daemon_only_session_start_not_enabled() {
        // --daemon만 있고 --message가 없으면 session_start_enabled는 false
        let json = r#"{
            "hooks": {
                "SessionStart": [
                    {"hooks": [{"type": "command", "command": "agent-toast --daemon"}]}
                ]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.session_start_enabled);
    }

    #[test]
    fn parse_empty_hooks_object() {
        let json = r#"{"hooks": {}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.stop_enabled);
        assert!(!config.notification_permission_enabled);
    }

    #[test]
    fn parse_hooks_with_null_values() {
        let json = r#"{"hooks": {"Stop": null}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.stop_enabled);
    }

    #[test]
    fn parse_agent_toast_partial_settings() {
        let json = r#"{
            "agent_toast": {
                "locale": "en"
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.locale, "en");
        // 나머지는 기본값
        assert_eq!(config.notification_position, "bottom_right");
        assert!(config.notification_sound);
    }

    #[test]
    fn parse_unknown_notification_matcher_ignored() {
        let json = r#"{
            "hooks": {
                "Notification": [
                    {
                        "matcher": "unknown_matcher",
                        "hooks": [{"type": "command", "command": "agent-toast --message=\"test\""}]
                    }
                ]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.notification_permission_enabled);
        assert!(!config.notification_elicitation_enabled);
        assert!(!config.notification_idle_enabled);
    }

    // ── build_hook_entry variations ──

    #[test]
    fn build_hook_entry_command_with_special_chars() {
        let cmd = r#"agent-toast --message="test \"quoted\" text""#;
        let entry = build_hook_entry(None, cmd, None);
        assert_eq!(entry["hooks"][0]["command"].as_str().unwrap(), cmd);
    }

    #[test]
    fn build_hook_entry_long_command() {
        let long_cmd = format!("agent-toast --message=\"{}\"", "A".repeat(1000));
        let entry = build_hook_entry(None, &long_cmd, None);
        assert_eq!(entry["hooks"][0]["command"].as_str().unwrap(), long_cmd);
    }

    #[test]
    fn build_hook_entry_empty_command() {
        let entry = build_hook_entry(None, "", None);
        assert_eq!(entry["hooks"][0]["command"].as_str().unwrap(), "");
    }

    // ── extract_agent_toast_cmd / is_agent_toast_entry tests ──

    #[test]
    fn agent_toast_entry_detected() {
        let entry = build_hook_entry(None, "C:\\path\\agent-toast.exe --event stop", None);
        assert!(is_agent_toast_entry(&entry));
        assert_eq!(
            extract_agent_toast_cmd(&entry).unwrap(),
            "C:\\path\\agent-toast.exe --event stop"
        );
    }

    #[test]
    fn agent_toast_entry_case_insensitive() {
        // "Agent Toast" (productName 형식) 도 감지
        let entry = build_hook_entry(None, "C:\\path\\Agent Toast.exe --daemon", None);
        assert!(is_agent_toast_entry(&entry));
    }

    #[test]
    fn non_agent_toast_entry_not_detected() {
        let entry = build_hook_entry(None, "some-other-tool --flag", None);
        assert!(!is_agent_toast_entry(&entry));
        assert!(extract_agent_toast_cmd(&entry).is_none());
    }

    #[test]
    fn malformed_entry_not_detected() {
        // "hooks" 키가 없는 구조
        let entry = serde_json::json!({"command": "agent-toast --daemon"});
        assert!(!is_agent_toast_entry(&entry));
    }

    #[test]
    fn empty_hooks_array_not_detected() {
        let entry = serde_json::json!({"hooks": []});
        assert!(!is_agent_toast_entry(&entry));
    }

    #[test]
    fn hooks_without_command_not_detected() {
        let entry = serde_json::json!({"hooks": [{"type": "command"}]});
        assert!(!is_agent_toast_entry(&entry));
    }

    // ── Default function tests ──

    #[test]
    fn default_functions_return_expected_values() {
        assert_eq!(default_title_display_mode(), "project");
        assert!(default_auto_close_on_focus());
        assert_eq!(default_auto_dismiss_seconds(), 0);
        assert_eq!(default_notification_position(), "bottom_right");
        assert!(default_notification_sound());
        assert_eq!(default_notification_monitor(), "primary");
        let locale = default_locale();
        assert!(locale == "ko" || locale == "en", "locale must be ko or en");
    }

    // ── detect_system_locale tests ──

    #[test]
    fn detect_system_locale_returns_valid_locale() {
        let locale = detect_system_locale();
        assert!(
            locale == "ko" || locale == "en",
            "detect_system_locale must return 'ko' or 'en', got '{}'",
            locale
        );
    }

    // ── Path function tests ──

    #[test]
    fn settings_path_ends_with_settings_json() {
        let path = settings_path();
        assert!(path.ends_with("settings.json"));
        assert!(path.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn codex_config_path_ends_with_config_toml() {
        let path = codex_config_path();
        assert!(path.ends_with("config.toml"));
        assert!(path.to_string_lossy().contains(".codex"));
    }

    #[test]
    fn exe_path_unquoted_returns_nonempty() {
        let path = exe_path_unquoted();
        assert!(!path.is_empty());
    }

    #[test]
    fn exe_path_for_shell_is_quoted() {
        let path = exe_path_for_shell();
        assert!(path.starts_with('"'));
        assert!(path.ends_with('"'));
        // 따옴표 제거하면 exe_path_unquoted와 같아야 함
        let unquoted = &path[1..path.len() - 1];
        assert_eq!(unquoted, exe_path_unquoted());
    }

    // ── parse_hook_config_from_json: individual hook type messages ──

    #[test]
    fn parse_session_end_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "SessionEnd": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"세션 종료됨\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.session_end_enabled);
        assert_eq!(config.session_end_message, "세션 종료됨");
    }

    #[test]
    fn parse_subagent_stop_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "SubagentStop": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"서브 종료\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.subagent_stop_enabled);
        assert_eq!(config.subagent_stop_message, "서브 종료");
    }

    #[test]
    fn parse_pre_compact_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "PreCompact": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"압축 시작\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.pre_compact_enabled);
        assert_eq!(config.pre_compact_message, "압축 시작");
    }

    #[test]
    fn parse_setup_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "Setup": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"셋업 완료\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.setup_enabled);
        assert_eq!(config.setup_message, "셋업 완료");
    }

    #[test]
    fn parse_user_prompt_submit_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "UserPromptSubmit": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"프롬프트 제출\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.user_prompt_submit_enabled);
        assert_eq!(config.user_prompt_submit_message, "프롬프트 제출");
    }

    #[test]
    fn parse_pre_tool_use_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"도구 실행 시작\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.pre_tool_use_enabled);
        assert_eq!(config.pre_tool_use_message, "도구 실행 시작");
    }

    #[test]
    fn parse_post_tool_use_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "PostToolUse": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"도구 완료\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.post_tool_use_enabled);
        assert_eq!(config.post_tool_use_message, "도구 완료");
    }

    #[test]
    fn parse_post_tool_use_failure_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "PostToolUseFailure": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event error --message=\"도구 에러\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.post_tool_use_failure_enabled);
        assert_eq!(config.post_tool_use_failure_message, "도구 에러");
    }

    #[test]
    fn parse_permission_request_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "PermissionRequest": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event user_input_required --message=\"권한 요청\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.permission_request_enabled);
        assert_eq!(config.permission_request_message, "권한 요청");
    }

    #[test]
    fn parse_subagent_start_hook_message() {
        let json = serde_json::json!({
            "hooks": {
                "SubagentStart": [{
                    "hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"서브 시작\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.subagent_start_enabled);
        assert_eq!(config.subagent_start_message, "서브 시작");
    }

    // ── parse: agent_toast settings with codex_enabled ──

    #[test]
    fn parse_agent_toast_codex_enabled_true() {
        let json = r#"{"agent_toast": {"codex_enabled": true}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.codex_enabled);
    }

    #[test]
    fn parse_agent_toast_codex_enabled_false() {
        let json = r#"{"agent_toast": {"codex_enabled": false}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.codex_enabled);
    }

    // ── parse: Notification hook without matcher field ──

    #[test]
    fn parse_notification_without_matcher_ignored() {
        let json = serde_json::json!({
            "hooks": {
                "Notification": [{
                    "hooks": [{"type": "command", "command": "agent-toast --message=\"test\""}]
                }]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        // matcher가 빈 문자열이면 어떤 notification 타입에도 매칭 안 됨
        assert!(!config.notification_permission_enabled);
        assert!(!config.notification_elicitation_enabled);
        assert!(!config.notification_idle_enabled);
    }

    // ── parse: session_start with message → enabled ──

    #[test]
    fn parse_session_start_with_message_enabled() {
        let json = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {"hooks": [{"type": "command", "command": "agent-toast --daemon"}]},
                    {"hooks": [{"type": "command", "command": "agent-toast --event session_start --message=\"시작!\""}]}
                ]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.session_start_enabled);
        assert_eq!(config.session_start_message, "시작!");
    }

    // ── parse: agent_toast settings all fields ──

    #[test]
    fn parse_agent_toast_all_settings() {
        let json = serde_json::json!({
            "agent_toast": {
                "title_display_mode": "window",
                "auto_close_on_focus": false,
                "auto_dismiss_seconds": 15,
                "notification_position": "top_right",
                "notification_sound": false,
                "notification_monitor": "2",
                "locale": "en",
                "codex_enabled": true
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert_eq!(config.title_display_mode, "window");
        assert!(!config.auto_close_on_focus);
        assert_eq!(config.auto_dismiss_seconds, 15);
        assert_eq!(config.notification_position, "top_right");
        assert!(!config.notification_sound);
        assert_eq!(config.notification_monitor, "2");
        assert_eq!(config.locale, "en");
        assert!(config.codex_enabled);
    }

    // ── extract_agent_toast_cmd: case insensitive patterns ──

    #[test]
    fn extract_agent_toast_cmd_mixed_case() {
        let entry = build_hook_entry(None, "C:\\Program Files\\Agent-Toast.exe --daemon", None);
        assert!(extract_agent_toast_cmd(&entry).is_some());
    }

    #[test]
    fn extract_agent_toast_cmd_product_name_with_space() {
        let entry = build_hook_entry(
            None,
            "C:\\Users\\test\\Agent Toast\\Agent Toast.exe --daemon",
            None,
        );
        assert!(extract_agent_toast_cmd(&entry).is_some());
    }

    #[test]
    fn extract_agent_toast_cmd_unrelated_tool() {
        let entry = build_hook_entry(None, "notify-send 'hello'", None);
        assert!(extract_agent_toast_cmd(&entry).is_none());
    }

    // ── HookConfig default locale-dependent messages ──

    #[test]
    fn hook_config_default_messages_not_empty() {
        let config = HookConfig::default();
        assert!(!config.stop_message.is_empty());
        assert!(!config.permission_request_message.is_empty());
        assert!(!config.notification_permission_message.is_empty());
        assert!(!config.notification_elicitation_message.is_empty());
        assert!(!config.setup_message.is_empty());
        assert!(!config.session_start_message.is_empty());
        assert!(!config.session_end_message.is_empty());
        assert!(!config.subagent_start_message.is_empty());
        assert!(!config.subagent_stop_message.is_empty());
        assert!(!config.user_prompt_submit_message.is_empty());
        assert!(!config.pre_tool_use_message.is_empty());
        assert!(!config.post_tool_use_message.is_empty());
        assert!(!config.post_tool_use_failure_message.is_empty());
        assert!(!config.pre_compact_message.is_empty());
        assert!(!config.notification_idle_message.is_empty());
    }

    // ── build_hook_entry: verify structure ──

    #[test]
    fn build_hook_entry_structure_correct() {
        let entry = build_hook_entry(Some("test_matcher"), "test-cmd --flag", Some(10));
        // entry는 Object
        assert!(entry.is_object());
        // matcher 포함
        assert_eq!(entry["matcher"].as_str().unwrap(), "test_matcher");
        // hooks 배열 길이 1
        assert_eq!(entry["hooks"].as_array().unwrap().len(), 1);
        // hook 타입은 command
        assert_eq!(entry["hooks"][0]["type"].as_str().unwrap(), "command");
        // command 값
        assert_eq!(
            entry["hooks"][0]["command"].as_str().unwrap(),
            "test-cmd --flag"
        );
        // timeout 값
        assert_eq!(entry["hooks"][0]["timeout"].as_u64().unwrap(), 10);
    }

    #[test]
    fn build_hook_entry_no_timeout_field_when_none() {
        let entry = build_hook_entry(None, "cmd", None);
        assert!(entry["hooks"][0]["timeout"].is_null());
    }

    // ── Boundary value tests ──

    #[test]
    fn parse_auto_dismiss_seconds_u64_to_u32_boundary() {
        // u32::MAX 값 (4294967295)
        let json = r#"{"agent_toast": {"auto_dismiss_seconds": 4294967295}}"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.auto_dismiss_seconds, u32::MAX);
    }

    #[test]
    fn parse_auto_dismiss_seconds_zero() {
        let json = r#"{"agent_toast": {"auto_dismiss_seconds": 0}}"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.auto_dismiss_seconds, 0);
    }

    #[test]
    fn parse_auto_dismiss_seconds_missing_defaults_to_zero() {
        let json = r#"{"agent_toast": {}}"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.auto_dismiss_seconds, 0);
    }

    #[test]
    fn extract_message_message_at_end_of_string() {
        let cmd = "agent-toast --message=last";
        assert_eq!(extract_message(cmd), Some("last".to_string()));
    }

    #[test]
    fn extract_message_very_long_message() {
        let long_msg = "가".repeat(5000);
        let cmd = format!("agent-toast --message=\"{}\"", long_msg);
        assert_eq!(extract_message(&cmd), Some(long_msg));
    }

    #[test]
    fn extract_message_only_flag_equals() {
        // --message= (값 없이 = 만)
        let cmd = "agent-toast --message= --other";
        assert_eq!(extract_message(cmd), Some("".to_string()));
    }

    #[test]
    fn extract_message_only_flag_equals_at_end() {
        let cmd = "agent-toast --message=";
        assert_eq!(extract_message(cmd), Some("".to_string()));
    }

    #[test]
    fn parse_hooks_with_empty_array() {
        let json = r#"{"hooks": {"Stop": [], "Notification": []}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.stop_enabled);
        assert!(!config.notification_permission_enabled);
    }

    #[test]
    fn parse_hooks_with_many_non_agent_toast_entries() {
        let json = serde_json::json!({
            "hooks": {
                "Stop": [
                    {"hooks": [{"type": "command", "command": "tool-1 --flag"}]},
                    {"hooks": [{"type": "command", "command": "tool-2 --flag"}]},
                    {"hooks": [{"type": "command", "command": "tool-3 --flag"}]},
                    {"hooks": [{"type": "command", "command": "tool-4 --flag"}]},
                    {"hooks": [{"type": "command", "command": "tool-5 --flag"}]}
                ]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(!config.stop_enabled);
    }

    #[test]
    fn parse_hooks_agent_toast_last_in_array() {
        // 배열 마지막에 agent-toast가 있어도 감지
        let json = serde_json::json!({
            "hooks": {
                "Stop": [
                    {"hooks": [{"type": "command", "command": "other-tool --done"}]},
                    {"hooks": [{"type": "command", "command": "another-tool --notify"}]},
                    {"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message=\"마지막\""}]}
                ]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.stop_enabled);
        assert_eq!(config.stop_message, "마지막");
    }

    #[test]
    fn build_hook_entry_timeout_zero() {
        let entry = build_hook_entry(None, "cmd", Some(0));
        assert_eq!(entry["hooks"][0]["timeout"].as_u64().unwrap(), 0);
    }

    #[test]
    fn build_hook_entry_timeout_u32_max() {
        let entry = build_hook_entry(None, "cmd", Some(u32::MAX));
        assert_eq!(
            entry["hooks"][0]["timeout"].as_u64().unwrap(),
            u32::MAX as u64
        );
    }

    #[test]
    fn build_hook_entry_unicode_matcher() {
        let entry = build_hook_entry(Some("한글_매처"), "agent-toast --test", None);
        assert_eq!(entry["matcher"].as_str().unwrap(), "한글_매처");
    }

    #[test]
    fn extract_agent_toast_cmd_multiple_hooks_in_entry() {
        // hooks 배열에 여러 hook이 있고 두 번째가 agent-toast인 경우
        let entry = serde_json::json!({
            "hooks": [
                {"type": "command", "command": "other-tool --flag"},
                {"type": "command", "command": "agent-toast --event test"}
            ]
        });
        let result = extract_agent_toast_cmd(&entry);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "agent-toast --event test");
    }

    #[test]
    fn extract_agent_toast_cmd_hook_without_type() {
        let entry = serde_json::json!({
            "hooks": [{"command": "agent-toast --test"}]
        });
        // "type" 필드가 없어도 "command" 필드만 있으면 감지
        let result = extract_agent_toast_cmd(&entry);
        assert!(result.is_some());
    }

    #[test]
    fn hook_config_auto_dismiss_u32_max() {
        let config = HookConfig {
            auto_dismiss_seconds: u32::MAX,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.auto_dismiss_seconds, u32::MAX);
    }

    #[test]
    fn parse_json_with_extra_top_level_keys() {
        // agent_toast나 hooks 외의 키가 있어도 무시
        let json = r#"{
            "mcpServers": {},
            "permissions": [],
            "agent_toast": {"locale": "en"},
            "hooks": {}
        }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.locale, "en");
    }

    #[test]
    fn parse_notification_multiple_same_matcher() {
        // 같은 matcher가 여러 번 등장하면 마지막 메시지가 적용
        let json = serde_json::json!({
            "hooks": {
                "Notification": [
                    {
                        "matcher": "permission_prompt",
                        "hooks": [{"type": "command", "command": "agent-toast --message=\"첫번째\""}]
                    },
                    {
                        "matcher": "permission_prompt",
                        "hooks": [{"type": "command", "command": "agent-toast --message=\"두번째\""}]
                    }
                ]
            }
        });
        let config = parse_hook_config_from_json(&json.to_string());
        assert!(config.notification_permission_enabled);
        assert_eq!(config.notification_permission_message, "두번째");
    }

    // ── SessionStart daemon hook: no timeout ──

    #[test]
    fn session_start_daemon_hook_has_no_timeout() {
        // spawn 방식이므로 --daemon 훅에 timeout이 없어야 함
        let entry = build_hook_entry(None, "agent-toast --daemon", None);
        assert!(entry["hooks"][0]["timeout"].is_null());
    }

    #[test]
    fn session_start_daemon_hook_command_format() {
        // SessionStart 훅 커맨드는 "{exe} --daemon" 형식이어야 함
        let exe = r#""C:\path\agent-toast.exe""#;
        let entry = build_hook_entry(None, &format!("{} --daemon", exe), None);
        let cmd = entry["hooks"][0]["command"].as_str().unwrap();
        assert!(cmd.ends_with("--daemon"));
        assert!(cmd.contains("agent-toast"));
    }

    #[test]
    fn session_start_daemon_hook_no_matcher() {
        // SessionStart --daemon 훅은 matcher가 없어야 함 (모든 세션에서 실행)
        let entry = build_hook_entry(None, "agent-toast --daemon", None);
        assert!(entry["matcher"].is_null());
    }

    #[test]
    fn parse_daemon_with_timeout_still_not_enabled() {
        // --daemon에 timeout이 있어도 --message가 없으면 session_start_enabled는 false
        let json = r#"{
            "hooks": {
                "SessionStart": [
                    {"hooks": [{"type": "command", "command": "agent-toast --daemon", "timeout": 5}]}
                ]
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(!config.session_start_enabled);
    }

    #[test]
    fn hook_config_default_has_http_fields() {
        let cfg = HookConfig::default();
        assert!(!cfg.http_enabled);
        assert_eq!(cfg.http_port, 38787);
        assert!(cfg.show_hostname);
    }

    #[test]
    fn hook_config_deserializes_without_http_fields_uses_defaults() {
        // 세 신규 키(http_enabled, http_port, show_hostname)만 빠진 JSON.
        // 나머지 필드는 #[serde(default)] 없는 기존 필드이므로 모두 포함해야 역직렬화 성공.
        let json = r#"{
            "stop_enabled": false,
            "stop_message": "",
            "permission_request_enabled": false,
            "permission_request_message": "",
            "notification_permission_enabled": false,
            "notification_permission_message": "",
            "notification_elicitation_enabled": false,
            "notification_elicitation_message": "",
            "setup_enabled": false,
            "setup_message": "",
            "session_start_enabled": false,
            "session_start_message": "",
            "session_end_enabled": false,
            "session_end_message": "",
            "subagent_start_enabled": false,
            "subagent_start_message": "",
            "subagent_stop_enabled": false,
            "subagent_stop_message": "",
            "user_prompt_submit_enabled": false,
            "user_prompt_submit_message": "",
            "pre_tool_use_enabled": false,
            "pre_tool_use_message": "",
            "post_tool_use_enabled": false,
            "post_tool_use_message": "",
            "post_tool_use_failure_enabled": false,
            "post_tool_use_failure_message": "",
            "pre_compact_enabled": false,
            "pre_compact_message": "",
            "notification_idle_enabled": false,
            "notification_idle_message": ""
        }"#;
        let cfg: HookConfig = serde_json::from_str(json).unwrap();
        assert!(!cfg.http_enabled);
        assert_eq!(cfg.http_port, 38787);
        assert!(cfg.show_hostname);
    }

    #[test]
    fn parse_reads_http_fields_from_agent_toast_block() {
        // 실제 저장 포맷(agent_toast 하위 키)을 수동 파서가 읽어오는지 검증.
        let json = r#"{
            "agent_toast": {
                "http_enabled": true,
                "http_port": 9999,
                "show_hostname": false
            }
        }"#;
        let cfg = parse_hook_config_from_json(json);
        assert!(cfg.http_enabled);
        assert_eq!(cfg.http_port, 9999);
        assert!(!cfg.show_hostname);
    }

    #[test]
    fn parse_missing_http_block_falls_back_to_defaults() {
        let json = r#"{"agent_toast": {}}"#;
        let cfg = parse_hook_config_from_json(json);
        assert!(!cfg.http_enabled);
        assert_eq!(cfg.http_port, 38787);
        assert!(cfg.show_hostname);
    }

    #[test]
    fn write_agent_toast_settings_roundtrip_preserves_http_fields() {
        // save 경로가 세 키를 실제로 파일에 쓰는지, 그리고 parse 가 다시 읽는지 왕복 검증.
        let cfg = HookConfig {
            http_enabled: true,
            http_port: 7777,
            show_hostname: false,
            ..HookConfig::default()
        };

        let mut root = Value::Object(Default::default());
        write_agent_toast_settings(&mut root, &cfg);

        let serialized = serde_json::to_string(&root).unwrap();
        let parsed = parse_hook_config_from_json(&serialized);

        assert!(
            parsed.http_enabled,
            "http_enabled 이 저장/로드를 거쳐 유지되어야 함"
        );
        assert_eq!(parsed.http_port, 7777);
        assert!(!parsed.show_hostname);
    }

    #[test]
    fn write_agent_toast_settings_preserves_unknown_keys() {
        // agent_toast 하위에 미래 버전이 추가할 수도 있는 미지의 키를 덮어쓰지 않는지 확인.
        let mut root: Value =
            serde_json::from_str(r#"{"agent_toast": {"future_key": "keep-me"}}"#).unwrap();
        let cfg = HookConfig::default();
        write_agent_toast_settings(&mut root, &cfg);
        assert_eq!(
            root["agent_toast"]["future_key"].as_str(),
            Some("keep-me"),
            "미지의 키는 보존되어야 함",
        );
    }
}
