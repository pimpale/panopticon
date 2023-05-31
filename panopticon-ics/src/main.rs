#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod lazy_image;
mod timeline_widget;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use clap::Parser;
use eframe::egui;
use sscanf::scanf;
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::ops::Bound::{Excluded, Included, Unbounded};

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
            Box::new(MyApp::new(
                // get the earliest time listed or now
                snapshots
                    .first_key_value()
                    .map(|x| x.0.clone())
                    .unwrap_or(Local::now()),
                snapshots,
            ))
        }),
    )?;

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

    // variables that capture per snapshot state
    hint_text: String,
    currently_visible_shortcuts: Vec<String>,

    // variables that capture frame-to-frame temporary state
    scroll_dirty: bool,
}

impl MyApp {
    pub fn new(
        current_time: DateTime<Local>,
        snapshots: BTreeMap<DateTime<Local>, Snapshot>,
    ) -> Self {
        MyApp {
            zoom_multipler: 1,
            current_time,
            snapshots,
            hint_text: String::new(),
            currently_visible_shortcuts: Vec::new(),
            scroll_dirty: false,
        }
    }

    fn on_new_snapshot(&mut self) {
        // the hint text is the previous snapshot classification
        self.hint_text = self
            .snapshots
            .range((Unbounded, Excluded(self.current_time)))
            .next_back()
            .map(|(_, v)| v.classification.clone())
            .unwrap_or(String::new());
        // the shortcut codes are the most common in the previous 4 hours
        let mut popular_classifications = BTreeMap::new();
        for (_, v) in self.snapshots.range((
            Included(self.current_time - chrono::Duration::hours(4)),
            Excluded(self.current_time),
        )) {
            if !v.classification.is_empty() {
                popular_classifications
                    .entry(v.classification.clone())
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }

        let mut popular_classifications_vec: Vec<_> = popular_classifications.into_iter().collect();
        popular_classifications_vec.sort_by_cached_key(|(_, v)| -*v);
        popular_classifications_vec.truncate(10);
        self.currently_visible_shortcuts = popular_classifications_vec
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        // clear all except the nearest 32 on either side
        for (_, s) in self
            .snapshots
            .range_mut((Unbounded, Excluded(self.current_time)))
            .rev()
            .skip(32)
        {
            for s in s.screenshots.values_mut() {
                s.clear();
            }
        }
        for (_, s) in self
            .snapshots
            .range_mut((Excluded(self.current_time), Unbounded))
            .skip(32)
        {
            for s in s.screenshots.values_mut() {
                s.clear();
            }
        }
    }
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

                let response =
                    ui.add(egui::Slider::new(&mut self.zoom_multipler, 1..=100).text("Zoom"));
                if response.changed() {
                    self.scroll_dirty = true;
                }

                let timeline_resp = ui.add(TimelineWidget::new(
                    self.zoom_multipler,
                    &mut self.current_time,
                    self.scroll_dirty,
                    self.snapshots.iter().map(|(k, v)| {
                        (
                            *k,
                            TimelineMarker {
                                stroke: egui::Stroke {
                                    color: if !v.classification.is_empty() {
                                        egui::Color32::LIGHT_GREEN
                                    } else if v.afk {
                                        egui::Color32::LIGHT_YELLOW
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    },
                                    width: 1.0,
                                },
                                label: &v.classification,
                            },
                        )
                    }),
                ));
                self.scroll_dirty = false;
                if timeline_resp.changed {
                    self.on_new_snapshot();
                }
            });

        egui::TopBottomPanel::top("Controls").show(ctx, |ui| {
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
                let response = ui
                    .horizontal(|ui| {
                        ui.label("Current Task: ");
                        let task_entrybox =
                            egui::TextEdit::singleline(&mut snapshot.classification)
                                .hint_text(&self.hint_text);
                        return ui.add(task_entrybox);
                    })
                    .inner;

                let user_input_parse = |classification: &String| {
                    // if was empty, then accept hint
                    if classification.is_empty() {
                        return Ok(self.hint_text.clone());
                    } else {
                        let mut chars = classification.chars();
                        if chars.next() == Some('\\') {
                            let rest = chars.collect::<String>();
                            match rest.parse::<usize>() {
                                Ok(n) => {
                                    if n < self.currently_visible_shortcuts.len() {
                                        return Ok(self.currently_visible_shortcuts[n].clone());
                                    } else {
                                        return Err(format!("Invalid shortcut number: {}", n));
                                    }
                                }
                                Err(e) => {
                                    return Err(format!(
                                        "Couldn't parse shortcut code: {}",
                                        e.to_string()
                                    ));
                                }
                            }
                        } else {
                            return Ok(classification.clone());
                        }
                    }
                };

                let user_input_parse_result = user_input_parse(&snapshot.classification);

                match user_input_parse_result {
                    Ok(user_input_parse) => {
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            snapshot.classification = user_input_parse;
                            // if there's a one after, then grab its focus
                            if let Some((next_time, _)) = iter.next() {
                                // update pointer
                                self.current_time = *next_time;
                                self.scroll_dirty = true;
                                self.on_new_snapshot();
                                response.request_focus();
                            }
                        }
                    }
                    Err(error) => {
                        ui.colored_label(egui::Color32::RED, egui::RichText::new(error).small());
                    }
                }
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (i, x) in self.currently_visible_shortcuts.iter().enumerate() {
                            ui.label(format!("\\{}: {}", i, x));
                            ui.add_space(10.0);
                        }
                    });
                });

            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                self.current_time = self
                    .snapshots
                    .range((Unbounded, Excluded(self.current_time)))
                    .next_back()
                    .map(|(x, _)| x.clone())
                    .unwrap_or(self.current_time);
                self.scroll_dirty = true;
                self.on_new_snapshot();
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                self.current_time = self
                    .snapshots
                    .range((Excluded(self.current_time), Unbounded))
                    .next()
                    .map(|(x, _)| x.clone())
                    .unwrap_or(self.current_time);
                self.scroll_dirty = true;
                self.on_new_snapshot();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((_, snapshot)) = self.snapshots.range_mut(self.current_time..).next() {
                // show a list of the screenshots (expand horizontally to fill, but can take up as much space as needed vertically)
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
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
