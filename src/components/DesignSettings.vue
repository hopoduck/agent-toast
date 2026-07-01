<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { invoke } from "@tauri-apps/api/core";
import { Layers, Sparkles, Type } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { HookConfig, NotificationData, ToastStyle } from "../types";
import ToastCard from "./ToastCard.vue";

const { t, locale } = useI18n();

const config = defineModel<HookConfig>({ required: true });

const emit = defineEmits<{ "test-notification": [] }>();

type PreviewEvent = "task_complete" | "user_input_required" | "error";
const previewEvent = ref<PreviewEvent>("task_complete");
const previewEvents: PreviewEvent[] = [
  "task_complete",
  "user_input_required",
  "error",
];

const systemFonts = ref<string[]>([]);
onMounted(async () => {
  try {
    systemFonts.value = await invoke<string[]>("list_system_fonts");
  } catch {
    systemFonts.value = [];
  }
});

// shadcn(reka-ui) Select 는 빈 문자열 value 를 못 쓰므로 "기본"을 sentinel 로
// 표현하고 config 에는 "" 로 저장한다.
const FONT_DEFAULT = "__default__";
const fontSans = computed({
  get: () => config.value.toast_font_sans || FONT_DEFAULT,
  set: (v: string) => {
    config.value.toast_font_sans = v === FONT_DEFAULT ? "" : v;
  },
});
const fontMono = computed({
  get: () => config.value.toast_font_mono || FONT_DEFAULT,
  set: (v: string) => {
    config.value.toast_font_mono = v === FONT_DEFAULT ? "" : v;
  },
});

// 기본(번들) 폰트 — 빈 값일 때 실제로 적용되는 폰트명
const DEFAULT_SANS = "Pretendard";
const DEFAULT_MONO = "D2Coding";

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
  density: config.value.toast_density,
  font_sans: config.value.toast_font_sans,
  font_mono: config.value.toast_font_mono,
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

const densityOptions = [
  { value: "comfortable", labelKey: "design.density_comfortable" },
  { value: "compact", labelKey: "design.density_compact" },
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
        <div class="flex gap-1 items-center">
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
          <Button
            variant="secondary"
            size="sm"
            class="h-auto px-2 py-0.5 text-[11px] ml-1"
            @click="emit('test-notification')"
          >
            {{ t("design.test_notification") }}
          </Button>
        </div>
      </div>
      <div
        class="flex items-center justify-center rounded-[12px] border border-border bg-muted/30 py-5"
      >
        <!-- 실제 알림 창과 동일한 380 논리 픽셀 폭 (높이는 내용에 맞춤) -->
        <div class="w-[380px] pointer-events-none">
          <ToastCard
            :key="previewKey"
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
            t("design.density")
          }}</span>
          <Select :key="`density-${locale}`" v-model="config.toast_density">
            <SelectTrigger size="sm" class="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="o in densityOptions"
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

    <!-- Section: Fonts (시스템 폰트 선택, 빈 값=기본 번들) -->
    <section
      class="anim-item flex flex-col gap-1.5"
      style="animation-delay: 140ms"
    >
      <div class="flex items-center gap-1.5 px-1">
        <Type :size="12" class="text-muted-foreground/50" />
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50"
          >{{ t("design.section_font") }}</span
        >
      </div>
      <div
        class="rounded-[12px] border border-border overflow-hidden divide-y divide-border"
      >
        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("design.font_sans")
          }}</span>
          <Select :key="`font-sans-${locale}`" v-model="fontSans">
            <SelectTrigger
              size="sm"
              class="w-[180px]"
              :style="{
                fontFamily:
                  fontSans !== FONT_DEFAULT
                    ? `'${fontSans}'`
                    : `'${DEFAULT_SANS}'`,
              }"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent class="max-h-[280px]">
              <SelectItem
                :value="FONT_DEFAULT"
                :style="{ fontFamily: `'${DEFAULT_SANS}'` }"
              >
                {{ DEFAULT_SANS }} ({{ t("design.font_default") }})
              </SelectItem>
              <SelectItem
                v-for="f in systemFonts"
                :key="f"
                :value="f"
                :style="{ fontFamily: `'${f}'` }"
              >
                {{ f }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("design.font_mono")
          }}</span>
          <Select :key="`font-mono-${locale}`" v-model="fontMono">
            <SelectTrigger
              size="sm"
              class="w-[180px]"
              :style="{
                fontFamily:
                  fontMono !== FONT_DEFAULT
                    ? `'${fontMono}'`
                    : `'${DEFAULT_MONO}'`,
              }"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent class="max-h-[280px]">
              <SelectItem
                :value="FONT_DEFAULT"
                :style="{ fontFamily: `'${DEFAULT_MONO}'` }"
              >
                {{ DEFAULT_MONO }} ({{ t("design.font_default") }})
              </SelectItem>
              <SelectItem
                v-for="f in systemFonts"
                :key="f"
                :value="f"
                :style="{ fontFamily: `'${f}'` }"
              >
                {{ f }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>
  </div>
</template>
