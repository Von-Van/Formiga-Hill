//! A fishing trip at the pool: one companion fishes from the bank, a second (if one came) stands
//! by with the net, and the person plays it, until the light goes.
//!
//! Each kind of fish keeps to its own part of the pool, so where to cast is the first thing to
//! learn. A splash too close sends a wary fish off. Fish nearby come to the float, nibble a number
//! of times that is a tell of their kind (a perch always twice, a trout never), and then bite: the
//! float goes under, and that is the moment to strike. Striking on a nibble scares the fish off;
//! too late and it takes the bait. Once hooked, holding reels in, but holding while the fish pulls
//! strains the line, and too much snaps it; easing off lets it run. Sometimes the line snags on
//! something that comes home for the Hilltop instead.
//!
//! Who came along matters: a patient angler gives a longer moment to strike and casts steadier, a
//! playful one draws fish in more readily, a strong one strains the line less, and a lazybones who
//! nods off on the bank somehow gets more bites. A second companion with the net lands fish from
//! further out. And which fish are about leans towards who came (see `fish`).

use super::art::drawn_length;
use super::fish::{self, Fish};
use super::{Haunt, NET, SEAT, WATER, in_water};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{self, Find, Kind, Tier};
use crate::paint::{ellipse, line, mix, put, rect, rgb, rgba};
use crate::playground::{Playground, distance};
use crate::woods::rummage::{delighted, dim};
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{ActionKind, Gesture, TemperamentKind};
use std::f32::consts::{PI, TAU};

/// The light a trip starts with, how much each cast costs, and how much goes each second.
pub const LIGHT: f32 = 100.0;
const CAST_COST: f32 = 2.0;
const LIGHT_PER_SEC: f32 = 0.25;
/// Below this much light, the fish of the dusk come up.
const DUSK: f32 = 40.0;
/// How long the float is in the air.
const FLIGHT_SECS: f32 = 0.5;
/// How far off a fish notices the float.
const NOTICE: f32 = 54.0;
/// How long a scared fish keeps away.
const SPOOKED_SECS: f32 = 6.0;
/// The chance a cast snags on something worth bringing home.
const SNAG: f32 = 0.06;
/// Reeling in, a fish pulling, and the line's strain, per second.
const REEL_SPEED: f32 = 42.0;
const PULL_SPEED: f32 = 26.0;
const STRAIN: f32 = 1.0;
const EASE: f32 = 0.7;
/// How long a landed fish is held up to admire before it goes back.
const ADMIRE_SECS: f32 = 2.2;
/// With reduced motion, fish move in steps this far apart rather than gliding.
const STEP_SECS: f32 = 1.5;
/// How near the rod tip a fish is landed, before the net.
const LANDING: f32 = 10.0;

/// Where the rod's tip is, over the water from the one fishing.
pub const ROD_TIP: (f32, f32) = (SEAT.0 + 30.0, SEAT.1 - 40.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Arriving {
        since: f32,
    },
    /// Ready to cast.
    Ready,
    Casting {
        to: (f32, f32),
        since: f32,
    },
    /// The float is on the water.
    Waiting {
        float: (f32, f32),
        since: f32,
    },
    /// Something is on the line.
    Fighting(Fight),
    /// A fish landed, held up before it goes back.
    Landed {
        fish: &'static str,
        length: f32,
        since: f32,
    },
    /// Something other than a fish came up on the line.
    Snagged {
        find: &'static str,
        since: f32,
    },
    Leaving {
        since: f32,
    },
    Over,
}

impl Fight {
    /// Whether the fish is pulling right now.
    pub fn pulling(&self, now: f32) -> bool {
        now < self.pulling_until
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fight {
    /// Which of the fish about is hooked.
    swimmer: usize,
    /// How far out it is, along the line from the rod tip, in pixels.
    pub out: f32,
    /// Which way the line runs from the rod tip, as a unit step.
    towards: (f32, f32),
    /// How near the line is to snapping, from 0 to 1.
    pub strain: f32,
    pulling_until: f32,
    next_pull: f32,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Spooked { fish: &'static str },
    Nibble,
    Bite,
    TooSoon,
    Stolen,
    Hooked { fish: &'static str },
    Snapped,
    Landed { fish: &'static str, length: f32 },
    Snagged,
    Pointed { who: Id, haunt: Haunt },
    Dusk,
    Leaving,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Swim {
    Cruising,
    Coming,
    Nibbling { left: u8, next: f32 },
    Biting { until: f32 },
    Hooked,
}

#[derive(Clone, Debug)]
struct Swimmer {
    fish: &'static Fish,
    /// Centimetres, within its kind's range.
    length: f32,
    pos: (f32, f32),
    /// Where it was drawn, for reduced motion, which moves it in steps.
    shown: (f32, f32),
    heading: f32,
    target: (f32, f32),
    state: Swim,
    spooked_until: f32,
    /// The dusk's fish are only about once the light has gone low.
    surfaced: bool,
}

pub struct Angling {
    party: Vec<Id>,
    swimmers: Vec<Swimmer>,
    phase: Phase,
    light: f32,
    dusk_told: bool,
    /// Fish landed this trip, and their lengths; and anything that snagged.
    creel: Vec<(&'static str, f32)>,
    basket: Vec<&'static str>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    last_step: f32,
    clock: f32,
    /// Whether the person is holding the line in.
    holding: bool,
    /// What the snag will be, decided at the cast.
    snag: Option<&'static Find>,
    /// From who came along: how long the bite lasts, how far a cast can stray, how readily fish
    /// come, how hard the line is strained, how far out the net reaches.
    window: f32,
    stray: f32,
    lure: f32,
    grip: f32,
    reach: f32,
    /// Whether the one fishing has nodded off, and since when it has been waiting.
    dozing: bool,
}

/// What a fishing trip sets out with.
pub struct Outset {
    pub party: Vec<Id>,
    /// Trips in a row that caught no new kind of fish.
    pub drought: u32,
    pub seed: u64,
}

impl Angling {
    /// Sets out with the party, already on the bank in `ground`. `caught_before` says which kinds
    /// the colony has caught; `found_before` which finds it has, for anything that snags.
    pub fn new(
        ground: &mut Playground,
        outset: Outset,
        caught_before: impl Fn(&str) -> bool,
        now: f32,
    ) -> Self {
        let Outset {
            party,
            drought,
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let refs: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(seed);
        let swimmers = fish::about(&refs, caught_before, drought, &mut dice)
            .into_iter()
            .map(|fish| {
                let ((x, y), r) = fish.haunt.area();
                let pos = (x + dice.range(-r, r), y + dice.range(-r, r) * 0.4);
                let pos = if in_water(pos.0, pos.1) { pos } else { (x, y) };
                Swimmer {
                    fish,
                    length: dice.range(fish.length.0, fish.length.1),
                    pos,
                    shown: pos,
                    heading: dice.range(0.0, TAU),
                    target: pos,
                    state: Swim::Cruising,
                    spooked_until: 0.0,
                    surfaced: !fish.dusk,
                }
            })
            .collect();
        let angler = characters.first();
        let patience = angler.map_or(0.5, |c| 1.0 - c.axes.impulsiveness);
        let calm = angler.map_or(0.5, |c| 1.0 - c.axes.energy);
        let playful = angler.map_or(0.5, |c| c.axes.playfulness);
        let strong = angler.map_or(0.5, |c| (c.axes.energy + c.axes.feistiness) / 2.0);
        Self {
            party,
            swimmers,
            phase: Phase::Arriving { since: now },
            light: LIGHT,
            dusk_told: false,
            creel: Vec::new(),
            basket: Vec::new(),
            events: Vec::new(),
            dice,
            reduce_motion: ground.reduce_motion(),
            last_step: now,
            clock: now,
            holding: false,
            snag: None,
            window: 0.85 + 0.5 * patience,
            stray: 4.0 + 14.0 * (1.0 - (0.6 * patience + 0.4 * calm)),
            lure: 0.8 + 0.5 * playful,
            grip: 1.2 - 0.4 * strong,
            reach: if characters.len() > 1 { 12.0 } else { 0.0 },
            dozing: false,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.party
    }

    pub fn light_left(&self) -> f32 {
        (self.light / LIGHT).clamp(0.0, 1.0)
    }

    /// The fish landed this trip, and their lengths in centimetres.
    pub fn creel(&self) -> &[(&'static str, f32)] {
        &self.creel
    }

    /// Anything that snagged on the line and came home.
    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    /// Whether something is biting right now: the moment to strike.
    pub fn biting(&self) -> bool {
        self.at_float()
            .is_some_and(|index| matches!(self.swimmers[index].state, Swim::Biting { .. }))
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// The part of the pool under a point, for naming it to the person.
    pub fn haunt_at(x: f32, y: f32) -> Option<Haunt> {
        if !in_water(x, y) {
            return None;
        }
        Haunt::ALL.into_iter().find(|haunt| {
            let ((hx, hy), r) = haunt.area();
            distance((x, y), (hx, hy)) <= r
        })
    }

    /// Casts towards a point on the water. The float lands near it: how near depends on who is
    /// fishing.
    pub fn cast(&mut self, ground: &mut Playground, target: (f32, f32), now: f32) {
        if self.phase != Phase::Ready || !in_water(target.0, target.1) {
            return;
        }
        let angle = self.dice.range(0.0, TAU);
        let off = self.dice.range(0.0, self.stray);
        let mut to = (
            target.0 + angle.cos() * off,
            target.1 + angle.sin() * off * 0.5,
        );
        if !in_water(to.0, to.1) {
            to = target;
        }
        self.light -= CAST_COST;
        self.let_go();
        self.wake(ground, now);
        // Whoever was still on the way is on the rock with the rod, and beside it with the net.
        for (index, id) in self.party.iter().enumerate() {
            ground.teleport(*id, if index == 0 { SEAT } else { NET });
        }
        if let Some(angler) = self.party.first() {
            let cast = Beat::new(Gesture::Reach, ExpressionKind::Determined, 0.5);
            ground.direct(*angler, vec![Step::Beat(cast), Step::Beat(sitting())], now);
        }
        self.phase = if self.reduce_motion {
            self.splash(to, now);
            Phase::Waiting {
                float: to,
                since: now,
            }
        } else {
            Phase::Casting { to, since: now }
        };
    }

    /// The person strikes: on a bite, the fish is hooked; on a nibble, it is scared off; on
    /// nothing, the line comes in.
    pub fn strike(&mut self, ground: &mut Playground, now: f32) {
        let Phase::Waiting { float, .. } = self.phase else {
            return;
        };
        if let Some(index) = self.at_float() {
            match self.swimmers[index].state {
                Swim::Biting { .. } => {
                    self.swimmers[index].state = Swim::Hooked;
                    let towards = unit(ROD_TIP, float);
                    self.phase = Phase::Fighting(Fight {
                        swimmer: index,
                        out: distance(ROD_TIP, float),
                        towards,
                        strain: 0.0,
                        pulling_until: now + 0.6,
                        next_pull: now + 1.6,
                    });
                    self.events.push(Event::Hooked {
                        fish: self.swimmers[index].fish.id,
                    });
                    if let Some(angler) = self.party.first() {
                        let mut heave =
                            Beat::new(Gesture::Heave, ExpressionKind::Determined, 600.0);
                        heave.cue = Some(Cue::Exclaim);
                        ground.direct(*angler, vec![Step::Beat(heave)], now);
                    }
                    return;
                }
                Swim::Nibbling { .. } | Swim::Coming => {
                    self.scare(index, now);
                    self.events.push(Event::TooSoon);
                }
                _ => {}
            }
        }
        self.let_go();
        self.phase = Phase::Ready;
    }

    /// The float has left the water: whoever was coming to it, or at it, goes back to cruising.
    fn let_go(&mut self) {
        for swimmer in &mut self.swimmers {
            if matches!(
                swimmer.state,
                Swim::Coming | Swim::Nibbling { .. } | Swim::Biting { .. }
            ) {
                swimmer.state = Swim::Cruising;
                swimmer.target = swimmer.pos;
            }
        }
    }

    /// Whether the person is holding the line in, while something is on it.
    pub fn hold(&mut self, holding: bool) {
        self.holding = holding;
    }

    /// Calls it a day: everyone heads home.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.leave(ground, now);
        }
    }

    fn leave(&mut self, ground: &mut Playground, now: f32) {
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let steps = vec![Step::Walk {
                to: (-30.0 - 20.0 * index as f32, 196.0),
            }];
            ground.direct(*id, steps, now);
        }
        self.events.push(Event::Leaving);
        self.phase = Phase::Leaving { since: now };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        if !matches!(
            self.phase,
            Phase::Arriving { .. } | Phase::Leaving { .. } | Phase::Over
        ) {
            self.light -= LIGHT_PER_SEC * dt;
        }
        if self.light < DUSK && !self.dusk_told {
            self.dusk_told = true;
            for swimmer in &mut self.swimmers {
                swimmer.surfaced = true;
            }
            self.events.push(Event::Dusk);
        }
        self.swim(dt, now);
        match self.phase {
            Phase::Arriving { since } => {
                let busy = self.party.iter().any(|id| ground.busy(*id));
                if !busy || now - since > 5.0 {
                    self.settle(ground, now);
                    self.phase = Phase::Ready;
                }
            }
            Phase::Ready => {
                if self.light <= 0.0 {
                    self.leave(ground, now);
                }
            }
            Phase::Casting { to, since } => {
                if now - since >= FLIGHT_SECS {
                    self.splash(to, now);
                    self.phase = Phase::Waiting {
                        float: to,
                        since: now,
                    };
                }
            }
            Phase::Waiting { float, since } => self.wait(ground, float, since, now),
            Phase::Fighting(fight) => self.fight(ground, fight, dt, now),
            Phase::Landed { since, .. } | Phase::Snagged { since, .. } => {
                if now - since >= ADMIRE_SECS {
                    self.settle(ground, now);
                    self.phase = if self.light <= 0.0 {
                        self.leave(ground, now);
                        self.phase
                    } else {
                        Phase::Ready
                    };
                }
            }
            Phase::Leaving { since } => {
                let gone = self.party.iter().all(|id| !ground.busy(*id));
                if gone || now - since > 8.0 {
                    self.phase = Phase::Over;
                }
            }
            Phase::Over => {}
        }
    }

    /// The one fishing sits on its rock; a second stands by with the net.
    fn settle(&mut self, ground: &mut Playground, now: f32) {
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let (spot, beat) = if index == 0 {
                (SEAT, sitting())
            } else {
                (
                    NET,
                    Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0),
                )
            };
            let steps = vec![
                Step::Walk { to: spot },
                Step::FaceX(ROD_TIP.0 + 40.0),
                Step::Beat(beat),
            ];
            ground.direct(*id, steps, now);
        }
        self.dozing = false;
    }

    /// The one fishing wakes with a start, if it had nodded off.
    fn wake(&mut self, ground: &mut Playground, now: f32) {
        if !self.dozing {
            return;
        }
        self.dozing = false;
        if let Some(angler) = self.party.first() {
            let mut start = Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.5);
            start.cue = Some(Cue::Exclaim);
            ground.direct(*angler, vec![Step::Beat(start), Step::Beat(sitting())], now);
        }
    }

    /// The float lands: anything wary close by bolts, and the line may have caught on something.
    fn splash(&mut self, at: (f32, f32), now: f32) {
        for index in 0..self.swimmers.len() {
            let swimmer = &self.swimmers[index];
            if swimmer.surfaced && distance(swimmer.pos, at) < swimmer.fish.wary {
                self.scare(index, now);
                self.events.push(Event::Spooked {
                    fish: self.swimmers[index].fish.id,
                });
            }
        }
        self.snag = None;
        if self.dice.chance(SNAG) {
            self.snag = finds::surely(Kind::Scoop, &[], &[], |_| false, &mut self.dice);
        }
    }

    fn scare(&mut self, index: usize, now: f32) {
        let swimmer = &mut self.swimmers[index];
        swimmer.state = Swim::Cruising;
        swimmer.spooked_until = now + SPOOKED_SECS;
        swimmer.heading += PI;
        let ((x, y), r) = swimmer.fish.haunt.area();
        swimmer.target = (x - (swimmer.pos.0 - x).signum() * r, y);
    }

    /// The one engaged with the float, if any.
    fn at_float(&self) -> Option<usize> {
        self.swimmers.iter().position(|swimmer| {
            matches!(
                swimmer.state,
                Swim::Coming | Swim::Nibbling { .. } | Swim::Biting { .. }
            )
        })
    }

    fn wait(&mut self, ground: &mut Playground, float: (f32, f32), since: f32, now: f32) {
        // Something snagged, a moment after the cast.
        if let Some(find) = self.snag {
            if now - since > 1.0 {
                self.snag = None;
                self.let_go();
                self.basket.push(find.id);
                self.events.push(Event::Snagged);
                self.phase = Phase::Snagged {
                    find: find.id,
                    since: now,
                };
            }
            return;
        }
        // A lazybones left waiting long enough nods off, and the fish seem bolder for it.
        let lazy = self
            .party
            .first()
            .and_then(|id| ground.character(*id))
            .is_some_and(|c| c.kind == TemperamentKind::Lazybones);
        if lazy && !self.dozing && now - since > 8.0 && self.at_float().is_none() {
            self.dozing = true;
            if let Some(angler) = self.party.first() {
                let mut doze = Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 600.0);
                doze.cue = Some(Cue::Sleep);
                ground.direct(*angler, vec![Step::Beat(doze)], now);
            }
        }
        match self.at_float() {
            None => {
                // Anyone near enough may come over to look, the readier the lure.
                let lure = self.lure * if self.dozing { 1.3 } else { 1.0 };
                let dt = 1.0 / 30.0;
                let mut came = None;
                for (index, swimmer) in self.swimmers.iter().enumerate() {
                    if swimmer.surfaced
                        && now >= swimmer.spooked_until
                        && distance(swimmer.pos, float) < NOTICE
                        && self.dice.chance(0.35 * lure * dt)
                    {
                        came = Some(index);
                        break;
                    }
                }
                if let Some(index) = came {
                    self.swimmers[index].state = Swim::Coming;
                    self.swimmers[index].target = float;
                    self.point_out(ground, index, now);
                }
            }
            Some(index) => {
                let fish = self.swimmers[index].fish;
                match self.swimmers[index].state {
                    Swim::Coming => {
                        if distance(self.swimmers[index].pos, float) < 3.0 {
                            let (low, high) = fish.nibbles;
                            let left = low + (self.dice.unit() * f32::from(high - low + 1)) as u8;
                            self.swimmers[index].state = Swim::Nibbling {
                                left: left.min(high),
                                next: now + self.dice.range(0.6, 1.2),
                            };
                        }
                    }
                    Swim::Nibbling { left, next } if now >= next => {
                        if left == 0 {
                            self.swimmers[index].state = Swim::Biting {
                                until: now + fish.bite * self.window,
                            };
                            self.events.push(Event::Bite);
                            self.wake(ground, now);
                        } else {
                            self.swimmers[index].state = Swim::Nibbling {
                                left: left - 1,
                                next: now + self.dice.range(0.7, 1.3),
                            };
                            self.events.push(Event::Nibble);
                        }
                    }
                    Swim::Biting { until } if now >= until => {
                        // Too late: it takes the bait and goes.
                        self.scare(index, now);
                        self.events.push(Event::Stolen);
                        self.phase = Phase::Ready;
                    }
                    _ => {}
                }
            }
        }
    }

    /// A curious second companion points out a big one coming in.
    fn point_out(&mut self, ground: &mut Playground, index: usize, now: f32) {
        let Some(helper) = self.party.get(1).copied() else {
            return;
        };
        let curious = ground.character(helper).map_or(0.0, |c| c.axes.curiosity);
        let big = self.swimmers[index].fish.tier >= Tier::Uncommon;
        if big && self.dice.chance(0.3 + 0.6 * curious) {
            let mut point = Beat::new(Gesture::Reach, ExpressionKind::Curious, 0.8);
            point.cue = Some(Cue::Exclaim);
            let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
            ground.direct(helper, vec![Step::Beat(point), Step::Beat(watch)], now);
            self.events.push(Event::Pointed {
                who: helper,
                haunt: self.swimmers[index].fish.haunt,
            });
        }
    }

    fn fight(&mut self, ground: &mut Playground, mut fight: Fight, dt: f32, now: f32) {
        let fish = self.swimmers[fight.swimmer].fish;
        let length = self.swimmers[fight.swimmer].length;
        // Bigger of its kind pulls harder.
        let (low, high) = fish.length;
        let heft = 0.8 + 0.4 * ((length - low) / (high - low).max(1.0)).clamp(0.0, 1.0);
        let pulling = now < fight.pulling_until;
        if now >= fight.next_pull {
            fight.pulling_until = now + self.dice.range(0.5, 1.2) * (0.5 + fish.strength);
            fight.next_pull =
                fight.pulling_until + self.dice.range(0.8, 2.0) * (1.4 - fish.strength * 0.6);
        }
        if pulling {
            fight.out += PULL_SPEED * fish.strength * heft * dt;
            if self.holding {
                fight.strain += STRAIN * fish.strength * heft * self.grip * dt * 1.6;
            } else {
                fight.strain -= EASE * 0.4 * dt;
            }
        } else if self.holding {
            fight.out -= REEL_SPEED * dt;
            fight.strain -= EASE * 0.5 * dt;
        } else {
            fight.strain -= EASE * dt;
        }
        fight.strain = fight.strain.clamp(0.0, 1.2);
        fight.out = fight
            .out
            .min(distance(ROD_TIP, (WATER.0, WATER.1 - WATER.3)) + 20.0);
        let at = (
            ROD_TIP.0 + fight.towards.0 * fight.out,
            ROD_TIP.1 + fight.towards.1 * fight.out,
        );
        self.swimmers[fight.swimmer].pos = at;
        self.swimmers[fight.swimmer].shown = at;
        if fight.strain >= 1.0 {
            // Snap: it's away.
            self.scare(fight.swimmer, now);
            self.events.push(Event::Snapped);
            if let Some(angler) = self.party.first()
                && let Some(character) = ground.character(*angler)
            {
                let beat = if character.kind == TemperamentKind::Grump {
                    let mut stomp = Beat::new(Gesture::Stomp, ExpressionKind::Grumpy, 0.9);
                    stomp.cue = Some(Cue::Huff);
                    stomp
                } else {
                    Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.7)
                };
                ground.direct(*angler, vec![Step::Beat(beat), Step::Beat(sitting())], now);
            }
            self.phase = Phase::Ready;
            return;
        }
        if fight.out <= self.landing() {
            self.land(ground, fight.swimmer, now);
            return;
        }
        self.phase = Phase::Fighting(fight);
    }

    /// How near the rod tip a fish has to come to be landed: further with someone holding a net.
    fn landing(&self) -> f32 {
        LANDING + self.reach
    }

    fn land(&mut self, ground: &mut Playground, index: usize, now: f32) {
        let swimmer = self.swimmers.remove(index);
        let length = (swimmer.length * 10.0).round() / 10.0;
        self.creel.push((swimmer.fish.id, length));
        self.events.push(Event::Landed {
            fish: swimmer.fish.id,
            length,
        });
        for (order, id) in self.party.iter().enumerate() {
            if let Some(character) = ground.character(*id) {
                let mut beats = Vec::new();
                if order == 1 {
                    beats.push(Beat::new(Gesture::Reach, ExpressionKind::Joy, 0.6));
                }
                beats.extend(delighted(character, swimmer.fish.tier));
                ground.direct(*id, beats.into_iter().map(Step::Beat).collect(), now);
            }
        }
        self.phase = Phase::Landed {
            fish: swimmer.fish.id,
            length,
            since: now,
        };
        // A common one's place is taken by another of its kind, in time; a rarer one is caught for
        // the day.
        if swimmer.fish.tier == Tier::Common {
            let ((x, y), _) = swimmer.fish.haunt.area();
            let (low, high) = swimmer.fish.length;
            self.swimmers.push(Swimmer {
                length: self.dice.range(low, high),
                pos: (x, y),
                shown: (x, y),
                target: (x, y),
                state: Swim::Cruising,
                spooked_until: now + SPOOKED_SECS * 2.0,
                ..swimmer
            });
        }
    }

    /// The fish move about their haunts: each heads for a point in its part of the pool, turning
    /// gently, and picks another when it gets there. One coming to the float heads for that.
    fn swim(&mut self, dt: f32, now: f32) {
        let step_now = self.reduce_motion && now - self.last_step >= STEP_SECS;
        if step_now {
            self.last_step = now;
        }
        for swimmer in &mut self.swimmers {
            if swimmer.state == Swim::Hooked {
                continue;
            }
            let fleeing = now < swimmer.spooked_until;
            let to = swimmer.target;
            if distance(swimmer.pos, to) < 10.0 && swimmer.state == Swim::Cruising {
                let ((x, y), r) = swimmer.fish.haunt.area();
                let angle = self.dice.range(0.0, TAU);
                let reach = self.dice.range(0.0, r);
                let next = (x + angle.cos() * reach, y + angle.sin() * reach * 0.45);
                swimmer.target = if in_water(next.0, next.1) {
                    next
                } else {
                    (x, y)
                };
            }
            let nibbling = matches!(swimmer.state, Swim::Nibbling { .. } | Swim::Biting { .. });
            if nibbling {
                continue;
            }
            // One coming to the float swims straight to it.
            if swimmer.state == Swim::Coming {
                let gap = distance(swimmer.pos, to);
                let step = (swimmer.fish.speed * 1.2 * dt).min(gap);
                let (ux, uy) = unit(swimmer.pos, to);
                swimmer.heading = uy.atan2(ux);
                swimmer.pos = (swimmer.pos.0 + ux * step, swimmer.pos.1 + uy * step);
                if !self.reduce_motion || step_now {
                    swimmer.shown = swimmer.pos;
                }
                continue;
            }
            let want = (to.1 - swimmer.pos.1).atan2(to.0 - swimmer.pos.0);
            let turn =
                ((want - swimmer.heading + PI).rem_euclid(TAU) - PI).clamp(-3.0 * dt, 3.0 * dt);
            swimmer.heading += turn;
            let pace = swimmer.fish.speed
                * if fleeing { 2.2 } else { 1.0 }
                * if swimmer.state == Swim::Coming {
                    1.2
                } else {
                    0.6
                };
            let next = (
                swimmer.pos.0 + swimmer.heading.cos() * pace * dt,
                swimmer.pos.1 + swimmer.heading.sin() * pace * dt * 0.6,
            );
            if in_water(next.0, next.1) {
                swimmer.pos = next;
            } else {
                swimmer.heading += PI * 0.5;
            }
            if !self.reduce_motion || step_now {
                swimmer.shown = swimmer.pos;
            }
        }
    }

    /// The glade darkening, the fish beneath the surface, the line and float, the strain, the
    /// fish held up, and the creel.
    pub fn draw(&self, scene: &mut Canvas, now: f32) {
        dim(scene, self.light);
        for swimmer in self.swimmers.iter().filter(|s| s.surfaced) {
            draw_shadow(scene, swimmer, now, self.reduce_motion);
        }
        let tip = (ROD_TIP.0 as i32, ROD_TIP.1 as i32);
        draw_rod(scene, tip);
        let bob = |since: f32| {
            if self.reduce_motion {
                0
            } else {
                ((now - since) * 3.0).sin().round() as i32
            }
        };
        match self.phase {
            Phase::Casting { to, since } => {
                let t = ((now - since) / FLIGHT_SECS).clamp(0.0, 1.0);
                let x = ROD_TIP.0 + (to.0 - ROD_TIP.0) * t;
                let arc = (t * PI).sin() * 26.0;
                let y = ROD_TIP.1 + (to.1 - ROD_TIP.1) * t - arc;
                draw_line(scene, tip, (x as i32, y as i32), 0.0);
                draw_float(scene, (x as i32, y as i32), 0);
            }
            Phase::Waiting { float, since } => {
                let engaged = self.at_float().map(|index| self.swimmers[index].state);
                let (dip, ring) = match engaged {
                    Some(Swim::Biting { .. }) => (4, true),
                    Some(Swim::Nibbling { next, .. }) if next - now < 0.25 => (1, true),
                    _ => (bob(since), false),
                };
                let at = (float.0 as i32, float.1 as i32);
                draw_line(scene, tip, at, 4.0);
                if ring {
                    ripple(scene, at, if dip > 2 { 7 } else { 4 });
                }
                draw_float(scene, at, dip);
            }
            Phase::Fighting(fight) => {
                let at = (
                    (ROD_TIP.0 + fight.towards.0 * fight.out) as i32,
                    (ROD_TIP.1 + fight.towards.1 * fight.out) as i32,
                );
                draw_line(scene, tip, at, 0.0);
                if !self.reduce_motion && (now * 9.0) as i32 % 2 == 0 {
                    ripple(scene, at, 5);
                } else {
                    ripple(scene, at, 3);
                }
                draw_strain(scene, fight.strain);
            }
            Phase::Landed { fish, since, .. } => {
                let catch = super::art::catch(fish);
                let rise = if self.reduce_motion {
                    10
                } else {
                    (((now - since) / 0.4).min(1.0) * 10.0) as i32
                };
                let (width, height) = (catch.width() as i32, catch.height() as i32);
                let (left, top) = (
                    SEAT.0 as i32 + 8 - width / 2,
                    SEAT.1 as i32 - 44 - height - rise,
                );
                rect(
                    scene,
                    left - 3,
                    top - 3,
                    width + 6,
                    height + 6,
                    rgba(0xf6eed8, 225),
                );
                crate::paint::blit(scene, &catch, left, top);
            }
            Phase::Snagged { find, since } => {
                let icon = finds::art::icon(find);
                let rise = if self.reduce_motion {
                    8
                } else {
                    (((now - since) / 0.4).min(1.0) * 8.0) as i32
                };
                let (left, top) = (ROD_TIP.0 as i32 - 4, ROD_TIP.1 as i32 - 6 - rise);
                rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
                crate::paint::blit(scene, &icon, left, top);
            }
            _ => {}
        }
        self.draw_creel(scene);
    }

    /// The creel in the top right corner: an icon for each fish landed, and the light left.
    fn draw_creel(&self, scene: &mut Canvas) {
        const SLOT: i32 = 11;
        const SLOTS: i32 = 6;
        let width = SLOT * SLOTS + 3;
        let (left, top) = (scene.width() as i32 - width - 4, 4);
        rect(scene, left, top, width, SLOT + 7, rgba(0x1e2a2a, 150));
        let shown: Vec<&str> = self
            .creel
            .iter()
            .map(|(id, _)| *id)
            .rev()
            .take(SLOTS as usize)
            .collect();
        for slot in 0..SLOTS {
            let (x, y) = (left + 2 + slot * SLOT, top + 2);
            rect(scene, x, y, SLOT - 1, SLOT - 1, rgba(0xf6eed8, 60));
            if let Some(id) = shown.get(slot as usize) {
                crate::paint::blit(scene, &super::art::icon(id), x, y);
            }
        }
        let left_light = self.light_left();
        let bar = ((width - 4) as f32 * left_light) as i32;
        let gold = mix(rgb(0x6a5a9a), rgb(0xf5d25e), left_light);
        scene.fill_rect(left + 2, top + SLOT + 2, bar, 2, gold);
    }
}

/// Sitting on the bank with the rod, for as long as it takes.
fn sitting() -> Beat {
    Beat::new(Gesture::Sit, ExpressionKind::Content, 600.0)
}

fn unit(from: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let length = distance(from, to).max(0.001);
    ((to.0 - from.0) / length, (to.1 - from.1) / length)
}

/// A fish's shadow under the surface: a soft dark body and a tail that sways as it swims, sized
/// to it. With reduced motion the tail is still.
fn draw_shadow(scene: &mut Canvas, swimmer: &Swimmer, now: f32, reduce_motion: bool) {
    let long = drawn_length(swimmer.length) as f32 * 0.7;
    let (x, y) = swimmer.shown;
    let (dx, dy) = (swimmer.heading.cos(), swimmer.heading.sin() * 0.6);
    let shade = rgba(0x0e2228, 70);
    let half = (long / 2.0).max(2.0);
    let thick = (long / 6.0).max(1.0);
    let steps = (long as i32).max(3);
    for step in 0..=steps {
        let t = step as f32 / steps as f32 * 2.0 - 1.0;
        let width = thick * (1.0 - t * t).sqrt().max(0.25);
        let (px, py) = (x + dx * half * t, y + dy * half * t);
        for w in -(width as i32)..=(width as i32) {
            put(
                scene,
                (px - dy * w as f32) as i32,
                (py + dx * w as f32) as i32,
                shade,
            );
        }
    }
    let sway = if reduce_motion {
        0.0
    } else {
        (now * 8.0 + x).sin() * thick
    };
    // The tail: a little fan behind the body, swaying.
    let fin = thick + 1.0;
    for along in 1..=(fin as i32 + 1) {
        let back = half + along as f32;
        let spread = along as f32 * 0.9;
        for w in [-spread, spread] {
            let (px, py) = (x - dx * back, y - dy * back);
            put(
                scene,
                (px - dy * (w + sway)) as i32,
                (py + dx * (w + sway)) as i32,
                shade,
            );
        }
    }
}

/// The rod, from the hands of the one fishing up to its tip.
fn draw_rod(scene: &mut Canvas, tip: (i32, i32)) {
    let butt = (SEAT.0 as i32 + 8, SEAT.1 as i32 - 12);
    line(scene, butt, tip, rgb(0x5a3e2a));
    line(
        scene,
        (butt.0 + 1, butt.1),
        (tip.0 + 1, tip.1),
        rgb(0x8a6040),
    );
    line(scene, butt, (butt.0 + 3, butt.1 - 4), rgb(0xc8a070));
    put(scene, tip.0, tip.1, rgb(0xd0584c));
}

/// The line from the rod tip to the float, sagging a little.
fn draw_line(scene: &mut Canvas, from: (i32, i32), to: (i32, i32), sag: f32) {
    let steps = 40;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
        let y = from.1 as f32 + (to.1 - from.1) as f32 * t + (t * PI).sin() * sag;
        put(scene, x as i32, y as i32, rgba(0xeee6d4, 150));
    }
}

/// The float: red on top, white below, `dip` pixels pulled under.
fn draw_float(scene: &mut Canvas, at: (i32, i32), dip: i32) {
    let (x, y) = (at.0, at.1 + dip);
    if dip < 4 {
        put(scene, x, y - 3, rgb(0x8a2a30));
        put(scene, x, y - 2, rgb(0xd04a40));
        put(scene, x - 1, y - 2, rgb(0xe87058));
    }
    if dip < 2 {
        put(scene, x, y - 1, rgb(0xf6eed8));
        put(scene, x - 1, y - 1, rgb(0xffffff));
    }
    ellipse(scene, x, at.1 + 1, 2, 1, rgba(0x0e2228, 60));
}

fn ripple(scene: &mut Canvas, at: (i32, i32), radius: i32) {
    for step in 0..32 {
        let angle = step as f32 / 32.0 * TAU;
        put(
            scene,
            at.0 + (angle.cos() * radius as f32) as i32,
            at.1 + (angle.sin() * radius as f32 * 0.4) as i32,
            rgba(0xe9f6f7, 170),
        );
    }
}

/// How near the line is to snapping, on a little card over the one fishing: green to red.
fn draw_strain(scene: &mut Canvas, strain: f32) {
    let (x, y) = (SEAT.0 as i32 - 16, SEAT.1 as i32 - 42);
    rect(scene, x - 2, y - 2, 36, 7, rgba(0xf6eed8, 220));
    scene.fill_rect(x, y, 32, 3, rgb(0x5a4a3a));
    let filled = (32.0 * strain.clamp(0.0, 1.0)) as i32;
    let color = mix(rgb(0x6ab04a), rgb(0xd04a40), strain.clamp(0.0, 1.0));
    scene.fill_rect(x, y, filled, 3, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::fishing;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn trip(cast: &Cast, party: Vec<Id>) -> (Playground, Angling) {
        let mut ground = fishing::open(cast, &party, 0.0);
        let outset = Outset {
            party,
            drought: 0,
            seed: 9,
        };
        let angling = Angling::new(&mut ground, outset, |_| false, 0.0);
        (ground, angling)
    }

    fn run(
        ground: &mut Playground,
        angling: &mut Angling,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            angling.tick(ground, now);
            events.extend(angling.take_events());
        }
        events
    }

    /// Casts where the most fish are and waits, striking on the bite, until something is on.
    fn cast_and_hook(
        ground: &mut Playground,
        angling: &mut Angling,
        cast: &Cast,
        from: f32,
    ) -> f32 {
        let mut now = from;
        while now < from + 240.0 {
            if angling.phase() == Phase::Ready {
                let target = angling
                    .swimmers
                    .iter()
                    .filter(|s| s.surfaced && now >= s.spooked_until)
                    .map(|s| s.pos)
                    .next()
                    .unwrap_or((200.0, 112.0));
                // Not right on top of it, so as not to scare it.
                let aim = [
                    (target.0 + 34.0, target.1),
                    (target.0 - 34.0, target.1),
                    target,
                ]
                .into_iter()
                .find(|(x, y)| in_water(*x, *y))
                .unwrap_or(target);
                angling.cast(ground, aim, now);
            }
            if let Phase::Waiting { .. } = angling.phase()
                && let Some(index) = angling.at_float()
                && matches!(angling.swimmers[index].state, Swim::Biting { .. })
            {
                angling.strike(ground, now);
            }
            if matches!(angling.phase(), Phase::Fighting(_)) {
                return now;
            }
            run(ground, angling, cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
        }
        panic!("nothing bit in four minutes: {:?}", angling.phase());
    }

    #[test]
    fn a_patient_hand_lands_a_fish() {
        let cast = sample();
        let (mut ground, mut angling) = trip(&cast, vec![cast.members[0].id, cast.members[1].id]);
        run(&mut ground, &mut angling, &cast, 0.0, 6.0);
        assert_eq!(angling.phase(), Phase::Ready);
        let mut now = cast_and_hook(&mut ground, &mut angling, &cast, 6.0);
        // Reel only while it isn't pulling, as a patient hand does.
        let until = now + 60.0;
        while now < until && matches!(angling.phase(), Phase::Fighting(_)) {
            let Phase::Fighting(fight) = angling.phase() else {
                break;
            };
            angling.hold(now >= fight.pulling_until && fight.strain < 0.6);
            run(&mut ground, &mut angling, &cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
        }
        assert!(
            matches!(angling.phase(), Phase::Landed { .. }),
            "{:?}",
            angling.phase()
        );
        assert_eq!(angling.creel().len(), 1);
    }

    #[test]
    fn holding_on_through_every_pull_snaps_the_line() {
        let cast = sample();
        let (mut ground, mut angling) = trip(&cast, vec![cast.members[2].id]);
        run(&mut ground, &mut angling, &cast, 0.0, 6.0);
        let now = cast_and_hook(&mut ground, &mut angling, &cast, 6.0);
        // The strongest fish there is, pulling at once, so it can't simply be reeled in first.
        if let Phase::Fighting(mut fight) = angling.phase() {
            angling.swimmers[fight.swimmer].fish = fish::fish("old_one").unwrap();
            angling.swimmers[fight.swimmer].length = 110.0;
            fight.out = 120.0;
            fight.next_pull = now;
            angling.phase = Phase::Fighting(fight);
        }
        angling.hold(true);
        let events = run(&mut ground, &mut angling, &cast, now, now + 60.0);
        assert!(events.contains(&Event::Snapped), "{events:?}");
        assert!(angling.creel().is_empty());
    }

    #[test]
    fn striking_on_a_nibble_scares_the_fish_off() {
        let cast = sample();
        let (mut ground, mut angling) = trip(&cast, vec![cast.members[0].id]);
        run(&mut ground, &mut angling, &cast, 0.0, 6.0);
        let mut now = 6.0;
        let mut struck = false;
        while now < 300.0 && !struck {
            if angling.phase() == Phase::Ready {
                let target = angling
                    .swimmers
                    .iter()
                    .find(|s| s.surfaced && s.fish.nibbles.1 > 0 && now >= s.spooked_until)
                    .map(|s| s.pos);
                if let Some(target) = target {
                    let aim = [
                        (target.0 + 34.0, target.1),
                        (target.0 - 34.0, target.1),
                        target,
                    ]
                    .into_iter()
                    .find(|(x, y)| in_water(*x, *y))
                    .unwrap_or(target);
                    angling.cast(&mut ground, aim, now);
                }
            }
            if let Some(index) = angling.at_float()
                && matches!(angling.swimmers[index].state, Swim::Nibbling { .. })
            {
                angling.strike(&mut ground, now);
                assert!(angling.swimmers[index].spooked_until > now);
                assert_eq!(angling.phase(), Phase::Ready);
                struck = true;
            }
            run(&mut ground, &mut angling, &cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
        }
        assert!(struck, "nothing nibbled");
        assert!(angling.take_events().contains(&Event::TooSoon) || struck);
    }

    #[test]
    fn the_old_one_waits_for_dusk_and_the_light_runs_out() {
        let cast = sample();
        let (mut ground, mut angling) = trip(&cast, vec![cast.members[0].id]);
        // However the trip goes, the dusk's fish are hidden until the light is low.
        assert!(
            angling
                .swimmers
                .iter()
                .filter(|s| s.fish.dusk)
                .all(|s| !s.surfaced)
        );
        let events = run(&mut ground, &mut angling, &cast, 0.0, 600.0);
        assert!(events.contains(&Event::Dusk));
        assert!(angling.swimmers.iter().all(|s| s.surfaced));
        assert!(events.contains(&Event::Leaving), "{events:?}");
        assert_eq!(angling.phase(), Phase::Over);
    }

    #[test]
    fn who_is_fishing_changes_how_it_goes() {
        let cast = sample();
        let tunings: Vec<(f32, f32, f32, f32)> = cast
            .members
            .iter()
            .map(|member| {
                let (_, angling) = trip(&cast, vec![member.id]);
                (angling.window, angling.stray, angling.lure, angling.grip)
            })
            .collect();
        assert!(tunings.windows(2).any(|pair| pair[0] != pair[1]));
        let (_, alone) = trip(&cast, vec![cast.members[0].id]);
        let (_, paired) = trip(&cast, vec![cast.members[0].id, cast.members[1].id]);
        assert!(paired.landing() > alone.landing(), "a net reaches further");
    }

    #[test]
    fn with_reduced_motion_fish_move_in_steps_and_the_cast_is_a_cut() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut angling) = trip(&cast, vec![cast.members[0].id]);
        run(&mut ground, &mut angling, &cast, 0.0, 6.0);
        // Over six seconds, fish that glide would be drawn somewhere new nearly every frame; in
        // steps, only every step.
        let mut changes = 0;
        let mut last: Vec<(f32, f32)> = angling.swimmers.iter().map(|s| s.shown).collect();
        let mut now = 6.0;
        while now < 12.0 {
            run(&mut ground, &mut angling, &cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
            let shown: Vec<(f32, f32)> = angling.swimmers.iter().map(|s| s.shown).collect();
            if shown != last {
                changes += 1;
                last = shown;
            }
        }
        assert!(
            (2..=6).contains(&changes),
            "drawn somewhere new {changes} times in six seconds"
        );
        let ((x, y), _) = Haunt::Deep.area();
        angling.cast(&mut ground, (x, y), 9.0);
        assert!(
            matches!(angling.phase(), Phase::Waiting { .. }),
            "the float flew"
        );
    }

    #[test]
    fn haunts_are_named_where_they_are() {
        for haunt in Haunt::ALL {
            let ((x, y), _) = haunt.area();
            assert_eq!(Angling::haunt_at(x, y), Some(haunt));
        }
        assert_eq!(Angling::haunt_at(SEAT.0, SEAT.1), None);
    }
}
