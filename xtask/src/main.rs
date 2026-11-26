// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask icons`: renders `assets/icon.svg` into the committed icon
//! files (PNG, macOS `.icns`, Windows `.ico`).

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use resvg::{tiny_skia, usvg};

/// Sizes bundled into the Windows icon.
const ICO_SIZES: [u32; 6] = [16, 24, 32, 48, 64, 256];

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("icons") => icons(),
        _ => bail!("usage: cargo xtask icons"),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn render(tree: &usvg::Tree, size: u32) -> Result<tiny_skia::Pixmap> {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).context("invalid icon size")?;
    #[allow(clippy::cast_precision_loss)] // sizes are at most 1024
    let scale = size as f32 / tree.size().width();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

fn icons() -> Result<()> {
    let root = workspace_root();
    let assets = root.join("assets");
    let svg = fs::read(assets.join("icon.svg")).context("reading assets/icon.svg")?;
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default())?;

    // PNGs: 1024 for stores/docs, 256 for Linux.
    for size in [1024, 256] {
        render(&tree, size)?.save_png(assets.join(format!("icon-{size}.png")))?;
    }

    // macOS .icns
    let mut family = icns::IconFamily::new();
    for size in [16, 32, 64, 128, 256, 512, 1024] {
        let pixmap = render(&tree, size)?;
        let image =
            icns::Image::from_data(icns::PixelFormat::RGBA, size, size, demultiply(&pixmap))?;
        family.add_icon(&image)?;
    }
    family.write(BufWriter::new(File::create(assets.join("AppIcon.icns"))?))?;

    // Windows .ico
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in ICO_SIZES {
        let pixmap = render(&tree, size)?;
        let image = ico::IconImage::from_rgba_data(size, size, demultiply(&pixmap));
        dir.add_entry(ico::IconDirEntry::encode(&image)?);
    }
    dir.write(BufWriter::new(File::create(assets.join("icon.ico"))?))?;

    println!("wrote icons to {}", assets.display());
    Ok(())
}

/// tiny-skia stores premultiplied alpha; icon formats want straight RGBA.
fn demultiply(pixmap: &tiny_skia::Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect()
}
