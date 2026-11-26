// SPDX-License-Identifier: GPL-3.0-or-later
//! Protocol and device logic for the Razer DeathAdder Essential family.
//!
//! This crate performs no I/O: hardware access sits behind a transport trait
//! so everything here is unit-testable. Protocol knowledge comes from the
//! OpenRazer project.

#![forbid(unsafe_code)]

pub mod command;
pub mod device;
pub mod error;
pub mod model;
pub mod report;
pub mod transport;
pub mod types;

pub use command::Command;
pub use device::RazerDevice;
pub use error::{Error, Result};
pub use model::{Model, VENDOR_ID};
pub use report::Report;
pub use transport::{HidTransport, TransportError};
pub use types::{Brightness, Dpi, DpiPair, Effect, Led, PollingRate};
