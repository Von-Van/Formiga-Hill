//! Grooming and dressing up, in the window: brushing whoever the pointer is held on, the Green's
//! dress-up box, and carrying what everyone has on, and who shines, from place to place for the
//! rest of the visit. None of it goes home.

use super::{Area, HillApp};
use crate::cast::Id;
use crate::character::{Brushing, Offer};
use crate::costume;
use crate::playground::{Playground, distance};
use eframe::egui;

/// What the person has picked out of the dress-up box to put on someone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    Piece(&'static str),
    TakeOff,
}

impl HillApp {
    /// The ground on show, if it is somewhere companions can be groomed or dressed right now:
    /// not in the middle of a story or a game.
    fn ground_for_finery(&mut self) -> Option<&mut Playground> {
        match self.area {
            Area::Green => self.green.as_mut(),
            Area::Clubhouse if self.story.is_none() => {
                self.clubhouse.as_mut().map(|room| room.ground())
            }
            Area::Hilltop => self.hilltop.as_mut(),
            Area::Fairground => self
                .fairground
                .as_mut()
                .filter(|(_, game)| game.phase() == crate::fairground::Phase::Ready)
                .map(|(ground, _)| ground),
            _ => None,
        }
    }

    /// With the brush held out and the pointer held down on someone, each bit of movement across
    /// them is a stroke. Grooming done, they shine for the rest of the visit.
    pub(super) fn groom(&mut self, held: bool, now: f32) {
        let touching = (self.tool == Offer::Brush && held)
            .then_some(())
            .and(self.hovered)
            .zip(self.pointer);
        let Some((id, at)) = touching else {
            self.stroke = None;
            return;
        };
        let stroke = match self.stroke {
            Some((who, last)) if who == id => distance(last, at),
            _ => 0.0,
        };
        self.stroke = Some((id, at));
        let mut trust = std::mem::take(&mut self.trust);
        let reached = self
            .ground_for_finery()
            .and_then(|ground| ground.brush(id, stroke, &mut trust, now));
        self.trust = trust;
        if reached == Some(Brushing::Done) {
            self.groomed.insert(id);
            if let Some(member) = self.arrival.cast.member(id) {
                self.notice = Some((format!("{} is groomed and shining.", member.name), now));
            }
        }
    }

    /// Puts what was picked out of the box on someone on the Green, or takes theirs off. They
    /// take to it in their own way.
    pub(super) fn dress(&mut self, id: Id, now: f32) {
        let Some(pick) = self.picked else {
            return;
        };
        let name = self
            .arrival
            .cast
            .member(id)
            .map_or("Someone".to_owned(), |member| member.name.clone());
        let line = match pick {
            Pick::Piece(piece) => {
                self.costumes.insert(id, piece);
                let called = costume::piece(piece).map_or("something", |piece| piece.name);
                format!("{name} is wearing {}.", lower_first(called))
            }
            Pick::TakeOff => {
                if self.costumes.remove(&id).is_none() {
                    return;
                }
                format!("{name}'s costume goes back in the box.")
            }
        };
        if let (Some(green), Some(character)) = (
            self.green.as_mut(),
            self.arrival
                .cast
                .member(id)
                .map(crate::character::Character::of),
        ) {
            let beats = match pick {
                Pick::Piece(_) => costume::dressed(&character),
                Pick::TakeOff => Vec::new(),
            };
            green.direct(
                id,
                beats.into_iter().map(crate::actor::Step::Beat).collect(),
                now,
            );
        }
        self.dress_everyone();
        self.notice = Some((line, now));
    }

    /// Everyone wears what they have on, and shines if groomed, wherever they are.
    pub(super) fn dress_everyone(&mut self) {
        let (costumes, groomed) = (&self.costumes, &self.groomed);
        let grounds = [
            self.green.as_mut(),
            self.clubhouse.as_mut().map(|room| room.ground()),
            self.fairground.as_mut().map(|(ground, _)| ground),
            self.hilltop.as_mut(),
            self.woods.empty.as_mut(),
            self.woods.pool.as_mut(),
            self.woods.meadow.as_mut(),
            self.woods.outing.as_mut().map(|(ground, _)| ground),
            self.woods.fishing.as_mut().map(|(ground, _)| ground),
            self.woods.hunt.as_mut().map(|(ground, _)| ground),
        ];
        for ground in grounds.into_iter().flatten() {
            ground.dress(costumes, groomed);
        }
    }

    /// The dress-up box, open: every piece to pick, and a way to take one off.
    pub(super) fn dress_up_window(&mut self, ctx: &egui::Context) {
        if !self.dress_up || self.area != Area::Green {
            return;
        }
        if self.costume_icons.is_empty() {
            self.costume_icons = costume::PIECES
                .iter()
                .map(|piece| {
                    let icon = costume::art::icon(piece.id);
                    let scaled = scaled(&icon, 3);
                    ctx.load_texture(
                        format!("costume-{}", piece.id),
                        egui::ColorImage::from_rgba_unmultiplied(
                            [scaled.width() as usize, scaled.height() as usize],
                            &scaled.rgba_bytes(),
                        ),
                        egui::TextureOptions::NEAREST,
                    )
                })
                .collect();
        }
        let mut open = true;
        egui::Window::new("The dress-up box")
            .open(&mut open)
            .default_width(260.0)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Pick something, then click someone to put it on. Just for the visit.",
                    )
                    .italics(),
                );
                ui.add_space(4.0);
                egui::Grid::new("pieces").num_columns(2).show(ui, |ui| {
                    for (index, piece) in costume::PIECES.iter().enumerate() {
                        let chosen = self.picked == Some(Pick::Piece(piece.id));
                        let image = egui::Image::new(&self.costume_icons[index])
                            .fit_to_exact_size(egui::vec2(27.0, 27.0));
                        let button =
                            egui::Button::image_and_text(image, piece.name).selected(chosen);
                        if ui.add(button).clicked() {
                            self.picked = (!chosen).then_some(Pick::Piece(piece.id));
                        }
                        if index % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
                ui.separator();
                let taking_off = self.picked == Some(Pick::TakeOff);
                if ui
                    .selectable_label(taking_off, "Take a costume off")
                    .clicked()
                {
                    self.picked = (!taking_off).then_some(Pick::TakeOff);
                }
            });
        if !open {
            self.dress_up = false;
            self.picked = None;
        }
    }
}

/// A small picture made bigger by whole pixels, for showing in the window.
fn scaled(picture: &formiga_art::Canvas, by: u32) -> formiga_art::Canvas {
    let mut out = formiga_art::Canvas::new(picture.width() * by, picture.height() * by);
    for y in 0..out.height() as i32 {
        for x in 0..out.width() as i32 {
            out.set(x, y, picture.get(x / by as i32, y / by as i32));
        }
    }
    out
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(chars).collect()
    })
}
