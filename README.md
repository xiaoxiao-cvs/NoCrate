# NoCrate

一个轻量级的华硕主板硬件控制工具，用于替代 Armoury Crate（奥创中心）。

Armoury Crate 长期存在严重的系统资源泄露问题（句柄泄露可达百万级），且体积臃肿、后台进程繁多。NoCrate 直接与底层驱动通信，绕过华硕的用户态服务，仅保留最核心的硬件控制功能。

---

## 项目目标

- 替代 Armoury Crate 的风扇调节与 ARGB 灯效控制功能
- 消除 atkexComSvc.exe 等华硕服务的句柄泄露问题
- 提供轻量、美观、可靠的桌面控制面板
- 允许用户在替代后安全禁用华硕臃肿服务

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 | React + TypeScript + Tailwind CSS | 通过 Tauri WebView 渲染 |
| 后端 | Rust | 硬件通信、系统服务管理 |
| 框架 | Tauri v2 | 桌面应用壳，内置托盘、自启动、权限管理 |
| 风扇控制 | windows-rs (COM/WMI) | 调用 ASUSManagement WMI 接口 |
| ARGB 控制 | hidapi | 通过 USB HID 协议与 AURA 控制器通信 |
| 温度监控 | sysinfo + WMI | CPU/主板温度采集 |

## 硬件通信原理

### 风扇控制

通过 WMI 命名空间 `root\WMI` 中的 `ASUSManagement` 类，直接调用 ACPI 方法：

- `GetFanPolicy` / `SetFanPolicy` -- 风扇策略（静音/平衡/性能/手动）
- `GetManualFanCurve` / `SetManualFanCurve` -- 三点温度-转速曲线
- `SetManualFanCurvePro` -- 多点精细曲线

这些方法由华硕内核驱动 AsIO3.sys 提供，不依赖任何华硕用户态服务。

### ARGB 灯效控制

主板板载 AURA LED Controller 为标准 USB HID 设备（VID: 0x0B05），通过 65 字节 Feature Report 通信。协议已被 OpenRGB 项目完整逆向，支持：

- 静态色、呼吸、彩虹、循环等预设模式
- 逐 LED 独立控制（直接模式）
- 多通道（板载 RGB 头、ARGB 头）

## 功能规划

### 第一阶段 -- 核心功能

- 风扇策略切换（静音/平衡/性能/全速）
- 手动风扇曲线编辑（可视化温度-转速图表，支持拖拽调节）
- 实时温度与转速监控
- 系统托盘常驻，快捷切换配置

### 第二阶段 -- ARGB 灯效

- AURA 设备发现与识别
- 预设灯效模式选择
- 自定义静态颜色
- 多通道独立控制

### 第三阶段 -- 完善体验

- 配置文件导入/导出
- 开机自启动
- 华硕冗余服务一键禁用/还原
- 多配置方案快速切换（如：日常/游戏/安静）

## 系统要求

- Windows 10/11（需自带 WebView2 运行时）
- 华硕主板（需 AsIO3.sys 驱动正常运行）
- 管理员权限（WMI 硬件访问需要提权）

## 可替代的华硕组件

使用 NoCrate 后，以下华硕服务/进程可安全禁用：

| 组件 | 说明 | 禁用后影响 |
|------|------|------------|
| ArmouryCrateService | 奥创主服务 | 无 |
| AsusFanControlService | 风扇控制服务 | 由 NoCrate 接管 |
| atkexComSvc.exe | 句柄泄露元凶 | 无 |
| asus_framework.exe | NodeJS Web 前端 | 无 |

**不可禁用**：AsIO3.sys 内核驱动（NoCrate 依赖此驱动访问硬件）。

## 已验证的硬件环境

- ROG STRIX B850-G GAMING WIFI S（AMD B850 / AM5）
- AURA LED Controller: USB VID_0B05 PID_19AF
- 内核驱动: AsIO3.sys (Asusgio3)

理论上支持所有使用 AsIO3.sys 驱动和标准 AURA USB 协议的华硕主板，但不同型号的 WMI 方法参数可能存在差异，需要适配测试。

## 许可证

AGPL-3.0 License
