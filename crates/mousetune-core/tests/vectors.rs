// SPDX-License-Identifier: GPL-3.0-or-later
//! Byte-exact protocol vectors from the specification.

use hex_literal::hex;
use mousetune_core::{Brightness, Command, Dpi, DpiPair, Effect, Led, PollingRate, Report};

/// Header (8 bytes) plus `data_size` argument bytes of the encoded request.
fn wire_prefix(command: &Command) -> Vec<u8> {
    let bytes = command.to_report().encode();
    let data_size = usize::from(bytes[5]);
    bytes[..8 + data_size].to_vec()
}

#[test]
fn command_vectors() {
    let dpi_1600 = DpiPair::uniform(Dpi::new(1600).unwrap());
    let cases: Vec<(&str, Command, Vec<u8>)> = vec![
        (
            "static logo green",
            Command::set_effect(Led::Logo, Effect::Static),
            hex!("003f000000090f02 01040100000100ff00").to_vec(),
        ),
        (
            "off scroll",
            Command::set_effect(Led::Scroll, Effect::Off),
            hex!("003f000000060f02 010100000000").to_vec(),
        ),
        (
            "breathe logo",
            Command::set_effect(Led::Logo, Effect::Breathe),
            hex!("003f000000090f02 01040201000100ff00").to_vec(),
        ),
        (
            "brightness 0xB7",
            Command::set_brightness(Led::Logo, Brightness(0xB7)),
            hex!("003f000000030f04 0104b7").to_vec(),
        ),
        (
            "get brightness scroll",
            Command::get_brightness(Led::Scroll),
            hex!("003f000000030f84 0101 00").to_vec(),
        ),
        (
            "set DPI 1600",
            Command::set_dpi(dpi_1600),
            hex!("00ff000000070405 01064006400000").to_vec(),
        ),
        (
            "get DPI",
            Command::get_dpi(),
            hex!("00ff000000070485 00000000000000").to_vec(),
        ),
        (
            "polling 1000",
            Command::set_polling_rate(PollingRate::Hz1000),
            hex!("00ff000000010005 01").to_vec(),
        ),
        (
            "polling 500",
            Command::set_polling_rate(PollingRate::Hz500),
            hex!("00ff000000010005 02").to_vec(),
        ),
        (
            "polling 125",
            Command::set_polling_rate(PollingRate::Hz125),
            hex!("00ff000000010005 08").to_vec(),
        ),
        (
            "get polling",
            Command::get_polling_rate(),
            hex!("00ff000000010085 00").to_vec(),
        ),
        (
            "get firmware",
            Command::get_firmware_version(),
            hex!("00ff000000020081 0000").to_vec(),
        ),
        ("get serial", Command::get_serial(), {
            let mut expected = hex!("00ff000000160082").to_vec();
            expected.extend([0; 0x16]);
            expected
        }),
    ];

    for (name, command, expected) in cases {
        assert_eq!(wire_prefix(&command), expected, "{name}");
    }
}

#[test]
fn every_command_is_90_bytes_with_valid_crc() {
    let dpi = DpiPair {
        x: Dpi::new(100).unwrap(),
        y: Dpi::new(6400).unwrap(),
    };
    let mut commands = vec![
        Command::get_firmware_version(),
        Command::get_serial(),
        Command::set_dpi(dpi),
        Command::get_dpi(),
        Command::get_polling_rate(),
    ];
    for rate in PollingRate::ALL {
        commands.push(Command::set_polling_rate(rate));
    }
    for led in [Led::Logo, Led::Scroll] {
        commands.push(Command::get_brightness(led));
        commands.push(Command::set_brightness(led, Brightness(255)));
        for effect in [Effect::Off, Effect::Static, Effect::Breathe] {
            commands.push(Command::set_effect(led, effect));
        }
    }
    for command in commands {
        let bytes = command.to_report().encode();
        assert_eq!(bytes.len(), 90);
        assert!(Report::crc_is_valid(&bytes));
        assert_eq!(bytes[0], 0x00, "requests carry status 0");
    }
}

#[test]
fn transaction_ids_follow_the_protocol() {
    let led = Command::set_effect(Led::Logo, Effect::Static)
        .to_report()
        .encode();
    let info = Command::get_firmware_version().to_report().encode();
    assert_eq!(led[1], 0x3F);
    assert_eq!(info[1], 0xFF);
}

#[test]
fn static_and_breathe_always_send_pure_green() {
    for effect in [Effect::Static, Effect::Breathe] {
        let bytes = Command::set_effect(Led::Logo, effect).to_report().encode();
        assert_eq!(bytes[8 + 6..8 + 9], [0x00, 0xFF, 0x00]);
    }
}
