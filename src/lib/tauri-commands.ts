/**
 * Typed wrappers around Tauri `invoke()` for fan control commands.
 *
 * Each function corresponds to a `#[tauri::command]` defined in
 * `src-tauri/src/commands/fan.rs`.
 */
import { invoke } from "@tauri-apps/api/core";

import type {
  AsusHWSensor,
  DesktopFanCurve,
  DesktopFanMode,
  DesktopFanPolicy,
  FanCurve,
  FanInfo,
  FanTarget,
  LhmSensorSnapshot,
  LhmStatus,
  SioSnapshot,
  SioStatus,
  ThermalProfile,
  WmiBackend,
} from "@/lib/types";

/** Read a single fan's RPM. */
export async function getFanSpeed(target: FanTarget): Promise<number> {
  return invoke<number>("get_fan_speed", { target });
}

/** Read RPM for every detected fan header. */
export async function getAllFanSpeeds(): Promise<FanInfo[]> {
  return invoke<FanInfo[]>("get_all_fan_speeds");
}

/** Get the currently active thermal profile. */
export async function getThermalProfile(): Promise<ThermalProfile> {
  return invoke<ThermalProfile>("get_thermal_profile");
}

/** Set the thermal profile (standard / performance / silent). */
export async function setThermalProfile(
  profile: ThermalProfile,
): Promise<void> {
  return invoke<void>("set_thermal_profile", { profile });
}

/** Get a sensible default fan curve for a given target. */
export async function getDefaultFanCurve(
  target: FanTarget,
): Promise<FanCurve> {
  return invoke<FanCurve>("get_default_fan_curve", { target });
}

// ─── Desktop-specific commands ───────────────────────────────

/** Detect whether the WMI backend is "desktop" or "laptop". */
export async function getWmiBackend(): Promise<WmiBackend> {
  return invoke<WmiBackend>("get_wmi_backend");
}

/** Read fan policies for all present desktop fan headers. */
export async function getDesktopFanPolicies(): Promise<DesktopFanPolicy[]> {
  return invoke<DesktopFanPolicy[]>("get_desktop_fan_policies");
}

/** Write a single desktop fan header's policy. */
export async function setDesktopFanPolicy(
  policy: DesktopFanPolicy,
): Promise<void> {
  return invoke<void>("set_desktop_fan_policy", { policy });
}

/** 读取桌面风扇头在指定模式下的 8 点曲线。 */
export async function getDesktopFanCurve(
  fanType: number,
  mode: DesktopFanMode,
): Promise<DesktopFanCurve | null> {
  return invoke<DesktopFanCurve | null>("get_desktop_fan_curve", {
    fanType,
    mode,
  });
}

/** 写入桌面风扇头的 8 点曲线。 */
export async function setDesktopFanCurve(
  curve: DesktopFanCurve,
): Promise<void> {
  return invoke<void>("set_desktop_fan_curve", { curve });
}

/** 探测所有存在的风扇头及其支持的控制模式。 */
export async function probeDesktopFanTypes(): Promise<
  [number, DesktopFanMode[]][]
> {
  return invoke<[number, DesktopFanMode[]][]>("probe_desktop_fan_types");
}

// ─── ASUSHW sensor commands ──────────────────────────────────

/** Read all ASUSHW sensors (temperatures + fan RPMs). */
export async function getAsusHWSensors(): Promise<AsusHWSensor[]> {
  return invoke<AsusHWSensor[]>("get_asushw_sensors");
}

// ─── Super I/O 传感器命令 ────────────────────────────────────

/** 读取 Super I/O 芯片的所有风扇转速与温度传感器 */
export async function getSioSensors(): Promise<SioSnapshot> {
  return invoke<SioSnapshot>("get_sio_sensors");
}

/** 获取 Super I/O 模块状态（芯片型号、是否可用等） */
export async function getSioStatus(): Promise<SioStatus> {
  return invoke<SioStatus>("get_sio_status");
}

// ─── LibreHardwareMonitor 传感器命令 ─────────────────────────

/** 检测 LHM 服务是否可用。 */
export async function getLhmStatus(): Promise<LhmStatus> {
  return invoke<LhmStatus>("get_lhm_status");
}

/** 获取全部 LHM 传感器读数（分组）。 */
export async function getLhmSensors(): Promise<LhmSensorSnapshot> {
  return invoke<LhmSensorSnapshot>("get_lhm_sensors");
}
