//! Sound in the window: the piece for wherever the person is, and the Sound window, with its
//! levels and mute.

use super::{Area, HillApp};
use crate::audio::{Cue, Music, Track};
use crate::clearing::sovereign::State;
use eframe::egui;

/// What has been heard so far, so each thing is heard once.
#[derive(Debug, Default)]
pub struct Listening {
    /// Whether the Sound window is open.
    window: bool,
}

impl HillApp {
    /// The piece for wherever the person is: each place's own by day, and its night piece once
    /// the lamps are lit. Quiet while the train comes and goes, its own sounds being enough, and
    /// once the clearing's business is over.
    pub(super) fn wanted_music(&self) -> Option<Music> {
        let track = match self.area {
            Area::Station if !self.station.is_settled() => return None,
            Area::Station | Area::Green => Track::Pastoral,
            Area::Clubhouse => Track::Fireside,
            Area::Fairground => Track::Waltz,
            Area::Woods => Track::Woods,
            Area::Hilltop => Track::Hilltop,
            Area::Clearing => match &self.clearing {
                Some((_, sovereign)) if sovereign.state() != State::Over => Track::Secret,
                _ => return None,
            },
        };
        Some(Music::new(track, self.daylight.lamps() >= 0.5))
    }

    /// Keeps the music to wherever the person is.
    pub(super) fn listen(&mut self, _now: f32) {
        self.sound.music(self.wanted_music());
    }

    /// Mutes everything, or brings it back, and says which.
    pub(super) fn toggle_mute(&mut self, now: f32) {
        let muted = self.sound.toggle_mute();
        let line = if muted {
            "Sound off. Press M to bring it back."
        } else {
            self.sound.play(Cue::Click);
            "Sound on."
        };
        self.notice = Some((line.to_owned(), now));
    }

    /// The speaker in every bar, for the Sound window.
    pub(super) fn sound_button(&mut self, ui: &mut egui::Ui) {
        let muted = self.sound.levels().muted;
        let (icon, hint) = if muted {
            ("\u{1f507}", "Sound\u{2026} Muted: M brings it back")
        } else {
            ("\u{1f50a}", "Sound\u{2026} M mutes it")
        };
        if ui
            .selectable_label(self.listening.window, icon)
            .on_hover_text(hint)
            .clicked()
        {
            self.listening.window = !self.listening.window;
        }
    }

    /// The Sound window: how loud everything is, the music and the sounds, and mute. Changes are
    /// heard at once, and kept once a slider is let go.
    pub(super) fn sound_window(&mut self, ctx: &egui::Context) {
        if !self.listening.window {
            return;
        }
        let before = self.sound.levels();
        let mut levels = before;
        let mut open = true;
        let mut dragging = false;
        // Whether the master or the sounds' level was let go, to be heard at its new level.
        let mut set = false;
        egui::Window::new("Sound")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                egui::Grid::new("levels")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        // The music is heard at its new level as it is moved; the others
                        // with a click once they are let go.
                        for (label, level, clicks) in [
                            ("Master", &mut levels.master, true),
                            ("Music", &mut levels.music, false),
                            ("Sounds", &mut levels.sounds, true),
                        ] {
                            ui.label(label);
                            let mut percent = (*level * 100.0).round() as u32;
                            let slider = egui::Slider::new(&mut percent, 0..=100).suffix("%");
                            let response = ui.add_enabled(!levels.muted, slider);
                            if response.changed() {
                                *level = percent as f32 / 100.0;
                            }
                            dragging |= response.dragged();
                            let let_go = response.drag_stopped()
                                || response.changed() && !response.dragged();
                            set |= clicks && let_go;
                            ui.end_row();
                        }
                    });
                ui.add_space(4.0);
                ui.checkbox(&mut levels.muted, "Mute  M");
                if self.sound.silent() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "There is nothing here to play sound on, so the Hill is quiet.",
                        )
                        .italics()
                        .small(),
                    );
                }
            });
        if levels != before {
            self.sound.set_levels(levels);
        }
        // A quiet click as a level is let go, to hear it at; and as the sound comes back on.
        if set || (before.muted && !levels.muted) {
            self.sound.play(Cue::Click);
        }
        if !dragging {
            self.sound.keep();
        }
        self.listening.window = open;
    }
}
