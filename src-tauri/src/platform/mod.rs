//! OS probes for the two suppression rules.
//!
//! The scheduling core is pure, so it cannot ask the OS anything. These two
//! functions are the whole boundary - they are called once per tick and their
//! results are handed to the core as [`crate::core::Env`].

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{fullscreen_active, idle_secs};

#[cfg(not(windows))]
mod stub;
#[cfg(not(windows))]
pub use stub::{fullscreen_active, idle_secs};
