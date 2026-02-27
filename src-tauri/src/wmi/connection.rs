/// WMI COM connection management.
///
/// Manages the lifecycle of the COM connection to the `root\WMI` namespace,
/// providing access to `IWbemServices` for invoking ASUS WMI methods.
///
/// Supports two ASUS WMI backends:
///
/// - **Laptop** (`ASUSATKWMI_WMNB`): Uses `DSTS` / `DEVS` methods with
///   `Device_ID` / `Device_Status` / `Control_Status` parameters.
///   Instance path: `ASUSATKWMI_WMNB.InstanceName='ACPI\\ATK0110\\0_0'`.
///
/// - **Desktop** (`ASUSManagement`): Uses `device_status` / `device_ctrl`
///   methods with `device_id` / `ctrl_param` parameters.
///   Instance path: discovered at runtime via WQL enumeration.
use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator,
    WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_FLAG_RETURN_WBEM_COMPLETE,
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
}

impl AsusWmiBackend {
    /// Human-readable label for log output.
    pub fn label(&self) -> &str {
        match self {
            Self::Laptop { .. } => "Laptop (ASUSATKWMI_WMNB)",
            Self::Desktop { .. } => "Desktop (ASUSManagement)",
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

            // Set default security levels
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

            // Auto-detect backend
            let backend = Self::detect_backend(&services)?;
            eprintln!("WMI backend detected: {}", backend.label());

            Ok(Self { services, backend })
        }
    }

    /// Probe available ASUS WMI classes and return the first working backend.
    #[allow(unsafe_code)]
    unsafe fn detect_backend(services: &IWbemServices) -> Result<AsusWmiBackend> {
        // 1. Try desktop: ASUSManagement (enumerate instances via WQL)
        if let Ok(path) = Self::find_first_instance(services, "ASUSManagement") {
            return Ok(AsusWmiBackend::Desktop {
                instance_path: path,
            });
        }

        // 2. Try laptop: ASUSATKWMI_WMNB with common instance path
        let laptop_path = "ASUSATKWMI_WMNB.InstanceName='ACPI\\\\ATK0110\\\\0_0'";
        // Verify the class exists by trying GetObject on the class name
        let mut obj = None;
        let laptop_ok = services
            .GetObject(
                &BSTR::from("ASUSATKWMI_WMNB"),
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut obj),
                None,
            )
            .is_ok();
        if laptop_ok && obj.is_some() {
            return Ok(AsusWmiBackend::Laptop {
                instance_path: laptop_path.to_string(),
            });
        }

        Err(NoCrateError::Wmi(
            "未找到支持的 ASUS WMI 接口 (ASUSATKWMI_WMNB / ASUSManagement)".into(),
        ))
    }

    /// Run a WQL query to find the first instance of a class and return its
    /// `__RELPATH` (relative object path).
    #[allow(unsafe_code)]
    unsafe fn find_first_instance(
        services: &IWbemServices,
        class_name: &str,
    ) -> Result<String> {
        let query = format!("SELECT * FROM {class_name}");
        let enumerator = services.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from(query.as_str()),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )?;

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
    pub fn dsts(&self, device_id: u32) -> Result<u32> {
        match &self.backend {
            AsusWmiBackend::Laptop { instance_path } => {
                let out =
                    self.exec_method(instance_path, "DSTS", &[("Device_ID", device_id)])?;
                Self::get_property_u32(&out, "Device_Status")
            }
            AsusWmiBackend::Desktop { instance_path } => {
                let out =
                    self.exec_method(instance_path, "device_status", &[("device_id", device_id)])?;
                Self::get_property_u32(&out, "ctrl_param")
            }
        }
    }

    /// Write a device control value (equivalent to laptop `DEVS`).
    ///
    /// - **Laptop**: calls `DEVS(Device_ID, Control_Status)` → `Device_Status`
    /// - **Desktop**: calls `device_ctrl(device_id, ctrl_param)` → (void)
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
        }
    }

    /// Read a u32 value from a WMI class object property.
    #[allow(unsafe_code)]
    pub fn get_property_u32(obj: &IWbemClassObject, name: &str) -> Result<u32> {
        unsafe {
            let mut val = VARIANT::default();
            obj.Get(&BSTR::from(name), 0, &mut val, None, None)?;

            // Extract i4 (i32) from variant and cast to u32
            let i4: i32 = (&val).try_into().map_err(|_| {
                NoCrateError::Wmi(format!("Property {name} is not an i4/u32 value"))
            })?;
            Ok(i4 as u32)
        }
    }

    /// Read a string value from a WMI class object property.
    #[allow(unsafe_code)]
    pub fn get_property_string(obj: &IWbemClassObject, name: &str) -> Result<String> {
        unsafe {
            let mut val = VARIANT::default();
            obj.Get(&BSTR::from(name), 0, &mut val, None, None)?;

            let bstr: BSTR = (&val).try_into().map_err(|_| {
                NoCrateError::Wmi(format!("Property {name} is not a string value"))
            })?;
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
                NoCrateError::Wmi(format!(
                    "ExecMethod returned no output for {method_name}"
                ))
            })
        }
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
