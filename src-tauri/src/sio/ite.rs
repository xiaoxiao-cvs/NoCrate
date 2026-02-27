// ITE IT86xxE 系列 Super I/O 芯片传感器读取
//
// 支持型号：IT8628E、IT8686E、IT8688E、IT8689E、IT8695E
// 寄存器定义参考 LibreHardwareMonitor 与 ITE 数据手册

use super::chips::{Chip, FanReading, TempReading};
use super::driver::DriverHandle;
use crate::error::Result;

/// ITE IT86xxE 芯片实例
pub struct IteChip {
    name: String,
    chip_id: u16,
    /// Environment Controller I/O 基地址
    base_addr: u16,
}

impl IteChip {
    pub fn new(name: String, chip_id: u16, base_addr: u16) -> Self {
        Self {
            name,
            chip_id,
            base_addr,
        }
    }

    /// 读取 EC 寄存器
    /// ITE 使用 地址端口 (base + 0x05) / 数据端口 (base + 0x06)
    fn read_register(&self, drv: &DriverHandle, reg: u8) -> Result<u8> {
        drv.write_io_port_byte(self.base_addr + 0x05, reg)?;
        drv.read_io_port_byte(self.base_addr + 0x06)
    }
}

/// ITE 风扇转速计通道定义
/// 16-bit 计数值 = (高字节 << 8) | 低字节
/// RPM = 1,350,000 / count
#[derive(Clone)]
struct IteFanChannel {
    name: &'static str,
    /// 计数值低字节寄存器
    count_low_reg: u8,
    /// 计数值高字节寄存器（扩展寄存器）
    count_high_reg: u8,
    /// 通道编号
    channel: u8,
}

/// ITE 温度传感器通道定义
struct IteTempChannel {
    name: &'static str,
    /// 温度寄存器（8-bit 带符号整数）
    reg: u8,
    /// 通道编号
    channel: u8,
}

/// ITE 5 路风扇（IT8689E 最多 6 路，常见板子用 5 路）
const FAN_CHANNELS: &[IteFanChannel] = &[
    IteFanChannel { name: "CPU Fan",  count_low_reg: 0x0D, count_high_reg: 0x18, channel: 0 }, // FAN1
    IteFanChannel { name: "机箱 #1",  count_low_reg: 0x0E, count_high_reg: 0x19, channel: 1 }, // FAN2
    IteFanChannel { name: "机箱 #2",  count_low_reg: 0x0F, count_high_reg: 0x1A, channel: 2 }, // FAN3
    IteFanChannel { name: "机箱 #3",  count_low_reg: 0x80, count_high_reg: 0x81, channel: 3 }, // FAN4
    IteFanChannel { name: "机箱 #4",  count_low_reg: 0x82, count_high_reg: 0x83, channel: 4 }, // FAN5
];

/// 扩展风扇通道（IT8689E 第 6 路，较少见）
const FAN_CHANNELS_EXT: &[IteFanChannel] = &[
    IteFanChannel { name: "机箱 #5",  count_low_reg: 0x84, count_high_reg: 0x85, channel: 5 }, // FAN6
];

/// ITE 温度通道
const TEMP_CHANNELS: &[IteTempChannel] = &[
    IteTempChannel { name: "CPU",  reg: 0x29, channel: 0 }, // TMPIN1
    IteTempChannel { name: "主板", reg: 0x2A, channel: 1 }, // TMPIN2
    IteTempChannel { name: "辅助", reg: 0x2B, channel: 2 }, // TMPIN3
];

impl Chip for IteChip {
    fn chip_name(&self) -> &str {
        &self.name
    }

    fn read_fans(&self, drv: &DriverHandle) -> Result<Vec<FanReading>> {
        let mut fans = Vec::new();

        // 确认 16-bit 风扇计数器模式已开启
        // Configuration Register 0x0C bit 6: 16-bit 模式
        let config = self.read_register(drv, 0x0C)?;
        let is_16bit = (config & 0x40) != 0;

        let channels = if self.has_6_fans() {
            // IT8689E 等有 6 路风扇
            let mut all = FAN_CHANNELS.to_vec();
            all.extend_from_slice(FAN_CHANNELS_EXT);
            all
        } else {
            FAN_CHANNELS.to_vec()
        };

        for fc in &channels {
            let low = self.read_register(drv, fc.count_low_reg)? as u16;

            let count = if is_16bit {
                let high = self.read_register(drv, fc.count_high_reg)? as u16;
                (high << 8) | low
            } else {
                // 8-bit 模式下低字节即为全部计数值
                low
            };

            // 计算 RPM：count=0 或 0xFFFF 表示停转/未接入
            let rpm = if count == 0 || count == 0xFFFF {
                0
            } else {
                1_350_000 / count as u32
            };

            fans.push(FanReading {
                name: fc.name.to_string(),
                rpm,
                channel: fc.channel,
            });
        }

        Ok(fans)
    }

    fn read_temps(&self, drv: &DriverHandle) -> Result<Vec<TempReading>> {
        let mut temps = Vec::new();

        for tc in TEMP_CHANNELS {
            let raw = self.read_register(drv, tc.reg)? as i8;
            let temp_c = raw as f32;

            // 过滤无效读数
            if temp_c < -40.0 || temp_c > 125.0 {
                continue;
            }

            temps.push(TempReading {
                name: tc.name.to_string(),
                temp_c,
                channel: tc.channel,
            });
        }

        Ok(temps)
    }
}

impl IteChip {
    /// 判断是否支持 6 路风扇
    fn has_6_fans(&self) -> bool {
        matches!(self.chip_id, 0x8689 | 0x8695)
    }
}
