/// High-level AURA controller interface.
///
/// Wraps a HID device handle and provides typed methods for setting
/// effects and per-LED colours on ASUS motherboard AURA controllers.
use hidapi::{HidApi, HidDevice};
use serde::Serialize;

use crate::error::{NoCrateError, Result};

use super::protocol::{
    self, AuraEffect, AuraSpeed, RgbColor, AURA_MB_PIDS, AURA_VID, MAX_LEDS_PER_PACKET,
};

/// Information about a discovered AURA device.
#[derive(Debug, Clone, Serialize)]
pub struct AuraDeviceInfo {
    /// USB Product ID.
    pub pid: u16,
    /// Product name reported by the device (may be empty).
    pub product: String,
}

/// Handle to an open ASUS AURA controller.
///
/// Holds both the `HidApi` (keeps the library alive) and the open
/// `HidDevice`. The device is closed on drop.
pub struct AuraController {
    device: HidDevice,
    _api: HidApi,
    info: AuraDeviceInfo,
}

// HidDevice is Send but not Sync. We protect access with a Mutex
// in AppState, so this is safe.
#[allow(unsafe_code)]
unsafe impl Sync for AuraController {}

impl AuraController {
    /// Enumerate USB HID devices and open the first matching AURA
    /// motherboard controller.
    ///
    /// # Errors
    ///
    /// Returns `Hid` error if no AURA controller is found or the
    /// device cannot be opened.
    pub fn discover() -> Result<Self> {
        let api = HidApi::new()?;

        for &pid in AURA_MB_PIDS {
            if let Ok(device) = api.open(AURA_VID, pid) {
                let product = device
                    .get_product_string()
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                let info = AuraDeviceInfo {
                    pid,
                    product: product.clone(),
                };

                return Ok(Self {
                    device,
                    _api: api,
                    info,
                });
            }
        }

        Err(NoCrateError::Hid(
            "No AURA motherboard controller found. Checked PIDs: [{}]".replace(
                "[{}]",
                &AURA_MB_PIDS
                    .iter()
                    .map(|p| format!("0x{p:04X}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ))
    }

    /// Information about the connected device.
    #[must_use]
    pub fn info(&self) -> &AuraDeviceInfo {
        &self.info
    }

    // ── Effect mode ──────────────────────────────────────────

    /// Set an effect mode with a base colour and speed.
    ///
    /// Automatically sends a commit after the effect packet.
    pub fn set_effect(&self, effect: AuraEffect, color: RgbColor, speed: AuraSpeed) -> Result<()> {
        let report = protocol::build_set_effect(effect, color, speed);
        self.write(&report)?;

        // Commit
        let commit = protocol::build_commit();
        self.write(&commit)?;

        Ok(())
    }

    /// Convenience: set a solid static colour on all LEDs.
    pub fn set_static_color(&self, color: RgbColor) -> Result<()> {
        self.set_effect(AuraEffect::Static, color, AuraSpeed::Medium)
    }

    /// Turn all LEDs off.
    pub fn turn_off(&self) -> Result<()> {
        self.set_effect(AuraEffect::Off, RgbColor::BLACK, AuraSpeed::Medium)
    }

    // ── Direct per-LED control ───────────────────────────────

    /// Set individual LED colours in direct mode.
    ///
    /// Automatically batches into multiple HID packets if there are
    /// more LEDs than [`MAX_LEDS_PER_PACKET`].
    pub fn set_direct_colors(&self, colors: &[RgbColor]) -> Result<()> {
        for (chunk_idx, chunk) in colors.chunks(MAX_LEDS_PER_PACKET).enumerate() {
            let start = (chunk_idx * MAX_LEDS_PER_PACKET) as u8;
            let report = protocol::build_direct(start, chunk);
            self.write(&report)?;
        }
        Ok(())
    }

    // ── Internal I/O ─────────────────────────────────────────

    fn write(&self, report: &[u8]) -> Result<()> {
        let _ = self.device
            .write(report)
            .map_err(|e| NoCrateError::Hid(format!("HID write failed: {e}")))?;
        Ok(())
    }
}
