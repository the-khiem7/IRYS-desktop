---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "roadmap"
status: "active"
updated: "2026-08-24"
code_ref: "f0b78f0"
---

# Roadmap

**v0.1.2 is released** - `v0.1.0`, `v0.1.1` and `v0.1.2` have each published from a green Windows build with no manual release step. `v0.1.2` is the first tag whose GitHub release was checked directly (via the public API, not just source inspection) and carries all 4 real assets: NSIS `.exe`, `.msi`, `.deb`, `.AppImage`. It bundles the Phase 15 WinUI 3 UI (`7bb879b`) and the `main.rs` Windows-subsystem fix (`1e735a5`); the user has now completed its Windows desktop acceptance. The temporary design briefs were removed after acceptance in `f0b78f0`.

## Phases

| # | Phase | Status | Commit | Final evidence |
|---|---|---|---|---|
| 0 | Toolchain prerequisites | **abandoned by constraint** | - | MSVC Build Tools cannot be installed without admin: `0x80070005 … _bootstrapper is denied`, installer exit 5002. Superseded by phases 9 and 10. |
| 1 | Scaffold (Vite 2-entry, `src-vue/`, Cargo, tsconfig) | complete | `dbc5c1b` | `vite build` emits `index.html` + `break.html` as separate bundles. |
| 2 | Pure scheduling core + tests | **verified** | `32f3faa` | `cargo test`: **43 passed, 0 failed** in 0.01 s, on both Linux and windows-msvc. Covers both suppression rules, sleep/wake, pause/resume, clamping. |
| 3 | Scheduler, tray, window manager, IPC, capabilities | **verified at runtime** | `2693aeb` | Clippy clean, and the first run confirmed the tray icon appears with a live-updating tooltip, and that `invoke` works under minimal capabilities. |
| 4 | Break UI - AnimatedEye, CountdownRing, RuleGuide, Overlay + Toast | **runtime verified** | `9f74517`, `7bb879b` | On v0.1.2 the user confirmed full-screen and Corner reminders, plus Escape, Skip and Snooze dismissal. |
| 5 | Settings window + persistence + autostart | **runtime verified** | `9f74517` | Settings live countdown, persisted settings and autostart were confirmed by the user. Registry add/remove and re-login were not separately inspected. |
| 6 | Windows idle + fullscreen probes | **runtime verified** | `2c6d604` | Built on windows-msvc and confirmed live on v0.1.2: idle skips/resets and fullscreen defers the due break. |
| 7 | Icon generation | **complete** | `7c08bab` | `npm run icon` produced the desktop set; the eye reads correctly at 128px and in the tray. |
| 8 | Frontend verification | **complete** | - | `vue-tsc --noEmit` and `vite build` clean (break bundle 3.08 kB, carries no settings code). |
| 9 | Windows CI | **fully green** | `3dd940b` | All 3 jobs SUCCESS, including the release profile and both installers. |
| 10 | Docker verification path | **complete** | `2a77781` | `npm run verify`: frontend gates, fmt, clippy and 43 tests on a Linux toolchain, in seconds. Removed the dependency on CI logs that return 403 without a token. |
| 11 | Local Windows builds via cargo-xwin | **complete** | `86447a8` | `npm run build:windows` produces a real PE32 NSIS installer from Linux. 386 s cold release, 249 s cold dev, **35 s incremental**. |
| 12 | Release automation | **complete** | `507ccc8`, `519c761`, `ce33e30`, `ccf4340` | `release.yml` fired on `v0.1.0` and succeeded end to end: all gates on Windows, tag matched `tauri.conf.json`, both installers published non-draft. Repeated automatically for `v0.1.1` and `v0.1.2` with no manual step; `v0.1.2` confirmed via the GitHub API as `draft:false`, `prerelease:false`, with 4 uploaded assets. |
| 13 | Manual runtime verification | **verified on Windows v0.1.2** | - | User confirmed Phase 15 navigation, stored light/dark appearance, full-screen/Corner reminder, Escape/Skip/Snooze, idle/fullscreen suppression, sleep/wake and no-terminal GUI startup. |
| 14 | Linux packages in CI/release | **artifacts published, desktop runtime unverified** | `2f91859`, `ccf4340` | `ci.yml` adds `bundle-linux` on `ubuntu-22.04`; `release.yml` adds `build-linux` and uploads `.deb` + AppImage assets. `v0.1.2`'s GitHub release now carries both as real uploaded assets (`IRYS_0.1.2_amd64.deb`, `IRYS_0.1.2_amd64.AppImage`), which narrows the gap from "does the pipeline exist" to "does the resulting install/tray/break behave" - still not inspected on a Linux desktop. |
| 15 | WinUI 3 desktop UI refresh | **runtime verified on Windows v0.1.2** | `7bb879b` | Settings navigation and stored appearance, full-screen and Corner reminders, and escapability were user-verified. The temporary review briefs were intentionally removed in `f0b78f0`; Rust scheduling and IPC remain unchanged. |

## Dependencies

```mermaid
flowchart LR
    P2["2 · pure core"] --> P3["3 · scheduler + tray"]
    P6["6 · OS probes"] --> P3
    P3 --> P4["4 · break UI"]
    P3 --> P5["5 · settings"]
    P10["10 · docker verify"] --> P11["11 · cargo-xwin build"]
    P4 --> P11
    P5 --> P11
    P11 --> P13["13 · manual verification"]
    P6 --> P9["9 · windows CI"]
    P9 --> P12["12 · release"]
    P13 --> P12
```

Docker (10, 11) covers the development loop. Windows CI (9, 12) covers the two things a Linux container structurally cannot: `platform/win.rs` and the MSI.

## Fixes already worked through

Recorded so they are not rediscovered:

1. **`npm ci` failed** - `package-lock.json` was never regenerated after three `@tauri-apps/plugin-*` packages were dropped from `package.json`. *Whenever dependencies change, commit the lockfile.*
2. **`cargo fmt --all --check` failed** - the Rust was hand-written and rustfmt had never run. Verifiable locally, since fmt parses but never links.
3. **`cargo clippy -D warnings` failed** - one unused `Manager` import in `effects.rs`. `tray_by_id` is inherent on `AppHandle`, not a `Manager` method.
4. **Windows containers** - investigated and ruled out; see `hallucination`.
5. **`set -e` and `&&`** - `[ test ] && cmd` aborts a script when the test is false. Written twice, in `build-windows.sh` and `release.yml`. Use `if`.
6. **Vite 8 has no esbuild** - it builds on rolldown, so naming `'esbuild'` as the minifier fails to resolve.

## Remaining work

Windows desktop acceptance is complete for v0.1.2. The remaining boundaries are platform-specific:

1. **Linux desktop runtime** - install either published v0.1.2 Linux asset and verify launch, tray, break and dismissal on a real Linux desktop.
2. **MSI installation** - only the per-user NSIS installer has direct installation evidence; the MSI remains uninstalled.
3. **Optional operational checks** - inspect the autostart registry add/remove lifecycle and measure idle CPU if those release-quality metrics become required.

Regression note: the Linux container does **not** compile `platform/win.rs`. Treat any change to that file as CI-verified only.

UI implementation note: phase 15 is a visual/layout change, not permission or scheduler work. Preserve the existing Rust-owned state, IPC contract, two-window model, accessibility, reduced-motion support, and escapability while replacing the presentation.
