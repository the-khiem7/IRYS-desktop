<script setup lang="ts">
/**
 * The settings window — also the app's status page.
 *
 * Hidden at startup and opened from the tray; closing it hides rather than
 * quits, so Irys keeps its schedule with nothing on screen.
 *
 * Durations are edited in whole minutes or seconds, whichever suits the field,
 * and converted at the boundary. Rust clamps whatever arrives, so the inputs
 * only need to be reasonable, not defensive.
 */

import { computed } from 'vue'

import AnimatedEye from '@/components/AnimatedEye.vue'
import RuleGuide from '@/components/RuleGuide.vue'
import SettingRow from '@/components/SettingRow.vue'
import { useSettings } from '@/composables/useSettings'
import { useTimer } from '@/composables/useTimer'
import { breakNow, quit, togglePause } from '@/lib/ipc'
import { fmtMmss, type BreakStyle } from '@/lib/types'

const { snapshot } = useTimer()
const { settings, saving, error, update } = useSettings()

const paused = computed(() => snapshot.value?.phase === 'paused')

const statusLabel = computed(() => {
  const snap = snapshot.value
  if (!snap) return 'Starting…'
  if (snap.phase === 'paused') return 'Paused'
  if (snap.phase === 'break') return 'Rest your eyes'
  return 'Next break in'
})

const statusValue = computed(() => {
  const snap = snapshot.value
  if (!snap || snap.phase === 'paused') return null
  return snap.phase === 'break' ? `${snap.remainingSecs}s` : fmtMmss(snap.remainingSecs)
})

/** Parse a number input without letting an empty field become NaN. */
function num(event: Event, fallback: number): number {
  const raw = (event.target as HTMLInputElement).value
  const parsed = Number.parseInt(raw, 10)
  return Number.isFinite(parsed) ? parsed : fallback
}
</script>

<template>
  <main class="app">
    <header class="hero">
      <AnimatedEye :size="56" />
      <div>
        <h1>Irys</h1>
        <p>Your companion for healthier eyes</p>
      </div>
    </header>

    <section class="status" :class="{ paused }">
      <div class="status-text">
        <span class="status-label">{{ statusLabel }}</span>
        <span v-if="statusValue" class="status-value tnum">{{ statusValue }}</span>
      </div>
      <div class="status-actions">
        <button class="ghost" @click="togglePause()">
          {{ paused ? 'Resume' : 'Pause' }}
        </button>
        <button class="primary" @click="breakNow()">Break now</button>
      </div>
    </section>

    <section class="card">
      <h2>The 30-30-30 rule</h2>
      <RuleGuide compact />
    </section>

    <template v-if="settings">
      <section class="card">
        <h2>Timing</h2>

        <SettingRow label="Work interval" hint="How long between breaks.">
          <label class="stepper">
            <input
              type="number"
              min="1"
              max="240"
              :value="Math.round(settings.workSecs / 60)"
              @change="update({ workSecs: num($event, 30) * 60 })"
            />
            <span>min</span>
          </label>
        </SettingRow>

        <SettingRow label="Break length" hint="The rule says 30 seconds.">
          <label class="stepper">
            <input
              type="number"
              min="5"
              max="600"
              :value="settings.breakSecs"
              @change="update({ breakSecs: num($event, 30) })"
            />
            <span>sec</span>
          </label>
        </SettingRow>

        <SettingRow label="Snooze" hint="Used by Snooze, and when a break is deferred.">
          <label class="stepper">
            <input
              type="number"
              min="1"
              max="60"
              :value="Math.round(settings.snoozeSecs / 60)"
              @change="update({ snoozeSecs: num($event, 5) * 60 })"
            />
            <span>min</span>
          </label>
        </SettingRow>
      </section>

      <section class="card">
        <h2>Reminder</h2>

        <SettingRow label="Style" hint="How the reminder shows up.">
          <div class="segmented" role="radiogroup" aria-label="Reminder style">
            <button
              v-for="option in (['overlay', 'toast'] as BreakStyle[])"
              :key="option"
              role="radio"
              :aria-checked="settings.style === option"
              :class="{ on: settings.style === option }"
              @click="update({ style: option })"
            >
              {{ option === 'overlay' ? 'Full screen' : 'Corner' }}
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Heads-up notification" hint="A nudge 30 seconds before.">
          <input
            type="checkbox"
            class="switch"
            :checked="settings.prewarn"
            @change="update({ prewarn: !settings.prewarn })"
          />
        </SettingRow>

        <SettingRow label="Chime" hint="A soft two-note tone when a break starts.">
          <input
            type="checkbox"
            class="switch"
            :checked="settings.chime"
            @change="update({ chime: !settings.chime })"
          />
        </SettingRow>
      </section>

      <section class="card">
        <h2>Stay out of the way</h2>

        <SettingRow
          label="Skip when I'm away"
          hint="If you haven't touched the keyboard, your eyes are already resting."
        >
          <input
            type="checkbox"
            class="switch"
            :checked="settings.skipWhenIdle"
            @change="update({ skipWhenIdle: !settings.skipWhenIdle })"
          />
        </SettingRow>

        <SettingRow label="Away after" hint="Idle time that counts as away.">
          <label class="stepper">
            <input
              type="number"
              min="1"
              max="60"
              :value="Math.round(settings.idleThresholdSecs / 60) || 1"
              @change="update({ idleThresholdSecs: num($event, 1) * 60 })"
            />
            <span>min</span>
          </label>
        </SettingRow>

        <SettingRow
          label="Wait for full-screen apps"
          hint="Defers the break during calls, presentations and games instead of covering them."
        >
          <input
            type="checkbox"
            class="switch"
            :checked="settings.deferOnFullscreen"
            @change="update({ deferOnFullscreen: !settings.deferOnFullscreen })"
          />
        </SettingRow>
      </section>

      <section class="card">
        <h2>System</h2>

        <SettingRow label="Start with Windows" hint="Adds a per-user login entry. No admin needed.">
          <input
            type="checkbox"
            class="switch"
            :checked="settings.autostart"
            @change="update({ autostart: !settings.autostart })"
          />
        </SettingRow>
      </section>
    </template>

    <p v-else class="loading">Loading settings…</p>

    <p v-if="error" class="error" role="alert">{{ error }}</p>

    <footer>
      <span class="saving" :class="{ show: saving }">Saving…</span>
      <button class="quiet" @click="quit()">Quit Irys</button>
    </footer>
  </main>
</template>

<style scoped>
.app {
  height: 100%;
  padding: 22px 22px 16px;
  overflow-y: auto;
}

.hero {
  display: flex;
  gap: 14px;
  align-items: center;
  margin-bottom: 18px;
}

.hero h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.hero p {
  margin: 0;
  font-size: 13px;
  color: var(--text-dim);
}

.status {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  margin-bottom: 16px;
  color: #eef4f6;
  background: linear-gradient(135deg, var(--iris-deep), #0f4a49);
  border-radius: var(--radius-lg);
}

.status.paused {
  background: linear-gradient(135deg, #4a5560, #333c45);
}

.status-text {
  display: grid;
  gap: 1px;
}

.status-label {
  font-size: 12px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.75;
}

.status-value {
  font-size: 24px;
  font-weight: 600;
  line-height: 1.1;
}

.status-actions {
  display: flex;
  gap: 8px;
}

.card {
  padding: 6px 18px 10px;
  margin-bottom: 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.card h2 {
  margin: 12px 0 4px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.06em;
  color: var(--text-dim);
  text-transform: uppercase;
}

/* --- controls --- */

button {
  padding: 7px 14px;
  font-size: 13px;
  font-weight: 500;
  background: none;
  border: 1px solid transparent;
  border-radius: 999px;
  transition: background-color 160ms var(--ease);
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
  border-color: rgb(255 255 255 / 28%);
}

.ghost:hover {
  background: rgb(255 255 255 / 12%);
}

.quiet {
  color: var(--text-dim);
  border-color: var(--border);
}

.quiet:hover {
  color: #c0392b;
  border-color: currentColor;
}

.stepper {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  padding: 5px 10px;
  background: var(--surface-sunken);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.stepper input {
  width: 3.4em;
  font: inherit;
  color: inherit;
  text-align: right;
  background: none;
  border: 0;
  font-variant-numeric: tabular-nums;
}

.stepper input:focus {
  outline: none;
}

.stepper span {
  font-size: 12px;
  color: var(--text-dim);
}

.segmented {
  display: inline-flex;
  padding: 2px;
  background: var(--surface-sunken);
  border: 1px solid var(--border);
  border-radius: 9px;
}

.segmented button {
  padding: 5px 12px;
  font-size: 12px;
  border-radius: 7px;
}

.segmented button.on {
  background: var(--surface);
  box-shadow: var(--shadow);
}

/* Checkbox restyled as a switch — same semantics, keyboard and screen readers
   still see a checkbox. */
.switch {
  position: relative;
  width: 40px;
  height: 23px;
  margin: 0;
  appearance: none;
  background: var(--border);
  border-radius: 999px;
  transition: background-color 180ms var(--ease);
}

.switch::after {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 17px;
  height: 17px;
  content: '';
  background: #fff;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgb(0 0 0 / 25%);
  transition: transform 180ms var(--ease);
}

.switch:checked {
  background: var(--iris);
}

.switch:checked::after {
  transform: translateX(17px);
}

/* --- footer --- */

.loading,
.error {
  margin: 8px 0;
  font-size: 13px;
  color: var(--text-dim);
}

.error {
  color: #c0392b;
}

footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 6px;
}

.saving {
  font-size: 12px;
  color: var(--text-dim);
  opacity: 0;
  transition: opacity 160ms var(--ease);
}

.saving.show {
  opacity: 1;
}
</style>
