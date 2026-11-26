// SPDX-License-Identifier: GPL-3.0-or-later
//! MouseTune for DeathAdder Essential.

// Release builds on Windows are GUI apps: no console window.
#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod device_ops;
mod prefs;
mod state;
mod ui;
mod worker;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mousetune=info,mousetune_hid=info".into()),
        )
        .init();

    ui::run(|sink| worker::spawn(prefs::Prefs::load_default(), sink))
}
