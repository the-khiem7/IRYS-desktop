//! Windows implementations of the two OS probes.
//!
//! All `unsafe` in this project is confined to this file. Each block is a plain
//! query with no ownership transfer: every out-parameter is a fully initialised
//! local that Rust owns, and nothing here allocates or frees.

use std::mem::size_of;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

/// Seconds since the last keyboard or mouse input, system-wide.
///
/// Returns 0 if the OS declines to answer - the safe direction, since it means
/// "assume the user is here" and the reminder still fires.
pub fn idle_secs() -> u32 {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };

    // SAFETY: `info` is a fully initialised LASTINPUTINFO with a correct
    // `cbSize`; the call only writes `dwTime` and does not retain the pointer.
    let ok = unsafe { GetLastInputInfo(&mut info) }.as_bool();
    if !ok {
        return 0;
    }

    // SAFETY: no arguments, no out-params - cannot fail.
    let now = unsafe { GetTickCount64() };

    // `dwTime` is a 32-bit tick count that wraps roughly every 49 days, while
    // GetTickCount64 does not. Comparing in the low 32 bits with a wrapping
    // subtraction keeps this correct across a wrap instead of reporting a
    // nonsense multi-week idle time.
    let last = u64::from(info.dwTime);
    let now_low = now & 0xFFFF_FFFF;
    let delta_ms = now_low.wrapping_sub(last) & 0xFFFF_FFFF;

    (delta_ms / 1000) as u32
}

/// Whether the foreground window covers its entire monitor - a video call,
/// presentation, or game that we should not cover with an overlay.
///
/// Compares against the full monitor rect rather than the work area, so a
/// normally maximised window (which stops at the taskbar) is correctly *not*
/// treated as fullscreen.
pub fn fullscreen_active() -> bool {
    // SAFETY: every call below is a read-only query. `rect` and `mi` are
    // initialised locals owned by Rust, `mi.cbSize` is set as the API requires,
    // and each handle is checked before use. No pointer outlives this scope.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND::default() {
            // Nothing focused, e.g. the desktop or a lock screen.
            return false;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return false;
        }

        let mut mi = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut mi).as_bool() {
            return false;
        }

        let m = mi.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}
