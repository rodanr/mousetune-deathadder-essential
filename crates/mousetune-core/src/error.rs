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

    /// The transport failed.
    #[error(transparent)]
    Transport(#[from] crate::transport::TransportError),

    /// The mouse kept answering "busy" after all retries.
    #[error("the mouse is busy, try again")]
    DeviceBusy,

    /// The mouse reported that the command failed.
    #[error("the mouse rejected the command")]
    CommandFailed,

    /// The mouse did not answer in time.
    #[error("the mouse did not respond in time")]
    Timeout,

    /// The mouse does not support the command.
    #[error("this mouse does not support that command")]
    NotSupported,

    /// The reply carried a status byte outside the documented set.
    #[error("unexpected reply status {0:#04x}")]
    UnknownStatus(u8),

    /// The reply answered a different command than the one sent.
    #[error("the mouse answered a different command than requested")]
    UnexpectedReply,

    /// The reply contained a value outside what the protocol documents.
    #[error("the mouse returned an unexpected value")]
    UnexpectedValue,

    /// The requested LED does not exist on this model.
    #[error("this model has no such LED")]
    LedNotAvailable,
}

/// Convenience alias for results using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
