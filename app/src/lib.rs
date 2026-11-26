// SPDX-License-Identifier: GPL-3.0-or-later
//! MouseTune application: state machine, device worker and Slint UI.
//!
//! The binary in `main.rs` only wires these together; the library form lets
//! the `snapshots` example render the UI offscreen.

pub mod device_ops;
pub mod prefs;
pub mod state;
pub mod ui;
pub mod worker;
