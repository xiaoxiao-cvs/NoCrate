/// Fan control commands exposed to the frontend via Tauri's invoke system.
///
/// All WMI operations are dispatched to the dedicated WMI thread through
/// `AppState::wmi.execute()`, keeping the Tauri main / async runtime
/// unblocked.
use tauri::State;

use crate::state::AppState;
use crate::wmi::asus_mgmt::{self, FanCurve, FanInfo, FanTarget, ThermalProfile};

/// Get the current RPM for a specific fan header.
#[tauri::command]
pub fn get_fan_speed(state: State<'_, AppState>, target: FanTarget) -> Result<u32, String> {
    state
        .wmi
        .execute(move |conn| asus_mgmt::get_fan_speed(conn, target))
        .map_err(Into::into)
}

/// Get RPM readings for every detected fan header.
#[tauri::command]
pub fn get_all_fan_speeds(state: State<'_, AppState>) -> Result<Vec<FanInfo>, String> {
    state
        .wmi
        .execute(|conn| Ok(asus_mgmt::get_all_fan_speeds(conn)))
        .map_err(Into::into)
}

/// Get the currently active thermal profile.
#[tauri::command]
pub fn get_thermal_profile(state: State<'_, AppState>) -> Result<ThermalProfile, String> {
    state
        .wmi
        .execute(|conn| asus_mgmt::get_thermal_profile(conn))
        .map_err(Into::into)
}

/// Set the thermal profile (Standard / Performance / Silent).
#[tauri::command]
pub fn set_thermal_profile(
    state: State<'_, AppState>,
    profile: ThermalProfile,
) -> Result<(), String> {
    state
        .wmi
        .execute(move |conn| asus_mgmt::set_thermal_profile(conn, profile))
        .map_err(Into::into)
}

/// Get a sensible default fan curve for a given target.
///
/// Returns a local default — hardware curve read/write is not yet
/// implemented (requires byte-buffer WMI support).
#[tauri::command]
pub fn get_default_fan_curve(target: FanTarget) -> FanCurve {
    FanCurve::default_for(target)
}
