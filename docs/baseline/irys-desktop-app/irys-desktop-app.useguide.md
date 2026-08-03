---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "useguide"
status: "active"
updated: "2026-08-03"
code_ref: "3dd940b"
---

# Contracts and procedures

## IPC contract - the Rust/Vue boundary

Rust is the single source of truth. `Settings` is serialised
`#[serde(rename_all = "camelCase")]`, so `src-vue/lib/types.ts` must mirror the
field names exactly. Every `invoke` goes through `src-vue/lib/ipc.ts`, so a
renamed command is a type error rather than a silent runtime failure.

### Commands

| Command | Signature | Notes |
|---|---|---|
| `get_snapshot` | `() -> Snapshot` | Seeds the UI before the first tick. |
| `get_settings` | `() -> Settings` | |
| `set_settings` | `(settings: Settings) -> Result<Settings, String>` | **Returns what was actually stored**, which may differ - the core clamps. Always re-render from the return value. |
| `toggle_pause` | `() -> ()` | |
| `break_now` | `() -> ()` | Also un-pauses. |
| `skip_break` | `() -> ()` | Dismiss; restart the work interval. |
| `snooze_break` | `() -> ()` | Postpone by `snoozeSecs`. |
| `open_settings` | `() -> ()` | |
| `quit` | `() -> ()` | The only real exit. |

### Event

`irys://tick` carries a `Snapshot` once per second, to any window listening:

```ts
interface Snapshot {
  phase: 'work' | 'break' | 'paused'
  remainingSecs: number
  totalSecs: number   // current phase length, for the ring. Never zero.
  style: 'overlay' | 'toast'
  chime: boolean
}
```

### Settings and their clamps

Clamped in `core::Settings::sanitized()`. Out-of-range input is corrected, not
rejected.

| Field | Default | Range |
|---|---|---|
| `workSecs` | 1800 | 30 - 14400 |
| `breakSecs` | 30 | 5 - 600 |
| `snoozeSecs` | 300 | 30 - 3600 |
| `idleThresholdSecs` | 60 | 15 - 3600 |
| `style` | `overlay` | `overlay` \| `toast` |
| `chime` | `false` | |
| `prewarn` | `true` | Notification 30 s ahead |
| `skipWhenIdle` | `true` | |
| `deferOnFullscreen` | `true` | |
| `autostart` | `false` | Never enabled silently |

## Procedures

### Verify (CI is the build gate)

Local admin is unavailable, so `.github/workflows/ci.yml` on `windows-latest` is
where Rust is verified. Push, then read the run. Three jobs:

| Job | Runs |
|---|---|
| `frontend` | `npm ci`, `vue-tsc --noEmit`, `vite build` |
| `rust` | `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all-features` |
| `bundle` | `tauri build` → MSI + NSIS uploaded as `irys-windows-installers` |

### Verify locally (frontend only)

```bash
npx vue-tsc --noEmit
npx vite build
```

Both currently pass. Anything touching `src-tauri/` cannot be checked here.

### Regenerate the icon

```bash
npm run icon      # make-icon.mjs -> icon-source.png -> tauri icon
```

`scripts/make-icon.mjs` rasterises the eye and writes a PNG with only
`node:zlib` - no image library, no binary asset committed. Its palette mirrors the
`--iris*` tokens in `src-vue/styles/theme.css`; change one, change both. It emits
mobile icon sets that should be deleted - this is a desktop app.

### Manual checks once the app runs

Set `workSecs` to 60 first. In rough priority order:

1. **Always escapable** - Escape, Skip and Snooze each dismiss the overlay.
   *Non-negotiable; verify every time.*
2. **Background accuracy** - minimise everything, work elsewhere for a full
   interval, confirm the break still fires on time. This is the whole point, and
   the thing a JS timer would fail.
3. **Fires and ends on time**, tray tooltip counts down, both styles render.
4. **Idle skip** - leave input alone past the threshold; no break, interval resets.
5. **Fullscreen defer** - F11 a video across the break moment; the overlay is
   postponed, then fires after exiting.
6. **Tray lifecycle** - close settings; breaks keep firing. Quit actually exits.
7. **Autostart** - toggle on, confirm the `HKCU\...\Run` entry, re-login, confirm
   running; toggle off, confirm the entry is gone.
8. **Sleep/wake** - suspend across a break boundary; no stale break on resume.
9. **Footprint** - idle CPU near zero.

### Install without admin

Download the `bundle` job's artifact and use the **NSIS** installer - a per-user
install writes only under the user profile. Unsigned, so SmartScreen will warn on
first run until there is a code-signing certificate.

## Conventions to keep

- **Commit at every phase boundary**, with the phase named in the message.
- **No corporate identifiers** - no hostnames, account names, or domains in
  committed files or commit messages. Write the finding, not the fingerprint.
- **`PLAN.md`**: Mermaid for the architecture graph, ASCII for the file tree.
