---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "roadmap"
status: "blocked"
updated: "2026-08-03"
code_ref: "3dd940b"
---

# Roadmap

## Phases

| # | Phase | Status | Commit | Final evidence |
|---|---|---|---|---|
| 0 | Toolchain prerequisites | **abandoned by constraint** | - | Rust installed OK. MSVC Build Tools cannot be installed without admin; failed with `0x80070005 … _bootstrapper is denied`, installer exit 5002. Verified no MSVC or Windows SDK anywhere on the machine. Superseded by phase 9 (CI). |
| 1 | Scaffold (Vite 2-entry, `src-vue/`, Cargo, tsconfig) | complete | `dbc5c1b` | `vite build` emits `index.html` + `break.html` as separate bundles. |
| 2 | Pure scheduling core + tests | **complete and verified** | `32f3faa` | `cargo test`: **43 passed, 0 failed** in 0.01 s. Covers both suppression rules, sleep/wake, pause/resume, clamping. |
| 3 | Scheduler, tray, window manager, IPC, capabilities | **compiles clean** | `2693aeb` | `cargo clippy -D warnings` clean. Runtime behaviour still needs a desktop. |
| 4 | Break UI - AnimatedEye, CountdownRing, RuleGuide, Overlay + Toast | code complete, **appearance unverified** | `9f74517` | `vue-tsc` clean; `vite build` clean. Never rendered. |
| 5 | Settings window + persistence + autostart | code complete, **unverified** | `9f74517` | `vue-tsc` clean. Round-trip never exercised. |
| 6 | Windows idle + fullscreen probes | code complete, **still unverified** | `2c6d604` | `#[cfg(windows)]`, so the Linux container never parses it. The only Rust left unproven. All `unsafe` confined here. |
| 7 | Icon generation | **complete** | `7c08bab` | `npm run icon` produced the full desktop set; 128px output inspected and reads as an eye. |
| 8 | Frontend verification | **complete** | - | `vue-tsc --noEmit` clean; `vite build` clean (break bundle 3.08 kB, carries no settings code). |
| 9 | Windows CI as the build gate | **partially green** | `3dd940b` | Frontend job passes. Rust job now reaches `platform/win.rs`, which nothing else can compile. |
| 10 | Docker verification path | **complete and proven** | `2a77781` | `docker compose -f docker/compose.yml run --rm -T verify`: fmt, clippy and 43 tests all pass on a Linux toolchain, in seconds. Removed the dependency on unreadable CI logs. |

## Dependencies

```mermaid
flowchart LR
    P2["2 · pure core"] --> P3["3 · scheduler + tray"]
    P6["6 · OS probes"] --> P3
    P3 --> P4["4 · break UI"]
    P3 --> P5["5 · settings"]
    P7["7 · icon"] --> P9["9 · CI"]
    P4 --> P9
    P5 --> P9
    P8["8 · frontend verify"] --> P9
    P9 --> GATE["Rust verified · installers built"]
```

Phase 10 (Docker) verified phases 2-5 and 8. Phase 9 (Windows CI) remains the only
gate for phase 6 and the installers.

## Fixes already worked through

Recorded so they are not rediscovered:

1. **`npm ci` failed** - `package-lock.json` was never regenerated after three
   `@tauri-apps/plugin-*` packages were dropped from `package.json`. Fixed by
   `npm install`; reproduced locally first. *Whenever dependencies change, commit
   the lockfile.*
2. **`cargo fmt --all --check` failed** - the Rust was hand-written and rustfmt had
   never run. Fixed, and verifiable locally since fmt never links.
3. **`cargo clippy -D warnings` failed** - one unused `Manager` import in
   `effects.rs`. `tray_by_id` is inherent on `AppHandle`, not a `Manager` method.
4. **Windows containers** - investigated and ruled out; see `hallucination`.

## Next action

**Watch the Windows CI run for `bf6c7b9` - it is the first build of
`platform/win.rs`.**

Everything ahead of it now passes, so this run reaches code that has never been
compiled anywhere. Expect failures only in:

1. **`platform/win.rs` signatures** - whether `GetWindowRect` returns
   `Result<()>` or `BOOL`, and whether `HWND == HWND::default()` is valid. Both
   move between `windows` crate releases. Fix by reading the 0.61 docs for the
   exact return types.
2. **`tauri build` bundling** - WiX/NSIS toolchain download, and the release
   profile (`lto`, `panic = "abort"`) compiling for the first time.

Note the Linux container will *not* catch regressions in `platform/win.rs`. Treat
any change to that file as CI-only verified.

After green: download the NSIS artifact (per-user install, needs no admin) and run
the manual checks in `useguide`, starting with escapability.
