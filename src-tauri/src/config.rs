use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{NoCrateError, Result};

/// Global config file path, set once during app setup.
static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Application configuration persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// User-selected theme: "light" | "dark" | "system"
    pub theme: String,

    /// Whether to minimize to system tray on close
    pub close_to_tray: bool,

    /// Whether to launch at system startup
    pub auto_start: bool,

    /// Fan polling interval in milliseconds
    pub fan_poll_interval_ms: u64,

    /// Last selected thermal profile index (0=Standard, 1=Performance, 2=Silent)
    pub last_thermal_profile: u8,

    /// Last selected AURA effect name
    pub last_aura_effect: String,

    /// Last selected AURA color as hex (#RRGGBB)
    pub last_aura_color: String,

    /// Last selected AURA speed: "slow" | "medium" | "fast"
    pub last_aura_speed: String,

    /// Whether temperature threshold alerts are enabled
    pub temp_alert_enabled: bool,

    /// Temperature threshold in °C for alerts
    pub temp_alert_threshold: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            close_to_tray: false,
            auto_start: false,
            fan_poll_interval_ms: 2000,
            last_thermal_profile: 0,
            last_aura_effect: "static".into(),
            last_aura_color: "#ff0000".into(),
            last_aura_speed: "medium".into(),
            temp_alert_enabled: true,
            temp_alert_threshold: 90,
        }
    }
}

/// Thread-safe configuration store with automatic persistence.
pub struct ConfigStore {
    inner: RwLock<AppConfig>,
}

impl ConfigStore {
    /// Initialize the config store, loading from disk or creating defaults.
    pub fn init(app_data_dir: PathBuf) -> Result<Self> {
        let config_file = app_data_dir.join("config.json");

        // Ensure the directory exists
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NoCrateError::Config(format!("Failed to create config directory: {e}"))
            })?;
        }

        CONFIG_PATH
            .set(config_file.clone())
            .map_err(|_| NoCrateError::Config("Config path already initialized".into()))?;

        let config = if config_file.exists() {
            let data = fs::read_to_string(&config_file)
                .map_err(|e| NoCrateError::Config(format!("Failed to read config file: {e}")))?;
            serde_json::from_str(&data).unwrap_or_else(|e| {
                eprintln!("Warning: config parse error ({e}), using defaults");
                AppConfig::default()
            })
        } else {
            let default = AppConfig::default();
            // Write default config to disk
            let _ = Self::write_to_disk(&default);
            default
        };

        Ok(Self {
            inner: RwLock::new(config),
        })
    }

    /// Read the full config snapshot.
    pub fn get(&self) -> AppConfig {
        self.inner.read().clone()
    }

    /// Update config via a closure and persist to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to disk fails.
    pub fn update<F>(&self, f: F) -> Result<AppConfig>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut guard = self.inner.write();
        f(&mut guard);
        Self::write_to_disk(&guard)?;
        Ok(guard.clone())
    }

    /// Write config to disk.
    fn write_to_disk(config: &AppConfig) -> Result<()> {
        let Some(path) = CONFIG_PATH.get() else {
            return Err(NoCrateError::Config("Config path not initialized".into()));
        };

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| NoCrateError::Config(format!("Failed to serialize config: {e}")))?;

        fs::write(path, json)
            .map_err(|e| NoCrateError::Config(format!("Failed to write config file: {e}")))?;

        Ok(())
    }
}
