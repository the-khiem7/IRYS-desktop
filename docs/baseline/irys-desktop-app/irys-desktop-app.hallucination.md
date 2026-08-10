---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "hallucination"
status: "active"
updated: "2026-08-10"
code_ref: "ae9fccc"
---

# Open questions and closed decisions

## Now verified (was unverified)

Confirmed by execution, not inspection. A Docker Linux toolchain closed the compile-and-test gap; Windows CI covered `platform/win.rs`; the first manual install covered runtime.

| Claim | Evidence |
|---|---|
| The 43 core tests pass | `cargo test`: **43 passed, 0 failed**, 0.01 s, on Linux **and** windows-msvc |
| The whole crate compiles, `platform/win.rs` included | Windows CI green |
| The hand-written Win32 FFI is correct | `GetWindowRect` returns `Result<()>`, `HWND == HWND::default()` is valid, `GetMonitorInfoW` returns `BOOL` - all three right first time |
| Zero clippy warnings under `-D warnings` | one unused import found and fixed |
| `cursor_position()` / `monitor_from_point()` exist on `WebviewWindow` | type-checks against Tauri 2.11 |
| `StoreExt`, `autolaunch()`, `TrayIconBuilder`, `MenuItem::set_text` exist as used | type-check |
| The schedule logic is correct, not merely plausible | both suppression rules, sleep/wake, pause/resume and clamping all asserted |
| Release profile (`lto`, `codegen-units = 1`, `panic = "abort"`) builds | `tauri build` on Windows, and locally via cargo-xwin |
| **The NSIS installer works** | per-user install into local app data, no admin prompt |
| **The tray icon appears** | eye icon visible, tooltip counts down live (`next break in 29:54`) |
| **`invoke` and events work under minimal capabilities** | settings window shows a live countdown - resolves open question 1 |
| **The overlay renders** | transparent frameless always-on-top did **not** come out black on Windows 11; ring sweeps, eye animates, Skip and Snooze visible |
| Windows GUI startup has no terminal panel | `main.rs` selects the Windows subsystem for every Windows build. This code change still needs a rebuilt-app user check. |
| Skip, Snooze, background timing, Corner, autostart and persistence work | User-reported sustained Windows use; Escape, idle/fullscreen suppression and sleep/wake remain unverified. |
| **The release pipeline works end to end** | `v0.1.0` published non-draft with both installers. The tag check accepted `v0.1.0` against `0.1.0`, and the Windows gate ran before publishing |
| Linux packaging is wired into CI and releases | Source inspection at `ae9fccc`: Ubuntu jobs build `.deb` + AppImage and publish them with Windows artifacts. This is not evidence that a Linux run, artifact, or desktop workflow has succeeded. |

## Still unverified - do not treat as fact

Everything now compiles, all tests pass, and the app has been installed and run. What is left is behaviour that needs a person at a desktop.

| Claim | Why it is still unverified |
|---|---|
| Escape dismisses the overlay | Skip and Snooze are user-confirmed, but Escape itself has not yet been observed |
| Idle and fullscreen suppression work at runtime | The consuming logic is unit-tested and the probes compile, but they have never run against a live desktop |
| No stale break after sleep/wake | `SLEEP_GAP_SECS = 90` is unit-tested but never met a real suspend |
| The MSI installs correctly | Only the NSIS build has been installed |
| Linux `.deb` / AppImage build, install, tray, and break runtime work | Packaging configuration exists after `v0.1.0`, but no post-change run/artifact or Linux desktop observation was inspected |

## Open questions

1. ~~**Do Irys's own `#[tauri::command]`s really need no capability grant?**~~ **RESOLVED on the first run: yes, no grant is needed.** The settings window shows a live countdown, which means both `invoke` and event delivery work with only `core:event:allow-listen` / `allow-unlisten` granted. `core:default` was not required. This had been the one security-relevant guess in the build, and it resolved in favour of least privilege: neither window can reach the store, the filesystem, the shell, or the network.

2. **Will the WebAudio chime ever sound?** Webviews start an `AudioContext` suspended until a user gesture. The first chime of a session may be silent. Judged acceptable - the setting is off by default and the visual reminder is the real signal - but if a chime is wanted reliably, it needs a different mechanism.

3. **Is the toast's fixed taskbar allowance right?** Tauri 2 exposes monitor bounds but not the work area, so the toast keeps clear of the bottom edge by a fixed 56 px logical allowance. Wrong for an unusually tall, auto-hidden, or side-docked taskbar.

4. **Does `SystemTime`-based sleep detection behave on real hardware?** The scheduler measures wall-clock deltas specifically because a monotonic clock can stop across sleep. The `SLEEP_GAP_SECS = 90` threshold is a guess that only a real suspend/resume can validate.

5. **Does the overlay land on the right monitor?** Depends on open question in roadmap item 3 resolving in favour of the cursor-following path.

6. **Do the README's Escape/Skip/Snooze promises have manual evidence?** The current code wires Escape and both buttons to the Rust commands, and focuses overlay windows so Escape does not need a click first. That is strong code-inspection evidence, but no key/button press was observed in the manual run recorded by this pack; keep the runtime claim unverified until exercised.

## Closed decisions

| Decision | Resolution | Rationale |
|---|---|---|
| Where the timer lives | **Rust, not JavaScript** | Webviews throttle timers in hidden/minimised windows, and Irys spends its life with no window shown. A JS timer would drift silently - fatal for the app's only job. |
| Break presentation | **Both Overlay and Toast, user-selectable** | User's explicit choice: "both, let user choose on their need". One component tree serves both. |
| Configurability | **Full settings window + persistence** | User's explicit choice over hard-coded 30/30/30. Also makes a 1-minute test interval possible. |
| Background behaviours | **All four: tray, autostart, idle-skip, fullscreen-defer** | User selected all four. |
| Idle vs fullscreen handling | **Asymmetric: idle *resets*, fullscreen *defers*** | Idle means the eyes already rested, so a reminder is noise. A fullscreen app means the break is still owed - just not over someone's presentation. |
| Frontend directory | **`src-vue/`, not `src/`** | User's explicit request. `src-tauri/src/` keeps its name; cargo fixes it. |
| Plugin work location | **All in Rust; no JS plugin packages** | Deviation from plan, strictly tighter. Frontend then needs zero plugin permissions, so the always-on-top break window has no path to store, filesystem, shell or network. |
| `Effect::Chime` | **Dropped from the effect enum** | The break window plays the chime on the phase change it observes, removing a race where the event could arrive before the window finished loading. |
| `prewarn` setting | **Added to `Settings`** | The plan listed a heads-up notification under plugins but omitted it from the settings shape. |
| Multi-monitor overlay | **Out of scope** | User picked Overlay + Toast, not the all-monitors variant. `windows_mgr.rs` is built around show-on-a-monitor, so an `overlayAllMonitors` toggle later is contained. |
| TypeScript version | **Pinned `~5.9`, not latest `7.x`** | 7.x is the native rewrite; `vue-tsc` 3.x is validated against the TS 5 compiler API. |
| Vite minifier | **`minify: true`, not `'esbuild'`** | Verified failure: Vite 8 builds on rolldown and does not ship esbuild, so naming it fails to resolve. |
| `windows` crate version | **Pinned `0.61`** | Matches Tauri's own dependency so cargo shares one compiled copy. |
| Local MSVC install | **Abandoned** | Policy block, not a fixable error. |
| Local Rust verification | **Docker, Linux containers only** | Owner decision: do not install or invoke Rust tooling on the host. The Docker toolchain runs fmt, clippy and all 43 tests while its Cargo/target caches remain in volumes. |
| Windows containers for `platform/win.rs` | **Not possible here** | Verified: needs the privileged `com.docker.service` plus the Containers Windows feature, both admin-gated. The per-user install that works without admin is precisely the one that cannot switch engines. `-SwitchDaemon` fails with `context deadline exceeded`. Don't retry. |
| Where `platform/win.rs` gets verified | **Windows CI only** | No local option exists. It is ~40 lines of FFI, so the exposure is bounded. |
| Local Windows builds | **`cargo-xwin` in the Linux container** | Fetches Microsoft's Windows SDK and links with `lld-link`, producing a real PE32 NSIS installer without MSVC on the host. `llvm-rc` handles the Windows icon and manifest resources; `makensis` packages. Verified end to end. |
| MSI locally | **Not possible** | WiX is Windows-only. NSIS is the build that matters anyway, since it installs per-user without admin. |
| Local build profile | **Dev by default, release on request** | The release profile's `lto = true` and `codegen-units = 1` keep the binary small but disable parallel codegen and add a single-threaded whole-program pass. Measured: 386 s cold release, 249 s cold dev, **35 s incremental dev**. Windows builds now hide the console; use debugger/log capture for diagnostics. |
| Division of labour | **Docker for development, CI for release** | Explicit owner decision. The container gives a 35 s edit-rebuild loop; CI covers `platform/win.rs` and the MSI, and publishes on a tag. |
| Release trigger | **`v*` tag, published not drafted** | Pushing a version tag is already deliberate, so the tag is the gate. `release.yml` re-runs every gate on Windows first, because passing locally never compiled `platform/win.rs`. |
| Version scheme | **`v0.1.1` is the next release** | `v0.1.0` is already published. The tag must carry all three semver parts: `release.yml` compares the stripped tag against `tauri.conf.json`, so `v0.1` would fail against `0.1.1`. |
| Corporate identifiers in docs | **Scrubbed and prohibited** | A hostname and account names had been written into `PLAN.md`. Never pushed; removed by amend + `reflog expire` + `gc --prune=now`, verified absent from the whole object store. |
| Diagram format in `PLAN.md` | **Mermaid for the architecture graph, ASCII for the file tree** | User's correction - a file tree reads better as ASCII. |
| Desktop UI direction | **IRYS + Fluent settings; dark break** | User-facing product name is `IRYS`. Phase 15 is approved and implemented in `src-vue`: Settings use a NavigationView-like desktop composition with local selectable light/dark appearance; the wide calm break view is dark-only and retains the complete animated eye. Review artifacts remain in `docs/brief/` as visual references. |
