<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Moon, RotateCcw, Sun } from "lucide-vue-next";
import { onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast, Toaster } from "vue-sonner";
import "vue-sonner/style.css";
import AboutSettings from "./components/AboutSettings.vue";
import GeneralSettings from "./components/GeneralSettings.vue";
import HookSettings from "./components/HookSettings.vue";
import SlidingTabs from "./components/SlidingTabs.vue";
import type { HookConfig } from "./types";

const { t, locale } = useI18n();
const activeTab = ref<string>("general");

const isDark = ref(document.documentElement.classList.contains("dark"));

function toggleTheme() {
  isDark.value = !isDark.value;
  document.documentElement.classList.toggle("dark", isDark.value);
  localStorage.setItem("theme", isDark.value ? "dark" : "light");
}

const config = ref<HookConfig>({
  stop_enabled: true,
  stop_message: "작업이 완료되었습니다",
  notification_permission_enabled: false,
  notification_permission_message: "권한 승인이 필요합니다",
  notification_elicitation_enabled: false,
  notification_elicitation_message: "입력이 필요합니다",
  notification_idle_enabled: false,
  notification_idle_message: "입력을 기다리고 있습니다",
  session_start_enabled: false,
  session_start_message: "세션이 시작되었습니다",
  session_end_enabled: false,
  session_end_message: "세션이 종료되었습니다",
  subagent_stop_enabled: false,
  subagent_stop_message: "서브에이전트가 완료되었습니다",
  pre_compact_enabled: false,
  pre_compact_message: "컨텍스트 압축이 시작됩니다",
  setup_enabled: false,
  setup_message: "초기화가 실행되었습니다",
  user_prompt_submit_enabled: false,
  user_prompt_submit_message: "프롬프트가 제출되었습니다",
  pre_tool_use_enabled: false,
  pre_tool_use_message: "도구 실행이 시작됩니다",
  post_tool_use_enabled: false,
  post_tool_use_message: "도구 실행이 완료되었습니다",
  post_tool_use_failure_enabled: false,
  post_tool_use_failure_message: "도구 실행이 실패했습니다",
  permission_request_enabled: true,
  permission_request_message: "권한 요청이 발생했습니다",
  subagent_start_enabled: false,
  subagent_start_message: "서브에이전트가 시작되었습니다",
  title_display_mode: "project",
  auto_close_on_focus: true,
  auto_dismiss_seconds: 0,
  notification_position: "bottom_right",
  notification_sound: true,
  notification_monitor: "primary",
  locale: "ko",
  codex_enabled: false,
});

watch(
  () => config.value.locale,
  (newLocale) => {
    locale.value = newLocale;
  },
);

const configSaved = ref(true);
const exePath = ref("");
const savedExePath = ref<string | null>(null);
const exePathMismatch = ref(false);
const isSaving = ref(false);
const isDevMode = ref(false);

function normalizePath(p: string): string {
  return p.replace(/\//g, "\\").toLowerCase();
}

onMounted(async () => {
  // Check URL hash for initial tab
  const hash = window.location.hash.slice(1);
  if (hash && ["general", "hooks", "about"].includes(hash)) {
    activeTab.value = hash;
  }

  // Listen for hash changes (when window already exists)
  window.addEventListener("hashchange", () => {
    const h = window.location.hash.slice(1);
    if (h && ["general", "hooks", "about"].includes(h)) {
      activeTab.value = h;
    }
  });

  configSaved.value = await invoke<boolean>("is_hook_config_saved");
  if (configSaved.value) {
    config.value = await invoke<HookConfig>("get_hook_config");
  } else {
    const saved = await invoke<HookConfig>("get_hook_config");
    config.value.title_display_mode = saved.title_display_mode;
    config.value.auto_close_on_focus = saved.auto_close_on_focus;
    config.value.auto_dismiss_seconds = saved.auto_dismiss_seconds;
    config.value.notification_position = saved.notification_position;
    config.value.notification_sound = saved.notification_sound;
    config.value.notification_monitor = saved.notification_monitor;
    config.value.locale = saved.locale;
    config.value.codex_enabled = saved.codex_enabled;
  }
  locale.value = config.value.locale;
  exePath.value = await invoke<string>("get_exe_path");
  savedExePath.value = await invoke<string | null>("get_saved_exe_path");
  isDevMode.value = await invoke<boolean>("is_dev_mode");
  if (savedExePath.value) {
    exePathMismatch.value =
      normalizePath(savedExePath.value) !== normalizePath(exePath.value);
  }
});

async function onSave() {
  isSaving.value = true;
  try {
    const path = await invoke<string>("save_hook_config", {
      config: config.value,
    });
    const lines = path.split("\n");
    toast.success(t("setup.save_success", { path: lines[0] }), {
      description: lines[1]
        ? t("setup.save_success", { path: lines[1] })
        : undefined,
    });
    configSaved.value = true;
    exePathMismatch.value = false;
  } catch (e) {
    toast.error(t("setup.save_error", { msg: String(e) }));
  } finally {
    isSaving.value = false;
  }
}

async function onOpenSettings() {
  await invoke("open_settings_file");
}

function onReset() {
  const currentLocale = config.value.locale;
  config.value = {
    stop_enabled: true,
    stop_message: t("defaults.stop_message"),
    notification_permission_enabled: false,
    notification_permission_message: t(
      "defaults.notification_permission_message",
    ),
    notification_elicitation_enabled: false,
    notification_elicitation_message: t(
      "defaults.notification_elicitation_message",
    ),
    notification_idle_enabled: false,
    notification_idle_message: t("defaults.notification_idle_message"),
    session_start_enabled: false,
    session_start_message: t("defaults.session_start_message"),
    session_end_enabled: false,
    session_end_message: t("defaults.session_end_message"),
    subagent_stop_enabled: false,
    subagent_stop_message: t("defaults.subagent_stop_message"),
    pre_compact_enabled: false,
    pre_compact_message: t("defaults.pre_compact_message"),
    setup_enabled: false,
    setup_message: t("defaults.setup_message"),
    user_prompt_submit_enabled: false,
    user_prompt_submit_message: t("defaults.user_prompt_submit_message"),
    pre_tool_use_enabled: false,
    pre_tool_use_message: t("defaults.pre_tool_use_message"),
    post_tool_use_enabled: false,
    post_tool_use_message: t("defaults.post_tool_use_message"),
    post_tool_use_failure_enabled: false,
    post_tool_use_failure_message: t("defaults.post_tool_use_failure_message"),
    permission_request_enabled: true,
    permission_request_message: t("defaults.permission_request_message"),
    subagent_start_enabled: false,
    subagent_start_message: t("defaults.subagent_start_message"),
    title_display_mode: "project",
    auto_close_on_focus: true,
    auto_dismiss_seconds: 0,
    notification_position: "bottom_right",
    notification_sound: true,
    notification_monitor: "primary",
    locale: currentLocale,
    codex_enabled: false,
  };
}

async function onTestNotification() {
  await invoke("test_notification");
}

async function onClose() {
  await getCurrentWindow().close();
}
</script>

<template>
  <div class="bg-background h-full">
    <div
      class="max-w-[520px] mx-auto h-screen flex flex-col gap-4 p-6 overflow-y-auto"
    >
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <h1 class="text-xl font-semibold">{{ t("setup.title") }}</h1>
          <span
            v-if="isDevMode"
            class="px-1.5 py-0.5 text-[10px] font-bold bg-red-500 text-white rounded"
            >DEV</span
          >
        </div>
        <div class="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon-sm"
            @click="toggleTheme"
            :title="isDark ? 'Light mode' : 'Dark mode'"
          >
            <Sun v-if="isDark" :size="16" />
            <Moon v-else :size="16" />
          </Button>
          <AlertDialog>
            <AlertDialogTrigger as-child>
              <Button
                variant="ghost"
                size="icon-sm"
                :title="t('setup.reset_title')"
              >
                <RotateCcw :size="16" />
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{{ t('setup.reset_title') }}</AlertDialogTitle>
                <AlertDialogDescription>{{ t('setup.reset_confirm') }}</AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>{{ t('setup.cancel') }}</AlertDialogCancel>
                <AlertDialogAction @click="onReset">{{ t('setup.reset_action') }}</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </div>

      <!-- Path mismatch warning -->
      <Card
        v-if="exePathMismatch"
        class="border-event-warning/25 bg-event-warning/10 gap-1 py-3"
      >
        <CardContent class="flex flex-col gap-1 text-xs text-event-warning">
          <strong class="text-[13px]">{{
            $t("setup.path_mismatch_title")
          }}</strong>
          <p class="text-event-warning/70 text-sm">
            {{ $t("setup.path_mismatch_desc") }}
          </p>
          <div class="flex flex-col gap-0.5 text-[11px]">
            <div>
              <span class="text-event-warning/50">{{
                $t("setup.path_saved")
              }}</span>
              <code class="text-foreground/60 break-all">{{
                savedExePath
              }}</code>
            </div>
            <div>
              <span class="text-event-warning/50">{{
                $t("setup.path_current")
              }}</span>
              <code class="text-foreground/60 break-all">{{ exePath }}</code>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Not saved notice -->
      <Card v-if="!configSaved" class="border-primary/25 bg-accent gap-1 py-3">
        <CardContent class="flex flex-col gap-1 text-xs text-accent-foreground">
          <strong class="text-[13px]">{{ $t("setup.not_saved_title") }}</strong>
          <p class="text-accent-foreground/70 text-sm">
            {{ $t("setup.not_saved_desc") }}
          </p>
        </CardContent>
      </Card>

      <!-- Tabs -->
      <div class="flex flex-col flex-1 min-h-0 gap-4">
        <SlidingTabs
          v-model="activeTab"
          :tabs="[
            { value: 'general', label: t('setup.tab_general') },
            { value: 'hooks', label: t('setup.tab_hooks') },
            { value: 'about', label: t('setup.tab_about') },
          ]"
        />

        <div class="flex flex-col flex-1 min-h-0">
          <GeneralSettings
            v-if="activeTab === 'general'"
            v-model="config"
            @test-notification="onTestNotification"
          />
          <HookSettings v-if="activeTab === 'hooks'" v-model="config" />
          <AboutSettings v-if="activeTab === 'about'" />
        </div>
      </div>

      <!-- Exe info -->
      <div
        v-if="activeTab !== 'about'"
        class="flex flex-col gap-1 p-3 bg-muted rounded-lg"
      >
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground whitespace-nowrap">{{
            t("setup.exe_label")
          }}</span>
          <code class="text-[11px] text-foreground/60 break-all">{{
            exePath
          }}</code>
        </div>
        <span class="text-[11px] text-muted-foreground">{{
          t("setup.exe_hint")
        }}</span>
      </div>

      <!-- Actions -->
      <div v-if="activeTab !== 'about'" class="flex items-center gap-2">
        <Button variant="secondary" @click="onOpenSettings">
          {{ t("setup.open_settings") }}
        </Button>
        <div class="flex-1" />
        <Button variant="destructive" @click="onClose">
          {{ t("setup.close") }}
        </Button>
        <Button @click="onSave" :disabled="isSaving">
          {{ isSaving ? t("setup.saving") : t("setup.save") }}
        </Button>
      </div>
    </div>
    <Toaster
      position="top-center"
      :duration="3000"
      rich-colors
      class="opacity-80 drop-shadow-sm"
    />
  </div>
</template>
