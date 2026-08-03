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

### Verify locally in Docker (the fast loop - prefer this)

```bash
docker compose -f docker/compose.yml run --rm -T verify        # all Rust gates
docker compose -f docker/compose.yml run --rm verify bash      # poke around
```

Runs `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` on a Linux
toolchain, because the development machine has no MSVC linker and therefore cannot
run *any* cargo command that links - not even `cargo check`, since `tauri-build`'s
`build.rs` must be linked and executed first.

`verify.sh` runs all three gates even when an earlier one fails. That is
deliberate and the opposite of CI's fail-fast: a formatting failure used to hide
the clippy error, which in turn hid whether the tests passed.

Source is bind-mounted, so host edits need no image rebuild. `CARGO_TARGET_DIR`
points at a named volume so Linux artefacts never collide with the host's
Windows-MSVC `target/`.

**What this cannot verify**, by construction: `platform/win.rs` is
`#[cfg(windows)]` so a Linux compiler never parses it; likewise the tray, the
transparent overlay, WebView2, and the MSI/NSIS bundles.

Docker Desktop's **Windows container mode is not available here** - it needs the
privileged `com.docker.service` and the Containers Windows feature, both
admin-gated. The per-user Docker install that works without admin is exactly the
one that cannot switch engines. Don't spend time retrying it.

### Verify without Docker

```bash
npx vue-tsc --noEmit
npx vite build
cd src-tauri && cargo fmt --all --check   # works: fmt parses, never links
```

Any other cargo command fails locally. The error is misleading - Rust finds Git's
POSIX `link` utility on `PATH` and reports `link: extra operand ...`, not
"linker not found".

### Verify on CI (the Windows gate)

`.github/workflows/ci.yml` on `windows-latest` is the only place `platform/win.rs`
and the installers get built. Three jobs:

| Job | Runs |
|---|---|
| `frontend` | `npm ci`, `vue-tsc --noEmit`, `vite build` |
| `rust` | `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all-features` |
| `bundle` | `tauri build` → MSI + NSIS uploaded as `irys-windows-installers` |

Reading CI without a token: the public REST API exposes run and job metadata
(`/actions/runs`, `/actions/runs/{id}/jobs`), which gives per-step pass/fail. Log
downloads return **403** unauthenticated, so the container loop above is the way
to see actual compiler output.

Note `npm ci` requires `package-lock.json` to match `package.json` exactly. After
changing dependencies, run `npm install` and commit the lockfile, or CI fails
before it starts.

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
