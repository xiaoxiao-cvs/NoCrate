import { invoke } from "@tauri-apps/api/core";

export async function isAdmin(): Promise<boolean> {
  return invoke<boolean>("is_admin");
}

/**
 * Request UAC elevation and restart the app as administrator.
 * Rejects if the user cancels the UAC prompt.
 */
export async function restartAsAdmin(): Promise<void> {
  return invoke<void>("restart_as_admin");
}
