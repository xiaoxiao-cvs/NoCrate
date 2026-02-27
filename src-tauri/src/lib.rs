mod aura;
mod commands;
mod config;
mod error;
mod state;
mod wmi;

use state::AppState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
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

            // ── System Tray ──────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("NoCrate — ASUS 主板控制")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Intercept close if "close_to_tray" is enabled
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    if state.config.get().close_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::fan::get_fan_speed,
            commands::fan::get_all_fan_speeds,
            commands::fan::get_thermal_profile,
            commands::fan::set_thermal_profile,
            commands::fan::get_default_fan_curve,
            commands::fan::get_wmi_backend,
            commands::fan::get_desktop_fan_policies,
            commands::fan::set_desktop_fan_policy,
            commands::fan::get_asushw_sensors,
            commands::aura::aura_is_available,
            commands::aura::aura_get_device_info,
            commands::aura::aura_set_effect,
            commands::aura::aura_set_static_color,
            commands::aura::aura_turn_off,
            commands::aura::aura_set_direct_colors,
            commands::config::get_config,
            commands::config::update_config,
            commands::system::is_admin,
            commands::system::restart_as_admin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
