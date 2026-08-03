/**
 * Mirrors the serde types in `src-tauri/src/core/mod.rs`.
 *
 * Rust is the single source of truth for all of these. If you change a field
 * here, change it there too — `Settings` is serialised with
 * `#[serde(rename_all = "camelCase")]`, so the names must match exactly.
 */

export type BreakStyle = 'overlay' | 'toast'

export type PhaseKind = 'work' | 'break' | 'paused'

/** What the UI renders from. Pushed on every tick of the Rust clock. */
export interface Snapshot {
  phase: PhaseKind
  remainingSecs: number
  /** Length of the current phase, for the countdown ring. Never zero. */
  totalSecs: number
  style: BreakStyle
  chime: boolean
}

export interface Settings {
  workSecs: number
  breakSecs: number
  snoozeSecs: number
  style: BreakStyle
  chime: boolean
  prewarn: boolean
  skipWhenIdle: boolean
  idleThresholdSecs: number
  deferOnFullscreen: boolean
  autostart: boolean
}

/** Emitted by Rust once a second. */
export const EVENT_TICK = 'irys://tick'

/** `754` -> `"12:34"`, matching `fmt_mmss` on the Rust side. */
export function fmtMmss(secs: number): string {
  const safe = Math.max(0, Math.floor(secs))
  const mins = Math.floor(safe / 60)
  const rest = safe % 60
  return `${String(mins).padStart(2, '0')}:${String(rest).padStart(2, '0')}`
}
