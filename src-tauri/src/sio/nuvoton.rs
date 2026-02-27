// Nuvoton NCT67xxD 系列 Super I/O 芯片传感器读取
//
// 支持型号：NCT6791D、NCT6792D、NCT6795D、NCT6796D、NCT6798D、NCT6799D
// 寄存器定义参考 LibreHardwareMonitor 与 Nuvoton 数据手册

use super::chips::{Chip, FanReading, TempReading};
use super::driver::DriverHandle;
use crate::error::Result;

/// Nuvoton NCT67xxD 芯片实例
pub struct NuvotonChip {
    name: String,
    #[allow(dead_code)]
    chip_id: u16,
    /// HW Monitor I/O 基地址
    base_addr: u16,
}

impl NuvotonChip {
    pub fn new(name: String, chip_id: u16, base_addr: u16) -> Self {
        Self {
            name,
            chip_id,
            base_addr,
        }
    }

    /// 读取指定 bank 和寄存器的值
    /// Nuvoton 使用 bank 切换寄存器 (base + 0x4E) 选择 bank，
    /// 然后通过 (base + 0x4F) 访问对应寄存器
    fn read_register(&self, drv: &DriverHandle, bank: u8, reg: u8) -> Result<u8> {
        // 选择 bank
        drv.write_io_port_byte(self.base_addr + 0x4E, bank)?;
        // 写入寄存器地址
        drv.write_io_port_byte(self.base_addr + 0x05, reg)?;
        // 读取值
        drv.read_io_port_byte(self.base_addr + 0x06)
    }
}

/// 风扇转速计寄存器定义
/// Nuvoton NCT67xx 系列：Bank 4 / Bank 7 (取决于具体型号)
/// 每个风扇占 2 字节（16-bit 计数值），RPM = 1,350,000 / count
struct FanChannel {
    name: &'static str,
    /// 计数值高字节寄存器（Bank 4）
    count_high_reg: u8,
    /// 计数值低字节寄存器（Bank 4）
    count_low_reg: u8,
    /// 通道编号
    channel: u8,
}

/// 温度传感器寄存器定义
struct TempChannel {
    name: &'static str,
    /// 温度整数部分所在 bank
    bank: u8,
    /// 温度整数部分寄存器
    int_reg: u8,
    /// 温度小数部分所在 bank（可能和整数不同）
    frac_bank: u8,
    /// 温度小数部分寄存器
    frac_reg: u8,
    /// 通道编号
    channel: u8,
}

/// NCT67xx 系列 7 路风扇通道（Bank 4 寄存器）
const FAN_CHANNELS: &[FanChannel] = &[
    FanChannel { name: "CPU Fan",     count_high_reg: 0xC0, count_low_reg: 0xC1, channel: 0 }, // SYSFANIN
    FanChannel { name: "机箱 #1",     count_high_reg: 0xC2, count_low_reg: 0xC3, channel: 1 }, // CPUFANIN
    FanChannel { name: "机箱 #2",     count_high_reg: 0xC4, count_low_reg: 0xC5, channel: 2 }, // AUXFANIN0
    FanChannel { name: "机箱 #3",     count_high_reg: 0xC6, count_low_reg: 0xC7, channel: 3 }, // AUXFANIN1
    FanChannel { name: "机箱 #4",     count_high_reg: 0xC8, count_low_reg: 0xC9, channel: 4 }, // AUXFANIN2
    FanChannel { name: "机箱 #5",     count_high_reg: 0xCA, count_low_reg: 0xCB, channel: 5 }, // AUXFANIN3
    FanChannel { name: "机箱 #6",     count_high_reg: 0xCC, count_low_reg: 0xCD, channel: 6 }, // AUXFANIN4
];

/// NCT67xx 系列温度通道
/// Bank 0: SYSTIN / CPUTIN (传统)
/// Bank 7: PECI / TSI (AMD) 等新增通道
const TEMP_CHANNELS: &[TempChannel] = &[
    TempChannel { name: "主板",   bank: 0, int_reg: 0x73, frac_bank: 0, frac_reg: 0x74, channel: 0 }, // SYSTIN
    TempChannel { name: "CPU",    bank: 0, int_reg: 0x75, frac_bank: 0, frac_reg: 0x76, channel: 1 }, // CPUTIN
    TempChannel { name: "辅助",   bank: 0, int_reg: 0x77, frac_bank: 0, frac_reg: 0x78, channel: 2 }, // AUXTIN0
    TempChannel { name: "辅助 1", bank: 1, int_reg: 0x50, frac_bank: 1, frac_reg: 0x51, channel: 3 }, // AUXTIN1
    TempChannel { name: "辅助 2", bank: 2, int_reg: 0x50, frac_bank: 2, frac_reg: 0x51, channel: 4 }, // AUXTIN2
    TempChannel { name: "辅助 3", bank: 6, int_reg: 0x50, frac_bank: 6, frac_reg: 0x51, channel: 5 }, // AUXTIN3
];

impl Chip for NuvotonChip {
    fn chip_name(&self) -> &str {
        &self.name
    }

    fn read_fans(&self, drv: &DriverHandle) -> Result<Vec<FanReading>> {
        let mut fans = Vec::new();

        for fc in FAN_CHANNELS {
            // 风扇计数值在 Bank 4
            let high = self.read_register(drv, 4, fc.count_high_reg)? as u16;
            let low = self.read_register(drv, 4, fc.count_low_reg)? as u16;
            let count = (high << 8) | low;

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
            let int_val = self.read_register(drv, tc.bank, tc.int_reg)? as i8;
            let frac_val = self.read_register(drv, tc.frac_bank, tc.frac_reg)?;

            // 温度 = 整数部分 + 小数部分高 1 位（0.5°C 精度）
            let frac = if frac_val & 0x80 != 0 { 0.5 } else { 0.0 };
            let temp_c = int_val as f32 + frac;

            // 过滤明显无效的读数
            #[allow(clippy::cast_possible_truncation)]
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
