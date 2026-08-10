---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "useguide"
status: "active"
updated: "2026-08-10"
code_ref: "ae9fccc"
---

# Contracts and procedures

## IPC contract - the Rust/Vue boundary

Rust is the single source of truth. `Settings` is serialised `#[serde(rename_all = "camelCase")]`, so `src-vue/lib/types.ts` must mirror the field names exactly. Every `invoke` goes through `src-vue/lib/ipc.ts`, so a renamed command is a type error rather than a silent runtime failure.

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

Clamped in `core::Settings::sanitized()`. Out-of-range input is corrected, not rejected.

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

### The division of labour

**Docker is the development loop. CI is release automation.** Deliberate: the container gives an immediate answer, while CI covers the two things a Linux container structurally cannot.

| | Docker (local) | Windows CI |
|---|---|---|
| `vue-tsc`, `vite build` | ✅ | ✅ |
| `cargo fmt`, `clippy`, 43 tests | ✅ seconds | ✅ minutes |
| NSIS installer | ✅ via `cargo-xwin` | ✅ |
| `platform/win.rs` | ❌ cfg-gated out | ✅ only here |
| MSI | ❌ WiX is Windows-only | ✅ |
| Running the app | ❌ no desktop | ❌ no desktop |

### Verify locally in Docker (the fast loop - prefer this)

```bash
npm run verify                  # vue-tsc, vite build, cargo fmt, clippy, 43 tests
npm run build:windows           # NSIS installer into ./out - dev profile, fast
npm run build:windows:release   # optimised, same as what ships
npm run shell                   # bash inside the container
```

Measured build times:

| Build | Time |
|---|---|
| Cold, release profile | 386 s |
| Cold, dev profile | 249 s |
| **Incremental, dev profile** | **35 s** (14 s compile + packaging) |

Use the dev profile while iterating. The release profile sets `lto = true` and `codegen-units = 1` in `Cargo.toml` to keep the shipped binary small, which disables parallel codegen and adds a largely single-threaded whole-program pass. The dev build is 3.1 MB against 1.4 MB and slower at runtime, neither of which matters when the question is whether Escape dismisses the overlay.

A Windows dev build also has **no console window**: `main.rs` selects `windows_subsystem = "windows"` for every Windows build, so closing a terminal panel cannot terminate IRYS. Use the debugger or captured application logs for `eprintln!` diagnostics instead of relying on a visible console.

The development machine has no MSVC linker, so it cannot run *any* cargo command that links - not even `cargo check`, since `tauri-build`'s `build.rs` must be linked and executed first. A Linux toolchain sidesteps that.

`npm run build:windows` cross-compiles a genuine Windows binary with `cargo-xwin`, which fetches Microsoft's Windows SDK headers and import libraries and links with `lld-link`. `XWIN_ACCEPT_LICENSE=1` in the Dockerfile accepts that SDK licence; it was an explicit owner decision, recorded in the file itself rather than left implicit.

Output lands in `./out` on the host, because the build directory lives in a Docker volume the host cannot see.

`verify.sh` runs all three gates even when an earlier one fails. That is deliberate and the opposite of CI's fail-fast: a formatting failure used to hide the clippy error, which in turn hid whether the tests passed.

Source is bind-mounted, so host edits need no image rebuild. `CARGO_TARGET_DIR` points at a named volume so Linux artefacts never collide with the host's Windows-MSVC `target/`.

**What this cannot verify**, by construction: `platform/win.rs` is `#[cfg(windows)]` so a Linux compiler never parses it; likewise the tray, the transparent overlay, WebView2, and the MSI/NSIS bundles.

Docker Desktop's **Windows container mode is not available here** - it needs the privileged `com.docker.service` and the Containers Windows feature, both admin-gated. The per-user Docker install that works without admin is exactly the one that cannot switch engines. Don't spend time retrying it.

### Host-only frontend checks (never Cargo)

```bash
npx vue-tsc --noEmit
npx vite build
```

Do not run `cargo`, `rustup`, or any Rust build tooling on the host. This is an explicit containment rule: all Rust format, lint, test, metadata and bundle operations run through `npm run verify`, `npm run build:windows*`, or `npm run shell`, which use the Docker toolchain and keep Rust caches out of the machine.

### Verify on CI (the platform gates)

The Windows jobs remain the only configured place where `platform/win.rs` and the MSI are built. Current source also defines Ubuntu jobs for `.deb` and AppImage. Their configuration was inspected at `ae9fccc`; this checkpoint did not inspect a post-change workflow result or Linux runtime.

**`ci.yml`** - every push and PR:

| Job | Runs |
|---|---|
| `frontend` | `npm ci`, `vue-tsc --noEmit`, `vite build` |
| `rust` | `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all-features` |
| `bundle` | `tauri build` → MSI + NSIS uploaded as `irys-windows-installers` |
| `bundle-linux` | Ubuntu 22.04 installs WebKitGTK/AppIndicator dependencies, then uploads `.deb` + AppImage as `irys-linux-packages` |

**`release.yml`** - on a `v*` tag, or manual dispatch. Re-runs the Windows Rust gates, checks the tag matches `tauri.conf.json`, builds Windows installers and Linux packages in parallel, then publishes all four artifact types. The prior `v0.1.0` evidence predates the Linux jobs, so do not call Linux shipping verified until a later tag/run and desktop check are observed.

Reading CI without a token: the public REST API exposes run and job metadata (`/actions/runs`, `/actions/runs/{id}/jobs`), which gives per-step pass/fail. Log downloads return **403** unauthenticated, so the container loop above is the way to see actual compiler output.

Note `npm ci` requires `package-lock.json` to match `package.json` exactly. After changing dependencies, run `npm install` and commit the lockfile, or CI fails before it starts.

### Regenerate the icon

```bash
npm run icon      # make-icon.mjs -> icon-source.png -> tauri icon
```

`scripts/make-icon.mjs` rasterises the eye and writes a PNG with only `node:zlib` - no image library, no binary asset committed. Its palette mirrors the `--iris*` tokens in `src-vue/styles/theme.css`; change one, change both. It emits mobile icon sets that should be deleted - this is a desktop app.

### Cut a release

```bash
# 1. Bump the version in ALL FOUR places, or release.yml fails the tag check:
#    package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json,
#    and the irys entry in src-tauri/Cargo.lock
# 2. Verify, then commit
npm run verify
# 3. Tag with all three semver parts - `v0.1`, would fail against `0.1.1`
git tag -a v0.1.1 -m "..."
git push origin main
git push origin v0.1.1
```

`release.yml` then re-runs `fmt`, `clippy` and the tests **on Windows** before publishing. That is not redundant with `npm run verify`: the local loop never compiles `platform/win.rs`, so a tag's Windows build is genuinely the first one. It also builds the MSI, which Docker cannot, and publishes both installers.

To refresh `Cargo.lock` after a version bump without a local linker:

```bash
docker compose -f docker/compose.yml run --rm -T shell \
  bash -c 'cd /app/src-tauri && cargo metadata --format-version 1 > /dev/null'
```

Note `cargo metadata --no-deps` does **not** write the lockfile; only the full-graph resolve does.

### Install without admin

Use the **NSIS** build (`*-setup.exe`), from `./out` after a local build or from a release. It installs per-user, under the local app data directory, with no admin prompt - confirmed. The MSI is per-machine and needs admin. Both are unsigned, so SmartScreen warns once: *More info -> Run anyway*.

### Manual checks

Set the work interval to 1 minute first, and have `Ctrl+Shift+Esc` ready.

**Confirmed working:**

- Per-user install, no admin prompt
- Tray icon appears; tooltip counts down live
- Settings window renders with a live countdown, so `invoke` and events both work
- Overlay renders: dark veil, sweeping ring, animated eye, Skip and Snooze visible

**Still to do, in priority order:**

1. **Escape** - Skip and Snooze are user-confirmed. Observe Escape after every break-window change; it is non-negotiable for an always-on-top overlay.
2. **Idle skip** - leave input alone past the threshold; no break, interval resets.
3. **Fullscreen defer** - F11 a video across the break moment; the overlay is postponed, then fires after exiting.
4. **Tray lifecycle** - close settings; breaks keep firing. Quit actually exits.
5. **Autostart cleanup** - user-confirmed as working; separately inspect the `HKCU\...\Run` add/remove and re-login lifecycle when practical.
6. **Sleep/wake** - suspend across a break boundary; no stale break on resume.
7. **Footprint** - idle CPU near zero.

## Conventions to keep

- **Commit at every phase boundary**, with the phase named in the message.
- **No corporate identifiers** - no hostnames, account names, or domains in committed files or commit messages. Write the finding, not the fingerprint.
- **`PLAN.md`**: Mermaid for the architecture graph, ASCII for the file tree.
- **UI direction**: use **IRYS** as the display name. Settings use WinUI 3 / Windows 11 Fluent and offer light/dark appearance. The full-screen break is dark-only and reuses the complete animated SVG eye (blink, distant gaze and pupil dilation) to avoid a nighttime flash. Use `docs/brief/irys-winui3-settings.html` and `docs/brief/irys-winui3-break.html` for review intent; Phase 15 ports that intent into `src-vue` without importing the HTML as a runtime asset. Keep implementation local, CSP-compatible, keyboard-accessible, and respectful of reduced motion.
