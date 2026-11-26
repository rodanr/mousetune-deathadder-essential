// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed builders for every command the app sends. Nothing outside this
//! module constructs raw protocol bytes, and no undocumented commands exist.

use crate::report::{ARGS_LEN, Report};
use crate::types::{Brightness, DpiPair, Effect, Led, PollingRate};

/// Transaction id used for info, DPI and polling commands.
const TRANSACTION_DEFAULT: u8 = 0xFF;
/// Transaction id used for LED commands.
const TRANSACTION_LED: u8 = 0x3F;

/// Storage flag for LED commands: persist on the device.
const VARSTORE: u8 = 0x01;
/// Storage flag for DPI reads.
const NOSTORE: u8 = 0x00;

/// Green is the only colour these LEDs can show.
const GREEN: [u8; 3] = [0x00, 0xFF, 0x00];

/// Effect ids of the extended matrix effect command.
const EFFECT_STATIC: u8 = 0x01;
const EFFECT_BREATHE: u8 = 0x02;

/// A protocol command: header fields plus argument bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    transaction_id: u8,
    class: u8,
    id: u8,
    data_size: u8,
    args: [u8; ARGS_LEN],
}

impl Command {
    /// The fixed-size array makes an oversized command unrepresentable.
    fn new<const N: usize>(
        transaction_id: u8,
        class: u8,
        id: u8,
        data_size: u8,
        args: [u8; N],
    ) -> Self {
        const { assert!(N <= ARGS_LEN) };
        debug_assert!(N <= usize::from(data_size));
        let mut padded = [0; ARGS_LEN];
        padded[..N].copy_from_slice(&args);
        Self {
            transaction_id,
            class,
            id,
            data_size,
            args: padded,
        }
    }

    /// Command class byte.
    #[must_use]
    pub fn class(&self) -> u8 {
        self.class
    }

    /// Command id byte.
    #[must_use]
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Builds the request report for this command.
    #[must_use]
    pub fn to_report(&self) -> Report {
        Report {
            status: 0x00,
            transaction_id: self.transaction_id,
            data_size: self.data_size,
            command_class: self.class,
            command_id: self.id,
            args: self.args,
        }
    }

    /// Reads the firmware version (`major.minor` in reply args 0 and 1).
    #[must_use]
    pub fn get_firmware_version() -> Self {
        Self::new(TRANSACTION_DEFAULT, 0x00, 0x81, 0x02, [])
    }

    /// Reads the serial number (ASCII, NUL terminated, in reply args).
    #[must_use]
    pub fn get_serial() -> Self {
        Self::new(TRANSACTION_DEFAULT, 0x00, 0x82, 0x16, [])
    }

    /// Sets X and Y DPI.
    #[must_use]
    pub fn set_dpi(dpi: DpiPair) -> Self {
        let [x_hi, x_lo] = dpi.x.get().to_be_bytes();
        let [y_hi, y_lo] = dpi.y.get().to_be_bytes();
        Self::new(
            TRANSACTION_DEFAULT,
            0x04,
            0x05,
            0x07,
            [VARSTORE, x_hi, x_lo, y_hi, y_lo, 0x00, 0x00],
        )
    }

    /// Reads X and Y DPI.
    #[must_use]
    pub fn get_dpi() -> Self {
        Self::new(TRANSACTION_DEFAULT, 0x04, 0x85, 0x07, [NOSTORE])
    }

    /// Sets the polling rate.
    #[must_use]
    pub fn set_polling_rate(rate: PollingRate) -> Self {
        Self::new(TRANSACTION_DEFAULT, 0x00, 0x05, 0x01, [rate.to_byte()])
    }

    /// Reads the polling rate.
    #[must_use]
    pub fn get_polling_rate() -> Self {
        Self::new(TRANSACTION_DEFAULT, 0x00, 0x85, 0x01, [])
    }

    /// Sets an LED's brightness.
    #[must_use]
    pub fn set_brightness(led: Led, brightness: Brightness) -> Self {
        Self::new(
            TRANSACTION_LED,
            0x0F,
            0x04,
            0x03,
            [VARSTORE, led.id(), brightness.0],
        )
    }

    /// Reads an LED's brightness (reply arg 2).
    #[must_use]
    pub fn get_brightness(led: Led) -> Self {
        Self::new(TRANSACTION_LED, 0x0F, 0x84, 0x03, [VARSTORE, led.id()])
    }

    /// Sets an LED effect. Colour is always green.
    #[must_use]
    pub fn set_effect(led: Led, effect: Effect) -> Self {
        let led = led.id();
        match effect {
            Effect::Off => Self::new(
                TRANSACTION_LED,
                0x0F,
                0x02,
                0x06,
                [VARSTORE, led, 0, 0, 0, 0],
            ),
            Effect::Static => Self::colour_effect(led, EFFECT_STATIC, 0x00),
            Effect::Breathe => Self::colour_effect(led, EFFECT_BREATHE, 0x01),
        }
    }

    fn colour_effect(led: u8, effect: u8, parameter: u8) -> Self {
        let [r, g, b] = GREEN;
        Self::new(
            TRANSACTION_LED,
            0x0F,
            0x02,
            0x09,
            [VARSTORE, led, effect, parameter, 0x00, 0x01, r, g, b],
        )
    }
}
