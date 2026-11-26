# MouseTune mark

**Idea:** a tuning dial whose needle is the mouse.

The dial track is lit up to the needle, echoing the app's own slider fill.

## Files

| File | Use |
| ---- | --- |
| `mousetune-symbol-black.svg` | One-colour master (black). Printing, docs, light backgrounds. |
| `mousetune-symbol-white.svg` | Reversed master for dark backgrounds. |
| `../icon.svg` | Colour app icon on the macOS 824-px tile grid (48 px and up). |
| `../icon-small.svg` | Small-size cut for 16 to 32 px: full-bleed tile, bigger mark, no wheel slot. |

After changing the colour icons, run `cargo xtask icons` to regenerate `AppIcon.icns`, `icon.ico`, `icon-256.png` and `icon-1024.png`.

## Colours

| Role | Hex |
| ---- | --- |
| Tile, top to bottom | `#252D3A` → `#0E1218` |
| Accent (tuned range), same as the UI accent | `#5B9BFF` |
| Needle | `#F2F4F8` |
| Dial track | white at 16 % (30 % in the small cut) |

## Rules

- **Clear space:** keep space around the symbol of at least the needle's width.
- **Minimum size:** use the symbol alone down to 16 px, and use `icon-small.svg` for icons at 32 px and below.
- **Don't:**
  - recolour it in Razer green
  - add the Razer snake or other Razer marks
  - add gradients to the one-colour masters
  - rotate the needle to a different angle
  - set the mark in a box other than the app tile

MouseTune is independent of Razer Inc. Keep the mark free of Razer branding.
