//! Benchmarks for the image decode + downscale path.
//!
//! Run with `cargo bench`. A large image is generated once and decoded under
//! different downscale caps to measure the full open → resize → RGBA pipeline.

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{ImageBuffer, Rgba};
use rust_viewer_pro::decode::decode;

/// Generate a deterministic large PNG once and return its path.
fn fixture() -> PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join("rvp-bench-4000x3000.png");
    if !path.exists() {
        let (w, h) = (4000u32, 3000u32);
        let buf = ImageBuffer::from_fn(w, h, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
        });
        ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, buf.into_raw())
            .unwrap()
            .save(&path)
            .expect("write bench fixture");
    }
    path
}

fn bench_decode(c: &mut Criterion) {
    let path = fixture();
    let mut group = c.benchmark_group("decode_4000x3000");
    for max_side in [1024u32, 2048, 4096] {
        group.bench_with_input(
            BenchmarkId::from_parameter(max_side),
            &max_side,
            |b, &max_side| {
                b.iter(|| decode(&path, max_side).expect("decode"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
