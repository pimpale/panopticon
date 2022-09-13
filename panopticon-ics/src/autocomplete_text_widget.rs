use eframe::egui;
use std::{any::TypeId, ops::Range};

/// this struct allows us to uniquely namespace the sugestion popup
struct AutocompleteTextWidgetNamespace {}

pub struct AutocompleteTextWidget<'t, F, K> {
    candidate_generator: F,
    selected: Option<K>,
    text: &'t mut String,
}

impl<'t, F, K> AutocompleteTextWidget<'t, F, K> {
    pub fn new(text: &'t mut String, candidate_generator: F, selected: Option<K>) -> Self {
        AutocompleteTextWidget {
            candidate_generator,
            text,
            selected,
        }
    }
}

impl<'t, F, K> egui::Widget for AutocompleteTextWidget<'t, F, K>
where
    F: FnOnce(&dyn egui::TextBuffer) -> Vec<(K, String)>,
    // identity of an option
    K: PartialEq + Clone,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let te = egui::TextEdit::singleline(self.text).lock_focus(true);
        let te_re = ui.add(te);

        let popup_id = te_re
            .id
            .with(TypeId::of::<AutocompleteTextWidgetNamespace>())
            .with("suggestion_popup");

        let opts = (self.candidate_generator)(self.text);

        egui::popup_below_widget(ui, popup_id, &te_re, |ui| {
            // enter or tab selects the current option
            // up selects previous option
            // down selects next option

            let selected = self.selected.or(ui.memory().data.get_temp();

            let arrow_up = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp);
            let arrow_down = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown);
            let enter = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            let tab = ui
                .input_mut()
                .consume_key(egui::Modifiers::NONE, egui::Key::Tab);

            let x: Vec<_> = opts.into_iter().collect();

            for (i, (key, candidate)) in x.iter().enumerate() {
                let resp_label =
                    ui.selectable_label(self.selected.as_ref() == Some(key), candidate);
                if resp_label.clicked() || enter || tab {
                    // push completion
                    self.text.push_str(&candidate);
                    // close popup
                    ui.memory().close_popup();
                    // move cursor to end
                    set_cursor_pos(ui, te_re.id, self.text.chars().count());
                } else if arrow_up {
                    self.selected = Some(x[i.saturating_sub(1)].0);
                } else if arrow_down {
                    self.selected = Some(x[usize::min(x.len() - 1, i)].0);
                }
            }
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
