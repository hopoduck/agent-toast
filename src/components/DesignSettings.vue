<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Layers, Sparkles } from "lucide-vue-next";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { HookConfig, NotificationData, ToastStyle } from "../types";
import ToastCard from "./ToastCard.vue";

const { t, locale } = useI18n();

const config = defineModel<HookConfig>({ required: true });

type PreviewEvent = "task_complete" | "user_input_required" | "error";
const previewEvent = ref<PreviewEvent>("task_complete");
const previewEvents: PreviewEvent[] = [
  "task_complete",
  "user_input_required",
  "error",
];

const previewData = computed<NotificationData>(() => ({
  id: "preview",
  window_title: t("design.preview_title"),
  event_display: previewEvent.value,
  message: t("design.preview_message"),
  source_hwnd: 0,
  process_tree: [],
  auto_dismiss_seconds: 0,
  source: "claude",
  hostname: null,
  show_hostname: false,
}));

const previewStyle = computed<ToastStyle>(() => ({
  bar: config.value.toast_bar,
  border: config.value.toast_border,
  effects: config.value.toast_effects,
  body: config.value.toast_body,
}));

// 설정이 바뀔 때마다 카드를 재마운트해 등장형 이펙트(펄스)도 다시 재생
const previewKey = computed(
  () => `${previewEvent.value}-${JSON.stringify(previewStyle.value)}`,
);

const barOptions = [
  { value: "left", labelKey: "design.bar_left" },
  { value: "none", labelKey: "design.bar_none" },
] as const;

const borderOptions = [
  { value: "subtle", labelKey: "design.border_subtle" },
  { value: "accent", labelKey: "design.border_accent" },
] as const;

const bodyOptions = [
  { value: "glow", labelKey: "design.body_glow" },
  { value: "tint", labelKey: "design.body_tint" },
  { value: "flat", labelKey: "design.body_flat" },
] as const;

const effectOptions = [
  {
    value: "ring",
    labelKey: "design.effect_ring",
    descKey: "design.effect_ring_desc",
  },
  {
    value: "breathe",
    labelKey: "design.effect_breathe",
    descKey: "design.effect_breathe_desc",
  },
  {
    value: "pulse",
    labelKey: "design.effect_pulse",
    descKey: "design.effect_pulse_desc",
  },
  {
    value: "shimmer",
    labelKey: "design.effect_shimmer",
    descKey: "design.effect_shimmer_desc",
  },
] as const;

function effectEnabled(name: string): boolean {
  return config.value.toast_effects.includes(name);
}

function setEffect(name: string, on: boolean) {
  const list = config.value.toast_effects.filter((e) => e !== name);
  if (on) list.push(name);
  config.value.toast_effects = list;
}
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">
    <!-- Preview: 스크롤과 무관하게 상단 고정 -->
    <section
      class="anim-item sticky top-0 z-20 flex flex-col gap-1.5 bg-background pb-1"
      style="animation-delay: 0ms"
    >
      <div class="flex items-center justify-between px-1">
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50"
          >{{ t("design.preview") }}</span
        >
        <div class="flex gap-1">
          <button
            v-for="ev in previewEvents"
            :key="ev"
            class="px-2 py-0.5 text-[11px] rounded-md border transition-colors"
            :class="
              previewEvent === ev
                ? 'bg-accent text-accent-foreground border-transparent'
                : 'text-muted-foreground border-border hover:bg-muted/40'
            "
            @click="previewEvent = ev"
          >
            {{ t(`event.${ev}`) }}
          </button>
        </div>
      </div>
      <div
        class="flex items-center justify-center rounded-[12px] border border-border bg-muted/30 py-5"
      >
        <!-- 실제 알림 창과 동일한 380x140 논리 픽셀 -->
        <div class="w-[380px] h-[140px] pointer-events-none">
          <ToastCard
            :key="previewKey"
            class="h-full"
            :notification="previewData"
            :toast-style="previewStyle"
            :show="true"
          />
        </div>
      </div>
    </section>

    <p
      class="anim-item text-[13px] text-muted-foreground"
      style="animation-delay: 20ms"
    >
      {{ t("design.desc") }}
    </p>

    <!-- Section: Layers (배타 축 3개) -->
    <section
      class="anim-item flex flex-col gap-1.5"
      style="animation-delay: 60ms"
    >
      <div class="flex items-center gap-1.5 px-1">
        <Layers :size="12" class="text-muted-foreground/50" />
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50"
          >{{ t("design.section_layers") }}</span
        >
      </div>
      <div
        class="rounded-[12px] border border-border overflow-hidden divide-y divide-border"
      >
        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("design.bar")
          }}</span>
          <Select :key="`bar-${locale}`" v-model="config.toast_bar">
            <SelectTrigger size="sm" class="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="o in barOptions"
                :key="o.value"
                :value="o.value"
              >
                {{ t(o.labelKey) }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("design.border")
          }}</span>
          <Select :key="`border-${locale}`" v-model="config.toast_border">
            <SelectTrigger size="sm" class="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="o in borderOptions"
                :key="o.value"
                :value="o.value"
              >
                {{ t(o.labelKey) }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("design.body")
          }}</span>
          <Select :key="`body-${locale}`" v-model="config.toast_body">
            <SelectTrigger size="sm" class="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="o in bodyOptions"
                :key="o.value"
                :value="o.value"
              >
                {{ t(o.labelKey) }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>

    <!-- Section: Effects (토글 집합, 중첩 가능) -->
    <section
      class="anim-item flex flex-col gap-1.5"
      style="animation-delay: 100ms"
    >
      <div class="flex items-center gap-1.5 px-1">
        <Sparkles :size="12" class="text-muted-foreground/50" />
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50"
          >{{ t("design.section_effects") }}</span
        >
      </div>
      <div
        class="rounded-[12px] border border-border overflow-hidden divide-y divide-border"
      >
        <div
          v-for="e in effectOptions"
          :key="e.value"
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <div class="flex flex-col gap-0.5 min-w-0">
            <span class="text-sm font-medium text-foreground">{{
              t(e.labelKey)
            }}</span>
            <span class="text-[11px] text-muted-foreground">{{
              t(e.descKey)
            }}</span>
          </div>
          <Switch
            class="shrink-0"
            :model-value="effectEnabled(e.value)"
            @update:model-value="(v: boolean) => setEffect(e.value, v)"
          />
        </div>
      </div>
    </section>
  </div>
</template>
