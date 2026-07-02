<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Activity, BellOff, Eye, Globe } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { CounterSet, GlobalStats, Stats } from "../types";
import { Switch } from "./ui/switch";

const { t } = useI18n();
const stats = ref<Stats | null>(null);

const globalEnabled = ref(true);
const globalStats = ref<GlobalStats | null>(null);
const globalLoading = ref(false);
const globalError = ref(false);

async function loadGlobal() {
  if (!globalEnabled.value) return;
  globalLoading.value = true;
  globalError.value = false;
  try {
    globalStats.value = await invoke<GlobalStats>("get_global_stats");
  } catch {
    globalError.value = true;
  } finally {
    globalLoading.value = false;
  }
}

async function onToggleGlobal(v: boolean) {
  globalEnabled.value = v;
  try {
    await invoke("set_global_stats_enabled", { enabled: v });
  } catch {
    /* ignore */
  }
  if (v) await loadGlobal();
  else globalStats.value = null;
}

onMounted(async () => {
  try {
    stats.value = await invoke<Stats>("get_stats");
  } catch {
    /* ignore */
  }
  try {
    globalEnabled.value = await invoke<boolean>("get_global_stats_enabled");
  } catch {
    /* ignore */
  }
  await loadGlobal();
});

function emptyTotals(): CounterSet {
  return {
    shown: 0,
    activated: 0,
    closed_manual: 0,
    closed_timeout: 0,
    closed_focus: 0,
    skipped_focused: 0,
    skipped_ratelimit: 0,
  };
}

const totals = computed<CounterSet>(() => {
  const acc = emptyTotals();
  const s = stats.value;
  if (!s) return acc;
  for (const sources of Object.values(s.counts)) {
    for (const c of Object.values(sources)) {
      acc.shown += c.shown;
      acc.activated += c.activated;
      acc.closed_manual += c.closed_manual;
      acc.closed_timeout += c.closed_timeout;
      acc.closed_focus += c.closed_focus;
      acc.skipped_focused += c.skipped_focused;
      acc.skipped_ratelimit += c.skipped_ratelimit;
    }
  }
  return acc;
});

const hasData = computed<boolean>(() => {
  const c = totals.value;
  return (
    c.shown +
      c.activated +
      c.closed_manual +
      c.closed_timeout +
      c.closed_focus +
      c.skipped_focused +
      c.skipped_ratelimit >
    0
  );
});

const rate = computed<number>(() => {
  const { shown, activated } = totals.value;
  return shown > 0 ? Math.round((activated / shown) * 100) : 0;
});

const daysInUse = computed<number>(() => {
  const s = stats.value;
  if (!s?.since) return 0;
  const start = new Date(s.since).getTime();
  if (Number.isNaN(start)) return 0;
  return Math.floor((Date.now() - start) / 86_400_000) + 1;
});

const sinceLabel = computed<string>(() => {
  const s = stats.value;
  if (!s?.since) return "";
  const d = new Date(s.since);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString();
});

// Disposition: which terminal outcome dominates, as a percentage of all
// terminal events (a notification ends exactly one of these ways).
const disposition = computed<{ key: string; pct: number } | null>(() => {
  const c = totals.value;
  const base =
    c.activated + c.closed_manual + c.closed_timeout + c.closed_focus;
  if (base <= 0) return null;
  // Exclude "activated" — the view-rate insight already covers it; here we
  // surface how the *non-opened* notifications closed (avoids duplicating it).
  const top = [
    { reason: "timeout", n: c.closed_timeout },
    { reason: "manual", n: c.closed_manual },
    { reason: "focus", n: c.closed_focus },
  ].reduce((a, b) => (b.n > a.n ? b : a));
  if (top.n <= 0) return null;
  return {
    key: `setup.stat_insight_disp_${top.reason}`,
    pct: Math.round((top.n / base) * 100),
  };
});

// Interruptions the app prevented (skipped while focused or rate-limited).
const quiet = computed<number>(
  () => totals.value.skipped_focused + totals.value.skipped_ratelimit,
);

// Everything the app processed for you (delivered + silently skipped).
const handled = computed<number>(() => totals.value.shown + quiet.value);

// How notifications end, as a parts-of-whole breakdown.
const dispColor: Record<string, { bar: string; dot: string }> = {
  activated: { bar: "bg-event-success", dot: "text-event-success" },
  timeout: { bar: "bg-muted-foreground/45", dot: "text-muted-foreground" },
  manual: { bar: "bg-event-warning", dot: "text-event-warning" },
  focus: { bar: "bg-event-default", dot: "text-event-default" },
  skipped: { bar: "bg-primary", dot: "text-primary" },
};
const lifecycle = computed(() => {
  const c = totals.value;
  const counts: Array<[string, number]> = [
    ["activated", c.activated],
    ["timeout", c.closed_timeout],
    ["manual", c.closed_manual],
    ["focus", c.closed_focus],
    ["skipped", c.skipped_focused + c.skipped_ratelimit],
  ];
  const base = counts.reduce((s, [, n]) => s + n, 0);
  if (base <= 0) return null;
  return counts
    .filter(([, n]) => n > 0)
    .map(([key, n]) => ({
      key,
      n,
      pct: Math.round((n / base) * 100),
      bar: dispColor[key].bar,
      dot: dispColor[key].dot,
      label: t(`setup.stat_disp_${key}`),
    }));
});

const fmt = (n: number) => n.toLocaleString("en-US");

// Bars fill the full column width; fill is normalized to the section max
// (top row = full), so nothing looks half-empty. Min 6% keeps a sliver.
function eventBg(key: string): string {
  if (key === "task_complete") return "bg-event-success";
  if (key === "user_input_required") return "bg-event-warning";
  if (key === "error") return "bg-event-error";
  return "bg-event-default";
}

function bars(kind: "event" | "source") {
  const s = stats.value;
  if (!s) return [];
  const map = new Map<string, number>();
  for (const [event, sources] of Object.entries(s.counts)) {
    for (const [source, c] of Object.entries(sources)) {
      const key = kind === "event" ? event : source;
      map.set(key, (map.get(key) ?? 0) + c.shown);
    }
  }
  const rows = [...map.entries()]
    .map(([key, shown]) => ({ key, shown }))
    .filter((r) => r.shown > 0)
    .sort((a, b) => b.shown - a.shown);
  const max = Math.max(1, ...rows.map((r) => r.shown));
  return rows.map((r) => ({
    key: r.key,
    shown: r.shown,
    pct: Math.max(6, Math.round((r.shown / max) * 100)),
    color: kind === "event" ? eventBg(r.key) : "bg-primary",
  }));
}

const eventBars = computed(() => bars("event"));
const sourceBars = computed(() => bars("source"));
</script>

<template>
  <div class="overflow-y-auto p-1 font-mono">
    <div
      class="animate-in overflow-hidden rounded-xl border bg-card duration-300 fade-in"
    >
      <!-- terminal body -->
      <div class="p-[18px] text-[13px]">
        <div class="flex gap-1.5">
          <span class="text-primary">$</span>
          <span class="text-muted-foreground">agent-toast --stats</span>
        </div>

        <!-- empty state -->
        <template v-if="!hasData">
          <div class="mt-3 text-muted-foreground"># {{ t("setup.stat_empty") }}</div>
          <div class="text-muted-foreground/70"># {{ t("setup.stat_empty_hint") }}</div>
          <div class="mt-3">
            <span
              class="inline-block h-3.5 w-2 translate-y-0.5 animate-pulse bg-primary"
            />
          </div>
        </template>

        <template v-else>
          <!-- hero: total agent events the app handled for you -->
          <div class="my-4 flex flex-wrap items-end gap-x-4 gap-y-1">
            <div
              class="min-w-0 break-all text-[54px] font-bold leading-[0.85] tracking-[-0.03em] text-primary"
            >
              {{ fmt(handled) }}
            </div>
            <div
              class="pb-2 text-[11px] leading-relaxed tracking-wide text-muted-foreground"
            >
              {{ t("setup.stat_hero_label") }}<br />{{ t("setup.stat_hero_caption") }}
            </div>
          </div>

          <hr class="my-4 border-dashed border-border/65" />

          <!-- insights -->
          <div class="mb-2 text-[11px] tracking-[0.12em] text-muted-foreground/80">
            // {{ t("setup.stat_sec_insights") }}
          </div>
          <div class="flex flex-col gap-1.5">
            <div
              v-if="totals.shown > 0"
              class="flex items-start gap-2.5 leading-snug"
            >
              <Eye class="mt-0.5 size-3.5 shrink-0 text-event-success" />
              <span>{{
                t("setup.stat_insight_viewed", { shown: fmt(totals.shown), rate })
              }}</span>
            </div>
            <div
              v-if="totals.skipped_focused > 0"
              class="flex items-start gap-2.5 leading-snug"
            >
              <BellOff class="mt-0.5 size-3.5 shrink-0 text-primary" />
              <span>{{
                t("setup.stat_insight_skipped", { count: fmt(totals.skipped_focused) })
              }}</span>
            </div>
            <div v-if="disposition" class="flex items-start gap-2.5 leading-snug">
              <Activity class="mt-0.5 size-3.5 shrink-0 text-event-warning" />
              <span>{{ t(disposition.key, { pct: disposition.pct }) }}</span>
            </div>
          </div>

          <!-- lifecycle: how notifications end -->
          <template v-if="lifecycle">
            <hr class="my-4 border-dashed border-border/65" />
            <div
              class="mb-2 text-[11px] tracking-[0.12em] text-muted-foreground/80"
            >
              // {{ t("setup.stat_sec_breakdown") }}
            </div>
            <div class="flex h-[7px] overflow-hidden rounded-[2px]">
              <div
                v-for="seg in lifecycle"
                :key="seg.key"
                :class="seg.bar"
                :style="{ width: `${seg.pct}%` }"
              />
            </div>
            <div class="mt-2.5 flex flex-wrap gap-x-3.5 gap-y-1 text-[11px]">
              <span
                v-for="seg in lifecycle"
                :key="seg.key"
                class="inline-flex items-center gap-1"
              >
                <span :class="seg.dot">■</span>
                <span class="text-muted-foreground">{{ seg.label }}</span>
                <span class="font-mono tabular-nums">{{ fmt(seg.n) }}</span>
              </span>
            </div>
          </template>

          <!-- by event -->
          <template v-if="eventBars.length">
            <hr class="my-4 border-dashed border-border/65" />
            <div
              class="mb-2 text-[11px] tracking-[0.12em] text-muted-foreground/80"
            >
              // {{ t("setup.stat_sec_by_event") }}
            </div>
            <div
              v-for="row in eventBars"
              :key="row.key"
              class="grid grid-cols-[140px_1fr_3rem] items-center gap-3 py-1 text-xs"
            >
              <span class="truncate">{{ row.key }}</span>
              <div class="h-[7px] overflow-hidden rounded-[2px] bg-muted-foreground/15">
                <div
                  class="h-full rounded-[2px]"
                  :class="row.color"
                  :style="{ width: `${row.pct}%` }"
                />
              </div>
              <span class="text-right">{{ fmt(row.shown) }}</span>
            </div>
          </template>

          <!-- by source -->
          <template v-if="sourceBars.length">
            <div
              class="mb-2 mt-3 text-[11px] tracking-[0.12em] text-muted-foreground/80"
            >
              // {{ t("setup.stat_sec_by_source") }}
            </div>
            <div
              v-for="row in sourceBars"
              :key="row.key"
              class="grid grid-cols-[140px_1fr_3rem] items-center gap-3 py-1 text-xs"
            >
              <span class="truncate">{{ row.key }}</span>
              <div class="h-[7px] overflow-hidden rounded-[2px] bg-muted-foreground/15">
                <div
                  class="h-full rounded-[2px]"
                  :class="row.color"
                  :style="{ width: `${row.pct}%` }"
                />
              </div>
              <span class="text-right">{{ fmt(row.shown) }}</span>
            </div>
          </template>

          <hr class="my-4 border-dashed border-border/65" />
          <div class="text-[11px] text-muted-foreground">
            {{ t("setup.stat_footer", { date: sinceLabel, days: fmt(daysInUse) }) }}
            <span
              class="ml-0.5 inline-block h-3 w-1.5 translate-y-0.5 animate-pulse bg-primary"
            />
          </div>
        </template>

        <!-- global: anonymous worldwide aggregate -->
        <hr class="my-4 border-dashed border-border/65" />
        <div class="mb-2 text-[11px] tracking-[0.12em] text-muted-foreground/80">
          // {{ t("setup.stat_sec_global") }}
        </div>
        <div class="flex flex-col gap-1.5 text-[13px]">
          <div
            v-if="globalEnabled && globalLoading"
            class="h-4 w-2/3 animate-pulse rounded bg-muted-foreground/15 motion-reduce:animate-none"
          />
          <div
            v-else-if="globalEnabled && globalStats"
            class="flex items-start gap-2.5 leading-snug"
          >
            <Globe class="mt-0.5 size-3.5 shrink-0 text-event-default" />
            <span>{{
              t("setup.stat_global_insight", {
                shown: fmt(globalStats.totals.shown),
              })
            }}</span>
          </div>
          <div v-else-if="globalEnabled && globalError" class="text-muted-foreground">
            # {{ t("setup.stat_global_unavailable") }}
          </div>
          <div v-else-if="!globalEnabled" class="text-muted-foreground">
            # {{ t("setup.stat_global_off") }}
          </div>
        </div>
        <div class="mt-3 flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="text-xs">{{ t("setup.stat_global_share") }}</div>
            <div class="text-[11px] leading-snug text-muted-foreground">
              {{ t("setup.stat_global_share_desc") }}
            </div>
          </div>
          <Switch
            :model-value="globalEnabled"
            class="shrink-0"
            @update:model-value="onToggleGlobal"
          />
        </div>
      </div>
    </div>
  </div>
</template>
