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
};

// App.vue 의 eventStyles.success 그대로
const styles = {
  accent: "bg-gradient-to-b from-event-success to-event-success-deep",
  icon: "bg-event-success/20 text-event-success",
  label: "text-event-success",
  // App.vue: bg-event-success/20 text-event-success border-event-success/40 hover:bg-event-success/30
  // 영상에선 hover pseudo 못 쓰니 base + bg 분리. base 는 항상 적용.
  viewBtnBase: "text-event-success border-event-success/40",
  viewBtnBg: "bg-event-success/20",
  viewBtnBgHover: "bg-event-success/30",
  dismissBar: "bg-event-success/30",
};

export const FakeToast: React.FC<Props> = ({
  eventLabel,
  windowTitle,
  message,
  progress = 1,
  hovered = false,
}) => {
  return (
    // App.vue: h-screen flex rounded-xl overflow-hidden bg-overlay-bg select-none
    // (transition/opacity 제거 — Remotion 이 외부에서 slide-in 제어)
    <div
      className={
        "w-[380px] h-[140px] flex rounded-xl overflow-hidden bg-overlay-bg select-none " +
        "shadow-[0_20px_50px_rgba(0,0,0,0.5),0_0_0_1px_rgba(255,255,255,0.08)] " +
        "font-sans"
      }
    >
      {/* Accent bar — App.vue: w-1 shrink-0 + styles.accent */}
      <div className={"w-1 shrink-0 " + styles.accent} />

      {/* Content — App.vue: relative flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-lg */}
      <div className="relative flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-lg">
        {/* Dismiss progress — App.vue 의 absolute bottom-0 h-[3px] bg-overlay-subtle */}
        <div className="absolute bottom-0 left-0 right-0 h-[3px] bg-overlay-subtle">
          <div
            className={"h-full w-full origin-right " + styles.dismissBar}
            style={{ transform: `scaleX(${progress})` }}
          />
        </div>

        {/* Header — App.vue: flex items-center justify-between */}
        <div className="flex items-center justify-between">
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
            {/* Event label — text-[13px] font-semibold tracking-wide + styles.label */}
            <span
              className={
                "text-[13px] font-semibold tracking-wide " + styles.label
              }
            >
              {eventLabel}
            </span>
          </div>
          {/* Close X — size-6 rounded-md text-white/50 */}
          <button
            type="button"
            className="size-6 flex items-center justify-center rounded-md text-white/50"
          >
            <X size={14} />
          </button>
        </div>

        {/* Body — App.vue: flex flex-col gap-0.5 min-w-0 */}
        <div className="flex flex-col gap-0.5 min-w-0">
          {/* window_title — text-[14px] font-medium text-white/90 truncate leading-snug */}
          <div className="text-[14px] font-medium text-white/90 truncate leading-snug">
            {windowTitle}
          </div>
          {/* message (optional) — text-xs text-white/60 truncate leading-snug */}
          {message && (
            <div className="text-xs text-white/60 truncate leading-snug">
              {message}
            </div>
          )}
        </div>

        {/* Actions — App.vue: flex gap-1.5 */}
        <div className="flex gap-1.5">
          {/* View button — flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-medium rounded-md border + styles.viewBtn */}
          <button
            type="button"
            className={
              "flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-medium rounded-md border " +
              styles.viewBtnBase + " " +
              (hovered ? styles.viewBtnBgHover : styles.viewBtnBg)
            }
          >
            <ChevronRight size={14} />
            보기
          </button>
          {/* Close button — flex-1 py-1.5 text-[13px] font-medium rounded-md bg-white/15 text-white/80 border border-white/20 */}
          <button
            type="button"
            className="flex-1 py-1.5 text-[13px] font-medium rounded-md bg-white/15 text-white/80 border border-white/20"
          >
            닫기
          </button>
        </div>
      </div>
    </div>
  );
};
