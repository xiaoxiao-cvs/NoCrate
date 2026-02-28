use tauri::State;

use crate::config::AppConfig;
use crate::state::AppState;

/// Get the full application configuration.
#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.get())
}

/// Update one or more config fields. Only provided (Some) fields are applied.
#[tauri::command]
pub fn update_config(
    state: State<'_, AppState>,
    theme: Option<String>,
    close_to_tray: Option<bool>,
    auto_start: Option<bool>,
    fan_poll_interval_ms: Option<u64>,
    last_thermal_profile: Option<u8>,
    last_aura_effect: Option<String>,
    last_aura_color: Option<String>,
    last_aura_speed: Option<String>,
    temp_alert_enabled: Option<bool>,
    temp_alert_threshold: Option<u8>,
) -> Result<AppConfig, String> {
    state
        .config
        .update(|cfg| {
            if let Some(v) = theme {
                cfg.theme = v;
            }
            if let Some(v) = close_to_tray {
                cfg.close_to_tray = v;
            }
            if let Some(v) = auto_start {
                cfg.auto_start = v;
            }
            if let Some(v) = fan_poll_interval_ms {
                cfg.fan_poll_interval_ms = v;
            }
            if let Some(v) = last_thermal_profile {
                cfg.last_thermal_profile = v;
            }
            if let Some(v) = last_aura_effect {
                cfg.last_aura_effect = v;
            }
            if let Some(v) = last_aura_color {
                cfg.last_aura_color = v;
            }
            if let Some(v) = last_aura_speed {
                cfg.last_aura_speed = v;
            }
            if let Some(v) = temp_alert_enabled {
                cfg.temp_alert_enabled = v;
            }
            if let Some(v) = temp_alert_threshold {
                cfg.temp_alert_threshold = v;
            }
        })
        .map_err(|e| e.to_string())
}
