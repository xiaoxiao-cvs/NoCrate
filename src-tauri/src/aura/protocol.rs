/// ASUS AURA USB HID protocol constants and packet building.
///
/// Protocol details sourced from OpenRGB and community reverse-engineering.
/// The exact command layout may vary between controller firmware versions;
/// constants here target the most common ENE-based ASUS motherboard
/// controllers (VID 0x0B05).
use serde::{Deserialize, Serialize};

// ─── Device Identification ───────────────────────────────────

/// ASUS USB Vendor ID.
pub const AURA_VID: u16 = 0x0B05;

/// Known ASUS AURA motherboard controller Product IDs.
///
/// This list covers common desktop motherboard controllers. Boards
/// not listed may still work if they use the same protocol.
pub const AURA_MB_PIDS: &[u16] = &[
    0x1867, // AURA LED Controller (ENE)
    0x1869, // AURA Mainboard (older)
    0x18F3, // AURA Mainboard (newer)
    0x19AF, // AURA Addressable Gen 2
    0x1939, // ROG STRIX series
    0x1854, // PRIME / TUF series
];

// ─── HID Report ──────────────────────────────────────────────

/// Total HID report size: 1 byte Report ID + 64 bytes payload.
pub const REPORT_SIZE: usize = 65;

// ─── Command Bytes ───────────────────────────────────────────

/// Set an effect mode on a channel, or commit (with 0xFF sub-command).
pub const CMD_SET_EFFECT: u8 = 0x35;

/// Direct per-LED color control.
pub const CMD_DIRECT: u8 = 0x36;

/// Query firmware version.
pub const CMD_FIRMWARE: u8 = 0xB0;

/// Maximum LEDs addressable in a single direct-mode packet.
///
/// Each LED consumes 3 bytes (R, G, B).
/// Available payload after header bytes ≈ 60 → 20 LEDs.
pub const MAX_LEDS_PER_PACKET: usize = 20;

// ─── Effect Modes ────────────────────────────────────────────

/// Predefined AURA lighting effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraEffect {
    /// LEDs off.
    Off,
    /// Solid single color.
    Static,
    /// Pulse between color and off.
    Breathing,
    /// Cycle through the color wheel.
    ColorCycle,
    /// Flowing rainbow across LEDs.
    Rainbow,
    /// Full-spectrum sweep.
    SpectrumCycle,
}

impl AuraEffect {
    /// All effects, useful for UI enumeration.
    pub const ALL: [Self; 6] = [
        Self::Off,
        Self::Static,
        Self::Breathing,
        Self::ColorCycle,
        Self::Rainbow,
        Self::SpectrumCycle,
    ];

    /// Map to the protocol byte sent in the HID report.
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        match self {
            Self::Off => 0x00,
            Self::Static => 0x01,
            Self::Breathing => 0x02,
            Self::ColorCycle => 0x03,
            Self::Rainbow => 0x04,
            Self::SpectrumCycle => 0x05,
        }
    }

    /// Parse from a raw protocol byte.
    #[must_use]
    pub fn from_raw(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Off),
            0x01 => Some(Self::Static),
            0x02 => Some(Self::Breathing),
            0x03 => Some(Self::ColorCycle),
            0x04 => Some(Self::Rainbow),
            0x05 => Some(Self::SpectrumCycle),
            _ => None,
        }
    }
}

/// Effect speed preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraSpeed {
    Slow,
    Medium,
    Fast,
}

impl AuraSpeed {
    #[must_use]
    pub const fn to_raw(self) -> u8 {
        match self {
            Self::Slow => 0x03,
            Self::Medium => 0x02,
            Self::Fast => 0x01,
        }
    }
}

// ─── RGB Color ───────────────────────────────────────────────

/// An RGB colour value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// ─── Packet Builders ─────────────────────────────────────────

/// Build a blank 65-byte HID report and fill command + payload.
#[must_use]
pub fn build_report(cmd: u8, payload: &[u8]) -> [u8; REPORT_SIZE] {
    let mut buf = [0u8; REPORT_SIZE];
    buf[0] = 0x00; // Report ID
    buf[1] = cmd;
    let n = payload.len().min(REPORT_SIZE - 2);
    buf[2..2 + n].copy_from_slice(&payload[..n]);
    buf
}

/// Build a "set effect" report for channel 0.
#[must_use]
pub fn build_set_effect(
    effect: AuraEffect,
    color: RgbColor,
    speed: AuraSpeed,
) -> [u8; REPORT_SIZE] {
    build_report(
        CMD_SET_EFFECT,
        &[
            0x00, // channel 0
            effect.to_raw(),
            color.r,
            color.g,
            color.b,
            speed.to_raw(),
            0x00, // direction: default
        ],
    )
}

/// Build a "commit" report to apply the last effect change.
#[must_use]
pub fn build_commit() -> [u8; REPORT_SIZE] {
    build_report(CMD_SET_EFFECT, &[0xFF])
}

/// Build a "direct color" report for a slice of LEDs.
///
/// `start_led` is the zero-based LED index.
/// Up to [`MAX_LEDS_PER_PACKET`] LEDs in one report.
#[must_use]
pub fn build_direct(start_led: u8, colors: &[RgbColor]) -> [u8; REPORT_SIZE] {
    let count = colors.len().min(MAX_LEDS_PER_PACKET);
    // Payload: [start, count, R, G, B, R, G, B, …]
    let mut payload = Vec::with_capacity(2 + count * 3);
    payload.push(start_led);
    payload.push(count as u8);
    for c in &colors[..count] {
        payload.extend_from_slice(&[c.r, c.g, c.b]);
    }
    build_report(CMD_DIRECT, &payload)
}

/// Build a firmware-query report.
#[must_use]
pub fn build_firmware_query() -> [u8; REPORT_SIZE] {
    build_report(CMD_FIRMWARE, &[])
}
