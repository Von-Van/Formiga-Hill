//! Foraging along the hedgerow, in the window: setting off, choosing where to go and what to
//! pick, putting things back to make room, and bringing the basket home.

use super::HillApp;
use crate::cast::Id;
use crate::finds::{self, FORAGED};
use crate::hedgerow::foraging::{BASKET, Ending, Event, Foray, Outset, Phase};
use crate::hedgerow::produce::Stage;
use crate::hedgerow::{self, PATCHES, Plant, Reach};
use crate::memories::ColonyMemories;
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Band;

impl HillApp {
    /// Readies the hedgerow, with nobody foraging, and shows the Hilltop through its gate as it
    /// stands now.
    pub(super) fn open_hedgerow(&mut self, now: f32) {
        let hilltop = self.hilltop_standing();
        match &mut self.woods.hedgerow {
            Some(lane) => hedgerow::show_hilltop(lane, &hilltop),
            None => {
                let mut lane = hedgerow::open(&self.arrival.cast, &[], now, &hilltop);
                lane.set_fliers(hedgerow::growing());
                lane.set_daylight(self.daylight);
                self.woods.hedgerow = Some(lane);
            }
        }
    }

    /// Sets off along the hedgerow with whoever was chosen; the first leads.
    pub(super) fn set_off_foraging(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let close_pair = match party.as_slice() {
            [a, b] => cast
                .bond(*a, *b)
                .is_some_and(|bond| bond.warmth >= Band::High),
            _ => false,
        };
        let seed = cast
            .snapshot
            .colony_id
            .bytes()
            .fold(u64::from(colony.forays) << 32 | 0xf0a, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        let mut lane = hedgerow::open(cast, &party, now, &colony.hilltop);
        lane.set_daylight(self.daylight);
        let influence = crate::woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            drought: colony.forage_drought,
            close_pair,
            influence,
            seed,
        };
        let found = |id: &str| colony.finds.contains_key(id);
        let mut foray = Foray::new(&mut lane, outset, found, now);
        foray.set_hour_dark(self.daylight.darkness());
        self.woods.foray = Some((lane, foray));
        let line = note.unwrap_or(
            "Everything here ripens in its own time. Click a bush to go to it, and pick what is \
             ripe.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Plays the foray on.
    pub(super) fn tick_foraging(&mut self, now: f32) {
        let cast = &self.arrival.cast;
        let Some((lane, foray)) = &mut self.woods.foray else {
            return;
        };
        lane.tick(cast, now);
        foray.tick(lane, now);
        let events = foray.take_events();
        let name = |id: Id| {
            cast.member(id)
                .map_or("Someone", |member| member.name.as_str())
        };
        let what = |item: usize| {
            foray
                .item(item)
                .map_or("something", |(plant, _, _)| plant.produce())
        };
        let find_name = |id: &str| finds::find(id).map_or("something", |find| find.name);
        for event in events {
            let line = match event {
                Event::Opened { who, reach, pair } => match (reach, pair) {
                    (Reach::Tucked, _) => {
                        format!("{} is small enough for the tucked-away ones.", name(who))
                    }
                    (Reach::High, false) => {
                        format!("{} can climb for the high branches.", name(who))
                    }
                    _ => "Together, these two can bend the high branches down.".to_owned(),
                },
                Event::Spotted { who, item } => {
                    format!("{} has spotted {} under the leaves!", name(who), what(item))
                }
                Event::Uncovered { item } => {
                    format!("There's {} hiding under the leaves here.", what(item))
                }
                Event::Ripening { who, item } => {
                    let place = foray
                        .item(item)
                        .map_or("somewhere", |(_, patch, _)| PATCHES[patch].name);
                    format!(
                        "{} says {} by {} is nearly ripe.",
                        name(who),
                        what(item),
                        place
                    )
                }
                Event::Picked { find, .. } => format!("Picked! {}.", find_name(find)),
                Event::Saved { who, .. } => {
                    format!(
                        "Just past its best, but {} picked it so gently it came whole.",
                        name(who)
                    )
                }
                Event::NotYet { item } => {
                    format!("Not ripe yet: the {} stays on for now.", what(item))
                }
                Event::LookedOver { who, .. } => {
                    format!("{} looked it over and put it back: not yet.", name(who))
                }
                Event::Spoilt { item } => {
                    format!(
                        "Too late: the {} was past its best, and fell to bits.",
                        what(item)
                    )
                }
                Event::Ate { who, .. } => format!("{} ate that one. There were plenty.", name(who)),
                Event::BasketFull => {
                    "The basket's full. Put something back to make room, or head home.".to_owned()
                }
                Event::PutBack { find } => {
                    format!("Put back: {}. Left for the birds.", find_name(find))
                }
                Event::Dropped { item } => {
                    let rare = foray
                        .item(item)
                        .is_some_and(|(plant, _, _)| plant.find().tier >= finds::Tier::Rare);
                    if !rare {
                        continue;
                    }
                    format!("The {} has gone over and dropped.", what(item))
                }
                Event::ForTheBirds { who, count } => format!(
                    "{} leaves the ripe one{} for the birds.",
                    name(who),
                    if count == 1 { "" } else { "s" }
                ),
                Event::Thanked { who, item } => format!(
                    "A robin has brought {} {} for leaving them so much!",
                    name(who),
                    what(item)
                ),
                Event::Dusk => {
                    "The light is going. The honeysuckle opens, and down in the clover something \
                     catches the last of it\u{2026}"
                        .to_owned()
                }
                Event::Leaving(Ending::Dusk) => "The light's gone. Time to head home.".to_owned(),
                Event::Leaving(Ending::Chose) => "Heading home.".to_owned(),
            };
            self.notice = Some((line, now));
        }
        if foray.phase() == Phase::Over {
            self.finish_foraging(now);
        }
    }

    /// Brings the basket home: into the journal and the satchel, ready for the Hilltop.
    pub(super) fn finish_foraging(&mut self, now: f32) {
        let Some((_, foray)) = self.woods.foray.take() else {
            return;
        };
        let basket = foray.basket().to_vec();
        let new = self.memories.back_from_foraging(foray.party(), &basket);
        let line = match (basket.len(), new.len()) {
            (0, _) => "Home from the hedgerow, the basket empty this time.".to_owned(),
            (count, 0) => format!(
                "Home from the hedgerow with {count} thing{} picked. They're in the satchel, \
                 for the Hilltop.",
                if count == 1 { "" } else { "s" }
            ),
            (count, fresh) => format!(
                "Home from the hedgerow with {count} thing{} picked, {fresh} new to the \
                 journal. They're in the satchel, for the Hilltop.",
                if count == 1 { "" } else { "s" }
            ),
        };
        self.notice = Some((line, now));
    }

    /// The hedgerow with a foray under way: what grows there as it ripens, and the thing under
    /// the pointer marked.
    pub(super) fn compose_foray(&mut self, now: f32) -> Option<Canvas> {
        let pointer = self.pointer;
        let (lane, foray) = self.woods.foray.as_mut()?;
        lane.set_fliers(foray.produce(now));
        let mut scene = lane.compose(now);
        let hovered = pointer.and_then(|(x, y)| foray.item_at(x, y));
        foray.draw(&mut scene, hovered, now);
        Some(scene)
    }

    /// A click along the hedgerow: put something back from the basket, pick something ripe where
    /// the party is, or go to wherever was clicked.
    pub(super) fn foraging_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some((lane, foray)) = &mut self.woods.foray else {
            return;
        };
        let Some((x, y)) = pointer else {
            return;
        };
        if let Some(slot) = foray.basket_slot_at(x, y, super::SCENE_WIDTH) {
            foray.put_back(slot);
            return;
        }
        if let Some(item) = foray.item_at(x, y)
            && let Some((_, patch, _)) = foray.item(item)
            && foray.at() == Some(patch)
        {
            foray.pick(lane, item, now);
            return;
        }
        if let Some(patch) = foray.patch_at(x, y) {
            foray.choose(lane, patch, now);
        }
    }

    /// What is under the pointer along the hedgerow, and where to say so.
    pub(super) fn foraging_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let (_, foray) = self.woods.foray.as_ref()?;
        if matches!(foray.phase(), Phase::Leaving { .. } | Phase::Over) {
            return None;
        }
        let (x, y) = pointer?;
        if let Some(slot) = foray.basket_slot_at(x, y, super::SCENE_WIDTH) {
            let find = foray.basket()[slot];
            let name = finds::find(find).map_or("it", |find| find.name);
            return Some((format!("Put back {}", lower(name)), (x, y + 14.0)));
        }
        if let Some(item) = foray.item_at(x, y)
            && let Some((plant, patch, at)) = foray.item(item)
        {
            // One that looks food over says how ripe it really is.
            let verdict = match foray.inspected(item) {
                Some(Stage::Ripe) => " \u{b7} ripe!",
                Some(Stage::Over) => " \u{b7} past its best",
                Some(_) => " \u{b7} not yet",
                None => "",
            };
            let label = if foray.at() == Some(patch) {
                format!("{}{verdict}", plant.produce())
            } else {
                format!("{} \u{b7} {}", PATCHES[patch].name, plant.produce())
            };
            return Some((label, (at.0, at.1 - 8.0)));
        }
        let patch = foray.patch_at(x, y)?;
        let place = &PATCHES[patch];
        (foray.at() != Some(patch))
            .then(|| (place.name.to_owned(), (place.stand.0, place.stand.1 - 40.0)))
    }

    pub(super) fn foraging_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let Some((lane, foray)) = &self.woods.foray else {
            return;
        };
        ui.add(
            egui::ProgressBar::new(foray.light_left())
                .desired_width(70.0)
                .text("light"),
        );
        ui.label(format!("Basket {}/{BASKET}", foray.basket().len()));
        let hint = match foray.phase() {
            Phase::Choosing | Phase::Going { .. } => "Click a bush to go to it.",
            Phase::At { .. } if foray.basket().len() >= BASKET => {
                "Full! Click a basket slot to put something back."
            }
            Phase::At { .. } => "Pick what's ripe. Click elsewhere to move on.",
            _ => "",
        };
        if !hint.is_empty() {
            ui.label(egui::RichText::new(hint).strong());
        }
        let _ = lane;
        let leaving = matches!(foray.phase(), Phase::Leaving { .. } | Phase::Over);
        if ui
            .add_enabled(!leaving, egui::Button::new("Head home"))
            .clicked()
            && let Some((lane, foray)) = &mut self.woods.foray
        {
            foray.head_home(lane, now);
        }
    }
}

/// The hedgerow's part of the journal: what has been picked there, by whom first, and hints of
/// the rest by where they grow.
pub(super) fn journal(ui: &mut egui::Ui, colony: &ColonyMemories, who: &dyn Fn(&str) -> String) {
    let picked = FORAGED
        .iter()
        .filter(|find| colony.finds.contains_key(find.id))
        .count();
    ui.add_space(6.0);
    ui.strong(format!(
        "Picked at the hedgerow ({picked} of {}) \u{b7} {} foray{}",
        FORAGED.len(),
        colony.forays,
        if colony.forays == 1 { "" } else { "s" }
    ));
    for find in &FORAGED {
        match colony.finds.get(find.id) {
            Some(record) => {
                ui.label(egui::RichText::new(find.name).strong());
                ui.label(egui::RichText::new(find.blurb).small());
                ui.label(
                    egui::RichText::new(format!(
                        "Becomes {} \u{b7} brought home {}\u{d7} \u{b7} first picked with {}",
                        find.piece.to_lowercase(),
                        record.count,
                        who(&record.first_by)
                    ))
                    .small()
                    .italics(),
                );
            }
            None => {
                let place = Plant::of(find.id)
                    .and_then(|plant| {
                        PATCHES
                            .iter()
                            .find(|patch| patch.slots.iter().any(|slot| slot.plant == plant))
                    })
                    .map_or("along the hedgerow", |patch| patch.name);
                let when = if find.tier == finds::Tier::Exceptional {
                    ", in the last of the light"
                } else {
                    ""
                };
                ui.label(
                    egui::RichText::new(format!(
                        "??? \u{b7} something {} at {place}{when}",
                        find.tier.label()
                    ))
                    .weak(),
                );
            }
        }
    }
}

/// What sort of forager a companion makes, and whether it has been yet, for the person choosing
/// who to bring.
pub(super) fn forager_hint(
    colony: &ColonyMemories,
    id: Id,
    character: &crate::character::Character,
) -> String {
    let forays = colony.forays_by.get(&id.to_string()).copied().unwrap_or(0);
    let been = match forays {
        0 => "Never been foraging yet.".to_owned(),
        1 => "Been foraging once.".to_owned(),
        count => format!("{count} forays so far."),
    };
    format!("{} {been}", crate::hedgerow::foraging::forager(character))
}

/// A find's name as it reads mid-sentence.
fn lower(name: &str) -> String {
    let mut name = name.to_owned();
    if let Some(first) = name.get_mut(0..1) {
        first.make_ascii_lowercase();
    }
    name
}
