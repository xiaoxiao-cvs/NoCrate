import { invoke } from "@tauri-apps/api/core";

export async function isAdmin(): Promise<boolean> {
  return invoke<boolean>("is_admin");
}
