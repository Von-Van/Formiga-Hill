//! The Village Green: the first place the colony goes from the station, for free play.
//!
//! Nobody here follows a script. Each traveller decides for itself what to do next from who it
//! is (see `character`): wanders, sits on the blanket, naps in the oak's shade, shows off, or
//! goes to see a friend, plays with a playmate, or trails after its parent, all read from what
//! Desktop sent. Pairs that rub each other the wrong way keep their distance. The person at the
//! Hill can pet anyone, or hold out a snack or a toy, and each answers in its own way.

mod scenery;

use crate::actor::{Actor, Step, Whereabouts};
use crate::cast::{Cast, Id};
use crate::character::{Character, Company, Cue, Idea, Offer};
use crate::cues::draw_cue;
use crate::dice::Dice;
use crate::paint::blit;
use formiga_art::{Canvas, GazeDirection};
use scenery::{BLANKET, SHADE, WALK_BOTTOM, WALK_LEFT, WALK_RIGHT, WALK_TOP};
use std::collections::HashMap;

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};

/// How much a creature warms to the person with each kindness, and how far, over one visit.
const TRUST_STEP: f32 = 0.15;
const MAX_TRUST: f32 = 0.6;
/// How close two companions stand when one goes to see the other.
const BESIDE: f32 = 26.0;

#[derive(Clone, Copy)]
enum Ground {
    Lawn,
    Blanket,
    Shade,
}

pub struct Green {
    backdrop: Canvas,
    foreground: Canvas,
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
    /// Those with a part in a story, whom free play leaves alone.
    reserved: Vec<Id>,
    /// Whoever's line is on show, with a speech bubble over their head.
    speaker: Option<Id>,
}

impl Green {
    /// The colony arrives along the stepping stones from the right, unless motion is reduced, in
    /// which case everyone is simply there.
    pub fn new(cast: &Cast, now: f32) -> Self {
        let reduce_motion = cast.reduce_motion();
        let count = cast.members.len().max(1);
        let seed = cast
            .ids()
            .fold(0x6772_6565_6e00, |seed, id| seed ^ id.rotate_left(17));
        let actors = cast
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let share = (index as f32 + 0.5) / count as f32;
                let home = (
                    WALK_LEFT + 40.0 + (WALK_RIGHT - WALK_LEFT - 80.0) * share,
                    WALK_TOP + 8.0 + (index % 3) as f32 * 16.0,
                );
                if reduce_motion {
                    Actor::new(member, home, index % 2 == 0, true)
                } else {
                    let start = (
                        400.0 + index as f32 * 22.0,
                        170.0 + (index % 2) as f32 * 6.0,
                    );
                    let mut actor = Actor::new(member, start, false, false);
                    actor.begin(now, [Step::Walk { to: home }]);
                    actor
                }
            })
            .collect();
        Self {
            backdrop: scenery::backdrop(),
            foreground: scenery::foreground(),
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

    /// Takes these travellers out of free play for a story.
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

    /// A named spot on the green, with room for several: the `slot`th to arrive stands a little
    /// to one side of the one before.
    pub fn spot(&self, place: &str, slot: usize) -> (f32, f32) {
        let (x, y) = match place {
            "blanket" => (257.0, 181.0),
            "well" => (222.0, 154.0),
            "oak" => (88.0, 160.0),
            "swing" => (120.0, 156.0),
            "chest" => (334.0, 156.0),
            "left" => (70.0, 182.0),
            "right" => (318.0, 182.0),
            "front" => (192.0, 198.0),
            "back" => (192.0, 154.0),
            _ => (192.0, 176.0),
        };
        const SPREAD: [f32; 7] = [0.0, -22.0, 22.0, -44.0, 44.0, -66.0, 66.0];
        (
            (x + SPREAD[slot % SPREAD.len()]).clamp(WALK_LEFT, WALK_RIGHT),
            (y + (slot / SPREAD.len()) as f32 * 10.0 + (slot % 2) as f32 * 3.0)
                .clamp(WALK_TOP, WALK_BOTTOM),
        )
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

    /// The traveller drawn at a point in scene pixels, front-most first.
    pub fn actor_at(&mut self, x: f32, y: f32, now: f32) -> Option<Id> {
        let mut order = self.drawing_order();
        order.reverse();
        order.into_iter().find_map(|index| {
            let actor = &mut self.actors[index];
            let (left, top, right, bottom) = actor.bounds(now);
            let inside = x >= left as f32
                && x <= right as f32 + 1.0
                && y >= top as f32
                && y <= bottom as f32 + 1.0;
            inside.then_some(actor.id)
        })
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
        for &index in &order {
            self.actors[index].draw(&mut scene, now);
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

    /// Back to front: whoever stands further up the lawn is further away.
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
            .filter(|parent| self.index_of(*parent).is_some());
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
                to: self.open_spot(index, cast, Ground::Lawn),
            }],
            Idea::Sit if self.dice.chance(0.6) => vec![Step::Walk {
                to: self.open_spot(index, cast, Ground::Blanket),
            }],
            Idea::Nap => vec![Step::Walk {
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
        let side = if me.0 < target.0 { -1.0 } else { 1.0 };
        let mut x = target.0 + side * BESIDE;
        if !(WALK_LEFT..=WALK_RIGHT).contains(&x) {
            x = target.0 - side * BESIDE;
        }
        (
            x.clamp(WALK_LEFT, WALK_RIGHT),
            (target.1 + 2.0).clamp(WALK_TOP, WALK_BOTTOM),
        )
    }

    /// Somewhere on `ground` with room to stand: away from everyone else, well away from anyone
    /// this one is at odds with, and not too far to go.
    fn open_spot(&mut self, index: usize, cast: &Cast, ground: Ground) -> (f32, f32) {
        let (left, top, right, bottom) = match ground {
            Ground::Lawn => (WALK_LEFT, WALK_TOP, WALK_RIGHT, WALK_BOTTOM),
            Ground::Blanket => {
                let (back, front, (back_left, back_right), _) = BLANKET;
                (
                    back_left as f32 + 4.0,
                    back as f32 + 8.0,
                    back_right as f32 - 4.0,
                    front as f32 - 2.0,
                )
            }
            Ground::Shade => SHADE,
        };
        let me = &self.actors[index];
        let (id, here) = (me.id, me.pos);
        let mut best = (f32::MIN, here);
        for _ in 0..14 {
            let spot = (self.dice.range(left, right), self.dice.range(top, bottom));
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

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_art::BodyClip;
    use formiga_core::{ActionKind, Gesture};

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// Runs the green from `from` to `to` seconds, thirty ticks a second.
    fn run(green: &mut Green, cast: &Cast, from: f32, to: f32) {
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            green.tick(cast, now);
        }
    }

    fn on_the_lawn(green: &Green) -> bool {
        green.actors.iter().all(|actor| {
            (WALK_LEFT - 0.5..=WALK_RIGHT + 0.5).contains(&actor.pos.0)
                && (WALK_TOP - 0.5..=WALK_BOTTOM + 0.5).contains(&actor.pos.1)
        })
    }

    #[test]
    fn the_colony_walks_in_and_then_keeps_to_the_lawn() {
        let cast = sample();
        let mut green = Green::new(&cast, 0.0);
        assert!(
            green
                .actors
                .iter()
                .all(|actor| actor.pos.0 > SCENE_WIDTH as f32)
        );
        run(&mut green, &cast, 0.0, 20.0);
        assert!(on_the_lawn(&green));
        for second in 20..90 {
            run(&mut green, &cast, second as f32, second as f32 + 1.0);
            assert!(on_the_lawn(&green), "someone strayed by {second}s");
        }
    }

    #[test]
    fn free_play_keeps_everyone_busy_with_something() {
        let cast = sample();
        let mut green = Green::new(&cast, 0.0);
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
        let mut green = Green::new(&cast, 0.0);
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
        let mut green = Green::new(&cast, 0.0);
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
        let mut green = Green::new(&cast, 0.0);
        let start: Vec<_> = green.actors.iter().map(|actor| actor.pos).collect();
        run(&mut green, &cast, 0.0, 40.0);
        let now: Vec<_> = green.actors.iter().map(|actor| actor.pos).collect();
        assert_eq!(start, now);
    }

    #[test]
    fn hit_testing_finds_whoever_is_under_the_pointer() {
        let cast = sample();
        let mut green = Green::new(&cast, 0.0);
        run(&mut green, &cast, 0.0, 15.0);
        let id = green.actors[2].id;
        let (x, y) = green.head(id, 15.0).unwrap();
        assert_eq!(green.actor_at(x, y + 6.0, 15.0), Some(id));
        assert_eq!(green.actor_at(2.0, 2.0, 15.0), None);
    }

    #[test]
    fn composing_puts_the_colony_on_the_green() {
        let cast = sample();
        let mut green = Green::new(&cast, 0.0);
        run(&mut green, &cast, 0.0, 15.0);
        let scene = green.compose(15.0);
        let mut empty = scenery::backdrop();
        blit(&mut empty, &scenery::foreground(), 0, 0);
        assert_ne!(scene, empty);
    }
}
