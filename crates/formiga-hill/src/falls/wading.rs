//! Wading at the Far Falls: what the falls bring down, and catching it as the eddy brings it
//! round.
//!
//! The person plays. Now and then something comes over the lip with the water: first a glint up
//! under the arch, plain for a rare thing and faint for an exceptional one, then down it tumbles
//! into the foam, and the eddy carries it round the pool. Each time it comes past the wading stone
//! there is a moment to catch it, the water there marked gold; the rarer it is, the quicker it
//! comes round and the narrower the moment. A miss costs a little light and the water slackens a
//! little, so nobody is ever shut out; standing about costs light too, as the day goes on. What
//! comes down on a visit is chosen as the party arrives (see `finds::afar`).
//!
//! Who came along matters. Whoever is best with water widens the moment (see `rummage::knack`);
//! a second pair of paws helps, a close friend most of all; someone bold wades out further still;
//! someone curious spots the glint up at the lip and says so, and it shows the plainer for it.

use super::{BESIDE, EDDY, FOOT, LIP, WADE, WATCH};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{Find, Kind, Tier, afar, art};
use crate::paint::{blit, put, rect, rgb, rgba};
use crate::playground::Playground;
use crate::track::crew;
use crate::woods::Influence;
use crate::woods::rummage::{delighted, dim, draw_basket, knack};
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{ActionKind, Gesture, TemperamentKind};
use std::f32::consts::{PI, TAU};

/// The light a visit starts with, on its own; an expedition starts it partway through its day.
pub const LIGHT: f32 = 100.0;
/// How many things the falls bring down on one visit.
pub const COMING: usize = 2;
/// What a try costs, and how much light goes each second.
const TRY_COST: f32 = 3.0;
const LIGHT_PER_SEC: f32 = 0.25;
/// How long after settling the first thing comes, and how long between one and the next.
const FIRST_SECS: f32 = 2.5;
const BETWEEN: (f32, f32) = (2.5, 4.5);
/// How long a thing takes to fall from the lip to the pool.
const FALL_SECS: f32 = 0.7;
/// How long a find is held up to look at.
const REVEAL_SECS: f32 = 1.8;
/// Seconds for the eddy to bring something once round, before rarity hurries it.
const ROUND_SECS: f32 = 3.4;
/// The moment at the wading stone, before anything widens or narrows it, in radians of the
/// eddy, and the little extra a steady hand is allowed.
const ARC: f32 = 0.95;
const GRACE: f32 = 0.08;
/// The widest the moment grows, however many misses.
const WIDEST: f32 = 2.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The party coming down the path to the pool.
    Arriving {
        since: f32,
    },
    /// Watching the lip; the next thing comes over then.
    Watching {
        next_at: f32,
    },
    /// A glint up at the lip: something is about to come over.
    Glinting {
        since: f32,
    },
    /// Tumbling down the fall.
    Falling {
        since: f32,
    },
    /// Going round the eddy, to be caught as it comes past.
    Circling(Circle),
    /// Caught, and held up to look at.
    Caught {
        since: f32,
    },
    Leaving {
        since: f32,
    },
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    since: f32,
    /// Seconds for once round, and the moment's width, in radians.
    period: f32,
    width: f32,
    misses: u32,
    /// With reduced motion it only goes round while the person holds it.
    held_angle: f32,
    holding: bool,
}

impl Circle {
    /// How far round the eddy it is, from where the fall meets it, going right first.
    fn angle(&self, now: f32, reduce_motion: bool) -> f32 {
        if reduce_motion {
            self.held_angle
        } else {
            ((now - self.since) / self.period * TAU).rem_euclid(TAU)
        }
    }

    /// Whether, this far round, it is coming past the wading stone.
    fn at_the_stone(&self, angle: f32) -> bool {
        off_the_stone(angle).abs() <= self.width / 2.0 + GRACE
    }
}

/// How far round from the wading stone an angle is, either way.
fn off_the_stone(angle: f32) -> f32 {
    (angle - PI + PI).rem_euclid(TAU) - PI
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Something glints up at the lip; who spotted it, if anyone did.
    Glint {
        find: &'static str,
        spotted: Option<Id>,
    },
    /// Down it comes, into the pool, and round.
    Down,
    Caught {
        find: &'static str,
    },
    Slipped,
    /// The falls have brought down all they will today.
    AllDown,
    Leaving,
}

/// What a visit sets out with: the same shape as every Woods outing's.
pub struct Outset {
    /// Who came along; the first wades in.
    pub party: Vec<Id>,
    /// Visits to the far places in a row that brought nothing new home from afar.
    pub drought: u32,
    /// Whether two who came are close friends.
    pub close_pair: bool,
    /// What the Hilltop lends the Woods: light.
    pub influence: Influence,
    pub seed: u64,
}

pub struct Wading {
    /// How dark the hour has made the falls already, so the day's own dusk only darkens them past
    /// that.
    hour_dark: f32,
    party: Vec<Id>,
    characters: Vec<Character>,
    /// What the falls bring down this visit, in order, and which is next.
    coming: Vec<&'static Find>,
    next: usize,
    phase: Phase,
    light: f32,
    full: f32,
    basket: Vec<&'static str>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    clock: f32,
    /// Whether the person is holding the eddy round, with reduced motion.
    holding: bool,
    /// How much who came widens the moment.
    widen: f32,
    /// Who spotted the glint at the lip, for the one there now.
    spotted: Option<Id>,
}

impl Wading {
    /// Sets out with the party, already coming down the path in `ground`. `found_before` says
    /// which finds the colony has already.
    pub fn new(
        ground: &mut Playground,
        outset: Outset,
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
        let coming = afar::bring_down(COMING, &refs, found_before, drought, &mut dice);
        let best = characters
            .iter()
            .map(|c| knack(c, Kind::Scoop))
            .fold(0.85, f32::max);
        let help = match characters.len() {
            0 | 1 => 1.0,
            _ if close_pair => 1.3,
            _ => 1.15,
        };
        let bold = if characters.iter().any(crew::bold) {
            1.1
        } else {
            1.0
        };
        let full = LIGHT + influence.light;
        Self {
            hour_dark: 0.0,
            party,
            characters,
            coming,
            next: 0,
            phase: Phase::Arriving { since: now },
            light: full,
            full,
            basket: Vec::new(),
            events: Vec::new(),
            dice,
            reduce_motion: ground.reduce_motion(),
            clock: now,
            holding: false,
            widen: best * help * bold,
            spotted: None,
        }
    }

    /// Starts this visit partway through a longer day than its own, as a leg of an expedition:
    /// with `light` left of that day's `full`, and the basket carried in already in it.
    pub fn partway(mut self, light: f32, full: f32, basket: Vec<&'static str>) -> Self {
        self.light = light;
        self.full = full;
        self.basket = basket;
        self
    }

    /// The light left, on the day's own scale.
    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    /// What the falls bring down this visit, in order.
    #[cfg(test)]
    pub fn coming(&self) -> &[&'static Find] {
        &self.coming
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// How dark the hour has made the falls already: the day's dusk adds only what is more.
    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
    }

    /// The thing on its way down, or round, or held up now.
    fn current(&self) -> Option<&'static Find> {
        match self.phase {
            Phase::Glinting { .. }
            | Phase::Falling { .. }
            | Phase::Circling(_)
            | Phase::Caught { .. } => self.coming.get(self.next).copied(),
            _ => None,
        }
    }

    /// Whether what is going round is coming past the stone right now, for a steady hand to
    /// catch it: in the middle of the moment, not at its edges.
    pub fn at_the_stone(&self, now: f32) -> bool {
        match self.phase {
            Phase::Circling(circle) => {
                off_the_stone(circle.angle(now, self.reduce_motion)).abs() < circle.width / 4.0
            }
            _ => false,
        }
    }

    /// The person tries to catch what is going round.
    pub fn strike(&mut self, ground: &mut Playground, now: f32) {
        let Phase::Circling(mut circle) = self.phase else {
            return;
        };
        let Some(find) = self.current() else {
            return;
        };
        self.light -= TRY_COST;
        if circle.at_the_stone(circle.angle(now, self.reduce_motion)) {
            self.basket.push(find.id);
            self.events.push(Event::Caught { find: find.id });
            for (id, character) in self.party.iter().zip(&self.characters) {
                let mut beats = delighted(character, find.tier);
                if Some(*id) == self.party.first().copied() {
                    beats.insert(0, Beat::new(Gesture::Reach, ExpressionKind::Joy, 0.5));
                }
                ground.direct(*id, beats.into_iter().map(Step::Beat).collect(), now);
            }
            self.phase = Phase::Caught { since: now };
        } else {
            // Round it goes again, and the water slackens a little each time.
            let angle = circle.angle(now, self.reduce_motion);
            circle.misses += 1;
            circle.width = (circle.width * 1.15).min(WIDEST);
            circle.period *= 1.1;
            circle.since = now - angle / TAU * circle.period;
            self.events.push(Event::Slipped);
            if let Some(leader) = self.party.first() {
                let mut oops = Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.4);
                oops.cue = Some(Cue::Exclaim);
                let steps = vec![Step::Beat(oops), Step::Beat(wading())];
                ground.direct(*leader, steps, now);
            }
            self.phase = Phase::Circling(circle);
            if self.light <= 0.0 {
                self.leave(ground, now);
            }
        }
    }

    /// With reduced motion, the person brings it round by holding, and lets go to catch.
    pub fn hold(&mut self, ground: &mut Playground, holding: bool, dt: f32, now: f32) {
        self.holding = holding;
        if !self.reduce_motion {
            return;
        }
        let Phase::Circling(mut circle) = self.phase else {
            return;
        };
        if holding {
            circle.held_angle = (circle.held_angle + dt / circle.period * TAU).rem_euclid(TAU);
            circle.holding = true;
            self.phase = Phase::Circling(circle);
        } else if circle.holding {
            circle.holding = false;
            self.phase = Phase::Circling(circle);
            self.strike(ground, now);
        }
    }

    /// Calls the visit to an end: everyone heads off with what is in the basket.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.leave(ground, now);
        }
    }

    fn leave(&mut self, ground: &mut Playground, now: f32) {
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let steps = vec![Step::Walk {
                to: (-30.0 - 20.0 * index as f32, 194.0),
            }];
            ground.direct(*id, steps, now);
        }
        self.events.push(Event::Leaving);
        self.phase = Phase::Leaving { since: now };
    }

    /// Whoever wades in first onto the stone, a second beside, anyone else on the shingle.
    fn settle(&mut self, ground: &mut Playground, now: f32) {
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let to = match index {
                0 => WADE,
                1 => BESIDE,
                _ => WATCH,
            };
            let steps = vec![
                Step::Walk { to },
                Step::FaceX(FOOT.0),
                Step::Beat(self.at_the_pool(index)),
            ];
            ground.direct(*id, steps, now);
        }
    }

    /// Back to the water after something has been looked at, everyone where they stand.
    fn resume(&self, ground: &mut Playground, now: f32) {
        for (index, id) in self.party.iter().enumerate() {
            let steps = vec![Step::FaceX(FOOT.0), Step::Beat(self.at_the_pool(index))];
            ground.direct(*id, steps, now);
        }
    }

    /// What each does at the pool: the first wades, a second watches the water beside it, and
    /// anyone else sits on the shingle, or dozes if it is a lazybones.
    fn at_the_pool(&self, index: usize) -> Beat {
        let lazy = self
            .characters
            .get(index)
            .is_some_and(|c| c.kind == TemperamentKind::Lazybones);
        match index {
            0 => wading(),
            1 => watching(),
            _ if lazy => {
                let mut doze = Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 600.0);
                doze.cue = Some(Cue::Sleep);
                doze
            }
            _ => Beat::new(Gesture::Sit, ExpressionKind::Content, 600.0),
        }
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        let out = !matches!(
            self.phase,
            Phase::Arriving { .. } | Phase::Leaving { .. } | Phase::Over
        );
        if out {
            self.light -= LIGHT_PER_SEC * dt;
        }
        match self.phase {
            Phase::Arriving { since } => {
                let settled = self.party.iter().all(|id| !ground.busy(*id));
                if settled || now - since > 6.0 {
                    self.settle(ground, now);
                    self.phase = Phase::Watching {
                        next_at: now + FIRST_SECS,
                    };
                }
            }
            Phase::Watching { next_at } => {
                if self.next >= self.coming.len() {
                    self.events.push(Event::AllDown);
                    self.leave(ground, now);
                } else if now >= next_at {
                    self.glint(ground, now);
                }
            }
            Phase::Glinting { since } => {
                let tier = self.current().map_or(Tier::Rare, |find| find.tier);
                if now - since >= sign_secs(tier) {
                    if self.reduce_motion {
                        // A cut: there at the lip, and then round in the eddy.
                        self.round(now);
                    } else {
                        self.phase = Phase::Falling { since: now };
                    }
                }
            }
            Phase::Falling { since } => {
                if now - since >= FALL_SECS {
                    self.round(now);
                }
            }
            Phase::Circling(_) => {}
            Phase::Caught { since } => {
                if now - since >= REVEAL_SECS {
                    self.next += 1;
                    self.spotted = None;
                    self.resume(ground, now);
                    self.phase = Phase::Watching {
                        next_at: now + self.dice.range(BETWEEN.0, BETWEEN.1),
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
        let waiting = matches!(
            self.phase,
            Phase::Watching { .. }
                | Phase::Glinting { .. }
                | Phase::Falling { .. }
                | Phase::Circling(_)
        );
        if self.light <= 0.0 && waiting {
            self.leave(ground, now);
        }
    }

    /// Something glints at the lip; anyone curious may spot it and point.
    fn glint(&mut self, ground: &mut Playground, now: f32) {
        let Some(find) = self.coming.get(self.next).copied() else {
            return;
        };
        self.spotted = None;
        for (id, character) in self.party.iter().zip(&self.characters) {
            if crew::curious(character) && self.dice.chance(0.4 + 0.5 * character.axes.curiosity) {
                self.spotted = Some(*id);
                let mut point = Beat::new(Gesture::Reach, ExpressionKind::Curious, 0.9);
                point.cue = Some(Cue::Exclaim);
                ground.direct(
                    *id,
                    vec![
                        Step::FaceX(LIP.0),
                        Step::Beat(point),
                        Step::Beat(watching()),
                    ],
                    now,
                );
                break;
            }
        }
        self.events.push(Event::Glint {
            find: find.id,
            spotted: self.spotted,
        });
        self.phase = Phase::Glinting { since: now };
    }

    /// Into the pool, and round the eddy.
    fn round(&mut self, now: f32) {
        let tier = self.current().map_or(Tier::Rare, |find| find.tier);
        self.events.push(Event::Down);
        self.phase = Phase::Circling(Circle {
            since: now,
            period: ROUND_SECS * rarity_pace(tier),
            width: (ARC * rarity_arc(tier) * self.widen).min(WIDEST),
            misses: 0,
            held_angle: 0.0,
            holding: false,
        });
    }

    /// Where something going round is, `angle` round from the fall.
    fn on_the_eddy(angle: f32) -> (f32, f32) {
        let (cx, cy, rx, ry) = EDDY;
        (cx + angle.sin() * rx, cy - angle.cos() * ry)
    }

    /// The falls darkening as the day goes, the eddy and its moment, whatever is coming down or
    /// round, a find held up, and the basket.
    pub fn draw(&self, scene: &mut Canvas, now: f32) {
        dim(scene, self.light, self.hour_dark);
        let find = self.current();
        match self.phase {
            Phase::Glinting { since } => {
                if let Some(find) = find {
                    let t = ((now - since) / sign_secs(find.tier)).clamp(0.0, 1.0);
                    draw_glint(
                        scene,
                        find.tier,
                        self.spotted.is_some(),
                        t,
                        self.reduce_motion,
                    );
                }
            }
            Phase::Falling { since } => {
                if let Some(find) = find {
                    let t = ((now - since) / FALL_SECS).clamp(0.0, 1.0);
                    let y = LIP.1 + (FOOT.1 - LIP.1) * t * t;
                    let wobble = (t * 9.0).sin() * 2.0;
                    draw_thing(scene, find, (LIP.0 + wobble, y));
                }
            }
            Phase::Circling(circle) => {
                draw_eddy(scene, &circle);
                if let Some(find) = find {
                    let angle = circle.angle(now, self.reduce_motion);
                    let at = Self::on_the_eddy(angle);
                    draw_wake(scene, at, angle);
                    draw_thing(scene, find, at);
                }
            }
            Phase::Caught { since } => {
                if let Some(find) = find {
                    let rise = if self.reduce_motion {
                        8.0
                    } else {
                        ((now - since) / 0.4).min(1.0) * 8.0
                    };
                    let (x, y) = (WADE.0, WADE.1 - 40.0 - rise);
                    let icon = art::icon(find.id);
                    let (left, top) = (x as i32 - 4, y as i32);
                    rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
                    blit(scene, &icon, left, top);
                    for (dx, dy) in [(-5, -3), (11, 1), (3, -6), (12, -4)] {
                        put(scene, left + dx, top + dy, rgb(0xfff4c0));
                    }
                }
            }
            _ => {}
        }
        draw_basket(scene, &self.basket, self.light_left());
    }
}

/// Wading in at the stone, intent on the water, for as long as it takes.
fn wading() -> Beat {
    Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0)
}

/// Watching the water from the shingle.
fn watching() -> Beat {
    Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0)
}

/// How long the glint shows at the lip before it comes over: the rarer, the briefer.
fn sign_secs(tier: Tier) -> f32 {
    match tier {
        Tier::Common | Tier::Uncommon | Tier::Rare => 1.7,
        Tier::Exceptional => 1.2,
    }
}

/// How much narrower the moment is for rarer things.
fn rarity_arc(tier: Tier) -> f32 {
    match tier {
        Tier::Common | Tier::Uncommon => 1.0,
        Tier::Rare => 0.75,
        Tier::Exceptional => 0.6,
    }
}

/// How much quicker the eddy brings rarer things round.
fn rarity_pace(tier: Tier) -> f32 {
    match tier {
        Tier::Common | Tier::Uncommon => 1.0,
        Tier::Rare => 0.85,
        Tier::Exceptional => 0.74,
    }
}

/// A glint at the lip, `t` of the way through showing: a cross of light, plainer if somebody
/// spotted it and fainter the rarer it is. With reduced motion, a still mark.
fn draw_glint(scene: &mut Canvas, tier: Tier, spotted: bool, t: f32, reduce_motion: bool) {
    let mut alpha = match tier {
        Tier::Exceptional => 140.0,
        _ => 210.0,
    };
    if spotted {
        alpha = 245.0;
    }
    let bright = if reduce_motion {
        1.0
    } else {
        (t * PI).sin().max(0.2)
    };
    let glint = rgba(0xfff4c0, (alpha * bright) as u8);
    let (x, y) = (LIP.0 as i32, LIP.1 as i32);
    put(scene, x, y, rgba(0xffffff, (alpha * bright) as u8));
    for step in 1..=2 {
        for (dx, dy) in [(step, 0), (-step, 0), (0, step), (0, -step)] {
            put(scene, x + dx, y + dy, glint);
        }
    }
    if spotted {
        for (dx, dy) in [(-4, -4), (4, -4), (-4, 4), (4, 4)] {
            put(
                scene,
                x + dx,
                y + dy,
                rgba(0xfff4c0, (120.0 * bright) as u8),
            );
        }
    }
}

/// Something coming down or going round: its icon, small, with a glint on it.
fn draw_thing(scene: &mut Canvas, find: &Find, (x, y): (f32, f32)) {
    let icon = art::icon(find.id);
    let (left, top) = (x as i32 - 4, y as i32 - 6);
    blit(scene, &icon, left, top);
    put(scene, left + 1, top + 1, rgba(0xffffff, 200));
}

/// The water parting round something carried on the eddy: a little V behind it.
fn draw_wake(scene: &mut Canvas, (x, y): (f32, f32), angle: f32) {
    let foam = rgba(0xe9f6f7, 170);
    // Which way it is going: clockwise round the eddy, so along its tangent.
    let (dx, dy) = (angle.cos(), angle.sin() * 0.25);
    for step in 1..=4 {
        let back = (x - dx * (step as f32 * 2.0 + 3.0), y - dy * step as f32);
        let spread = step as f32 * 0.8;
        put(scene, back.0 as i32, (back.1 - spread) as i32 + 2, foam);
        put(scene, back.0 as i32, (back.1 + spread) as i32 + 2, foam);
    }
}

/// The eddy, faint on the water, and the moment at the wading stone marked gold.
fn draw_eddy(scene: &mut Canvas, circle: &Circle) {
    let (cx, cy, rx, ry) = EDDY;
    let steps = 120;
    for step in 0..steps {
        let angle = step as f32 / steps as f32 * TAU;
        let (x, y) = (cx + angle.sin() * rx, cy - angle.cos() * ry);
        let (x, y) = (x.round() as i32, y.round() as i32);
        if off_the_stone(angle).abs() <= circle.width / 2.0 {
            put(scene, x, y, rgb(0xf5d25e));
            put(scene, x, y + 1, rgb(0xb8862a));
        } else if step % 3 == 0 {
            put(scene, x, y, rgba(0xe9f6f7, 90));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::falls;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn visit(cast: &Cast, party: Vec<Id>, seed: u64) -> (Playground, Wading) {
        let mut ground = falls::open(cast, &party, 0.0);
        let outset = Outset {
            party,
            drought: 0,
            close_pair: false,
            influence: Influence::default(),
            seed,
        };
        let wading = Wading::new(&mut ground, outset, |_| false, 0.0);
        (ground, wading)
    }

    fn run(
        ground: &mut Playground,
        wading: &mut Wading,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            wading.tick(ground, now);
            events.extend(wading.take_events());
        }
        events
    }

    /// Plays on until something is going round, and says when.
    fn until_circling(ground: &mut Playground, wading: &mut Wading, cast: &Cast, from: f32) -> f32 {
        let mut now = from;
        while !matches!(wading.phase(), Phase::Circling(_)) {
            assert!(now < from + 60.0, "nothing came down: {:?}", wading.phase());
            run(ground, wading, cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
        }
        now
    }

    /// The next moment from `now` it is coming past the middle of the stone.
    fn when_at_the_stone(circle: &Circle, now: f32) -> f32 {
        let mut when = circle.since + 0.5 * circle.period;
        while when < now {
            when += circle.period;
        }
        when
    }

    #[test]
    fn what_comes_round_is_caught_at_the_stone_and_goes_in_the_basket() {
        let cast = sample();
        let (mut ground, mut wading) = visit(&cast, vec![cast.members[0].id], 3);
        let now = until_circling(&mut ground, &mut wading, &cast, 0.0);
        let Phase::Circling(circle) = wading.phase() else {
            unreachable!();
        };
        let first = wading.coming()[0].id;
        wading.strike(&mut ground, when_at_the_stone(&circle, now));
        assert_eq!(wading.basket(), &[first]);
        assert!(afar::is_from_afar(first));
        // The second comes down after the first has been looked at, and then that's all.
        let now = until_circling(&mut ground, &mut wading, &cast, now + 0.1);
        let Phase::Circling(circle) = wading.phase() else {
            unreachable!();
        };
        wading.strike(&mut ground, when_at_the_stone(&circle, now));
        assert_eq!(wading.basket().len(), COMING);
        let events = run(&mut ground, &mut wading, &cast, now, now + 20.0);
        assert!(events.contains(&Event::AllDown), "{events:?}");
        assert_eq!(wading.phase(), Phase::Over);
    }

    #[test]
    fn a_miss_costs_light_and_the_water_slackens() {
        let cast = sample();
        let (mut ground, mut wading) = visit(&cast, vec![cast.members[1].id], 4);
        let now = until_circling(&mut ground, &mut wading, &cast, 0.0);
        let Phase::Circling(before) = wading.phase() else {
            unreachable!();
        };
        let light = wading.light();
        // Opposite the stone, where the fall comes in.
        let opposite = when_at_the_stone(&before, now) + before.period / 2.0;
        wading.strike(&mut ground, opposite);
        let Phase::Circling(after) = wading.phase() else {
            panic!("{:?}", wading.phase());
        };
        assert!(after.width > before.width && after.period > before.period);
        assert!(wading.light() < light);
        assert!(wading.basket().is_empty());
    }

    #[test]
    fn rarer_things_come_round_quicker_and_give_a_narrower_moment() {
        assert!(rarity_arc(Tier::Exceptional) < rarity_arc(Tier::Rare));
        assert!(rarity_pace(Tier::Exceptional) < rarity_pace(Tier::Rare));
        assert!(sign_secs(Tier::Exceptional) < sign_secs(Tier::Rare));
    }

    #[test]
    fn who_comes_along_widens_the_moment() {
        let cast = sample();
        let widen = |party: Vec<Id>, close_pair: bool| {
            let mut ground = falls::open(&cast, &party, 0.0);
            let outset = Outset {
                party,
                drought: 0,
                close_pair,
                influence: Influence::default(),
                seed: 1,
            };
            Wading::new(&mut ground, outset, |_| false, 0.0).widen
        };
        let one: Vec<f32> = cast
            .members
            .iter()
            .map(|m| widen(vec![m.id], false))
            .collect();
        assert!(one.windows(2).any(|pair| pair[0] != pair[1]));
        let pair = vec![cast.members[0].id, cast.members[2].id];
        assert!(widen(pair.clone(), false) > widen(vec![pair[0]], false));
        assert!(widen(pair.clone(), true) > widen(pair, false));
    }

    #[test]
    fn standing_about_costs_light_and_the_visit_ends_when_it_goes() {
        let cast = sample();
        let (mut ground, mut wading) = visit(&cast, vec![cast.members[0].id], 5);
        let events = run(&mut ground, &mut wading, &cast, 0.0, 600.0);
        assert!(events.contains(&Event::Leaving));
        assert_eq!(wading.phase(), Phase::Over);
        assert!(wading.light() <= 0.5, "{} light left", wading.light());
    }

    #[test]
    fn with_reduced_motion_the_fall_is_a_cut_and_it_only_comes_round_while_held() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut wading) = visit(&cast, vec![cast.members[0].id], 6);
        let mut now = 0.0;
        let mut fell = false;
        while !matches!(wading.phase(), Phase::Circling(_)) {
            assert!(now < 60.0);
            fell |= matches!(wading.phase(), Phase::Falling { .. });
            run(&mut ground, &mut wading, &cast, now, now + 1.0 / 30.0);
            now += 1.0 / 30.0;
        }
        assert!(!fell, "it tumbled down the fall");
        let Phase::Circling(circle) = wading.phase() else {
            unreachable!();
        };
        assert_eq!(
            circle.angle(now + 50.0, true),
            0.0,
            "it went round on its own"
        );
        // Hold until it is in the middle of the moment, then let go.
        let dt = 1.0 / 120.0;
        while let Phase::Circling(circle) = wading.phase() {
            if off_the_stone(circle.held_angle).abs() < 0.03 {
                break;
            }
            wading.hold(&mut ground, true, dt, now);
            now += dt;
        }
        wading.hold(&mut ground, false, dt, now);
        assert_eq!(wading.basket().len(), 1);
    }
}
