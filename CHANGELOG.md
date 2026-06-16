# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-16

### Added

- Cross-platform image viewer (Linux & Windows) built on egui/eframe.
- Off-thread image decoding with neighbour prefetch and a bounded texture cache.
- Smooth wheel zoom centred on the cursor, drag-to-pan and fit-to-window.
- Folder navigation (open a file or a directory) for PNG, JPEG, BMP, WebP, GIF.
- Built-in self-update against GitHub Releases.
- Keyboard shortcuts and in-app help.
- Optional "set as default image viewer" at install time: a debconf prompt in
  the `.deb` and a file-association checkbox in the Windows installer.
- Packaging: Debian `.deb`, Linux AppImage, Windows installer + portable `.zip`.
- CI: format, clippy, tests and `cargo-deny` supply-chain checks.

[Unreleased]: https://github.com/micferna/rust-viewer-pro/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/micferna/rust-viewer-pro/releases/tag/v0.1.0
