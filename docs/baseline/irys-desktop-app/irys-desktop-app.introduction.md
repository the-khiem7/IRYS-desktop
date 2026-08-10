---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "introduction"
status: "active"
updated: "2026-08-10"
code_ref: "ae9fccc"
---

# Irys - scope and current truth

## Scope

A Windows-first desktop companion that keeps the **30-30-30 rule** for the user: every 30 minutes, look ~30 feet (10 m) away for 30 seconds. It runs from the system tray with no window visible, interrupts on schedule with an animated eye and a countdown, and stays quiet when the user is already away or presenting.

Stack: **Tauri 2 + Vue 3 + TypeScript + Vite**. Rust owns all timing and state; Vue is presentation only.

## Current truth

All eight planned phases are **implemented and committed** - 2,689 lines of Rust across 11 files, 1,510 lines of Vue/TS across 13 files, 43 `#[test]` cases in the core.

**The Rust now compiles and its tests pass.** A Docker Linux toolchain closed the gap that local policy created (see `useguide`). What that changed:

| Layer | State |
|---|---|
| Frontend types (`vue-tsc --noEmit`) | ✅ clean, locally and in container |
| Frontend build (`vite build`) | ✅ clean - 2 entries, break bundle 3.08 kB |
| Icon generation (`npm run icon`) | ✅ verified, output inspected visually |
| `cargo fmt --all --check` | ✅ passes - runs locally too, since fmt needs no linker |
| `cargo clippy -D warnings` | ✅ passes in container, zero warnings |
| **43 core tests** | ✅ **43 passed, 0 failed** - first execution, 0.01 s |
| `platform/win.rs` | ✅ compiles on windows-msvc in CI |
| Release profile (`lto`, `panic = "abort"`) | ✅ builds |
| MSI + NSIS installers | ✅ built, 2.83 MB artifact |
| Runtime, first manual run | ✅ **it works** - see below |
| Remaining manual checks | ❌ escapability, background accuracy, suppression, autostart |

Every automated gate in the plan passes. Three things this settled that had been guesses: the Tauri APIs (`cursor_position`, `monitor_from_point`, `StoreExt`, `autolaunch`, `TrayIconBuilder`) all type-check; the hand-written Win32 FFI was right first time (`GetWindowRect` returns `Result<()>`, `HWND == HWND::default()` is valid, `GetMonitorInfoW` returns `BOOL`); and the schedule logic is genuinely correct rather than merely plausible.

### First manual run - confirmed working

Installed from the locally cross-compiled NSIS build on 2026-08-04. Observed directly:

- **Per-user install**, into the local app data directory, with no admin prompt
- **Tray icon appears** and its tooltip counts down live (`Irys - next break in 29:54`), so `default_window_icon()` was not the risk it looked like
- **`invoke` and event delivery work** - the settings window shows a live countdown. This resolves the one security-relevant guess in the build: the minimal `core:event`-only capabilities really are sufficient for an app's own commands
- **The overlay renders correctly** - a transparent, frameless, always-on-top window did *not* come out as a black rectangle on Windows 11. The countdown ring sweeps, the eye animates, and Skip and Snooze are both visible
- **Skip and Snooze dismiss the overlay**; background/minimized timing, Corner reminder, autostart and persisted settings were subsequently confirmed by the user after sustained use

### Still to check by hand

- **Escape** - Skip and Snooze are user-confirmed; Escape remains to be observed after the upcoming break UI refresh
- **Runtime suppression** - idle/fullscreen behaviour needs a live desktop test
- **Sleep/wake** - suspend across a break boundary; no stale break should fire on resume
- **Windows GUI startup** - rebuild after the `windows_subsystem` change and confirm the app opens without a terminal panel

## Target

**v0.1.0 is released**, with the NSIS and MSI installers published from a green Windows build. Since that release, HEAD also configures CI and release automation to build Linux `.deb` and AppImage packages; no post-change tag, package artifact, or Linux desktop run was inspected in this checkpoint. What is left is hands-on confirmation, led by escapability and then background accuracy - a break firing on time with every window minimised, which is the entire reason the clock lives in Rust.

Phase 15 implements the Windows-first visual direction: **IRYS** is the user-facing product spelling. Settings use WinUI 3 / Windows 11 Fluent with a user-selectable local light/dark appearance; the full-screen break is intentionally dark-only to avoid a bright flash at night. The review prototypes remain the visual reference at `docs/brief/irys-winui3-settings.html` and `docs/brief/irys-winui3-break.html`; they are now ported into `src-vue` and await Windows desktop verification.

## Constraints

1. **No administrator rights on the development machine** - withheld by policy on a managed corporate workstation. MSVC Build Tools installs machine-wide and so cannot be installed. Without a linker, *nothing* Rust-side runs locally: not `cargo build`, not `cargo test`, and not even `cargo check`, because `tauri-build`'s `build.rs` must be linked and executed before checking begins. Rust is installed (user profile, not on `PATH`) and is not the problem. This is a policy block, not a misconfiguration - do not retry the install or attempt elevation, and never request admin credentials.
2. **Docker is the development loop; CI is release automation.** The owner has explicitly prohibited host Rust toolchains and host Cargo commands: every Rust format, lint, test and build operation runs in Docker. A per-user Docker Desktop install needs no admin, and a Linux toolchain runs every gate in seconds plus cross-compiles a real Windows installer via `cargo-xwin`. CI covers only what a Linux container structurally cannot: `platform/win.rs`, which is `#[cfg(windows)]`, and the MSI, which needs WiX. See `useguide`.
3. **No corporate identifiers in committed files** - no hostnames, account names, or domains, in docs, comments, or commit messages. Describe findings generically.
4. **Commit at every phase boundary**, not once at the end.
5. **The break overlay must always be dismissible.** A frameless, transparent, always-on-top window that could not be escaped is indistinguishable from UI-spoofing malware. Escape, Skip and Snooze are a requirement, not polish.
