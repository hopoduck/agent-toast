import React from "react";
import { Check, ChevronRight, X } from "lucide-react";
import { staticFile } from "remotion";

type Props = {
  /** 헤더의 작은 녹색 라벨. App.vue의 eventLabel. 예: "작업 완료" */
  eventLabel: string;
  /** 본문 큰 글씨 (App.vue 의 window_title). 예: "Claude Code" */
  windowTitle: string;
  /** 본문 작은 글씨 (App.vue 의 message). 예: "작업이 완료되었습니다" */
  message?: string;
  /** dismiss 진행률 0~1 (1=가득, 0=빈). App.vue의 shrink 애니메이션 대응 (우→좌 scaleX) */
  progress?: number;
  /** "보기" 버튼 호버 상태 */
  hovered?: boolean;
  /** "보기" 버튼 라벨 (로케일별) */
  viewLabel?: string;
  /** "닫기" 버튼 라벨 (로케일별) */
  closeLabel?: string;
};

// App.vue 의 eventStyles.success 그대로 (기본값 bar:none — 왼쪽 액센트 바 없음)
const styles = {
  icon: "bg-event-success/20 text-event-success",
  label: "text-event-success",
  // App.vue success viewBtn: bg-event-success text-zinc-950 border-transparent hover:bg-event-success-deep
  // 솔리드 채움 버튼. hover pseudo 못 쓰니 bg 만 분리, 글자/테두리는 항상 적용.
  viewBtnBase: "text-zinc-950 border-transparent",
  viewBtnBg: "bg-event-success",
  viewBtnBgHover: "bg-event-success-deep",
  dismissBar: "bg-event-success/30",
  // App.vue: :style="{ '--toast-accent': styles.accentVar }" — 코너 글로우/링 색
  accentVar: "var(--event-success)",
};

export const FakeToast: React.FC<Props> = ({
  eventLabel,
  windowTitle,
  message,
  progress = 1,
  hovered = false,
  viewLabel = "보기",
  closeLabel = "닫기",
}) => {
  return (
    // App.vue: relative h-screen flex rounded-xl overflow-hidden select-none
    //          bg-gradient-to-b from-toast-surface-from to-toast-surface-to shadow-[var(--toast-shadow)]
    // (transition/opacity 제거 — Remotion 이 외부에서 slide-in 제어)
    <div
      className={
        "relative w-[380px] h-[140px] flex rounded-xl overflow-hidden select-none " +
        "bg-gradient-to-b from-toast-surface-from to-toast-surface-to " +
        "shadow-[var(--toast-shadow)] font-sans"
      }
      style={{ ["--toast-accent" as string]: styles.accentVar }}
    >
      {/* Event-color corner glow + edge ring — App.vue 의 z-0 레이어 그대로 */}
      <div
        className="pointer-events-none absolute inset-0 rounded-xl z-0"
        style={{
          background:
            "radial-gradient(150px 80px at 0% 0%, color-mix(in oklch, var(--toast-accent) var(--toast-glow), transparent), transparent 72%)",
          boxShadow:
            "inset 0 0 0 1px color-mix(in oklch, var(--toast-accent) var(--toast-ring), transparent), inset 0 1px 0 0 var(--toast-highlight)",
        }}
      />

      {/* 왼쪽 액센트 바 없음 (bar:none 기본값) — App.vue 와 동일 */}

      {/* Content — App.vue: relative z-10 flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-[var(--toast-text-shadow)] */}
      <div className="relative z-10 flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-[var(--toast-text-shadow)]">
        {/* Dismiss progress — App.vue 의 absolute bottom-0 h-[3px] bg-overlay-subtle */}
        <div className="absolute bottom-0 left-0 right-0 h-[3px] bg-overlay-subtle">
          <div
            className={"h-full w-full origin-right " + styles.dismissBar}
            style={{ transform: `scaleX(${progress})` }}
          />
        </div>

        {/* Header — App.vue: flex items-center justify-between text-shadow-none */}
        <div className="flex items-center justify-between text-shadow-none">
          <div className="flex items-center gap-1.5">
            {/* Event icon — size-5 rounded-md + styles.icon */}
            <span
              className={
                "size-5 rounded-md flex items-center justify-center " +
                styles.icon
              }
            >
              <Check size={12} />
            </span>
            {/* Source logo (real claude.svg) — size-3.5 object-contain opacity-85 */}
            <img
              className="size-3.5 object-contain opacity-85"
              src={staticFile("claude.svg")}
              alt=""
            />
            {/* Event label — text-[13px] font-semibold tracking-wide + accent text-shadow glow + styles.label */}
            <span
              className={
                "text-[13px] font-semibold tracking-wide text-shadow-[0_0_8px_color-mix(in_oklch,var(--toast-accent)_55%,transparent)] " +
                styles.label
              }
            >
              {eventLabel}
            </span>
          </div>
          {/* Close X — App.vue: size-6 rounded-md text-toast-fg-dim */}
          <button
            type="button"
            className="size-6 flex items-center justify-center rounded-md text-toast-fg-dim"
          >
            <X size={14} />
          </button>
        </div>

        {/* Body — App.vue: flex flex-col gap-0.5 min-w-0 */}
        <div className="flex flex-col gap-0.5 min-w-0">
          {/* window_title — App.vue: text-[14px] font-bold text-toast-fg truncate leading-snug */}
          <div className="text-[14px] font-bold text-toast-fg truncate leading-snug">
            {windowTitle}
          </div>
          {/* message (optional) — App.vue: text-xs font-medium text-toast-fg-dim line-clamp-2 leading-snug */}
          {message && (
            <div className="text-xs font-medium text-toast-fg-dim line-clamp-2 leading-snug">
              {message}
            </div>
          )}
        </div>

        {/* Actions — App.vue: flex gap-1.5 */}
        <div className="flex gap-1.5">
          {/* View button — App.vue: flex-1 ... py-1.5 text-[13px] font-semibold rounded-md border + styles.viewBtn (솔리드) */}
          <button
            type="button"
            className={
              "flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-semibold rounded-md border " +
              styles.viewBtnBase + " " +
              (hovered ? styles.viewBtnBgHover : styles.viewBtnBg)
            }
          >
            <ChevronRight size={14} />
            {viewLabel}
          </button>
          {/* Close button — App.vue: flex-1 py-1.5 text-[13px] font-medium rounded-md
              bg-[toast-fg 7%] text-toast-fg border border-toast-border */}
          <button
            type="button"
            className="flex-1 py-1.5 text-[13px] font-medium rounded-md bg-[color-mix(in_oklch,var(--toast-fg)_7%,transparent)] text-toast-fg border border-toast-border"
          >
            {closeLabel}
          </button>
        </div>
      </div>
    </div>
  );
};
