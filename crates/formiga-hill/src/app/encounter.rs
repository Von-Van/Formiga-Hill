//! The clearing in the window: following the glint, playing the Sovereign through, and bringing
//! its arrow home.

use super::{Area, HillApp, INK, PAPER};
use crate::audio::Cue;
use crate::cast::Id;
use crate::clearing::{
    self,
    sovereign::{Event, Sovereign, State},
    stage::Speaker,
};
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Band;

impl HillApp {
    /// The party followed the glint: the basket so far comes home, and they are in the clearing.
    pub(super) fn follow_the_glint(&mut self, now: f32) {
        let Some((_, rummage)) = &self.woods.outing else {
            return;
        };
        let party: Vec<Id> = rummage.party().to_vec();
        self.finish_outing(now);
        let cast = &self.arrival.cast;
        let close_pair = match party.as_slice() {
            [a, b] => cast
                .bond(*a, *b)
                .is_some_and(|bond| bond.warmth >= Band::High),
            _ => false,
        };
        let mut ground = clearing::open(cast, &party, now);
        let sovereign = Sovereign::new(&mut ground, cast, &party, close_pair, now);
        self.clearing = Some((ground, sovereign));
        self.notice = None;
        self.go_to(Area::Clearing, now);
        self.sound.play(Cue::Sting);
    }

    pub(super) fn tick_clearing(&mut self, now: f32) {
        let cast = &self.arrival.cast;
        let Some((ground, sovereign)) = &mut self.clearing else {
            return;
        };
        ground.tick(cast, now);
        sovereign.tick(ground, cast, now);
        for event in sovereign.take_events() {
            match event {
                Event::Bested => {
                    self.sound.play(Cue::Bested);
                    let party = sovereign.party();
                    let first = self.memories.bested_the_sovereign(&party);
                    let line = if first {
                        "The Sovereign's arrow is in the satchel, for the Hilltop."
                    } else {
                        "Seen off again. Nobody will ever quite believe it."
                    };
                    self.notice = Some((line.to_owned(), now));
                }
            }
        }
    }

    pub(super) fn compose_clearing(&mut self, now: f32) -> Canvas {
        match &mut self.clearing {
            Some((ground, sovereign)) => clearing::compose(ground, sovereign, now),
            None => Canvas::new(super::SCENE_WIDTH, super::SCENE_HEIGHT),
        }
    }

    /// Reads on past the line showing.
    pub(super) fn clearing_read_on(&mut self) {
        if let Some((_, sovereign)) = &mut self.clearing {
            sovereign.stage.read_on();
        }
    }

    /// Picks an attack by its number on the menu.
    pub(super) fn clearing_choose(&mut self, index: usize) {
        if let Some((_, sovereign)) = &mut self.clearing {
            sovereign.choose(index);
        }
    }

    /// The line being said, or the menu of attacks, or the way back out.
    pub(super) fn clearing_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        self.sound_button(ui);
        let cast = &self.arrival.cast;
        let Some((_, sovereign)) = &mut self.clearing else {
            return;
        };
        let mut chosen = None;
        let mut leave = false;
        egui::Frame::new()
            .fill(
                if sovereign
                    .stage
                    .line
                    .as_ref()
                    .is_some_and(|(speaker, _)| *speaker == Speaker::Boss)
                {
                    egui::Color32::from_rgb(0x1a, 0x10, 0x26)
                } else {
                    PAPER
                },
            )
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some((speaker, text)) = &sovereign.stage.line {
                    let (name, color, size) = match speaker {
                        Speaker::Boss => (
                            "THE CURSOR SOVEREIGN".to_owned(),
                            egui::Color32::from_rgb(0xf5, 0xd2, 0x5e),
                            17.0,
                        ),
                        Speaker::Member(id) => (
                            cast.member(*id).map_or(String::new(), |m| m.name.clone()),
                            INK,
                            16.0,
                        ),
                        Speaker::Narrator => (String::new(), INK, 15.0),
                    };
                    if !name.is_empty() {
                        ui.label(egui::RichText::new(name).strong().color(color));
                    }
                    let mut words =
                        egui::RichText::new(text)
                            .size(size)
                            .color(if *speaker == Speaker::Boss {
                                egui::Color32::from_rgb(0xf6, 0xee, 0xd8)
                            } else {
                                INK
                            });
                    if *speaker == Speaker::Narrator {
                        words = words.italics();
                    }
                    ui.horizontal_wrapped(|ui| ui.label(words));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Next  \u{25b8}").clicked() {
                            sovereign.stage.read_on();
                        }
                    });
                } else if sovereign.state() == State::Choosing {
                    ui.label(egui::RichText::new("What will you do?").strong().color(INK));
                    ui.horizontal_wrapped(|ui| {
                        for (index, choice) in sovereign.choices().iter().enumerate() {
                            if ui
                                .button(format!("{}  {}", index + 1, choice.name))
                                .clicked()
                            {
                                chosen = Some(index);
                            }
                        }
                    });
                } else if sovereign.state() == State::Over {
                    if let Some((notice, _)) = &self.notice {
                        ui.label(egui::RichText::new(notice).italics().color(INK));
                    }
                    leave = ui.button("Back to the Woods").clicked();
                } else {
                    ui.label(egui::RichText::new("\u{2026}").color(INK));
                }
            });
        if let Some(index) = chosen {
            sovereign.choose(index);
        }
        if leave {
            self.clearing = None;
            self.go_to(Area::Woods, now);
        }
    }
}
