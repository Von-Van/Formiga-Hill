//! A playground: any place at the Hill where the colony is free to play.
//!
//! Nobody here follows a script. Each traveller decides for itself what to do next from who it
//! is (see `character`): wanders, sits down, naps in the shade, shows off, or goes to see a
//! friend, plays with a playmate, or trails after its parent, all read from what Desktop sent.
//! Pairs that rub each other the wrong way keep their distance. The person at the Hill can pet
//! anyone, or hold out a snack or a toy, and each answers in its own way. A story or a game can
//! take some of them out of free play for a while and direct them instead.
//!
//! What differs from place to place is its `Layout`: where anyone may stand, its named spots,
//! somewhere to sit and somewhere shady, and the way in. Props stand on the ground among the
//! colony and are drawn in depth order with them, so someone can be behind a barrel.

use crate::actor::{Actor, Step, Whereabouts};
use crate::cast::{Cast, Id};
use crate::character::{Character, Company, Cue, Idea, Offer};
use crate::cues::draw_cue;
use crate::dice::Dice;
use crate::paint::blit;
use formiga_art::{Canvas, GazeDirection};
use std::collections::HashMap;

/// How much a creature warms to the person with each kindness, and how far, over one visit.
const TRUST_STEP: f32 = 0.15;
const MAX_TRUST: f32 = 0.6;
/// How close two companions stand when one goes to see the other.
const BESIDE: f32 = 26.0;
/// How long a prop shakes when someone looks behind it.
const RUSTLE_SECS: f32 = 0.35;

/// A rectangle of ground: left, top, right, bottom.
pub type Patch = (f32, f32, f32, f32);

/// What makes one playground different from another.
#[derive(Clone, Copy)]
pub struct Layout {
    /// Whether someone can stand here.
    pub walkable: fn(f32, f32) -> bool,
    /// The ground anyone might be sent to, for choosing spots in.
    pub ground: Patch,
    /// Named spots a story can send someone to.
    pub spots: &'static [(&'static str, (f32, f32))],
    /// Somewhere to sit, if there is one.
    pub seats: Option<Patch>,
    /// Somewhere shady to nap, if there is one.
    pub shade: Option<Patch>,
    /// Where the colony comes in from, one after another, offset by `entrance_step` each.
    pub entrance: (f32, f32),
    pub entrance_step: (f32, f32),
}

impl Layout {
    /// The nearest place someone can stand to `(x, y)`, looking up and down the same column.
    fn nearest(&self, x: f32, y: f32) -> (f32, f32) {
        let (left, top, right, bottom) = self.ground;
        let x = x.clamp(left, right);
        let y = y.clamp(top, bottom);
        for step in 0..200 {
            for candidate in [y + step as f32, y - step as f32] {
                if (self.walkable)(x, candidate) {
                    return (x, candidate);
                }
            }
        }
        (x, y)
    }
}

/// Something standing on the ground that anyone further back is hidden behind.
pub struct Prop {
    pub sprite: Canvas,
    /// The top-left corner of the sprite in the scene.
    pub at: (i32, i32),
    /// The row where it meets the ground: whoever stands above this is behind it.
    pub base: f32,
    rustled: f32,
}

impl Prop {
    pub fn new(sprite: Canvas, at: (i32, i32), base: f32) -> Self {
        Self {
            sprite,
            at,
            base,
            rustled: f32::MIN,
        }
    }

    /// The opaque part of the prop in scene pixels, inclusive.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        let (left, top, right, bottom) = self.sprite.alpha_bounds().unwrap_or((0, 0, 0, 0));
        (
            self.at.0 + left as i32,
            self.at.1 + top as i32,
            self.at.0 + right as i32,
            self.at.1 + bottom as i32,
        )
    }
}

#[derive(Clone, Copy)]
enum Ground {
    Anywhere,
    Seats,
    Shade,
}

pub struct Playground {
    layout: Layout,
    backdrop: Canvas,
    foreground: Canvas,
    props: Vec<Prop>,
    actors: Vec<Actor>,
    dice: Dice,
    /// How much each has warmed to the person this visit. Never kept: the next visit starts
    /// fresh, and nothing is lost by staying away.
    trust: HashMap<Id, f32>,
    /// Who was last offered what, and how many times running.
    streak: Option<(Id, Offer, u32)>,
    clock: f32,
    pointer: Option<(f32, f32)>,
    reduce_motion: bool,
    /// Those taken out of free play by a story or a game.
    reserved: Vec<Id>,
    /// Whoever's line is on show, with a speech bubble over their head.
    speaker: Option<Id>,
}

impl Playground {
    /// The colony arrives by the layout's way in, unless motion is reduced, in which case
    /// everyone is simply there.
    pub fn new(
        cast: &Cast,
        now: f32,
        layout: Layout,
        backdrop: Canvas,
        foreground: Canvas,
        props: Vec<Prop>,
    ) -> Self {
        let reduce_motion = cast.reduce_motion();
        let count = cast.members.len().max(1);
        let seed = cast
            .ids()
            .fold(0x6772_6565_6e00, |seed, id| seed ^ id.rotate_left(17));
        let (left, top, right, bottom) = layout.ground;
        let actors = cast
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let share = (index as f32 + 0.5) / count as f32;
                let home = layout.nearest(
                    left + 60.0 + (right - left - 100.0) * share,
                    top + (bottom - top) * (0.3 + (index % 3) as f32 * 0.2),
                );
                if reduce_motion {
                    Actor::new(member, home, index % 2 == 0, true)
                } else {
                    let start = (
                        layout.entrance.0 + layout.entrance_step.0 * index as f32,
                        layout.entrance.1 + layout.entrance_step.1 * (index % 2) as f32,
                    );
                    let facing_right = home.0 > start.0;
                    let mut actor = Actor::new(member, start, facing_right, false);
                    actor.begin(now, [Step::Walk { to: home }]);
                    actor
                }
            })
            .collect();
        Self {
            layout,
            backdrop,
            foreground,
            props,
            actors,
            dice: Dice::new(seed),
            trust: HashMap::new(),
            streak: None,
            clock: now,
            pointer: None,
            reduce_motion,
            reserved: Vec::new(),
            speaker: None,
        }
    }

    pub fn reduce_motion(&self) -> bool {
        self.reduce_motion
    }

    /// Takes these travellers out of free play.
    pub fn reserve(&mut self, ids: Vec<Id>) {
        self.reserved = ids;
    }

    /// Gives everyone back to free play.
    pub fn release(&mut self) {
        self.reserved.clear();
        self.speaker = None;
    }

    pub fn set_speaker(&mut self, speaker: Option<Id>) {
        self.speaker = speaker;
    }

    /// Has a traveller start on `steps`, dropping whatever it was doing.
    pub fn direct(&mut self, id: Id, steps: Vec<Step>, now: f32) {
        if let Some(index) = self.index_of(id) {
            self.actors[index].begin(now, steps);
        }
    }

    /// Whether a traveller still has something to finish.
    pub fn busy(&self, id: Id) -> bool {
        self.index_of(id)
            .is_some_and(|index| !self.actors[index].is_idle())
    }

    pub fn character(&self, id: Id) -> Option<&Character> {
        self.index_of(id).map(|index| &self.actors[index].character)
    }

    pub fn position(&self, id: Id) -> Option<(f32, f32)> {
        self.index_of(id).map(|index| self.actors[index].pos)
    }

    pub fn ids(&self) -> Vec<Id> {
        self.actors.iter().map(|actor| actor.id).collect()
    }

    /// Raises a traveller off the ground by `lift` pixels as drawn: peeking over something.
    pub fn set_lift(&mut self, id: Id, lift: f32) {
        if let Some(index) = self.index_of(id) {
            self.actors[index].lift = lift;
        }
    }

    /// Puts a traveller somewhere at once, without walking there.
    pub fn teleport(&mut self, id: Id, to: (f32, f32)) {
        if let Some(index) = self.index_of(id) {
            self.actors[index].pos = to;
        }
    }

    /// Shakes a prop for a moment, as when someone looks behind it.
    pub fn rustle(&mut self, prop: usize, now: f32) {
        if let Some(prop) = self.props.get_mut(prop) {
            prop.rustled = now;
        }
    }

    /// A named spot, with room for several: the `slot`th to arrive stands a little to one side
    /// of the one before.
    pub fn spot(&self, place: &str, slot: usize) -> (f32, f32) {
        let (x, y) = self
            .layout
            .spots
            .iter()
            .find(|(name, _)| *name == place)
            .or_else(|| self.layout.spots.iter().find(|(name, _)| *name == "centre"))
            .map_or_else(|| self.layout.nearest(192.0, 150.0), |(_, at)| *at);
        const SPREAD: [f32; 7] = [0.0, -22.0, 22.0, -44.0, 44.0, -66.0, 66.0];
        let (left, top, right, bottom) = self.layout.ground;
        let spot = (
            (x + SPREAD[slot % SPREAD.len()]).clamp(left, right),
            (y + (slot / SPREAD.len()) as f32 * 10.0 + (slot % 2) as f32 * 3.0).clamp(top, bottom),
        );
        self.layout.nearest(spot.0, spot.1)
    }

    /// Where `mover` would stand beside `target`.
    pub fn beside_of(&self, mover: Id, target: Id) -> (f32, f32) {
        match self.index_of(mover) {
            Some(index) => self.beside(index, target),
            None => self.spot("centre", 0),
        }
    }

    /// Where the pointer is, in scene pixels, for whoever looks up at it.
    pub fn set_pointer(&mut self, pointer: Option<(f32, f32)>) {
        self.pointer = pointer;
    }

    pub fn tick(&mut self, cast: &Cast, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        let everyone: Vec<Whereabouts> = self
            .actors
            .iter()
            .map(|actor| Whereabouts {
                id: actor.id,
                pos: actor.pos,
                walking: actor.walking(),
            })
            .collect();
        for actor in &mut self.actors {
            actor.advance(now, dt, &everyone);
        }
        for index in 0..self.actors.len() {
            if self.actors[index].is_idle() && !self.reserved.contains(&self.actors[index].id) {
                self.plan(index, cast, now);
            }
        }
        self.look_at_the_pointer();
    }

    /// Holds something out to a traveller, who answers in its own way.
    pub fn offer(&mut self, id: Id, offer: Offer, now: f32) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let in_a_row = match self.streak {
            Some((last, last_offer, count)) if last == id && last_offer == offer => count + 1,
            _ => 0,
        };
        self.streak = Some((id, offer, in_a_row));
        let trust = self.trust.get(&id).copied().unwrap_or(0.0);
        let beats = self.actors[index]
            .character
            .react(offer, trust, in_a_row, &mut self.dice);
        self.trust.insert(id, (trust + TRUST_STEP).min(MAX_TRUST));
        let turn = self.pointer.map(|(x, _)| Step::FaceX(x));
        self.actors[index].begin(
            now,
            turn.into_iter().chain(beats.into_iter().map(Step::Beat)),
        );
    }

    /// The traveller drawn at a point in scene pixels, front-most first. Anyone behind a prop
    /// at that point is not there to be found by pointing.
    pub fn actor_at(&mut self, x: f32, y: f32, now: f32) -> Option<Id> {
        let inside = |(left, top, right, bottom): (i32, i32, i32, i32)| {
            x >= left as f32
                && x <= right as f32 + 1.0
                && y >= top as f32
                && y <= bottom as f32 + 1.0
        };
        let mut order = self.drawing_order();
        order.reverse();
        for index in order {
            let actor = &mut self.actors[index];
            if inside(actor.bounds(now)) {
                let behind = self
                    .props
                    .iter()
                    .any(|prop| prop.base > actor.pos.1 && inside(prop.bounds()));
                return (!behind).then_some(actor.id);
            }
        }
        None
    }

    /// The top middle of a traveller as drawn, for a name tag over its head.
    pub fn head(&mut self, id: Id, now: f32) -> Option<(f32, f32)> {
        let index = self.index_of(id)?;
        let (left, top, right, _) = self.actors[index].bounds(now);
        Some(((left + right + 1) as f32 / 2.0, top as f32))
    }

    pub fn compose(&mut self, now: f32) -> Canvas {
        let mut scene = self.backdrop.clone();
        let order = self.drawing_order();
        for &index in &order {
            self.actors[index].draw_shadow(&mut scene, now);
        }
        // Travellers and props together, back to front.
        let mut layers: Vec<(f32, Option<usize>, Option<usize>)> = order
            .iter()
            .map(|&index| (self.actors[index].pos.1, Some(index), None))
            .chain(
                self.props
                    .iter()
                    .enumerate()
                    .map(|(index, prop)| (prop.base, None, Some(index))),
            )
            .collect();
        layers.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, actor, prop) in layers {
            if let Some(index) = actor {
                self.actors[index].draw(&mut scene, now);
            }
            if let Some(index) = prop {
                let prop = &self.props[index];
                let shaking = now - prop.rustled < RUSTLE_SECS && !self.reduce_motion;
                let nudge = if shaking {
                    ((now * 40.0) as i32 % 2) * 2 - 1
                } else {
                    0
                };
                blit(&mut scene, &prop.sprite, prop.at.0 + nudge, prop.at.1);
            }
        }
        for &index in &order {
            let actor = &mut self.actors[index];
            let speaking = self.speaker == Some(actor.id);
            let Some(cue) = actor
                .current_beat()
                .and_then(|beat| beat.cue)
                .or(speaking.then_some(Cue::Speech))
            else {
                continue;
            };
            let age = if speaking {
                now
            } else {
                actor.step_elapsed(now)
            };
            let (left, top, right, _) = actor.bounds(now);
            draw_cue(
                &mut scene,
                cue,
                (left + right) / 2,
                top - 2,
                age,
                self.reduce_motion,
            );
        }
        blit(&mut scene, &self.foreground, 0, 0);
        scene
    }

    fn index_of(&self, id: Id) -> Option<usize> {
        self.actors.iter().position(|actor| actor.id == id)
    }

    /// Back to front: whoever stands further up the ground is further away.
    fn drawing_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.actors.len()).collect();
        order.sort_by(|&a, &b| self.actors[a].pos.1.total_cmp(&self.actors[b].pos.1));
        order
    }

    /// Gives an idle traveller something to do, and anyone it is doing it with.
    fn plan(&mut self, index: usize, cast: &Cast, now: f32) {
        let id = self.actors[index].id;
        let free: Vec<Id> = self
            .actors
            .iter()
            .filter(|actor| actor.id != id && actor.is_free() && !self.reserved.contains(&actor.id))
            .map(|actor| actor.id)
            .collect();
        let parent = self.actors[index]
            .character
            .parent
            .filter(|parent| self.index_of(*parent).is_some() && !self.reserved.contains(parent));
        let company = Company {
            friend: cast.closest_friend(id, free.iter().copied()),
            playmate: cast.playmate(id, free.iter().copied()),
            parent,
        };
        let mut idea = self.actors[index].character.idea(company, &mut self.dice);
        if self.reduce_motion {
            // A tableau rather than a stroll: everything happens where each one stands.
            idea = match idea {
                Idea::Wander | Idea::Visit(_) | Idea::PlayWith(_) | Idea::FollowParent(_) => {
                    Idea::Rest
                }
                other => other,
            };
        }
        let beats = self.actors[index].character.beats_for(idea, &mut self.dice);
        let mut steps = match idea {
            Idea::Wander => vec![Step::Walk {
                to: self.open_spot(index, cast, Ground::Anywhere),
            }],
            Idea::Sit if self.layout.seats.is_some() && self.dice.chance(0.6) => vec![Step::Walk {
                to: self.open_spot(index, cast, Ground::Seats),
            }],
            Idea::Nap if self.layout.shade.is_some() => vec![Step::Walk {
                to: self.open_spot(index, cast, Ground::Shade),
            }],
            Idea::Visit(other) | Idea::PlayWith(other) | Idea::FollowParent(other) => vec![
                Step::Walk {
                    to: self.beside(index, other),
                },
                Step::Face(other),
            ],
            _ => Vec::new(),
        };
        if self.reduce_motion {
            steps.retain(|step| !matches!(step, Step::Walk { .. } | Step::Stride { .. }));
        }
        steps.extend(beats.into_iter().map(Step::Beat));
        self.actors[index].begin(now, steps);

        // The friend or playmate stops what it was idling at and joins in.
        if let Idea::Visit(other) | Idea::PlayWith(other) = idea
            && let Some(partner) = self.index_of(other)
        {
            let character = &self.actors[partner].character;
            let joined = match idea {
                Idea::PlayWith(_) => character.play_together(),
                _ => character.beats_for(Idea::Visit(id), &mut self.dice),
            };
            let steps = [Step::Await(id), Step::Face(id)]
                .into_iter()
                .chain(joined.into_iter().map(Step::Beat));
            self.actors[partner].begin(now, steps);
        }
    }

    /// A spot beside another traveller, on the side this one is coming from.
    fn beside(&self, index: usize, other: Id) -> (f32, f32) {
        let me = self.actors[index].pos;
        let Some(target) = self.index_of(other).map(|o| self.actors[o].destination()) else {
            return me;
        };
        let (_, top, _, bottom) = self.layout.ground;
        let side = if me.0 < target.0 { -1.0 } else { 1.0 };
        let y = (target.1 + 2.0).clamp(top, bottom);
        let mut x = target.0 + side * BESIDE;
        if !(self.layout.walkable)(x, y) {
            x = target.0 - side * BESIDE;
        }
        self.layout.nearest(x, y)
    }

    /// Somewhere on `ground` with room to stand: away from everyone else, well away from anyone
    /// this one is at odds with, and not too far to go.
    fn open_spot(&mut self, index: usize, cast: &Cast, ground: Ground) -> (f32, f32) {
        let (left, top, right, bottom) = match ground {
            Ground::Anywhere => self.layout.ground,
            Ground::Seats => self.layout.seats.unwrap_or(self.layout.ground),
            Ground::Shade => self.layout.shade.unwrap_or(self.layout.ground),
        };
        let me = &self.actors[index];
        let (id, here) = (me.id, me.pos);
        let mut best = (f32::MIN, self.layout.nearest(here.0, here.1));
        for _ in 0..14 {
            let spot = (self.dice.range(left, right), self.dice.range(top, bottom));
            if !(self.layout.walkable)(spot.0, spot.1) {
                continue;
            }
            let mut score = -distance(spot, here) * 0.15;
            let mut nearest = f32::MAX;
            for other in self.actors.iter().filter(|other| other.id != id) {
                let apart = distance(spot, other.destination());
                nearest = nearest.min(apart);
                if apart < 80.0 && cast.at_odds(id, other.id) {
                    score -= 200.0;
                }
            }
            score += nearest.min(60.0);
            if score > best.0 {
                best = (score, spot);
            }
        }
        best.1
    }

    fn look_at_the_pointer(&mut self) {
        for actor in &mut self.actors {
            actor.gaze = match self.pointer {
                Some((x, y))
                    if (x - actor.pos.0).abs() < 90.0
                        && y < actor.pos.1 + 12.0
                        && y > actor.pos.1 - 90.0 =>
                {
                    let across = if x > actor.pos.0 + 6.0 {
                        1
                    } else if x < actor.pos.0 - 6.0 {
                        -1
                    } else {
                        0
                    };
                    let up = if y < actor.pos.1 - 48.0 {
                        -1
                    } else if y > actor.pos.1 - 6.0 {
                        1
                    } else {
                        0
                    };
                    GazeDirection::new(across, up)
                }
                _ => GazeDirection::default(),
            };
        }
    }
}

pub fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::green;
    use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
    use formiga_art::BodyClip;
    use formiga_core::{ActionKind, Gesture};

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// Runs a playground from `from` to `to` seconds, thirty ticks a second.
    pub fn run(playground: &mut Playground, cast: &Cast, from: f32, to: f32) {
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            playground.tick(cast, now);
        }
    }

    fn on_the_ground(playground: &Playground) -> bool {
        playground.actors.iter().all(|actor| {
            let (x, y) = actor.pos;
            [(0.0, 0.0), (0.5, 0.0), (-0.5, 0.0), (0.0, 0.5), (0.0, -0.5)]
                .iter()
                .any(|(dx, dy)| (playground.layout.walkable)(x + dx, y + dy))
        })
    }

    #[test]
    fn the_colony_walks_in_and_then_keeps_to_the_ground() {
        let cast = sample();
        let mut green = green::open(&cast, 0.0);
        assert!(
            green
                .actors
                .iter()
                .all(|actor| actor.pos.0 > SCENE_WIDTH as f32)
        );
        run(&mut green, &cast, 0.0, 20.0);
        assert!(on_the_ground(&green));
        for second in 20..90 {
            run(&mut green, &cast, second as f32, second as f32 + 1.0);
            assert!(on_the_ground(&green), "someone strayed by {second}s");
        }
    }

    #[test]
    fn free_play_keeps_everyone_busy_with_something() {
        let cast = sample();
        let mut green = green::open(&cast, 0.0);
        let start: Vec<_> = green.actors.iter().map(|actor| actor.pos).collect();
        run(&mut green, &cast, 0.0, 60.0);
        let moved = green
            .actors
            .iter()
            .zip(&start)
            .filter(|(actor, start)| distance(actor.pos, **start) > 1.0)
            .count();
        assert_eq!(moved, green.actors.len());
    }

    #[test]
    fn a_pat_is_answered_at_once() {
        let cast = sample();
        let mut green = green::open(&cast, 0.0);
        run(&mut green, &cast, 0.0, 12.0);
        let id = green.actors[0].id;
        green.set_pointer(Some((10.0, 150.0)));
        green.offer(id, Offer::Pet, 12.0);
        green.tick(&cast, 12.05);
        let actor = &green.actors[0];
        assert!(!actor.facing_right, "it turned to the hand");
        let clip = actor.current_beat().map(|beat| beat.clip);
        assert!(
            matches!(
                clip,
                Some(BodyClip::Action(ActionKind::PetReaction))
                    | Some(BodyClip::Gesture(Gesture::Gasp | Gesture::Huff))
            ),
            "answered with {clip:?}"
        );
    }

    #[test]
    fn being_kind_builds_trust_but_only_so_far() {
        let cast = sample();
        let mut green = green::open(&cast, 0.0);
        let id = green.actors[1].id;
        for pat in 0..10 {
            green.offer(id, Offer::Snack, pat as f32);
        }
        assert_eq!(green.trust[&id], MAX_TRUST);
    }

    #[test]
    fn with_reduced_motion_nobody_strolls() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let mut green = green::open(&cast, 0.0);
        let start: Vec<_> = green.actors.iter().map(|actor| actor.pos).collect();
        run(&mut green, &cast, 0.0, 40.0);
        let now: Vec<_> = green.actors.iter().map(|actor| actor.pos).collect();
        assert_eq!(start, now);
    }

    #[test]
    fn hit_testing_finds_whoever_is_under_the_pointer() {
        let cast = sample();
        let mut green = green::open(&cast, 0.0);
        run(&mut green, &cast, 0.0, 15.0);
        let id = green.actors[2].id;
        let (x, y) = green.head(id, 15.0).unwrap();
        assert_eq!(green.actor_at(x, y + 6.0, 15.0), Some(id));
        assert_eq!(green.actor_at(2.0, 2.0, 15.0), None);
    }

    #[test]
    fn someone_behind_a_prop_is_hidden_by_it() {
        let cast = sample();
        let mut sprite = Canvas::new(60, 60);
        sprite.fill_rect(0, 0, 60, 60, formiga_art::Rgba::new(120, 80, 40, 255));
        let props = vec![Prop::new(sprite, (170, 100), 162.0)];
        let mut playground = Playground::new(
            &cast,
            0.0,
            green::layout(),
            Canvas::new(SCENE_WIDTH, SCENE_HEIGHT),
            Canvas::new(SCENE_WIDTH, SCENE_HEIGHT),
            props,
        );
        let id = playground.ids()[0];
        playground.reserve(vec![id]);
        playground.direct(id, vec![], 0.0);
        let index = playground.index_of(id).unwrap();
        playground.actors[index].pos = (200.0, 150.0);
        assert_eq!(playground.actor_at(200.0, 140.0, 0.0), None);
        let scene = playground.compose(0.0);
        assert_eq!(
            scene.get(200, 140),
            formiga_art::Rgba::new(120, 80, 40, 255)
        );
    }
}
