// SPDX-License-Identifier: GPL-3.0-or-later
//! The fixed-size 90-byte Razer feature report.
//!
//! Layout (see OpenRazer `razerchromacommon.h`):
//!
//! | byte    | field                          |
//! |---------|--------------------------------|
//! | 0       | status                         |
//! | 1       | transaction id                 |
//! | 2..=3   | remaining packets (always 0)   |
//! | 4       | protocol type (always 0)       |
//! | 5       | data size                      |
//! | 6       | command class                  |
//! | 7       | command id                     |
//! | 8..=87  | arguments (80 bytes)           |
//! | 88      | CRC: XOR of bytes 2..=87       |
//! | 89      | reserved (0)                   |

use crate::error::{Error, Result};

/// Total size of a report in bytes.
pub const REPORT_LEN: usize = 90;
/// Number of argument bytes a report can carry.
pub const ARGS_LEN: usize = 80;

const OFFSET_STATUS: usize = 0;
const OFFSET_TRANSACTION: usize = 1;
const OFFSET_DATA_SIZE: usize = 5;
const OFFSET_CLASS: usize = 6;
const OFFSET_ID: usize = 7;
const OFFSET_ARGS: usize = 8;
const OFFSET_CRC: usize = 88;
const CRC_RANGE: std::ops::RangeInclusive<usize> = 2..=87;

/// A decoded 90-byte Razer report.
///
/// The same layout is used for requests and replies; only `status` differs
/// (`0x00` in requests, a result code in replies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Status byte: `0x00` in requests, a result code in replies.
    pub status: u8,
    /// Transaction id selecting the device sub-controller.
    pub transaction_id: u8,
    /// Number of meaningful argument bytes.
    pub data_size: u8,
    /// Command class.
    pub command_class: u8,
    /// Command id within the class.
    pub command_id: u8,
    /// Argument bytes, zero padded to 80.
    pub args: [u8; ARGS_LEN],
}

impl Report {
    /// Builds a request report with status `0x00` from raw argument bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ArgumentsTooLong`] if `args` exceeds 80 bytes.
    pub fn request(
        transaction_id: u8,
        command_class: u8,
        command_id: u8,
        data_size: u8,
        args: &[u8],
    ) -> Result<Self> {
        if args.len() > ARGS_LEN {
            return Err(Error::ArgumentsTooLong {
                max: ARGS_LEN,
                actual: args.len(),
            });
        }
        let mut padded = [0; ARGS_LEN];
        padded[..args.len()].copy_from_slice(args);
        Ok(Self {
            status: 0x00,
            transaction_id,
            data_size,
            command_class,
            command_id,
            args: padded,
        })
    }

    /// Serialises the report, computing the CRC.
    #[must_use]
    pub fn encode(&self) -> [u8; REPORT_LEN] {
        let mut bytes = [0; REPORT_LEN];
        bytes[OFFSET_STATUS] = self.status;
        bytes[OFFSET_TRANSACTION] = self.transaction_id;
        bytes[OFFSET_DATA_SIZE] = self.data_size;
        bytes[OFFSET_CLASS] = self.command_class;
        bytes[OFFSET_ID] = self.command_id;
        bytes[OFFSET_ARGS..OFFSET_ARGS + ARGS_LEN].copy_from_slice(&self.args);
        bytes[OFFSET_CRC] = crc(&bytes);
        bytes
    }

    /// Parses a report received from the device.
    ///
    /// The CRC is not enforced here; use [`Report::crc_is_valid`] on the raw
    /// bytes if you need it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidReportLength`] unless `bytes` is exactly 90 bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let bytes: &[u8; REPORT_LEN] =
            bytes.try_into().map_err(|_| Error::InvalidReportLength {
                expected: REPORT_LEN,
                actual: bytes.len(),
            })?;
        let mut args = [0; ARGS_LEN];
        args.copy_from_slice(&bytes[OFFSET_ARGS..OFFSET_ARGS + ARGS_LEN]);
        Ok(Self {
            status: bytes[OFFSET_STATUS],
            transaction_id: bytes[OFFSET_TRANSACTION],
            data_size: bytes[OFFSET_DATA_SIZE],
            command_class: bytes[OFFSET_CLASS],
            command_id: bytes[OFFSET_ID],
            args,
        })
    }

    /// Returns `true` if the CRC byte of a raw report matches its contents.
    #[must_use]
    pub fn crc_is_valid(bytes: &[u8; REPORT_LEN]) -> bool {
        bytes[OFFSET_CRC] == crc(bytes)
    }
}

/// XOR of bytes `2..=87`.
fn crc(bytes: &[u8; REPORT_LEN]) -> u8 {
    bytes[CRC_RANGE].iter().fold(0, |acc, byte| acc ^ byte)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn encoded_report_is_90_bytes_with_expected_header() {
        let report = Report::request(0x3F, 0x0F, 0x04, 3, &[0x01, 0x04, 0xB7]).unwrap();
        let bytes = report.encode();
        assert_eq!(bytes.len(), 90);
        assert_eq!(
            bytes[..11],
            [0x00, 0x3F, 0, 0, 0, 3, 0x0F, 0x04, 0x01, 0x04, 0xB7]
        );
        assert_eq!(bytes[89], 0);
    }

    #[test]
    fn crc_is_xor_of_bytes_2_to_87() {
        // size 0x09, class 0x0F, id 0x02, args 01 04 01 00 00 01 00 FF 00
        let report = Report::request(0x3F, 0x0F, 0x02, 9, &[1, 4, 1, 0, 0, 1, 0, 0xFF, 0]).unwrap();
        let bytes = report.encode();
        assert_eq!(bytes[88], 0xFE);
        assert!(Report::crc_is_valid(&bytes));
    }

    #[test]
    fn crc_ignores_status_and_transaction_bytes() {
        let a = Report::request(0x3F, 0, 5, 1, &[1]).unwrap().encode();
        let b = Report::request(0xFF, 0, 5, 1, &[1]).unwrap().encode();
        assert_eq!(a[88], b[88]);
    }

    #[test]
    fn corrupted_crc_is_detected() {
        let mut bytes = Report::request(0xFF, 0, 0x81, 2, &[]).unwrap().encode();
        bytes[88] ^= 0xFF;
        assert!(!Report::crc_is_valid(&bytes));
    }

    #[test]
    fn decode_rejects_wrong_length() {
        for len in [0, 1, 89, 91, 91 + 90] {
            assert_eq!(
                Report::decode(&vec![0; len]),
                Err(Error::InvalidReportLength {
                    expected: 90,
                    actual: len
                })
            );
        }
    }

    #[test]
    fn request_rejects_oversized_arguments() {
        assert_eq!(
            Report::request(0xFF, 0, 0, 0, &[0; 81]),
            Err(Error::ArgumentsTooLong {
                max: 80,
                actual: 81
            })
        );
    }

    proptest! {
        #[test]
        fn encode_decode_round_trips(
            status: u8, transaction_id: u8, data_size: u8, class: u8, id: u8,
            args in proptest::collection::vec(any::<u8>(), 80),
        ) {
            let mut padded = [0u8; ARGS_LEN];
            padded.copy_from_slice(&args);
            let report = Report {
                status, transaction_id, data_size, command_class: class, command_id: id,
                args: padded,
            };
            let bytes = report.encode();
            prop_assert!(Report::crc_is_valid(&bytes));
            prop_assert_eq!(Report::decode(&bytes).unwrap(), report);
        }
    }
}
