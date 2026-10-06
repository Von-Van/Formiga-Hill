//! The Hilltop in the window: placing finds from the satchel on its spots, planting what grows,
//! moving them about, and the journal of everything the Woods has turned up. Building is in
//! `plans`.

use super::departures::grown_since;
use super::{Card, HillApp, paper};
use crate::audio::Cue;
use crate::finds::{self, CATALOGUE, Kind, growing, plans};
use crate::hilltop::{self, Arrangement, SPOTS, Standing};
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
    /// A plan, to be built.
    Plan(&'static str),
}

/// How a find in the satchel is listed, by name as the plans speak of it, and what it becomes on
/// the Hilltop, for the person to read on hover.
fn in_the_satchel(id: &str) -> (String, String) {
    match finds::find(id) {
        Some(find) if growing::growth(id).is_some() => (
            find.name.to_owned(),
            format!("Planted, it grows into {}", find.piece.to_lowercase()),
        ),
        Some(find) => (
            find.name.to_owned(),
            format!("Stands as {}", find.piece.to_lowercase()),
        ),
        None => (id.to_owned(), String::new()),
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
                self.sound.play(Cue::Grown);
            }
        }
    }

    /// What stands on the summit, from wherever it is seen: anything still being built isn't up
    /// yet, and its spot stands empty while the colony works, on the skylines below too.
    pub(super) fn hilltop_standing(&self) -> Arrangement {
        let mut up = self.memories.colony().hilltop.clone();
        if let Some(spot) = self.going_up() {
            up.remove(&spot);
        }
        up
    }

    /// Puts the summit's pieces where the memories say they stand, here and on every skyline
    /// that shows it.
    pub(super) fn refresh_hilltop(&mut self) {
        let up = self.hilltop_standing();
        let placed = hilltop::placed(&up);
        self.piece_bounds = placed
            .iter()
            .map(|(spot, prop)| (*spot, prop.bounds()))
            .collect();
        let props = placed.into_iter().map(|(_, prop)| prop).collect();
        if let Some(ground) = &mut self.hilltop {
            ground.set_props(props);
            ground.set_attractions(hilltop::attractions(&up));
            let lights = hilltop::nightlights(&up, ground.backdrop());
            ground.set_nightlights(lights);
        }
        // The Hill seen from below changes too, once there is something to see.
        self.station.show_hilltop(&up);
        if let Some((ground, _)) = &mut self.fairground {
            crate::fairground::show_hilltop(ground, &up);
        }
        if let Some(green) = &mut self.green {
            crate::green::show_hilltop(green, &up);
        }
        if let Some(room) = &mut self.clubhouse {
            room.show_hilltop(&up);
        }
        if let Some(meadow) = &mut self.woods.meadow {
            crate::meadow::show_hilltop(meadow, &up);
        }
        if let Some(lane) = &mut self.woods.hedgerow {
            crate::hedgerow::show_hilltop(lane, &up);
        }
        if let Some(track) = &mut self.woods.track {
            crate::track::show_hilltop(track, &up);
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
        self.draw_crafting(&mut scene, now);
        scene
    }

    /// A click on the summit: place, move, or pick up a piece. Says whether it was taken.
    pub(super) fn hilltop_click(&mut self, pointer: Option<(f32, f32)>) -> bool {
        let Some(spot) = self.hilltop_spot(pointer) else {
            return false;
        };
        // Nothing on the spot being built on can be picked up while the colony works there.
        if self.going_up() == Some(spot) {
            return true;
        }
        let occupied = self.memories.colony().hilltop.contains_key(&spot);
        let stood = match self.placing.take() {
            Some(Placing::FromSatchel(id)) => self.memories.place(spot, &id),
            Some(Placing::Lifted(standing)) => self.memories.replant(spot, &standing),
            Some(Placing::FromSpot(from)) => {
                self.memories.move_piece(from, spot);
                from != spot
            }
            Some(Placing::Plan(plan)) => {
                let now = self.now();
                self.start_building(spot, plan, now);
                return true;
            }
            None if occupied => {
                self.placing = Some(Placing::FromSpot(spot));
                return true;
            }
            None => return false,
        };
        if stood {
            self.sound.play(Cue::Thunk);
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
            (Some(Placing::Plan(_)), _) => "Build here".to_owned(),
            (Some(_), _) => "Here".to_owned(),
            (None, _) if self.going_up() == Some(spot) => "Being built".to_owned(),
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

    pub(super) fn hilltop_tray(&mut self, ui: &mut egui::Ui) {
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
                paper::slip(ui, line);
                if paper::button(ui, "Cancel").clicked() {
                    self.placing = None;
                }
            }
            Some(Placing::Lifted(standing)) => {
                paper::slip(
                    ui,
                    format!(
                        "Choose a spot to plant {} again.",
                        standing.name().to_lowercase()
                    ),
                );
                if paper::button(ui, "Cancel").clicked() {
                    self.placing = None;
                }
            }
            Some(Placing::Plan(plan)) => {
                let name = plans::plan(plan).map_or("it", |plan| plan.name);
                paper::slip(
                    ui,
                    format!("Choose a spot to build {}.", name.to_lowercase()),
                );
                if paper::button(ui, "Cancel").clicked() {
                    self.placing = None;
                }
            }
            Some(Placing::FromSpot(spot)) => {
                let standing = colony.hilltop.get(&spot);
                let name = standing.map_or("", Standing::name);
                let built = standing.is_some_and(|standing| standing.plan().is_some());
                paper::slip(
                    ui,
                    format!("Moving {}: choose a spot.", name.to_lowercase()),
                );
                if built {
                    if paper::button(ui, "Take apart").clicked() {
                        self.placing = None;
                        let now = self.now();
                        self.take_apart(spot, now);
                    }
                } else if paper::button(ui, "Back in the satchel").clicked() {
                    self.memories.pick_up(spot);
                    self.placing = None;
                    self.refresh_hilltop();
                }
                if paper::button(ui, "Cancel").clicked() {
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
                let satchel_button =
                    paper::Button::new(format!("Satchel ({count}) \u{25b4}")).enabled(count > 0);
                let (_, chosen) = paper::menu(ui, satchel_button, false, |ui| {
                    let mut chosen = None;
                    for (id, count) in &satchel {
                        let (name, becomes) = in_the_satchel(id);
                        let label = if *count > 1 {
                            format!("{name} \u{d7}{count}")
                        } else {
                            name
                        };
                        let row = paper::button(ui, label);
                        if paper::explain(ui, row, &becomes).clicked() {
                            chosen = Some(Placing::FromSatchel(id.clone()));
                        }
                    }
                    for standing in &lifted {
                        let row = paper::button(ui, format!("{}, lifted", standing.name()));
                        let about =
                            "Lifted from the Hilltop as far as it had grown, to plant again";
                        if paper::explain(ui, row, about).clicked() {
                            chosen = Some(Placing::Lifted(standing.clone()));
                        }
                    }
                    chosen
                });
                if let Some(Some(chosen)) = chosen {
                    self.placing = Some(chosen);
                }
                let hint = if count == 0 && colony.hilltop.is_empty() {
                    "Finds from the Woods can stand up here."
                } else if !colony.hilltop.is_empty() {
                    "Click something to move it."
                } else {
                    ""
                };
                if !hint.is_empty() {
                    paper::hint(ui, hint);
                }
            }
        }
        // Plans only once there is an idea for one: the first find that goes into any.
        if !self.thought_of().is_empty() {
            let ready = self.ready_to_build().len();
            let plans = if ready > 0 {
                format!("Plans ({ready} ready)")
            } else {
                "Plans".to_owned()
            };
            if ui
                .add(paper::Button::new(plans).selected(self.showing(Card::Plans)))
                .clicked()
            {
                self.toggle(Card::Plans);
            }
        }
    }

    /// Everything the Woods can turn up: what has been found, by whom, and hints of the rest.
    pub(super) fn journal_window(&mut self, ctx: &egui::Context) {
        self.show_card(ctx, Card::Journal, "Journal", 0.5, |app, ui| {
            app.journal_page(ui)
        });
    }

    fn journal_page(&self, ui: &mut egui::Ui) {
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
        {
            {
                paper::words(
                    ui,
                    format!(
                        "{} of {} found \u{b7} {} outing{} to the Woods",
                        CATALOGUE
                            .iter()
                            .filter(|find| colony.finds.contains_key(find.id))
                            .count(),
                        CATALOGUE.len(),
                        colony.outings,
                        if colony.outings == 1 { "" } else { "s" }
                    ),
                );
                {
                    for kind in Kind::ALL {
                        let of_kind: Vec<_> =
                            CATALOGUE.iter().filter(|find| find.kind == kind).collect();
                        let found = of_kind
                            .iter()
                            .filter(|find| colony.finds.contains_key(find.id))
                            .count();
                        ui.add_space(6.0);
                        paper::heading(
                            ui,
                            format!(
                                "Found {} ({found} of {})",
                                kind.whereabouts(),
                                of_kind.len()
                            ),
                        );
                        for find in of_kind {
                            match colony.finds.get(find.id) {
                                Some(record) => {
                                    let becomes = if growing::growth(find.id).is_some() {
                                        "Grows into"
                                    } else {
                                        "Becomes"
                                    };
                                    paper::name(ui, find.name);
                                    paper::words(ui, find.blurb);
                                    paper::aside(
                                        ui,
                                        format!(
                                            "{becomes} {} \u{b7} brought home {}\u{d7} \u{b7} first found with {}",
                                            find.piece.to_lowercase(),
                                            record.count,
                                            who(&record.first_by)
                                        ),
                                    );
                                }
                                None => {
                                    paper::faint(
                                        ui,
                                        format!(
                                            "??? \u{b7} something {} {}",
                                            find.tier.label(),
                                            kind.whereabouts()
                                        ),
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
                        paper::heading(ui, "Relics");
                        for relic in relics {
                            paper::name(ui, relic.name);
                            paper::words(ui, relic.blurb);
                            paper::aside(ui, format!("Becomes {}", relic.piece.to_lowercase()));
                        }
                    }
                    // What is planted and still coming up, on the Hilltop or lifted into the satchel.
                    let planted = colony.hilltop.values().map(|standing| (standing, true));
                    let lifted = colony.lifted.iter().map(|standing| (standing, false));
                    let growing: Vec<_> = planted
                        .chain(lifted)
                        .filter(|(standing, _)| standing.growing())
                        .collect();
                    if !growing.is_empty() {
                        ui.add_space(6.0);
                        paper::heading(ui, "Growing");
                        for (standing, up) in growing {
                            let Some(find) = finds::find(standing.id()) else {
                                continue;
                            };
                            let line = if up {
                                format!(
                                    "Grows a little with every visit, into {}",
                                    find.piece.to_lowercase()
                                )
                            } else {
                                format!(
                                    "Lifted into the satchel; it grows on into {} once it is planted again",
                                    find.piece.to_lowercase()
                                )
                            };
                            paper::name(ui, standing.name());
                            paper::aside(ui, line);
                        }
                    }
                    self.journal_plans(ui);
                    let fish = &crate::fishing::fish::CATALOGUE;
                    let caught = fish
                        .iter()
                        .filter(|kind| colony.fish.contains_key(kind.id))
                        .count();
                    ui.add_space(6.0);
                    paper::heading(
                        ui,
                        format!("Caught at the pool ({caught} of {})", fish.len()),
                    );
                    for kind in fish {
                        match colony.fish.get(kind.id) {
                            Some(record) => {
                                paper::name(ui, kind.name);
                                paper::words(ui, kind.blurb);
                                paper::aside(
                                    ui,
                                    format!(
                                        "Landed {}\u{d7}, longest {:.1} cm \u{b7} first caught by {}",
                                        record.count,
                                        record.longest,
                                        who(&record.first_by)
                                    ),
                                );
                            }
                            None => {
                                paper::faint(
                                    ui,
                                    format!(
                                        "??? \u{b7} something {} in {}",
                                        kind.tier.label(),
                                        kind.haunt.name()
                                    ),
                                );
                            }
                        }
                    }
                    let bugs = &crate::meadow::bugs::CATALOGUE;
                    let caught = bugs
                        .iter()
                        .filter(|kind| colony.bugs.contains_key(kind.id))
                        .count();
                    ui.add_space(6.0);
                    paper::heading(
                        ui,
                        format!("Caught in the meadow ({caught} of {})", bugs.len()),
                    );
                    for kind in bugs {
                        match colony.bugs.get(kind.id) {
                            Some(record) => {
                                paper::name(ui, kind.name);
                                paper::words(ui, kind.blurb);
                                paper::aside(
                                    ui,
                                    format!(
                                        "Caught {}\u{d7}, biggest {:.0} mm across \u{b7} first caught by {}",
                                        record.count,
                                        record.biggest,
                                        who(&record.first_by)
                                    ),
                                );
                            }
                            None => {
                                let when = if kind.dusk { " at dusk" } else { "" };
                                paper::faint(
                                    ui,
                                    format!(
                                        "??? \u{b7} something {} in {}{when}",
                                        kind.tier.label(),
                                        kind.haunt.name()
                                    ),
                                );
                            }
                        }
                    }
                    super::foraging::journal(ui, colony, &who);
                    super::expedition::journal(ui, colony, &who);
                    super::scavenging::journal(ui, colony, &who);
                    if !colony.outings_by.is_empty() {
                        ui.add_space(6.0);
                        paper::heading(ui, "Who has been");
                        for (id, count) in &colony.outings_by {
                            paper::words(ui, format!("{}: {count}", who(id)));
                        }
                    }
                }
            }
        }
    }
}
