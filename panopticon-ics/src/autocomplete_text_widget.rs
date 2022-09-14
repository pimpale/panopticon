use eframe::egui;
use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

/// this struct allows us to uniquely namespace the sugestion popup
#[derive(Clone)]
struct AutocompleteTextWidgetMemory {
    opts: Vec<String>,
    selected_idx: usize,
    first_frame: bool,
}

pub struct AutocompleteTextWidget<'t> {
    candidate_generator: Box<dyn FnOnce(&dyn egui::TextBuffer) -> Vec<String>>,
    text: &'t mut String,
    
}

impl<'t> AutocompleteTextWidget<'t> {
    pub fn new(
        text: &'t mut String,
        candidate_generator: impl FnOnce(&dyn egui::TextBuffer) -> Vec<String> + 'static,
    ) -> Self {
        AutocompleteTextWidget {
            candidate_generator: Box::new(candidate_generator),
            text,
        }
    }
}

impl<'t> egui::Widget for AutocompleteTextWidget<'t> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // wrapping utils
        let wrap = |a: isize, lim: usize| isize::rem_euclid(a, lim as isize) as usize;
        let wrapping_add = |a: usize, off: isize, lim: usize| wrap(a as isize + off, lim);

        let te = egui::TextEdit::singleline(self.text).lock_focus(true);
        let te_re = ui.add(te);

        if te_re.gained_focus() {
            println!("GAINED FOCUS");
        }
        if te_re.lost_focus() {
            println!("LOST FOCUS");
        }

        let popup_id = te_re
            .id
            .with(TypeId::of::<AutocompleteTextWidgetMemory>())
            .with("suggestion_popup");

        // compute memory
        if te_re.changed() || te_re.gained_focus() {
            let opts = (self.candidate_generator)(self.text);
            if opts.len() > 0 {
                ui.memory().data.insert_temp(
                    popup_id,
                    Arc::new(Mutex::new(AutocompleteTextWidgetMemory {
                        opts,
                        selected_idx: 0,
                        first_frame: true,
                    })),
                );
                ui.memory().open_popup(popup_id);
            } else {
                ui.memory()
                    .data
                    .remove::<Arc<Mutex<AutocompleteTextWidgetMemory>>>(popup_id);
            }
        }

        // will only display if popup is showing (hidden state in memory)
        egui::popup_below_widget(ui, popup_id, &te_re, |ui| {
            let data_mutex = ui
                .memory()
                .data
                .get_temp::<Arc<Mutex<AutocompleteTextWidgetMemory>>>(popup_id)
                // we maintain an invariant that if popup is showing, the memory must be available
                .unwrap();
            let mut data = data_mutex.lock().unwrap();

            // enter or tab selects the current option
            // up selects previous option
            // down selects next option

            let arrow_up = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp);
            let arrow_down = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown);
            let tab = !data.first_frame
                && ui
                    .input_mut()
                    .consume_key(egui::Modifiers::NONE, egui::Key::Tab);

            let mut clicked_idx = None;

            for (i, candidate) in data.opts.iter().enumerate() {
                let resp_label = ui.selectable_label(i == data.selected_idx, candidate);
                if resp_label.clicked() {
                    clicked_idx = Some(i);
                }
            }

            if tab || clicked_idx.is_some() {
                let commiting_idx = clicked_idx.unwrap_or(data.selected_idx);
                let candidate = &data.opts[commiting_idx];
                // push completion
                self.text.push_str(candidate);
                // move cursor to end
                set_cursor_pos(ui, te_re.id, self.text.chars().count());
                // close popup
                ui.memory().toggle_popup(popup_id);
                // request focus goes back to the thing
                te_re.request_focus();
            } else {
                // arrows wrap
                if arrow_down {
                    data.selected_idx = wrapping_add(data.selected_idx, 1, data.opts.len())
                }
                if arrow_up {
                    data.selected_idx = wrapping_add(data.selected_idx, -1, data.opts.len())
                }
            }
            data.first_frame = false;
        });

        return te_re;
    }
}

fn set_cursor_pos(ui: &mut egui::Ui, te_id: egui::Id, char_pos: usize) {
    let mut state = egui::TextEdit::load_state(ui.ctx(), te_id).unwrap();
    let ccursor = egui::text::CCursor::new(char_pos);
    state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
    egui::TextEdit::store_state(ui.ctx(), te_id, state);
}

fn str_range(parent: &str, sub: &str) -> std::ops::Range<usize> {
    let beg = sub.as_ptr() as usize - parent.as_ptr() as usize;
    let end = beg + sub.len();
    beg..end
}
