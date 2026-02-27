/**
 * Shared TypeScript type definitions.
 *
 * These mirror the Rust serde types in `wmi::asus_mgmt` so that
 * Tauri invoke calls are fully typed end-to-end.
 */

// ─── Fan ─────────────────────────────────────────────────────

/** Identifies a fan header on the motherboard. */
export type FanTarget = "cpu" | "gpu" | "mid";

/** RPM snapshot for a single fan header. */
export interface FanInfo {
  target: FanTarget;
  rpm: number;
}

/** A single temperature → duty-cycle mapping point. */
export interface FanCurvePoint {
  /** Temperature threshold in °C (0–100). */
  temp_c: number;
  /** Fan duty-cycle percentage (0–100). */
  duty_pct: number;
}

/** A complete fan curve with 8 control points. */
export interface FanCurve {
  target: FanTarget;
  points: FanCurvePoint[];
}

// ─── Thermal Profile ─────────────────────────────────────────

/** ASUS thermal-profile presets. */
export type ThermalProfile = "standard" | "performance" | "silent";

/** Display metadata for a thermal profile. */
export interface ThermalProfileMeta {
  id: ThermalProfile;
  label: string;
  description: string;
}

export const THERMAL_PROFILES: ThermalProfileMeta[] = [
  {
    id: "silent",
    label: "静音",
    description: "降低风扇转速，优先安静",
  },
  {
    id: "standard",
    label: "标准",
    description: "平衡性能与噪音",
  },
  {
    id: "performance",
    label: "性能",
    description: "全速散热，最大性能",
  },
];

// ─── Fan Target Display ──────────────────────────────────────

/** Display label for a fan target. */
export const FAN_TARGET_LABELS: Record<FanTarget, string> = {
  cpu: "CPU",
  gpu: "GPU",
  mid: "机箱",
};

// ─── Desktop Fan Policy ──────────────────────────────────────

/** Fan control mode on desktop boards. */
export type DesktopFanMode = "PWM" | "AUTO";

/** Fan curve profile on desktop boards. */
export type DesktopFanProfile = "MANUAL" | "STANDARD";

/** Complete fan policy for a single desktop fan header. */
export interface DesktopFanPolicy {
  fan_type: number;
  mode: DesktopFanMode;
  profile: DesktopFanProfile;
  source: string;
  low_limit: number;
}

/** Display names for desktop fan header indices. */
export const DESKTOP_FAN_NAMES: Record<number, string> = {
  0: "CPU Fan",
  1: "机箱风扇 1",
  2: "机箱风扇 2",
  3: "机箱风扇 3",
  4: "机箱风扇 4",
  5: "机箱风扇 5",
  6: "机箱风扇 6",
  7: "机箱风扇 7",
};

/** WMI backend type. */
export type WmiBackend = "desktop" | "laptop" | "asushw";

// ─── ASUSHW Sensor types ─────────────────────────────────────

/** A single sensor reading from the ASUSHW backend. */
export interface AsusHWSensor {
  index: number;
  name: string;
  /** "temperature" | "fan" */
  sensor_type: string;
  /** °C for temperature, RPM for fans */
  value: number;
  source: number;
  data_type: number;
}
