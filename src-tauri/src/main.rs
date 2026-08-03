// Keeps a console window from appearing alongside the app on Windows release
// builds. Debug builds keep it so `eprintln!` diagnostics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    irys_lib::run()
}
