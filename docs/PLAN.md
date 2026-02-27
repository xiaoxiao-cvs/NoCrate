# NoCrate — 后续开发计划

> 基于 RESEARCH.md 的调研结论制定

---

## Phase 1：完善风扇控制核心（1-2 天）

### 1.1 后端 — 接入 GetManualFanCurvePro / SetManualFanCurvePro

**目标**：用 `asus_mgmt.rs` 中已有的 `FanCurve` / `FanCurvePoint` 结构体，
打通与 `GetManualFanCurvePro` WMI 方法的读写链路。

- [ ] 新增 `get_desktop_fan_curve_pro(conn, fan_type, mode) -> FanCurve`
  - WMI 方法：`GetManualFanCurvePro`
  - 参数：`FanType: u8`, `Mode: String`（"PWM" / "DC" / "AUTO"）
  - 返回：解析 `ErrorCode`, `Point1Temp`~`Point8Temp`, `Point1Duty`~`Point8Duty`
  - 将已有的 `FanCurvePoint { temp, duty }` 结构体数组填充

- [ ] 新增 `set_desktop_fan_curve_pro(conn, fan_type, mode, &FanCurve) -> Result`
  - WMI 方法：`SetManualFanCurvePro`
  - 将 8 个 Point 的 Temp/Duty 打包成 WMI 参数
  - 校验温度单调递增、Duty 在 0-100 范围

- [ ] 适配 `DesktopFanMode` 枚举（PWM/DC/Auto）与 WMI Mode 字符串的互转

### 1.2 Tauri 命令层

- [ ] `commands/fan.rs` 新增：
  - `get_desktop_fan_curve(fan_type, mode) -> FanCurve`
  - `set_desktop_fan_curve(fan_type, mode, curve) -> ()`
  - `get_available_fan_types() -> Vec<FanInfo>` — 枚举 0-7，过滤 ErrorCode≠3
  - `get_available_fan_modes(fan_type) -> Vec<DesktopFanMode>`

### 1.3 前端 — 风扇曲线编辑器

- [ ] `fan-page.tsx` 重构：
  - 顶部：风扇选择器（CPU Fan / Chassis Fan 1 等）
  - 中间：8 点曲线可视化图表（拖拽编辑温度-Duty 映射）
  - 底部：策略选择器（Mode: PWM/DC/AUTO, Profile: Standard/Silent/Manual）
  - 应用 / 恢复默认 按钮
  - RPM 实时值暂显示 "需安装传感器服务" 占位提示

---

## Phase 2：传感器实时监控 — LibreHardwareMonitor 集成（2-3 天）

### 2.1 方案选择

采用 **LHM WMI 接口**（`root\LibreHardwareMonitor` 命名空间）。

前置条件：用户安装 [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)
并以管理员权限运行。

### 2.2 后端

- [ ] 新建 `src-tauri/src/wmi/lhm.rs`：
  - 检测 LHM WMI 命名空间是否存在
  - 查询 `root\LibreHardwareMonitor\Hardware` — 枚举硬件
  - 查询 `root\LibreHardwareMonitor\Sensor` — 按 SensorType 过滤：
    - `Temperature` — CPU Package/Core, Motherboard, VRM
    - `Fan` — 各风扇 RPM
    - `Control` — 风扇 PWM 占空比
    - `Voltage` — 电压轨
  - 轮询间隔 1-2 秒，推送到前端

- [ ] `connection.rs` — 新增 LHM WMI 连接（独立于 `root\WMI` 的连接实例）

### 2.3 Tauri 命令 / 事件

- [ ] `commands/sensor.rs`：
  - `get_sensor_status() -> SensorServiceStatus`（未安装 / 已安装未运行 / 正常）
  - `get_all_sensors() -> Vec<SensorReading>`
  - `start_sensor_polling(interval_ms)` — 启动后台轮询
  - `stop_sensor_polling()`
- [ ] Tauri Event：`sensor-update` — 推送实时传感器数据到前端

### 2.4 前端

- [ ] Dashboard / 仪表盘页面：
  - CPU 温度、风扇 RPM、主板温度实时显示
  - 传感器历史图表（最近 5 分钟）
- [ ] `fan-page.tsx` 集成：
  - 实时 RPM 显示在风扇曲线图上
  - 当前温度以竖线标注在曲线图上

---

## Phase 3：SIO 模块处置 & 代码质量（1 天）

### 3.1 SIO 模块

两种方案：

- **A. 移除**：删除 `src-tauri/src/sio/` 全部 6 个文件，Cargo.toml 移除 winring0 依赖
- **B. Feature Gate**：`Cargo.toml` 加 `[features] sio = ["winring0"]`，
  所有 SIO 代码用 `#[cfg(feature = "sio")]` 守卫。默认不编译。
  后续如果找到 AM5 LPC 解锁方案可重新启用。

**推荐 B**：保留研究成果但不影响编译体积和安全性。

### 3.2 代码清理

- [ ] 移除 `asus_mgmt.rs` 中已确认无效的 device_id 常量和 `get_fan_speed` / `get_all_fan_speeds` 函数
  （它们基于 `device_status`，在桌面板上不工作）
- [ ] 或保留但标记 `#[deprecated]` + 文档注释说明仅限笔记本
- [ ] `examples/test_asio.rs` 归入 `docs/` 作为测试备忘，或删除
- [ ] 统一错误处理：所有 WMI 方法返回 `Result<T, NoCrateError>`

---

## Phase 4：用户体验完善（2-3 天）

- [ ] 设置页面：
  - LHM 状态检测 + 安装引导
  - 风扇曲线 Profile 导入/导出（JSON）
  - 开机自启设置
  - 语言切换（中/英）
- [ ] 系统托盘：
  - 最小化到托盘
  - 托盘菜单快速切换风扇 Profile
  - 温度/RPM 简要显示
- [ ] 通知：
  - 温度超阈值警告
  - 风扇异常（RPM=0 但策略非停转）通知

---

## Phase 5：长期路线图

| 优先级 | 功能 | 说明 |
|--------|------|------|
| P1 | AURA 场景联动 | 温度→RGB 颜色映射 |
| P1 | 多 Profile 快速切换 | 游戏/静音/办公一键切换 |
| P2 | Windows 性能计划联动 | 电源方案变化时自动切换 Profile |
| P2 | 插件系统 | 支持社区贡献的主板适配 |
| P3 | Linux 支持 | 基于 asus-wmi 内核模块，桌面板兼容性更好 |
| P3 | 其他主板品牌 | MSI/Gigabyte 的 WMI 接口研究 |

---

## 技术债务

| 项目 | 描述 | 优先级 |
|------|------|--------|
| `connection.rs` 编译警告 | line 419 附近的 latent error，需 cargo clean 验证 | 高 |
| WMI 线程模型 | 当前用 parking_lot Mutex + 单线程，可改为 actor 模型 | 中 |
| 前端状态管理 | 目前直接 useState，可能需要 zustand/jotai | 中 |
| 测试覆盖 | 后端零测试，需 mock WMI 层添加单元测试 | 中 |
| CI/CD | 无自动构建/发布流水线 | 低 |
