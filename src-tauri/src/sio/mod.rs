// Super I/O 模块
// 通过 WinRing0x64 内核驱动读取 Super I/O 芯片的风扇转速和温度传感器

pub mod chips;
pub mod detect;
pub mod driver;
pub mod ite;
pub mod nuvoton;

use parking_lot::Mutex;

use chips::{Chip, SioSnapshot, SioStatus};
use driver::DriverHandle;
use crate::error::{Result};

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
