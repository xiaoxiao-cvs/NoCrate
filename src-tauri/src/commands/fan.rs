/// Fan control commands exposed to the frontend via Tauri's invoke system.
///
/// All WMI operations are dispatched to the dedicated WMI thread through
/// `AppState::wmi.execute()`, keeping the Tauri main / async runtime
/// unblocked.
use tauri::State;

use crate::state::AppState;
use crate::wmi::asus_mgmt::{
    self, AsusHWSensor, DesktopFanPolicy, FanCurve, FanInfo, FanTarget, ThermalProfile,
};

/// Helper: get a reference to the WmiThread or return an error string.
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

/// Get the current RPM for a specific fan header.
#[tauri::command]
pub fn get_fan_speed(state: State<'_, AppState>, target: FanTarget) -> Result<u32, String> {
    with_wmi(&state, move |conn| asus_mgmt::get_fan_speed(conn, target))
}

/// Get RPM readings for every detected fan header.
#[tauri::command]
pub fn get_all_fan_speeds(state: State<'_, AppState>) -> Result<Vec<FanInfo>, String> {
    with_wmi(&state, |conn| Ok(asus_mgmt::get_all_fan_speeds(conn)))
}

/// Get the currently active thermal profile.
#[tauri::command]
pub fn get_thermal_profile(state: State<'_, AppState>) -> Result<ThermalProfile, String> {
    with_wmi(&state, |conn| asus_mgmt::get_thermal_profile(conn))
}

/// Set the thermal profile (Standard / Performance / Silent).
#[tauri::command]
pub fn set_thermal_profile(
    state: State<'_, AppState>,
    profile: ThermalProfile,
) -> Result<(), String> {
    with_wmi(&state, move |conn| {
        asus_mgmt::set_thermal_profile(conn, profile)
    })
}

/// Get a sensible default fan curve for a given target.
///
/// Returns a local default — hardware curve read/write is not yet
/// implemented (requires byte-buffer WMI support).
#[tauri::command]
pub fn get_default_fan_curve(target: FanTarget) -> FanCurve {
    FanCurve::default_for(target)
}

// ---------------------------------------------------------------------------
// Desktop-specific commands
// ---------------------------------------------------------------------------

/// Returns `"desktop"`, `"laptop"`, `"asushw"`, or `"unavailable"` depending on the detected backend.
#[tauri::command]
pub fn get_wmi_backend(state: State<'_, AppState>) -> Result<String, String> {
    match &state.wmi {
        Some(wmi) => wmi
            .execute(|conn| Ok(conn.backend.backend_type().to_string()))
            .map_err(Into::into),
        None => Ok("unavailable".to_string()),
    }
}

/// Get fan policies for all present desktop fan headers.
///
/// Only meaningful when the backend is `desktop`.
#[tauri::command]
pub fn get_desktop_fan_policies(
    state: State<'_, AppState>,
) -> Result<Vec<DesktopFanPolicy>, String> {
    with_wmi(&state, |conn| {
        Ok(asus_mgmt::get_all_desktop_fan_policies(conn))
    })
}

/// Update a single desktop fan header's policy.
///
/// Only meaningful when the backend is `desktop`.
#[tauri::command]
pub fn set_desktop_fan_policy(
    state: State<'_, AppState>,
    policy: DesktopFanPolicy,
) -> Result<(), String> {
    with_wmi(&state, move |conn| {
        asus_mgmt::set_desktop_fan_policy(conn, &policy)
    })
}

// ---------------------------------------------------------------------------
// ASUSHW sensor commands
// ---------------------------------------------------------------------------

/// Get all detected ASUSHW sensors (temperatures + fan RPMs).
///
/// Only meaningful when the backend is `asushw`.
#[tauri::command]
pub fn get_asushw_sensors(state: State<'_, AppState>) -> Result<Vec<AsusHWSensor>, String> {
    with_wmi(&state, |conn| Ok(asus_mgmt::get_asushw_sensors(conn)))
}

/// 测试 asio_hw_fun* WMI 方法的可用性。
///
/// 返回一个包含 (方法名, 返回值/错误) 的诊断列表。
#[tauri::command]
pub fn test_asio_hw_fun(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    with_wmi(&state, |conn| {
        let results = conn.test_asio_hw_fun()?;
        Ok(results
            .into_iter()
            .map(|(label, r)| {
                let val = match r {
                    Ok(v) => format!("{v} (0x{v:02X})"),
                    Err(e) => format!("ERROR: {e}"),
                };
                (label, val)
            })
            .collect())
    })
}

// ---------------------------------------------------------------------------
// Super I/O 传感器命令
// ---------------------------------------------------------------------------

use crate::sio::chips::{SioSnapshot, SioStatus};

/// 获取 Super I/O 传感器快照（风扇 RPM + 温度）
#[tauri::command]
pub fn get_sio_sensors(state: State<'_, AppState>) -> Result<SioSnapshot, String> {
    let sio = state.sio.as_ref().ok_or_else(|| {
        state
            .sio_error
            .as_deref()
            .unwrap_or("SIO 未初始化")
            .to_string()
    })?;
    sio.read_all().map_err(|e| e.to_string())
}

/// 获取 Super I/O 状态信息
#[tauri::command]
pub fn get_sio_status(state: State<'_, AppState>) -> SioStatus {
    match &state.sio {
        Some(sio) => sio.status(),
        None => {
            crate::sio::unavailable_status(state.sio_error.as_deref().unwrap_or("SIO 未初始化"))
        }
    }
}
