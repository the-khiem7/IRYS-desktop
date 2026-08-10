// IRYS is a GUI application on Windows. Use the Windows subsystem in both
// debug and release builds so starting it never leaves an attached terminal
// panel that can terminate the app when closed.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    irys_lib::run()
}
