//! A bug hunt in the meadow: one companion carries the net, a second (if one came) helps, and the
//! person plays it, until the light goes.
//!
//! Each kind keeps to its own part of the meadow and gets about in its own way, and that way is
//! the thing to learn. Choose a bug, then hold to creep up on it and let go to keep still: moving
//! anywhere near it sets its nerves going, and too much sends it off. Once it is in reach of the
//! net, swing, but at the right moment. A butterfly basking with its wings open sees the net
//! coming; shut, it doesn't. A grasshopper that has started to chirp is about to jump. A bee is
//! only still while it is in a flower, a dragonfly while it hangs in the air, a firefly while it
//! is lit. A beetle can be had any time, if it hasn't been frightened. A swing at the wrong moment
//! sends the bug off, though now and then the net comes up with something else worth taking home.
//!
//! Who came along matters. A quiet companion hardly troubles a bug as it creeps; a quick one
//! closes in fast and is noticed for it. A playful one draws butterflies to see what it is doing.
//! A little one can creep right into the brambles. A dozy one, left still long enough, may find
//! something has landed on it. A curious one points out the rare bugs, and a second companion
//! helps, a close friend most of all. Caught bugs are looked at and let go, and remembered.

use super::art::{self, Pose};
use super::bugs::{self, Bug, Way};
use super::{GROUND, Haunt, walkable};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character};
use crate::dice::Dice;
use crate::finds::{self, Find, Kind, Tier};
use crate::paint::{ellipse, line, put, rgb, rgba};
use crate::playground::{Playground, Prop, distance};
use crate::track::crew;
use crate::woods::rummage::{delighted, dim};
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{Gesture, TemperamentKind};

/// The light an outing starts with, what it costs to creep and to swing, and how much goes each
/// second.
pub const LIGHT: f32 = 100.0;
const LIGHT_PER_SEC: f32 = 0.25;
const CREEP_COST: f32 = 0.04;
const SWING_COST: f32 = 2.0;
/// Below this much light, the dusk's bugs come out.
const DUSK: f32 = 40.0;
/// How long the net takes to come down.
const SWING_SECS: f32 = 0.3;
/// How long a caught bug is held up to look at before it is let go.
const ADMIRE_SECS: f32 = 2.2;
/// How long a frightened bug keeps out of the meadow, and how long a common one let go is away.
const AWAY_SECS: f32 = 9.0;
const RETURN_SECS: f32 = 18.0;
/// Where the net catches, from whoever holds it: this far ahead and up, and how far round.
const NET_AHEAD: f32 = 16.0;
const NET_UP: f32 = 20.0;
const NET_ROUND: (f32, f32) = (10.0, 14.0);
/// A butterfly's basking rhythm on a perch: wings open, then shut.
const WINGS_OPEN_SECS: f32 = 1.0;
const WINGS_SHUT_SECS: f32 = 0.7;
/// How long a grasshopper chirps before it jumps.
const CHIRP_SECS: f32 = 0.9;
/// A firefly's rhythm: lit, then dark.
const LIT_SECS: f32 = 1.3;
const DARK_SECS: f32 = 1.5;
/// How fast nerves settle again while nobody moves.
const CALMING: f32 = 0.35;
/// The chance a swing that misses brings up something else instead.
const NETTED: f32 = 0.1;
/// How far apart the one with the net and a helper keep, so neither stands over the other.
const HELPER_ROOM: f32 = 18.0;
/// How long a dozy companion must keep still for something to land on it.
const DOZE_SECS: f32 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Arriving {
        since: f32,
    },
    /// Choosing a bug, or stalking the one chosen.
    Hunting,
    /// The net coming down.
    Swinging {
        target: usize,
        since: f32,
    },
    /// A bug caught and held up, before it is let go.
    Caught {
        bug: &'static str,
        size: f32,
        since: f32,
    },
    /// Something other than a bug came up in the net.
    Netted {
        find: &'static str,
        since: f32,
    },
    Leaving {
        since: f32,
    },
    Over,
}

/// Why a swing missed, which is the lesson in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// It was on the move.
    InFlight,
    /// Wings open, it saw the net coming.
    WingsOpen,
    /// It had started to chirp, and jumped.
    Chirping,
    /// It went dark.
    Dark,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Stalking { bug: &'static str },
    Startled { bug: &'static str },
    Missed { bug: &'static str, why: Why },
    OutOfReach,
    Caught { bug: &'static str, size: f32 },
    Netted { find: &'static str },
    LandedOn { who: Id, bug: &'static str },
    Pointed { who: Id, bug: &'static str },
    Dusk,
    Leaving,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Flight {
    /// At a perch since then, until it moves on.
    Settled { since: f32, until: f32 },
    /// On its way to another perch. `fleeing` if frightened off, which nothing catches.
    Moving {
        from: (f32, f32),
        to: usize,
        since: f32,
        takes: f32,
        fleeing: bool,
    },
    /// Gone from the meadow, frightened off or let go, until then.
    Away { until: f32 },
    /// In the net.
    Netted,
    /// Let go for good.
    Gone,
}

#[derive(Clone, Debug)]
struct Flier {
    bug: &'static Bug,
    /// Millimetres across, within its kind's range.
    size: f32,
    perch: usize,
    pos: (f32, f32),
    facing_right: bool,
    flight: Flight,
    /// How rattled it is, from 0 to 1, when it goes.
    nerve: f32,
    /// When it was last frightened off, for a second fright sending it right away.
    startled_at: f32,
    /// The dusk's bugs are only out once the light has gone low.
    out: bool,
    /// Settled on a companion rather than a perch.
    riding: Option<Id>,
    /// Where in its own rhythm it is, so two of a kind keep different time.
    offset: f32,
}

impl Flier {
    fn here(&self) -> bool {
        self.out
            && !matches!(
                self.flight,
                Flight::Away { .. } | Flight::Netted | Flight::Gone
            )
    }

    /// Whether it is open to the net right now, and if not, why.
    fn open(&self, now: f32) -> Result<(), Why> {
        let t = now + self.offset;
        if self.bug.way == Way::Blink {
            let cycle = LIT_SECS + DARK_SECS;
            return if t.rem_euclid(cycle) < LIT_SECS {
                Ok(())
            } else {
                Err(Why::Dark)
            };
        }
        if self.riding.is_some() {
            return Ok(());
        }
        match self.flight {
            Flight::Settled { since, until } => match self.bug.way {
                Way::Flutter => {
                    if wings_open(now - since) {
                        Err(Why::WingsOpen)
                    } else {
                        Ok(())
                    }
                }
                Way::Hop if until - now < CHIRP_SECS => Err(Why::Chirping),
                _ => Ok(()),
            },
            Flight::Moving { fleeing: false, .. } if self.bug.way == Way::Crawl => Ok(()),
            _ => Err(Why::InFlight),
        }
    }

    /// The pose and frame it is drawn in.
    fn pose(&self, now: f32, reduce_motion: bool) -> (Pose, usize) {
        let beat = |pose: Pose, per_sec: f32| {
            let frames = art::frames(self.bug.id, pose).max(1);
            if reduce_motion {
                (pose, 0)
            } else {
                (pose, ((now + self.offset) * per_sec) as usize % frames)
            }
        };
        match self.flight {
            Flight::Settled { since, until } => match self.bug.way {
                Way::Flutter => (Pose::Settled, usize::from(!wings_open(now - since))),
                Way::Hop => (Pose::Settled, usize::from(until - now < CHIRP_SECS)),
                Way::Dart if self.bug.haunt.perches()[self.perch].1 > 112.0 => {
                    beat(Pose::Flying, 14.0)
                }
                Way::Blink => beat(Pose::Flying, 6.0),
                _ => (Pose::Settled, 0),
            },
            Flight::Moving { fleeing: false, .. } if self.bug.way == Way::Crawl => {
                beat(Pose::Crawling, 4.0)
            }
            _ => beat(Pose::Flying, 12.0),
        }
    }
}

fn wings_open(settled_for: f32) -> bool {
    settled_for.rem_euclid(WINGS_OPEN_SECS + WINGS_SHUT_SECS) < WINGS_OPEN_SECS
}

/// What a bug hunt sets out with.
pub struct Outset {
    pub party: Vec<Id>,
    /// Outings in a row that caught no new kind of bug.
    pub drought: u32,
    /// Whether the two who came are close.
    pub close_pair: bool,
    /// What the Hilltop lends the Woods: light, and the dusk's bugs out sooner.
    pub influence: crate::woods::Influence,
    pub seed: u64,
}

pub struct Hunt {
    party: Vec<Id>,
    characters: Vec<Character>,
    fliers: Vec<Flier>,
    phase: Phase,
    /// Which bug the person has chosen, if any.
    target: Option<usize>,
    light: f32,
    full: f32,
    dusk: f32,
    dusk_told: bool,
    /// Bugs caught this outing and their sizes, and anything else the net brought up.
    jar: Vec<(&'static str, f32)>,
    basket: Vec<&'static str>,
    found: Vec<&'static str>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    clock: f32,
    /// How dark the hour has made the meadow already, so the outing's own dusk only darkens it
    /// past that.
    hour_dark: f32,
    /// Whether the person is holding to creep.
    holding: bool,
    /// Where the dozy one was last seen, and since when it has kept to that spot.
    still: Option<Stillness>,
    /// From who came along: how little a creep rattles a bug, how fast the creep is, how much
    /// further the net reaches, how readily bugs come near, who can creep into the brambles, and
    /// who dozes.
    hush: f32,
    creep: f32,
    reach: f32,
    lure: f32,
    little: bool,
    dozer: Option<Id>,
}

impl Hunt {
    /// Sets out with the party, already in the meadow in `ground`. `caught_before` says which
    /// kinds the colony has caught; `found_before` which finds it has, for anything else the net
    /// brings up.
    pub fn new(
        ground: &mut Playground,
        outset: Outset,
        caught_before: impl Fn(&str) -> bool,
        found_before: impl Fn(&str) -> bool,
        now: f32,
    ) -> Self {
        let Outset {
            party,
            drought,
            close_pair,
            influence,
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let refs: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(seed);
        let fliers: Vec<Flier> = bugs::out(&refs, caught_before, drought, &mut dice)
            .into_iter()
            .map(|bug| {
                let perches = bug.haunt.perches();
                let perch = (dice.unit() * perches.len() as f32) as usize % perches.len();
                Flier {
                    bug,
                    size: dice.range(bug.size.0, bug.size.1),
                    perch,
                    pos: perches[perch],
                    facing_right: dice.chance(0.5),
                    flight: Flight::Settled {
                        since: now - dice.range(0.0, 1.5),
                        until: now + dice.range(bug.settles.0, bug.settles.1),
                    },
                    nerve: 0.0,
                    startled_at: f32::MIN,
                    out: !bug.dusk,
                    riding: None,
                    offset: dice.range(0.0, 3.0),
                }
            })
            .collect();
        let netter = characters.first();
        let calm = netter.map_or(0.5, |c| 1.0 - c.axes.energy);
        let steady = netter.map_or(0.5, |c| 1.0 - c.axes.impulsiveness);
        let quiet_kind = netter.is_some_and(|c| {
            matches!(
                c.kind,
                TemperamentKind::Scholar | TemperamentKind::Wallflower | TemperamentKind::Lazybones
            )
        });
        let hush = (0.3 * calm + 0.2 * steady + if quiet_kind { 0.15 } else { 0.0 }).min(0.6);
        let creep = netter.map_or(16.0, |c| 12.0 + 14.0 * c.axes.energy);
        let reach = match characters.len() {
            0 | 1 => 0.0,
            _ if close_pair => 6.0,
            _ => 3.0,
        };
        let lure = characters
            .iter()
            .map(|c| c.axes.playfulness)
            .fold(0.0, f32::max)
            * 0.4;
        let little = netter.is_some_and(crew::little);
        let dozer = party
            .iter()
            .zip(&characters)
            .find(|(_, c)| c.kind == TemperamentKind::Lazybones || c.axes.energy < 0.25)
            .map(|(id, _)| *id);
        let mut hunt = Self {
            party,
            fliers,
            phase: Phase::Arriving { since: now },
            target: None,
            light: LIGHT + influence.light,
            full: LIGHT + influence.light,
            dusk: DUSK + influence.earlier,
            dusk_told: false,
            jar: Vec::new(),
            basket: Vec::new(),
            found: finds::CATALOGUE
                .iter()
                .map(|find| find.id)
                .filter(|id| found_before(id))
                .collect(),
            events: Vec::new(),
            dice,
            reduce_motion: ground.reduce_motion(),
            clock: now,
            hour_dark: 0.0,
            holding: false,
            still: None,
            hush: if close_pair { hush + 0.1 } else { hush },
            creep,
            reach,
            lure,
            little,
            dozer,
            characters,
        };
        hunt.point_out(false);
        hunt
    }

    /// Starts this hunt partway through a longer day than a hunt's own, as a leg of an
    /// expedition: with `light` left of that day's `full`, so the dusk's bugs wait for the
    /// expedition's dusk.
    pub fn partway(mut self, light: f32, full: f32) -> Self {
        self.light = light;
        self.full = full;
        self
    }

    /// The light left, on the day's own scale.
    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.party
    }

    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    /// The bugs caught this outing, and their sizes in millimetres.
    pub fn jar(&self) -> &[(&'static str, f32)] {
        &self.jar
    }

    /// Anything else the net brought up, to take home.
    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// How dark the hour has made the meadow already: the outing's dusk adds only what is more.
    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
    }

    /// The bug chosen, if any: its kind, and where it is.
    pub fn target(&self) -> Option<(&'static Bug, (f32, f32))> {
        let flier = &self.fliers[self.target?];
        flier.here().then_some((flier.bug, flier.pos))
    }

    /// The bug under a point, nearest first, for choosing it.
    pub fn bug_at(&self, x: f32, y: f32) -> Option<usize> {
        self.fliers
            .iter()
            .enumerate()
            .filter(|(_, flier)| flier.here())
            .map(|(index, flier)| (index, distance(flier.pos, (x, y))))
            .filter(|(_, off)| *off <= 9.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// What the bug under a point is called, for a label as the pointer passes over it.
    pub fn name_at(&self, x: f32, y: f32) -> Option<(&'static str, (f32, f32))> {
        let flier = &self.fliers[self.bug_at(x, y)?];
        Some((flier.bug.name, flier.pos))
    }

    /// Chooses the bug under a point to go after. The one with the net crouches and turns to it.
    pub fn choose(&mut self, ground: &mut Playground, x: f32, y: f32, now: f32) -> bool {
        if self.phase != Phase::Hunting {
            return false;
        }
        let Some(index) = self.bug_at(x, y) else {
            return false;
        };
        self.target = Some(index);
        let flier = &self.fliers[index];
        self.events.push(Event::Stalking { bug: flier.bug.id });
        let pos = flier.pos;
        if let Some(netter) = self.party.first() {
            let crouch = Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0);
            ground.direct(*netter, vec![Step::FaceX(pos.0), Step::Beat(crouch)], now);
        }
        // A second companion comes round to help, keeping back from the bug and watching.
        if let Some(netter) = self.party.first()
            && let Some(at) = ground.position(*netter)
        {
            let stand = self.stand_for(pos, at);
            self.bring_helper(ground, at, stand, pos.0, now);
        }
        true
    }

    /// Whether the person is holding to creep closer.
    pub fn hold(&mut self, holding: bool) {
        self.holding = holding;
    }

    /// Whether the chosen bug is within the net's reach right now.
    pub fn in_reach(&self, ground: &Playground) -> bool {
        self.target
            .and_then(|index| self.fliers.get(index))
            .filter(|flier| flier.here())
            .is_some_and(|flier| self.reaches(ground, flier))
    }

    fn reaches(&self, ground: &Playground, flier: &Flier) -> bool {
        let Some(netter) = self.party.first() else {
            return false;
        };
        if flier.riding == Some(*netter) {
            return true;
        }
        let Some(at) = ground.position(*netter) else {
            return false;
        };
        let side = if flier.pos.0 >= at.0 { 1.0 } else { -1.0 };
        let centre = (at.0 + side * NET_AHEAD, at.1 - NET_UP);
        let (rx, ry) = (NET_ROUND.0 + self.reach, NET_ROUND.1 + self.reach);
        ((flier.pos.0 - centre.0) / rx).powi(2) + ((flier.pos.1 - centre.1) / ry).powi(2) <= 1.0
    }

    /// How rattled the chosen bug is, from 0 to 1: at 1 it goes.
    pub fn nerve(&self) -> f32 {
        self.target
            .and_then(|index| self.fliers.get(index))
            .map_or(0.0, |flier| flier.nerve)
    }

    /// Whether the chosen bug would be caught by a swing right now: for a steady hand, in the
    /// renders and tests.
    pub fn open_now(&self, ground: &Playground, now: f32) -> bool {
        self.in_reach(ground)
            && self
                .target
                .is_some_and(|index| self.fliers[index].open(now).is_ok())
    }

    /// The person swings the net.
    pub fn swing(&mut self, ground: &mut Playground, now: f32) {
        if self.phase != Phase::Hunting {
            return;
        }
        let Some(target) = self.target.filter(|index| self.fliers[*index].here()) else {
            return;
        };
        self.light -= SWING_COST;
        if let Some(netter) = self.party.first() {
            let swish = Beat::new(Gesture::Reach, ExpressionKind::Determined, SWING_SECS);
            let crouch = Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0);
            ground.direct(*netter, vec![Step::Beat(swish), Step::Beat(crouch)], now);
        }
        self.phase = Phase::Swinging { target, since: now };
    }

    /// Plays the hunt on. Call every frame with the party's playground.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        match self.phase {
            Phase::Arriving { since } => {
                let settled = self.party.iter().all(|id| !ground.busy(*id));
                if settled || now - since > 6.0 {
                    // Out of free play for the hunt: nobody wanders off with the net.
                    ground.reserve(self.party.clone());
                    self.phase = Phase::Hunting;
                }
            }
            Phase::Hunting => {
                self.light -= LIGHT_PER_SEC * dt;
                self.creep_on(ground, dt, now);
            }
            Phase::Swinging { target, since } if now - since >= SWING_SECS => {
                self.resolve(ground, target, now);
            }
            Phase::Caught { since, .. } | Phase::Netted { since, .. }
                if now - since >= ADMIRE_SECS =>
            {
                if let Some(netter) = self.party.first() {
                    let crouch = Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0);
                    ground.direct(*netter, vec![Step::Beat(crouch)], now);
                }
                self.phase = Phase::Hunting;
            }
            Phase::Leaving { since } if now - since >= 1.5 => self.phase = Phase::Over,
            _ => {}
        }
        // Through the swing and the admiring too: whatever the net carrier is doing, the helper
        // is never stood on.
        if !matches!(
            self.phase,
            Phase::Arriving { .. } | Phase::Leaving { .. } | Phase::Over
        ) {
            self.keep_helper_clear(ground, now);
        }
        if !self.dusk_told && self.light < self.dusk {
            self.dusk_told = true;
            for flier in &mut self.fliers {
                if !flier.out {
                    flier.out = true;
                    // In from the edge of the meadow to a perch.
                    let from = (flier.pos.0, GROUND.1 - 40.0);
                    flier.flight = Flight::Moving {
                        from,
                        to: flier.perch,
                        since: now,
                        takes: 2.5,
                        fleeing: false,
                    };
                    flier.pos = from;
                }
            }
            self.events.push(Event::Dusk);
            self.point_out(true);
        }
        if self.light <= 0.0 && self.phase == Phase::Hunting {
            self.head_home(ground, now);
        }
        self.fly(ground, now);
    }

    /// The person calls it a day, or the light has gone.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            return;
        }
        for id in &self.party {
            ground.direct(*id, vec![Step::Walk { to: (-30.0, 186.0) }], now);
        }
        self.events.push(Event::Leaving);
        self.phase = Phase::Leaving { since: now };
    }

    /// The one with the net creeps towards the chosen bug while the person holds, and keeps
    /// still when they let go. Moving near any bug rattles it; keeping still calms it.
    fn creep_on(&mut self, ground: &mut Playground, dt: f32, now: f32) {
        let Some(&netter) = self.party.first() else {
            return;
        };
        let Some(at) = ground.position(netter) else {
            return;
        };
        let mut moved = 0.0;
        if self.holding
            && let Some(index) = self.target.filter(|index| self.fliers[*index].here())
            && !self.reaches(ground, &self.fliers[index])
        {
            let goal = self.stand_for(self.fliers[index].pos, at);
            let off = distance(at, goal);
            if off > 0.5 {
                let step = (self.creep * dt).min(off);
                let to = (
                    at.0 + (goal.0 - at.0) / off * step,
                    at.1 + (goal.1 - at.1) / off * step,
                );
                ground.teleport(netter, to);
                moved = step;
                self.light -= CREEP_COST * step;
            }
            ground.direct(
                netter,
                vec![
                    Step::FaceX(self.fliers[index].pos.0),
                    Step::Beat(Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0)),
                ],
                now,
            );
        }
        let speed = if dt > 0.0 { moved / dt } else { 0.0 };
        let at = ground.position(netter).unwrap_or(at);
        let mut startled = Vec::new();
        for (index, flier) in self.fliers.iter_mut().enumerate() {
            if !flier.here() || flier.riding.is_some() {
                continue;
            }
            let near = distance(at, flier.pos) < flier.bug.notice;
            if near && speed > 0.0 {
                let busy = if matches!(flier.flight, Flight::Moving { .. }) {
                    0.4
                } else {
                    1.0
                };
                // Faster is noticed more for every pixel of ground covered, not just sooner.
                let haste = (speed / 18.0).powf(1.5);
                flier.nerve += flier.bug.skittish * dt * haste * (1.0 - self.hush) * busy;
            } else {
                flier.nerve = (flier.nerve - CALMING * dt).max(0.0);
            }
            if flier.nerve >= 1.0 {
                startled.push(index);
            }
        }
        for index in startled {
            self.frighten(index, now);
        }
    }

    /// Sends the second companion, if one came, to watch from beside where the one with the net
    /// will stand, at `stand`, behind them from the bug at `bug_x`: never on top of them, and
    /// off the way they creep there from `from`, so they never creep onto it either.
    fn bring_helper(
        &self,
        ground: &mut Playground,
        from: (f32, f32),
        stand: (f32, f32),
        bug_x: f32,
        now: f32,
    ) {
        let Some(&helper) = self.party.get(1) else {
            return;
        };
        let back = if stand.0 < bug_x { -1.0 } else { 1.0 };
        let clear =
            |(x, y): (f32, f32)| walkable(x, y) && off_the_way((x, y), from, stand) >= HELPER_ROOM;
        // Side by side first: straight in front or behind, one hides the other.
        let preferred = [
            (back * 28.0, 4.0),
            (back * 28.0, 14.0),
            (back * 28.0, -10.0),
            (back * 42.0, 2.0),
            (-back * 30.0, 12.0),
            (back * 22.0, 22.0),
            (0.0, 24.0),
        ]
        .into_iter()
        .map(|(dx, dy)| (stand.0 + dx, stand.1 + dy))
        .find(|&spot| clear(spot));
        // Hemmed in, by the pond or at the brambles: the nearest clear ground going round.
        let Some(spot) = preferred.or_else(|| {
            (0..6).find_map(|ring| {
                let radius = HELPER_ROOM + 4.0 + ring as f32 * 8.0;
                (0..16).find_map(|step| {
                    let angle = step as f32 / 16.0 * std::f32::consts::TAU;
                    let spot = (
                        stand.0 + angle.cos() * radius,
                        stand.1 + angle.sin() * radius,
                    );
                    clear(spot).then_some(spot)
                })
            })
        }) else {
            return;
        };
        let watch = Beat::new(Gesture::Watch, ExpressionKind::Focused, 600.0);
        ground.direct(
            helper,
            vec![
                Step::Walk { to: spot },
                Step::FaceX(bug_x),
                Step::Beat(watch),
            ],
            now,
        );
    }

    /// If the second companion is in the way of the one with the net, standing where it is or
    /// where it is creeping to, it steps aside before the net carrier gets there.
    fn keep_helper_clear(&self, ground: &mut Playground, now: f32) {
        let (Some(&netter), Some(&helper)) = (self.party.first(), self.party.get(1)) else {
            return;
        };
        let (Some(at), Some(helping)) = (ground.position(netter), ground.position(helper)) else {
            return;
        };
        let target = self.target.and_then(|index| self.fliers.get(index));
        let stand = target.map_or(at, |flier| self.stand_for(flier.pos, at));
        if off_the_way(helping, at, stand) >= HELPER_ROOM || ground.walking(helper) {
            return;
        }
        let bug_x = target.map_or(at.0 + 1.0, |flier| flier.pos.0);
        self.bring_helper(ground, at, stand, bug_x, now);
    }

    /// Where to stand to have a bug at `pos` in the net: before it and below, on whichever side
    /// can be stood on, the nearer first.
    fn stand_for(&self, pos: (f32, f32), from: (f32, f32)) -> (f32, f32) {
        let little = self.little;
        let can_stand = |x: f32, y: f32| {
            walkable(x, y) || (little && (6.0..=70.0).contains(&x) && (70.0..=206.0).contains(&y))
        };
        let near_side = if from.0 <= pos.0 { -1.0 } else { 1.0 };
        for side in [near_side, -near_side] {
            let x = pos.0 + side * NET_AHEAD;
            let y = pos.1 + NET_UP;
            for dy in 0..60 {
                for y in [y + dy as f32, y - dy as f32] {
                    if can_stand(x, y) {
                        return (x, y);
                    }
                }
            }
        }
        from
    }

    /// The net comes down: the bug is caught if it is in reach and open to it; otherwise it is
    /// off, and now and then something else is in the net instead.
    fn resolve(&mut self, ground: &mut Playground, target: usize, now: f32) {
        let flier = &self.fliers[target];
        let bug = flier.bug;
        if !flier.here() {
            self.phase = Phase::Hunting;
            return;
        }
        if !self.reaches(ground, flier) {
            self.events.push(Event::OutOfReach);
            let index = target;
            self.fliers[index].nerve += 0.5;
            if self.fliers[index].nerve >= 1.0 {
                self.frighten(index, now);
            }
            self.phase = Phase::Hunting;
            return;
        }
        match flier.open(now) {
            Ok(()) => {
                let size = flier.size;
                self.fliers[target].flight = Flight::Netted;
                self.fliers[target].riding = None;
                self.jar.push((bug.id, size));
                self.events.push(Event::Caught { bug: bug.id, size });
                self.target = None;
                if let (Some(netter), Some(character)) =
                    (self.party.first(), self.characters.first())
                {
                    let beats = delighted(character, bug.tier);
                    ground.direct(*netter, beats.into_iter().map(Step::Beat).collect(), now);
                }
                self.phase = Phase::Caught {
                    bug: bug.id,
                    size,
                    since: now,
                };
            }
            Err(why) => {
                // The miss says why, which is the lesson in it; the bug just goes.
                self.events.push(Event::Missed { bug: bug.id, why });
                self.send_off(target, now);
                let kind = match bug.haunt {
                    Haunt::Log | Haunt::Stump => Kind::Reach,
                    Haunt::Reeds => Kind::Scoop,
                    _ => Kind::Shake,
                };
                let netted = if self.dice.chance(NETTED) {
                    let party: Vec<&Character> = self.characters.iter().collect();
                    let found = &self.found;
                    finds::surely(kind, &[], &party, |id| found.contains(&id), &mut self.dice)
                } else {
                    None
                };
                if let Some(find) = netted {
                    self.take_home(find, now);
                } else {
                    self.phase = Phase::Hunting;
                }
            }
        }
    }

    fn take_home(&mut self, find: &'static Find, now: f32) {
        self.basket.push(find.id);
        self.events.push(Event::Netted { find: find.id });
        self.phase = Phase::Netted {
            find: find.id,
            since: now,
        };
    }

    /// Sends a bug off, frightened, and says so.
    fn frighten(&mut self, index: usize, now: f32) {
        self.events.push(Event::Startled {
            bug: self.fliers[index].bug.id,
        });
        self.send_off(index, now);
    }

    /// Sends a bug off: across its haunt if this is the first fright in a while, out of the
    /// meadow altogether if it is the second. Says nothing: a miss has already said why.
    fn send_off(&mut self, index: usize, now: f32) {
        let far = self.far_perch(index);
        let flier = &mut self.fliers[index];
        flier.riding = None;
        flier.nerve = 0.3;
        if now - flier.startled_at < 10.0 {
            flier.flight = Flight::Away {
                until: now + AWAY_SECS,
            };
        } else {
            let takes = (distance(flier.pos, flier.bug.haunt.perches()[far])
                / (flier.bug.speed * 2.0).max(8.0))
            .max(0.4);
            flier.flight = Flight::Moving {
                from: flier.pos,
                to: far,
                since: now,
                takes,
                fleeing: true,
            };
            flier.perch = far;
        }
        flier.startled_at = now;
    }

    /// The perch of a bug's haunt furthest from the one with the net.
    fn far_perch(&self, index: usize) -> usize {
        let flier = &self.fliers[index];
        let from = flier.pos;
        flier
            .bug
            .haunt
            .perches()
            .iter()
            .enumerate()
            .max_by(|a, b| distance(*a.1, from).total_cmp(&distance(*b.1, from)))
            .map_or(0, |(index, _)| index)
    }

    /// Every bug about its business: settling, moving on, coming back.
    fn fly(&mut self, ground: &mut Playground, now: f32) {
        let netter_at = self.party.first().and_then(|id| ground.position(*id));
        // The dozy one is judged by its own stillness, whoever is carrying the net.
        let dozing = self
            .dozer
            .and_then(|id| ground.position(id))
            .is_some_and(|at| {
                let still = self.still.get_or_insert(Stillness { at, since: now });
                still.seen(at, now) >= DOZE_SECS
            });
        let dozer = self
            .dozer
            .filter(|_| dozing && self.phase == Phase::Hunting);
        for index in 0..self.fliers.len() {
            let flier = &self.fliers[index];
            if !flier.out {
                continue;
            }
            match flier.flight {
                Flight::Settled { until, .. } if now >= until => {
                    // Moves on: somewhere near the party if something playful drew it, onto a
                    // dozing companion if one has kept still long enough, otherwise anywhere.
                    let perches = flier.bug.haunt.perches();
                    let lands_on = dozer.filter(|id| {
                        matches!(flier.bug.way, Way::Flutter | Way::Buzz)
                            && !self.fliers.iter().any(|f| f.riding == Some(*id))
                            && self.dice.chance(0.3)
                    });
                    if let Some(id) = lands_on
                        && let Some(head) = ground.head(id, now)
                    {
                        let flier = &mut self.fliers[index];
                        flier.riding = Some(id);
                        flier.pos = head;
                        flier.flight = Flight::Settled {
                            since: now,
                            until: now + 6.0,
                        };
                        self.events.push(Event::LandedOn {
                            who: id,
                            bug: flier.bug.id,
                        });
                        continue;
                    }
                    let next = if let Some(at) = netter_at
                        && self.dice.chance(self.lure)
                    {
                        nearest(perches, at)
                    } else {
                        (self.dice.unit() * perches.len() as f32) as usize % perches.len()
                    };
                    let flier = &mut self.fliers[index];
                    let to = perches[next];
                    let takes = (distance(flier.pos, to) / flier.bug.speed.max(1.0)).max(0.3);
                    flier.riding = None;
                    flier.flight = Flight::Moving {
                        from: flier.pos,
                        to: next,
                        since: now,
                        takes,
                        fleeing: false,
                    };
                    flier.perch = next;
                    flier.facing_right = to.0 >= flier.pos.0;
                }
                Flight::Settled { .. } => {
                    if let Some(id) = flier.riding
                        && let Some(head) = ground.head(id, now)
                    {
                        self.fliers[index].pos = head;
                    }
                }
                Flight::Moving {
                    from,
                    to,
                    since,
                    takes,
                    ..
                } => {
                    let flier = &mut self.fliers[index];
                    let goal = flier.bug.haunt.perches()[to];
                    let t = ((now - since) / takes).clamp(0.0, 1.0);
                    flier.pos = if self.reduce_motion {
                        // A cut rather than a flight: there, then here.
                        if t < 1.0 { from } else { goal }
                    } else {
                        path(flier.bug.way, from, goal, t, now + flier.offset)
                    };
                    if t >= 1.0 {
                        let settles = self.dice.range(flier.bug.settles.0, flier.bug.settles.1);
                        flier.pos = goal;
                        flier.flight = Flight::Settled {
                            since: now,
                            until: now + settles,
                        };
                    }
                }
                Flight::Away { until } if now >= until => {
                    let perches = flier.bug.haunt.perches();
                    let back = (self.dice.unit() * perches.len() as f32) as usize % perches.len();
                    let flier = &mut self.fliers[index];
                    let from = (perches[back].0, GROUND.1 - 40.0);
                    flier.nerve = 0.0;
                    flier.pos = from;
                    flier.perch = back;
                    flier.flight = Flight::Moving {
                        from,
                        to: back,
                        since: now,
                        takes: 2.0,
                        fleeing: false,
                    };
                }
                // Let go once it has been looked at: a common kind back later, bigger or smaller,
                // and anything rarer away for good.
                Flight::Netted
                    if !matches!(self.phase, Phase::Caught { .. } | Phase::Swinging { .. }) =>
                {
                    let common = flier.bug.tier == Tier::Common;
                    let size = self.dice.range(flier.bug.size.0, flier.bug.size.1);
                    let flier = &mut self.fliers[index];
                    if common {
                        flier.size = size;
                        flier.flight = Flight::Away {
                            until: now + RETURN_SECS,
                        };
                    } else {
                        flier.flight = Flight::Gone;
                    }
                }
                _ => {}
            }
        }
        if self.target.is_some_and(|index| !self.fliers[index].here()) {
            self.target = None;
        }
    }

    /// Someone curious points out the rarest bug about, as the hunt starts or the dusk's come
    /// out.
    fn point_out(&mut self, dusk: bool) {
        let Some(pointer) = self
            .party
            .iter()
            .zip(&self.characters)
            .find(|(_, c)| c.kind == TemperamentKind::Explorer || c.axes.curiosity >= 0.75)
            .map(|(id, _)| *id)
        else {
            return;
        };
        if let Some(rarest) = self
            .fliers
            .iter()
            .filter(|flier| flier.out && flier.bug.dusk == dusk && flier.bug.tier >= Tier::Rare)
            .max_by_key(|flier| flier.bug.tier)
        {
            self.events.push(Event::Pointed {
                who: pointer,
                bug: rarest.bug.id,
            });
        }
    }

    /// Every bug that is about, as a sprite standing on its own row among everyone.
    pub fn fliers(&self, now: f32) -> Vec<Prop> {
        self.fliers
            .iter()
            .filter(|flier| flier.here())
            .map(|flier| {
                let (pose, frame) = flier.pose(now, self.reduce_motion);
                let mut sprite = art::sprite(flier.bug.id, pose, frame);
                let (mut ax, ay) = art::anchor(flier.bug.id, pose);
                if !flier.facing_right {
                    sprite = mirrored(&sprite);
                    ax = sprite.width() as i32 - 1 - ax;
                }
                // A bug on a companion's head stands where that companion does.
                let base = match flier.riding {
                    Some(_) => flier.pos.1 + 40.0,
                    None => flier.pos.1 + 10.0,
                };
                Prop::new(
                    sprite,
                    (flier.pos.0 as i32 - ax, flier.pos.1 as i32 - ay),
                    base,
                )
            })
            .collect()
    }

    /// The meadow darkening as the light goes, fireflies' glow, the chosen bug marked, the net,
    /// and a catch held up.
    pub fn draw(&self, scene: &mut Canvas, ground: &Playground, now: f32) {
        dim(scene, self.light, self.hour_dark);
        for flier in self.fliers.iter().filter(|flier| flier.here()) {
            if flier.bug.way == Way::Blink && flier.open(now).is_ok() {
                crate::daylight::glow(
                    scene,
                    (flier.pos.0 as i32, flier.pos.1 as i32),
                    7,
                    rgb(0xe8ff8a),
                );
                put(scene, flier.pos.0 as i32, flier.pos.1 as i32, rgb(0xf6ffcc));
            }
        }
        if let Some(index) = self.target
            && self.phase == Phase::Hunting
        {
            let flier = &self.fliers[index];
            if flier.here() {
                let color = if self.reaches(ground, flier) {
                    rgb(0xf5d25e)
                } else {
                    rgba(0xfdfbf5, 220)
                };
                brackets(scene, flier.pos, 5, color);
                jitters(scene, flier.pos, flier.nerve);
            }
        }
        self.draw_net(scene, ground, now);
        match self.phase {
            Phase::Caught { bug, since, .. } => {
                self.hold_up(scene, ground, &art::close_up(bug), since, now)
            }
            Phase::Netted { find, since } => {
                self.hold_up(
                    scene,
                    ground,
                    &crate::finds::art::piece(find).sprite,
                    since,
                    now,
                );
            }
            _ => {}
        }
    }

    /// The net in the hands of the one carrying it: held up while nothing is chosen, lowered
    /// towards the chosen bug, and swept down when swung.
    fn draw_net(&self, scene: &mut Canvas, ground: &Playground, now: f32) {
        let Some(netter) = self.party.first() else {
            return;
        };
        let Some(at) = ground.position(*netter) else {
            return;
        };
        let target = self
            .target
            .and_then(|index| self.fliers.get(index))
            .filter(|flier| flier.here());
        let side = target.map_or(1.0, |flier| if flier.pos.0 >= at.0 { 1.0 } else { -1.0 });
        let hand = (at.0 + side * 6.0, at.1 - 12.0);
        let ready = (at.0 + side * NET_AHEAD, at.1 - NET_UP);
        let raised = (at.0 + side * 8.0, at.1 - 34.0);
        let hoop = match self.phase {
            Phase::Swinging { since, .. } if !self.reduce_motion => {
                let t = ((now - since) / SWING_SECS).clamp(0.0, 1.0);
                let t = t * t;
                (
                    raised.0 + (ready.0 - raised.0) * t,
                    raised.1 + (ready.1 - raised.1) * t,
                )
            }
            Phase::Swinging { .. } | Phase::Hunting if target.is_some() => ready,
            Phase::Leaving { .. } | Phase::Over => return,
            _ => raised,
        };
        let pole = rgb(0x8c6e50);
        line(
            scene,
            (hand.0 as i32, hand.1 as i32),
            (hoop.0 as i32, hoop.1 as i32),
            pole,
        );
        let (hx, hy) = (hoop.0 as i32, hoop.1 as i32);
        ellipse(scene, hx, hy, 6, 5, rgba(0xfdfbf5, 60));
        for step in 0..24 {
            let angle = step as f32 / 24.0 * std::f32::consts::TAU;
            let x = hx + (angle.cos() * 6.0).round() as i32;
            let y = hy + (angle.sin() * 5.0).round() as i32;
            put(scene, x, y, rgb(0xd8cdb4));
        }
    }

    /// Something held up over the head of the one with the net, rising a little as it comes.
    fn hold_up(
        &self,
        scene: &mut Canvas,
        ground: &Playground,
        sprite: &Canvas,
        since: f32,
        now: f32,
    ) {
        let Some(netter) = self.party.first() else {
            return;
        };
        let Some(at) = ground.position(*netter) else {
            return;
        };
        let rise = if self.reduce_motion {
            8.0
        } else {
            ((now - since) / 0.4).min(1.0) * 8.0
        };
        let (width, height) = (sprite.width() as i32, sprite.height() as i32);
        let x = at.0 as i32 - width / 2;
        let y = at.1 as i32 - 40 - height - rise as i32;
        crate::daylight::glow(
            scene,
            (at.0 as i32, y + height / 2),
            width.max(height),
            rgba(0xfff3d0, 200),
        );
        crate::paint::blit(scene, sprite, x, y);
    }
}

/// How long someone has kept to one spot.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Stillness {
    at: (f32, f32),
    since: f32,
}

impl Stillness {
    /// Notes where they are now, and says how long they have kept still: any move starts it
    /// again.
    fn seen(&mut self, at: (f32, f32), now: f32) -> f32 {
        if distance(at, self.at) > 0.5 {
            *self = Self { at, since: now };
        }
        now - self.since
    }
}

/// The way a kind of bug goes from one perch to the next.
fn path(way: Way, from: (f32, f32), to: (f32, f32), t: f32, clock: f32) -> (f32, f32) {
    let straight = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
    match way {
        // A wandering, bobbing flutter.
        Way::Flutter => (
            straight.0 + (clock * 5.0).sin() * 3.0,
            straight.1 - (t * std::f32::consts::PI).sin() * 10.0 + (clock * 9.0).sin() * 2.0,
        ),
        // Straight, with a buzz in it.
        Way::Buzz => (straight.0, straight.1 + (clock * 23.0).sin()),
        // An arc up and over.
        Way::Hop => (
            straight.0,
            straight.1 - (t * std::f32::consts::PI).sin() * 14.0,
        ),
        // A dart: fast away, slowing in.
        Way::Dart => {
            let t = 1.0 - (1.0 - t).powi(3);
            (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
        }
        // Drifting, rising and falling.
        Way::Blink => (straight.0, straight.1 - 6.0 + (clock * 1.7).sin() * 3.0),
        Way::Crawl => straight,
    }
}

/// How far `spot` is from the way between `from` and `to`, at its nearest, as it looks: a step
/// in front or behind counts for half a step aside, since standing just in front of someone
/// still hides them.
fn off_the_way(spot: (f32, f32), from: (f32, f32), to: (f32, f32)) -> f32 {
    let squash = |(x, y): (f32, f32)| (x, y * 0.5);
    let (spot, from, to) = (squash(spot), squash(from), squash(to));
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx * dx + dy * dy;
    if length < 1e-6 {
        return distance(spot, from);
    }
    let along = (((spot.0 - from.0) * dx + (spot.1 - from.1) * dy) / length).clamp(0.0, 1.0);
    distance(spot, (from.0 + dx * along, from.1 + dy * along))
}

fn nearest(perches: &[(f32, f32)], to: (f32, f32)) -> usize {
    perches
        .iter()
        .enumerate()
        .min_by(|a, b| distance(*a.1, to).total_cmp(&distance(*b.1, to)))
        .map_or(0, |(index, _)| index)
}

/// Four little corners round a point: the bug the person has chosen.
fn brackets(scene: &mut Canvas, (x, y): (f32, f32), half: i32, color: formiga_art::Rgba) {
    let (x, y) = (x as i32, y as i32);
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (cx, cy) = (x + sx * half, y + sy * half);
        put(scene, cx, cy, color);
        put(scene, cx - sx, cy, color);
        put(scene, cx, cy - sy, color);
    }
}

/// Little lines of alarm by a bug as its nerves go: one, then two, then three. The tell to keep
/// still.
fn jitters(scene: &mut Canvas, (x, y): (f32, f32), nerve: f32) {
    let marks = match nerve {
        n if n > 0.85 => 3,
        n if n > 0.6 => 2,
        n if n > 0.35 => 1,
        _ => 0,
    };
    let color = if marks == 3 {
        rgb(0xff8a6a)
    } else {
        rgba(0xfdfbf5, 230)
    };
    let (x, y) = (x as i32, y as i32);
    for mark in 0..marks {
        let (mx, my) = (x + 7 + mark * 2, y - 8 + mark);
        put(scene, mx, my, color);
        put(scene, mx, my + 1, color);
    }
}

fn mirrored(sprite: &Canvas) -> Canvas {
    let (width, height) = (sprite.width() as i32, sprite.height() as i32);
    let mut out = Canvas::new(sprite.width(), sprite.height());
    for y in 0..height {
        for x in 0..width {
            out.set(width - 1 - x, y, sprite.get(x, y));
        }
    }
    out
}

/// What a companion is like with a net, for choosing who to bring.
pub fn netter(character: &Character) -> &'static str {
    if crew::little(character) {
        "Small enough to creep right into the brambles."
    } else if character.kind == TemperamentKind::Lazybones || character.axes.energy < 0.25 {
        "Keeps so still that things land on it."
    } else if character.axes.playfulness >= 0.7 {
        "Butterflies come to see what it's doing."
    } else if character.axes.energy >= 0.7 {
        "Quick to close in, and noticed for it."
    } else if character.axes.energy <= 0.35 || character.kind == TemperamentKind::Scholar {
        "Light on its feet: bugs hardly notice it creeping."
    } else {
        "Steady with a net."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::woods::Influence;

    fn hunt_with(party: usize, seed: u64) -> (Cast, Playground, Hunt) {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let ids: Vec<Id> = cast.ids().take(party).collect();
        let mut ground = super::super::open(&cast, &ids, 0.0, &Default::default());
        let hunt = Hunt::new(
            &mut ground,
            Outset {
                party: ids,
                drought: 0,
                close_pair: false,
                influence: Influence::default(),
                seed,
            },
            |_| false,
            |_| false,
            0.0,
        );
        (cast, ground, hunt)
    }

    fn run(cast: &Cast, ground: &mut Playground, hunt: &mut Hunt, from: f32, to: f32) -> f32 {
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            hunt.tick(ground, now);
        }
        now
    }

    #[test]
    fn a_settled_butterfly_is_open_to_the_net_only_with_its_wings_shut() {
        let (_, _, mut hunt) = hunt_with(1, 1);
        let mut flier = hunt.fliers.remove(0);
        flier.bug = bugs::bug("cabbage_white").unwrap();
        flier.offset = 0.0;
        flier.flight = Flight::Settled {
            since: 0.0,
            until: 10.0,
        };
        assert_eq!(flier.open(0.5), Err(Why::WingsOpen));
        assert_eq!(flier.open(WINGS_OPEN_SECS + 0.2), Ok(()));
    }

    #[test]
    fn a_grasshopper_that_has_started_to_chirp_is_about_to_jump() {
        let (_, _, mut hunt) = hunt_with(1, 1);
        let mut flier = hunt.fliers.remove(0);
        flier.bug = bugs::bug("grasshopper").unwrap();
        flier.flight = Flight::Settled {
            since: 0.0,
            until: 3.0,
        };
        assert_eq!(flier.open(1.0), Ok(()));
        assert_eq!(flier.open(2.5), Err(Why::Chirping));
    }

    #[test]
    fn a_firefly_is_only_caught_while_it_is_lit() {
        let (_, _, mut hunt) = hunt_with(1, 1);
        let mut flier = hunt.fliers.remove(0);
        flier.bug = bugs::bug("firefly").unwrap();
        flier.offset = 0.0;
        assert_eq!(flier.open(0.5), Ok(()));
        assert_eq!(flier.open(LIT_SECS + 0.5), Err(Why::Dark));
    }

    #[test]
    fn nothing_in_flight_is_caught_but_a_beetle_crawling() {
        let (_, _, mut hunt) = hunt_with(1, 1);
        let mut flier = hunt.fliers.remove(0);
        flier.flight = Flight::Moving {
            from: (0.0, 0.0),
            to: 0,
            since: 0.0,
            takes: 5.0,
            fleeing: false,
        };
        flier.bug = bugs::bug("bumblebee").unwrap();
        assert_eq!(flier.open(1.0), Err(Why::InFlight));
        flier.bug = bugs::bug("stag_beetle").unwrap();
        assert_eq!(flier.open(1.0), Ok(()));
    }

    #[test]
    fn a_swing_at_the_wrong_moment_says_why_rather_than_only_that_it_flew() {
        let (cast, mut ground, mut hunt) = hunt_with(1, 3);
        let mut now = run(&cast, &mut ground, &mut hunt, 0.0, 8.0);
        assert_eq!(hunt.phase(), Phase::Hunting);
        // A cabbage white just settled, wings open and watching, with the net right before it.
        let index = 0;
        let perch = bugs::bug("cabbage_white").unwrap().haunt.perches()[0];
        let flier = &mut hunt.fliers[index];
        flier.bug = bugs::bug("cabbage_white").unwrap();
        flier.out = true;
        flier.perch = 0;
        flier.pos = perch;
        flier.flight = Flight::Settled {
            since: now,
            until: now + 60.0,
        };
        let stand = hunt.stand_for(perch, (192.0, 180.0));
        ground.teleport(hunt.party[0], stand);
        hunt.target = Some(index);
        hunt.take_events();
        hunt.swing(&mut ground, now);
        now += SWING_SECS + 0.05;
        hunt.tick(&mut ground, now);
        let events = hunt.take_events();
        assert!(
            events.contains(&Event::Missed {
                bug: "cabbage_white",
                why: Why::WingsOpen
            }),
            "{events:?}"
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::Startled { .. })),
            "the miss was talked over: {events:?}"
        );
    }

    #[test]
    fn a_helper_never_ends_up_standing_over_the_one_with_the_net() {
        let (cast, mut ground, mut hunt) = hunt_with(2, 9);
        let mut now = run(&cast, &mut ground, &mut hunt, 0.0, 8.0);
        let (netter, helper) = (hunt.party[0], hunt.party[1]);
        // Both standing still, the helper exactly where the one with the net is.
        let at = ground.position(netter).unwrap();
        ground.teleport(helper, at);
        let wait = Beat::new(Gesture::Watch, ExpressionKind::Focused, 600.0);
        ground.direct(netter, vec![Step::Beat(wait)], now);
        ground.direct(helper, vec![Step::Beat(wait)], now);
        for _ in 0..240 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            hunt.tick(&mut ground, now);
        }
        let apart = distance(
            ground.position(netter).unwrap(),
            ground.position(helper).unwrap(),
        );
        assert!(apart >= HELPER_ROOM - 1.0, "only {apart} px apart");
    }

    #[test]
    fn a_helper_waits_off_the_way_the_net_creeps_and_is_never_stood_on() {
        for seed in [3, 9, 11] {
            let (cast, mut ground, mut hunt) = hunt_with(2, seed);
            let (netter, helper) = (hunt.party[0], hunt.party[1]);
            let mut now = 0.0;
            while now < 120.0 {
                now += 1.0 / 30.0;
                ground.tick(&cast, now);
                if hunt.phase() == Phase::Hunting {
                    // Played as a patient hand would: the nearest bug, crept up on while it is
                    // calm, and netted when it is open to it.
                    let at = ground.position(netter).unwrap();
                    if hunt.target().is_none()
                        && let Some(nearest) = (0..hunt.fliers.len())
                            .filter(|index| hunt.fliers[*index].here())
                            .min_by(|a, b| {
                                distance(hunt.fliers[*a].pos, at)
                                    .total_cmp(&distance(hunt.fliers[*b].pos, at))
                            })
                    {
                        let (x, y) = hunt.fliers[nearest].pos;
                        hunt.choose(&mut ground, x, y, now);
                    }
                    hunt.hold(!hunt.in_reach(&ground) && hunt.nerve() < 0.55);
                    if hunt.open_now(&ground, now) {
                        hunt.swing(&mut ground, now);
                    }
                }
                hunt.tick(&mut ground, now);
                if !matches!(hunt.phase(), Phase::Arriving { .. }) && !ground.walking(helper) {
                    let at = ground.position(netter).unwrap();
                    let apart = off_the_way(ground.position(helper).unwrap(), at, at);
                    assert!(
                        apart >= HELPER_ROOM - 1.0,
                        "seed {seed}, {now:.1} seconds in, {:?}: only {apart:.1} px apart, netter {:?} helper {:?}",
                        hunt.phase(),
                        ground.position(netter),
                        ground.position(helper)
                    );
                }
            }
        }
    }

    #[test]
    fn a_helper_finds_room_even_where_the_ground_is_hemmed_in() {
        let (cast, mut ground, hunt) = hunt_with(2, 9);
        let helper = hunt.party[1];
        // Netting from the bank by the far reeds, with the pond and the meadow's edge close by.
        let stand = hunt.stand_for((362.0, 108.0), (192.0, 180.0));
        ground.reserve(hunt.party.clone());
        hunt.bring_helper(&mut ground, stand, stand, 362.0, 0.0);
        let mut now = 0.0;
        while now < 40.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
        }
        let at = ground.position(helper).unwrap();
        assert!(
            walkable(at.0, at.1),
            "the helper stood somewhere nobody can"
        );
        let apart = distance(at, stand);
        assert!(
            (HELPER_ROOM - 1.0..=70.0).contains(&apart),
            "the helper is {apart} px from the one with the net, not alongside"
        );
    }

    #[test]
    fn keeping_still_is_counted_from_the_last_move() {
        let mut still = Stillness {
            at: (10.0, 10.0),
            since: 0.0,
        };
        assert_eq!(still.seen((10.0, 10.0), 4.0), 4.0);
        assert_eq!(still.seen((14.0, 10.0), 5.0), 0.0, "a move starts it again");
        assert_eq!(still.seen((14.0, 10.0), 11.0), 6.0);
    }

    #[test]
    fn creeping_up_closes_in_and_keeping_still_does_not() {
        let (cast, mut ground, mut hunt) = hunt_with(1, 3);
        let now = run(&cast, &mut ground, &mut hunt, 0.0, 8.0);
        let netter = hunt.party[0];
        let index = hunt
            .fliers
            .iter()
            .position(|flier| flier.here())
            .expect("nothing out");
        let pos = hunt.fliers[index].pos;
        assert!(hunt.choose(&mut ground, pos.0, pos.1, now));
        let before = ground.position(netter).unwrap();
        let still = run(&cast, &mut ground, &mut hunt, now, now + 1.0);
        assert_eq!(
            ground.position(netter).unwrap(),
            before,
            "it crept unbidden"
        );
        hunt.hold(true);
        run(&cast, &mut ground, &mut hunt, still, still + 1.0);
        let goal = hunt.stand_for(hunt.fliers[index].pos, before);
        assert!(
            distance(ground.position(netter).unwrap(), goal) < distance(before, goal),
            "holding did not creep it closer"
        );
    }

    #[test]
    fn rushing_a_bug_frightens_it_off_and_creeping_quietly_need_not() {
        let (_, _, mut hunt) = hunt_with(1, 4);
        let index = 0;
        hunt.fliers[index].nerve = 0.0;
        hunt.hush = 0.0;
        // Moving at a run right beside it, for a second, rattles it past bearing.
        let bug = hunt.fliers[index].bug;
        let nerve = bug.skittish * 1.0 * (60.0 / 18.0);
        assert!(
            nerve >= 1.0 || bug.skittish < 0.3,
            "{} is never frightened",
            bug.id
        );
        // At a creep, with a quiet companion, the same second hardly does.
        let gentle = bug.skittish * 1.0 * (8.0 / 18.0) * (1.0 - 0.6);
        assert!(gentle < 0.5);
    }

    #[test]
    fn every_kind_can_be_caught_from_somewhere_anyone_can_stand() {
        let (_, ground, hunt) = hunt_with(1, 5);
        let _ = ground;
        for bug in &bugs::CATALOGUE {
            let reachable = bug.haunt.perches().iter().any(|&perch| {
                let stand = hunt.stand_for(perch, (192.0, 180.0));
                let side = if perch.0 >= stand.0 { 1.0 } else { -1.0 };
                let centre = (stand.0 + side * NET_AHEAD, stand.1 - NET_UP);
                walkable(stand.0, stand.1)
                    && ((perch.0 - centre.0) / NET_ROUND.0).powi(2)
                        + ((perch.1 - centre.1) / NET_ROUND.1).powi(2)
                        <= 1.0
            });
            assert!(reachable, "nobody but a little one can reach {}", bug.id);
        }
    }

    #[test]
    fn a_hunt_ends_when_the_light_goes() {
        let (cast, mut ground, mut hunt) = hunt_with(2, 6);
        run(&cast, &mut ground, &mut hunt, 0.0, 500.0);
        assert_eq!(hunt.phase(), Phase::Over);
    }

    #[test]
    fn the_dusks_bugs_come_out_as_the_light_goes() {
        let (cast, mut ground, mut hunt) = hunt_with(1, 7);
        for flier in &mut hunt.fliers {
            flier.bug = bugs::bug("moon_moth").unwrap();
            flier.perch = 0;
            flier.pos = flier.bug.haunt.perches()[0];
            flier.out = false;
        }
        let now = run(&cast, &mut ground, &mut hunt, 0.0, 10.0);
        assert!(hunt.fliers.iter().all(|flier| !flier.here()));
        hunt.light = DUSK - 1.0;
        run(&cast, &mut ground, &mut hunt, now, now + 1.0);
        assert!(hunt.fliers.iter().all(|flier| flier.out));
        assert!(hunt.take_events().contains(&Event::Dusk));
    }

    #[test]
    fn with_reduced_motion_a_bug_cuts_from_perch_to_perch() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let ids: Vec<Id> = cast.ids().take(1).collect();
        let mut ground = super::super::open(&cast, &ids, 0.0, &Default::default());
        let mut hunt = Hunt::new(
            &mut ground,
            Outset {
                party: ids,
                drought: 0,
                close_pair: false,
                influence: Influence::default(),
                seed: 8,
            },
            |_| false,
            |_| false,
            0.0,
        );
        let mut now = 0.0;
        let mut seen = 0;
        while now < 60.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            hunt.tick(&mut ground, now);
            for flier in hunt.fliers.iter().filter(|flier| flier.here()) {
                let perches = flier.bug.haunt.perches();
                if let Flight::Moving { from, .. } = flier.flight {
                    assert!(
                        flier.pos == from || perches.contains(&flier.pos),
                        "{} was caught between perches",
                        flier.bug.id
                    );
                    seen += 1;
                }
            }
        }
        assert!(seen > 0, "nothing moved at all");
    }
}
