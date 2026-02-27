// Super I/O 芯片通用类型定义与 Chip trait

use serde::Serialize;

use super::driver::DriverHandle;
use crate::error::Result;

/// 风扇转速读数
#[derive(Debug, Clone, Serialize)]
pub struct FanReading {
    /// 风扇名称（如 "CPU Fan"、"Chassis #1"）
    pub name: String,
    /// 转速 (RPM)，0 表示停转或未接入
    pub rpm: u32,
    /// Super I/O 物理通道编号
    pub channel: u8,
}

/// 温度传感器读数
#[derive(Debug, Clone, Serialize)]
pub struct TempReading {
    /// 传感器名称（如 "CPU"、"Mainboard"、"AUXTIN1"）
    pub name: String,
    /// 温度（摄氏度）
    pub temp_c: f32,
    /// Super I/O 物理通道编号
    pub channel: u8,
}

/// 当前所有传感器读数的快照
#[derive(Debug, Clone, Serialize)]
pub struct SioSnapshot {
    /// 所有风扇读数
    pub fans: Vec<FanReading>,
    /// 所有温度读数
    pub temps: Vec<TempReading>,
    /// 芯片型号名称
    pub chip_name: String,
}

/// Super I/O 芯片状态信息
#[derive(Debug, Clone, Serialize)]
pub struct SioStatus {
    /// 是否可用
    pub available: bool,
    /// 芯片型号（如有）
    pub chip_name: Option<String>,
    /// 错误信息（如有）
    pub error: Option<String>,
}

/// Super I/O 芯片 trait
/// 每种芯片系列（Nuvoton、ITE）各自实现此 trait
pub trait Chip: Send + Sync {
    /// 返回芯片型号名称
    fn chip_name(&self) -> &str;

    /// 读取所有风扇转速
    fn read_fans(&self, drv: &DriverHandle) -> Result<Vec<FanReading>>;

    /// 读取所有温度传感器
    fn read_temps(&self, drv: &DriverHandle) -> Result<Vec<TempReading>>;
}
