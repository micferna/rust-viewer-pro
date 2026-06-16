# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-06-16

### Changed

- CI/release: update GitHub Actions (`checkout` v6, `upload-artifact` v7,
  `download-artifact` v8, `action-gh-release` v3).
- Dependabot: group cargo and github-actions updates, skip cargo major bumps,
  monthly cadence.
- README: replace the version badge (shields token-pool failures) with a stable
  download badge.
- Bump `log` to 0.4.32.

## [0.1.1] - 2026-06-16

### Security

- Harden image decoding against hostile/booby-trapped files:
  - decompression-bomb limits (≤ 30000 px per side, ≤ 1 GiB per allocation);
  - panic isolation on the decode thread (`catch_unwind`) so a malformed image
    shows an error instead of crashing the app;
  - content-based format detection instead of trusting the file extension.
- Add `SECURITY.md` documenting the threat model and reporting process.

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

[Unreleased]: https://github.com/micferna/rust-viewer-pro/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/micferna/rust-viewer-pro/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/micferna/rust-viewer-pro/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/micferna/rust-viewer-pro/releases/tag/v0.1.0
