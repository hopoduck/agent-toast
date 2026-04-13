<script setup lang="ts">
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { useI18n } from "vue-i18n";
import { Bell, Bot, MessageSquare, MoreHorizontal, RefreshCw, Star, Wrench } from "lucide-vue-next";
import claudeLogo from "../assets/claude.svg";
import openaiLogo from "../assets/openai.svg";
import type { HookConfig } from "../types";

const { t, locale } = useI18n();
const config = defineModel<HookConfig>({ required: true });

type HookEntry = { key: string; recommended?: boolean };
type HookGroup = { labelKo: string; labelEn: string; icon: unknown; hooks: HookEntry[] };

const hookGroups: HookGroup[] = [
  {
    labelKo: "권장",
    labelEn: "Recommended",
    icon: Star,
    hooks: [
      { key: "stop", recommended: true },
      { key: "permission_request", recommended: true },
    ],
  },
  {
    labelKo: "세션 생명주기",
    labelEn: "Session Lifecycle",
    icon: RefreshCw,
    hooks: [
      { key: "setup" },
      { key: "session_start" },
      { key: "session_end" },
    ],
  },
  {
    labelKo: "사용자 입력",
    labelEn: "User Input",
    icon: MessageSquare,
    hooks: [{ key: "user_prompt_submit" }],
  },
  {
    labelKo: "도구 실행",
    labelEn: "Tool Execution",
    icon: Wrench,
    hooks: [
      { key: "pre_tool_use" },
      { key: "post_tool_use" },
      { key: "post_tool_use_failure" },
    ],
  },
  {
    labelKo: "서브에이전트",
    labelEn: "Sub-agent",
    icon: Bot,
    hooks: [{ key: "subagent_start" }, { key: "subagent_stop" }],
  },
  {
    labelKo: "알림",
    labelEn: "Notification",
    icon: Bell,
    hooks: [
      { key: "notification_permission" },
      { key: "notification_elicitation" },
      { key: "notification_idle" },
    ],
  },
  {
    labelKo: "기타",
    labelEn: "Other",
    icon: MoreHorizontal,
    hooks: [{ key: "pre_compact" }],
  },
];

const groupLabel = (g: HookGroup) =>
  locale.value === "ko" ? g.labelKo : g.labelEn;

function isEnabled(key: string): boolean {
  return !!(config.value as any)[key + "_enabled"];
}
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">
    <p
      class="anim-item text-[13px] text-muted-foreground"
      style="animation-delay: 0ms"
    >
      {{ t("hooks.desc") }}
    </p>

    <!-- Claude Code Section -->
    <div class="anim-item flex flex-col gap-3" style="animation-delay: 20ms">
      <div class="flex items-center gap-1.5 px-1">
        <img :src="claudeLogo" class="size-3 object-contain opacity-70" alt="" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-section-claude">
          {{ t("hooks.claude_code_section") }}
        </span>
      </div>

      <div class="flex flex-col gap-2.5">
        <section
          v-for="(group, gi) in hookGroups"
          :key="gi"
          class="flex flex-col gap-1.5"
          :style="`animation-delay: ${40 + gi * 30}ms`"
        >
          <!-- Group header (same style as GeneralSettings sections) -->
          <div class="flex items-center gap-1.5 px-1">
            <component :is="group.icon" :size="12" class="text-muted-foreground/50" />
            <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">
              {{ groupLabel(group) }}
            </span>
          </div>

          <!-- Rows -->
          <div class="rounded-[12px] border border-border overflow-hidden divide-y divide-border">
            <div
              v-for="hook in group.hooks"
              :key="hook.key"
              class="bg-card flex flex-col"
            >
              <!-- Toggle row -->
              <label
                class="flex items-start gap-2.5 cursor-pointer px-3.5 py-2.5 hover:bg-muted/20 transition-colors duration-100"
              >
                <Switch
                  v-model="(config as any)[hook.key + '_enabled']"
                  class="mt-0.5 shrink-0"
                />
                <div class="flex flex-col gap-0.5 flex-1 min-w-0">
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-sm font-medium text-foreground">
                      {{ t(`hooks.${hook.key}_name`) }}
                    </span>
                    <Badge
                      v-if="hook.recommended"
                      variant="outline"
                      class="ml-0.5 text-[10px] text-event-success border-event-success/25 bg-event-success/15"
                    >
                      {{ t("hooks.recommended") }}
                    </Badge>
                  </div>
                  <span class="text-xs text-muted-foreground">
                    {{ t(`hooks.${hook.key}_desc`) }}
                  </span>
                </div>
              </label>

              <!-- Expandable message input -->
              <div
                class="grid transition-[grid-template-rows,opacity] duration-200 ease-out"
                :class="
                  isEnabled(hook.key)
                    ? 'grid-rows-[1fr] opacity-100'
                    : 'grid-rows-[0fr] opacity-0'
                "
              >
                <div class="overflow-hidden">
                  <div class="px-3.5 pt-1 pb-3">
                    <Input
                      v-model="(config as any)[hook.key + '_message']"
                      type="text"
                      :placeholder="t(`hooks.${hook.key}_placeholder`)"
                      class="text-sm"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

    <!-- Codex Section -->
    <div class="anim-item flex flex-col gap-1.5" style="animation-delay: 250ms">
      <div class="flex items-center gap-1.5 px-1">
        <img :src="openaiLogo" class="size-3 object-contain opacity-70" alt="" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-section-codex">
          {{ t("hooks.codex_section") }}
        </span>
      </div>
      <div class="rounded-[12px] border border-border overflow-hidden">
        <label class="flex items-start gap-2.5 cursor-pointer px-3.5 py-2.5 bg-card hover:bg-muted/20 transition-colors duration-100">
          <Switch v-model="config.codex_enabled" class="mt-0.5 shrink-0" />
          <div class="flex flex-col gap-0.5">
            <span class="text-sm font-medium text-foreground">{{ t("hooks.codex_name") }}</span>
            <span class="text-xs text-muted-foreground">{{ t("hooks.codex_desc") }}</span>
          </div>
        </label>
      </div>
    </div>

    <p
      class="anim-item text-xs text-muted-foreground px-3 py-2 bg-muted/50 border border-border rounded-[12px]"
      style="animation-delay: 280ms"
    >
      {{ t("hooks.notice") }}
    </p>
  </div>
</template>
