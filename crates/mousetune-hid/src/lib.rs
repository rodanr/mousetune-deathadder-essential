// SPDX-License-Identifier: GPL-3.0-or-later
//! hidapi-backed transport and device discovery for MouseTune.
//!
//! All operating-system specifics (IOKit, hid.dll, hidraw) are handled by
//! hidapi; this crate only adapts it to [`mousetune_core::HidTransport`] and
//! decides which of the mouse's HID interfaces to talk to.

mod access;
mod discovery;
mod error_map;
mod transport;

pub use access::{
    INPUT_MONITORING_SETTINGS_URL, InputAccess, input_monitoring_access, request_input_monitoring,
};
pub use discovery::{ConnectError, Connected, Connection, Discovery};
pub use transport::HidapiTransport;
