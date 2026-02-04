<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";

const props = defineProps<{
  tabs: { value: string; label: string }[];
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const tabRefs = ref<HTMLButtonElement[]>([]);
const indicatorStyle = ref({ left: "0px", width: "0px" });

function updateIndicator() {
  const idx = props.tabs.findIndex((t) => t.value === props.modelValue);
  const el = tabRefs.value[idx];
  if (el) {
    indicatorStyle.value = {
      left: `${el.offsetLeft}px`,
      width: `${el.offsetWidth}px`,
    };
  }
}

onMounted(() => nextTick(updateIndicator));
watch(() => props.modelValue, () => nextTick(updateIndicator));
watch(() => props.tabs, () => nextTick(updateIndicator), { deep: true });
</script>

<template>
  <div class="relative border-b border-border">
    <div class="flex gap-4">
      <button
        v-for="(tab, i) in tabs"
        :key="tab.value"
        :ref="(el) => { tabRefs[i] = el as HTMLButtonElement }"
        type="button"
        class="pb-2.5 text-sm font-medium transition-colors"
        :class="modelValue === tab.value ? 'text-foreground' : 'text-muted-foreground hover:text-foreground/70'"
        @click="emit('update:modelValue', tab.value)"
      >
        {{ tab.label }}
      </button>
    </div>
    <span
      class="absolute bottom-0 h-[3px] rounded-full bg-primary transition-all duration-250 ease-in-out"
      :style="indicatorStyle"
    />
  </div>
</template>
