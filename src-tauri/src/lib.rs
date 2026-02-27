mod commands;
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
            // Initialize WMI connection and application state.
            // If WMI fails (e.g. no ASUS drivers), we still start the app
            // but commands will return errors.
            match AppState::new() {
                Ok(state) => {
                    let _ = app.manage(state);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to initialize WMI: {e}");
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
