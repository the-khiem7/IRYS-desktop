---
baseline_schema: "2.0"
pack: "irys-desktop-app"
document: "introduction"
status: "active"
updated: "2026-08-03"
code_ref: "3dd940b"
---

# Irys — scope and current truth

## Scope

A Windows desktop companion that keeps the **30-30-30 rule** for the user: every
30 minutes, look ~30 feet (10 m) away for 30 seconds. It runs from the system
tray with no window visible, interrupts on schedule with an animated eye and a
countdown, and stays quiet when the user is already away or presenting.

Stack: **Tauri 2 + Vue 3 + TypeScript + Vite**. Rust owns all timing and state;
Vue is presentation only.

## Current truth

All eight planned phases are **implemented and committed** — 2,689 lines of Rust
across 11 files, 1,510 lines of Vue/TS across 13 files, 43 `#[test]` cases in the
core. Working tree clean at `3dd940b`.

Verification is **split**, and this is the single most important fact in this pack:

| Layer | State |
|---|---|
| Frontend types (`vue-tsc --noEmit`) | ✅ verified clean locally |
| Frontend build (`vite build`) | ✅ verified clean locally — 2 entries, break bundle 3.08 kB |
| Icon generation (`npm run icon`) | ✅ verified, output inspected visually |
| **All Rust** | ❌ **never compiled** |
| **CI workflow** | ❌ **never run** |
| **Either window's appearance** | ❌ **never seen** |

The Rust has been read closely — two real borrow-check errors in `core` were
found and fixed by inspection — but reading is not compiling. Treat every Rust
claim in this pack as *intended* behaviour, not observed behaviour.

## Target

A green CI run producing MSI and NSIS installers, then hands-on confirmation that
breaks fire on schedule while every window is minimised.

## Constraints

1. **No administrator rights on the development machine** — withheld by policy on
   a managed corporate workstation. MSVC Build Tools installs machine-wide and so
   cannot be installed. Without a linker, *nothing* Rust-side runs locally: not
   `cargo build`, not `cargo test`, and not even `cargo check`, because
   `tauri-build`'s `build.rs` must be linked and executed before checking begins.
   Rust is installed (user profile, not on `PATH`) and is not the problem.
   This is a policy block, not a misconfiguration — do not retry the install or
   attempt elevation, and never request admin credentials.
2. **Verification happens in CI**, on a runner that has MSVC. See `useguide`.
3. **No corporate identifiers in committed files** — no hostnames, account names,
   or domains, in docs, comments, or commit messages. Describe findings
   generically.
4. **Commit at every phase boundary**, not once at the end.
5. **The break overlay must always be dismissible.** A frameless, transparent,
   always-on-top window that could not be escaped is indistinguishable from
   UI-spoofing malware. Escape, Skip and Snooze are a requirement, not polish.
