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
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { HookConfig, MonitorInfo } from "../types";

const { t, locale } = useI18n();

const config = defineModel<HookConfig>({ required: true });

const titleOptions = [
  { value: "project", labelKey: "general.title_project" },
  { value: "window", labelKey: "general.title_window" },
] as const;

const positionOptions = [
  { value: "top_left", labelKey: "general.pos_top_left", rotation: -45 },
  { value: "top_right", labelKey: "general.pos_top_right", rotation: 45 },
  { value: "bottom_left", labelKey: "general.pos_bottom_left", rotation: -135 },
  {
    value: "bottom_right",
    labelKey: "general.pos_bottom_right",
    rotation: 135,
  },
] as const;

const emit = defineEmits<{
  "test-notification": [];
}>();

const monitors = ref<MonitorInfo[]>([]);

onMounted(async () => {
  try {
    monitors.value = await invoke<MonitorInfo[]>("get_monitor_list");
  } catch (e) {
    console.error("Failed to get monitor list:", e);
  }
});
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-3 overflow-y-auto">
    <p class="text-[13px] text-muted-foreground">{{ t("general.desc") }}</p>

    <div class="flex flex-col gap-2">
      <!-- Language -->
      <div
        class="flex items-center justify-between bg-card border rounded-[10px] px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.language")
        }}</span>
        <Select v-model="config.locale">
          <SelectTrigger size="sm" class="w-[130px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ko">한국어</SelectItem>
            <SelectItem value="en">English</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- Title display -->
      <div
        class="flex items-center justify-between bg-card border rounded-[10px] px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.title_display")
        }}</span>
        <Select :key="`title-${locale}`" v-model="config.title_display_mode">
          <SelectTrigger size="sm" class="w-[130px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="o in titleOptions"
              :key="o.value"
              :value="o.value"
            >
              {{ t(o.labelKey) }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- Auto dismiss -->
      <div
        class="flex items-center justify-between bg-card border rounded-[10px] px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.auto_dismiss")
        }}</span>
        <div class="flex flex-col items-end gap-1">
          <NumberField
            v-model="config.auto_dismiss_seconds"
            :min="0"
            :max="300"
            :step="1"
            class="w-[100px]"
          >
            <NumberFieldContent>
              <NumberFieldDecrement class="p-2" />
              <NumberFieldInput class="h-7 text-xs" />
              <NumberFieldIncrement class="p-2" />
            </NumberFieldContent>
          </NumberField>
          <span class="text-[11px] text-muted-foreground">{{
            t("general.auto_dismiss_off")
          }}</span>
        </div>
      </div>

      <!-- Position -->
      <div
        class="flex items-center justify-between bg-card border rounded-[10px] px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.position")
        }}</span>
        <div class="grid grid-cols-2 gap-1">
          <Button
            v-for="pos in positionOptions"
            :key="pos.value"
            :variant="
              config.notification_position === pos.value ? 'default' : 'outline'
            "
            size="icon-sm"
            type="button"
            :title="t(pos.labelKey)"
            @click="config.notification_position = pos.value"
          >
            <span
              class="inline-block leading-none"
              :style="{ transform: `rotate(${pos.rotation}deg)` }"
              >↑</span
            >
          </Button>
        </div>
      </div>

      <!-- Monitor -->
      <div
        class="flex items-center justify-between bg-card border rounded-[10px] px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.monitor")
        }}</span>
        <Select :key="`monitor-${locale}`" v-model="config.notification_monitor">
          <SelectTrigger size="sm" class="w-[130px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="primary">{{
              t("general.monitor_primary")
            }}</SelectItem>
            <SelectItem v-for="(m, i) in monitors" :key="i" :value="String(i)">
              {{ m.name }}
              {{ m.is_primary ? t("general.monitor_primary_suffix") : "" }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- Auto close on focus -->
      <div
        class="flex items-center justify-between bg-card border rounded-lg px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.auto_close_focus")
        }}</span>
        <Switch v-model="config.auto_close_on_focus" />
      </div>

      <!-- Sound -->
      <div
        class="flex items-center justify-between bg-card border rounded-lg px-3.5 py-3"
      >
        <span class="text-sm font-medium text-foreground">{{
          t("general.sound")
        }}</span>
        <Switch v-model="config.notification_sound" />
      </div>
    </div>

    <Button variant="outline" class="w-full" @click="emit('test-notification')">
      {{ t("general.test_notification") }}
    </Button>
  </div>
</template>
