<script setup lang="ts">
/**
 * Progress ring around the break countdown.
 *
 * Driven by `stroke-dashoffset` with a one-second linear transition, which
 * matches the Rust tick rate exactly - so the ring sweeps continuously instead
 * of stepping once per second.
 */

import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    /** Fraction still to go, 0-1. */
    progress: number
    size?: number
    stroke?: number
  }>(),
  { size: 220, stroke: 6 },
)

const RADIUS = 54
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

const offset = computed(() => {
  const clamped = Math.min(1, Math.max(0, props.progress))
  return CIRCUMFERENCE * (1 - clamped)
})
</script>

<template>
  <div class="ring-wrap" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg class="ring" viewBox="0 0 120 120" aria-hidden="true">
      <circle class="track" cx="60" cy="60" :r="RADIUS" :stroke-width="stroke" />
      <circle
        class="value"
        cx="60"
        cy="60"
        :r="RADIUS"
        :stroke-width="stroke"
        :stroke-dasharray="CIRCUMFERENCE"
        :stroke-dashoffset="offset"
      />
    </svg>
    <div class="inner">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.ring-wrap {
  position: relative;
  display: grid;
  place-items: center;
}

.ring {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  /* Start the sweep at 12 o'clock. */
  transform: rotate(-90deg);
}

.track,
.value {
  fill: none;
  stroke-linecap: round;
}

.track {
  stroke: currentColor;
  opacity: 0.14;
}

.value {
  stroke: var(--iris);
  transition: stroke-dashoffset 1s linear;
  filter: drop-shadow(0 0 6px color-mix(in srgb, var(--iris) 45%, transparent));
}

.inner {
  position: relative;
  display: grid;
  place-items: center;
  text-align: center;
}
</style>
