//! Irys — a 30-30-30 rule companion that lives in the system tray.
//!
//! Layering, outermost to innermost:
//!
//! - `commands` / `tray`      — ways the user asks for something
//! - `scheduler`              — the 1 Hz clock and the one mutation path
//! - `core`                   — the pure state machine (all the real rules)
//! - `effects` / `windows_mgr`— carrying out what the core decided
//! - `platform`              — the only place that talks to the OS

mod commands;
mod core;
mod effects;
mod platform;
mod scheduler;
mod settings_store;
mod tray;
mod windows_mgr;

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::core::Machine;
use crate::scheduler::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_settings,
            commands::set_settings,
            commands::toggle_pause,
            commands::break_now,
            commands::skip_break,
            commands::snooze_break,
            commands::open_settings,
            commands::quit,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let settings = settings_store::load(&handle);
            app.manage(AppState::new(Machine::new(settings)));

            tray::build(&handle)?;
            scheduler::spawn(handle);

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing a window never quits Irys. The settings window hides back
            // to the tray; closing the break window is treated as a Skip, so the
            // schedule restarts cleanly instead of leaving a phantom break.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();

                if window.label() == windows_mgr::BREAK {
                    scheduler::dispatch(window.app_handle(), |machine| machine.skip());
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to start Irys")
        .run(|_app, event| {
            // Having no window on screen is Irys's normal state, so the default
            // "last window closed means exit" behaviour would kill the app the
            // first time the user closes settings. An explicit `app.exit(code)`
            // carries a code and is let through.
            if let RunEvent::ExitRequested { code: None, api, .. } = event {
                api.prevent_exit();
            }
        });
}
