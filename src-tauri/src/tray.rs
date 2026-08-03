//! The tray icon: Irys's only permanent presence on screen.

use std::sync::Mutex;

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::{scheduler, windows_mgr};

pub const TRAY_ID: &str = "irys-tray";

/// Handle to the one menu item whose label changes at runtime.
struct PauseItem {
    item: MenuItem<Wry>,
    paused: Mutex<bool>,
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let break_now = MenuItem::with_id(app, "break_now", "Take a break now", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Pause", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Irys", true, None::<&str>)?;
    let sep_a = PredefinedMenuItem::separator(app)?;
    let sep_b = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[&break_now, &toggle, &sep_a, &settings, &sep_b, &quit],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Irys")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;

    app.manage(PauseItem {
        item: toggle,
        paused: Mutex::new(false),
    });

    Ok(())
}

/// Relabel Pause/Resume, but only when the state actually flips.
///
/// The tooltip is refreshed every second; the menu deliberately is not —
/// rewriting menu items at 1 Hz costs more than it is worth and can flicker
/// while the menu is open.
pub fn sync_pause_label(app: &AppHandle, paused: bool) {
    let Some(state) = app.try_state::<PauseItem>() else {
        return;
    };

    let mut last = state
        .paused
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    if *last == paused {
        return;
    }
    *last = paused;

    let _ = state.item.set_text(if paused { "Resume" } else { "Pause" });
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "break_now" => scheduler::dispatch(app, |machine| machine.break_now()),
        "toggle" => scheduler::dispatch(app, |machine| machine.toggle_pause()),
        "settings" => windows_mgr::show_settings(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    // Left click opens settings; right click is left to the menu.
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        windows_mgr::show_settings(tray.app_handle());
    }
}
