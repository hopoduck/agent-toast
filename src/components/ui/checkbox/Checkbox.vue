<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { Check } from "lucide-vue-next"
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<CheckboxRootProps & {
  class?: HTMLAttributes["class"]
  variant?: "default" | "claude" | "codex"
}>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, "class", "variant")

const forwarded = useForwardPropsEmits(delegatedProps, emits)

const variantClasses = {
  default: "data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground data-[state=checked]:border-primary",
  claude: "data-[state=checked]:bg-section-claude data-[state=checked]:text-white data-[state=checked]:border-section-claude",
  codex: "data-[state=checked]:bg-section-codex data-[state=checked]:text-background data-[state=checked]:border-section-codex",
}
</script>

<template>
  <CheckboxRoot
    v-slot="slotProps"
    data-slot="checkbox"
    v-bind="forwarded"
    :class="
      cn('peer border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive size-4 shrink-0 rounded-[4px] border shadow-xs transition-all duration-150 outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
         variantClasses[props.variant || 'default'],
         props.class)"
  >
    <CheckboxIndicator
      data-slot="checkbox-indicator"
      class="grid place-content-center text-current data-[state=checked]:animate-in data-[state=checked]:zoom-in-50 data-[state=unchecked]:animate-out data-[state=unchecked]:zoom-out-50 duration-150"
    >
      <slot v-bind="slotProps">
        <Check class="size-3.5" />
      </slot>
    </CheckboxIndicator>
  </CheckboxRoot>
</template>
