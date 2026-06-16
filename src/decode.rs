//! Off-thread image decoding.
//!
//! Decoding large images is expensive and must never run on the UI thread.
//! A dedicated worker thread receives [`Request`]s and returns [`Decoded`]
//! results over channels; the UI polls them each frame.

use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use egui::ColorImage;
use image::imageops::FilterType;
use image::{ImageReader, Limits};

/// Hard caps applied while decoding untrusted images, to defuse decompression
/// bombs (a tiny file declaring enormous dimensions). Generous enough for real
/// photos and panoramas, small enough to never exhaust memory.
const MAX_DIMENSION: u32 = 30_000;
const MAX_ALLOC_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB per allocation

/// A decoded image ready to be uploaded as a GPU texture.
pub struct Decoded {
    /// The image index this result belongs to (used to match the request).
    pub index: usize,
    /// Decoded pixels, or a human-readable error message.
    pub result: Result<ColorImage, String>,
}

struct Request {
    index: usize,
    path: PathBuf,
}

/// Handle to the background decoding worker.
pub struct Decoder {
    req_tx: Sender<Request>,
    res_rx: Receiver<Decoded>,
}

impl Decoder {
    /// Spawn the worker thread. `ctx` is used to wake the UI when a result is
    /// ready; `max_side` caps the longest edge of decoded images.
    pub fn new(ctx: egui::Context, max_side: u32) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<Request>();
        let (res_tx, res_rx) = mpsc::channel::<Decoded>();

        std::thread::Builder::new()
            .name("image-decoder".to_owned())
            .spawn(move || worker(&req_rx, &res_tx, &ctx, max_side))
            .expect("failed to spawn decoder thread");

        Self { req_tx, res_rx }
    }

    /// Queue an image for decoding. Cheap; returns immediately.
    pub fn request(&self, index: usize, path: PathBuf) {
        // A send error only happens if the worker died, in which case the UI
        // simply never receives the result — nothing actionable here.
        let _ = self.req_tx.send(Request { index, path });
    }

    /// Drain all results decoded since the last call.
    pub fn drain(&self) -> Vec<Decoded> {
        self.res_rx.try_iter().collect()
    }
}

fn worker(
    req_rx: &Receiver<Request>,
    res_tx: &Sender<Decoded>,
    ctx: &egui::Context,
    max_side: u32,
) {
    while let Ok(req) = req_rx.recv() {
        // A malformed/hostile image can make a decoder panic. Catch it so one
        // bad file shows an error instead of taking down the whole app, and the
        // worker keeps serving the next requests.
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| decode(&req.path, max_side)))
            .unwrap_or_else(|_| {
                Err(format!(
                    "{}: image rejected (decoder error)",
                    req.path.display()
                ))
            });
        if res_tx
            .send(Decoded {
                index: req.index,
                result,
            })
            .is_err()
        {
            break; // UI gone — stop working.
        }
        ctx.request_repaint();
    }
}

/// Decode and downscale a single image to an egui [`ColorImage`].
///
/// The format is detected from the file's *content*, not its extension, and
/// decoding runs under strict size/allocation limits.
pub fn decode(path: &Path, max_side: u32) -> Result<ColorImage, String> {
    let err = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());

    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC_BYTES);

    let mut reader = ImageReader::open(path)
        .map_err(|e| err(&e))?
        .with_guessed_format()
        .map_err(|e| err(&e))?;
    reader.limits(limits);
    let img = reader.decode().map_err(|e| err(&e))?;

    let (w, h) = (img.width(), img.height());
    let img = if w.max(h) > max_side {
        img.resize(max_side, max_side, FilterType::Triangle)
    } else {
        img
    };

    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rvp-test-{name}"))
    }

    #[test]
    fn decodes_a_valid_image_and_downscales() {
        let path = temp_path("valid.png");
        image::RgbaImage::from_pixel(200, 100, image::Rgba([10, 20, 30, 255]))
            .save(&path)
            .unwrap();

        // No downscale needed.
        let full = decode(&path, 4096).unwrap();
        assert_eq!(full.size, [200, 100]);

        // Downscaled to fit within 50 px on the longest edge.
        let small = decode(&path, 50).unwrap();
        assert!(small.size[0] <= 50 && small.size[1] <= 50);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_garbage_without_panicking() {
        let path = temp_path("garbage.png");
        std::fs::write(&path, b"this is definitely not an image").unwrap();

        // Must return an error, not panic or hang.
        assert!(decode(&path, 4096).is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_missing_file() {
        assert!(decode(Path::new("/nonexistent/rvp/none.png"), 4096).is_err());
    }
}
