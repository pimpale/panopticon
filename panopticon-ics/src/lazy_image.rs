use eframe::egui;
use std::path::PathBuf;

pub struct LazyImage {
    path: PathBuf,
    texture: Option<Result<egui::TextureHandle, String>>,
}

impl LazyImage {
    pub fn new(path: PathBuf) -> LazyImage {
        LazyImage { path, texture: None }
    }

    pub fn show_max_size(&mut self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let texture = self.texture.get_or_insert_with(|| {
            let img = image::open(&self.path).map_err(|e| e.to_string())?.into_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            // screenshots are opaque, so premultiplied == unmultiplied and skips a pass
            let img = egui::ColorImage::from_rgba_premultiplied(size, &img);
            Ok(ui.ctx().load_texture(self.path.to_string_lossy(), img, Default::default()))
        });
        match texture {
            Ok(texture) => ui.add(egui::Image::from_texture(&*texture).max_size(size)),
            Err(e) => ui.colored_label(egui::Color32::RED, e.as_str()),
        }
    }

    pub fn clear(&mut self) {
        self.texture = None;
    }
}
