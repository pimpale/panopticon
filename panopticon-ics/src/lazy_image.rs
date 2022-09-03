use eframe::egui::{self, Response};
use egui_extras::RetainedImage;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, TryLockError};

pub struct LazyImage {
    path: PathBuf,
    img: Arc<Mutex<Option<RetainedImage>>>,
    loading_started: bool,
}

impl LazyImage {
    pub fn new(path: PathBuf) -> LazyImage {
        LazyImage {
            path,
            img: Arc::new(Mutex::new(None)),
            loading_started: false,
        }
    }

    fn placeholder(ui: &mut egui::Ui) -> Response {
        ui.add(egui::Spinner::new())
    }

    pub fn show_max_size(&mut self, ui: &mut egui::Ui, size: egui::epaint::Vec2) -> egui::Response {
        // if not started loading, start loading image
        if !self.loading_started {
            let img_mut = self.img.clone();
            let path = self.path.clone();
            std::thread::spawn(move || {
                let mut img = img_mut.lock().unwrap();
                let data = fs::read(path.clone()).unwrap();
                let retained_image =
                    RetainedImage::from_image_bytes(path.to_string_lossy(), &data).unwrap();
                *img = Some(retained_image);
            });
            self.loading_started = true;
        }

        // check image state
        match self.img.try_lock() {
            Ok(mut mutex_guard) => match *mutex_guard {
                // render image
                Some(ref mut img) => img.show_max_size(ui, size),
                // otherwise return spinner
                None => Self::placeholder(ui),
            },
            // if blocking, then return spinner
            Err(TryLockError::WouldBlock) => Self::placeholder(ui),
            // panic on poison
            Err(TryLockError::Poisoned(x)) => Err(x).unwrap(),
        }
    }
}
