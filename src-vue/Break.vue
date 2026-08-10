<script setup lang="ts">
/** Dark-only break reminder. Rust decides whether the window is an overlay or toast. */
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'

import AnimatedEye from '@/components/AnimatedEye.vue'
import CountdownRing from '@/components/CountdownRing.vue'
import { useTimer } from '@/composables/useTimer'
import { skipBreak, snoozeBreak } from '@/lib/ipc'

const { snapshot } = useTimer()
const isOverlay = computed(() => snapshot.value?.style !== 'toast')
const secondsLeft = computed(() => snapshot.value?.remainingSecs ?? 0)
const progress = computed(() => {
  const snap = snapshot.value
  return snap ? snap.remainingSecs / Math.max(1, snap.totalSecs) : 1
})

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
    for (const [freq, delay] of [[528, 0], [792, 0.14]] as const) {
      const oscillator = ctx.createOscillator()
      oscillator.type = 'sine'
      oscillator.frequency.value = freq
      oscillator.connect(gain)
      oscillator.start(now + delay)
      oscillator.stop(now + 1.8)
    }
    window.setTimeout(() => void ctx.close(), 2200)
  } catch { /* Silence is an acceptable fallback. */ }
}

watch(() => snapshot.value?.phase, (phase, previous) => {
  if (phase === 'break' && previous !== 'break' && snapshot.value?.chime) playChime()
})

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
  <div class="break" :class="isOverlay ? 'as-overlay' : 'as-toast'" role="dialog" aria-label="Time to rest your eyes">
    <template v-if="isOverlay">
      <header class="overlay-top"><span class="brand-dot" aria-hidden="true"></span><strong>IRYS</strong><span>Eye break</span></header>
      <main class="overlay-panel">
        <div class="eye-side">
          <CountdownRing :progress="progress" :size="330" :stroke="6">
            <div class="ring-inner"><AnimatedEye :size="164" resting /><p class="count tnum" aria-live="polite">{{ secondsLeft }}<span>s</span></p></div>
          </CountdownRing>
        </div>
        <section class="copy-side">
          <p class="eyebrow">TIME TO RESET</p>
          <h1>Look into the distance</h1>
          <p class="lede">Find something about 30 feet away. Let your focus soften, blink a few times, and give your eyes a short reset.</p>
          <div class="rule-guide" aria-label="30-30-30 rule"><div><b>30</b><span>minutes of focus</span></div><div><b>30</b><span>feet away</span></div><div><b>30</b><span>seconds to rest</span></div></div>
          <div class="actions"><button class="secondary" type="button" @click="snoozeBreak()">Snooze 5 minutes</button><button class="primary" type="button" @click="skipBreak()">Skip this break</button></div>
          <p class="hint">Press <kbd>Esc</kbd> to skip · This reminder is always dismissible</p>
        </section>
      </main>
    </template>

    <template v-else>
      <CountdownRing :progress="progress" :size="62" :stroke="7"><span class="count-sm tnum" aria-live="polite">{{ secondsLeft }}</span></CountdownRing>
      <div class="toast-copy"><strong>Rest your eyes</strong><span>Look 30 feet away for {{ secondsLeft }} seconds</span></div>
      <div class="toast-actions"><button class="secondary sm" type="button" @click="snoozeBreak()">+5m</button><button class="primary sm" type="button" @click="skipBreak()">Skip</button></div>
    </template>
  </div>
</template>

<style scoped>
.break { height:100%; color:#f4f8fb; }.as-overlay { min-height:100%; padding:70px 9vw 42px; background:radial-gradient(ellipse at 46% 38%,#183a59 0%,#102437 35%,#0e141d 76%); animation:fade-in 420ms var(--ease) both; }.overlay-top { position:absolute; top:22px; left:30px; display:flex; gap:9px; align-items:center; color:#d6e0ea; font-size:13px; }.overlay-top strong { letter-spacing:.06em; }.overlay-top span:last-child { padding-left:9px; color:#aebfcd; border-left:1px solid #a8c7e042; }.brand-dot { width:9px; height:9px; background:#72bfff; border-radius:50%; box-shadow:0 0 18px #72bfff99; }.overlay-panel { display:grid; grid-template-columns:.9fr 1.1fr; gap:clamp(36px,6vw,80px); align-items:center; width:min(1000px,100%); min-height:min(560px,calc(100vh - 112px)); margin:auto; padding:52px 64px; background:#17212cd9; border:1px solid #a8c7e01f; border-radius:8px; box-shadow:0 30px 90px #0009; backdrop-filter:blur(20px); }.eye-side { display:grid; place-items:center; }.eye-side :deep(.ring-wrap) { border-radius:50%; background:conic-gradient(#72bfff calc(v-bind(progress) * 1turn),#ffffff1c 0); box-shadow:0 0 65px #2d84c54d; }.eye-side :deep(.ring-wrap)::before { position:absolute; inset:15px; content:''; background:#101c29; border-radius:50%; }.eye-side :deep(.ring) { z-index:1; }.eye-side :deep(.inner) { z-index:2; }.ring-inner { display:grid; place-items:center; gap:8px; }.ring-inner :deep(.eye) { color:#72bfff; }.count { margin:0; font-size:27px; font-weight:600; }.count span { margin-left:2px; color:#b7c3cf; font-size:14px; font-weight:400; }.copy-side h1 { margin:0; font-size:clamp(32px,3.3vw,42px); letter-spacing:-.035em; }.eyebrow { margin:0 0 8px; color:#72bfff; font-size:11px; font-weight:650; letter-spacing:.1em; }.lede { max-width:440px; color:#b7c3cf; font-size:17px; line-height:1.55; }.rule-guide { display:grid; grid-template-columns:repeat(3,1fr); gap:8px; margin:29px 0; }.rule-guide div { padding:13px; background:#ffffff0a; border:1px solid #a8c7e020; border-radius:4px; }.rule-guide b { display:block; color:#72bfff; font-size:19px; }.rule-guide span { color:#bdc8d3; font-size:12px; }.actions { display:flex; gap:10px; }.primary,.secondary { padding:11px 18px; border:1px solid transparent; border-radius:4px; font-size:14px; font-weight:600; }.primary { color:#002b4b; background:#72bfff; }.primary:hover { background:#a5d8ff; }.secondary { color:#f4f8fb; background:#ffffff0d; border-color:#a8c7e04a; }.secondary:hover { background:#ffffff18; }.hint { margin:16px 0 0; color:#9eafbf; font-size:12px; }.hint kbd { padding:2px 5px; color:#fff; border:1px solid #a8c7e052; border-radius:3px; font:inherit; }
.as-toast { display:flex; gap:14px; align-items:center; height:100%; padding:16px 18px; background:#17212cf7; border:1px solid #a8c7e038; border-radius:8px; box-shadow:0 10px 34px #0009; animation:slide-in 320ms var(--ease) both; }.as-toast :deep(.value) { stroke:#72bfff; }.as-toast :deep(.track) { color:#dceeff; }.count-sm { color:#f4f8fb; font-size:17px; font-weight:600; }.toast-copy { display:grid; flex:1; gap:2px; min-width:0; }.toast-copy strong { font-size:14px; }.toast-copy span { color:#b7c3cf; font-size:12px; }.toast-actions { display:grid; gap:6px; }.sm { padding:5px 12px; font-size:12px; }
@keyframes fade-in { from { opacity:0; } } @keyframes slide-in { from { transform:translateX(16px); opacity:0; } }
@media (max-width:980px) { .overlay-panel { grid-template-columns:1fr; gap:26px; padding:40px; }.eye-side :deep(.ring-wrap) { transform:scale(.78); margin:-36px; }.copy-side { text-align:center; }.lede { margin-inline:auto; }.rule-guide { max-width:440px; margin-inline:auto; }.actions { justify-content:center; } }
</style>
