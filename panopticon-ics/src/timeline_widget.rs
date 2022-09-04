use chrono::{DateTime, Duration, DurationRound, Local};
use eframe::egui;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

const BASE_PIXELS_PER_HOUR: f32 = 50.0;

pub struct TimelineWidget<T> {
    zoom_multipler: u32,
    selected_time: DateTime<Local>,
    data_points: BTreeMap<DateTime<Local>, T>,
}

impl<T> TimelineWidget<T> {
    pub fn new(
        zoom_multipler: u32,
        selected_time: DateTime<Local>,
        data_points: BTreeMap<DateTime<Local>, T>,
    ) -> Self {
        TimelineWidget {
            zoom_multipler,
            data_points,
            selected_time,
        }
    }

    pub fn data_points_mut(&mut self) -> &mut BTreeMap<DateTime<Local>, T> {
        &mut self.data_points
    }

    // returns the entry of the adjacent data point (if exists)
    pub fn adjacent_data_point_mut<'a>(&'a mut self) -> Option<(&'a DateTime<Local>, &'a mut T)> {
        self.data_points.range_mut(self.selected_time..).next()
    }

    // returns the entry of the next data point (if exists)
    pub fn next_data_point<'a>(&'a self) -> Option<(&'a DateTime<Local>, &'a T)> {
        self.data_points
            .range((Excluded(self.selected_time), Unbounded))
            .next()
    }

    // returns the entry of the previous data point (if exists)
    pub fn previous_data_point<'a>(&'a self) -> Option<(&'a DateTime<Local>, &'a T)> {
        self.data_points
            .range((Unbounded, Excluded(self.selected_time)))
            .next_back()
    }

    // returns a mutable reference to the zoom multiplier
    pub fn zoom_multipler_mut(&mut self) -> &mut u32 {
        &mut self.zoom_multipler
    }

    // returns a mutable reference to the
    pub fn selected_time_mut(&mut self) -> &mut DateTime<Local> {
        &mut self.selected_time
    }

    // computes the pixel y_offset in this component for a given time
    fn get_y_offset(&self, first_hour: DateTime<Local>, time: DateTime<Local>) -> f32 {
        let hours = (time - first_hour).num_seconds() as f32 / 60.0 / 60.0;
        return hours * BASE_PIXELS_PER_HOUR * self.zoom_multipler as f32;
    }

    // computes the time for a given in this component for a given pixel y_offset
    fn get_time(&self, first_hour: DateTime<Local>, y_offset: f32) -> DateTime<Local> {
        let hours_offset = y_offset / (BASE_PIXELS_PER_HOUR * self.zoom_multipler as f32);
        return first_hour + Duration::seconds((hours_offset * 60.0 * 60.0) as i64);
    }

    fn draw_in_viewport(&mut self, ui: &mut egui::Ui, visible_rect: egui::Rect) -> egui::Response {
        // calculate first hour to display
        let first_hour = self
            .data_points
            .first_key_value()
            .map(|x| x.0.clone())
            .unwrap_or(self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap();

        let last_hour = self
            .data_points
            .last_key_value()
            .map(|x| x.0.clone())
            .unwrap_or(self.selected_time)
            .duration_trunc(Duration::hours(1))
            .unwrap()
            + Duration::hours(1);

        let (response, painter) = ui.allocate_painter(
            egui::Vec2 {
                x: 200.0,
                y: self.zoom_multipler as f32
                    * (last_hour - first_hour).num_hours() as f32
                    * BASE_PIXELS_PER_HOUR,
            },
            egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );

        let time_mark_region = response.rect;

        let widget_visuals =  ui.style().interact(&response);

        // if has focus then paint the background a lighter shade of gray
        painter.rect_filled(
            time_mark_region,
            widget_visuals.rounding,
            widget_visuals.bg_fill,
        );
        painter.rect_stroke(
            time_mark_region,
            widget_visuals.rounding,
            widget_visuals.bg_stroke,
        );

        // decide on time increment based on zoom level
        let time_increment = match self.zoom_multipler {
            1..=4 => Duration::hours(1),
            5..=9 => Duration::minutes(15),
            10..=49 => Duration::minutes(5),
            50..=100 => Duration::minutes(1),
            _ => Duration::seconds(1),
        };

        let first_visible_time = self.get_time(first_hour, visible_rect.top());
        let last_visible_time = self.get_time(first_hour, visible_rect.bottom());

        // paint hlines and labels
        {
            // get first visible hline
            let mut current_time = first_visible_time.duration_trunc(time_increment).unwrap();

            while current_time <= last_visible_time {
                let y_offset = self.get_y_offset(first_hour, current_time);

                painter.hline(
                    time_mark_region.left()..=time_mark_region.right(),
                    time_mark_region.top() + y_offset,
                    widget_visuals.fg_stroke,
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

        // paint event markers
        {
            let event_marker_x_offset = 75.0;

            // draw all snapshot times
            for snapshot_time in self
                .data_points
                .range(first_visible_time..=last_visible_time)
                .map(|x| x.0.clone())
            {
                let y_offset = self.get_y_offset(first_hour, snapshot_time);

                painter.hline(
                    (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right(),
                    time_mark_region.top() + y_offset,
                    widget_visuals.fg_stroke,
                );
            }
            // paint current_time marker
            painter.hline(
                (time_mark_region.left() + event_marker_x_offset)..=time_mark_region.right(),
                time_mark_region.top() + self.get_y_offset(first_hour, self.selected_time),
                egui::Stroke::new(widget_visuals.fg_stroke.width, egui::Color32::RED),
            );
        }

        if response.clicked() {
            response.request_focus();
        }

        // if we double clicked a mouse on the calendar, set to that time
        if response.double_clicked() {
            if let Some(p) = response.interact_pointer_pos() {
                // get y  offset wrt time_mark_region
                let y_offset = p.y - time_mark_region.top();
                let time = self.get_time(first_hour, y_offset);
                self.selected_time = time;
            }
        }

        // if the calendar is focused and we pressed the up or down keys
        if response.has_focus() {
            println!("focus!");
            if ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
            {
                if let Some(previous) = self.previous_data_point().map(|x| x.0.clone()) {
                    self.selected_time = previous;
                }
                let y_off =
                    time_mark_region.top() + self.get_y_offset(first_hour, self.selected_time);
                let selected_time_rect =
                    egui::Rect::from_x_y_ranges(time_mark_region.x_range(), y_off..=y_off);
                ui.scroll_to_rect(selected_time_rect, Some(egui::Align::Center));
            }

            if ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            {
                if let Some(next) = self.next_data_point().map(|x| x.0.clone()) {
                    self.selected_time = next;
                }
                let y_off =
                    time_mark_region.top() + self.get_y_offset(first_hour, self.selected_time);
                let selected_time_rect =
                    egui::Rect::from_x_y_ranges(time_mark_region.x_range(), y_off..=y_off);
                ui.scroll_to_rect(selected_time_rect, Some(egui::Align::Center));
            }
        }

        return response;
    }
}

impl<T> egui::Widget for &mut TimelineWidget<T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        return egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, rect| self.draw_in_viewport(ui, rect))
            .inner;
    }
}
