mod aura;
mod commands;
mod config;
mod error;
mod state;
mod wmi;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve the app data directory for config persistence
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            // Initialize WMI connection and application state.
            // If WMI fails (e.g. no ASUS drivers), we still start the app
            // but commands will return errors.
            match AppState::new(app_data_dir) {
                Ok(state) => {
                    let _ = app.manage(state);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to initialize app state: {e}");
                    eprintln!("Fan control features will be unavailable.");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::fan::get_fan_speed,
            commands::fan::get_all_fan_speeds,
            commands::fan::get_thermal_profile,
            commands::fan::set_thermal_profile,
            commands::fan::get_default_fan_curve,
            commands::aura::aura_is_available,
            commands::aura::aura_get_device_info,
            commands::aura::aura_set_effect,
            commands::aura::aura_set_static_color,
            commands::aura::aura_turn_off,
            commands::aura::aura_set_direct_colors,
            commands::config::get_config,
            commands::config::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
