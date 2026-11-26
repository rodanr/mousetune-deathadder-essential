# Changelog

All notable changes are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.1] - 2026-11-26

### Fixed

- The macOS app icon had an undecodable 48 px entry that could show as noise
  in Finder or the Dock at that size.

### Changed

- New releases update the Homebrew cask automatically, so
  `brew upgrade --cask mousetune` sees them right away.

## [0.1.0] - 2026-11-26

### Added

- Control DPI (100 to 6400, separate X/Y), polling rate (125/500/1000 Hz) and
  logo/scroll LED effect and brightness on the DeathAdder Essential,
  Essential White Edition and Essential (2021).
- Slint user interface with a dark HUD look: a segmented DPI gauge on a
  logarithmic scale with an editable value, chamfered panels and the bundled
  Oxanium typeface. The window has a fixed size.
- Original "dial" app icon (the mouse as the needle of a tuning dial), with
  one-colour masters and a dedicated small-size cut.
- macOS universal app bundle, DMG and zip; Windows and Linux archives;
  Homebrew cask in the rodanr/tap tap.

[Unreleased]: https://github.com/rodanr/mousetune-deathadder-essential/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/rodanr/mousetune-deathadder-essential/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/rodanr/mousetune-deathadder-essential/releases/tag/v0.1.0
