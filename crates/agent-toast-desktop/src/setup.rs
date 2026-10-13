use agent_toast_core::hook_config::{is_agent_toast_cmd, merge_agent_toast_hooks, HookEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

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
    // Codex 전용
    /// 질문(request_user_input) 알림. Claude 질문은 권한 요청 훅으로 들어온다.
    #[serde(default = "default_true")]
    pub question_enabled: bool,
    #[serde(default = "default_question_message")]
    pub question_message: String,
    #[serde(default)]
    pub post_compact_enabled: bool,
    #[serde(default = "default_post_compact_message")]
    pub post_compact_message: String,
    #[serde(default)]
    pub interrupt_enabled: bool,
    #[serde(default = "default_interrupt_message")]
    pub interrupt_message: String,
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
    /// 커스텀 알림 소리 파일 경로 (앱 sounds 폴더 내 복사본). None = 시스템 기본음
    #[serde(default)]
    pub notification_sound_file: Option<String>,
    /// 알림 표시 모니터: "primary", "0", "1", ...
    #[serde(default = "default_notification_monitor")]
    pub notification_monitor: String,
    /// UI 언어: "ko", "en"
    #[serde(default = "default_locale")]
    pub locale: String,
    /// 세션 시작 시 앱 자동 실행 여부 (--daemon 훅)
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    /// Claude Code 에 훅 등록 여부 (`~/.claude/settings.json`)
    #[serde(default = "default_true")]
    pub claude_enabled: bool,
    /// Codex 에 훅 등록 여부 (`~/.codex/hooks.json`)
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
    /// UI 테마: "system" | "light" | "dark" (기본 system = OS 설정 추종)
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 알림 본문에 에이전트의 마지막 메시지(또는 도구 설명)를 동적으로 사용 (기본 true).
    /// 켜지면 모든 알림 훅 커맨드에 `--dynamic` 이 붙고, 입력칸 문구는 추출 실패 시
    /// fallback 으로 동작한다.
    #[serde(default = "default_dynamic_message_enabled")]
    pub dynamic_message_enabled: bool,
    /// 알림 토스트 액센트 바: "left" | "none"
    #[serde(default = "default_toast_bar")]
    pub toast_bar: String,
    /// 알림 토스트 테두리: "subtle"(은은) | "accent"(이벤트색 강조)
    #[serde(default = "default_toast_border")]
    pub toast_border: String,
    /// 알림 토스트 이펙트 (중첩 가능): "ring" | "breathe" | "pulse" | "shimmer"
    #[serde(default)]
    pub toast_effects: Vec<String>,
    /// 알림 토스트 배경: "glow"(코너 글로우) | "tint"(전체 틴트) | "flat"
    #[serde(default = "default_toast_body")]
    pub toast_body: String,
    /// 알림 토스트 밀도: "comfortable"(기본) | "compact"(높이 축소)
    #[serde(default = "default_toast_density")]
    pub toast_density: String,
    /// 알림 토스트 본문 폰트 패밀리명. "" = 기본(번들 Pretendard)
    #[serde(default)]
    pub toast_font_sans: String,
    /// 알림 토스트 코드 폰트 패밀리명. "" = 기본(번들 D2Coding)
    #[serde(default)]
    pub toast_font_mono: String,
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

fn default_theme() -> String {
    "system".into()
}

fn default_dynamic_message_enabled() -> bool {
    true
}

fn default_toast_bar() -> String {
    "none".into()
}

fn default_toast_border() -> String {
    "subtle".into()
}

fn default_toast_body() -> String {
    "glow".into()
}

fn default_toast_density() -> String {
    "comfortable".into()
}

fn default_locale() -> String {
    detect_system_locale()
}

fn default_true() -> bool {
    true
}

fn localized(ko: &str, en: &str) -> String {
    if detect_system_locale() == "ko" {
        ko
    } else {
        en
    }
    .into()
}

fn default_question_message() -> String {
    localized("답변이 필요합니다", "Your answer is needed")
}

fn default_post_compact_message() -> String {
    localized("컨텍스트 압축이 끝났습니다", "Context compaction finished")
}

fn default_interrupt_message() -> String {
    localized("진행 중인 응답이 중단되었습니다", "Response interrupted")
}

/// Whether an agent's home folder (`~/.claude`, `~/.codex`) exists, i.e. the
/// agent has been run on this machine.
fn agent_home_exists(dir: &str) -> bool {
    dirs::home_dir().is_some_and(|home| home.join(dir).is_dir())
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPresence {
    pub claude: bool,
    pub codex: bool,
}

/// 설치 흔적이 있는 에이전트. 훅 탭이 "설치되지 않음" 표시에 쓴다.
#[tauri::command]
pub fn get_agent_presence() -> AgentPresence {
    AgentPresence {
        claude: agent_home_exists(".claude"),
        codex: agent_home_exists(".codex"),
    }
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
            // Codex 전용
            question_enabled: true,
            question_message: default_question_message(),
            post_compact_enabled: false,
            post_compact_message: default_post_compact_message(),
            interrupt_enabled: false,
            interrupt_message: default_interrupt_message(),
            // 설정
            title_display_mode: "project".into(),
            auto_close_on_focus: true,
            auto_dismiss_seconds: 0,
            http_enabled: false,
            http_port: default_http_port(),
            show_hostname: true,
            notification_position: "bottom_right".into(),
            notification_sound: true,
            notification_sound_file: None,
            notification_monitor: "primary".into(),
            locale,
            auto_start: true,
            // 첫 실행: 설치된 에이전트를 켠다. 둘 다 흔적이 없으면 Claude 만.
            claude_enabled: agent_home_exists(".claude") || !agent_home_exists(".codex"),
            codex_enabled: agent_home_exists(".codex"),
            version: String::new(),
            theme: default_theme(),
            dynamic_message_enabled: default_dynamic_message_enabled(),
            toast_bar: default_toast_bar(),
            toast_border: default_toast_border(),
            toast_effects: Vec::new(),
            toast_body: default_toast_body(),
            toast_density: default_toast_density(),
            toast_font_sans: String::new(),
            toast_font_mono: String::new(),
        }
    }
}

/// Where one agent registers a notification row: hook event and optional matcher.
type HookTarget = Option<(&'static str, Option<&'static str>)>;

/// One notification row of the hooks tab. The row's switch and message are
/// shared; each agent that supports it gets its own hook.
struct EventSpec {
    /// `HookConfig` field prefix (`<key>_enabled`, `<key>_message`)
    key: &'static str,
    /// `--event` value, which picks the toast's color
    cli_event: &'static str,
    claude: HookTarget,
    codex: HookTarget,
}

/// Claude's `AskUserQuestion` arrives as a PermissionRequest, so `question`
/// only needs a Codex hook. Codex has no Notification event, so those rows stay
/// Claude-only.
const EVENT_SPECS: &[EventSpec] = &[
    EventSpec {
        key: "session_start",
        cli_event: "session_start",
        claude: Some(("SessionStart", None)),
        codex: Some(("SessionStart", None)),
    },
    EventSpec {
        key: "stop",
        cli_event: "task_complete",
        claude: Some(("Stop", None)),
        codex: Some(("Stop", None)),
    },
    EventSpec {
        key: "notification_permission",
        cli_event: "user_input_required",
        claude: Some(("Notification", Some("permission_prompt"))),
        codex: None,
    },
    EventSpec {
        key: "notification_elicitation",
        cli_event: "user_input_required",
        claude: Some(("Notification", Some("elicitation_dialog"))),
        codex: None,
    },
    EventSpec {
        key: "notification_idle",
        cli_event: "user_input_required",
        claude: Some(("Notification", Some("idle_prompt"))),
        codex: None,
    },
    EventSpec {
        key: "session_end",
        cli_event: "task_complete",
        claude: Some(("SessionEnd", None)),
        codex: Some(("SessionEnd", None)),
    },
    EventSpec {
        key: "subagent_stop",
        cli_event: "task_complete",
        claude: Some(("SubagentStop", None)),
        codex: Some(("SubagentStop", None)),
    },
    EventSpec {
        key: "pre_compact",
        cli_event: "task_complete",
        claude: Some(("PreCompact", None)),
        codex: Some(("PreCompact", None)),
    },
    EventSpec {
        key: "setup",
        cli_event: "task_complete",
        claude: Some(("Setup", None)),
        codex: None,
    },
    EventSpec {
        key: "user_prompt_submit",
        cli_event: "task_complete",
        claude: Some(("UserPromptSubmit", None)),
        codex: Some(("UserPromptSubmit", None)),
    },
    EventSpec {
        key: "pre_tool_use",
        cli_event: "task_complete",
        claude: Some(("PreToolUse", None)),
        codex: Some(("PreToolUse", None)),
    },
    EventSpec {
        key: "post_tool_use",
        cli_event: "task_complete",
        claude: Some(("PostToolUse", None)),
        codex: Some(("PostToolUse", None)),
    },
    EventSpec {
        key: "post_tool_use_failure",
        cli_event: "error",
        claude: Some(("PostToolUseFailure", None)),
        codex: None,
    },
    EventSpec {
        key: "permission_request",
        cli_event: "user_input_required",
        claude: Some(("PermissionRequest", None)),
        codex: Some(("PermissionRequest", None)),
    },
    EventSpec {
        key: "subagent_start",
        cli_event: "task_complete",
        claude: Some(("SubagentStart", None)),
        codex: Some(("SubagentStart", None)),
    },
    EventSpec {
        key: "question",
        cli_event: "user_input_required",
        claude: None,
        codex: Some(("PreToolUse", Some("^request_user_input(_async)?$"))),
    },
    EventSpec {
        key: "post_compact",
        cli_event: "task_complete",
        claude: None,
        codex: Some(("PostCompact", None)),
    },
    EventSpec {
        key: "interrupt",
        cli_event: "task_complete",
        claude: None,
        codex: Some(("Interrupt", None)),
    },
];

/// Field access by [`EventSpec::key`].
macro_rules! hook_event_fields {
    ($($key:literal => $enabled:ident, $message:ident;)*) => {
        impl HookConfig {
            fn event(&self, key: &str) -> Option<(bool, &str)> {
                match key {
                    $($key => Some((self.$enabled, self.$message.as_str())),)*
                    _ => None,
                }
            }

            fn event_mut(&mut self, key: &str) -> Option<(&mut bool, &mut String)> {
                match key {
                    $($key => Some((&mut self.$enabled, &mut self.$message)),)*
                    _ => None,
                }
            }
        }
    };
}

hook_event_fields! {
    "session_start" => session_start_enabled, session_start_message;
    "stop" => stop_enabled, stop_message;
    "notification_permission" => notification_permission_enabled, notification_permission_message;
    "notification_elicitation" => notification_elicitation_enabled, notification_elicitation_message;
    "notification_idle" => notification_idle_enabled, notification_idle_message;
    "session_end" => session_end_enabled, session_end_message;
    "subagent_stop" => subagent_stop_enabled, subagent_stop_message;
    "pre_compact" => pre_compact_enabled, pre_compact_message;
    "setup" => setup_enabled, setup_message;
    "user_prompt_submit" => user_prompt_submit_enabled, user_prompt_submit_message;
    "pre_tool_use" => pre_tool_use_enabled, pre_tool_use_message;
    "post_tool_use" => post_tool_use_enabled, post_tool_use_message;
    "post_tool_use_failure" => post_tool_use_failure_enabled, post_tool_use_failure_message;
    "permission_request" => permission_request_enabled, permission_request_message;
    "subagent_start" => subagent_start_enabled, subagent_start_message;
    "question" => question_enabled, question_message;
    "post_compact" => post_compact_enabled, post_compact_message;
    "interrupt" => interrupt_enabled, interrupt_message;
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

/// 커스텀 알림 소리 복사본 보관 폴더 (%LOCALAPPDATA%\agent-toast\sounds)
pub fn sounds_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("agent-toast")
        .join("sounds")
}

/// 선택한 사운드 파일을 sounds 폴더로 복사하고 복사본 경로를 반환.
/// 기존 파일은 삭제하지 않는다. 정리는 저장 시점의 prune_sounds_dir 담당.
fn copy_sound_into(src: &Path, dir: &Path) -> Result<String, String> {
    let name = src
        .file_name()
        .ok_or_else(|| "잘못된 파일 경로".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let dest = dir.join(name);
    // sounds 폴더 안의 파일을 다시 선택한 경우 자기 자신 복사 방지
    if dest != src {
        std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
    }
    Ok(dest.to_string_lossy().to_string())
}

/// sounds 폴더에서 keep 이외의 파일을 모두 삭제 (설정 저장 시 호출)
fn prune_sounds_dir(dir: &Path, keep: Option<&Path>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if keep.is_some_and(|k| k == p.as_path()) {
            continue;
        }
        let _ = std::fs::remove_file(&p);
    }
}

/// 선택한 사운드 파일을 앱 데이터 폴더로 복사 (설정 저장 전 단계, 설정 파일은 건드리지 않음)
#[tauri::command]
pub fn copy_notification_sound_file(path: String) -> Result<String, String> {
    copy_sound_into(Path::new(&path), &sounds_dir())
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
        notification_sound_file: root["agent_toast"]["notification_sound_file"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        notification_monitor: root["agent_toast"]["notification_monitor"]
            .as_str()
            .unwrap_or("primary")
            .to_string(),
        locale: root["agent_toast"]["locale"]
            .as_str()
            .unwrap_or("ko")
            .to_string(),
        auto_start: root["agent_toast"]["auto_start"].as_bool().unwrap_or(true),
        claude_enabled: root["agent_toast"]["claude_enabled"]
            .as_bool()
            .unwrap_or(true),
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
        dynamic_message_enabled: root["agent_toast"]["dynamic_message_enabled"]
            .as_bool()
            .unwrap_or_else(default_dynamic_message_enabled),
        toast_bar: root["agent_toast"]["toast_bar"]
            .as_str()
            .unwrap_or("none")
            .to_string(),
        toast_border: root["agent_toast"]["toast_border"]
            .as_str()
            .unwrap_or("subtle")
            .to_string(),
        toast_effects: root["agent_toast"]["toast_effects"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        toast_body: root["agent_toast"]["toast_body"]
            .as_str()
            .unwrap_or("glow")
            .to_string(),
        toast_density: root["agent_toast"]["toast_density"]
            .as_str()
            .unwrap_or("comfortable")
            .to_string(),
        toast_font_sans: root["agent_toast"]["toast_font_sans"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        toast_font_mono: root["agent_toast"]["toast_font_mono"]
            .as_str()
            .unwrap_or("")
            .to_string(),
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

    apply_saved_events(&mut config, &root["agent_toast"]["events"]);

    config
}

/// Restore event rows from the `agent_toast.events` snapshot. Registered Claude
/// hooks stay the source of truth for the rows they carry (so hand edits to
/// settings.json still show up); the snapshot fills in the rest: Codex-only
/// rows, and every row while Claude's hooks are unregistered.
fn apply_saved_events(config: &mut HookConfig, events: &Value) {
    let Some(events) = events.as_object() else {
        return;
    };
    let claude_enabled = config.claude_enabled;
    for spec in EVENT_SPECS {
        if claude_enabled && spec.claude.is_some() {
            continue;
        }
        let (Some(saved), Some((enabled, message))) =
            (events.get(spec.key), config.event_mut(spec.key))
        else {
            continue;
        };
        if let Some(b) = saved["enabled"].as_bool() {
            *enabled = b;
        }
        if let Some(m) = saved["message"].as_str() {
            *message = m.to_string();
        }
    }
}

/// Every event row as `{ key: { enabled, message } }`, for `agent_toast.events`.
fn saved_events(config: &HookConfig) -> Value {
    let mut events = serde_json::Map::new();
    for spec in EVENT_SPECS {
        if let Some((enabled, message)) = config.event(spec.key) {
            events.insert(
                spec.key.into(),
                serde_json::json!({ "enabled": enabled, "message": message }),
            );
        }
    }
    Value::Object(events)
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
        "notification_sound_file".into(),
        config
            .notification_sound_file
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    cn.insert(
        "notification_monitor".into(),
        Value::String(config.notification_monitor.clone()),
    );
    cn.insert("locale".into(), Value::String(config.locale.clone()));
    cn.insert("auto_start".into(), Value::Bool(config.auto_start));
    cn.insert("claude_enabled".into(), Value::Bool(config.claude_enabled));
    cn.insert("codex_enabled".into(), Value::Bool(config.codex_enabled));
    cn.insert("events".into(), saved_events(config));
    cn.insert("http_enabled".into(), Value::Bool(config.http_enabled));
    cn.insert("http_port".into(), Value::Number(config.http_port.into()));
    cn.insert("show_hostname".into(), Value::Bool(config.show_hostname));
    cn.insert(
        "dynamic_message_enabled".into(),
        Value::Bool(config.dynamic_message_enabled),
    );
    cn.insert("toast_bar".into(), Value::String(config.toast_bar.clone()));
    cn.insert(
        "toast_border".into(),
        Value::String(config.toast_border.clone()),
    );
    cn.insert(
        "toast_effects".into(),
        Value::Array(
            config
                .toast_effects
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );
    cn.insert(
        "toast_body".into(),
        Value::String(config.toast_body.clone()),
    );
    cn.insert(
        "toast_density".into(),
        Value::String(config.toast_density.clone()),
    );
    cn.insert(
        "toast_font_sans".into(),
        Value::String(config.toast_font_sans.clone()),
    );
    cn.insert(
        "toast_font_mono".into(),
        Value::String(config.toast_font_mono.clone()),
    );
    cn.insert(
        "version".into(),
        Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );
    root["agent_toast"] = Value::Object(cn);
}

/// Append ` --dynamic` to every notification hook command when dynamic mode is
/// enabled, leaving the infrastructure `--daemon` entry untouched (it carries no
/// message). With `--dynamic` the CLI derives the toast body from the hook's
/// stdin JSON instead of the static `--message`.
fn apply_dynamic_flag(entries: &mut [HookEntry], enabled: bool) {
    if !enabled {
        return;
    }
    for e in entries.iter_mut() {
        if !e.command.contains("--daemon") {
            e.command.push_str(" --dynamic");
        }
    }
}

/// One-shot migration for hooks registered before `dynamic_message_enabled`
/// defaulted to true: surgically append ` --dynamic` to existing agent-toast
/// notification commands (no regeneration — hand-tuned messages and exe paths
/// stay untouched), then record `agent_toast.dynamic_message_enabled: true` so
/// subsequent runs are a no-op.
///
/// Returns `Some(new_json)` only when the file should be rewritten. `None`:
/// - the key already exists (user made a choice, or already migrated),
/// - no agent-toast hooks are registered (fresh install — the new default
///   applies on first save),
/// - the content isn't a JSON object.
fn migrate_dynamic_message_default(content: &str) -> Option<String> {
    let Ok(mut root) = serde_json::from_str::<Value>(content) else {
        return None;
    };
    if !root.is_object() {
        return None;
    }
    if root["agent_toast"]["dynamic_message_enabled"]
        .as_bool()
        .is_some()
    {
        return None;
    }

    let mut found_agent_toast = false;
    if let Some(events) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        for groups in events.values_mut() {
            let Some(groups) = groups.as_array_mut() else {
                continue;
            };
            for group in groups {
                let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                    continue;
                };
                for hook in hooks {
                    let Some(cmd) = hook.get("command").and_then(Value::as_str) else {
                        continue;
                    };
                    if !is_agent_toast_cmd(cmd) {
                        continue;
                    }
                    found_agent_toast = true;
                    if !cmd.contains("--daemon") && !cmd.contains("--dynamic") {
                        hook["command"] = Value::String(format!("{cmd} --dynamic"));
                    }
                }
            }
        }
    }
    if !found_agent_toast {
        return None;
    }

    // Record the resolved choice — this is what makes the migration idempotent.
    root["agent_toast"]["dynamic_message_enabled"] = Value::Bool(true);
    serde_json::to_string_pretty(&root).ok()
}

/// Run [`migrate_dynamic_message_default`] against `~/.claude/settings.json`.
/// Called once at daemon startup; cheap no-op once migrated (or for fresh
/// installs), and only writes the file when something actually changed.
pub fn run_dynamic_message_migration() {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    if let Some(updated) = migrate_dynamic_message_default(&content) {
        match std::fs::write(&path, updated) {
            Ok(()) => log::info!("[migrate] appended --dynamic to existing agent-toast hooks"),
            Err(e) => log::warn!("[migrate] failed to write settings.json: {e}"),
        }
    }
}

/// Save hook config to ~/.claude/settings.json, preserving other fields
#[tauri::command]
pub fn save_hook_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::notification::NotificationManagerState>,
    config: HookConfig,
) -> Result<SaveResult, String> {
    let path = settings_path();

    // Read existing settings or create new object
    let mut root: Value = if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    let exe = exe_path_for_shell();

    // Codex 먼저: hooks.json 이 깨져 있으면 아무것도 쓰지 않고 실패한다.
    let codex_hooks_changed = save_codex_hooks(&config, &exe)?;

    // Merge entries into root, preserving non-agent-toast hooks. A disabled
    // Claude keeps no agent-toast hooks at all.
    let entries = if config.claude_enabled {
        build_hook_entries(&config, &exe, Agent::Claude)
    } else {
        Vec::new()
    };
    root = merge_agent_toast_hooks(root, &entries);

    write_agent_toast_settings(&mut root, &config);

    // Ensure .claude directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())?;

    // 사운드 복사본 정리: 설정에 참조된 파일만 남긴다 (기본값 복원 후 저장 시 전체 삭제)
    prune_sounds_dir(
        &sounds_dir(),
        config.notification_sound_file.as_deref().map(Path::new),
    );

    // HTTP 서버 상태를 새 설정과 동기화 (즉시 시작/중지/재시작)
    crate::sync_http_server(&app).map_err(|e| format!("HTTP 서버 시작 실패: {e}"))?;

    // 저장 후 이미 떠 있는 알림들의 위치를 즉시 반영
    crate::notification::reposition_all(&app, &state);

    // 트레이 메뉴 텍스트를 현재 locale에 맞게 갱신
    crate::update_tray_locale(&app);

    Ok(SaveResult {
        codex_hooks_path: (config.codex_enabled && codex_hooks_changed)
            .then(|| codex_hooks_path().to_string_lossy().to_string()),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveResult {
    /// Set when this save registered or changed Codex hooks: Codex runs them
    /// only after the user trusts them again in its `/hooks` screen.
    pub codex_hooks_path: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
}

/// The agent-toast hook commands one agent should carry for `config`: the
/// optional `--daemon` launcher plus one entry per enabled row that agent
/// supports.
fn build_hook_entries(config: &HookConfig, exe: &str, agent: Agent) -> Vec<HookEntry> {
    // Codex's Windows session shell is PowerShell: a quoted executable path
    // needs the call operator, otherwise arguments such as --source fail to parse.
    let exe = if cfg!(windows) && matches!(agent, Agent::Codex) {
        format!("& {exe}")
    } else {
        exe.to_string()
    };
    // Codex hooks name their source so the CLI reads `cwd` from stdin and
    // answers with JSON; Claude commands stay as they always were.
    let source = match agent {
        Agent::Claude => "",
        Agent::Codex => " --source codex",
    };
    let mut entries: Vec<HookEntry> = Vec::new();

    if config.auto_start {
        entries.push(HookEntry {
            event_key: "SessionStart",
            matcher: None,
            command: format!("{exe} --daemon{source}"),
        });
    }

    for spec in EVENT_SPECS {
        let target = match agent {
            Agent::Claude => spec.claude,
            Agent::Codex => spec.codex,
        };
        let Some((event_key, matcher)) = target else {
            continue;
        };
        let Some((true, message)) = config.event(spec.key) else {
            continue;
        };
        entries.push(HookEntry {
            event_key,
            matcher,
            command: format!(
                "{exe}{source} --event {} --message \"{message}\"",
                spec.cli_event
            ),
        });
    }

    // When dynamic mode is on, every notification command derives its body from
    // the hook's stdin JSON (the infrastructure `--daemon` entry stays untouched).
    apply_dynamic_flag(&mut entries, config.dynamic_message_enabled);
    entries
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

fn agent_settings_path(agent: Agent) -> PathBuf {
    match agent {
        Agent::Claude => settings_path(),
        Agent::Codex => codex_hooks_path(),
    }
}

/// Which agent files exist yet: each appears only after the first save that
/// registers that agent, so the setup window greys out the missing ones.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsFiles {
    pub claude: bool,
    pub codex: bool,
}

#[tauri::command]
pub fn get_settings_files() -> SettingsFiles {
    SettingsFiles {
        claude: agent_settings_path(Agent::Claude).exists(),
        codex: agent_settings_path(Agent::Codex).exists(),
    }
}

#[tauri::command]
pub fn open_settings_file(agent: Agent) -> Result<(), String> {
    let path = agent_settings_path(agent);
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

/// 설정 파일에서 notification_sound_file 값만 빠르게 읽기. None = 시스템 기본음
pub fn load_notification_sound_file() -> Option<String> {
    let path = settings_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let root: Value = serde_json::from_str(&content).ok()?;
    root["agent_toast"]["notification_sound_file"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
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

/// settings.json 문자열에서 agent_toast.theme 추출 — 실패/누락 시 "system".
fn read_theme_from_json(content: &str) -> String {
    serde_json::from_str::<Value>(content)
        .ok()
        .and_then(|root| root["agent_toast"]["theme"].as_str().map(str::to_string))
        .unwrap_or_else(default_theme)
}

/// settings.json 문자열에 agent_toast.theme 을 설정하고 나머지 키는 보존하여 반환.
fn set_theme_in_json(content: &str, theme: &str) -> String {
    let mut root: Value =
        serde_json::from_str(content).unwrap_or_else(|_| Value::Object(Default::default()));
    if !root.is_object() {
        root = Value::Object(Default::default());
    }
    let obj = root.as_object_mut().unwrap();
    let at = obj
        .entry("agent_toast")
        .or_insert_with(|| Value::Object(Default::default()));
    if !at.is_object() {
        *at = Value::Object(Default::default());
    }
    at.as_object_mut()
        .unwrap()
        .insert("theme".into(), Value::String(theme.to_string()));
    serde_json::to_string_pretty(&root).unwrap_or_else(|_| content.to_string())
}

/// settings.json 에서 theme 읽기 — 실패 시 "system".
pub fn read_theme() -> String {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => read_theme_from_json(&content),
        Err(_) => default_theme(),
    }
}

/// settings.json 의 theme 만 갱신(다른 키/훅 보존). 파일 없으면 새로 생성.
pub fn write_theme(theme: &str) -> Result<(), String> {
    let path = settings_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let updated = set_theme_in_json(&content, theme);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, updated).map_err(|e| e.to_string())
}

/// settings.json 문자열에서 agent_toast.global_stats_enabled 추출 — 기본 true (옵트아웃).
fn read_global_stats_from_json(content: &str) -> bool {
    serde_json::from_str::<Value>(content)
        .ok()
        .and_then(|root| root["agent_toast"]["global_stats_enabled"].as_bool())
        .unwrap_or(true)
}

/// settings.json 문자열에 agent_toast.global_stats_enabled 를 설정(나머지 키 보존).
fn set_global_stats_in_json(content: &str, enabled: bool) -> String {
    let mut root: Value =
        serde_json::from_str(content).unwrap_or_else(|_| Value::Object(Default::default()));
    if !root.is_object() {
        root = Value::Object(Default::default());
    }
    let obj = root.as_object_mut().unwrap();
    let at = obj
        .entry("agent_toast")
        .or_insert_with(|| Value::Object(Default::default()));
    if !at.is_object() {
        *at = Value::Object(Default::default());
    }
    at.as_object_mut()
        .unwrap()
        .insert("global_stats_enabled".into(), Value::Bool(enabled));
    serde_json::to_string_pretty(&root).unwrap_or_else(|_| content.to_string())
}

/// settings.json에서 global_stats_enabled 읽기 — 파일 없음/실패 시 true.
pub fn read_global_stats_enabled() -> bool {
    match std::fs::read_to_string(settings_path()) {
        Ok(content) => read_global_stats_from_json(&content),
        Err(_) => true,
    }
}

#[tauri::command]
pub fn get_global_stats_enabled() -> bool {
    read_global_stats_enabled()
}

#[tauri::command]
pub fn set_global_stats_enabled(enabled: bool) -> Result<(), String> {
    let path = settings_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let updated = set_global_stats_in_json(&content, enabled);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, updated).map_err(|e| e.to_string())
}

/// 알림 창이 렌더링에 사용하는 토스트 디자인 축 값.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToastStyle {
    pub bar: String,
    pub border: String,
    pub effects: Vec<String>,
    pub body: String,
    pub density: String,
    pub font_sans: String,
    pub font_mono: String,
}

/// settings.json 문자열에서 ToastStyle 추출 — 누락/실패 시 기본값.
fn toast_style_from_json(content: &str) -> ToastStyle {
    let config = parse_hook_config_from_json(content);
    ToastStyle {
        bar: config.toast_bar,
        border: config.toast_border,
        effects: config.toast_effects,
        body: config.toast_body,
        density: config.toast_density,
        font_sans: config.toast_font_sans,
        font_mono: config.toast_font_mono,
    }
}

/// 알림 창 mount 시 1회 호출 — 파일 읽기 실패 시 기본값.
#[tauri::command]
pub fn get_toast_style() -> ToastStyle {
    let path = settings_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    toast_style_from_json(&content)
}

fn codex_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

fn codex_config_path() -> PathBuf {
    codex_dir().join("config.toml")
}

fn codex_hooks_path() -> PathBuf {
    codex_dir().join("hooks.json")
}

/// Register (or remove) agent-toast's Codex hooks in `~/.codex/hooks.json`,
/// which shares Claude's `{ "hooks": { Event: [groups] } }` layout, and drop
/// the `notify` entry older versions wrote to `config.toml`. Returns whether
/// hooks.json changed. A hooks.json that is not JSON is left alone and fails
/// the save rather than being overwritten.
fn save_codex_hooks(config: &HookConfig, exe: &str) -> Result<bool, String> {
    let entries = if config.codex_enabled {
        build_hook_entries(config, exe, Agent::Codex)
    } else {
        Vec::new()
    };

    let path = codex_hooks_path();
    let existing = std::fs::read_to_string(&path).ok();
    let mut changed = false;
    // Nothing registered and nothing to register: don't create the file.
    if existing.is_some() || !entries.is_empty() {
        let updated = merge_codex_hooks_json(existing.as_deref(), &entries)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if existing.as_deref() != Some(updated.as_str()) {
            std::fs::create_dir_all(codex_dir()).map_err(|e| e.to_string())?;
            std::fs::write(&path, updated).map_err(|e| e.to_string())?;
            changed = true;
        }
    }

    // The legacy notify would deliver every turn end a second time next to
    // the Stop hook.
    let config_path = codex_config_path();
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Some(updated) = strip_legacy_codex_notify(&content) {
            std::fs::write(&config_path, updated).map_err(|e| e.to_string())?;
        }
    }
    Ok(changed)
}

/// Pure part of [`save_codex_hooks`]: hooks.json content with agent-toast's
/// entries replaced by `entries`, everything else kept.
fn merge_codex_hooks_json(existing: Option<&str>, entries: &[HookEntry]) -> Result<String, String> {
    let root = match existing {
        Some(content) if !content.trim().is_empty() => {
            let root: Value = serde_json::from_str(content).map_err(|e| e.to_string())?;
            if !root.is_object() {
                return Err("not a JSON object".into());
            }
            root
        }
        _ => Value::Object(Default::default()),
    };
    let root = merge_agent_toast_hooks(root, entries);
    serde_json::to_string_pretty(&root).map_err(|e| e.to_string())
}

/// `config.toml` without the `notify` entry, when that entry is agent-toast's
/// (`notify = ["…agent-toast.exe", "--codex"]`). `None` leaves the file
/// untouched: no notify, someone else's notify, or unparsable TOML. Edits in
/// place so the user's comments and layout survive.
fn strip_legacy_codex_notify(content: &str) -> Option<String> {
    let mut doc = content.parse::<toml_edit::DocumentMut>().ok()?;
    let ours = doc
        .get("notify")
        .and_then(|item| item.as_array())
        .is_some_and(|arr| {
            arr.iter()
                .any(|v| v.as_str().is_some_and(is_agent_toast_cmd))
        });
    if !ours {
        return None;
    }
    doc.remove("notify");
    Some(doc.to_string())
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

    // ── notification_sound_file tests ──

    #[test]
    fn parse_notification_sound_file_absent_is_none() {
        let config = parse_hook_config_from_json(r#"{"agent_toast": {}}"#);
        assert_eq!(config.notification_sound_file, None);
    }

    #[test]
    fn parse_notification_sound_file_present() {
        let json = r#"{"agent_toast": {"notification_sound_file": "C:\\data\\agent-toast\\sounds\\ding.mp3"}}"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(
            config.notification_sound_file.as_deref(),
            Some("C:\\data\\agent-toast\\sounds\\ding.mp3")
        );
    }

    #[test]
    fn parse_notification_sound_file_empty_is_none() {
        let json = r#"{"agent_toast": {"notification_sound_file": ""}}"#;
        assert_eq!(
            parse_hook_config_from_json(json).notification_sound_file,
            None
        );
    }

    #[test]
    fn write_notification_sound_file_some_and_none() {
        let mut root = serde_json::json!({});
        let mut config = HookConfig {
            notification_sound_file: Some("C:\\s\\ding.mp3".into()),
            ..HookConfig::default()
        };
        write_agent_toast_settings(&mut root, &config);
        assert_eq!(
            root["agent_toast"]["notification_sound_file"].as_str(),
            Some("C:\\s\\ding.mp3")
        );

        config.notification_sound_file = None;
        write_agent_toast_settings(&mut root, &config);
        assert!(root["agent_toast"]["notification_sound_file"].is_null());
    }

    #[test]
    fn copy_and_prune_sounds_dir() {
        let base =
            std::env::temp_dir().join(format!("agent-toast-test-sounds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("sounds");

        // 원본 파일 준비
        std::fs::create_dir_all(&base).unwrap();
        let src = base.join("ding.mp3");
        std::fs::write(&src, b"fake-mp3").unwrap();

        // 복사: 폴더가 없어도 생성되고, 원본 파일명이 유지된 복사본 경로 반환
        let stored = copy_sound_into(&src, &dir).unwrap();
        let stored_path = PathBuf::from(&stored);
        assert!(stored_path.exists());
        assert_eq!(stored_path.file_name().unwrap(), "ding.mp3");

        // 두 번째 파일 복사: 첫 파일은 아직 남아 있어야 한다 (복사는 삭제하지 않음)
        let src2 = base.join("dong.wav");
        std::fs::write(&src2, b"fake-wav").unwrap();
        let stored2 = copy_sound_into(&src2, &dir).unwrap();
        assert!(stored_path.exists());

        // prune: keep 파일만 남는다
        prune_sounds_dir(&dir, Some(Path::new(&stored2)));
        assert!(!stored_path.exists());
        assert!(PathBuf::from(&stored2).exists());

        // keep=None 이면 전부 삭제
        prune_sounds_dir(&dir, None);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);

        // 존재하지 않는 폴더는 no-op (panic 없음)
        prune_sounds_dir(&base.join("nope"), None);

        let _ = std::fs::remove_dir_all(&base);
    }

    // ── global_stats_enabled tests ──

    #[test]
    fn global_stats_default_true_on_missing() {
        assert!(read_global_stats_from_json("{}"));
        assert!(read_global_stats_from_json("not json"));
    }

    #[test]
    fn global_stats_reads_stored_false() {
        let json = r#"{"agent_toast":{"global_stats_enabled":false}}"#;
        assert!(!read_global_stats_from_json(json));
    }

    #[test]
    fn set_global_stats_preserves_other_keys() {
        let json = r#"{"hooks":{"Stop":[]},"agent_toast":{"theme":"dark"}}"#;
        let updated = set_global_stats_in_json(json, false);
        assert!(!read_global_stats_from_json(&updated));
        let v: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(v["agent_toast"]["theme"], "dark");
        assert!(v["hooks"]["Stop"].is_array());
    }

    #[test]
    fn set_global_stats_creates_structure_from_empty() {
        let updated = set_global_stats_in_json("", true);
        let v: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(v["agent_toast"]["global_stats_enabled"], true);
    }

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
    fn toast_style_from_json_defaults() {
        let style = toast_style_from_json("{}");
        assert_eq!(style.bar, "none");
        assert_eq!(style.border, "subtle");
        assert!(style.effects.is_empty());
        assert_eq!(style.body, "glow");
        assert_eq!(style.density, "comfortable");
    }

    #[test]
    fn toast_style_from_json_values() {
        let json = r#"{
            "agent_toast": {
                "toast_bar": "left",
                "toast_border": "accent",
                "toast_effects": ["shimmer"],
                "toast_body": "flat",
                "toast_density": "compact"
            }
        }"#;
        let style = toast_style_from_json(json);
        assert_eq!(style.bar, "left");
        assert_eq!(style.border, "accent");
        assert_eq!(style.effects, vec!["shimmer"]);
        assert_eq!(style.body, "flat");
        assert_eq!(style.density, "compact");
    }

    #[test]
    fn toast_style_carries_fonts() {
        let json = r#"{
            "agent_toast": {
                "toast_font_sans": "Nanum Gothic",
                "toast_font_mono": "Consolas"
            }
        }"#;
        let style = toast_style_from_json(json);
        assert_eq!(style.font_sans, "Nanum Gothic");
        assert_eq!(style.font_mono, "Consolas");
    }

    /// agent_toast 섹션에 toast_* 키가 없으면 기본값 (현재 디자인과 동일 조합)
    #[test]
    fn parse_toast_style_defaults_when_missing() {
        let config = parse_hook_config_from_json("{}");
        assert_eq!(config.toast_bar, "none");
        assert_eq!(config.toast_border, "subtle");
        assert!(config.toast_effects.is_empty());
        assert_eq!(config.toast_body, "glow");
        assert_eq!(config.toast_density, "comfortable");
    }

    #[test]
    fn parse_toast_style_values() {
        let json = r#"{
            "agent_toast": {
                "toast_bar": "left",
                "toast_border": "accent",
                "toast_effects": ["ring", "breathe"],
                "toast_body": "tint",
                "toast_density": "compact"
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.toast_bar, "left");
        assert_eq!(config.toast_border, "accent");
        assert_eq!(config.toast_effects, vec!["ring", "breathe"]);
        assert_eq!(config.toast_body, "tint");
        assert_eq!(config.toast_density, "compact");
    }

    /// toast_effects 배열에 문자열이 아닌 항목이 섞여도 무시하고 파싱 (전방 호환)
    #[test]
    fn parse_toast_effects_skips_non_strings() {
        let json = r#"{ "agent_toast": { "toast_effects": ["ring", 42, null] } }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.toast_effects, vec!["ring"]);
    }

    /// write → parse 라운드트립으로 toast_* 4키 보존 확인
    #[test]
    fn toast_style_write_roundtrip() {
        let config = HookConfig {
            toast_bar: "left".into(),
            toast_border: "accent".into(),
            toast_effects: vec!["pulse".into(), "shimmer".into()],
            toast_body: "flat".into(),
            toast_density: "compact".into(),
            ..HookConfig::default()
        };

        let mut root = Value::Object(Default::default());
        write_agent_toast_settings(&mut root, &config);
        let parsed = parse_hook_config_from_json(&root.to_string());

        assert_eq!(parsed.toast_bar, "left");
        assert_eq!(parsed.toast_border, "accent");
        assert_eq!(parsed.toast_effects, vec!["pulse", "shimmer"]);
        assert_eq!(parsed.toast_body, "flat");
        assert_eq!(parsed.toast_density, "compact");
    }

    #[test]
    fn parse_toast_fonts_explicit() {
        let json = r#"{
            "agent_toast": {
                "toast_font_sans": "Nanum Gothic",
                "toast_font_mono": "Consolas"
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.toast_font_sans, "Nanum Gothic");
        assert_eq!(config.toast_font_mono, "Consolas");
    }

    #[test]
    fn parse_toast_fonts_default_empty() {
        let json = r#"{ "agent_toast": {} }"#;
        let config = parse_hook_config_from_json(json);
        assert_eq!(config.toast_font_sans, "");
        assert_eq!(config.toast_font_mono, "");
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

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn read_theme_from_json_returns_stored_value() {
        let json = r#"{"agent_toast":{"theme":"light"}}"#;
        assert_eq!(read_theme_from_json(json), "light");
    }

    #[test]
    fn read_theme_from_json_defaults_to_system_when_missing() {
        let json = r#"{"agent_toast":{"locale":"ko"}}"#;
        assert_eq!(read_theme_from_json(json), "system");
    }

    #[test]
    fn read_theme_from_json_defaults_to_system_when_invalid() {
        assert_eq!(read_theme_from_json("not json"), "system");
    }

    #[test]
    fn set_theme_in_json_sets_value_and_preserves_other_keys() {
        let json = r#"{"agent_toast":{"locale":"ko"},"otherTool":{"x":1}}"#;
        let out = set_theme_in_json(json, "dark");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["agent_toast"]["theme"], "dark");
        assert_eq!(v["agent_toast"]["locale"], "ko");
        assert_eq!(v["otherTool"]["x"], 1);
    }

    #[test]
    fn set_theme_in_json_creates_structure_when_empty() {
        let out = set_theme_in_json("", "light");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["agent_toast"]["theme"], "light");
    }

    #[test]
    fn parse_reads_dynamic_message_enabled() {
        let json = r#"{"agent_toast":{"dynamic_message_enabled":true}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.dynamic_message_enabled);
    }

    #[test]
    fn parse_dynamic_message_defaults_true_when_absent() {
        let config = parse_hook_config_from_json("{}");
        assert!(config.dynamic_message_enabled);
    }

    #[test]
    fn apply_dynamic_flag_targets_notification_commands_only() {
        let mut entries = vec![
            HookEntry {
                event_key: "SessionStart",
                matcher: None,
                command: "\"at.exe\" --daemon".into(),
            },
            HookEntry {
                event_key: "Stop",
                matcher: None,
                command: "\"at.exe\" --event task_complete --message \"x\"".into(),
            },
        ];
        apply_dynamic_flag(&mut entries, true);
        assert_eq!(
            entries[0].command, "\"at.exe\" --daemon",
            "--daemon 엔트리는 무변경"
        );
        assert!(
            entries[1].command.ends_with(" --dynamic"),
            "알림 커맨드에 --dynamic 추가"
        );
    }

    #[test]
    fn apply_dynamic_flag_noop_when_disabled() {
        let mut entries = vec![HookEntry {
            event_key: "Stop",
            matcher: None,
            command: "\"at.exe\" --event task_complete".into(),
        }];
        apply_dynamic_flag(&mut entries, false);
        assert!(!entries[0].command.contains("--dynamic"));
    }

    /// 키 없음 + agent-toast 훅 존재 시 settings.json 의 전형적인 형태
    fn legacy_settings_json() -> String {
        serde_json::json!({
            "hooks": {
                "SessionStart": [{
                    "hooks": [{ "type": "command", "command": "\"C:\\at\\agent-toast.exe\" --daemon" }]
                }],
                "Stop": [{
                    "hooks": [
                        { "type": "command", "command": "\"C:\\at\\agent-toast.exe\" --event task_complete --message \"내 문구\"" },
                        { "type": "command", "command": "/usr/bin/other-tool --flag" }
                    ]
                }]
            },
            "agent_toast": { "theme": "dark" }
        })
        .to_string()
    }

    #[test]
    fn migrate_appends_dynamic_and_records_key() {
        let out = migrate_dynamic_message_default(&legacy_settings_json()).expect("should migrate");
        let v: Value = serde_json::from_str(&out).unwrap();
        let stop_cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(
            stop_cmd.ends_with(" --dynamic"),
            "알림 커맨드에 --dynamic 추가"
        );
        assert!(
            stop_cmd.contains("--message \"내 문구\""),
            "손수정 문구 보존"
        );
        assert_eq!(
            v["hooks"]["SessionStart"][0]["hooks"][0]["command"],
            "\"C:\\at\\agent-toast.exe\" --daemon",
            "--daemon 엔트리는 무변경"
        );
        assert_eq!(
            v["hooks"]["Stop"][0]["hooks"][1]["command"], "/usr/bin/other-tool --flag",
            "비 agent-toast 훅은 무변경"
        );
        assert_eq!(v["agent_toast"]["dynamic_message_enabled"], true, "키 기록");
        assert_eq!(
            v["agent_toast"]["theme"], "dark",
            "기존 agent_toast 키 보존"
        );
    }

    #[test]
    fn migrate_noop_when_key_present() {
        for stored in [true, false] {
            let json = serde_json::json!({
                "agent_toast": { "dynamic_message_enabled": stored },
                "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "\"at.exe\" --event task_complete" }] }] }
            })
            .to_string();
            assert!(
                migrate_dynamic_message_default(&json).is_none(),
                "키가 {stored} 로 있으면 사용자 선택 존중 → no-op"
            );
        }
    }

    #[test]
    fn migrate_noop_without_agent_toast_hooks() {
        let json = serde_json::json!({
            "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "/usr/bin/other-tool" }] }] }
        })
        .to_string();
        assert!(
            migrate_dynamic_message_default(&json).is_none(),
            "신규 사용자 → no-op"
        );
        assert!(
            migrate_dynamic_message_default("{}").is_none(),
            "빈 설정 → no-op"
        );
    }

    #[test]
    fn migrate_no_double_append_when_dynamic_already_present() {
        let json = serde_json::json!({
            "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "\"C:\\at\\agent-toast.exe\" --event task_complete --dynamic" }] }] }
        })
        .to_string();
        // 훅은 있으니 키 기록을 위해 Some 이지만, 커맨드는 그대로여야 한다.
        let out = migrate_dynamic_message_default(&json).expect("key should still be recorded");
        let v: Value = serde_json::from_str(&out).unwrap();
        let cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(!cmd.contains("--dynamic --dynamic"), "중복 append 금지");
        assert_eq!(v["agent_toast"]["dynamic_message_enabled"], true);
    }

    #[test]
    fn migrate_covers_send_commands_too() {
        // 데스크톱 settings.json 에 수동 등록된 agent-toast-send 훅도 동일하게 마이그레이션
        let json = serde_json::json!({
            "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "agent-toast-send --url http://x:38787 --event task_complete" }] }] }
        })
        .to_string();
        let out = migrate_dynamic_message_default(&json).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(cmd.ends_with(" --dynamic"));
    }

    #[test]
    fn migrate_noop_on_invalid_or_non_object_json() {
        assert!(migrate_dynamic_message_default("not json {").is_none());
        assert!(migrate_dynamic_message_default("[1,2,3]").is_none());
        assert!(migrate_dynamic_message_default("").is_none());
    }

    #[test]
    fn migrate_is_idempotent() {
        let first = migrate_dynamic_message_default(&legacy_settings_json()).unwrap();
        assert!(
            migrate_dynamic_message_default(&first).is_none(),
            "1회 실행 후엔 키가 있으니 영구 no-op"
        );
    }

    // ── 에이전트별 훅 생성 ──

    const EXE: &str = "\"C:\\agent-toast.exe\"";

    fn entry_summary(entries: &[HookEntry]) -> Vec<(&'static str, Option<&'static str>, String)> {
        entries
            .iter()
            .map(|e| (e.event_key, e.matcher, e.command.clone()))
            .collect()
    }

    fn static_config() -> HookConfig {
        HookConfig {
            dynamic_message_enabled: false,
            stop_message: "완료".into(),
            permission_request_message: "권한".into(),
            question_message: "질문".into(),
            ..HookConfig::default()
        }
    }

    #[test]
    fn every_event_spec_has_config_fields() {
        let mut config = HookConfig::default();
        for spec in EVENT_SPECS {
            assert!(config.event(spec.key).is_some(), "{}", spec.key);
            assert!(config.event_mut(spec.key).is_some(), "{}", spec.key);
            assert!(
                spec.claude.is_some() || spec.codex.is_some(),
                "{} 는 어느 에이전트에도 등록되지 않음",
                spec.key
            );
        }
    }

    #[test]
    fn default_claude_entries_keep_existing_command_format() {
        let entries = build_hook_entries(&static_config(), EXE, Agent::Claude);
        assert_eq!(
            entry_summary(&entries),
            vec![
                ("SessionStart", None, format!("{EXE} --daemon")),
                (
                    "Stop",
                    None,
                    format!("{EXE} --event task_complete --message \"완료\"")
                ),
                (
                    "PermissionRequest",
                    None,
                    format!("{EXE} --event user_input_required --message \"권한\"")
                ),
            ]
        );
    }

    #[test]
    fn default_codex_entries_cover_completion_permission_and_question() {
        let exe = if cfg!(windows) {
            format!("& {EXE}")
        } else {
            EXE.to_string()
        };
        let entries = build_hook_entries(&static_config(), EXE, Agent::Codex);
        assert_eq!(
            entry_summary(&entries),
            vec![
                (
                    "SessionStart",
                    None,
                    format!("{exe} --daemon --source codex")
                ),
                (
                    "Stop",
                    None,
                    format!("{exe} --source codex --event task_complete --message \"완료\"")
                ),
                (
                    "PermissionRequest",
                    None,
                    format!("{exe} --source codex --event user_input_required --message \"권한\"")
                ),
                (
                    "PreToolUse",
                    Some("^request_user_input(_async)?$"),
                    format!("{exe} --source codex --event user_input_required --message \"질문\"")
                ),
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn codex_commands_parse_in_powershell_with_spaces_in_exe_path() {
        let config = HookConfig {
            dynamic_message_enabled: true,
            ..static_config()
        };
        for entry in build_hook_entries(
            &config,
            "\"C:\\Program Files\\Agent Toast\\agent-toast.exe\"",
            Agent::Codex,
        ) {
            let output = std::process::Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "[scriptblock]::Create($env:AGENT_TOAST_TEST_COMMAND) | Out-Null",
                ])
                .env("AGENT_TOAST_TEST_COMMAND", &entry.command)
                .output()
                .expect("PowerShell must be available on Windows");
            assert!(
                output.status.success(),
                "{}: {}",
                entry.event_key,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn agent_only_rows_stay_with_their_agent() {
        let config = HookConfig {
            notification_idle_enabled: true,
            post_tool_use_failure_enabled: true,
            interrupt_enabled: true,
            post_compact_enabled: true,
            ..static_config()
        };
        let claude: Vec<_> = build_hook_entries(&config, EXE, Agent::Claude)
            .iter()
            .map(|e| e.event_key)
            .collect();
        let codex: Vec<_> = build_hook_entries(&config, EXE, Agent::Codex)
            .iter()
            .map(|e| e.event_key)
            .collect();
        assert!(claude.contains(&"Notification"));
        assert!(claude.contains(&"PostToolUseFailure"));
        assert!(!claude.contains(&"Interrupt"));
        assert!(!claude.contains(&"PostCompact"));
        assert!(
            !claude.contains(&"PreToolUse"),
            "질문 행은 Claude 에 훅을 만들지 않음"
        );
        assert!(codex.contains(&"Interrupt"));
        assert!(codex.contains(&"PostCompact"));
        assert!(!codex.contains(&"Notification"));
        assert!(!codex.contains(&"PostToolUseFailure"));
    }

    #[test]
    fn codex_entries_get_dynamic_flag_except_daemon() {
        let config = HookConfig {
            dynamic_message_enabled: true,
            ..HookConfig::default()
        };
        for e in build_hook_entries(&config, EXE, Agent::Codex) {
            assert_eq!(
                e.command.ends_with(" --dynamic"),
                !e.command.contains("--daemon"),
                "{}",
                e.command
            );
        }
    }

    #[test]
    fn no_daemon_entry_without_auto_start() {
        let config = HookConfig {
            auto_start: false,
            ..static_config()
        };
        for agent in [Agent::Claude, Agent::Codex] {
            assert!(build_hook_entries(&config, EXE, agent)
                .iter()
                .all(|e| !e.command.contains("--daemon")));
        }
    }

    // ── 공용 이벤트 스냅샷 ──

    #[test]
    fn saved_events_roundtrip_while_claude_disabled() {
        let config = HookConfig {
            claude_enabled: false,
            codex_enabled: true,
            stop_enabled: false,
            pre_compact_enabled: true,
            pre_compact_message: "압축".into(),
            interrupt_enabled: true,
            ..static_config()
        };
        let mut root = serde_json::json!({});
        write_agent_toast_settings(&mut root, &config);
        // Claude 훅은 하나도 없는 상태 (비활성화로 전부 제거됨)
        let parsed = parse_hook_config_from_json(&root.to_string());
        assert!(!parsed.claude_enabled);
        assert!(parsed.codex_enabled);
        assert!(!parsed.stop_enabled);
        assert!(parsed.permission_request_enabled);
        assert!(parsed.pre_compact_enabled);
        assert_eq!(parsed.pre_compact_message, "압축");
        assert!(parsed.interrupt_enabled);
        assert!(parsed.question_enabled);
        assert_eq!(parsed.question_message, "질문");
    }

    #[test]
    fn registered_claude_hooks_win_over_snapshot() {
        let json = r#"{
            "hooks": {
                "Stop": [{"hooks": [{"type": "command", "command": "agent-toast --event task_complete --message \"직접 수정\""}]}]
            },
            "agent_toast": {
                "events": {
                    "stop": {"enabled": true, "message": "스냅샷"},
                    "permission_request": {"enabled": true, "message": "스냅샷"},
                    "question": {"enabled": false, "message": "질문 스냅샷"}
                }
            }
        }"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.claude_enabled);
        assert_eq!(config.stop_message, "직접 수정");
        assert!(
            !config.permission_request_enabled,
            "Claude 가 켜져 있으면 등록된 훅이 기준"
        );
        assert!(!config.question_enabled);
        assert_eq!(config.question_message, "질문 스냅샷");
    }

    #[test]
    fn legacy_settings_without_snapshot_default_question_on() {
        let json = r#"{"agent_toast": {"codex_enabled": true}}"#;
        let config = parse_hook_config_from_json(json);
        assert!(config.claude_enabled);
        assert!(config.question_enabled);
        assert!(!config.post_compact_enabled);
        assert!(!config.interrupt_enabled);
    }

    // ── Codex hooks.json ──

    #[test]
    fn codex_hooks_json_created_when_absent() {
        let entries = build_hook_entries(&static_config(), EXE, Agent::Codex);
        let out: Value =
            serde_json::from_str(&merge_codex_hooks_json(None, &entries).unwrap()).unwrap();
        assert_eq!(
            out["hooks"]["PreToolUse"][0]["matcher"],
            "^request_user_input(_async)?$"
        );
        assert_eq!(out["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn codex_hooks_json_keeps_other_hooks_and_replaces_ours() {
        let existing = r#"{
            "hooks": {
                "Stop": [
                    {"hooks": [{"type": "command", "command": "python3 other.py"}]},
                    {"hooks": [{"type": "command", "command": "\"C:\\old\\agent-toast.exe\" --source codex --event task_complete --message \"옛 문구\""}]}
                ],
                "Interrupt": [
                    {"hooks": [{"type": "command", "command": "\"C:\\old\\agent-toast.exe\" --source codex --event task_complete --message \"x\""}]}
                ]
            },
            "other": 1
        }"#;
        let entries = build_hook_entries(&static_config(), EXE, Agent::Codex);
        let out: Value =
            serde_json::from_str(&merge_codex_hooks_json(Some(existing), &entries).unwrap())
                .unwrap();
        let stop = out["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2);
        assert_eq!(stop[0]["hooks"][0]["command"], "python3 other.py");
        assert!(stop[1]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("\"완료\""));
        assert!(out["hooks"]["Interrupt"].as_array().unwrap().is_empty());
        assert_eq!(out["other"], 1);
    }

    #[test]
    fn codex_hooks_json_disable_removes_only_ours() {
        let existing = r#"{"hooks": {"Stop": [
            {"hooks": [{"type": "command", "command": "python3 other.py"}]},
            {"hooks": [{"type": "command", "command": "agent-toast.exe --source codex --event task_complete"}]}
        ]}}"#;
        let out: Value =
            serde_json::from_str(&merge_codex_hooks_json(Some(existing), &[]).unwrap()).unwrap();
        let stop = out["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(stop[0]["hooks"][0]["command"], "python3 other.py");
    }

    #[test]
    fn codex_hooks_json_invalid_is_an_error() {
        assert!(merge_codex_hooks_json(Some("{ broken"), &[]).is_err());
        assert!(merge_codex_hooks_json(Some("[1]"), &[]).is_err());
        assert!(merge_codex_hooks_json(Some("  "), &[]).is_ok());
    }

    // ── 기존 Codex notify 정리 ──

    #[test]
    fn strip_legacy_notify_removes_ours_and_keeps_layout() {
        let content = "# my settings\nmodel = \"gpt\"\nnotify = [\"C:\\\\Users\\\\a\\\\agent-toast.exe\", \"--codex\"]\n\n[desktop]\nfoo = true # keep\n";
        let out = strip_legacy_codex_notify(content).unwrap();
        assert!(!out.contains("notify"));
        assert!(out.contains("# my settings"));
        assert!(out.contains("foo = true # keep"));
    }

    #[test]
    fn strip_legacy_notify_leaves_other_programs_alone() {
        let content = "notify = [\"C:\\\\x\\\\codex-computer-use.exe\", \"turn-ended\"]\n";
        assert!(strip_legacy_codex_notify(content).is_none());
        assert!(strip_legacy_codex_notify("model = \"gpt\"\n").is_none());
        assert!(strip_legacy_codex_notify("not = = toml").is_none());
    }
}
