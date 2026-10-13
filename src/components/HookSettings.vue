<script setup lang="ts">
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { invoke } from "@tauri-apps/api/core";
import { Bot, ChevronRight, Layers, MessageSquareText, Puzzle, Star } from "lucide-vue-next";
import { onMounted, ref, type Component } from "vue";
import { useI18n } from "vue-i18n";
import claudeLogo from "../assets/claude.svg";
import openaiLogo from "../assets/openai.svg";
import type { AgentPresence, HookConfig } from "../types";

const { t } = useI18n();
const config = defineModel<HookConfig>({ required: true });

type AgentId = "claude" | "codex";
type Hook = { event: string; matcher?: string };
type HookRow = { key: string; claude?: Hook; codex?: Hook; recommended?: boolean };
type HookGroup = { id: string; icon: Component; collapsible: boolean; rows: HookRow[] };

// 백엔드 setup.rs 의 EVENT_SPECS 와 같은 매핑. 행 하나의 스위치·문구를 지원하는 에이전트가 함께 쓴다.
const hookGroups: HookGroup[] = [
  {
    id: "frequent",
    icon: Star,
    collapsible: false,
    rows: [
      { key: "stop", claude: { event: "Stop" }, codex: { event: "Stop" }, recommended: true },
      {
        key: "permission_request",
        claude: { event: "PermissionRequest" },
        codex: { event: "PermissionRequest" },
        recommended: true,
      },
      {
        key: "question",
        codex: { event: "PreToolUse", matcher: "^request_user_input(_async)?$" },
        recommended: true,
      },
    ],
  },
  {
    id: "common",
    icon: Layers,
    collapsible: true,
    rows: [
      { key: "session_start", claude: { event: "SessionStart" }, codex: { event: "SessionStart" } },
      { key: "session_end", claude: { event: "SessionEnd" }, codex: { event: "SessionEnd" } },
      {
        key: "user_prompt_submit",
        claude: { event: "UserPromptSubmit" },
        codex: { event: "UserPromptSubmit" },
      },
      { key: "pre_tool_use", claude: { event: "PreToolUse" }, codex: { event: "PreToolUse" } },
      { key: "post_tool_use", claude: { event: "PostToolUse" }, codex: { event: "PostToolUse" } },
      { key: "subagent_start", claude: { event: "SubagentStart" }, codex: { event: "SubagentStart" } },
      { key: "subagent_stop", claude: { event: "SubagentStop" }, codex: { event: "SubagentStop" } },
      { key: "pre_compact", claude: { event: "PreCompact" }, codex: { event: "PreCompact" } },
    ],
  },
  {
    id: "specific",
    icon: Puzzle,
    collapsible: true,
    rows: [
      { key: "post_tool_use_failure", claude: { event: "PostToolUseFailure" } },
      {
        key: "notification_permission",
        claude: { event: "Notification", matcher: "permission_prompt" },
      },
      {
        key: "notification_elicitation",
        claude: { event: "Notification", matcher: "elicitation_dialog" },
      },
      { key: "notification_idle", claude: { event: "Notification", matcher: "idle_prompt" } },
      { key: "setup", claude: { event: "Setup" } },
      { key: "post_compact", codex: { event: "PostCompact" } },
      { key: "interrupt", codex: { event: "Interrupt" } },
    ],
  },
];

const agents: { id: AgentId; name: string; logo: string; logoClass: string; nameClass: string }[] = [
  {
    id: "claude",
    name: "Claude Code",
    logo: claudeLogo,
    logoClass: "",
    nameClass: "text-section-claude",
  },
  {
    // openai.svg 는 흰색이라 라이트 테마에서만 반전
    id: "codex",
    name: "Codex",
    logo: openaiLogo,
    logoClass: "invert dark:invert-0",
    nameClass: "text-section-codex",
  },
];

const presence = ref<AgentPresence>({ claude: true, codex: true });

onMounted(async () => {
  try {
    presence.value = await invoke<AgentPresence>("get_agent_presence");
  } catch {
    /* 표시용 힌트일 뿐이라 실패해도 무시 */
  }
});

function agentEnabled(id: AgentId): boolean {
  return id === "claude" ? config.value.claude_enabled : config.value.codex_enabled;
}

function setAgentEnabled(id: AgentId, value: boolean) {
  if (id === "claude") config.value.claude_enabled = value;
  else config.value.codex_enabled = value;
}

function isEnabled(key: string): boolean {
  return !!(config.value as any)[key + "_enabled"];
}

/** 켜진 에이전트 중 하나라도 이 행을 지원하는지. 꺼진 행의 선택값은 그대로 보존된다. */
function isAvailable(row: HookRow): boolean {
  return (!!row.claude && config.value.claude_enabled) || (!!row.codex && config.value.codex_enabled);
}

const hookLabel = (h: Hook) => (h.matcher ? `${h.event} · ${h.matcher}` : h.event);

/** 두 에이전트가 같은 훅이면 칩 하나, 다르거나 한쪽 전용이면 에이전트 이름을 붙여 따로 */
function hookChips(row: HookRow): { agent?: AgentId; label: string }[] {
  if (row.claude && row.codex && hookLabel(row.claude) === hookLabel(row.codex)) {
    return [{ label: hookLabel(row.claude) }];
  }
  const chips: { agent?: AgentId; label: string }[] = [];
  if (row.claude) chips.push({ agent: "claude", label: hookLabel(row.claude) });
  if (row.codex) chips.push({ agent: "codex", label: hookLabel(row.codex) });
  return chips;
}

// 접힌 그룹 안에 켜 둔 알림이 있으면 처음부터 펼쳐 둔다 (최초 1회만 계산)
const openGroups = ref<Record<string, boolean>>(
  Object.fromEntries(
    hookGroups.map((g) => [g.id, !g.collapsible || g.rows.some((r) => isEnabled(r.key))]),
  ),
);

function toggleGroup(id: string) {
  openGroups.value[id] = !openGroups.value[id];
}
</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">
    <p class="anim-item text-[13px] text-muted-foreground" style="animation-delay: 0ms">
      {{ t("hooks.desc") }}
    </p>

    <!-- Agents -->
    <section class="anim-item flex flex-col gap-1.5" style="animation-delay: 20ms">
      <div class="flex items-center gap-1.5 px-1">
        <Bot :size="12" class="text-muted-foreground/50" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">
          {{ t("hooks.agents_title") }}
        </span>
      </div>
      <div class="grid grid-cols-2 gap-2">
        <label
          v-for="agent in agents"
          :key="agent.id"
          class="flex items-center gap-2.5 rounded-[12px] border border-border bg-card px-3.5 py-2.5 cursor-pointer hover:bg-muted/20 transition-colors duration-100"
        >
          <img :src="agent.logo" class="size-4 shrink-0 object-contain" :class="agent.logoClass" alt="" />
          <div class="flex flex-col min-w-0 flex-1">
            <span class="text-sm font-medium truncate" :class="agent.nameClass">{{ agent.name }}</span>
            <span v-if="!presence[agent.id]" class="text-[11px] leading-tight text-muted-foreground">
              {{ t("hooks.agent_not_installed") }}
            </span>
          </div>
          <Switch
            :model-value="agentEnabled(agent.id)"
            class="shrink-0"
            @update:model-value="(v: boolean) => setAgentEnabled(agent.id, v)"
          />
        </label>
      </div>
    </section>

    <!-- Dynamic message -->
    <section class="anim-item flex flex-col gap-1.5" style="animation-delay: 40ms">
      <div class="flex items-center gap-1.5 px-1">
        <MessageSquareText :size="12" class="text-muted-foreground/50" />
        <span class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50">
          {{ t("hooks.message_title") }}
        </span>
      </div>
      <label
        class="flex items-center justify-between gap-3 rounded-[12px] border border-border bg-card px-3.5 py-2.5 cursor-pointer hover:bg-muted/20 transition-colors duration-100"
      >
        <div class="flex flex-col gap-0.5 min-w-0 pr-2">
          <span class="text-sm font-medium text-foreground">{{ t("hooks.dynamic_message") }}</span>
          <span class="text-[11px] text-muted-foreground leading-tight">{{ t("hooks.dynamic_message_desc") }}</span>
        </div>
        <Switch v-model="config.dynamic_message_enabled" class="shrink-0" />
      </label>
    </section>

    <!-- Event groups -->
    <section
      v-for="(group, gi) in hookGroups"
      :key="group.id"
      class="anim-item flex flex-col gap-1.5"
      :style="`animation-delay: ${60 + gi * 30}ms`"
    >
      <component
        :is="group.collapsible ? 'button' : 'div'"
        :type="group.collapsible ? 'button' : undefined"
        class="flex items-center gap-1.5 px-1 text-left"
        :class="group.collapsible ? 'cursor-pointer group/head' : ''"
        :aria-expanded="group.collapsible ? openGroups[group.id] : undefined"
        @click="group.collapsible && toggleGroup(group.id)"
      >
        <component :is="group.icon" :size="12" class="text-muted-foreground/50" />
        <span
          class="text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground/50 group-hover/head:text-muted-foreground transition-colors"
        >
          {{ t(`hooks.group_${group.id}`) }}
        </span>
        <ChevronRight
          v-if="group.collapsible"
          :size="12"
          class="text-muted-foreground/50 transition-transform duration-200 motion-reduce:transition-none"
          :class="openGroups[group.id] ? 'rotate-90' : ''"
        />
      </component>

      <div
        class="grid transition-[grid-template-rows,opacity] duration-200 ease-out motion-reduce:transition-none"
        :class="openGroups[group.id] ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'"
      >
        <div class="overflow-hidden" :inert="!openGroups[group.id]">
          <div class="rounded-[12px] border border-border overflow-hidden divide-y divide-border">
            <div
              v-for="row in group.rows"
              :key="row.key"
              class="bg-card flex flex-col"
            >
              <!-- Toggle row -->
              <label
                class="flex items-start gap-2.5 px-3.5 py-2.5 transition-colors duration-100"
                :class="isAvailable(row) ? 'cursor-pointer hover:bg-muted/20' : 'cursor-default'"
              >
                <Switch
                  v-model="(config as any)[row.key + '_enabled']"
                  :disabled="!isAvailable(row)"
                  class="mt-0.5 shrink-0"
                />
                <!-- Switch 는 disabled 스타일로 스스로 흐려지니 텍스트만 흐린다 -->
                <div
                  class="flex flex-col gap-1 flex-1 min-w-0 transition-opacity duration-150"
                  :class="isAvailable(row) ? '' : 'opacity-50'"
                >
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-sm font-medium text-foreground">
                      {{ t(`hooks.${row.key}_name`) }}
                    </span>
                    <span
                      v-for="chip in hookChips(row)"
                      :key="(chip.agent ?? '') + chip.label"
                      class="max-w-full rounded-[4px] border border-border bg-muted/50 px-1.5 py-px font-mono text-[10px] text-muted-foreground [overflow-wrap:anywhere]"
                    >
                      <b
                        v-if="chip.agent"
                        class="mr-1 font-sans font-semibold"
                        :class="chip.agent === 'claude' ? 'text-section-claude' : 'text-section-codex'"
                        >{{ chip.agent === "claude" ? "Claude" : "Codex" }}</b
                      >{{ chip.label }}
                    </span>
                    <Badge
                      v-if="row.recommended"
                      variant="outline"
                      class="text-[10px] text-event-success border-event-success/25 bg-event-success/15"
                    >
                      {{ t("hooks.recommended") }}
                    </Badge>
                  </div>
                  <span class="text-xs text-muted-foreground">
                    {{ t(`hooks.${row.key}_desc`) }}
                  </span>
                </div>
              </label>

              <!-- Expandable message input -->
              <div
                class="grid transition-[grid-template-rows,opacity] duration-200 ease-out motion-reduce:transition-none"
                :class="
                  isEnabled(row.key) && isAvailable(row)
                    ? 'grid-rows-[1fr] opacity-100'
                    : 'grid-rows-[0fr] opacity-0'
                "
              >
                <div class="overflow-hidden">
                  <div class="flex flex-col gap-1 px-3.5 pt-1 pb-3">
                    <span class="text-[11px] text-muted-foreground">
                      {{ config.dynamic_message_enabled ? t("hooks.message_fallback_label") : t("hooks.message_label") }}
                    </span>
                    <Input
                      v-model="(config as any)[row.key + '_message']"
                      type="text"
                      :placeholder="t(`hooks.${row.key}_placeholder`)"
                      class="text-sm"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <p
      class="anim-item text-xs text-muted-foreground px-3 py-2 bg-muted/50 border border-border rounded-[12px]"
      style="animation-delay: 160ms"
    >
      {{ t("hooks.notice") }}
    </p>
  </div>
</template>
