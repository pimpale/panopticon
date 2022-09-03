use chrono::{DateTime, Duration, DurationRound, Local};
use eframe::egui;
use std::collections::BTreeSet;

const BASE_PIXELS_PER_HOUR: f32 = 50.0;

pub struct TimelineWidget {
    zoom_multipler: u32,
    selected_time: DateTime<Local>,
    times: BTreeSet<DateTime<Local>>,
}

impl TimelineWidget {
    pub fn new<I>(zoom_multipler: u32, selected_time: DateTime<Local>, times: I) -> Self
    where
        I: IntoIterator<Item = DateTime<Local>>,
    {
        TimelineWidget {
            zoom_multipler,
            times: times.into_iter().collect(),
            selected_time,
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
            .unwrap_or(self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap();
        let last_hour = self
            .times
            .last()
            .cloned()
            .unwrap_or(self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap()
            + Duration::hours(1);

        // computes the pixel offset in this component for a given time
        let get_y_offset = |time: DateTime<Local>| {
            let hours = (time - first_hour).num_seconds() as f32 / 60.0 / 60.0;
            return hours * BASE_PIXELS_PER_HOUR * self.zoom_multipler as f32;
        };

        let (response, painter) = ui.allocate_painter(
            egui::Vec2 {
                x: 200.0,
                y: self.zoom_multipler as f32
                    * (last_hour - first_hour).num_hours() as f32
                    * BASE_PIXELS_PER_HOUR,
            },
            egui::Sense::click_and_drag(),
        );

        let time_mark_region = response.rect;

        println!("{:?}", time_mark_region);

        let alpha = 1.0;

        let visuals = ui.style().visuals.clone();
        let widget_visuals = ui.style().noninteractive();

        // paint background rect
        painter.rect_filled(
            time_mark_region.shrink(visuals.clip_rect_margin),
            widget_visuals.rounding.ne,
            widget_visuals.bg_fill.linear_multiply(alpha * 0.8),
        );

        // decide on time increment based on zoom level
        let time_increment = match self.zoom_multipler {
            1..=4 => Duration::hours(1),
            5..=9 => Duration::minutes(15),
            10..=49 => Duration::minutes(5),
            50..=100 => Duration::minutes(1),
            _ => Duration::seconds(1),
        };

        let mut current_time = first_hour.clone();

        while current_time <= last_hour {
            let y_offset = get_y_offset(current_time);

            painter.hline(
                time_mark_region.left()..=time_mark_region.right(),
                time_mark_region.top() + y_offset,
                widget_visuals.bg_stroke,
            );

            painter.text(
                egui::epaint::pos2(
                    time_mark_region.left(),
                    time_mark_region.top() + y_offset + 2.0,
                ),
                egui::Align2::LEFT_TOP,
                current_time.format("%m/%d %H:%M:%S"),
                egui::TextStyle::Small.resolve(ui.style()),
                widget_visuals.text_color(),
            );

            current_time += time_increment;
        }

        // paint event markers
        {
            let event_marker_x_offset = 75.0;

            // draw all snapshot times
            for snapshot_time in self.times.iter().cloned() {
                let y_offset = get_y_offset(snapshot_time);

                painter.hline(
                    (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right(),
                    time_mark_region.top() + y_offset,
                    widget_visuals.fg_stroke,
                );
            }
            // paint current_time marker
            painter.hline(
                (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right(),
                time_mark_region.top() + get_y_offset(self.selected_time),
                egui::Stroke::new(1.0, egui::Color32::RED),
            );
        }

        return response;
    }
}
