//! A traveller performing on a stage: where it stands, which way it faces, what it is doing, and
//! the frames `formiga-art` draws for it, face and all.
//!
//! What an actor does is a queue of steps: walk somewhere, turn to someone, wait for them, play a
//! beat. The area it is in decides what goes in the queue; the actor only carries it out.

use crate::cast::{Id, Member};
use crate::character::{Beat, Character};
use crate::fairground::prizes::{self, CARRIED_ABOVE, Prize};
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
    /// The top of its head above its face, between its ears, in the frame's own pixels: where a
    /// hat sits.
    crown: (i32, i32),
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
    /// Where its face is in a resting frame facing right, and how far below its crown: where a
    /// costume piece is placed.
    face: (i32, i32),
    face_below_crown: i32,
    /// A piece from the Green's dress-up box, worn for the visit, if it has one on.
    costume: Option<&'static str>,
    /// A prize won at hoopla, carried for the visit, if it has one.
    prize: Option<Prize>,
    /// Groomed till it shines, for the rest of the visit.
    shining: bool,
    /// In a sack for the sack race, and which one: each racer's has its own coloured band.
    sack: Option<u8>,
    /// How far across its sack goes in a frame facing right, from its resting frame, so the
    /// sack keeps its size whatever it does in it.
    girth: Option<(i32, i32)>,
    /// How wide and how tall it stands at rest, in pixels: how much room it takes in a line, and
    /// how much of whoever is behind it it hides.
    width: f32,
    height: f32,
    /// Tipped over on its side, as after a tumble in its sack.
    tipped: bool,
    /// The frame with its sack and costume on, turned on its side, as last drawn tipped over.
    turned: Option<(FrameKey, Canvas)>,
    /// The frame row its feet rest on.
    foot_row: i32,
    pub pos: (f32, f32),
    pub facing_right: bool,
    pub gaze: GazeDirection,
    /// How far off the ground it is drawn: peeking up over something it is hiding behind.
    pub lift: f32,
    steps: VecDeque<Step>,
    /// When the step at the front began.
    step_since: f32,
    frames: HashMap<FrameKey, Frame>,
    /// Each frame's sack, by the frame and the sack, drawn on first use as frames are.
    sacks: HashMap<(FrameKey, u8), Canvas>,
    /// Blinks come round every `period` seconds, offset by `phase`, so no two are in step.
    blink: (f32, f32),
    reduce_motion: bool,
}

impl Actor {
    pub fn new(member: &Member, pos: (f32, f32), facing_right: bool, reduce_motion: bool) -> Self {
        let baseline = CreatureRenderer::resting_baseline(member.genome(), reduce_motion);
        let seed = (member.id % 1009) as f32 / 1009.0;
        let resting = CreatureRenderer::render_dressed_body_frame(
            member.genome(),
            member.dress,
            ActionKind::Idle,
            0,
            reduce_motion,
        );
        let face = (resting.face_anchor.x, resting.face_anchor.y);
        let face_below_crown = face.1 - crown_of(&resting.canvas, face.0).unwrap_or(face.1);
        Self {
            id: member.id,
            character: Character::of(member),
            appearance: member.genome().clone(),
            dress: member.dress,
            face,
            face_below_crown,
            costume: None,
            prize: None,
            shining: false,
            sack: None,
            girth: crate::fairground::gear::girth(&resting.canvas),
            width: resting
                .canvas
                .alpha_bounds()
                .map_or(FRAME_SIZE as f32 / 2.0, |(left, _, right, _)| {
                    (right - left + 1) as f32
                }),
            height: resting
                .canvas
                .alpha_bounds()
                .map_or(FRAME_SIZE as f32 / 2.0, |(_, top, _, bottom)| {
                    (bottom - top + 1) as f32
                }),
            tipped: false,
            turned: None,
            foot_row: FRAME_SIZE as i32 - 1 - baseline as i32,
            pos,
            facing_right,
            gaze: GazeDirection::default(),
            lift: 0.0,
            steps: VecDeque::new(),
            step_since: 0.0,
            frames: HashMap::new(),
            sacks: HashMap::new(),
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

    /// How wide it stands at rest.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// How tall it stands at rest.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Where the middle of its face is drawn, in scene pixels, as it stands at rest.
    #[cfg(test)]
    pub fn face_at(&self) -> (i32, i32) {
        let (x, y) = self.origin();
        (x + face_x(self.face.0, self.facing_right), y + self.face.1)
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
        let face_x = face_x(self.face.0, key.facing_right);
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
            let crown = (face_x, crown_of(&canvas, face_x).unwrap_or(bounds.1));
            Frame {
                canvas,
                bounds,
                crown,
            }
        })
    }

    fn origin(&self) -> (i32, i32) {
        (
            self.pos.0.round() as i32 - FRAME_SIZE as i32 / 2,
            (self.pos.1 - self.lift).round() as i32 - self.foot_row,
        )
    }

    /// The opaque part of what it shows at `now`, in scene pixels, inclusive: what it wears too,
    /// so a tall hat is pointed at, and tagged above, along with its wearer.
    pub fn bounds(&mut self, now: f32) -> (i32, i32, i32, i32) {
        let key = self.pose(now);
        if self.tipped {
            let (turned, (x, y)) = self.turned_over(key);
            let (left, top, right, bottom) =
                turned.alpha_bounds().map_or((0, 0, 0, 0), |(a, b, c, d)| {
                    (a as i32, b as i32, c as i32, d as i32)
                });
            return (x + left, y + top, x + right, y + bottom);
        }
        let (x, y) = self.origin();
        let frame = self.frame(key);
        let (left, top, right, bottom) = frame.bounds;
        let crown = (x + frame.crown.0, y + frame.crown.1);
        let body = (x + left, y + top, x + right, y + bottom);
        let worn = self.costume.and_then(|id| {
            let (sprite, at) = placed_costume(id, crown, self.face_below_crown, key.facing_right)?;
            let (l, t, r, b) = sprite.alpha_bounds()?;
            Some((
                at.0 + l as i32,
                at.1 + t as i32,
                at.0 + r as i32,
                at.1 + b as i32,
            ))
        });
        let body = match worn {
            Some((l, t, r, b)) => (body.0.min(l), body.1.min(t), body.2.max(r), body.3.max(b)),
            None => body,
        };
        // A prize carried rises over its head, either side of it.
        match self.prize {
            Some(_) => (
                body.0.min(crown.0 - 12),
                body.1.min(crown.1 - CARRIED_ABOVE),
                body.2.max(crown.0 + 12),
                body.3,
            ),
            None => body,
        }
    }

    /// The shade under its feet, as wide as its body whatever it wears. Drawn for everyone
    /// before anyone, so nobody's shadow falls on somebody else.
    pub fn draw_shadow(&mut self, scene: &mut Canvas, now: f32) {
        let key = self.pose(now);
        let (x, _) = self.origin();
        let (left, _, right, _) = self.frame(key).bounds;
        let (left, right) = (x + left, x + right);
        let half = ((right - left) / 2 - 2).max(4);
        let (cx, cy) = (self.pos.0.round() as i32, self.pos.1.round() as i32 + 1);
        ellipse(scene, cx, cy, half, 1, rgba(0x2c3a24, 60));
        ellipse(scene, cx, cy, half - 3, 1, rgba(0x2c3a24, 40));
    }

    /// What it shows at `now`, on its own canvas: for a close-up.
    pub fn picture(&mut self, now: f32) -> Canvas {
        let key = self.pose(now);
        self.frame(key).canvas.clone()
    }

    pub fn draw(&mut self, scene: &mut Canvas, now: f32) {
        let key = self.pose(now);
        if self.tipped {
            let (turned, (x, y)) = self.turned_over(key);
            blit(scene, &turned, x, y);
            return;
        }
        let (x, y) = self.origin();
        let frame = self.frame(key);
        // `blit` takes the frame by reference while `self` is borrowed for the cache.
        let (canvas, crown) = (frame.canvas.clone(), frame.crown);
        blit(scene, &canvas, x, y);
        if self.shining {
            draw_shine(scene, &canvas, (x, y), now, self.reduce_motion, self.id);
        }
        if let Some(sack) = self.sack_for(key) {
            blit(scene, &sack, x, y);
        }
        if let Some(id) = self.costume {
            draw_costume(
                scene,
                id,
                (x + crown.0, y + crown.1),
                self.face_below_crown,
                key.facing_right,
            );
        }
        if let Some(prize) = self.prize {
            prizes::carried(
                scene,
                prize,
                (x + crown.0, y + crown.1),
                self.face_below_crown + THROAT_BELOW_FACE,
                key.facing_right,
                now,
                self.reduce_motion,
            );
        }
    }

    /// Puts on a piece from the dress-up box, or takes it off.
    pub fn wear(&mut self, costume: Option<&'static str>) {
        self.costume = costume;
    }

    /// Carries a prize won at hoopla, or nothing.
    pub fn carry(&mut self, prize: Option<Prize>) {
        self.prize = prize;
    }

    /// Groomed till it shines, or not.
    pub fn shine(&mut self, shining: bool) {
        self.shining = shining;
    }

    /// Climbs into a sack for the sack race, the one with this band, or out of it.
    pub fn sack(&mut self, sack: Option<u8>) {
        self.sack = sack;
    }

    /// Tips over on its side, or gets up again.
    pub fn tip(&mut self, tipped: bool) {
        self.tipped = tipped;
        if !tipped {
            self.turned = None;
        }
    }

    /// The sack drawn over this frame, if it is in one.
    fn sack_for(&mut self, key: FrameKey) -> Option<Canvas> {
        let band = self.sack?;
        if self.sacks.len() >= FRAME_CACHE_LIMIT {
            self.sacks.clear();
        }
        if !self.sacks.contains_key(&(key, band)) {
            let (from, to) = self.girth?;
            // The renderer draws facing right and mirrors.
            let girth = if key.facing_right {
                (from, to)
            } else {
                (FRAME_SIZE as i32 - 1 - to, FRAME_SIZE as i32 - 1 - from)
            };
            let frame = self.frame(key).canvas.clone();
            let sack = crate::fairground::gear::sack(&frame, band, girth);
            self.sacks.insert((key, band), sack);
        }
        self.sacks.get(&(key, band)).cloned()
    }

    /// The frame with whatever it has on, turned on its side with its head the way it was
    /// facing, as if it had fallen forward, and where its top-left goes in the scene: lying on
    /// the ground where it stood.
    fn turned_over(&mut self, key: FrameKey) -> (Canvas, (i32, i32)) {
        if !matches!(&self.turned, Some((turned_key, _)) if *turned_key == key) {
            let frame = self.frame(key);
            let (mut whole, crown) = (frame.canvas.clone(), frame.crown);
            if let Some(sack) = self.sack_for(key) {
                blit(&mut whole, &sack, 0, 0);
            }
            if let Some(id) = self.costume {
                draw_costume(
                    &mut whole,
                    id,
                    crown,
                    self.face_below_crown,
                    key.facing_right,
                );
            }
            // Anything longer than it is tall rolls onto its back instead, legs in the air.
            let (left, top, right, bottom) = whole.alpha_bounds().unwrap_or((0, 0, 0, 0));
            let fallen = if right - left > bottom - top {
                upside_down(&whole)
            } else {
                turn(&whole, key.facing_right)
            };
            self.turned = Some((key, fallen));
        }
        let turned = self.turned.as_ref().map(|(_, canvas)| canvas.clone());
        let turned = turned.unwrap_or_else(|| Canvas::new(1, 1));
        let (left, _, right, bottom) =
            turned.alpha_bounds().map_or((0, 0, 0, 0), |(a, b, c, d)| {
                (a as i32, b as i32, c as i32, d as i32)
            });
        let at = (
            self.pos.0.round() as i32 - (left + right) / 2,
            (self.pos.1 - self.lift).round() as i32 - bottom,
        );
        (turned, at)
    }
}

/// A picture turned upside down.
fn upside_down(canvas: &Canvas) -> Canvas {
    let height = canvas.height() as i32;
    let mut turned = Canvas::new(canvas.width(), canvas.height());
    for y in 0..height {
        for x in 0..canvas.width() as i32 {
            turned.set(x, height - 1 - y, canvas.get(x, y));
        }
    }
    turned
}

/// A picture turned a quarter round: clockwise, so its top comes to the right, if `clockwise`;
/// otherwise the other way. Pixel for pixel, as a quarter turn loses nothing.
fn turn(canvas: &Canvas, clockwise: bool) -> Canvas {
    let (width, height) = (canvas.width() as i32, canvas.height() as i32);
    let mut turned = Canvas::new(canvas.height(), canvas.width());
    for y in 0..height {
        for x in 0..width {
            let pixel = canvas.get(x, y);
            if pixel.a == 0 {
                continue;
            }
            let (tx, ty) = if clockwise {
                (height - 1 - y, x)
            } else {
                (y, width - 1 - x)
            };
            turned.set(tx, ty, pixel);
        }
    }
    turned
}

/// A freshly groomed coat: a few glints that come and go across it, each on a pixel of the
/// creature itself. With reduced motion they hold still.
fn draw_shine(
    scene: &mut Canvas,
    frame: &Canvas,
    (x, y): (i32, i32),
    now: f32,
    reduce_motion: bool,
    id: Id,
) {
    let Some((left, top, right, bottom)) = frame.alpha_bounds() else {
        return;
    };
    let (left, top, right, bottom) = (left as i32, top as i32, right as i32, bottom as i32);
    let beat = if reduce_motion { 0 } else { (now * 2.5) as u32 };
    for glint in 0..3u32 {
        let salt = (id as u32).wrapping_add(glint * 97).wrapping_add(beat * 13);
        let gx = left
            + (crate::paint::noise(glint as i32, beat as i32, salt) % (right - left + 1) as u32)
                as i32;
        let gy = top
            + (crate::paint::noise(beat as i32, glint as i32, salt)
                % ((bottom - top) / 2 + 1) as u32) as i32;
        if frame.get(gx, gy).a > 0 {
            let at = (x + gx, y + gy);
            crate::paint::put(
                scene,
                at.0,
                at.1,
                formiga_art::Rgba::new(255, 255, 255, 230),
            );
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                crate::paint::put(
                    scene,
                    at.0 + dx,
                    at.1 + dy,
                    formiga_art::Rgba::new(255, 250, 220, 120),
                );
            }
        }
    }
}

/// Where the face is across a frame facing this way: the renderer draws facing right and mirrors.
fn face_x(face_x: i32, facing_right: bool) -> i32 {
    if facing_right {
        face_x
    } else {
        FRAME_SIZE as i32 - 1 - face_x
    }
}

/// The top of the head above `x`: the highest row where what is drawn across `x` is as wide as a
/// head, so a hat sits on the head itself rather than on an ear tip, a tuft or an antenna.
fn crown_of(frame: &Canvas, x: i32) -> Option<i32> {
    let width = frame.width() as i32;
    let solid = |cx: i32, y: i32| cx >= 0 && cx < width && frame.get(cx, y).a > 0;
    let wide = |y: i32| {
        (x - 1..=x + 1).filter(|&cx| solid(cx, y)).any(|start| {
            let mut left = start;
            while solid(left - 1, y) {
                left -= 1;
            }
            let mut right = start;
            while solid(right + 1, y) {
                right += 1;
            }
            right - left + 1 >= HEAD_WIDE
        })
    };
    (0..frame.height() as i32)
        .find(|&y| wide(y))
        .or_else(|| (0..frame.height() as i32).find(|&y| (x - 1..=x + 1).any(|cx| solid(cx, y))))
}

/// How far under the middle of the face the throat is, where a neck piece sits.
const THROAT_BELOW_FACE: i32 = 6;
/// How wide a row must be to count as the top of a head rather than an ear. Eight, not seven: a
/// long ear with a speck beside it, as one shows between its ears mid-cheer, makes seven, and a
/// hat put there sits on the ear tips.
const HEAD_WIDE: i32 = 8;

/// A costume piece on a companion whose crown is at `crown` in the scene, its throat
/// `face_below_crown` and a little further down, facing the way it faces.
fn draw_costume(
    scene: &mut Canvas,
    id: &str,
    crown: (i32, i32),
    face_below_crown: i32,
    facing_right: bool,
) {
    if let Some((sprite, at)) = placed_costume(id, crown, face_below_crown, facing_right) {
        blit(scene, &sprite, at.0, at.1);
    }
}

/// A costume piece as it is drawn on its wearer, and where its top-left corner goes in the scene.
fn placed_costume(
    id: &str,
    crown: (i32, i32),
    face_below_crown: i32,
    facing_right: bool,
) -> Option<(Canvas, (i32, i32))> {
    let piece = crate::costume::piece(id)?;
    let mut sprite = crate::costume::art::sprite(id);
    let (mut ax, ay) = crate::costume::art::anchor(id);
    if !facing_right {
        let (width, height) = (sprite.width() as i32, sprite.height() as i32);
        let mut mirrored = Canvas::new(sprite.width(), sprite.height());
        for py in 0..height {
            for px in 0..width {
                mirrored.set(width - 1 - px, py, sprite.get(px, py));
            }
        }
        sprite = mirrored;
        ax = width - 1 - ax;
    }
    let point = match piece.slot {
        crate::costume::Slot::Crown => crown,
        // The throat: just under the face, however tall the head above it.
        crate::costume::Slot::Neck => (crown.0, crown.1 + face_below_crown + THROAT_BELOW_FACE),
    };
    Some((sprite, (point.0 - ax, point.1 - ay)))
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
    fn what_it_wears_is_part_of_it() {
        let mut actor = actor();
        let bare = actor.bounds(0.0);
        actor.wear(Some("top_hat"));
        let hatted = actor.bounds(0.0);
        assert!(
            hatted.1 < bare.1,
            "the hat rises above {bare:?}: {hatted:?}"
        );
        // Every pixel of the hat as drawn is within what can be pointed at.
        let mut scene = Canvas::new(200, 216);
        actor.draw(&mut scene, 0.0);
        let mut bare_scene = Canvas::new(200, 216);
        actor.wear(None);
        actor.draw(&mut bare_scene, 0.0);
        for y in 0..216 {
            for x in 0..200 {
                if scene.get(x, y) != bare_scene.get(x, y) {
                    let (l, t, r, b) = hatted;
                    assert!(
                        (l..=r).contains(&x) && (t..=b).contains(&y),
                        "({x}, {y}) of the hat is outside {hatted:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_sack_goes_over_its_lower_body_and_it_stands_where_it_did() {
        let (mut plain, mut sacked) = (actor(), actor());
        sacked.sack(Some(2));
        let (mut without, mut with) = (Canvas::new(200, 216), Canvas::new(200, 216));
        plain.draw(&mut without, 0.0);
        sacked.draw(&mut with, 0.0);
        assert_ne!(without, with);
        let (_, top, _, bottom) = plain.bounds(0.0);
        assert_eq!(
            sacked.bounds(0.0).3,
            bottom,
            "standing in its sack where it stood"
        );
        // Its head and shoulders are its own.
        for y in 0..top + (bottom - top) / 3 {
            for x in 0..200 {
                assert_eq!(without.get(x, y), with.get(x, y), "({x}, {y}) changed");
            }
        }
        sacked.sack(None);
        let mut again = Canvas::new(200, 216);
        sacked.draw(&mut again, 0.0);
        assert_eq!(again, without, "out of the sack again");
    }

    #[test]
    fn tipped_over_it_lies_on_its_side_where_it_stood_and_gets_up_again() {
        let mut actor = actor();
        let (left, top, right, bottom) = actor.bounds(0.0);
        actor.tip(true);
        let lying = actor.bounds(0.0);
        assert_eq!(lying.3, bottom, "on the ground where it stood");
        let (wide, tall) = (right - left, bottom - top);
        if wide > tall {
            assert_eq!(
                (lying.2 - lying.0, lying.3 - lying.1),
                (wide, tall),
                "on its back"
            );
        } else {
            assert_eq!(
                (lying.2 - lying.0, lying.3 - lying.1),
                (tall, wide),
                "on its side"
            );
        }
        actor.tip(false);
        assert_eq!(actor.bounds(0.0), (left, top, right, bottom));
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
