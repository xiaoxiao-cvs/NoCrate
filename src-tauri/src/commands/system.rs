/// System-level commands (admin check, UAC elevation, auto-start, etc.)
use std::os::windows::ffi::OsStrExt;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Registry::{
    RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Check whether the current process is running with elevated (admin) privileges.
#[tauri::command]
pub fn is_admin() -> bool {
    is_elevated().unwrap_or(false)
}

/// Re-launch the current executable with UAC elevation ("Run as administrator"),
/// then exit the current (non-elevated) instance.
///
/// If the user declines the UAC prompt, `ShellExecuteW` returns ≤ 32 and we
/// return an error instead of exiting.
#[tauri::command]
pub fn restart_as_admin(app: tauri::AppHandle) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("无法获取当前程序路径: {e}"))?;
    let exe_wide: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();

    #[allow(unsafe_code)]
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW returns an HINSTANCE; values > 32 indicate success.
    if result.0 as usize > 32 {
        // New elevated process is starting — exit the current one.
        app.exit(0);
        Ok(())
    } else {
        Err("用户取消了管理员提权请求".into())
    }
}

// ---------------------------------------------------------------------------
// Auto-start (Windows registry)
// ---------------------------------------------------------------------------

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_VALUE_NAME: &str = "NoCrate";

/// Enable or disable auto-start at login via the Windows registry.
///
/// Writes to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
#[tauri::command]
pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| format!("无法获取当前程序路径: {e}"))?;
        let exe_path = format!("\"{}\"", exe.display());
        registry_set_run_value(APP_VALUE_NAME, &exe_path)
            .map_err(|e| format!("写入注册表失败: {e}"))
    } else {
        registry_delete_run_value(APP_VALUE_NAME)
            .map_err(|e| format!("删除注册表项失败: {e}"))
    }
}

/// Check whether the auto-start registry key is currently set.
#[tauri::command]
pub fn get_auto_start_enabled() -> bool {
    registry_has_run_value(APP_VALUE_NAME)
}

/// Write a value to `HKCU\...\Run`.
#[allow(unsafe_code)]
fn registry_set_run_value(name: &str, value: &str) -> windows::core::Result<()> {
    unsafe {
        let mut key = Default::default();
        let subkey: Vec<u16> = RUN_KEY.encode_utf16().chain(std::iter::once(0)).collect();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()?;

        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let value_w: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes: &[u8] = std::slice::from_raw_parts(
            value_w.as_ptr().cast(),
            value_w.len() * 2,
        );

        RegSetValueExW(
            key,
            PCWSTR(name_w.as_ptr()),
            None,
            REG_SZ,
            Some(bytes),
        )
        .ok()?;

        Ok(())
    }
}

/// Remove a value from `HKCU\...\Run`.
#[allow(unsafe_code)]
fn registry_delete_run_value(name: &str) -> windows::core::Result<()> {
    unsafe {
        let mut key = Default::default();
        let subkey: Vec<u16> = RUN_KEY.encode_utf16().chain(std::iter::once(0)).collect();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()?;

        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        // Ignore error if value doesn't exist
        let _ = RegDeleteValueW(key, PCWSTR(name_w.as_ptr()));
        Ok(())
    }
}

/// Check whether a value exists in `HKCU\...\Run`.
#[allow(unsafe_code)]
fn registry_has_run_value(name: &str) -> bool {
    use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, KEY_READ};
    unsafe {
        let mut key = Default::default();
        let subkey: Vec<u16> = RUN_KEY.encode_utf16().chain(std::iter::once(0)).collect();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
        .ok()
        .is_err()
        {
            return false;
        }

        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        RegQueryValueExW(
            key,
            PCWSTR(name_w.as_ptr()),
            None,
            None,
            None,
            None,
        )
        .ok()
        .is_ok()
    }
}

#[allow(unsafe_code)]
fn is_elevated() -> Option<bool> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;

        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast()),
            size,
            &mut ret_len,
        );

        let _ = CloseHandle(token);
        result.ok()?;

        Some(elevation.TokenIsElevated != 0)
    }
}
