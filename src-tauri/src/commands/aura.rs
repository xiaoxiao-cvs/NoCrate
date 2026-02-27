/// AURA ARGB control commands exposed to the frontend.
///
/// All operations acquire the `AppState::aura` Mutex and delegate to
/// `AuraController` methods. If no controller was discovered at
/// startup, commands return an error.
use tauri::State;

use crate::aura::controller::AuraDeviceInfo;
use crate::aura::protocol::{AuraEffect, AuraSpeed, RgbColor};
use crate::state::AppState;

/// Helper: borrow the AURA controller or return an error string.
fn with_aura<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&crate::aura::controller::AuraController) -> crate::error::Result<T>,
) -> Result<T, String> {
    let guard = state.aura.lock();
    let ctrl = guard
        .as_ref()
        .ok_or_else(|| "AURA controller not available".to_string())?;
    f(ctrl).map_err(Into::into)
}

/// Check whether an AURA controller is connected.
#[tauri::command]
pub fn aura_is_available(state: State<'_, AppState>) -> bool {
    state.aura.lock().is_some()
}

/// Get info about the connected AURA controller.
#[tauri::command]
pub fn aura_get_device_info(state: State<'_, AppState>) -> Result<AuraDeviceInfo, String> {
    let guard = state.aura.lock();
    let ctrl = guard
        .as_ref()
        .ok_or_else(|| "AURA controller not available".to_string())?;
    Ok(ctrl.info().clone())
}

/// Set an effect mode with colour and speed.
#[tauri::command]
pub fn aura_set_effect(
    state: State<'_, AppState>,
    effect: AuraEffect,
    color: RgbColor,
    speed: AuraSpeed,
) -> Result<(), String> {
    with_aura(&state, |ctrl| ctrl.set_effect(effect, color, speed))
}

/// Set a static solid colour on all LEDs.
#[tauri::command]
pub fn aura_set_static_color(state: State<'_, AppState>, color: RgbColor) -> Result<(), String> {
    with_aura(&state, |ctrl| ctrl.set_static_color(color))
}

/// Turn all LEDs off.
#[tauri::command]
pub fn aura_turn_off(state: State<'_, AppState>) -> Result<(), String> {
    with_aura(&state, |ctrl| ctrl.turn_off())
}

/// Set individual LED colours in direct mode.
#[tauri::command]
pub fn aura_set_direct_colors(
    state: State<'_, AppState>,
    colors: Vec<RgbColor>,
) -> Result<(), String> {
    with_aura(&state, |ctrl| ctrl.set_direct_colors(&colors))
}
