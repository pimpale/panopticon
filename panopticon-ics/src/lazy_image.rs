use eframe::egui;
use std::fs;
use std::path::PathBuf;

pub struct LazyImage {
    path: PathBuf,
    img: Option<Result<egui_extras::RetainedImage, String>>,
}

impl LazyImage {
    pub fn new(path: PathBuf) -> LazyImage {
        LazyImage { path, img: None }
    }

    pub fn show_max_size(&mut self, ui: &mut egui::Ui, size: egui::epaint::Vec2) -> egui::Response {
        let img = self.img.get_or_insert_with(|| {
            // Load the texture only once.
            egui_extras::RetainedImage::from_image_bytes(
                self.path.to_string_lossy(),
                &fs::read(&self.path).unwrap(),
            )
        });

        match img {
            Ok(img) => img.show_max_size(ui, size),
            Err(err) => {
                ui.label(err.clone())
            }
        }
    }

    pub fn clear(&mut self) {
        self.img = None;
    }
}
