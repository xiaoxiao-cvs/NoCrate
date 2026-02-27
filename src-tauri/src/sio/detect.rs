// Super I/O 芯片自动检测
// 通过 I/O 端口 0x2E/0x4E 探测配置空间，识别 Nuvoton / ITE 芯片型号并读取 HW Monitor 基地址

use super::chips::Chip;
use super::driver::DriverHandle;
use super::ite::IteChip;
use super::nuvoton::NuvotonChip;
use crate::error::{NoCrateError, Result};

/// 探测芯片，返回初始化好的 Chip 实现
pub fn detect_chip(drv: &DriverHandle) -> Result<Box<dyn Chip>> {
    eprintln!("[SIO] 开始芯片检测...");
    // 依次在两个标准配置端口上探测
    for &config_port in &[0x2E_u16, 0x4E_u16] {
        eprintln!("[SIO] 探测配置端口 0x{config_port:02X}");
        // 先尝试 Nuvoton/Winbond（Fintek 共用入口序列）
        if let Some(chip) = try_nuvoton(drv, config_port)? {
            return Ok(chip);
        }

        // 再尝试 ITE
        if let Some(chip) = try_ite(drv, config_port)? {
            return Ok(chip);
        }
    }

    Err(NoCrateError::Sio(
        "未检测到已支持的 Super I/O 芯片（Nuvoton NCT67xx / ITE IT86xx）".into(),
    ))
}

/// 尝试以 Nuvoton/Winbond 协议探测
fn try_nuvoton(drv: &DriverHandle, port: u16) -> Result<Option<Box<dyn Chip>>> {
    let data_port = port + 1;

    // Nuvoton 进入扩展功能模式：向配置端口连写两次 0x87
    drv.write_io_port_byte(port, 0x87)?;
    drv.write_io_port_byte(port, 0x87)?;

    // 读取芯片 ID（寄存器 0x20 高字节、0x21 低字节）
    drv.write_io_port_byte(port, 0x20)?;
    let id_high = drv.read_io_port_byte(data_port)? as u16;
    drv.write_io_port_byte(port, 0x21)?;
    let id_low = drv.read_io_port_byte(data_port)? as u16;

    let chip_id = (id_high << 8) | id_low;

    eprintln!("[SIO]   Nuvoton 探测 @ 0x{port:02X}: ID=0x{chip_id:04X} (high=0x{id_high:02X}, low=0x{id_low:02X})");

    // 按高字节+掩码匹配已知 Nuvoton 芯片（低 nibble 为硅版本号，可忽略）
    // 参考 LibreHardwareMonitor LPCIO.cs 的 chip_id & 0xFFF0 匹配逻辑
    let chip_name = match chip_id & 0xFFF0 {
        0xD420 => "NCT6796D",
        0xD450 => "NCT6797D",
        0xD580 => "NCT6798D",
        0xD800 => "NCT6799D",
        0xC800 => "NCT6791D",
        0xC910 => "NCT6792D",
        0xC950 => "NCT6795D",
        _ => {
            // 不是 Nuvoton，退出扩展功能模式
            drv.write_io_port_byte(port, 0xAA)?;
            return Ok(None);
        }
    };

    // 选择 LDN 0x0B（Nuvoton HW Monitor 逻辑设备号）
    drv.write_io_port_byte(port, 0x07)?;
    drv.write_io_port_byte(data_port, 0x0B)?;

    // 读取 HW Monitor 基地址（寄存器 0x60 高字节、0x61 低字节）
    drv.write_io_port_byte(port, 0x60)?;
    let base_high = drv.read_io_port_byte(data_port)? as u16;
    drv.write_io_port_byte(port, 0x61)?;
    let base_low = drv.read_io_port_byte(data_port)? as u16;

    let base_addr = (base_high << 8) | base_low;

    // 退出扩展功能模式
    drv.write_io_port_byte(port, 0xAA)?;

    if base_addr == 0 || base_addr == 0xFFFF {
        return Ok(None);
    }

    eprintln!(
        "SIO: 检测到 {chip_name}，Chip ID=0x{chip_id:04X}，HW Monitor 基地址=0x{base_addr:04X}"
    );

    // 确保 LPC 桥解码此 I/O 范围（AMD FCH 需要显式配置）
    if let Err(e) = drv.enable_lpc_io_decode(base_addr) {
        eprintln!("[SIO] LPC I/O 解码配置警告: {e}");
    }

    Ok(Some(Box::new(NuvotonChip::new(
        chip_name.to_string(),
        chip_id,
        base_addr,
    ))))
}

/// 尝试以 ITE 协议探测
fn try_ite(drv: &DriverHandle, port: u16) -> Result<Option<Box<dyn Chip>>> {
    let data_port = port + 1;

    // ITE 进入配置模式的密钥序列（取决于端口地址）
    let key_sequence: &[u8] = if port == 0x2E {
        &[0x87, 0x01, 0x55, 0x55]
    } else {
        &[0x87, 0x01, 0x55, 0xAA]
    };

    for &byte in key_sequence {
        drv.write_io_port_byte(port, byte)?;
    }

    // 读取芯片 ID（寄存器 0x20 高字节、0x21 低字节）
    drv.write_io_port_byte(port, 0x20)?;
    let id_high = drv.read_io_port_byte(data_port)? as u16;
    drv.write_io_port_byte(port, 0x21)?;
    let id_low = drv.read_io_port_byte(data_port)? as u16;

    let chip_id = (id_high << 8) | id_low;

    eprintln!("[SIO]   ITE 探测 @ 0x{port:02X}: ID=0x{chip_id:04X} (high=0x{id_high:02X}, low=0x{id_low:02X})");

    // 检查是否为已知的 ITE 芯片
    let chip_name = match chip_id {
        0x8688 => "IT8688E",
        0x8689 => "IT8689E",
        0x8695 => "IT8695E",
        0x8686 => "IT8686E",
        0x8628 => "IT8628E",
        _ => {
            // 不是 ITE，退出配置模式
            drv.write_io_port_byte(port, 0x02)?;
            drv.write_io_port_byte(data_port, 0x02)?;
            return Ok(None);
        }
    };

    // 选择 LDN 0x04（ITE Environment Controller 逻辑设备号）
    drv.write_io_port_byte(port, 0x07)?;
    drv.write_io_port_byte(data_port, 0x04)?;

    // 读取 EC 基地址（寄存器 0x60 高字节、0x61 低字节）
    drv.write_io_port_byte(port, 0x60)?;
    let base_high = drv.read_io_port_byte(data_port)? as u16;
    drv.write_io_port_byte(port, 0x61)?;
    let base_low = drv.read_io_port_byte(data_port)? as u16;

    let base_addr = (base_high << 8) | base_low;

    // 退出配置模式
    drv.write_io_port_byte(port, 0x02)?;
    drv.write_io_port_byte(data_port, 0x02)?;

    if base_addr == 0 || base_addr == 0xFFFF {
        return Ok(None);
    }

    eprintln!("SIO: 检测到 {chip_name}，Chip ID=0x{chip_id:04X}，EC 基地址=0x{base_addr:04X}");

    Ok(Some(Box::new(IteChip::new(
        chip_name.to_string(),
        chip_id,
        base_addr,
    ))))
}
