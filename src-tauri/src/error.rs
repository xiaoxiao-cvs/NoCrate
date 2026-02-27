use serde::Serialize;
use thiserror::Error;

/// Unified error type for NoCrate operations.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum NoCrateError {
    #[error("WMI error: {0}")]
    Wmi(String),

    #[error("Windows API error: HRESULT 0x{0:08X}")]
    WindowsApi(u32),

    #[error("HID error: {0}")]
    Hid(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Convenience Result type alias with `NoCrateError`.
pub type Result<T> = std::result::Result<T, NoCrateError>;

impl From<windows::core::Error> for NoCrateError {
    fn from(err: windows::core::Error) -> Self {
        Self::WindowsApi(err.code().0 as u32)
    }
}

impl From<NoCrateError> for String {
    fn from(err: NoCrateError) -> Self {
        err.to_string()
    }
}
