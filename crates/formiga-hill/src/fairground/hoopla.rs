//! Hoopla, played by the colony at the hoopla stall while the person watches. Each player in turn
//! takes its three rings off the end of the counter, goes back to the line beside the stall, and
//! throws them one at a time at the pegs and prizes set out along the counter: each ring arcs over
//! and rings a prize, bounces off a peg, or falls short in the sawdust. Whoever is waiting queues
//! along the front of the carousel; whoever is not playing, or has had its go, watches from in
//! front of the stall.
//!
//! Nobody is written in. How true each ring flies comes from the companion: a patient or
//! scholarly one steadies itself and measures every throw, an impulsive one throws fast and
//! loose, a show-off throws its last behind its back (wild, but now and then spectacular), a
//! lazybones throws sitting down, and a little one gets to stand at the nearer line. A prize rung
//! is the winner's to carry about for the rest of the visit, unless it is an affectionate one, who
//! takes it straight over to its closest friend. A miss is only a miss: the rings go back on the
//! counter, and nobody's turn costs anything.

use super::gear::{self, Hoop};
use super::prizes::Prize;
use super::scenery::COUNTER;
use super::{LINE_GAP, Path, line_up, sizes};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::{Playground, Prop};
use formiga_art::ExpressionKind;
use formiga_core::{ActionKind, Gesture, TemperamentKind};

/// The row everything set out on the counter stands on.
pub const COUNTER_TOP: i32 = COUNTER.1 - 1;
/// The pegs along the counter, and the blocks the prizes stand on between them: where the middle
/// of each is. The nearest block to the line has the windmill on it, then the balloon, and the
/// pennant furthest.
pub const PEGS: [i32; 4] = [292, 306, 320, 334];
pub const BLOCKS: [i32; 3] = [299, 313, 327];
/// Where the rings are stacked, at the end of the counter nearest the line.
pub const STACK: (i32, i32) = (283, COUNTER_TOP);
/// How high a peg stands, to the top of its knob.
pub const PEG_TALL: i32 = 8;
/// Where a player throws from: the line, beside the stall; and the nearer line, for a little one.
const LINE: (f32, f32) = (246.0, 128.0);
const NEAR_LINE: (f32, f32) = (260.0, 128.0);
/// Where a player stands to take its rings off the end of the counter.
const COLLECT: (f32, f32) = (268.0, 126.0);
/// Where those waiting their turn queue, along the front of the carousel and facing the stall, in
/// two rows.
const QUEUE: [Path; 2] = [
    &[(230.0, 131.0), (138.0, 131.0)],
    &[(232.0, 168.0), (138.0, 168.0)],
];
/// Where everyone watches from: in front of the stall, in rows, clear of where a ring falls short
/// and of the handcart.
const WATCH: [Path; 3] = [
    &[(286.0, 148.0), (352.0, 148.0)],
    &[(270.0, 168.0), (356.0, 168.0)],
    &[(258.0, 188.0), (332.0, 188.0)],
];
/// How many rings each player throws.
pub const RINGS: usize = 3;
/// Taking the rings off the counter; measuring a throw; turning round to throw one behind its
/// back; winding up; and the throw itself.
const PICK_SECS: f32 = 0.6;
const MEASURE_SECS: f32 = 0.9;
const TURN_ROUND_SECS: f32 = 1.0;
const WIND_SECS: f32 = 0.35;
const HASTY_WIND_SECS: f32 = 0.12;
const THROW_SECS: f32 = 0.3;
/// How long each one takes a ring landing, and an impulsive one, before the next.
const AFTER_SECS: f32 = 0.9;
const HASTY_AFTER_SECS: f32 = 0.45;
/// Handing what it won over to a friend.
const HANDING_SECS: f32 = 1.3;
/// The longest anyone is given to get anywhere.
const WALKING_SECS: f32 = 10.0;
/// How long everyone cheers at the end.
const CELEBRATE_SECS: f32 = 4.0;
/// How long a ring hangs in the air after a peg, coming down.
const BOUNCE_SECS: f32 = 0.35;
/// How long a rung prize shines for.
pub const SHINE_SECS: f32 = 1.6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Ready,
    Playing {
        since: f32,
    },
    /// Everyone has had a go; the colony is cheering whoever rang the most.
    Over {
        since: f32,
    },
}

/// Where a ring ends up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    /// Over a prize's block: the prize is won.
    Rung { prize: Prize },
    /// Off a peg, and down onto the counter or into the sawdust.
    Peg,
    /// Short of the counter, in the sawdust.
    Short,
}

/// Something that happened at the stall, for the person watching to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Up to the line, or the nearer line for a little one.
    Up { player: Id, near: bool },
    /// Steadying itself, and measuring the throw.
    Measures { player: Id },
    /// Turning its back to throw.
    BehindTheBack { player: Id },
    /// A ring came down: the first ring is 0.
    Landed {
        player: Id,
        ring: usize,
        landing: Landing,
        behind_back: bool,
    },
    /// What it won, taken over to its closest friend.
    Gave { from: Id, to: Id, prize: Prize },
    /// Its three rings thrown, and how many rang a prize.
    TurnOver { player: Id, rings: u32 },
    /// Everyone has had a go: see `tally`.
    AllDone,
}

/// One throw, planned from who is throwing: what it aims at, where the ring ends up, and how it
/// is thrown.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Throw {
    /// The block it is aimed at.
    aim: usize,
    landing: Landing,
    behind_back: bool,
    /// For a ring off a peg, which peg; and where any ring that misses comes down to rest.
    peg: usize,
    down: (i32, i32),
    /// How long it is readied for, and how long the ring is in the air.
    ready: f32,
    flight: f32,
}

/// What it does before each throw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Manner {
    /// Steadies itself and measures it up first.
    Measures,
    /// Throws fast, without a pause.
    Hasty,
    /// Throws sitting down.
    Sitting,
    Plain,
}

#[derive(Clone, Debug)]
struct Turn {
    player: Id,
    near: bool,
    manner: Manner,
    throws: Vec<Throw>,
    /// Its closest friend here, to give what it wins to, if it is affectionate.
    give_to: Option<Id>,
}

impl Turn {
    /// How many of its rings rang a prize.
    fn rung(&self) -> u32 {
        self.throws
            .iter()
            .filter(|throw| matches!(throw.landing, Landing::Rung { .. }))
            .count() as u32
    }

    /// The last prize it rang, which it carries away.
    fn won(&self) -> Option<Prize> {
        self.throws
            .iter()
            .rev()
            .find_map(|throw| match throw.landing {
                Landing::Rung { prize } => Some(prize),
                _ => None,
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    /// Off to the end of the counter for its rings.
    Collecting {
        since: f32,
    },
    Picking {
        until: f32,
    },
    /// Back to its line with them.
    ToTheLine {
        since: f32,
    },
    /// Measuring, or turning its back, and winding up.
    Readying {
        throw: usize,
        until: f32,
    },
    /// The ring leaves its paws at `since`, from `from`.
    Flying {
        throw: usize,
        since: f32,
        from: (f32, f32),
    },
    Landed {
        throw: usize,
        until: f32,
    },
    /// Off to its friend with what it won.
    Giving {
        since: f32,
    },
    Handing {
        until: f32,
    },
    Done {
        until: f32,
    },
}

/// Where a ring thrown this turn came to rest.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Rest {
    /// Round a prize's block.
    Round(usize),
    /// Lying on the counter, or in the sawdust.
    Counter((i32, i32)),
    Ground((i32, i32)),
}

/// How steady a companion's throwing paw is, from 0 to 1: its patience above all, and a scholar's
/// care.
pub fn steadiness(character: &Character) -> f32 {
    let mut steady = 0.22 + 0.62 * (1.0 - character.axes.impulsiveness);
    if character.kind == TemperamentKind::Scholar {
        steady += 0.1;
    }
    steady.clamp(0.1, 0.95)
}

/// How far a companion throws well, in pixels: further the bigger, livelier and feistier it is,
/// less far sitting down.
pub fn reach(character: &Character, sitting: bool) -> f32 {
    let a = character.axes;
    let arm = 0.45 * character.size.clamp(0.3, 1.3) + 0.3 * a.energy + 0.25 * a.feistiness;
    let reach = 34.0 + 46.0 * arm;
    if sitting { reach - 12.0 } else { reach }
}

/// Whether a companion gets to stand at the nearer line: a little one.
fn little(character: &Character) -> bool {
    character.parent.is_some() || character.size < 0.8
}

/// How a companion goes about each throw.
fn manner(character: &Character) -> Manner {
    if character.kind == TemperamentKind::Lazybones {
        Manner::Sitting
    } else if character.kind == TemperamentKind::Scholar || character.axes.impulsiveness < 0.3 {
        Manner::Measures
    } else if character.axes.impulsiveness > 0.65 {
        Manner::Hasty
    } else {
        Manner::Plain
    }
}

/// Whether a companion gives whatever it wins away to its closest friend.
fn giving(character: &Character) -> bool {
    character.kind == TemperamentKind::Sweetheart || character.axes.affection >= 0.85
}

/// How keen a companion is to have a go: whoever is keenest goes first.
fn keenness(character: &Character) -> f32 {
    let a = character.axes;
    let showy = if character.kind == TemperamentKind::Showoff {
        0.3
    } else {
        0.0
    };
    0.4 * a.playfulness + 0.3 * a.boldness + 0.3 * a.social + showy
}

/// A player's three throws, planned from who it is. Each is aimed at a prize (a bold one aims at
/// the furthest, a cautious one at the nearest) and comes down as its steadiness and its reach
/// have it: rung, off a peg, or short. A show-off's last is thrown behind its back.
fn plan(id: Id, character: &Character, give_to: Option<Id>, dice: &mut Dice) -> Turn {
    let near = little(character);
    let manner = manner(character);
    let from = if near { NEAR_LINE.0 } else { LINE.0 };
    let steady = steadiness(character);
    let reach = reach(character, manner == Manner::Sitting);
    let throws = (0..RINGS)
        .map(|ring| {
            let behind_back = character.kind == TemperamentKind::Showoff && ring == RINGS - 1;
            let bold = character.axes.boldness;
            let leaning = if bold > 0.66 {
                2
            } else if bold < 0.33 {
                0
            } else {
                1
            };
            let aim = if dice.chance(0.3) {
                (dice.unit() * 3.0) as usize % 3
            } else {
                leaning
            };
            let distance = BLOCKS[aim] as f32 - from;
            // Out of its reach, it falls short more often; rushed, more often still.
            let mut short = ((distance - reach) / 30.0).clamp(0.0, 0.55);
            let mut rung = steady * 0.72 * (1.0 - short);
            match manner {
                Manner::Measures => rung *= 1.12,
                Manner::Hasty => {
                    rung *= 0.8;
                    short += 0.08;
                }
                Manner::Sitting | Manner::Plain => {}
            }
            if behind_back {
                // Wild: mostly anywhere but where it was meant, and now and then spectacular.
                rung = 0.14;
                short = 0.3;
            }
            let roll = dice.unit();
            let landing = if roll < rung {
                Landing::Rung {
                    prize: Prize::ALL[aim],
                }
            } else if roll < rung + short {
                Landing::Short
            } else {
                Landing::Peg
            };
            // A peg either side of the block it was aimed at; and wherever a miss comes down.
            let peg = aim + usize::from(dice.chance(0.5));
            let down = match landing {
                Landing::Short => (
                    (from + 12.0 + (distance - 12.0) * dice.range(0.3, 0.62)).round() as i32,
                    LINE.1 as i32 + dice.range(-3.0, 4.0).round() as i32,
                ),
                Landing::Peg if dice.chance(0.5) => (
                    PEGS[peg] + if dice.chance(0.5) { 4 } else { -4 },
                    COUNTER_TOP,
                ),
                Landing::Peg => (
                    PEGS[peg] + if dice.chance(0.5) { 3 } else { -3 },
                    COUNTER.1 + 18 + dice.range(0.0, 6.0).round() as i32,
                ),
                Landing::Rung { .. } => (BLOCKS[aim], COUNTER_TOP),
            };
            let ready = match manner {
                Manner::Measures => MEASURE_SECS + WIND_SECS,
                Manner::Hasty => HASTY_WIND_SECS,
                Manner::Sitting | Manner::Plain => WIND_SECS + dice.range(0.0, 0.3),
            } + if behind_back { TURN_ROUND_SECS } else { 0.0 };
            let span = (match landing {
                Landing::Peg => PEGS[peg] as f32,
                _ => down.0 as f32,
            } - from)
                .abs();
            Throw {
                aim,
                landing,
                behind_back,
                peg,
                down,
                ready,
                flight: 0.45 + 0.006 * span + if behind_back { 0.2 } else { 0.0 },
            }
        })
        .collect();
    Turn {
        player: id,
        near,
        manner,
        throws,
        give_to: give_to.filter(|_| giving(character)),
    }
}

pub struct Hoopla {
    phase: Phase,
    turns: Vec<Turn>,
    current: usize,
    stage: Stage,
    watchers: Vec<Id>,
    events: Vec<Event>,
    round: u64,
    reduce_motion: bool,
    /// Where each ring thrown this turn came to rest, by throw.
    rings: Vec<(usize, Rest)>,
    /// When each block's prize was last rung, for its shine, and whether it has been carried off
    /// this turn and is waiting to be put back.
    shine: [Option<f32>; 3],
    taken: [bool; 3],
    /// The last game's players, the most rings first, with how many each rang.
    tally: Vec<(Id, u32)>,
    /// The prizes won, each carried by whoever has it, for the rest of the visit.
    carried: Vec<(Id, Prize)>,
    /// Where each one waiting or watching was last sent to stand.
    placed: Vec<(Id, (f32, f32))>,
}

impl Default for Hoopla {
    fn default() -> Self {
        Self::new()
    }
}

impl Hoopla {
    pub fn new() -> Self {
        Self {
            phase: Phase::Ready,
            turns: Vec::new(),
            current: 0,
            stage: Stage::Done { until: 0.0 },
            watchers: Vec::new(),
            events: Vec::new(),
            round: 0,
            reduce_motion: false,
            rings: Vec::new(),
            shine: [None; 3],
            taken: [false; 3],
            tally: Vec::new(),
            carried: Vec::new(),
            placed: Vec::new(),
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Whoever is throwing now.
    pub fn player(&self) -> Option<Id> {
        match self.phase {
            Phase::Playing { .. } => self.turns.get(self.current).map(|turn| turn.player),
            _ => None,
        }
    }

    /// How many have had their go, of how many are playing.
    pub fn progress(&self) -> (usize, usize) {
        let done = match self.phase {
            Phase::Playing { .. } => self.current,
            _ => self.turns.len(),
        };
        (done, self.turns.len())
    }

    /// How many rings each player has rung so far this game, or last game once it is over: the
    /// most first, ties in the order they went. Only those who have finished their turn count.
    pub fn tally(&self) -> Vec<(Id, u32)> {
        if self.phase == Phase::Ready {
            return self.tally.clone();
        }
        let done = match self.phase {
            Phase::Playing { .. } => self.current,
            _ => self.turns.len(),
        };
        let mut tally: Vec<(Id, u32)> = self
            .turns
            .iter()
            .take(done)
            .map(|turn| (turn.player, turn.rung()))
            .collect();
        tally.sort_by_key(|(_, rings)| std::cmp::Reverse(*rings));
        tally
    }

    /// Who carries which prize won here, for the rest of the visit.
    pub fn carried(&self) -> &[(Id, Prize)] {
        &self.carried
    }

    /// Starts a game for `players`, or everyone, keenest first. The rest watch.
    pub fn start(&mut self, ground: &mut Playground, cast: &Cast, players: &[Id], now: f32) {
        let everyone = ground.ids();
        let mut chosen: Vec<Id> = players
            .iter()
            .copied()
            .filter(|id| everyone.contains(id))
            .collect();
        if chosen.is_empty() {
            chosen.clone_from(&everyone);
        }
        if chosen.is_empty() {
            return;
        }
        self.round += 1;
        let mut dice = Dice::new(
            self.round.wrapping_mul(0x400b_1a00) ^ chosen.iter().fold(5, |seed, id| seed ^ id),
        );
        self.reduce_motion = ground.reduce_motion();
        chosen.sort_by(|a, b| {
            let keen = |id: &Id| ground.character(*id).map_or(0.0, keenness);
            keen(b).total_cmp(&keen(a)).then(a.cmp(b))
        });
        self.turns = chosen
            .iter()
            .filter_map(|id| {
                let character = ground.character(*id)?;
                let friend = cast.closest_friend(*id, everyone.iter().copied());
                Some(plan(*id, character, friend, &mut dice))
            })
            .collect();
        self.watchers = everyone
            .iter()
            .copied()
            .filter(|id| !chosen.contains(id))
            .collect();
        self.events.clear();
        self.rings.clear();
        self.shine = [None; 3];
        self.taken = [false; 3];
        self.current = 0;
        self.placed.clear();
        ground.reserve(everyone);
        self.phase = Phase::Playing { since: now };
        self.place_everyone(ground, now);
        self.begin_turn(ground, now);
    }

    /// Those waiting their turn, first in line first, and those watching: everyone not playing,
    /// then everyone who has had a go, in the order they went.
    fn lines(&self) -> (Vec<Id>, Vec<Id>) {
        let waiting = self
            .turns
            .iter()
            .skip(self.current + 1)
            .map(|turn| turn.player)
            .collect();
        let watching = self
            .watchers
            .iter()
            .copied()
            .chain(self.turns.iter().take(self.current).map(|turn| turn.player))
            .collect();
        (waiting, watching)
    }

    /// Everyone waiting to its place in the queue, facing the stall, and everyone else to its
    /// place in front of the stall, facing whoever is throwing: each a little apart from the
    /// next, however big. Only those whose place has changed move.
    fn place_everyone(&mut self, ground: &mut Playground, now: f32) {
        let busy = self.player();
        let (waiting, watching) = self.lines();
        let queue = line_up(&QUEUE, &sizes(ground, &waiting), LINE_GAP);
        let watch = line_up(&WATCH, &sizes(ground, &watching), LINE_GAP);
        let places = waiting
            .iter()
            .zip(queue)
            .map(|(id, at)| (*id, at, true))
            .chain(watching.iter().zip(watch).map(|(id, at)| (*id, at, false)));
        for (id, at, queueing) in places.collect::<Vec<_>>() {
            if busy == Some(id) {
                continue;
            }
            let moved = self
                .placed
                .iter()
                .find(|(placed, _)| *placed == id)
                .is_none_or(|(_, was)| *was != at);
            if !moved {
                continue;
            }
            self.placed.retain(|(placed, _)| *placed != id);
            self.placed.push((id, at));
            let facing = if queueing { COUNTER.0 as f32 } else { LINE.0 };
            let wait = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
            ground.direct(
                id,
                vec![
                    Step::Stride { to: at },
                    Step::FaceX(facing),
                    Step::Beat(wait),
                ],
                now,
            );
        }
    }

    /// The next player goes for its rings: the counter is set out again, and the rings stacked.
    fn begin_turn(&mut self, ground: &mut Playground, now: f32) {
        let Some(turn) = self.turns.get(self.current) else {
            return;
        };
        self.rings.clear();
        self.taken = [false; 3];
        ground.direct(
            turn.player,
            vec![Step::Stride { to: COLLECT }, Step::FaceX(COLLECT.0 + 100.0)],
            now,
        );
        self.events.push(Event::Up {
            player: turn.player,
            near: turn.near,
        });
        self.stage = Stage::Collecting { since: now };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        match self.phase {
            Phase::Ready => {}
            Phase::Playing { .. } => self.play(ground, now),
            Phase::Over { since } => {
                if now - since >= CELEBRATE_SECS {
                    self.tally = self.tally();
                    for id in ground.ids() {
                        ground.direct(id, Vec::new(), now);
                    }
                    ground.release();
                    self.turns.clear();
                    self.watchers.clear();
                    self.rings.clear();
                    self.phase = Phase::Ready;
                }
            }
        }
    }

    /// Whether `id` has got to `spot` and stopped, or has been given long enough to.
    fn arrived(ground: &Playground, id: Id, spot: (f32, f32), since: f32, now: f32) -> bool {
        let there = ground
            .position(id)
            .is_some_and(|at| (at.0 - spot.0).abs() < 1.0 && (at.1 - spot.1).abs() < 1.0)
            && !ground.walking(id);
        there || now - since >= WALKING_SECS
    }

    fn play(&mut self, ground: &mut Playground, now: f32) {
        let Some(turn) = self.turns.get(self.current).cloned() else {
            return;
        };
        let line = if turn.near { NEAR_LINE } else { LINE };
        match self.stage {
            Stage::Collecting { since } => {
                if !Self::arrived(ground, turn.player, COLLECT, since, now) {
                    return;
                }
                ground.teleport(turn.player, COLLECT);
                let reach = Beat::new(Gesture::Reach, ExpressionKind::Curious, PICK_SECS);
                ground.direct(turn.player, vec![Step::Beat(reach)], now);
                self.stage = Stage::Picking {
                    until: now + PICK_SECS,
                };
            }
            Stage::Picking { until } => {
                if now < until {
                    return;
                }
                ground.direct(
                    turn.player,
                    vec![Step::Stride { to: line }, Step::FaceX(line.0 + 100.0)],
                    now,
                );
                self.stage = Stage::ToTheLine { since: now };
            }
            Stage::ToTheLine { since } => {
                if !Self::arrived(ground, turn.player, line, since, now) {
                    return;
                }
                ground.teleport(turn.player, line);
                self.ready(ground, &turn, 0, now);
            }
            Stage::Readying { throw, until } => {
                if now < until {
                    return;
                }
                let t = turn.throws[throw];
                // The ring leaves its paws: in front of it, or up over its shoulder, behind its
                // back.
                let from = self.release(ground, turn.player, t.behind_back, now);
                let face = if turn.manner == Manner::Sitting {
                    ExpressionKind::Content
                } else {
                    ExpressionKind::Determined
                };
                let throw_beat = if t.behind_back {
                    Beat::new(Gesture::Strut, ExpressionKind::Smug, THROW_SECS)
                } else {
                    Beat::new(Gesture::Reach, face, THROW_SECS)
                };
                let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
                let mut steps = vec![Step::Beat(throw_beat), Step::Beat(watch)];
                if t.behind_back {
                    // Turned round once it has thrown, to see where it went.
                    steps.insert(1, Step::FaceX(line.0 + 100.0));
                }
                ground.direct(turn.player, steps, now);
                self.stage = Stage::Flying {
                    throw,
                    since: now,
                    from,
                };
            }
            Stage::Flying { throw, since, .. } => {
                let t = turn.throws[throw];
                let lasts = t.flight
                    + if t.landing == Landing::Peg {
                        BOUNCE_SECS
                    } else {
                        0.0
                    };
                if now - since < lasts {
                    return;
                }
                self.land(ground, &turn, throw, now);
            }
            Stage::Landed { throw, until } => {
                if now < until {
                    return;
                }
                if throw + 1 < turn.throws.len() {
                    self.ready(ground, &turn, throw + 1, now);
                } else {
                    self.turn_over(ground, &turn, now);
                }
            }
            Stage::Giving { since } => {
                let Some(friend) = turn.give_to else {
                    self.stage = Stage::Done { until: now };
                    return;
                };
                let beside = ground.beside_of(turn.player, friend);
                let there = ground
                    .position(turn.player)
                    .is_some_and(|at| crate::playground::distance(at, beside) < 3.0)
                    && !ground.walking(turn.player);
                if !there && now - since < WALKING_SECS {
                    if !ground.walking(turn.player) {
                        ground.direct(turn.player, vec![Step::Stride { to: beside }], now);
                    }
                    return;
                }
                let mut give =
                    Beat::new(Gesture::Reach, ExpressionKind::Affectionate, HANDING_SECS);
                give.cue = Some(Cue::Heart);
                ground.direct(turn.player, vec![Step::Face(friend), Step::Beat(give)], now);
                let delighted = ground.character(friend).map_or(
                    Beat::new(Gesture::Cheer, ExpressionKind::Joy, HANDING_SECS),
                    |c| c.celebrate(HANDING_SECS),
                );
                let watch = Beat::new(Gesture::Watch, ExpressionKind::Joy, 600.0);
                ground.direct(
                    friend,
                    vec![
                        Step::Face(turn.player),
                        Step::Beat(delighted),
                        Step::Beat(watch),
                    ],
                    now,
                );
                if let Some(prize) = turn.won() {
                    self.carry(friend, prize);
                    self.events.push(Event::Gave {
                        from: turn.player,
                        to: friend,
                        prize,
                    });
                }
                self.stage = Stage::Handing {
                    until: now + HANDING_SECS,
                };
            }
            Stage::Handing { until } | Stage::Done { until } => {
                if now >= until {
                    self.next_turn(ground, &turn, now);
                }
            }
        }
    }

    /// Readies throw `throw`: measuring it up, turning its back, or sitting down to it, and
    /// winding up.
    fn ready(&mut self, ground: &mut Playground, turn: &Turn, throw: usize, now: f32) {
        let t = turn.throws[throw];
        let mut steps = Vec::new();
        if t.behind_back {
            // Its back to the stall, with a look over its shoulder at the crowd.
            steps.push(Step::FaceX(LINE.0 - 100.0));
            let mut flourish = Beat::new(Gesture::Strut, ExpressionKind::Smug, TURN_ROUND_SECS);
            flourish.cue = Some(Cue::Sparkle);
            steps.push(Step::Beat(flourish));
            self.events.push(Event::BehindTheBack {
                player: turn.player,
            });
        }
        let wind = t.ready - if t.behind_back { TURN_ROUND_SECS } else { 0.0 };
        match turn.manner {
            Manner::Measures if !t.behind_back => {
                // Along its paw at the prize, then steady.
                steps.push(Step::Beat(Beat::new(
                    Gesture::Reach,
                    ExpressionKind::Focused,
                    MEASURE_SECS * 0.5,
                )));
                steps.push(Step::Beat(Beat::new(
                    Gesture::Watch,
                    ExpressionKind::Focused,
                    MEASURE_SECS * 0.5,
                )));
                steps.push(Step::Beat(Beat::new(
                    Gesture::Crouch,
                    ExpressionKind::Determined,
                    wind - MEASURE_SECS,
                )));
                if throw == 0 {
                    self.events.push(Event::Measures {
                        player: turn.player,
                    });
                }
            }
            Manner::Sitting => {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Sit,
                    ExpressionKind::Sleepy,
                    wind,
                )));
            }
            _ => {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Crouch,
                    ExpressionKind::Determined,
                    wind,
                )));
            }
        }
        ground.direct(turn.player, steps, now);
        self.stage = Stage::Readying {
            throw,
            until: now + t.ready,
        };
    }

    /// Where a ring leaves a thrower's paws: just in front of it, or up over its shoulder if it
    /// throws behind its back.
    fn release(&self, ground: &mut Playground, id: Id, behind_back: bool, now: f32) -> (f32, f32) {
        let feet = ground.position(id).unwrap_or(LINE);
        let crown = ground.head(id, now).unwrap_or((feet.0, feet.1 - 24.0));
        if behind_back {
            (crown.0 + 4.0, crown.1 - 2.0)
        } else {
            (crown.0 + 6.0, crown.1 + (feet.1 - crown.1) * 0.55)
        }
    }

    /// A ring has come down: where it rests, how the thrower takes it, and the watchers.
    fn land(&mut self, ground: &mut Playground, turn: &Turn, throw: usize, now: f32) {
        let t = turn.throws[throw];
        let rest = match t.landing {
            Landing::Rung { .. } => Rest::Round(t.aim),
            Landing::Short => Rest::Ground(t.down),
            Landing::Peg if t.down.1 == COUNTER_TOP => Rest::Counter(t.down),
            Landing::Peg => Rest::Ground(t.down),
        };
        self.rings.push((throw, rest));
        if let Landing::Rung { .. } = t.landing {
            self.shine[t.aim] = Some(now);
        }
        self.events.push(Event::Landed {
            player: turn.player,
            ring: throw,
            landing: t.landing,
            behind_back: t.behind_back,
        });
        let beats = self.reaction(ground, turn, t);
        ground.direct(
            turn.player,
            beats.into_iter().map(Step::Beat).collect(),
            now,
        );
        // A ring rung is cheered, and one behind the back gasped at first.
        if let Landing::Rung { .. } = t.landing {
            let (waiting, watching) = self.lines();
            for id in waiting.into_iter().chain(watching) {
                let mut steps = Vec::new();
                if t.behind_back {
                    let mut gasp = Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.5);
                    gasp.cue = Some(Cue::Exclaim);
                    steps.push(Step::Beat(gasp));
                }
                steps.push(Step::Beat(Beat::new(
                    Gesture::Cheer,
                    ExpressionKind::Joy,
                    0.8,
                )));
                steps.push(Step::Beat(Beat::new(
                    Gesture::Watch,
                    ExpressionKind::Curious,
                    600.0,
                )));
                ground.direct(id, steps, now);
            }
        }
        let pause = if turn.manner == Manner::Hasty {
            HASTY_AFTER_SECS
        } else {
            AFTER_SECS
        };
        self.stage = Stage::Landed {
            throw,
            until: now + pause,
        };
    }

    /// How a thrower takes where its ring came down, in its own way: a prize rung is celebrated,
    /// a ring off a peg gasped at, and one short taken as its temperament takes it.
    fn reaction(&self, ground: &Playground, turn: &Turn, throw: Throw) -> Vec<Beat> {
        let Some(c) = ground.character(turn.player) else {
            return Vec::new();
        };
        let cue = |mut beat: Beat, cue: Cue| {
            beat.cue = Some(cue);
            beat
        };
        let mut beats = match throw.landing {
            Landing::Rung { .. } if throw.behind_back => vec![cue(
                Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.4),
                Cue::Sparkle,
            )],
            Landing::Rung { .. } => match c.kind {
                TemperamentKind::Showoff => vec![cue(
                    Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.0),
                    Cue::Sparkle,
                )],
                TemperamentKind::Grump => vec![cue(
                    Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.0),
                    Cue::Sparkle,
                )],
                TemperamentKind::Wallflower => vec![cue(
                    Beat::new(ActionKind::Idle, ExpressionKind::Joy, 1.0),
                    Cue::Heart,
                )],
                TemperamentKind::Lazybones => vec![
                    c.celebrate(0.6),
                    Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 0.5),
                ],
                _ => vec![c.celebrate(1.0)],
            },
            Landing::Peg => {
                let gasp = cue(
                    Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.4),
                    Cue::Exclaim,
                );
                let after = match c.kind {
                    TemperamentKind::Grump => cue(
                        Beat::new(Gesture::Stomp, ExpressionKind::Grumpy, 0.6),
                        Cue::Huff,
                    ),
                    _ if c.axes.playfulness > 0.6 => {
                        cue(Beat::new(Gesture::Bop, ExpressionKind::Joy, 0.6), Cue::Note)
                    }
                    _ => Beat::new(ActionKind::Idle, ExpressionKind::Worried, 0.5),
                };
                vec![gasp, after]
            }
            Landing::Short => match c.kind {
                TemperamentKind::Grump => vec![cue(
                    Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 0.8),
                    Cue::Huff,
                )],
                TemperamentKind::Lazybones => {
                    vec![Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 0.8)]
                }
                _ if c.axes.playfulness > 0.6 => vec![cue(
                    Beat::new(Gesture::Bop, ExpressionKind::Joy, 0.8),
                    Cue::Note,
                )],
                _ => vec![Beat::new(ActionKind::Idle, ExpressionKind::Content, 0.6)],
            },
        };
        let rest = if turn.manner == Manner::Sitting {
            Beat::new(Gesture::Sit, ExpressionKind::Sleepy, 600.0)
        } else {
            Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0)
        };
        beats.push(rest);
        beats
    }

    /// Its three rings thrown: what it rang it takes from the counter, to keep or to give away.
    fn turn_over(&mut self, ground: &mut Playground, turn: &Turn, now: f32) {
        let rung = turn.rung();
        self.events.push(Event::TurnOver {
            player: turn.player,
            rings: rung,
        });
        for throw in &turn.throws {
            if let Landing::Rung { .. } = throw.landing {
                self.taken[throw.aim] = true;
            }
        }
        match (turn.won(), turn.give_to) {
            (Some(_), Some(_)) => {
                self.stage = Stage::Giving { since: now };
            }
            (Some(prize), None) => {
                self.carry(turn.player, prize);
                let keep = ground
                    .character(turn.player)
                    .map_or(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.0), |c| {
                        c.celebrate(1.0)
                    });
                ground.direct(turn.player, vec![Step::Beat(keep)], now);
                self.stage = Stage::Done { until: now + 1.0 };
            }
            (None, _) => {
                self.stage = Stage::Done { until: now + 0.4 };
            }
        }
    }

    /// `id` carries `prize` from now on, in place of anything it carried before.
    fn carry(&mut self, id: Id, prize: Prize) {
        self.carried.retain(|(carrier, _)| *carrier != id);
        self.carried.push((id, prize));
    }

    /// The player goes to watch; the queue moves up; and the next one goes for its rings, or, if
    /// that was everyone, the game is over.
    fn next_turn(&mut self, ground: &mut Playground, turn: &Turn, now: f32) {
        self.placed.retain(|(id, _)| *id != turn.player);
        if let Some(friend) = turn.give_to {
            // Back to its place, after being given something.
            self.placed.retain(|(id, _)| *id != friend);
        }
        self.current += 1;
        self.rings.clear();
        self.place_everyone(ground, now);
        if self.current >= self.turns.len() {
            self.over(ground, now);
            return;
        }
        self.begin_turn(ground, now);
    }

    /// Everyone has had a go: the colony cheers whoever rang the most, and they celebrate.
    fn over(&mut self, ground: &mut Playground, now: f32) {
        self.stage = Stage::Done { until: now };
        self.current = self.turns.len();
        let tally = self.tally();
        let most = tally.first().map_or(0, |(_, rings)| *rings);
        for id in ground.ids() {
            let Some(c) = ground.character(id) else {
                continue;
            };
            let best = most > 0
                && tally
                    .iter()
                    .any(|(who, rings)| *who == id && *rings == most);
            let beat = if best {
                c.celebrate(2.0)
            } else {
                Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.2)
            };
            let watch = Beat::new(ActionKind::Idle, ExpressionKind::Joy, 600.0);
            ground.direct(id, vec![Step::Beat(beat), Step::Beat(watch)], now);
        }
        self.tally = tally;
        self.events.push(Event::AllDone);
        self.phase = Phase::Over { since: now };
    }

    /// Calls the game off: everyone back to playing. Whatever was won is still carried.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        for id in self.watchers.clone() {
            ground.direct(id, Vec::new(), now);
        }
        for turn in &self.turns {
            if let Some(c) = ground.character(turn.player) {
                ground.direct(turn.player, vec![Step::Beat(c.celebrate(1.0))], now);
            }
        }
        ground.release();
        self.turns.clear();
        self.watchers.clear();
        self.events.clear();
        self.rings.clear();
        self.taken = [false; 3];
        self.phase = Phase::Ready;
    }

    /// Where the ring in the air is, and how far through its spin, if one is in the air. With
    /// motion reduced it is not seen in the air at all: it is simply where it comes down, and its
    /// arc is drawn whole while it would have flown (see `trail`).
    fn flying(&self, now: f32) -> Option<((f32, f32), f32)> {
        let (Phase::Playing { .. }, Stage::Flying { throw, since, from }) =
            (self.phase, self.stage)
        else {
            return None;
        };
        let t = self.turns.get(self.current)?.throws.get(throw).copied()?;
        if self.reduce_motion {
            return None;
        }
        let elapsed = now - since;
        let (to, apex) = Self::target(t);
        if elapsed < t.flight {
            let along = elapsed / t.flight;
            return Some((arc(from, to, apex, along), along * 3.0));
        }
        // Off the peg and down.
        let along = ((elapsed - t.flight) / BOUNCE_SECS).min(1.0);
        let down = (t.down.0 as f32, t.down.1 as f32);
        Some((arc(to, down, 5.0, along), 3.0 + along * 2.0))
    }

    /// Where a throw's first arc comes down, and how high it goes: over a block, onto a peg's
    /// knob, or into the sawdust; and higher, thrown behind the back.
    fn target(t: Throw) -> ((f32, f32), f32) {
        let to = match t.landing {
            Landing::Rung { .. } => (BLOCKS[t.aim] as f32, COUNTER_TOP as f32 - 2.0),
            Landing::Peg => (PEGS[t.peg] as f32, (COUNTER_TOP - PEG_TALL) as f32),
            Landing::Short => (t.down.0 as f32, t.down.1 as f32),
        };
        // Low, so it flies against the bare boards of the stall rather than its shelf of prizes;
        // a lob behind the back goes higher.
        let apex = if t.behind_back { 18.0 } else { 6.0 };
        (to, apex)
    }

    /// The arc the ring in the air has flown so far, as dots: all of it, cut to where it comes
    /// down, with motion reduced.
    fn trail(&self, now: f32) -> Vec<(i32, i32)> {
        let (Phase::Playing { .. }, Stage::Flying { throw, since, from }) =
            (self.phase, self.stage)
        else {
            return Vec::new();
        };
        let Some(t) = self
            .turns
            .get(self.current)
            .and_then(|turn| turn.throws.get(throw).copied())
        else {
            return Vec::new();
        };
        let (to, apex) = Self::target(t);
        let flown = if self.reduce_motion {
            1.0
        } else {
            ((now - since) / t.flight).min(1.0)
        };
        let mut dots = Vec::new();
        let steps = 36;
        for step in 0..steps {
            let along = step as f32 / steps as f32;
            if along > flown {
                break;
            }
            let (x, y) = arc(from, to, apex, along);
            dots.push((x.round() as i32, y.round() as i32));
        }
        if self.reduce_motion && t.landing == Landing::Peg {
            let down = (t.down.0 as f32, t.down.1 as f32);
            for step in 0..8 {
                let (x, y) = arc(to, down, 5.0, step as f32 / 8.0);
                dots.push((x.round() as i32, y.round() as i32));
            }
        }
        dots
    }

    /// The rings each player still holds: none until it has taken them off the counter, then
    /// fewer with every throw.
    fn held(&self) -> usize {
        match self.stage {
            Stage::ToTheLine { .. } => RINGS,
            Stage::Readying { throw, .. } => RINGS - throw,
            Stage::Flying { throw, .. } | Stage::Landed { throw, .. } => RINGS - throw - 1,
            _ => 0,
        }
    }

    /// How many rings are stacked on the counter, waiting.
    fn stacked(&self) -> usize {
        match (self.phase, self.stage) {
            (Phase::Playing { .. }, Stage::Collecting { .. }) => RINGS,
            (Phase::Playing { .. }, _) => 0,
            _ => RINGS,
        }
    }

    /// What is set out along the counter: the pegs, the prizes on their blocks and the rings round
    /// them or lying where they came down, and the rings stacked at its end. Lit by the stall's
    /// bulbs after dark, as the stall is.
    pub fn counter(&self, now: f32) -> Prop {
        let still = self.reduce_motion;
        let on_show: [bool; 3] = std::array::from_fn(|block| !self.taken[block]);
        let shine: [Option<f32>; 3] = std::array::from_fn(|block| {
            self.shine[block]
                .map(|at| now - at)
                .filter(|since| *since < SHINE_SECS)
        });
        let hoop = |throw: usize| Hoop::ALL[throw % Hoop::ALL.len()];
        let round: Vec<(usize, Hoop)> = self
            .rings
            .iter()
            .filter_map(|(throw, rest)| match rest {
                Rest::Round(block) => Some((*block, hoop(*throw))),
                _ => None,
            })
            .collect();
        let lying: Vec<((i32, i32), Hoop)> = self
            .rings
            .iter()
            .filter_map(|(throw, rest)| match rest {
                Rest::Counter(at) => Some((*at, hoop(*throw))),
                _ => None,
            })
            .collect();
        gear::hoopla_counter(on_show, shine, &round, &lying, self.stacked(), now, still)
    }

    /// The rest of the game's gear, while it is on: the lines chalked for throwing from, the rings
    /// still in the thrower's paws, the one in the air and its arc, and any in the sawdust.
    pub fn gear(&self, ground: &mut Playground, now: f32) -> Vec<Prop> {
        let hoop = |throw: usize| Hoop::ALL[throw % Hoop::ALL.len()];
        let mut gear = Vec::new();
        for (throw, rest) in &self.rings {
            if let Rest::Ground(at) = rest {
                gear.push(gear::hoop_lying(*at, hoop(*throw)));
            }
        }
        if matches!(self.phase, Phase::Playing { .. }) {
            gear.push(gear::chalk_line(
                (LINE.0 as i32 + 8, LINE.1 as i32 - 6),
                (LINE.0 as i32 + 5, LINE.1 as i32 + 5),
            ));
            gear.push(gear::chalk_line(
                (NEAR_LINE.0 as i32 + 8, NEAR_LINE.1 as i32 - 6),
                (NEAR_LINE.0 as i32 + 5, NEAR_LINE.1 as i32 + 5),
            ));
            if let Some(turn) = self.turns.get(self.current) {
                let held = self.held();
                if held > 0
                    && let (Some(feet), Some(crown)) =
                        (ground.position(turn.player), ground.head(turn.player, now))
                {
                    let throw = RINGS - held;
                    let grip = (
                        (crown.0 + 6.0).round() as i32,
                        (crown.1 + (feet.1 - crown.1) * 0.6).round() as i32,
                    );
                    gear.push(gear::hoops_held(grip, &Hoop::ALL[throw..], feet.1));
                }
            }
        }
        let trail = self.trail(now);
        let flying = self.flying(now);
        if !trail.is_empty() || flying.is_some() {
            let throw = match self.stage {
                Stage::Flying { throw, .. } => throw,
                _ => 0,
            };
            gear.push(gear::hoop_flying(
                flying.map(|((x, y), spin)| ((x.round() as i32, y.round() as i32), spin)),
                hoop(throw),
                &trail,
            ));
        }
        gear
    }
}

/// A point `along` an arc from `from` to `to` that rises `apex` above the higher of the two.
fn arc(from: (f32, f32), to: (f32, f32), apex: f32, along: f32) -> (f32, f32) {
    let along = along.clamp(0.0, 1.0);
    let x = from.0 + (to.0 - from.0) * along;
    let straight = from.1 + (to.1 - from.1) * along;
    let lift = (apex + (from.1 - to.1).abs() * 0.5) * 4.0 * along * (1.0 - along);
    (x, straight - lift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fairground;
    use formiga_travel::Traveler;

    const TICK: f32 = 1.0 / 30.0;

    fn colony(tune: impl Fn(usize, &mut Traveler)) -> Cast {
        let mut snapshot = formiga_travel::sample::snapshot();
        for (index, traveler) in snapshot.travelers.iter_mut().enumerate() {
            tune(index, traveler);
        }
        Cast::new(snapshot).unwrap()
    }

    fn sample() -> Cast {
        colony(|_, _| {})
    }

    /// The fairground with the colony settled in, and the stall.
    fn ready(cast: &Cast) -> (Playground, Hoopla) {
        let (mut ground, _) = fairground::open(cast, 0.0);
        let mut now = 0.0;
        while now < 6.0 {
            now += TICK;
            ground.tick(cast, now);
        }
        (ground, Hoopla::new())
    }

    /// A game from `from` until everyone is back to playing, or `to`: what happened, and when
    /// it was over.
    fn run(
        ground: &mut Playground,
        hoopla: &mut Hoopla,
        cast: &Cast,
        from: f32,
        to: f32,
        mut each: impl FnMut(&mut Hoopla, &mut Playground, f32),
    ) -> (Vec<Event>, Option<f32>) {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += TICK;
            ground.tick(cast, now);
            hoopla.tick(ground, now);
            each(hoopla, ground, now);
            events.extend(hoopla.take_events());
            if hoopla.phase() == Phase::Ready {
                return (events, Some(now));
            }
        }
        (events, None)
    }

    fn game(cast: &Cast, players: &[Id]) -> (Vec<Event>, Vec<(Id, u32)>, Hoopla) {
        let (mut ground, mut hoopla) = ready(cast);
        hoopla.start(&mut ground, cast, players, 6.0);
        let (events, over) = run(&mut ground, &mut hoopla, cast, 6.0, 400.0, |_, _, _| {});
        assert!(over.is_some(), "the game never ended: {events:?}");
        let tally = hoopla.tally();
        (events, tally, hoopla)
    }

    fn landings(events: &[Event], who: Id) -> Vec<Landing> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Landed {
                    player, landing, ..
                } if *player == who => Some(*landing),
                _ => None,
            })
            .collect()
    }

    fn character(tune: impl FnOnce(&mut Character)) -> Character {
        let mut character = Character::of(&sample().members[1]);
        character.parent = None;
        character.size = 1.0;
        character.kind = TemperamentKind::Guardian;
        tune(&mut character);
        character
    }

    #[test]
    fn everyone_throws_three_rings_and_each_rings_a_prize_bounces_off_a_peg_or_falls_short() {
        let cast = sample();
        let (events, tally, _) = game(&cast, &[]);
        assert_eq!(tally.len(), cast.members.len(), "everyone has a result");
        for member in &cast.members {
            let landed = landings(&events, member.id);
            assert_eq!(landed.len(), RINGS, "{} threw {landed:?}", member.name);
            let rung = landed
                .iter()
                .filter(|landing| matches!(landing, Landing::Rung { .. }))
                .count() as u32;
            let told = events.iter().find_map(|event| match event {
                Event::TurnOver { player, rings } if *player == member.id => Some(*rings),
                _ => None,
            });
            assert_eq!(told, Some(rung), "{}", member.name);
            assert!(tally.contains(&(member.id, rung)));
        }
        assert!(tally.windows(2).all(|pair| pair[0].1 >= pair[1].1));
        assert_eq!(events.last(), Some(&Event::AllDone));
    }

    #[test]
    fn whoever_is_picked_throws_and_the_rest_watch() {
        let cast = sample();
        let picked = [cast.members[0].id, cast.members[3].id];
        let (events, tally, _) = game(&cast, &picked);
        let mut played: Vec<Id> = tally.iter().map(|(id, _)| *id).collect();
        played.sort();
        let mut wanted = picked.to_vec();
        wanted.sort();
        assert_eq!(played, wanted);
        for member in &cast.members {
            let threw = !landings(&events, member.id).is_empty();
            assert_eq!(threw, picked.contains(&member.id), "{}", member.name);
        }
    }

    #[test]
    fn the_game_always_ends_however_the_colony_is() {
        let tunings: [fn(usize, &mut Traveler); 4] = [
            |_, t| t.character.temperament = TemperamentKind::Lazybones.into(),
            |_, t| t.character.temperament = TemperamentKind::Showoff.into(),
            |_, t| {
                t.character.temperament = TemperamentKind::Sweetheart.into();
                t.character.axes.affection = 1.0;
            },
            |_, t| t.character.axes.impulsiveness = 1.0,
        ];
        for tune in tunings {
            let cast = colony(tune);
            let (_, tally, _) = game(&cast, &[]);
            assert_eq!(tally.len(), cast.members.len());
        }
        let big = crate::fairground::testing::colony_of(12);
        let (_, tally, _) = game(&big, &[]);
        assert_eq!(tally.len(), 12);
    }

    #[test]
    fn a_patient_hand_rings_more_than_an_impulsive_one() {
        let rung = |impulsiveness: f32| {
            let c = character(|c| c.axes.impulsiveness = impulsiveness);
            let mut dice = Dice::new(9);
            (0..300)
                .map(|_| plan(1, &c, None, &mut dice).rung())
                .sum::<u32>()
        };
        let (patient, impulsive) = (rung(0.1), rung(0.95));
        assert!(
            patient > impulsive + impulsive / 2,
            "{patient} rung patiently, {impulsive} impulsively"
        );
        // Nobody is hopeless.
        assert!(impulsive > 20);
        let measured = character(|c| c.axes.impulsiveness = 0.1);
        let hasty = character(|c| c.axes.impulsiveness = 0.9);
        let mut dice = Dice::new(3);
        let (careful, quick) = (
            plan(1, &measured, None, &mut dice),
            plan(2, &hasty, None, &mut dice),
        );
        assert_eq!(
            (careful.manner, quick.manner),
            (Manner::Measures, Manner::Hasty)
        );
        let readying = |turn: &Turn| turn.throws.iter().map(|t| t.ready).sum::<f32>();
        assert!(
            readying(&careful) > readying(&quick) * 3.0,
            "the impulsive throw fast"
        );
    }

    #[test]
    fn a_little_one_stands_at_the_nearer_line_and_rings_as_often_as_anyone() {
        let cast = sample();
        let little = cast.members.iter().find(|m| m.parent().is_some()).unwrap();
        let mut c = Character::of(little);
        let mut dice = Dice::new(4);
        let turn = plan(little.id, &c, None, &mut dice);
        assert!(turn.near);
        let (mut ground, mut hoopla) = ready(&cast);
        hoopla.start(&mut ground, &cast, &[little.id], 6.0);
        let mut threw_from = None;
        run(
            &mut ground,
            &mut hoopla,
            &cast,
            6.0,
            120.0,
            |hoopla, ground, _| {
                if threw_from.is_none() && matches!(hoopla.stage, Stage::Readying { .. }) {
                    threw_from = ground.position(little.id);
                }
            },
        );
        assert_eq!(threw_from, Some(NEAR_LINE));
        // From the nearer line, small as it is, it rings near enough as often as one grown.
        let rung = |c: &Character| {
            let mut dice = Dice::new(11);
            (0..300)
                .map(|_| plan(1, c, None, &mut dice).rung())
                .sum::<u32>()
        };
        let small = rung(&c);
        c.parent = None;
        c.size = 1.0;
        let grown = rung(&c);
        assert!(small * 10 >= grown * 7, "{small} rung small, {grown} grown");
    }

    #[test]
    fn a_show_off_throws_its_last_behind_its_back_wildly_but_now_and_then_spectacularly() {
        let showy = character(|c| c.kind = TemperamentKind::Showoff);
        let mut dice = Dice::new(5);
        let turns: Vec<Turn> = (0..400).map(|_| plan(1, &showy, None, &mut dice)).collect();
        for turn in &turns {
            let behind: Vec<bool> = turn.throws.iter().map(|t| t.behind_back).collect();
            assert_eq!(behind, [false, false, true]);
        }
        let rung = turns
            .iter()
            .filter(|turn| matches!(turn.throws[2].landing, Landing::Rung { .. }))
            .count();
        assert!(rung > 20 && rung < 120, "{rung} of 400 behind the back");
        // Nobody else throws behind its back.
        let plain = character(|_| {});
        assert!(
            (0..50)
                .flat_map(|_| plan(1, &plain, None, &mut dice).throws)
                .all(|t| !t.behind_back)
        );
        // In a game it is told, and turns its back to do it.
        let cast = colony(|i, t| {
            if i == 1 {
                t.character.temperament = TemperamentKind::Showoff.into();
            }
        });
        let showoff = cast.members[1].id;
        let (events, _, _) = game(&cast, &[showoff]);
        assert!(events.contains(&Event::BehindTheBack { player: showoff }));
    }

    #[test]
    fn an_affectionate_one_gives_what_it_wins_to_its_closest_friend() {
        // Everyone a sure shot, and one of them as fond as can be.
        let cast = colony(|i, t| {
            t.character.axes.impulsiveness = 0.0;
            t.character.axes.affection = if i == 1 { 1.0 } else { 0.3 };
            t.character.temperament = TemperamentKind::Guardian.into();
        });
        let fond = cast.members[1].id;
        let friend = cast.closest_friend(fond, cast.ids()).unwrap();
        let mut rounds = 0;
        let (mut ground, mut hoopla) = ready(&cast);
        loop {
            rounds += 1;
            let from = 6.0 + rounds as f32 * 200.0;
            hoopla.start(&mut ground, &cast, &[fond], from);
            let (events, over) = run(
                &mut ground,
                &mut hoopla,
                &cast,
                from,
                from + 200.0,
                |_, _, _| {},
            );
            assert!(over.is_some());
            let won = events.iter().any(|event| {
                matches!(event, Event::Landed { player, landing: Landing::Rung { .. }, .. } if *player == fond)
            });
            if won {
                let gave = events.iter().find_map(|event| match event {
                    Event::Gave { from, to, prize } if *from == fond => Some((*to, *prize)),
                    _ => None,
                });
                let (to, prize) = gave.expect("it kept what it won");
                assert_eq!(to, friend);
                assert!(hoopla.carried().contains(&(friend, prize)));
                assert!(hoopla.carried().iter().all(|(who, _)| *who != fond));
                break;
            }
            assert!(rounds < 10, "it never rang anything");
        }
        // One less fond keeps what it wins.
        let mut dice = Dice::new(6);
        let cool = character(|c| c.axes.affection = 0.3);
        assert_eq!(plan(1, &cool, Some(2), &mut dice).give_to, None);
    }

    #[test]
    fn a_prize_won_is_carried_and_drawn_with_whoever_carries_it() {
        let cast = colony(|_, t| t.character.axes.impulsiveness = 0.0);
        let (mut ground, mut hoopla) = ready(&cast);
        let mut now = 6.0;
        let mut round = 0;
        while hoopla.carried().is_empty() {
            round += 1;
            assert!(round < 10, "nobody ever rang anything");
            hoopla.start(&mut ground, &cast, &[], now);
            let (_, over) = run(
                &mut ground,
                &mut hoopla,
                &cast,
                now,
                now + 400.0,
                |_, _, _| {},
            );
            now = over.unwrap();
        }
        let (carrier, _) = hoopla.carried()[0];
        let before = ground.bounds(carrier, now).unwrap();
        ground.carry(hoopla.carried());
        let after = ground.bounds(carrier, now).unwrap();
        assert!(
            after.1 < before.1,
            "the prize shows over its carrier: {after:?}"
        );
    }

    #[test]
    fn the_same_colony_with_other_temperaments_plays_differently() {
        let others = formiga_travel::sample::snapshot();
        let swapped = colony(|i, t| {
            let other = &others.travelers[(i + 2) % others.travelers.len()];
            t.character = other.character.clone();
            t.motion = other.motion;
        });
        let (first, _, _) = game(&sample(), &[]);
        let (second, _, _) = game(&swapped, &[]);
        assert_ne!(first, second);
    }

    #[test]
    fn with_reduced_motion_each_ring_cuts_to_where_it_lands_its_arc_drawn_whole() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut hoopla) = ready(&cast);
        hoopla.start(&mut ground, &cast, &[], 6.0);
        let mut arcs = 0;
        let (events, over) = run(
            &mut ground,
            &mut hoopla,
            &cast,
            6.0,
            400.0,
            |hoopla, _, now| {
                assert!(hoopla.flying(now).is_none(), "a ring was seen in the air");
                if matches!(hoopla.stage, Stage::Flying { .. }) {
                    let trail = hoopla.trail(now);
                    assert!(trail.len() >= 20, "the arc is drawn whole");
                    arcs += 1;
                }
            },
        );
        assert!(over.is_some(), "{events:?}");
        assert!(arcs > 0);
        // Without it, the ring flies, and its arc grows behind it.
        let cast = sample();
        let (mut ground, mut hoopla) = ready(&cast);
        hoopla.start(&mut ground, &cast, &[], 6.0);
        let mut seen = Vec::new();
        run(
            &mut ground,
            &mut hoopla,
            &cast,
            6.0,
            60.0,
            |hoopla, _, now| {
                if let Some((at, _)) = hoopla.flying(now) {
                    seen.push((at, hoopla.trail(now).len()));
                }
            },
        );
        assert!(seen.len() > 5);
        assert!(seen.windows(2).any(|pair| pair[0].0 != pair[1].0));
        assert!(seen.first().unwrap().1 < seen.last().unwrap().1);
    }

    #[test]
    fn calling_it_off_lets_everyone_go_and_keeps_what_was_won() {
        let cast = sample();
        let (mut ground, mut hoopla) = ready(&cast);
        hoopla.start(&mut ground, &cast, &[], 6.0);
        run(&mut ground, &mut hoopla, &cast, 6.0, 20.0, |_, _, _| {});
        assert!(hoopla.player().is_some());
        let carried = hoopla.carried().to_vec();
        hoopla.stop(&mut ground, 20.0);
        assert_eq!(hoopla.phase(), Phase::Ready);
        assert_eq!(hoopla.player(), None);
        assert_eq!(hoopla.carried(), carried.as_slice());
        let before: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        let mut now = 20.0;
        while now < 50.0 {
            now += TICK;
            ground.tick(&cast, now);
        }
        let after: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        assert_ne!(before, after, "nobody moved off");
    }
}
