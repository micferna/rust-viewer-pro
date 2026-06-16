<div align="center">

# Rust Viewer Pro

**Fast, cross-platform image viewer written in Rust.**

[![CI](https://github.com/micferna/rust-viewer-pro/actions/workflows/ci.yml/badge.svg)](https://github.com/micferna/rust-viewer-pro/actions/workflows/ci.yml)
[![Release](https://github.com/micferna/rust-viewer-pro/actions/workflows/release.yml/badge.svg)](https://github.com/micferna/rust-viewer-pro/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/micferna/rust-viewer-pro?sort=semver)](https://github.com/micferna/rust-viewer-pro/releases/latest)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](#license)

</div>

---

## Features

- ⚡ **Fast** — images are decoded on a background thread; the UI never blocks.
- 🔭 **Smooth zoom & pan** — wheel zoom centred on the cursor, drag to pan, fit-to-window.
- 🗂️ **Folder navigation** — open a file or a whole directory and step through it.
- 🖼️ **Formats** — PNG, JPEG, BMP, WebP, GIF.
- 🔄 **Built-in updates** — checks GitHub Releases and updates itself in place.
- 🧩 **Default viewer** — opt in to file associations at install time (Linux & Windows).
- 💻 **Cross-platform** — Linux (`.deb`, AppImage) and Windows (installer + portable).

## Install

### Linux

Download the latest `.deb` or `.AppImage` from the
[releases page](https://github.com/micferna/rust-viewer-pro/releases/latest).

```sh
# Debian / Ubuntu — the installer asks whether to set it as the default
# image viewer (you can also change this later in your desktop settings).
sudo apt install ./rust-viewer-pro_*_amd64.deb

# Portable AppImage
chmod +x rust-viewer-pro-x86_64.AppImage
./rust-viewer-pro-x86_64.AppImage
```

### Windows

Two options:

- **Installer (recommended)** — run `rust-viewer-pro-setup-x86_64.exe`. It
  installs the app, adds a Start-menu shortcut and offers a checkbox to
  associate image files, so Rust Viewer Pro shows up under
  **Settings → Default apps**.
- **Portable** — download `rust-viewer-pro-x86_64-pc-windows-msvc.zip`, unzip
  and run `rust-viewer-pro.exe`.

The first launch shows a SmartScreen warning because the binary is not yet
code-signed — click **More info → Run anyway**.

### From source

```sh
cargo install --git https://github.com/micferna/rust-viewer-pro
```

## Usage

```sh
rust-viewer-pro path/to/image.png   # open a file (and its folder)
rust-viewer-pro path/to/folder      # open a folder
rust-viewer-pro --no-update-check   # skip the startup update check
```

### Keyboard shortcuts

| Keys           | Action                              |
| -------------- | ----------------------------------- |
| `→` / `Space`  | Next image                          |
| `←` / `Backspace` | Previous image                   |
| `Home` / `End` | First / last image                  |
| Mouse wheel    | Zoom (centred on cursor)            |
| `+` / `-`      | Zoom in / out                       |
| Drag           | Pan                                 |
| `0` / `R`      | Reset view (fit to window)          |
| `F` / `F11`    | Toggle fullscreen                   |
| `H` / `F1`     | Toggle help                         |

## Updates

On startup the app queries GitHub Releases. If a newer version exists, an
**Update** button appears in the toolbar; clicking it downloads the artifact for
your platform, replaces the running binary and offers to restart. Disable the
check with `--no-update-check`.

> System-wide `.deb` installs live in `/usr/bin` (root-owned); update those
> through your package manager. AppImage and portable installs self-update.

## Development

```sh
cargo run -- assets                 # run
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo test
cargo bench                         # decode benchmarks (criterion)
cargo deny check                    # licenses + advisories
cargo audit                         # RustSec vulnerabilities
```

See [PERFORMANCE.md](PERFORMANCE.md) for the optimisation notes and benchmarks.

## License

Dual-licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
