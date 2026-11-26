# Contributing

Thanks for helping! This is a small project; the bar is "clean, tested, honest".

## Setup

```sh
git clone https://github.com/rodanr/mousetune-deathadder-essential
cd mousetune-deathadder-essential
cargo test --workspace
```

Linux needs `libudev-dev libfontconfig1-dev libxkbcommon-dev libxkbcommon-x11-dev`.
You do not need a mouse to run the tests.

## Before you open a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

CI runs the same checks on Linux, macOS and Windows.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): imperative summary`, at most 72 characters, with a body that
explains *why* when it is not obvious.

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`.
Scopes: `core`, `hid`, `app`, `macos`, `homebrew`. Keep each commit buildable
and its tests green.

## Rules for protocol changes

MouseTune only sends commands documented by [OpenRazer](https://github.com/openrazer/openrazer).

- Cite the OpenRazer source for any new command.
- Add a byte-exact vector to `crates/mousetune-core/tests/vectors.rs`.
- Validate input with a type (see `types.rs`), never at the call site.
- Do not add undocumented commands, or support for other Razer devices, without discussion.

## Code style

- `mousetune-core` stays free of I/O and `unsafe`; hardware sits behind `HidTransport`.
- No `unwrap` or `expect` outside tests. Errors are typed and user-readable.
- Public items have doc comments; files start with an SPDX header.
- The UI takes every colour and size from `app/ui/theme.slint`.

## Reporting bugs

Please include your mouse model, OS version, MouseTune version and a debug log
(`RUST_LOG=mousetune=debug`). The issue form asks for these.

## Disclaimer

Do not use Razer logos, the triple-snake mark, product photos or Razer green in
any contribution. MouseTune is independent of Razer Inc.
