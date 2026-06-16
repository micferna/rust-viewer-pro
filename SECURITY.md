# Security policy

## Reporting a vulnerability

Please report security issues privately via GitHub Security Advisories
("Report a vulnerability" on the **Security** tab) rather than a public issue.
We aim to acknowledge reports within a few days.

## Threat model

Rust Viewer Pro opens **untrusted image files**. The decoders are the main
attack surface, so the design minimises and contains that risk.

### What protects you

- **Memory-safe, pure-Rust decoders.** PNG, JPEG, WebP and GIF are decoded by
  the Rust `image` crate (`png`, `zune-jpeg`, `image-webp`, `gif`) — not the C
  libraries (`libpng`, `libjpeg`, `libwebp`) behind most historical
  image-parsing RCEs (e.g. CVE-2023-4863). Rust's memory safety removes the
  buffer-overflow class that turns a malformed image into code execution.
- **`#![forbid(unsafe_code)]`** in this project's own crates.
- **No SVG, no EXIF parsing.** We deliberately do not support SVG (XML / scripts
  / XXE) nor parse EXIF metadata, avoiding two large attack surfaces.
- **Decompression-bomb limits.** Decoding runs under hard caps
  (max 30000 px per side, ≤ 1 GiB per allocation), so a tiny file that declares
  enormous dimensions is rejected instead of exhausting memory.
- **Panic isolation.** Decoding happens on a worker thread and is wrapped in
  `catch_unwind`; a hostile image that makes a decoder panic shows an error
  rather than crashing the application.
- **Content-based format detection.** The format is determined from the file's
  bytes, not its extension.

### Residual risk

- A malformed image may still fail to decode — by design it is rejected with an
  on-screen error; it cannot run code or read other files.
- Decoder correctness ultimately depends on upstream crates. We track advisories
  with `cargo-deny` / `cargo-audit` in CI and Dependabot keeps dependencies
  current.

## Updates & supply chain

- Releases are built in GitHub Actions from tagged commits; each release ships a
  `SHA256SUMS` file.
- The in-app self-update downloads release assets from GitHub over TLS (rustls).
- Dependencies are vetted in CI (`cargo deny check`: advisories, licenses,
  sources) on every push and pull request.
