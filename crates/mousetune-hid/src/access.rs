// SPDX-License-Identifier: GPL-3.0-or-later
//! macOS Input Monitoring permission.
//!
//! macOS gates raw HID access behind the "Input Monitoring" privacy setting.
//! This module is the only place in the workspace that uses `unsafe`: two
//! calls into IOKit. On other platforms everything reports "granted".

/// State of the Input Monitoring permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAccess {
    /// Access is allowed (always the case outside macOS).
    Granted,
    /// The user denied access, or has not decided yet and must be asked.
    Denied,
}

/// Deep link to the Input Monitoring pane of System Settings.
pub const INPUT_MONITORING_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent";

/// Reports whether the app may read input devices, without prompting.
#[must_use]
pub fn input_monitoring_access() -> InputAccess {
    imp::check()
}

/// Shows the system permission prompt (macOS) and returns whether access was
/// granted immediately. Returns `true` outside macOS.
#[must_use]
pub fn request_input_monitoring() -> bool {
    imp::request()
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)] // FFI to IOKit; see SAFETY comments below.
mod imp {
    use super::InputAccess;

    /// `kIOHIDRequestTypeListenEvent`.
    const REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
    /// `kIOHIDAccessTypeGranted`.
    const ACCESS_GRANTED: u32 = 0;

    #[link(name = "IOKit", kind = "framework")]
    unsafe extern "C" {
        fn IOHIDCheckAccess(request_type: u32) -> u32;
        fn IOHIDRequestAccess(request_type: u32) -> bool;
    }

    pub(super) fn check() -> InputAccess {
        // SAFETY: `IOHIDCheckAccess` takes a plain enum value, has no
        // pointer arguments and no preconditions (macOS 10.15+).
        let status = unsafe { IOHIDCheckAccess(REQUEST_TYPE_LISTEN_EVENT) };
        from_raw(status)
    }

    pub(super) fn request() -> bool {
        // SAFETY: `IOHIDRequestAccess` takes a plain enum value, has no
        // pointer arguments and no preconditions (macOS 10.15+).
        unsafe { IOHIDRequestAccess(REQUEST_TYPE_LISTEN_EVENT) }
    }

    pub(super) fn from_raw(status: u32) -> InputAccess {
        if status == ACCESS_GRANTED {
            InputAccess::Granted
        } else {
            InputAccess::Denied
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn only_the_granted_code_means_granted() {
            assert_eq!(from_raw(0), InputAccess::Granted);
            assert_eq!(from_raw(1), InputAccess::Denied); // denied
            assert_eq!(from_raw(2), InputAccess::Denied); // unknown / not asked yet
        }

        #[test]
        fn querying_access_does_not_crash() {
            let _ = check();
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::InputAccess;

    pub(super) fn check() -> InputAccess {
        InputAccess::Granted
    }

    pub(super) fn request() -> bool {
        true
    }
}
