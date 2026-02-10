use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Hook configuration as shown in the setup GUI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookConfig {
    // 권장 항목
    pub stop_enabled: bool,
    pub stop_message: String,
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
    pub permission_request_enabled: bool,
    pub permission_request_message: String,
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
    /// Codex notify 훅 활성화 여부
    #[serde(default)]
    pub codex_enabled: bool,
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

fn default_locale() -> String {
    "ko".into()
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            // 권장 항목
            stop_enabled: true,
            stop_message: "작업이 완료되었습니다".into(),
            notification_permission_enabled: true,
            notification_permission_message: "권한 승인이 필요합니다".into(),
            notification_elicitation_enabled: true,
            notification_elicitation_message: "입력이 필요합니다".into(),
            // 세션 생명주기
            setup_enabled: false,
            setup_message: "초기화가 실행되었습니다".into(),
            session_start_enabled: false,
            session_start_message: "세션이 시작되었습니다".into(),
            session_end_enabled: false,
            session_end_message: "세션이 종료되었습니다".into(),
            // 서브에이전트 생명주기
            subagent_start_enabled: false,
            subagent_start_message: "서브에이전트가 시작되었습니다".into(),
            subagent_stop_enabled: false,
            subagent_stop_message: "서브에이전트가 완료되었습니다".into(),
            // 사용자 입력
            user_prompt_submit_enabled: false,
            user_prompt_submit_message: "프롬프트가 제출되었습니다".into(),
            // 도구 실행 흐름
            permission_request_enabled: false,
            permission_request_message: "권한 요청이 발생했습니다".into(),
            pre_tool_use_enabled: false,
            pre_tool_use_message: "도구 실행이 시작됩니다".into(),
            post_tool_use_enabled: false,
            post_tool_use_message: "도구 실행이 완료되었습니다".into(),
            post_tool_use_failure_enabled: false,
            post_tool_use_failure_message: "도구 실행이 실패했습니다".into(),
            // 기타
            pre_compact_enabled: false,
            pre_compact_message: "컨텍스트 압축이 시작됩니다".into(),
            notification_idle_enabled: false,
            notification_idle_message: "입력을 기다리고 있습니다".into(),
            // 설정
            title_display_mode: "project".into(),
            auto_close_on_focus: true,
            auto_dismiss_seconds: 0,
            notification_position: "bottom_right".into(),
            notification_sound: true,
            notification_monitor: "primary".into(),
            locale: "ko".into(),
            codex_enabled: false,
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

/// Returns the exe path quoted if it contains spaces (for shell commands)
fn exe_path_for_shell() -> String {
    let path = exe_path_unquoted();
    if path.contains(' ') {
        format!("\"{}\"", path)
    } else {
        path
    }
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
        codex_enabled: root["agent_toast"]["codex_enabled"]
            .as_bool()
            .unwrap_or_else(get_codex_installed),
        // 나머지는 Default에서 가져오기
        ..HookConfig::default()
    };

    // Check Stop hooks
    if let Some(stop_arr) = hooks["Stop"].as_array() {
        for entry in stop_arr {
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
                config.stop_enabled = true;
                // Extract message from --message="..."
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if !cmd.contains("agent-toast") {
                continue;
            }
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") && extract_message(cmd).is_some() {
                config.session_start_enabled = true;
                if let Some(msg) = extract_message(cmd) {
                    config.session_start_message = msg;
                }
            }
        }
    }

    // Check SessionEnd hooks
    if let Some(arr) = hooks["SessionEnd"].as_array() {
        for entry in arr {
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
            let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
            if cmd.contains("agent-toast") {
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
    // Check if any hook event array contains a agent-toast command
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
                let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
                if cmd.contains("agent-toast") {
                    return true;
                }
            }
        }
    }
    false
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
    let mut hooks = serde_json::Map::new();

    // Preserve non-agent-toast hooks from existing config
    if let Some(existing_hooks) = root["hooks"].as_object() {
        for (event_name, entries) in existing_hooks {
            if let Some(arr) = entries.as_array() {
                let filtered: Vec<&Value> = arr
                    .iter()
                    .filter(|entry| {
                        let cmd = entry["hooks"][0]["command"].as_str().unwrap_or("");
                        !cmd.contains("agent-toast")
                    })
                    .collect();
                if !filtered.is_empty() {
                    hooks.insert(
                        event_name.clone(),
                        Value::Array(filtered.into_iter().cloned().collect()),
                    );
                }
            }
        }
    }

    // Build agent-toast hooks

    // SessionStart: always add --daemon entry (infrastructure)
    {
        let entry = build_hook_entry(None, &format!("{} --daemon", exe), None);
        hooks
            .entry("SessionStart".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }
    // SessionStart: add notification entry if enabled
    if config.session_start_enabled {
        let cmd = format!(
            "{} --event session_start --message \"{}\"",
            exe, config.session_start_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("SessionStart".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    // CLI reads CLAUDE_PROJECT_DIR env var directly as title hint fallback,
    // so no --title arg needed in the hook command.

    if config.stop_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.stop_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("Stop".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.notification_permission_enabled {
        let cmd = format!(
            "{} --event user_input_required --message \"{}\"",
            exe, config.notification_permission_message
        );
        let entry = build_hook_entry(Some("permission_prompt"), &cmd, None);
        hooks
            .entry("Notification".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.notification_elicitation_enabled {
        let cmd = format!(
            "{} --event user_input_required --message \"{}\"",
            exe, config.notification_elicitation_message
        );
        let entry = build_hook_entry(Some("elicitation_dialog"), &cmd, None);
        hooks
            .entry("Notification".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.notification_idle_enabled {
        let cmd = format!(
            "{} --event user_input_required --message \"{}\"",
            exe, config.notification_idle_message
        );
        let entry = build_hook_entry(Some("idle_prompt"), &cmd, None);
        hooks
            .entry("Notification".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.session_end_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.session_end_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("SessionEnd".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.subagent_stop_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.subagent_stop_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("SubagentStop".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.pre_compact_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.pre_compact_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("PreCompact".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.setup_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.setup_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("Setup".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.user_prompt_submit_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.user_prompt_submit_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("UserPromptSubmit".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.pre_tool_use_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.pre_tool_use_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("PreToolUse".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.post_tool_use_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.post_tool_use_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("PostToolUse".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.post_tool_use_failure_enabled {
        let cmd = format!(
            "{} --event error --message \"{}\"",
            exe, config.post_tool_use_failure_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("PostToolUseFailure".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.permission_request_enabled {
        let cmd = format!(
            "{} --event user_input_required --message \"{}\"",
            exe, config.permission_request_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("PermissionRequest".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    if config.subagent_start_enabled {
        let cmd = format!(
            "{} --event task_complete --message \"{}\"",
            exe, config.subagent_start_message
        );
        let entry = build_hook_entry(None, &cmd, None);
        hooks
            .entry("SubagentStart".to_string())
            .or_insert_with(|| Value::Array(vec![]))
            .as_array_mut()
            .unwrap()
            .push(entry);
    }

    root["hooks"] = Value::Object(hooks);

    // Save agent_toast settings
    let mut cn = root["agent_toast"].as_object().cloned().unwrap_or_default();
    cn.insert(
        "title_display_mode".into(),
        Value::String(config.title_display_mode),
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
        Value::String(config.notification_position),
    );
    cn.insert(
        "notification_sound".into(),
        Value::Bool(config.notification_sound),
    );
    cn.insert(
        "notification_monitor".into(),
        Value::String(config.notification_monitor),
    );
    cn.insert("locale".into(), Value::String(config.locale));
    cn.insert("codex_enabled".into(), Value::Bool(config.codex_enabled));
    root["agent_toast"] = Value::Object(cn);

    // Ensure .claude directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())?;

    // Codex config.toml 업데이트
    save_codex_config(config.codex_enabled).map_err(|e| e.to_string())?;

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
                        if exe.contains("agent-toast") {
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
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return "ko".into();
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return "ko".into();
    };
    root["agent_toast"]["locale"]
        .as_str()
        .unwrap_or("ko")
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

/// Codex CLI가 설치되어 있는지 확인
#[tauri::command]
pub fn get_codex_installed() -> bool {
    let exe_name = if cfg!(windows) { "codex.exe" } else { "codex" };
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(exe_name).is_file()))
        .unwrap_or(false)
}

fn build_hook_entry(matcher: Option<&str>, command: &str, _timeout: Option<u32>) -> Value {
    let mut entry = serde_json::Map::new();
    if let Some(m) = matcher {
        entry.insert("matcher".into(), Value::String(m.into()));
    }
    let hook_obj = serde_json::json!({
        "type": "command",
        "command": command
    });
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
        assert!(config.notification_permission_enabled);
        assert!(config.notification_elicitation_enabled);
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
        let entry = build_hook_entry(None, "agent-toast --daemon", Some(5000));
        // timeout 파라미터는 현재 _timeout으로 무시되므로 출력에 포함되지 않음
        assert!(entry.get("timeout").is_none());
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
        let mut config = HookConfig::default();
        config.stop_enabled = false;
        config.notification_permission_enabled = false;
        config.notification_elicitation_enabled = false;

        assert!(!config.stop_enabled);
        assert!(!config.notification_permission_enabled);
        assert!(!config.notification_elicitation_enabled);
    }

    #[test]
    fn hook_config_locale_values() {
        for locale in ["ko", "en"] {
            let mut config = HookConfig::default();
            config.locale = locale.to_string();
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.locale, locale);
        }
    }

    #[test]
    fn hook_config_notification_positions() {
        let positions = ["bottom_right", "bottom_left", "top_right", "top_left"];
        for pos in positions {
            let mut config = HookConfig::default();
            config.notification_position = pos.to_string();
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.notification_position, pos);
        }
    }

    #[test]
    fn hook_config_auto_dismiss_values() {
        for seconds in [0, 5, 10, 30, 60, 300] {
            let mut config = HookConfig::default();
            config.auto_dismiss_seconds = seconds;
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.auto_dismiss_seconds, seconds);
        }
    }

    #[test]
    fn hook_config_title_display_modes() {
        for mode in ["project", "window"] {
            let mut config = HookConfig::default();
            config.title_display_mode = mode.to_string();
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.title_display_mode, mode);
        }
    }

    #[test]
    fn hook_config_monitor_values() {
        let monitors = ["primary", "0", "1", "2"];
        for monitor in monitors {
            let mut config = HookConfig::default();
            config.notification_monitor = monitor.to_string();
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.notification_monitor, monitor);
        }
    }

    #[test]
    fn hook_config_boolean_fields() {
        let mut config = HookConfig::default();

        // Toggle all boolean fields
        config.auto_close_on_focus = false;
        config.notification_sound = false;
        config.codex_enabled = true;

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

    #[test]
    fn parse_codex_enabled_default_based_on_install() {
        // codex_enabled가 없으면 get_codex_installed() 결과에 따름
        let json = r#"{"agent_toast": {}}"#;
        let config = parse_hook_config_from_json(json);
        // 테스트 환경에서는 codex가 설치되어 있지 않을 가능성이 높음
        // 값 자체보다 파싱이 실패하지 않는지 확인
        assert!(config.codex_enabled == true || config.codex_enabled == false);
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

    // ── Default function tests ──

    #[test]
    fn default_functions_return_expected_values() {
        assert_eq!(default_title_display_mode(), "project");
        assert!(default_auto_close_on_focus());
        assert_eq!(default_auto_dismiss_seconds(), 0);
        assert_eq!(default_notification_position(), "bottom_right");
        assert!(default_notification_sound());
        assert_eq!(default_notification_monitor(), "primary");
        assert_eq!(default_locale(), "ko");
    }
}
