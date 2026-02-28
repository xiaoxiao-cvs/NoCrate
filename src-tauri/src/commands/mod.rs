pub mod aura;
pub mod config;
pub mod fan;
pub mod sensor;
pub mod system;

/// Placeholder greet command for initial setup verification.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! NoCrate is running.")
}
