// SPDX-License-Identifier: GPL-3.0-or-later
//! Translates hidapi errors into [`TransportError`]s.
//!
//! hidapi reports most failures as free-form messages, so classification is
//! best effort and kept in one place with tests.

use std::io::ErrorKind;

use hidapi::HidError;
use mousetune_core::TransportError;

/// IOKit `kIOReturnNotPermitted`, printed by hidapi on open failures.
const IOKIT_NOT_PERMITTED: &str = "0xe00002e2";
/// IOKit `kIOReturnExclusiveAccess`.
const IOKIT_EXCLUSIVE_ACCESS: &str = "0xe00002c5";

pub(crate) fn map_hid_error(error: &HidError) -> TransportError {
    match error {
        HidError::IoError { error } => match error.kind() {
            ErrorKind::PermissionDenied => TransportError::PermissionDenied,
            ErrorKind::NotFound | ErrorKind::BrokenPipe => TransportError::Disconnected,
            _ => TransportError::Io(error.to_string()),
        },
        HidError::HidApiError { message } => map_message(message),
        other => TransportError::Io(other.to_string()),
    }
}

fn map_message(message: &str) -> TransportError {
    let lower = message.to_ascii_lowercase();
    if lower.contains(IOKIT_NOT_PERMITTED) || lower.contains("not permitted") {
        TransportError::PermissionDenied
    } else if lower.contains(IOKIT_EXCLUSIVE_ACCESS) || lower.contains("exclusive access") {
        TransportError::InUse
    } else if lower.contains("disconnected") {
        TransportError::Disconnected
    } else if lower.contains("permission denied") || lower.contains("access is denied") {
        TransportError::PermissionDenied
    } else {
        TransportError::Io(message.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api(message: &str) -> HidError {
        HidError::HidApiError {
            message: message.to_owned(),
        }
    }

    #[test]
    fn macos_not_permitted_maps_to_permission_denied() {
        let error = api(
            "hid_open_path: failed to open IOHIDDevice from mach entry: (0xE00002E2) not permitted",
        );
        assert_eq!(map_hid_error(&error), TransportError::PermissionDenied);
    }

    #[test]
    fn macos_exclusive_access_maps_to_in_use() {
        let error = api(
            "failed to open IOHIDDevice: (0xE00002C5) (iokit/common) exclusive access and device already open",
        );
        assert_eq!(map_hid_error(&error), TransportError::InUse);
    }

    #[test]
    fn unplugged_device_maps_to_disconnected() {
        assert_eq!(
            map_hid_error(&api("Device is disconnected")),
            TransportError::Disconnected
        );
    }

    #[test]
    fn io_error_kinds_are_mapped() {
        let denied = HidError::IoError {
            error: ErrorKind::PermissionDenied.into(),
        };
        assert_eq!(map_hid_error(&denied), TransportError::PermissionDenied);
        let gone = HidError::IoError {
            error: ErrorKind::NotFound.into(),
        };
        assert_eq!(map_hid_error(&gone), TransportError::Disconnected);
    }

    #[test]
    fn unknown_messages_are_kept_verbatim() {
        assert_eq!(
            map_hid_error(&api("something odd")),
            TransportError::Io("something odd".to_owned())
        );
    }
}
