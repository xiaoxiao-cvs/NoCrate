/// System-level commands (admin check, UAC elevation, etc.)
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
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

use std::os::windows::ffi::OsStrExt;

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
