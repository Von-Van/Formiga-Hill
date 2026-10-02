//! A traveller performing on a stage: where it stands, which way it faces, what it is doing, and
//! the frames `formiga-art` draws for it, face and all.
//!
//! What an actor does is a queue of steps: walk somewhere, turn to someone, wait for them, play a
//! beat. The area it is in decides what goes in the queue; the actor only carries it out.

use crate::cast::{Id, Member};
use crate::character::{Beat, Character};
use crate::paint::{blit, ellipse, rgba};
use formiga_art::{
    AccessoryArt, AnimationSpec, BodyClip, Canvas, CreatureRenderer, ExpressionKind, EyelidPose,
    FRAME_SIZE, FaceRenderState, GazeDirection,
};
use formiga_core::{ActionKind, AppearanceGenome};
use std::collections::{HashMap, VecDeque};

/// One thing an actor does, in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    /// Walk until its feet are at this point.
    Walk {
        to: (f32, f32),
    },
    /// The same, briskly: somewhere to be, because a story is waiting on it.
    Stride {
        to: (f32, f32),
    },
    /// Turn towards another actor.
    Face(Id),
    /// Turn towards a point across the scene.
    FaceX(f32),
    /// Wait where it is until another actor has stopped walking.
    Await(Id),
    Beat(Beat),
}

/// Where everyone is, for the steps that depend on someone else.
pub struct Whereabouts {
    pub id: Id,
    pub pos: (f32, f32),
    pub walking: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct FrameKey {
    clip: BodyClip,
    frame: u8,
    facing_right: bool,
    expression: ExpressionKind,
    eyelids: EyelidPose,
    gaze: GazeDirection,
}

struct Frame {
    canvas: Canvas,
    /// The opaque part, inclusive, in the frame's own pixels.
    bounds: (i32, i32, i32, i32),
}

/// Frames are drawn on first use and kept; this many is a generous visit's worth.
const FRAME_CACHE_LIMIT: usize = 1500;
/// How long one turn of a spin lasts.
const SPIN_TURN: f32 = 0.22;
/// How much faster a stride is than a stroll.
const STRIDE: f32 = 1.6;

pub struct Actor {
    pub id: Id,
    pub character: Character,
    appearance: AppearanceGenome,
    dress: Option<AccessoryArt>,
    /// The frame row its feet rest on.
    foot_row: i32,
    pub pos: (f32, f32),
    pub facing_right: bool,
    pub gaze: GazeDirection,
    steps: VecDeque<Step>,
    /// When the step at the front began.
    step_since: f32,
    frames: HashMap<FrameKey, Frame>,
    /// Blinks come round every `period` seconds, offset by `phase`, so no two are in step.
    blink: (f32, f32),
    reduce_motion: bool,
}

impl Actor {
    pub fn new(member: &Member, pos: (f32, f32), facing_right: bool, reduce_motion: bool) -> Self {
        let baseline = CreatureRenderer::resting_baseline(member.genome(), reduce_motion);
        let seed = (member.id % 1009) as f32 / 1009.0;
        Self {
            id: member.id,
            character: Character::of(member),
            appearance: member.genome().clone(),
            dress: member.dress,
            foot_row: FRAME_SIZE as i32 - 1 - baseline as i32,
            pos,
            facing_right,
            gaze: GazeDirection::default(),
            steps: VecDeque::new(),
            step_since: 0.0,
            frames: HashMap::new(),
            blink: (3.2 + seed * 2.4, seed * 5.0),
            reduce_motion,
        }
    }

    /// Drops whatever it was doing and starts on `steps`.
    pub fn begin(&mut self, now: f32, steps: impl IntoIterator<Item = Step>) {
        self.steps.clear();
        self.steps.extend(steps);
        self.step_since = now;
        // A cut is over before it could be drawn mid-stride.
        while self.reduce_motion
            && let Some(Step::Walk { to } | Step::Stride { to }) = self.steps.front().copied()
        {
            self.cut_to(to);
            self.steps.pop_front();
        }
    }

    /// With motion reduced a walk is a cut: it is simply there, facing the way it went.
    fn cut_to(&mut self, to: (f32, f32)) {
        if (to.0 - self.pos.0).abs() > 0.5 {
            self.facing_right = to.0 > self.pos.0;
        }
        self.pos = to;
    }

    pub fn walking(&self) -> bool {
        matches!(
            self.steps.front(),
            Some(Step::Walk { .. } | Step::Stride { .. })
        )
    }

    pub fn current_beat(&self) -> Option<&Beat> {
        match self.steps.front() {
            Some(Step::Beat(beat)) => Some(beat),
            _ => None,
        }
    }

    /// Free to be asked to do something else: nothing queued but idling.
    pub fn is_free(&self) -> bool {
        self.steps
            .iter()
            .all(|step| matches!(step, Step::Beat(beat) if beat.idle))
    }

    pub fn is_idle(&self) -> bool {
        self.steps.is_empty()
    }

    /// Where it is headed, or where it is if it is going nowhere.
    pub fn destination(&self) -> (f32, f32) {
        self.steps
            .iter()
            .rev()
            .find_map(|step| match step {
                Step::Walk { to } | Step::Stride { to } => Some(*to),
                _ => None,
            })
            .unwrap_or(self.pos)
    }

    /// How far into the current step it is.
    pub fn step_elapsed(&self, now: f32) -> f32 {
        (now - self.step_since).max(0.0)
    }

    /// Carries out its steps for `dt` seconds up to `now`.
    pub fn advance(&mut self, now: f32, dt: f32, everyone: &[Whereabouts]) {
        let find = |id: Id| everyone.iter().find(|other| other.id == id);
        let mut budget = dt;
        while let Some(step) = self.steps.front().copied() {
            let done = match step {
                Step::Walk { to } | Step::Stride { to } if self.reduce_motion => {
                    self.cut_to(to);
                    true
                }
                Step::Walk { to } | Step::Stride { to } => {
                    let (dx, dy) = (to.0 - self.pos.0, to.1 - self.pos.1);
                    let distance = (dx * dx + dy * dy).sqrt();
                    if dx.abs() > 0.5 {
                        self.facing_right = dx > 0.0;
                    }
                    let speed = match step {
                        Step::Stride { .. } => self.character.walk_speed() * STRIDE,
                        _ => self.character.walk_speed(),
                    };
                    let stride = speed * budget;
                    if stride >= distance {
                        self.pos = to;
                        budget -= distance / speed;
                        true
                    } else {
                        self.pos.0 += dx / distance * stride;
                        self.pos.1 += dy / distance * stride;
                        budget = 0.0;
                        false
                    }
                }
                Step::Face(id) => {
                    if let Some(other) = find(id) {
                        self.face_towards(other.pos.0);
                    }
                    true
                }
                Step::FaceX(x) => {
                    self.face_towards(x);
                    true
                }
                Step::Await(id) => find(id).is_none_or(|other| !other.walking),
                Step::Beat(beat) => now - self.step_since >= beat.seconds,
            };
            if !done {
                break;
            }
            self.steps.pop_front();
            self.step_since = now;
        }
    }

    fn face_towards(&mut self, x: f32) {
        if (x - self.pos.0).abs() > 1.0 {
            self.facing_right = x > self.pos.0;
        }
    }

    /// What its body and face show at `now`.
    fn pose(&self, now: f32) -> FrameKey {
        let elapsed = self.step_elapsed(now);
        let still = self.reduce_motion;
        let (clip, frame, expression, mut facing_right) = match self.steps.front() {
            Some(Step::Walk { .. } | Step::Stride { .. }) => {
                let clip = BodyClip::Action(ActionKind::Traverse);
                (
                    clip,
                    frame_of(clip, elapsed, still),
                    self.character.walk_face(),
                    self.facing_right,
                )
            }
            Some(Step::Beat(beat)) => {
                let frame = if beat.held {
                    0
                } else {
                    frame_of(beat.clip, elapsed, still)
                };
                let turned = beat.spin && !still && (elapsed / SPIN_TURN) as i32 % 2 == 1;
                (
                    beat.clip,
                    frame,
                    beat.expression,
                    self.facing_right ^ turned,
                )
            }
            _ => {
                let clip = BodyClip::Action(ActionKind::Idle);
                let breathing = now + self.blink.1;
                (
                    clip,
                    frame_of(clip, breathing, still),
                    self.character.idle_face(),
                    self.facing_right,
                )
            }
        };
        let asleep = clip == BodyClip::Action(ActionKind::Sleep);
        let eyelids = if asleep {
            EyelidPose::Closed
        } else if expression == ExpressionKind::Sleepy {
            EyelidPose::Half
        } else if !still && (now + self.blink.1) % self.blink.0 < 0.12 {
            EyelidPose::Closed
        } else {
            EyelidPose::Open
        };
        let gaze = if self.walking() || asleep {
            GazeDirection::default()
        } else {
            self.gaze
        };
        if self.reduce_motion {
            facing_right = self.facing_right;
        }
        FrameKey {
            clip,
            frame,
            facing_right,
            expression,
            eyelids,
            gaze,
        }
    }

    fn frame(&mut self, key: FrameKey) -> &Frame {
        if self.frames.len() >= FRAME_CACHE_LIMIT {
            self.frames.clear();
        }
        let (appearance, dress, reduce_motion) = (&self.appearance, self.dress, self.reduce_motion);
        self.frames.entry(key).or_insert_with(|| {
            let canvas = CreatureRenderer::render_dressed_composited_frame(
                appearance,
                dress,
                key.clip,
                key.frame,
                key.facing_right,
                reduce_motion,
                FaceRenderState {
                    expression: key.expression,
                    eyelids: key.eyelids,
                    gaze: key.gaze,
                },
            );
            let size = FRAME_SIZE as i32;
            let bounds = canvas
                .alpha_bounds()
                .map(|(a, b, c, d)| (a as i32, b as i32, c as i32, d as i32))
                .unwrap_or((0, 0, size - 1, size - 1));
            Frame { canvas, bounds }
        })
    }

    fn origin(&self) -> (i32, i32) {
        (
            self.pos.0.round() as i32 - FRAME_SIZE as i32 / 2,
            self.pos.1.round() as i32 - self.foot_row,
        )
    }

    /// The opaque part of what it shows at `now`, in scene pixels, inclusive.
    pub fn bounds(&mut self, now: f32) -> (i32, i32, i32, i32) {
        let key = self.pose(now);
        let (x, y) = self.origin();
        let (left, top, right, bottom) = self.frame(key).bounds;
        (x + left, y + top, x + right, y + bottom)
    }

    /// The shade under its feet. Drawn for everyone before anyone, so nobody's shadow falls on
    /// somebody else.
    pub fn draw_shadow(&mut self, scene: &mut Canvas, now: f32) {
        let (left, _, right, _) = self.bounds(now);
        let half = ((right - left) / 2 - 2).max(4);
        let (cx, cy) = (self.pos.0.round() as i32, self.pos.1.round() as i32 + 1);
        ellipse(scene, cx, cy, half, 1, rgba(0x2c3a24, 60));
        ellipse(scene, cx, cy, half - 3, 1, rgba(0x2c3a24, 40));
    }

    pub fn draw(&mut self, scene: &mut Canvas, now: f32) {
        let key = self.pose(now);
        let (x, y) = self.origin();
        let frame = &self.frame(key).canvas;
        // `blit` takes the frame by reference while `self` is borrowed for the cache.
        let frame = frame.clone();
        blit(scene, &frame, x, y);
    }
}

fn frame_of(clip: BodyClip, elapsed: f32, still: bool) -> u8 {
    if still {
        0
    } else {
        AnimationSpec::for_clip(clip).frame_at(elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_core::Gesture;

    fn actor() -> Actor {
        let cast = crate::cast::Cast::new(formiga_travel::sample::snapshot()).unwrap();
        Actor::new(&cast.members[0], (100.0, 180.0), true, false)
    }

    #[test]
    fn walking_moves_it_at_its_own_pace_and_turns_it_the_way_it_goes() {
        let mut actor = actor();
        actor.begin(0.0, [Step::Walk { to: (40.0, 180.0) }]);
        actor.advance(0.0, 1.0, &[]);
        let moved = 100.0 - actor.pos.0;
        assert!((moved - actor.character.walk_speed()).abs() < 0.01);
        assert!(!actor.facing_right);
        actor.advance(1.0, 10.0, &[]);
        assert_eq!(actor.pos, (40.0, 180.0));
        assert!(actor.is_idle());
    }

    #[test]
    fn with_reduced_motion_a_walk_is_a_cut() {
        let cast = crate::cast::Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut actor = Actor::new(&cast.members[0], (100.0, 180.0), true, true);
        actor.begin(0.0, [Step::Stride { to: (40.0, 180.0) }]);
        assert_eq!(actor.pos, (40.0, 180.0), "there before it is first drawn");
        assert!(!actor.facing_right);
        assert!(actor.is_idle());

        let wave = Beat::new(Gesture::Reach, ExpressionKind::Joy, 1.0);
        actor.begin(0.0, [Step::Beat(wave), Step::Walk { to: (160.0, 170.0) }]);
        actor.advance(0.5, 0.5, &[]);
        assert_eq!(actor.pos, (40.0, 180.0), "it waves where it is first");
        actor.advance(1.0, 0.5, &[]);
        assert_eq!(actor.pos, (160.0, 170.0));
        assert!(actor.facing_right);
        assert!(actor.is_idle());
    }

    #[test]
    fn beats_last_as_long_as_they_say() {
        let mut actor = actor();
        actor.begin(
            0.0,
            [Step::Beat(Beat::new(
                Gesture::Cheer,
                ExpressionKind::Joy,
                1.0,
            ))],
        );
        actor.advance(0.5, 0.5, &[]);
        assert!(actor.current_beat().is_some());
        actor.advance(1.0, 0.5, &[]);
        assert!(actor.is_idle());
    }

    #[test]
    fn waiting_for_someone_lasts_until_they_arrive() {
        let mut actor = actor();
        actor.begin(0.0, [Step::Await(7)]);
        let walking = [Whereabouts {
            id: 7,
            pos: (0.0, 0.0),
            walking: true,
        }];
        actor.advance(0.0, 0.1, &walking);
        assert!(!actor.is_idle());
        let arrived = [Whereabouts {
            id: 7,
            pos: (60.0, 0.0),
            walking: false,
        }];
        actor.begin(0.1, [Step::Await(7), Step::Face(7)]);
        actor.advance(0.1, 0.1, &arrived);
        assert!(actor.is_idle());
        assert!(
            !actor.facing_right,
            "it turned to face the one it waited for"
        );
    }

    #[test]
    fn it_stands_with_its_feet_where_it_is() {
        let mut actor = actor();
        let (_, _, _, bottom) = actor.bounds(0.0);
        assert!(
            (bottom - 180).abs() <= 1,
            "feet at {bottom}, standing at 180"
        );
    }

    #[test]
    fn frames_are_drawn_once_and_reused() {
        let mut actor = actor();
        let mut scene = Canvas::new(200, 216);
        actor.draw(&mut scene, 0.0);
        let drawn = actor.frames.len();
        actor.draw(&mut scene, 0.0);
        assert_eq!(actor.frames.len(), drawn);
    }
}
