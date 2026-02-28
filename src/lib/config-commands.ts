import { invoke } from "@tauri-apps/api/core";

// ─── Types ───────────────────────────────────────────────────
export interface AppConfig {
  theme: string;
  close_to_tray: boolean;
  auto_start: boolean;
  fan_poll_interval_ms: number;
  last_thermal_profile: number;
  last_aura_effect: string;
  last_aura_color: string;
  last_aura_speed: string;
  temp_alert_enabled: boolean;
  temp_alert_threshold: number;
}

export type ConfigUpdate = Partial<AppConfig>;

// ─── Invoke Wrappers ─────────────────────────────────────────
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function updateConfig(
  updates: ConfigUpdate,
): Promise<AppConfig> {
  return invoke<AppConfig>("update_config", updates);
}
