//! Positioning and visibility for the two windows.
//!
//! Both windows are declared in `tauri.conf.json` and created hidden at startup
//! rather than on demand: creating a webview takes long enough to show a white
//! flash, which is a poor look for something that appears over your work.

use tauri::{AppHandle, LogicalSize, Manager, Monitor, PhysicalPosition, WebviewWindow};

use crate::core::BreakStyle;

pub const BREAK: &str = "break";
pub const SETTINGS: &str = "settings";

const TOAST_W: f64 = 380.0;
const TOAST_H: f64 = 150.0;
const TOAST_MARGIN: f64 = 24.0;
/// Tauri 2 exposes a monitor's full bounds but not its work area, so the toast
/// keeps clear of the bottom edge by a fixed allowance instead of measuring the
/// taskbar. Generous enough for a taskbar at its default height.
const TASKBAR_ALLOWANCE: f64 = 56.0;

pub fn show_break(app: &AppHandle, style: BreakStyle) {
    let Some(win) = app.get_webview_window(BREAK) else {
        return;
    };

    let monitor = target_monitor(app, &win);

    match style {
        BreakStyle::Overlay => place_overlay(&win, monitor.as_ref()),
        BreakStyle::Toast => place_toast(&win, monitor.as_ref()),
    }

    let _ = win.set_always_on_top(true);
    let _ = win.show();

    if matches!(style, BreakStyle::Overlay) {
        // Focus so Escape works without needing a click first. This only
        // requests window focus — it never captures raw input or blocks the OS,
        // and Skip/Snooze stay visible, so the overlay is always escapable.
        let _ = win.set_focus();
    }
}

pub fn hide_break(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(BREAK) {
        let _ = win.hide();
    }
}

pub fn show_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(SETTINGS) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// The monitor the break should appear on: the one holding the cursor, since
/// that is the screen the user is actually working on. Falls back to the primary
/// monitor, and then to leaving the window where it is.
fn target_monitor(app: &AppHandle, win: &WebviewWindow) -> Option<Monitor> {
    if let Ok(cursor) = win.cursor_position() {
        if let Ok(Some(monitor)) = win.monitor_from_point(cursor.x, cursor.y) {
            return Some(monitor);
        }
    }
    app.primary_monitor().ok().flatten()
}

fn place_overlay(win: &WebviewWindow, monitor: Option<&Monitor>) {
    if let Some(monitor) = monitor {
        // Cover the monitor by matching its bounds rather than asking for real
        // fullscreen: fullscreen mode on Windows fights with transparency and
        // with always-on-top.
        let _ = win.set_position(*monitor.position());
        let _ = win.set_size(*monitor.size());
    }
}

fn place_toast(win: &WebviewWindow, monitor: Option<&Monitor>) {
    let _ = win.set_size(LogicalSize::new(TOAST_W, TOAST_H));

    if let Some(monitor) = monitor {
        let scale = monitor.scale_factor();
        let origin = monitor.position();
        let size = monitor.size();

        let to_px = |logical: f64| (logical * scale).round() as i32;

        let x = origin.x + size.width as i32 - to_px(TOAST_W) - to_px(TOAST_MARGIN);
        let y = origin.y + size.height as i32
            - to_px(TOAST_H)
            - to_px(TOAST_MARGIN + TASKBAR_ALLOWANCE);

        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}
