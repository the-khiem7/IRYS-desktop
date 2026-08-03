/**
 * Typed wrappers over the Rust command surface.
 *
 * Every `invoke` in the app goes through this file, so no component contains a
 * raw command string and a renamed command is a type error rather than a
 * silent runtime failure.
 */

import { invoke } from '@tauri-apps/api/core'

import type { Settings, Snapshot } from './types'

export const getSnapshot = (): Promise<Snapshot> => invoke('get_snapshot')

export const getSettings = (): Promise<Settings> => invoke('get_settings')

/**
 * Returns what was actually stored, which can differ from what was sent -
 * the Rust core clamps out-of-range durations. Always re-render from the
 * result rather than assuming the request was applied verbatim.
 */
export const setSettings = (settings: Settings): Promise<Settings> =>
  invoke('set_settings', { settings })

export const togglePause = (): Promise<void> => invoke('toggle_pause')

export const breakNow = (): Promise<void> => invoke('break_now')

/** Dismiss the current break and restart the work interval. */
export const skipBreak = (): Promise<void> => invoke('skip_break')

/** Put the break off by the snooze interval instead of dismissing it. */
export const snoozeBreak = (): Promise<void> => invoke('snooze_break')

export const openSettings = (): Promise<void> => invoke('open_settings')

export const quit = (): Promise<void> => invoke('quit')
