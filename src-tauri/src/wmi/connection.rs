/// WMI COM connection management.
///
/// Manages the lifecycle of the COM connection to the `root\WMI` namespace,
/// providing access to `IWbemServices` for invoking ASUS WMI methods.

use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE,
    RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator,
    WBEM_FLAG_RETURN_WBEM_COMPLETE,
};

use crate::error::{NoCrateError, Result};

/// RAII wrapper around a WMI connection to `root\WMI`.
///
/// COM is initialized on construction and cleaned up on drop.
/// This struct is **not** `Send`/`Sync` — it must live on the thread
/// that created it. Use the `WmiThread` helper for cross-thread access.
pub struct WmiConnection {
    services: IWbemServices,
}

impl WmiConnection {
    /// Initialize COM and connect to `root\WMI`.
    ///
    /// # Safety
    ///
    /// Calls COM initialization functions. Must be called from a thread that
    /// has not already initialized COM with an incompatible threading model.
    ///
    /// # Errors
    ///
    /// Returns `NoCrateError::WindowsApi` if any COM/WMI call fails.
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
            ).ok();

            // Create WMI locator
            let locator: IWbemLocator =
                CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

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

            Ok(Self { services })
        }
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
            // Get class definition to obtain method signature
            let class_obj = self.get_object(object_path)?;

            // Get input parameter definition for the method
            let mut in_params_def = None;
            class_obj.GetMethod(
                &BSTR::from(method_name),
                0,
                &mut in_params_def,
                &mut None,
            )?;

            let in_params = match in_params_def {
                Some(def) => {
                    let instance = def.SpawnInstance(0)?;
                    // Fill in parameters
                    for &(name, value) in params {
                        let variant = VARIANT::from(i32::try_from(value).unwrap_or(value as i32));
                        instance.Put(
                            &BSTR::from(name),
                            0,
                            &variant,
                            0,
                        )?;
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
}

impl Drop for WmiConnection {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
