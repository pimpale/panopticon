#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::egui;
use sscanf::lazy_static::lazy::Lazy;
use sscanf::scanf;
use std::collections::HashSet;
use std::collections::{hash_map::Entry, HashMap};
use std::fs;
use std::path::PathBuf;

mod lazy_image;
use lazy_image::LazyImage;

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
            let (hms, screen, afk_str) =
                scanf!(input_data, "{}_screen-{}{}", str, u64, str).map_err(|e| e.to_string())?;

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
        Box::new(|_| {
            Box::new(MyApp {
                snapshots: snapshots.into_iter().fold(
                    HashMap::new(),
                    |mut acc, (dt, screen, afk, path)| match acc.entry(dt) {
                        Entry::Vacant(x) => {
                            let screenshots = HashMap::from([(screen, LazyImage::new(path))]);
                            x.insert(Snapshot { screenshots, afk });
                            acc
                        }
                        Entry::Occupied(x) => {
                            x.get_mut().screenshots.insert(screen, LazyImage::new(path));
                            acc
                        }
                    },
                ),
            })
        }),
    );

    return Ok(());
}

struct Snapshot {
    // TODO: use more images
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
            for (k, snapshot) in self.snapshots.iter_mut() {
                let img = snapshot.screenshot.get_texture();
                img.show_max_size(ui, ui.available_size());
            }
        });
    }
}
