//! Undo and redo, for the story builder and the map editor: egui's own `Undoer` keeps a draft as
//! it was each time it settled, so a word typed or a place dragged is one step back, not one
//! for every letter or every pixel.

use super::paper;
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, util::undoer::Undoer};

/// Command-Z steps back, and Command-Shift-Z forward again: Ctrl on Windows.
const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);

/// What has been done to one draft, to step back through.
pub(super) struct History<T: Clone + PartialEq> {
    /// Which draft it is of, so opening another starts afresh.
    of: Option<String>,
    undoer: Undoer<T>,
}

impl<T: Clone + PartialEq> Default for History<T> {
    fn default() -> Self {
        Self {
            of: None,
            undoer: Undoer::default(),
        }
    }
}

impl<T: Clone + PartialEq> History<T> {
    /// Notes `draft`, called `name`, as it is now: once a frame while it is open.
    pub(super) fn feed(&mut self, ctx: &egui::Context, name: &str, draft: &T) {
        if self.of.as_deref() != Some(name) {
            self.of = Some(name.to_owned());
            self.undoer = Undoer::default();
        }
        self.undoer.feed_state(ctx.input(|input| input.time), draft);
    }

    /// The Undo and Redo buttons, and their keys while nothing is being typed: the draft as it
    /// was, or as it will be again, if either was asked for.
    pub(super) fn buttons(&mut self, ui: &mut egui::Ui, draft: &T) -> Option<T> {
        let typing = ui.ctx().egui_wants_keyboard_input();
        let (mut back, mut forward) = (false, false);
        if !typing {
            // Redo first: Command-Shift-Z would also match Command-Z, which ignores Shift.
            forward = ui.input_mut(|input| input.consume_shortcut(&REDO));
            back = !forward && ui.input_mut(|input| input.consume_shortcut(&UNDO));
        }
        let format = |shortcut: &KeyboardShortcut| ui.ctx().format_shortcut(shortcut);
        let (undo_keys, redo_keys) = (format(&UNDO), format(&REDO));
        let undo = ui.add(paper::Button::new("Undo").enabled(self.undoer.has_undo(draft)));
        paper::name_it(ui, &undo, &format!("Undo  {undo_keys}"));
        back |= undo.clicked();
        let redo = ui.add(paper::Button::new("Redo").enabled(self.undoer.has_redo(draft)));
        paper::name_it(ui, &redo, &format!("Redo  {redo_keys}"));
        forward |= redo.clicked();
        if back {
            self.undoer.undo(draft).cloned()
        } else if forward {
            self.undoer.redo(draft).cloned()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settled_change_is_one_step_back_and_forward_again() {
        let mut history: History<String> = History::default();
        let before = String::from("The Lost Kite");
        let after = String::from("The Lost Kite Again");
        // Fed as the window feeds it, and given time to settle after each change.
        for (time, text) in [(0.0, &before), (2.0, &after), (4.0, &after)] {
            history.undoer.feed_state(time, text);
        }
        assert!(history.undoer.has_undo(&after));
        let back = history.undoer.undo(&after).cloned().unwrap();
        assert_eq!(back, before);
        assert_eq!(history.undoer.redo(&back).cloned().unwrap(), after);
    }

    #[test]
    fn another_draft_starts_with_nothing_to_undo() {
        let ctx = egui::Context::default();
        let mut history = History::default();
        history.feed(&ctx, "kite", &1);
        history.feed(&ctx, "kite", &2);
        assert!(history.undoer.has_undo(&2));
        history.feed(&ctx, "pond", &7);
        assert!(!history.undoer.has_undo(&7));
    }
}
