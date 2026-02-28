/// LibreHardwareMonitor WMI sensor interface.
///
/// Queries the `root\LibreHardwareMonitor` WMI namespace (populated by LHM
/// when running as admin) for hardware and sensor data.
///
/// LHM WMI classes:
/// - `Hardware`: `Identifier`, `Name`, `HardwareType`, `Parent`
/// - `Sensor`: `Identifier`, `Name`, `SensorType`, `Value`, `Min`, `Max`, `Parent`
///
/// SensorType values (string): Voltage, Clock, Temperature, Load, Fan,
/// Flow, Control, Level, Factor, Power, Data, SmallData, Throughput,
/// TimeSpan, Energy, Noise.
use serde::Serialize;

use crate::error::Result;
use crate::wmi::connection::WmiConnection;

// ───────────────────────────── Types ──────────────────────────────

/// LHM service availability status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LhmStatus {
    /// LHM WMI namespace is accessible and has sensors.
    Available { sensor_count: usize },
    /// LHM WMI namespace exists but no sensors found.
    NoSensors,
    /// LHM WMI namespace is not accessible (not installed or not running).
    Unavailable,
}

/// A single LHM sensor reading.
#[derive(Debug, Clone, Serialize)]
pub struct LhmSensor {
    /// Unique sensor identifier, e.g. `/amdcpu/0/temperature/2`.
    pub identifier: String,
    /// Human-readable name, e.g. "CPU Package", "Fan #1".
    pub name: String,
    /// Sensor type: "Temperature", "Fan", "Control", "Voltage", etc.
    pub sensor_type: String,
    /// Current value (unit depends on sensor_type):
    /// - Temperature: °C
    /// - Fan: RPM
    /// - Control: % (0-100)
    /// - Voltage: V
    /// - Clock: MHz
    /// - Load: %
    /// - Power: W
    pub value: f32,
    /// Minimum observed value since LHM started.
    pub min: f32,
    /// Maximum observed value since LHM started.
    pub max: f32,
    /// Parent hardware identifier, e.g. `/amdcpu/0`.
    pub parent: String,
}

/// A group of sensors categorized by type.
#[derive(Debug, Clone, Serialize)]
pub struct LhmSensorSnapshot {
    pub temperatures: Vec<LhmSensor>,
    pub fans: Vec<LhmSensor>,
    pub controls: Vec<LhmSensor>,
    pub voltages: Vec<LhmSensor>,
    pub clocks: Vec<LhmSensor>,
    pub loads: Vec<LhmSensor>,
    pub powers: Vec<LhmSensor>,
}

// ───────────────────────────── Queries ─────────────────────────────

/// Check if LHM WMI is accessible.
pub fn get_lhm_status(conn: &WmiConnection) -> LhmStatus {
    if conn.lhm_services().is_none() {
        return LhmStatus::Unavailable;
    }
    match conn.lhm_query("SELECT Identifier FROM Sensor") {
        Ok(rows) if !rows.is_empty() => LhmStatus::Available {
            sensor_count: rows.len(),
        },
        Ok(_) => LhmStatus::NoSensors,
        Err(_) => LhmStatus::Unavailable,
    }
}

/// Read a single sensor from a WMI result object.
fn parse_sensor(
    _conn: &WmiConnection,
    obj: &windows::Win32::System::Wmi::IWbemClassObject,
) -> Option<LhmSensor> {
    let identifier = WmiConnection::get_property_string(obj, "Identifier").ok()?;
    let name = WmiConnection::get_property_string(obj, "Name").ok()?;
    let sensor_type = WmiConnection::get_property_string(obj, "SensorType").ok()?;
    let parent = WmiConnection::get_property_string(obj, "Parent")
        .ok()
        .unwrap_or_default();

    // Value/Min/Max are float properties — read as VARIANT and convert
    let value = get_property_f32(obj, "Value").unwrap_or(0.0);
    let min = get_property_f32(obj, "Min").unwrap_or(0.0);
    let max = get_property_f32(obj, "Max").unwrap_or(0.0);

    Some(LhmSensor {
        identifier,
        name,
        sensor_type,
        value,
        min,
        max,
        parent,
    })
}

/// Read a float property from a WMI object.
///
/// LHM sensor Value/Min/Max are typically VT_R4 (single-precision float).
#[allow(unsafe_code)]
fn get_property_f32(
    obj: &windows::Win32::System::Wmi::IWbemClassObject,
    name: &str,
) -> Option<f32> {
    use windows::core::BSTR;
    use windows::Win32::System::Variant::{VariantChangeType, VARIANT, VAR_CHANGE_FLAGS, VT_R4};

    unsafe {
        let mut val = VARIANT::default();
        obj.Get(&BSTR::from(name), 0, &mut val, None, None).ok()?;

        // Coerce any numeric type to VT_R4 (single-precision float)
        let mut coerced = VARIANT::default();
        VariantChangeType(&mut coerced, &val, VAR_CHANGE_FLAGS(0), VT_R4).ok()?;

        // Extract the f32 from the VARIANT's anonymous union
        // VT_R4 is stored in Anonymous.Anonymous.Anonymous.fltVal
        Some(coerced.Anonymous.Anonymous.Anonymous.fltVal)
    }
}

/// Fetch all sensors and group them by type.
pub fn get_all_sensors(conn: &WmiConnection) -> Result<LhmSensorSnapshot> {
    let rows = conn.lhm_query("SELECT * FROM Sensor")?;

    let mut snapshot = LhmSensorSnapshot {
        temperatures: Vec::new(),
        fans: Vec::new(),
        controls: Vec::new(),
        voltages: Vec::new(),
        clocks: Vec::new(),
        loads: Vec::new(),
        powers: Vec::new(),
    };

    for obj in &rows {
        if let Some(sensor) = parse_sensor(conn, obj) {
            match sensor.sensor_type.as_str() {
                "Temperature" => snapshot.temperatures.push(sensor),
                "Fan" => snapshot.fans.push(sensor),
                "Control" => snapshot.controls.push(sensor),
                "Voltage" => snapshot.voltages.push(sensor),
                "Clock" => snapshot.clocks.push(sensor),
                "Load" => snapshot.loads.push(sensor),
                "Power" => snapshot.powers.push(sensor),
                _ => {} // Ignore other types
            }
        }
    }

    Ok(snapshot)
}

/// Fetch only temperature and fan sensors (lightweight query for dashboard).
pub fn get_temp_and_fan_sensors(conn: &WmiConnection) -> Result<(Vec<LhmSensor>, Vec<LhmSensor>)> {
    let temps_rows = conn.lhm_query("SELECT * FROM Sensor WHERE SensorType = 'Temperature'")?;
    let fans_rows = conn.lhm_query("SELECT * FROM Sensor WHERE SensorType = 'Fan'")?;

    let temps: Vec<LhmSensor> = temps_rows
        .iter()
        .filter_map(|obj| parse_sensor(conn, obj))
        .collect();

    let fans: Vec<LhmSensor> = fans_rows
        .iter()
        .filter_map(|obj| parse_sensor(conn, obj))
        .collect();

    Ok((temps, fans))
}
