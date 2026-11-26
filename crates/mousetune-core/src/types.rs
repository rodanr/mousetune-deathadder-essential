// SPDX-License-Identifier: GPL-3.0-or-later
//! Validated domain types. Invalid values cannot be constructed, so command
//! builders never need to range-check.

use crate::error::{Error, Result};

/// A DPI value in the supported 100–6400 range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dpi(u16);

impl Dpi {
    /// Lowest supported DPI.
    pub const MIN: u16 = 100;
    /// Highest supported DPI.
    pub const MAX: u16 = 6400;

    /// Creates a DPI value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DpiOutOfRange`] outside `100..=6400`.
    pub fn new(value: u16) -> Result<Self> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::DpiOutOfRange(value))
        }
    }

    /// The raw DPI value.
    #[must_use]
    pub fn get(self) -> u16 {
        self.0
    }
}

/// Independent X and Y sensor resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpiPair {
    /// Horizontal DPI.
    pub x: Dpi,
    /// Vertical DPI.
    pub y: Dpi,
}

impl DpiPair {
    /// Same DPI on both axes.
    #[must_use]
    pub fn uniform(dpi: Dpi) -> Self {
        Self { x: dpi, y: dpi }
    }
}

/// USB polling rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollingRate {
    /// 125 Hz.
    Hz125,
    /// 500 Hz.
    Hz500,
    /// 1000 Hz.
    Hz1000,
}

impl PollingRate {
    /// All rates, slowest first.
    pub const ALL: [Self; 3] = [Self::Hz125, Self::Hz500, Self::Hz1000];

    /// The rate in hertz.
    #[must_use]
    pub fn hertz(self) -> u16 {
        match self {
            Self::Hz125 => 125,
            Self::Hz500 => 500,
            Self::Hz1000 => 1000,
        }
    }

    /// Wire encoding used by the device.
    pub(crate) fn to_byte(self) -> u8 {
        match self {
            Self::Hz1000 => 0x01,
            Self::Hz500 => 0x02,
            Self::Hz125 => 0x08,
        }
    }

    /// Decodes the wire encoding, if known.
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Hz1000),
            0x02 => Some(Self::Hz500),
            0x08 => Some(Self::Hz125),
            _ => None,
        }
    }
}

/// An LED zone on the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Led {
    /// The logo LED.
    Logo,
    /// The scroll wheel LED.
    Scroll,
}

impl Led {
    /// Wire identifier of the LED.
    pub(crate) fn id(self) -> u8 {
        match self {
            Self::Logo => 0x04,
            Self::Scroll => 0x01,
        }
    }
}

/// Lighting effect. The LEDs are green only, so no colour is exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// LED off.
    Off,
    /// Constant light.
    Static,
    /// Slow pulsing.
    Breathe,
}

/// LED brightness, `0..=255` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brightness(pub u8);

impl Brightness {
    /// Converts a percentage, clamping values above 100.
    #[must_use]
    pub fn from_percent(percent: u8) -> Self {
        let percent = u16::from(percent.min(100));
        // 100 * 255 / 100 = 255, so the result always fits in a u8.
        Self(u8::try_from((percent * 255 + 50) / 100).unwrap_or(u8::MAX))
    }

    /// The brightness as a rounded percentage.
    #[must_use]
    pub fn percent(self) -> u8 {
        u8::try_from((u16::from(self.0) * 100 + 127) / 255).unwrap_or(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_accepts_bounds_and_rejects_outside() {
        assert!(Dpi::new(100).is_ok());
        assert!(Dpi::new(6400).is_ok());
        assert_eq!(Dpi::new(99), Err(Error::DpiOutOfRange(99)));
        assert_eq!(Dpi::new(6401), Err(Error::DpiOutOfRange(6401)));
        assert_eq!(Dpi::new(0), Err(Error::DpiOutOfRange(0)));
    }

    #[test]
    fn polling_rate_byte_mapping_round_trips() {
        for rate in PollingRate::ALL {
            assert_eq!(PollingRate::from_byte(rate.to_byte()), Some(rate));
        }
        assert_eq!(PollingRate::Hz1000.to_byte(), 0x01);
        assert_eq!(PollingRate::Hz500.to_byte(), 0x02);
        assert_eq!(PollingRate::Hz125.to_byte(), 0x08);
        assert_eq!(PollingRate::from_byte(0x03), None);
    }

    #[test]
    fn brightness_percent_conversion() {
        assert_eq!(Brightness::from_percent(0), Brightness(0));
        assert_eq!(Brightness::from_percent(100), Brightness(255));
        assert_eq!(Brightness::from_percent(250), Brightness(255));
        for percent in 0..=100 {
            assert_eq!(Brightness::from_percent(percent).percent(), percent);
        }
    }
}
