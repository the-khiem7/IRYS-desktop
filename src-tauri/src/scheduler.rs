//! The clock that drives the core, and the single path through which anything
//! mutates it.
//!
//! This lives in Rust rather than the webview on purpose: browser engines
//! throttle timers in hidden, minimised, or background windows, and Irys spends
//! almost all of its life with no window visible at all. A JS timer would drift
//! silently - which would break the one thing this app exists to do.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};
use tokio::time::{interval, MissedTickBehavior};

use crate::core::{Effect, Env, Machine};
use crate::{effects, platform};

pub struct AppState {
    machine: Mutex<Machine>,
}

impl AppState {
    pub fn new(machine: Machine) -> Self {
        Self {
            machine: Mutex::new(machine),
        }
    }

    /// Recovers from a poisoned lock instead of propagating the panic: one
    /// failed command must not silently stop the scheduler for the whole
    /// session, which the user would experience as breaks simply never coming.
    pub fn machine(&self) -> MutexGuard<'_, Machine> {
        self.machine.lock().unwrap_or_else(|err| err.into_inner())
    }
}

/// Apply an action to the machine, run whatever effects it returns, and push
/// fresh state to every window. Commands, the tray, and the break window all go
/// through here, so there is exactly one place where state changes.
pub fn dispatch(app: &AppHandle, action: impl FnOnce(&mut Machine) -> Vec<Effect>) {
    let state = app.state::<AppState>();

    // Scoped so the lock is released before effects run - showing a window can
    // re-enter Tauri, and holding the machine lock across that invites deadlock.
    let (fx, snapshot) = {
        let mut machine = state.machine();
        let fx = action(&mut machine);
        (fx, machine.snapshot())
    };

    effects::run(app, fx);
    effects::broadcast(app, snapshot);
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last = SystemTime::now();

        loop {
            ticker.tick().await;

            let now = SystemTime::now();
            // Wall clock rather than Instant: a monotonic clock can stop across
            // system sleep, and noticing that gap is exactly what lets the core
            // avoid firing a stale break the moment the lid opens. A backwards
            // clock step (NTP correction) falls back to a single second.
            let elapsed = now
                .duration_since(last)
                .map(|d| d.as_secs().min(u64::from(u32::MAX)) as u32)
                .unwrap_or(1);
            last = now;

            let env = Env {
                idle_secs: platform::idle_secs(),
                fullscreen_active: platform::fullscreen_active(),
            };

            dispatch(&app, |machine| machine.tick(env, elapsed));
        }
    });
}
