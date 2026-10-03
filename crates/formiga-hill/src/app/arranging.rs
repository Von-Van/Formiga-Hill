//! The Hilltop in the window: placing finds from the satchel on its spots, planting what grows,
//! moving them about, and the journal of everything the Woods has turned up.

use super::HillApp;
use super::departures::grown_since;
use crate::finds::{self, CATALOGUE, Kind, growing};
use crate::hilltop::{self, SPOTS, Standing};
use crate::paint::{put, rgb, rgba};
use eframe::egui;
use formiga_art::Canvas;
use std::f32::consts::TAU;

/// What the person is about to stand somewhere on the Hilltop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placing {
    FromSatchel(String),
    /// Something growing, lifted into the satchel, to be planted again as it is.
    Lifted(Standing),
    FromSpot(u8),
}

/// How a find in the satchel is listed: by what it becomes, or as something to plant.
fn in_the_satchel(id: &str) -> String {
    match finds::find(id) {
        Some(find) if growing::growth(id).is_some() => format!("{}, for planting", find.name),
        Some(find) => find.piece.to_owned(),
        None => id.to_owned(),
    }
}

impl HillApp {
    /// Readies the summit the first time anyone goes there, with a word for anything that has
    /// grown since the last visit.
    pub(super) fn open_hilltop(&mut self, now: f32) {
        if self.hilltop.is_none() {
            let arrangement = &self.memories.colony().hilltop;
            self.hilltop = Some(hilltop::open(&self.arrival.cast, now, arrangement));
            self.refresh_hilltop();
            if let Some(line) = grown_since(&self.grown) {
                self.notice = Some((line, now));
            }
        }
    }

    /// Puts the summit's pieces where the memories say they stand.
    fn refresh_hilltop(&mut self) {
        let arrangement = self.memories.colony().hilltop.clone();
        let placed = hilltop::placed(&arrangement);
        self.piece_bounds = placed
            .iter()
            .map(|(spot, prop)| (*spot, prop.bounds()))
            .collect();
        let props = placed.into_iter().map(|(_, prop)| prop).collect();
        if let Some(ground) = &mut self.hilltop {
            ground.set_props(props);
            ground.set_attractions(hilltop::attractions(&arrangement));
            let lights = hilltop::nightlights(&arrangement, ground.backdrop());
            ground.set_nightlights(lights);
        }
        // The Hill seen from below changes too.
        self.station.show_hilltop(&arrangement);
        if let Some((ground, _)) = &mut self.fairground {
            crate::fairground::show_hilltop(ground, &arrangement);
        }
        if let Some(green) = &mut self.green {
            crate::green::show_hilltop(green, &arrangement);
        }
        if let Some(room) = &mut self.clubhouse {
            room.show_hilltop(&arrangement);
        }
        if let Some(meadow) = &mut self.woods.meadow {
            crate::meadow::show_hilltop(meadow, &arrangement);
        }
    }

    /// The spot under a point: a piece standing there, front-most first, or an open spot.
    fn hilltop_spot(&self, pointer: Option<(f32, f32)>) -> Option<u8> {
        let (x, y) = pointer?;
        let on_piece = self
            .piece_bounds
            .iter()
            .filter(|(_, (left, top, right, bottom))| {
                x >= *left as f32
                    && x <= *right as f32 + 1.0
                    && y >= *top as f32
                    && y <= *bottom as f32 + 1.0
            })
            .max_by(|a, b| {
                SPOTS[usize::from(a.0)]
                    .1
                    .total_cmp(&SPOTS[usize::from(b.0)].1)
            })
            .map(|(spot, _)| *spot);
        on_piece.or_else(|| hilltop::spot_at(x, y))
    }

    pub(super) fn compose_hilltop(&mut self, now: f32) -> Canvas {
        let hovered = self.hilltop_spot(self.pointer);
        let placing = self.placing.is_some();
        let Some(ground) = &mut self.hilltop else {
            return Canvas::new(super::SCENE_WIDTH, super::SCENE_HEIGHT);
        };
        let mut scene = ground.compose(now);
        // While placing, every spot shows as a ring on the grass, the one under the pointer gold.
        if placing {
            for (index, (x, y)) in SPOTS.iter().enumerate() {
                let here = hovered == Some(index as u8);
                let color = if here {
                    rgb(0xf5d25e)
                } else {
                    rgba(0xf6eed8, 170)
                };
                for step in 0..40 {
                    let angle = step as f32 / 40.0 * TAU;
                    let (dx, dy) = (angle.cos() * 15.0, angle.sin() * 4.5);
                    put(&mut scene, (x + dx) as i32, (y + dy) as i32, color);
                }
            }
        }
        scene
    }

    /// A click on the summit: place, move, or pick up a piece. Says whether it was taken.
    pub(super) fn hilltop_click(&mut self, pointer: Option<(f32, f32)>) -> bool {
        let Some(spot) = self.hilltop_spot(pointer) else {
            return false;
        };
        let occupied = self.memories.colony().hilltop.contains_key(&spot);
        match self.placing.take() {
            Some(Placing::FromSatchel(id)) => {
                self.memories.place(spot, &id);
            }
            Some(Placing::Lifted(standing)) => {
                self.memories.replant(spot, &standing);
            }
            Some(Placing::FromSpot(from)) => self.memories.move_piece(from, spot),
            None if occupied => {
                self.placing = Some(Placing::FromSpot(spot));
                return true;
            }
            None => return false,
        }
        self.refresh_hilltop();
        true
    }

    /// The name of the piece under the pointer, or where the one being placed would go.
    pub(super) fn hilltop_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let spot = self.hilltop_spot(pointer)?;
        let (x, y) = SPOTS[usize::from(spot)];
        let standing = self.memories.colony().hilltop.get(&spot);
        let label = match (&self.placing, standing) {
            (Some(_), _) => "Here".to_owned(),
            (None, Some(standing)) => standing.name().to_owned(),
            (None, None) => return None,
        };
        let top = self
            .piece_bounds
            .iter()
            .find(|(at, _)| *at == spot)
            .map_or(y - 20.0, |(_, (_, top, _, _))| *top as f32);
        Some((label, (x, top - 6.0)))
    }

    pub(super) fn hilltop_bar(&mut self, ui: &mut egui::Ui) {
        let colony = self.memories.colony();
        match self.placing.clone() {
            Some(Placing::FromSatchel(id)) => {
                let line = match finds::find(&id) {
                    Some(find) if growing::growth(&id).is_some() => {
                        format!("Choose a spot to plant {}.", find.name.to_lowercase())
                    }
                    Some(find) => format!("Choose a spot for {}.", find.piece.to_lowercase()),
                    None => "Choose a spot.".to_owned(),
                };
                ui.label(line);
                if ui.button("Cancel").clicked() {
                    self.placing = None;
                }
            }
            Some(Placing::Lifted(standing)) => {
                ui.label(format!(
                    "Choose a spot to plant {} again.",
                    standing.name().to_lowercase()
                ));
                if ui.button("Cancel").clicked() {
                    self.placing = None;
                }
            }
            Some(Placing::FromSpot(spot)) => {
                let name = colony.hilltop.get(&spot).map_or("", Standing::name);
                ui.label(format!("Moving {}: choose a spot.", name.to_lowercase()));
                if ui.button("Back in the satchel").clicked() {
                    self.memories.pick_up(spot);
                    self.placing = None;
                    self.refresh_hilltop();
                }
                if ui.button("Cancel").clicked() {
                    self.placing = None;
                }
            }
            None => {
                let satchel: Vec<(String, u32)> = colony
                    .satchel
                    .iter()
                    .map(|(id, count)| (id.clone(), *count))
                    .collect();
                let lifted = colony.lifted.clone();
                let count = satchel.iter().map(|(_, n)| n).sum::<u32>() + lifted.len() as u32;
                let mut chosen = None;
                ui.add_enabled_ui(count > 0, |ui| {
                    egui::ComboBox::from_id_salt("satchel")
                        .selected_text(format!("Satchel ({count})"))
                        .show_ui(ui, |ui| {
                            for (id, count) in &satchel {
                                let label = if *count > 1 {
                                    format!("{} \u{d7}{count}", in_the_satchel(id))
                                } else {
                                    in_the_satchel(id)
                                };
                                if ui.selectable_label(false, label).clicked() {
                                    chosen = Some(Placing::FromSatchel(id.clone()));
                                }
                            }
                            for standing in &lifted {
                                let label = format!("{}, lifted", standing.name());
                                if ui.selectable_label(false, label).clicked() {
                                    chosen = Some(Placing::Lifted(standing.clone()));
                                }
                            }
                        });
                });
                if chosen.is_some() {
                    self.placing = chosen;
                }
                let hint = if count == 0 && colony.hilltop.is_empty() {
                    "Finds from the Woods can stand up here."
                } else if !colony.hilltop.is_empty() {
                    "Click something to move it."
                } else {
                    ""
                };
                if !hint.is_empty() {
                    ui.label(egui::RichText::new(hint).italics());
                }
            }
        }
        if ui.button("Journal").clicked() {
            self.journal = !self.journal;
        }
    }

    /// Everything the Woods can turn up: what has been found, by whom, and hints of the rest.
    pub(super) fn journal_window(&mut self, ctx: &egui::Context) {
        if !self.journal {
            return;
        }
        let mut open = true;
        let colony = self.memories.colony();
        let cast = &self.arrival.cast;
        let who = |id: &str| {
            cast.members
                .iter()
                .find(|member| member.id.to_string() == id)
                .map_or(
                    "a companion who stayed home this time".to_owned(),
                    |member| member.name.clone(),
                )
        };
        egui::Window::new("Journal")
            .open(&mut open)
            .default_width(360.0)
            .default_height(420.0)
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} of {} found \u{b7} {} outing{} to the Woods",
                    colony.finds.len(),
                    CATALOGUE.len(),
                    colony.outings,
                    if colony.outings == 1 { "" } else { "s" }
                ));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for kind in Kind::ALL {
                        let of_kind: Vec<_> = CATALOGUE.iter().filter(|find| find.kind == kind).collect();
                        let found = of_kind.iter().filter(|find| colony.finds.contains_key(find.id)).count();
                        ui.add_space(6.0);
                        ui.strong(format!("Found {} ({found} of {})", kind.whereabouts(), of_kind.len()));
                        for find in of_kind {
                            match colony.finds.get(find.id) {
                                Some(record) => {
                                    let becomes = if growing::growth(find.id).is_some() { "Grows into" } else { "Becomes" };
                                    ui.label(egui::RichText::new(find.name).strong());
                                    ui.label(egui::RichText::new(find.blurb).small());
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{becomes} {} \u{b7} brought home {}\u{d7} \u{b7} first found with {}",
                                            find.piece.to_lowercase(),
                                            record.count,
                                            who(&record.first_by)
                                        ))
                                        .small()
                                        .italics(),
                                    );
                                }
                                None => {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "??? \u{b7} something {} {}",
                                            find.tier.label(),
                                            kind.whereabouts()
                                        ))
                                        .weak(),
                                    );
                                }
                            }
                        }
                    }
                    // Relics only appear once there is one to show: they are a secret until then.
                    let relics: Vec<_> = crate::finds::RELICS
                        .iter()
                        .filter(|relic| colony.finds.contains_key(relic.id))
                        .collect();
                    if !relics.is_empty() {
                        ui.add_space(6.0);
                        ui.strong("Relics");
                        for relic in relics {
                            ui.label(egui::RichText::new(relic.name).strong());
                            ui.label(egui::RichText::new(relic.blurb).small());
                            ui.label(
                                egui::RichText::new(format!("Becomes {}", relic.piece.to_lowercase()))
                                    .small()
                                    .italics(),
                            );
                        }
                    }
                    // What is planted and still coming up, on the Hilltop or lifted into the satchel.
                    let planted = colony.hilltop.values().map(|standing| (standing, true));
                    let lifted = colony.lifted.iter().map(|standing| (standing, false));
                    let growing: Vec<_> = planted.chain(lifted).filter(|(standing, _)| standing.growing()).collect();
                    if !growing.is_empty() {
                        ui.add_space(6.0);
                        ui.strong("Growing");
                        for (standing, up) in growing {
                            let Some(find) = finds::find(standing.id()) else {
                                continue;
                            };
                            let line = if up {
                                format!("Grows a little with every visit, into {}", find.piece.to_lowercase())
                            } else {
                                format!("Lifted into the satchel; it grows on into {} once it is planted again", find.piece.to_lowercase())
                            };
                            ui.label(egui::RichText::new(standing.name()).strong());
                            ui.label(egui::RichText::new(line).small().italics());
                        }
                    }
                    let fish = &crate::fishing::fish::CATALOGUE;
                    let caught = fish.iter().filter(|kind| colony.fish.contains_key(kind.id)).count();
                    ui.add_space(6.0);
                    ui.strong(format!("Caught at the pool ({caught} of {})", fish.len()));
                    for kind in fish {
                        match colony.fish.get(kind.id) {
                            Some(record) => {
                                ui.label(egui::RichText::new(kind.name).strong());
                                ui.label(egui::RichText::new(kind.blurb).small());
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Landed {}\u{d7}, longest {:.1} cm \u{b7} first caught by {}",
                                        record.count,
                                        record.longest,
                                        who(&record.first_by)
                                    ))
                                    .small()
                                    .italics(),
                                );
                            }
                            None => {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "??? \u{b7} something {} in {}",
                                        kind.tier.label(),
                                        kind.haunt.name()
                                    ))
                                    .weak(),
                                );
                            }
                        }
                    }
                    let bugs = &crate::meadow::bugs::CATALOGUE;
                    let caught = bugs.iter().filter(|kind| colony.bugs.contains_key(kind.id)).count();
                    ui.add_space(6.0);
                    ui.strong(format!("Caught in the meadow ({caught} of {})", bugs.len()));
                    for kind in bugs {
                        match colony.bugs.get(kind.id) {
                            Some(record) => {
                                ui.label(egui::RichText::new(kind.name).strong());
                                ui.label(egui::RichText::new(kind.blurb).small());
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Caught {}\u{d7}, biggest {:.0} mm across \u{b7} first caught by {}",
                                        record.count,
                                        record.biggest,
                                        who(&record.first_by)
                                    ))
                                    .small()
                                    .italics(),
                                );
                            }
                            None => {
                                let when = if kind.dusk { " at dusk" } else { "" };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "??? \u{b7} something {} in {}{when}",
                                        kind.tier.label(),
                                        kind.haunt.name()
                                    ))
                                    .weak(),
                                );
                            }
                        }
                    }
                    if !colony.outings_by.is_empty() {
                        ui.add_space(6.0);
                        ui.strong("Who has been");
                        for (id, count) in &colony.outings_by {
                            ui.label(format!("{}: {count}", who(id)));
                        }
                    }
                });
            });
        self.journal = open;
    }
}
