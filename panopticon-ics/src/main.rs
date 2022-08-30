#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::egui::plot::Text;
use eframe::{egui, epaint::TextureHandle};
use sscanf::scanf;
use std::collections::HashSet;
use std::collections::{hash_map::Entry, HashMap};
use std::path::PathBuf;
use std::{fs, os::unix::prelude::OsStrExt};

#[derive(Parser, Clone)]
#[clap(name = "panopticon-ics")]
#[clap(author = "Govind Pimpale <gpimpale29@gmail.com>")]
#[clap(version = "0.1")]
/// Converts panopticon data to an ICS file
struct Opts {
    /// Panopticon image directory
    dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let Opts { dir } = Opts::parse();

    let mut snapshots = vec![];

    // parse each day folder
    for maybe_day_path in fs::read_dir(dir)? {
        let day_path = maybe_day_path?;
        let day = NaiveDate::parse_from_str(&day_path.file_name().to_string_lossy(), "%Y-%m-%d")?;
        // parse each snapshot
        for maybe_snapshot_path in fs::read_dir(day_path.path())? {
            let snapshot_path = maybe_snapshot_path?;
            let input_data = snapshot_path.file_name().to_string_lossy().to_string();
            let (hms, screen, afk_str) = scanf!(input_data, "{}_screen-{}{}", str, u64, str)
                .map_err(|e| e.to_string())?;

            let afk = afk_str == "_AFK.png";

            // get real time
            let time = Local
                .from_local_datetime(&day.and_time(NaiveTime::parse_from_str(hms, "%H:%M:%S")?))
                .unwrap();

            snapshots.push((time, screen, afk, snapshot_path.path()));
        }
    }

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "panopticon-ics",
        options,
        Box::new(|cc| {
            Box::new(MyApp {
                snapshots: snapshots.into_iter().fold(
                    HashMap::new(),
                    |mut acc, (dt, screen, afk, path)| {
                        match acc.entry(dt) {
                            Entry::Vacant(entry) => {
                                entry.insert(Snapshot {
                                    screenshots: HashMap::from([(
                                        screen,
                                        LazyImage { path, tex: None },
                                    )]),
                                    afk,
                                });
                            }
                            Entry::Occupied(mut entry) => {
                                entry
                                    .get_mut()
                                    .screenshots
                                    .insert(screen, LazyImage { path, tex: None });
                            }
                        };

                        return acc;
                    },
                ),
            })
        }),
    );

    return Ok(());
}

struct LazyImage {
    path: PathBuf,
    tex: Option<TextureHandle>,
}

impl LazyImage {
    fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
        let image = image::io::Reader::open(path)?.decode()?;
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        ))
    }

    fn get_texture(&mut self, ctx: &egui::Context) -> &mut TextureHandle {
        return self.tex.get_or_insert_with(|| {
            // Load the texture only once.
            ctx.load_texture(
                self.path.to_string_lossy(),
                Self::load_image_from_path(&self.path).unwrap(),
                egui::TextureFilter::Linear,
            )
        });
    }
}

struct Snapshot {
    screenshots: HashMap<u64, LazyImage>,
    afk: bool,
}

struct MyApp {
    snapshots: HashMap<DateTime<Local>, Snapshot>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            for (k, mut v) in self.snapshots.iter_mut() {
                ui.horizontal(|ui| {
                    for (k, mut v) in v.screenshots.iter_mut() {
                        let tex = v.get_texture(ctx);
                        let tex_size = tex.size_vec2();
                        ui.image(tex, tex_size);
                    }
                });
            }
        });
    }
}
