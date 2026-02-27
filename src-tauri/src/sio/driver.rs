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
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, CreateServiceW, DeleteService, OpenSCManagerW,
    OpenServiceW, StartServiceW, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_DEMAND_START,
    SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, SERVICE_STATUS,
};
use windows::Win32::System::IO::DeviceIoControl;

use crate::error::{NoCrateError, Result};

/// WinRing0 IOCTL 命令码
/// CTL_CODE(DeviceType=40000, Function, METHOD_BUFFERED, Access)
/// READ_IO_PORT_BYTE:  CTL_CODE(0x9C40, 0x833, 0, FILE_READ_ACCESS=1)
/// WRITE_IO_PORT_BYTE: CTL_CODE(0x9C40, 0x836, 0, FILE_WRITE_ACCESS=2)
/// READ_PCI_CONFIG:    CTL_CODE(0x9C40, 0x851, 0, FILE_READ_ACCESS=1)
/// WRITE_PCI_CONFIG:   CTL_CODE(0x9C40, 0x852, 0, FILE_WRITE_ACCESS=2)
const IOCTL_OLS_READ_IO_PORT_BYTE: u32 = 0x9C40_60CC;
const IOCTL_OLS_WRITE_IO_PORT_BYTE: u32 = 0x9C40_A0D8;
/// DWORD I/O 端口读写（用于 PCI CF8/CFC 直接访问）
/// READ_IO_PORT_DWORD: CTL_CODE(0x9C40, 0x835, 0, FILE_READ_ACCESS=1)
const IOCTL_OLS_READ_IO_PORT_DWORD: u32 = 0x9C40_60D4;
/// WRITE_IO_PORT_DWORD: CTL_CODE(0x9C40, 0x838, 0, FILE_WRITE_ACCESS=2)
const IOCTL_OLS_WRITE_IO_PORT_DWORD: u32 = 0x9C40_A0E0;

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
        let driver_path_abs = std::fs::canonicalize(driver_path)
            .map_err(|e| NoCrateError::Sio(format!("无法解析驱动路径: {e}")))?;

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
        // WinRing0 OLS_WRITE_IO_PORT_INPUT 结构体：
        // struct { ULONG PortNumber; union { ULONG LongData; UCHAR CharData; }; }
        // 共 8 字节：前 4 字节 = 端口号，后 4 字节 = 数据（仅低字节有效）
        #[repr(C)]
        struct WriteInput {
            port: u32,
            data: u32,
        }
        let mut input = WriteInput {
            port: port as u32,
            data: value as u32,
        };
        let mut bytes_returned: u32 = 0;

        unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_OLS_WRITE_IO_PORT_BYTE,
                Some(std::ptr::addr_of_mut!(input).cast()),
                // 传入 5 字节（offsetof(CharData) + sizeof(u8)），与原版 C 代码一致
                5u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|e| NoCrateError::Sio(format!("写入 I/O 端口 0x{port:04X} 失败: {e}")))?;
        }

        Ok(())
    }

    /// 读取 I/O 端口 DWORD（用于 PCI CF8/CFC 访问）
    fn read_io_port_dword(&self, port: u16) -> Result<u32> {
        let mut input = port as u32;
        let mut output: u32 = 0;
        let mut bytes_returned: u32 = 0;

        unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_OLS_READ_IO_PORT_DWORD,
                Some(std::ptr::addr_of_mut!(input).cast()),
                std::mem::size_of::<u32>() as u32,
                Some(std::ptr::addr_of_mut!(output).cast()),
                std::mem::size_of::<u32>() as u32,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|e| {
                NoCrateError::Sio(format!("读取 I/O 端口 DWORD 0x{port:04X} 失败: {e}"))
            })?;
        }

        Ok(output)
    }

    /// 写入 I/O 端口 DWORD（用于 PCI CF8/CFC 访问）
    fn write_io_port_dword(&self, port: u16, value: u32) -> Result<()> {
        #[repr(C)]
        struct WriteInput {
            port: u32,
            data: u32,
        }
        let mut input = WriteInput {
            port: port as u32,
            data: value,
        };
        let mut bytes_returned: u32 = 0;

        unsafe {
            DeviceIoControl(
                self.device,
                IOCTL_OLS_WRITE_IO_PORT_DWORD,
                Some(std::ptr::addr_of_mut!(input).cast()),
                std::mem::size_of::<WriteInput>() as u32, // 8 字节（DWORD 完整写入）
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|e| {
                NoCrateError::Sio(format!("写入 I/O 端口 DWORD 0x{port:04X} 失败: {e}"))
            })?;
        }

        Ok(())
    }

    /// 通过传统 PCI CF8/CFC 端口读取配置空间 DWORD
    /// 比 HalGetBusData IOCTL 更可靠，直接操作 I/O 端口 0xCF8/0xCFC
    pub fn read_pci_config(&self, bus: u8, dev: u8, func: u8, reg_addr: u32) -> Result<u32> {
        // CONFIG_ADDRESS = (1<<31) | (bus<<16) | (device<<11) | (function<<8) | (register & 0xFC)
        let config_addr: u32 = 0x8000_0000
            | ((bus as u32) << 16)
            | (((dev as u32) & 0x1F) << 11)
            | (((func as u32) & 0x07) << 8)
            | (reg_addr & 0xFC);

        self.write_io_port_dword(0xCF8, config_addr)?;
        self.read_io_port_dword(0xCFC)
    }

    /// 通过传统 PCI CF8/CFC 端口写入配置空间 DWORD
    pub fn write_pci_config(
        &self,
        bus: u8,
        dev: u8,
        func: u8,
        reg_addr: u32,
        value: u32,
    ) -> Result<()> {
        let config_addr: u32 = 0x8000_0000
            | ((bus as u32) << 16)
            | (((dev as u32) & 0x1F) << 11)
            | (((func as u32) & 0x07) << 8)
            | (reg_addr & 0xFC);

        self.write_io_port_dword(0xCF8, config_addr)?;
        self.write_io_port_dword(0xCFC, value)
    }

    /// 检查并启用 AMD FCH LPC 桥接器对指定 I/O 范围的解码
    /// 用于确保 Super I/O HW Monitor 的 ISA I/O 空间被正确转发到 LPC 总线
    pub fn enable_lpc_io_decode(&self, base_addr: u16) -> Result<()> {
        // AMD FCH LPC 桥: Bus 0, Device 0x14, Function 3
        const BUS: u8 = 0;
        const DEV: u8 = 0x14;
        const FUNC: u8 = 3;

        // 读取 LPC 桥接器的 Vendor/Device ID 验证
        let vid_did = self.read_pci_config(BUS, DEV, FUNC, 0x00)?;
        eprintln!("[SIO-LPC] LPC bridge VendorID:DeviceID = 0x{vid_did:08X}");

        // 读取当前 I/O 解码使能状态
        let io_decode_enable = self.read_pci_config(BUS, DEV, FUNC, 0x44)?;
        eprintln!("[SIO-LPC] IO Port Decode Enable (0x44) = 0x{io_decode_enable:08X}");

        let io_mem_decode = self.read_pci_config(BUS, DEV, FUNC, 0x48)?;
        eprintln!("[SIO-LPC] IO/Mem Decode Enable (0x48) = 0x{io_mem_decode:08X}");

        // 读取 Wide I/O 解码范围
        let wide_io0 = self.read_pci_config(BUS, DEV, FUNC, 0x64)?;
        eprintln!("[SIO-LPC] Wide IO Range 0 (0x64) = 0x{wide_io0:08X}");

        let wide_io1 = self.read_pci_config(BUS, DEV, FUNC, 0x68)?;
        eprintln!("[SIO-LPC] Wide IO Range 1 (0x68) = 0x{wide_io1:08X}");

        let wide_io2 = self.read_pci_config(BUS, DEV, FUNC, 0x90)?;
        eprintln!("[SIO-LPC] Wide IO Range 2 (0x90) = 0x{wide_io2:08X}");

        // 检查 base_addr 是否已在某个 Wide I/O 范围中
        // Wide IO 0: bit[15:0] of offset 0x64
        let w0_base = (wide_io0 & 0xFFFF) as u16;
        let w0_enabled = io_mem_decode & 0x01 != 0;
        // Wide IO 1: bit[31:16] of offset 0x64
        let w1_base = ((wide_io0 >> 16) & 0xFFFF) as u16;
        let w1_enabled = io_mem_decode & 0x04 != 0;
        // Wide IO 2: bit[15:0] of offset 0x90
        let w2_base = (wide_io2 & 0xFFFF) as u16;
        let w2_enabled = io_mem_decode & (1 << 18) != 0;

        eprintln!("[SIO-LPC] Wide IO 0: base=0x{w0_base:04X} enabled={w0_enabled}");
        eprintln!("[SIO-LPC] Wide IO 1: base=0x{w1_base:04X} enabled={w1_enabled}");
        eprintln!("[SIO-LPC] Wide IO 2: base=0x{w2_base:04X} enabled={w2_enabled}");

        let already_decoded = (w0_enabled && w0_base == base_addr)
            || (w1_enabled && w1_base == base_addr)
            || (w2_enabled && w2_base == base_addr);

        if already_decoded {
            eprintln!("[SIO-LPC] HW Monitor I/O 范围已启用解码");
            return Ok(());
        }

        // 尝试找一个未使用的 Wide IO 范围来启用 base_addr 解码
        if !w0_enabled {
            eprintln!("[SIO-LPC] 配置 Wide IO 0 = 0x{base_addr:04X}");
            let new_wide = (wide_io0 & 0xFFFF0000) | (base_addr as u32);
            self.write_pci_config(BUS, DEV, FUNC, 0x64, new_wide)?;
            let new_enable = io_mem_decode | 0x01;
            self.write_pci_config(BUS, DEV, FUNC, 0x48, new_enable)?;
        } else if !w1_enabled {
            eprintln!("[SIO-LPC] 配置 Wide IO 1 = 0x{base_addr:04X}");
            let new_wide = (wide_io0 & 0x0000FFFF) | ((base_addr as u32) << 16);
            self.write_pci_config(BUS, DEV, FUNC, 0x64, new_wide)?;
            let new_enable = io_mem_decode | 0x04;
            self.write_pci_config(BUS, DEV, FUNC, 0x48, new_enable)?;
        } else if !w2_enabled {
            eprintln!("[SIO-LPC] 配置 Wide IO 2 = 0x{base_addr:04X}");
            let new_wide = (wide_io2 & 0xFFFF0000) | (base_addr as u32);
            self.write_pci_config(BUS, DEV, FUNC, 0x90, new_wide)?;
            let new_enable = io_mem_decode | (1 << 18);
            self.write_pci_config(BUS, DEV, FUNC, 0x48, new_enable)?;
        } else {
            return Err(NoCrateError::Sio(
                "所有 Wide I/O 解码范围已用尽，无法为 HW Monitor 添加 ISA 解码".into(),
            ));
        }

        // 验证
        let test_val = self.read_io_port_byte(base_addr + 5)?;
        eprintln!("[SIO-LPC] 解码配置后验证: read base+5 = 0x{test_val:02X}");

        Ok(())
    }
}

impl Drop for DriverHandle {
    fn drop(&mut self) {
        unsafe {
            // 关闭设备句柄
            let _ = CloseHandle(self.device);

            // 停止并删除驱动服务
            if let Ok(scm) = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS) {
                let svc_name = to_wide(SERVICE_NAME);
                if let Ok(svc) = OpenServiceW(scm, PCWSTR(svc_name.as_ptr()), SERVICE_ALL_ACCESS) {
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
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
