/// ASUS WMI interface wrapper.
///
/// Provides typed access to the ASUS motherboard management interface
/// exposed via WMI. Supports two backends:
///
/// - **Laptop** (`ASUSATKWMI_WMNB`): Methods `DSTS` / `DEVS`
/// - **Desktop** (`ASUSManagement`): Methods `device_status` / `device_ctrl`
///
/// The low-level `dsts` / `devs` calls are routed through
/// [`WmiConnection::dsts`] / [`WmiConnection::devs`] which handle
/// the backend-specific method names and parameter mapping.
///
/// Device IDs sourced from the Linux kernel `asus-wmi` driver
/// (`include/linux/platform_data/x86/asus-wmi.h`) and the Armoury Crate
/// / ASUS WMI desktop driver.
use serde::{Deserialize, Serialize};

use crate::error::{NoCrateError, Result};
use crate::wmi::connection::{AsusWmiBackend, WmiConnection, WmiParam};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// ASUS WMI Device IDs — shared between laptop and desktop backends.
///
/// Reference: Linux kernel `include/linux/platform_data/x86/asus-wmi.h`
///            and g-helper `AsusACPI.cs`.
pub mod device_id {
    /// CPU fan tachometer — RPM (read-only).
    pub const CPU_FAN_SPEED: u32 = 0x0011_0013;

    /// GPU / chassis-fan-1 tachometer — RPM (read-only).
    pub const GPU_FAN_SPEED: u32 = 0x0011_0014;

    /// Middle / chassis-fan-2 tachometer — RPM (read-only).
    pub const MID_FAN_SPEED: u32 = 0x0011_0031;

    /// Thermal control master switch.
    #[allow(dead_code)]
    pub const THERMAL_CTRL: u32 = 0x0011_0011;

    /// Fan control mode.
    #[allow(dead_code)]
    pub const FAN_CTRL: u32 = 0x0011_0012;

    /// Throttle thermal policy — the overall "profile"
    /// (Standard 0 / Performance 1 / Silent 2).
    pub const THROTTLE_THERMAL_POLICY: u32 = 0x0012_0075;

    /// CPU fan-curve data (read/write).
    #[allow(dead_code)]
    pub const CPU_FAN_CURVE: u32 = 0x0011_0024;

    /// GPU fan-curve data (read/write).
    #[allow(dead_code)]
    pub const GPU_FAN_CURVE: u32 = 0x0011_0025;

    /// Middle fan-curve data (read/write).
    #[allow(dead_code)]
    pub const MID_FAN_CURVE: u32 = 0x0011_0032;

    /// Firmware version query.
    #[allow(dead_code)]
    pub const CMD_FIRMWARE: u32 = 0x0002_0013;
}

// ---------------------------------------------------------------------------
// Low-level WMI helpers
// ---------------------------------------------------------------------------

/// Read a device status value — routed through the detected backend.
///
/// Returns the raw status u32.
pub fn dsts(conn: &WmiConnection, device_id: u32) -> Result<u32> {
    conn.dsts(device_id)
}

/// Write a device control value — routed through the detected backend.
///
/// Returns the raw result status.
pub fn devs(conn: &WmiConnection, device_id: u32, control: u32) -> Result<u32> {
    conn.devs(device_id, control)
}

// ---------------------------------------------------------------------------
// Typed enums
// ---------------------------------------------------------------------------

/// Identifies which fan header to query or control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FanTarget {
    /// CPU fan header.
    Cpu,
    /// GPU / first chassis fan header.
    Gpu,
    /// Middle / second chassis fan header.
    Mid,
}

impl FanTarget {
    /// All known fan targets, handy for iteration.
    pub const ALL: [Self; 3] = [Self::Cpu, Self::Gpu, Self::Mid];

    /// DSTS device ID for reading this fan's RPM.
    #[must_use]
    pub const fn speed_device_id(self) -> u32 {
        match self {
            Self::Cpu => device_id::CPU_FAN_SPEED,
            Self::Gpu => device_id::GPU_FAN_SPEED,
            Self::Mid => device_id::MID_FAN_SPEED,
        }
    }

    /// Device ID for this fan's curve data.
    #[must_use]
    #[allow(dead_code)]
    pub const fn curve_device_id(self) -> u32 {
        match self {
            Self::Cpu => device_id::CPU_FAN_CURVE,
            Self::Gpu => device_id::GPU_FAN_CURVE,
            Self::Mid => device_id::MID_FAN_CURVE,
        }
    }
}

/// ASUS thermal-profile presets.
///
/// These correspond to the three profiles available in ASUS BIOS and
/// Armoury Crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalProfile {
    /// Default balanced mode.
    Standard,
    /// Maximum performance / fan boost.
    Performance,
    /// Quieter fans at the cost of thermals.
    Silent,
}

impl ThermalProfile {
    /// All profiles, handy for iteration / UI.
    #[allow(dead_code)]
    pub const ALL: [Self; 3] = [Self::Standard, Self::Performance, Self::Silent];

    /// Convert to the raw DEVS control value.
    #[must_use]
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Standard => 0,
            Self::Performance => 1,
            Self::Silent => 2,
        }
    }

    /// Parse from a raw DSTS status value.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value & 0xFF {
            0 => Some(Self::Standard),
            1 => Some(Self::Performance),
            2 => Some(Self::Silent),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Fan speed
// ---------------------------------------------------------------------------

/// Read the current fan speed in RPM for the given target.
///
/// Returns `0` if the fan header is unconnected or unsupported.
pub fn get_fan_speed(conn: &WmiConnection, target: FanTarget) -> Result<u32> {
    let raw = dsts(conn, target.speed_device_id())?;
    // Lower 16 bits hold the RPM; upper bits may carry status flags.
    Ok(raw & 0xFFFF)
}

/// Snapshot of a single fan header.
#[derive(Debug, Clone, Serialize)]
pub struct FanInfo {
    pub target: FanTarget,
    pub rpm: u32,
}

/// Read speeds for all known fan headers.
///
/// Headers that fail to respond (e.g. not present on a given board) are
/// silently skipped.
pub fn get_all_fan_speeds(conn: &WmiConnection) -> Vec<FanInfo> {
    FanTarget::ALL
        .iter()
        .filter_map(|&target| {
            get_fan_speed(conn, target)
                .ok()
                .map(|rpm| FanInfo { target, rpm })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Thermal profile
// ---------------------------------------------------------------------------

/// Read the currently active thermal profile.
pub fn get_thermal_profile(conn: &WmiConnection) -> Result<ThermalProfile> {
    let raw = dsts(conn, device_id::THROTTLE_THERMAL_POLICY)?;
    ThermalProfile::from_raw(raw)
        .ok_or_else(|| NoCrateError::Wmi(format!("Unknown thermal-profile raw value: 0x{raw:08X}")))
}

/// Set the active thermal profile.
pub fn set_thermal_profile(conn: &WmiConnection, profile: ThermalProfile) -> Result<()> {
    let _status = devs(conn, device_id::THROTTLE_THERMAL_POLICY, profile.to_raw())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Fan curve
// ---------------------------------------------------------------------------

/// Number of control points in an ASUS fan curve.
pub const FAN_CURVE_POINTS: usize = 8;

/// A single temperature → duty-cycle mapping point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FanCurvePoint {
    /// Temperature threshold in °C (0–100).
    pub temp_c: u8,
    /// Fan duty-cycle percentage (0–100).
    pub duty_pct: u8,
}

/// A complete fan curve with [`FAN_CURVE_POINTS`] pairs.
///
/// Points must be sorted by ascending temperature. The fan controller
/// linearly interpolates between adjacent points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanCurve {
    pub target: FanTarget,
    pub points: [FanCurvePoint; FAN_CURVE_POINTS],
}

impl FanCurve {
    /// A sensible default curve: gentle ramp from 30 % at 30 °C to 100 % at 90 °C.
    #[must_use]
    pub fn default_for(target: FanTarget) -> Self {
        Self {
            target,
            points: [
                FanCurvePoint {
                    temp_c: 30,
                    duty_pct: 30,
                },
                FanCurvePoint {
                    temp_c: 40,
                    duty_pct: 35,
                },
                FanCurvePoint {
                    temp_c: 50,
                    duty_pct: 45,
                },
                FanCurvePoint {
                    temp_c: 60,
                    duty_pct: 55,
                },
                FanCurvePoint {
                    temp_c: 70,
                    duty_pct: 65,
                },
                FanCurvePoint {
                    temp_c: 75,
                    duty_pct: 75,
                },
                FanCurvePoint {
                    temp_c: 80,
                    duty_pct: 85,
                },
                FanCurvePoint {
                    temp_c: 90,
                    duty_pct: 100,
                },
            ],
        }
    }
}

// NOTE: get_fan_curve / set_fan_curve via WMI are not yet implemented.
//
// The ASUS fan-curve device IDs return/accept 16-byte buffers
// (8 temperatures + 8 duty percentages), which requires extending
// `WmiConnection` with byte-array VARIANT extraction.
//
// Until then fan curves are managed locally in the frontend and
// persisted to the config file. The thermal-profile toggle (Standard /
// Performance / Silent) is the primary hardware control path.

// ===========================================================================
// Desktop motherboard support (ASUSManagement WMI class)
// ===========================================================================

/// Maximum number of fan headers to probe on a desktop motherboard.
///
/// ASUS desktop boards typically expose FanType 0–3 via `GetFanPolicy`.
/// Headers returning `ErrorCode != 0` are considered absent.
const DESKTOP_MAX_FAN_HEADERS: u8 = 8;

/// Fan control mode on desktop boards.
///
/// Returned by `GetFanPolicy` as the `Mode` string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DesktopFanMode {
    /// Voltage-controlled (DC).
    Pwm,
    /// Automatic control.
    Auto,
}

impl DesktopFanMode {
    fn from_wmi(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "PWM" => Self::Pwm,
            _ => Self::Auto,
        }
    }

    fn to_wmi(&self) -> &str {
        match self {
            Self::Pwm => "PWM",
            Self::Auto => "AUTO",
        }
    }
}

/// Fan policy profile on desktop boards.
///
/// Returned by `GetFanPolicy` as the `Profile` string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DesktopFanProfile {
    /// User-defined manual curve.
    Manual,
    /// Default automatic curve.
    Standard,
}

impl DesktopFanProfile {
    #[allow(dead_code)]
    fn from_wmi(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "MANUAL" => Self::Manual,
            _ => Self::Standard,
        }
    }

    fn to_wmi(&self) -> &str {
        match self {
            Self::Manual => "MANUAL",
            Self::Standard => "STANDARD",
        }
    }
}

/// Complete fan policy for a single desktop fan header.
///
/// Read via `ASUSManagement.GetFanPolicy(FanType)` and written back
/// via `ASUSManagement.SetFanPolicy(...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopFanPolicy {
    /// Fan header index (0-based: 0 = CPU, 1–3 = chassis).
    pub fan_type: u8,
    /// Control mode: PWM (DC) or AUTO.
    pub mode: DesktopFanMode,
    /// Curve profile: MANUAL or STANDARD.
    pub profile: DesktopFanProfile,
    /// Temperature source (e.g. "CPU").
    pub source: String,
    /// Minimum RPM threshold.
    pub low_limit: u32,
}

/// Friendly display names for desktop fan headers.
#[allow(dead_code)]
const DESKTOP_FAN_NAMES: [&str; 8] = [
    "CPU Fan",
    "Chassis Fan 1",
    "Chassis Fan 2",
    "Chassis Fan 3",
    "Chassis Fan 4",
    "Chassis Fan 5",
    "Chassis Fan 6",
    "Chassis Fan 7",
];

/// Get the display name for a desktop fan header index.
#[allow(dead_code)]
pub fn desktop_fan_name(fan_type: u8) -> &'static str {
    DESKTOP_FAN_NAMES
        .get(fan_type as usize)
        .unwrap_or(&"Unknown Fan")
}

/// Read the fan policy for a single desktop fan header.
///
/// Returns `None` if the header does not exist (ErrorCode != 0).
pub fn get_desktop_fan_policy(
    conn: &WmiConnection,
    fan_type: u8,
) -> Result<Option<DesktopFanPolicy>> {
    let instance_path = match &conn.backend {
        AsusWmiBackend::Desktop { instance_path } => instance_path.clone(),
        _ => {
            return Err(NoCrateError::Wmi(
                "GetFanPolicy is only available on desktop backends".into(),
            ));
        }
    };

    let out = conn.exec_method_v2(
        &instance_path,
        "GetFanPolicy",
        &[("FanType", WmiParam::U8(fan_type))],
    )?;

    let error_code = WmiConnection::get_property_u32(&out, "ErrorCode")?;
    if error_code != 0 {
        return Ok(None); // Fan header not present
    }

    let mode = WmiConnection::get_property_string(&out, "Mode")?;
    let profile = WmiConnection::get_property_string(&out, "Profile")?;
    let source = WmiConnection::get_property_string(&out, "Source")?;
    let low_limit = WmiConnection::get_property_u32(&out, "LowLimit")?;

    Ok(Some(DesktopFanPolicy {
        fan_type,
        mode: DesktopFanMode::from_wmi(&mode),
        profile: DesktopFanProfile::from_wmi(&profile),
        source,
        low_limit,
    }))
}

/// Read fan policies for all present desktop fan headers.
///
/// Probes FanType 0 through [`DESKTOP_MAX_FAN_HEADERS`] and returns
/// only headers that respond without error.
pub fn get_all_desktop_fan_policies(conn: &WmiConnection) -> Vec<DesktopFanPolicy> {
    (0..DESKTOP_MAX_FAN_HEADERS)
        .filter_map(|ft| get_desktop_fan_policy(conn, ft).ok().flatten())
        .collect()
}

/// Write a fan policy to a desktop fan header.
///
/// # Errors
///
/// Returns an error if the WMI call fails or the backend is not desktop.
pub fn set_desktop_fan_policy(conn: &WmiConnection, policy: &DesktopFanPolicy) -> Result<()> {
    let instance_path = match &conn.backend {
        AsusWmiBackend::Desktop { instance_path } => instance_path.clone(),
        _ => {
            return Err(NoCrateError::Wmi(
                "SetFanPolicy is only available on desktop backends".into(),
            ));
        }
    };

    let out = conn.exec_method_v2(
        &instance_path,
        "SetFanPolicy",
        &[
            ("FanType", WmiParam::U8(policy.fan_type)),
            ("LowLimit", WmiParam::U32(policy.low_limit)),
            ("Mode", WmiParam::Str(policy.mode.to_wmi())),
            ("Profile", WmiParam::Str(policy.profile.to_wmi())),
            ("Source", WmiParam::Str(&policy.source)),
        ],
    )?;

    let error_code = WmiConnection::get_property_u32(&out, "ErrorCode")?;
    if error_code != 0 {
        return Err(NoCrateError::Wmi(format!(
            "SetFanPolicy failed for FanType {} with ErrorCode {error_code}",
            policy.fan_type,
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Backend detection helper
// ---------------------------------------------------------------------------

/// Returns `true` if the current WMI connection uses the desktop backend.
#[allow(dead_code)]
pub fn is_desktop_backend(conn: &WmiConnection) -> bool {
    matches!(conn.backend, AsusWmiBackend::Desktop { .. })
}

// ---------------------------------------------------------------------------
// ASUSHW Sensor types & helpers
// ---------------------------------------------------------------------------

/// A single sensor reading from the ASUSHW backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsusHWSensor {
    /// Zero-based sensor index.
    pub index: u32,
    /// Human-readable name (e.g. "CPU Temperature", "CPU Fan").
    pub name: String,
    /// `"temperature"` (°C) or `"fan"` (RPM).
    pub sensor_type: String,
    /// Current value (°C for temps, RPM for fans).
    pub value: f32,
    /// Internal source group ID (for `sensor_update_buffer`).
    pub source: u32,
    /// Internal data-type flag (3 = micro-units).
    pub data_type: u32,
}

/// Discover all sensors from the ASUSHW backend.
///
/// Enumerates sensors via `sensor_get_number` / `sensor_get_info`,
/// updates buffers, and reads current values.
pub fn get_asushw_sensors(conn: &WmiConnection) -> Vec<AsusHWSensor> {
    let count = match conn.asushw_sensor_count() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ASUSHW] sensor_get_number failed: {e}");
            return vec![];
        }
    };
    eprintln!("[ASUSHW] Found {count} sensors");

    // Collect sensor metadata
    let mut sensors = Vec::new();
    let mut sources = std::collections::HashSet::new();

    for i in 0..count {
        match conn.asushw_sensor_info(i) {
            Ok((source, stype, data_type, name)) => {
                let type_str = match stype {
                    1 => "temperature",
                    2 => "fan",
                    _ => continue, // skip unknown types
                };
                let _ = sources.insert(source);
                sensors.push(AsusHWSensor {
                    index: i,
                    name,
                    sensor_type: type_str.to_string(),
                    value: 0.0,
                    source,
                    data_type,
                });
            }
            Err(e) => eprintln!("[ASUSHW] sensor_get_info({i}) failed: {e}"),
        }
    }

    // Update all source buffers
    for &src in &sources {
        if let Err(e) = conn.asushw_update_buffer(src) {
            eprintln!("[ASUSHW] sensor_update_buffer({src}) failed: {e}");
        }
    }

    // Read current values
    for sensor in &mut sensors {
        match conn.asushw_sensor_value(sensor.index) {
            Ok(raw) => {
                sensor.value = if sensor.data_type == 3 {
                    // Micro-units (e.g. 60_000_000 → 60.0°C)
                    raw as f32 / 1_000_000.0
                } else {
                    raw as f32
                };
            }
            Err(e) => {
                eprintln!("[ASUSHW] sensor_get_value({}) failed: {e}", sensor.index);
            }
        }
    }

    sensors
}
