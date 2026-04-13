<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { t } = useI18n();

const steps = [
  { num: "01", key: "step1" },
  { num: "02", key: "step2" },
  { num: "03", key: "step3" },
];

</script>

<template>
  <div class="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto">

    <!-- Steps -->
    <div class="flex flex-col gap-2">
      <div
        v-for="(step, i) in steps"
        :key="step.key"
        class="step-card group relative overflow-hidden rounded-xl border border-border bg-card px-4 py-3.5 transition-all duration-200 hover:border-primary/35 hover:shadow-sm"
        :class="`step-${i}`"
      >
        <!-- Ghost watermark number -->
        <span
          class="pointer-events-none absolute left-2 top-1 select-none font-black leading-none text-foreground/[0.06] transition-colors duration-200 group-hover:text-primary/12"
          style="font-size: 48px; font-family: 'Montserrat', sans-serif;"
        >{{ step.num }}</span>

        <!-- Content -->
        <div class="flex items-start gap-3">
          <div class="flex flex-col gap-0.5 min-w-0 pl-10">
            <span class="text-[13px] font-semibold text-foreground leading-snug">
              {{ t(`about.howto_${step.key}_title`) }}
            </span>
            <span class="text-[11.5px] text-muted-foreground leading-relaxed">
              {{ t(`about.howto_${step.key}_desc`) }}
            </span>
          </div>
        </div>
      </div>
    </div>


  </div>
</template>

<style scoped>
/* Staggered mount animation */
.step-card,
.feature-card {
  animation: fadeUp 0.3s ease both;
}

.step-0 { animation-delay: 0ms; }
.step-1 { animation-delay: 70ms; }
.step-2 { animation-delay: 140ms; }

@keyframes fadeUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

</style>
