//! The IPC surface. Every command is a thin wrapper over
//! [`scheduler::dispatch`], so the frontend can never mutate state by any other
//! route and never has to reimplement scheduling rules.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::core::{Settings, Snapshot};
use crate::scheduler::{self, AppState};
use crate::{settings_store, windows_mgr};

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> Snapshot {
    state.machine().snapshot()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.machine().settings()
}

/// Apply, persist, and return what was actually stored - which may differ from
/// what was sent, because the core clamps out-of-range values. The settings UI
/// re-renders from the return value so the user sees the real state.
#[tauri::command]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    scheduler::dispatch(&app, |machine| machine.apply(settings));

    let stored = app.state::<AppState>().machine().settings();
    settings_store::save(&app, &stored)?;
    sync_autostart(&app, stored.autostart);

    Ok(stored)
}

#[tauri::command]
pub fn toggle_pause(app: AppHandle) {
    scheduler::dispatch(&app, |machine| machine.toggle_pause());
}

#[tauri::command]
pub fn break_now(app: AppHandle) {
    scheduler::dispatch(&app, |machine| machine.break_now());
}

#[tauri::command]
pub fn skip_break(app: AppHandle) {
    scheduler::dispatch(&app, |machine| machine.skip());
}

#[tauri::command]
pub fn snooze_break(app: AppHandle) {
    scheduler::dispatch(&app, |machine| machine.snooze());
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    windows_mgr::show_settings(&app);
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

/// Keep the login-item registration in step with the setting.
///
/// This writes a per-user `HKCU\...\Run` entry - no elevation, visible in Task
/// Manager's Startup tab, and removed again the moment the user turns it off.
/// A failure here is reported but never blocks saving the rest of the settings.
fn sync_autostart(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();

    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };

    if let Err(err) = result {
        eprintln!("irys: could not update the autostart entry: {err}");
    }
}
