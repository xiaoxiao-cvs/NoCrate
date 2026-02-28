/// Sensor monitoring commands exposed to the frontend via Tauri's invoke system.
///
/// Provides access to LibreHardwareMonitor (LHM) WMI sensor data.
/// All WMI operations are dispatched to the dedicated WMI thread.
use tauri::State;

use crate::state::AppState;
use crate::wmi::lhm::{self, LhmSensorSnapshot, LhmStatus};

/// Helper: execute a closure on the WMI thread.
fn with_wmi<F, T>(state: &State<'_, AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&crate::wmi::connection::WmiConnection) -> crate::error::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let wmi = state.wmi.as_ref().ok_or_else(|| {
        state
            .wmi_error
            .as_deref()
            .unwrap_or("WMI 未初始化")
            .to_string()
    })?;
    wmi.execute(f).map_err(Into::into)
}

/// Check if LibreHardwareMonitor is accessible.
#[tauri::command]
pub fn get_lhm_status(state: State<'_, AppState>) -> Result<LhmStatus, String> {
    with_wmi(&state, |conn| Ok(lhm::get_lhm_status(conn)))
}

/// Get all sensor readings grouped by type.
#[tauri::command]
pub fn get_lhm_sensors(state: State<'_, AppState>) -> Result<LhmSensorSnapshot, String> {
    with_wmi(&state, |conn| lhm::get_all_sensors(conn))
}
