/// ASUS ATK WMI interface wrapper.
///
/// Provides typed access to the ASUS motherboard management interface
/// exposed via WMI class `ASUSATKWMI_WMNB`:
///
/// - **DSTS** (Device STatus): Read device values (fan speed, thermal profile, …)
/// - **DEVS** (DEVice Set): Write device values (thermal profile, fan control, …)
///
/// Device IDs sourced from the Linux kernel `asus-wmi` driver
/// (`include/linux/platform_data/x86/asus-wmi.h`).
///
/// # Hardware Compatibility
///
/// Requires an ASUS motherboard with the ATK0110 ACPI device and the
/// corresponding WMI driver (typically installed alongside ASUS chipset /
/// system-utility drivers).
use serde::{Deserialize, Serialize};

use crate::error::{NoCrateError, Result};
use crate::wmi::connection::WmiConnection;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// WMI instance path for the ASUS ATK interface.
///
/// This assumes the standard ATK0110 ACPI device. If a board uses a
/// different instance name the constant will need adjustment.
const ATK_OBJECT_PATH: &str = "ASUSATKWMI_WMNB.InstanceName='ACPI\\\\ATK0110\\\\0_0'";

/// ASUS WMI Device IDs for use with the `DSTS` and `DEVS` methods.
///
/// Reference: Linux kernel `include/linux/platform_data/x86/asus-wmi.h`
pub mod device_id {
    /// CPU fan tachometer — RPM (read-only via DSTS).
    pub const CPU_FAN_SPEED: u32 = 0x0011_0013;

    /// GPU / chassis-fan-1 tachometer — RPM (read-only via DSTS).
    pub const GPU_FAN_SPEED: u32 = 0x0011_0014;

    /// Middle / chassis-fan-2 tachometer — RPM (read-only via DSTS).
    pub const MID_FAN_SPEED: u32 = 0x0011_0031;

    /// Thermal control master switch (DEVS).
    pub const THERMAL_CTRL: u32 = 0x0011_0011;

    /// Fan control mode (DEVS).
    pub const FAN_CTRL: u32 = 0x0011_0012;

    /// Throttle thermal policy — the overall "profile"
    /// (Standard 0 / Performance 1 / Silent 2).
    /// Read via DSTS, write via DEVS.
    pub const THROTTLE_THERMAL_POLICY: u32 = 0x0012_0075;

    /// CPU fan-curve data (read/write via DSTS/DEVS).
    pub const CPU_FAN_CURVE: u32 = 0x0011_0024;

    /// GPU fan-curve data (read/write via DSTS/DEVS).
    pub const GPU_FAN_CURVE: u32 = 0x0011_0025;

    /// Middle fan-curve data (read/write via DSTS/DEVS).
    pub const MID_FAN_CURVE: u32 = 0x0011_0032;
}

// ---------------------------------------------------------------------------
// Low-level WMI helpers
// ---------------------------------------------------------------------------

/// Read a device status value via the **DSTS** WMI method.
///
/// Returns the raw `Device_Status` u32.
pub fn dsts(conn: &WmiConnection, device_id: u32) -> Result<u32> {
    let out = conn.exec_method(ATK_OBJECT_PATH, "DSTS", &[("Device_ID", device_id)])?;
    WmiConnection::get_property_u32(&out, "Device_Status")
}

/// Write a device control value via the **DEVS** WMI method.
///
/// Returns the raw result `Device_Status`.
pub fn devs(conn: &WmiConnection, device_id: u32, control: u32) -> Result<u32> {
    let out = conn.exec_method(
        ATK_OBJECT_PATH,
        "DEVS",
        &[("Device_ID", device_id), ("Control_Status", control)],
    )?;
    WmiConnection::get_property_u32(&out, "Device_Status")
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
