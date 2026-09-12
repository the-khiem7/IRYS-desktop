<script setup lang="ts">
/**
 * IRYS settings and status window.
 *
 * The view is deliberately desktop-first, but it remains a thin client: Rust
 * owns the timer and validates every preference returned by `set_settings`.
 * Appearance is the only browser-local preference, so it never expands that
 * IPC contract or affects the dark-only break window.
 */

import { computed, onMounted, onUnmounted, ref } from 'vue'

import AnimatedEye from '@/components/AnimatedEye.vue'
import { useSettings } from '@/composables/useSettings'
import { useTimer } from '@/composables/useTimer'
import { breakNow, quit, togglePause } from '@/lib/ipc'
import { fmtMmss, type BreakStyle } from '@/lib/types'

type Appearance = 'light' | 'dark'

const { snapshot } = useTimer()
const { settings, saving, error, update } = useSettings()
const appearance = ref<Appearance>('light')
const activeSection = ref('overview')
const localHour = ref(new Date().getHours())
const greeting = computed(() =>
  localHour.value < 12 ? 'Good morning' : localHour.value < 18 ? 'Good afternoon' : 'Good evening',
)
let greetingInterval: ReturnType<typeof setInterval> | undefined

function refreshGreeting() {
  localHour.value = new Date().getHours()
}

const paused = computed(() => snapshot.value?.phase === 'paused')

const statusLabel = computed(() => {
  const snap = snapshot.value
  if (!snap) return 'Starting…'
  if (snap.phase === 'paused') return 'Paused'
  if (snap.phase === 'break') return 'Rest your eyes'
  return 'Next eye break'
})

const statusValue = computed(() => {
  const snap = snapshot.value
  if (!snap || snap.phase === 'paused') return null
  return snap.phase === 'break' ? `${snap.remainingSecs}s` : fmtMmss(snap.remainingSecs)
})

const statusDescription = computed(() =>
  paused.value
    ? 'Your routine is paused. Resume whenever you are ready.'
    : 'Your routine is running quietly in the system tray.',
)

function num(event: Event, fallback: number): number {
  const parsed = Number.parseInt((event.target as HTMLInputElement).value, 10)
  return Number.isFinite(parsed) ? parsed : fallback
}

function setAppearance(next: Appearance) {
  appearance.value = next
  document.documentElement.dataset.theme = next
  localStorage.setItem('irys:appearance', next)
}

function toggleAppearance() {
  setAppearance(appearance.value === 'light' ? 'dark' : 'light')
}

function showSection(id: string) {
  activeSection.value = id
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

onMounted(() => {
  refreshGreeting()
  greetingInterval = setInterval(refreshGreeting, 60_000)
  window.addEventListener('focus', refreshGreeting)
  document.addEventListener('visibilitychange', refreshGreeting)
  const stored = localStorage.getItem('irys:appearance')
  const preferred: Appearance =
    stored === 'dark' || stored === 'light'
      ? stored
      : window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
  setAppearance(preferred)
})

onUnmounted(() => {
  clearInterval(greetingInterval)
  window.removeEventListener('focus', refreshGreeting)
  document.removeEventListener('visibilitychange', refreshGreeting)
})
</script>

<template>
  <div class="settings-shell">
    <header class="titlebar" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region>
        <AnimatedEye :size="24" />
        <span>IRYS</span>
      </div>
      <button
        class="appearance"
        type="button"
        :aria-label="appearance === 'dark' ? 'Switch to light appearance' : 'Switch to dark appearance'"
        @click="toggleAppearance"
      >
        <span aria-hidden="true">{{ appearance === 'dark' ? '☀' : '◐' }}</span>
        {{ appearance === 'dark' ? 'Light' : 'Dark' }}
      </button>
    </header>

    <div class="desktop-layout">
      <aside class="sidebar" aria-label="Settings navigation">
        <p class="sidebar-label">SETTINGS</p>
        <button
          v-for="item in [
            ['overview', '⌂', 'Overview'],
            ['timing', '◷', 'Timing'],
            ['presence', '◌', 'Focus & presence'],
            ['system', '⚙', 'System'],
          ]"
          :key="item[0]"
          type="button"
          :class="{ active: activeSection === item[0] }"
          :aria-current="activeSection === item[0] ? 'page' : undefined"
          @click="showSection(item[0])"
        >
          <span aria-hidden="true">{{ item[1] }}</span>{{ item[2] }}
        </button>

        <p class="sidebar-foot">IRYS 0.1.2<br />Your preferences stay on this device.</p>
      </aside>

      <main class="settings-content">
        <section id="overview" class="overview-section">
          <p class="breadcrumb">IRYS / Overview</p>
          <h1>{{ greeting }}</h1>
          <p class="intro">Your eye-break routine is ready when you are. Make it fit your workday without leaving your desktop.</p>

          <div class="overview-grid">
            <article class="next-card" :class="{ paused }">
              <span class="eyebrow">{{ statusLabel }}</span>
              <strong v-if="statusValue" class="status-time tnum">{{ statusValue }}</strong>
              <strong v-else-if="paused" class="status-time">Paused</strong>
              <strong v-else class="status-time">Preparing…</strong>
              <p>{{ statusDescription }}</p>
              <div class="actions">
                <button class="secondary" type="button" @click="togglePause()">
                  {{ paused ? 'Resume routine' : 'Pause routine' }}
                </button>
                <button class="primary" type="button" @click="breakNow()">Take a break now</button>
              </div>
            </article>

            <article class="rule-card">
              <div class="rule-card-heading">
                <div><span class="eyebrow">YOUR ROUTINE</span><h2>The 30–30–30 rule</h2></div>
                <span class="rule-chip">Active</span>
              </div>
              <p class="rule-summary">A short distance reset, built into your workday.</p>
              <div class="rule-steps" aria-label="30 minutes, 30 feet, 30 seconds">
                <div class="rule-step"><b>{{ settings ? Math.round(settings.workSecs / 60) : 30 }}</b><span>minutes</span><small>Focus before a break</small></div>
                <div class="rule-step"><b>30</b><span>feet away</span><small>Shift your gaze</small></div>
                <div class="rule-step"><b>{{ settings ? settings.breakSecs : 30 }}</b><span>seconds</span><small>Let your eyes rest</small></div>
              </div>
            </article>
          </div>
        </section>

        <template v-if="settings">
          <section id="timing" class="setting-section">
            <div class="section-heading"><div><h2>Timing</h2><p>A pace that fits your workday.</p></div></div>
            <article class="settings-card">
              <div class="setting-row"><div><h3>Work interval</h3><p>Time between breaks.</p></div><label class="number-field"><input type="number" min="1" max="240" :value="Math.round(settings.workSecs / 60)" @change="update({ workSecs: num($event, 30) * 60 })" /><span>minutes</span></label></div>
              <div class="setting-row"><div><h3>Break length</h3><p>How long to look away.</p></div><label class="number-field"><input type="number" min="5" max="600" :value="settings.breakSecs" @change="update({ breakSecs: num($event, 30) })" /><span>seconds</span></label></div>
              <div class="setting-row"><div><h3>Snooze length</h3><p>Extra time when you are not ready yet.</p></div><label class="number-field"><input type="number" min="1" max="60" :value="Math.round(settings.snoozeSecs / 60)" @change="update({ snoozeSecs: num($event, 5) * 60 })" /><span>minutes</span></label></div>
            </article>
          </section>

          <section id="presence" class="setting-section two-column-section">
            <div class="section-heading"><div><h2>Focus &amp; presence</h2><p>Keep IRYS helpful, never disruptive.</p></div></div>
            <div class="settings-columns">
              <article class="settings-card">
                <div class="card-title"><h3>Reminder</h3><p>How IRYS gets your attention.</p></div>
                <div class="setting-row"><div><h3>Reminder style</h3><p>Full screen or a small corner card.</p></div><div class="segmented" role="radiogroup" aria-label="Reminder style"><button v-for="option in (['overlay', 'toast'] as BreakStyle[])" :key="option" type="button" role="radio" :aria-checked="settings.style === option" :class="{ on: settings.style === option }" @click="update({ style: option })">{{ option === 'overlay' ? 'Full screen' : 'Corner' }}</button></div></div>
                <div class="setting-row"><div><h3>Heads-up notification</h3><p>A gentle nudge 30 seconds early.</p></div><input class="switch" type="checkbox" :checked="settings.prewarn" aria-label="Heads-up notification" @change="update({ prewarn: !settings.prewarn })" /></div>
                <div class="setting-row"><div><h3>Chime</h3><p>A soft tone when a break begins.</p></div><input class="switch" type="checkbox" :checked="settings.chime" aria-label="Chime" @change="update({ chime: !settings.chime })" /></div>
              </article>

              <article class="settings-card">
                <div class="card-title"><h3>Stay out of the way</h3><p>Let your screen take priority.</p></div>
                <div class="setting-row"><div><h3>Skip when I’m away</h3><p>Reset the interval if you have been idle.</p></div><input class="switch" type="checkbox" :checked="settings.skipWhenIdle" aria-label="Skip when I am away" @change="update({ skipWhenIdle: !settings.skipWhenIdle })" /></div>
                <div class="setting-row"><div><h3>Away after</h3><p>Idle time that counts as away.</p></div><label class="number-field"><input type="number" min="1" max="60" :value="Math.round(settings.idleThresholdSecs / 60) || 1" @change="update({ idleThresholdSecs: num($event, 1) * 60 })" /><span>minutes</span></label></div>
                <div class="setting-row"><div><h3>Wait for full-screen apps</h3><p>Defer breaks during calls, presentations and games.</p></div><input class="switch" type="checkbox" :checked="settings.deferOnFullscreen" aria-label="Wait for full-screen apps" @change="update({ deferOnFullscreen: !settings.deferOnFullscreen })" /></div>
              </article>
            </div>
          </section>

          <section id="system" class="setting-section">
            <div class="section-heading"><div><h2>System</h2><p>How IRYS fits into Windows.</p></div></div>
            <article class="settings-card">
              <div class="setting-row"><div><h3>Start with Windows</h3><p>Start your routine when you sign in. No administrator access is needed.</p></div><input class="switch" type="checkbox" :checked="settings.autostart" aria-label="Start with Windows" @change="update({ autostart: !settings.autostart })" /></div>
              <div class="setting-row"><div><h3>About IRYS</h3><p>Version 0.1.2 · Your settings are saved automatically.</p></div><button class="secondary" type="button" @click="quit()">Quit IRYS</button></div>
            </article>
          </section>
        </template>

        <p v-else class="loading">Loading settings…</p>
        <p v-if="error" class="error" role="alert">{{ error }}</p>
        <p class="save-status" :class="{ visible: saving }" aria-live="polite">Saving changes…</p>
      </main>
    </div>
  </div>
</template>

<style scoped>
.settings-shell { min-height:100%; background:var(--app-bg); }
.titlebar { height:48px; display:flex; align-items:center; padding:0 16px; border-bottom:1px solid var(--chrome-border); background:color-mix(in srgb,var(--titlebar) 88%,transparent); backdrop-filter:blur(24px); }
.brand { display:flex; gap:8px; align-items:center; font-size:14px; font-weight:650; letter-spacing:.04em; }.brand :deep(.eye) { color:var(--accent); }
.appearance { display:flex; gap:7px; align-items:center; margin-left:auto; padding:6px 10px; color:var(--text-dim); background:transparent; border:1px solid transparent; border-radius:4px; font-size:12px; }.appearance:hover { color:var(--text); background:var(--control-hover); border-color:var(--control-border); }
.desktop-layout { display:grid; grid-template-columns:220px minmax(0,1fr); min-height:calc(100vh - 48px); }.sidebar { display:flex; flex-direction:column; gap:3px; padding:22px 10px 16px; border-right:1px solid var(--chrome-border); background:var(--sidebar-bg); }.sidebar-label { margin:0 10px 7px; color:var(--text-faint); font-size:11px; font-weight:600; letter-spacing:.08em; }.sidebar button { display:flex; gap:11px; align-items:center; padding:9px 10px; color:var(--text-dim); text-align:left; background:transparent; border:1px solid transparent; border-radius:4px; font-size:14px; }.sidebar button span { width:16px; color:var(--text-faint); text-align:center; }.sidebar button:hover { color:var(--text); background:var(--control-hover); }.sidebar button.active { color:var(--text); background:var(--nav-active); font-weight:600; }.sidebar button.active span { color:var(--accent); }.sidebar-foot { margin:auto 10px 0; padding-top:16px; color:var(--text-faint); border-top:1px solid var(--chrome-border); font-size:11px; line-height:1.55; }
.settings-content { height:calc(100vh - 48px); padding:32px clamp(28px,4vw,62px) 44px; overflow:auto; scroll-behavior:smooth; }.overview-section,.setting-section { scroll-margin-top:24px; }.breadcrumb,.eyebrow { color:var(--accent); font-size:11px; font-weight:650; letter-spacing:.08em; }.breadcrumb { margin:0 0 5px; }.overview-section h1 { margin:0; font-size:30px; letter-spacing:-.035em; }.intro { max-width:670px; margin:7px 0 23px; color:var(--text-dim); }
.overview-grid,.settings-columns { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:16px; }.next-card,.rule-card,.settings-card { background:var(--surface); border:1px solid var(--card-border); border-radius:8px; box-shadow:var(--card-shadow); }.next-card { padding:22px; background:linear-gradient(125deg,var(--accent-wash),var(--surface) 68%); }.next-card.paused { background:linear-gradient(125deg,var(--paused-wash),var(--surface) 68%); }.status-time { display:block; margin-top:5px; font-size:34px; letter-spacing:-.05em; line-height:1; }.next-card p,.rule-card p { min-height:42px; margin:10px 0 18px; color:var(--text-dim); font-size:13px; }.actions { display:flex; gap:8px; flex-wrap:wrap; }.primary,.secondary { min-height:32px; padding:6px 12px; border:1px solid transparent; border-radius:4px; font-size:13px; font-weight:600; }.primary { color:white; background:var(--accent); }.primary:hover { background:var(--accent-hover); }.secondary { color:var(--text); background:var(--control-bg); border-color:var(--control-border); }.secondary:hover { background:var(--control-hover); }
.rule-card { overflow:hidden; padding:0; }.rule-card-heading { display:flex; align-items:flex-start; justify-content:space-between; gap:16px; padding:20px 22px 16px; background:linear-gradient(120deg,var(--accent-wash),var(--surface) 68%); border-bottom:1px solid var(--card-border); }.rule-card h2 { margin:4px 0 0; font-size:20px; letter-spacing:-.02em; }.rule-chip { flex:0 0 auto; padding:3px 8px; color:var(--accent); background:color-mix(in srgb,var(--accent) 10%,transparent); border:1px solid color-mix(in srgb,var(--accent) 20%,transparent); border-radius:999px; font-size:11px; font-weight:650; }.rule-card .rule-summary { min-height:0; margin:13px 22px 0; color:var(--text-dim); font-size:13px; }.rule-steps { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); margin:17px 22px 22px; overflow:hidden; background:var(--subtle-fill); border:1px solid var(--card-border); border-radius:6px; }.rule-step { display:grid; align-content:center; gap:1px; min-height:88px; padding:12px; }.rule-step + .rule-step { border-left:1px solid var(--card-border); }.rule-step b { color:var(--accent); font-size:25px; line-height:1; letter-spacing:-.04em; }.rule-step span { color:var(--text); font-size:12px; font-weight:600; }.rule-step small { margin-top:5px; color:var(--text-dim); font-size:10px; line-height:1.25; }
.setting-section { margin-top:34px; }.section-heading { display:flex; align-items:flex-end; justify-content:space-between; margin-bottom:10px; }.section-heading h2,.card-title h3 { margin:0; font-size:18px; letter-spacing:-.01em; }.section-heading p,.card-title p { margin:2px 0 0; color:var(--text-dim); font-size:13px; }.settings-card { padding:0 18px; }.card-title { padding:16px 0 8px; }.setting-row { display:grid; grid-template-columns:minmax(0,1fr) auto; gap:18px; align-items:center; min-height:73px; border-top:1px solid var(--card-border); }.settings-card .setting-row:first-of-type { border-top:0; }.card-title + .setting-row { border-top:0; }.setting-row h3 { margin:0; font-size:14px; font-weight:600; }.setting-row p { max-width:440px; margin:2px 0 0; color:var(--text-dim); font-size:12px; }.number-field { display:flex; align-items:center; gap:7px; min-width:122px; padding:5px 8px; background:var(--input-bg); border:1px solid var(--control-border); border-radius:4px; }.number-field:focus-within { border-color:var(--accent); outline:1px solid var(--accent); }.number-field input { width:42px; padding:0; color:var(--text); text-align:right; background:transparent; border:0; outline:0; font:inherit; font-variant-numeric:tabular-nums; }.number-field span { color:var(--text-dim); font-size:12px; }.segmented { display:flex; padding:2px; background:var(--subtle-fill); border:1px solid var(--control-border); border-radius:5px; }.segmented button { padding:5px 9px; color:var(--text-dim); background:transparent; border:0; border-radius:3px; font-size:12px; }.segmented button.on { color:var(--text); background:var(--surface); box-shadow:0 1px 2px rgb(0 0 0 / 12%); }.switch { position:relative; width:40px; height:20px; margin:0; appearance:none; background:var(--switch-off); border:0; border-radius:10px; transition:background-color 160ms var(--ease); }.switch::after { position:absolute; top:2px; left:2px; width:16px; height:16px; content:''; background:#fff; border-radius:50%; box-shadow:0 1px 2px rgb(0 0 0 / 25%); transition:transform 160ms var(--ease); }.switch:checked { background:var(--accent); }.switch:checked::after { transform:translateX(20px); }.loading,.error,.save-status { margin:16px 0 0; color:var(--text-dim); font-size:13px; }.error { color:#c42b1c; }.save-status { opacity:0; transition:opacity 160ms var(--ease); }.save-status.visible { opacity:1; }
@media (max-width:860px) { .desktop-layout { grid-template-columns:180px minmax(0,1fr); }.settings-content { padding:28px; }.overview-grid,.settings-columns { grid-template-columns:1fr; } }
@media (max-width:650px) { .desktop-layout { grid-template-columns:1fr; }.sidebar { display:none; }.settings-content { height:calc(100vh - 48px); padding:22px; } }
</style>
