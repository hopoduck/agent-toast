export interface NotificationData {
  id: string;
  window_title: string;
  event_display: string;
  message: string | null;
  source_hwnd: number;
  process_tree: number[];
  auto_dismiss_seconds: number;
  source: string;
  hostname: string | null;
  show_hostname: boolean;
}

export interface HookConfig {
  stop_enabled: boolean;
  stop_message: string;
  notification_permission_enabled: boolean;
  notification_permission_message: string;
  notification_elicitation_enabled: boolean;
  notification_elicitation_message: string;
  notification_idle_enabled: boolean;
  notification_idle_message: string;
  session_start_enabled: boolean;
  session_start_message: string;
  session_end_enabled: boolean;
  session_end_message: string;
  subagent_stop_enabled: boolean;
  subagent_stop_message: string;
  pre_compact_enabled: boolean;
  pre_compact_message: string;
  setup_enabled: boolean;
  setup_message: string;
  user_prompt_submit_enabled: boolean;
  user_prompt_submit_message: string;
  pre_tool_use_enabled: boolean;
  pre_tool_use_message: string;
  post_tool_use_enabled: boolean;
  post_tool_use_message: string;
  post_tool_use_failure_enabled: boolean;
  post_tool_use_failure_message: string;
  permission_request_enabled: boolean;
  permission_request_message: string;
  subagent_start_enabled: boolean;
  subagent_start_message: string;
  title_display_mode: string;
  auto_close_on_focus: boolean;
  auto_dismiss_seconds: number;
  notification_position: string;
  notification_sound: boolean;
  notification_monitor: string;
  locale: string;
  auto_start: boolean;
  codex_enabled: boolean;
  http_enabled: boolean;
  http_port: number;
  show_hostname: boolean;
  dynamic_message_enabled: boolean;
  toast_bar: string;
  toast_border: string;
  toast_effects: string[];
  toast_body: string;
  toast_font_sans: string;
  toast_font_mono: string;
}

export interface MonitorInfo {
  name: string;
  work_area: [number, number, number, number];
  is_primary: boolean;
}

export interface ToastStyle {
  /** "left" | "none" */
  bar: string;
  /** "subtle" | "accent" */
  border: string;
  /** "ring" | "breathe" | "pulse" | "shimmer" — 중첩 가능 */
  effects: string[];
  /** "glow" | "tint" | "flat" */
  body: string;
  /** 본문 폰트 패밀리명. "" = 기본(번들) */
  font_sans: string;
  /** 코드 폰트 패밀리명. "" = 기본(번들) */
  font_mono: string;
}
