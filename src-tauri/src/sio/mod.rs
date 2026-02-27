// Super I/O 模块
// 通过 WinRing0x64 内核驱动读取 Super I/O 芯片的风扇转速和温度传感器

pub mod chips;
pub mod detect;
pub mod driver;
pub mod ite;
pub mod nuvoton;

use parking_lot::Mutex;

use crate::error::Result;
use chips::{Chip, SioSnapshot, SioStatus};
use driver::DriverHandle;

/// Super I/O 传感器监控器
/// 持有驱动句柄和芯片实例，通过 Mutex 保证线程安全
pub struct SioMonitor {
    inner: Mutex<SioInner>,
    chip_name: String,
}

struct SioInner {
    driver: DriverHandle,
    chip: Box<dyn Chip>,
}

impl SioMonitor {
    /// 初始化 SIO 监控器
    /// 加载 WinRing0 驱动 → 探测 Super I/O 芯片 → 返回初始化完成的监控器
    pub fn init(resource_dir: &std::path::Path) -> Result<Self> {
        let driver = DriverHandle::open(resource_dir)?;
        let chip = detect::detect_chip(&driver)?;
        let chip_name = chip.chip_name().to_string();

        eprintln!("SIO: 初始化成功，芯片: {chip_name}");

        // ===== 诊断：检查 ISA HW Monitor 访问 =====
        {
            let base: u16 = 0x0290; // 从检测中获知的基地址
            eprintln!("[SIO-DIAG] base=0x{base:04X}");

            // 1) 先检查 LDN 0x0B 激活状态
            let cfg_port: u16 = 0x2E;
            let cfg_data: u16 = cfg_port + 1;
            // 进入扩展功能模式
            driver.write_io_port_byte(cfg_port, 0x87)?;
            driver.write_io_port_byte(cfg_port, 0x87)?;
            // 选择 LDN 0x0B
            driver.write_io_port_byte(cfg_port, 0x07)?;
            driver.write_io_port_byte(cfg_data, 0x0B)?;
            // 读取激活寄存器 0x30
            driver.write_io_port_byte(cfg_port, 0x30)?;
            let activate = driver.read_io_port_byte(cfg_data)?;
            eprintln!(
                "[SIO-DIAG] LDN 0x0B activate reg=0x{activate:02X} (bit0={})",
                activate & 1
            );
            // 重读基地址确认
            driver.write_io_port_byte(cfg_port, 0x60)?;
            let bh = driver.read_io_port_byte(cfg_data)?;
            driver.write_io_port_byte(cfg_port, 0x61)?;
            let bl = driver.read_io_port_byte(cfg_data)?;
            eprintln!("[SIO-DIAG] Re-read base=0x{:02X}{:02X}", bh, bl);
            // 退出配置模式
            driver.write_io_port_byte(cfg_port, 0xAA)?;

            // 2) 尝试不同端口偏移读取
            for offset in [0u16, 1, 5, 6, 7] {
                let v = driver.read_io_port_byte(base + offset)?;
                eprintln!(
                    "[SIO-DIAG] raw read base+0x{offset:X} (0x{:04X}) = 0x{v:02X}",
                    base + offset
                );
            }

            // 3) 标准 ISA 访问：写地址端口、读数据端口
            //    读 bank 0, reg 0x4F (Nuvoton vendor ID, 应为 0x5C)
            driver.write_io_port_byte(base + 5, 0x4E)?;
            driver.write_io_port_byte(base + 6, 0x00)?; // bank 0
            driver.write_io_port_byte(base + 5, 0x4F)?;
            let vendor = driver.read_io_port_byte(base + 6)?;
            eprintln!("[SIO-DIAG] Bank0 Reg0x4F (vendor ID) = 0x{vendor:02X} (expect 0x5C)");

            // 读 bank 0, reg 0x27 (SYSTIN temp)
            driver.write_io_port_byte(base + 5, 0x4E)?;
            driver.write_io_port_byte(base + 6, 0x00)?;
            driver.write_io_port_byte(base + 5, 0x27)?;
            let systin = driver.read_io_port_byte(base + 6)?;
            eprintln!("[SIO-DIAG] Bank0 Reg0x27 (SYSTIN) = 0x{systin:02X} ({systin}°C ?)");

            // 4) 尝试用 base+0/base+1 作为地址/数据端口
            driver.write_io_port_byte(base, 0x4F)?;
            let v2 = driver.read_io_port_byte(base + 1)?;
            eprintln!("[SIO-DIAG] alt access base+0/+1: reg 0x4F = 0x{v2:02X}");
        }
        // ===== 诊断结束 =====

        // 初始化后立即做一次测试读取，输出诊断信息
        {
            let fans = chip.read_fans(&driver)?;
            let temps = chip.read_temps(&driver)?;
            eprintln!("[SIO] 测试读取 — 风扇:");
            for f in &fans {
                eprintln!("[SIO]   {} (ch{}): {} RPM", f.name, f.channel, f.rpm);
            }
            eprintln!("[SIO] 测试读取 — 温度:");
            for t in &temps {
                eprintln!("[SIO]   {} (ch{}): {:.1}°C", t.name, t.channel, t.temp_c);
            }
        }

        Ok(Self {
            inner: Mutex::new(SioInner { driver, chip }),
            chip_name,
        })
    }

    /// 读取所有传感器数据快照
    pub fn read_all(&self) -> Result<SioSnapshot> {
        let inner = self.inner.lock();
        let fans = inner.chip.read_fans(&inner.driver)?;
        let temps = inner.chip.read_temps(&inner.driver)?;

        Ok(SioSnapshot {
            fans,
            temps,
            chip_name: self.chip_name.clone(),
        })
    }

    /// 获取状态信息
    pub fn status(&self) -> SioStatus {
        SioStatus {
            available: true,
            chip_name: Some(self.chip_name.clone()),
            error: None,
        }
    }
}

/// SIO 不可用时的状态
pub fn unavailable_status(error: &str) -> SioStatus {
    SioStatus {
        available: false,
        chip_name: None,
        error: Some(error.to_string()),
    }
}
