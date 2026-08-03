---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "introduction"
status: "active"
updated: "2026-08-03"
code_ref: "3dd940b"
---

# Irys - scope and current truth

## Scope

A Windows desktop companion that keeps the **30-30-30 rule** for the user: every
30 minutes, look ~30 feet (10 m) away for 30 seconds. It runs from the system
tray with no window visible, interrupts on schedule with an animated eye and a
countdown, and stays quiet when the user is already away or presenting.

Stack: **Tauri 2 + Vue 3 + TypeScript + Vite**. Rust owns all timing and state;
Vue is presentation only.

## Current truth

All eight planned phases are **implemented and committed** - 2,689 lines of Rust
across 11 files, 1,510 lines of Vue/TS across 13 files, 43 `#[test]` cases in the
core.

**The Rust now compiles and its tests pass.** A Docker Linux toolchain closed the
gap that local policy created (see `useguide`). What that changed:

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
| **Runtime behaviour** | ❌ **never observed** - needs a desktop |
| **Either window's appearance** | ❌ **never seen** |

Every automated gate in the plan passes. Three things this settled that had been
guesses: the Tauri APIs (`cursor_position`, `monitor_from_point`, `StoreExt`,
`autolaunch`, `TrayIconBuilder`) all type-check; the hand-written Win32 FFI was
right first time (`GetWindowRect` returns `Result<()>`, `HWND == HWND::default()`
is valid, `GetMonitorInfoW` returns `BOOL`); and the schedule logic is genuinely
correct rather than merely plausible.

What remains is **only** what a machine cannot check for itself: whether the tray
appears, whether the overlay renders and is escapable, whether the Win32 probes
actually suppress a break, and whether it all still fires on time with every
window minimised.

## Target

A green Windows CI run producing MSI and NSIS installers, then hands-on
confirmation that breaks fire on schedule while every window is minimised.

## Constraints

1. **No administrator rights on the development machine** - withheld by policy on
   a managed corporate workstation. MSVC Build Tools installs machine-wide and so
   cannot be installed. Without a linker, *nothing* Rust-side runs locally: not
   `cargo build`, not `cargo test`, and not even `cargo check`, because
   `tauri-build`'s `build.rs` must be linked and executed before checking begins.
   Rust is installed (user profile, not on `PATH`) and is not the problem.
   This is a policy block, not a misconfiguration - do not retry the install or
   attempt elevation, and never request admin credentials.
2. **Verification happens in CI**, on a runner that has MSVC. See `useguide`.
3. **No corporate identifiers in committed files** - no hostnames, account names,
   or domains, in docs, comments, or commit messages. Describe findings
   generically.
4. **Commit at every phase boundary**, not once at the end.
5. **The break overlay must always be dismissible.** A frameless, transparent,
   always-on-top window that could not be escaped is indistinguishable from
   UI-spoofing malware. Escape, Skip and Snooze are a requirement, not polish.
