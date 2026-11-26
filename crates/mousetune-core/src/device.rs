// SPDX-License-Identifier: GPL-3.0-or-later
//! High level mouse API on top of a [`HidTransport`].

use std::time::Duration;

use crate::command::Command;
use crate::error::{Error, Result};
use crate::model::Model;
use crate::report::Report;
use crate::transport::HidTransport;
use crate::types::{Brightness, Dpi, DpiPair, Effect, Led, PollingRate};

/// Pause between writing a command and reading the reply.
const REPLY_DELAY: Duration = Duration::from_millis(2);
/// Pause between polls while the device reports "busy".
const BUSY_DELAY: Duration = Duration::from_millis(10);
/// How many times a busy reply is polled again before giving up.
const BUSY_RETRIES: u32 = 5;

const STATUS_BUSY: u8 = 0x01;
const STATUS_OK: u8 = 0x02;
const STATUS_FAILURE: u8 = 0x03;
const STATUS_TIMEOUT: u8 = 0x04;
const STATUS_NOT_SUPPORTED: u8 = 0x05;

/// Length of the serial number field in a reply.
const SERIAL_LEN: usize = 22;

type Sleeper = Box<dyn FnMut(Duration) + Send>;

/// A connected mouse.
pub struct RazerDevice<T> {
    transport: T,
    model: Model,
    sleep: Sleeper,
}

impl<T: HidTransport> RazerDevice<T> {
    /// Wraps a transport that talks to a mouse of the given model.
    #[must_use]
    pub fn new(transport: T, model: Model) -> Self {
        Self::with_sleeper(transport, model, std::thread::sleep)
    }

    /// Like [`RazerDevice::new`] with an injectable sleep function, so tests
    /// run without real delays.
    #[must_use]
    pub fn with_sleeper(
        transport: T,
        model: Model,
        sleep: impl FnMut(Duration) + Send + 'static,
    ) -> Self {
        Self {
            transport,
            model,
            sleep: Box::new(sleep),
        }
    }

    /// The model this device was opened as.
    #[must_use]
    pub fn model(&self) -> Model {
        self.model
    }

    /// Sends a command and returns the validated reply.
    ///
    /// A "busy" reply is polled again (without resending) up to five times,
    /// 10 ms apart.
    ///
    /// # Errors
    ///
    /// Returns an error for transport failures, non-OK status codes, or a
    /// reply that does not answer the request.
    pub fn execute(&mut self, command: &Command) -> Result<Report> {
        self.transport
            .send_feature_report(&command.to_report().encode())?;
        (self.sleep)(REPLY_DELAY);

        let mut busy_retries = 0;
        loop {
            let reply = Report::decode(&self.transport.get_feature_report()?)?;
            match reply.status {
                STATUS_OK => {
                    return if reply.command_class == command.class()
                        && reply.command_id == command.id()
                    {
                        Ok(reply)
                    } else {
                        Err(Error::UnexpectedReply)
                    };
                }
                STATUS_BUSY if busy_retries < BUSY_RETRIES => {
                    busy_retries += 1;
                    (self.sleep)(BUSY_DELAY);
                }
                STATUS_BUSY => return Err(Error::DeviceBusy),
                STATUS_FAILURE => return Err(Error::CommandFailed),
                STATUS_TIMEOUT => return Err(Error::Timeout),
                STATUS_NOT_SUPPORTED => return Err(Error::NotSupported),
                other => return Err(Error::UnknownStatus(other)),
            }
        }
    }

    /// Firmware version as `(major, minor)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    pub fn firmware_version(&mut self) -> Result<(u8, u8)> {
        let reply = self.execute(&Command::get_firmware_version())?;
        Ok((reply.args[0], reply.args[1]))
    }

    /// Serial number.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    pub fn serial(&mut self) -> Result<String> {
        let reply = self.execute(&Command::get_serial())?;
        let field = &reply.args[..SERIAL_LEN];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(field.len());
        Ok(field[..end]
            .iter()
            .filter(|byte| byte.is_ascii_graphic())
            .map(|&b| char::from(b))
            .collect())
    }

    /// Current X/Y DPI.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails or the mouse reports a DPI
    /// outside the supported range.
    pub fn dpi(&mut self) -> Result<DpiPair> {
        let reply = self.execute(&Command::get_dpi())?;
        let axis = |hi: u8, lo: u8| Dpi::new(u16::from_be_bytes([hi, lo]));
        Ok(DpiPair {
            x: axis(reply.args[1], reply.args[2])?,
            y: axis(reply.args[3], reply.args[4])?,
        })
    }

    /// Sets X/Y DPI.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    pub fn set_dpi(&mut self, dpi: DpiPair) -> Result<()> {
        self.execute(&Command::set_dpi(dpi)).map(drop)
    }

    /// Current polling rate.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails or the rate is unknown.
    pub fn polling_rate(&mut self) -> Result<PollingRate> {
        let reply = self.execute(&Command::get_polling_rate())?;
        PollingRate::from_byte(reply.args[0]).ok_or(Error::UnexpectedValue)
    }

    /// Sets the polling rate.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    pub fn set_polling_rate(&mut self, rate: PollingRate) -> Result<()> {
        self.execute(&Command::set_polling_rate(rate)).map(drop)
    }

    /// Current brightness of an LED.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LedNotAvailable`] if the model lacks the LED, or an
    /// error if the command fails.
    pub fn brightness(&mut self, led: Led) -> Result<Brightness> {
        self.require_led(led)?;
        let reply = self.execute(&Command::get_brightness(led))?;
        Ok(Brightness(reply.args[2]))
    }

    /// Sets the brightness of an LED.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LedNotAvailable`] if the model lacks the LED, or an
    /// error if the command fails.
    pub fn set_brightness(&mut self, led: Led, brightness: Brightness) -> Result<()> {
        self.require_led(led)?;
        self.execute(&Command::set_brightness(led, brightness))
            .map(drop)
    }

    /// Sets the effect of an LED.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LedNotAvailable`] if the model lacks the LED, or an
    /// error if the command fails.
    pub fn set_effect(&mut self, led: Led, effect: Effect) -> Result<()> {
        self.require_led(led)?;
        self.execute(&Command::set_effect(led, effect)).map(drop)
    }

    fn require_led(&self, led: Led) -> Result<()> {
        if self.model.leds().contains(&led) {
            Ok(())
        } else {
            Err(Error::LedNotAvailable)
        }
    }
}
