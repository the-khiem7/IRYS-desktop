<script setup lang="ts">
/**
 * The face of the app: a hand-drawn SVG eye that blinks and looks away.
 *
 * Every part is inline geometry and CSS keyframes - no image, no sprite, no
 * icon dependency - so it scales cleanly from the 22px tray-adjacent size in
 * settings to filling a 4K overlay, and adds nothing to the CSP.
 *
 * The `resting` variant is the one shown during a break: the gaze drifts out to
 * a far point and the pupil dilates, which is what an eye actually does when it
 * stops focusing up close. It is a small piece of modelling-the-behaviour, and
 * it gives people something to follow with their own eyes.
 */

withDefaults(
  defineProps<{
    /** Pixel size of the (square) viewport. */
    size?: number
    /** Distance-gaze animation for breaks, vs. a calm idle blink. */
    resting?: boolean
  }>(),
  { size: 120, resting: false },
)
</script>

<template>
  <svg
    class="eye"
    :class="{ resting }"
    :width="size"
    :height="size"
    viewBox="0 0 120 120"
    role="img"
    aria-label="An eye, resting"
  >
    <defs>
      <radialGradient id="irisFill" cx="50%" cy="45%" r="60%">
        <stop offset="0%" :stop-color="'var(--iris-glow)'" />
        <stop offset="55%" :stop-color="'var(--iris)'" />
        <stop offset="100%" :stop-color="'var(--iris-deep)'" />
      </radialGradient>

      <!-- Keeps the iris from spilling past the lids as it drifts. -->
      <clipPath id="lidClip">
        <path d="M8 60c14-22 31-33 52-33s38 11 52 33c-14 22-31 33-52 33S22 82 8 60Z" />
      </clipPath>
    </defs>

    <!-- Blinking is applied to the whole eye rather than a moving lid shape, so
         it works on any background including the translucent overlay veil. -->
    <g class="blink">
      <g clip-path="url(#lidClip)">
        <path
          class="sclera"
          d="M8 60c14-22 31-33 52-33s38 11 52 33c-14 22-31 33-52 33S22 82 8 60Z"
        />
        <g class="gaze">
          <circle class="iris" cx="60" cy="60" r="19" fill="url(#irisFill)" />
          <circle class="pupil" cx="60" cy="60" r="8" />
          <!-- Specular highlight: the one detail that stops it reading as a logo. -->
          <circle class="glint" cx="53" cy="52" r="4" />
        </g>
      </g>

      <path
        class="lid-line"
        d="M8 60c14-22 31-33 52-33s38 11 52 33c-14 22-31 33-52 33S22 82 8 60Z"
      />
      <path class="lash" d="M60 20v-9M92 29l6-7M28 29l-6-7" />
    </g>
  </svg>
</template>

<style scoped>
.eye {
  display: block;
  overflow: visible;
}

.sclera {
  fill: color-mix(in srgb, var(--iris) 8%, white);
}

@media (prefers-color-scheme: dark) {
  .sclera {
    fill: color-mix(in srgb, var(--iris) 12%, #0d1116);
  }
}

.pupil {
  fill: #0c1418;
}

.glint {
  fill: #fff;
  opacity: 0.85;
}

.lid-line,
.lash {
  fill: none;
  stroke: currentColor;
  stroke-width: 3.5;
  stroke-linecap: round;
  opacity: 0.9;
}

.lash {
  stroke-width: 3;
  opacity: 0.45;
}

/* --- motion --- */

.blink {
  transform-origin: 60px 60px;
  animation: blink 7s var(--ease) infinite;
}

/* Squashing to a sliver reads as a blink without needing a lid shape that
   matches the background behind it. */
@keyframes blink {
  0%,
  92%,
  100% {
    transform: scaleY(1);
  }
  95% {
    transform: scaleY(0.06);
  }
}

.gaze {
  transform-origin: 60px 60px;
}

.pupil {
  transition: r 1.2s var(--ease);
}

/* Idle: small, unhurried glances. */
.eye:not(.resting) .gaze {
  animation: glance 9s var(--ease) infinite;
}

@keyframes glance {
  0%,
  100% {
    transform: translate(0, 0);
  }
  30% {
    transform: translate(7px, -2px);
  }
  60% {
    transform: translate(-6px, 1px);
  }
}

/* Resting: settle on a far point and hold there, pupil dilating as the
   focusing muscles let go. */
.eye.resting .gaze {
  animation: look-far 10s var(--ease) infinite;
}

.eye.resting .pupil {
  animation: dilate 10s var(--ease) infinite;
}

@keyframes look-far {
  0% {
    transform: translate(0, 0);
  }
  25%,
  70% {
    transform: translate(11px, -5px);
  }
  100% {
    transform: translate(0, 0);
  }
}

@keyframes dilate {
  0%,
  100% {
    r: 8px;
  }
  30%,
  70% {
    r: 12px;
  }
}

/* With reduced motion the eye simply stays open and still - theme.css
   collapses the durations, and this keeps the resting pupil from being stuck
   mid-animation. */
@media (prefers-reduced-motion: reduce) {
  .eye.resting .pupil {
    r: 11px;
  }
}
</style>
