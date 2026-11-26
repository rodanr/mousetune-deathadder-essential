// SPDX-License-Identifier: GPL-3.0-or-later
//! Finds a supported mouse and opens the right HID interface.
//!
//! The mouse exposes several HID interfaces and hidapi does not report
//! their maximum feature report size, so each candidate is opened and probed
//! with a read-only "get firmware" command; the first that answers wins.

use std::ffi::CString;

use hidapi::{HidApi, HidError};
use mousetune_core::{Model, RazerDevice, TransportError, VENDOR_ID};

use crate::error_map::map_hid_error;
use crate::transport::HidapiTransport;

/// HID usage page / usage of a generic desktop mouse.
const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
const USAGE_MOUSE: u16 = 0x02;

/// Why a supported mouse could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectError {
    /// The OS denied access (macOS Input Monitoring, Linux udev rule, ...).
    PermissionDenied,
    /// Another program (for example Razer Synapse) holds the device.
    InUse,
    /// The mouse was found but no interface answered the firmware probe.
    NoResponse(String),
}

/// A mouse that is open and answered the firmware probe.
pub struct Connected {
    /// The ready-to-use device.
    pub device: RazerDevice<HidapiTransport>,
    /// Firmware `(major, minor)` read during the probe.
    pub firmware: (u8, u8),
}

/// Result of one discovery pass.
pub enum Connection {
    /// No Razer mouse is plugged in.
    Absent,
    /// A Razer mouse is plugged in, but not a DeathAdder Essential.
    Unsupported {
        /// USB product id of the unsupported mouse.
        product_id: u16,
    },
    /// A supported mouse is present but could not be opened.
    Failed {
        /// The detected model.
        model: Model,
        /// What went wrong.
        error: ConnectError,
    },
    /// A supported mouse is open and ready.
    Connected(Box<Connected>),
}

/// How a HID interface is classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Supported(Model),
    UnsupportedMouse,
    Ignored,
}

fn classify(vendor_id: u16, product_id: u16, usage_page: u16, usage: u16) -> Kind {
    if vendor_id != VENDOR_ID {
        return Kind::Ignored;
    }
    if let Some(model) = Model::from_product_id(product_id) {
        return Kind::Supported(model);
    }
    if usage_page == USAGE_PAGE_GENERIC_DESKTOP && usage == USAGE_MOUSE {
        Kind::UnsupportedMouse
    } else {
        Kind::Ignored
    }
}

struct Candidate {
    model: Model,
    path: CString,
}

/// Owns the hidapi context and performs discovery passes.
pub struct Discovery {
    api: HidApi,
}

impl Discovery {
    /// Initialises hidapi (non-exclusive on macOS).
    ///
    /// # Errors
    ///
    /// Returns the hidapi initialisation error.
    pub fn new() -> Result<Self, HidError> {
        Ok(Self {
            api: HidApi::new()?,
        })
    }

    /// Re-enumerates devices and tries to open a supported mouse.
    ///
    /// Cheap enough to call about once per second for hot-plug detection.
    pub fn connect(&mut self) -> Connection {
        if let Err(error) = self.api.refresh_devices() {
            tracing::warn!(%error, "failed to refresh HID device list");
        }

        let mut candidates = Vec::new();
        let mut unsupported = None;
        for info in self.api.device_list() {
            match classify(
                info.vendor_id(),
                info.product_id(),
                info.usage_page(),
                info.usage(),
            ) {
                Kind::Supported(model) => {
                    candidates.push(Candidate {
                        model,
                        path: info.path().to_owned(),
                    });
                }
                Kind::UnsupportedMouse => unsupported = Some(info.product_id()),
                Kind::Ignored => {}
            }
        }

        let Some(first) = candidates.first() else {
            return unsupported.map_or(Connection::Absent, |product_id| Connection::Unsupported {
                product_id,
            });
        };
        let model = first.model;

        let mut last_error = ConnectError::NoResponse("no HID interface responded".to_owned());
        for candidate in candidates.into_iter().filter(|c| c.model == model) {
            match self.try_open(&candidate) {
                Ok(connected) => return Connection::Connected(Box::new(connected)),
                Err(error) => {
                    tracing::debug!(?error, "candidate interface rejected");
                    last_error = error;
                }
            }
        }
        Connection::Failed {
            model,
            error: last_error,
        }
    }

    fn try_open(&self, candidate: &Candidate) -> Result<Connected, ConnectError> {
        let hid = self
            .api
            .open_path(&candidate.path)
            .map_err(|error| to_connect_error(&map_hid_error(&error)))?;
        let mut device = RazerDevice::new(HidapiTransport::new(hid), candidate.model);
        let firmware = device
            .firmware_version()
            .map_err(|error| ConnectError::NoResponse(error.to_string()))?;
        Ok(Connected { device, firmware })
    }
}

fn to_connect_error(error: &TransportError) -> ConnectError {
    match error {
        TransportError::PermissionDenied => ConnectError::PermissionDenied,
        TransportError::InUse => ConnectError::InUse,
        other => ConnectError::NoResponse(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_pids_are_recognised() {
        assert_eq!(
            classify(0x1532, 0x0098, 0, 0),
            Kind::Supported(Model::Essential2021)
        );
    }

    #[test]
    fn other_razer_mice_are_flagged_unsupported() {
        assert_eq!(classify(0x1532, 0x0084, 0x01, 0x02), Kind::UnsupportedMouse);
    }

    #[test]
    fn razer_non_mouse_interfaces_and_other_vendors_are_ignored() {
        assert_eq!(classify(0x1532, 0x0084, 0xFF00, 0x01), Kind::Ignored);
        assert_eq!(classify(0x046D, 0x006E, 0x01, 0x02), Kind::Ignored);
    }

    #[test]
    fn transport_errors_become_connect_errors() {
        assert_eq!(
            to_connect_error(&TransportError::PermissionDenied),
            ConnectError::PermissionDenied
        );
        assert_eq!(
            to_connect_error(&TransportError::InUse),
            ConnectError::InUse
        );
    }
}
