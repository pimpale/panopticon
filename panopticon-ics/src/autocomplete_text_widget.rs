use eframe::egui;
use std::{collections::BTreeSet, ops::Range};

pub struct AutocompleteTextWidget<'a> {
    candidates: Vec<&'a str>,
    input: String,
    ac_state: AcState,
    cursor_to_end: bool,
    accepted_string: String,
}

impl<'a> AutocompleteTextWidget<'a> {
    pub fn new<I>(candidates: I, input: String) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        AutocompleteTextWidget
    }
}

impl<'a> egui::Widget for AutocompleteTextWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let te_id = ui.make_persistent_id("text_edit_ac");
        let up_pressed = ui
            .input_mut()
            .consume_key(egui::Modifiers::default(), egui::Key::ArrowUp);
        let down_pressed = ui
            .input_mut()
            .consume_key(egui::Modifiers::default(), egui::Key::ArrowDown);
        let end_pos = self.input.chars().count();
        let te = egui::TextEdit::singleline(&mut self.input)
            .lock_focus(true)
            .id(te_id);
        if self.cursor_to_end {
            set_cursor_pos(ui, te_id, end_pos);
        }
        let re = ui.add(te);

        self.ac_state.input_changed = re.changed();

        let msg = autocomplete_popup_below(
            &mut self.input,
            &mut self.ac_state,
            self.candidates.into(),
            ui,
            &re,
            up_pressed,
            down_pressed,
        );
        if msg.applied {
            self.cursor_to_end = true;
        } else {
            self.cursor_to_end = false;
        }
        if ui.input().key_pressed(egui::Key::Enter) {
            self.accepted_string = self.input.clone();
            re.request_focus();
        }
        if msg.stole_focus {
            re.request_focus();
        }
        return re;
    }
}

fn set_cursor_pos(ui: &mut egui::Ui, te_id: egui::Id, char_pos: usize) {
    let mut state = egui::TextEdit::load_state(ui.ctx(), te_id).unwrap();
    let ccursor = egui::text::CCursor::new(char_pos);
    state.set_ccursor_range(Some(egui::text::CCursorRange::one(ccursor)));
    egui::TextEdit::store_state(ui.ctx(), te_id, state);
}

pub struct AcState {
    /// Selection index in the autocomplete list
    select: Option<usize>,
    /// Input changed this frame
    pub input_changed: bool,
}

impl Default for AcState {
    fn default() -> Self {
        Self {
            select: Some(0),
            input_changed: true,
        }
    }
}

#[derive(Default)]
pub struct PopupMsg {
    /// Returns whether a suggestion was applied or not
    pub applied: bool,
    /// Whether the popup stole focus (for example on pressing enter)
    pub stole_focus: bool,
}

/// Popup for autocompleting.
fn autocomplete_popup_below(
    string: &mut String,
    state: &mut AcState,
    candidates: &[&str],
    ui: &mut egui::Ui,
    response: &egui::Response,
    up_pressed: bool,
    down_pressed: bool,
) -> PopupMsg {
    let mut ret_msg = PopupMsg::default();
    let popup_id = ui.make_persistent_id("autocomplete_popup");
    let last_char_is_terminating = string.chars().last().map_or(true, |c| !c.is_alphabetic());
    let last = if last_char_is_terminating {
        ""
    } else {
        string.split_ascii_whitespace().last().unwrap_or("")
    };
    if down_pressed {
        match &mut state.select {
            None => state.select = Some(0),
            Some(sel) => *sel += 1,
        }
    }
    if let Some(sel) = &mut state.select {
        if up_pressed {
            if *sel > 0 {
                *sel -= 1;
            } else {
                // Allow selecting "Nothing" by going above first element
                state.select = None;
            }
        }
    } else if state.input_changed {
        // Always select index 0 when input was changed for convenience
        state.select = Some(0);
    }
    if !string.is_empty() && !last.is_empty() {
        let mut exact_match = None;
        // Get length of list and also whether there is an exact match
        let mut i = 0;
        let len = candidates
            .iter()
            .filter(|candidate| {
                if **candidate == last {
                    exact_match = Some(i);
                }
                let predicate = candidate.contains(last);
                if predicate {
                    i += 1;
                }
                predicate
            })
            .count();
        match exact_match {
            Some(idx) if state.input_changed => state.select = Some(idx),
            _ => {}
        }
        if len > 0 {
            if let Some(selection) = &mut state.select {
                if *selection >= len {
                    *selection = len - 1;
                }
            }
            let mut complete = None;
            egui::popup_below_widget(ui, popup_id, response, |ui| {
                for (i, &candidate) in candidates
                    .iter()
                    .filter(|candidate| candidate.contains(last))
                    .enumerate()
                {
                    if ui
                        .selectable_label(state.select == Some(i), candidate)
                        .clicked()
                    {
                        complete = Some(candidate);
                    }
                    let return_pressed = ui.input().key_pressed(egui::Key::Enter);
                    if state.select == Some(i)
                        && (ui.input().key_pressed(egui::Key::Tab) || return_pressed)
                    {
                        complete = Some(candidate);
                        if return_pressed {
                            ret_msg.stole_focus = true;
                        }
                    }
                }
            });
            if let Some(candidate) = complete {
                let range = str_range(string, last);
                string.replace_range(range, candidate);
                state.input_changed = false;
                ret_msg.applied = true;
            }
            if !string.is_empty() {
                ui.memory().open_popup(popup_id);
            } else {
                ui.memory().close_popup();
            }
        }
    }
    ret_msg
}

fn str_range(parent: &str, sub: &str) -> Range<usize> {
    let beg = sub.as_ptr() as usize - parent.as_ptr() as usize;
    let end = beg + sub.len();
    beg..end
}
