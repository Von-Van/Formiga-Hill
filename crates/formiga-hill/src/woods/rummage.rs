//! A rummage in the Woods: one outing with a companion or two, until the basket is full or the
//! light goes.
//!
//! The person plays. Spots with something in them give little signs now and then: soil crumbling,
//! a glint in a hollow, a ripple, leaves shivering. Common things show often and plainly; rare
//! ones faintly and seldom, and the rarest only once the light starts to go, so staying out longer
//! is how to find them, and the light is what it costs. Choosing a spot sends the companion there,
//! and walking costs a little light, so a sensible route matters. At the spot a ring turns, and
//! the person catches the moment it passes the bright arc; rarer things give a narrower arc and a
//! quicker turn. A miss costs a little light and the arc widens, so nobody is ever shut out. An
//! empty spot costs a rummage and gives nothing, which is why the signs are worth watching.
//!
//! Who came along matters. Each companion has a knack, read from its temperament, that widens the
//! arc for one kind of spot; a curious one points out signs it notices; a lively one gets about
//! for less light; two together catch things more easily, and a close pair more easily still. And
//! what is out there leans towards who came (see `finds`).

use super::{EXTRAS, Influence, Opener, SPOTS, Spot};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{self, Find, Kind, Tier, art};
use crate::paint::{blit, mix, put, rect, rgb, rgba};
use crate::playground::{Playground, Prop, distance};
use formiga_art::{Canvas, ExpressionKind, Rgba};
use formiga_core::{ActionKind, Gesture, TemperamentKind};
use std::f32::consts::{PI, TAU};

/// The light an outing starts with.
pub const LIGHT: f32 = 100.0;
/// How many finds the basket holds.
pub const BASKET: usize = 6;
/// What a try at catching something costs, and a rummage that turns up nothing.
const TRY_COST: f32 = 4.0;
const EMPTY_COST: f32 = 5.0;
/// Light per pixel walked, for a companion of middling liveliness.
const WALK_COST: f32 = 0.035;
/// Below this much light, rare things start to show themselves.
pub const DUSK: f32 = 55.0;
/// Below this much light, the glade starts to darken.
const DIMMING: f32 = 45.0;
/// How long a sign shows.
const SIGN_SECS: f32 = 1.2;
/// The catching ring, and its bright arc before anything widens or narrows it, in radians.
const RING: f32 = 13.0;
const ARC: f32 = 1.1;
const GRACE: f32 = 0.08;
/// How long a find is held up to look at.
const REVEAL_SECS: f32 = 1.8;
/// How far beside the leader a second companion stands.
const BESIDE: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The companions are walking in.
    Arriving {
        since: f32,
    },
    /// Waiting for the person to choose a spot.
    Exploring,
    Going {
        spot: usize,
    },
    Catching(Catch),
    /// Searching a spot with nothing in it.
    Rummaging {
        spot: usize,
        until: f32,
    },
    Found {
        spot: usize,
        since: f32,
    },
    Leaving {
        since: f32,
        why: Ending,
    },
    /// Everyone has gone home; the basket is ready to be kept.
    Over,
    /// They followed the glint, and are somewhere else now.
    Beckoned,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Catch {
    pub spot: usize,
    since: f32,
    /// The middle of the bright arc, clockwise from the top, and its width.
    centre: f32,
    width: f32,
    /// Seconds for the marker to go once round.
    period: f32,
    misses: u32,
    /// With reduced motion the marker only moves while the person holds it.
    held_angle: f32,
    holding: bool,
}

impl Catch {
    fn marker(&self, now: f32, reduce_motion: bool) -> f32 {
        if reduce_motion {
            self.held_angle
        } else {
            ((now - self.since) / self.period * TAU).rem_euclid(TAU)
        }
    }

    fn on_the_arc(&self, angle: f32) -> bool {
        let off = (angle - self.centre + PI).rem_euclid(TAU) - PI;
        off.abs() <= self.width / 2.0 + GRACE
    }
}

/// Why an outing ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Dusk,
    Full,
    Chose,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Someone in the party opened up a spot only they could.
    Opened {
        who: Id,
        spot: usize,
    },
    Noticed {
        who: Id,
        spot: usize,
    },
    Got {
        find: &'static str,
        spot: usize,
    },
    Slipped {
        spot: usize,
    },
    Nothing {
        spot: usize,
    },
    AlreadyLooked {
        spot: usize,
    },
    Leaving(Ending),
    /// Something out of place has appeared between the trees.
    Glint,
    /// They followed it.
    Beckoned,
}

#[derive(Clone, Copy, Debug, Default)]
struct Sign {
    next: f32,
    until: f32,
}

/// What an outing sets out with.
pub struct Outset {
    /// Who came along; the first leads.
    pub party: Vec<Id>,
    /// Outings in a row that found nothing new.
    pub drought: u32,
    /// Whether the two who came are close friends.
    pub close_pair: bool,
    /// What the Hilltop lends the Woods.
    pub influence: Influence,
    /// Whether, late in the day, something out of place may show itself.
    pub beckons: bool,
    pub seed: u64,
}

/// Where the glint shows, far off between the trees across the stream, and where to stand to look.
const GLINT: Spot = Spot {
    name: "something glinting between the trees",
    kind: Kind::Reach,
    sign: (262.0, 98.0),
    stand: (258.0, 140.0),
};
/// How late in the day it shows: below this share of the light.
const GLINT_LIGHT: f32 = 0.2;

pub struct Rummage {
    /// How dark the hour has made the glade already, so the outing's own dusk only darkens it
    /// past that.
    hour_dark: f32,
    /// Whether the glint may show, and where among the spots it is once it has.
    beckons: bool,
    glint: Option<usize>,
    party: Vec<Id>,
    /// The glade's spots, and any extra ones this party opened, each with who opened it.
    spots: Vec<Spot>,
    opened: Vec<(usize, Opener, Id)>,
    /// Whether each extra spot was searched when the glade last showed them.
    extras_shown: Option<Vec<bool>>,
    /// Who is searching the spot being searched: whoever opened it, or the first who came.
    leader: Option<Id>,
    /// The light the outing started with.
    full: f32,
    /// Below this much light, rare things start to show themselves.
    dusk: f32,
    caches: Vec<Option<&'static Find>>,
    searched: Vec<bool>,
    signs: Vec<Sign>,
    light: f32,
    basket: Vec<&'static str>,
    phase: Phase,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    /// Where the walk to the next spot starts from: the last spot searched, or the way in.
    from: (f32, f32),
    /// For each kind of spot, how much the party widens the arc.
    knack: [f32; 4],
    /// How much a second pair of paws helps.
    help: f32,
    /// How much light walking costs this party, per pixel.
    walk_cost: f32,
}

/// How much a companion widens the arc for one kind of spot, read from its temperament.
pub fn knack(character: &Character, kind: Kind) -> f32 {
    let a = character.axes;
    let little = f32::from(u8::from(character.parent.is_some()));
    let score = match kind {
        Kind::Dig => 0.5 * a.energy + 0.5 * a.feistiness,
        // Small paws for small holes.
        Kind::Reach => 0.6 * a.boldness + 0.4 * a.curiosity + 0.25 * little,
        Kind::Scoop => 0.6 * (1.0 - a.impulsiveness) + 0.4 * (1.0 - a.energy),
        Kind::Shake => 0.6 * a.playfulness + 0.4 * a.impulsiveness,
    };
    0.85 + 0.5 * score.clamp(0.0, 1.0)
}

/// What a companion is best at, for the person choosing who to bring.
pub fn best_at(character: &Character) -> Kind {
    Kind::ALL
        .into_iter()
        .max_by(|a, b| knack(character, *a).total_cmp(&knack(character, *b)))
        .unwrap_or(Kind::Dig)
}

impl Rummage {
    /// Sets out with the party, already standing in `ground`. `found_before` says which finds the
    /// colony has already.
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
            beckons,
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let refs: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(seed);
        let kinds: Vec<Kind> = SPOTS.iter().map(|spot| spot.kind).collect();
        let mut caches = finds::stock(
            &kinds,
            &refs,
            &found_before,
            drought,
            &influence.richer,
            &mut dice,
        );
        // The extra spots this company opens: always something there, and under the boulder the
        // better of two.
        let mut spots = SPOTS.to_vec();
        let mut opened = Vec::new();
        let mut events = Vec::new();
        for extra in EXTRAS {
            let Some(who) = opener(&party, &characters, extra.opener, close_pair) else {
                continue;
            };
            let mut roll = |stocked: &[_]| {
                finds::surely(extra.spot.kind, stocked, &refs, &found_before, &mut dice)
            };
            let mut cache = roll(&caches);
            if extra.opener == Opener::ClosePair {
                let other = roll(&caches);
                if other.map(|f| f.tier) > cache.map(|f| f.tier) {
                    cache = other;
                }
            }
            opened.push((spots.len(), extra.opener, who));
            events.push(Event::Opened {
                who,
                spot: spots.len(),
            });
            spots.push(extra.spot);
            caches.push(cache);
        }
        let mut knack_by_kind = [1.0; 4];
        for kind in Kind::ALL {
            knack_by_kind[kind.index()] = characters
                .iter()
                .map(|c| knack(c, kind))
                .fold(0.85, f32::max);
        }
        let help = match characters.len() {
            0 | 1 => 1.0,
            _ if close_pair => 1.3,
            _ => 1.15,
        };
        // The slowest sets the pace.
        let liveliness = characters.iter().map(|c| c.axes.energy).fold(1.0, f32::min);
        let mut signs = vec![Sign::default(); spots.len()];
        for sign in &mut signs {
            sign.next = now + 2.0 + dice.range(0.0, 3.0);
        }
        let full = LIGHT + influence.light;
        Self {
            party,
            searched: vec![false; spots.len()],
            spots,
            opened,
            extras_shown: None,
            leader: None,
            full,
            dusk: DUSK + influence.earlier,
            caches,
            signs,
            light: full,
            basket: Vec::new(),
            phase: Phase::Arriving { since: now },
            events,
            dice,
            reduce_motion: ground.reduce_motion(),
            hour_dark: 0.0,
            from: (-24.0, 192.0),
            knack: knack_by_kind,
            help,
            walk_cost: WALK_COST / (0.8 + 0.4 * liveliness),
            beckons,
            glint: None,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.party
    }

    /// One of this outing's spots: the glade's, or an extra one this party opened.
    pub fn spot(&self, index: usize) -> Option<&Spot> {
        self.spots.get(index)
    }

    pub fn spot_count(&self) -> usize {
        self.spots.len()
    }

    /// Who opened up an extra spot, if it is one.
    pub fn opener_of(&self, spot: usize) -> Option<Opener> {
        self.opened
            .iter()
            .find(|(at, _, _)| *at == spot)
            .map(|(_, opener, _)| *opener)
    }

    /// How much of the light is left, from 0 to 1.
    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    pub fn searched(&self, spot: usize) -> bool {
        self.searched.get(spot).copied().unwrap_or(false)
    }

    /// What has happened since last asked.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Whether the marker is on the bright arc right now, for a steady hand to catch.
    pub fn on_the_gold(&self, now: f32) -> bool {
        match self.phase {
            Phase::Catching(catch) => {
                let angle = catch.marker(now, self.reduce_motion);
                let off = (angle - catch.centre + PI).rem_euclid(TAU) - PI;
                off.abs() < catch.width / 4.0
            }
            _ => false,
        }
    }

    /// The light walking from where the party last searched to `spot` would cost.
    pub fn walk_cost(&self, spot: usize) -> f32 {
        self.spots
            .get(spot)
            .map_or(0.0, |spot| distance(self.from, spot.stand) * self.walk_cost)
    }

    /// The spot under a point in the scene, if any.
    pub fn spot_at(&self, x: f32, y: f32) -> Option<usize> {
        self.spots
            .iter()
            .enumerate()
            .map(|(index, spot)| {
                let near = distance((x, y), spot.sign).min(distance((x, y), spot.stand) + 4.0);
                (index, near)
            })
            .filter(|(_, near)| *near <= 18.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The person chose a spot: the party goes over to search it.
    pub fn choose(&mut self, ground: &mut Playground, spot: usize, now: f32) {
        let ready = matches!(self.phase, Phase::Exploring | Phase::Catching(_));
        let Some(place) = self.spots.get(spot).copied() else {
            return;
        };
        if !ready {
            return;
        }
        if self.searched[spot] {
            self.events.push(Event::AlreadyLooked { spot });
            return;
        }
        self.light -= self.walk_cost(spot);
        self.from = place.stand;
        ground.reserve(self.party.clone());
        let face = place.sign.0 + 0.5;
        // Whoever opened an extra spot searches it; otherwise the first who came.
        let leader = self
            .opened
            .iter()
            .find(|(at, _, _)| *at == spot)
            .map(|(_, _, who)| *who)
            .or_else(|| self.party.first().copied());
        self.leader = leader;
        let order: Vec<Id> = leader
            .into_iter()
            .chain(self.party.iter().copied().filter(|id| Some(*id) != leader))
            .collect();
        for (index, id) in order.iter().enumerate() {
            let to = if index == 0 {
                place.stand
            } else {
                beside(ground, place.stand)
            };
            let mut steps = Vec::new();
            if index == 0 && self.party.len() > 1 {
                steps.push(Step::Stride { to });
            } else {
                steps.push(Step::Walk { to });
            }
            steps.push(Step::FaceX(face));
            ground.direct(*id, steps, now);
        }
        self.phase = Phase::Going { spot };
    }

    /// The person caught the moment, or tried to.
    pub fn strike(&mut self, ground: &mut Playground, now: f32) {
        let Phase::Catching(mut catch) = self.phase else {
            return;
        };
        let angle = catch.marker(now, self.reduce_motion);
        self.light -= TRY_COST;
        let Some(find) = self.caches[catch.spot] else {
            return;
        };
        if catch.on_the_arc(angle) {
            self.searched[catch.spot] = true;
            self.basket.push(find.id);
            self.events.push(Event::Got {
                find: find.id,
                spot: catch.spot,
            });
            for id in &self.party {
                if let Some(character) = ground.character(*id) {
                    let beats = delighted(character, find.tier);
                    ground.direct(*id, beats.into_iter().map(Step::Beat).collect(), now);
                }
            }
            self.phase = Phase::Found {
                spot: catch.spot,
                since: now,
            };
        } else {
            // It slipped deeper. Try again: the arc is a little kinder each time.
            catch.misses += 1;
            catch.width = (catch.width * 1.15).min(2.6);
            catch.since = now;
            catch.held_angle = 0.0;
            self.events.push(Event::Slipped { spot: catch.spot });
            if let Some(leader) = self.leader.as_ref() {
                let mut oops = Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.4);
                oops.cue = Some(Cue::Exclaim);
                let steps = vec![Step::Beat(oops), Step::Beat(searching(find.kind))];
                ground.direct(*leader, steps, now);
            }
            self.phase = Phase::Catching(catch);
            if self.light <= 0.0 {
                self.leave(ground, Ending::Dusk, now);
            }
        }
    }

    /// With reduced motion, the person moves the marker round by holding, and lets go to catch.
    pub fn hold(&mut self, ground: &mut Playground, holding: bool, dt: f32, now: f32) {
        if !self.reduce_motion {
            return;
        }
        let Phase::Catching(mut catch) = self.phase else {
            return;
        };
        if holding {
            catch.held_angle = (catch.held_angle + dt / catch.period * TAU).rem_euclid(TAU);
            catch.holding = true;
            self.phase = Phase::Catching(catch);
        } else if catch.holding {
            catch.holding = false;
            self.phase = Phase::Catching(catch);
            self.strike(ground, now);
        }
    }

    /// Calls the outing to an end: everyone heads home with what is in the basket.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.leave(ground, Ending::Chose, now);
        }
    }

    fn leave(&mut self, ground: &mut Playground, why: Ending, now: f32) {
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let tired = ground
                .character(*id)
                .is_some_and(|c| c.kind == TemperamentKind::Lazybones)
                && why == Ending::Dusk;
            let mut steps = Vec::new();
            if tired {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Yawn,
                    ExpressionKind::Yawning,
                    1.0,
                )));
            }
            steps.push(Step::Walk {
                to: (-30.0 - 20.0 * index as f32, 194.0),
            });
            ground.direct(*id, steps, now);
        }
        self.events.push(Event::Leaving(why));
        self.phase = Phase::Leaving { since: now, why };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let searched: Vec<bool> = self
            .opened
            .iter()
            .map(|(i, _, _)| self.searched[*i])
            .collect();
        if self.extras_shown.as_ref() != Some(&searched) {
            ground.set_props(self.extras());
            self.extras_shown = Some(searched);
        }
        self.update_signs(ground, now);
        if self.beckons
            && self.glint.is_none()
            && self.light < self.full * GLINT_LIGHT
            && matches!(self.phase, Phase::Exploring)
        {
            self.glint = Some(self.spots.len());
            self.spots.push(GLINT);
            self.caches.push(None);
            self.searched.push(false);
            self.signs.push(Sign::default());
            self.events.push(Event::Glint);
        }
        let leader = self.leader.or_else(|| self.party.first().copied());
        let busy = |ground: &Playground| leader.is_some_and(|id| ground.busy(id));
        match self.phase {
            Phase::Arriving { since } => {
                if !busy(ground) || now - since > 5.0 {
                    self.phase = Phase::Exploring;
                }
            }
            Phase::Exploring => {
                if self.light <= 0.0 {
                    self.leave(ground, Ending::Dusk, now);
                }
            }
            Phase::Going { spot } => {
                if busy(ground) {
                    return;
                }
                if self.glint == Some(spot) {
                    self.events.push(Event::Beckoned);
                    self.phase = Phase::Beckoned;
                    return;
                }
                let kind = self.spots[spot].kind;
                if let Some(leader) = leader {
                    ground.direct(leader, vec![Step::Beat(searching(kind))], now);
                }
                for helper in self.party.iter().filter(|id| Some(**id) != leader) {
                    let watch = Beat::new(Gesture::Watch, ExpressionKind::Curious, 600.0);
                    ground.direct(*helper, vec![Step::Beat(watch)], now);
                }
                match self.caches[spot] {
                    Some(find) => {
                        let centre = self.dice.range(PI * 0.6, PI * 1.8);
                        self.phase = Phase::Catching(Catch {
                            spot,
                            since: now,
                            centre,
                            width: ARC
                                * rarity_arc(find.tier)
                                * self.knack[kind.index()]
                                * self.help,
                            period: 1.5 * rarity_pace(find.tier),
                            misses: 0,
                            held_angle: 0.0,
                            holding: false,
                        });
                    }
                    None => {
                        self.light -= EMPTY_COST;
                        self.phase = Phase::Rummaging {
                            spot,
                            until: now + 1.3,
                        };
                    }
                }
            }
            Phase::Catching(_) => {}
            Phase::Rummaging { spot, until } => {
                if now < until {
                    return;
                }
                self.searched[spot] = true;
                self.events.push(Event::Nothing { spot });
                for id in &self.party {
                    if let Some(character) = ground.character(*id) {
                        let beats = empty_handed(character);
                        ground.direct(*id, beats.into_iter().map(Step::Beat).collect(), now);
                    }
                }
                self.carry_on(ground, now);
            }
            Phase::Found { since, .. } => {
                if now - since >= REVEAL_SECS {
                    self.carry_on(ground, now);
                }
            }
            Phase::Leaving { since, .. } => {
                let gone = self.party.iter().all(|id| !ground.busy(*id));
                if gone || now - since > 8.0 {
                    self.phase = Phase::Over;
                }
            }
            Phase::Over | Phase::Beckoned => {}
        }
    }

    /// After a search: back to choosing, unless the basket is full or the light has gone.
    fn carry_on(&mut self, ground: &mut Playground, now: f32) {
        if self.basket.len() >= BASKET {
            self.leave(ground, Ending::Full, now);
        } else if self.light <= 0.0 {
            self.leave(ground, Ending::Dusk, now);
        } else {
            ground.release();
            self.phase = Phase::Exploring;
        }
    }

    /// Signs come and go at the spots that hold something not yet found. A curious companion
    /// with nothing else to do may notice one and point it out.
    fn update_signs(&mut self, ground: &mut Playground, now: f32) {
        let exploring = matches!(self.phase, Phase::Exploring);
        for (index, spot) in self.spots.clone().iter().enumerate() {
            let Some(find) = self.caches[index] else {
                continue;
            };
            if self.searched[index] || now < self.signs[index].next {
                continue;
            }
            let (low, high) = match find.tier {
                Tier::Common => (2.5, 4.5),
                Tier::Uncommon => (4.0, 7.0),
                Tier::Rare => (6.0, 10.0),
                Tier::Exceptional => (9.0, 14.0),
            };
            self.signs[index].next = now + self.dice.range(low, high);
            // The rarest only show themselves as the light goes.
            if find.tier >= Tier::Rare && self.light > self.dusk {
                continue;
            }
            self.signs[index].until = now + SIGN_SECS;
            if !exploring {
                continue;
            }
            for id in self.party.clone() {
                let Some(character) = ground.character(id) else {
                    continue;
                };
                let near = ground
                    .position(id)
                    .is_some_and(|at| distance(at, spot.sign) < 170.0);
                let chance = 0.2 + 0.6 * character.axes.curiosity;
                if near && !ground.busy(id) && self.dice.chance(chance) {
                    let mut point = Beat::new(Gesture::Reach, ExpressionKind::Curious, 0.9);
                    point.cue = Some(Cue::Exclaim);
                    let steps = vec![Step::FaceX(spot.sign.0), Step::Beat(point)];
                    ground.direct(id, steps, now);
                    self.events.push(Event::Noticed {
                        who: id,
                        spot: index,
                    });
                    break;
                }
            }
        }
    }

    /// The extra spots this party opened, standing in the glade among everyone, so a companion
    /// in front of one is drawn in front of it: the boulder as it lies, or rolled once searched.
    fn extras(&self) -> Vec<Prop> {
        self.opened
            .iter()
            .map(|(index, opener, _)| {
                let spot = &self.spots[*index];
                let searched = self.searched[*index];
                standing(extra_base(*opener, searched), |scene| {
                    draw_extra(scene, spot, *opener, searched)
                })
            })
            .collect()
    }

    /// How dark the hour has made the glade already: the outing's dusk adds only what is more.
    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
    }

    /// The glade darkening as the light goes, the signs, the ring, a find held up, the basket.
    pub fn draw(&self, scene: &mut Canvas, now: f32) {
        dim(scene, self.light, self.hour_dark);
        if let Some(index) = self.glint {
            draw_glint(scene, self.spots[index].sign, now, self.reduce_motion);
        }
        for (index, spot) in self.spots.iter().enumerate() {
            let sign = self.signs[index];
            if now < sign.until
                && !self.searched[index]
                && let Some(find) = self.caches[index]
            {
                let t = 1.0 - (sign.until - now) / SIGN_SECS;
                draw_sign(scene, spot, find.tier, t, self.reduce_motion);
            }
        }
        match self.phase {
            Phase::Catching(catch) => {
                draw_ring(
                    scene,
                    &catch,
                    self.spots[catch.spot].sign,
                    now,
                    self.reduce_motion,
                );
            }
            Phase::Found { spot, since } => {
                if let Some(find) = self.caches[spot] {
                    let rise = if self.reduce_motion {
                        8.0
                    } else {
                        ((now - since) / 0.4).min(1.0) * 8.0
                    };
                    let (x, y) = self.spots[spot].sign;
                    let icon = art::icon(find.id);
                    let (left, top) = (x as i32 - 4, (y - 6.0 - rise) as i32);
                    // A little card behind it, so it reads against anything.
                    rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
                    blit(scene, &icon, left, top);
                    if find.tier >= Tier::Rare {
                        for (dx, dy) in [(-5, -3), (11, 1), (3, -6)] {
                            put(scene, left + dx, top + dy, rgb(0xfff4c0));
                        }
                    }
                }
            }
            _ => {}
        }
        self.draw_basket(scene);
    }

    /// The basket in the top right corner: a slot for each find it holds, and the light left.
    fn draw_basket(&self, scene: &mut Canvas) {
        const SLOT: i32 = 11;
        let width = SLOT * BASKET as i32 + 3;
        let (left, top) = (scene.width() as i32 - width - 4, 4);
        rect(scene, left, top, width, SLOT + 7, rgba(0x2a2018, 150));
        for slot in 0..BASKET as i32 {
            let (x, y) = (left + 2 + slot * SLOT, top + 2);
            rect(scene, x, y, SLOT - 1, SLOT - 1, rgba(0xf6eed8, 60));
            if let Some(id) = self.basket.get(slot as usize) {
                blit(scene, &art::icon(id), x, y);
            }
        }
        // The light left, gold going to dusk.
        let bar = ((width - 4) as f32 * (self.light / self.full).clamp(0.0, 1.0)) as i32;
        let gold = mix(
            rgb(0x6a5a9a),
            rgb(0xf5d25e),
            (self.light / self.full).clamp(0.0, 1.0),
        );
        scene.fill_rect(left + 2, top + SLOT + 2, bar, 2, gold);
    }
}

/// Where a second companion stands to help at a spot: towards the middle of the glade, unless the
/// log or the oak is in the way there.
fn beside(ground: &Playground, stand: (f32, f32)) -> (f32, f32) {
    let side = if stand.0 > 190.0 { -BESIDE } else { BESIDE };
    ground.beside_point((stand.0, stand.1 + 3.0), side)
}

/// Who in the party opens up an extra spot, if anyone: the most curious explorer, the first
/// little one, or the first of a close pair.
fn opener(party: &[Id], characters: &[Character], opener: Opener, close_pair: bool) -> Option<Id> {
    let with = |test: &dyn Fn(&Character) -> bool| {
        party
            .iter()
            .zip(characters)
            .filter(|(_, c)| test(c))
            .max_by(|a, b| a.1.axes.curiosity.total_cmp(&b.1.axes.curiosity))
            .map(|(id, _)| *id)
    };
    match opener {
        Opener::Explorer => {
            with(&|c| c.kind == TemperamentKind::Explorer || c.axes.curiosity >= 0.8)
        }
        Opener::LittleOne => with(&|c| c.parent.is_some()),
        Opener::ClosePair => close_pair.then(|| party.first().copied()).flatten(),
    }
}

/// The place an extra spot opens on, painted to the glade's own standard: a badger's sett under
/// the oak's roots, a crevice where a root leaves the trunk, a boulder furred with moss (rolled
/// aside once searched).
fn draw_extra(scene: &mut Canvas, spot: &Spot, opener: Opener, searched: bool) {
    use super::scenery::{badger_sett, mossy_boulder, root_crevice};
    match opener {
        Opener::Explorer => badger_sett(scene, spot.sign),
        Opener::LittleOne => root_crevice(scene, spot.sign),
        Opener::ClosePair => mossy_boulder(scene, spot.sign, searched),
    }
}

/// The row an extra spot stands on: anyone further down the picture is in front of it.
fn extra_base(opener: Opener, searched: bool) -> f32 {
    match (opener, searched) {
        (Opener::Explorer, _) => 198.0,
        (Opener::LittleOne, _) => 186.0,
        (Opener::ClosePair, false) => 194.0,
        (Opener::ClosePair, true) => 186.0,
    }
}

/// Something painted onto the glade, made into a prop that stands with its foot on `base`.
fn standing(base: f32, paint: impl FnOnce(&mut Canvas)) -> Prop {
    let mut layer = Canvas::new(crate::station::SCENE_WIDTH, crate::station::SCENE_HEIGHT);
    paint(&mut layer);
    let Some((left, top, right, bottom)) = layer.alpha_bounds() else {
        return Prop::new(Canvas::new(1, 1), (0, 0), base);
    };
    let (left, top) = (left as i32, top as i32);
    let mut sprite = Canvas::new(
        (right as i32 - left + 1) as u32,
        (bottom as i32 - top + 1) as u32,
    );
    for y in 0..sprite.height() as i32 {
        for x in 0..sprite.width() as i32 {
            sprite.set(x, y, layer.get(left + x, top + y));
        }
    }
    Prop::new(sprite, (left, top), base)
}

/// The glint: a small white pointer, outlined in black, where nothing like it should be, pulsing.
fn draw_glint(scene: &mut Canvas, at: (f32, f32), now: f32, reduce_motion: bool) {
    const ARROW: [&str; 8] = [
        "#....", "##...", "#o#..", "#oo#.", "#ooo#", "#o##.", "##.#.", "#..#.",
    ];
    let pulse = if reduce_motion {
        1.0
    } else {
        0.6 + 0.4 * (now * 3.0).sin()
    };
    let (x, y) = (at.0 as i32 - 2, at.1 as i32 - 4);
    for (dy, row) in ARROW.iter().enumerate() {
        for (dx, cell) in row.bytes().enumerate() {
            let color = match cell {
                b'#' => rgba(0x0c0a10, (255.0 * pulse) as u8),
                b'o' => rgba(0xffffff, (255.0 * pulse) as u8),
                _ => continue,
            };
            put(scene, x + dx as i32, y + dy as i32, color);
        }
    }
    if !reduce_motion && (now * 2.0).fract() < 0.3 {
        for (dx, dy) in [(-3, -2), (6, 1), (2, -5)] {
            put(scene, x + dx, y + dy, rgba(0xfff4c0, 200));
        }
    }
}

/// How much narrower the arc is for rarer things.
fn rarity_arc(tier: Tier) -> f32 {
    match tier {
        Tier::Common => 1.0,
        Tier::Uncommon => 0.8,
        Tier::Rare => 0.62,
        Tier::Exceptional => 0.5,
    }
}

/// How much quicker the ring turns for rarer things.
fn rarity_pace(tier: Tier) -> f32 {
    match tier {
        Tier::Common => 1.0,
        Tier::Uncommon => 0.92,
        Tier::Rare => 0.85,
        Tier::Exceptional => 0.78,
    }
}

/// The leader at work on a spot, for as long as it takes.
fn searching(kind: Kind) -> Beat {
    match kind {
        Kind::Dig => Beat::new(Gesture::Crouch, ExpressionKind::Determined, 600.0),
        Kind::Reach => Beat::new(Gesture::Reach, ExpressionKind::Focused, 600.0),
        Kind::Scoop => Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0),
        Kind::Shake => Beat::new(Gesture::Bop, ExpressionKind::Determined, 600.0),
    }
}

/// Something found! Each takes it its own way, and the rare ones are celebrated.
pub(crate) fn delighted(character: &Character, tier: Tier) -> Vec<Beat> {
    let with = |mut beat: Beat, cue: Cue| {
        beat.cue = Some(cue);
        beat
    };
    let mut beats = Vec::new();
    if tier >= Tier::Rare {
        beats.push(with(character.celebrate(1.4), Cue::Sparkle));
    }
    beats.push(match character.kind {
        TemperamentKind::Showoff => with(
            Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2),
            Cue::Sparkle,
        ),
        TemperamentKind::Scholar => Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.2),
        TemperamentKind::Grump => with(
            Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.0),
            Cue::Huff,
        ),
        TemperamentKind::Lazybones => Beat::new(Gesture::Yawn, ExpressionKind::Content, 1.0),
        TemperamentKind::Sweetheart => with(
            Beat::new(Gesture::Beg, ExpressionKind::Affectionate, 1.0),
            Cue::Heart,
        ),
        TemperamentKind::Wallflower => Beat::new(Gesture::Peek, ExpressionKind::Content, 1.0),
        TemperamentKind::Oddball => {
            let mut twirl = Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.0);
            twirl.spin = true;
            twirl
        }
        TemperamentKind::Troublemaker => Beat::new(Gesture::Bop, ExpressionKind::Smug, 1.0),
        TemperamentKind::Guardian => Beat::new(ActionKind::Idle, ExpressionKind::Determined, 0.9),
        TemperamentKind::Explorer => Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.0),
    });
    beats
}

/// Nothing there.
fn empty_handed(character: &Character) -> Vec<Beat> {
    vec![match character.kind {
        TemperamentKind::Grump => {
            let mut stomp = Beat::new(Gesture::Stomp, ExpressionKind::Grumpy, 0.8);
            stomp.cue = Some(Cue::Huff);
            stomp
        }
        TemperamentKind::Lazybones => Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.0),
        TemperamentKind::Explorer => Beat::new(Gesture::Watch, ExpressionKind::Curious, 0.8),
        _ => Beat::new(ActionKind::Idle, ExpressionKind::Bored, 0.8),
    }]
}

/// The glade darkening towards dusk as the light runs out.
pub(crate) fn dim(scene: &mut Canvas, light: f32, already: f32) {
    let amount = ((DIMMING - light) / DIMMING).clamp(0.0, 1.0) * 0.45 - already;
    if amount <= 0.0 {
        return;
    }
    let dusk = rgb(0x2a2848);
    for y in 0..scene.height() as i32 {
        for x in 0..scene.width() as i32 {
            scene.set(x, y, mix(scene.get(x, y), dusk, amount));
        }
    }
}

/// A sign at a spot, `t` of the way through showing. Rarer things show more faintly. With reduced
/// motion each is a still mark that comes and goes.
fn draw_sign(scene: &mut Canvas, spot: &Spot, tier: Tier, t: f32, reduce_motion: bool) {
    let alpha = match tier {
        Tier::Common => 255,
        Tier::Uncommon => 215,
        Tier::Rare => 160,
        Tier::Exceptional => 120,
    };
    let (x, y) = (spot.sign.0 as i32, spot.sign.1 as i32);
    let bounce = if reduce_motion { 0.5 } else { t };
    let ink = |hex: u32| -> Rgba { rgba(hex, alpha) };
    match spot.kind {
        // Crumbs of soil popping up.
        Kind::Dig => {
            for (dx, lift) in [(-3, 1.0), (0, 1.6), (3, 1.2)] {
                let up = ((bounce * PI).sin() * 4.0 * lift) as i32;
                put(scene, x + dx, y - up, ink(0x5a3a22));
                put(scene, x + dx, y - up - 1, ink(0x8a6040));
            }
        }
        // A glint in the dark.
        Kind::Reach => {
            let size = if tier >= Tier::Rare { 1 } else { 2 };
            let bright = if reduce_motion {
                1.0
            } else {
                (bounce * PI).sin()
            };
            let glint = rgba(0xfff4c0, (f32::from(alpha) * bright) as u8);
            put(scene, x, y, glint);
            for step in 1..=size {
                for (dx, dy) in [(step, 0), (-step, 0), (0, step), (0, -step)] {
                    put(scene, x + dx, y + dy, glint);
                }
            }
        }
        // Ripples spreading on the water.
        Kind::Scoop => {
            let radius = if reduce_motion {
                5.0
            } else {
                2.0 + bounce * 7.0
            };
            let fade = if reduce_motion {
                1.0
            } else {
                1.0 - bounce * 0.7
            };
            let ring = rgba(0xe9f6f7, (f32::from(alpha) * fade) as u8);
            for step in 0..28 {
                let angle = step as f32 / 28.0 * TAU;
                let (dx, dy) = (angle.cos() * radius, angle.sin() * radius * 0.4);
                put(scene, x + dx as i32, y + dy as i32, ring);
            }
            put(scene, x, y - 1, ring);
        }
        // Leaves shivering.
        Kind::Shake => {
            let shake = if reduce_motion {
                0
            } else if (t * 12.0) as i32 % 2 == 0 {
                1
            } else {
                -1
            };
            for (dx, dy) in [(-4, -2), (-1, -4), (3, -3), (5, 0), (-3, 2), (1, 1)] {
                put(scene, x + dx + shake, y + dy, ink(0xb6e08a));
            }
            let fall = if reduce_motion {
                3
            } else {
                (bounce * 8.0) as i32
            };
            put(scene, x + 2, y + 2 + fall, ink(0xd8c060));
        }
    }
}

/// The catching ring: a pale circle, its bright arc, and the marker going round.
fn draw_ring(scene: &mut Canvas, catch: &Catch, at: (f32, f32), now: f32, reduce_motion: bool) {
    let point = |angle: f32, radius: f32| {
        (
            (at.0 + angle.sin() * radius).round() as i32,
            (at.1 - angle.cos() * radius).round() as i32,
        )
    };
    let steps = 96;
    for step in 0..steps {
        let angle = step as f32 / steps as f32 * TAU;
        let off = (angle - catch.centre + PI).rem_euclid(TAU) - PI;
        let bright = off.abs() <= catch.width / 2.0;
        let (x, y) = point(angle, RING);
        if bright {
            put(scene, x, y, rgb(0xf5d25e));
            let (ix, iy) = point(angle, RING - 1.0);
            put(scene, ix, iy, rgb(0xe0a82a));
            let (ox, oy) = point(angle, RING + 1.0);
            put(scene, ox, oy, rgb(0x9a6a1a));
        } else {
            put(scene, x, y, rgba(0xf6eed8, 150));
        }
    }
    let (mx, my) = point(catch.marker(now, reduce_motion), RING);
    scene.fill_rect(mx - 2, my - 2, 5, 5, rgb(0x3a2a40));
    scene.fill_rect(mx - 1, my - 1, 3, 3, rgb(0xffffff));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::woods;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn outing(cast: &Cast, party: Vec<Id>) -> (Playground, Rummage) {
        let mut ground = woods::open(cast, &party, 0.0);
        let outset = Outset {
            party,
            drought: 0,
            close_pair: false,
            influence: Influence::default(),
            beckons: false,
            seed: 11,
        };
        let rummage = Rummage::new(&mut ground, outset, |_| false, 0.0);
        (ground, rummage)
    }

    fn run(
        ground: &mut Playground,
        rummage: &mut Rummage,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            rummage.tick(ground, now);
            events.extend(rummage.take_events());
        }
        events
    }

    /// The next moment from `now` the marker is at `angle`.
    fn when_at(catch: &Catch, angle: f32, now: f32) -> f32 {
        let mut when = catch.since + angle.rem_euclid(TAU) / TAU * catch.period;
        while when < now {
            when += catch.period;
        }
        when
    }

    /// Strikes at the very middle of the arc, as a sure hand would.
    fn catch_it(ground: &mut Playground, rummage: &mut Rummage, now: f32) {
        let Phase::Catching(catch) = rummage.phase else {
            panic!("not catching: {:?}", rummage.phase);
        };
        rummage.strike(ground, when_at(&catch, catch.centre, now));
    }

    #[test]
    fn a_good_catch_puts_the_find_in_the_basket() {
        let cast = sample();
        let party = vec![cast.members[0].id];
        let (mut ground, mut rummage) = outing(&cast, party);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        assert_eq!(rummage.phase, Phase::Exploring);
        let spot = rummage.caches.iter().position(Option::is_some).unwrap();
        rummage.choose(&mut ground, spot, 6.0);
        run(&mut ground, &mut rummage, &cast, 6.0, 16.0);
        assert!(
            matches!(rummage.phase, Phase::Catching(_)),
            "{:?}",
            rummage.phase
        );
        catch_it(&mut ground, &mut rummage, 16.0);
        assert_eq!(rummage.basket().len(), 1);
        assert!(rummage.searched(spot));
        let events = run(&mut ground, &mut rummage, &cast, 16.0, 20.0);
        assert_eq!(rummage.phase, Phase::Exploring, "{events:?}");
    }

    #[test]
    fn a_miss_costs_light_and_widens_the_arc() {
        let cast = sample();
        let (mut ground, mut rummage) = outing(&cast, vec![cast.members[1].id]);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        let spot = rummage.caches.iter().position(Option::is_some).unwrap();
        rummage.choose(&mut ground, spot, 6.0);
        run(&mut ground, &mut rummage, &cast, 6.0, 16.0);
        let Phase::Catching(before) = rummage.phase else {
            panic!();
        };
        let light = rummage.light;
        // Opposite the arc.
        rummage.strike(&mut ground, when_at(&before, before.centre + PI, 16.0));
        let Phase::Catching(after) = rummage.phase else {
            panic!("{:?}", rummage.phase);
        };
        assert!(after.width > before.width);
        assert!(rummage.light < light);
        assert!(rummage.basket().is_empty());
    }

    #[test]
    fn an_empty_spot_costs_a_rummage_and_gives_nothing() {
        let cast = sample();
        let (mut ground, mut rummage) = outing(&cast, vec![cast.members[2].id]);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        let Some(spot) = rummage.caches.iter().position(Option::is_none) else {
            return;
        };
        rummage.choose(&mut ground, spot, 6.0);
        let events = run(&mut ground, &mut rummage, &cast, 6.0, 20.0);
        assert!(events.contains(&Event::Nothing { spot }));
        assert!(rummage.searched(spot));
        rummage.choose(&mut ground, spot, 20.0);
        assert!(
            rummage
                .take_events()
                .contains(&Event::AlreadyLooked { spot })
        );
    }

    #[test]
    fn the_light_runs_out_and_everyone_goes_home_with_the_basket() {
        let cast = sample();
        let party = vec![cast.members[0].id, cast.members[1].id];
        let (mut ground, mut rummage) = outing(&cast, party);
        let mut now = 0.0;
        let mut events = run(&mut ground, &mut rummage, &cast, now, 6.0);
        now = 6.0;
        // Search every spot in turn, missing on purpose, until the light goes.
        let mut spot = 0;
        while !matches!(rummage.phase, Phase::Leaving { .. } | Phase::Over) && now < 600.0 {
            if rummage.phase == Phase::Exploring {
                while rummage.searched(spot % rummage.spot_count()) {
                    spot += 1;
                }
                let count = rummage.spot_count();
                rummage.choose(&mut ground, spot % count, now);
            }
            if let Phase::Catching(catch) = rummage.phase {
                let when = when_at(&catch, catch.centre + PI, now);
                events.extend(run(&mut ground, &mut rummage, &cast, now, when));
                now = when;
                rummage.strike(&mut ground, now);
            }
            events.extend(run(&mut ground, &mut rummage, &cast, now, now + 1.0));
            now += 1.0;
        }
        events.extend(run(&mut ground, &mut rummage, &cast, now, now + 12.0));
        assert!(events.contains(&Event::Leaving(Ending::Dusk)), "{events:?}");
        assert_eq!(rummage.phase, Phase::Over);
    }

    #[test]
    fn the_rarest_signs_wait_for_dusk() {
        let cast = sample();
        let (mut ground, mut rummage) = outing(&cast, vec![cast.members[0].id]);
        let rare = (0..rummage.spot_count())
            .find(|i| rummage.caches[*i].is_some_and(|f| f.tier >= Tier::Rare));
        let Some(rare) = rare else {
            return;
        };
        run(&mut ground, &mut rummage, &cast, 0.0, 60.0);
        assert_eq!(rummage.signs[rare].until, 0.0, "it showed in full daylight");
        rummage.light = rummage.dusk - 1.0;
        run(&mut ground, &mut rummage, &cast, 60.0, 90.0);
        assert!(rummage.signs[rare].until > 60.0);
    }

    fn setting_off(cast: &Cast, party: Vec<Id>, close_pair: bool) -> (Playground, Rummage) {
        let mut ground = woods::open(cast, &party, 0.0);
        let outset = Outset {
            party,
            drought: 0,
            close_pair,
            influence: Influence::default(),
            beckons: false,
            seed: 3,
        };
        let rummage = Rummage::new(&mut ground, outset, |_| false, 0.0);
        (ground, rummage)
    }

    fn opened(rummage: &Rummage) -> Vec<Opener> {
        (SPOTS.len()..rummage.spot_count())
            .filter_map(|spot| rummage.opener_of(spot))
            .collect()
    }

    #[test]
    fn who_comes_along_opens_extra_spots_and_they_always_hold_something() {
        let cast = sample();
        let characters: Vec<Character> = cast.members.iter().map(Character::of).collect();
        let Some(little) = cast
            .members
            .iter()
            .position(|member| member.parent().is_some())
        else {
            return;
        };
        let (_, mut rummage) = setting_off(&cast, vec![cast.members[little].id], false);
        assert!(opened(&rummage).contains(&Opener::LittleOne));
        assert!(
            rummage
                .take_events()
                .iter()
                .any(|event| matches!(event, Event::Opened { .. }))
        );
        for spot in SPOTS.len()..rummage.spot_count() {
            assert!(
                rummage.caches[spot].is_some(),
                "an extra spot came up empty"
            );
        }
        // Somebody grown, not an explorer and not very curious, opens nothing on their own.
        let plain = characters.iter().position(|c| {
            c.parent.is_none() && c.kind != TemperamentKind::Explorer && c.axes.curiosity < 0.8
        });
        if let Some(plain) = plain {
            let (_, rummage) = setting_off(&cast, vec![cast.members[plain].id], false);
            assert!(opened(&rummage).is_empty());
            assert_eq!(rummage.spot_count(), SPOTS.len());
        }
        let pair = vec![cast.members[0].id, cast.members[1].id];
        let (_, rummage) = setting_off(&cast, pair, true);
        assert!(opened(&rummage).contains(&Opener::ClosePair));
    }

    #[test]
    fn an_extra_spot_holds_something_however_the_dice_fall() {
        let cast = sample();
        let pair = vec![cast.members[0].id, cast.members[1].id];
        let little = cast.members.iter().find(|member| member.parent().is_some());
        let parties = [(pair, true)]
            .into_iter()
            .chain(little.map(|little| (vec![little.id], false)));
        for (party, close_pair) in parties {
            let mut ground = woods::open(&cast, &party, 0.0);
            for seed in 0..200 {
                let outset = Outset {
                    party: party.clone(),
                    drought: 0,
                    close_pair,
                    influence: Influence::default(),
                    beckons: false,
                    seed,
                };
                let rummage = Rummage::new(&mut ground, outset, |_| false, 0.0);
                assert!(rummage.spot_count() > SPOTS.len());
                for spot in SPOTS.len()..rummage.spot_count() {
                    assert!(
                        rummage.caches[spot].is_some(),
                        "with seed {seed}, {} came up empty",
                        rummage.spots[spot].name
                    );
                }
            }
        }
    }

    #[test]
    fn a_second_companion_helps_from_open_ground_at_every_spot() {
        let cast = sample();
        let ground = woods::open(&cast, &[cast.members[0].id, cast.members[1].id], 0.0);
        for spot in SPOTS.iter().chain(EXTRAS.iter().map(|extra| &extra.spot)) {
            let (x, y) = beside(&ground, spot.stand);
            assert!(
                woods::walkable(x, y),
                "helping at {}, it would stand at ({x}, {y})",
                spot.name
            );
        }
    }

    #[test]
    fn whoever_searches_an_extra_spot_is_drawn_in_front_of_it() {
        for extra in EXTRAS {
            for searched in [false, true] {
                let base = extra_base(extra.opener, searched);
                assert!(
                    base < extra.spot.stand.1,
                    "{:?} would be drawn over whoever stands at it",
                    extra.opener
                );
            }
        }
    }

    #[test]
    fn whoever_opened_a_spot_is_the_one_who_searches_it() {
        let cast = sample();
        let Some(little) = cast.members.iter().find(|member| member.parent().is_some()) else {
            return;
        };
        let grown = cast
            .members
            .iter()
            .find(|member| member.parent().is_none())
            .unwrap();
        let (mut ground, mut rummage) = setting_off(&cast, vec![grown.id, little.id], false);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        let crevice = (SPOTS.len()..rummage.spot_count())
            .find(|spot| rummage.opener_of(*spot) == Some(Opener::LittleOne))
            .unwrap();
        rummage.choose(&mut ground, crevice, 6.0);
        run(&mut ground, &mut rummage, &cast, 6.0, 16.0);
        assert_eq!(rummage.leader, Some(little.id));
        let at = ground.position(little.id).unwrap();
        assert!(
            distance(at, rummage.spots[crevice].stand) < 2.0,
            "the little one is at {at:?}"
        );
    }

    #[test]
    fn late_in_the_day_something_may_glint_and_beckon() {
        let cast = sample();
        let party = vec![cast.members[0].id];
        let mut ground = woods::open(&cast, &party, 0.0);
        let outset = Outset {
            party,
            drought: 0,
            close_pair: false,
            influence: Influence::default(),
            beckons: true,
            seed: 4,
        };
        let mut rummage = Rummage::new(&mut ground, outset, |_| false, 0.0);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        assert!(rummage.glint.is_none(), "it showed in daylight");
        rummage.light = rummage.full * 0.1;
        let events = run(&mut ground, &mut rummage, &cast, 6.0, 7.0);
        assert!(events.contains(&Event::Glint));
        let glint = rummage.glint.unwrap();
        rummage.choose(&mut ground, glint, 7.0);
        let events = run(&mut ground, &mut rummage, &cast, 7.0, 20.0);
        assert!(events.contains(&Event::Beckoned), "{events:?}");
        assert_eq!(rummage.phase(), Phase::Beckoned);
    }

    #[test]
    fn knacks_come_from_temperament_and_differ() {
        let cast = sample();
        let characters: Vec<Character> = cast.members.iter().map(Character::of).collect();
        let best: std::collections::BTreeSet<Kind> = characters.iter().map(best_at).collect();
        assert!(
            best.len() >= 2,
            "everyone in the sample is best at the same thing"
        );
        for character in &characters {
            for kind in Kind::ALL {
                let k = knack(character, kind);
                assert!((0.85..=1.35).contains(&k));
            }
        }
    }

    #[test]
    fn with_reduced_motion_the_marker_moves_only_while_held() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut rummage) = outing(&cast, vec![cast.members[0].id]);
        run(&mut ground, &mut rummage, &cast, 0.0, 6.0);
        let spot = rummage.caches.iter().position(Option::is_some).unwrap();
        rummage.choose(&mut ground, spot, 6.0);
        run(&mut ground, &mut rummage, &cast, 6.0, 10.0);
        let Phase::Catching(catch) = rummage.phase else {
            panic!("{:?}", rummage.phase);
        };
        assert_eq!(catch.marker(100.0, true), 0.0, "it moved on its own");
        // Hold until the marker sits in the middle of the arc, then let go.
        let mut now = 10.0;
        let dt = 1.0 / 120.0;
        while let Phase::Catching(catch) = rummage.phase {
            let off = (catch.held_angle - catch.centre + PI).rem_euclid(TAU) - PI;
            if off.abs() < 0.03 {
                break;
            }
            rummage.hold(&mut ground, true, dt, now);
            now += dt;
        }
        rummage.hold(&mut ground, false, dt, now);
        assert_eq!(rummage.basket().len(), 1);
    }
}
