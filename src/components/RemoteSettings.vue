<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { Input } from "@/components/ui/input";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Switch } from "@/components/ui/switch";
import SlidingTabs from "./SlidingTabs.vue";
import CodeBlock from "./CodeBlock.vue";
import { Radio } from "lucide-vue-next";
import type { HookConfig } from "../types";

const { t } = useI18n();
const config = defineModel<HookConfig>({ required: true });

type OsTarget = "linux-x86_64" | "linux-aarch64" | "windows-x86_64";
const selectedOs = ref<OsTarget>("linux-x86_64");
const osTabs: { value: OsTarget; label: string }[] = [
  { value: "linux-x86_64", label: "Linux x86_64" },
  { value: "linux-aarch64", label: "Linux aarch64" },
  { value: "windows-x86_64", label: "Windows x86_64" },
];

const url = ref("");
const hostname = ref("");
let cachedTailscaleHost: string | null | undefined = undefined;

function deriveDefaultUrl(port: number | undefined, host: string): string {
  if (!port) return "";
  return `http://${host}:${port}`;
}

async function resolveDefaultHost(): Promise<string> {
  if (cachedTailscaleHost === undefined) {
    try {
      cachedTailscaleHost = await invoke<string | null>(
        "get_tailscale_hostname",
      );
    } catch {
      cachedTailscaleHost = null;
    }
  }
  return cachedTailscaleHost ?? "0.0.0.0";
}

watch(
  () => [config.value.http_enabled, config.value.http_port] as const,
  async ([enabled, port]) => {
    if (!enabled) return;
    if (!url.value) {
      url.value = deriveDefaultUrl(port, "0.0.0.0");
      const host = await resolveDefaultHost();
      if (config.value.http_enabled) {
        url.value = deriveDefaultUrl(port, host);
      }
    }
  },
  { immediate: true },
);

function shellQuote(s: string): string {
  return `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

const installCmd = computed(() => {
  const u = url.value.trim();
  if (!u) return "";
  const h = hostname.value.trim();
  const hostPart = h ? ` --hostname ${shellQuote(h)}` : "";
  return `agent-toast-send init --url ${u}${hostPart}`;
});

const hookCmd = computed(() => {
  const u = url.value.trim();
  if (!u) return "";
  const h = hostname.value.trim();
  const hostPart = h ? ` --hostname ${shellQuote(h)}` : "";
  const msg = t("defaults.stop_message");
  return `agent-toast-send --url ${u} --event task_complete --message ${shellQuote(msg)}${hostPart}`;
});

const downloadSnippet = computed(() => {
  const baseUrl =
    "https://github.com/hopoduck/agent-toast/releases/latest/download";
  switch (selectedOs.value) {
    case "linux-x86_64":
      return [
        `curl -L ${baseUrl}/agent-toast-send-linux-x86_64 \\`,
        `  -o ~/.local/bin/agent-toast-send`,
        `chmod +x ~/.local/bin/agent-toast-send`,
      ].join("\n");
    case "linux-aarch64":
      return [
        `curl -L ${baseUrl}/agent-toast-send-linux-aarch64 \\`,
        `  -o ~/.local/bin/agent-toast-send`,
        `chmod +x ~/.local/bin/agent-toast-send`,
      ].join("\n");
    case "windows-x86_64":
      return [
        `# PowerShell`,
        `$dest = "$env:USERPROFILE\\.local\\bin"`,
        `New-Item -ItemType Directory -Force -Path $dest | Out-Null`,
        `Invoke-WebRequest -Uri ${baseUrl}/agent-toast-send-windows-x86_64.exe -OutFile "$dest\\agent-toast-send.exe"`,
      ].join("\n");
  }
  return "";
});
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">
    <!-- Section: Server -->
    <section
      class="anim-item flex flex-col gap-1.5"
      style="animation-delay: 0ms"
    >
      <div class="flex items-center gap-1.5 px-1">
        <Radio :size="12" class="text-muted-foreground/50" />
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50"
          >{{ t("remote.title") }}</span
        >
      </div>
      <div
        class="rounded-[12px] border border-border overflow-hidden divide-y divide-border"
      >
        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("remote.enabled")
          }}</span>
          <Switch v-model="config.http_enabled" />
        </div>
        <div
          v-if="config.http_enabled"
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground shrink-0">{{
            t("remote.port")
          }}</span>
          <NumberField
            v-model="config.http_port"
            :min="1"
            :max="65535"
            :format-options="{ useGrouping: false, maximumFractionDigits: 0 }"
            class="w-[120px]"
          >
            <NumberFieldContent>
              <NumberFieldDecrement />
              <NumberFieldInput class="h-7 text-xs" />
              <NumberFieldIncrement />
            </NumberFieldContent>
          </NumberField>
        </div>
        <div
          class="flex items-center justify-between bg-card px-3.5 py-2.5 gap-3 hover:bg-muted/20 transition-colors duration-100"
        >
          <span class="text-sm font-medium text-foreground">{{
            t("remote.showHostname")
          }}</span>
          <Switch v-model="config.show_hostname" />
        </div>
      </div>
    </section>

    <!-- Disabled state -->
    <div
      v-if="!config.http_enabled"
      class="anim-item relative overflow-hidden rounded-xl border border-dashed border-border bg-muted/20 px-4 py-5 text-center"
      style="animation-delay: 40ms"
    >
      <p class="text-[13px] leading-relaxed text-muted-foreground">
        {{ t("remote.disabled_hint") }}
      </p>
    </div>

    <!-- Guide steps (only when enabled) — matches HowtoSettings card pattern -->
    <template v-if="config.http_enabled">
      <!-- STEP 01: Download -->
      <div
        class="anim-item group relative shrink-0 overflow-hidden rounded-xl border border-border bg-card px-4 py-3.5 transition-all duration-200 hover:border-primary/35 hover:shadow-sm"
        style="animation-delay: 40ms"
      >
        <span
          class="pointer-events-none absolute left-2 top-1 select-none font-black leading-none text-foreground/[0.06] transition-colors duration-200 group-hover:text-primary/12"
          style="font-size: 48px; font-family: 'Montserrat', sans-serif"
          >01</span
        >
        <div class="flex flex-col gap-2 pl-10">
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-semibold leading-snug text-foreground"
              >{{ t("remote.step1_title") }}</span
            >
            <span
              class="text-[11.5px] leading-relaxed text-muted-foreground"
              >{{ t("remote.snippet.guideTitle") }}</span
            >
          </div>
          <SlidingTabs
            class="mt-1"
            :tabs="osTabs"
            :model-value="selectedOs"
            @update:model-value="(v) => (selectedOs = v as OsTarget)"
          />
          <CodeBlock :code="downloadSnippet" />
        </div>
      </div>

      <!-- STEP 02: Connection -->
      <div
        class="anim-item group relative shrink-0 overflow-hidden rounded-xl border border-border bg-card px-4 py-3.5 transition-all duration-200 hover:border-primary/35 hover:shadow-sm"
        style="animation-delay: 90ms"
      >
        <span
          class="pointer-events-none absolute left-2 top-1 select-none font-black leading-none text-foreground/[0.06] transition-colors duration-200 group-hover:text-primary/12"
          style="font-size: 48px; font-family: 'Montserrat', sans-serif"
          >02</span
        >
        <div class="flex flex-col gap-3 pl-10">
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-semibold leading-snug text-foreground"
              >{{ t("remote.step2_title") }}</span
            >
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-[12px] font-medium text-foreground">{{
              t("remote.snippet.urlLabel")
            }}</label>
            <Input v-model="url" class="h-9 font-mono text-[12.5px]" />
            <p
              class="text-[11px] leading-relaxed text-muted-foreground/85"
              >{{ t("remote.snippet.urlHint") }}</p
            >
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-[12px] font-medium text-foreground">{{
              t("remote.snippet.hostnameLabel")
            }}</label>
            <Input v-model="hostname" class="h-9 text-[12.5px]" />
            <p
              class="text-[11px] leading-relaxed text-muted-foreground/85"
              >{{ t("remote.snippet.hostnameHint") }}</p
            >
          </div>
        </div>
      </div>

      <!-- STEP 03: Use -->
      <div
        v-if="installCmd || hookCmd"
        class="anim-item group relative shrink-0 overflow-hidden rounded-xl border border-border bg-card px-4 py-3.5 transition-all duration-200 hover:border-primary/35 hover:shadow-sm"
        style="animation-delay: 140ms"
      >
        <span
          class="pointer-events-none absolute left-2 top-1 select-none font-black leading-none text-foreground/[0.06] transition-colors duration-200 group-hover:text-primary/12"
          style="font-size: 48px; font-family: 'Montserrat', sans-serif"
          >03</span
        >
        <div class="flex flex-col gap-3 pl-10">
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-semibold leading-snug text-foreground"
              >{{ t("remote.step3_title") }}</span
            >
          </div>
          <div v-if="installCmd" class="flex flex-col gap-1.5">
            <label
              class="text-[12px] font-medium text-foreground"
              >{{ t("remote.snippet.installHeader") }}</label
            >
            <CodeBlock :code="installCmd" />
          </div>
          <div v-if="hookCmd" class="flex flex-col gap-1.5">
            <label
              class="text-[12px] font-medium text-foreground"
              >{{ t("remote.snippet.hookHeader") }}</label
            >
            <CodeBlock :code="hookCmd" />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
