//! Settings persistence.
//!
//! One JSON file in the app config directory holding user preferences and
//! nothing else - no credentials, no tokens, no telemetry. A missing,
//! unreadable, or hand-mangled file falls back to defaults rather than failing
//! to start, and anything that does load is clamped by
//! [`Settings::sanitized`](crate::core::Settings::sanitized).

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::core::Settings;

const STORE_FILE: &str = "settings.json";
const KEY: &str = "settings";

pub fn load(app: &AppHandle) -> Settings {
    let Ok(store) = app.store(STORE_FILE) else {
        return Settings::default();
    };

    store
        .get(KEY)
        .and_then(|value| serde_json::from_value::<Settings>(value).ok())
        .map(Settings::sanitized)
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|err| format!("could not open the settings store: {err}"))?;

    let value = serde_json::to_value(settings)
        .map_err(|err| format!("could not serialise settings: {err}"))?;

    store.set(KEY, value);
    store
        .save()
        .map_err(|err| format!("could not write settings: {err}"))
}
