//! The high striker, played by the colony while the person watches. Each player takes its turn:
//! it steps up, takes the mallet, swings it up and brings it down on the pad, and the puck shoots
//! up the striped tower towards the bell. Whoever is not playing, or has had its go, watches from
//! in front of the carousel.
//!
//! Nobody is written in. How high the puck goes is how hard the companion swings (its feistiness,
//! energy and boldness, and a little less if it is small) times how well it times the swing: a
//! patient one winds up steadily and brings it down at the top, an impulsive one often swings
//! before it is ready. A show-off does a flourish first; a scholar squints up at the marks; a
//! lazybones gives it a half-hearted tap before a proper go; and a little one's parent comes and
//! helps it hold the mallet, and together they can ring the bell. A ring makes the bell shake and
//! shine, and whoever rang it celebrates in its own way. Every swing counts for something: there
//! is no losing, only how high.

use super::gear::{self, Mallet, RING_SECS};
use super::scenery::PAD;
use super::{LINE_GAP, Path, line_up, sizes};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::{Playground, Prop};
use formiga_art::ExpressionKind;
use formiga_core::{ActionKind, Gesture, TemperamentKind};

/// Where a player stands to swing, just to the left of the pad, and where a parent helping its
/// little one stands, behind it.
pub const SPOT: (f32, f32) = (110.0, 125.0);
const HELPER_SPOT: (f32, f32) = (101.0, 123.0);
/// The way the queue goes, each one waiting a little apart from the next however big it is: from
/// just behind whoever is swinging, along the front of the big top, round in front of the hay and
/// out along the front of the sawdust, clear of the props.
const QUEUE: [Path; 1] = [&[
    (90.0, 129.0),
    (50.0, 132.0),
    (31.0, 142.0),
    (28.0, 158.0),
    (44.0, 170.0),
    (72.0, 175.0),
    (102.0, 179.0),
    (134.0, 188.0),
    (262.0, 192.0),
]];
/// Where everyone watches from once they have had their go, or if they are not playing: in front
/// of the carousel, in rows, each a little apart from the next and clear of the drum.
const WATCH: [Path; 3] = [
    &[(144.0, 134.0), (234.0, 134.0)],
    &[(156.0, 149.0), (232.0, 149.0)],
    &[(140.0, 163.0), (240.0, 163.0)],
];
/// Picking up the mallet.
const PICK_UP_SECS: f32 = 0.5;
const FLOURISH_SECS: f32 = 1.5;
const SQUINT_SECS: f32 = 1.7;
/// The mallet stays down on the pad this long after the strike.
const STRIKE_SECS: f32 = 0.35;
/// How long each one takes a swing in its own way, and a half-hearted tap.
const AFTER_SECS: f32 = 1.8;
const AFTER_TAP_SECS: f32 = 1.2;
const PUT_DOWN_SECS: f32 = 0.4;
/// How long everyone cheers at the end.
const CELEBRATE_SECS: f32 = 4.0;
/// The longest anyone is given to step up.
const STEPPING_SECS: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Ready,
    Playing {
        since: f32,
    },
    /// Everyone has had a go; the colony is cheering whoever went highest.
    Over {
        since: f32,
    },
}

/// Something that happened at the striker, for the person watching to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Up to the striker, with its parent if it is a little one.
    Up {
        player: Id,
        helper: Option<Id>,
    },
    Flourish {
        player: Id,
    },
    Squint {
        player: Id,
    },
    Tap {
        player: Id,
    },
    /// A swing, and how high the puck went, from 0 to 1 (the bell).
    Swung {
        player: Id,
        helper: Option<Id>,
        height: f32,
        early: bool,
    },
    Rang {
        player: Id,
        helper: Option<Id>,
    },
    /// Everyone has had a go: see `ranking`.
    AllDone,
}

/// Before the swing, a show-off shows off and a scholar works out how high the bell is.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Prelude {
    None,
    Flourish,
    Squint,
}

/// One swing: how long it is wound up for, how high it sends the puck, whether it came before
/// the companion was ready, and whether it was only a tap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    pub wind: f32,
    pub height: f32,
    pub early: bool,
    pub tap: bool,
}

#[derive(Clone, Debug)]
struct Turn {
    player: Id,
    helper: Option<Id>,
    prelude: Prelude,
    swings: Vec<Swing>,
    /// The highest it has sent the puck so far, once a swing has come down.
    best: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    Stepping { since: f32 },
    Prelude { until: f32 },
    WindUp { swing: usize, until: f32 },
    Flight { swing: usize, since: f32 },
    After { swing: usize, until: f32 },
    Done { until: f32 },
}

/// How hard a companion swings, from 0 to 1: its feistiness, energy and boldness, and a little
/// less if it is small.
pub fn strength(character: &Character) -> f32 {
    let a = character.axes;
    let heft = 0.4 * a.feistiness + 0.3 * a.energy + 0.3 * a.boldness;
    (0.45 + 0.55 * heft) * (0.55 + 0.45 * character.size.clamp(0.3, 1.0))
}

/// How a companion times a swing, from 0 to 1, and whether it came early: a patient one times
/// it steadily, an impulsive one often swings before it is ready.
pub fn timing(character: &Character, dice: &mut Dice) -> (f32, bool) {
    let impulsive = character.axes.impulsiveness;
    if dice.chance(((impulsive - 0.3) * 1.4).clamp(0.0, 0.85)) {
        (dice.range(0.3, 0.65), true)
    } else {
        (1.0 - dice.range(0.0, 0.06 + 0.3 * impulsive), false)
    }
}

/// How high the puck goes, from 0 to 1 (the bell): strength times timing.
pub fn height(strength: f32, timing: f32) -> f32 {
    (strength * (0.45 + 0.85 * timing)).clamp(0.03, 1.0)
}

/// The mark on the tower a puck sent `height` reaches: 1 to 9 up the stripes, 10 for the bell.
pub fn mark(height: f32) -> u32 {
    if height >= 1.0 {
        10
    } else {
        ((height * 10.0).floor() as u32).clamp(1, 9)
    }
}

/// How long the puck takes to go up, hangs, and comes down, for a swing that sends it `height`.
fn flight(height: f32) -> (f32, f32, f32) {
    (0.25 + 0.75 * height, 0.2, 0.2 + 0.45 * height)
}

/// How keen a companion is to have a go: whoever is keenest goes first.
fn keenness(character: &Character) -> f32 {
    let a = character.axes;
    let showy = if character.kind == TemperamentKind::Showoff {
        0.3
    } else {
        0.0
    };
    0.4 * a.feistiness + 0.3 * a.boldness + 0.3 * a.energy + showy
}

pub struct HighStriker {
    phase: Phase,
    turns: Vec<Turn>,
    current: usize,
    stage: Stage,
    watchers: Vec<Id>,
    events: Vec<Event>,
    round: u64,
    /// When the bell last rang.
    rung: Option<f32>,
    reduce_motion: bool,
    /// The last game's players, highest first, with how high each went.
    ranking: Vec<(Id, f32)>,
    /// Where each one waiting or watching was last sent to stand.
    placed: Vec<(Id, (f32, f32))>,
}

impl Default for HighStriker {
    fn default() -> Self {
        Self::new()
    }
}

impl HighStriker {
    pub fn new() -> Self {
        Self {
            phase: Phase::Ready,
            turns: Vec::new(),
            current: 0,
            stage: Stage::Done { until: 0.0 },
            watchers: Vec::new(),
            events: Vec::new(),
            round: 0,
            rung: None,
            reduce_motion: false,
            ranking: Vec::new(),
            placed: Vec::new(),
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Whoever is having a go now, and its parent if it is helping.
    pub fn player(&self) -> Option<(Id, Option<Id>)> {
        match self.phase {
            Phase::Playing { .. } => self.turns.get(self.current).map(|t| (t.player, t.helper)),
            _ => None,
        }
    }

    /// How many have had their go, of how many are playing.
    pub fn tally(&self) -> (usize, usize) {
        let done = match self.phase {
            Phase::Playing { .. } => self.current,
            _ => self.turns.len(),
        };
        (done, self.turns.len())
    }

    /// The best each player has done so far this game, or last game once it is over: highest
    /// first, ties in the order they went.
    pub fn ranking(&self) -> Vec<(Id, f32)> {
        if self.phase == Phase::Ready {
            return self.ranking.clone();
        }
        let mut ranking: Vec<(Id, f32)> = self
            .turns
            .iter()
            .filter_map(|turn| turn.best.map(|best| (turn.player, best)))
            .collect();
        ranking.sort_by(|a, b| b.1.total_cmp(&a.1));
        ranking
    }

    /// Starts a game for `players`, or everyone, keenest first. The rest watch.
    pub fn start(&mut self, ground: &mut Playground, players: &[Id], now: f32) {
        let everyone = ground.ids();
        let mut chosen: Vec<Id> = players
            .iter()
            .copied()
            .filter(|id| everyone.contains(id))
            .collect();
        if chosen.is_empty() {
            chosen = everyone.clone();
        }
        if chosen.is_empty() {
            return;
        }
        self.round += 1;
        let mut dice = Dice::new(
            self.round.wrapping_mul(0x51f1_5e11) ^ chosen.iter().fold(3, |seed, id| seed ^ id),
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
                // A little one's parent, if it is here, helps it hold the mallet.
                let parent = character
                    .parent
                    .filter(|parent| everyone.contains(parent))
                    .and_then(|parent| ground.character(parent).map(|c| (parent, c)));
                Some(plan(*id, character, parent, &mut dice))
            })
            .collect();
        self.watchers = everyone
            .iter()
            .copied()
            .filter(|id| !chosen.contains(id))
            .collect();
        self.events.clear();
        self.rung = None;
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

    /// Everyone waiting to their place in the queue, facing the striker, and everyone else to
    /// theirs among the watchers in front of the carousel: each a little apart from the next,
    /// however big. Only those whose place has changed move; whoever is up, or helping, stays put
    /// (its place is kept for it).
    fn place_everyone(&mut self, ground: &mut Playground, now: f32) {
        let busy: Vec<Id> = self
            .turns
            .get(self.current)
            .map(|turn| std::iter::once(turn.player).chain(turn.helper).collect())
            .unwrap_or_default();
        let (waiting, watching) = self.lines();
        let queue = line_up(&QUEUE, &sizes(ground, &waiting), LINE_GAP);
        let watch = line_up(&WATCH, &sizes(ground, &watching), LINE_GAP);
        let places = waiting
            .iter()
            .zip(queue)
            .map(|(id, at)| (*id, at, true))
            .chain(watching.iter().zip(watch).map(|(id, at)| (*id, at, false)));
        for (id, at, queueing) in places.collect::<Vec<_>>() {
            if busy.contains(&id) {
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
            // The queue faces the striker's pad, from whichever side; the watchers face whoever
            // is swinging.
            let facing = if queueing { PAD.0 as f32 } else { SPOT.0 };
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

    /// The current player steps up, with its parent if it is helping.
    fn begin_turn(&mut self, ground: &mut Playground, now: f32) {
        let Some(turn) = self.turns.get(self.current) else {
            return;
        };
        ground.direct(
            turn.player,
            vec![Step::Stride { to: SPOT }, Step::FaceX(SPOT.0 + 100.0)],
            now,
        );
        if let Some(helper) = turn.helper {
            ground.direct(
                helper,
                vec![
                    Step::Stride { to: HELPER_SPOT },
                    Step::FaceX(SPOT.0 + 100.0),
                ],
                now,
            );
        }
        self.events.push(Event::Up {
            player: turn.player,
            helper: turn.helper,
        });
        self.stage = Stage::Stepping { since: now };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        match self.phase {
            Phase::Ready => {}
            Phase::Playing { .. } => self.play(ground, now),
            Phase::Over { since } => {
                if now - since >= CELEBRATE_SECS {
                    self.ranking = self.ranking();
                    // Everyone stood watching the last swing is free to play again.
                    for id in ground.ids() {
                        ground.direct(id, Vec::new(), now);
                    }
                    ground.release();
                    self.turns.clear();
                    self.watchers.clear();
                    self.phase = Phase::Ready;
                }
            }
        }
    }

    fn play(&mut self, ground: &mut Playground, now: f32) {
        let Some(turn) = self.turns.get(self.current).cloned() else {
            return;
        };
        let both = |ground: &mut Playground, beats: &[Beat]| {
            for id in std::iter::once(turn.player).chain(turn.helper) {
                let steps = beats.iter().map(|beat| Step::Beat(*beat)).collect();
                ground.direct(id, steps, now);
            }
        };
        match self.stage {
            Stage::Stepping { since } => {
                let there = |id: Id, spot: (f32, f32)| {
                    ground.position(id).is_some_and(|at| {
                        (at.0 - spot.0).abs() < 1.0 && (at.1 - spot.1).abs() < 1.0
                    }) && !ground.walking(id)
                };
                // The first waits for everyone else to be in their places too, so nobody walks
                // across in front of it.
                let arrived = there(turn.player, SPOT)
                    && turn.helper.is_none_or(|helper| there(helper, HELPER_SPOT))
                    && (self.current > 0 || ground.ids().into_iter().all(|id| !ground.walking(id)));
                if !arrived && now - since < STEPPING_SECS {
                    return;
                }
                ground.teleport(turn.player, SPOT);
                if let Some(helper) = turn.helper {
                    ground.teleport(helper, HELPER_SPOT);
                }
                let character = ground.character(turn.player).cloned();
                let pick_up = Beat::new(Gesture::Reach, ExpressionKind::Determined, PICK_UP_SECS);
                let mut beats = vec![pick_up];
                let mut until = now + PICK_UP_SECS;
                match turn.prelude {
                    Prelude::Flourish => {
                        let mut strut =
                            Beat::new(Gesture::Strut, ExpressionKind::Smug, FLOURISH_SECS);
                        strut.cue = Some(Cue::Sparkle);
                        beats.push(strut);
                        until += FLOURISH_SECS;
                        self.events.push(Event::Flourish {
                            player: turn.player,
                        });
                    }
                    Prelude::Squint => {
                        beats.push(Beat::new(
                            Gesture::Watch,
                            ExpressionKind::Focused,
                            SQUINT_SECS,
                        ));
                        until += SQUINT_SECS;
                        self.events.push(Event::Squint {
                            player: turn.player,
                        });
                    }
                    Prelude::None => {}
                }
                let face = character.as_ref().map_or(ExpressionKind::Determined, |c| {
                    if c.kind == TemperamentKind::Lazybones {
                        ExpressionKind::Sleepy
                    } else {
                        ExpressionKind::Determined
                    }
                });
                beats.push(Beat::new(ActionKind::Idle, face, 600.0));
                ground.direct(
                    turn.player,
                    beats.into_iter().map(Step::Beat).collect(),
                    now,
                );
                if let Some(helper) = turn.helper {
                    let fond = Beat::new(ActionKind::Idle, ExpressionKind::Affectionate, 600.0);
                    ground.direct(helper, vec![Step::Beat(fond)], now);
                }
                self.stage = Stage::Prelude { until };
            }
            Stage::Prelude { until } => {
                if now >= until {
                    self.wind_up(ground, &turn, 0, now);
                }
            }
            Stage::WindUp { swing, until } => {
                if now >= until {
                    let s = turn.swings[swing];
                    let face = if s.early {
                        ExpressionKind::Startled
                    } else {
                        ExpressionKind::Determined
                    };
                    let strike = if s.tap {
                        Beat::new(Gesture::Reach, ExpressionKind::Bored, STRIKE_SECS)
                    } else {
                        Beat::new(Gesture::Crouch, face, STRIKE_SECS)
                    };
                    let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
                    both(ground, &[strike, watch]);
                    self.events.push(if s.tap {
                        Event::Tap {
                            player: turn.player,
                        }
                    } else {
                        Event::Swung {
                            player: turn.player,
                            helper: turn.helper,
                            height: s.height,
                            early: s.early,
                        }
                    });
                    self.stage = Stage::Flight { swing, since: now };
                }
            }
            Stage::Flight { swing, since } => {
                let s = turn.swings[swing];
                let (rise, hang, fall) = flight(s.height);
                if s.height >= 1.0 && self.rung.is_none_or(|at| at < since) && now - since >= rise {
                    self.rung = Some(now);
                    self.events.push(Event::Rang {
                        player: turn.player,
                        helper: turn.helper,
                    });
                }
                if now - since >= rise + hang + fall {
                    let best = &mut self.turns[self.current].best;
                    *best = Some(best.unwrap_or(0.0).max(s.height));
                    let beats = self.reaction(ground, &turn, s);
                    for (id, beats) in beats {
                        ground.direct(id, beats.into_iter().map(Step::Beat).collect(), now);
                    }
                    let seconds = if s.tap { AFTER_TAP_SECS } else { AFTER_SECS };
                    self.stage = Stage::After {
                        swing,
                        until: now + seconds,
                    };
                }
            }
            Stage::After { swing, until } => {
                if now < until {
                    return;
                }
                if swing + 1 < turn.swings.len() {
                    self.wind_up(ground, &turn, swing + 1, now);
                } else {
                    let put_down =
                        Beat::new(Gesture::Reach, ExpressionKind::Content, PUT_DOWN_SECS);
                    both(ground, &[put_down]);
                    self.stage = Stage::Done {
                        until: now + PUT_DOWN_SECS,
                    };
                }
            }
            Stage::Done { until } => {
                if now < until {
                    return;
                }
                self.next_turn(ground, &turn, now);
            }
        }
    }

    /// Up goes the mallet, for as long as this one winds up.
    fn wind_up(&mut self, ground: &mut Playground, turn: &Turn, swing: usize, now: f32) {
        let s = turn.swings[swing];
        let raise = if s.tap {
            Beat::new(Gesture::Reach, ExpressionKind::Bored, s.wind)
        } else {
            Beat::new(Gesture::Stretch, ExpressionKind::Determined, s.wind)
        };
        for id in std::iter::once(turn.player).chain(turn.helper) {
            ground.direct(id, vec![Step::Beat(raise)], now);
        }
        self.stage = Stage::WindUp {
            swing,
            until: now + s.wind,
        };
    }

    /// The player goes to watch, and its parent, if it helped, back to its place; the queue moves
    /// up; and the next one steps up, or, if that was everyone, the game is over.
    fn next_turn(&mut self, ground: &mut Playground, turn: &Turn, now: f32) {
        // Back from the striker, to wherever its place now is.
        self.placed
            .retain(|(id, _)| *id != turn.player && Some(*id) != turn.helper);
        self.current += 1;
        self.place_everyone(ground, now);
        if self.current >= self.turns.len() {
            self.over(ground, now);
            return;
        }
        self.begin_turn(ground, now);
    }

    /// Everyone has had a go: the colony cheers whoever went highest, and that one celebrates.
    fn over(&mut self, ground: &mut Playground, now: f32) {
        self.stage = Stage::Done { until: now };
        let ranking = self.ranking();
        let top = ranking.first().map(|(id, _)| *id);
        for id in ground.ids() {
            let Some(c) = ground.character(id) else {
                continue;
            };
            let beat = if Some(id) == top {
                c.celebrate(2.0)
            } else {
                Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.2)
            };
            let watch = Beat::new(ActionKind::Idle, ExpressionKind::Joy, 600.0);
            ground.direct(id, vec![Step::Beat(beat), Step::Beat(watch)], now);
        }
        self.ranking = ranking;
        self.events.push(Event::AllDone);
        self.phase = Phase::Over { since: now };
        self.current = self.turns.len();
    }

    /// How a player takes a swing, in its own way: a ring is celebrated, a near thing gasped
    /// at, and anything else taken as its temperament takes it. Its parent, if it helped, is
    /// proud of it whatever.
    fn reaction(&self, ground: &Playground, turn: &Turn, swing: Swing) -> Vec<(Id, Vec<Beat>)> {
        let Some(c) = ground.character(turn.player) else {
            return Vec::new();
        };
        let cue = |mut beat: Beat, cue: Cue| {
            beat.cue = Some(cue);
            beat
        };
        let rest = Beat::new(ActionKind::Idle, ExpressionKind::Content, 600.0);
        let mut beats = if swing.tap {
            vec![
                Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 0.9),
                Beat::new(ActionKind::Idle, ExpressionKind::Bored, 600.0),
            ]
        } else if swing.height >= 1.0 {
            match c.kind {
                TemperamentKind::Showoff => vec![cue(
                    Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.8),
                    Cue::Sparkle,
                )],
                TemperamentKind::Grump => vec![
                    cue(
                        Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.0),
                        Cue::Huff,
                    ),
                    cue(
                        Beat::new(ActionKind::Idle, ExpressionKind::Smug, 0.8),
                        Cue::Sparkle,
                    ),
                ],
                TemperamentKind::Lazybones => vec![
                    c.celebrate(1.0),
                    Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 0.8),
                ],
                TemperamentKind::Wallflower => vec![cue(
                    Beat::new(ActionKind::Idle, ExpressionKind::Joy, 1.8),
                    Cue::Heart,
                )],
                _ => vec![c.celebrate(1.8)],
            }
        } else if swing.height >= 0.8 {
            // So close.
            vec![
                cue(
                    Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.5),
                    Cue::Exclaim,
                ),
                cue(
                    Beat::new(ActionKind::Idle, ExpressionKind::Joy, 1.3),
                    Cue::Note,
                ),
            ]
        } else {
            match c.kind {
                TemperamentKind::Grump => vec![cue(
                    Beat::new(Gesture::Stomp, ExpressionKind::Grumpy, 1.4),
                    Cue::Huff,
                )],
                TemperamentKind::Showoff => {
                    vec![Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.4)]
                }
                TemperamentKind::Lazybones => {
                    vec![Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.3)]
                }
                _ if c.axes.playfulness > 0.6 => vec![cue(
                    Beat::new(Gesture::Bop, ExpressionKind::Joy, 1.4),
                    Cue::Note,
                )],
                _ => vec![Beat::new(ActionKind::Idle, ExpressionKind::Content, 1.4)],
            }
        };
        beats.push(rest);
        let mut all = vec![(turn.player, beats)];
        if let Some(helper) = turn.helper {
            let proud = if swing.height >= 1.0 {
                ground
                    .character(helper)
                    .map_or(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.8), |c| {
                        c.celebrate(1.8)
                    })
            } else {
                cue(
                    Beat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, 1.6),
                    Cue::Heart,
                )
            };
            all.push((helper, vec![proud, rest]));
        }
        all
    }

    /// Calls the game off: everyone back to playing, pleased with themselves.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        // Whoever was queueing, watching or helping stops; then the players are pleased.
        let waiting = self
            .watchers
            .iter()
            .copied()
            .chain(self.turns.iter().filter_map(|turn| turn.helper));
        for id in waiting.collect::<Vec<_>>() {
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
        self.phase = Phase::Ready;
    }

    /// How high the puck is now, from 0 to 1. With motion reduced it is simply at the top of its
    /// flight for the length of it: a cut there and a cut back.
    pub fn puck(&self, now: f32) -> f32 {
        let (Phase::Playing { .. }, Stage::Flight { swing, since }) = (self.phase, self.stage)
        else {
            return 0.0;
        };
        let Some(s) = self
            .turns
            .get(self.current)
            .and_then(|t| t.swings.get(swing))
        else {
            return 0.0;
        };
        let (rise, hang, fall) = flight(s.height);
        let t = now - since;
        if self.reduce_motion {
            return if t < rise + hang + fall {
                s.height
            } else {
                0.0
            };
        }
        if t < rise {
            let along = t / rise;
            s.height * (1.0 - (1.0 - along) * (1.0 - along))
        } else if t < rise + hang {
            s.height
        } else {
            let along = ((t - rise - hang) / fall).min(1.0);
            s.height * (1.0 - along * along)
        }
    }

    /// Whether the bell is ringing, and since how long.
    pub fn ringing(&self, now: f32) -> Option<f32> {
        self.rung
            .map(|at| now - at)
            .filter(|since| *since < RING_SECS)
    }

    /// Where the mallet is: leaning on the plinth, or in the hands of whoever is swinging.
    fn mallet(&self, ground: &mut Playground, now: f32) -> Mallet {
        let Some(turn) = self.turns.get(self.current) else {
            return Mallet::Leaning;
        };
        if !matches!(self.phase, Phase::Playing { .. }) {
            return Mallet::Leaning;
        }
        // A parent helping holds it from behind, over its little one's paws.
        let Some(crown) = ground.head(turn.player, now) else {
            return Mallet::Leaning;
        };
        let feet = ground.position(turn.player).map_or(SPOT.1, |at| at.1);
        match self.stage {
            Stage::Stepping { .. } | Stage::Done { .. } => Mallet::Leaning,
            Stage::Prelude { until } if picked_up(now, until, turn) => {
                Mallet::ready(crown, feet, true)
            }
            Stage::Prelude { .. } => Mallet::Leaning,
            // A half-hearted tap hardly lifts it.
            Stage::WindUp { swing, .. } if turn.swings[swing].tap => {
                Mallet::ready(crown, feet, true)
            }
            Stage::WindUp { .. } => Mallet::raised(crown, feet, true),
            Stage::Flight { since, .. } if now - since < STRIKE_SECS => {
                Mallet::down(crown, feet, true)
            }
            Stage::Flight { .. } | Stage::After { .. } => Mallet::ready(crown, feet, true),
        }
    }

    /// The striker's moving parts: the bell, the puck and the mallet, wherever they are.
    pub fn gear(&self, ground: &mut Playground, now: f32) -> Vec<Prop> {
        let mallet = self.mallet(ground, now);
        vec![
            gear::bell(self.ringing(now), self.reduce_motion),
            gear::puck(self.puck(now)),
            gear::mallet(mallet),
        ]
    }
}

/// A player's turn, planned from who it is: its prelude, and its swing or swings. A little one
/// whose parent is here swings with it, the parent setting the time and putting its back into it.
fn plan(id: Id, character: &Character, parent: Option<(Id, &Character)>, dice: &mut Dice) -> Turn {
    let prelude = match character.kind {
        TemperamentKind::Showoff => Prelude::Flourish,
        TemperamentKind::Scholar => Prelude::Squint,
        _ => Prelude::None,
    };
    let (power, timer) = match parent {
        Some((_, grown)) => (
            (strength(grown) + 0.5 * strength(character) + 0.05).min(1.0),
            grown,
        ),
        None => (strength(character), character),
    };
    let mut swings = Vec::new();
    let mut power = power;
    if character.kind == TemperamentKind::Lazybones && parent.is_none() {
        swings.push(Swing {
            wind: 0.35,
            height: dice.range(0.06, 0.16),
            early: false,
            tap: true,
        });
        // And then it puts its back into it.
        power = (power + 0.1).min(1.0);
    }
    let (timing, early) = timing(timer, dice);
    let wind = if early {
        dice.range(0.25, 0.4)
    } else {
        0.6 + 0.6 * (1.0 - timer.axes.impulsiveness)
    };
    swings.push(Swing {
        wind,
        height: height(power, timing),
        early,
        tap: false,
    });
    Turn {
        player: id,
        helper: parent.map(|(parent, _)| parent),
        prelude,
        swings,
        best: None,
    }
}

/// Whether the player has got the mallet in its paws yet, partway through its prelude.
fn picked_up(now: f32, until: f32, turn: &Turn) -> bool {
    let prelude = match turn.prelude {
        Prelude::Flourish => FLOURISH_SECS,
        Prelude::Squint => SQUINT_SECS,
        Prelude::None => 0.0,
    };
    until - now < prelude + PICK_UP_SECS * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
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

    /// The fairground with the colony settled in, and the striker.
    fn ready(cast: &Cast) -> (Playground, HighStriker) {
        let (mut ground, _) = fairground::open(cast, 0.0);
        let mut now = 0.0;
        while now < 6.0 {
            now += TICK;
            ground.tick(cast, now);
        }
        (ground, HighStriker::new())
    }

    /// A game from `from` until everyone is back to playing, or `to`: what happened, and when
    /// it was over.
    fn run(
        ground: &mut Playground,
        striker: &mut HighStriker,
        cast: &Cast,
        from: f32,
        to: f32,
        mut each: impl FnMut(&mut HighStriker, &mut Playground, f32),
    ) -> (Vec<Event>, Option<f32>) {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += TICK;
            ground.tick(cast, now);
            striker.tick(ground, now);
            each(striker, ground, now);
            events.extend(striker.take_events());
            if striker.phase() == Phase::Ready {
                return (events, Some(now));
            }
        }
        (events, None)
    }

    fn game(cast: &Cast, players: &[Id]) -> (Vec<Event>, Vec<(Id, f32)>) {
        let (mut ground, mut striker) = ready(cast);
        striker.start(&mut ground, players, 6.0);
        let (events, over) = run(&mut ground, &mut striker, cast, 6.0, 300.0, |_, _, _| {});
        assert!(over.is_some(), "the game never ended: {events:?}");
        (events, striker.ranking())
    }

    #[test]
    fn everyone_has_a_go_and_the_ranking_puts_the_highest_first() {
        let cast = sample();
        let (events, ranking) = game(&cast, &[]);
        assert_eq!(ranking.len(), cast.members.len());
        for member in &cast.members {
            assert!(
                ranking.iter().any(|(id, _)| *id == member.id),
                "{} had no go",
                member.name
            );
        }
        assert!(ranking.windows(2).all(|pair| pair[0].1 >= pair[1].1));
        assert!(
            ranking
                .iter()
                .all(|(_, height)| (0.0..=1.0).contains(height))
        );
        assert_eq!(events.last(), Some(&Event::AllDone));
        let ups = events
            .iter()
            .filter(|e| matches!(e, Event::Up { .. }))
            .count();
        assert_eq!(ups, cast.members.len());
    }

    #[test]
    fn whoever_is_picked_has_a_go_and_the_rest_watch() {
        let cast = sample();
        let picked = [cast.members[2].id, cast.members[4].id];
        let (_, ranking) = game(&cast, &picked);
        let mut played: Vec<Id> = ranking.iter().map(|(id, _)| *id).collect();
        played.sort();
        let mut wanted = picked.to_vec();
        wanted.sort();
        assert_eq!(played, wanted);
    }

    #[test]
    fn the_game_always_ends_however_the_colony_is() {
        let tunings: [fn(usize, &mut Traveler); 3] = [
            |_, t| t.character.temperament = TemperamentKind::Lazybones.into(),
            |_, t| t.character.temperament = TemperamentKind::Showoff.into(),
            |_, t| {
                t.character.axes.impulsiveness = 1.0;
                t.character.axes.feistiness = 1.0;
            },
        ];
        for tune in tunings {
            let cast = colony(tune);
            let (events, ranking) = game(&cast, &[]);
            assert_eq!(ranking.len(), cast.members.len(), "{events:?}");
        }
    }

    fn character(tune: impl FnOnce(&mut Character)) -> Character {
        let mut character = Character::of(&sample().members[1]);
        character.parent = None;
        character.size = 1.0;
        tune(&mut character);
        character
    }

    #[test]
    fn the_puck_climbs_by_strength_times_timing() {
        let strong = character(|c| {
            c.axes.feistiness = 1.0;
            c.axes.energy = 1.0;
            c.axes.boldness = 1.0;
        });
        let weak = character(|c| {
            c.axes.feistiness = 0.0;
            c.axes.energy = 0.0;
            c.axes.boldness = 0.0;
        });
        assert!(strength(&strong) > strength(&weak) * 1.5);
        assert!(
            height(strength(&strong), 1.0) >= 1.0,
            "the strongest can ring it"
        );
        assert!(
            height(strength(&weak), 1.0) < 1.0,
            "the weakest never rings it alone"
        );
        assert!(height(strength(&strong), 0.4) < height(strength(&strong), 0.95));
        let mut small = strong.clone();
        small.size = 0.6;
        assert!(
            strength(&small) < strength(&strong),
            "a little one swings lighter"
        );
    }

    #[test]
    fn an_impulsive_one_often_swings_early_and_a_patient_one_steadily() {
        let swings = |impulsiveness: f32| {
            let c = character(|c| c.axes.impulsiveness = impulsiveness);
            let mut dice = Dice::new(42);
            (0..300).map(|_| timing(&c, &mut dice)).collect::<Vec<_>>()
        };
        let impulsive = swings(0.95);
        let patient = swings(0.1);
        let early = |swings: &[(f32, bool)]| swings.iter().filter(|(_, early)| *early).count();
        assert!(early(&impulsive) > 150, "{}", early(&impulsive));
        assert_eq!(early(&patient), 0);
        assert!(patient.iter().all(|(timing, _)| *timing > 0.85));
        let average = |swings: &[(f32, bool)]| {
            swings.iter().map(|(timing, _)| timing).sum::<f32>() / swings.len() as f32
        };
        assert!(average(&patient) > average(&impulsive) + 0.2);
    }

    #[test]
    fn a_show_off_flourishes_and_a_scholar_squints_up_at_the_marks_first() {
        let cast = colony(|i, t| {
            t.character.temperament = match i {
                1 => TemperamentKind::Showoff.into(),
                3 => TemperamentKind::Scholar.into(),
                _ => TemperamentKind::Guardian.into(),
            };
        });
        let (showy, scholar) = (cast.members[1].id, cast.members[3].id);
        let (events, _) = game(&cast, &[]);
        let at = |wanted: Event| events.iter().position(|e| *e == wanted);
        let swung = |player: Id| {
            events
                .iter()
                .position(|e| matches!(e, Event::Swung { player: p, .. } if *p == player))
        };
        assert!(at(Event::Flourish { player: showy }) < swung(showy));
        assert!(at(Event::Squint { player: scholar }) < swung(scholar));
        assert!(at(Event::Flourish { player: showy }).is_some());
        assert!(at(Event::Squint { player: scholar }).is_some());
        let others = events
            .iter()
            .filter(|e| matches!(e, Event::Flourish { .. } | Event::Squint { .. }))
            .count();
        assert_eq!(others, 2, "nobody else shows off or squints");
    }

    #[test]
    fn a_lazybones_gives_it_a_half_hearted_tap_and_then_a_proper_swing() {
        let cast = colony(|i, t| {
            t.character.temperament = if i == 2 {
                TemperamentKind::Lazybones.into()
            } else {
                TemperamentKind::Guardian.into()
            };
        });
        let lazy = cast.members[2].id;
        let (events, ranking) = game(&cast, &[]);
        let tap = events
            .iter()
            .position(|e| *e == Event::Tap { player: lazy });
        let swing = events
            .iter()
            .position(|e| matches!(e, Event::Swung { player, .. } if *player == lazy));
        assert!(tap.is_some() && tap < swing, "{events:?}");
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Tap { .. }))
                .count(),
            1
        );
        let best = ranking.iter().find(|(id, _)| *id == lazy).unwrap().1;
        assert!(best > 0.2, "the proper swing counts, not the tap");
    }

    #[test]
    fn a_little_one_swings_with_its_parent_and_together_they_can_ring_the_bell() {
        let cast = sample();
        let little = cast.members.iter().find(|m| m.parent().is_some()).unwrap();
        let (mini, parent) = (little.id, little.parent().unwrap());
        let (mut ground, mut striker) = ready(&cast);
        let mut rang = 0;
        for round in 0..4 {
            let from = 6.0 + round as f32 * 100.0;
            striker.start(&mut ground, &[mini], from);
            let (events, over) = run(
                &mut ground,
                &mut striker,
                &cast,
                from,
                from + 100.0,
                |_, _, _| {},
            );
            assert!(over.is_some());
            assert!(events.contains(&Event::Up {
                player: mini,
                helper: Some(parent)
            }));
            rang += events
                .iter()
                .filter(|e| {
                    **e == Event::Rang {
                        player: mini,
                        helper: Some(parent),
                    }
                })
                .count();
        }
        assert!(rang > 0, "the two of them never rang it");
        // Alone, the little one would never manage it.
        let alone = Character::of(little);
        assert!(height(strength(&alone), 1.0) < 1.0);
    }

    #[test]
    fn ringing_the_bell_makes_it_ring_and_shine_and_the_ringer_celebrates() {
        let cast = colony(|_, t| {
            t.character.axes.feistiness = 1.0;
            t.character.axes.energy = 1.0;
            t.character.axes.boldness = 1.0;
            t.character.axes.impulsiveness = 0.0;
        });
        let player = cast.members[1].id;
        let (mut ground, mut striker) = ready(&cast);
        striker.start(&mut ground, &[player], 6.0);
        let mut shining = None;
        let (events, _) = run(
            &mut ground,
            &mut striker,
            &cast,
            6.0,
            100.0,
            |striker, _, now| {
                if shining.is_none() && striker.ringing(now).is_some() {
                    shining = Some(striker.puck(now));
                }
            },
        );
        assert!(
            events.contains(&Event::Rang {
                player,
                helper: None
            }),
            "{events:?}"
        );
        assert_eq!(shining, Some(1.0), "the puck was at the bell as it rang");
        let still = gear::bell(None, false).sprite;
        let rung = gear::bell(Some(0.2), false).sprite;
        assert_ne!(still, rung, "the bell shines when it rings");
        // And how it is taken: a celebration, where a low swing is not.
        let turn = Turn {
            player,
            helper: None,
            prelude: Prelude::None,
            swings: Vec::new(),
            best: None,
        };
        let swing = |height| Swing {
            wind: 1.0,
            height,
            early: false,
            tap: false,
        };
        let cheered = striker.reaction(&ground, &turn, swing(1.0));
        let shrugged = striker.reaction(&ground, &turn, swing(0.3));
        let sparkles = |beats: &[(Id, Vec<Beat>)]| {
            beats[0].1.iter().any(|beat| beat.cue == Some(Cue::Sparkle))
        };
        assert!(sparkles(&cheered) && !sparkles(&shrugged));
    }

    #[test]
    fn the_same_colony_with_other_temperaments_plays_differently() {
        let others = formiga_travel::sample::snapshot();
        let swapped = colony(|i, t| {
            let other = &others.travelers[(i + 2) % others.travelers.len()];
            t.character = other.character.clone();
            t.motion = other.motion;
        });
        let (first, ranking) = game(&sample(), &[]);
        let (second, other_ranking) = game(&swapped, &[]);
        assert_ne!(first, second);
        assert_ne!(ranking, other_ranking);
    }

    #[test]
    fn with_reduced_motion_the_puck_cuts_to_its_height_and_back() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut striker) = ready(&cast);
        striker.start(&mut ground, &[], 6.0);
        let mut heights = Vec::new();
        let mut swung = Vec::new();
        let (events, over) = run(
            &mut ground,
            &mut striker,
            &cast,
            6.0,
            300.0,
            |striker, _, now| {
                heights.push(striker.puck(now));
                if let Stage::Flight { swing, .. } = striker.stage
                    && let Some(turn) = striker.turns.get(striker.current)
                {
                    swung.push(turn.swings[swing].height);
                }
            },
        );
        assert!(over.is_some(), "{events:?}");
        // Only ever at rest or at the top of a swing: never on its way.
        for height in heights {
            assert!(
                height == 0.0 || swung.contains(&height),
                "the puck was seen at {height} on its way"
            );
        }
        assert!(!swung.is_empty());
    }

    #[test]
    fn calling_it_off_lets_everyone_go() {
        let cast = sample();
        let (mut ground, mut striker) = ready(&cast);
        striker.start(&mut ground, &[], 6.0);
        run(&mut ground, &mut striker, &cast, 6.0, 12.0, |_, _, _| {});
        assert!(striker.player().is_some());
        striker.stop(&mut ground, 12.0);
        assert_eq!(striker.phase(), Phase::Ready);
        assert_eq!(striker.player(), None);
        assert_eq!(striker.mallet(&mut ground, 12.0), Mallet::Leaning);
    }

    /// How many of `ids` have moved, `secs` after `from`, from where they were then.
    fn moved_within(
        ground: &mut Playground,
        cast: &Cast,
        ids: &[Id],
        from: f32,
        secs: f32,
    ) -> usize {
        let was: Vec<_> = ids.iter().map(|id| ground.position(*id).unwrap()).collect();
        let mut now = from;
        while now < from + secs {
            now += TICK;
            ground.tick(cast, now);
        }
        ids.iter()
            .zip(&was)
            .filter(|(id, was)| {
                crate::playground::distance(ground.position(**id).unwrap(), **was) > 2.0
            })
            .count()
    }

    #[test]
    fn after_a_game_everyone_goes_back_to_playing() {
        let cast = sample();
        let (mut ground, mut striker) = ready(&cast);
        striker.start(&mut ground, &[], 6.0);
        let (events, over) = run(&mut ground, &mut striker, &cast, 6.0, 300.0, |_, _, _| {});
        let over = over.unwrap_or_else(|| panic!("the game never ended: {events:?}"));
        let everyone = ground.ids();
        assert!(
            moved_within(&mut ground, &cast, &everyone, over, 30.0) > 0,
            "the colony is still stood at the striker half a minute on"
        );
    }

    #[test]
    fn the_queue_waits_in_an_orderly_line_each_a_little_apart_and_clear_of_the_props() {
        use crate::fairground::testing::{area, colony_of, overlap};
        for count in [6, 9] {
            let cast = colony_of(count);
            let (mut ground, mut striker) = ready(&cast);
            striker.start(&mut ground, &[], 6.0);
            // Until everyone is in its place, while the first is still up.
            let mut now = 6.0;
            let settled = |striker: &HighStriker, ground: &Playground| {
                let (waiting, watching) = striker.lines();
                waiting
                    .iter()
                    .chain(&watching)
                    .all(|id| !ground.walking(*id))
            };
            while now < 40.0 && !settled(&striker, &ground) {
                now += TICK;
                ground.tick(&cast, now);
                striker.tick(&mut ground, now);
            }
            assert_eq!(
                striker.current, 0,
                "the first had its go before the rest were in line"
            );
            let (waiting, _) = striker.lines();
            assert!(
                waiting.len() >= count - 2,
                "{count}: only {} waiting",
                waiting.len()
            );
            let drawn: Vec<_> = waiting
                .iter()
                .map(|id| ground.bounds(*id, now).unwrap())
                .collect();
            for id in &waiting {
                eprintln!(
                    "{} at {:?} drawn {:?} size {:?}",
                    cast.member(*id).unwrap().name,
                    ground.position(*id),
                    ground.bounds(*id, now),
                    (ground.width(*id), ground.height(*id))
                );
            }
            for (a, first) in drawn.iter().enumerate() {
                for second in &drawn[a + 1..] {
                    let shared = overlap(*first, *second);
                    assert!(
                        shared * 10 <= area(*first).min(area(*second)),
                        "{count}: two in the queue stand over each other: {first:?} {second:?}"
                    );
                }
                // Well in front of anything it is drawn over: never behind it, or on top of it.
                for prop in ground.props() {
                    if overlap(*first, prop.bounds()) > 0 {
                        assert!(
                            prop.base + 8.0 <= first.3 as f32,
                            "{count}: someone queues at {first:?}, against a prop at {:?}",
                            prop.bounds()
                        );
                    }
                }
            }
            // Everyone waiting faces the striker.
            for id in &waiting {
                let (x, _) = ground.position(*id).unwrap();
                let facing_right = ground.facing_right(*id).unwrap();
                assert_eq!(facing_right, x < PAD.0 as f32, "{count}: {id} faces away");
            }
        }
    }

    #[test]
    fn calling_a_game_off_lets_the_watchers_go_too() {
        let cast = sample();
        let (mut ground, mut striker) = ready(&cast);
        // Two have a go, and the rest watch.
        let players: Vec<Id> = cast.ids().take(2).collect();
        striker.start(&mut ground, &players, 6.0);
        let (_, over) = run(&mut ground, &mut striker, &cast, 6.0, 20.0, |_, _, _| {});
        assert!(over.is_none(), "the game was over too soon to call off");
        let watchers = striker.watchers.clone();
        assert!(!watchers.is_empty());
        striker.stop(&mut ground, 20.0);
        assert!(
            moved_within(&mut ground, &cast, &watchers, 20.0, 30.0) > 0,
            "no watcher has moved in the half minute since the game was called off"
        );
    }
}
