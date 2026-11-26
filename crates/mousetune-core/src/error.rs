// SPDX-License-Identifier: GPL-3.0-or-later
//! Error type shared by the whole crate.

use thiserror::Error;

/// Everything that can go wrong while talking to the mouse.
///
/// The [`Display`](std::fmt::Display) text is written for end users and is
/// shown in the UI as is.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    /// A report buffer did not have the fixed protocol size.
    #[error("unexpected report length: expected {expected} bytes, got {actual}")]
    InvalidReportLength {
        /// Required length in bytes.
        expected: usize,
        /// Length that was supplied.
        actual: usize,
    },

    /// A command carried more argument bytes than a report can hold.
    #[error("command arguments too long: {actual} bytes (maximum {max})")]
    ArgumentsTooLong {
        /// Maximum number of argument bytes.
        max: usize,
        /// Number of bytes that was supplied.
        actual: usize,
    },

    /// A DPI value was outside the supported range.
    #[error("DPI {0} is out of range (100–6400)")]
    DpiOutOfRange(u16),
}

/// Convenience alias for results using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
