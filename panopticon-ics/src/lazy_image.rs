use std::path::PathBuf;
use egui_extras::RetainedImage;
use std::fs;

pub struct LazyImage {
    path: PathBuf,
    tex: Option<RetainedImage>,
}

impl LazyImage {
    pub fn new(path: PathBuf) -> LazyImage {
        LazyImage { path, tex: None }
    }

    pub fn get_texture(&mut self) -> &mut RetainedImage {
        return self.tex.get_or_insert_with(|| {
            // Load the texture only once.
            let data = fs::read(&self.path).unwrap();
            RetainedImage::from_image_bytes(self.path.to_string_lossy(), &data).unwrap()
        });
    }
}
