// SPDX-License-Identifier: GPL-3.0-or-later
//! Device reads and writes that do not depend on the real hardware, so they
//! can be tested with a fake transport.

use mousetune_core::{
    Brightness, DpiPair, Effect, Error, HidTransport, Led, PollingRate, RazerDevice,
};

use crate::prefs::Prefs;
use crate::state::{DeviceState, LedState, Section};

/// A change requested by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Set DPI.
    Dpi(DpiPair),
    /// Set polling rate.
    Polling(PollingRate),
    /// Set an LED effect.
    Effect(Led, Effect),
    /// Set an LED brightness.
    Brightness(Led, Brightness),
}

impl Change {
    /// The UI section this change belongs to.
    #[must_use]
    pub fn section(self) -> Section {
        match self {
            Self::Dpi(_) => Section::Dpi,
            Self::Polling(_) => Section::Polling,
            Self::Effect(led, _) | Self::Brightness(led, _) => Section::Led(led),
        }
    }
}

/// Reads every setting from a freshly connected mouse.
///
/// # Errors
///
/// Returns the first device error encountered.
pub fn read_state<T: HidTransport>(
    device: &mut RazerDevice<T>,
    firmware: (u8, u8),
    prefs: &Prefs,
) -> Result<DeviceState, Error> {
    let model = device.model();
    let serial = device.serial()?;
    let dpi = device.dpi()?;
    let polling = device.polling_rate()?;
    let leds = model
        .leds()
        .iter()
        .map(|&led| {
            Ok(LedState {
                led,
                effect: prefs.effect(&serial, led),
                brightness: device.brightness(led)?,
            })
        })
        .collect::<Result<_, Error>>()?;
    Ok(DeviceState {
        model,
        firmware,
        serial,
        dpi,
        polling,
        leds,
    })
}

/// Sends a change to the mouse and mirrors it into `state` on success.
///
/// # Errors
///
/// Returns the device error; `state` is left untouched in that case.
pub fn apply<T: HidTransport>(
    device: &mut RazerDevice<T>,
    state: &mut DeviceState,
    change: Change,
) -> Result<(), Error> {
    match change {
        Change::Dpi(dpi) => {
            device.set_dpi(dpi)?;
            state.dpi = dpi;
        }
        Change::Polling(rate) => {
            device.set_polling_rate(rate)?;
            state.polling = rate;
        }
        Change::Effect(led, effect) => {
            device.set_effect(led, effect)?;
            led_state(state, led)?.effect = Some(effect);
        }
        Change::Brightness(led, brightness) => {
            device.set_brightness(led, brightness)?;
            led_state(state, led)?.brightness = brightness;
        }
    }
    Ok(())
}

fn led_state(state: &mut DeviceState, led: Led) -> Result<&mut LedState, Error> {
    state
        .leds
        .iter_mut()
        .find(|entry| entry.led == led)
        .ok_or(Error::LedNotAvailable)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use mousetune_core::report::{ARGS_LEN, REPORT_LEN};
    use mousetune_core::{Dpi, Model, Report, TransportError};

    use super::*;

    /// Answers every request with OK, echoing class/id, with canned args per command.
    #[derive(Default)]
    struct FakeMouse {
        last: Option<Report>,
        failing: bool,
        sent: VecDeque<Report>,
    }

    impl HidTransport for FakeMouse {
        fn send_feature_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), TransportError> {
            if self.failing {
                return Err(TransportError::Disconnected);
            }
            let request = Report::decode(report).unwrap();
            self.sent.push_back(request.clone());
            self.last = Some(request);
            Ok(())
        }

        fn get_feature_report(&mut self) -> Result<[u8; REPORT_LEN], TransportError> {
            let request = self.last.clone().unwrap();
            let mut args = [0; ARGS_LEN];
            match (request.command_class, request.command_id) {
                (0x00, 0x82) => args[..4].copy_from_slice(b"SN01"),
                (0x04, 0x85) => args[1..5].copy_from_slice(&[0x03, 0x20, 0x06, 0x40]), // 800 / 1600
                (0x00, 0x85) => args[0] = 0x02,
                (0x0F, 0x84) => args[2] = 0x80,
                _ => {}
            }
            Ok(Report {
                status: 0x02,
                args,
                ..request
            }
            .encode())
        }
    }

    fn device(model: Model) -> RazerDevice<FakeMouse> {
        RazerDevice::with_sleeper(FakeMouse::default(), model, |_| {})
    }

    #[test]
    fn reads_all_settings_and_only_the_leds_the_model_has() {
        let mut prefs = Prefs::load(None);
        prefs.set_effect("SN01", Led::Logo, Effect::Breathe);

        let state = read_state(&mut device(Model::Essential), (1, 2), &prefs).unwrap();
        assert_eq!(state.serial, "SN01");
        assert_eq!((state.dpi.x.get(), state.dpi.y.get()), (800, 1600));
        assert_eq!(state.polling, PollingRate::Hz500);
        assert_eq!(state.leds.len(), 2);
        assert_eq!(state.leds[0].effect, Some(Effect::Breathe));
        assert_eq!(state.leds[1].effect, None);
        assert_eq!(state.leds[0].brightness, Brightness(0x80));

        let state = read_state(&mut device(Model::Essential2021), (1, 2), &prefs).unwrap();
        assert_eq!(state.leds.len(), 1);
    }

    #[test]
    fn apply_updates_state_only_on_success() {
        let prefs = Prefs::load(None);
        let mut dev = device(Model::Essential);
        let mut state = read_state(&mut dev, (1, 0), &prefs).unwrap();

        let dpi = DpiPair::uniform(Dpi::new(3200).unwrap());
        apply(&mut dev, &mut state, Change::Dpi(dpi)).unwrap();
        assert_eq!(state.dpi, dpi);

        apply(
            &mut dev,
            &mut state,
            Change::Effect(Led::Scroll, Effect::Static),
        )
        .unwrap();
        assert_eq!(state.leds[1].effect, Some(Effect::Static));

        let before = state.clone();
        let mut failing = RazerDevice::with_sleeper(
            FakeMouse {
                failing: true,
                ..FakeMouse::default()
            },
            Model::Essential,
            |_| {},
        );
        assert!(
            apply(
                &mut failing,
                &mut state,
                Change::Polling(PollingRate::Hz125)
            )
            .is_err()
        );
        assert_eq!(state, before);
    }

    #[test]
    fn changes_map_to_their_sections() {
        let dpi = DpiPair::uniform(Dpi::new(800).unwrap());
        assert_eq!(Change::Dpi(dpi).section(), Section::Dpi);
        assert_eq!(
            Change::Polling(PollingRate::Hz500).section(),
            Section::Polling
        );
        assert_eq!(
            Change::Brightness(Led::Scroll, Brightness(1)).section(),
            Section::Led(Led::Scroll)
        );
    }
}
