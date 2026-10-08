//! A treasure hunt: following a torn map off along the old track, with a companion or two, until
//! the chest is dug up or the light goes.
//!
//! The person plays. At each fork the map's line says which way, and the person matches it to
//! the ways and what stands beside them, and chooses. The right way leads on to the next fork;
//! a wrong one costs light and ends in a nook with a heap in it, which can be scavenged before
//! going back (see `scavenging`), so it is never wasted. At the end is the old milestone and four
//! places that might be dug: the right one has the chest, and the treasure in it comes home in
//! the chest, beside the basket. If the light goes first, the map is kept for another day, and
//! the way is the same.
//!
//! Who came along matters (see `crew`): a scholar reads the words that have faded, an explorer
//! has a hunch at every fork, right more often than not, someone suspicious doubts aloud before a
//! wrong turn (and now and then before a right one), and someone curious spots the landmark the
//! map names when it stands far off down a way.

use super::crew::{self, Crew};
use super::landmarks;
use super::routes::{Dir, Fork, Route, Spot};
use super::scavenging::{self, BASKET, Scavenge};
use super::scenery;
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{DROUGHT, Find, NOVELTY, SCAVENGED, Tier, art, drawn_to};
use crate::paint::{blit, put, rect, rgb, rgba};
use crate::playground::{Playground, Prop, distance};
use crate::woods::Influence;
use crate::woods::rummage::{self, delighted, dim};
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{Gesture, TemperamentKind};

/// The light a hunt starts with, before the Hilltop lends any.
pub const LIGHT: f32 = 100.0;
/// What following a way costs, coming back from a wrong one, and digging.
const WAY_COST: f32 = 9.0;
const BACK_COST: f32 = 7.0;
const DIG_COST: f32 = 6.0;
/// How long a dig takes, and the chest takes to open.
const DIG_SECS: f32 = 1.6;
const OPEN_SECS: f32 = 2.4;
/// How often an explorer's hunch is right.
const HUNCH: f32 = 0.75;
/// How likely someone suspicious is to doubt a wrong way, and a right one.
const DOUBT_WRONG: f32 = 0.8;
const DOUBT_RIGHT: f32 = 0.25;
/// Where the party stands at a fork, coming up to it, and at the dig.
pub const START: (f32, f32) = (192.0, 200.0);

/// What a hunt sets out with.
pub struct Outset {
    /// Who came along; the first leads.
    pub party: Vec<Id>,
    pub close_pair: bool,
    /// What the Hilltop lends the Woods.
    pub influence: Influence,
    /// The map being followed.
    pub map: u64,
    /// Chests in a row that held nothing new.
    pub drought: u32,
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Walking up to the first fork.
    Arriving {
        since: f32,
    },
    /// At a fork, reading the map.
    Reading {
        fork: usize,
    },
    /// Off along a way.
    Going {
        fork: usize,
        way: usize,
        since: f32,
    },
    /// In the nook at the end of a wrong way, at its heap.
    Nook {
        fork: usize,
        way: usize,
    },
    /// At the milestone, choosing where to dig.
    AtDig,
    Digging {
        spot: usize,
        since: f32,
    },
    /// The chest dug up and opening.
    Opening {
        since: f32,
    },
    Leaving {
        since: f32,
        why: Ending,
    },
    Over,
}

/// Why a hunt ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    /// The chest was found.
    Found,
    Dusk,
    Chose,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// At a fork.
    Fork {
        fork: usize,
    },
    /// A scholar read the faded words.
    Read {
        who: Id,
    },
    /// Someone curious spotted the landmark the map names, far off.
    Spotted {
        who: Id,
        way: usize,
    },
    /// An explorer has a hunch.
    Hunch {
        who: Id,
        way: usize,
    },
    /// Someone suspicious isn't sure.
    Doubt {
        who: Id,
        way: usize,
    },
    /// That way has been tried already.
    Tried {
        way: usize,
    },
    /// A wrong way: it ends at a heap.
    Nook,
    /// Back at the fork from a nook.
    Back,
    /// Something happened at the nook's heap.
    Heap(scavenging::Event),
    /// At the old milestone.
    Milestone,
    /// Dug, and nothing there.
    Nothing {
        spot: usize,
    },
    /// Dug, and there is the chest.
    Chest,
    /// What was in it.
    Treasure {
        find: &'static str,
    },
    Also {
        find: &'static str,
    },
    Leaving(Ending),
}

/// What a careful reader would do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Way(usize),
    Back,
    Dig(usize),
    Home,
    Wait,
}

pub struct Hunt {
    hour_dark: f32,
    route: Route,
    map: u64,
    crew: Crew,
    phase: Phase,
    light: f32,
    full: f32,
    basket: Vec<&'static str>,
    /// What came out of the chest, carried home in it.
    chest: Vec<&'static str>,
    /// Torn maps found in the nooks' heaps, by who found each.
    maps: Vec<Id>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    /// Ways tried at each fork.
    tried: Vec<Vec<bool>>,
    /// The way someone doubted, if the person hasn't gone that way regardless.
    doubted: Option<usize>,
    /// Each fork's hunch, once had.
    hunches: Vec<Option<usize>>,
    dug: Vec<bool>,
    /// The heap at the end of a wrong way, while the party is there.
    nook: Option<Scavenge>,
    found_before: Vec<&'static str>,
    drought: u32,
    /// The view is being changed: from what, to what.
    view: View,
}

/// What the person is looking at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Fork(usize),
    Nook,
    Dig,
}

impl Hunt {
    /// Sets out with the party along the way the map begins, in `ground` (laid out with
    /// `layout`). `found_before` says which finds the colony has already.
    pub fn new(
        ground: &mut Playground,
        outset: Outset,
        found_before: impl Fn(&str) -> bool,
        now: f32,
    ) -> Self {
        let Outset {
            party,
            close_pair,
            influence,
            map,
            drought,
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let crew = Crew::new(party, characters, close_pair);
        let route = Route::of(map);
        let full = LIGHT + influence.light;
        let mut hunt = Self {
            hour_dark: 0.0,
            tried: route
                .forks
                .iter()
                .map(|fork| vec![false; fork.ways.len()])
                .collect(),
            hunches: vec![None; route.forks.len() + 1],
            dug: vec![false; route.dig.spots.len()],
            route,
            map,
            crew,
            phase: Phase::Arriving { since: now },
            light: full,
            full,
            basket: Vec::new(),
            chest: Vec::new(),
            maps: Vec::new(),
            events: Vec::new(),
            dice: Dice::new(seed),
            reduce_motion: ground.reduce_motion(),
            doubted: None,
            nook: None,
            found_before: SCAVENGED
                .iter()
                .map(|find| find.id)
                .filter(|id| found_before(id))
                .collect(),
            drought,
            view: View::Fork(0),
        };
        hunt.show(ground, View::Fork(0), now);
        hunt
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.crew.party
    }

    pub fn map(&self) -> u64 {
        self.map
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    /// What came home: the basket, and whatever was in the chest.
    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    pub fn chest(&self) -> &[&'static str] {
        &self.chest
    }

    /// How far open the chest is, while it is opening: with reduced motion it is simply open.
    pub fn opening(&self, now: f32) -> Option<f32> {
        match self.phase {
            Phase::Opening { .. } if self.reduce_motion => Some(1.0),
            Phase::Opening { since } => Some(((now - since) / OPEN_SECS).clamp(0.0, 1.0)),
            _ => None,
        }
    }

    /// Torn maps found in the nooks along the way, by who found each.
    pub fn maps(&self) -> &[Id] {
        &self.maps
    }

    /// Whether the chest was found.
    pub fn dug_up(&self) -> bool {
        !self.chest.is_empty()
    }

    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
        if let Some(nook) = &mut self.nook {
            nook.set_hour_dark(darkness);
        }
    }

    /// The heap at the end of a wrong way, while the party is there.
    pub fn nook(&self) -> Option<&Scavenge> {
        self.nook.as_ref()
    }

    pub fn nook_mut(&mut self) -> Option<&mut Scavenge> {
        self.nook.as_mut()
    }

    /// Whether the faded words can be read: someone who reads them came along.
    pub fn reads_faded(&self) -> bool {
        self.crew.first(crew::scholar).is_some()
    }

    /// The fork the party is at, or heading from.
    pub fn fork(&self) -> Option<usize> {
        match self.phase {
            Phase::Reading { fork } | Phase::Going { fork, .. } | Phase::Nook { fork, .. } => {
                Some(fork)
            }
            _ => None,
        }
    }

    /// The line of the map for where the party is: the fork's, or the dig's.
    pub fn line(&self) -> Option<String> {
        let read = self.reads_faded();
        match self.phase {
            Phase::AtDig | Phase::Digging { .. } => Some(self.route.dig.line(read)),
            _ => self
                .fork()
                .and_then(|fork| self.route.forks.get(fork))
                .map(|fork| fork.line(read)),
        }
    }

    /// Whether a way at the fork has been tried already.
    pub fn tried(&self, fork: usize, way: usize) -> bool {
        self.tried
            .get(fork)
            .and_then(|tried| tried.get(way))
            .copied()
            .unwrap_or(false)
    }

    /// The way at the fork under a point: along its path, or by its end.
    pub fn way_at(&self, x: f32, y: f32) -> Option<usize> {
        let Phase::Reading { fork } = self.phase else {
            return None;
        };
        let fork = &self.route.forks[fork];
        fork.ways
            .iter()
            .enumerate()
            .map(|(index, dir)| {
                let near = scenery::way_path(*dir)
                    .iter()
                    .map(|point| distance((x, y), *point))
                    .fold(f32::MAX, f32::min);
                (index, near)
            })
            .filter(|(_, near)| *near < 22.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The dig spot under a point.
    pub fn spot_at(&self, x: f32, y: f32) -> Option<usize> {
        if !matches!(self.phase, Phase::AtDig) {
            return None;
        }
        self.route
            .dig
            .spots
            .iter()
            .enumerate()
            .map(|(index, spot)| (index, distance((x, y), (spot.at.0, spot.at.1 - 3.0))))
            .filter(|(_, near)| *near < 12.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// What to call the way under the pointer.
    pub fn describe_way(&self, way: usize) -> Option<String> {
        let fork = self.route.forks.get(self.fork()?)?;
        let dir = fork.ways.get(way)?;
        let mut words = vec![dir.name().to_owned()];
        if fork.stream == Some(way) {
            words.push("over the stream".to_owned());
        }
        if self.tried(self.fork()?, way) {
            words.push("been that way".to_owned());
        }
        Some(words.join(" \u{b7} "))
    }

    /// What to call a dig spot under the pointer.
    pub fn describe_spot(&self, spot: usize) -> Option<String> {
        let at = self.route.dig.spots.get(spot)?;
        let mut words = vec![
            format!(
                "{} paces from the milestone",
                super::routes::number(at.paces)
            ),
            format!("by {}", at.feature.name()),
        ];
        if self.dug[spot] {
            words.push("dug".to_owned());
        }
        Some(words.join(" \u{b7} "))
    }

    /// The person chose a way at the fork. Someone suspicious may doubt it first; going that way
    /// regardless is choosing it again.
    pub fn choose(&mut self, ground: &mut Playground, way: usize, now: f32) {
        let Phase::Reading { fork } = self.phase else {
            return;
        };
        let Some(dir) = self.route.forks[fork].ways.get(way).copied() else {
            return;
        };
        if self.tried(fork, way) {
            self.events.push(Event::Tried { way });
            return;
        }
        let right = way == self.route.forks[fork].right;
        if self.doubted != Some(way)
            && let Some(doubter) = self.crew.first(crew::suspicious)
        {
            let chance = if right { DOUBT_RIGHT } else { DOUBT_WRONG };
            if self.dice.chance(chance) {
                self.doubted = Some(way);
                self.events.push(Event::Doubt { who: doubter, way });
                let mut doubt = Beat::new(Gesture::Worry, ExpressionKind::Worried, 1.4);
                doubt.cue = Some(Cue::Exclaim);
                ground.direct(doubter, vec![Step::Beat(doubt)], now);
                return;
            }
        }
        self.doubted = None;
        self.light -= WAY_COST;
        let end = scenery::way_end(dir);
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let to = (end.0 + 10.0 * index as f32, end.1 + 3.0 * index as f32);
            ground.direct(id, vec![Step::Walk { to }], now);
        }
        self.phase = Phase::Going {
            fork,
            way,
            since: now,
        };
    }

    /// Back to the fork from a nook, with whatever the heap gave.
    pub fn back(&mut self, ground: &mut Playground, now: f32) {
        let Phase::Nook { fork, way } = self.phase else {
            return;
        };
        if let Some(nook) = self.nook.take() {
            self.basket = nook.basket().to_vec();
            self.maps.extend_from_slice(nook.maps());
            let (light, _) = nook.light();
            self.light = light;
        }
        self.light -= BACK_COST;
        self.tried[fork][way] = true;
        self.events.push(Event::Back);
        self.show(ground, View::Fork(fork), now);
        self.phase = Phase::Reading { fork };
        if self.light <= 0.0 {
            self.leave(ground, Ending::Dusk, now);
        }
    }

    /// The person chose where to dig.
    pub fn dig(&mut self, ground: &mut Playground, spot: usize, now: f32) {
        if !matches!(self.phase, Phase::AtDig) || spot >= self.route.dig.spots.len() {
            return;
        }
        if self.dug[spot] {
            self.events.push(Event::Nothing { spot });
            return;
        }
        self.light -= DIG_COST;
        let at = self.route.dig.spots[spot].at;
        let dig = Beat::new(Gesture::Crouch, ExpressionKind::Determined, DIG_SECS);
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let to = if index == 0 {
                (at.0 - 17.0, at.1 + 2.0)
            } else {
                ground.beside_point((at.0 - 17.0, at.1 + 4.0), -20.0)
            };
            ground.direct(
                id,
                vec![Step::Walk { to }, Step::FaceX(at.0), Step::Beat(dig)],
                now,
            );
        }
        self.phase = Phase::Digging { spot, since: now };
    }

    /// Calls the hunt to an end: everyone heads home with what they have, and the map is kept if
    /// the chest is still in the ground.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.come_out_of_the_nook();
            self.leave(ground, Ending::Chose, now);
        }
    }

    fn come_out_of_the_nook(&mut self) {
        if let Some(nook) = self.nook.take() {
            self.basket = nook.basket().to_vec();
            self.maps.extend_from_slice(nook.maps());
            self.light = nook.light().0;
        }
    }

    fn leave(&mut self, ground: &mut Playground, why: Ending, now: f32) {
        ground.reserve(self.crew.party.clone());
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let tired = why == Ending::Dusk
                && ground
                    .character(id)
                    .is_some_and(|c| c.kind == TemperamentKind::Lazybones);
            let mut steps = Vec::new();
            if tired {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Yawn,
                    ExpressionKind::Yawning,
                    1.0,
                )));
            }
            steps.push(Step::Walk {
                to: (START.0 - 18.0 * index as f32, 236.0),
            });
            ground.direct(id, steps, now);
        }
        self.events.push(Event::Leaving(why));
        self.phase = Phase::Leaving { since: now, why };
    }

    /// Changes what the party is looking at: a fork, a nook or the dig, with the party walking up
    /// into it from the front (or simply there, with reduced motion).
    fn show(&mut self, ground: &mut Playground, view: View, now: f32) {
        self.view = view;
        let backdrop = match view {
            View::Fork(fork) => scenery::fork(&self.route.forks[fork]),
            View::Nook => scenery::nook(),
            View::Dig => scenery::dig_site(&self.route.dig),
        };
        ground.set_backdrop(backdrop);
        ground.set_props(match view {
            View::Fork(fork) => landmarks::props(&self.route.forks[fork]),
            View::Nook => Vec::new(),
            View::Dig => landmarks::dig_props(&self.route.dig, &self.dug, None),
        });
        ground.reserve(self.crew.party.clone());
        let stand = match view {
            View::Nook => super::NOOK.stand,
            _ => START,
        };
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let to = if index == 0 {
                stand
            } else {
                ground.beside_point((stand.0, stand.1 + 3.0), -20.0)
            };
            ground.teleport(id, (to.0 - 4.0 + 8.0 * index as f32, 232.0));
            let face = match view {
                View::Nook => (super::NOOK.left + super::NOOK.width / 2) as f32,
                _ => 192.0,
            };
            ground.direct(id, vec![Step::Walk { to }, Step::FaceX(face)], now);
        }
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        if let Some(nook) = &mut self.nook {
            nook.tick(ground, now);
            for event in nook.take_events() {
                self.events.push(Event::Heap(event));
            }
            if matches!(
                nook.phase(),
                scavenging::Phase::Leaving { .. } | scavenging::Phase::Over
            ) {
                // The light went while they were at it.
                self.come_out_of_the_nook();
                self.leave(ground, Ending::Dusk, now);
                return;
            }
        }
        let leader = self.crew.party.first().copied();
        let walking = |ground: &Playground| leader.is_some_and(|id| ground.walking(id));
        let busy = |ground: &Playground| leader.is_some_and(|id| ground.busy(id));
        match self.phase {
            Phase::Arriving { since } => {
                if !walking(ground) || now - since > 6.0 {
                    self.arrive_at_fork(ground, 0, now);
                }
            }
            Phase::Going { fork, way, since } => {
                if !walking(ground) || now - since > 6.0 {
                    if way == self.route.forks[fork].right {
                        if fork + 1 < self.route.forks.len() {
                            self.show(ground, View::Fork(fork + 1), now);
                            self.phase = Phase::Arriving { since: now };
                            self.arrive_at_fork(ground, fork + 1, now);
                        } else {
                            self.show(ground, View::Dig, now);
                            self.phase = Phase::AtDig;
                            self.events.push(Event::Milestone);
                            self.hunch(ground, None, now);
                        }
                    } else {
                        self.show(ground, View::Nook, now);
                        let found = self.found_before.clone();
                        let seed = self.map ^ ((fork as u64) << 8 | way as u64).rotate_left(29);
                        let mut nook = Scavenge::nook(
                            ground,
                            self.crew.clone(),
                            (self.light, self.full),
                            self.basket.clone(),
                            move |id| found.contains(&id),
                            seed,
                            now,
                        );
                        nook.set_hour_dark(self.hour_dark);
                        self.nook = Some(nook);
                        self.events.push(Event::Nook);
                        self.phase = Phase::Nook { fork, way };
                    }
                }
            }
            Phase::Digging { spot, since } => {
                if busy(ground) && now - since < DIG_SECS + 6.0 {
                    return;
                }
                self.dug[spot] = true;
                if spot == self.route.dig.right {
                    self.events.push(Event::Chest);
                    self.open_chest(ground, now);
                    self.phase = Phase::Opening { since: now };
                } else {
                    self.events.push(Event::Nothing { spot });
                    ground.set_props(landmarks::dig_props(&self.route.dig, &self.dug, None));
                    self.phase = Phase::AtDig;
                }
            }
            Phase::Opening { since } => {
                ground.set_props(landmarks::dig_props(
                    &self.route.dig,
                    &self.dug,
                    self.opening(now),
                ));
                if now - since >= OPEN_SECS {
                    self.leave(ground, Ending::Found, now);
                }
            }
            Phase::Leaving { since, .. } => {
                let gone = self.crew.party.iter().all(|id| !ground.busy(*id));
                if gone || now - since > 8.0 {
                    self.phase = Phase::Over;
                }
            }
            Phase::Reading { .. } | Phase::Nook { .. } | Phase::AtDig | Phase::Over => {}
        }
        if self.light <= 0.0 && matches!(self.phase, Phase::Reading { .. } | Phase::AtDig) {
            self.leave(ground, Ending::Dusk, now);
        }
    }

    /// At a fork: the companions read the map their own ways.
    fn arrive_at_fork(&mut self, ground: &mut Playground, fork: usize, now: f32) {
        self.phase = Phase::Reading { fork };
        self.doubted = None;
        self.events.push(Event::Fork { fork });
        let at = &self.route.forks[fork];
        if at.faded.is_some()
            && let Some(scholar) = self.crew.first(crew::scholar)
        {
            self.events.push(Event::Read { who: scholar });
            let read = Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.2);
            ground.direct(scholar, vec![Step::Beat(read)], now);
        }
        if let Some((_, Spot::Far(way))) = at.sought()
            && let Some(spotter) = self.crew.first(crew::curious)
        {
            self.events.push(Event::Spotted { who: spotter, way });
            let mut point = Beat::new(Gesture::Reach, ExpressionKind::Curious, 1.0);
            point.cue = Some(Cue::Exclaim);
            let x = scenery::way_end(at.ways[way]).0;
            ground.direct(spotter, vec![Step::FaceX(x), Step::Beat(point)], now);
        }
        self.hunch(ground, Some(fork), now);
    }

    /// An explorer's hunch: which way, at a fork, or where to dig. Right more often than not.
    fn hunch(&mut self, ground: &mut Playground, fork: Option<usize>, now: f32) {
        let Some(explorer) = self.crew.first(crew::explorer) else {
            return;
        };
        let (count, right) = match fork {
            Some(fork) => (
                self.route.forks[fork].ways.len(),
                self.route.forks[fork].right,
            ),
            None => (self.route.dig.spots.len(), self.route.dig.right),
        };
        let slot = fork.unwrap_or(self.route.forks.len());
        let way = *self.hunches[slot].get_or_insert_with(|| {
            if self.dice.chance(HUNCH) {
                right
            } else {
                let wrong: Vec<usize> = (0..count).filter(|way| *way != right).collect();
                wrong[(self.dice.unit() * wrong.len() as f32) as usize % wrong.len()]
            }
        });
        if fork.is_some() {
            self.events.push(Event::Hunch { who: explorer, way });
            let x = scenery::way_end(self.route.forks[slot].ways[way]).0;
            let point = Beat::new(Gesture::Reach, ExpressionKind::Determined, 1.0);
            ground.direct(explorer, vec![Step::FaceX(x), Step::Beat(point)], now);
        } else {
            self.events.push(Event::Hunch { who: explorer, way });
        }
    }

    /// The explorer's hunch at a fork, or at the dig, if one came and has had one.
    pub fn hunch_at(&self, fork: Option<usize>) -> Option<usize> {
        self.crew.first(crew::explorer)?;
        self.hunches
            .get(fork.unwrap_or(self.route.forks.len()))
            .copied()
            .flatten()
    }

    /// Opens the chest: see `fill_chest`.
    fn open_chest(&mut self, ground: &mut Playground, now: f32) {
        let party: Vec<&Character> = self.crew.characters.iter().collect();
        let found = self.found_before.clone();
        let filled = fill_chest(
            &party,
            &|id| found.contains(&id),
            self.drought,
            &mut self.dice,
        );
        for (index, find) in filled.into_iter().enumerate() {
            self.chest.push(find.id);
            self.events.push(if index == 0 {
                Event::Treasure { find: find.id }
            } else {
                Event::Also { find: find.id }
            });
        }
        for id in self.crew.party.clone() {
            let Some(character) = self.crew.character(id).cloned() else {
                continue;
            };
            let beats = delighted(&character, Tier::Exceptional);
            ground.direct(id, beats.into_iter().map(Step::Beat).collect(), now);
        }
    }

    /// The scene: dimming as the light goes, the nook's heap, a hunch or a spotted landmark
    /// marked, the chest's treasure held up, and the basket and the chest in the corner.
    pub fn props(&self, now: f32) -> Vec<Prop> {
        self.nook
            .as_ref()
            .map(|nook| nook.props(now))
            .unwrap_or_default()
    }

    pub fn draw(&self, scene: &mut Canvas, hovered: Option<(usize, usize)>, now: f32) {
        if let Some(nook) = &self.nook {
            nook.draw(scene, hovered, now);
            self.draw_chest_slot(scene);
            return;
        }
        dim(scene, self.light, self.hour_dark);
        if let Phase::Reading { fork } = self.phase {
            let at = &self.route.forks[fork];
            if let Some((_, Spot::Far(way))) = at.sought()
                && self.crew.first(crew::curious).is_some()
            {
                let (x, y) = landmarks::far_point(at.ways.len(), at.ways[way]);
                mark(scene, (x as i32, y as i32 - 18), rgb(0xfff4c0));
            }
        }
        if let Phase::Opening { since } = self.phase
            && let Some(id) = self.chest.first()
        {
            let at = self.route.dig.spots[self.route.dig.right].at;
            let t = if self.reduce_motion {
                1.0
            } else {
                ((now - since) / 0.8).clamp(0.0, 1.0)
            };
            let (left, top) = (at.0 as i32 - 4, at.1 as i32 - 26 - (t * 8.0) as i32);
            rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
            blit(scene, &art::icon(id), left, top);
            for (dx, dy) in [(-5, -3), (11, 1), (3, -6), (12, -5)] {
                put(scene, left + dx, top + dy, rgb(0xfff4c0));
            }
        }
        self.draw_basket(scene);
    }

    /// The basket in the top right corner and the light under it, as on a scavenge, and the chest
    /// slot beside it.
    fn draw_basket(&self, scene: &mut Canvas) {
        rummage::draw_basket(scene, &self.basket, self.light_left());
        self.draw_chest_slot(scene);
    }

    /// Whatever came out of the chest, in its own slot left of the basket.
    fn draw_chest_slot(&self, scene: &mut Canvas) {
        if self.chest.is_empty() {
            return;
        }
        const SLOT: i32 = 11;
        let width = SLOT * BASKET as i32 + 3;
        let left = scene.width() as i32 - width - 4 - (SLOT * 2 + 6);
        rect(scene, left, 4, SLOT * 2 + 3, SLOT + 4, rgba(0x4a3018, 170));
        for (index, id) in self.chest.iter().enumerate() {
            blit(scene, &art::icon(id), left + 2 + index as i32 * SLOT, 6);
        }
    }

    /// What a careful reader would do next, for the renders and the tests: at a fork, the way the
    /// line names as far as it can be read, the explorer's hunch among those it might be, and
    /// never a way already tried; back from a nook at once; at the milestone, the place the line
    /// means, or the hunch among those it might.
    pub fn canny(&self) -> Move {
        let read = self.reads_faded();
        match self.phase {
            Phase::Reading { fork } => {
                let at = &self.route.forks[fork];
                let mut might: Vec<usize> = at
                    .might_be(read)
                    .into_iter()
                    .filter(|way| !self.tried(fork, *way))
                    .collect();
                if might.is_empty() {
                    might = (0..at.ways.len())
                        .filter(|way| !self.tried(fork, *way))
                        .collect();
                }
                let hunch = self.hunch_at(Some(fork)).filter(|way| might.contains(way));
                hunch
                    .or(might.first().copied())
                    .map_or(Move::Wait, Move::Way)
            }
            Phase::Nook { .. } => Move::Back,
            Phase::AtDig => {
                let might: Vec<usize> = self
                    .route
                    .dig
                    .might_be(read)
                    .into_iter()
                    .filter(|spot| !self.dug[*spot])
                    .collect();
                let hunch = self.hunch_at(None).filter(|spot| might.contains(spot));
                hunch
                    .or(might.first().copied())
                    .map_or(Move::Home, Move::Dig)
            }
            _ => Move::Wait,
        }
    }

    /// The direction of a way at the fork the party is at, for the tray.
    pub fn way_dir(&self, way: usize) -> Option<Dir> {
        let fork: &Fork = self.route.forks.get(self.fork()?)?;
        fork.ways.get(way).copied()
    }
}

/// What a chest holds: a treasure, rare or exceptional, the undiscovered ones more often and
/// something new for certain once the last chest held nothing new, leaning towards who came; and
/// something from the heaps with it.
pub fn fill_chest(
    party: &[&Character],
    found_before: &dyn Fn(&str) -> bool,
    drought: u32,
    dice: &mut Dice,
) -> Vec<&'static Find> {
    let weight = |find: &Find| {
        let novelty = if found_before(find.id) { 1.0 } else { NOVELTY };
        find.tier.weight() * drawn_to(find.leanings, party) * novelty
    };
    let treasures: Vec<&'static Find> = SCAVENGED
        .iter()
        .filter(|find| super::heap::in_chests(find))
        .collect();
    let new: Vec<&'static Find> = treasures
        .iter()
        .copied()
        .filter(|find| !found_before(find.id))
        .collect();
    let pool = if drought >= DROUGHT.saturating_sub(1) && !new.is_empty() {
        new
    } else {
        treasures
    };
    let mut chest = Vec::new();
    let weights: Vec<f32> = pool.iter().map(|find| weight(find)).collect();
    if let Some(treasure) = dice.weighted(&weights).map(|index| pool[index]) {
        chest.push(treasure);
    }
    let small: Vec<&'static Find> = SCAVENGED
        .iter()
        .filter(|find| find.tier <= Tier::Uncommon)
        .collect();
    let weights: Vec<f32> = small.iter().map(|find| weight(find)).collect();
    if let Some(also) = dice.weighted(&weights).map(|index| small[index]) {
        chest.push(also);
    }
    chest
}

/// A little marker over something far off: a pointing chevron.
fn mark(scene: &mut Canvas, (x, y): (i32, i32), color: formiga_art::Rgba) {
    for (dx, dy) in [(0, 0), (-1, -1), (1, -1), (-2, -2), (2, -2), (0, -1)] {
        put(scene, x + dx, y + dy, color);
    }
    put(scene, x, y + 1, rgba(0x2a2018, 160));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::{Cast, named, sample};
    use crate::finds;
    use crate::track;

    fn setting_off(cast: &Cast, names: &[&str], map: u64, seed: u64) -> (Playground, Hunt) {
        let party: Vec<Id> = names.iter().map(|name| named(cast, name)).collect();
        let mut ground = track::hunt_ground(cast, &party, 0.0);
        let outset = Outset {
            party,
            close_pair: false,
            influence: Influence::default(),
            map,
            drought: 0,
            seed,
        };
        let hunt = Hunt::new(&mut ground, outset, |_| false, 0.0);
        (ground, hunt)
    }

    fn run(
        ground: &mut Playground,
        hunt: &mut Hunt,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            hunt.tick(ground, now);
            events.extend(hunt.take_events());
        }
        events
    }

    /// Follows the hunt through, making `pick` at each fork and the dig, until it ends.
    fn play(
        cast: &Cast,
        names: &[&str],
        map: u64,
        seed: u64,
        pick: &mut dyn FnMut(&Hunt) -> Move,
    ) -> (Hunt, Vec<Event>) {
        let (mut ground, mut hunt) = setting_off(cast, names, map, seed);
        let mut events = Vec::new();
        let mut now = 0.0;
        while hunt.phase() != Phase::Over && now < 600.0 {
            events.extend(run(&mut ground, &mut hunt, cast, now, now + 0.5));
            now += 0.5;
            match pick(&hunt) {
                Move::Way(way) => hunt.choose(&mut ground, way, now),
                Move::Back => hunt.back(&mut ground, now),
                Move::Dig(spot) => hunt.dig(&mut ground, spot, now),
                Move::Home => hunt.head_home(&mut ground, now),
                Move::Wait => {}
            }
        }
        (hunt, events)
    }

    #[test]
    fn reading_the_map_right_leads_to_the_chest() {
        let cast = sample();
        for map in 0..12 {
            let (hunt, events) = play(&cast, &["Fig"], map, 1, &mut |hunt| {
                // Every line read in full, as if the faded words could be made out.
                match hunt.phase() {
                    Phase::Reading { fork } => Move::Way(hunt.route().forks[fork].right),
                    Phase::AtDig => Move::Dig(hunt.route().dig.right),
                    Phase::Nook { .. } => Move::Back,
                    _ => Move::Wait,
                }
            });
            assert!(hunt.dug_up(), "map {map}: {events:?}");
            assert!(events.contains(&Event::Leaving(Ending::Found)));
            let treasure = finds::find(hunt.chest()[0]).unwrap();
            assert!(treasure.tier >= Tier::Rare && finds::is_scavenged(treasure.id));
            assert!(
                !events.iter().any(|e| matches!(e, Event::Nook)),
                "a right way led astray"
            );
        }
    }

    #[test]
    fn a_wrong_way_costs_light_and_leads_to_a_heap_then_back_to_the_fork() {
        let cast = sample();
        let (mut ground, mut hunt) = setting_off(&cast, &["Button"], 3, 2);
        run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        let Phase::Reading { fork } = hunt.phase() else {
            panic!("{:?}", hunt.phase());
        };
        let wrong = (0..hunt.route().forks[fork].ways.len())
            .find(|way| *way != hunt.route().forks[fork].right)
            .unwrap();
        let light = hunt.light;
        hunt.choose(&mut ground, wrong, 8.0);
        let events = run(&mut ground, &mut hunt, &cast, 8.0, 16.0);
        assert!(events.contains(&Event::Nook), "{events:?}");
        assert!(matches!(hunt.phase(), Phase::Nook { .. }));
        assert!(
            hunt.nook().is_some_and(|nook| nook.heap_count() == 1),
            "a heap there"
        );
        hunt.back(&mut ground, 16.0);
        assert_eq!(hunt.phase(), Phase::Reading { fork });
        assert!(hunt.light < light - WAY_COST, "the detour cost light");
        assert!(hunt.tried(fork, wrong));
        hunt.choose(&mut ground, wrong, 17.0);
        assert!(hunt.take_events().contains(&Event::Tried { way: wrong }));
    }

    #[test]
    fn a_careful_reader_reaches_the_chest_more_often_than_a_guesser() {
        let cast = sample();
        let mut careful = 0;
        let mut guessing = 0;
        for map in 0..24 {
            let (hunt, _) = play(&cast, &["Fig"], map, map, &mut |hunt| hunt.canny());
            careful += usize::from(hunt.dug_up());
            let mut dice = Dice::new(map + 100);
            let (hunt, _) = play(&cast, &["Fig"], map, map, &mut |hunt| match hunt.phase() {
                Phase::Reading { fork } => {
                    let count = hunt.route().forks[fork].ways.len();
                    Move::Way((dice.unit() * count as f32) as usize % count)
                }
                Phase::AtDig => Move::Dig((dice.unit() * 4.0) as usize % 4),
                Phase::Nook { .. } => Move::Back,
                _ => Move::Wait,
            });
            guessing += usize::from(hunt.dug_up());
        }
        assert!(careful > guessing, "careful {careful}, guessing {guessing}");
        assert!(
            careful >= 20,
            "a careful reader found only {careful} of 24 chests"
        );
    }

    #[test]
    fn the_light_running_out_ends_the_hunt_without_the_chest() {
        let cast = sample();
        for right in [true, false] {
            let (mut ground, mut hunt) = setting_off(&cast, &["Fig"], 5, 3);
            run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
            let light = hunt.light;
            run(&mut ground, &mut hunt, &cast, 8.0, 14.0);
            assert_eq!(hunt.light, light, "standing at a fork costs nothing");
            hunt.light = 3.0;
            let fork = &hunt.route().forks[0];
            let way = if right {
                fork.right
            } else {
                (0..fork.ways.len()).find(|way| *way != fork.right).unwrap()
            };
            // Gone that way regardless of any doubt.
            hunt.choose(&mut ground, way, 14.0);
            hunt.choose(&mut ground, way, 14.0);
            let events = run(&mut ground, &mut hunt, &cast, 14.0, 40.0);
            assert!(events.contains(&Event::Leaving(Ending::Dusk)), "{events:?}");
            assert!(!hunt.dug_up());
            assert_eq!(hunt.phase(), Phase::Over);
        }
    }

    #[test]
    fn a_scholar_reads_the_faded_words() {
        let cast = sample();
        let map = (0..400)
            .find(|map| Route::of(*map).forks[0].faded.is_some())
            .unwrap();
        let (mut ground, mut hunt) = setting_off(&cast, &["Fig"], map, 4);
        run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        assert!(hunt.line().unwrap().contains('\u{2026}'));
        hunt.crew.characters[0].kind = TemperamentKind::Scholar;
        assert!(!hunt.line().unwrap().contains('\u{2026}'));
        let (mut ground, mut hunt) = hunt_with_scholar(&cast, map);
        let events = run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        assert!(events.iter().any(|e| matches!(e, Event::Read { .. })));
    }

    fn hunt_with_scholar(cast: &Cast, map: u64) -> (Playground, Hunt) {
        let party = vec![named(cast, "Fig")];
        let mut ground = track::hunt_ground(cast, &party, 0.0);
        let outset = Outset {
            party: party.clone(),
            close_pair: false,
            influence: Influence::default(),
            map,
            drought: 0,
            seed: 5,
        };
        let mut hunt = Hunt::new(&mut ground, outset, |_| false, 0.0);
        hunt.crew.characters[0].kind = TemperamentKind::Scholar;
        (ground, hunt)
    }

    #[test]
    fn an_explorer_has_a_hunch_right_more_often_than_not() {
        let cast = sample();
        let (mut right, mut hunches) = (0, 0);
        for map in 0..60 {
            let (mut ground, mut hunt) = setting_off(&cast, &["Tansy"], map, map);
            let events = run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
            for event in events {
                if let Event::Hunch { way, .. } = event {
                    hunches += 1;
                    right += usize::from(way == hunt.route().forks[0].right);
                }
            }
            let (mut ground, mut plain) = setting_off(&cast, &["Fig"], map, map);
            let events = run(&mut ground, &mut plain, &cast, 0.0, 8.0);
            assert!(!events.iter().any(|e| matches!(e, Event::Hunch { .. })));
        }
        assert_eq!(hunches, 60);
        assert!(right > 35 && right < 60, "right {right} of 60");
    }

    #[test]
    fn someone_suspicious_doubts_wrong_ways_more_than_right_ones_and_going_anyway_goes() {
        let cast = sample();
        let (mut wrong_doubts, mut right_doubts) = (0, 0);
        for map in 0..60 {
            for take_right in [false, true] {
                let (mut ground, mut hunt) = setting_off(&cast, &["Fig"], map, map * 2 + 1);
                run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
                let Phase::Reading { fork } = hunt.phase() else {
                    panic!();
                };
                let at = &hunt.route().forks[fork];
                let way = if take_right {
                    at.right
                } else {
                    (0..at.ways.len()).find(|w| *w != at.right).unwrap()
                };
                hunt.choose(&mut ground, way, 8.0);
                if hunt
                    .take_events()
                    .iter()
                    .any(|e| matches!(e, Event::Doubt { .. }))
                {
                    if take_right {
                        right_doubts += 1;
                    } else {
                        wrong_doubts += 1;
                    }
                    assert!(matches!(hunt.phase(), Phase::Reading { .. }), "it waited");
                    hunt.choose(&mut ground, way, 9.0);
                    assert!(
                        matches!(hunt.phase(), Phase::Going { .. }),
                        "going anyway goes"
                    );
                }
            }
        }
        assert!(
            wrong_doubts > right_doubts * 2,
            "{wrong_doubts} against {right_doubts}"
        );
        assert!(
            right_doubts > 0,
            "a suspicious one never doubts a right way"
        );
        // Nobody suspicious, nobody doubts.
        let (mut ground, mut hunt) = setting_off(&cast, &["Tansy"], 1, 1);
        run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        hunt.choose(&mut ground, 0, 8.0);
        assert!(
            !hunt
                .take_events()
                .iter()
                .any(|e| matches!(e, Event::Doubt { .. }))
        );
    }

    #[test]
    fn someone_curious_spots_a_far_landmark() {
        let cast = sample();
        let map = (0..400)
            .find(|map| matches!(Route::of(*map).forks[0].sought(), Some((_, Spot::Far(_)))))
            .expect("no map names something far off at its first fork");
        let (mut ground, mut hunt) = setting_off(&cast, &["Tansy"], map, 1);
        let events = run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        assert!(
            events.iter().any(|e| matches!(e, Event::Spotted { .. })),
            "{events:?}"
        );
        let (mut ground, mut hunt) = setting_off(&cast, &["Fig"], map, 1);
        let events = run(&mut ground, &mut hunt, &cast, 0.0, 8.0);
        assert!(!events.iter().any(|e| matches!(e, Event::Spotted { .. })));
    }

    #[test]
    fn a_chest_holds_something_new_after_a_dry_spell() {
        let cast = sample();
        let party = vec![named(&cast, "Fig")];
        for seed in 0..40 {
            let mut ground = track::hunt_ground(&cast, &party, 0.0);
            let outset = Outset {
                party: party.clone(),
                close_pair: false,
                influence: Influence::default(),
                map: seed,
                drought: DROUGHT,
                seed,
            };
            // Everything found but the golden axe.
            let mut hunt = Hunt::new(&mut ground, outset, |id| id != "golden_axe", 0.0);
            hunt.open_chest(&mut ground, 0.0);
            assert_eq!(hunt.chest()[0], "golden_axe");
        }
    }

    #[test]
    fn with_reduced_motion_the_chest_is_simply_open() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut hunt) = setting_off(&cast, &["Mochi"], 4, 4);
        let mut now = 0.0;
        while now < 300.0 {
            run(&mut ground, &mut hunt, &cast, now, now + 0.5);
            now += 0.5;
            match hunt.phase() {
                Phase::Reading { fork } => {
                    let right = hunt.route().forks[fork].right;
                    hunt.choose(&mut ground, right, now);
                }
                Phase::AtDig => hunt.dig(&mut ground, hunt.route().dig.right, now),
                Phase::Opening { .. } => break,
                _ => {}
            }
        }
        assert!(
            matches!(hunt.phase(), Phase::Opening { .. }),
            "never opened it"
        );
        assert_eq!(
            hunt.opening(now),
            Some(1.0),
            "the chest creaks open bit by bit"
        );
    }

    #[test]
    fn with_reduced_motion_the_party_is_simply_at_each_fork() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut hunt) = setting_off(&cast, &["Fig"], 2, 2);
        run(&mut ground, &mut hunt, &cast, 0.0, 0.2);
        assert!(
            matches!(hunt.phase(), Phase::Reading { .. }),
            "{:?}",
            hunt.phase()
        );
        assert_eq!(ground.position(named(&cast, "Fig")), Some(START));
        let right = hunt.route().forks[0].right;
        hunt.choose(&mut ground, right, 0.2);
        run(&mut ground, &mut hunt, &cast, 0.2, 0.5);
        assert!(
            matches!(hunt.phase(), Phase::Reading { fork: 1 }),
            "{:?}",
            hunt.phase()
        );
    }
}
