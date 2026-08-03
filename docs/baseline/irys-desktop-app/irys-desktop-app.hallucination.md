---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "hallucination"
status: "active"
updated: "2026-08-03"
code_ref: "3dd940b"
---

# Open questions and closed decisions

## Unverified claims — do not treat as fact

Everything Rust-side is written but never compiled. Specifically, these are
*design intentions* with no execution behind them:

| Claim | Why it is unverified |
|---|---|
| The 43 core tests pass | `cargo test` has never run |
| `platform/win.rs` compiles against `windows` 0.61 | Signatures for `GetWindowRect` / `HWND` comparison shift between releases |
| `cursor_position()` and `monitor_from_point()` exist on `WebviewWindow` | Taken from Tauri 2 API memory, not checked against 2.11 |
| Per-window capabilities are sufficient | See open question 1 |
| The tray icon appears and its tooltip updates | Never run |
| Idle and fullscreen detection actually suppress breaks | Logic is unit-tested in principle; the OS probes feeding it are not |
| Either window looks as intended | Never rendered; no screenshot exists |
| `codegen-units=1` + `lto` + `panic=abort` release profile builds | Never built |

## Open questions

1. **Do Irys's own `#[tauri::command]`s really need no capability grant?**
   Each window is granted only `core:event:allow-listen` / `allow-unlisten`, on
   the understanding that an app's own commands are exempt from the permission
   system. If `invoke` is rejected at runtime, add `core:default` to
   `capabilities/settings.json` and re-tighten from there. This is the one
   security-relevant guess in the build, and it fails loudly rather than silently.

2. **Will the WebAudio chime ever sound?** Webviews start an `AudioContext`
   suspended until a user gesture. The first chime of a session may be silent.
   Judged acceptable — the setting is off by default and the visual reminder is
   the real signal — but if a chime is wanted reliably, it needs a different
   mechanism.

3. **Is the toast's fixed taskbar allowance right?** Tauri 2 exposes monitor
   bounds but not the work area, so the toast keeps clear of the bottom edge by a
   fixed 56 px logical allowance. Wrong for an unusually tall, auto-hidden, or
   side-docked taskbar.

4. **Does `SystemTime`-based sleep detection behave on real hardware?** The
   scheduler measures wall-clock deltas specifically because a monotonic clock can
   stop across sleep. The `SLEEP_GAP_SECS = 90` threshold is a guess that only a
   real suspend/resume can validate.

5. **Does the overlay land on the right monitor?** Depends on open question in
   roadmap item 3 resolving in favour of the cursor-following path.

## Closed decisions

| Decision | Resolution | Rationale |
|---|---|---|
| Where the timer lives | **Rust, not JavaScript** | Webviews throttle timers in hidden/minimised windows, and Irys spends its life with no window shown. A JS timer would drift silently — fatal for the app's only job. |
| Break presentation | **Both Overlay and Toast, user-selectable** | User's explicit choice: "both, let user choose on their need". One component tree serves both. |
| Configurability | **Full settings window + persistence** | User's explicit choice over hard-coded 30/30/30. Also makes a 1-minute test interval possible. |
| Background behaviours | **All four: tray, autostart, idle-skip, fullscreen-defer** | User selected all four. |
| Idle vs fullscreen handling | **Asymmetric: idle *resets*, fullscreen *defers*** | Idle means the eyes already rested, so a reminder is noise. A fullscreen app means the break is still owed — just not over someone's presentation. |
| Frontend directory | **`src-vue/`, not `src/`** | User's explicit request. `src-tauri/src/` keeps its name; cargo fixes it. |
| Plugin work location | **All in Rust; no JS plugin packages** | Deviation from plan, strictly tighter. Frontend then needs zero plugin permissions, so the always-on-top break window has no path to store, filesystem, shell or network. |
| `Effect::Chime` | **Dropped from the effect enum** | The break window plays the chime on the phase change it observes, removing a race where the event could arrive before the window finished loading. |
| `prewarn` setting | **Added to `Settings`** | The plan listed a heads-up notification under plugins but omitted it from the settings shape. |
| Multi-monitor overlay | **Out of scope** | User picked Overlay + Toast, not the all-monitors variant. `windows_mgr.rs` is built around show-on-a-monitor, so an `overlayAllMonitors` toggle later is contained. |
| TypeScript version | **Pinned `~5.9`, not latest `7.x`** | 7.x is the native rewrite; `vue-tsc` 3.x is validated against the TS 5 compiler API. |
| Vite minifier | **`minify: true`, not `'esbuild'`** | Verified failure: Vite 8 builds on rolldown and does not ship esbuild, so naming it fails to resolve. |
| `windows` crate version | **Pinned `0.61`** | Matches Tauri's own dependency so cargo shares one compiled copy. |
| Local MSVC install | **Abandoned** | Policy block, not a fixable error. Verification moved to CI. |
| Corporate identifiers in docs | **Scrubbed and prohibited** | A hostname and account names had been written into `PLAN.md`. Never pushed; removed by amend + `reflog expire` + `gc --prune=now`, verified absent from the whole object store. |
| Diagram format in `PLAN.md` | **Mermaid for the architecture graph, ASCII for the file tree** | User's correction — a file tree reads better as ASCII. |
