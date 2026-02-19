<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { computed, onMounted, ref, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import logoPng from "../assets/logo.png";

const { t } = useI18n();
const version = ref("...");
const portable = ref(false);

// Update state
type UpdateStatus = "idle" | "checking" | "available" | "up-to-date" | "downloading" | "ready";
const updateStatus = ref<UpdateStatus>("idle");
const updateInfo = shallowRef<Update | null>(null);
const downloadProgress = ref(0);

const statusMessage = computed(() => {
  switch (updateStatus.value) {
    case "checking":
      return t("about.update_checking");
    case "available":
      return t("about.update_available", { version: updateInfo.value?.version ?? "" });
    case "up-to-date":
      return t("about.update_up_to_date");
    case "downloading":
      return t("about.update_downloading", { percent: downloadProgress.value });
    case "ready":
      return t("about.update_available", { version: updateInfo.value?.version ?? "" });
    default:
      return "";
  }
});

async function checkForUpdates() {
  if (portable.value) {
    toast.info(t("about.update_manual"));
    await openUrl("https://github.com/hopoduck/agent-toast/releases/latest");
    return;
  }

  updateStatus.value = "checking";

  try {
    const update = await check();
    if (update) {
      updateInfo.value = update;
      updateStatus.value = "available";
    } else {
      updateStatus.value = "up-to-date";
    }
  } catch {
    toast.info(t("about.update_manual"));
    await openUrl("https://github.com/hopoduck/agent-toast/releases/latest");
    updateStatus.value = "idle";
  }
}

async function downloadAndInstall() {
  if (!updateInfo.value) return;

  updateStatus.value = "downloading";
  downloadProgress.value = 0;

  let totalSize = 0;
  let downloadedSize = 0;

  try {
    await updateInfo.value.downloadAndInstall((event) => {
      if (event.event === "Started") {
        totalSize = event.data.contentLength ?? 0;
        downloadedSize = 0;
        downloadProgress.value = 0;
      } else if (event.event === "Progress") {
        downloadedSize += event.data.chunkLength;
        if (totalSize > 0) {
          downloadProgress.value = Math.round((downloadedSize / totalSize) * 100);
        }
      } else if (event.event === "Finished") {
        downloadProgress.value = 100;
      }
    });
    updateStatus.value = "ready";
  } catch {
    toast.info(t("about.update_manual"));
    await openUrl("https://github.com/hopoduck/agent-toast/releases/latest");
    updateStatus.value = "idle";
  }
}

async function restartApp() {
  await relaunch();
}

onMounted(async () => {
  version.value = await getVersion();
  portable.value = await invoke<boolean>("is_portable");
});
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-5 overflow-y-auto">
    <!-- Header with Logo -->
    <div class="flex items-center gap-4">
      <img :src="logoPng" width="56" height="56" alt="Agent Toast" class="rounded-xl shadow-sm" />
      <div class="flex flex-col gap-1">
        <h2 class="text-2xl font-bold text-foreground m-0">Agent Toast</h2>
        <div class="flex items-center gap-2">
          <span class="text-sm text-muted-foreground">v{{ version }}</span>
          <Separator orientation="vertical" class="h-3" />
          <a
            href="https://github.com/hopoduck/agent-toast"
            target="_blank"
            class="text-sm text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            GitHub
          </a>
        </div>
      </div>
    </div>

    <p class="text-sm text-muted-foreground leading-relaxed m-0">{{ t('about.description') }}</p>

    <!-- Update Section -->
    <div class="flex flex-col gap-2">
      <!-- Status message -->
      <span
        v-if="statusMessage"
        class="text-sm"
        :class="{
          'text-muted-foreground': updateStatus === 'checking' || updateStatus === 'up-to-date',
          'text-green-600 dark:text-green-400': updateStatus === 'available' || updateStatus === 'ready'
        }"
      >
        {{ statusMessage }}
      </span>

      <!-- Progress bar -->
      <Progress v-if="updateStatus === 'downloading'" :model-value="downloadProgress" class="h-1.5" />

      <!-- Buttons -->
      <div class="flex gap-2">
        <Button
          v-if="updateStatus === 'idle' || updateStatus === 'up-to-date'"
          variant="outline"
          size="sm"
          @click="checkForUpdates"
        >
          {{ t('about.update_check') }}
        </Button>

        <Button v-else-if="updateStatus === 'checking'" variant="outline" size="sm" disabled>
          <svg class="animate-spin h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ t('about.update_checking') }}
        </Button>

        <Button v-else-if="updateStatus === 'available'" size="sm" @click="downloadAndInstall">
          {{ t('about.update_download') }}
        </Button>

        <Button v-else-if="updateStatus === 'ready'" size="sm" @click="restartApp">
          {{ t('about.update_restart') }}
        </Button>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-auto pt-2">
      <p class="text-xs text-muted-foreground/50 m-0">{{ t('about.made_with') }}</p>
    </div>
  </div>
</template>
