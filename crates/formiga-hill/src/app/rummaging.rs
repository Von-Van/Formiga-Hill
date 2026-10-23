//! The Woods in the window: choosing who comes along, the outing's controls, and keeping what
//! came home.

use super::{HillApp, paper, plural};
use crate::cast::Id;
use crate::character::Character;
use crate::finds;
use crate::playground::Playground;
use crate::woods::rummage::{self, BASKET, Ending, Event, Outset, Phase, Rummage};
use crate::woods::{self, Opener};
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Band;

/// How many companions can come along at once.
pub const PARTY: usize = 2;

/// What the person is going to the Woods to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Activity {
    #[default]
    Rummage,
    Fish,
    Bugs,
    Forage,
    Expedition,
    /// An adventure someone has laid out on a map of its own, walked as an expedition is.
    Adventure,
    /// Scavenging the heaps along the old track.
    Scavenge,
    /// Following a torn map along the old track.
    Treasure,
}

impl Activity {
    /// What it is called on the Woods' tray.
    pub fn label(self) -> &'static str {
        match self {
            Self::Rummage => "Rummage",
            Self::Fish => "Fish",
            Self::Bugs => "Catch bugs",
            Self::Forage => "Forage",
            Self::Expedition => "Expedition",
            Self::Adventure => "Adventure",
            Self::Scavenge => "Scavenge",
            Self::Treasure => "Follow a map",
        }
    }
}

#[derive(Default)]
pub struct Woods {
    /// Who the person has chosen to bring; the first leads, or fishes.
    pub party: Vec<Id>,
    pub activity: Activity,
    /// The glade and the pool with nobody there, between trips.
    pub empty: Option<Playground>,
    pub pool: Option<Playground>,
    pub outing: Option<(Playground, Rummage)>,
    pub fishing: Option<(Playground, crate::fishing::angling::Angling)>,
    /// The meadow with nobody there between hunts, and a hunt under way.
    pub meadow: Option<Playground>,
    pub hunt: Option<(Playground, crate::meadow::catching::Hunt)>,
    /// Whether the person is holding to creep up on a bug.
    pub creeping: bool,
    /// The hedgerow with nobody foraging, and a foray under way.
    pub hedgerow: Option<Playground>,
    pub foray: Option<(Playground, crate::hedgerow::foraging::Foray)>,
    /// An expedition under way, and the map as planned for whoever is chosen.
    pub expedition: Option<crate::expedition::Expedition>,
    pub plan: Option<(PlannedFor, Canvas)>,
    /// The adventure chosen to go on, by package and id, and the one under way, if the
    /// expedition is one.
    pub adventure: Option<(String, String)>,
    pub playing: Option<super::adventures::Playing>,
    /// An adventure being made on the map editor, what has been done to it, to undo, and the
    /// ones begun, by name and title, as last read from the drafts kept: read again only once
    /// one is kept or thrown away.
    pub making: Option<super::mapmaking::Making>,
    pub making_history: super::undo::History<crate::adventure::draft::Draft>,
    pub begun: Option<Vec<(String, String)>>,
    /// The old track with nobody there, a scavenge under way, and a map being followed, and
    /// which of the colony's maps is chosen to follow.
    pub track: Option<Playground>,
    pub scavenge: Option<(Playground, crate::track::scavenging::Scavenge)>,
    pub treasure: Option<(Playground, crate::track::treasure::Hunt)>,
    pub map: usize,
}

impl Woods {
    /// Every ground in the Woods that is up: the places with nobody there, whatever is under way,
    /// and the map and the stop of an expedition. Whatever reaches everyone wherever they are,
    /// the hour's light or what they have on, goes through here, so no ground is left out.
    pub fn grounds_mut(&mut self) -> impl Iterator<Item = &mut Playground> {
        // Every field named, so a new ground can't be missed here.
        let Self {
            party: _,
            activity: _,
            empty,
            pool,
            outing,
            fishing,
            meadow,
            hunt,
            creeping: _,
            hedgerow,
            foray,
            expedition,
            plan: _,
            adventure: _,
            playing: _,
            making: _,
            making_history: _,
            begun: _,
            track,
            scavenge,
            treasure,
            map: _,
        } = self;
        [
            empty.as_mut(),
            pool.as_mut(),
            meadow.as_mut(),
            hedgerow.as_mut(),
            track.as_mut(),
            outing.as_mut().map(|(ground, _)| ground),
            fishing.as_mut().map(|(ground, _)| ground),
            hunt.as_mut().map(|(ground, _)| ground),
            foray.as_mut().map(|(ground, _)| ground),
            scavenge.as_mut().map(|(ground, _)| ground),
            treasure.as_mut().map(|(ground, _)| ground),
        ]
        .into_iter()
        .flatten()
        .chain(
            expedition
                .iter_mut()
                .flat_map(crate::expedition::Expedition::grounds_mut),
        )
    }
}

/// What the planning map was drawn for, so it is drawn again whenever any of it changes: who is
/// coming, the far places the colony has been to, and whether anything on the Hilltop sees far.
#[derive(PartialEq)]
pub struct PlannedFor {
    /// The adventure planned for, by package and id; none for the Woods' own map.
    pub adventure: Option<(String, String)>,
    pub party: Vec<Id>,
    pub far_places: std::collections::BTreeSet<String>,
    pub far_sight: bool,
}

impl HillApp {
    /// Readies the glade the first time anyone goes there.
    pub(super) fn open_woods(&mut self, now: f32) {
        if self.woods.empty.is_none() {
            self.woods.empty = Some(woods::open(&self.arrival.cast, &[], now));
        }
        if self.woods.pool.is_none() {
            self.woods.pool = Some(crate::fishing::open(&self.arrival.cast, &[], now));
        }
        if self.woods.meadow.is_none() {
            let hilltop = self.hilltop_standing();
            self.woods.meadow = Some(crate::meadow::open(&self.arrival.cast, &[], now, &hilltop));
        }
        self.open_hedgerow(now);
        self.open_track(now);
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
            beckons: crate::clearing::sovereign::may_beckon(colony),
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
        if self.woods.expedition.is_some() {
            self.tick_expedition(now, holding, self.woods.creeping, dt);
            return;
        }
        if self.woods.fishing.is_some() {
            self.tick_fishing(now, holding);
            return;
        }
        if self.woods.hunt.is_some() {
            self.tick_bug_hunt(now, self.woods.creeping);
            return;
        }
        if self.woods.foray.is_some() {
            self.tick_foraging(now);
            return;
        }
        if self.woods.scavenge.is_some() {
            self.tick_scavenging(now);
            return;
        }
        if self.woods.treasure.is_some() {
            self.tick_treasure(now);
            return;
        }
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
        let events = rummage.take_events();
        let place = |spot: usize| rummage.spot(spot).map_or("somewhere", |spot| spot.name);
        for event in events {
            if let Some(cue) = super::sound::rummaged(&event) {
                self.sound.play(cue);
            }
            let line = match event {
                Event::Opened { who, spot } => match rummage.opener_of(spot) {
                    Some(Opener::Explorer) => {
                        format!("{} has found a way to {}!", name(who), place(spot))
                    }
                    Some(Opener::LittleOne) => {
                        format!("{} is small enough for {}!", name(who), place(spot))
                    }
                    _ => format!("Together, these two could shift {}!", place(spot)),
                },
                Event::Noticed { who, spot } => {
                    format!("{} spotted something by {}.", name(who), place(spot))
                }
                Event::Got { find, .. } => {
                    let find = finds::find(find).map_or("something", |find| find.name);
                    format!("Got it! {find}.")
                }
                Event::Slipped { .. } => "It slipped away deeper. Try again!".to_owned(),
                Event::Nothing { spot } => format!("Nothing at {}.", place(spot)),
                Event::AlreadyLooked { spot } => {
                    format!("You've looked at {} already.", place(spot))
                }
                Event::Leaving(Ending::Dusk) => "The light's going. Time to head home.".to_owned(),
                Event::Leaving(Ending::Full) => "The basket's full! Home we go.".to_owned(),
                Event::Leaving(Ending::Chose) => "Heading home.".to_owned(),
                Event::Glint => {
                    "Something is glinting between the trees, far off. It doesn't look like \
                                 it belongs here."
                        .to_owned()
                }
                Event::Beckoned => {
                    "Through the trees, into a clearing that isn't on any map\u{2026}".to_owned()
                }
            };
            self.notice = Some((line, now));
        }
        match rummage.phase() {
            Phase::Over => self.finish_outing(now),
            Phase::Beckoned => self.follow_the_glint(now),
            _ => {}
        }
    }

    /// Brings the basket home: into the journal and the satchel, ready for the Hilltop.
    pub(super) fn finish_outing(&mut self, now: f32) {
        let Some((_, rummage)) = self.woods.outing.take() else {
            return;
        };
        let basket = rummage.basket().to_vec();
        let new = self.memories.back_from_the_woods(rummage.party(), &basket);
        // Now and then, rolled up in a hollow with whatever was found there: a torn map.
        let outings = self.memories.colony().outings;
        let map = !basket.is_empty()
            && crate::track::stray_map(outings, &self.arrival.cast.snapshot.colony_id)
            && rummage
                .party()
                .first()
                .is_some_and(|leader| self.memories.found_a_map(*leader));
        let mut line = match (basket.len(), new.len()) {
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
        if map {
            line.push_str(" And rolled up in a hollow with them: a torn map!");
        }
        self.notice = Some((line, now));
        self.grant(super::departures::Outing::Rummaging, rummage.party(), now);
    }

    pub(super) fn compose_woods(&mut self, now: f32) -> Canvas {
        if let Some(scene) = self.compose_making() {
            return scene;
        }
        if let Some(scene) = self.compose_expedition(now) {
            return scene;
        }
        if let Some(scene) = self.compose_pool(now) {
            return scene;
        }
        if let Some(scene) = self.compose_meadow(now) {
            return scene;
        }
        if let Some(scene) = self.compose_foray(now) {
            return scene;
        }
        if let Some(scene) = self.compose_scavenge(now) {
            return scene;
        }
        if let Some(scene) = self.compose_treasure(now) {
            return scene;
        }
        if matches!(self.woods.activity, Activity::Scavenge | Activity::Treasure)
            && self.woods.outing.is_none()
            && let Some(track) = &mut self.woods.track
        {
            return track.compose(now);
        }
        if self.woods.activity == Activity::Forage
            && self.woods.outing.is_none()
            && let Some(lane) = &mut self.woods.hedgerow
        {
            return lane.compose(now);
        }
        if self.woods.activity == Activity::Bugs
            && self.woods.outing.is_none()
            && let Some(meadow) = &mut self.woods.meadow
        {
            return meadow.compose(now);
        }
        if self.woods.activity == Activity::Fish
            && self.woods.outing.is_none()
            && let Some(pool) = &mut self.woods.pool
        {
            return pool.compose(now);
        }
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
        if self.woods.expedition.is_some() {
            self.expedition_click(pointer, now);
            return;
        }
        if self.woods.making.is_some() {
            self.making_click(pointer, now);
            return;
        }
        if self.woods.fishing.is_some() {
            self.fishing_click(pointer, now);
            return;
        }
        if self.woods.hunt.is_some() {
            self.bug_hunt_click(pointer, now);
            return;
        }
        if self.woods.foray.is_some() {
            self.foraging_click(pointer, now);
            return;
        }
        if self.woods.scavenge.is_some() {
            self.scavenging_click(pointer, now);
            return;
        }
        if self.woods.treasure.is_some() {
            self.treasure_click(pointer, now);
            return;
        }
        let Some((ground, rummage)) = &mut self.woods.outing else {
            return;
        };
        let spot = pointer.and_then(|(x, y)| rummage.spot_at(x, y));
        match rummage.phase() {
            Phase::Catching(catch) if spot.is_none_or(|spot| spot == catch.spot) => {
                if !ground.reduce_motion() {
                    rummage.strike(ground, now);
                }
            }
            Phase::Exploring | Phase::Catching(_) => {
                if let Some(spot) = spot {
                    rummage.choose(ground, spot, now);
                    self.sound.play(crate::audio::Cue::Rummage);
                }
            }
            _ => {}
        }
    }

    /// The key for catching, for anyone not using the pointer.
    pub(super) fn woods_strike(&mut self, now: f32) {
        self.expedition_strike(now);
        self.fishing_strike(now);
        self.bug_hunt_swing(now);
        if let Some((ground, rummage)) = &mut self.woods.outing
            && matches!(rummage.phase(), Phase::Catching(_))
            && !ground.reduce_motion()
        {
            rummage.strike(ground, now);
        }
    }

    /// What to call the spot under the pointer, and where, while choosing.
    pub(super) fn woods_hover(&self, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
        if self.woods.expedition.is_some() {
            return self.expedition_hover(pointer);
        }
        if self.woods.making.is_some() {
            return self.making_hover(pointer);
        }
        if self.woods.fishing.is_some() {
            return self.fishing_hover(pointer);
        }
        if self.woods.hunt.is_some() {
            return self.bug_hunt_hover(pointer);
        }
        if self.woods.foray.is_some() {
            return self.foraging_hover(pointer);
        }
        if self.woods.scavenge.is_some() {
            return self.scavenging_hover(pointer);
        }
        if self.woods.treasure.is_some() {
            return self.treasure_hover(pointer);
        }
        let (_, rummage) = self.woods.outing.as_ref()?;
        if !matches!(rummage.phase(), Phase::Exploring | Phase::Catching(_)) {
            return None;
        }
        let spot = pointer.and_then(|(x, y)| rummage.spot_at(x, y))?;
        let place = *rummage.spot(spot)?;
        let label = if rummage.searched(spot) {
            format!("{} (looked)", place.name)
        } else {
            format!("{} \u{b7} {}", place.name, place.kind.verb())
        };
        Some((label, (place.sign.0, place.sign.1 - 12.0)))
    }

    pub(super) fn woods_tray(&mut self, ui: &mut egui::Ui, now: f32) {
        let mut set_off = false;
        let mut home = false;
        if self.woods.expedition.is_some() {
            self.expedition_tray(ui, now);
            return;
        }
        if self.woods.making.is_some() {
            self.making_tray(ui, now);
            return;
        }
        if self.woods.fishing.is_some() {
            self.fishing_tray(ui, now);
            return;
        }
        if self.woods.hunt.is_some() {
            self.bug_hunt_tray(ui, now);
            return;
        }
        if self.woods.foray.is_some() {
            self.foraging_tray(ui, now);
            return;
        }
        if self.woods.scavenge.is_some() {
            self.scavenging_tray(ui, now);
            return;
        }
        if self.woods.treasure.is_some() {
            self.treasure_tray(ui, now);
            return;
        }
        match &self.woods.outing {
            None => {
                let doing = paper::Button::new(format!("{} \u{25b4}", self.woods.activity.label()));
                paper::menu(ui, doing, false, |ui| {
                    for activity in [
                        Activity::Rummage,
                        Activity::Fish,
                        Activity::Bugs,
                        Activity::Forage,
                        Activity::Expedition,
                        Activity::Adventure,
                    ] {
                        paper::choice(ui, &mut self.woods.activity, activity, activity.label());
                    }
                    self.track_activities(ui);
                });
                self.map_choice(ui);
                if self.woods.activity == Activity::Adventure {
                    self.adventure_choice(ui);
                    self.making_choice(ui);
                    let bring = ui.add(paper::Button::new("Open a shared one\u{2026}"));
                    if paper::explain(ui, bring, "Look over an adventure someone shared, first")
                        .clicked()
                    {
                        self.choose_shared_file(now);
                    }
                }
                // Only an expedition, or an adventure, takes three.
                let most = if matches!(
                    self.woods.activity,
                    Activity::Expedition | Activity::Adventure
                ) {
                    crate::expedition::PARTY
                } else {
                    PARTY
                };
                self.woods.party.truncate(most);
                paper::slip(ui, "Who's coming?");
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
                    let hint = match self.woods.activity {
                        Activity::Rummage => format!(
                            "Good at {}. {} outing{} so far.",
                            rummage::best_at(&character).knack(),
                            outings,
                            plural(outings as usize)
                        ),
                        Activity::Fish => angler(&character).to_owned(),
                        Activity::Bugs => crate::meadow::catching::netter(&character).to_owned(),
                        Activity::Forage => {
                            super::foraging::forager_hint(colony, member.id, &character)
                        }
                        Activity::Expedition | Activity::Adventure => {
                            super::expedition::expeditioner_hint(colony, member.id, &character)
                        }
                        Activity::Scavenge => {
                            super::scavenging::scavenger_hint(colony, member.id, &character)
                        }
                        Activity::Treasure => super::scavenging::reader_hint(&character),
                    };
                    let pick = ui.add(paper::Button::new(&member.name).selected(chosen));
                    if paper::explain(ui, pick, &hint).clicked() {
                        if chosen {
                            self.woods.party.retain(|id| *id != member.id);
                        } else if self.woods.party.len() < most {
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
                    .add(paper::Button::new("Set off").enabled(ready))
                    .clicked();
            }
            Some((ground, rummage)) => {
                paper::meter(ui, rummage.light_left(), "light");
                paper::slip(ui, format!("Basket {}/{BASKET}", rummage.basket().len()));
                let hint = match rummage.phase() {
                    Phase::Catching(_) if ground.reduce_motion() => {
                        "Hold to turn the marker; let go on the gold."
                    }
                    Phase::Catching(_) => "Click (or Space) as the marker crosses the gold!",
                    _ => "",
                };
                if !hint.is_empty() {
                    paper::hint(ui, hint);
                }
                let leaving = matches!(rummage.phase(), Phase::Leaving { .. } | Phase::Over);
                home = ui
                    .add(paper::Button::new("Head home").enabled(!leaving))
                    .clicked();
            }
        }
        if set_off {
            match self.woods.activity {
                Activity::Rummage => self.set_off(now),
                Activity::Fish => self.set_off_fishing(now),
                Activity::Bugs => self.set_off_bug_hunting(now),
                Activity::Forage => self.set_off_foraging(now),
                Activity::Expedition => self.set_off_expedition(now),
                Activity::Adventure => self.set_off_adventure(now),
                Activity::Scavenge => self.set_off_scavenging(now),
                Activity::Treasure => self.set_off_treasure(now),
            }
        }
        if home && let Some((ground, rummage)) = &mut self.woods.outing {
            rummage.head_home(ground, now);
        }
    }
}

/// What sort of angler a companion makes, for the person choosing who fishes (the first chosen).
fn angler(character: &Character) -> &'static str {
    let a = character.axes;
    let patience = 1.0 - a.impulsiveness;
    let strength = (a.energy + a.feistiness) / 2.0;
    if character.kind == formiga_core::TemperamentKind::Lazybones {
        "Likely to nod off with the rod, and the fish seem to like that."
    } else if patience >= a.playfulness && patience >= strength {
        "Patient with a rod: a longer moment to strike, and a steady cast."
    } else if a.playfulness >= strength {
        "Playful at the water's edge: fish come to its float readily."
    } else {
        "Strong on the line: hard to snap."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::hedgerow::foraging::{self, Foray};
    use crate::track::scavenging::{self, Scavenge};
    use crate::woods::Influence;

    #[test]
    fn every_ground_in_the_woods_is_reached_foraging_and_scavenging_too() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let party: Vec<Id> = cast.members.iter().take(PARTY).map(|m| m.id).collect();
        let hilltop = crate::hilltop::Arrangement::new();
        let mut woods = Woods {
            empty: Some(woods::open(&cast, &[], 0.0)),
            hedgerow: Some(crate::hedgerow::open(&cast, &[], 0.0, &hilltop)),
            ..Woods::default()
        };
        let mut lane = crate::hedgerow::open(&cast, &party, 0.0, &hilltop);
        let outset = foraging::Outset {
            party: party.clone(),
            drought: 0,
            close_pair: false,
            influence: Influence::default(),
            seed: 1,
        };
        let foray = Foray::new(&mut lane, outset, |_| false, 0.0);
        woods.foray = Some((lane, foray));
        let mut track = crate::track::open(&cast, &party, 0.0, &hilltop);
        let outset = scavenging::Outset {
            party,
            drought: 0,
            close_pair: false,
            influence: Influence::default(),
            maps_held: 0,
            seed: 1,
        };
        let scavenge = Scavenge::new(&mut track, outset, |_| false, 0.0);
        woods.scavenge = Some((track, scavenge));
        let reached: Vec<*const Playground> = woods
            .grounds_mut()
            .map(|ground| ground as *const Playground)
            .collect();
        assert_eq!(reached.len(), 4);
        let foraging = &woods.foray.as_ref().unwrap().0 as *const Playground;
        let scavenging = &woods.scavenge.as_ref().unwrap().0 as *const Playground;
        assert!(reached.contains(&foraging), "the foray is left out");
        assert!(reached.contains(&scavenging), "the scavenge is left out");
    }
}
