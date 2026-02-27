// WinRing0x64 内核驱动管理
// 负责驱动安装、卸载、设备句柄管理，以及 I/O 端口读写原语
#![allow(unsafe_code)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
    OpenServiceW, StartServiceW, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_ERROR_NORMAL,
    SERVICE_KERNEL_DRIVER, SERVICE_DEMAND_START, SERVICE_STATUS,
};

use crate::error::{NoCrateError, Result};

/// WinRing0 IOCTL 命令码（从 WinRing0 开源头文件提取）
const IOCTL_OLS_READ_IO_PORT_BYTE: u32 = 0x9C40_2480;
const IOCTL_OLS_WRITE_IO_PORT_BYTE: u32 = 0x9C40_2488;

/// 驱动设备路径
const DEVICE_PATH: &str = r"\\.\WinRing0_1_2_0";
/// 驱动服务名称
const SERVICE_NAME: &str = "WinRing0_1_2_0";

/// WinRing0 驱动句柄，持有设备和服务控制管理器的引用。
/// Drop 时自动关闭设备句柄并卸载驱动服务。
pub struct DriverHandle {
    device: HANDLE,
    #[allow(dead_code)]
    driver_path: PathBuf,
}

// HANDLE (DeviceIoControl) 可以安全地跨线程使用
// 我们通过 Mutex 保证同一时刻只有一个线程在调用
#[allow(unsafe_code)]
unsafe impl Send for DriverHandle {}
#[allow(unsafe_code)]
unsafe impl Sync for DriverHandle {}

impl DriverHandle {
    /// 安装并打开 WinRing0 内核驱动。
    ///
    /// 流程：提取 .sys 文件到临时目录 → 注册为内核服务 → 启动服务 → 打开设备句柄
    pub fn open(resource_dir: &std::path::Path) -> Result<Self> {
        // 驱动 .sys 文件路径（从 Tauri 资源目录提取）
        let driver_path = resource_dir.join("WinRing0x64.sys");
        if !driver_path.exists() {
            // 如果资源目录没有，尝试当前 exe 同目录
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("WinRing0x64.sys")));
            if let Some(alt) = exe_dir {
                if alt.exists() {
                    return Self::open_with_path(&alt);
                }
            }
            return Err(NoCrateError::Sio(format!(
                "找不到驱动文件: {}",
                driver_path.display()
            )));
        }
        Self::open_with_path(&driver_path)
    }

    /// 使用指定路径的驱动文件安装并打开
    fn open_with_path(driver_path: &std::path::Path) -> Result<Self> {
        let driver_path_abs = std::fs::canonicalize(driver_path).map_err(|e| {
            NoCrateError::Sio(format!("无法解析驱动路径: {e}"))
        })?;

        // 先尝试用已有服务启动
        if let Err(_) = Self::try_start_existing_service() {
            // 服务不存在，需要创建
            Self::install_service(&driver_path_abs)?;
        }

        // 打开设备句柄
        let device = Self::open_device()?;

        Ok(Self {
            device,
            driver_path: driver_path_abs,
        })
    }

    /// 尝试启动已经存在的驱动服务
    fn try_start_existing_service() -> Result<()> {
        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS)
                .map_err(|e| NoCrateError::Sio(format!("无法打开服务控制管理器: {e}")))?;

            let svc_name = to_wide(SERVICE_NAME);
            let svc = OpenServiceW(scm, PCWSTR(svc_name.as_ptr()), SERVICE_ALL_ACCESS);

            match svc {
                Ok(svc_handle) => {
                    // 服务已存在，尝试启动（可能已经在运行）
                    let _ = StartServiceW(svc_handle, None);
                    let _ = CloseServiceHandle(svc_handle);
                    let _ = CloseServiceHandle(scm);
                    Ok(())
                }
                Err(_) => {
                    let _ = CloseServiceHandle(scm);
                    Err(NoCrateError::Sio("服务不存在".into()))
                }
            }
        }
    }

    /// 创建并启动内核驱动服务
    fn install_service(driver_path: &std::path::Path) -> Result<()> {
        unsafe {
            let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS)
                .map_err(|e| NoCrateError::Sio(format!("无法打开服务控制管理器: {e}")))?;

            let svc_name = to_wide(SERVICE_NAME);
            let display_name = to_wide("WinRing0_1_2_0");
            let binary_path = to_wide(&driver_path.to_string_lossy());

            let svc = CreateServiceW(
                scm,
                PCWSTR(svc_name.as_ptr()),
                PCWSTR(display_name.as_ptr()),
                SERVICE_ALL_ACCESS,
                SERVICE_KERNEL_DRIVER,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                PCWSTR(binary_path.as_ptr()),
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            )
            .map_err(|e| {
                let _ = CloseServiceHandle(scm);
                NoCrateError::Sio(format!("无法创建驱动服务: {e}"))
            })?;

            let start_result = StartServiceW(svc, None);
            if let Err(e) = start_result {
                // 如果已经在运行，忽略错误
                let code = e.code().0 as u32;
                if code != 0x80070420 {
                    // ERROR_SERVICE_ALREADY_RUNNING
                    let _ = DeleteService(svc);
                    let _ = CloseServiceHandle(svc);
                    let _ = CloseServiceHandle(scm);
                    return Err(NoCrateError::Sio(format!("无法启动驱动服务: {e}")));
                }
            }

            let _ = CloseServiceHandle(svc);
            let _ = CloseServiceHandle(scm);
            Ok(())
        }
    }

    /// 打开驱动设备句柄
    fn open_device() -> Result<HANDLE> {
        use windows::Win32::Foundation::GENERIC_READ;
        use windows::Win32::Foundation::GENERIC_WRITE;

        let path = to_wide(DEVICE_PATH);
        unsafe {
            let handle = CreateFileW(
                PCWSTR(path.as_ptr()),
                (GENERIC_READ.0 | GENERIC_WRITE.0).into(),
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .map_err(|e| NoCrateError::Sio(format!("无法打开驱动设备: {e}")))?;

            if handle == INVALID_HANDLE_VALUE {
                return Err(NoCrateError::Sio("打开驱动设备返回无效句柄".into()));
            }

            Ok(handle)
        }
    }

    /// 从 I/O 端口读取一个字节
    pub fn read_io_port_byte(&self, port: u16) -> Result<u8> {
        let mut input = port as u32;
        let mut output: u32 = 0;
        let mut bytes_returned: u32 = 0;

        unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_OLS_READ_IO_PORT_BYTE,
                Some(std::ptr::addr_of_mut!(input).cast()),
                std::mem::size_of::<u32>() as u32,
                Some(std::ptr::addr_of_mut!(output).cast()),
                std::mem::size_of::<u32>() as u32,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|e| NoCrateError::Sio(format!("读取 I/O 端口 0x{port:04X} 失败: {e}")))?;
        }

        Ok(output as u8)
    }

    /// 写入一个字节到 I/O 端口
    pub fn write_io_port_byte(&self, port: u16, value: u8) -> Result<()> {
        // WinRing0 WRITE_IO_PORT_BYTE 输入格式：低 16 位 = 端口，第 3 个字节 = 数据
        let mut input: u32 = (port as u32) | ((value as u32) << 16);
        let mut bytes_returned: u32 = 0;

        unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_OLS_WRITE_IO_PORT_BYTE,
                Some(std::ptr::addr_of_mut!(input).cast()),
                std::mem::size_of::<u32>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|e| NoCrateError::Sio(format!("写入 I/O 端口 0x{port:04X} 失败: {e}")))?;
        }

        Ok(())
    }
}

impl Drop for DriverHandle {
    fn drop(&mut self) {
        unsafe {
            // 关闭设备句柄
            let _ = CloseHandle(self.device);

            // 停止并删除驱动服务
            if let Ok(scm) =
                OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS)
            {
                let svc_name = to_wide(SERVICE_NAME);
                if let Ok(svc) =
                    OpenServiceW(scm, PCWSTR(svc_name.as_ptr()), SERVICE_ALL_ACCESS)
                {
                    let mut status = SERVICE_STATUS::default();
                    let _ = ControlService(svc, 1, &mut status); // 1 = SERVICE_CONTROL_STOP
                    let _ = DeleteService(svc);
                    let _ = CloseServiceHandle(svc);
                }
                let _ = CloseServiceHandle(scm);
            }
        }
    }
}

/// 将 &str 转换为以 null 结尾的宽字符串
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
