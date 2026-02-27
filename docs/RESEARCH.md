# NoCrate — ASUS 桌面主板 WMI 逆向研究报告

> 测试平台：ASUS ROG STRIX B850-G GAMING WIFI · AMD Ryzen 9 9950X · AM5
> 日期：2026-02-28

---

## 1. 背景

NoCrate 旨在替代 ASUS Armoury Crate，以轻量级 Tauri 应用实现主板风扇策略、
AURA RGB、温度监控等功能。核心挑战在于逆向 ASUS 私有 WMI 接口以获取传感器数据。

---

## 2. WMI 命名空间 & 类扫描结果

### 2.1 `root\WMI` — 匹配 ASUS/Fan/Temp/Sensor/Thermal 的类

| # | 类名 | 有实例？ | 说明 |
|---|------|---------|------|
| 1 | **ASUSManagement** | ✅ | 核心！69 个方法，桌面板控制的全部入口 |
| 2 | **AsusLedControlWmi** | ✅ | LED/AURA 控制接口 |
| 3 | **AsusWpbtWmi** | ✅ | Windows Platform Binary Table |
| 4 | MSAcpi_ThermalZoneTemperature | ❌ | ACPI 热区——本 BIOS 未暴露 |
| 5 | KernelThermalPolicyChange | ❌ | 事件类 |
| 6 | KernelThermalConstraintChange | ❌ | 事件类 |
| 7 | SensorClassExtensionControlGuid | ❌ | 传感器框架 GUID |
| 8 | MSNdis_80211_NumberOfAntennas | ❌ | Wi-Fi，匹配了关键字但无关 |

### 2.2 `root\CIMV2` — 标准硬件监控类

Win32_Fan、Win32_TemperatureProbe、CIM_Fan、CIM_TemperatureSensor、
Win32_PerfFormattedData_Counters_ThermalZoneInformation
→ **全部无实例**。Windows 标准传感器框架在 ASUS 桌面板上完全空白。

### 2.3 结论

**所有硬件控制入口都集中在 `ASUSManagement` 一个类的方法里。**
不存在 `ASUSHW_Fan` 之类的独立传感器数据类。

---

## 3. ASUSManagement 方法清单（69 个）

已完成全量枚举，按功能分组：

### 3.1 底层寄存器访问（asio_hw_fun 系列）— ⛔ 已确认锁死

| 方法 | 说明 | 状态 |
|------|------|------|
| `asio_hw_fun07/08` | I/O 端口 读/写 | 全返回 0 |
| `asio_hw_fun11~29` | SIO 寄存器、PCI Config、MSR 等 | 全返回 0 |
| `asio_hw_fun42~47` | SMBus 低级访问 | 全返回 0 |

**根因**：BIOS ACPI 方法内部需要验证调用者身份（来自 Armoury Crate 内核驱动的
魔法签名/握手），未经认证的调用虽能正确执行（返回正确的 VT 类型），但 BIOS
层面静默拦截，只返回零值。

### 3.2 高级 API — ✅ 已验证工作

| 方法 | 参数 | 状态 | 说明 |
|------|------|------|------|
| **GetFanPolicy** | FanType:u8 | ✅ 完美 | 返回 Mode/Profile/Source/LowLimit |
| **SetFanPolicy** | FanType, Mode, Profile, Source, LowLimit | ✅ 可用 | ErrorCode=2 当参数缺失 |
| **GetManualFanCurvePro** | FanType:u8, Mode:string | ✅ 完美 | 8 点曲线（PWM/DC/AUTO 全支持） |
| **SetManualFanCurvePro** | FanType, Mode, Point1-8 Temp+Duty | ✅ 可用 | 需正确参数 |
| **GetBootOptionName** | Handle:u16 | ✅ | 返回 "Windows Boot Manager" |
| **GetLastError** | (无) | ✅ | 最近一次操作的错误码 |
| **GetBootOrder** | (无) | ✅ | BIOS 启动顺序 |

### 3.3 有效但范围有限

| 方法 | 状态 | 说明 |
|------|------|------|
| GetManualFanCurve | ErrorCode=3 | 旧版 3 点曲线，B850-G 不支持（只支持 Pro） |
| GetOptionData | ErrorCode=14 | BIOS 选项查询，需要知道 OptionName |
| GetEyptValue1-4 | 返回 0xFFFFFFFF | 可能是加密/未实现 |
| read_smbus_byte/word/block | 0x8004100F | WDM Buffer 不匹配，需要精确大小 |
| device_status / device_ctrl | 全返回 0 | 见下文详细分析 |

### 3.4 device_status 暴力扫描 — ⛔ 全零

对以下区间逐个扫描 Device ID，**无一返回非零非 0xFFFFFFFF 值**：

```
0x0000_00xx  基础状态区间
0x0001_00xx ~ 0x000F_00xx  各 DevType 前缀
0x0010_00xx ~ 0x0014_00xx  含笔记本已知的风扇/温度/电源区间
0x0020_00xx ~ 0x0060_00xx  更高位区间
0x0010_10xx  AURA 区间
```

**结论**：`device_status` / `device_ctrl`（对应笔记本的 DSTS/DEVS）
在 B850-G 桌面板上被**完全锁死**，与 asio_hw_fun 系列同样受到 BIOS 认证拦截。
之前用的 0x00110013 等 ID 来自 Linux asus-wmi 驱动，它们可能仅适用于笔记本。

---

## 4. GetManualFanCurvePro 实测数据

使用正确的 Mode 参数（"PWM" / "DC" / "AUTO"）调用成功！

### FanType=0（CPU Fan）

| Mode | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 |
|------|----|----|----|----|----|----|----|----|
| PWM | 25°C→35% | 36°C→38% | 46°C→41% | 54°C→53% | 59°C→65% | 63°C→79% | 67°C→100% | 100°C→100% |
| DC | 20°C→60% | 30°C→60% | 40°C→60% | 54°C→60% | 58°C→81% | 65°C→90% | 70°C→100% | 100°C→100% |
| AUTO | 同 DC | | | | | | | |

### FanType=1（Chassis Fan）

| Mode | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 |
|------|----|----|----|----|----|----|----|----|
| PWM | 21°C→46% | 30°C→46% | 36°C→46% | 47°C→51% | 57°C→69% | 64°C→90% | 70°C→100% | 100°C→100% |
| DC | 20°C→60% | 30°C→63% | 40°C→67% | 50°C→73% | 58°C→81% | 65°C→90% | 70°C→100% | 100°C→100% |
| AUTO | 同 PWM | | | | | | | |

FanType 2-7 → ErrorCode=3（无对应风扇头），符合 B850-G 实际只有 2 个可控风扇头。

---

## 5. GetFanPolicy 实测数据

| FanType | Mode | Profile | Source | LowLimit |
|---------|------|---------|--------|----------|
| 0 (CPU) | PWM | MANUAL | *(空)* | 200 |
| 1 (Chassis1) | AUTO | MANUAL | CPU | 200 |
| 2 (Chassis2) | AUTO | STANDARD | CPU | 200 |
| 3 (Chassis3) | AUTO | STANDARD | CPU | 200 |
| 4-7 | — | — | — | *(无数据)* |

---

## 6. 能力矩阵

| 功能 | 读取 | 写入 | 方法 | 备注 |
|------|:----:|:----:|------|------|
| 风扇策略 (Mode/Profile/Source) | ✅ | ✅ | GetFanPolicy / SetFanPolicy | |
| 8 点风扇曲线 | ✅ | ✅ | GetManualFanCurvePro / SetManualFanCurvePro | PWM/DC/AUTO |
| 风扇 RPM 实时值 | ❌ | — | device_status 锁死 | 需第三方方案 |
| CPU/主板温度实时值 | ❌ | — | device_status 锁死 | 需第三方方案 |
| AURA RGB 控制 | ✅ | ✅ | HID（已实现） | 走 USB HID，不依赖 WMI |
| BIOS 启动顺序 | ✅ | ⚠️ | GetBootOrder / SetBootOrder | |

---

## 7. 实时传感器方案：拥抱 LibreHardwareMonitor

由于 ASUS WMI 的 `device_status` 和 `asio_hw_fun*` 全部被锁死，
**实时温度和 RPM 读取需要依赖第三方方案**。

推荐方案：**LibreHardwareMonitor (LHM)**

- 开源（MPL-2.0），活跃维护
- 自带内核驱动，可直读 Super I/O (NCT6799D) 寄存器
- 暴露 WMI 接口 `root\LibreHardwareMonitor`
- 或通过 HTTP API / 共享内存获取

集成路径选项：

| 方案 | 优点 | 缺点 |
|------|------|------|
| A. LHM WMI 接口 | 架构统一，复用现有 WMI 代码 | 需用户安装 LHM |
| B. LHM HTTP API | 最简单 | 需 LHM 开着 Web 服务器 |
| C. 内嵌 LHM DLL | 无需用户安装 | .NET 依赖，分发体积大 |
| D. 移植 LHM SIO 逻辑 | 零依赖 | 工作量大，需自带内核驱动 |

**推荐方案 A**：通过 WMI 查询 `root\LibreHardwareMonitor\Sensor`，
让用户安装 LHM 作为传感器服务后台。

---

## 8. 项目架构现状

```
src-tauri/src/
├── lib.rs              # Tauri 入口，注册所有命令
├── main.rs             # Windows 入口
├── error.rs            # NoCrateError 统一错误类型
├── state.rs            # AppState（WMI + AURA 初始化）
├── config.rs           # 用户配置持久化
├── wmi/
│   ├── mod.rs          # WmiThread 跨线程封装
│   ├── connection.rs   # WMI COM 连接 + exec_method + 后端探测
│   └── asus_mgmt.rs    # 类型化 API：FanPolicy/FanCurve/ThermalProfile
├── aura/
│   ├── mod.rs          # AURA HID 管理
│   ├── controller.rs   # AuraController 状态机
│   └── protocol.rs     # USB HID 协议定义
├── sio/                # ⚠️ 已废弃——SIO 直读方案不可行
│   ├── mod.rs
│   ├── driver.rs       # WinRing0x64.sys 驱动加载
│   ├── detect.rs       # SIO 芯片检测
│   ├── chips.rs        # 芯片 ID 常量
│   ├── nuvoton.rs      # NCT6799D 寄存器读取
│   └── ite.rs          # ITE 芯片（未实现）
└── commands/
    ├── mod.rs
    ├── fan.rs           # 风扇控制 Tauri 命令
    ├── aura.rs          # AURA RGB Tauri 命令
    ├── config.rs        # 配置管理命令
    └── system.rs        # 系统信息命令

src/                    # React 前端
├── App.tsx
├── pages/
│   ├── fan-page.tsx    # 风扇控制页
│   ├── aura-page.tsx   # AURA RGB 页
│   └── settings-page.tsx
├── components/
├── hooks/
├── layouts/
└── styles/
```
