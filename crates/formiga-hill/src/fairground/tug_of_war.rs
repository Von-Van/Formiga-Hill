//! The tug-of-war, pulled by the colony while the person watches. Two sides take up a rope laid
//! across a chalk line on the clear sawdust at the front, with hay strewn about the line to
//! tumble into, and a red ribbon tied at the rope's middle. The ribbon drifts as one side outpulls
//! the other, until one side draws the other over the line: the losers tumble softly into the
//! hay, the winners sit down hard, and then everyone laughs and celebrates in their own way.
//!
//! Nobody is written in. Unless the person picks the sides, the colony sorts itself: friends
//! together, those who don't get on on opposite sides, a parent with its little ones, and the
//! strength about even. How hard each one pulls comes from its feistiness, its energy and its
//! size, and pulling together matters: a close pair on one side pulls in rhythm, two who don't get
//! on pull out of step, a lazybones lets the rope go slack a moment, a show-off stops to wave, a
//! parent pulls all the harder beside its little one, and an affectionate one cheers its side on.
//! Losing costs nothing: it is only the hay, and a laugh.

use super::gear;
use super::{LINE_GAP, Path, line_up, sizes};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::{Playground, Prop};
use formiga_art::{BodyClip, ExpressionKind};
use formiga_core::{ActionKind, Gesture, TemperamentKind};
use formiga_travel::Band;

/// The chalk line across the middle, and the row the rope is pulled along.
pub const LINE_X: f32 = 198.0;
pub const ROPE_Y: f32 = 180.0;
/// How far either side of the line each side may stand: clear of the straw at the left and the
/// handcart at the right.
const REACH: (f32, f32) = (66.0, 332.0);
/// How far from the line the one at the front of each side stands, between its nearer edge and
/// the line.
const FRONT: f32 = 6.0;
/// How far the ribbon must be drawn past the line for a side to win: the losers' front one is
/// then well over it.
pub const WIN: f32 = 13.0;
/// How fast the rope goes the way it is pulled, in pixels a second when one side has it all.
const GIVE: f32 = 7.0;
/// Once one side is ahead, how much faster it pulls away the longer the bout goes on, at most,
/// and how long that takes to come on: so every bout ends.
const DECIDING: f32 = 3.5;
const DECIDING_SECS: f32 = 24.0;
/// However it goes, the side ahead has won by this long after "pull!".
const LONGEST: f32 = 70.0;
/// "Take the strain…", and "pull!" after this long.
const STRAIN_SECS: f32 = 1.6;
/// The longest anyone is given to take up the rope.
const TAKING_UP_SECS: f32 = 12.0;
/// The losers stumbling over into the hay, and everyone laughing after.
const TUMBLE_SECS: f32 = 1.8;
const LAUGH_SECS: f32 = 4.5;
/// A slack moment, a wave, and a cheer.
const SLACK_SECS: f32 = 1.4;
const WAVE_SECS: f32 = 1.6;
const CHEER_SECS: f32 = 1.0;
/// How long a cheer heartens its side for.
const HEARTENED_SECS: f32 = 3.0;
/// When the first word about how each side is pulling together comes, after "pull!", and how
/// long after it each next one.
const REMARK_FIRST: f32 = 0.8;
const REMARK_SECS: f32 = 1.8;
/// How long one heave of the rope takes.
const HEAVE_SECS: f32 = 1.2;
/// With motion reduced, the rope and everyone on it are shown at each moment, a cut between.
const CUT_SECS: f32 = 1.0;
/// Where those not pulling watch from: behind the rope, each a little apart from the next.
const WATCH: [Path; 2] = [
    &[(144.0, 140.0), (332.0, 140.0)],
    &[(150.0, 124.0), (232.0, 124.0)],
];

/// One side of the rope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// Which way along the rope this side pulls: towards its own end.
    fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Ready,
    /// Everyone taking up the rope.
    TakingUp {
        since: f32,
    },
    /// "Take the strain…": "pull!" comes at `pull`.
    Strain {
        since: f32,
        pull: f32,
    },
    Pulling {
        since: f32,
    },
    /// One side has drawn the other over the line, and the losers are tumbling into the hay.
    Tumbling {
        since: f32,
        winners: Side,
    },
    /// Everyone laughing and celebrating.
    Over {
        since: f32,
        winners: Side,
    },
}

/// Something that happened at the rope, for the person watching to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The sides are taken up: by the person's choice, or as the colony sorted itself. See
    /// `sides`.
    Sides {
        chosen: bool,
    },
    /// A close pair on one side, pulling in rhythm; and two who don't get on, out of step.
    InStep {
        a: Id,
        b: Id,
    },
    OutOfStep {
        a: Id,
        b: Id,
    },
    /// A parent pulling all the harder beside its little one.
    Beside {
        parent: Id,
        little: Id,
    },
    /// "Pull!"
    Pull,
    /// The rope gone slack in its paws a moment; a wave to the crowd; a cheer for its side.
    Slack {
        puller: Id,
    },
    Waves {
        puller: Id,
    },
    Cheers {
        puller: Id,
    },
    /// One side has drawn the other over the line, `took` seconds after "pull!".
    Won {
        side: Side,
        took: f32,
    },
    /// Everyone has laughed it off and gone back to playing.
    AllDone,
}

/// Something a puller does for a moment instead of pulling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Moment {
    Slack,
    Wave,
    Cheer,
}

#[derive(Clone, Debug)]
struct Puller {
    id: Id,
    side: Side,
    /// Where it stands with the rope where it started.
    home: (f32, f32),
    /// How hard it pulls, from its feistiness, energy and size, and how much more or less for
    /// how it pulls with its side.
    strength: f32,
    together: f32,
    /// How much it tires over a long bout, from its energy.
    tiring: f32,
    /// Its heave's rhythm, in turns: matched with a close friend's, against a rival's.
    rhythm: f32,
    /// When it stops pulling a moment, and why, in seconds after "pull!".
    moments: Vec<(f32, Moment)>,
    /// What it is shown doing, so it is only told again when that changes.
    shown: Option<(BodyClip, ExpressionKind, Option<Cue>, bool)>,
}

impl Puller {
    /// What it is doing at `pulling` seconds after "pull!", if anything but pulling.
    fn moment(&self, pulling: f32) -> Option<Moment> {
        self.moments.iter().find_map(|&(at, moment)| {
            let lasts = match moment {
                Moment::Slack => SLACK_SECS,
                Moment::Wave => WAVE_SECS,
                Moment::Cheer => CHEER_SECS,
            };
            (pulling >= at && pulling < at + lasts).then_some(moment)
        })
    }
}

/// How hard a companion pulls, from 0 to about 1: its feistiness and its energy, and its size.
pub fn strength(character: &Character) -> f32 {
    let a = character.axes;
    (0.25 + 0.45 * a.feistiness + 0.3 * a.energy) * (0.45 + 0.55 * character.size.clamp(0.3, 1.3))
}

/// Whether a pair is close enough to pull in rhythm.
fn close(cast: &Cast, a: Id, b: Id) -> bool {
    cast.bond(a, b)
        .is_some_and(|bond| bond.warmth >= Band::High && bond.friction < bond.warmth)
}

/// Whether a companion cheers its side on.
fn cheerful(character: &Character) -> bool {
    character.kind == TemperamentKind::Sweetheart || character.axes.affection >= 0.8
}

/// The colony sorting itself into two sides: a parent with its little ones always together, as
/// even in number as can be, and of all the ways that leaves, the one that best keeps friends
/// together, puts those who don't get on on opposite sides, and evens out the strength. The side
/// with the first of them in it is the left.
pub fn sort(cast: &Cast, pullers: &[(Id, Character)]) -> (Vec<Id>, Vec<Id>) {
    // Families go together: a little one goes where its parent goes.
    let family = |index: usize| {
        pullers[index]
            .1
            .parent
            .and_then(|parent| pullers.iter().position(|(other, _)| *other == parent))
            .unwrap_or(index)
    };
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for index in 0..pullers.len() {
        let head = family(index);
        match groups.iter_mut().find(|group| family(group[0]) == head) {
            Some(group) => group.push(index),
            None => groups.push(vec![index]),
        }
    }
    let strengths: Vec<f32> = pullers.iter().map(|(_, c)| strength(c)).collect();
    let count = pullers.len();
    let mut best: Option<(f32, u32)> = None;
    let mut fewest_apart = usize::MAX;
    // Which side each group goes on, the first group always on the left.
    let ways = 1u32 << groups.len().saturating_sub(1);
    let side_of = |mask: u32, member: usize| -> Side {
        let group = groups.iter().position(|g| g.contains(&member)).unwrap_or(0);
        if group > 0 && mask & (1 << (group - 1)) != 0 {
            Side::Right
        } else {
            Side::Left
        }
    };
    for mask in 0..ways {
        let lefts = (0..count)
            .filter(|&m| side_of(mask, m) == Side::Left)
            .count();
        if lefts == 0 || lefts == count {
            continue;
        }
        fewest_apart = fewest_apart.min(lefts.abs_diff(count - lefts));
    }
    for mask in 0..ways {
        let lefts = (0..count)
            .filter(|&m| side_of(mask, m) == Side::Left)
            .count();
        if lefts == 0 || lefts == count || lefts.abs_diff(count - lefts) > fewest_apart {
            continue;
        }
        let mut score = 0.0;
        let (mut left, mut right) = (0.0, 0.0);
        for (m, pull) in strengths.iter().enumerate() {
            match side_of(mask, m) {
                Side::Left => left += pull,
                Side::Right => right += pull,
            }
        }
        score -= 3.0 * (left - right).abs() / (left + right).max(0.01);
        for a in 0..count {
            for b in a + 1..count {
                let (ida, idb) = (pullers[a].0, pullers[b].0);
                let together = side_of(mask, a) == side_of(mask, b);
                if cast.at_odds(ida, idb) {
                    score += if together { -1.0 } else { 1.0 };
                } else if let Some(bond) = cast.bond(ida, idb)
                    && bond.warmth >= Band::Medium
                    && bond.friction < bond.warmth
                    && together
                {
                    score += if bond.warmth >= Band::High { 0.5 } else { 0.25 };
                }
            }
        }
        if best.is_none_or(|(top, _)| score > top + 1e-4) {
            best = Some((score, mask));
        }
    }
    let mask = best.map_or(0, |(_, mask)| mask);
    let (mut left, mut right) = (Vec::new(), Vec::new());
    for (m, (id, _)) in pullers.iter().enumerate() {
        match side_of(mask, m) {
            Side::Left => left.push(*id),
            Side::Right => right.push(*id),
        }
    }
    if right.is_empty() && left.len() > 1 {
        // Only one family here: it pulls against itself, the parent against its little ones.
        right = left.split_off(1);
    }
    (left, right)
}

pub struct TugOfWar {
    phase: Phase,
    pullers: Vec<Puller>,
    watchers: Vec<Id>,
    events: Vec<Event>,
    round: u64,
    dice: Dice,
    reduce_motion: bool,
    /// How far the rope has been pulled from where it started: towards the right side's end if
    /// more than nothing.
    offset: f32,
    /// How far it is shown pulled: with motion reduced, only where it was at the last cut.
    shown_offset: f32,
    last_cut: f32,
    /// Whether the person chose the sides.
    chosen: bool,
    /// When each side was last cheered on.
    heartened: [Option<f32>; 2],
    /// The last bout's sides, and who won it.
    last: Option<(Vec<Id>, Vec<Id>, Side)>,
    /// Where each of the losers comes to rest in the hay.
    tumbles: Vec<(Id, (f32, f32))>,
    /// How each side pulls together, told one at a time once the pull is under way, and how
    /// many have been told.
    remarks: Vec<Event>,
    remarked: usize,
    clock: f32,
}

impl Default for TugOfWar {
    fn default() -> Self {
        Self::new()
    }
}

impl TugOfWar {
    pub fn new() -> Self {
        Self {
            phase: Phase::Ready,
            pullers: Vec::new(),
            watchers: Vec::new(),
            events: Vec::new(),
            round: 0,
            dice: Dice::new(1),
            reduce_motion: false,
            offset: 0.0,
            shown_offset: 0.0,
            last_cut: 0.0,
            chosen: false,
            heartened: [None; 2],
            last: None,
            tumbles: Vec::new(),
            remarks: Vec::new(),
            remarked: 0,
            clock: 0.0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Who is on each side, front first, this bout or the last.
    pub fn sides(&self) -> (Vec<Id>, Vec<Id>) {
        if self.pullers.is_empty() {
            return self
                .last
                .as_ref()
                .map_or((Vec::new(), Vec::new()), |(left, right, _)| {
                    (left.clone(), right.clone())
                });
        }
        let on = |side: Side| {
            self.pullers
                .iter()
                .filter(|p| p.side == side)
                .map(|p| p.id)
                .collect()
        };
        (on(Side::Left), on(Side::Right))
    }

    /// Who won the last bout, once there is one.
    pub fn winners(&self) -> Option<Side> {
        match self.phase {
            Phase::Tumbling { winners, .. } | Phase::Over { winners, .. } => Some(winners),
            Phase::Ready => self.last.as_ref().map(|(_, _, side)| *side),
            _ => None,
        }
    }

    /// Every player's result in the last bout: whether its side won.
    #[cfg(test)]
    pub fn results(&self) -> Vec<(Id, bool)> {
        let Some(winners) = self.winners() else {
            return Vec::new();
        };
        let (left, right) = self.sides();
        left.into_iter()
            .map(|id| (id, winners == Side::Left))
            .chain(right.into_iter().map(|id| (id, winners == Side::Right)))
            .collect()
    }

    /// How far the ribbon is from the line, and which way: towards the right side if more than
    /// nothing.
    pub fn ribbon(&self) -> f32 {
        self.offset
    }

    /// Seconds since "pull!".
    pub fn pulling_for(&self, now: f32) -> f32 {
        match self.phase {
            Phase::Pulling { since } => now - since,
            _ => 0.0,
        }
    }

    /// Starts a bout: the person's own sides, if it has chosen them, or else everyone (or those
    /// picked) as the colony sorts itself. The rest watch.
    pub fn start(
        &mut self,
        ground: &mut Playground,
        cast: &Cast,
        players: &[Id],
        sides: Option<(Vec<Id>, Vec<Id>)>,
        now: f32,
    ) {
        let everyone = ground.ids();
        let here = |ids: Vec<Id>| -> Vec<Id> {
            ids.into_iter().filter(|id| everyone.contains(id)).collect()
        };
        let chosen = sides
            .map(|(left, right)| (here(left), here(right)))
            .filter(|(left, right)| !left.is_empty() && !right.is_empty());
        let (left, right) = match &chosen {
            Some(sides) => sides.clone(),
            None => {
                let mut playing = here(players.to_vec());
                if playing.len() < 2 {
                    playing.clone_from(&everyone);
                }
                let characters: Vec<(Id, Character)> = playing
                    .iter()
                    .filter_map(|id| ground.character(*id).map(|c| (*id, c.clone())))
                    .collect();
                sort(cast, &characters)
            }
        };
        if left.is_empty() || right.is_empty() {
            return;
        }
        self.round += 1;
        self.dice = Dice::new(
            self.round.wrapping_mul(0x7a6f_0003)
                ^ left.iter().chain(&right).fold(11, |seed, id| seed ^ id),
        );
        self.reduce_motion = ground.reduce_motion();
        self.chosen = chosen.is_some();
        self.events.clear();
        self.remarks.clear();
        self.remarked = 0;
        self.offset = 0.0;
        self.shown_offset = 0.0;
        self.heartened = [None; 2];
        self.clock = now;
        self.pullers = Vec::new();
        for (side, ids) in [(Side::Left, &left), (Side::Right, &right)] {
            self.take_side(ground, side, ids);
        }
        self.pull_together(cast);
        self.watchers = everyone
            .iter()
            .copied()
            .filter(|id| !left.contains(id) && !right.contains(id))
            .collect();
        let mut reserved: Vec<Id> = self.pullers.iter().map(|p| p.id).collect();
        reserved.extend(&self.watchers);
        ground.reserve(reserved);
        for puller in &self.pullers {
            let facing = LINE_X;
            ground.direct(
                puller.id,
                vec![Step::Stride { to: puller.home }, Step::FaceX(facing)],
                now,
            );
        }
        let spots = line_up(&WATCH, &sizes(ground, &self.watchers), LINE_GAP);
        for (id, spot) in self.watchers.iter().zip(spots) {
            let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
            ground.direct(
                *id,
                vec![
                    Step::Stride { to: spot },
                    Step::FaceX(LINE_X),
                    Step::Beat(watch),
                ],
                now,
            );
        }
        self.events.push(Event::Sides {
            chosen: self.chosen,
        });
        self.phase = Phase::TakingUp { since: now };
    }

    /// One side takes up its end of the rope: the weakest at the front and the strongest at the
    /// back, each little one just in front of its parent, standing along the rope out from the
    /// line, as close together as there is room for and staggered a little if crowded.
    fn take_side(&mut self, ground: &Playground, side: Side, ids: &[Id]) {
        let character = |id: Id| ground.character(id).cloned();
        let mut order: Vec<(Id, Character)> = ids
            .iter()
            .filter_map(|id| character(*id).map(|c| (*id, c)))
            .collect();
        order.sort_by(|a, b| {
            strength(&a.1)
                .total_cmp(&strength(&b.1))
                .then(a.0.cmp(&b.0))
        });
        // A little one just in front of its parent.
        let minis: Vec<(Id, Id)> = order
            .iter()
            .filter_map(|(id, c)| c.parent.filter(|p| ids.contains(p)).map(|p| (*id, p)))
            .collect();
        for (mini, parent) in &minis {
            if let Some(at) = order.iter().position(|(id, _)| id == mini) {
                let moved = order.remove(at);
                let before = order
                    .iter()
                    .position(|(id, _)| id == parent)
                    .unwrap_or(order.len());
                order.insert(before, moved);
            }
        }
        let widths: Vec<f32> = order
            .iter()
            .map(|(id, _)| ground.width(*id).unwrap_or(24.0))
            .collect();
        let room = match side {
            Side::Left => LINE_X - FRONT - REACH.0,
            Side::Right => REACH.1 - LINE_X - FRONT,
        };
        let total: f32 = widths.iter().sum();
        let gaps = (widths.len().max(2) - 1) as f32;
        let gap = ((room - total) / gaps).clamp(-16.0, 2.0);
        let crowded = gap < -6.0;
        let mut along = FRONT;
        for (index, ((id, c), width)) in order.iter().zip(&widths).enumerate() {
            if index > 0 {
                along += gap;
            }
            along += width / 2.0;
            let x = LINE_X + side.sign() * along;
            along += width / 2.0;
            // Crowded, every other one stands a little nearer, so all are seen.
            let y = if crowded && index % 2 == 1 {
                ROPE_Y + 4.0
            } else {
                ROPE_Y
            };
            let tiring = 0.3 * (1.0 - c.axes.energy);
            self.pullers.push(Puller {
                id: *id,
                side,
                home: (x, y),
                strength: strength(c),
                together: 1.0,
                tiring,
                rhythm: self.dice.unit(),
                moments: Vec::new(),
                shown: None,
            });
        }
    }

    /// How each side pulls together: a close pair in rhythm, two who don't get on out of step, a
    /// parent harder beside its little one; and the moments each stops to let the rope go slack,
    /// to wave, or to cheer its side on.
    fn pull_together(&mut self, cast: &Cast) {
        let count = self.pullers.len();
        for a in 0..count {
            for b in a + 1..count {
                let (pa, pb) = (&self.pullers[a], &self.pullers[b]);
                if pa.side != pb.side {
                    continue;
                }
                let (ida, idb) = (pa.id, pb.id);
                if close(cast, ida, idb) {
                    let rhythm = self.pullers[a].rhythm;
                    self.pullers[b].rhythm = rhythm;
                    self.pullers[a].together *= 1.15;
                    self.pullers[b].together *= 1.15;
                    self.remarks.push(Event::InStep { a: ida, b: idb });
                } else if cast.at_odds(ida, idb) {
                    let rhythm = self.pullers[a].rhythm;
                    self.pullers[b].rhythm = (rhythm + 0.5).fract();
                    self.pullers[a].together *= 0.86;
                    self.pullers[b].together *= 0.86;
                    self.remarks.push(Event::OutOfStep { a: ida, b: idb });
                }
            }
        }
        // A parent beside its little one, on the same side.
        for index in 0..count {
            let id = self.pullers[index].id;
            let side = self.pullers[index].side;
            let little = self.pullers.iter().find(|p| {
                p.side == side
                    && cast
                        .member(p.id)
                        .and_then(|member| member.parent())
                        .is_some_and(|parent| parent == id)
            });
            if let Some(little) = little {
                let little = little.id;
                self.pullers[index].together *= 1.25;
                self.remarks.push(Event::Beside { parent: id, little });
            }
        }
        // The moments each stops pulling, as it is.
        for index in 0..count {
            let Some(member) = cast.member(self.pullers[index].id) else {
                continue;
            };
            let character = Character::of(member);
            let mut moments = Vec::new();
            if character.kind == TemperamentKind::Lazybones {
                moments.push((self.dice.range(2.5, 6.0), Moment::Slack));
                if self.dice.chance(0.6) {
                    moments.push((self.dice.range(9.0, 14.0), Moment::Slack));
                }
            }
            if character.kind == TemperamentKind::Showoff {
                moments.push((self.dice.range(3.0, 7.0), Moment::Wave));
            }
            if cheerful(&character) {
                moments.push((self.dice.range(1.5, 4.0), Moment::Cheer));
                if self.dice.chance(0.5) {
                    moments.push((self.dice.range(8.0, 12.0), Moment::Cheer));
                }
            }
            moments.sort_by(|a, b| a.0.total_cmp(&b.0));
            self.pullers[index].moments = moments;
        }
    }

    /// Plays the bout on.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        match self.phase {
            Phase::Ready => {}
            Phase::TakingUp { since } => {
                let all_there = self.pullers.iter().all(|p| {
                    ground.position(p.id).is_some_and(|at| {
                        (at.0 - p.home.0).abs() < 1.0 && (at.1 - p.home.1).abs() < 1.0
                    }) && !ground.walking(p.id)
                });
                if all_there || now - since > TAKING_UP_SECS {
                    for puller in &mut self.pullers {
                        ground.teleport(puller.id, puller.home);
                        puller.shown = None;
                    }
                    self.phase = Phase::Strain {
                        since: now,
                        pull: now + STRAIN_SECS,
                    };
                    self.show(ground, now);
                }
            }
            Phase::Strain { pull, .. } => {
                if now >= pull {
                    self.events.push(Event::Pull);
                    self.last_cut = now;
                    self.phase = Phase::Pulling { since: now };
                }
                self.show(ground, now);
            }
            Phase::Pulling { since } => {
                self.pull(now - since, dt);
                let pulling = now - since;
                let decided = if self.offset.abs() >= WIN {
                    Some(if self.offset > 0.0 {
                        Side::Right
                    } else {
                        Side::Left
                    })
                } else if pulling >= LONGEST {
                    Some(self.ahead())
                } else {
                    None
                };
                self.show(ground, now);
                if let Some(winners) = decided {
                    self.won(ground, winners, now, pulling);
                }
            }
            Phase::Tumbling { since, winners } => {
                let landed = self.tumbles.iter().all(|(id, to)| {
                    ground
                        .position(*id)
                        .is_some_and(|at| (at.0 - to.0).abs() < 1.0 && (at.1 - to.1).abs() < 1.0)
                });
                if (landed && now - since >= TUMBLE_SECS) || now - since >= TUMBLE_SECS * 3.0 {
                    for (id, to) in &self.tumbles {
                        ground.teleport(*id, *to);
                    }
                    self.laugh(ground, winners, now);
                }
            }
            Phase::Over { since, winners } => {
                if now - since >= LAUGH_SECS {
                    self.finish(ground, winners, now);
                }
            }
        }
    }

    /// The side the ribbon is towards, or failing that the side that pulls harder; the left
    /// failing both.
    fn ahead(&self) -> Side {
        if self.offset > 0.0 {
            return Side::Right;
        }
        if self.offset < 0.0 {
            return Side::Left;
        }
        let pull = |side: Side| -> f32 {
            self.pullers
                .iter()
                .filter(|p| p.side == side)
                .map(|p| p.strength * p.together)
                .sum()
        };
        if pull(Side::Right) > pull(Side::Left) {
            Side::Right
        } else {
            Side::Left
        }
    }

    /// How hard each side pulls at `pulling` seconds after "pull!", everyone heaving in their own
    /// rhythm, tiring, stopping for a moment, or heartened by a cheer.
    fn heave(&self, pulling: f32) -> (f32, f32) {
        let (mut left, mut right) = (0.0, 0.0);
        for puller in &self.pullers {
            let heave = (std::f32::consts::TAU * (pulling / HEAVE_SECS + puller.rhythm)).sin();
            let mut pull = puller.strength * puller.together * (0.82 + 0.18 * heave);
            pull *= 1.0 - puller.tiring * (pulling / 40.0).min(1.0);
            match puller.moment(pulling) {
                Some(Moment::Slack | Moment::Wave) => pull = 0.0,
                Some(Moment::Cheer) => pull *= 0.5,
                None => {}
            }
            let side = match puller.side {
                Side::Left => 0,
                Side::Right => 1,
            };
            if self.heartened[side].is_some_and(|at| pulling - at < HEARTENED_SECS) {
                pull *= 1.08;
            }
            match puller.side {
                Side::Left => left += pull,
                Side::Right => right += pull,
            }
        }
        (left, right)
    }

    /// The rope goes the way it is pulled for `dt`, and the side ahead pulls away the longer the
    /// bout goes on. Tells of each slack moment, wave and cheer as it comes, and how each side is
    /// pulling together, a word at a time.
    fn pull(&mut self, pulling: f32, dt: f32) {
        // How each side is pulling together, told one at a time.
        let due = ((pulling - REMARK_FIRST) / REMARK_SECS).floor() as i64 + 1;
        while (self.remarked as i64) < due && self.remarked < self.remarks.len() {
            self.events.push(self.remarks[self.remarked]);
            self.remarked += 1;
        }
        for index in 0..self.pullers.len() {
            let puller = &self.pullers[index];
            let started = puller
                .moments
                .iter()
                .find(|&&(at, _)| at <= pulling && at > pulling - dt)
                .map(|&(_, moment)| moment);
            let (id, side) = (puller.id, puller.side);
            match started {
                Some(Moment::Slack) => self.events.push(Event::Slack { puller: id }),
                Some(Moment::Wave) => self.events.push(Event::Waves { puller: id }),
                Some(Moment::Cheer) => {
                    self.events.push(Event::Cheers { puller: id });
                    let side = match side {
                        Side::Left => 0,
                        Side::Right => 1,
                    };
                    self.heartened[side] = Some(pulling + CHEER_SECS);
                }
                None => {}
            }
        }
        let (left, right) = self.heave(pulling);
        let mut give = GIVE * (right - left) / (left + right).max(0.01);
        let deciding = (pulling / DECIDING_SECS).min(1.0);
        give += DECIDING * self.offset.signum() * deciding * deciding;
        give += self.dice.range(-0.6, 0.6);
        self.offset += give * dt;
    }

    /// One side has drawn the other over: the losers stumble forward and tumble into the hay,
    /// and the winners sit down hard as the rope gives.
    fn won(&mut self, ground: &mut Playground, winners: Side, now: f32, took: f32) {
        self.offset = winners.sign() * self.offset.abs().max(WIN);
        self.shown_offset = self.offset;
        self.show(ground, now);
        self.events.push(Event::Won {
            side: winners,
            took,
        });
        self.tumbles = tumbles(&self.pullers, winners, self.offset);
        for (id, to) in &self.tumbles {
            let stumble = Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.3);
            let mut over = Beat::new(Gesture::Balance, ExpressionKind::Startled, 600.0);
            over.cue = Some(Cue::Exclaim);
            ground.direct(
                *id,
                vec![
                    Step::Beat(stumble),
                    Step::Stride { to: *to },
                    Step::Beat(over),
                ],
                now,
            );
        }
        // The rope gives, and down they sit, hard.
        for winner in self.pullers.iter().filter(|p| p.side == winners) {
            if let Some(at) = ground.position(winner.id) {
                ground.teleport(winner.id, (at.0 + winners.sign() * 3.0, at.1));
            }
            let bump = Beat::new(Gesture::Sit, ExpressionKind::Startled, 0.9);
            let pleased = Beat::new(Gesture::Sit, ExpressionKind::Joy, 600.0);
            ground.direct(winner.id, vec![Step::Beat(bump), Step::Beat(pleased)], now);
        }
        self.phase = Phase::Tumbling {
            since: now,
            winners,
        };
    }

    /// Everyone laughs it off: the losers lying in the hay, laughing, and the winners up and
    /// celebrating, each in its own way; the watchers cheer.
    fn laugh(&mut self, ground: &mut Playground, winners: Side, now: f32) {
        for puller in self.pullers.clone() {
            let Some(c) = ground.character(puller.id).cloned() else {
                continue;
            };
            let beats = if puller.side == winners {
                let mut beats = vec![c.celebrate(1.6)];
                if c.kind == TemperamentKind::Showoff {
                    let mut strut = Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2);
                    strut.cue = Some(Cue::Sparkle);
                    beats.push(strut);
                }
                beats.push(Beat::new(ActionKind::Idle, ExpressionKind::Joy, 600.0));
                beats
            } else {
                ground.tip(puller.id, true);
                match c.kind {
                    // A grump has a huff about it, and can't help a smile after.
                    TemperamentKind::Grump => vec![
                        cued(
                            Beat::new(ActionKind::Idle, ExpressionKind::Grumpy, 1.0),
                            Cue::Huff,
                        ),
                        Beat::new(ActionKind::Idle, ExpressionKind::Smug, 600.0),
                    ],
                    TemperamentKind::Lazybones => vec![
                        cued(
                            Beat::new(ActionKind::Idle, ExpressionKind::Joy, 1.0),
                            Cue::Note,
                        ),
                        cued(
                            Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 600.0),
                            Cue::Sleep,
                        ),
                    ],
                    _ => vec![cued(
                        Beat::new(ActionKind::Idle, ExpressionKind::Joy, 600.0),
                        Cue::Note,
                    )],
                }
            };
            ground.direct(puller.id, beats.into_iter().map(Step::Beat).collect(), now);
        }
        for id in self.watchers.clone() {
            if let Some(c) = ground.character(id).cloned() {
                let cheer = c.celebrate(1.2);
                let watch = Beat::new(Gesture::Watch, ExpressionKind::Joy, 600.0);
                ground.direct(id, vec![Step::Beat(cheer), Step::Beat(watch)], now);
            }
        }
        self.phase = Phase::Over {
            since: now,
            winners,
        };
    }

    /// Up out of the hay, laughing, and everyone back to playing.
    fn finish(&mut self, ground: &mut Playground, winners: Side, now: f32) {
        let sides = self.sides();
        for puller in &self.pullers {
            ground.tip(puller.id, false);
            ground.direct(puller.id, Vec::new(), now);
        }
        for id in &self.watchers {
            ground.direct(*id, Vec::new(), now);
        }
        ground.release();
        self.last = Some((sides.0, sides.1, winners));
        self.pullers.clear();
        self.watchers.clear();
        self.offset = 0.0;
        self.shown_offset = 0.0;
        self.events.push(Event::AllDone);
        self.phase = Phase::Ready;
    }

    /// Calls the bout off: everyone up, and back to playing.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        for puller in &self.pullers {
            ground.tip(puller.id, false);
            ground.direct(puller.id, Vec::new(), now);
        }
        for id in &self.watchers {
            ground.direct(*id, Vec::new(), now);
        }
        ground.release();
        self.pullers.clear();
        self.watchers.clear();
        self.events.clear();
        self.offset = 0.0;
        self.shown_offset = 0.0;
        self.phase = Phase::Ready;
    }

    /// Shows everyone on the rope where it has been pulled to, heaving in their own rhythm (or
    /// stopped for a moment); with motion reduced, a cut to where it is each second, and held
    /// poses.
    fn show(&mut self, ground: &mut Playground, now: f32) {
        let (pulling, straining) = match self.phase {
            Phase::Pulling { since } => (now - since, true),
            Phase::Strain { .. } => (0.0, true),
            _ => return,
        };
        if self.reduce_motion {
            if now - self.last_cut >= CUT_SECS || !matches!(self.phase, Phase::Pulling { .. }) {
                self.last_cut = now;
                self.shown_offset = self.offset;
            }
        } else {
            self.shown_offset = self.offset;
        }
        for index in 0..self.pullers.len() {
            let puller = &self.pullers[index];
            let moment = puller.moment(pulling);
            // Leaning back into each heave, towards its own end of the rope.
            let lean = if self.reduce_motion || moment.is_some() || !straining {
                0.0
            } else {
                let heave = (std::f32::consts::TAU * (pulling / HEAVE_SECS + puller.rhythm)).sin();
                puller.side.sign() * heave.max(0.0)
            };
            let at = (
                (puller.home.0 + self.shown_offset + lean).round(),
                puller.home.1,
            );
            ground.teleport(puller.id, at);
            let (clip, face, cue, facing_line) = match moment {
                Some(Moment::Slack) => (
                    BodyClip::Gesture(Gesture::Yawn),
                    ExpressionKind::Yawning,
                    None,
                    true,
                ),
                Some(Moment::Wave) => (
                    BodyClip::Gesture(Gesture::Strut),
                    ExpressionKind::Smug,
                    Some(Cue::Sparkle),
                    false,
                ),
                Some(Moment::Cheer) => (
                    BodyClip::Gesture(Gesture::Cheer),
                    ExpressionKind::Joy,
                    Some(Cue::Note),
                    true,
                ),
                None => (
                    BodyClip::Gesture(Gesture::Heave),
                    ExpressionKind::Determined,
                    None,
                    true,
                ),
            };
            let look = (clip, face, cue, facing_line);
            if puller.shown != Some(look) {
                let mut beat = Beat::new(clip, face, 600.0);
                beat.cue = cue;
                // Waving, it turns to the crowd behind it.
                let facing = if facing_line {
                    LINE_X
                } else {
                    at.0 + puller.side.sign() * 100.0
                };
                ground.direct(puller.id, vec![Step::FaceX(facing), Step::Beat(beat)], now);
                self.pullers[index].shown = Some(look);
            }
        }
    }

    /// The rope, its ribbon, the chalk line and the hay, while a bout is on: the rope through the
    /// paws of everyone on it, wherever it has been pulled to.
    pub fn gear(&self, ground: &mut Playground, now: f32) -> Vec<Prop> {
        if self.phase == Phase::Ready {
            return Vec::new();
        }
        let mut gear = vec![
            gear::hay(
                (LINE_X.round() as i32, ROPE_Y.round() as i32),
                strewn(&self.pullers).round() as i32,
            ),
            gear::chalk_line(
                (LINE_X.round() as i32 + 2, ROPE_Y as i32 - 9),
                (LINE_X.round() as i32 - 2, ROPE_Y as i32 + 9),
            ),
        ];
        let ribbon = (LINE_X + self.shown_offset).round() as i32;
        let holding = matches!(self.phase, Phase::Strain { .. } | Phase::Pulling { .. });
        let pulling = self.pulling_for(now);
        let paws: Vec<((i32, i32), Side)> = self
            .pullers
            .iter()
            .filter_map(|p| {
                let feet = ground.position(p.id)?;
                let crown = ground.head(p.id, now)?;
                // Let go of, while it waves, and slack in its paws while it lets it go slack.
                let slack = match p.moment(pulling) {
                    Some(Moment::Wave) if holding => return None,
                    Some(Moment::Slack) if holding => 3.0,
                    _ => 0.0,
                };
                let ahead = -p.side.sign();
                Some((
                    (
                        (crown.0 + ahead * 6.0).round() as i32,
                        (crown.1 + (feet.1 - crown.1) * 0.58 + slack).round() as i32,
                    ),
                    p.side,
                ))
            })
            .collect();
        let lying = !holding || paws.is_empty();
        gear.push(gear::rope(
            &paws,
            ribbon,
            ROPE_Y.round() as i32,
            lying,
            (REACH.0.round() as i32 - 8, REACH.1.round() as i32 + 8),
        ));
        gear
    }
}

/// How far the losers are dragged on as they go over, before they tumble.
const DRAGGED: f32 = 10.0;

/// Where each of the losing side tumbles into the hay: dragged on a little further the way the
/// rope went, from wherever it stood on it, so the one at the front goes right over the line and
/// the rest come down along the rope behind it.
fn tumbles(pullers: &[Puller], winners: Side, offset: f32) -> Vec<(Id, (f32, f32))> {
    pullers
        .iter()
        .filter(|p| p.side != winners)
        .map(|p| {
            let x = p.home.0 + offset + winners.sign() * DRAGGED;
            (p.id, (x.round(), p.home.1))
        })
        .collect()
}

/// How far either side of the line the hay is strewn: under the whole of the bigger side, for
/// whichever side ends up in it.
fn strewn(pullers: &[Puller]) -> f32 {
    let furthest = pullers
        .iter()
        .map(|p| (p.home.0 - LINE_X).abs())
        .fold(0.0, f32::max);
    (furthest + 8.0).max(30.0)
}

/// `beat`, with a sign over its head.
fn cued(mut beat: Beat, cue: Cue) -> Beat {
    beat.cue = Some(cue);
    beat
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fairground;
    use formiga_travel::{TravelRole, TravelSnapshot, Traveler};

    const TICK: f32 = 1.0 / 30.0;

    fn snapshot(tune: impl Fn(usize, &mut Traveler)) -> TravelSnapshot {
        let mut snapshot = formiga_travel::sample::snapshot();
        for (index, traveler) in snapshot.travelers.iter_mut().enumerate() {
            tune(index, traveler);
        }
        snapshot
    }

    fn colony(tune: impl Fn(usize, &mut Traveler)) -> Cast {
        Cast::new(snapshot(tune)).unwrap()
    }

    fn sample() -> Cast {
        colony(|_, _| {})
    }

    /// The sample colony's grown-ups, with no little ones among them, all getting on well enough
    /// but none of them close.
    fn grown_ups() -> TravelSnapshot {
        let mut snapshot = snapshot(|_, _| {});
        snapshot.travelers.retain(|t| t.role == TravelRole::Adult);
        for pair in &mut snapshot.relationships {
            pair.affinity = Band::Low;
            pair.avoidance = Band::None;
        }
        snapshot
    }

    /// Sets how two travellers get on.
    fn bond(snapshot: &mut TravelSnapshot, a: usize, b: usize, warmth: Band, friction: Band) {
        let (a, b) = (snapshot.travelers[a].id, snapshot.travelers[b].id);
        for pair in &mut snapshot.relationships {
            if (pair.a == a && pair.b == b) || (pair.a == b && pair.b == a) {
                pair.affinity = warmth;
                pair.avoidance = friction;
            }
        }
    }

    fn characters(cast: &Cast) -> Vec<(Id, Character)> {
        cast.members
            .iter()
            .map(|member| (member.id, Character::of(member)))
            .collect()
    }

    /// The fairground with the colony settled in, and a rope to pull.
    fn ready(cast: &Cast) -> (Playground, TugOfWar) {
        let (mut ground, _) = fairground::open(cast, 0.0);
        let mut now = 0.0;
        while now < 6.0 {
            now += TICK;
            ground.tick(cast, now);
        }
        (ground, TugOfWar::new())
    }

    /// A bout from `from` until everyone is back to playing, or `to`: what happened, and when it
    /// was over.
    fn run(
        ground: &mut Playground,
        tug: &mut TugOfWar,
        cast: &Cast,
        from: f32,
        to: f32,
        mut each: impl FnMut(&mut TugOfWar, &mut Playground, f32),
    ) -> (Vec<Event>, Option<f32>) {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += TICK;
            ground.tick(cast, now);
            tug.tick(ground, now);
            each(tug, ground, now);
            events.extend(tug.take_events());
            if tug.phase() == Phase::Ready {
                return (events, Some(now));
            }
        }
        (events, None)
    }

    fn bout(cast: &Cast, sides: Option<(Vec<Id>, Vec<Id>)>) -> (Vec<Event>, TugOfWar) {
        let (mut ground, mut tug) = ready(cast);
        tug.start(&mut ground, cast, &[], sides, 6.0);
        let (events, over) = run(&mut ground, &mut tug, cast, 6.0, 300.0, |_, _, _| {});
        assert!(over.is_some(), "the bout never ended: {events:?}");
        (events, tug)
    }

    fn won(events: &[Event]) -> Option<Side> {
        events.iter().find_map(|event| match event {
            Event::Won { side, .. } => Some(*side),
            _ => None,
        })
    }

    #[test]
    fn the_colony_sorts_itself_into_even_sides_keeping_families_together() {
        let cast = sample();
        let (left, right) = sort(&cast, &characters(&cast));
        assert_eq!(left.len() + right.len(), cast.members.len());
        assert!(left.len().abs_diff(right.len()) <= 1, "{left:?} {right:?}");
        // A little one is always on its parent's side.
        for member in &cast.members {
            if let Some(parent) = member.parent() {
                assert_eq!(
                    left.contains(&member.id),
                    left.contains(&parent),
                    "{} was sorted away from its parent",
                    member.name
                );
            }
        }
        // And the strength is about even.
        let pulls = |ids: &[Id]| -> f32 {
            ids.iter()
                .map(|id| strength(&Character::of(cast.member(*id).unwrap())))
                .sum()
        };
        let (l, r) = (pulls(&left), pulls(&right));
        assert!((l - r).abs() / (l + r) < 0.2, "{l} against {r}");
    }

    #[test]
    fn rivals_are_sorted_onto_opposite_sides_and_close_friends_onto_the_same_one() {
        let mut snapshot = grown_ups();
        bond(&mut snapshot, 0, 1, Band::Low, Band::High);
        bond(&mut snapshot, 2, 3, Band::High, Band::None);
        let cast = Cast::new(snapshot).unwrap();
        let ids: Vec<Id> = cast.ids().collect();
        assert!(cast.at_odds(ids[0], ids[1]));
        assert!(close(&cast, ids[2], ids[3]));
        let (left, _) = sort(&cast, &characters(&cast));
        let together = |a: Id, b: Id| left.contains(&a) == left.contains(&b);
        assert!(!together(ids[0], ids[1]), "the rivals were put together");
        assert!(together(ids[2], ids[3]), "the friends were split up");
        // Swap who is close and who is at odds, and the sides change with them.
        let mut snapshot = grown_ups();
        bond(&mut snapshot, 0, 1, Band::High, Band::None);
        bond(&mut snapshot, 2, 3, Band::Low, Band::High);
        let swapped = Cast::new(snapshot).unwrap();
        let (left, _) = sort(&swapped, &characters(&swapped));
        let together = |a: Id, b: Id| left.contains(&a) == left.contains(&b);
        assert!(together(ids[0], ids[1]));
        assert!(!together(ids[2], ids[3]));
    }

    #[test]
    fn the_persons_own_sides_are_pulled_as_picked() {
        let cast = sample();
        let ids: Vec<Id> = cast.ids().collect();
        // Lopsided, and not as the colony would sort itself: one against three.
        let picked = (vec![ids[3]], vec![ids[0], ids[1], ids[4]]);
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], Some(picked.clone()), 6.0);
        let sorted = |mut ids: Vec<Id>| {
            ids.sort_unstable();
            ids
        };
        let (left, right) = tug.sides();
        assert_eq!(
            (sorted(left), sorted(right)),
            (sorted(picked.0), sorted(picked.1))
        );
        let (events, over) = run(&mut ground, &mut tug, &cast, 6.0, 300.0, |_, _, _| {});
        assert!(over.is_some());
        assert_eq!(events.first(), Some(&Event::Sides { chosen: true }));
        // Whoever was left out watched.
        let results = tug.results();
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|(id, _)| *id != ids[2] && *id != ids[5]));
        // Sides with nobody on one are no sides: the colony sorts itself instead.
        let (mut ground, mut tug) = ready(&cast);
        tug.start(
            &mut ground,
            &cast,
            &[],
            Some((vec![ids[0]], Vec::new())),
            6.0,
        );
        let (left, right) = tug.sides();
        assert_eq!(left.len() + right.len(), cast.members.len());
        assert!(tug.take_events().contains(&Event::Sides { chosen: false }));
    }

    #[test]
    fn the_ribbon_drifts_until_one_side_draws_the_other_over_the_line() {
        let cast = sample();
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], None, 6.0);
        let (mut ribbon, mut over_the_line) = (Vec::new(), None);
        let (events, over) = run(
            &mut ground,
            &mut tug,
            &cast,
            6.0,
            300.0,
            |tug, _, _| match tug.phase() {
                Phase::Pulling { .. } => ribbon.push(tug.ribbon()),
                Phase::Tumbling { .. } if over_the_line.is_none() => {
                    over_the_line = Some(tug.ribbon());
                }
                _ => {}
            },
        );
        assert!(over.is_some());
        assert!(ribbon.len() > 30, "the pull was over at once");
        let moved = ribbon.windows(2).filter(|pair| pair[0] != pair[1]).count();
        assert!(moved > ribbon.len() / 2, "the ribbon hardly moved");
        assert!(
            ribbon.iter().all(|at| at.abs() < WIN),
            "it was over the line mid-pull"
        );
        let side = won(&events).expect("nobody won");
        let last = over_the_line.expect("nobody was drawn over the line");
        assert!(last.abs() >= WIN, "it was won short of the line");
        assert_eq!(side, if last > 0.0 { Side::Right } else { Side::Left });
        assert!(events.contains(&Event::Pull));
        assert_eq!(events.last(), Some(&Event::AllDone));
    }

    #[test]
    fn every_bout_ends_however_the_colony_is_and_everyone_has_a_result() {
        let tunings: [fn(usize, &mut Traveler); 4] = [
            |_, t| t.character.temperament = TemperamentKind::Lazybones.into(),
            |_, t| t.character.temperament = TemperamentKind::Showoff.into(),
            // Everyone alike, so neither side is any stronger.
            |_, t| {
                t.character.axes.feistiness = 0.5;
                t.character.axes.energy = 0.5;
                t.stature_percent = 100;
                t.scale_percent = 100;
            },
            |_, t| {
                t.character.temperament = TemperamentKind::Sweetheart.into();
                t.character.axes.affection = 1.0;
            },
        ];
        for tune in tunings {
            let cast = colony(tune);
            let (events, tug) = bout(&cast, None);
            assert!(won(&events).is_some(), "{events:?}");
            let results = tug.results();
            assert_eq!(results.len(), cast.members.len());
            let winners = results.iter().filter(|(_, won)| *won).count();
            assert!(winners > 0 && winners < results.len());
        }
        let big = crate::fairground::testing::colony_of(12);
        let (events, tug) = bout(&big, None);
        assert!(won(&events).is_some());
        assert_eq!(tug.results().len(), 12);
    }

    #[test]
    fn the_stronger_side_mostly_wins() {
        // Three big, feisty, lively ones against three small, gentle, sleepy ones.
        let cast = colony(|i, t| {
            let strong = i < 3;
            t.character.axes.feistiness = if strong { 0.95 } else { 0.1 };
            t.character.axes.energy = if strong { 0.95 } else { 0.1 };
            t.character.axes.affection = 0.3;
            t.character.temperament = TemperamentKind::Guardian.into();
            t.stature_percent = if strong { 115 } else { 90 };
        });
        let ids: Vec<Id> = cast.ids().collect();
        let sides = (ids[..3].to_vec(), ids[3..].to_vec());
        let (mut ground, mut tug) = ready(&cast);
        let mut strong_won = 0;
        for round in 0..5 {
            let from = 6.0 + round as f32 * 200.0;
            tug.start(&mut ground, &cast, &[], Some(sides.clone()), from);
            let (events, over) = run(
                &mut ground,
                &mut tug,
                &cast,
                from,
                from + 200.0,
                |_, _, _| {},
            );
            assert!(over.is_some());
            if won(&events) == Some(Side::Left) {
                strong_won += 1;
            }
        }
        assert!(
            strong_won >= 4,
            "the strong side won only {strong_won} of 5"
        );
    }

    #[test]
    fn a_close_pair_pulls_in_rhythm_and_two_who_dont_get_on_pull_out_of_step() {
        let mut snapshot = grown_ups();
        bond(&mut snapshot, 0, 1, Band::High, Band::None);
        bond(&mut snapshot, 2, 3, Band::Low, Band::High);
        let cast = Cast::new(snapshot).unwrap();
        let ids: Vec<Id> = cast.ids().collect();
        // The close pair on one side, the pair at odds on the other.
        let sides = (vec![ids[0], ids[1]], vec![ids[2], ids[3]]);
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], Some(sides), 6.0);
        let puller = |id: Id| tug.pullers.iter().find(|p| p.id == id).unwrap().clone();
        let (a, b, c, d) = (
            puller(ids[0]),
            puller(ids[1]),
            puller(ids[2]),
            puller(ids[3]),
        );
        assert_eq!(a.rhythm, b.rhythm, "the close pair heave together");
        assert!(a.together > 1.0 && b.together > 1.0);
        assert!(
            ((c.rhythm - d.rhythm).abs() - 0.5).abs() < 1e-4,
            "out of step"
        );
        assert!(c.together < 1.0 && d.together < 1.0);
        let (events, _) = run(&mut ground, &mut tug, &cast, 6.0, 300.0, |_, _, _| {});
        assert!(events.contains(&Event::InStep {
            a: ids[0],
            b: ids[1]
        }));
        assert!(events.contains(&Event::OutOfStep {
            a: ids[2],
            b: ids[3]
        }));
    }

    #[test]
    fn a_lazybones_goes_slack_a_show_off_waves_and_an_affectionate_one_cheers_its_side_on() {
        let cast = colony(|i, t| {
            t.character.axes.affection = 0.3;
            t.character.temperament = match i {
                0 => TemperamentKind::Lazybones.into(),
                1 => TemperamentKind::Showoff.into(),
                _ => TemperamentKind::Guardian.into(),
            };
            if i == 2 {
                t.character.axes.affection = 1.0;
            }
        });
        let ids: Vec<Id> = cast.ids().collect();
        let sides = (vec![ids[0], ids[3], ids[4]], vec![ids[1], ids[2], ids[5]]);
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], Some(sides), 6.0);
        // Evenly matched enough to go on long enough for all of it.
        for puller in &mut tug.pullers {
            puller.strength = 0.5;
            puller.together = 1.0;
        }
        let mut slack = None;
        let (events, _) = run(&mut ground, &mut tug, &cast, 6.0, 300.0, |tug, _, now| {
            let pulling = tug.pulling_for(now);
            let lazy = tug.pullers.iter().find(|p| p.id == ids[0]);
            if slack.is_none()
                && let Some(lazy) = lazy
                && lazy.moment(pulling) == Some(Moment::Slack)
            {
                let (left, right) = tug.heave(pulling);
                slack = Some((left, right));
            }
        });
        assert!(
            events.contains(&Event::Slack { puller: ids[0] }),
            "{events:?}"
        );
        assert!(events.contains(&Event::Waves { puller: ids[1] }));
        assert!(events.contains(&Event::Cheers { puller: ids[2] }));
        let others = events
            .iter()
            .filter(|event| match event {
                Event::Slack { puller } => *puller != ids[0],
                Event::Waves { puller } => *puller != ids[1],
                Event::Cheers { puller } => *puller != ids[2],
                _ => false,
            })
            .count();
        assert_eq!(others, 0, "nobody else does any of it: {events:?}");
        // While the lazybones has the rope slack, its side is pulling with one less.
        let (left, right) = slack.expect("it never went slack mid-pull");
        assert!(left < right, "{left} against {right}");
    }

    #[test]
    fn a_parent_stands_beside_its_little_one_and_pulls_all_the_harder() {
        let cast = sample();
        let little = cast.members.iter().find(|m| m.parent().is_some()).unwrap();
        let (mini, parent) = (little.id, little.parent().unwrap());
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], None, 6.0);
        let puller = |id: Id| tug.pullers.iter().find(|p| p.id == id).unwrap().clone();
        assert_eq!(puller(mini).side, puller(parent).side);
        // Next to each other along the rope, the little one in front.
        let on_side: Vec<Id> = tug
            .pullers
            .iter()
            .filter(|p| p.side == puller(mini).side)
            .map(|p| p.id)
            .collect();
        let at = |id: Id| on_side.iter().position(|own| *own == id).unwrap();
        assert_eq!(at(parent), at(mini) + 1);
        let unaided = strength(&Character::of(cast.member(parent).unwrap()));
        let helped = puller(parent);
        assert!(helped.strength * helped.together > unaided * 1.2);
        let (events, _) = run(&mut ground, &mut tug, &cast, 6.0, 300.0, |_, _, _| {});
        assert!(events.contains(&Event::Beside {
            parent,
            little: mini
        }));
    }

    #[test]
    fn the_losers_tumble_softly_into_the_hay_and_everyone_laughs_it_off() {
        let cast = sample();
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], None, 6.0);
        let mut lying = Vec::new();
        let (events, over) = run(
            &mut ground,
            &mut tug,
            &cast,
            6.0,
            300.0,
            |tug, ground, _| {
                let Phase::Over { winners, .. } = tug.phase() else {
                    return;
                };
                if !lying.is_empty() {
                    return;
                }
                for puller in &tug.pullers {
                    let down = ground.tipped(puller.id).unwrap();
                    assert_eq!(down, puller.side != winners, "{}", puller.id);
                    if down {
                        let (x, _) = ground.position(puller.id).unwrap();
                        assert!(
                            (x - LINE_X).abs() <= strewn(&tug.pullers),
                            "it tumbled clear of the hay"
                        );
                        lying.push(puller.id);
                    }
                }
            },
        );
        assert!(over.is_some(), "{events:?}");
        assert!(!lying.is_empty(), "nobody tumbled");
        // And up again after, back to playing.
        for id in lying {
            assert_eq!(ground.tipped(id), Some(false));
        }
    }

    #[test]
    fn the_same_colony_with_other_temperaments_pulls_differently() {
        let others = formiga_travel::sample::snapshot();
        let swapped = colony(|i, t| {
            let other = &others.travelers[(i + 2) % others.travelers.len()];
            t.character = other.character.clone();
            t.motion = other.motion;
        });
        let (first, _) = bout(&sample(), None);
        let (second, _) = bout(&swapped, None);
        assert_ne!(first, second);
    }

    #[test]
    fn with_reduced_motion_the_rope_cuts_from_moment_to_moment() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], None, 6.0);
        let (mut ticks, mut moves) = (0, 0);
        let mut last: Vec<(f32, f32)> = Vec::new();
        let (_, over) = run(
            &mut ground,
            &mut tug,
            &cast,
            6.0,
            300.0,
            |tug, ground, _| {
                if !matches!(tug.phase(), Phase::Pulling { .. }) {
                    return;
                }
                let now: Vec<(f32, f32)> = tug
                    .pullers
                    .iter()
                    .map(|p| ground.position(p.id).unwrap())
                    .collect();
                ticks += 1;
                if !last.is_empty() && now != last {
                    moves += 1;
                }
                last = now;
            },
        );
        assert!(over.is_some());
        assert!(moves * 8 < ticks, "{moves} moves in {ticks} ticks");
        assert!(moves > 2, "the rope never moved");
    }

    #[test]
    fn calling_it_off_lets_everyone_up_and_go() {
        let cast = sample();
        let (mut ground, mut tug) = ready(&cast);
        tug.start(&mut ground, &cast, &[], None, 6.0);
        run(&mut ground, &mut tug, &cast, 6.0, 22.0, |_, _, _| {});
        assert_ne!(tug.phase(), Phase::Ready);
        tug.stop(&mut ground, 22.0);
        assert_eq!(tug.phase(), Phase::Ready);
        assert!(tug.gear(&mut ground, 22.0).is_empty());
        let before: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        let mut now = 22.0;
        while now < 50.0 {
            now += TICK;
            ground.tick(&cast, now);
        }
        let after: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        assert_ne!(before, after, "nobody moved off");
    }
}
