<script setup lang="ts">
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { HookConfig } from "../types";

const { t } = useI18n();
const config = defineModel<HookConfig>({ required: true });
const codexInstalled = ref(false);

onMounted(async () => {
  try {
    codexInstalled.value = await invoke<boolean>("get_codex_installed");
    // Codex가 설치되어 있으면 기본값으로 활성화
    if (codexInstalled.value && config.value) {
      config.value.codex_enabled = true;
    }
  } catch (_) { /* ignore */ }
});

const hooks = [
  // 권장 항목
  { key: 'stop', recommended: true },
  { key: 'permission_request', recommended: true },
  // 세션 생명주기
  { key: 'setup', recommended: false },
  { key: 'session_start', recommended: false },
  { key: 'session_end', recommended: false },
  // 사용자 입력
  { key: 'user_prompt_submit', recommended: false },
  // 도구 실행 흐름
  { key: 'pre_tool_use', recommended: false },
  { key: 'post_tool_use', recommended: false },
  { key: 'post_tool_use_failure', recommended: false },
  // 서브에이전트 생명주기
  { key: 'subagent_start', recommended: false },
  { key: 'subagent_stop', recommended: false },
  // Notification 훅
  { key: 'notification_permission', recommended: false },
  { key: 'notification_elicitation', recommended: false },
  { key: 'notification_idle', recommended: false },
  // 기타
  { key: 'pre_compact', recommended: false },
] as const;
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-3 overflow-y-auto">
    <p class="text-[13px] text-muted-foreground">{{ t('hooks.desc') }}</p>

    <!-- Claude Code Section -->
    <div class="flex flex-col gap-2.5">
      <h3 class="text-sm font-semibold text-section-claude">{{ t('hooks.claude_code_section') }}</h3>
      <div class="flex flex-col gap-3">
        <div
          v-for="hook in hooks"
          :key="hook.key"
          class="flex flex-col gap-2.5 bg-card border rounded-[10px] px-3.5 py-3"
        >
          <label class="flex items-start gap-2.5 cursor-pointer">
            <Checkbox
              v-model="(config as any)[hook.key + '_enabled']"
              variant="claude"
              class="mt-0.5"
            />
            <div class="flex flex-col gap-0.5">
              <span class="text-sm font-medium text-foreground">
                {{ t(`hooks.${hook.key}_name`) }}
                <Badge v-if="hook.recommended" variant="outline" class="ml-1.5 text-[10px] text-event-success border-event-success/25 bg-event-success/15">
                  {{ t('hooks.recommended') }}
                </Badge>
              </span>
              <span class="text-xs text-muted-foreground">{{ t(`hooks.${hook.key}_desc`) }}</span>
            </div>
          </label>
          <Input
            v-if="(config as any)[hook.key + '_enabled']"
            type="text"
            v-model="(config as any)[hook.key + '_message']"
            :placeholder="t(`hooks.${hook.key}_placeholder`)"
            class="text-sm"
          />
        </div>
      </div>
    </div>

    <!-- Codex Section -->
    <div class="flex flex-col gap-2.5">
      <h3 class="text-sm font-semibold text-section-codex">{{ t('hooks.codex_section') }}</h3>
      <div class="flex flex-col gap-2.5 bg-card border rounded-[10px] px-3.5 py-3">
        <label class="flex items-start gap-2.5 cursor-pointer">
          <Checkbox
            v-model="config.codex_enabled"
            variant="codex"
            class="mt-0.5"
          />
          <div class="flex flex-col gap-0.5">
            <span class="text-sm font-medium text-foreground">{{ t('hooks.codex_name') }}</span>
            <span class="text-xs text-muted-foreground">{{ t('hooks.codex_desc') }}</span>
          </div>
        </label>
        <p v-if="!codexInstalled" class="text-xs text-muted-foreground px-2.5 py-1.5 bg-muted rounded-md">
          {{ t('hooks.codex_not_installed') }}
        </p>
      </div>
    </div>

    <p class="text-xs text-muted-foreground px-3 py-2 bg-muted/50 border border-border rounded-md [&_code]:bg-secondary [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:rounded [&_code]:text-[11px] [&_code]:font-medium" v-html="t('hooks.notice', { code: '<code>/hooks</code>' })"></p>
  </div>
</template>
