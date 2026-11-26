// SPDX-License-Identifier: GPL-3.0-or-later
//! What the UI shows. Pure data, no I/O.

use mousetune_core::{Brightness, DpiPair, Effect, Led, Model, PollingRate};

/// Everything the window can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// No mouse found yet.
    Searching,
    /// A Razer mouse that is not a DeathAdder Essential is plugged in.
    Unsupported,
    /// macOS Input Monitoring (or an equivalent OS permission) is missing.
    NeedsPermission,
    /// Another program holds the mouse.
    Busy,
    /// Something else went wrong; the message is shown to the user.
    Error(String),
    /// A supported mouse is ready.
    Connected(DeviceState),
}

/// Settings read from a connected mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceState {
    /// Detected model.
    pub model: Model,
    /// Firmware `(major, minor)`.
    pub firmware: (u8, u8),
    /// Serial number, used to remember lighting effects.
    pub serial: String,
    /// Current DPI.
    pub dpi: DpiPair,
    /// Current polling rate.
    pub polling: PollingRate,
    /// One entry per LED the model has.
    pub leds: Vec<LedState>,
}

/// State of one LED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedState {
    /// Which LED.
    pub led: Led,
    /// Last effect chosen in MouseTune. The mouse cannot report its effect,
    /// so this is `None` until the user picks one.
    pub effect: Option<Effect>,
    /// Brightness read from the mouse.
    pub brightness: Brightness,
}

/// A part of the UI that applies settings, for inline status and errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// DPI card.
    Dpi,
    /// Polling-rate card.
    Polling,
    /// One lighting card.
    Led(Led),
}
