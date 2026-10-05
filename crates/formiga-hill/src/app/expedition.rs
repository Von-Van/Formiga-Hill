//! Expeditions in the window: planning one on the map, choosing the way, the stops along it,
//! making room in the basket, and the day's page in the journal.

use super::HillApp;
use crate::cast::Id;
use crate::character::Character;
use crate::expedition::legs::{Input, LegEvent, Play};
use crate::expedition::map::{self, Opener, PLACES, Stop};
use crate::expedition::{self, BASKET, Droughts, Ending, Event, Expedition, Known, Outset, Phase};
use crate::finds::{self, AFAR, Tier};
use crate::memories::{ColonyMemories, Homecoming};
use crate::playground::distance;
use crate::{falls, fishing, hedgerow, meadow, track, woods};
use eframe::egui;
use formiga_art::Canvas;
use std::collections::BTreeSet;

impl HillApp {
    /// What the colony knows, for an expedition setting off or being planned.
    fn known(&self) -> Known {
        let colony = self.memories.colony();
        Known {
            finds: colony.finds.keys().cloned().collect(),
            fish: colony.fish.keys().cloned().collect(),
            bugs: colony.bugs.keys().cloned().collect(),
            droughts: Droughts {
                rummage: colony.drought,
                fish: colony.fish_drought,
                bugs: colony.bug_drought,
                forage: colony.forage_drought,
                scavenge: colony.scavenge_drought,
                far: colony.far_drought,
            },
            hilltop: self.hilltop_standing(),
            far_places: colony.far_places.clone(),
            maps_held: colony.maps.len(),
        }
    }

    /// Sets off on an expedition with whoever was chosen, up to three; the first leads the way.
    pub(super) fn set_off_expedition(&mut self, now: f32) {
        let party: Vec<Id> = self
            .woods
            .party
            .iter()
            .copied()
            .take(expedition::PARTY)
            .collect();
        if party.is_empty() {
            return;
        }
        let known = self.known();
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let seed = cast
            .snapshot
            .colony_id
            .bytes()
            .fold(u64::from(colony.expeditions) << 32 | 0xe4b, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        let influence = woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            influence,
            seed,
        };
        let mut expedition = Expedition::new(cast, outset, known, now);
        expedition.set_daylight(self.daylight);
        self.woods.expedition = Some(expedition);
        let line = note.unwrap_or(
            "Choose the way at each fork: click a place on the map to go there, and click where \
             you are to stop.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Plays the expedition on.
    pub(super) fn tick_expedition(&mut self, now: f32, holding: bool, creeping: bool, dt: f32) {
        let cast = &self.arrival.cast;
        let Some(expedition) = &mut self.woods.expedition else {
            return;
        };
        let input = Input {
            holding,
            creeping,
            dt,
        };
        expedition.tick(cast, input, now);
        for event in expedition.take_events() {
            if let Some(cue) = super::sound::expedition_heard(&event) {
                self.sound.play(cue);
            }
            if let Some(line) = tell(expedition, cast, event) {
                self.notice = Some((line, now));
            }
        }
        if expedition.phase() == Phase::Over {
            self.finish_expedition(now);
        }
    }

    /// Brings the expedition home: everything in the basket into the journal and the satchel,
    /// fish and bugs into the journal, any torn maps kept for another day, and the day's page.
    pub(super) fn finish_expedition(&mut self, now: f32) {
        let Some(mut expedition) = self.woods.expedition.take() else {
            return;
        };
        // Leaving the Woods partway through a stop still brings its basket home.
        expedition.close_leg(now);
        let route: Vec<&str> = expedition
            .route()
            .iter()
            .map(|place| PLACES[*place].id)
            .collect();
        let stops: Vec<&str> = expedition
            .stops()
            .iter()
            .map(|place| PLACES[*place].id)
            .collect();
        let far: Vec<&str> = expedition
            .route()
            .iter()
            .filter(|place| PLACES[**place].hidden)
            .map(|place| PLACES[*place].id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let furthest = PLACES[expedition.furthest()];
        let found: Vec<(&str, u64)> = expedition
            .basket()
            .iter()
            .map(|carried| (carried.find, carried.by))
            .collect();
        let was = self
            .memories
            .colony()
            .furthest
            .as_ref()
            .map_or(0, |furthest| furthest.depth);
        let home = Homecoming {
            party: expedition.party(),
            route: &route,
            stops: &stops,
            furthest: (furthest.id, furthest.depth),
            far: &far,
            found: &found,
            fish: expedition.fish(),
            bugs: expedition.bugs(),
            maps: expedition.maps(),
        };
        let new = self.memories.back_from_the_expedition(&home);
        let count = found.len();
        let mut line = match (count, new.len()) {
            (0, _) => "Home from the expedition, the basket empty this time.".to_owned(),
            (count, 0) => format!(
                "Home from the expedition with {count} find{}. They're in the satchel, for the \
                 Hilltop.",
                plural(count)
            ),
            (count, fresh) => format!(
                "Home from the expedition with {count} find{}, {fresh} new to the journal. \
                 They're in the satchel, for the Hilltop.",
                plural(count)
            ),
        };
        let caught = expedition.fish().len() + expedition.bugs().len();
        if caught > 0 {
            line.push_str(&format!(" {caught} caught and let go on the way."));
        }
        match expedition.maps().len() {
            0 => {}
            1 => line.push_str(" And a torn map, to follow another day."),
            count => line.push_str(&format!(" And {count} torn maps, to follow another day.")),
        }
        if furthest.depth > was {
            line.push_str(&format!(" The furthest yet: {}!", furthest.name));
        }
        line.push_str(" The day's page is in the journal.");
        self.notice = Some((line, now));
    }

    /// The scene in the Woods while an expedition is under way, or being planned.
    pub(super) fn compose_expedition(&mut self, now: f32) -> Option<Canvas> {
        let pointer = self.pointer;
        if let Some(expedition) = &mut self.woods.expedition {
            return Some(expedition.compose(now, pointer));
        }
        if self.woods.activity != super::rummaging::Activity::Expedition {
            return None;
        }
        // Planning: the map, showing the ways the chosen company would open, and the far places
        // the colony has found or can see from the Hilltop.
        let now_for = super::rummaging::PlannedFor {
            party: self.woods.party.clone(),
            far_places: self.memories.colony().far_places.clone(),
            far_sight: expedition::far_sight(&self.hilltop_standing()),
        };
        let stale = self
            .woods
            .plan
            .as_ref()
            .is_none_or(|(planned_for, _)| *planned_for != now_for);
        if stale {
            let known = self.known();
            let picture = expedition::planned(&self.arrival.cast, &now_for.party, &known);
            self.woods.plan = Some((now_for, picture));
        }
        let (_, picture) = self.woods.plan.as_ref()?;
        let mut scene = picture.clone();
        crate::daylight::light(&mut scene, self.daylight.indoors(), picture, &[]);
        Some(scene)
    }

    /// A click on the map, or at a stop.
    pub(super) fn expedition_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let cast = &self.arrival.cast;
        let Some(expedition) = &mut self.woods.expedition else {
            return;
        };
        match expedition.phase() {
            Phase::Stopped => {
                if let Some(act) = expedition.leg_mut().and_then(|leg| leg.click(pointer, now)) {
                    self.sound.play(super::sound::acted(act));
                }
            }
            Phase::Choosing | Phase::MakingRoom => {
                let Some((x, y)) = pointer else {
                    return;
                };
                if let Some(slot) = expedition.basket_slot_at(x, y) {
                    expedition.leave_behind(slot, now);
                } else if let Some(place) = expedition.place_at(x, y) {
                    if place == expedition.at() {
                        expedition.stop(cast, now);
                    } else {
                        expedition.go(place, now);
                    }
                }
                for event in expedition.take_events() {
                    if let Some(cue) = super::sound::expedition_heard(&event) {
                        self.sound.play(cue);
                    }
                    if let Some(line) = tell(expedition, cast, event) {
                        self.notice = Some((line, now));
                    }
                }
            }
            _ => {}
        }
    }

    /// The key for trying, striking or swinging, at a stop.
    pub(super) fn expedition_strike(&mut self, now: f32) {
        if let Some(expedition) = &mut self.woods.expedition
            && let Some(leg) = expedition.leg_mut()
            && let Some(act) = leg.strike(now)
        {
            self.sound.play(super::sound::acted(act));
        }
    }

    /// The pointer over the expedition's scene: where the creatures look, and who is under it.
    pub(super) fn expedition_point(&mut self, pointer: Option<(f32, f32)>, now: f32) -> Option<Id> {
        let expedition = self.woods.expedition.as_mut()?;
        let ground = expedition.ground_mut();
        ground.set_pointer(pointer);
        pointer.and_then(|(x, y)| ground.actor_at(x, y, now))
    }

    /// What to call whatever is under the pointer, and where.
    pub(super) fn expedition_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let expedition = self.woods.expedition.as_ref()?;
        let (x, y) = pointer?;
        match expedition.phase() {
            Phase::Stopped => leg_hover(expedition, (x, y)),
            Phase::Choosing | Phase::MakingRoom => {
                if let Some(slot) = expedition.basket_slot_at(x, y) {
                    let find = expedition.basket()[slot].find;
                    let name = finds::find(find).map_or("it", |find| find.name);
                    return Some((format!("Leave {} behind", lower(name)), (x, y + 14.0)));
                }
                let place = expedition.place_at(x, y)?;
                let here = &PLACES[place];
                let label = if place == expedition.at() {
                    if expedition.can_stop() {
                        format!("{} \u{b7} {}", here.name, verb(here.stop))
                    } else if here.stop.played() {
                        format!("{} \u{b7} been here today", here.name)
                    } else {
                        here.name.to_owned()
                    }
                } else {
                    let name = if expedition.sighted(place) {
                        here.name
                    } else {
                        "somewhere in the mist"
                    };
                    match expedition.way_to(place) {
                        Some(way) if way.cost > expedition.light() + 0.01 => {
                            format!("{name} \u{b7} too far for the light left")
                        }
                        Some(_) if !expedition.sighted(place) => {
                            format!("{name} \u{b7} who knows how far?")
                        }
                        Some(way) => format!(
                            "{name} \u{b7} {} of the day",
                            share(way.cost, expedition.full())
                        ),
                        None => format!("{name} \u{b7} not from here"),
                    }
                };
                let (px, py) = here.at;
                Some((label, (px, py - 34.0)))
            }
            _ => None,
        }
    }

    pub(super) fn expedition_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let cast = &self.arrival.cast;
        let Some(expedition) = &mut self.woods.expedition else {
            return;
        };
        ui.add(
            egui::ProgressBar::new(expedition.light_left())
                .desired_width(70.0)
                .text("day"),
        );
        ui.label(format!("Basket {}/{BASKET}", expedition.basket().len()));
        let mut stop = false;
        let mut move_on = false;
        let reduce_motion = cast.reduce_motion();
        match expedition.phase() {
            Phase::Arriving { .. } => {
                ui.label("Onto the map\u{2026}");
            }
            Phase::Choosing => {
                if expedition.can_stop() {
                    let here = PLACES[expedition.at()].stop;
                    stop = ui.button(capital(verb(here))).clicked();
                }
                ui.label(egui::RichText::new("Click a place on the map to go there.").strong());
            }
            Phase::Walking { to, .. } => {
                let name = if expedition.sighted(to) {
                    PLACES[to].name
                } else {
                    "somewhere in the mist"
                };
                ui.label(format!("On the way to {name}\u{2026}"));
            }
            Phase::Stopped => {
                if let Some(leg) = expedition.leg_mut() {
                    if let Play::Scavenge { scavenge, .. } = &mut leg.play {
                        super::scavenging::hands_bar(ui, cast, scavenge);
                    }
                    let hint = leg_hint(&leg.play, &leg.ground, reduce_motion);
                    if !hint.is_empty() {
                        ui.label(egui::RichText::new(hint).strong());
                    }
                    move_on = ui
                        .add_enabled(!leg.leaving(), egui::Button::new("Move on"))
                        .clicked();
                }
            }
            Phase::MakingRoom => {
                ui.label(
                    egui::RichText::new(
                        "Too much to carry: click something in the basket to leave it behind.",
                    )
                    .strong(),
                );
            }
            Phase::Homeward { .. } | Phase::Over => {
                ui.label("Heading home\u{2026}");
            }
        }
        let going = matches!(expedition.phase(), Phase::Homeward { .. } | Phase::Over);
        let home = ui
            .add_enabled(!going, egui::Button::new("Head home"))
            .clicked();
        let Some(expedition) = &mut self.woods.expedition else {
            return;
        };
        if stop {
            expedition.stop(cast, now);
        }
        if move_on {
            expedition.move_on(now);
        }
        if home {
            expedition.head_home(now);
        }
        for event in expedition.take_events() {
            if let Some(cue) = super::sound::expedition_heard(&event) {
                self.sound.play(cue);
            }
            if let Some(line) = tell(expedition, cast, event) {
                self.notice = Some((line, now));
            }
        }
    }
}

/// What a stop is for, as a button or a label: "Rummage here".
fn verb(stop: Stop) -> &'static str {
    match stop {
        Stop::Rummage => "rummage here",
        Stop::Fish => "fish here",
        Stop::Bugs => "catch a bug here",
        Stop::Forage => "forage here",
        Stop::Scavenge => "scavenge here",
        Stop::Rest => "rest on the log",
        Stop::Falls => "wade in at the falls",
        Stop::Edge | Stop::Fork => "",
    }
}

/// How much of the day something takes, as the person reads it.
fn share(cost: f32, full: f32) -> &'static str {
    match cost / full.max(1.0) {
        part if part < 0.045 => "a little",
        part if part < 0.08 => "a while",
        _ => "a good part",
    }
}

/// The hint for the stop under way, as its own bar gives it.
fn leg_hint(
    play: &Play,
    ground: &crate::playground::Playground,
    reduce_motion: bool,
) -> &'static str {
    match play {
        Play::Rummage { rummage, .. } => match rummage.phase() {
            woods::rummage::Phase::Catching(_) if reduce_motion => {
                "Hold to turn the marker; let go on the gold."
            }
            woods::rummage::Phase::Catching(_) => {
                "Click (or Space) as the marker crosses the gold!"
            }
            woods::rummage::Phase::Exploring => "Choose somewhere to search. Watch for signs.",
            _ => "",
        },
        Play::Fish { angling, casts } => match angling.phase() {
            fishing::angling::Phase::Ready if *casts < expedition::legs::CASTS => {
                "Click the water to cast."
            }
            fishing::angling::Phase::Waiting { .. } => {
                "Strike (click or Space) when the float goes under."
            }
            fishing::angling::Phase::Fighting(fight) if fight.strain > 0.7 => "Ease off!",
            fishing::angling::Phase::Fighting(_) => "Hold to reel in.",
            _ => "",
        },
        Play::Bugs { hunt, .. } => match (hunt.phase(), hunt.target()) {
            (meadow::catching::Phase::Hunting, None) => "Click a bug to go after it.",
            (meadow::catching::Phase::Hunting, Some(_)) if hunt.in_reach(ground) => {
                "In reach! Space to swing."
            }
            (meadow::catching::Phase::Hunting, Some(_)) => "Hold to creep closer.",
            _ => "",
        },
        Play::Forage { foray } => match foray.phase() {
            hedgerow::foraging::Phase::Choosing | hedgerow::foraging::Phase::Going { .. } => {
                "Click a bush to go to it: one patch, then on."
            }
            hedgerow::foraging::Phase::At { .. } if foray.basket().len() >= BASKET => {
                "Full! Click a basket slot to put something back."
            }
            hedgerow::foraging::Phase::At { .. } => "Pick what's ripe, then move on.",
            _ => "",
        },
        Play::Scavenge { scavenge, heaps } => match scavenge.phase() {
            track::scavenging::Phase::Choosing | track::scavenging::Phase::Going { .. } => {
                "Click a heap to go to it: a heap or two, then on."
            }
            track::scavenging::Phase::At { .. } if scavenge.basket().len() >= BASKET => {
                "Full! Click a basket slot to put something back."
            }
            track::scavenging::Phase::At { .. } if heaps.len() >= expedition::legs::HEAPS => {
                "Click something in the heap, then move on."
            }
            track::scavenging::Phase::At { .. } => "Click something in the heap.",
            _ => "",
        },
        Play::Rest { .. } => "A rest on the fallen log.",
        Play::Falls { wading } => match wading.phase() {
            falls::wading::Phase::Circling(_) if reduce_motion => {
                "Hold to bring it round; let go at the gold."
            }
            falls::wading::Phase::Circling(_) => "Click (or Space) as it comes past the stone!",
            falls::wading::Phase::Watching { .. } | falls::wading::Phase::Glinting { .. } => {
                "Watch the top of the falls."
            }
            _ => "",
        },
    }
}

/// What is under the pointer at a stop, as its own place names it.
fn leg_hover(expedition: &Expedition, (x, y): (f32, f32)) -> Option<(String, (f32, f32))> {
    let leg = expedition.leg()?;
    match &leg.play {
        Play::Rummage { rummage, .. } => {
            if !matches!(
                rummage.phase(),
                woods::rummage::Phase::Exploring | woods::rummage::Phase::Catching(_)
            ) {
                return None;
            }
            let spot = rummage.spot_at(x, y)?;
            let place = *rummage.spot(spot)?;
            let label = if rummage.searched(spot) {
                format!("{} (looked)", place.name)
            } else {
                format!("{} \u{b7} {}", place.name, place.kind.verb())
            };
            Some((label, (place.sign.0, place.sign.1 - 12.0)))
        }
        Play::Fish { angling, .. } => {
            if angling.phase() != fishing::angling::Phase::Ready {
                return None;
            }
            let haunt = fishing::angling::Angling::haunt_at(x, y)?;
            Some((haunt.name().to_owned(), (x, y - 10.0)))
        }
        Play::Bugs { hunt, .. } => {
            if hunt.phase() != meadow::catching::Phase::Hunting {
                return None;
            }
            let (name, at) = hunt.name_at(x, y)?;
            Some((name.to_owned(), (at.0, at.1 - 10.0)))
        }
        Play::Forage { foray } => {
            if matches!(
                foray.phase(),
                hedgerow::foraging::Phase::Leaving { .. } | hedgerow::foraging::Phase::Over
            ) {
                return None;
            }
            if let Some(slot) = foray.basket_slot_at(x, y, super::SCENE_WIDTH) {
                let find = foray.basket()[slot];
                let name = finds::find(find).map_or("it", |find| find.name);
                return Some((format!("Put back {}", lower(name)), (x, y + 14.0)));
            }
            if let Some(item) = foray.item_at(x, y)
                && let Some((plant, patch, at)) = foray.item(item)
            {
                let label = if foray.at() == Some(patch) {
                    plant.produce().to_owned()
                } else {
                    format!(
                        "{} \u{b7} {}",
                        hedgerow::PATCHES[patch].name,
                        plant.produce()
                    )
                };
                return Some((label, (at.0, at.1 - 8.0)));
            }
            let patch = foray.patch_at(x, y)?;
            let place = &hedgerow::PATCHES[patch];
            (foray.at().is_none())
                .then(|| (place.name.to_owned(), (place.stand.0, place.stand.1 - 40.0)))
        }
        Play::Scavenge { scavenge, heaps } => {
            let may_go = heaps.len() < expedition::legs::HEAPS;
            super::scavenging::heap_hover(scavenge, Some((x, y)), may_go)
        }
        Play::Rest { .. } => None,
        Play::Falls { .. } => {
            let (fx, fy) = falls::FOOT;
            (distance((x, y), (fx, fy)) < 24.0)
                .then(|| ("the Far Falls".to_owned(), (fx, fy - 30.0)))
        }
    }
}

/// What to tell the person of something that happened, if anything.
fn tell(expedition: &Expedition, cast: &crate::cast::Cast, event: Event) -> Option<String> {
    let name = |id: Id| {
        cast.member(id)
            .map_or("Someone", |member| member.name.as_str())
            .to_owned()
    };
    let place = |index: usize| PLACES[index].name;
    let find_name = |id: &str| finds::find(id).map_or("something", |find| find.name);
    Some(match event {
        Event::Opened { opener, who } => match opener {
            Opener::Explorer => format!("{} knows a deer track through the Woods.", name(who)),
            Opener::LittleOne => {
                format!("{} is small enough for the gap in the hedge.", name(who))
            }
            Opener::ClosePair => {
                let other = expedition
                    .company()
                    .close_pair
                    .map(|(a, b)| if a == who { b } else { a });
                match other {
                    Some(other) => format!(
                        "{} and {} can cross the stepping stones together.",
                        name(who),
                        name(other)
                    ),
                    None => "Together, these two can cross the stepping stones.".to_owned(),
                }
            }
            Opener::Bold => format!("{} will take the steep way up the crag.", name(who)),
            Opener::Reader => format!("{} reads old signs.", name(who)),
        },
        Event::Spied { place: far } => format!(
            "From the telescope on the Hilltop you can see {}, far off in the Woods. It's on the map.",
            place(far)
        ),
        Event::Read { who, place: to } => format!(
            "{} reads the old signpost: \u{201c}The old way, to {}.\u{201d}",
            name(who),
            place(to)
        ),
        Event::Arrived { place: here } => {
            let stop = PLACES[here].stop;
            if stop.played() && !expedition.stops().contains(&here) {
                format!(
                    "{}. Click it to {}, or choose the way on.",
                    capital(place(here)),
                    verb(stop)
                )
            } else {
                format!("{}.", capital(place(here)))
            }
        }
        Event::Found { place: far } => format!(
            "Out of the mist: {}! Nobody has come this far before.",
            place(far)
        ),
        Event::TooFar { place: there } => {
            format!(
                "There isn't enough of the day left to get to {}.",
                place(there)
            )
        }
        Event::Began {
            place: here,
            leader,
        } => match PLACES[here].stop {
            Stop::Rummage => format!("{} leads the way into the glade.", name(leader)),
            Stop::Fish => format!("{} takes the rod. A cast or two, then on.", name(leader)),
            Stop::Bugs => format!("{} has the net. One bug, then on.", name(leader)),
            Stop::Forage => format!(
                "{} leads along the hedgerow. One patch, then on.",
                name(leader)
            ),
            Stop::Scavenge => format!(
                "{} leads the way up the old track. A heap or two, then on.",
                name(leader)
            ),
            Stop::Rest => "Everyone sits along the fallen log.".to_owned(),
            Stop::Falls => format!("{} wades in at the foot of the falls.", name(leader)),
            Stop::Edge | Stop::Fork => return None,
        },
        Event::Leg(event) => return tell_leg(expedition, cast, &name, event),
        Event::Ended {
            place: here,
            found,
            caught,
        } => {
            let mut line = format!("On from {}", place(here));
            match (found, caught) {
                (0, 0) => line.push('.'),
                (found, 0) => line.push_str(&format!(" with {found} find{}.", plural(found))),
                (0, caught) => line.push_str(&format!(", {caught} caught and let go.")),
                (found, caught) => line.push_str(&format!(
                    " with {found} find{}, and {caught} caught and let go.",
                    plural(found)
                )),
            }
            line
        }
        Event::TooFull => {
            "Too much to carry! Click something in the basket to leave it behind.".to_owned()
        }
        Event::Left { find } => format!("Left behind: {}.", lower(find_name(find))),
        Event::Dusk => "The day's light is going. Time to head home.".to_owned(),
        Event::Homeward(Ending::Dusk) => {
            "Home through the dusk, with everything in the basket.".to_owned()
        }
        Event::Homeward(Ending::Chose) => "Heading home, with everything in the basket.".to_owned(),
    })
}

/// What to tell the person of something at a stop, as the activity there tells it.
fn tell_leg(
    expedition: &Expedition,
    cast: &crate::cast::Cast,
    name: &dyn Fn(Id) -> String,
    event: LegEvent,
) -> Option<String> {
    let find_name = |id: &str| finds::find(id).map_or("something", |find| find.name);
    let leg = expedition.leg();
    Some(match event {
        LegEvent::Rummage(event) => {
            use woods::rummage::{Ending, Event};
            let spot_name = |spot: usize| match leg.map(|leg| &leg.play) {
                Some(Play::Rummage { rummage, .. }) => {
                    rummage.spot(spot).map_or("somewhere", |spot| spot.name)
                }
                _ => "somewhere",
            };
            match event {
                Event::Opened { who, spot } => {
                    let opener = match leg.map(|leg| &leg.play) {
                        Some(Play::Rummage { rummage, .. }) => rummage.opener_of(spot),
                        _ => None,
                    };
                    match opener {
                        Some(woods::Opener::Explorer) => {
                            format!("{} has found a way to {}!", name(who), spot_name(spot))
                        }
                        Some(woods::Opener::LittleOne) => {
                            format!("{} is small enough for {}!", name(who), spot_name(spot))
                        }
                        _ => format!("Together, these two could shift {}!", spot_name(spot)),
                    }
                }
                Event::Noticed { who, spot } => {
                    format!("{} spotted something by {}.", name(who), spot_name(spot))
                }
                Event::Got { find, .. } => format!("Got it! {}.", find_name(find)),
                Event::Slipped { .. } => "It slipped away deeper. Try again!".to_owned(),
                Event::Nothing { spot } => format!("Nothing at {}.", spot_name(spot)),
                Event::AlreadyLooked { spot } => {
                    format!("You've looked at {} already.", spot_name(spot))
                }
                Event::Leaving(Ending::Full) => "The basket's full! On we go.".to_owned(),
                Event::Leaving(Ending::Dusk) => "The light's going.".to_owned(),
                Event::Leaving(Ending::Chose) => "That's the glade for today. On we go.".to_owned(),
                Event::Glint | Event::Beckoned => return None,
            }
        }
        LegEvent::Fish(event) => {
            use fishing::angling::Event;
            let kind = |id: &str| fishing::fish::fish(id).map_or("a fish", |fish| fish.name);
            match event {
                Event::Spooked { fish } => format!("{} darted off at the splash.", kind(fish)),
                Event::Nibble => "A nibble\u{2026}".to_owned(),
                Event::Bite => "It's under! Strike!".to_owned(),
                Event::TooSoon => "Too soon! That was only a nibble.".to_owned(),
                Event::Stolen => "Too late: it took the bait and went.".to_owned(),
                Event::Hooked { .. } => {
                    "Hooked! Hold to reel in, and ease off when it pulls.".to_owned()
                }
                Event::Snapped => "Snap! It got away.".to_owned(),
                Event::SlippedOff => "In the dusk, it slipped quietly off the hook.".to_owned(),
                Event::Landed { fish, length } => {
                    format!("{}, {length:.1} cm! Admired, and let go.", kind(fish))
                }
                Event::Snagged => "Something snagged on the line!".to_owned(),
                Event::Pointed { who, haunt } => {
                    format!("{} spotted a big one by {}.", name(who), haunt.name())
                }
                Event::Dusk => {
                    "The light is going. Something stirs in the deep water\u{2026}".to_owned()
                }
                Event::Leaving => "On from the pool.".to_owned(),
            }
        }
        LegEvent::Bugs(event) => {
            use meadow::catching::{Event, Why};
            let kind = |id: &str| meadow::bugs::bug(id).map_or("A bug", |bug| bug.name);
            match event {
                Event::Stalking { bug } => format!(
                    "After {}. Hold to creep closer; let go to keep still.",
                    lower(kind(bug))
                ),
                Event::Startled { bug } => format!("{} saw you coming, and was off!", kind(bug)),
                Event::Missed { bug, why } => match why {
                    Why::InFlight => format!("Missed: {} was on the move.", lower(kind(bug))),
                    Why::WingsOpen => format!("{} saw the net coming.", kind(bug)),
                    Why::Chirping => format!("Too late: {} jumped.", lower(kind(bug))),
                    Why::Dark => format!("{} went dark just as the net came down.", kind(bug)),
                },
                Event::OutOfReach => "Not close enough yet.".to_owned(),
                Event::Caught { bug, size } => format!(
                    "Caught! {}, {size:.0} mm across. Admired, and let go.",
                    kind(bug)
                ),
                Event::Netted { find } => {
                    format!(
                        "Not the bug, but something else in the net: {}!",
                        find_name(find)
                    )
                }
                Event::LandedOn { who, bug } => format!(
                    "{} has landed on {}, who is keeping very still.",
                    kind(bug),
                    name(who)
                ),
                Event::Pointed { who, bug } => {
                    format!("{} has spotted {}!", name(who), lower(kind(bug)))
                }
                Event::Dusk => {
                    "The light is going. Something pale is stirring in the brambles\u{2026}"
                        .to_owned()
                }
                Event::Leaving => "On from the meadow.".to_owned(),
            }
        }
        LegEvent::Forage(event) => {
            use hedgerow::foraging::{Ending, Event};
            let what = |item: usize| match leg.map(|leg| &leg.play) {
                Some(Play::Forage { foray }) => foray
                    .item(item)
                    .map_or("something", |(plant, _, _)| plant.produce()),
                _ => "something",
            };
            match event {
                Event::Opened { .. } | Event::Dropped { .. } | Event::Ripening { .. } => {
                    return None;
                }
                Event::Spotted { who, item } => {
                    format!("{} has spotted {} under the leaves!", name(who), what(item))
                }
                Event::Uncovered { item } => {
                    format!("There's {} hiding under the leaves here.", what(item))
                }
                Event::Picked { find, .. } => format!("Picked! {}.", find_name(find)),
                Event::Saved { who, .. } => format!(
                    "Just past its best, but {} picked it so gently it came whole.",
                    name(who)
                ),
                Event::NotYet { item } => {
                    format!("Not ripe yet: the {} stays on for now.", what(item))
                }
                Event::LookedOver { who, .. } => {
                    format!("{} looked it over and put it back: not yet.", name(who))
                }
                Event::Spoilt { item } => format!(
                    "Too late: the {} was past its best, and fell to bits.",
                    what(item)
                ),
                Event::Ate { who, .. } => format!("{} ate that one. There were plenty.", name(who)),
                Event::BasketFull => {
                    "The basket's full. Put something back to make room, or move on.".to_owned()
                }
                Event::PutBack { find } => {
                    format!("Put back: {}. Left for the birds.", find_name(find))
                }
                Event::ForTheBirds { who, .. } => {
                    format!("{} leaves the ripe ones for the birds.", name(who))
                }
                Event::Thanked { who, item } => format!(
                    "A robin has brought {} {} for leaving them so much!",
                    name(who),
                    what(item)
                ),
                Event::Dusk => "The light is going along the hedgerow\u{2026}".to_owned(),
                Event::Leaving(Ending::Dusk) => "The light's gone.".to_owned(),
                Event::Leaving(Ending::Chose) => "On from the hedgerow.".to_owned(),
            }
        }
        LegEvent::Scavenge(event) => {
            use track::scavenging::{Ending, Event};
            match event {
                Event::BasketFull => {
                    "The basket's full. Put something back to make room, or move on.".to_owned()
                }
                Event::Leaving(Ending::Dusk) => "The light's gone.".to_owned(),
                Event::Leaving(Ending::Chose) => "On from the old track.".to_owned(),
                // Anything else as the old track's own outings tell it.
                event => return super::scavenging::heap_line(cast, event),
            }
        }
        LegEvent::Rest(event) => {
            use crate::expedition::picnic::Event;
            match event {
                Event::Shared => "Out come the snacks, shared along the log.".to_owned(),
                Event::Dozed { who } => format!("{} has dozed off in the sun.", name(who)),
                Event::Snuggled { a, b } => format!("{} and {} sit close.", name(a), name(b)),
                Event::Played { a, b } => format!("{} and {} can't sit still.", name(a), name(b)),
                Event::Done => "Rested. On we go.".to_owned(),
            }
        }
        LegEvent::Falls(event) => {
            use falls::wading::Event;
            match event {
                Event::Glint {
                    spotted: Some(who), ..
                } => format!(
                    "{} spotted something glinting at the top of the falls!",
                    name(who)
                ),
                Event::Glint { find, .. } => {
                    let faint =
                        finds::find(find).is_some_and(|find| find.tier >= Tier::Exceptional);
                    if faint {
                        "Something glints, very faintly, at the top of the falls\u{2026}".to_owned()
                    } else {
                        "Something glints at the top of the falls\u{2026}".to_owned()
                    }
                }
                Event::Down => {
                    "Down it comes! Catch it as the eddy brings it past the stone.".to_owned()
                }
                Event::Caught { find } => format!("Caught! {}.", find_name(find)),
                Event::Slipped => "It went past. Round it comes again.".to_owned(),
                Event::AllDown => "That's all the falls bring down today.".to_owned(),
                Event::Leaving => "On from the falls.".to_owned(),
            }
        }
    })
}

/// The expeditions' part of the journal: how far the colony has gone, everything brought from
/// afar, and the latest expeditions' pages, the latest first.
pub(super) fn journal(ui: &mut egui::Ui, colony: &ColonyMemories, who: &dyn Fn(&str) -> String) {
    ui.add_space(6.0);
    ui.strong(format!("Expeditions \u{b7} {} so far", colony.expeditions));
    if let Some(furthest) = &colony.furthest {
        let place = map::place(&furthest.place).map_or("somewhere", |place| PLACES[place].name);
        let party: Vec<String> = furthest.party.iter().map(|id| who(id)).collect();
        ui.label(
            egui::RichText::new(format!(
                "The furthest yet: {place}, first reached by {}",
                listed(&party)
            ))
            .small()
            .italics(),
        );
    }
    let found = AFAR
        .iter()
        .filter(|find| colony.finds.contains_key(find.id))
        .count();
    ui.label(
        egui::RichText::new(format!("Brought from afar ({found} of {})", AFAR.len())).strong(),
    );
    for find in &AFAR {
        match colony.finds.get(find.id) {
            Some(record) => {
                ui.label(egui::RichText::new(find.name).strong());
                ui.label(egui::RichText::new(find.blurb).small());
                ui.label(
                    egui::RichText::new(format!(
                        "Becomes {} \u{b7} brought home {}\u{d7} \u{b7} first found by {}",
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
                        "??? \u{b7} something {}, a long way into the Woods",
                        find.tier.label()
                    ))
                    .weak(),
                );
            }
        }
    }
    for page in colony.expedition_pages.iter().rev() {
        ui.add_space(4.0);
        let route: Vec<&str> = page
            .route
            .iter()
            .map(|id| map::place(id).map_or("somewhere", |place| PLACES[place].name))
            .collect();
        let party: Vec<String> = page.party.iter().map(|id| who(id)).collect();
        ui.label(egui::RichText::new(format!("With {}", listed(&party))).strong());
        ui.label(egui::RichText::new(capital(&route.join(" \u{2192} "))).small());
        if !page.found.is_empty() {
            let found: Vec<String> = page
                .found
                .iter()
                .map(|(find, by)| {
                    let name = finds::find(find).map_or("something", |find| find.name);
                    format!("{} ({})", lower(name), who(by))
                })
                .collect();
            ui.label(
                egui::RichText::new(format!("Found {}", listed(&found)))
                    .small()
                    .italics(),
            );
        }
        if !page.caught.is_empty() {
            let caught: Vec<String> = page
                .caught
                .iter()
                .map(|id| {
                    fishing::fish::fish(id)
                        .map(|fish| fish.name)
                        .or_else(|| meadow::bugs::bug(id).map(|bug| bug.name))
                        .map_or("something".to_owned(), lower)
                })
                .collect();
            ui.label(
                egui::RichText::new(format!("Caught and let go: {}", listed(&caught)))
                    .small()
                    .italics(),
            );
        }
    }
}

/// What a companion would make of an expedition, for the person choosing who to bring: the
/// ways it opens, and how many it has been on.
pub(super) fn expeditioner_hint(colony: &ColonyMemories, id: Id, character: &Character) -> String {
    let mut ways = Vec::new();
    if map::explorer(character) {
        ways.push("knows a deer track");
    }
    if character.parent.is_some() {
        ways.push("fits through the gap in the hedge");
    }
    if map::bold(character) {
        ways.push("takes the steep way");
    }
    if map::reader(character) {
        ways.push("reads old signposts");
    }
    if character.kind == formiga_core::TemperamentKind::Lazybones {
        ways.push("is glad of a rest on the log");
    }
    let said = if ways.is_empty() {
        "Steady on the path".to_owned()
    } else {
        capital(&ways.into_iter().take(2).collect::<Vec<_>>().join(", and "))
    };
    let been = colony
        .expeditions_by
        .get(&id.to_string())
        .copied()
        .unwrap_or(0);
    format!(
        "{said}. {} expedition{} so far.",
        been,
        if been == 1 { "" } else { "s" }
    )
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// A list as it reads in a sentence: "Mochi, Pip and Fig".
fn listed(items: &[String]) -> String {
    match items {
        [] => "nobody".to_owned(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// A name as it reads mid-sentence.
fn lower(name: &str) -> String {
    let mut name = name.to_owned();
    if let Some(first) = name.get_mut(0..1) {
        first.make_ascii_lowercase();
    }
    name
}

/// A phrase as it starts a sentence.
fn capital(text: &str) -> String {
    let mut text = text.to_owned();
    if let Some(first) = text.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    text
}
