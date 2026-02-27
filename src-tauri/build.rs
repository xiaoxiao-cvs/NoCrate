fn main() {
    // Embed a Windows application manifest that requests administrator privileges.
    // This is required for WMI/ACPI fan control and HID device access.
    #[cfg(target_os = "windows")]
    {
        let mut res = tauri_build::WindowsAttributes::new();
        res = res.app_manifest(include_str!("nocrate.exe.manifest"));
        let attrs = tauri_build::Attributes::new().windows_attributes(res);
        tauri_build::try_build(attrs).expect("failed to run tauri build");
    }

    #[cfg(not(target_os = "windows"))]
    {
        tauri_build::build();
    }
}
