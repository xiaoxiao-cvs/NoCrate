/// WMI COM connection management.
///
/// Manages the lifecycle of the COM connection to the `root\WMI` namespace,
/// providing access to `IWbemServices` for invoking ASUS WMI methods.
///
/// Supports three ASUS WMI backends:
///
/// - **Laptop** (`ASUSATKWMI_WMNB`): Uses `DSTS` / `DEVS` methods with
///   `Device_ID` / `Device_Status` / `Control_Status` parameters.
///   Instance path: `ASUSATKWMI_WMNB.InstanceName='ACPI\\ATK0110\\0_0'`.
///
/// - **Desktop** (`ASUSManagement`): Uses `device_status` / `device_ctrl`
///   methods with `device_id` / `ctrl_param` parameters.
///   Instance path: discovered at runtime via instance enumeration.
///
/// - **ASUSHW** (`ASUSHW`): Sensor-based backend providing read-only access
///   to temperature and fan RPM data via `sensor_get_*` methods.
///   Used as fallback when `ASUSManagement` is unavailable.
use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket, CoUninitialize,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL,
    RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Variant::{VariantChangeType, VARIANT, VAR_CHANGE_FLAGS, VT_I4};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_FORWARD_ONLY,
    WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_FLAG_RETURN_WBEM_COMPLETE,
};

use crate::error::{NoCrateError, Result};

/// A typed WMI method parameter value.
///
/// Used with [`WmiConnection::exec_method_v2`] to pass mixed-type
/// parameters to WMI methods.
pub enum WmiParam<'a> {
    /// 8-bit unsigned integer (CIM `uint8`).
    U8(u8),
    /// 32-bit unsigned integer (CIM `uint32` / `sint32`).
    U32(u32),
    /// String value (CIM `string`).
    Str(&'a str),
}

/// Detected ASUS WMI backend variant.
#[derive(Debug, Clone)]
pub enum AsusWmiBackend {
    /// Laptop: class `ASUSATKWMI_WMNB`, methods `DSTS`/`DEVS`.
    Laptop { instance_path: String },
    /// Desktop motherboard: class `ASUSManagement`, methods
    /// `device_status`/`device_ctrl`.
    Desktop { instance_path: String },
    /// ASUSHW sensor-based backend (read-only fan RPM & temperatures).
    /// Uses `sensor_get_*` / `sensor_update_buffer` methods.
    AsusHW { instance_path: String },
}

impl AsusWmiBackend {
    /// Human-readable label for log output.
    pub fn label(&self) -> &str {
        match self {
            Self::Laptop { .. } => "Laptop (ASUSATKWMI_WMNB)",
            Self::Desktop { .. } => "Desktop (ASUSManagement)",
            Self::AsusHW { .. } => "Desktop (ASUSHW Sensors)",
        }
    }

    /// The raw backend type string for frontend consumption.
    pub fn backend_type(&self) -> &str {
        match self {
            Self::Laptop { .. } => "laptop",
            Self::Desktop { .. } => "desktop",
            Self::AsusHW { .. } => "asushw",
        }
    }
}

/// RAII wrapper around a WMI connection to `root\WMI`.
///
/// COM is initialized on construction and cleaned up on drop.
/// This struct is **not** `Send`/`Sync` — it must live on the thread
/// that created it. Use the `WmiThread` helper for cross-thread access.
pub struct WmiConnection {
    services: IWbemServices,
    pub backend: AsusWmiBackend,
}

impl WmiConnection {
    /// Initialize COM, connect to `root\WMI`, and auto-detect the ASUS
    /// WMI backend (laptop vs desktop).
    ///
    /// Detection order:
    /// 1. Try `ASUSManagement` (desktop motherboards)
    /// 2. Try `ASUSATKWMI_WMNB` (laptops) with hardcoded instance path
    ///
    /// # Safety
    ///
    /// Calls COM initialization functions. Must be called from a thread that
    /// has not already initialized COM with an incompatible threading model.
    ///
    /// # Errors
    ///
    /// Returns `NoCrateError::Wmi` if no supported ASUS WMI class is found.
    #[allow(unsafe_code)]
    pub fn new() -> Result<Self> {
        unsafe {
            // Initialize COM runtime
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

            // Set default security levels (may fail if already called by
            // WebView2/Tauri — that's OK, we apply per-proxy security below)
            let _ = CoInitializeSecurity(
                None,
                -1,
                None,
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            )
            .ok();

            // Create WMI locator
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

            // Connect to root\WMI namespace
            let services = locator.ConnectServer(
                &BSTR::from("root\\WMI"),
                &BSTR::new(),
                &BSTR::new(),
                &BSTR::new(),
                0,
                &BSTR::new(),
                None,
            )?;
            eprintln!("[WMI] Connected to root\\WMI namespace");

            // Set per-proxy security — CRITICAL for WMI calls to succeed
            // when process-wide CoInitializeSecurity was set by another
            // component (e.g. WebView2).
            //   dwauthnsvc   = 10 (RPC_C_AUTHN_WINNT / NTLM)
            //   dwauthzsvc   = 0  (RPC_C_AUTHZ_NONE)
            //   principal    = null
            //   authn level  = RPC_C_AUTHN_LEVEL_CALL
            //   imp level    = RPC_C_IMP_LEVEL_IMPERSONATE
            let proxy_result = CoSetProxyBlanket(
                &services,
                10, // RPC_C_AUTHN_WINNT
                0,  // RPC_C_AUTHZ_NONE
                None,
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            );
            if let Err(ref e) = proxy_result {
                eprintln!("[WMI] CoSetProxyBlanket failed (non-fatal): {e}");
            }

            // Auto-detect backend
            let backend = Self::detect_backend(&services)?;
            eprintln!("[WMI] Backend detected: {}", backend.label());

            Ok(Self { services, backend })
        }
    }

    /// Probe available ASUS WMI classes and return the first working backend.
    ///
    /// Detection order:
    /// 1. `ASUSManagement` (desktop motherboard fan control)
    /// 2. `ASUSATKWMI_WMNB` (laptop ACPI)
    /// 3. `ASUSHW` (ASUS hardware sensor monitoring — read-only)
    #[allow(unsafe_code)]
    unsafe fn detect_backend(services: &IWbemServices) -> Result<AsusWmiBackend> {
        // 1. Try desktop: ASUSManagement (enumerate instances)
        eprintln!("[WMI] Probing ASUSManagement …");
        match Self::find_first_instance(services, "ASUSManagement") {
            Ok(path) => {
                eprintln!("[WMI]   ✓ ASUSManagement found: {path}");
                return Ok(AsusWmiBackend::Desktop {
                    instance_path: path,
                });
            }
            Err(e) => eprintln!("[WMI]   ✗ ASUSManagement: {e}"),
        }

        // 2. Try laptop: ASUSATKWMI_WMNB with common instance path
        eprintln!("[WMI] Probing ASUSATKWMI_WMNB …");
        let laptop_path = "ASUSATKWMI_WMNB.InstanceName='ACPI\\\\ATK0110\\\\0_0'";
        match Self::find_first_instance(services, "ASUSATKWMI_WMNB") {
            Ok(path) => {
                eprintln!("[WMI]   ✓ ASUSATKWMI_WMNB found: {path}");
                return Ok(AsusWmiBackend::Laptop {
                    instance_path: path,
                });
            }
            Err(e) => {
                eprintln!("[WMI]   ✗ ASUSATKWMI_WMNB enumerate: {e}");
                // Fallback: try GetObject on the class definition only
                let mut obj = None;
                let ok = services
                    .GetObject(
                        &BSTR::from("ASUSATKWMI_WMNB"),
                        WBEM_FLAG_RETURN_WBEM_COMPLETE,
                        None,
                        Some(&mut obj),
                        None,
                    )
                    .is_ok();
                if ok && obj.is_some() {
                    eprintln!("[WMI]   ✓ ASUSATKWMI_WMNB class exists (using hardcoded path)");
                    return Ok(AsusWmiBackend::Laptop {
                        instance_path: laptop_path.to_string(),
                    });
                }
                eprintln!("[WMI]   ✗ ASUSATKWMI_WMNB class not found");
            }
        }

        // 3. Try ASUSHW (sensor-only backend, used by FanControl.AsusWMI)
        eprintln!("[WMI] Probing ASUSHW …");
        match Self::find_first_instance(services, "ASUSHW") {
            Ok(path) => {
                eprintln!("[WMI]   ✓ ASUSHW found: {path}");
                return Ok(AsusWmiBackend::AsusHW {
                    instance_path: path,
                });
            }
            Err(e) => eprintln!("[WMI]   ✗ ASUSHW: {e}"),
        }

        Err(NoCrateError::Wmi(
            "未找到支持的 ASUS WMI 接口 (ASUSManagement / ASUSATKWMI_WMNB / ASUSHW)".into(),
        ))
    }

    /// Enumerate instances of a WMI class using `CreateInstanceEnum`
    /// (matches .NET `ManagementClass.GetInstances()`) and return the
    /// `__RELPATH` (relative object path) of the first instance found.
    ///
    /// This is more reliable than `ExecQuery` when process-wide COM
    /// security settings differ from what WQL queries expect.
    #[allow(unsafe_code)]
    unsafe fn find_first_instance(services: &IWbemServices, class_name: &str) -> Result<String> {
        // CreateInstanceEnum is the COM equivalent of .NET GetInstances()
        let enumerator = services
            .CreateInstanceEnum(
                &BSTR::from(class_name),
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                None,
            )
            .map_err(|e| {
                NoCrateError::Wmi(format!("CreateInstanceEnum failed for {class_name}: {e}"))
            })?;

        let mut returned: u32 = 0;
        let mut row = [None; 1];
        enumerator.Next(5000, &mut row, &mut returned).ok()?;

        if returned == 0 || row[0].is_none() {
            return Err(NoCrateError::Wmi(format!(
                "No instances found for {class_name}"
            )));
        }

        let obj = row[0].as_ref().unwrap();
        let mut val = VARIANT::default();
        obj.Get(&BSTR::from("__RELPATH"), 0, &mut val, None, None)?;

        // __RELPATH is a string VARIANT
        let path_bstr: BSTR = (&val).try_into().map_err(|_| {
            NoCrateError::Wmi(format!("__RELPATH for {class_name} is not a string"))
        })?;
        Ok(path_bstr.to_string())
    }

    /// Get a WMI class definition object.
    #[allow(unsafe_code)]
    pub fn get_object(&self, path: &str) -> Result<IWbemClassObject> {
        unsafe {
            let mut obj = None;
            self.services.GetObject(
                &BSTR::from(path),
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut obj),
                None,
            )?;
            obj.ok_or_else(|| NoCrateError::Wmi(format!("GetObject returned None for {path}")))
        }
    }

    /// Execute a WMI method on a given object path.
    ///
    /// 1. Gets the class definition
    /// 2. Gets the method input parameter signature
    /// 3. Spawns an instance and fills parameters
    /// 4. Calls ExecMethod and returns the output object
    #[allow(unsafe_code)]
    pub fn exec_method(
        &self,
        object_path: &str,
        method_name: &str,
        params: &[(&str, u32)],
    ) -> Result<IWbemClassObject> {
        unsafe {
            // GetMethod only works on class definitions, not instances.
            // Extract the class name (everything before the first '.') so
            // we can retrieve the class definition for the method signature.
            let class_name = object_path.split('.').next().unwrap_or(object_path);
            let class_obj = self.get_object(class_name)?;

            // Get input parameter definition for the method
            let mut in_params_def = None;
            class_obj.GetMethod(&BSTR::from(method_name), 0, &mut in_params_def, &mut None)?;

            let in_params = match in_params_def {
                Some(def) => {
                    let instance = def.SpawnInstance(0)?;
                    // Fill in parameters
                    for &(name, value) in params {
                        let variant = VARIANT::from(i32::try_from(value).unwrap_or(value as i32));
                        instance.Put(&BSTR::from(name), 0, &variant, 0)?;
                    }
                    Some(instance)
                }
                None => None,
            };

            // Execute the method
            let mut out_params = None;
            self.services.ExecMethod(
                &BSTR::from(object_path),
                &BSTR::from(method_name),
                Default::default(),
                None,
                in_params.as_ref(),
                Some(&mut out_params),
                None,
            )?;

            out_params.ok_or_else(|| {
                NoCrateError::Wmi(format!("ExecMethod returned no output for {method_name}"))
            })
        }
    }

    // -----------------------------------------------------------------------
    // High-level ASUS device helpers (backend-aware)
    // -----------------------------------------------------------------------

    /// Read a device status value (equivalent to laptop `DSTS`).
    ///
    /// - **Laptop**: calls `DSTS(Device_ID)` → `Device_Status`
    /// - **Desktop**: calls `device_status(device_id)` → `ctrl_param`
    /// - **AsusHW**: not supported (sensor-only backend)
    pub fn dsts(&self, device_id: u32) -> Result<u32> {
        match &self.backend {
            AsusWmiBackend::Laptop { instance_path } => {
                let out = self.exec_method(instance_path, "DSTS", &[("Device_ID", device_id)])?;
                Self::get_property_u32(&out, "Device_Status")
            }
            AsusWmiBackend::Desktop { instance_path } => {
                let out =
                    self.exec_method(instance_path, "device_status", &[("device_id", device_id)])?;
                Self::get_property_u32(&out, "ctrl_param")
            }
            AsusWmiBackend::AsusHW { .. } => Err(NoCrateError::Wmi(
                "ASUSHW 后端不支持 device_status 操作".into(),
            )),
        }
    }

    /// Write a device control value (equivalent to laptop `DEVS`).
    ///
    /// - **Laptop**: calls `DEVS(Device_ID, Control_Status)` → `Device_Status`
    /// - **Desktop**: calls `device_ctrl(device_id, ctrl_param)` → (void)
    /// - **AsusHW**: not supported (sensor-only backend)
    pub fn devs(&self, device_id: u32, control: u32) -> Result<u32> {
        match &self.backend {
            AsusWmiBackend::Laptop { instance_path } => {
                let out = self.exec_method(
                    instance_path,
                    "DEVS",
                    &[("Device_ID", device_id), ("Control_Status", control)],
                )?;
                Self::get_property_u32(&out, "Device_Status")
            }
            AsusWmiBackend::Desktop { instance_path } => {
                // device_ctrl may not return a meaningful value
                let _out = self.exec_method(
                    instance_path,
                    "device_ctrl",
                    &[("device_id", device_id), ("ctrl_param", control)],
                )?;
                Ok(1) // Success sentinel (matching laptop convention)
            }
            AsusWmiBackend::AsusHW { .. } => Err(NoCrateError::Wmi(
                "ASUSHW 后端不支持 device_ctrl 操作".into(),
            )),
        }
    }

    /// Read a u32 value from a WMI class object property.
    ///
    /// Handles multiple VARIANT types (VT_UI1, VT_I2, VT_UI2, VT_I4, VT_UI4)
    /// by coercing to VT_I4 before extraction when direct i32 extraction fails.
    #[allow(unsafe_code)]
    pub fn get_property_u32(obj: &IWbemClassObject, name: &str) -> Result<u32> {
        unsafe {
            let mut val = VARIANT::default();
            obj.Get(&BSTR::from(name), 0, &mut val, None, None)?;

            // 先尝试直接 i32 转换（最常见的 VT_I4 类型）
            if let Ok(i4) = i32::try_from(&val) {
                return Ok(i4 as u32);
            }

            // 对于其他数值类型 (VT_UI1, VT_I2, VT_UI2, VT_UI4 等)，
            // 使用 VariantChangeType 强制转换为 VT_I4
            let mut coerced = VARIANT::default();
            VariantChangeType(&mut coerced, &val, VAR_CHANGE_FLAGS(0), VT_I4)
                .map_err(|e| NoCrateError::Wmi(format!("属性 {name} 无法转换为数值类型: {e}")))?;

            let i4: i32 = (&coerced)
                .try_into()
                .map_err(|_| NoCrateError::Wmi(format!("属性 {name} 转换后仍非 i32")))?;
            Ok(i4 as u32)
        }
    }

    /// Read a string value from a WMI class object property.
    #[allow(unsafe_code)]
    pub fn get_property_string(obj: &IWbemClassObject, name: &str) -> Result<String> {
        unsafe {
            let mut val = VARIANT::default();
            obj.Get(&BSTR::from(name), 0, &mut val, None, None)?;

            let bstr: BSTR = (&val)
                .try_into()
                .map_err(|_| NoCrateError::Wmi(format!("Property {name} is not a string value")))?;
            Ok(bstr.to_string())
        }
    }

    /// Execute a WMI method with mixed-type parameters.
    ///
    /// Similar to [`exec_method`] but accepts [`WmiParam`] values
    /// supporting `u8`, `u32`, and string parameters.
    #[allow(unsafe_code)]
    pub fn exec_method_v2(
        &self,
        object_path: &str,
        method_name: &str,
        params: &[(&str, WmiParam<'_>)],
    ) -> Result<IWbemClassObject> {
        unsafe {
            let class_name = object_path.split('.').next().unwrap_or(object_path);
            let class_obj = self.get_object(class_name)?;

            let mut in_params_def = None;
            class_obj.GetMethod(&BSTR::from(method_name), 0, &mut in_params_def, &mut None)?;

            let in_params = match in_params_def {
                Some(def) => {
                    let instance = def.SpawnInstance(0)?;
                    for &(name, ref value) in params {
                        let variant = match value {
                            WmiParam::U8(v) => VARIANT::from(i32::from(*v)),
                            WmiParam::U32(v) => {
                                VARIANT::from(i32::try_from(*v).unwrap_or(*v as i32))
                            }
                            WmiParam::Str(s) => VARIANT::from(BSTR::from(*s)),
                        };
                        instance.Put(&BSTR::from(name), 0, &variant, 0)?;
                    }
                    Some(instance)
                }
                None => None,
            };

            let mut out_params = None;
            self.services.ExecMethod(
                &BSTR::from(object_path),
                &BSTR::from(method_name),
                Default::default(),
                None,
                in_params.as_ref(),
                Some(&mut out_params),
                None,
            )?;

            out_params.ok_or_else(|| {
                NoCrateError::Wmi(format!("ExecMethod returned no output for {method_name}"))
            })
        }
    }

    // -----------------------------------------------------------------------
    // ASUSHW sensor helpers
    // -----------------------------------------------------------------------

    /// Get the ASUSHW instance path, if the backend is AsusHW.
    fn asushw_path(&self) -> Result<&str> {
        match &self.backend {
            AsusWmiBackend::AsusHW { instance_path } => Ok(instance_path),
            _ => Err(NoCrateError::Wmi("当前后端不是 ASUSHW".into())),
        }
    }

    /// Query the ASUSHW sensor version (`sensor_get_version`).
    #[allow(dead_code)]
    pub fn asushw_sensor_version(&self) -> Result<u32> {
        let path = self.asushw_path()?;
        let out = self.exec_method(path, "sensor_get_version", &[])?;
        Self::get_property_u32(&out, "Data")
    }

    /// Get the total number of ASUSHW sensors (`sensor_get_number`).
    pub fn asushw_sensor_count(&self) -> Result<u32> {
        let path = self.asushw_path()?;
        let out = self.exec_method(path, "sensor_get_number", &[])?;
        Self::get_property_u32(&out, "Data")
    }

    /// Get sensor info for a given index (`sensor_get_info`).
    ///
    /// Returns `(source, sensor_type, data_type, name)`:
    /// - `sensor_type` 1 = temperature, 2 = fan
    /// - `data_type` 3 = value in micro-units (divide by 1_000_000)
    pub fn asushw_sensor_info(&self, index: u32) -> Result<(u32, u32, u32, String)> {
        let path = self.asushw_path()?;
        let out = self.exec_method(path, "sensor_get_info", &[("Index", index)])?;
        let source = Self::get_property_u32(&out, "Source")?;
        let sensor_type = Self::get_property_u32(&out, "Type")?;
        let data_type = Self::get_property_u32(&out, "Data_Type")?;
        let name = Self::get_property_string(&out, "Name")?;
        Ok((source, sensor_type, data_type, name))
    }

    /// Update the sensor buffer for a source group (`sensor_update_buffer`).
    pub fn asushw_update_buffer(&self, source: u32) -> Result<()> {
        let path = self.asushw_path()?;
        let _ = self.exec_method(path, "sensor_update_buffer", &[("Source", source)])?;
        Ok(())
    }

    /// Read the current value of a sensor (`sensor_get_value`).
    pub fn asushw_sensor_value(&self, index: u32) -> Result<u32> {
        let path = self.asushw_path()?;
        let out = self.exec_method(path, "sensor_get_value", &[("Index", index)])?;
        Self::get_property_u32(&out, "Data")
    }

    // -----------------------------------------------------------------------
    // asio_hw_fun* 硬件访问方法（仅 Desktop 后端）
    // -----------------------------------------------------------------------

    /// 获取 Desktop 后端实例路径。
    fn desktop_path(&self) -> Result<&str> {
        match &self.backend {
            AsusWmiBackend::Desktop { instance_path } => Ok(instance_path),
            _ => Err(NoCrateError::Wmi("当前后端不是 Desktop".into())),
        }
    }

    /// 测试 asio_hw_fun* 方法的可用性，返回诊断结果。
    ///
    /// 依次调用 fun07（I/O 端口读）、fun21（SIO Bank+Index 读）、
    /// fun19（SIO LDN 寄存器读），并报告每个的返回值。
    pub fn test_asio_hw_fun(&self) -> Result<Vec<(String, Result<u32>)>> {
        let path = self.desktop_path()?;
        let mut results = Vec::new();

        // --- fun07: 读 I/O 端口字节 ---
        // 读 0x2E（SIO config port，应返回非零值如果 SIO 存在）
        let label = "fun07(wPort=0x2E)".to_string();
        let r = self
            .exec_method_v2(path, "asio_hw_fun07", &[("wPort", WmiParam::U32(0x2E))])
            .and_then(|out| Self::get_property_u32(&out, "bData"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // 读 0x0295（Nuvoton ISA addr port）
        let label = "fun07(wPort=0x0295)".to_string();
        let r = self
            .exec_method_v2(path, "asio_hw_fun07", &[("wPort", WmiParam::U32(0x0295))])
            .and_then(|out| Self::get_property_u32(&out, "bData"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // 读 0x61（NMI 状态端口，通常有值）
        let label = "fun07(wPort=0x61)".to_string();
        let r = self
            .exec_method_v2(path, "asio_hw_fun07", &[("wPort", WmiParam::U32(0x61))])
            .and_then(|out| Self::get_property_u32(&out, "bData"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // --- fun21: 按 Bank+Index 读 HW Monitor 寄存器 ---
        // Bank 0, Index 0x4F = Vendor ID（期望 0x5C = Nuvoton）
        let label = "fun21(Bank=0, Index=0x4F) [Vendor ID]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun21",
                &[("Bank", WmiParam::U8(0)), ("Index", WmiParam::U8(0x4F))],
            )
            .and_then(|out| Self::get_property_u32(&out, "Data"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // Bank 0, Index 0x27 = SYSTIN temp
        let label = "fun21(Bank=0, Index=0x27) [SYSTIN]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun21",
                &[("Bank", WmiParam::U8(0)), ("Index", WmiParam::U8(0x27))],
            )
            .and_then(|out| Self::get_property_u32(&out, "Data"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // Bank 4, Index 0xC0/0xC1 = Fan 0 tach
        let label = "fun21(Bank=4, Index=0xC0) [Fan0 high]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun21",
                &[("Bank", WmiParam::U8(4)), ("Index", WmiParam::U8(0xC0))],
            )
            .and_then(|out| Self::get_property_u32(&out, "Data"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        let label = "fun21(Bank=4, Index=0xC1) [Fan0 low]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun21",
                &[("Bank", WmiParam::U8(4)), ("Index", WmiParam::U8(0xC1))],
            )
            .and_then(|out| Self::get_property_u32(&out, "Data"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // --- fun19: 读 SIO LDN 寄存器 ---
        // LDN 0x0B (HW monitor), Index 0x20 = Chip ID high（期望 0xD8）
        let label = "fun19(LDN=0x0B, Index=0x20) [ChipID high]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun19",
                &[("LDN", WmiParam::U8(0x0B)), ("Index", WmiParam::U8(0x20))],
            )
            .and_then(|out| Self::get_property_u32(&out, "Data"));
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        // --- fun23: 批量读 Bank+Index ---
        let label = "fun23('00,4F') [Vendor ID batch]".to_string();
        let r = self
            .exec_method_v2(
                path,
                "asio_hw_fun23",
                &[("BankIndexArray", WmiParam::Str("00,4F"))],
            )
            .and_then(|out| Self::get_property_string(&out, "DataArray"))
            .map(|s| {
                eprintln!("[WMI-TEST] fun23 DataArray raw: '{s}'");
                // 尝试解析返回的字符串
                s.parse::<u32>().unwrap_or(0xDEAD)
            });
        eprintln!("[WMI-TEST] {label}: {:?}", r);
        results.push((label, r));

        Ok(results)
    }
}

impl Drop for WmiConnection {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
