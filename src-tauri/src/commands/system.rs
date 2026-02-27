/// System-level commands (admin check, etc.)
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Check whether the current process is running with elevated (admin) privileges.
#[tauri::command]
pub fn is_admin() -> bool {
    is_elevated().unwrap_or(false)
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
