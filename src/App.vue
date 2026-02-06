<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { Check, ChevronRight, Circle, Pencil, X } from "lucide-vue-next";
import { computed, onMounted, ref, type Component } from "vue";
import { useI18n } from "vue-i18n";
import claudeLogo from "./assets/claude.svg";
import openaiLogo from "./assets/openai.svg";

const { t, locale } = useI18n();

interface NotificationData {
  id: string;
  window_title: string;
  event_display: string;
  message: string | null;
  source_hwnd: number;
  process_tree: number[];
  auto_dismiss_seconds: number;
  source: string;
}

type EventType = "default" | "success" | "warning" | "error" | "codex";

const notification = ref<NotificationData | null>(null);
const show = ref(false);
const dismissActive = ref(false);
let dismissTimer: ReturnType<typeof setTimeout> | null = null;
let currentNotificationId: string | null = null;

function startDismissTimer() {
  if (dismissTimer) clearTimeout(dismissTimer);
  dismissActive.value = false;
  const seconds = notification.value?.auto_dismiss_seconds ?? 0;
  if (seconds > 0) {
    requestAnimationFrame(() => {
      dismissActive.value = true;
    });
    dismissTimer = setTimeout(() => {
      onClose();
    }, seconds * 1000);
  }
}

const isCodex = computed(() => notification.value?.source === "codex");

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

const isUpdater = computed(() => notification.value?.source === "updater");

const viewButtonText = computed(() =>
  isUpdater.value ? t("notification.update") : t("notification.view"),
);

const eventStyles: Record<
  EventType,
  {
    accent: string;
    icon: string;
    label: string;
    viewBtn: string;
    dismissBar: string;
  }
> = {
  default: {
    accent: "bg-gradient-to-b from-event-default-from to-event-default-to",
    icon: "bg-event-default/20 text-event-default",
    label: "text-event-default",
    viewBtn:
      "bg-event-default/20 text-event-default border-event-default/40 hover:bg-event-default/30",
    dismissBar: "bg-overlay-border/25",
  },
  success: {
    accent: "bg-gradient-to-b from-event-success to-event-success-deep",
    icon: "bg-event-success/20 text-event-success",
    label: "text-event-success",
    viewBtn:
      "bg-event-success/20 text-event-success border-event-success/40 hover:bg-event-success/30",
    dismissBar: "bg-event-success/30",
  },
  warning: {
    accent: "bg-gradient-to-b from-event-warning to-event-warning-deep",
    icon: "bg-event-warning/20 text-event-warning",
    label: "text-event-warning",
    viewBtn:
      "bg-event-warning/20 text-event-warning border-event-warning/40 hover:bg-event-warning/30",
    dismissBar: "bg-event-warning/30",
  },
  error: {
    accent: "bg-gradient-to-b from-event-error to-event-error-deep",
    icon: "bg-event-error/20 text-event-error",
    label: "text-event-error",
    viewBtn:
      "bg-event-error/20 text-event-error border-event-error/40 hover:bg-event-error/30",
    dismissBar: "bg-event-error/30",
  },
  codex: {
    accent: "bg-gradient-to-b from-event-codex to-event-codex-deep",
    icon: "bg-event-codex/20 text-event-codex",
    label: "text-event-codex",
    viewBtn:
      "bg-event-codex/20 text-event-codex border-event-codex/40 hover:bg-event-codex/30",
    dismissBar: "bg-event-codex/30",
  },
};

const styles = computed(() => eventStyles[eventType.value]);

function showNotification() {
  const id = notification.value?.id;
  if (!id || id === currentNotificationId) return;
  currentNotificationId = id;

  requestAnimationFrame(() => {
    show.value = true;
    startDismissTimer();
  });
}

onMounted(async () => {
  try {
    const savedLocale = await invoke<string>("get_locale");
    if (savedLocale) locale.value = savedLocale;
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

  // For update notifications, directly download and install
  if (notification.value.source === "updater") {
    const closeId = notification.value.id;
    show.value = false;

    try {
      const update = await check();
      if (update) {
        await invoke("mark_update_pending", { version: update.version });
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (e) {
      console.error("Update failed:", e);
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
  }, 200);
}
</script>

<template>
  <div
    v-if="notification"
    class="h-screen flex rounded-xl overflow-hidden bg-overlay-bg select-none opacity-0 translate-x-5 transition-all duration-300 ease-out"
    :class="{ 'opacity-100! translate-x-0!': show }"
  >
    <!-- Accent bar -->
    <div class="w-1 shrink-0" :class="styles.accent" />

    <!-- Content -->
    <div
      class="relative flex-1 flex flex-col justify-between p-3 min-w-0 text-shadow-lg"
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
                }
              : {}
          "
        />
      </div>

      <!-- Header -->
      <div class="flex items-center justify-between">
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
            class="text-[13px] font-semibold tracking-wide"
            :class="styles.label"
            >{{ eventLabel }}</span
          >
        </div>
        <button
          class="size-6 flex items-center justify-center rounded-md text-white/50 hover:text-white/80 hover:bg-white/10 transition-colors"
          @click="onClose"
          :aria-label="t('notification.close')"
        >
          <X :size="14" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex flex-col gap-0.5 min-w-0">
        <div
          class="text-[14px] font-medium text-white/90 truncate leading-snug"
        >
          {{ notification.window_title }}
        </div>
        <div
          v-if="notification.message"
          class="text-xs text-white/60 truncate leading-snug"
        >
          {{ notification.message }}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-1.5">
        <button
          class="flex-1 flex items-center justify-center gap-1 py-1.5 text-[13px] font-medium rounded-md border transition-colors"
          :class="styles.viewBtn"
          @click="onView"
        >
          <ChevronRight :size="14" />
          {{ viewButtonText }}
        </button>
        <button
          class="flex-1 py-1.5 text-[13px] font-medium rounded-md bg-white/15 text-white/80 border border-white/20 hover:bg-white/25 transition-colors"
          @click="onClose"
        >
          {{ t("notification.close") }}
        </button>
      </div>
    </div>
  </div>
</template>
