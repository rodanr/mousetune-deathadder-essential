// SPDX-License-Identifier: GPL-3.0-or-later
//! Compiles the Slint UI.

fn main() -> Result<(), slint_build::CompileError> {
    slint_build::compile("ui/app.slint")
}
