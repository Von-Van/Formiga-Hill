//! The sack race, run by the colony while the person watches. The racers line up at a chalk line
//! across the front of the fairground, climb into their sacks and hop for the ribbon strung
//! between two posts at the far end; whoever is not racing watches from beside the finish.
//!
//! Nobody is written in. How far and how fast each one hops comes from its pace, its energy and
//! its size, so a little one hops small. Everything else comes from who it is: an impulsive one
//! may jump the start and send everyone back to the line, and tumbles more often on the way (and
//! always gets up again, laughing); a lazybones may sit down for a breather; a show-off stops to
//! wave to the crowd just short of the line, and may be overtaken doing it; an affectionate one
//! stops to help a friend up who has tumbled; a parent keeps beside its little one; and rivals
//! glare at each other as one passes the other. Nobody loses anything by coming last.

use super::gear;
use super::{LINE_GAP, Path, line_up, sizes};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::{Playground, Prop};
use formiga_art::{BodyClip, ExpressionKind};
use formiga_core::{ActionKind, Gesture, TemperamentKind};

/// The course runs across the clear sawdust at the front of the fairground, its lanes from the
/// furthest back to the nearest the front, at most `LANE_GAP` apart. The start line and the finish
/// run across it on a slant, parallel, so every lane is as long as the next and nobody stands
/// square behind anyone else: the back lane starts at `START_X`, and each lane nearer the front a
/// little further back, so whoever is in front stands down and to the left of its neighbour,
/// over its tail and never its face (they all face the way they hop). The front lane starts clear
/// of the straw bale, and the back lane finishes clear of the stall and the handcart.
const START_X: f32 = 218.0;
const SLANT: f32 = 1.75;
pub const COURSE: f32 = 136.0;
const LANES: (f32, f32) = (146.0, 206.0);
const LANE_GAP: f32 = 12.0;
/// How far behind the line a racer stands to start.
const TOE: f32 = 8.0;
/// Where those not racing watch from: along the back of the course, the nearest the finish first,
/// each a little apart from the next.
const CROWD: [Path; 1] = [&[(338.0, 132.0), (144.0, 132.0)]];
/// "Ready, steady…", and "go!" after this long.
const STEADY_SECS: f32 = 1.8;
/// The longest the racers are given to get to the line.
const LINING_UP_SECS: f32 = 14.0;
const TUMBLE_SECS: f32 = 1.6;
const BREATHER_SECS: f32 = 2.8;
const WAVE_SECS: f32 = 1.9;
const HELP_SECS: f32 = 1.1;
const GLARE_SECS: f32 = 0.9;
/// How long a jumper stands embarrassed before shuffling back to the line.
const SHEEPISH_SECS: f32 = 0.9;
/// Nobody tumbles more than this many times in a race.
const MOST_TUMBLES: u32 = 2;
/// How long the lead has to be held before anyone says so.
const LEAD_HELD_SECS: f32 = 0.35;
/// How long the colony cheers before going back to playing.
const CELEBRATE_SECS: f32 = 4.5;
/// With motion reduced the race is shown as a run of moments, a cut between each.
const CUT_SECS: f32 = 1.0;
/// However it goes, the race is over by this long after "go": anyone still on the course is
/// waved over the line. (Nobody hops so slowly; it is only a promise.)
const LONGEST: f32 = 150.0;

/// Where the start line crosses row `y`.
fn start_at(y: f32) -> f32 {
    START_X - (y - LANES.0) * SLANT
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Nobody is racing.
    Ready,
    /// The racers are on their way to the line, and climbing into their sacks.
    LiningUp {
        since: f32,
    },
    /// "Ready, steady…": "go!" comes at `go`.
    Steady {
        since: f32,
        go: f32,
    },
    Racing {
        since: f32,
    },
    /// Everyone is over the line, and the colony is cheering.
    Over {
        since: f32,
    },
}

/// Something that happened in the race, for the person watching to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Steady,
    /// Off before "go"; everyone goes back to the line.
    FalseStart {
        racer: Id,
    },
    Go,
    /// The lead changed hands.
    Lead {
        racer: Id,
    },
    Tumble {
        racer: Id,
    },
    Breather {
        racer: Id,
    },
    Waving {
        racer: Id,
    },
    HelpedUp {
        helper: Id,
        racer: Id,
    },
    Glare {
        racer: Id,
        rival: Id,
    },
    Finished {
        racer: Id,
        place: usize,
        took: f32,
    },
    /// Everyone is over the line; `took` is the winner's time.
    AllHome {
        took: f32,
    },
}

/// How a racer hops: how far, how long in the air, how long it crouches between, and how high.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gait {
    pub length: f32,
    pub air: f32,
    pub crouch: f32,
    pub height: f32,
}

impl Gait {
    /// From pace, energy and size: a lively one hops quick and far, a sleepy one slow and short,
    /// and a little one small.
    pub fn of(character: &Character) -> Self {
        let a = character.axes;
        let pace = character.pace.clamp(0.0, 1.0);
        let scale = 0.4 + 0.6 * character.size.clamp(0.3, 1.3);
        let length = (6.0 + 5.0 * a.energy + 3.0 * pace) * scale;
        Self {
            length,
            air: 0.3 + 0.16 * (1.0 - pace),
            crouch: 0.1 + 0.22 * (1.0 - a.energy),
            height: (length * 0.55).max(3.0),
        }
    }

    /// Pixels a second, hopping steadily.
    pub fn speed(self) -> f32 {
        self.length / (self.air + self.crouch)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Doing {
    /// On its way to the line.
    Lining,
    AtTheLine,
    Hopping,
    /// Off before "go": a hop or two on, then back to the line, sheepish.
    Jumped {
        back: bool,
    },
    Tumbled {
        until: f32,
    },
    Resting {
        until: f32,
    },
    Waving {
        until: f32,
    },
    /// Off to help `friend` up: going over to it, then hauling it up until `until`.
    Helping {
        friend: Id,
        until: Option<f32>,
    },
    /// Waiting for its little one, or its parent, to be up and hopping again.
    Minding,
    /// Over the line, hopping on to a stop.
    Home,
}

/// What a racer is shown doing, so it is only told again when that changes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    clip: BodyClip,
    face: ExpressionKind,
    cue: Option<Cue>,
    tipped: bool,
    facing_right: bool,
}

#[derive(Clone, Debug)]
struct Racer {
    id: Id,
    lane: f32,
    band: u8,
    /// Where its start line is, and how far along the course it is from there.
    start: f32,
    along: f32,
    gait: Gait,
    /// Where along the hop under way set off from, how far it goes, and when it began. A hop of
    /// no length is a crouch, waiting.
    from: f32,
    length: f32,
    since: f32,
    doing: Doing,
    /// When it started doing what it is doing.
    doing_since: f32,
    /// Read once from its character.
    impulsive: f32,
    face: ExpressionKind,
    sits_at: Option<f32>,
    waves_at: Option<f32>,
    kind_hearted: bool,
    friends: Vec<Id>,
    rivals: Vec<Id>,
    /// Its little one, or its parent, racing too.
    little_one: Option<Id>,
    parent: Option<Id>,
    tumbles: u32,
    helped: bool,
    glaring_until: f32,
    finished: Option<f32>,
    /// How far along it stops, past the line.
    stop: f32,
    shown: Option<Look>,
    /// How far along it is shown: with motion reduced, only where it was at the last cut.
    shown_along: f32,
    /// Hops landed so far this race, and whether the one under way has landed yet.
    landings: u32,
    landed: bool,
}

impl Racer {
    fn racing(&self) -> bool {
        self.finished.is_none()
    }

    /// Stopped for the moment: down, sat, waving, or helping someone up.
    fn held_up(&self) -> bool {
        self.racing()
            && matches!(
                self.doing,
                Doing::Tumbled { .. }
                    | Doing::Resting { .. }
                    | Doing::Waving { .. }
                    | Doing::Helping { .. }
            )
    }

    /// Its family in the race: its little one, or its parent.
    fn family(&self) -> Option<Id> {
        self.little_one.or(self.parent)
    }

    fn x(&self, along: f32) -> f32 {
        self.start + along
    }

    /// Carries a hop on: crouched a moment, then up and along. Says whether it has landed.
    fn hop(&mut self, now: f32) -> bool {
        let t = now - self.since - self.gait.crouch;
        if t < 0.0 {
            return false;
        }
        if self.length <= 0.0 {
            return true;
        }
        let through = (t / self.gait.air).min(1.0);
        self.along = self.from + self.length * through;
        if through >= 1.0 && !self.landed {
            self.landed = true;
            self.landings += 1;
        }
        through >= 1.0
    }

    fn next_hop(&mut self, now: f32, length: f32) {
        self.from = self.along;
        self.length = length.max(0.0);
        self.since = now;
        self.landed = false;
    }

    /// How far into the air part of its hop it is, from 0 to 1, if it is in the air.
    fn airborne(&self, now: f32) -> Option<f32> {
        let t = now - self.since - self.gait.crouch;
        (self.length > 0.0 && t >= 0.0 && t < self.gait.air).then(|| t / self.gait.air)
    }
}

pub struct SackRace {
    phase: Phase,
    racers: Vec<Racer>,
    crowd: Vec<Id>,
    events: Vec<Event>,
    round: u64,
    dice: Dice,
    false_started: bool,
    /// Whoever jumps the start this time round, and when.
    jumps: Vec<(usize, f32)>,
    /// Who is in front and since when, and who was last said to be.
    front: Option<(Id, f32)>,
    announced: Option<Id>,
    /// Rivals who have glared at each other already this race.
    glared: Vec<(Id, Id)>,
    broken: Option<f32>,
    /// Whether the crowd has something to cheer.
    cheer: bool,
    last_cut: f32,
    reduce_motion: bool,
    /// The order the race finished in, so far or last time, with each one's time.
    results: Vec<(Id, f32)>,
    /// When it was last ticked, and how long that tick was: whatever is not a hop moves by the
    /// second, not by the frame, so it goes the same however often it is drawn.
    clock: f32,
    dt: f32,
}

impl Default for SackRace {
    fn default() -> Self {
        Self::new()
    }
}

impl SackRace {
    pub fn new() -> Self {
        Self {
            phase: Phase::Ready,
            racers: Vec::new(),
            crowd: Vec::new(),
            events: Vec::new(),
            round: 0,
            dice: Dice::new(1),
            false_started: false,
            jumps: Vec::new(),
            front: None,
            announced: None,
            glared: Vec::new(),
            broken: None,
            cheer: false,
            last_cut: 0.0,
            reduce_motion: false,
            results: Vec::new(),
            clock: 0.0,
            dt: 0.0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// What has happened since last asked.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Who is in front, once the race is on.
    pub fn leader(&self) -> Option<Id> {
        match self.phase {
            Phase::Racing { .. } | Phase::Over { .. } => self.announced,
            _ => None,
        }
    }

    /// The order the race finished in, so far or last time, with each one's time.
    pub fn results(&self) -> &[(Id, f32)] {
        &self.results
    }

    /// How many hops have landed this race, everyone's together, for each to be heard.
    pub fn hops(&self) -> u32 {
        self.racers.iter().map(|racer| racer.landings).sum()
    }

    /// Seconds since "go".
    pub fn racing_for(&self, now: f32) -> f32 {
        match self.phase {
            Phase::Racing { since } => now - since,
            _ => 0.0,
        }
    }

    /// Starts a race between `racers`, or everyone if fewer than two are named. The rest watch.
    pub fn start(&mut self, ground: &mut Playground, cast: &Cast, racers: &[Id], now: f32) {
        let everyone = ground.ids();
        let mut chosen: Vec<Id> = racers
            .iter()
            .copied()
            .filter(|id| everyone.contains(id))
            .collect();
        if chosen.len() < 2 {
            chosen.clone_from(&everyone);
        }
        if chosen.len() < 2 {
            return;
        }
        self.round += 1;
        self.clock = now;
        self.dice = Dice::new(
            self.round.wrapping_mul(0x5ac4_2ace) ^ chosen.iter().fold(7, |seed, id| seed ^ id),
        );
        self.reduce_motion = ground.reduce_motion();
        self.events.clear();
        self.false_started = false;
        self.jumps.clear();
        self.front = None;
        self.announced = None;
        self.glared.clear();
        self.broken = None;
        self.cheer = false;
        self.results.clear();

        // The tallest at the back, so nobody is lost behind someone taller; and a parent comes
        // forward to the lane just behind its little ones, so they race side by side.
        let mut order: Vec<(Id, Character)> = chosen
            .iter()
            .filter_map(|id| ground.character(*id).map(|c| (*id, c.clone())))
            .collect();
        let tall = |id: Id| ground.height(id).unwrap_or(0.0);
        let wide = |id: Id| ground.width(id).unwrap_or(0.0);
        order.sort_by(|a, b| {
            tall(b.0)
                .total_cmp(&tall(a.0))
                .then(wide(b.0).total_cmp(&wide(a.0)))
                .then(a.0.cmp(&b.0))
        });
        let minis: Vec<(Id, Id)> = order
            .iter()
            .filter_map(|(id, c)| c.parent.filter(|p| chosen.contains(p)).map(|p| (*id, p)))
            .collect();
        let mut parents: Vec<Id> = Vec::new();
        for (_, parent) in &minis {
            if !parents.contains(parent) {
                parents.push(*parent);
            }
        }
        for parent in parents {
            let family = |id: &Id| *id == parent || minis.contains(&(*id, parent));
            let Some(first_mini) = order
                .iter()
                .position(|(id, _)| minis.contains(&(*id, parent)))
            else {
                continue;
            };
            let at = order[..first_mini]
                .iter()
                .filter(|(id, _)| !family(id))
                .count();
            let (mut moving, rest): (Vec<_>, Vec<_>) =
                order.into_iter().partition(|(id, _)| family(id));
            // The parent first, then its little ones in the order they had.
            moving.sort_by_key(|(id, _)| *id != parent);
            order = rest;
            order.splice(at..at, moving);
        }
        let count = order.len();
        let gap = ((LANES.1 - LANES.0) / (count - 1).max(1) as f32).min(LANE_GAP);
        let first = (LANES.0 + LANES.1) / 2.0 - gap * (count - 1) as f32 / 2.0;
        let ids: Vec<Id> = order.iter().map(|(id, _)| *id).collect();
        let dice = &mut self.dice;
        self.racers = order
            .iter()
            .enumerate()
            .map(|(index, (id, c))| {
                let a = c.axes;
                let lane = first + gap * index as f32;
                let showy = c.kind == TemperamentKind::Showoff || a.boldness > 0.82;
                let lazy = c.kind == TemperamentKind::Lazybones
                    && dice.chance(0.55 + 0.45 * (1.0 - a.energy));
                let little_one = minis
                    .iter()
                    .find(|(_, parent)| parent == id)
                    .map(|(mini, _)| *mini);
                let friends = ids
                    .iter()
                    .copied()
                    .filter(|other| {
                        other != id
                            && cast.bond(*id, *other).is_some_and(|bond| {
                                bond.warmth >= formiga_travel::Band::Medium
                                    && bond.friction < bond.warmth
                            })
                    })
                    .collect();
                let rivals = ids
                    .iter()
                    .copied()
                    .filter(|other| other != id && cast.at_odds(*id, *other))
                    .collect();
                Racer {
                    id: *id,
                    lane,
                    band: index as u8,
                    start: start_at(lane),
                    along: -TOE,
                    gait: Gait::of(c),
                    from: -TOE,
                    length: 0.0,
                    since: now,
                    doing: Doing::Lining,
                    doing_since: now,
                    impulsive: a.impulsiveness,
                    face: race_face(c),
                    // A parent minding its little one has no time for breathers or waving.
                    sits_at: (lazy && little_one.is_none())
                        .then(|| COURSE * dice.range(0.35, 0.65)),
                    waves_at: (showy && little_one.is_none())
                        .then(|| COURSE - dice.range(14.0, 24.0)),
                    kind_hearted: a.affection >= 0.7 || c.kind == TemperamentKind::Sweetheart,
                    friends,
                    rivals,
                    little_one,
                    parent: c.parent.filter(|parent| chosen.contains(parent)),
                    tumbles: 0,
                    helped: false,
                    glaring_until: 0.0,
                    finished: None,
                    stop: COURSE + 10.0 + (index % 2) as f32 * 5.0,
                    shown: None,
                    shown_along: -TOE,
                    landings: 0,
                    landed: false,
                }
            })
            .collect();
        // A parent hops in step with its little one.
        for index in 0..self.racers.len() {
            if let Some(gait) = self.racers[index]
                .little_one
                .and_then(|little| self.racer(little))
                .map(|little| little.gait)
            {
                self.racers[index].gait = gait;
            }
        }

        self.crowd = everyone
            .iter()
            .copied()
            .filter(|id| !chosen.contains(id))
            .collect();
        let mut reserved = ids;
        reserved.extend(&self.crowd);
        ground.reserve(reserved);
        for racer in &self.racers {
            let spot = (racer.x(-TOE), racer.lane);
            ground.direct(racer.id, vec![Step::Stride { to: spot }], now);
        }
        let spots = line_up(&CROWD, &sizes(ground, &self.crowd), LINE_GAP);
        // Each turned towards the middle of the course.
        let middle = start_at((LANES.0 + LANES.1) / 2.0) + COURSE / 2.0;
        for (id, spot) in self.crowd.iter().zip(spots) {
            let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
            let steps = vec![
                Step::Stride { to: spot },
                Step::FaceX(middle),
                Step::Beat(watch),
            ];
            ground.direct(*id, steps, now);
        }
        self.phase = Phase::LiningUp { since: now };
    }

    fn racer(&self, id: Id) -> Option<&Racer> {
        self.racers.iter().find(|racer| racer.id == id)
    }

    fn index(&self, id: Id) -> Option<usize> {
        self.racers.iter().position(|racer| racer.id == id)
    }

    /// Plays the race on.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        self.dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        match self.phase {
            Phase::Ready => return,
            Phase::LiningUp { since } => self.line_up(ground, now, since),
            Phase::Steady { go, .. } => self.steady(now, go),
            Phase::Racing { since } => self.race(now, since),
            Phase::Over { since } => {
                if now - since >= CELEBRATE_SECS {
                    // The racers' poses past the line last only as long as the cheering.
                    for racer in &self.racers {
                        ground.direct(racer.id, Vec::new(), now);
                    }
                    self.finish(ground, now);
                    return;
                }
            }
        }
        if !matches!(self.phase, Phase::LiningUp { .. } | Phase::Ready) {
            self.show(ground, now);
        }
    }

    /// The gear set out on the course while a race is on: the chalk line, the posts and the
    /// ribbon.
    pub fn gear(&self, now: f32) -> Vec<Prop> {
        if self.phase == Phase::Ready {
            return Vec::new();
        }
        let (back, front) = (LANES.0 - 4.0, LANES.1 + 3.0);
        let finish = |y: f32| ((start_at(y) + COURSE).round() as i32, y as i32);
        let (back_post, front_post) = (finish(back), finish(front));
        vec![
            gear::chalk_line(
                (start_at(back).round() as i32, back as i32),
                (start_at(front).round() as i32, front as i32),
            ),
            gear::post(back_post.0, back_post.1),
            gear::post(front_post.0, front_post.1),
            gear::ribbon(
                back_post,
                front_post,
                self.broken.map(|at| now - at),
                self.reduce_motion,
            ),
        ]
    }

    /// Up to the line, and into the sacks as each one gets there.
    fn line_up(&mut self, ground: &mut Playground, now: f32, since: f32) {
        let late = now - since > LINING_UP_SECS;
        for racer in &mut self.racers {
            if racer.doing != Doing::Lining {
                continue;
            }
            let spot = (racer.x(-TOE), racer.lane);
            let there = ground
                .position(racer.id)
                .is_some_and(|at| (at.0 - spot.0).abs() < 1.0 && (at.1 - spot.1).abs() < 1.0);
            if there || late {
                ground.teleport(racer.id, spot);
                ground.sack(racer.id, Some(racer.band));
                let steps = vec![
                    Step::FaceX(spot.0 + 100.0),
                    Step::Beat(Beat::new(Gesture::Crouch, ExpressionKind::Content, 0.5)),
                    Step::Beat(Beat::new(ActionKind::Idle, racer.face, 600.0)),
                ];
                ground.direct(racer.id, steps, now);
                racer.doing = Doing::AtTheLine;
                racer.doing_since = now;
            }
        }
        let settled = self
            .racers
            .iter()
            .all(|racer| racer.doing == Doing::AtTheLine && now - racer.doing_since > 0.6);
        if settled {
            self.ready_steady(now);
        }
    }

    /// "Ready, steady…": and whoever can't wait, as its character has it, may be off early.
    /// Nobody jumps the start twice in one race.
    fn ready_steady(&mut self, now: f32) {
        self.jumps.clear();
        if !self.false_started {
            for index in 0..self.racers.len() {
                let impulsive = self.racers[index].impulsive;
                if impulsive > 0.62 && self.dice.chance(((impulsive - 0.55) * 1.8).min(0.85)) {
                    let at = now + self.dice.range(0.5, STEADY_SECS - 0.25);
                    self.jumps.push((index, at));
                }
            }
        }
        for racer in &mut self.racers {
            racer.doing = Doing::AtTheLine;
            racer.along = -TOE;
            racer.shown_along = -TOE;
        }
        self.events.push(Event::Steady);
        self.phase = Phase::Steady {
            since: now,
            go: now + STEADY_SECS,
        };
    }

    fn steady(&mut self, now: f32, go: f32) {
        // Whoever can't wait any longer is off.
        let mut waiting = Vec::new();
        for (index, at) in std::mem::take(&mut self.jumps) {
            if now < at {
                waiting.push((index, at));
                continue;
            }
            let racer = &mut self.racers[index];
            racer.doing = Doing::Jumped { back: false };
            racer.doing_since = now;
            racer.next_hop(now, racer.gait.length);
            self.false_started = true;
            self.events.push(Event::FalseStart { racer: racer.id });
        }
        self.jumps = waiting;
        // A hop and a half on before it notices nobody else went; then, after a moment's
        // embarrassment, back to the line.
        let mut out = false;
        let mut returned = false;
        for racer in &mut self.racers {
            match racer.doing {
                Doing::Jumped { back: false } => {
                    out = true;
                    if racer.hop(now) {
                        if racer.along >= racer.gait.length * 1.4 - TOE {
                            racer.doing = Doing::Jumped { back: true };
                            racer.doing_since = now;
                        } else {
                            racer.next_hop(now, racer.gait.length);
                        }
                    }
                }
                Doing::Jumped { back: true } => {
                    if now - racer.doing_since > SHEEPISH_SECS {
                        racer.along = (racer.along - racer.gait.speed() * 0.6 * self.dt).max(-TOE);
                        if racer.along <= -TOE {
                            racer.doing = Doing::AtTheLine;
                            racer.doing_since = now;
                            returned = true;
                            continue;
                        }
                    }
                    out = true;
                }
                _ => {}
            }
        }
        if out {
            return;
        }
        if returned {
            // Everyone back at the line: "ready, steady…" again.
            self.ready_steady(now);
            return;
        }
        if now >= go {
            for racer in &mut self.racers {
                racer.doing = Doing::Hopping;
                racer.doing_since = now;
                racer.next_hop(now, racer.gait.length);
            }
            self.events.push(Event::Go);
            self.last_cut = now;
            self.phase = Phase::Racing { since: now };
        }
    }

    fn race(&mut self, now: f32, since: f32) {
        let before: Vec<f32> = self.racers.iter().map(|racer| racer.along).collect();
        for index in 0..self.racers.len() {
            self.run(index, now);
        }
        if now - since > LONGEST {
            for racer in &mut self.racers {
                if racer.racing() {
                    racer.along = COURSE;
                }
            }
        }
        self.glares(&before, now);
        for index in 0..self.racers.len() {
            let racer = &self.racers[index];
            if !racer.racing() || racer.along < COURSE {
                continue;
            }
            let (id, took, place) = (racer.id, now - since, self.results.len() + 1);
            self.results.push((id, took));
            if place == 1 {
                self.broken = Some(now);
                self.announced = Some(id);
                self.cheer = true;
            }
            let racer = &mut self.racers[index];
            racer.finished = Some(took);
            racer.doing = Doing::Home;
            racer.doing_since = now;
            if self.reduce_motion {
                // A cut to it stopped past the line.
                racer.along = racer.stop;
                racer.length = 0.0;
            }
            self.events.push(Event::Finished {
                racer: id,
                place,
                took,
            });
        }
        self.lead(now, since);
        if self.racers.iter().all(|racer| !racer.racing()) {
            let took = self.results.first().map_or(0.0, |(_, took)| *took);
            self.events.push(Event::AllHome { took });
            self.phase = Phase::Over { since: now };
            self.cheer = true;
        }
    }

    /// One racer's going on, for a tick.
    fn run(&mut self, index: usize, now: f32) {
        let racer = self.racers[index].clone();
        match racer.doing {
            Doing::Hopping => {
                if self.racers[index].hop(now) {
                    self.landed(index, now);
                }
            }
            Doing::Minding => {
                let ready = racer
                    .family()
                    .and_then(|id| self.racer(id))
                    .is_none_or(|family| !family.held_up());
                if ready {
                    self.carry_on(index, now);
                }
            }
            Doing::Tumbled { until } | Doing::Resting { until } | Doing::Waving { until } => {
                if now >= until {
                    self.carry_on(index, now);
                }
            }
            Doing::Helping { friend, until } => {
                let Some(fallen) = self.index(friend) else {
                    self.carry_on(index, now);
                    return;
                };
                match until {
                    None => {
                        // Over to it, beside it on the side it is coming from.
                        let at = self.racers[fallen].along;
                        let target = if racer.along <= at {
                            at - 9.0
                        } else {
                            at + 9.0
                        };
                        let dt = self.dt;
                        let helper = &mut self.racers[index];
                        let step = helper.gait.speed() * 0.8 * dt;
                        let gap = target - helper.along;
                        helper.along += gap.clamp(-step, step);
                        if gap.abs() <= step {
                            let until = now + HELP_SECS;
                            helper.doing = Doing::Helping {
                                friend,
                                until: Some(until),
                            };
                            helper.doing_since = now;
                            helper.helped = true;
                            self.racers[fallen].doing = Doing::Tumbled { until };
                            self.events.push(Event::HelpedUp {
                                helper: racer.id,
                                racer: friend,
                            });
                        }
                    }
                    Some(until) if now >= until => self.carry_on(index, now),
                    Some(_) => {}
                }
            }
            Doing::Home => {
                let racer = &mut self.racers[index];
                if racer.hop(now) {
                    let length = racer.gait.length.min(racer.stop - racer.along);
                    if length > 0.5 {
                        racer.next_hop(now, length);
                    } else {
                        racer.length = 0.0;
                    }
                }
            }
            _ => {}
        }
    }

    fn set_doing(&mut self, index: usize, doing: Doing, now: f32) {
        let racer = &mut self.racers[index];
        racer.doing = doing;
        racer.doing_since = now;
        // With motion reduced, a change of what it is doing is a moment worth cutting to.
        racer.shown_along = racer.along;
    }

    /// Back to hopping.
    fn carry_on(&mut self, index: usize, now: f32) {
        self.set_doing(index, Doing::Hopping, now);
        let length = self.next_length(index);
        self.racers[index].next_hop(now, length);
    }

    /// How far its next hop goes: about its usual, never past its little one.
    fn next_length(&mut self, index: usize) -> f32 {
        let racer = &self.racers[index];
        let mut length = racer.gait.length * self.dice.range(0.86, 1.12);
        if let Some(little) = racer.little_one.and_then(|id| self.racer(id))
            && little.racing()
        {
            // Beside it, never ahead of it.
            length = length.min(little.along - 1.0 - racer.along);
        }
        length.max(0.0)
    }

    /// A hop has landed: on it goes, unless it sits down, stops to wave, tumbles, or waits for
    /// its little one.
    fn landed(&mut self, index: usize, now: f32) {
        let racer = self.racers[index].clone();
        if let Some(at) = racer.sits_at
            && racer.along >= at
        {
            self.racers[index].sits_at = None;
            let until = now + BREATHER_SECS;
            self.set_doing(index, Doing::Resting { until }, now);
            self.events.push(Event::Breather { racer: racer.id });
            return;
        }
        if let Some(at) = racer.waves_at
            && racer.along >= at
        {
            self.racers[index].waves_at = None;
            let until = now + WAVE_SECS;
            self.set_doing(index, Doing::Waving { until }, now);
            self.events.push(Event::Waving { racer: racer.id });
            return;
        }
        let progress = racer.along / COURSE;
        let wobbly = 0.004 + 0.07 * (racer.impulsive - 0.35).max(0.0);
        if racer.tumbles < MOST_TUMBLES
            && (0.08..0.92).contains(&progress)
            && self.dice.chance(wobbly)
        {
            self.tumble(index, now);
            return;
        }
        // Family waits for family.
        if racer
            .family()
            .and_then(|id| self.racer(id))
            .is_some_and(Racer::held_up)
        {
            self.set_doing(index, Doing::Minding, now);
            return;
        }
        let length = self.next_length(index);
        self.racers[index].next_hop(now, length);
    }

    /// Down it goes, in its sack. Its parent comes to help it up, or a kind-hearted friend near
    /// enough; it stays down, laughing, until they get there.
    fn tumble(&mut self, index: usize, now: f32) {
        self.racers[index].tumbles += 1;
        let until = now + TUMBLE_SECS;
        self.set_doing(index, Doing::Tumbled { until }, now);
        let fallen = self.racers[index].clone();
        self.events.push(Event::Tumble { racer: fallen.id });
        let parent = |r: &Racer| r.little_one == Some(fallen.id);
        let helper = self
            .racers
            .iter()
            .enumerate()
            .filter(|(other, r)| {
                *other != index
                    && r.racing()
                    && matches!(r.doing, Doing::Hopping | Doing::Minding)
                    && (parent(r)
                        || (r.kind_hearted
                            && !r.helped
                            && r.friends.contains(&fallen.id)
                            && (r.along - fallen.along).abs() < 45.0))
            })
            .min_by(|a, b| {
                let near = |r: &Racer| {
                    if parent(r) {
                        -1.0
                    } else {
                        (r.along - fallen.along).abs()
                    }
                };
                near(a.1).total_cmp(&near(b.1))
            })
            .map(|(other, _)| other);
        if let Some(helper) = helper {
            let doing = Doing::Helping {
                friend: fallen.id,
                until: None,
            };
            self.set_doing(helper, doing, now);
            self.racers[index].doing = Doing::Tumbled { until: f32::MAX };
        }
    }

    /// Rivals glare at each other as one passes the other, once a race.
    fn glares(&mut self, before: &[f32], now: f32) {
        for a in 0..self.racers.len() {
            for b in 0..self.racers.len() {
                let (racer, rival) = (&self.racers[a], &self.racers[b]);
                if a == b || !racer.rivals.contains(&rival.id) {
                    continue;
                }
                let pair = (racer.id.min(rival.id), racer.id.max(rival.id));
                let passed = before[a] < before[b] && racer.along > rival.along;
                if passed && !self.glared.contains(&pair) {
                    self.glared.push(pair);
                    self.events.push(Event::Glare {
                        racer: racer.id,
                        rival: rival.id,
                    });
                    self.racers[a].glaring_until = now + GLARE_SECS;
                    self.racers[b].glaring_until = now + GLARE_SECS;
                }
            }
        }
    }

    /// Says when the lead changes hands, once it has been held a moment.
    fn lead(&mut self, now: f32, since: f32) {
        if !self.results.is_empty() {
            return;
        }
        let Some(front) = self
            .racers
            .iter()
            .max_by(|a, b| a.along.total_cmp(&b.along).then(b.id.cmp(&a.id)))
            .map(|racer| racer.id)
        else {
            return;
        };
        if self.front.is_none_or(|(id, _)| id != front) {
            self.front = Some((front, now));
        }
        if let Some((id, held_since)) = self.front
            && now - held_since >= LEAD_HELD_SECS
            && now - since >= 1.5
            && self.announced != Some(id)
        {
            self.announced = Some(id);
            self.events.push(Event::Lead { racer: id });
        }
    }

    /// Shows everyone where they are and what they are doing.
    fn show(&mut self, ground: &mut Playground, now: f32) {
        let racing = matches!(self.phase, Phase::Racing { .. });
        let cut = self.reduce_motion && now - self.last_cut >= CUT_SECS;
        if cut {
            self.last_cut = now;
        }
        if std::mem::take(&mut self.cheer) {
            for id in &self.crowd {
                if let Some(c) = ground.character(*id) {
                    let steps = vec![
                        Step::Beat(c.celebrate(1.4)),
                        Step::Beat(Beat::new(Gesture::Watch, ExpressionKind::Joy, 600.0)),
                    ];
                    ground.direct(*id, steps, now);
                }
            }
        }
        for index in 0..self.racers.len() {
            let look = self.look(index, now);
            let racer = &mut self.racers[index];
            let mut lift = 0.0;
            let along = if self.reduce_motion {
                if cut || !racing || racer.finished.is_some() {
                    racer.shown_along = racer.along;
                }
                racer.shown_along
            } else {
                if let Some(air) = racer.airborne(now) {
                    lift = racer.gait.height * 4.0 * air * (1.0 - air);
                }
                racer.along
            };
            let at = (racer.x(along), racer.lane);
            ground.teleport(racer.id, at);
            ground.set_lift(racer.id, lift);
            if racer.shown != Some(look) {
                ground.tip(racer.id, look.tipped);
                let mut beat = Beat::new(look.clip, look.face, 600.0);
                beat.cue = look.cue;
                let facing = if look.facing_right {
                    at.0 + 100.0
                } else {
                    at.0 - 100.0
                };
                ground.direct(racer.id, vec![Step::FaceX(facing), Step::Beat(beat)], now);
                racer.shown = Some(look);
            }
        }
    }

    /// What a racer looks like doing what it is doing.
    fn look(&self, index: usize, now: f32) -> Look {
        let racer = &self.racers[index];
        let mut look = Look {
            clip: BodyClip::Action(ActionKind::Idle),
            face: racer.face,
            cue: None,
            tipped: false,
            facing_right: true,
        };
        let since = now - racer.doing_since;
        match racer.doing {
            Doing::Lining | Doing::AtTheLine => {
                let jumped = self
                    .racers
                    .iter()
                    .any(|r| matches!(r.doing, Doing::Jumped { .. }));
                if jumped {
                    look.face = ExpressionKind::Curious;
                } else if matches!(self.phase, Phase::Steady { .. }) {
                    look.clip = BodyClip::Gesture(Gesture::Crouch);
                    look.face = ExpressionKind::Determined;
                }
            }
            Doing::Hopping | Doing::Jumped { back: false } | Doing::Home
                if racer.airborne(now).is_some() || self.reduce_motion && racer.racing() =>
            {
                look.clip = BodyClip::Gesture(Gesture::Balance);
                self.hopping_face(racer, now, since, &mut look);
            }
            Doing::Hopping | Doing::Jumped { back: false } => {
                look.clip = BodyClip::Gesture(Gesture::Crouch);
                self.hopping_face(racer, now, since, &mut look);
            }
            Doing::Jumped { back: true } => {
                look.clip = BodyClip::Gesture(Gesture::Peek);
                look.face = ExpressionKind::Content;
                look.facing_right = false;
            }
            Doing::Tumbled { .. } => {
                if since < 0.25 {
                    look.clip = BodyClip::Gesture(Gesture::Gasp);
                    look.face = ExpressionKind::Startled;
                    look.cue = Some(Cue::Exclaim);
                } else {
                    look.tipped = true;
                    look.face = if since < 0.9 {
                        ExpressionKind::Startled
                    } else {
                        ExpressionKind::Joy
                    };
                }
            }
            Doing::Resting { until } => {
                if until - now > 0.9 {
                    look.clip = BodyClip::Gesture(Gesture::Sit);
                    look.face = ExpressionKind::Sleepy;
                    look.cue = Some(Cue::Sleep);
                } else {
                    look.clip = BodyClip::Gesture(Gesture::Yawn);
                    look.face = ExpressionKind::Yawning;
                }
            }
            Doing::Waving { .. } => {
                look.clip = BodyClip::Gesture(Gesture::Strut);
                look.face = ExpressionKind::Smug;
                look.cue = Some(Cue::Sparkle);
            }
            Doing::Helping { friend, until } => {
                if let Some(fallen) = self.racer(friend) {
                    look.facing_right = fallen.along >= racer.along;
                }
                if until.is_some() {
                    look.clip = BodyClip::Gesture(Gesture::Heave);
                    look.face = ExpressionKind::Affectionate;
                    look.cue = Some(Cue::Heart);
                } else {
                    look.clip = BodyClip::Gesture(Gesture::Balance);
                    look.face = ExpressionKind::Worried;
                }
            }
            Doing::Minding => {
                look.face = ExpressionKind::Affectionate;
                if let Some(family) = racer.family().and_then(|id| self.racer(id)) {
                    look.facing_right = family.along >= racer.along;
                }
            }
            Doing::Home => {
                let beat = self.home_beat(index);
                look.clip = beat.clip;
                look.face = beat.expression;
                look.cue = beat.cue;
                look.facing_right = false;
            }
        }
        look
    }

    /// The face it hops with: its own, a glare as it passes a rival, a laugh just up from a
    /// tumble.
    fn hopping_face(&self, racer: &Racer, now: f32, since: f32, look: &mut Look) {
        if now < racer.glaring_until {
            look.face = ExpressionKind::Grumpy;
            look.cue = Some(Cue::Huff);
        } else if racer.doing == Doing::Hopping && since < 0.6 && racer.tumbles > 0 {
            look.face = ExpressionKind::Joy;
            look.cue = Some(Cue::Note);
        }
    }

    /// How a racer takes being over the line, stopped and turned to watch the rest come in: the
    /// winner celebrates in its own way, and so does everyone once all are home; until then the
    /// rest are pleased with themselves, each its own way.
    fn home_beat(&self, index: usize) -> Beat {
        let racer = &self.racers[index];
        let first = self.results.first().map(|(id, _)| *id) == Some(racer.id);
        if first || matches!(self.phase, Phase::Over { .. }) {
            let mut cheer = Beat::new(Gesture::Cheer, ExpressionKind::Joy, 600.0);
            cheer.cue = Some(Cue::Sparkle);
            return cheer;
        }
        match racer.face {
            ExpressionKind::Grumpy => {
                let mut huff = Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 600.0);
                huff.cue = Some(Cue::Huff);
                huff
            }
            ExpressionKind::Sleepy => Beat::new(Gesture::Sit, ExpressionKind::Sleepy, 600.0),
            ExpressionKind::Smug => Beat::new(Gesture::Strut, ExpressionKind::Smug, 600.0),
            _ => Beat::new(Gesture::Watch, ExpressionKind::Joy, 600.0),
        }
    }

    /// Calls the race off: out of the sacks, and back to playing, pleased with themselves.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        for racer in &self.racers {
            if let Some(c) = ground.character(racer.id) {
                let beat = c.celebrate(1.0);
                ground.direct(racer.id, vec![Step::Beat(beat)], now);
            }
        }
        self.finish(ground, now);
        self.events.clear();
    }

    /// Out of the sacks, and everyone back to playing: the crowd stops watching too.
    fn finish(&mut self, ground: &mut Playground, now: f32) {
        for id in &self.crowd {
            ground.direct(*id, Vec::new(), now);
        }
        for racer in &self.racers {
            ground.sack(racer.id, None);
            ground.tip(racer.id, false);
            ground.set_lift(racer.id, 0.0);
        }
        ground.release();
        self.racers.clear();
        self.crowd.clear();
        self.jumps.clear();
        self.phase = Phase::Ready;
    }
}

/// The face it races with.
fn race_face(character: &Character) -> ExpressionKind {
    match character.kind {
        TemperamentKind::Grump => ExpressionKind::Grumpy,
        TemperamentKind::Showoff => ExpressionKind::Smug,
        TemperamentKind::Lazybones => ExpressionKind::Sleepy,
        _ if character.axes.playfulness > 0.65 => ExpressionKind::Joy,
        _ => ExpressionKind::Determined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fairground;
    use formiga_travel::{Band, Traveler};

    const TICK: f32 = 1.0 / 30.0;

    /// The sample colony, each traveller tuned by `tune`.
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

    /// How a race went: what happened, the finish, and when everyone was back to playing.
    struct Run {
        events: Vec<Event>,
        results: Vec<(Id, f32)>,
        over_at: Option<f32>,
    }

    /// The fairground with the colony settled in, and a race to run.
    fn ready(cast: &Cast) -> (Playground, SackRace) {
        let (mut ground, _) = fairground::open(cast, 0.0);
        let mut now = 0.0;
        while now < 6.0 {
            now += TICK;
            ground.tick(cast, now);
        }
        (ground, SackRace::new())
    }

    /// Plays a race on from `from` until everyone is back to playing, or `to`, calling `each`
    /// every tick.
    fn run(
        ground: &mut Playground,
        race: &mut SackRace,
        cast: &Cast,
        from: f32,
        to: f32,
        mut each: impl FnMut(&mut SackRace, &mut Playground, f32),
    ) -> Run {
        let mut events = Vec::new();
        let mut now = from;
        let mut over_at = None;
        while now < to {
            now += TICK;
            ground.tick(cast, now);
            race.tick(ground, now);
            each(race, ground, now);
            events.extend(race.take_events());
            if race.phase() == Phase::Ready {
                over_at = Some(now);
                break;
            }
        }
        Run {
            events,
            results: race.results().to_vec(),
            over_at,
        }
    }

    fn race_between(cast: &Cast, racers: &[Id]) -> Run {
        let (mut ground, mut race) = ready(cast);
        race.start(&mut ground, cast, racers, 6.0);
        run(&mut ground, &mut race, cast, 6.0, 400.0, |_, _, _| {})
    }

    fn count(events: &[Event], wanted: impl Fn(&Event) -> bool) -> usize {
        events.iter().filter(|event| wanted(event)).count()
    }

    #[test]
    fn every_hop_is_counted_once_as_it_lands() {
        let cast = sample();
        let racers = cast.members.len() as u32;
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[], 6.0);
        let mut counted = 0;
        let mut most = 0;
        run(&mut ground, &mut race, &cast, 6.0, 400.0, |race, _, _| {
            let hops = race.hops();
            if race.phase() != Phase::Ready {
                assert!(hops >= counted, "the count went back");
                assert!(hops - counted <= racers, "a hop was counted twice");
            }
            counted = hops;
            most = most.max(hops);
        });
        // Nobody covers the course in fewer than a handful of hops.
        assert!(most >= racers * 5, "only {most} hops heard");
        assert_eq!(race.hops(), 0, "a race over leaves nothing to hear");
    }

    #[test]
    fn everyone_lines_up_hops_home_and_gets_a_time() {
        let cast = sample();
        let run = race_between(&cast, &[]);
        assert!(
            run.over_at.is_some(),
            "the race never ended: {:?}",
            run.events
        );
        assert_eq!(run.results.len(), cast.members.len());
        for member in &cast.members {
            assert!(
                run.results.iter().any(|(id, _)| *id == member.id),
                "{} has no time",
                member.name
            );
        }
        assert!(run.results.windows(2).all(|pair| pair[0].1 <= pair[1].1));
        assert!(
            run.results
                .iter()
                .all(|(_, took)| *took > 3.0 && *took < LONGEST)
        );
        assert_eq!(count(&run.events, |e| *e == Event::Go), 1);
        assert!(
            run.events
                .iter()
                .any(|e| matches!(e, Event::AllHome { .. }))
        );
        let (winner, took) = run.results[0];
        assert!(run.events.contains(&Event::Finished {
            racer: winner,
            place: 1,
            took
        }));
    }

    #[test]
    fn two_picked_race_and_the_rest_watch() {
        let cast = sample();
        let pair = [cast.members[1].id, cast.members[3].id];
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &pair, 6.0);
        assert_eq!(race.racers.len(), 2);
        assert_eq!(race.crowd.len(), cast.members.len() - 2);
        let run = run(&mut ground, &mut race, &cast, 6.0, 300.0, |_, _, _| {});
        let mut finished: Vec<Id> = run.results.iter().map(|(id, _)| *id).collect();
        finished.sort();
        let mut wanted = pair.to_vec();
        wanted.sort();
        assert_eq!(finished, wanted);
    }

    #[test]
    fn the_race_always_ends_however_the_colony_is() {
        let tunings: [fn(usize, &mut Traveler); 4] = [
            |_, t| t.character.axes.impulsiveness = 1.0,
            |_, t| {
                t.character.temperament = TemperamentKind::Lazybones.into();
                t.character.axes.energy = 0.0;
                t.motion.activity = 0.0;
            },
            |_, t| {
                t.character.temperament = TemperamentKind::Showoff.into();
                t.character.axes.affection = 1.0;
                t.character.axes.impulsiveness = 0.9;
            },
            |i, t| t.character.axes.impulsiveness = (i % 2) as f32,
        ];
        for tune in tunings {
            let cast = colony(tune);
            let (mut ground, mut race) = ready(&cast);
            for round in 0..3 {
                let from = 6.0 + round as f32 * 200.0;
                race.start(&mut ground, &cast, &[], from);
                let run = run(
                    &mut ground,
                    &mut race,
                    &cast,
                    from,
                    from + 200.0,
                    |_, _, _| {},
                );
                assert!(
                    run.over_at.is_some(),
                    "round {round} never ended: {:?}",
                    run.events
                );
                assert_eq!(run.results.len(), cast.members.len());
            }
        }
    }

    #[test]
    fn an_impulsive_racer_can_jump_the_start_and_everyone_goes_back_to_the_line() {
        let cast = colony(|_, t| t.character.axes.impulsiveness = 1.0);
        let (mut ground, mut race) = ready(&cast);
        let mut jumped = 0;
        for round in 0..4 {
            let from = 6.0 + round as f32 * 200.0;
            race.start(&mut ground, &cast, &[], from);
            let mut at_the_go = Vec::new();
            let run = run(
                &mut ground,
                &mut race,
                &cast,
                from,
                from + 200.0,
                |race, _, _| {
                    if race.events.contains(&Event::Go) && at_the_go.is_empty() {
                        at_the_go = race.racers.iter().map(|r| r.from).collect();
                    }
                },
            );
            let false_starts = count(&run.events, |e| matches!(e, Event::FalseStart { .. }));
            jumped += false_starts;
            if false_starts > 0 {
                // Back to the line, and "ready, steady…" again, before anyone goes.
                let go = run.events.iter().position(|e| *e == Event::Go).unwrap();
                let jump = run
                    .events
                    .iter()
                    .position(|e| matches!(e, Event::FalseStart { .. }))
                    .unwrap();
                assert!(jump < go);
                assert_eq!(count(&run.events[jump..go], |e| *e == Event::Steady), 1);
                assert!(at_the_go.iter().all(|from| *from == -TOE), "{at_the_go:?}");
            }
            // Never twice in one race.
            assert!(
                count(&run.events, |e| *e == Event::Steady) <= 2,
                "{:?}",
                run.events
            );
        }
        assert!(jumped > 0, "nobody that impulsive ever jumped the start");
    }

    #[test]
    fn a_patient_colony_never_jumps_the_start() {
        let cast = colony(|_, t| t.character.axes.impulsiveness = 0.3);
        let (mut ground, mut race) = ready(&cast);
        for round in 0..4 {
            let from = 6.0 + round as f32 * 200.0;
            race.start(&mut ground, &cast, &[], from);
            let run = run(
                &mut ground,
                &mut race,
                &cast,
                from,
                from + 200.0,
                |_, _, _| {},
            );
            assert_eq!(
                count(&run.events, |e| matches!(e, Event::FalseStart { .. })),
                0
            );
        }
    }

    #[test]
    fn a_lazybones_sits_down_for_a_breather() {
        // One traveller, made the laziest of lazybones; nobody else is one.
        let cast = colony(|i, t| {
            if i == 2 {
                t.character.temperament = TemperamentKind::Lazybones.into();
                t.character.axes.energy = 0.0;
            } else {
                t.character.temperament = TemperamentKind::Guardian.into();
            }
        });
        let lazy = cast.members[2].id;
        let run = race_between(&cast, &[]);
        let breathers: Vec<Id> = run
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Breather { racer } => Some(*racer),
                _ => None,
            })
            .collect();
        assert_eq!(breathers, vec![lazy]);
        assert!(
            run.results.iter().any(|(id, _)| *id == lazy),
            "it got up again and finished"
        );
    }

    /// Two adults alike in everything that makes them hop, neither of them tumbling, one of them
    /// a show-off if `showoff`.
    fn twins(showoff: bool) -> (Cast, [Id; 2]) {
        let cast = colony(|i, t| {
            t.character.axes.impulsiveness = 0.0;
            t.character.axes.energy = 0.6;
            t.character.axes.boldness = 0.5;
            t.motion.activity = 0.6;
            t.stature_percent = 100;
            t.character.temperament = TemperamentKind::Guardian.into();
            if i == 1 && showoff {
                t.character.temperament = TemperamentKind::Showoff.into();
            }
        });
        let pair = [cast.members[1].id, cast.members[3].id];
        (cast, pair)
    }

    #[test]
    fn a_show_off_waves_to_the_crowd_just_short_of_the_line_and_is_overtaken() {
        let (cast, [showy, other]) = twins(true);
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[showy, other], 6.0);
        let mut waved_at = None;
        let run = run(&mut ground, &mut race, &cast, 6.0, 300.0, |race, _, _| {
            if waved_at.is_none() && race.events.contains(&Event::Waving { racer: showy }) {
                waved_at = race.racer(showy).map(|r| r.along);
            }
        });
        let along = waved_at.expect("the show-off never waved");
        assert!(along > COURSE - 30.0 && along < COURSE, "waved at {along}");
        assert_eq!(run.results[0].0, other, "waving cost it the race");
        // With no show-off, nobody stops to wave.
        let (cast, pair) = twins(false);
        let run = race_between(&cast, &pair);
        assert_eq!(count(&run.events, |e| matches!(e, Event::Waving { .. })), 0);
    }

    #[test]
    fn an_affectionate_racer_stops_to_help_a_friend_up() {
        // Nobody tumbles of its own accord: one is tipped over partway, just ahead of a friend.
        for (affection, helps) in [(0.95, true), (0.3, false)] {
            let cast = colony(move |_, t| {
                t.character.axes.impulsiveness = 0.0;
                t.character.axes.affection = affection;
                t.character.temperament = TemperamentKind::Guardian.into();
            });
            // Two adults who are friends, without family in the race to help instead.
            let (a, b) = (cast.members[1].id, cast.members[4].id);
            assert!(
                cast.bond(a, b)
                    .is_some_and(|bond| bond.warmth >= Band::Medium && bond.friction < bond.warmth)
            );
            let (mut ground, mut race) = ready(&cast);
            race.start(&mut ground, &cast, &[a, b], 6.0);
            let mut tipped = false;
            let run = run(&mut ground, &mut race, &cast, 6.0, 300.0, |race, _, now| {
                if !tipped && race.racing_for(now) > 3.0 {
                    tipped = true;
                    let (fallen, friend) = (race.index(b).unwrap(), race.index(a).unwrap());
                    race.racers[friend].along = race.racers[fallen].along - 10.0;
                    race.racers[friend].length = 0.0;
                    race.tumble(fallen, now);
                }
            });
            let helped = run.events.contains(&Event::HelpedUp {
                helper: a,
                racer: b,
            });
            assert_eq!(helped, helps, "affection {affection}: {:?}", run.events);
            assert_eq!(run.results.len(), 2, "both got home");
        }
    }

    #[test]
    fn a_parent_keeps_beside_its_little_one() {
        let cast = sample();
        let little = cast.members.iter().find(|m| m.parent().is_some()).unwrap();
        let (mini, parent) = (little.id, little.parent().unwrap());
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[], 6.0);
        let lanes = (
            race.racer(mini).unwrap().lane,
            race.racer(parent).unwrap().lane,
        );
        assert!(
            lanes.0 > lanes.1,
            "the little one races in the lane in front"
        );
        let mut widest: f32 = 0.0;
        let run = run(&mut ground, &mut race, &cast, 6.0, 300.0, |race, _, _| {
            let (Some(m), Some(p)) = (race.racer(mini), race.racer(parent)) else {
                return;
            };
            if matches!(race.phase, Phase::Racing { .. }) && m.racing() && p.racing() {
                assert!(p.along <= m.along + 0.01, "the parent got ahead");
                widest = widest.max(m.along - p.along);
            }
        });
        assert!(widest < 25.0, "they were {widest} apart");
        let time = |id: Id| run.results.iter().find(|(r, _)| *r == id).unwrap().1;
        assert!(time(parent) >= time(mini) && time(parent) - time(mini) < 4.0);
    }

    #[test]
    fn rivals_glare_at_each_other_as_one_passes_the_other() {
        for at_odds in [true, false] {
            let mut snapshot = formiga_travel::sample::snapshot();
            for (index, traveler) in snapshot.travelers.iter_mut().enumerate() {
                traveler.character.axes.impulsiveness = 0.0;
                // One quick, one slow.
                let lively = if index == 1 { 1.0 } else { 0.3 };
                traveler.character.axes.energy = lively;
                traveler.motion.activity = lively;
            }
            let (a, b) = (snapshot.travelers[1].id, snapshot.travelers[3].id);
            for pair in &mut snapshot.relationships {
                if (pair.a == a && pair.b == b) || (pair.a == b && pair.b == a) {
                    pair.affinity = if at_odds { Band::Low } else { Band::High };
                    pair.avoidance = if at_odds { Band::High } else { Band::None };
                }
            }
            let cast = Cast::new(snapshot).unwrap();
            let (a, b) = (a.0, b.0);
            assert_eq!(cast.at_odds(a, b), at_odds);
            let (mut ground, mut race) = ready(&cast);
            race.start(&mut ground, &cast, &[a, b], 6.0);
            // The slower is given a head start, so the quicker has to pass it.
            let speed = |race: &SackRace, id: Id| race.racer(id).unwrap().gait.speed();
            let slower = if speed(&race, a) < speed(&race, b) {
                a
            } else {
                b
            };
            let mut given = false;
            let run = run(&mut ground, &mut race, &cast, 6.0, 300.0, |race, _, _| {
                if !given && matches!(race.phase, Phase::Racing { .. }) {
                    given = true;
                    let index = race.index(slower).unwrap();
                    race.racers[index].along = 20.0;
                    race.racers[index].from = 20.0;
                }
            });
            let glares = count(&run.events, |e| matches!(e, Event::Glare { .. }));
            assert_eq!(glares, usize::from(at_odds), "{:?}", run.events);
        }
    }

    #[test]
    fn tumbles_come_more_often_with_impulsiveness_and_everyone_gets_up() {
        let tumbles = |impulsiveness: f32| {
            let cast = colony(move |_, t| t.character.axes.impulsiveness = impulsiveness);
            let (mut ground, mut race) = ready(&cast);
            let mut total = 0;
            for round in 0..6 {
                let from = 6.0 + round as f32 * 200.0;
                race.start(&mut ground, &cast, &[], from);
                let run = run(
                    &mut ground,
                    &mut race,
                    &cast,
                    from,
                    from + 200.0,
                    |_, _, _| {},
                );
                total += count(&run.events, |e| matches!(e, Event::Tumble { .. }));
                assert_eq!(run.results.len(), cast.members.len(), "someone stayed down");
            }
            total
        };
        let (wild, steady) = (tumbles(1.0), tumbles(0.2));
        assert!(
            wild > steady * 2 && wild >= 4,
            "{wild} tumbles against {steady}"
        );
    }

    #[test]
    fn a_little_one_hops_small() {
        let cast = sample();
        let little = cast.members.iter().find(|m| m.parent().is_some()).unwrap();
        let mut grown = Character::of(little);
        grown.size = 1.0;
        let (small, big) = (Gait::of(&Character::of(little)), Gait::of(&grown));
        assert!(small.length < big.length * 0.85 && small.height < big.height);
        assert!(small.speed() < big.speed());
    }

    #[test]
    fn the_same_colony_with_other_temperaments_races_differently() {
        let others = formiga_travel::sample::snapshot();
        let swapped = colony(|i, t| {
            let other = &others.travelers[(i + 2) % others.travelers.len()];
            t.character = other.character.clone();
            t.motion = other.motion;
        });
        let (first, second) = (race_between(&sample(), &[]), race_between(&swapped, &[]));
        let order = |run: &Run| run.results.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        assert_ne!(order(&first), order(&second));
        assert_ne!(first.events, second.events);
    }

    #[test]
    fn with_reduced_motion_the_race_cuts_from_moment_to_moment() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[], 6.0);
        let mut last: Vec<(f32, f32)> = Vec::new();
        let (mut ticks, mut moves) = (0, 0);
        let run = run(
            &mut ground,
            &mut race,
            &cast,
            6.0,
            300.0,
            |race, ground, _| {
                if !matches!(race.phase, Phase::Racing { .. }) {
                    return;
                }
                let now: Vec<(f32, f32)> = race
                    .racers
                    .iter()
                    .map(|r| ground.position(r.id).unwrap())
                    .collect();
                ticks += 1;
                if !last.is_empty() && now != last {
                    moves += 1;
                }
                last = now;
            },
        );
        assert!(run.over_at.is_some());
        assert_eq!(run.results.len(), cast.members.len());
        // Held still between cuts: nobody is seen hopping along.
        assert!(moves * 8 < ticks, "{moves} moves in {ticks} ticks");
        assert!(moves > 3, "the race never moved on");
    }

    #[test]
    fn calling_the_race_off_lets_everyone_out_of_their_sacks() {
        let cast = sample();
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[], 6.0);
        run(&mut ground, &mut race, &cast, 6.0, 20.0, |_, _, _| {});
        assert_ne!(race.phase(), Phase::Ready);
        race.stop(&mut ground, 20.0);
        assert_eq!(race.phase(), Phase::Ready);
        assert!(race.racers.is_empty() && race.gear(20.0).is_empty());
        // Back to playing: free play moves them on.
        let before: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        let mut now = 20.0;
        while now < 40.0 {
            now += TICK;
            ground.tick(&cast, now);
        }
        let after: Vec<_> = ground.ids().iter().map(|id| ground.position(*id)).collect();
        assert_ne!(before, after);
    }

    #[test]
    fn after_a_race_the_racers_go_back_to_playing() {
        let cast = sample();
        let (mut ground, mut race) = ready(&cast);
        let racers: Vec<Id> = cast.ids().take(3).collect();
        race.start(&mut ground, &cast, &racers, 6.0);
        let over = run(&mut ground, &mut race, &cast, 6.0, 400.0, |_, _, _| {})
            .over_at
            .expect("the race never ended");
        let was: Vec<_> = racers
            .iter()
            .map(|id| ground.position(*id).unwrap())
            .collect();
        let mut now = over;
        while now < over + 30.0 {
            now += TICK;
            ground.tick(&cast, now);
        }
        let moved = racers
            .iter()
            .zip(&was)
            .filter(|(id, was)| {
                crate::playground::distance(ground.position(**id).unwrap(), **was) > 2.0
            })
            .count();
        assert!(
            moved > 0,
            "the racers are still posing at the line half a minute on"
        );
    }

    #[test]
    fn six_racers_have_room_at_the_line_and_along_the_way() {
        use crate::fairground::testing::overlap;
        let cast = sample();
        assert_eq!(cast.members.len(), 6);
        let (mut ground, mut race) = ready(&cast);
        race.start(&mut ground, &cast, &[], 6.0);
        let mut now = 6.0;
        while !matches!(race.phase(), Phase::Steady { .. }) && now < 40.0 {
            now += TICK;
            ground.tick(&cast, now);
            race.tick(&mut ground, now);
        }
        assert!(
            matches!(race.phase(), Phase::Steady { .. }),
            "nobody got to the line"
        );
        // A lane each, well apart, and nobody in front hiding the face of anyone behind.
        let mut lanes: Vec<f32> = race.racers.iter().map(|racer| racer.lane).collect();
        lanes.sort_by(f32::total_cmp);
        assert!(
            lanes.windows(2).all(|pair| pair[1] - pair[0] >= 10.0),
            "{lanes:?}"
        );
        for racer in &race.racers {
            let (x, y) = ground.face(racer.id).unwrap();
            let face = (x - 4, y - 4, x + 4, y + 4);
            for other in &race.racers {
                if other.lane > racer.lane {
                    let drawn = ground.bounds(other.id, now).unwrap();
                    assert_eq!(
                        overlap(face, drawn),
                        0,
                        "the racer in lane {} hides the face of the one in lane {}",
                        other.lane,
                        racer.lane
                    );
                }
            }
        }
        // Nobody stands at the line or hops along the course behind something, or against it:
        // anything a racer is drawn over is well behind it.
        let mut checked = 0;
        while race.phase() != Phase::Ready && now < 300.0 {
            now += TICK;
            ground.tick(&cast, now);
            race.tick(&mut ground, now);
            if !matches!(race.phase(), Phase::Steady { .. } | Phase::Racing { .. }) {
                continue;
            }
            for racer in &race.racers {
                let drawn = ground.bounds(racer.id, now).unwrap();
                for prop in ground.props() {
                    if overlap(drawn, prop.bounds()) > 0 {
                        assert!(
                            prop.base + 8.0 <= racer.lane,
                            "a racer at {drawn:?} is against a prop at {:?}",
                            prop.bounds()
                        );
                    }
                }
                checked += 1;
            }
        }
        assert!(checked > 1000, "the race was hardly seen");
    }

    #[test]
    fn a_helper_goes_over_as_fast_however_often_the_race_is_drawn() {
        let cast = sample();
        // How long a racer takes to reach a fallen friend, ticked every `tick` seconds.
        let reaching = |tick: f32| {
            let (mut ground, mut race) = ready(&cast);
            let racers: Vec<Id> = cast.ids().take(2).collect();
            race.start(&mut ground, &cast, &racers, 6.0);
            race.phase = Phase::Racing { since: 6.0 };
            let fallen = race.racers[1].id;
            race.racers[1].along = 60.0;
            race.racers[1].doing = Doing::Tumbled { until: 1000.0 };
            race.racers[0].along = 0.0;
            race.racers[0].doing = Doing::Helping {
                friend: fallen,
                until: None,
            };
            let mut now = 6.0;
            while matches!(race.racers[0].doing, Doing::Helping { until: None, .. }) && now < 60.0 {
                now += tick;
                race.tick(&mut ground, now);
            }
            now - 6.0
        };
        let (smooth, slow) = (reaching(1.0 / 120.0), reaching(1.0 / 20.0));
        assert!(
            (smooth - slow).abs() < 0.15,
            "{smooth:.2}s at 120 frames a second, {slow:.2}s at 20"
        );
    }
}
