/**
 * Settings, loaded from and written back to Rust.
 *
 * Rust owns validation: `set_settings` returns what was *actually* stored after
 * clamping, and we render from that. So typing an absurd interval corrects
 * itself in the UI rather than silently disagreeing with the real schedule.
 */

import { computed, ref } from 'vue'

import { getSettings, setSettings } from '@/lib/ipc'
import type { Settings } from '@/lib/types'

export function useSettings() {
  const settings = ref<Settings | null>(null)
  const saving = ref(false)
  const error = ref<string | null>(null)

  const loaded = computed(() => settings.value !== null)

  void getSettings()
    .then((loadedSettings) => {
      settings.value = loadedSettings
    })
    .catch((err: unknown) => {
      error.value = `Could not load settings: ${String(err)}`
    })

  /** Patch one or more fields and persist the result. */
  async function update(patch: Partial<Settings>) {
    if (!settings.value) return

    const next = { ...settings.value, ...patch }
    // Optimistic, so toggles feel instant; the response is authoritative.
    settings.value = next

    saving.value = true
    error.value = null
    try {
      settings.value = await setSettings(next)
    } catch (err: unknown) {
      error.value = `Could not save settings: ${String(err)}`
    } finally {
      saving.value = false
    }
  }

  return { settings, loaded, saving, error, update }
}
