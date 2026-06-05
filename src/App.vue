<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import ToastCard from "./components/ToastCard.vue";
import type { NotificationData, ToastStyle } from "./types";

const { locale } = useI18n();

const notification = ref<NotificationData | null>(null);
const show = ref(false);
const isDevMode = ref(false);
const dismissActive = ref(false);
const dismissPaused = ref(false);
const toastStyle = ref<ToastStyle>({
  bar: "left",
  border: "subtle",
  effects: [],
  body: "glow",
});
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

const isUpdateAvailable = () =>
  notification.value?.source === "updater" &&
  notification.value?.event_display === "update_available";

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
    toastStyle.value = await invoke<ToastStyle>("get_toast_style");
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
  if (notification.value.source === "updater" && !isUpdateAvailable()) {
    const closeId = notification.value.id;
    show.value = false;
    await invoke("open_settings", { tab: "about" });
    setTimeout(async () => {
      await invoke("close_notify", { id: closeId });
    }, 300);
    return;
  }

  // For update_available notifications, directly download and install
  if (isUpdateAvailable()) {
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
  <ToastCard
    v-if="notification"
    class="h-screen"
    :notification="notification"
    :toast-style="toastStyle"
    :show="show"
    :dismiss-active="dismissActive"
    :dismiss-paused="dismissPaused"
    :is-dev-mode="isDevMode"
    @view="onView"
    @close="onClose"
    @mouseenter="pauseDismiss"
    @mouseleave="resumeDismiss"
  />
</template>
