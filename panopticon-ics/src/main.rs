#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::{egui, epaint::TextureHandle};
use nom::{
    branch::alt,
    bytes::{complete::tag, complete::take_until},
    character::complete,
    sequence::tuple,
};
use std::collections::HashSet;
use std::collections::{hash_map::Entry, HashMap};
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

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let Opts { dir } = Opts::parse();

    let parser = tuple((
        take_until("_"),
        tag("_screen-"),
        complete::u64,
        alt((tag("_"), tag("_AFK"))),
    ));

    let snapshots = vec![];

    // parse each day folder
    for maybe_day_path in fs::read_dir(dir)? {
        let day_path = maybe_day_path?;
        let day = NaiveDate::parse_from_str(&day_path.file_name().to_string_lossy(), "%Y-%m-%d")?;
        // parse each snapshot
        for maybe_snapshot_path in fs::read_dir(day_path.path())? {
            let snapshot_path = maybe_snapshot_path?;
            let (_, (hms, _, screen, afk)) = parser(snapshot_path.file_name().as_bytes())?;

            // get real time
            let time = Local
                .from_local_datetime(&day.and_time(NaiveTime::parse_from_str(
                    &String::from_utf8_lossy(hms),
                    "%H:%M:%S",
                )?))
                .unwrap();

            // afk true if image ends with _AFK
            let afk = afk == b"_AFK";

            // load image
            let img = load_image_from_path(&snapshot_path.path())?;

            snapshots.push((time, screen, afk, img));
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
                    |acc, (dt, screen, afk, img)| {
                        // load tex
                        let tex = cc.egui_ctx.load_texture(
                            &format!("{}_screen{}", dt, screen),
                            img,
                            egui::TextureFilter::Linear,
                        );

                        match acc.entry(dt) {
                            Entry::Vacant(entry) => {
                                entry.insert(Snapshot {
                                    screenshots: HashMap::from([(screen, tex)]),
                                    afk,
                                });
                            }
                            Entry::Occupied(entry) => {
                                entry.get_mut().screenshots.insert(screen, tex);
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

struct Snapshot {
    screenshots: HashMap<u64, TextureHandle>,
    afk: bool,
}

struct MyApp {
    snapshots: HashMap<DateTime<Local>, Snapshot>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.dir);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.dir, self.age));
        });
    }
}
