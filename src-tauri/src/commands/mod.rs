pub mod aura;
pub mod fan;

/// Placeholder greet command for initial setup verification.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! NoCrate is running.")
}
