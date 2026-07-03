<script setup lang="ts">
import { Button } from "@/components/ui/button";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Eye,
  FolderOpen,
  MonitorDot,
  Play,
  RotateCcw,
  SlidersHorizontal,
  Square,
} from "lucide-vue-next";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import type { HookConfig, MonitorInfo } from "../types";

const { t, locale } = useI18n();

const config = defineModel<HookConfig>({ required: true });

const titleOptions = [
  { value: "project", labelKey: "general.title_project" },
  { value: "window", labelKey: "general.title_window" },
] as const;

const positionOptions = [
  { value: "top_left", labelKey: "general.pos_top_left" },
  { value: "top_right", labelKey: "general.pos_top_right" },
  { value: "bottom_left", labelKey: "general.pos_bottom_left" },
  { value: "bottom_right", labelKey: "general.pos_bottom_right" },
] as const;

const emit = defineEmits<{
  "test-notification": [];
}>();

const monitors = ref<MonitorInfo[]>([]);

const previewPlaying = ref(false);
let unlistenPreviewEnded: UnlistenFn | undefined;

onMounted(async () => {
  try {
    monitors.value = await invoke<MonitorInfo[]>("get_monitor_list");
  } catch (e) {
    console.error("Failed to get monitor list:", e);
  }
  // 재생이 끝까지 가면 백엔드가 보내는 이벤트로 토글 버튼을 ▶로 복귀
  unlistenPreviewEnded = await getCurrentWebviewWindow().listen(
    "sound-preview-ended",
    () => {
      previewPlaying.value = false;
    },
  );
});

onUnmounted(() => {
  unlistenPreviewEnded?.();
  if (previewPlaying.value) {
    previewPlaying.value = false;
    invoke("stop_notification_sound");
  }
});

const soundFileName = computed(() => {
  const p = config.value.notification_sound_file;
  if (!p) return null;
  return p.split(/[\\/]/).pop() ?? p;
});

async function onBrowseSound() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: t("general.sound_file_filter"),
        extensions: ["mp3", "wav", "flac", "ogg", "aac", "wma"],
      },
    ],
  });
  if (typeof selected !== "string") return;
  try {
    const stored = await invoke<string>("copy_notification_sound_file", {
      path: selected,
    });
    config.value.notification_sound_file = stored;
  } catch (e) {
    toast.error(t("general.sound_copy_error", { msg: String(e) }));
  }
}

async function onPreviewSound() {
  if (previewPlaying.value) {
    previewPlaying.value = false;
    await invoke("stop_notification_sound");
    return;
  }
  previewPlaying.value = true;
  await invoke("preview_notification_sound", {
    path: config.value.notification_sound_file,
  });
}

function onResetSound() {
  config.value.notification_sound_file = null;
  if (previewPlaying.value) {
    previewPlaying.value = false;
    invoke("stop_notification_sound");
  }
}
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">
    <p class="anim-item text-[13px] text-muted-foreground" style="animation-delay:0ms">
      {{ t("general.desc") }}
    </p>

    <!-- Section: Display -->
    <section class="anim-item flex flex-col gap-1.5" style="animation-delay:20ms">
      <div class="flex items-center gap-1.5 px-1">
        <Eye :size="12" class="text-muted-foreground/50" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">표시</span>
      </div>
      <div class="rounded-[12px] border border-border overflow-hidden divide-y divide-border">
        <!-- Language -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.language") }}</span>
          <Select v-model="config.locale">
            <SelectTrigger size="sm" class="w-[120px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="ko">한국어</SelectItem>
              <SelectItem value="en">English</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- Title display -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.title_display") }}</span>
          <Select :key="`title-${locale}`" v-model="config.title_display_mode">
            <SelectTrigger size="sm" class="w-[120px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="o in titleOptions" :key="o.value" :value="o.value">
                {{ t(o.labelKey) }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>

    <!-- Section: Position -->
    <section class="anim-item flex flex-col gap-1.5" style="animation-delay:60ms">
      <div class="flex items-center gap-1.5 px-1">
        <MonitorDot :size="12" class="text-muted-foreground/50" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">위치</span>
      </div>
      <div class="rounded-[12px] border border-border overflow-hidden divide-y divide-border">
        <!-- Corner position picker -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.position") }}</span>

          <!-- Mini monitor corner picker -->
          <div class="relative w-[84px] h-[56px] rounded-[8px] border-2 border-border bg-muted shrink-0">
            <!-- Inner screen -->
            <div class="absolute inset-[7px] rounded-[3px] border border-border bg-background"></div>
            <!-- Stand bottom -->
            <div class="absolute -bottom-[5px] left-1/2 -translate-x-1/2 w-8 h-[3px] rounded-full bg-border"></div>

            <!-- Corner buttons -->
            <button
              v-for="pos in positionOptions"
              :key="pos.value"
              type="button"
              :title="t(pos.labelKey)"
              :class="[
                'absolute w-[18px] h-[18px] rounded-[4px] flex items-center justify-center transition-all duration-150',
                pos.value === 'top_left'     ? 'top-[2px] left-[2px]'   : '',
                pos.value === 'top_right'    ? 'top-[2px] right-[2px]'  : '',
                pos.value === 'bottom_left'  ? 'bottom-[2px] left-[2px]': '',
                pos.value === 'bottom_right' ? 'bottom-[2px] right-[2px]': '',
                config.notification_position === pos.value
                  ? 'bg-primary shadow-sm scale-110'
                  : 'bg-muted-foreground/30 hover:bg-muted-foreground/45',
              ]"
              @click="config.notification_position = pos.value"
            >
              <div :class="[
                'w-[7px] h-[7px] rounded-[2px] transition-all duration-150',
                config.notification_position === pos.value
                  ? 'bg-primary-foreground'
                  : 'bg-muted-foreground/70',
              ]"></div>
            </button>
          </div>
        </div>

        <!-- Monitor -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.monitor") }}</span>
          <Select :key="`monitor-${locale}`" v-model="config.notification_monitor">
            <SelectTrigger size="sm" class="w-[120px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="primary">{{ t("general.monitor_primary") }}</SelectItem>
              <SelectItem v-for="(m, i) in monitors" :key="i" :value="String(i)">
                {{ m.name }}{{ m.is_primary ? ` ${t("general.monitor_primary_suffix")}` : "" }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>

    <!-- Section: Behavior -->
    <section class="anim-item flex flex-col gap-1.5" style="animation-delay:100ms">
      <div class="flex items-center gap-1.5 px-1">
        <SlidersHorizontal :size="12" class="text-muted-foreground/50" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">동작</span>
      </div>
      <div class="rounded-[12px] border border-border overflow-hidden divide-y divide-border">
        <!-- Dynamic message -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <div class="flex flex-col gap-0.5 min-w-0 pr-2">
            <span class="text-sm font-medium text-foreground">{{ t("general.dynamic_message") }}</span>
            <span class="text-[11px] text-muted-foreground leading-tight">{{ t("general.dynamic_message_desc") }}</span>
          </div>
          <Switch v-model="config.dynamic_message_enabled" class="shrink-0" />
        </div>

        <!-- Auto dismiss -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <div class="flex flex-col gap-0.5">
            <span class="text-sm font-medium text-foreground">{{ t("general.auto_dismiss") }}</span>
            <span class="text-[11px] text-muted-foreground leading-none">{{ t("general.auto_dismiss_off") }}</span>
          </div>
          <NumberField
            v-model="config.auto_dismiss_seconds"
            :min="0"
            :max="300"
            :step="1"
            class="w-[96px]"
          >
            <NumberFieldContent>
              <NumberFieldDecrement class="p-2" />
              <NumberFieldInput class="h-7 text-xs" />
              <NumberFieldIncrement class="p-2" />
            </NumberFieldContent>
          </NumberField>
        </div>

        <!-- Auto close on focus -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.auto_close_focus") }}</span>
          <Switch v-model="config.auto_close_on_focus" />
        </div>

        <!-- Sound -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <span class="text-sm font-medium text-foreground">{{ t("general.sound") }}</span>
          <Switch v-model="config.notification_sound" />
        </div>

        <!-- Sound file (visible only while sound is on) -->
        <div
          v-if="config.notification_sound"
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <div class="flex flex-col gap-0.5 min-w-0 pr-2">
            <span class="text-sm font-medium text-foreground">{{ t("general.sound_file") }}</span>
            <span class="text-[11px] text-muted-foreground leading-tight truncate">
              {{ soundFileName ?? t("general.sound_file_default") }}
            </span>
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <Button
              variant="ghost"
              size="icon"
              class="size-7"
              :title="previewPlaying ? t('general.sound_stop') : t('general.sound_preview')"
              @click="onPreviewSound"
            >
              <component :is="previewPlaying ? Square : Play" />
            </Button>
            <Button
              v-if="config.notification_sound_file"
              variant="ghost"
              size="icon"
              class="size-7"
              :title="t('general.sound_reset')"
              @click="onResetSound"
            >
              <RotateCcw />
            </Button>
            <Button variant="outline" size="sm" class="h-7 text-xs" @click="onBrowseSound">
              <FolderOpen /> {{ t("general.sound_browse") }}
            </Button>
          </div>
        </div>

        <!-- Auto start -->
        <div class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100">
          <div class="flex flex-col gap-0.5">
            <span class="text-sm font-medium text-foreground">{{ t("general.auto_start") }}</span>
            <span class="text-[11px] text-muted-foreground leading-none">{{ t("general.auto_start_desc") }}</span>
          </div>
          <Switch v-model="config.auto_start" />
        </div>
      </div>
    </section>

    <!-- Test notification button -->
    <Button
      variant="outline"
      class="anim-item w-full"
      style="animation-delay:180ms"
      @click="emit('test-notification')"
    >
      {{ t("general.test_notification") }}
    </Button>
  </div>
</template>
