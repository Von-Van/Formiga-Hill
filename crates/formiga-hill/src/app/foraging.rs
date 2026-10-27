//! Foraging along the hedgerow, in the window: setting off, choosing where to go and what to
//! pick, putting things back to make room, and bringing the basket home.

use super::outing::{Outing, Told};
use super::{HillApp, lower, paper, plural};
use crate::audio::Cue;
use crate::cast::{Cast, Id};
use crate::finds::{self, FORAGED};
use crate::hedgerow::foraging::{BASKET, Ending, Event, Foray, Outset, Phase};
use crate::hedgerow::produce::Stage;
use crate::hedgerow::{self, PATCHES, Plant, Reach};
use crate::memories::ColonyMemories;
use crate::playground::Playground;
use eframe::egui;

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
        let close_pair = cast.close_pair(&party);
        let seed = self.outing_seed(colony.forays, 0xf0a);
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
        self.woods.outing = Some(Outing::Foraging(lane, foray));
        let line = note.unwrap_or(
            "Everything here ripens in its own time. Click a bush to go to it, and pick what is \
             ripe.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Brings the basket home: into the journal and the satchel, ready for the Hilltop.
    pub(super) fn finish_foraging(&mut self, foray: &Foray, now: f32) {
        let basket = foray.basket().to_vec();
        let new = self.memories.back_from_foraging(foray.party(), &basket);
        let line = match (basket.len(), new.len()) {
            (0, _) => "Home from the hedgerow, the basket empty this time.".to_owned(),
            (count, 0) => format!(
                "Home from the hedgerow with {count} thing{} picked. They're in the satchel, \
                 for the Hilltop.",
                plural(count)
            ),
            (count, fresh) => format!(
                "Home from the hedgerow with {count} thing{} picked, {fresh} new to the \
                 journal. They're in the satchel, for the Hilltop.",
                plural(count)
            ),
        };
        self.notice = Some((line, now));
        self.grant(super::departures::Outing::Foraging, foray.party(), now);
    }
}

/// What the foray's events sound like, and say.
pub(super) fn told(cast: &Cast, foray: &Foray, events: Vec<Event>) -> Vec<Told> {
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
    events
        .into_iter()
        .map(|event| {
            let cue = super::sound::foraged(&event);
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
                        return Told { cue, line: None };
                    }
                    format!("The {} has gone over and dropped.", what(item))
                }
                Event::ForTheBirds { who, count } => format!(
                    "{} leaves the ripe one{} for the birds.",
                    name(who),
                    plural(count as usize)
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
            Told {
                cue,
                line: Some(line),
            }
        })
        .collect()
}

/// A click along the hedgerow: put something back from the basket, pick something ripe where
/// the party is, or go to wherever was clicked.
pub(super) fn click(
    lane: &mut Playground,
    foray: &mut Foray,
    pointer: Option<(f32, f32)>,
    now: f32,
) -> Option<Cue> {
    let (x, y) = pointer?;
    if let Some(slot) = foray.basket_slot_at(x, y, super::SCENE_WIDTH) {
        foray.put_back(slot);
        return None;
    }
    if let Some(item) = foray.item_at(x, y)
        && let Some((_, patch, _)) = foray.item(item)
        && foray.at() == Some(patch)
    {
        foray.pick(lane, item, now);
        return None;
    }
    let patch = foray.patch_at(x, y)?;
    foray.choose(lane, patch, now);
    Some(Cue::Forage)
}

/// What is under the pointer along the hedgerow, and where to say so.
pub(super) fn hover(foray: &Foray, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
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

/// The foray's tray: the light left, the basket, what to do, and the way home.
pub(super) fn tray(ui: &mut egui::Ui, lane: &mut Playground, foray: &mut Foray, now: f32) {
    paper::meter(ui, foray.light_left(), "light");
    paper::slip(ui, format!("Basket {}/{BASKET}", foray.basket().len()));
    let hint = match foray.phase() {
        Phase::Choosing | Phase::Going { .. } => "Click a bush to go to it.",
        Phase::At { .. } if foray.basket().len() >= BASKET => {
            "Full! Click a basket slot to put something back."
        }
        Phase::At { .. } => "Pick what's ripe. Click elsewhere to move on.",
        _ => "",
    };
    if !hint.is_empty() {
        paper::hint(ui, hint);
    }
    let leaving = matches!(foray.phase(), Phase::Leaving { .. } | Phase::Over);
    if ui
        .add(paper::Button::new("Head home").enabled(!leaving))
        .clicked()
    {
        foray.head_home(lane, now);
    }
}

/// The hedgerow's part of the journal: what has been picked there, by whom first, and hints of
/// the rest by where they grow.
/// How the hedgerow chapter of the journal stands, for its line in the contents.
pub(super) fn journal_line(colony: &ColonyMemories) -> String {
    let picked = FORAGED
        .iter()
        .filter(|find| colony.finds.contains_key(find.id))
        .count();
    format!(
        "{picked} of {} picked \u{b7} {} foray{}",
        FORAGED.len(),
        colony.forays,
        plural(colony.forays as usize)
    )
}

/// The hedgerow's chapter of the journal: what has been picked, in full. Returns a hint of each
/// thing still to pick.
pub(super) fn journal(
    ui: &mut egui::Ui,
    colony: &ColonyMemories,
    who: &dyn Fn(&str) -> String,
) -> Vec<String> {
    let mut hints = Vec::new();
    for find in &FORAGED {
        match colony.finds.get(find.id) {
            Some(record) => {
                let becomes = if finds::growing::growth(find.id).is_some() {
                    "Grows into"
                } else {
                    "Becomes"
                };
                paper::name(ui, find.name);
                paper::words(ui, find.blurb);
                paper::aside(
                    ui,
                    format!(
                        "{becomes} {} \u{b7} brought home {}\u{d7} \u{b7} first picked with {}",
                        find.piece.to_lowercase(),
                        record.count,
                        who(&record.first_by)
                    ),
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
                hints.push(format!("something {} at {place}{when}", find.tier.label()));
            }
        }
    }
    hints
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
