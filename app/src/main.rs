// SPDX-License-Identifier: GPL-3.0-or-later
//! MouseTune for DeathAdder Essential.

#[expect(dead_code, reason = "wired up by the UI in the next commit")]
mod device_ops;
mod prefs;
#[expect(dead_code, reason = "wired up by the UI in the next commit")]
mod state;
#[expect(dead_code, reason = "wired up by the UI in the next commit")]
mod worker;

use std::sync::mpsc;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mousetune=info".into()),
        )
        .init();

    let (events_tx, events_rx) = mpsc::channel();
    let _worker = worker::spawn(prefs::Prefs::load_default(), move |event| {
        let _ = events_tx.send(event);
    })?;

    // Temporary headless driver; replaced by the Slint UI.
    for event in events_rx {
        println!("{event:?}");
    }
    Ok(())
}
