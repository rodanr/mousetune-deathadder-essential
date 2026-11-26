// SPDX-License-Identifier: GPL-3.0-or-later
//! Protocol and device logic for the Razer DeathAdder Essential family.
//!
//! This crate performs no I/O: hardware access sits behind a transport trait
//! so everything here is unit-testable. Protocol knowledge comes from the
//! OpenRazer project.

pub mod error;
pub mod report;

pub use error::{Error, Result};
pub use report::Report;
