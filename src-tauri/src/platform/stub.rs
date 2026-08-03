//! Non-Windows fallback.
//!
//! Reporting "never idle, never fullscreen" makes the schedule behave exactly as
//! if both suppression rules were switched off, which is the right default until
//! a real macOS/Linux implementation lands. Nothing else has to change: the core
//! and the whole UI are already platform-agnostic.

pub fn idle_secs() -> u32 {
    0
}

pub fn fullscreen_active() -> bool {
    false
}
