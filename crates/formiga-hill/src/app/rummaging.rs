//! The Woods in the window: choosing who comes along, the outing's controls, and keeping what
//! came home.

use super::HillApp;
use crate::cast::Id;
use crate::character::Character;
use crate::finds;
use crate::playground::Playground;
use crate::woods::rummage::{self, BASKET, Ending, Event, Outset, Phase, Rummage};
use crate::woods::{self, SPOTS};
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Band;

/// How many companions can come along at once.
pub const PARTY: usize = 2;

#[derive(Default)]
pub struct Woods {
    /// Who the person has chosen to bring; the first leads.
    pub party: Vec<Id>,
    /// The glade with nobody in it, between outings.
    pub empty: Option<Playground>,
    pub outing: Option<(Playground, Rummage)>,
}

impl HillApp {
    /// Readies the glade the first time anyone goes there.
    pub(super) fn open_woods(&mut self, now: f32) {
        if self.woods.empty.is_none() {
            self.woods.empty = Some(woods::open(&self.arrival.cast, &[], now));
        }
        if self.woods.party.is_empty()
            && let Some(first) = self.arrival.cast.members.first()
        {
            self.woods.party.push(first.id);
        }
    }

    /// Sets off with whoever was chosen.
    fn set_off(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let cast = &self.arrival.cast;
        let close_pair = match party.as_slice() {
            [a, b] => cast
                .bond(*a, *b)
                .is_some_and(|bond| bond.warmth >= Band::High),
            _ => false,
        };
        let colony = self.memories.colony();
        let seed = cast
            .snapshot
            .colony_id
            .bytes()
            .fold(u64::from(colony.outings) << 32, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        let mut ground = woods::open(cast, &party, now);
        let influence = woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            drought: colony.drought,
            close_pair,
            influence,
            seed,
        };
        let found = |id: &str| colony.finds.contains_key(id);
        let rummage = Rummage::new(&mut ground, outset, found, now);
        self.woods.outing = Some((ground, rummage));
        let line = note.unwrap_or("Choose somewhere to search. Watch for signs.");
        self.notice = Some((line.to_owned(), now));
    }

    /// Plays the outing on; with reduced motion, `holding` turns the marker.
    pub(super) fn tick_woods(&mut self, now: f32, holding: bool, dt: f32) {
        let cast = &self.arrival.cast;
        if let Some(empty) = &mut self.woods.empty
            && self.woods.outing.is_none()
        {
            empty.tick(cast, now);
        }
        let Some((ground, rummage)) = &mut self.woods.outing else {
            return;
        };
        ground.tick(cast, now);
        rummage.hold(ground, holding, dt, now);
        rummage.tick(ground, now);
        let name = |id: Id| {
            cast.member(id)
                .map_or("Someone", |member| member.name.as_str())
        };
        for event in rummage.take_events() {
            let line = match event {
                Event::Noticed { who, spot } => {
                    format!("{} spotted something by {}.", name(who), SPOTS[spot].name)
                }
                Event::Got { find, .. } => {
                    let find = finds::find(find).map_or("something", |find| find.name);
                    format!("Got it! {find}.")
                }
                Event::Slipped { .. } => "It slipped away deeper. Try again!".to_owned(),
                Event::Nothing { spot } => format!("Nothing at {}.", SPOTS[spot].name),
                Event::AlreadyLooked { spot } => {
                    format!("You've looked at {} already.", SPOTS[spot].name)
                }
                Event::Leaving(Ending::Dusk) => "The light's going. Time to head home.".to_owned(),
                Event::Leaving(Ending::Full) => "The basket's full! Home we go.".to_owned(),
                Event::Leaving(Ending::Chose) => "Heading home.".to_owned(),
            };
            self.notice = Some((line, now));
        }
        if rummage.phase() == Phase::Over {
            self.finish_outing(now);
        }
    }

    /// Brings the basket home: into the journal and the satchel, ready for the Hilltop.
    pub(super) fn finish_outing(&mut self, now: f32) {
        let Some((_, rummage)) = self.woods.outing.take() else {
            return;
        };
        let basket = rummage.basket().to_vec();
        let new = self.memories.back_from_the_woods(rummage.party(), &basket);
        let line = match (basket.len(), new.len()) {
            (0, _) => "Home from the Woods, empty-pawed this time.".to_owned(),
            (count, 0) => format!(
                "Home with {count} find{}. They're in the satchel, for the Hilltop.",
                plural(count)
            ),
            (count, fresh) => format!(
                "Home with {count} find{}, {fresh} new to the journal. They're in the satchel, \
                 for the Hilltop.",
                plural(count)
            ),
        };
        self.notice = Some((line, now));
    }

    pub(super) fn compose_woods(&mut self, now: f32) -> Canvas {
        match (&mut self.woods.outing, &mut self.woods.empty) {
            (Some((ground, rummage)), _) => {
                let mut scene = ground.compose(now);
                rummage.draw(&mut scene, now);
                scene
            }
            (None, Some(empty)) => empty.compose(now),
            (None, None) => Canvas::new(super::SCENE_WIDTH, super::SCENE_HEIGHT),
        }
    }

    /// A click in the glade: choose a spot, or catch the moment.
    pub(super) fn woods_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some((ground, rummage)) = &mut self.woods.outing else {
            return;
        };
        let spot = pointer.and_then(|(x, y)| Rummage::spot_at(x, y));
        match rummage.phase() {
            Phase::Catching(catch) if spot.is_none_or(|spot| spot == catch.spot) => {
                if !ground.reduce_motion() {
                    rummage.strike(ground, now);
                }
            }
            Phase::Exploring | Phase::Catching(_) => {
                if let Some(spot) = spot {
                    rummage.choose(ground, spot, now);
                }
            }
            _ => {}
        }
    }

    /// The key for catching, for anyone not using the pointer.
    pub(super) fn woods_strike(&mut self, now: f32) {
        if let Some((ground, rummage)) = &mut self.woods.outing
            && matches!(rummage.phase(), Phase::Catching(_))
            && !ground.reduce_motion()
        {
            rummage.strike(ground, now);
        }
    }

    /// What to call the spot under the pointer, and where, while choosing.
    pub(super) fn woods_hover(&self, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
        let (_, rummage) = self.woods.outing.as_ref()?;
        if !matches!(rummage.phase(), Phase::Exploring | Phase::Catching(_)) {
            return None;
        }
        let spot = pointer.and_then(|(x, y)| Rummage::spot_at(x, y))?;
        let place = SPOTS[spot];
        let label = if rummage.searched(spot) {
            format!("{} (looked)", place.name)
        } else {
            format!("{} \u{b7} {}", place.name, place.kind.verb())
        };
        Some((label, (place.sign.0, place.sign.1 - 12.0)))
    }

    pub(super) fn woods_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let mut set_off = false;
        let mut home = false;
        match &self.woods.outing {
            None => {
                ui.label("Who's coming?");
                let cast = &self.arrival.cast;
                let colony = self.memories.colony();
                for member in &cast.members {
                    let chosen = self.woods.party.contains(&member.id);
                    let character = Character::of(member);
                    let outings = colony
                        .outings_by
                        .get(&member.id.to_string())
                        .copied()
                        .unwrap_or(0);
                    let hint = format!(
                        "Good at {}. {} outing{} so far.",
                        rummage::best_at(&character).knack(),
                        outings,
                        plural(outings as usize)
                    );
                    if ui
                        .selectable_label(chosen, &member.name)
                        .on_hover_text(hint)
                        .clicked()
                    {
                        if chosen {
                            self.woods.party.retain(|id| *id != member.id);
                        } else if self.woods.party.len() < PARTY {
                            self.woods.party.push(member.id);
                        } else {
                            // Swap out whoever was chosen last.
                            self.woods.party.pop();
                            self.woods.party.push(member.id);
                        }
                    }
                }
                let ready = !self.woods.party.is_empty();
                set_off = ui
                    .add_enabled(ready, egui::Button::new("Set off"))
                    .clicked();
            }
            Some((ground, rummage)) => {
                ui.add(
                    egui::ProgressBar::new(rummage.light_left())
                        .desired_width(70.0)
                        .text("light"),
                );
                ui.label(format!("Basket {}/{BASKET}", rummage.basket().len()));
                let hint = match rummage.phase() {
                    Phase::Catching(_) if ground.reduce_motion() => {
                        "Hold to turn the marker; let go on the gold."
                    }
                    Phase::Catching(_) => "Click (or Space) as the marker crosses the gold!",
                    _ => "",
                };
                if !hint.is_empty() {
                    ui.label(egui::RichText::new(hint).strong());
                }
                let leaving = matches!(rummage.phase(), Phase::Leaving { .. } | Phase::Over);
                home = ui
                    .add_enabled(!leaving, egui::Button::new("Head home"))
                    .clicked();
            }
        }
        if ui.button("Journal").clicked() {
            self.journal = !self.journal;
        }
        if set_off {
            self.set_off(now);
        }
        if home && let Some((ground, rummage)) = &mut self.woods.outing {
            rummage.head_home(ground, now);
        }
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
