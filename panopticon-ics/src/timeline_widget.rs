use chrono::{DateTime, Local};
use eframe::egui::{Response, Ui, Widget};

pub struct TimelineWidget {
    zoom_multipler: f32,
    now: DateTime<Local>,
    times: Vec<DateTime<Local>>,
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

impl Widget for TimelineWidget {
    fn ui(self, ui: &mut Ui) -> Response {
    }
}
