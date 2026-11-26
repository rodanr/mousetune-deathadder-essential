// SPDX-License-Identifier: GPL-3.0-or-later
//! The hardware boundary. The real implementation lives in `mousetune-hid`;
//! tests use an in-memory fake.

use thiserror::Error;

use crate::report::REPORT_LEN;

/// Failure reported by a [`HidTransport`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TransportError {
    /// The operating system denied access to the device (for example macOS
    /// Input Monitoring has not been granted).
    #[error("access to the mouse was denied by the operating system")]
    PermissionDenied,
    /// Another program holds the device exclusively.
    #[error("the mouse is in use by another program")]
    InUse,
    /// The device was unplugged.
    #[error("the mouse was disconnected")]
    Disconnected,
    /// Any other I/O failure.
    #[error("communication with the mouse failed: {0}")]
    Io(String),
}

/// A channel that can exchange 90-byte feature reports (report id 0) with a
/// mouse. Implementations deal with any platform framing of the report id.
pub trait HidTransport {
    /// Sends one feature report.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportError`] if the report could not be written.
    fn send_feature_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), TransportError>;

    /// Reads one feature report.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportError`] if the report could not be read.
    fn get_feature_report(&mut self) -> Result<[u8; REPORT_LEN], TransportError>;
}
