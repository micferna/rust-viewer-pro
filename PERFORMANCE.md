# Performance notes

Rust Viewer Pro is built so the UI thread never blocks. This document records
the design choices, the build configuration and reproducible benchmarks.

## Architecture

| Concern                | Approach |
| ---------------------- | -------- |
| Decoding               | Runs on a dedicated worker thread. The UI sends a `(index, path)` request and polls results each frame — it never calls `image::open` itself. |
| Navigation latency     | The current image **and both neighbours** are decoded ahead of time, so stepping through a folder usually hits a warm texture (instant). |
| Memory                 | The texture cache is bounded to a 3-image window around the current index; textures outside it are dropped, freeing GPU memory. |
| Large images           | Decoded images are downscaled so their longest edge is ≤ 4096 px, keeping GPU uploads within driver limits while leaving headroom to zoom. |
| Repaints               | The worker calls `Context::request_repaint()` when a result is ready, so the UI wakes only when there is something new to show. |

### Before / after

The original prototype called `image::open(...)` **synchronously inside the
update loop**. Decoding a 12-megapixel image therefore froze the window for the
full decode time (see below) on every navigation. The reworked pipeline moves
that cost off the UI thread entirely: the window stays interactive and shows a
`Chargement…` placeholder until the texture is ready.

## Build configuration

Release profile (`Cargo.toml`):

```toml
[profile.release]
opt-level = 3        # full optimisation
lto = "fat"          # cross-crate inlining
codegen-units = 1    # best codegen at the cost of build time
panic = "unwind"     # required to catch decoder panics on hostile images
strip = true         # strip symbols
```

Only the codecs we ship are compiled in (`image` is built with
`default-features = false` and an explicit format list), and eframe uses the
`glow` backend only — the `wgpu`/`metal` stack is never built.

## Benchmarks

Decode + downscale of a generated 4000×3000 (12 MP) PNG, measured with
[criterion](https://github.com/bheisler/criterion.rs):

| Downscale cap (longest edge) | Median time |
| ---------------------------- | ----------- |
| 1024 px                      | ~186 ms     |
| 2048 px                      | ~388 ms     |
| 4096 px (no resize needed)   | ~176 ms     |

Notes:

- The 4096 px row performs **no** resize (the source is 4000 px wide), so it is
  the pure decode + RGBA conversion cost.
- Resizing dominates when it runs; the cost scales with the **output** pixel
  count, which is why 2048 px (≈3.1 MP out) is slower than 1024 px (≈0.8 MP out).
- Every figure above is time spent on the **worker thread**, not the UI thread.

Reproduce:

```sh
cargo bench
# HTML report: target/criterion/report/index.html
```

## Release binary

`target/release/rust-viewer-pro`, x86_64 Linux, stripped: **~11 MB**
(statically includes the TLS stack for self-update).
