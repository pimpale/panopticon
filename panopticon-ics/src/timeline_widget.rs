use chrono::{DateTime, Duration, DurationRound, Local};
use eframe::egui;
use std::collections::BTreeMap;

const BASE_PIXELS_PER_HOUR: f32 = 50.0;

fn duration_from_hours_f32(hours: f32) -> Duration {
    return Duration::milliseconds((hours * 60.0 * 60.0 * 1000.0) as i64);
}

fn duration_to_hours_f32(dur: Duration) -> f32 {
    return dur.num_milliseconds() as f32 / (60.0 * 60.0 * 1000.0);
}

pub struct TimelineMarker<'a> {
    pub stroke: egui::epaint::Stroke,
    pub label: &'a str,
}

pub struct TimelineWidget<'a, 'b> {
    zoom_multipler: u32,
    selected_time: &'a mut DateTime<Local>,
    scroll_to_selected: bool,
    markers: BTreeMap<DateTime<Local>, TimelineMarker<'b>>,
}

impl<'a, 'b> TimelineWidget<'a, 'b> {
    pub fn new<I>(
        zoom_multipler: u32,
        selected_time: &'a mut DateTime<Local>,
        scroll_to_selected: bool,
        markers: I,
    ) -> Self
    where
        I: IntoIterator<Item = (DateTime<Local>, TimelineMarker<'b>)>,
    {
        TimelineWidget {
            zoom_multipler,
            markers: markers.into_iter().collect(),
            selected_time,
            scroll_to_selected,
        }
    }

    // calculate first hour to display
    fn first_hour(&self) -> DateTime<Local> {
        return self
            .markers
            .first_key_value()
            .map(|x| x.0.clone())
            .unwrap_or(*self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap();
    }

    // calculate last hour to display
    fn last_hour(&self) -> DateTime<Local> {
        return self
            .markers
            .last_key_value()
            .map(|x| x.0.clone())
            .unwrap_or(*self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap()
            + Duration::hours(1);
    }

    fn hours_to_pixels(&self, hours: Duration) -> f32 {
        return duration_to_hours_f32(hours) * BASE_PIXELS_PER_HOUR * self.zoom_multipler as f32;
    }

    fn pixels_to_hours(&self, pixels: f32) -> Duration {
        return duration_from_hours_f32(
            pixels / (BASE_PIXELS_PER_HOUR * self.zoom_multipler as f32),
        );
    }

    // computes the pixel y_offset in this component for a given time
    fn get_y_offset(&self, time: DateTime<Local>) -> f32 {
        return self.hours_to_pixels(time - self.first_hour());
    }

    // computes the time for a given in this component for a given pixel y_offset
    fn get_time(&self, y_offset: f32) -> DateTime<Local> {
        return self.first_hour() + self.pixels_to_hours(y_offset);
    }

    fn draw_in_viewport(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> egui::Response {
        let (mut response, painter) = ui.allocate_painter(
            egui::Vec2 {
                x: 200.0,
                y: self.hours_to_pixels(self.last_hour() - self.first_hour()),
            },
            egui::Sense::click_and_drag(),
        );

        let time_mark_region = response.rect;

        let widget_visuals = ui.style().noninteractive();

        // decide on time increment based on zoom level
        let time_increment = match self.zoom_multipler {
            1..=4 => Duration::hours(1),
            5..=9 => Duration::minutes(15),
            10..=49 => Duration::minutes(5),
            50..=100 => Duration::minutes(1),
            _ => Duration::seconds(1),
        };

        let first_visible_time = self.get_time(rect.top());
        let last_visible_time = self.get_time(rect.bottom());

        // paint hlines and labels
        {
            // get first visible hline
            let mut current_time = first_visible_time.duration_trunc(time_increment).unwrap();

            while current_time <= last_visible_time {
                let y_offset = self.get_y_offset(current_time);

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
        }

        // paint event markers + text
        {
            let event_marker_x_offset = 75.0;

            let mut prev_y_offset = self.get_y_offset(self.first_hour());

            // draw all markers
            for (marker_time, marker_data) in
                self.markers.range(first_visible_time..=last_visible_time)
            {
                let y_offset = self.get_y_offset(*marker_time);

                let xrange =
                    (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right();

                painter.hline(
                    xrange.clone(),
                    time_mark_region.top() + y_offset,
                    marker_data.stroke,
                );

                let galley = painter.layout_no_wrap(
                    marker_data.label.to_owned(),
                    egui::FontId::proportional(10.0),
                    marker_data.stroke.color,
                );

                if galley.rect.height() < (y_offset - prev_y_offset) {
                    painter.galley(
                        egui::Pos2 {
                            x: time_mark_region.left() + event_marker_x_offset,
                            y: time_mark_region.top() + y_offset - galley.rect.height(),
                        },
                        galley,
                    );
                }

                prev_y_offset = y_offset;
            }
            // paint current_time marker
            painter.hline(
                (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right(),
                time_mark_region.top() + self.get_y_offset(*self.selected_time),
                egui::Stroke::new(1.0, egui::Color32::RED),
            );
        }

        // if we double clicked a mouse on the calendar, set to that time
        if response.clicked() {
            if let Some(p) = response.interact_pointer_pos() {
                // get y  offset wrt time_mark_region
                let y_offset = p.y - time_mark_region.top();
                let time = self.get_time(y_offset);
                // we're fine with accepting anything within 10 pixels either direction
                let permissible_error = self.pixels_to_hours(10.0);

                // get all times within a range of the true value, and then sort them to see which one is closest
                if let Some((selected_time, _)) = self
                    .markers
                    .range((time - permissible_error)..=(time + permissible_error))
                    .min_by_key(|(k, _)| i64::abs(time.timestamp_nanos() - k.timestamp_nanos()))
                {
                    *self.selected_time = *selected_time;
                    response.mark_changed();
                }
            }
        }

        return response;
    }
}

impl<'a, 'b> egui::Widget for TimelineWidget<'a, 'b> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut scroll_area = egui::ScrollArea::vertical();

        if self.scroll_to_selected {
            let visible_section_duration = self.pixels_to_hours(ui.available_height());

            // check if content will overflow
            if self.last_hour() - self.first_hour() > visible_section_duration {
                let offset = (self.get_y_offset(*self.selected_time) - ui.available_height() / 2.0)
                    .clamp(
                        // no need to scroll if negative
                        0.0,
                        // if the scroll would scroll past content, dont
                        self.get_y_offset(self.last_hour() - visible_section_duration),
                    );

                scroll_area = scroll_area.vertical_scroll_offset(offset);
            }
        }

        return scroll_area
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, rect| self.draw_in_viewport(ui, rect))
            .inner;
    }
}
