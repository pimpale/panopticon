use eframe::egui;
use std::path::PathBuf;

pub struct LazyImage {
    uri: String,
}

impl LazyImage {
    pub fn new(path: PathBuf) -> LazyImage {
        // The `file://` loader (egui_extras "file" feature) reads and caches the
        // image lazily on first display.
        LazyImage {
            uri: format!("file://{}", path.to_string_lossy()),
        }
    }

    pub fn show_max_size(&mut self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        ui.add(egui::Image::from_uri(self.uri.clone()).max_size(size))
    }

    pub fn clear(&mut self, ctx: &egui::Context) {
        // Drop the cached texture/bytes so far-away snapshots don't accumulate.
        ctx.forget_image(&self.uri);
    }
}
