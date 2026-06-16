//! Rust Viewer Pro — GUI entry point.
#![forbid(unsafe_code)]
// On Windows, do not spawn a console window for the GUI build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use clap::Parser;

use rust_viewer_pro::app::ImageViewerApp;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Image file or directory to open.
    path: Option<String>,

    /// Disable the automatic update check on startup.
    #[arg(long)]
    no_update_check: bool,
}

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    let viewport = egui::ViewportBuilder::default()
        .with_title("Rust Viewer Pro")
        .with_inner_size([1100.0, 750.0])
        .with_min_inner_size([400.0, 300.0])
        .with_icon(load_icon());

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Rust Viewer Pro",
        options,
        Box::new(move |cc| Box::new(ImageViewerApp::new(cc, args.path, !args.no_update_check))),
    )
}

/// Decode the embedded window icon. Falls back to an empty icon on failure.
fn load_icon() -> Arc<egui::IconData> {
    const ICON: &[u8] = include_bytes!("../assets/icon-256.png");
    let icon = image::load_from_memory(ICON)
        .map(|img| {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            egui::IconData {
                rgba: img.into_raw(),
                width,
                height,
            }
        })
        .unwrap_or_default();
    Arc::new(icon)
}
