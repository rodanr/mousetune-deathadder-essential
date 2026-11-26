// SPDX-License-Identifier: GPL-3.0-or-later
//! `RazerDevice` behaviour against a scripted fake mouse.

#![allow(clippy::unwrap_used)] // test helpers may panic freely

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mousetune_core::report::{ARGS_LEN, REPORT_LEN};
use mousetune_core::{
    Brightness, Dpi, DpiPair, Effect, Error, HidTransport, Led, Model, PollingRate, RazerDevice,
    Report, TransportError,
};

#[derive(Default)]
struct Script {
    sent: Vec<[u8; REPORT_LEN]>,
    replies: VecDeque<Result<[u8; REPORT_LEN], TransportError>>,
    gets: usize,
}

/// Fake transport; clones share the same script so tests can inspect it.
#[derive(Clone, Default)]
struct FakeMouse(Arc<Mutex<Script>>);

impl FakeMouse {
    fn queue(&self, reply: Result<[u8; REPORT_LEN], TransportError>) {
        self.0.lock().unwrap().replies.push_back(reply);
    }
    fn sent(&self) -> Vec<[u8; REPORT_LEN]> {
        self.0.lock().unwrap().sent.clone()
    }
    fn gets(&self) -> usize {
        self.0.lock().unwrap().gets
    }
}

impl HidTransport for FakeMouse {
    fn send_feature_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), TransportError> {
        self.0.lock().unwrap().sent.push(*report);
        Ok(())
    }
    fn get_feature_report(&mut self) -> Result<[u8; REPORT_LEN], TransportError> {
        let mut script = self.0.lock().unwrap();
        script.gets += 1;
        script
            .replies
            .pop_front()
            .unwrap_or(Err(TransportError::Io("script exhausted".into())))
    }
}

/// Builds a reply to command `(class, id)` with the given status and args.
fn reply(status: u8, class: u8, id: u8, args: &[u8]) -> [u8; REPORT_LEN] {
    let mut padded = [0; ARGS_LEN];
    padded[..args.len()].copy_from_slice(args);
    Report {
        status,
        transaction_id: 0xFF,
        data_size: 0,
        command_class: class,
        command_id: id,
        args: padded,
    }
    .encode()
}

fn device(model: Model) -> (RazerDevice<FakeMouse>, FakeMouse, Arc<Mutex<Vec<Duration>>>) {
    let mouse = FakeMouse::default();
    let sleeps = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&sleeps);
    let device = RazerDevice::with_sleeper(mouse.clone(), model, move |d| {
        recorded.lock().unwrap().push(d);
    });
    (device, mouse, sleeps)
}

#[test]
fn reads_firmware_version() {
    let (mut dev, mouse, sleeps) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x00, 0x81, &[1, 9])));
    assert_eq!(dev.firmware_version().unwrap(), (1, 9));
    assert_eq!(mouse.sent().len(), 1);
    assert_eq!(*sleeps.lock().unwrap(), vec![Duration::from_millis(2)]);
}

#[test]
fn reads_serial_until_nul() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x00, 0x82, b"PM2001H12345\0junk")));
    assert_eq!(dev.serial().unwrap(), "PM2001H12345");
}

#[test]
fn reads_dpi_pair() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x04, 0x85, &[0, 0x06, 0x40, 0x0C, 0x80])));
    let dpi = dev.dpi().unwrap();
    assert_eq!((dpi.x.get(), dpi.y.get()), (1600, 3200));
}

#[test]
fn out_of_range_dpi_from_device_is_an_error() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x04, 0x85, &[0, 0x00, 0x00, 0x06, 0x40])));
    assert_eq!(dev.dpi(), Err(Error::DpiOutOfRange(0)));
}

#[test]
fn reads_polling_rate_and_rejects_unknown_byte() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x00, 0x85, &[0x08])));
    assert_eq!(dev.polling_rate().unwrap(), PollingRate::Hz125);
    mouse.queue(Ok(reply(0x02, 0x00, 0x85, &[0x07])));
    assert_eq!(dev.polling_rate(), Err(Error::UnexpectedValue));
}

#[test]
fn reads_brightness_from_arg_2() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x0F, 0x84, &[1, 4, 0xB7])));
    assert_eq!(dev.brightness(Led::Logo).unwrap(), Brightness(0xB7));
}

#[test]
fn busy_reply_is_polled_again_without_resending() {
    let (mut dev, mouse, sleeps) = device(Model::Essential);
    mouse.queue(Ok(reply(0x01, 0x00, 0x05, &[])));
    mouse.queue(Ok(reply(0x01, 0x00, 0x05, &[])));
    mouse.queue(Ok(reply(0x02, 0x00, 0x05, &[0x01])));
    dev.set_polling_rate(PollingRate::Hz1000).unwrap();
    assert_eq!(mouse.sent().len(), 1, "command must not be resent");
    assert_eq!(mouse.gets(), 3);
    let ms = Duration::from_millis;
    assert_eq!(*sleeps.lock().unwrap(), vec![ms(2), ms(10), ms(10)]);
}

#[test]
fn gives_up_after_five_busy_retries() {
    let (mut dev, mouse, _) = device(Model::Essential);
    for _ in 0..10 {
        mouse.queue(Ok(reply(0x01, 0x00, 0x05, &[])));
    }
    assert_eq!(
        dev.set_polling_rate(PollingRate::Hz500),
        Err(Error::DeviceBusy)
    );
    assert_eq!(mouse.gets(), 6, "one read plus five retries");
}

#[test]
fn status_codes_map_to_errors() {
    for (status, expected) in [
        (0x03, Error::CommandFailed),
        (0x04, Error::Timeout),
        (0x05, Error::NotSupported),
        (0x09, Error::UnknownStatus(0x09)),
    ] {
        let (mut dev, mouse, _) = device(Model::Essential);
        mouse.queue(Ok(reply(status, 0x00, 0x81, &[])));
        assert_eq!(dev.firmware_version(), Err(expected));
    }
}

#[test]
fn reply_for_a_different_command_is_rejected() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x04, 0x85, &[])));
    assert_eq!(dev.firmware_version(), Err(Error::UnexpectedReply));
}

#[test]
fn transport_errors_propagate() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Err(TransportError::Disconnected));
    assert_eq!(
        dev.firmware_version(),
        Err(Error::Transport(TransportError::Disconnected))
    );
}

#[test]
fn set_dpi_sends_the_documented_bytes() {
    let (mut dev, mouse, _) = device(Model::Essential);
    mouse.queue(Ok(reply(0x02, 0x04, 0x05, &[])));
    dev.set_dpi(DpiPair::uniform(Dpi::new(1600).unwrap()))
        .unwrap();
    assert_eq!(
        mouse.sent()[0][..15],
        [0, 0xFF, 0, 0, 0, 7, 4, 5, 1, 6, 0x40, 6, 0x40, 0, 0]
    );
}

#[test]
fn the_2021_model_refuses_scroll_led_commands_without_touching_the_device() {
    let (mut dev, mouse, _) = device(Model::Essential2021);
    assert_eq!(
        dev.set_effect(Led::Scroll, Effect::Static),
        Err(Error::LedNotAvailable)
    );
    assert_eq!(
        dev.set_brightness(Led::Scroll, Brightness(1)),
        Err(Error::LedNotAvailable)
    );
    assert_eq!(dev.brightness(Led::Scroll), Err(Error::LedNotAvailable));
    assert_eq!(mouse.sent().len(), 0);

    mouse.queue(Ok(reply(0x02, 0x0F, 0x02, &[])));
    dev.set_effect(Led::Logo, Effect::Static).unwrap();
}
