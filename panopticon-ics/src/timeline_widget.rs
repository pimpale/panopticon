use chrono::{DateTime, Duration, DurationRound, Local};
use eframe::egui;
use std::collections::BTreeSet;

const BASE_PIXELS_PER_HOUR: f32 = 50.0;

pub struct TimelineWidget {
    zoom_multipler: f32,
    now: DateTime<Local>,
    times: BTreeSet<DateTime<Local>>,
}

impl TimelineWidget {
    pub fn new<I>(zoom_multipler: f32, now: DateTime<Local>, times: I) -> Self
    where
        I: IntoIterator<Item = DateTime<Local>>,
    {
        TimelineWidget {
            zoom_multipler,
            times: times.into_iter().collect(),
            now,
        }
    }
}

impl egui::Widget for TimelineWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // calculate first hour to display
        let first_hour = self
            .times
            .first()
            .cloned()
            .unwrap_or(self.now)
            .duration_trunc(Duration::hours(1))
            .unwrap();
        let last_hour = self
            .times
            .last()
            .cloned()
            .unwrap_or(self.now)
            .duration_trunc(Duration::hours(1))
            .unwrap()
            + Duration::hours(1);

        println!(
            "{:?} {:?} {:?}",
            first_hour,
            last_hour,
            (last_hour - first_hour).num_hours()
        );

        let (response, painter) = ui.allocate_painter(
            egui::Vec2 {
                x: 200.0,
                y: self.zoom_multipler * (last_hour - first_hour).num_hours() as f32 * BASE_PIXELS_PER_HOUR,
            },
            egui::Sense::click_and_drag(),
        );

        let time_mark_region = response.rect;

        println!("{:?}", time_mark_region);

        let alpha = 1.0;

        let visuals = ui.style().visuals.clone();
        let widget_visuals = ui.style().noninteractive();

        painter.rect_filled(
            time_mark_region.shrink(visuals.clip_rect_margin),
            widget_visuals.rounding.ne,
            widget_visuals.bg_fill.linear_multiply(alpha * 0.8),
        );

        let c = time_mark_region.center();
        let r = time_mark_region.width() / 2.0 - 1.0;
        let color = egui::epaint::Color32::from_gray(128);
        let stroke = egui::Stroke::new(1.0, color);
        painter.circle_stroke(c, r, stroke);

        let mut current_time = first_hour.clone();

        while current_time < last_hour {
            println!("{}", current_time.format("%m/%d %H:%M:%S"));
            let y = (current_time - first_hour).num_hours() as f32 * BASE_PIXELS_PER_HOUR * self.zoom_multipler;
            let x = time_mark_region.center().x;

            let text = current_time.format("%m/%d %H:%M:%S");
            painter.text(
                egui::epaint::pos2(x, y),
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Monospace.resolve(ui.style()),
                widget_visuals.text_color().linear_multiply(alpha),
            );

            current_time += Duration::hours(1);
        }

        return response;
    }
}
