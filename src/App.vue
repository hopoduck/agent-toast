<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { Check, ChevronRight, Circle, Pencil, X } from "lucide-vue-next";
import { computed, onMounted, ref, type Component } from "vue";
import { useI18n } from "vue-i18n";
import claudeLogo from "./assets/claude.svg";
import openaiLogo from "./assets/openai.svg";
import type { NotificationData } from "./types";

const { t, locale } = useI18n();

type EventType = "default" | "success" | "warning" | "error" | "codex";

const notification = ref<NotificationData | null>(null);
const show = ref(false);
const isDevMode = ref(false);
const dismissActive = ref(false);
const dismissPaused = ref(false);
let dismissTimer: ReturnType<typeof setTimeout> | null = null;
let currentNotificationId: string | null = null;
let dismissStartedAt = 0;
let dismissRemainingMs = 0;

function startDismissTimer() {
  if (dismissTimer) clearTimeout(dismissTimer);
  dismissActive.value = false;
  const seconds = notification.value?.auto_dismiss_seconds ?? 0;
  if (seconds > 0) {
    dismissRemainingMs = seconds * 1000;
    dismissPaused.value = false;
    requestAnimationFrame(() => {
      dismissActive.value = true;
    });
    dismissStartedAt = performance.now();
    dismissTimer = setTimeout(onClose, dismissRemainingMs);
  }
}

function pauseDismiss() {
  if (dismissTimer === null || dismissPaused.value) return;
  clearTimeout(dismissTimer);
  dismissTimer = null;
  dismissRemainingMs -= performance.now() - dismissStartedAt;
  dismissPaused.value = true;
}

function resumeDismiss() {
  if (!dismissPaused.value) return;
  dismissPaused.value = false;
  if (dismissRemainingMs <= 0) {
    onClose();
    return;
  }
  dismissStartedAt = performance.now();
  dismissTimer = setTimeout(onClose, dismissRemainingMs);
}

const isCodex = computed(() => notification.value?.source === "codex");

const isRemote = computed(() => !!notification.value?.hostname);
const showHostname = computed(
  () => isRemote.value && !!notification.value?.show_hostname,
);
const hostnameLabel = computed(() =>
  showHostname.value ? notification.value!.hostname : null,
);
const truncate = (s: string | null | undefined, n = 500) =>
  !s ? s : s.length > n ? s.slice(0, n) + "…" : s;

const eventType = computed<EventType>(() => {
  if (!notification.value) return "default";
  if (isCodex.value) return "codex";
  const ev = notification.value.event_display;
  if (ev === "task_complete") return "success";
  if (ev === "user_input_required") return "warning";
  if (ev === "error") return "error";
  return "default";
});

const eventLabel = computed(() => {
  if (!notification.value) return "";
  const ev = notification.value.event_display;
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
  () => notification.value?.source === "updater" && notification.value?.event_display === "update_available",
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

function showNotification() {
  const id = notification.value?.id;
  if (!id || id === currentNotificationId) return;
  currentNotificationId = id;

  // double rAF: 초기 상태(opacity-0/scale)를 한 프레임 그린 뒤 토글해야 전환이 재생됨
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      show.value = true;
      startDismissTimer();
    });
  });
}

function applyTheme(theme: string) {
  const dark =
    theme === "dark" ||
    (theme === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.classList.toggle("dark", dark);
}

onMounted(async () => {
  try {
    const savedLocale = await invoke<string>("get_locale");
    if (savedLocale) locale.value = savedLocale;
    const savedTheme = await invoke<string>("get_theme");
    applyTheme(savedTheme || "system");
    isDevMode.value = await invoke<boolean>("is_dev_mode");
  } catch {
    /* ignore */
  }

  const data = await invoke<NotificationData | null>("get_notification_data");
  if (data) {
    notification.value = data;
    showNotification();
  }

  getCurrentWebviewWindow().listen<NotificationData>(
    "notification-data",
    (event) => {
      notification.value = event.payload;
      showNotification();
    },
  );
});

async function onView() {
  if (!notification.value) return;

  // For update_completed notifications, open settings
  if (notification.value.source === "updater" && !isUpdateAvailable.value) {
    const closeId = notification.value.id;
    show.value = false;
    await invoke("open_settings", { tab: "about" });
    setTimeout(async () => {
      await invoke("close_notify", { id: closeId });
    }, 300);
    return;
  }

  // For update_available notifications, directly download and install
  if (isUpdateAvailable.value) {
    const closeId = notification.value.id;
    show.value = false;

    const portable = await invoke<boolean>("is_portable");
    if (portable) {
      await openUrl("https://github.com/hopoduck/agent-toast/releases/latest");
      await invoke("close_notify", { id: closeId });
      return;
    }

    try {
      const update = await check();
      if (update) {
        await invoke("mark_update_pending", { version: update.version });
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch {
      await openUrl("https://github.com/hopoduck/agent-toast/releases/latest");
    }

    await invoke("close_notify", { id: closeId });
    return;
  }

  await invoke("activate_source", {
    hwnd: notification.value.source_hwnd,
    id: notification.value.id,
  });
}

async function onClose() {
  if (!notification.value) return;
  const closeId = notification.value.id;
  show.value = false;
  currentNotificationId = null;
  setTimeout(async () => {
    await invoke("close_notify", { id: closeId });
  }, 300);
}
</script>

<template>
  <div
    v-if="notification"
    class="relative h-screen flex rounded-xl overflow-hidden select-none bg-gradient-to-b from-toast-surface-from to-toast-surface-to shadow-[var(--toast-shadow)] opacity-0 translate-x-5 scale-[0.97] transition-all duration-300 ease-out"
    :class="{ 'opacity-100! translate-x-0! scale-100!': show }"
    :style="{ '--toast-accent': styles.accentVar }"
    @mouseenter="pauseDismiss"
    @mouseleave="resumeDismiss"
  >
    <!-- Event-color corner glow + edge ring -->
    <div
      class="pointer-events-none absolute inset-0 rounded-xl z-0"
      :style="{
        background:
          'radial-gradient(150px 80px at 0% 0%, color-mix(in oklch, var(--toast-accent) var(--toast-glow), transparent), transparent 72%)',
        boxShadow:
          'inset 0 0 0 1px color-mix(in oklch, var(--toast-accent) var(--toast-ring), transparent), inset 0 1px 0 0 var(--toast-highlight)',
      }"
    />

    <!-- Accent bar -->
    <div class="relative z-10 w-1 shrink-0" :class="styles.accent" />

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
          @click="onClose"
          :aria-label="t('notification.close')"
        >
          <X :size="14" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex flex-col gap-0.5 min-w-0">
        <div
          class="text-[14px] font-bold text-toast-fg truncate leading-snug"
        >
          {{ truncate(notification.window_title) }}
          <span
            v-if="hostnameLabel"
            class="ml-1.5 text-[11px] font-normal text-toast-fg-dim"
          >
            @ {{ truncate(hostnameLabel) }}
          </span>
        </div>
        <div
          v-if="notification.message"
          class="text-xs font-medium text-toast-fg-dim line-clamp-2 leading-snug"
        >
          {{ truncate(notification.message) }}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-1.5">
        <button
          v-if="!isRemote || isUpdateAvailable"
          class="flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-semibold rounded-md border transition-colors"
          :class="styles.viewBtn"
          @click="onView"
        >
          <ChevronRight :size="14" />
          {{ viewButtonText }}
        </button>
        <button
          class="flex-1 py-1.5 text-[13px] font-medium rounded-md bg-[color-mix(in_oklch,var(--toast-fg)_7%,transparent)] text-toast-fg border border-toast-border hover:bg-[color-mix(in_oklch,var(--toast-fg)_14%,transparent)] transition-colors"
          @click="onClose"
        >
          {{ t("notification.close") }}
        </button>
      </div>
    </div>
  </div>
</template>
