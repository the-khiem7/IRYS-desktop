//! Carries out what the core decided.
//!
//! The core returns [`Effect`] values describing *what* should happen; this
//! module is the only place that knows *how*. Keeping the two apart is what
//! lets the whole schedule be tested without a window.

// `tray_by_id` is an inherent method on AppHandle, so no Manager import here;
// `emit` comes from Emitter and `notification()` from NotificationExt.
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::core::{Effect, PhaseKind, Snapshot};
use crate::{tray, windows_mgr};

/// Emitted every second to any window that is listening.
pub const EVENT_TICK: &str = "irys://tick";

pub fn run(app: &AppHandle, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::ShowBreak(style) => windows_mgr::show_break(app, style),
            Effect::HideBreak => windows_mgr::hide_break(app),
            Effect::Prewarn => prewarn(app),
            Effect::Tray(label) => set_tooltip(app, &label),
        }
    }
}

/// Push current state to the UI. Failures are ignored by design: no window
/// needs to be open for the schedule to keep running.
pub fn broadcast(app: &AppHandle, snapshot: Snapshot) {
    let _ = app.emit(EVENT_TICK, snapshot);
    tray::sync_pause_label(app, snapshot.phase == PhaseKind::Paused);
}

fn prewarn(app: &AppHandle) {
    let _ = app
        .notification()
        .builder()
        .title("IRYS")
        .body("Eye break in 30 seconds - line up something far away to look at.")
        .show();
}

fn set_tooltip(app: &AppHandle, label: &str) {
    if let Some(tray) = app.tray_by_id(tray::TRAY_ID) {
        let _ = tray.set_tooltip(Some(label));
    }
}
