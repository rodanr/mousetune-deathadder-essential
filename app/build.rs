// SPDX-License-Identifier: GPL-3.0-or-later
//! Compiles the Slint UI and, on Windows, embeds the application icon.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint_build::compile("ui/app.slint")?;

    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("../assets/icon.ico")
        .compile()?;

    Ok(())
}
