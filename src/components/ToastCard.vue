<script setup lang="ts">
import { Check, ChevronRight, Circle, Pencil, X } from "lucide-vue-next";
import MarkdownIt from "markdown-it";
import { computed, type Component } from "vue";
import { useI18n } from "vue-i18n";
import claudeLogo from "../assets/claude.svg";
import openaiLogo from "../assets/openai.svg";
import type { NotificationData, ToastStyle } from "../types";

const { t } = useI18n();

type EventType = "default" | "success" | "warning" | "error" | "codex";

const props = withDefaults(
  defineProps<{
    notification: NotificationData;
    toastStyle: ToastStyle;
    show: boolean;
    dismissActive?: boolean;
    dismissPaused?: boolean;
    isDevMode?: boolean;
  }>(),
  { dismissActive: false, dismissPaused: false, isDevMode: false },
);

const emit = defineEmits<{ view: []; close: [] }>();

const isCodex = computed(() => props.notification.source === "codex");
const isRemote = computed(() => !!props.notification.hostname);
const showHostname = computed(
  () => isRemote.value && !!props.notification.show_hostname,
);
const hostnameLabel = computed(() =>
  showHostname.value ? props.notification.hostname : null,
);
const truncate = (s: string | null | undefined, n = 500) =>
  !s ? s : s.length > n ? s.slice(0, n) + "…" : s;

// zero 프리셋: 모든 규칙 꺼진 상태에서 인라인 강조만 활성화 — 나머지 문법은 원문 그대로
const md = new MarkdownIt("zero", { html: false }).enable([
  "emphasis",
  "backticks",
  "strikethrough",
]);

const messageHtml = computed(() => {
  const message = truncate(props.notification.message);
  return message ? md.renderInline(message) : null;
});

const eventType = computed<EventType>(() => {
  if (isCodex.value) return "codex";
  const ev = props.notification.event_display;
  if (ev === "task_complete") return "success";
  if (ev === "user_input_required") return "warning";
  if (ev === "error") return "error";
  return "default";
});

const eventLabel = computed(() => {
  const ev = props.notification.event_display;
  const key = `event.${ev}`;
  const translated = t(key);
  return translated === key ? ev : translated;
});

const eventIconMap: Record<EventType, Component> = {
  success: Check,
  warning: Pencil,
  error: X,
  codex: Check,
  default: Circle,
};

const eventIcon = computed(() => eventIconMap[eventType.value]);

const sourceLogo = computed(() => (isCodex.value ? openaiLogo : claudeLogo));

const isUpdateAvailable = computed(
  () =>
    props.notification.source === "updater" &&
    props.notification.event_display === "update_available",
);

const viewButtonText = computed(() =>
  isUpdateAvailable.value ? t("notification.update") : t("notification.view"),
);

const eventStyles: Record<
  EventType,
  {
    accent: string;
    icon: string;
    label: string;
    viewBtn: string;
    dismissBar: string;
    accentVar: string;
  }
> = {
  default: {
    accent: "bg-gradient-to-b from-event-default-from to-event-default-to",
    icon: "bg-event-default/20 text-event-default",
    label: "text-event-default",
    viewBtn:
      "bg-event-default text-zinc-950 border-transparent hover:bg-event-default-to",
    dismissBar: "bg-overlay-border/25",
    accentVar: "var(--event-default)",
  },
  success: {
    accent: "bg-gradient-to-b from-event-success to-event-success-deep",
    icon: "bg-event-success/20 text-event-success",
    label: "text-event-success",
    viewBtn:
      "bg-event-success text-zinc-950 border-transparent hover:bg-event-success-deep",
    dismissBar: "bg-event-success/30",
    accentVar: "var(--event-success)",
  },
  warning: {
    accent: "bg-gradient-to-b from-event-warning to-event-warning-deep",
    icon: "bg-event-warning/20 text-event-warning",
    label: "text-event-warning",
    viewBtn:
      "bg-event-warning text-zinc-950 border-transparent hover:bg-event-warning-deep",
    dismissBar: "bg-event-warning/30",
    accentVar: "var(--event-warning)",
  },
  error: {
    accent: "bg-gradient-to-b from-event-error to-event-error-deep",
    icon: "bg-event-error/20 text-event-error",
    label: "text-event-error",
    viewBtn:
      "bg-event-error text-zinc-950 border-transparent hover:bg-event-error-deep",
    dismissBar: "bg-event-error/30",
    accentVar: "var(--event-error)",
  },
  codex: {
    accent: "bg-gradient-to-b from-event-codex to-event-codex-deep",
    icon: "bg-event-codex/20 text-event-codex",
    label: "text-event-codex",
    viewBtn:
      "bg-event-codex text-zinc-950 border-transparent hover:bg-event-codex-deep",
    dismissBar: "bg-event-codex/30",
    accentVar: "var(--event-codex)",
  },
};

const styles = computed(() => eventStyles[eventType.value]);

const hasEffect = (name: string) => props.toastStyle.effects.includes(name);

// 회전 꼬리 페이드: 길이가 다른 대시를 머리 위치에 정렬해 겹침.
// SVG는 경로 방향 그라데이션이 없어 계단 근사가 한계 —
// 겹을 잘게(12) 나누고 꼬리 겹에 미세 블러를 줘 경계를 지운다
const TAIL_SEG_COUNT = 12;
const tailSegments = Array.from({ length: TAIL_SEG_COUNT }, (_, i) => {
  const head = i / (TAIL_SEG_COUNT - 1); // 0 = 꼬리 끝, 1 = 머리
  const len = 14 - 12 * head; // 대시 길이 14 → 2
  return {
    dash: `${len} ${100 - len}`,
    shift: `${len - 14}px`,
    opacity: 0.08 + 0.92 * head ** 1.6,
    blur: head === 1 ? undefined : "blur(0.5px)",
  };
});

// 배경(body) 축: glow = 좌상단 라디얼, tint = 표면 전체, flat = 없음
const bodyBackground = computed(() => {
  if (props.toastStyle.body === "tint")
    return "linear-gradient(180deg, color-mix(in oklch, var(--toast-accent) 16%, transparent), color-mix(in oklch, var(--toast-accent) 9%, transparent))";
  if (props.toastStyle.body === "flat") return "none";
  return "radial-gradient(150px 80px at 0% 0%, color-mix(in oklch, var(--toast-accent) var(--toast-glow), transparent), transparent 72%)";
});

// 테두리(border) 축: subtle = 기존 --toast-ring 비율, accent = 이벤트색 70%
const ringShadow = computed(() => {
  const ring =
    props.toastStyle.border === "accent"
      ? "color-mix(in oklch, var(--toast-accent) 70%, transparent)"
      : "color-mix(in oklch, var(--toast-accent) var(--toast-ring), transparent)";
  return `inset 0 0 0 1px ${ring}, inset 0 1px 0 0 var(--toast-highlight)`;
});
</script>

<template>
  <div
    class="relative flex rounded-xl overflow-hidden select-none bg-gradient-to-b from-toast-surface-from to-toast-surface-to shadow-[var(--toast-shadow)] opacity-0 translate-x-5 scale-[0.97] transition-all duration-300 ease-out"
    :class="{ 'opacity-100! translate-x-0! scale-100!': show }"
    :style="{ '--toast-accent': styles.accentVar }"
  >
    <!-- 배경 + 테두리 레이어 (breathe 이펙트 대상) -->
    <div
      class="pointer-events-none absolute inset-0 rounded-xl z-0"
      :class="{ 'toast-breathe': hasEffect('breathe') }"
      :style="{ background: bodyBackground, boxShadow: ringShadow }"
    />

    <!-- effect: pulse (등장 시 안쪽 파동) -->
    <div
      v-if="hasEffect('pulse')"
      class="pointer-events-none absolute inset-0 rounded-xl z-0 toast-pulse"
    />

    <!-- effect: shimmer (광택 스윕) -->
    <div
      v-if="hasEffect('shimmer')"
      class="pointer-events-none absolute inset-0 rounded-xl z-0 toast-shimmer"
    />

    <!-- effect: ring (SVG 대시 — 둘레 기준 균등 속도 회전) -->
    <svg
      v-if="hasEffect('ring')"
      class="pointer-events-none absolute inset-0 z-0 size-full"
    >
      <rect
        class="toast-ring-tail toast-ring-tail-glow"
        x="1.25"
        y="1.25"
        rx="10.75"
        pathLength="100"
      />
      <rect
        v-for="(s, i) in tailSegments"
        :key="i"
        class="toast-ring-tail"
        x="1.25"
        y="1.25"
        rx="10.75"
        pathLength="100"
        :style="{
          strokeDasharray: s.dash,
          '--tail-shift': s.shift,
          opacity: s.opacity,
          filter: s.blur,
        }"
      />
    </svg>

    <!-- 바(bar) 축: 왼쪽 액센트 바 -->
    <div
      v-if="toastStyle.bar === 'left'"
      class="relative z-10 w-1 shrink-0"
      :class="styles.accent"
    />

    <!-- Content -->
    <div
      class="relative z-10 flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-[var(--toast-text-shadow)]"
    >
      <!-- Dismiss progress -->
      <div
        v-if="notification.auto_dismiss_seconds > 0"
        class="absolute bottom-0 left-0 right-0 h-[3px] bg-overlay-subtle"
      >
        <div
          class="h-full w-full origin-right"
          :class="[
            styles.dismissBar,
            dismissActive ? 'animate-[shrink_linear_forwards]' : 'scale-x-100',
          ]"
          :style="
            dismissActive
              ? {
                  animationDuration: notification.auto_dismiss_seconds + 's',
                  animationName: 'shrink',
                  animationTimingFunction: 'linear',
                  animationFillMode: 'forwards',
                  animationPlayState: dismissPaused ? 'paused' : 'running',
                }
              : {}
          "
        />
      </div>

      <!-- Header -->
      <div class="flex items-center justify-between text-shadow-none">
        <div class="flex items-center gap-1.5">
          <span
            class="size-5 rounded-md flex items-center justify-center"
            :class="styles.icon"
          >
            <component :is="eventIcon" :size="12" />
          </span>
          <img
            class="size-3.5 object-contain opacity-85"
            :src="sourceLogo"
            alt=""
          />
          <span
            class="text-[13px] font-semibold tracking-wide text-shadow-[0_0_8px_color-mix(in_oklch,var(--toast-accent)_55%,transparent)]"
            :class="styles.label"
            >{{ eventLabel }}</span
          >
          <span
            v-if="isDevMode"
            class="px-1 py-0.5 text-[9px] font-bold bg-red-500 text-white rounded"
            >DEV</span
          >
        </div>
        <button
          class="size-6 flex items-center justify-center rounded-md text-toast-fg-dim hover:text-toast-fg hover:bg-[color-mix(in_oklch,var(--toast-fg)_10%,transparent)] transition-colors"
          @click="emit('close')"
          :aria-label="t('notification.close')"
        >
          <X :size="14" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex flex-col gap-0.5 min-w-0">
        <div class="text-[14px] font-bold text-toast-fg truncate leading-snug">
          {{ truncate(notification.window_title) }}
          <span
            v-if="hostnameLabel"
            class="ml-1.5 text-[11px] font-normal text-toast-fg-dim"
          >
            @ {{ truncate(hostnameLabel) }}
          </span>
        </div>
        <!-- messageHtml은 markdown-it(zero, html:false)이 이스케이프한 인라인 강조 결과 -->
        <div
          v-if="messageHtml"
          class="text-xs font-medium text-toast-fg-dim line-clamp-2 leading-snug [&_strong]:font-semibold [&_strong]:text-toast-fg [&_code]:font-mono [&_code]:text-[11px] [&_code]:rounded [&_code]:px-1 [&_code]:bg-[color-mix(in_oklch,var(--toast-fg)_10%,transparent)]"
          v-html="messageHtml"
        />
      </div>

      <!-- Actions -->
      <div class="flex gap-1.5">
        <button
          v-if="!isRemote || isUpdateAvailable"
          class="flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-semibold rounded-md border transition-colors"
          :class="styles.viewBtn"
          @click="emit('view')"
        >
          <ChevronRight :size="14" />
          {{ viewButtonText }}
        </button>
        <button
          class="flex-1 py-1.5 text-[13px] font-medium rounded-md bg-[color-mix(in_oklch,var(--toast-fg)_7%,transparent)] text-toast-fg border border-toast-border hover:bg-[color-mix(in_oklch,var(--toast-fg)_14%,transparent)] transition-colors"
          @click="emit('close')"
        >
          {{ t("notification.close") }}
        </button>
      </div>
    </div>
  </div>
</template>
