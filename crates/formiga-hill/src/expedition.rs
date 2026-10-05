//! An expedition: a longer outing than any of the Woods' activities, with one to three
//! companions, that strings them together on one day's light and one basket, and goes further in
//! than any of them: to the fallen log for a rest, past the old signpost, and to the Far Falls,
//! which only an expedition reaches.
//!
//! The person plays. The expedition is planned on a map of the Woods (see `map`) and the way is
//! chosen at each fork as the party goes: walking a path takes some of the day, and who came
//! opens ways only they can take. At each stop the party can play a short leg of what is done
//! there (see `legs`), started partway through the expedition's day with what is in the basket,
//! handing back the light left and the basket as it is then. The basket holds six all day, so
//! what to keep matters: back from a stop with too much, something has to be left before going
//! on. The day has its own dusk, whatever the hour: once the light has gone, or whenever the
//! person likes, the party heads home with everything in the basket. Nothing in it is lost.
//!
//! What stands on the Hilltop helps: whatever lends the Woods light lends the expedition's day as
//! much, and a telescope shows the Far Falls on the map from the start.

pub mod legs;
pub mod map;
pub mod picnic;
mod scenery;

use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character};
use crate::daylight::{Daylight, Nightlights};
use crate::finds::art;
use crate::hilltop::Arrangement;
use crate::paint::{blit, mix, put, rect, rgb, rgba};
use crate::playground::{Layout, Patch, Playground, distance};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::woods::Influence;
use crate::woods::rummage::dim;
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{Gesture, TemperamentKind};
use formiga_travel::Band;
use legs::{Caught, Input, Leg, LegEvent, Start};
use map::{Company, EDGE, Opener, PATHS, PLACES, SIGNPOST, Stop, Way};
pub use scenery::Shown;
use std::collections::BTreeSet;

/// The light an expedition's day starts with, before the Hilltop lends any: a longer day than
/// any one outing's.
pub const LIGHT: f32 = 180.0;
/// How many finds the basket holds: the one Woods basket, all day.
pub const BASKET: usize = crate::woods::rummage::BASKET;
/// How many can come.
pub const PARTY: usize = 3;
/// What a rest on the log takes of the day.
pub const REST_COST: f32 = 6.0;

/// Where on the map each of the party walks, about the place the leader is at.
const FORMATION: [(f32, f32); PARTY] = [(0.0, 0.0), (-15.0, 3.0), (15.0, 3.0)];
/// How briskly the party crosses the map, in pixels a second: a map's pace, not a stroll's.
const MAP_PACE: f32 = 64.0;
/// How long the leader points the way before setting off, and with reduced motion how long that
/// is held before the cut to where they are going.
const POINTING: f32 = 0.4;
const HELD: f32 = 0.9;
/// Where the party comes onto the map from, and goes off it at the end: the lane from the Hill.
const LANE: (f32, f32) = (-24.0, 186.0);

/// Each activity's dry spell as it stands, for its legs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Droughts {
    pub rummage: u32,
    pub fish: u32,
    pub bugs: u32,
    pub forage: u32,
    /// Visits to the far places in a row with nothing new from afar.
    pub far: u32,
}

/// What the colony knows as the expedition sets off, from its memories.
#[derive(Clone, Debug, Default)]
pub struct Known {
    pub finds: BTreeSet<String>,
    pub fish: BTreeSet<String>,
    pub bugs: BTreeSet<String>,
    pub droughts: Droughts,
    pub hilltop: Arrangement,
    /// The far places it has been to, by their ids on the map.
    pub far_places: BTreeSet<String>,
}

/// What an expedition sets out with.
pub struct Outset {
    /// Who came along, one to three; the first leads the way.
    pub party: Vec<Id>,
    /// What the Hilltop lends the Woods: the day's light, and rare things sooner.
    pub influence: Influence,
    pub seed: u64,
}

/// Whether something to look far off with stands on the Hilltop: the little telescope, or
/// anything built with its lens.
pub fn far_sight(hilltop: &Arrangement) -> bool {
    hilltop
        .values()
        .any(|standing| standing.finds().contains(&"brass_lens"))
}

/// The map as it would be planned for `party`: the ways that company would open, and the far
/// places already in sight.
pub fn planned(cast: &Cast, party: &[Id], known: &Known) -> Canvas {
    let members: Vec<(Id, Character)> = party
        .iter()
        .filter_map(|id| cast.member(*id).map(|member| (*id, Character::of(member))))
        .take(PARTY)
        .collect();
    let close = |a: Id, b: Id| {
        cast.bond(a, b)
            .is_some_and(|bond| bond.warmth >= Band::High)
    };
    let company = Company::of(&members, close);
    let mut sighted = [false; PLACES.len()];
    for (index, place) in PLACES.iter().enumerate() {
        sighted[index] =
            !place.hidden || known.far_places.contains(place.id) || far_sight(&known.hilltop);
    }
    scenery::map(&Expedition::shown_for(&company, &sighted, false))
}

/// Something in the basket, and who found it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Carried {
    pub find: &'static str,
    pub by: Id,
}

/// Why the party headed home.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Dusk,
    Chose,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The party coming onto the map, up the lane from the Hill.
    Arriving {
        since: f32,
    },
    /// On the map, choosing the way on, or whether to stop here.
    Choosing,
    /// On the way along a path to a place.
    Walking {
        path: usize,
        to: usize,
    },
    /// At a stop, playing its leg.
    Stopped,
    /// Back from a stop with more than the basket holds: something has to be left first.
    MakingRoom,
    /// Heading home with the basket.
    Homeward {
        since: f32,
        why: Ending,
    },
    Over,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Someone in the party opens a way only some company takes.
    Opened {
        opener: Opener,
        who: Id,
    },
    /// From the telescope on the Hilltop, a far place shows on the map.
    Spied {
        place: usize,
    },
    /// Someone read the old signpost: the old way, and where it goes.
    Read {
        who: Id,
        place: usize,
    },
    Arrived {
        place: usize,
    },
    /// A far place found, out of the mist.
    Found {
        place: usize,
    },
    /// Not enough of the day left to get there.
    TooFar {
        place: usize,
    },
    /// A leg begins at a stop, whoever leads it.
    Began {
        place: usize,
        leader: Id,
    },
    /// Something in a leg, as its activity tells it.
    Leg(LegEvent),
    /// A leg is over: how many finds it added, and how many fish or bugs were caught.
    Ended {
        place: usize,
        found: usize,
        caught: usize,
    },
    /// More than the basket holds: something has to be left.
    TooFull,
    /// Something left behind to make room.
    Left {
        find: &'static str,
    },
    /// The day's light is going: time to head home.
    Dusk,
    Homeward(Ending),
}

/// A choice of way on, from where the party is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Onward {
    pub to: usize,
    pub path: usize,
    /// How much of the day it would take.
    pub cost: f32,
}

pub struct Expedition {
    party: Vec<Id>,
    characters: Vec<Character>,
    company: Company,
    /// Every pair in the party that gets on well enough to be called close.
    close: Vec<(Id, Id)>,
    /// How much more or less of the day walking takes this party.
    pace: f32,
    light: f32,
    full: f32,
    basket: Vec<Carried>,
    /// Fish landed and bugs caught on the way, each with its size and who caught it.
    fish: Vec<Caught>,
    bugs: Vec<Caught>,
    at: usize,
    route: Vec<usize>,
    stops: Vec<usize>,
    /// Which places are in sight on the map, and whether the signpost has been read.
    sighted: [bool; PLACES.len()],
    read: bool,
    phase: Phase,
    leg: Option<Leg>,
    /// The map, with the party on it.
    map: Playground,
    shown: Shown,
    known: Known,
    influence: Influence,
    events: Vec<Event>,
    seed: u64,
    reduce_motion: bool,
    daylight: Daylight,
    /// Heading home as soon as the leg under way is over.
    home_after_leg: bool,
    /// The party on its way across the map.
    trek: Option<Trek>,
}

impl Expedition {
    pub fn new(cast: &Cast, outset: Outset, known: Known, now: f32) -> Self {
        let Outset {
            party,
            influence,
            seed,
        } = outset;
        let party: Vec<Id> = party
            .into_iter()
            .filter(|id| cast.member(*id).is_some())
            .take(PARTY)
            .collect();
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| cast.member(*id).map(Character::of))
            .collect();
        let close_bond = |a: Id, b: Id| {
            cast.bond(a, b)
                .is_some_and(|bond| bond.warmth >= Band::High)
        };
        let members: Vec<(Id, Character)> = party
            .iter()
            .copied()
            .zip(characters.iter().cloned())
            .collect();
        let company = Company::of(&members, close_bond);
        let close: Vec<(Id, Id)> = party
            .iter()
            .enumerate()
            .flat_map(|(index, a)| party[index + 1..].iter().map(move |b| (*a, *b)))
            .filter(|(a, b)| close_bond(*a, *b))
            .collect();
        let refs: Vec<&Character> = characters.iter().collect();
        let pace = map::pace(&refs);
        let mut events = Vec::new();
        for opener in Opener::ALL {
            // The signpost's old way is told of when it is read.
            if opener == Opener::Reader {
                continue;
            }
            if let Some(who) = company.who(opener).first() {
                events.push(Event::Opened { opener, who: *who });
            }
        }
        let mut sighted = [false; PLACES.len()];
        for (index, place) in PLACES.iter().enumerate() {
            sighted[index] = !place.hidden || known.far_places.contains(place.id);
        }
        if far_sight(&known.hilltop) {
            for (index, place) in PLACES.iter().enumerate() {
                if place.hidden && !sighted[index] {
                    sighted[index] = true;
                    events.push(Event::Spied { place: index });
                }
            }
        }
        let full = LIGHT + influence.light;
        let shown = Self::shown_for(&company, &sighted, false);
        let mut ground = Playground::with_members(
            cast,
            &party,
            now,
            layout(),
            scenery::map(&shown),
            Canvas::new(SCENE_WIDTH, SCENE_HEIGHT),
            Vec::new(),
        );
        // The map is paper on a table: lit as a room is, by lamplight after dark.
        ground.set_nightlights(Nightlights {
            lamps: Canvas::new(1, 1),
            sky: Canvas::new(1, 1),
            indoors: true,
        });
        let reduce_motion = ground.reduce_motion();
        let mut expedition = Self {
            party,
            characters,
            company,
            close,
            pace,
            light: full,
            full,
            basket: Vec::new(),
            fish: Vec::new(),
            bugs: Vec::new(),
            at: EDGE,
            route: vec![EDGE],
            stops: Vec::new(),
            sighted,
            read: false,
            phase: Phase::Arriving { since: now },
            leg: None,
            map: ground,
            shown,
            known,
            influence,
            events,
            seed,
            reduce_motion,
            daylight: Daylight::default(),
            home_after_leg: false,
            trek: None,
        };
        expedition.walk_in(now);
        expedition
    }

    /// The paths and places the map shows to this company: the open paths and its own
    /// shortcuts, the old way once read, and the far places once in sight.
    fn shown_for(company: &Company, sighted: &[bool; PLACES.len()], read: bool) -> Shown {
        let mut paths = [false; PATHS.len()];
        for (index, path) in PATHS.iter().enumerate() {
            paths[index] = map::walkable(path, company, read);
        }
        Shown {
            places: *sighted,
            paths,
        }
    }

    /// Up the lane from the Hill, onto the map at the edge of the Woods.
    fn walk_in(&mut self, now: f32) {
        self.map.reserve(self.party.clone());
        self.set_out(vec![LANE, PLACES[EDGE].at], 0.0, now);
    }

    /// Sets the party off across the map along `points`, after `pause` seconds pointing the
    /// way: held a little longer with reduced motion, which then cuts to the end.
    fn set_out(&mut self, points: Vec<(f32, f32)>, pause: f32, now: f32) {
        let pause = if self.reduce_motion { HELD } else { pause };
        if let Some(start) = points.first() {
            for (index, id) in self.party.iter().enumerate() {
                self.map.teleport(*id, formation(*start, index));
            }
        }
        if pause > 0.0
            && let (Some(leader), Some(next)) = (self.party.first(), points.get(1))
        {
            let point = Beat::new(Gesture::Reach, ExpressionKind::Determined, pause);
            self.map
                .direct(*leader, vec![Step::FaceX(next.0), Step::Beat(point)], now);
            for id in self.party.iter().skip(1) {
                let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, pause);
                self.map
                    .direct(*id, vec![Step::FaceX(next.0), Step::Beat(watch)], now);
            }
        }
        self.trek = Some(Trek {
            points,
            since: now + pause,
            heading: None,
        });
    }

    /// Moves the party along its way, at the map's pace: or, with reduced motion, cuts to the
    /// end once the way has been pointed out. Says whether they have got there.
    fn trek_on(&mut self, now: f32) -> bool {
        let Some(trek) = &mut self.trek else {
            return true;
        };
        let walked = now - trek.since;
        if walked < 0.0 {
            return false;
        }
        let travelled = if self.reduce_motion {
            f32::MAX
        } else {
            walked * MAP_PACE
        };
        let (here, towards, done) = trek.along(travelled);
        let segment = trek.segment(travelled);
        let turned = trek.heading != Some(segment);
        trek.heading = Some(segment);
        for (index, id) in self.party.iter().enumerate() {
            self.map.teleport(*id, formation(here, index));
            if done {
                self.map.direct(*id, Vec::new(), now);
            } else if turned {
                let face = self
                    .characters
                    .get(index)
                    .map_or(ExpressionKind::Content, Character::walk_face);
                let walking = Beat::new(formiga_core::ActionKind::Traverse, face, 600.0);
                self.map
                    .direct(*id, vec![Step::FaceX(towards.0), Step::Beat(walking)], now);
            }
        }
        if done {
            self.trek = None;
        }
        done
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.party
    }

    pub fn company(&self) -> &Company {
        &self.company
    }

    /// The light left, on the day's scale, and as a share of it.
    pub fn light(&self) -> f32 {
        match &self.leg {
            Some(leg) if !matches!(leg.play, legs::Play::Rest { .. }) => leg.light(),
            _ => self.light,
        }
    }

    pub fn light_left(&self) -> f32 {
        (self.light() / self.full).clamp(0.0, 1.0)
    }

    /// The whole of the day's light.
    pub fn full(&self) -> f32 {
        self.full
    }

    /// What is in the basket, and who found each, as it stands between stops.
    pub fn basket(&self) -> &[Carried] {
        &self.basket
    }

    /// Where the party is, every place it has been, and where it stopped.
    pub fn at(&self) -> usize {
        self.at
    }

    pub fn route(&self) -> &[usize] {
        &self.route
    }

    pub fn stops(&self) -> &[usize] {
        &self.stops
    }

    /// The fish and bugs caught on the way.
    pub fn fish(&self) -> &[Caught] {
        &self.fish
    }

    pub fn bugs(&self) -> &[Caught] {
        &self.bugs
    }

    /// Whether a place is in sight on the map.
    pub fn sighted(&self, place: usize) -> bool {
        self.sighted[place]
    }

    /// The leg under way, if the party is at a stop.
    pub fn leg(&self) -> Option<&Leg> {
        self.leg.as_ref()
    }

    pub fn leg_mut(&mut self) -> Option<&mut Leg> {
        self.leg.as_mut()
    }

    /// Where the party is now: the stop they are at, or the map.
    pub fn ground_mut(&mut self) -> &mut Playground {
        match &mut self.leg {
            Some(leg) => &mut leg.ground,
            None => &mut self.map,
        }
    }

    /// Brings any stop under way to a close at once, its basket as it is, as when the person
    /// leaves the Woods partway through it.
    pub fn close_leg(&mut self, now: f32) {
        if self.leg.is_some() {
            self.end_leg(now);
        }
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// The furthest place reached, and how far in it is.
    pub fn furthest(&self) -> usize {
        self.route
            .iter()
            .copied()
            .max_by_key(|place| PLACES[*place].depth)
            .unwrap_or(EDGE)
    }

    /// The ways on from where the party is: each place a path it can walk goes to, by the
    /// shortest such path.
    pub fn ways(&self) -> Vec<Onward> {
        let mut ways: Vec<Onward> = Vec::new();
        for (index, path) in PATHS.iter().enumerate() {
            if !map::walkable(path, &self.company, self.read) {
                continue;
            }
            let Some(to) = path.from(self.at) else {
                continue;
            };
            let way = Onward {
                to,
                path: index,
                cost: path.length * self.pace,
            };
            match ways.iter_mut().find(|known| known.to == to) {
                Some(known) if way.cost < known.cost => *known = way,
                Some(_) => {}
                None => ways.push(way),
            }
        }
        ways
    }

    /// The way to a place from here, if there is one.
    pub fn way_to(&self, place: usize) -> Option<Onward> {
        self.ways().into_iter().find(|way| way.to == place)
    }

    /// Whether the party can stop and play here now: a stop with something to do, not played
    /// already today, with light left to do it in.
    pub fn can_stop(&self) -> bool {
        self.phase == Phase::Choosing
            && PLACES[self.at].stop.played()
            && !self.stops.contains(&self.at)
            && self.light > 0.0
    }

    /// The person chose a place: the party sets off along the path there, if there is one from
    /// here and the day has light enough left for it.
    pub fn go(&mut self, place: usize, now: f32) -> bool {
        if self.phase != Phase::Choosing {
            return false;
        }
        let Some(way) = self.way_to(place) else {
            return false;
        };
        if way.cost > self.light + 0.01 {
            self.events.push(Event::TooFar { place });
            return false;
        }
        self.light -= way.cost;
        let points = PATHS[way.path].walk_from(self.at);
        self.set_out(points, POINTING, now);
        self.phase = Phase::Walking {
            path: way.path,
            to: place,
        };
        true
    }

    /// Arrived somewhere: anything hidden there is found, a signpost is read if someone can,
    /// and the party looks about.
    fn arrive(&mut self, place: usize, now: f32) {
        self.at = place;
        self.route.push(place);
        if !self.sighted[place] {
            self.sighted[place] = true;
            self.events.push(Event::Found { place });
        }
        self.events.push(Event::Arrived { place });
        if place == SIGNPOST
            && !self.read
            && let Some(reader) = self.company.reader
        {
            self.read = true;
            let to = PATHS
                .iter()
                .find(|path| matches!(path.way, Way::Shortcut(_, Opener::Reader)))
                .and_then(|path| path.from(SIGNPOST))
                .unwrap_or(SIGNPOST);
            for (index, place) in PLACES.iter().enumerate() {
                if place.hidden && PATHS.iter().any(|path| path.from(SIGNPOST) == Some(index)) {
                    self.sighted[index] = true;
                }
            }
            self.events.push(Event::Read {
                who: reader,
                place: to,
            });
            let mut reads = Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.6);
            reads.cue = Some(crate::character::Cue::Exclaim);
            self.map.direct(reader, vec![Step::Beat(reads)], now);
        }
        self.repaint();
        self.phase = Phase::Choosing;
    }

    /// Repaints the map, if what it shows has changed.
    fn repaint(&mut self) {
        let shown = Self::shown_for(&self.company, &self.sighted, self.read);
        if shown != self.shown {
            self.map.set_backdrop(scenery::map(&shown));
            self.shown = shown;
        }
    }

    /// The person chose to stop and play here: the party goes into the place and its leg begins.
    pub fn stop(&mut self, cast: &Cast, now: f32) -> bool {
        if !self.can_stop() {
            return false;
        }
        let place = self.at;
        let stop = PLACES[place].stop;
        let members: Vec<(Id, Character)> = self
            .party
            .iter()
            .copied()
            .zip(self.characters.iter().cloned())
            .collect();
        let players = legs::players(stop, &members);
        let close_pair = players.iter().enumerate().any(|(index, a)| {
            players[index + 1..]
                .iter()
                .any(|b| self.close.contains(&(*a, *b)) || self.close.contains(&(*b, *a)))
        });
        if stop == Stop::Rest {
            self.light -= REST_COST.min(self.light);
        }
        let carried: Vec<&'static str> = self.basket.iter().map(|carried| carried.find).collect();
        let finds = &self.known.finds;
        let in_hand = carried.clone();
        let found = move |id: &str| finds.contains(id) || in_hand.contains(&id);
        let fish = |id: &str| self.known.fish.contains(id);
        let bugs = |id: &str| self.known.bugs.contains(id);
        let start = Start {
            place,
            stop,
            everyone: &self.party,
            players,
            close_pair,
            light: self.light,
            full: self.full,
            basket: carried,
            influence: &self.influence,
            droughts: self.known.droughts,
            found: &found,
            fish: &fish,
            bugs: &bugs,
            hilltop: &self.known.hilltop,
            map: self.map.backdrop(),
            daylight: self.daylight,
            seed: self
                .seed
                .wrapping_add((self.stops.len() as u64 + 1).wrapping_mul(0x9e37_79b9)),
        };
        let Some(leg) = Leg::start(cast, start, now) else {
            return false;
        };
        self.events.push(Event::Began {
            place,
            leader: leg.leader,
        });
        self.stops.push(place);
        self.leg = Some(leg);
        self.phase = Phase::Stopped;
        true
    }

    /// The leg is over: the party is back on the map with the light left, the basket as it is,
    /// and anything caught.
    fn end_leg(&mut self, now: f32) {
        let Some(leg) = self.leg.take() else {
            return;
        };
        self.light = leg.light().max(0.0);
        let before = self.basket.len();
        self.basket = carried(&self.basket, &leg.basket(), leg.leader);
        let (fish, bugs) = leg.caught();
        let caught = fish.len() + bugs.len();
        self.fish.extend(fish);
        self.bugs.extend(bugs);
        self.events.push(Event::Ended {
            place: leg.place,
            found: self.basket.len().saturating_sub(before),
            caught,
        });
        // Back on the map, where they were.
        for (index, id) in self.party.iter().enumerate() {
            self.map.teleport(*id, formation(PLACES[self.at].at, index));
            self.map.direct(*id, Vec::new(), now);
        }
        self.after_stop(now);
    }

    /// After a stop: make room first if there is too much to carry, then home if the light has
    /// gone, or on.
    fn after_stop(&mut self, now: f32) {
        if self.basket.len() > BASKET {
            self.events.push(Event::TooFull);
            self.phase = Phase::MakingRoom;
        } else if self.home_after_leg {
            self.homeward(Ending::Chose, now);
        } else if self.light <= 0.0 {
            self.events.push(Event::Dusk);
            self.homeward(Ending::Dusk, now);
        } else {
            self.phase = Phase::Choosing;
        }
    }

    /// Leaves something from the basket behind, to make room or by choice, between stops.
    pub fn leave_behind(&mut self, slot: usize, now: f32) -> bool {
        if !matches!(self.phase, Phase::Choosing | Phase::MakingRoom) || slot >= self.basket.len() {
            return false;
        }
        let left = self.basket.remove(slot);
        self.events.push(Event::Left { find: left.find });
        if self.phase == Phase::MakingRoom && self.basket.len() <= BASKET {
            self.after_stop(now);
        }
        true
    }

    /// The person would rather move on from the stop: the party finishes up there.
    pub fn move_on(&mut self, now: f32) {
        if let Some(leg) = &mut self.leg {
            leg.move_on(now);
        }
    }

    /// Calls the expedition to an end: everyone heads home with what is in the basket, once any
    /// leg under way is finished, and once the basket is light enough to carry.
    pub fn head_home(&mut self, now: f32) {
        match self.phase {
            Phase::Stopped => {
                self.home_after_leg = true;
                self.move_on(now);
            }
            Phase::Choosing | Phase::Walking { .. } | Phase::Arriving { .. } => {
                self.homeward(Ending::Chose, now)
            }
            Phase::MakingRoom => self.home_after_leg = true,
            Phase::Homeward { .. } | Phase::Over => {}
        }
    }

    fn homeward(&mut self, why: Ending, now: f32) {
        // Back to the edge the way they came, as the crow flies on the map, and off down the lane.
        let here = match &self.trek {
            Some(trek) => trek.along((now - trek.since).max(0.0) * MAP_PACE).0,
            None => PLACES[self.at].at,
        };
        let tired = why == Ending::Dusk
            && self
                .characters
                .iter()
                .any(|c| c.kind == TemperamentKind::Lazybones);
        if tired
            && let Some(lazy) = self
                .party
                .iter()
                .zip(&self.characters)
                .find_map(|(id, c)| (c.kind == TemperamentKind::Lazybones).then_some(*id))
        {
            let yawn = Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.0);
            self.map.direct(lazy, vec![Step::Beat(yawn)], now);
        }
        let mut points = vec![here];
        if distance(here, PLACES[EDGE].at) > 2.0 {
            points.push(PLACES[EDGE].at);
        }
        points.push(LANE);
        self.trek = None;
        self.set_out(points, if tired { 1.0 } else { 0.0 }, now);
        self.events.push(Event::Homeward(why));
        self.phase = Phase::Homeward { since: now, why };
    }

    /// Plays the expedition on: the map, or the leg under way.
    pub fn tick(&mut self, cast: &Cast, input: Input, now: f32) {
        if let Some(leg) = &mut self.leg {
            let events = leg.tick(cast, input, now);
            self.events.extend(events.into_iter().map(Event::Leg));
            if leg.over() {
                self.end_leg(now);
            }
            return;
        }
        self.map.tick(cast, now);
        let there = self.trek_on(now);
        match self.phase {
            Phase::Arriving { .. } if there => self.phase = Phase::Choosing,
            Phase::Walking { to, .. } if there => self.arrive(to, now),
            Phase::Homeward { .. } if there => self.phase = Phase::Over,
            _ => {}
        }
    }

    /// The light of the hour, everywhere the expedition is.
    pub fn set_daylight(&mut self, daylight: Daylight) {
        self.daylight = daylight;
        self.map.set_daylight(daylight);
        if let Some(leg) = &mut self.leg {
            leg.set_daylight(daylight);
        }
    }

    /// The place on the map under a point, if any.
    pub fn place_at(&self, x: f32, y: f32) -> Option<usize> {
        PLACES
            .iter()
            .enumerate()
            .filter(|(index, _)| self.sighted[*index] || self.way_to(*index).is_some())
            .map(|(index, place)| {
                let (px, py) = place.at;
                let near = distance((x, y), (px, py - 12.0)).min(distance((x, y), (px, py)));
                (index, near)
            })
            .filter(|(_, near)| *near <= 18.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The basket's slot under a point on the map, for leaving something behind.
    pub fn basket_slot_at(&self, x: f32, y: f32) -> Option<usize> {
        let (left, top, slot) = basket_frame();
        let (x, y) = (x as i32 - left - 2, y as i32 - top - 2);
        if x < 0 || y < 0 || y >= slot {
            return None;
        }
        let index = (x / slot) as usize;
        (index < self.basket.len()).then_some(index)
    }

    /// The scene as it is now: the map with the party on it, or the stop they are at.
    pub fn compose(&mut self, now: f32, pointer: Option<(f32, f32)>) -> Canvas {
        if let Some(leg) = &mut self.leg {
            return leg.compose(now, pointer);
        }
        let mut scene = self.map.compose(now);
        dim(&mut scene, self.light, self.daylight.darkness());
        if let Phase::Walking { path, to, .. } = self.phase {
            let from = PATHS[path].from(to).unwrap_or(self.at);
            draw_way(&mut scene, &PATHS[path].walk_from(from), true, true);
        }
        if self.phase == Phase::Choosing {
            let hovered = pointer.and_then(|(x, y)| self.place_at(x, y));
            for way in self.ways() {
                let affordable = way.cost <= self.light + 0.01;
                draw_way(
                    &mut scene,
                    &PATHS[way.path].walk_from(self.at),
                    hovered == Some(way.to),
                    affordable,
                );
            }
        }
        for place in &self.stops {
            draw_tick(&mut scene, PLACES[*place].at);
        }
        self.draw_basket(&mut scene);
        scene
    }

    /// The basket in the top right corner, and the day's light under it, as every Woods basket
    /// is drawn; anything over what it holds is shown waiting beside it.
    fn draw_basket(&self, scene: &mut Canvas) {
        let (left, top, slot) = basket_frame();
        let width = slot * BASKET as i32 + 3;
        rect(scene, left, top, width, slot + 7, rgba(0x2a2018, 150));
        for index in 0..BASKET.max(self.basket.len()) as i32 {
            let (x, y) = (left + 2 + index * slot, top + 2);
            let over = index >= BASKET as i32;
            let (x, y) = if over {
                (left - 2 - (index - BASKET as i32 + 1) * slot, y)
            } else {
                (x, y)
            };
            let ground = if over {
                rgba(0xd05a40, 150)
            } else {
                rgba(0xf6eed8, 60)
            };
            rect(scene, x, y, slot - 1, slot - 1, ground);
            if let Some(carried) = self.basket.get(index as usize) {
                blit(scene, &art::icon(carried.find), x, y);
            }
        }
        let share = self.light_left();
        let bar = ((width - 4) as f32 * share) as i32;
        let gold = mix(rgb(0x6a5a9a), rgb(0xf5d25e), share);
        scene.fill_rect(left + 2, top + slot + 2, bar, 2, gold);
    }
}

/// The party's way across the map: the points along it, when they set off along it, and which
/// stretch of it they are on, for turning to face along each.
#[derive(Clone, Debug)]
struct Trek {
    points: Vec<(f32, f32)>,
    since: f32,
    heading: Option<usize>,
}

impl Trek {
    /// Where they are `travelled` pixels along, which way they are heading, and whether that is
    /// the end.
    fn along(&self, travelled: f32) -> ((f32, f32), (f32, f32), bool) {
        let mut left = travelled;
        for pair in self.points.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            let length = distance(from, to);
            if left <= length && length > 0.0 {
                let t = left / length;
                let at = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
                return (at, to, false);
            }
            left -= length;
        }
        let end = self.points.last().copied().unwrap_or((0.0, 0.0));
        (end, end, true)
    }

    /// Which stretch of the way `travelled` pixels along is on.
    fn segment(&self, travelled: f32) -> usize {
        let mut left = travelled;
        for (index, pair) in self.points.windows(2).enumerate() {
            let length = distance(pair[0], pair[1]);
            if left <= length {
                return index;
            }
            left -= length;
        }
        self.points.len()
    }
}

/// Where the `index`th of the party stands about a point on the map.
fn formation(at: (f32, f32), index: usize) -> (f32, f32) {
    let (dx, dy) = FORMATION[index % FORMATION.len()];
    (at.0 + dx, at.1 + dy)
}

/// The basket, as it stands after a leg handed back `now`: whatever was carried in and is still
/// there keeps who found it, and anything new was found by whoever led the leg.
fn carried(before: &[Carried], now: &[&'static str], leader: Id) -> Vec<Carried> {
    let mut old: Vec<Option<Carried>> = before.iter().copied().map(Some).collect();
    now.iter()
        .map(|find| {
            let kept = old
                .iter_mut()
                .find(|carried| carried.is_some_and(|carried| carried.find == *find))
                .and_then(Option::take);
            kept.unwrap_or(Carried { find, by: leader })
        })
        .collect()
}

/// Where the basket is drawn: left, top, and the size of a slot.
fn basket_frame() -> (i32, i32, i32) {
    const SLOT: i32 = 11;
    let across = SLOT * BASKET as i32 + 3;
    (SCENE_WIDTH as i32 - across - 4, 4, SLOT)
}

/// Where anyone can stand on the map: anywhere on the paper.
fn walkable(x: f32, y: f32) -> bool {
    (-60.0..=380.0).contains(&x) && (16.0..=212.0).contains(&y)
}

const MAP_SPOTS: [(&str, (f32, f32)); 1] = [("centre", (36.0, 182.0))];

fn layout() -> Layout {
    let ground: Patch = (4.0, 16.0, 380.0, 212.0);
    Layout {
        walkable,
        ground,
        spots: &MAP_SPOTS,
        seats: None,
        shade: None,
        entrance: LANE,
        entrance_step: (-18.0, 0.0),
    }
}

/// A way on from here, inked bolder over the path: gold under the pointer, faded where there is
/// not enough of the day left for it.
fn draw_way(scene: &mut Canvas, points: &[(f32, f32)], hovered: bool, affordable: bool) {
    let ink = match (hovered, affordable) {
        (true, true) => rgb(0xf5d25e),
        (_, true) => rgba(0xfff4d8, 210),
        (_, false) => rgba(0x8a7a6a, 140),
    };
    let mut walked = 0.0;
    for pair in points.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let length = distance(from, to).max(1.0);
        let mut along = 0.0;
        while along < length {
            let t = along / length;
            let (x, y) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            // Dashes of three, gaps of two.
            if (walked + along) as i32 % 5 < 3 {
                put(scene, x as i32, y as i32, ink);
                if hovered {
                    put(scene, x as i32, y as i32 + 1, rgba(0x7a4a1a, 160));
                }
            }
            along += 1.0;
        }
        walked += length;
    }
}

/// A tick by a place where the party has stopped today.
fn draw_tick(scene: &mut Canvas, (x, y): (f32, f32)) {
    let (x, y) = (x as i32 + 14, y as i32 - 26);
    let ink = rgb(0x3a6a2a);
    for (dx, dy) in [(0, 2), (1, 3), (2, 2), (3, 1), (4, 0)] {
        put(scene, x + dx, y + dy, ink);
        put(scene, x + dx, y + dy + 1, rgba(0x1a3a1a, 120));
    }
}

/// The basket carried in, drawn in the top left corner of a stop whose own corner shows what it
/// caught instead, with whatever this stop has added since.
pub(crate) fn draw_carried(scene: &mut Canvas, carried: &[&'static str], added: &[&'static str]) {
    const SLOT: i32 = 11;
    let width = SLOT * BASKET as i32 + 3;
    let (left, top) = (4, 4);
    rect(scene, left, top, width, SLOT + 4, rgba(0x2a2018, 150));
    let all: Vec<&str> = carried.iter().chain(added).copied().collect();
    for index in 0..BASKET.max(all.len()) as i32 {
        let (x, y) = (left + 2 + index * SLOT, top + 2);
        let ground = if index >= BASKET as i32 {
            rgba(0xd05a40, 150)
        } else {
            rgba(0xf6eed8, 60)
        };
        rect(scene, x, y, SLOT - 1, SLOT - 1, ground);
        if let Some(id) = all.get(index as usize) {
            blit(scene, &art::icon(id), x, y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds;
    use crate::hilltop::Standing;
    use legs::Play;
    use map::{FAR_FALLS, GLADE, LOG, POOL};

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn named(cast: &Cast, name: &str) -> Id {
        cast.members
            .iter()
            .find(|member| member.name == name)
            .unwrap_or_else(|| panic!("no {name} in the sample"))
            .id
    }

    fn setting_off(cast: &Cast, party: Vec<Id>, known: Known) -> Expedition {
        let outset = Outset {
            party,
            influence: Influence::default(),
            seed: 5,
        };
        Expedition::new(cast, outset, known, 0.0)
    }

    /// Plays on, with a steady hand at any stop, until `done` says so.
    fn play(
        trip: &mut Expedition,
        cast: &Cast,
        from: f32,
        done: impl Fn(&Expedition) -> bool,
    ) -> f32 {
        let mut now = from;
        while !done(trip) {
            assert!(now < from + 400.0, "stuck at {:?}", trip.phase());
            now += 1.0 / 30.0;
            let input = trip
                .leg_mut()
                .map(|leg| leg.steady(now))
                .unwrap_or_default();
            trip.tick(cast, input, now);
        }
        now
    }

    fn choosing(trip: &Expedition) -> bool {
        trip.phase() == Phase::Choosing
    }

    /// Onto the map, and ready to choose the way.
    fn arrived(trip: &mut Expedition, cast: &Cast) -> f32 {
        play(trip, cast, 0.0, choosing)
    }

    /// Plays on until the party has got wherever it was going.
    fn walk_on(trip: &mut Expedition, cast: &Cast, now: f32) -> f32 {
        play(trip, cast, now, |trip| {
            !matches!(trip.phase(), Phase::Walking { .. })
        })
    }

    /// Goes on to `place`, which must be the next place on.
    fn walk(trip: &mut Expedition, cast: &Cast, place: usize, now: f32) -> f32 {
        assert!(trip.go(place, now), "no way on to {}", PLACES[place].name);
        walk_on(trip, cast, now)
    }

    /// Stops and plays the leg here with a steady hand until the party is back on the map.
    fn stop_here(trip: &mut Expedition, cast: &Cast, now: f32) -> f32 {
        assert!(
            trip.stop(cast, now),
            "can't stop at {}",
            PLACES[trip.at()].name
        );
        play(trip, cast, now, |trip| trip.phase() != Phase::Stopped)
    }

    fn finds_of(trip: &Expedition) -> Vec<&'static str> {
        trip.basket().iter().map(|carried| carried.find).collect()
    }

    #[test]
    fn walking_a_path_spends_the_days_light_and_a_lively_party_spends_less() {
        let cast = sample();
        let mut trip = setting_off(&cast, vec![named(&cast, "Mochi")], Known::default());
        let now = arrived(&mut trip, &cast);
        assert_eq!(trip.light(), LIGHT);
        assert!(
            !trip.go(FAR_FALLS, now),
            "the Far Falls are not one path from the edge"
        );
        let way = trip.way_to(GLADE).unwrap();
        assert!(trip.go(GLADE, now));
        assert!((trip.light() - (LIGHT - way.cost)).abs() < 0.001);
        let now = walk_on(&mut trip, &cast, now);
        assert_eq!(trip.at(), GLADE);
        let cost = |name: &str| {
            let trip = setting_off(&cast, vec![named(&cast, name)], Known::default());
            trip.way_to(GLADE).unwrap().cost
        };
        // Biscuit is lively and Fig a lazybones: the slowest sets the pace.
        assert!(cost("Biscuit") < cost("Fig"));
        // Not enough of the day left to get somewhere: the party stays put.
        trip.light = 1.0;
        trip.take_events();
        assert!(!trip.go(LOG, now));
        assert!(trip.take_events().contains(&Event::TooFar { place: LOG }));
        assert_eq!(trip.phase(), Phase::Choosing);
    }

    #[test]
    fn the_shared_light_flows_through_paths_and_legs() {
        let cast = sample();
        let party = vec![named(&cast, "Tansy"), named(&cast, "Fig")];
        let mut trip = setting_off(&cast, party, Known::default());
        let mut now = arrived(&mut trip, &cast);
        // Through every kind of stop, the light only ever goes down, each leg starts with what
        // the walk there left, and the day goes on from what the leg left.
        for place in [GLADE, LOG, map::SIGNPOST, POOL, map::SIGNPOST, FAR_FALLS] {
            if trip.phase() != Phase::Choosing {
                break;
            }
            let before = trip.light();
            assert!(trip.go(place, now));
            assert!(
                trip.light() < before,
                "walking to {} took nothing",
                PLACES[place].name
            );
            now = walk_on(&mut trip, &cast, now);
            if !PLACES[place].stop.played() {
                continue;
            }
            let arrived_with = trip.light();
            assert!(trip.stop(&cast, now));
            let rest = PLACES[place].stop == Stop::Rest;
            let starts_with = trip.leg().unwrap().light();
            if rest {
                assert!((arrived_with - starts_with - REST_COST).abs() < 0.001);
            } else {
                assert_eq!(
                    starts_with, arrived_with,
                    "the leg didn't start where the day was"
                );
            }
            let mut last = starts_with;
            while trip.phase() == Phase::Stopped {
                now += 1.0 / 30.0;
                let input = trip
                    .leg_mut()
                    .map(|leg| leg.steady(now))
                    .unwrap_or_default();
                trip.tick(&cast, input, now);
                if let Some(leg) = trip.leg() {
                    assert!(leg.light() <= last + 0.001, "the light came back");
                    last = leg.light();
                }
                assert!(now < 2000.0, "stuck at {}", PLACES[place].name);
            }
            assert!(
                (trip.light() - last.max(0.0)).abs() < 0.001,
                "the day didn't go on from what {} left",
                PLACES[place].name
            );
            assert!(
                trip.light() < arrived_with,
                "{} took none of the day",
                PLACES[place].name
            );
            while trip.phase() == Phase::MakingRoom {
                trip.leave_behind(0, now);
            }
        }
        assert!(trip.stops().len() >= 3, "the day ran out too soon to tell");
    }

    #[test]
    fn each_leg_hands_back_light_and_finds_and_the_basket_it_was_given() {
        let cast = sample();
        let party = [named(&cast, "Tansy"), named(&cast, "Mochi")];
        let map = scenery::map(&Shown {
            places: [true; PLACES.len()],
            paths: [true; PATHS.len()],
        });
        let influence = Influence::default();
        let hilltop = Arrangement::new();
        let nothing = |_: &str| false;
        for (place, stop) in [
            (GLADE, Stop::Rummage),
            (POOL, Stop::Fish),
            (map::MEADOW, Stop::Bugs),
            (map::HEDGEROW, Stop::Forage),
            (FAR_FALLS, Stop::Falls),
            (LOG, Stop::Rest),
        ] {
            let carried = vec!["pinecone", "geode"];
            let start = Start {
                place,
                stop,
                everyone: &party,
                players: party.to_vec(),
                close_pair: false,
                light: 120.0,
                full: LIGHT,
                basket: carried.clone(),
                influence: &influence,
                droughts: Droughts::default(),
                found: &nothing,
                fish: &nothing,
                bugs: &nothing,
                hilltop: &hilltop,
                map: &map,
                daylight: Daylight::default(),
                seed: 9,
            };
            let mut leg = Leg::start(&cast, start, 0.0).unwrap();
            let mut now = 0.0;
            while !leg.over() {
                now += 1.0 / 30.0;
                let input = leg.steady(now);
                leg.tick(&cast, input, now);
                if now > 150.0 && !leg.leaving() {
                    leg.move_on(now);
                }
                assert!(now < 400.0, "{stop:?} never ended");
            }
            let basket = leg.basket();
            assert_eq!(
                &basket[..2],
                &carried[..],
                "{stop:?} lost what it was given"
            );
            let from = |catalogue: &[finds::Find], id: &str| catalogue.iter().any(|f| f.id == id);
            for id in &basket[2..] {
                let right = match stop {
                    Stop::Rummage | Stop::Fish | Stop::Bugs => from(&finds::CATALOGUE, id),
                    Stop::Forage => from(&finds::FORAGED, id),
                    Stop::Falls => from(&finds::AFAR, id),
                    _ => false,
                };
                assert!(right, "{id} came from {stop:?}");
            }
            match stop {
                Stop::Rest => assert_eq!(leg.light(), 120.0, "a rest's cost is the expedition's"),
                _ => assert!(leg.light() < 120.0, "{stop:?} took no light"),
            }
            if matches!(stop, Stop::Rummage | Stop::Falls) {
                assert!(basket.len() > 2, "a steady hand found nothing at {stop:?}");
            }
            let (fish, bugs) = leg.caught();
            assert!(
                fish.iter()
                    .all(|(id, _, _)| crate::fishing::fish::fish(id).is_some())
            );
            assert!(
                bugs.iter()
                    .all(|(id, _, _)| crate::meadow::bugs::bug(id).is_some())
            );
        }
    }

    #[test]
    fn ordinary_outings_start_their_own_day_with_an_empty_basket() {
        use crate::{falls, fishing, hedgerow, meadow, woods};
        let cast = sample();
        let party = vec![cast.members[0].id];
        let lent = Influence {
            light: 6.0,
            ..Influence::default()
        };
        let mut ground = woods::open(&cast, &party, 0.0);
        let rummage = woods::rummage::Rummage::new(
            &mut ground,
            woods::rummage::Outset {
                party: party.clone(),
                drought: 0,
                close_pair: false,
                influence: lent.clone(),
                beckons: false,
                seed: 1,
            },
            |_| false,
            0.0,
        );
        assert_eq!(rummage.light(), woods::rummage::LIGHT + 6.0);
        assert_eq!(rummage.light_left(), 1.0);
        assert!(rummage.basket().is_empty());
        let mut pool = fishing::open(&cast, &party, 0.0);
        let angling = fishing::angling::Angling::new(
            &mut pool,
            fishing::angling::Outset {
                party: party.clone(),
                drought: 0,
                influence: lent.clone(),
                seed: 1,
            },
            |_| false,
            |_| false,
            0.0,
        );
        assert_eq!(angling.light(), fishing::angling::LIGHT + 6.0);
        assert_eq!(angling.light_left(), 1.0);
        assert!(angling.basket().is_empty());
        let mut field = meadow::open(&cast, &party, 0.0, &Arrangement::new());
        let hunt = meadow::catching::Hunt::new(
            &mut field,
            meadow::catching::Outset {
                party: party.clone(),
                drought: 0,
                close_pair: false,
                influence: lent.clone(),
                seed: 1,
            },
            |_| false,
            |_| false,
            0.0,
        );
        assert_eq!(hunt.light(), meadow::catching::LIGHT + 6.0);
        assert_eq!(hunt.light_left(), 1.0);
        assert!(hunt.basket().is_empty());
        let mut lane = hedgerow::open(&cast, &party, 0.0, &Arrangement::new());
        let foray = hedgerow::foraging::Foray::new(
            &mut lane,
            hedgerow::foraging::Outset {
                party: party.clone(),
                drought: 0,
                close_pair: false,
                influence: lent.clone(),
                seed: 1,
            },
            |_| false,
            0.0,
        );
        assert_eq!(foray.light(), hedgerow::foraging::LIGHT + 6.0);
        assert_eq!(foray.light_left(), 1.0);
        assert!(foray.basket().is_empty());
        let mut pool = falls::open(&cast, &party, 0.0);
        let wading = falls::wading::Wading::new(
            &mut pool,
            falls::wading::Outset {
                party,
                drought: 0,
                close_pair: false,
                influence: lent,
                seed: 1,
            },
            |_| false,
            0.0,
        );
        assert_eq!(wading.light(), falls::wading::LIGHT + 6.0);
        assert!(wading.basket().is_empty());
    }

    #[test]
    fn the_basket_holds_six_across_stops_and_anything_over_must_be_made_room_for() {
        let cast = sample();
        let tansy = named(&cast, "Tansy");
        let mut trip = setting_off(&cast, vec![tansy], Known::default());
        let now = arrived(&mut trip, &cast);
        trip.basket = [
            "pinecone",
            "geode",
            "jay_feather",
            "old_nest",
            "mussel_shell",
            "sun_coin",
        ]
        .into_iter()
        .map(|find| Carried { find, by: tansy })
        .collect();
        let now = walk(&mut trip, &cast, GLADE, now);
        let mut now = stop_here(&mut trip, &cast, now);
        assert_eq!(
            trip.phase(),
            Phase::MakingRoom,
            "a full basket took more without asking"
        );
        assert_eq!(trip.basket().len(), BASKET + 1);
        assert!(trip.take_events().contains(&Event::TooFull));
        // Nothing goes on until something is left: not the way on, not another stop.
        assert!(!trip.go(map::SIGNPOST, now));
        assert!(!trip.stop(&cast, now));
        let new = trip.basket()[BASKET].find;
        assert!(trip.leave_behind(0, now), "leave the pinecone");
        assert_eq!(trip.phase(), Phase::Choosing);
        assert_eq!(trip.basket().len(), BASKET);
        assert!(finds_of(&trip).contains(&new), "the new find was kept");
        assert!(!finds_of(&trip).contains(&"pinecone"));
        // Between stops the basket never holds more than six, however the day goes.
        for place in [map::SIGNPOST, POOL, map::SIGNPOST, FAR_FALLS] {
            now = walk(&mut trip, &cast, place, now);
            if trip.can_stop() {
                now = stop_here(&mut trip, &cast, now);
                while trip.phase() == Phase::MakingRoom {
                    trip.leave_behind(0, now);
                }
            }
            assert!(trip.basket().len() <= BASKET);
        }
    }

    #[test]
    fn nothing_in_the_basket_is_ever_lost() {
        let cast = sample();
        let party = vec![
            named(&cast, "Button"),
            named(&cast, "Pip"),
            named(&cast, "Mochi"),
        ];
        let mut trip = setting_off(&cast, party, Known::default());
        let mut now = arrived(&mut trip, &cast);
        // Every kind of stop with a steady hand: whatever was in the basket going in is still
        // there coming out, unless the person put it back.
        for place in [map::HEDGEROW, GLADE, map::SIGNPOST, POOL, FAR_FALLS] {
            now = walk(&mut trip, &cast, place, now);
            if !trip.can_stop() {
                continue;
            }
            let before = finds_of(&trip);
            assert!(trip.stop(&cast, now));
            let mut put_back = Vec::new();
            let stopped = now;
            while trip.phase() == Phase::Stopped {
                assert!(now < stopped + 300.0, "stuck at {}", PLACES[place].name);
                now += 1.0 / 30.0;
                let input = trip
                    .leg_mut()
                    .map(|leg| leg.steady(now))
                    .unwrap_or_default();
                trip.tick(&cast, input, now);
                for event in trip.take_events() {
                    if let Event::Leg(LegEvent::Forage(
                        crate::hedgerow::foraging::Event::PutBack { find },
                    )) = event
                    {
                        put_back.push(find);
                    }
                }
            }
            let mut after = finds_of(&trip);
            for find in before {
                if let Some(at) = after.iter().position(|kept| *kept == find) {
                    after.remove(at);
                } else {
                    let at = put_back.iter().position(|left| *left == find);
                    assert!(at.is_some(), "{find} was lost at {}", PLACES[place].name);
                    put_back.remove(at.unwrap_or(0));
                }
            }
            while trip.phase() == Phase::MakingRoom {
                trip.leave_behind(trip.basket().len() - 1, now);
            }
        }
        assert!(!trip.basket().is_empty(), "nothing to lose");
        // At dusk, partway through a stop and leaving the Woods: it all goes home.
        let before = finds_of(&trip);
        trip.stops.clear();
        trip.light = 1.0;
        assert!(trip.stop(&cast, now));
        for _ in 0..90 {
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
        }
        trip.close_leg(now);
        for find in &before {
            assert!(finds_of(&trip).contains(find), "{find} was lost at dusk");
        }
        trip.head_home(now);
        play(&mut trip, &cast, now, |trip| trip.phase() == Phase::Over);
        for find in &before {
            assert!(
                finds_of(&trip).contains(find),
                "{find} was lost on the way home"
            );
        }
    }

    #[test]
    fn the_far_falls_are_found_out_of_the_mist_and_anyone_on_their_own_brings_something_home() {
        let cast = sample();
        for member in &cast.members {
            let mut trip = setting_off(&cast, vec![member.id], Known::default());
            assert!(
                !trip.sighted(FAR_FALLS),
                "the falls showed before anyone went"
            );
            let mut now = arrived(&mut trip, &cast);
            for place in [GLADE, map::SIGNPOST, FAR_FALLS] {
                now = walk(&mut trip, &cast, place, now);
            }
            assert!(trip.sighted(FAR_FALLS));
            assert!(
                trip.take_events()
                    .contains(&Event::Found { place: FAR_FALLS })
            );
            stop_here(&mut trip, &cast, now);
            assert!(
                finds_of(&trip).iter().any(|id| finds::is_from_afar(id)),
                "{} brought nothing home from the falls",
                member.name
            );
        }
        // Seen from the telescope on the Hilltop, or been to before, they show from the start.
        let lens = Known {
            hilltop: Arrangement::from([(3, Standing::from("brass_lens"))]),
            ..Known::default()
        };
        let mut trip = setting_off(&cast, vec![cast.members[0].id], lens);
        assert!(trip.sighted(FAR_FALLS));
        assert!(
            trip.take_events()
                .contains(&Event::Spied { place: FAR_FALLS })
        );
        let built = Known {
            hilltop: Arrangement::from([(3, Standing::built("great_telescope"))]),
            ..Known::default()
        };
        assert!(setting_off(&cast, vec![cast.members[0].id], built).sighted(FAR_FALLS));
        let been = Known {
            far_places: BTreeSet::from(["far_falls".to_owned()]),
            ..Known::default()
        };
        let mut trip = setting_off(&cast, vec![cast.members[0].id], been);
        assert!(trip.sighted(FAR_FALLS));
        assert!(
            !trip
                .take_events()
                .contains(&Event::Spied { place: FAR_FALLS })
        );
    }

    #[test]
    fn someone_who_reads_old_things_reads_the_signpost_and_the_old_way_opens() {
        let cast = sample();
        let mochi = named(&cast, "Mochi");
        let mut trip = setting_off(&cast, vec![mochi], Known::default());
        trip.company.reader = Some(mochi);
        let mut now = arrived(&mut trip, &cast);
        now = walk(&mut trip, &cast, GLADE, now);
        assert!(trip.way_to(FAR_FALLS).is_none());
        walk(&mut trip, &cast, map::SIGNPOST, now);
        assert!(trip.read);
        assert!(
            trip.sighted(FAR_FALLS),
            "the signpost says where the old way goes"
        );
        assert!(trip.take_events().contains(&Event::Read {
            who: mochi,
            place: FAR_FALLS
        }));
        let old_way = trip.way_to(FAR_FALLS).unwrap();
        assert!(matches!(
            PATHS[old_way.path].way,
            Way::Shortcut(_, Opener::Reader)
        ));
        // Nobody to read it: the long way round is all there is, into the mist.
        let mut plain = setting_off(&cast, vec![mochi], Known::default());
        let mut now = arrived(&mut plain, &cast);
        now = walk(&mut plain, &cast, GLADE, now);
        walk(&mut plain, &cast, map::SIGNPOST, now);
        assert!(!plain.read && !plain.sighted(FAR_FALLS));
        let long_way = plain.way_to(FAR_FALLS).unwrap();
        assert_eq!(PATHS[long_way.path].way, Way::Open);
        assert!(old_way.cost < long_way.cost);
    }

    #[test]
    fn with_reduced_motion_the_way_is_pointed_out_and_then_they_are_there() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let party = vec![named(&cast, "Mochi"), named(&cast, "Pip")];
        let mut trip = setting_off(&cast, party.clone(), Known::default());
        let now = arrived(&mut trip, &cast);
        let start = trip.map.position(party[0]).unwrap();
        assert!(trip.go(GLADE, now));
        let end = formation(PLACES[GLADE].at, 0);
        let mut later = now;
        let mut seen = Vec::new();
        while matches!(trip.phase(), Phase::Walking { .. }) {
            later += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), later);
            seen.push(trip.map.position(party[0]).unwrap());
            assert!(later < now + 5.0);
        }
        assert!(
            seen.iter().all(|at| *at == start || *at == end),
            "caught between the edge and the glade"
        );
        assert!(
            later - now >= HELD - 0.05,
            "the way wasn't pointed out first"
        );
        // And at the picnic, everyone stays sat where they are.
        let later = walk(&mut trip, &cast, LOG, later);
        assert!(trip.stop(&cast, later));
        let seats: Vec<(f32, f32)> = party
            .iter()
            .map(|id| trip.ground_mut().position(*id).unwrap())
            .collect();
        let mut now = later;
        while trip.phase() == Phase::Stopped {
            assert!(now < later + 60.0, "the rest never ended");
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
            if let Some(leg) = trip.leg() {
                for (id, seat) in party.iter().zip(&seats) {
                    assert_eq!(leg.ground.position(*id), Some(*seat), "someone got up");
                }
            }
        }
    }

    #[test]
    fn a_rest_on_the_log_takes_a_little_of_the_day_and_everyone_takes_it_their_own_way() {
        let cast = sample();
        let (fig, mochi, pip) = (
            named(&cast, "Fig"),
            named(&cast, "Mochi"),
            named(&cast, "Pip"),
        );
        let mut trip = setting_off(&cast, vec![fig, mochi, pip], Known::default());
        let mut now = arrived(&mut trip, &cast);
        now = walk(&mut trip, &cast, GLADE, now);
        now = walk(&mut trip, &cast, LOG, now);
        let (light, basket) = (trip.light(), finds_of(&trip));
        trip.take_events();
        assert!(trip.stop(&cast, now));
        let mut events = trip.take_events();
        let mut later = now;
        while trip.phase() == Phase::Stopped {
            assert!(later < now + 60.0, "the rest never ended");
            later += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), later);
            events.extend(trip.take_events());
        }
        assert!((light - trip.light() - REST_COST).abs() < 0.001);
        assert_eq!(finds_of(&trip), basket);
        assert!(later - now >= picnic::REST_SECS - 0.1);
        let rest: Vec<picnic::Event> = events
            .iter()
            .filter_map(|event| match event {
                Event::Leg(LegEvent::Rest(event)) => Some(*event),
                _ => None,
            })
            .collect();
        assert!(
            rest.contains(&picnic::Event::Dozed { who: fig }),
            "{rest:?}"
        );
        assert!(
            rest.contains(&picnic::Event::Snuggled { a: mochi, b: pip })
                || rest.contains(&picnic::Event::Snuggled { a: pip, b: mochi }),
            "{rest:?}"
        );
        assert!(!trip.can_stop(), "rested here already today");
    }

    #[test]
    fn each_stop_is_a_short_leg_of_its_own_activity() {
        let cast = sample();
        let mut trip = setting_off(&cast, vec![named(&cast, "Mochi")], Known::default());
        let mut now = arrived(&mut trip, &cast);
        // A few searches in the glade, then on.
        now = walk(&mut trip, &cast, GLADE, now);
        assert!(trip.stop(&cast, now));
        let mut searched = 0;
        let stopped = now;
        while trip.phase() == Phase::Stopped {
            assert!(now < stopped + 200.0, "the glade was never left");
            now += 1.0 / 30.0;
            let input = trip
                .leg_mut()
                .map(|leg| leg.steady(now))
                .unwrap_or_default();
            trip.tick(&cast, input, now);
            searched += trip
                .take_events()
                .iter()
                .filter(|event| {
                    matches!(
                        event,
                        Event::Leg(LegEvent::Rummage(
                            crate::woods::rummage::Event::Got { .. }
                                | crate::woods::rummage::Event::Nothing { .. }
                        ))
                    )
                })
                .count();
        }
        assert_eq!(searched as u32, legs::SEARCHES);
        // A cast or two at the pool: once they are cast, a click on the water casts nothing.
        now = walk(&mut trip, &cast, map::SIGNPOST, now);
        now = walk(&mut trip, &cast, POOL, now);
        assert!(trip.stop(&cast, now));
        let ready = |trip: &Expedition| {
            matches!(
                trip.leg().map(|leg| &leg.play),
                Some(Play::Fish { angling, .. })
                    if angling.phase() == crate::fishing::angling::Phase::Ready
            )
        };
        let stopped = now;
        while !ready(&trip) {
            assert!(now < stopped + 30.0, "never ready to cast");
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
        }
        let leg = trip.leg_mut().unwrap();
        if let Play::Fish { casts, .. } = &mut leg.play {
            *casts = legs::CASTS;
        }
        leg.click(Some((200.0, 112.0)), now);
        assert!(ready(&trip), "cast a third time");
        // And the next tick sees the leg is done, and the party back to the map.
        let stopped = now;
        while trip.phase() == Phase::Stopped {
            assert!(now < stopped + 30.0, "the pool was never left");
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
        }
    }

    #[test]
    fn at_the_hedgerow_one_patch_is_picked_and_then_on() {
        let cast = sample();
        let mut trip = setting_off(&cast, vec![named(&cast, "Button")], Known::default());
        let mut now = arrived(&mut trip, &cast);
        now = walk(&mut trip, &cast, map::HEDGEROW, now);
        assert!(trip.stop(&cast, now));
        let (first, other) = (0, 6);
        let click = |trip: &mut Expedition, patch: usize, now: f32| {
            let (x, y) = crate::hedgerow::PATCHES[patch].stand;
            trip.leg_mut().unwrap().click(Some((x, y - 8.0)), now);
        };
        let foray = |trip: &Expedition| match trip.leg().map(|leg| &leg.play) {
            Some(Play::Forage { foray }) => (foray.phase(), foray.at()),
            _ => panic!("the leg ended"),
        };
        // Down the lane, then to one patch.
        while foray(&trip).0 != crate::hedgerow::foraging::Phase::Choosing {
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
            assert!(now < 100.0);
        }
        click(&mut trip, first, now);
        while foray(&trip).1.is_none() {
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
            assert!(now < 200.0);
        }
        click(&mut trip, other, now);
        for _ in 0..60 {
            now += 1.0 / 30.0;
            trip.tick(&cast, Input::default(), now);
        }
        assert_eq!(foray(&trip).1, Some(first), "it went on to another patch");
    }

    #[test]
    fn whoever_is_best_at_a_stop_leads_it_and_whoever_opens_something_there_comes_too() {
        let cast = sample();
        let members: Vec<(Id, Character)> = ["Biscuit", "Mochi", "Fig", "Pip"]
            .iter()
            .map(|name| {
                let id = named(&cast, name);
                (id, Character::of(cast.member(id).unwrap()))
            })
            .collect();
        let (biscuit, pip) = (members[0].0, members[3].0);
        // Impulsive Biscuit is never the one with the rod.
        assert_ne!(legs::players(Stop::Fish, &members)[0], biscuit);
        // At the glade a little one comes along for the crevice, over a plainer helper.
        let glade = legs::players(Stop::Rummage, &members[..3]);
        assert!(!glade.contains(&pip));
        let glade = legs::players(Stop::Rummage, &members);
        assert_eq!(glade.len(), 2);
        assert!(glade.contains(&pip), "{glade:?}");
        // At the falls and on the log, everyone.
        assert_eq!(legs::players(Stop::Rest, &members).len(), 4);
        assert_eq!(legs::players(Stop::Falls, &members).len(), 4);
    }

    #[test]
    fn the_secret_never_shows_on_an_expedition() {
        let cast = sample();
        let party = [cast.members[0].id];
        let map = scenery::map(&Shown {
            places: [true; PLACES.len()],
            paths: [true; PATHS.len()],
        });
        let hilltop = Arrangement::from([(3, Standing::from("brass_lens"))]);
        let influence = Influence::default();
        let nothing = |_: &str| false;
        // Late in the expedition's day, lower than the glint ever needs, in the glade.
        let start = Start {
            place: GLADE,
            stop: Stop::Rummage,
            everyone: &party,
            players: party.to_vec(),
            close_pair: false,
            light: 4.0,
            full: LIGHT,
            basket: Vec::new(),
            influence: &influence,
            droughts: Droughts::default(),
            found: &nothing,
            fish: &nothing,
            bugs: &nothing,
            hilltop: &hilltop,
            map: &map,
            daylight: Daylight::default(),
            seed: 2,
        };
        let mut leg = Leg::start(&cast, start, 0.0).unwrap();
        let mut now = 0.0;
        while !leg.over() && now < 60.0 {
            now += 1.0 / 30.0;
            for event in leg.tick(&cast, Input::default(), now) {
                assert!(
                    !matches!(
                        event,
                        LegEvent::Rummage(
                            crate::woods::rummage::Event::Glint
                                | crate::woods::rummage::Event::Beckoned
                        )
                    ),
                    "the glint showed on an expedition"
                );
            }
        }
    }
}
