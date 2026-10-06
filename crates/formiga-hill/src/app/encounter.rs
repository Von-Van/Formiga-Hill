//! The clearing in the window: following the glint, playing the Sovereign through, and bringing
//! its arrow home.

use super::{Area, HillApp, paper};
use crate::audio::Cue;
use crate::cast::Id;
use crate::clearing::{
    self, boss,
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

    /// The menu of attacks, or the way back out; and reading on while someone speaks.
    pub(super) fn clearing_tray(&mut self, ui: &mut egui::Ui, now: f32) {
        let Some((_, sovereign)) = &mut self.clearing else {
            return;
        };
        let mut chosen = None;
        let mut leave = false;
        if sovereign.stage.line.is_some() {
            let next = ui.add(paper::Button::new("Next \u{25b8}"));
            if paper::explain(ui, next, "Or click the scene, or press Space").clicked() {
                sovereign.stage.read_on();
            }
        } else if sovereign.state() == State::Choosing {
            paper::hint(ui, "What will you do?");
            for (index, choice) in sovereign.choices().iter().enumerate() {
                if paper::button(ui, format!("{}  {}", index + 1, choice.name)).clicked() {
                    chosen = Some(index);
                }
            }
        } else if sovereign.state() == State::Over {
            leave = paper::button(ui, "Back to the Woods").clicked();
        }
        if let Some(index) = chosen {
            sovereign.choose(index);
        }
        if leave {
            self.clearing = None;
            self.go_to(Area::Woods, now);
        }
    }

    /// The line being said, in a box over whoever says it: the Sovereign's from the dark, over
    /// its pointer; the party's over their heads; and the telling in a caption at the top.
    pub(super) fn clearing_speech(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        scene: egui::Rect,
        now: f32,
    ) {
        let cast = &self.arrival.cast;
        let Some((ground, sovereign)) = &mut self.clearing else {
            return;
        };
        let Some((speaker, text)) = &sovereign.stage.line else {
            return;
        };
        let point = scene.width() / super::SCENE_WIDTH as f32;
        let to_screen = |(x, y): (f32, f32)| scene.min + egui::vec2(x, y) * point;
        let (boss_move, since) = sovereign.stage.boss;
        let (head, name, voice) = match speaker {
            Speaker::Boss => (
                Some(to_screen(boss::tip(boss_move, now - since))),
                "The Cursor Sovereign".to_owned(),
                paper::Voice::Night,
            ),
            Speaker::Member(id) => (
                ground.head(*id, now).map(|(x, y)| to_screen((x, y - 2.0))),
                cast.member(*id)
                    .map_or(String::new(), |member| member.name.clone()),
                paper::Voice::Speaker,
            ),
            Speaker::Narrator => (None, String::new(), paper::Voice::Narrator),
        };
        paper::speech(ctx, painter, scene, head, Some(&name), text, voice, true);
    }
}
