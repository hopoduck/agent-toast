<script setup lang="ts">
import { ref, computed } from "vue";
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
import { X } from "lucide-vue-next";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();

const url = ref("");
const hostname = ref("");

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
  return `agent-toast-send --url ${u} --event task_complete --message "작업 완료"${hostPart}`;
});

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
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
        class="bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 w-full max-w-xl translate-x-[-50%] translate-y-[-50%] rounded-lg border p-6 shadow-lg duration-200"
      >
        <div class="flex items-center justify-between mb-4">
          <DialogTitle class="text-lg font-semibold">
            {{ t("remote.snippet.installHeader") }}
          </DialogTitle>
          <DialogClose
            class="rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
          >
            <X :size="16" />
          </DialogClose>
        </div>

        <div class="space-y-3">
          <div class="space-y-1">
            <label class="text-sm font-medium">{{ t("remote.snippet.urlLabel") }}</label>
            <Input v-model="url" placeholder="http://your-desktop:8787" />
            <p class="text-xs text-muted-foreground">
              {{ t("remote.snippet.urlHint") }}
            </p>
          </div>

          <div class="space-y-1">
            <label class="text-sm font-medium">{{ t("remote.snippet.hostnameLabel") }}</label>
            <Input v-model="hostname" placeholder="prod" />
          </div>

          <div v-if="installCmd" class="space-y-1">
            <div class="flex items-center justify-between">
              <label class="text-sm font-medium">{{ t("remote.snippet.installHeader") }}</label>
              <Button size="sm" variant="outline" @click="copy(installCmd)">
                {{ t("remote.snippet.copyBtn") }}
              </Button>
            </div>
            <pre
              class="text-xs bg-muted p-2 rounded whitespace-pre-wrap break-all"
            >{{ installCmd }}</pre>
          </div>

          <div v-if="hookCmd" class="space-y-1">
            <label class="text-sm font-medium">{{ t("remote.snippet.hookHeader") }}</label>
            <pre
              class="text-xs bg-muted p-2 rounded whitespace-pre-wrap break-all"
            >{{ hookCmd }}</pre>
          </div>

          <details class="text-xs">
            <summary class="cursor-pointer font-medium">
              {{ t("remote.snippet.guideTitle") }}
            </summary>
            <pre class="mt-2 bg-muted p-2 rounded whitespace-pre-wrap break-all"># Linux x86_64
curl -L https://github.com/hopoduck/agent-toast/releases/latest/download/agent-toast-send-linux-x86_64 \
  -o ~/.local/bin/agent-toast-send
chmod +x ~/.local/bin/agent-toast-send

# Linux aarch64 은 -linux-aarch64 로 교체</pre>
          </details>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
