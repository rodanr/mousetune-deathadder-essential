// SPDX-License-Identifier: GPL-3.0-or-later
//! Renders every screen of the UI offscreen to PNGs.
//!
//! ```sh
//! cargo run -p mousetune --example snapshots --features snapshots -- docs/screenshots
//! ```

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use anyhow::{Context, Result};
use mousetune::state::{AppState, DeviceState, LedState};
use mousetune::ui::{MainWindow, present};
use mousetune_core::{Brightness, Dpi, DpiPair, Effect, Led, Model, PollingRate};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, LogicalSize, PhysicalSize, PlatformError};

/// The window's fixed logical size (`width`/`height` in app.slint).
const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 592.0;
/// Render at 2x like a Retina display.
const SCALE: f32 = 2.0;

/// Offscreen platform with a manual clock, so animations can be finished
/// before a frame is captured.
struct Offscreen {
    window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
}

impl Platform for Offscreen {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.clock.get()
    }
}

fn dpi(value: u16) -> Result<Dpi> {
    Dpi::new(value).context("valid DPI")
}

fn connected(model: Model, leds: Vec<LedState>, dpi_pair: DpiPair) -> AppState {
    AppState::Connected(DeviceState {
        model,
        firmware: (1, 4),
        serial: "PM2140H03912345".to_owned(),
        dpi: dpi_pair,
        polling: PollingRate::Hz1000,
        leds,
    })
}

fn main() -> Result<()> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "docs/screenshots".into()),
    );
    std::fs::create_dir_all(&out)?;

    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    window.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
        scale_factor: SCALE,
    });
    let clock = Rc::new(Cell::new(Duration::ZERO));
    slint::platform::set_platform(Box::new(Offscreen {
        window: window.clone(),
        clock: Rc::clone(&clock),
    }))
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    let ui = MainWindow::new()?;
    ui.set_version(concat!("v", env!("CARGO_PKG_VERSION")).into());

    let uniform = DpiPair::uniform(dpi(1600)?);
    let essential = connected(
        Model::Essential,
        vec![
            LedState {
                led: Led::Logo,
                effect: Some(Effect::Static),
                brightness: Brightness(191),
            },
            LedState {
                led: Led::Scroll,
                effect: None,
                brightness: Brightness(128),
            },
        ],
        uniform,
    );
    let essential_2021 = connected(
        Model::Essential2021,
        vec![LedState {
            led: Led::Logo,
            effect: Some(Effect::Breathe),
            brightness: Brightness(255),
        }],
        DpiPair {
            x: dpi(1600)?,
            y: dpi(800)?,
        },
    );
    let cases = [
        ("connected", essential),
        ("connected-2021-separate-xy", essential_2021),
        ("searching", AppState::Searching),
        ("unsupported", AppState::Unsupported),
        ("permission", AppState::NeedsPermission),
        ("busy", AppState::Busy),
        (
            "error",
            AppState::Error("the mouse did not respond in time".into()),
        ),
    ];

    for (name, state) in &cases {
        ui.set_separate_xy(false);
        present(&ui, state);
        render(&ui, &window, &clock, &out.join(format!("{name}.png")))?;
    }

    // Feedback must not move the layout: same screen with "Applied" showing.
    present(&ui, &cases[0].1);
    ui.set_dpi_status("Applied".into());
    render(&ui, &window, &clock, &out.join("connected-applied.png"))?;
    ui.set_dpi_status("".into());
    println!("wrote {} snapshots to {}", cases.len() + 1, out.display());
    Ok(())
}

fn render(
    ui: &MainWindow,
    window: &MinimalSoftwareWindow,
    clock: &Cell<Duration>,
    path: &Path,
) -> Result<()> {
    ui.show()?;
    ui.window().set_size(LogicalSize::new(WIDTH, HEIGHT));
    // Let every transition finish before capturing.
    clock.set(clock.get() + Duration::from_secs(1));
    slint::platform::update_timers_and_animations();
    let PhysicalSize { width, height } = ui.window().size();
    let (w, h) = (usize::try_from(width)?, usize::try_from(height)?);

    let mut buffer = vec![PremultipliedRgbaColor::default(); w * h];
    window.request_redraw();
    window.draw_if_needed(|renderer| {
        renderer.render(&mut buffer, w);
    });

    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    let bytes: Vec<u8> = buffer
        .iter()
        .flat_map(|p| [p.red, p.green, p.blue, p.alpha])
        .collect();
    writer.write_image_data(&bytes)?;
    Ok(())
}
