// SPDX-License-Identifier: GPL-3.0-or-later
//! [`HidTransport`] on top of a hidapi device.

use hidapi::HidDevice;
use mousetune_core::report::REPORT_LEN;
use mousetune_core::{HidTransport, TransportError};

use crate::error_map::map_hid_error;

/// hidapi framing: one leading report-id byte (always 0 for this mouse).
const FRAMED_LEN: usize = REPORT_LEN + 1;

/// Sends and receives 90-byte feature reports through hidapi, hiding the
/// report-id byte that hidapi requires.
pub struct HidapiTransport {
    device: HidDevice,
}

impl HidapiTransport {
    pub(crate) fn new(device: HidDevice) -> Self {
        Self { device }
    }
}

impl HidTransport for HidapiTransport {
    fn send_feature_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), TransportError> {
        let mut framed = [0_u8; FRAMED_LEN];
        framed[1..].copy_from_slice(report);
        self.device
            .send_feature_report(&framed)
            .map_err(|e| map_hid_error(&e))
    }

    fn get_feature_report(&mut self) -> Result<[u8; REPORT_LEN], TransportError> {
        let mut framed = [0_u8; FRAMED_LEN];
        // framed[0] is the report id (0); hidapi leaves the data in framed[1..].
        let read = self
            .device
            .get_feature_report(&mut framed)
            .map_err(|e| map_hid_error(&e))?;
        if read < REPORT_LEN {
            return Err(TransportError::Io(format!(
                "short feature report ({read} bytes)"
            )));
        }
        let mut report = [0_u8; REPORT_LEN];
        report.copy_from_slice(&framed[1..]);
        Ok(report)
    }
}
