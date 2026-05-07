<script setup lang="ts">
import { ref } from "vue";
import { Check, Copy } from "lucide-vue-next";
import { Button } from "@/components/ui/button";

defineProps<{
  code: string;
}>();

const copied = ref(false);
let timer: ReturnType<typeof setTimeout> | null = null;

async function onCopy(text: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch {
    // clipboard may be restricted in some contexts
  }
}
</script>

<template>
  <div class="group/code relative">
    <Button
      variant="default"
      size="icon-sm"
      class="absolute right-1.5 top-1.5 size-6 opacity-30 transition-opacity group-hover/code:opacity-70"
      @click="onCopy(code)"
    >
      <Check v-if="copied" class="size-3.5" />
      <Copy v-else class="size-3.5" />
    </Button>
    <pre
      class="overflow-x-auto whitespace-pre rounded-lg border border-border bg-muted/40 px-3 py-2.5 pr-10 font-mono text-[12px] leading-[1.6] text-foreground/90"
    ><code>{{ code }}</code></pre>
  </div>
</template>
