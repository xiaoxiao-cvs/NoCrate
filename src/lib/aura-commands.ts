/**
 * AURA ARGB type definitions and Tauri invoke wrappers.
 */
import { invoke } from "@tauri-apps/api/core";

// ─── Types ───────────────────────────────────────────────────

export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

export type AuraEffect =
  | "off"
  | "static"
  | "breathing"
  | "color_cycle"
  | "rainbow"
  | "spectrum_cycle";

export type AuraSpeed = "slow" | "medium" | "fast";

export interface AuraDeviceInfo {
  pid: number;
  product: string;
}

/** Display metadata for effects. */
export interface AuraEffectMeta {
  id: AuraEffect;
  label: string;
  hasColor: boolean;
}

export const AURA_EFFECTS: AuraEffectMeta[] = [
  { id: "off", label: "关闭", hasColor: false },
  { id: "static", label: "静态", hasColor: true },
  { id: "breathing", label: "呼吸", hasColor: true },
  { id: "color_cycle", label: "循环", hasColor: false },
  { id: "rainbow", label: "彩虹", hasColor: false },
  { id: "spectrum_cycle", label: "光谱", hasColor: false },
];

export const AURA_SPEEDS: { id: AuraSpeed; label: string }[] = [
  { id: "slow", label: "慢" },
  { id: "medium", label: "中" },
  { id: "fast", label: "快" },
];

// ─── Color helpers ───────────────────────────────────────────

export function rgbToHex(c: RgbColor): string {
  const hex = (n: number) => n.toString(16).padStart(2, "0");
  return `#${hex(c.r)}${hex(c.g)}${hex(c.b)}`;
}

export function hexToRgb(hex: string): RgbColor {
  const h = hex.replace("#", "");
  return {
    r: parseInt(h.slice(0, 2), 16),
    g: parseInt(h.slice(2, 4), 16),
    b: parseInt(h.slice(4, 6), 16),
  };
}

// ─── Invoke wrappers ─────────────────────────────────────────

export async function auraIsAvailable(): Promise<boolean> {
  return invoke<boolean>("aura_is_available");
}

export async function auraGetDeviceInfo(): Promise<AuraDeviceInfo> {
  return invoke<AuraDeviceInfo>("aura_get_device_info");
}

export async function auraSetEffect(
  effect: AuraEffect,
  color: RgbColor,
  speed: AuraSpeed,
): Promise<void> {
  return invoke<void>("aura_set_effect", { effect, color, speed });
}

export async function auraSetStaticColor(color: RgbColor): Promise<void> {
  return invoke<void>("aura_set_static_color", { color });
}

export async function auraTurnOff(): Promise<void> {
  return invoke<void>("aura_turn_off");
}

export async function auraSetDirectColors(
  colors: RgbColor[],
): Promise<void> {
  return invoke<void>("aura_set_direct_colors", { colors });
}
