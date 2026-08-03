<script setup lang="ts">
/**
 * The break reminder, in both presentations.
 *
 * One component tree serves the fullscreen Overlay and the corner Toast; Rust
 * has already sized and positioned the window by the time this renders, and
 * tells us which style it chose via the snapshot.
 *
 * The window is never destroyed - Rust shows and hides it - so this component
 * stays mounted for the whole session and reacts to phase changes.
 */

import { computed, onBeforeUnmount, onMounted, watch } from 'vue'

import AnimatedEye from '@/components/AnimatedEye.vue'
import CountdownRing from '@/components/CountdownRing.vue'
import RuleGuide from '@/components/RuleGuide.vue'
import { useTimer } from '@/composables/useTimer'
import { skipBreak, snoozeBreak } from '@/lib/ipc'

const { snapshot } = useTimer()

const isOverlay = computed(() => snapshot.value?.style !== 'toast')
const secondsLeft = computed(() => snapshot.value?.remainingSecs ?? 0)

const progress = computed(() => {
  const snap = snapshot.value
  if (!snap) return 1
  return snap.remainingSecs / Math.max(1, snap.totalSecs)
})

/**
 * A short two-note sine chime, synthesised rather than shipped as an audio
 * file - nothing to load, and nothing to loosen in the CSP for.
 *
 * Webviews start an AudioContext suspended until there has been a user gesture,
 * so the very first chime of a session may be silent. Not worth surfacing: the
 * setting is off by default and the visual reminder is the real signal.
 */
function playChime() {
  try {
    const ctx = new AudioContext()
    void ctx.resume()

    const now = ctx.currentTime
    const gain = ctx.createGain()
    gain.gain.setValueAtTime(0, now)
    gain.gain.linearRampToValueAtTime(0.1, now + 0.05)
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 1.6)
    gain.connect(ctx.destination)

    // A fifth apart, arriving slightly staggered so it reads as a chime
    // rather than a beep.
    for (const [freq, delay] of [
      [528, 0],
      [792, 0.14],
    ] as const) {
      const osc = ctx.createOscillator()
      osc.type = 'sine'
      osc.frequency.value = freq
      osc.connect(gain)
      osc.start(now + delay)
      osc.stop(now + 1.8)
    }

    window.setTimeout(() => void ctx.close(), 2200)
  } catch {
    // No audio device, or audio blocked. Silence is an acceptable outcome.
  }
}

watch(
  () => snapshot.value?.phase,
  (phase, previous) => {
    if (phase === 'break' && previous !== 'break' && snapshot.value?.chime) {
      playChime()
    }
  },
)

/**
 * Escape always dismisses. The overlay is frameless, always-on-top and covers
 * the screen, so a guaranteed keyboard exit is not a nicety - it is the thing
 * that keeps it from behaving like malware. Never remove this.
 */
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    void skipBreak()
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div
    class="break"
    :class="isOverlay ? 'as-overlay' : 'as-toast'"
    role="dialog"
    aria-label="Time to rest your eyes"
  >
    <!-- Overlay: room to breathe, and the ring is the focal point. -->
    <template v-if="isOverlay">
      <CountdownRing :progress="progress" :size="300">
        <div class="ring-inner">
          <AnimatedEye :size="128" resting />
          <p class="count tnum" aria-live="polite">
            {{ secondsLeft }}<span class="unit">s</span>
          </p>
        </div>
      </CountdownRing>

      <h1>Look into the distance</h1>
      <p class="lede">
        About 10 metres away - out a window if you can. Let your eyes go soft and
        blink a few times.
      </p>

      <RuleGuide />

      <div class="actions">
        <button class="ghost" @click="snoozeBreak()">Snooze 5 min</button>
        <button class="primary" @click="skipBreak()">Skip · Esc</button>
      </div>
    </template>

    <!-- Toast: same information, none of the screen. -->
    <template v-else>
      <CountdownRing :progress="progress" :size="62" :stroke="8">
        <span class="count-sm tnum" aria-live="polite">{{ secondsLeft }}</span>
      </CountdownRing>

      <div class="toast-copy">
        <strong>Rest your eyes</strong>
        <span>Look ~10 m away for {{ secondsLeft }}s</span>
      </div>

      <div class="toast-actions">
        <button class="ghost sm" @click="snoozeBreak()">+5m</button>
        <button class="primary sm" @click="skipBreak()">Skip</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.break {
  height: 100%;
  color: #eef4f6;
}

/* --- overlay --- */

.as-overlay {
  display: flex;
  flex-direction: column;
  gap: 22px;
  align-items: center;
  justify-content: center;
  padding: 6vh 8vw;
  text-align: center;

  /* Deliberately dim in both light and dark themes. A white fullscreen flash
     would be the exact opposite of resting someone's eyes. */
  background:
    radial-gradient(
      120% 90% at 50% 38%,
      rgb(28 42 48 / 97%) 0%,
      rgb(11 15 19 / 98%) 70%
    ),
    rgb(11 15 19 / 98%);
  animation: fade-in 420ms var(--ease) both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
}

.ring-inner {
  display: grid;
  gap: 4px;
  place-items: center;
}

.count {
  margin: 0;
  font-size: 30px;
  font-weight: 600;
  line-height: 1;
}

.count .unit {
  margin-left: 2px;
  font-size: 15px;
  font-weight: 400;
  opacity: 0.6;
}

.as-overlay h1 {
  margin: 0;
  font-size: clamp(24px, 3.2vw, 40px);
  font-weight: 600;
  letter-spacing: -0.01em;
}

.lede {
  max-width: 46ch;
  margin: 0;
  font-size: 15px;
  opacity: 0.72;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 10px;
}

/* --- toast --- */

.as-toast {
  display: flex;
  gap: 14px;
  align-items: center;
  padding: 16px 18px;
  background: rgb(23 27 33 / 97%);
  border: 1px solid rgb(255 255 255 / 10%);
  border-radius: var(--radius-lg);
  box-shadow: 0 10px 34px rgb(0 0 0 / 45%);
  animation: slide-in 320ms var(--ease) both;
}

@keyframes slide-in {
  from {
    transform: translateX(16px);
    opacity: 0;
  }
}

.count-sm {
  font-size: 17px;
  font-weight: 600;
}

.toast-copy {
  display: grid;
  flex: 1;
  gap: 2px;
  min-width: 0;
}

.toast-copy strong {
  font-size: 14px;
  font-weight: 600;
}

.toast-copy span {
  font-size: 12px;
  opacity: 0.68;
}

.toast-actions {
  display: grid;
  gap: 6px;
}

/* --- buttons --- */

button {
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  background: none;
  border: 1px solid transparent;
  border-radius: 999px;
  transition:
    background-color 160ms var(--ease),
    border-color 160ms var(--ease),
    transform 160ms var(--ease);
}

button:active {
  transform: scale(0.97);
}

button.sm {
  padding: 5px 12px;
  font-size: 12px;
}

.primary {
  color: #06231f;
  background: var(--iris-glow);
}

.primary:hover {
  background: #9df0ec;
}

.ghost {
  color: inherit;
  border-color: rgb(255 255 255 / 22%);
}

.ghost:hover {
  background: rgb(255 255 255 / 8%);
}
</style>
