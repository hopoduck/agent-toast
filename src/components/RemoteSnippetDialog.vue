<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import SlidingTabs from "./SlidingTabs.vue";
import { Check, Copy, X } from "lucide-vue-next";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  bindAddr?: string;
}>();
const { t } = useI18n();

function deriveDefaultUrl(bindAddr?: string): string {
  const addr = (bindAddr ?? "").trim();
  if (!addr) return "";
  return /^https?:\/\//i.test(addr) ? addr : `http://${addr}`;
}

const url = ref("");
const hostname = ref("");

watch(
  () => open.value,
  (isOpen) => {
    if (isOpen) {
      url.value = deriveDefaultUrl(props.bindAddr);
    }
  },
  { immediate: true },
);

type OsTarget = "linux-x86_64" | "linux-aarch64" | "windows-x86_64";
const selectedOs = ref<OsTarget>("linux-x86_64");

const osTabs: { value: OsTarget; label: string }[] = [
  { value: "linux-x86_64", label: "Linux x86_64" },
  { value: "linux-aarch64", label: "Linux aarch64" },
  { value: "windows-x86_64", label: "Windows x86_64" },
];

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

const copiedKey = ref<string>("");
let copiedTimer: ReturnType<typeof setTimeout> | null = null;

async function copy(text: string, key: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copiedKey.value = key;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copiedKey.value = "";
    }, 1500);
  } catch {
    // ignore — clipboard API may be restricted in some contexts
  }
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay
        class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/80"
      />
      <DialogContent
        class="bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 w-full max-w-xl max-h-[90vh] overflow-y-auto translate-x-[-50%] translate-y-[-50%] rounded-lg border p-6 shadow-lg duration-200"
      >
        <div class="flex items-center justify-between mb-4">
          <DialogTitle class="text-lg font-semibold">
            {{ t("remote.snippetBtn") }}
          </DialogTitle>
          <DialogClose
            class="rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
          >
            <X :size="16" />
          </DialogClose>
        </div>

        <div class="space-y-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">{{
              t("remote.snippet.guideTitle")
            }}</label>
            <SlidingTabs
              :tabs="osTabs"
              :model-value="selectedOs"
              @update:model-value="(v) => (selectedOs = v as OsTarget)"
            />
            <div class="relative">
              <Button
                variant="outline"
                size="icon-sm"
                class="absolute top-1 right-1 size-6 text-muted-foreground hover:text-foreground"
                :title="
                  copiedKey === 'os-' + selectedOs
                    ? t('remote.snippet.copiedBtn')
                    : t('remote.snippet.copyBtn')
                "
                @click="copy(downloadSnippet, 'os-' + selectedOs)"
              >
                <Check
                  v-if="copiedKey === 'os-' + selectedOs"
                  class="size-3"
                />
                <Copy v-else class="size-3" />
              </Button>
              <pre
                class="text-xs bg-muted p-2 pr-10 rounded whitespace-pre-wrap break-all font-mono"
              >{{ downloadSnippet }}</pre>
            </div>
          </div>

          <div class="border-t" />

          <div class="space-y-1">
            <label class="text-sm font-medium">{{
              t("remote.snippet.urlLabel")
            }}</label>
            <Input v-model="url" />
            <p class="text-xs text-muted-foreground">
              {{ t("remote.snippet.urlHint") }}
            </p>
          </div>

          <div class="space-y-1">
            <label class="text-sm font-medium">{{
              t("remote.snippet.hostnameLabel")
            }}</label>
            <Input v-model="hostname" />
            <p class="text-xs text-muted-foreground">
              {{ t("remote.snippet.hostnameHint") }}
            </p>
          </div>

          <div v-if="installCmd || hookCmd" class="border-t" />

          <div v-if="installCmd" class="space-y-1">
            <label class="text-sm font-medium">{{
              t("remote.snippet.installHeader")
            }}</label>
            <div class="relative">
              <Button
                variant="outline"
                size="icon-sm"
                class="absolute top-1 right-1 size-6 text-muted-foreground hover:text-foreground"
                :title="
                  copiedKey === 'install'
                    ? t('remote.snippet.copiedBtn')
                    : t('remote.snippet.copyBtn')
                "
                @click="copy(installCmd, 'install')"
              >
                <Check v-if="copiedKey === 'install'" class="size-3" />
                <Copy v-else class="size-3" />
              </Button>
              <pre
                class="text-xs bg-muted p-2 pr-10 rounded whitespace-pre-wrap break-all font-mono"
              >{{ installCmd }}</pre>
            </div>
          </div>

          <div v-if="hookCmd" class="space-y-1">
            <label class="text-sm font-medium">{{
              t("remote.snippet.hookHeader")
            }}</label>
            <div class="relative">
              <Button
                variant="outline"
                size="icon-sm"
                class="absolute top-1 right-1 size-6 text-muted-foreground hover:text-foreground"
                :title="
                  copiedKey === 'hook'
                    ? t('remote.snippet.copiedBtn')
                    : t('remote.snippet.copyBtn')
                "
                @click="copy(hookCmd, 'hook')"
              >
                <Check v-if="copiedKey === 'hook'" class="size-3" />
                <Copy v-else class="size-3" />
              </Button>
              <pre
                class="text-xs bg-muted p-2 pr-10 rounded whitespace-pre-wrap break-all font-mono"
              >{{ hookCmd }}</pre>
            </div>
          </div>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
