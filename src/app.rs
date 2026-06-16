//! The eframe application: state, input handling and rendering.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use eframe::egui;
use egui::{Align2, Color32, FontId, Rect, TextureHandle, Vec2};

use crate::decode::Decoder;
use crate::image_loader;
use crate::input::InputState;
use crate::update::{self, UpdateStatus, Updater};

/// Longest edge (px) a decoded image is downscaled to. Bounds GPU memory and
/// stays within driver texture-size limits while leaving room to zoom in.
const MAX_TEXTURE_SIDE: u32 = 4096;

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 32.0;

pub struct ImageViewerApp {
    images: Vec<PathBuf>,
    index: usize,

    decoder: Decoder,
    /// Decoded textures, keyed by playlist index. Bounded to a small window
    /// around the current image.
    textures: HashMap<usize, TextureHandle>,
    /// Indices currently being decoded (avoids duplicate requests).
    pending: HashSet<usize>,
    /// Last decode error for the current image, if any.
    error: Option<String>,

    // View transform.
    zoom: f32,
    offset: Vec2,
    /// When set, the view is recomputed to fit the window each frame.
    fit: bool,

    updater: Updater,
    show_help: bool,
}

impl ImageViewerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        path: Option<String>,
        check_updates: bool,
    ) -> Self {
        let ctx = cc.egui_ctx.clone();

        let (images, index) = match path {
            Some(ref p) => {
                let list = image_loader::load_from_path(p);
                let idx = image_loader::index_of(&list, Path::new(p));
                (list, idx)
            }
            None => (Vec::new(), 0),
        };

        let updater = Updater::new(ctx.clone());
        if check_updates {
            updater.check();
        }

        let mut app = Self {
            images,
            index,
            decoder: Decoder::new(ctx, MAX_TEXTURE_SIDE),
            textures: HashMap::new(),
            pending: HashSet::new(),
            error: None,
            zoom: 1.0,
            offset: Vec2::ZERO,
            fit: true,
            updater,
            show_help: false,
        };
        app.ensure_window_loaded();
        app
    }

    /// Indices that should be kept decoded: current and its neighbours.
    fn window_indices(&self) -> Vec<usize> {
        let n = self.images.len();
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            return vec![0];
        }
        vec![self.index, (self.index + 1) % n, (self.index + n - 1) % n]
    }

    /// Queue decodes for the current image and neighbours, and evict the rest.
    fn ensure_window_loaded(&mut self) {
        let window = self.window_indices();
        let keep: HashSet<usize> = window.iter().copied().collect();

        self.textures.retain(|k, _| keep.contains(k));
        self.pending.retain(|k| keep.contains(k));

        for idx in window {
            if !self.textures.contains_key(&idx) && !self.pending.contains(&idx) {
                if let Some(path) = self.images.get(idx) {
                    self.pending.insert(idx);
                    self.decoder.request(idx, path.clone());
                }
            }
        }
    }

    /// Pull decoded images from the worker and turn them into textures.
    fn collect_decoded(&mut self, ctx: &egui::Context) {
        for decoded in self.decoder.drain() {
            self.pending.remove(&decoded.index);
            match decoded.result {
                Ok(image) => {
                    let handle = ctx.load_texture("image", image, egui::TextureOptions::LINEAR);
                    self.textures.insert(decoded.index, handle);
                    if decoded.index == self.index {
                        self.error = None;
                    }
                }
                Err(e) => {
                    if decoded.index == self.index {
                        self.error = Some(e);
                    }
                }
            }
        }
    }

    fn next(&mut self) {
        if self.images.len() > 1 {
            self.index = (self.index + 1) % self.images.len();
            self.on_image_changed();
        }
    }

    fn prev(&mut self) {
        if self.images.len() > 1 {
            self.index = (self.index + self.images.len() - 1) % self.images.len();
            self.on_image_changed();
        }
    }

    fn go_to(&mut self, index: usize) {
        if index < self.images.len() && index != self.index {
            self.index = index;
            self.on_image_changed();
        }
    }

    fn on_image_changed(&mut self) {
        self.error = None;
        self.fit = true;
        self.ensure_window_loaded();
    }

    fn open_path(&mut self, path: &Path) {
        let Some(p) = path.to_str() else { return };
        self.images = image_loader::load_from_path(p);
        self.index = image_loader::index_of(&self.images, path);
        self.textures.clear();
        self.pending.clear();
        self.on_image_changed();
    }

    fn handle_keys(&mut self, ctx: &egui::Context, input: &InputState) {
        if input.next {
            self.next();
        }
        if input.prev {
            self.prev();
        }
        if input.first {
            self.go_to(0);
        }
        if input.last && !self.images.is_empty() {
            self.go_to(self.images.len() - 1);
        }
        if input.zoom_in {
            self.zoom = (self.zoom * 1.25).clamp(MIN_ZOOM, MAX_ZOOM);
            self.fit = false;
        }
        if input.zoom_out {
            self.zoom = (self.zoom / 1.25).clamp(MIN_ZOOM, MAX_ZOOM);
            self.fit = false;
        }
        if input.reset_view {
            self.fit = true;
        }
        if input.toggle_help {
            self.show_help = !self.show_help;
        }
        if input.toggle_fullscreen {
            let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.into_iter().find_map(|f| f.path) {
            self.open_path(&path);
        }
    }
}

impl eframe::App for ImageViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let input = InputState::read(ctx);
        self.handle_keys(ctx, &input);
        self.handle_dropped_files(ctx);
        self.collect_decoded(ctx);

        self.top_bar(ctx);
        self.help_window(ctx);
        self.central(ctx);
    }
}

impl ImageViewerApp {
    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.images.is_empty() {
                    ui.label("Aucune image");
                } else {
                    let name = self.images[self.index]
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?");
                    ui.label(format!("{} / {}", self.index + 1, self.images.len()));
                    ui.separator();
                    ui.label(name);
                    ui.separator();
                    ui.monospace(format!("{:.0}%", self.zoom * 100.0));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("？ Aide").clicked() {
                        self.show_help = !self.show_help;
                    }
                    self.update_widgets(ui);
                });
            });
        });
    }

    /// Update banner / controls in the top bar.
    fn update_widgets(&mut self, ui: &mut egui::Ui) {
        match self.updater.status() {
            UpdateStatus::Checking => {
                ui.add(egui::Spinner::new());
                ui.label("Vérification des mises à jour…");
            }
            UpdateStatus::Available { version, .. } => {
                if ui
                    .button(format!("⬇ Mettre à jour vers v{version}"))
                    .on_hover_text("Télécharge et installe la nouvelle version")
                    .clicked()
                {
                    self.updater.install();
                }
            }
            UpdateStatus::Installing => {
                ui.add(egui::Spinner::new());
                ui.label("Installation de la mise à jour…");
            }
            UpdateStatus::Installed { version } => {
                if ui.button(format!("↻ Redémarrer (v{version})")).clicked() {
                    let _ = update::restart();
                }
                ui.colored_label(Color32::LIGHT_GREEN, "Mise à jour installée");
            }
            UpdateStatus::Failed(_) => {
                ui.weak("Maj indisponible");
            }
            UpdateStatus::Idle | UpdateStatus::UpToDate => {}
        }
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.show_help {
            return;
        }
        let mut open = self.show_help;
        egui::Window::new("Aide — Rust Viewer Pro")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                egui::Grid::new("help_grid").striped(true).show(ui, |ui| {
                    for (keys, action) in SHORTCUTS {
                        ui.monospace(*keys);
                        ui.label(*action);
                        ui.end_row();
                    }
                });
                ui.separator();
                ui.label(format!("Version {}", Updater::current_version()));
            });
        self.show_help = open;
    }

    fn central(&mut self, ctx: &egui::Context) {
        let frame = egui::Frame::none().fill(Color32::from_gray(18));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            let rect = response.rect;
            let center = rect.center();

            if self.images.is_empty() {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    "Glissez-déposez une image ou un dossier ici",
                    FontId::proportional(20.0),
                    Color32::GRAY,
                );
                return;
            }

            let Some(texture) = self.textures.get(&self.index) else {
                // Still decoding (or failed).
                if let Some(err) = &self.error {
                    painter.text(
                        center,
                        Align2::CENTER_CENTER,
                        format!("Erreur : {err}"),
                        FontId::proportional(16.0),
                        Color32::LIGHT_RED,
                    );
                } else {
                    painter.text(
                        center,
                        Align2::CENTER_CENTER,
                        "Chargement…",
                        FontId::proportional(18.0),
                        Color32::GRAY,
                    );
                }
                return;
            };

            let img_size = texture.size_vec2();

            // Fit-to-window recomputes the transform each frame (keeps fitting
            // on resize) until the user zooms or pans.
            if self.fit {
                let scale = (rect.width() / img_size.x).min(rect.height() / img_size.y);
                self.zoom = scale.clamp(MIN_ZOOM, MAX_ZOOM);
                self.offset = Vec2::ZERO;
            }

            // Pan via drag.
            if response.dragged() {
                self.offset += response.drag_delta();
                self.fit = false;
            }

            // Zoom around the cursor via scroll wheel.
            if response.hovered() {
                let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    if let Some(cursor) = response.hover_pos() {
                        let old = self.zoom;
                        let new = (self.zoom * (1.0 + scroll * 0.0015)).clamp(MIN_ZOOM, MAX_ZOOM);
                        let factor = new / old;
                        self.offset = (cursor - center) * (1.0 - factor) + self.offset * factor;
                        self.zoom = new;
                        self.fit = false;
                    }
                }
            }

            let display = img_size * self.zoom;
            let image_center = center + self.offset;
            let image_rect = Rect::from_center_size(image_center, display);
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(texture.id(), image_rect, uv, Color32::WHITE);
        });
    }
}

/// Keyboard shortcuts shown in the help window.
const SHORTCUTS: &[(&str, &str)] = &[
    ("→ / Espace", "Image suivante"),
    ("← / Retour", "Image précédente"),
    ("Début / Fin", "Première / dernière image"),
    ("Molette", "Zoom (centré sur le curseur)"),
    ("+ / -", "Zoom avant / arrière"),
    ("Glisser", "Déplacer l'image"),
    ("0 / R", "Réinitialiser la vue (ajuster)"),
    ("F / F11", "Plein écran"),
    ("H / F1", "Afficher / masquer cette aide"),
];
