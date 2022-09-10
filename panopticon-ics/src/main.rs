#![feature(map_first_last)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod lazy_image;
mod timeline_widget;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::egui;
use sscanf::scanf;
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::ops::Bound::{Excluded, Unbounded};

use lazy_image::LazyImage;
use timeline_widget::{TimelineMarker, TimelineWidget};

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
                        classification: String::new(),
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
                scroll_dirty: false,
            })
        }),
    );

    return Ok(());
}

struct Snapshot {
    screenshots: BTreeMap<u64, LazyImage>,
    afk: bool,
    classification: String,
}

struct MyApp {
    // variables that capture relatively permanent state
    snapshots: BTreeMap<DateTime<Local>, Snapshot>,
    current_time: DateTime<Local>,
    zoom_multipler: u32,

    // variables that capture temporary state
    scroll_dirty: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Calendar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Panopticon ICS");

                ui.collapsing("Controls", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("- ").strong());
                        ui.label(egui::RichText::new("<Up>").code());
                        ui.label("view previous snapshot");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("- ").strong());
                        ui.label(egui::RichText::new("<Down>").code());
                        ui.label("view next snapshot");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("- ").strong());
                        ui.label(egui::RichText::new("<Enter>").code());
                        ui.label("commit classification");
                    });
                });

                ui.heading("Calendar ");
                let old_zoom = self.zoom_multipler;

                ui.add(egui::Slider::new(&mut self.zoom_multipler, 1..=100).text("Zoom"));

                // if we change the zoom, make sure to center the scroll!
                if self.zoom_multipler != old_zoom {
                    self.scroll_dirty = true;
                }

                ui.add(TimelineWidget::new(
                    self.zoom_multipler,
                    &mut self.current_time,
                    self.scroll_dirty,
                    self.snapshots.iter().map(|(k, v)| {
                        (
                            *k,
                            TimelineMarker {
                                stroke: egui::Stroke {
                                    color: if v.classification.len() > 0 {
                                        egui::Color32::LIGHT_GREEN
                                    } else if v.afk {
                                        egui::Color32::LIGHT_GRAY
                                    } else {
                                        egui::Color32::LIGHT_BLUE
                                    },
                                    width: 1.0,
                                },
                            },
                        )
                    }),
                ));
                self.scroll_dirty = false;
            });

        egui::TopBottomPanel::bottom("Controls").show(ctx, |ui| {
            // the hint text is the previous snapshot classification
            let hint_text = self
                .snapshots
                .range((Unbounded, Excluded(self.current_time)))
                .next_back()
                .map(|(_, v)| v.classification.clone())
                .unwrap_or(String::new());

            // create iterator to view current snapshot and next
            let mut iter = self.snapshots.range_mut(self.current_time..);

            // this draws the actual labeler
            if let Some((time, snapshot)) = iter.next() {
                // put other flags here
                ui.horizontal_wrapped(|ui| {
                    ui.heading(time.format("%Y-%m-%d %H:%M:%S").to_string());
                    ui.add_space(20.0);
                    if snapshot.afk {
                        ui.label(
                            egui::RichText::new("AFK")
                                .color(egui::Color32::BLACK)
                                .background_color(egui::Color32::YELLOW)
                                .heading(),
                        );
                    }
                });

                ui.separator();

                // keyboard controls
                ui.horizontal(|ui| {
                    ui.label("Current Task: ");

                    let task_entrybox = egui::TextEdit::singleline(&mut snapshot.classification)
                        .hint_text(hint_text);
                    let response = ui.add(task_entrybox);

                    if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        // if there's a one after, then grab its focus
                        if let Some((next_time, _)) = iter.next() {
                            // update pointer
                            self.current_time = *next_time;
                            self.scroll_dirty = true;

                            // regrab focus so we can keep typing
                            response.request_focus()
                        }
                    }
                });
            }

            if ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
            {
                self.current_time = self
                    .snapshots
                    .range((Unbounded, Excluded(self.current_time)))
                    .next_back()
                    .map(|(x, _)| x.clone())
                    .unwrap_or(self.current_time);
                self.scroll_dirty = true;
            }
            if ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            {
                self.current_time = self
                    .snapshots
                    .range((Excluded(self.current_time), Unbounded))
                    .next()
                    .map(|(x, _)| x.clone())
                    .unwrap_or(self.current_time);
                self.scroll_dirty = true;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((_, snapshot)) = self.snapshots.range_mut(self.current_time..).next() {
                // show a list of the screenshots (expand horizontally to fill, but can take up as much space as needed vertically)
                egui::ScrollArea::vertical()
                    .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for lazy_img in snapshot.screenshots.values_mut() {
                            lazy_img
                                .show_max_size(ui, [ui.available_width(), f32::INFINITY].into());
                        }
                    });
            }
        });
    }
}
