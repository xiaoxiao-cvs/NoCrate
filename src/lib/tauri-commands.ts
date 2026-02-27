/**
 * Typed wrappers around Tauri `invoke()` for fan control commands.
 *
 * Each function corresponds to a `#[tauri::command]` defined in
 * `src-tauri/src/commands/fan.rs`.
 */
import { invoke } from "@tauri-apps/api/core";

import type {
  DesktopFanPolicy,
  FanCurve,
  FanInfo,
  FanTarget,
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
