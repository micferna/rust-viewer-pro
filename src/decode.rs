//! Off-thread image decoding.
//!
//! Decoding large images is expensive and must never run on the UI thread.
//! A dedicated worker thread receives [`Request`]s and returns [`Decoded`]
//! results over channels; the UI polls them each frame.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use egui::ColorImage;
use image::imageops::FilterType;

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
        let result = decode(&req.path, max_side);
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
pub fn decode(path: &std::path::Path, max_side: u32) -> Result<ColorImage, String> {
    let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?;

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
