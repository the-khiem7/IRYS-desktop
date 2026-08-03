<script setup lang="ts">
/**
 * The rule itself, stated as three steps.
 *
 * Shown during the break as in-the-moment coaching (`compact` off) and in
 * settings as a reminder of what the app is for (`compact` on). The wording is
 * imperative during a break because that is when someone needs telling what to
 * do, not educating.
 */

withDefaults(defineProps<{ compact?: boolean }>(), { compact: false })

const STEPS = [
  { n: '30', unit: 'minutes', text: 'Look up from the screen.' },
  { n: '30', unit: 'feet', text: 'Find something far off — about 10 metres.' },
  { n: '30', unit: 'seconds', text: 'Hold your gaze there and let your eyes relax.' },
] as const
</script>

<template>
  <ol class="guide" :class="{ compact }">
    <li v-for="step in STEPS" :key="step.unit">
      <span class="num tnum">{{ step.n }}</span>
      <span class="unit">{{ step.unit }}</span>
      <span class="text">{{ step.text }}</span>
    </li>
  </ol>
</template>

<style scoped>
.guide {
  display: flex;
  gap: 28px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.guide li {
  display: grid;
  grid-template-columns: auto auto;
  grid-template-rows: auto auto;
  gap: 0 8px;
  align-items: baseline;
  max-width: 240px;
}

.num {
  font-size: 30px;
  font-weight: 600;
  line-height: 1;
  color: var(--iris);
}

.unit {
  font-size: 13px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  opacity: 0.75;
}

.text {
  grid-column: 1 / -1;
  margin-top: 6px;
  font-size: 13px;
  opacity: 0.75;
}

/* Settings has far less room, so the steps stack and the numerals shrink. */
.guide.compact {
  flex-direction: column;
  gap: 10px;
}

.guide.compact li {
  max-width: none;
  grid-template-columns: 2.2em auto;
  grid-template-rows: auto;
  align-items: center;
}

.guide.compact .num {
  font-size: 19px;
}

.guide.compact .unit {
  display: none;
}

.guide.compact .text {
  grid-column: 2;
  margin-top: 0;
}
</style>
