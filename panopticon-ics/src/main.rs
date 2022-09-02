#![feature(map_first_last)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod lazy_image;
mod timeline_widget;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::egui;
use sscanf::scanf;
use std::collections::HashSet;
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;

use lazy_image::LazyImage;
use timeline_widget::TimelineWidget;

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

    let mut snapshots = BTreeMap::new();

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
            let time = match Local
                .from_local_datetime(&day.and_time(NaiveTime::parse_from_str(hms, "%H:%M:%S")?))
            {
                chrono::LocalResult::None => Err(format!(
                    "No valid time corresponds to path {:?}/{:?}",
                    day_path.file_name().to_string_lossy(),
                    snapshot_path.file_name().to_string_lossy()
                ))?,
                chrono::LocalResult::Single(t) => t,
                chrono::LocalResult::Ambiguous(t, _) => t,
            };

            // create image
            let lazy_image = LazyImage::new(snapshot_path.path());
            match snapshots.entry(time) {
                Entry::Vacant(x) => {
                    x.insert(Snapshot {
                        screenshots: BTreeMap::from([(screen, lazy_image)]),
                        afk,
                    });
                }
                Entry::Occupied(mut x) => {
                    x.get_mut().screenshots.insert(screen, lazy_image);
                }
            }
        }
    }

    eframe::run_native(
        "panopticon-ics",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            // get the earliest time listed or now
            let current_time = snapshots
                .first_key_value()
                .map(|x| x.0.clone())
                .unwrap_or(Local::now());
            Box::new(MyApp {
                zoom_multipler: 1,
                current_time,
                snapshots,
            })
        }),
    );

    return Ok(());
}

struct Snapshot {
    screenshots: BTreeMap<u64, LazyImage>,
    afk: bool,
}

struct MyApp {
    snapshots: BTreeMap<DateTime<Local>, Snapshot>,
    current_time: DateTime<Local>,
    zoom_multipler: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Calendar").resizable(false).show(ctx, |ui| {
            ui.heading("Calendar");
            ui.add(egui::Slider::new(&mut self.zoom_multipler, 1..=100).text("Zoom"));
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(TimelineWidget::new(
                    self.zoom_multipler,
                    self.current_time,
                    self.snapshots.keys().cloned(),
                ))
            });
        });

        let current_snapshot = self.snapshots.range_mut(self.current_time..).next();

        egui::TopBottomPanel::bottom("Controls").show(ctx, |ui| {
            ui.heading("My egui Application");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((time, snapshot)) = current_snapshot {
                // show a list of the screenshots (expand horizontally to fill, but can take up as much space as needed vertically)
                egui::ScrollArea::vertical()
                    .always_show_scroll(true)
                    .show(ui, |ui| {
                        for lazy_img in snapshot.screenshots.values_mut() {
                            let img = lazy_img.get_texture();
                            img.show_max_size(ui, [ui.available_width(), f32::INFINITY].into());
                        }
                    });
            }
        });
    }
}
