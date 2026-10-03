//! How each fish looks when it is landed and held up, and its icon for the journal.
//!
//! Every fish but the minnow is built the same way, so that they look as if they share a pool: a
//! body that swells from the tail to the shoulder and rounds off at the snout, banded from a dark
//! back to a pale belly and lit from the upper left; fins behind it and in front of it, each
//! showing its rays; then what makes the kind itself (the perch's bars, the trout's haloed spots,
//! the carp's big scales), the gill cover, the mouth and the eye. A kind is drawn as long as the
//! biggest of it grows, and its fins and markings are placed along its body rather than at fixed
//! pixels, so the drawing follows the catalogue. The minnow is too small to build: it is placed
//! pixel by pixel.

use super::fish::fish;
use crate::paint::{Ramp, chance, mix, noise, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Icons are this many pixels square, as finds' are.
pub const ICON: u32 = 9;

/// How long a fish of `length` centimetres is drawn, in pixels, when it is held up.
pub fn drawn_length(length: f32) -> u32 {
    (8.0 + length * 0.36).round() as u32
}

/// A fish side on, facing right, drawn at the size of the biggest of its kind.
pub fn catch(id: &str) -> Canvas {
    let length = drawn_length(fish(id).map_or(20.0, |fish| fish.length.1)) as i32;
    match id {
        "minnow" => minnow(length),
        "roach" => roach(length),
        "perch" => perch(length),
        "trout" => trout(length),
        "carp" => carp(length),
        "pike" => pike(length),
        "golden_tench" => tench(length),
        "old_one" => old_one(length),
        _ => plain(length),
    }
}

/// A fish's icon for the journal, nine pixels square; empty for a fish Hill doesn't know.
pub fn icon(id: &str) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    let (rows, inks): (&[&str; 9], Vec<(u8, Rgba)>) = match id {
        "minnow" => (&MINNOW_ICON, minnow_inks()),
        "roach" => (&ROACH_ICON, roach_inks()),
        "perch" => (&PERCH_ICON, perch_inks()),
        "trout" => (&TROUT_ICON, trout_inks()),
        "carp" => (&CARP_ICON, carp_inks()),
        "pike" => (&PIKE_ICON, pike_inks()),
        "golden_tench" => (&TENCH_ICON, tench_inks()),
        "old_one" => (&OLD_ONE_ICON, old_one_inks()),
        _ => return canvas,
    };
    stamp(&mut canvas, (0, 0), rows, &inks);
    canvas
}

// ---------------------------------------------------------------------------------------------
// Bodies, fins and eyes
// ---------------------------------------------------------------------------------------------

/// A fish's body, side on and facing right: how deep it is at each point along it, from where
/// the tail fin joins (`u` = 0) to the tip of the snout (`u` = 1).
#[derive(Clone, Copy)]
struct Body {
    /// Where the tail joins and where the snout ends, in pixels across the canvas.
    root: f32,
    nose: f32,
    /// The middle of the body at its deepest, in pixels down the canvas.
    mid: f32,
    /// How deep it is at its deepest, in pixels.
    depth: f32,
    /// How far along it is deepest.
    peak: f32,
    /// How deep it is where the tail joins, and at the snout, as shares of its depth.
    wrist: f32,
    snout: f32,
    /// How much of its depth is above the middle: more for a humped back.
    above: f32,
    /// How the head rounds off: 2 for an even curve, less for a tapering one.
    head: f32,
    /// How far below the middle the snout ends, in pixels: a brow that slopes down to the lips.
    chin: f32,
    /// How far the tail end is lifted, in pixels: a landed fish flexes.
    flex: f32,
}

impl Body {
    fn along(&self, x: f32) -> f32 {
        (x - self.root) / (self.nose - self.root)
    }

    fn x(&self, u: f32) -> f32 {
        self.root + u * (self.nose - self.root)
    }

    /// How far along the head `u` is, from the deepest point (0) to the snout (1).
    fn forward(&self, u: f32) -> f32 {
        ((u - self.peak) / (1.0 - self.peak)).clamp(0.0, 1.0)
    }

    /// How deep the body is at `u`, as a share of its depth.
    fn swell(&self, u: f32) -> f32 {
        if u <= self.peak {
            let t = (u / self.peak).clamp(0.0, 1.0);
            self.wrist + (1.0 - self.wrist) * (t * FRAC_PI_2).sin()
        } else {
            let t = self.forward(u);
            let round = (1.0 - t.powf(self.head)).max(0.0).powf(1.0 / self.head);
            self.snout + (1.0 - self.snout) * round
        }
    }

    fn centre(&self, u: f32) -> f32 {
        self.mid - self.flex * (1.0 - u.clamp(0.0, 1.0)).powi(2)
            + self.chin * self.forward(u).powi(2)
    }

    fn top(&self, u: f32) -> f32 {
        self.centre(u) - self.depth * self.above * self.swell(u)
    }

    fn bottom(&self, u: f32) -> f32 {
        self.centre(u) + self.depth * (1.0 - self.above) * self.swell(u)
    }

    /// How steeply the middle of the body climbs towards the tail, in pixels for each pixel.
    fn lift(&self) -> f32 {
        2.0 * self.flex / (self.nose - self.root)
    }

    fn holds(&self, x: i32, y: i32) -> bool {
        let u = self.along(x as f32 + 0.5);
        let y = y as f32 + 0.5;
        (0.0..=1.0).contains(&u) && self.top(u) <= y && y <= self.bottom(u)
    }

    /// Whether a pixel of the body is on its outline. Where the tail joins is not: the tail
    /// carries on from there.
    fn rim(&self, x: i32, y: i32) -> bool {
        let joined = |x: i32, y: i32| {
            self.holds(x, y)
                || (self.along(x as f32 + 0.5) < 0.0 && {
                    let y = y as f32 + 0.5;
                    self.top(0.0) <= y && y <= self.bottom(0.0)
                })
        };
        !(joined(x - 1, y) && joined(x + 1, y) && joined(x, y - 1) && joined(x, y + 1))
    }

    /// Where a pixel sits on the body: along it, and across it from the back (-1) to the belly
    /// (1).
    fn place(&self, x: i32, y: i32) -> (f32, f32) {
        let u = self.along(x as f32 + 0.5);
        let (top, bottom) = (self.top(u), self.bottom(u));
        (u, (y as f32 + 0.5 - top) / (bottom - top) * 2.0 - 1.0)
    }

    /// The pixel at a place on the body.
    fn point(&self, u: f32, v: f32) -> (i32, i32) {
        let (top, bottom) = (self.top(u), self.bottom(u));
        (
            self.x(u).floor() as i32,
            (top + (v + 1.0) / 2.0 * (bottom - top)).floor() as i32,
        )
    }
}

/// What a body is painted in, from its back to its belly: across it, the back gives way to the
/// flank at `upper` and the flank to the belly at `lower`.
struct Coat {
    back: Ramp,
    flank: Ramp,
    belly: Ramp,
    upper: f32,
    lower: f32,
}

impl Coat {
    fn ramp(&self, across: f32) -> Ramp {
        if across < self.upper {
            self.back
        } else if across < self.lower {
            self.flank
        } else {
            self.belly
        }
    }
}

/// How much light falls on a body at `(u, v)`: it is rounded across, so the upper flank faces
/// the light from the upper left, and its ends turn, the head away from the light and the tail
/// towards it.
fn light(body: &Body, u: f32, v: f32) -> f32 {
    let facing = (1.0 - v * v).max(0.0).sqrt();
    let turn = if u > body.peak {
        0.5 * body.forward(u).powi(2)
    } else {
        -0.4 * (body.peak - u) / body.peak
    };
    -0.6 * v + 0.65 * facing - 0.3 * turn
}

fn tone(ramp: Ramp, light: f32) -> Rgba {
    if light > 0.98 {
        ramp.shine
    } else if light > 0.72 {
        ramp.light
    } else if light > 0.3 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// Paints the body in its coat, outlined in each band's own edge tone. Noise dithers where one
/// band and one tone give way to the next, so the body is never striped by its own shading.
fn paint_body(s: &mut Canvas, body: &Body, coat: &Coat, salt: u32) {
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !body.holds(x, y) {
                continue;
            }
            let (u, v) = body.place(x, y);
            let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
            let ramp = coat.ramp(v + jitter * 0.12);
            let color = if body.rim(x, y) {
                ramp.edge
            } else {
                tone(ramp, light(body, u, v) + jitter * 0.08)
            };
            put(s, x, y, color);
        }
    }
}

/// A wet gleam along the upper flank, where the light catches the shoulder: a line of a band's
/// light tone with its shine in the middle.
fn gleam(s: &mut Canvas, body: &Body, coat: &Coat, (from, to): (f32, f32), across: f32) {
    let (x0, x1) = (body.x(from).round() as i32, body.x(to).round() as i32);
    for x in x0..x1 {
        let u = body.along(x as f32 + 0.5);
        let (_, y) = body.point(u, across);
        if !body.holds(x, y) || body.rim(x, y) {
            continue;
        }
        let t = (x - x0) as f32 / (x1 - x0).max(1) as f32;
        let ramp = coat.ramp(across);
        let color = if (0.3..0.7).contains(&t) {
            ramp.shine
        } else {
            ramp.light
        };
        put(s, x, y, color);
    }
}

/// Paints over the inside of the body, leaving its outline, asking `mark` what goes where; it
/// is given the pixel, its place on the body and the colour already there.
fn mark(
    s: &mut Canvas,
    body: &Body,
    mut mark: impl FnMut(i32, i32, f32, f32, Rgba) -> Option<Rgba>,
) {
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !body.holds(x, y) || body.rim(x, y) {
                continue;
            }
            let (u, v) = body.place(x, y);
            if let Some(color) = mark(x, y, u, v, s.get(x, y)) {
                put(s, x, y, color);
            }
        }
    }
}

/// Whether `(x, y)` is inside the polygon `points`, by the even-odd rule.
fn within(points: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut inside = false;
    for (index, &(x0, y0)) in points.iter().enumerate() {
        let (x1, y1) = points[(index + 1) % points.len()];
        if (y0 <= y) != (y1 <= y) && x < x0 + (y - y0) / (y1 - y0) * (x1 - x0) {
            inside = !inside;
        }
    }
    inside
}

/// How a fin's rays spread: from a point, fanning out about a heading, as far as they reach.
struct Rays {
    from: (f32, f32),
    heading: f32,
    reach: f32,
}

/// Paints a fin wherever `inside` says it is within `bounds` (left, top, right, bottom), its rays
/// as stripes once it is big enough to show them. Above `split` it is lit, in `upper`'s base and
/// light; below, in `lower`'s shadow and base. A fin big enough to have an inside is outlined in
/// its own edge tone; a little one, which would be all outline, is edged a tone lighter. A fin is
/// drawn before the body it grows from, so its root is hidden; one lying against the body is
/// drawn after it, and `opacity` lets the body show through.
fn paint_fin(
    s: &mut Canvas,
    inside: &dyn Fn(i32, i32) -> bool,
    (left, top, right, bottom): (i32, i32, i32, i32),
    rays: Rays,
    split: f32,
    (upper, lower): (Ramp, Ramp),
    opacity: u8,
) {
    for y in top..=bottom {
        for x in left..=right {
            if !inside(x, y) {
                continue;
            }
            let lit = (y as f32 + 0.5) < split;
            let ramp = if lit { upper } else { lower };
            let rim =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (dx, dy) = (x as f32 + 0.5 - rays.from.0, y as f32 + 0.5 - rays.from.1);
            let turn = (dy.atan2(dx) - rays.heading + PI).rem_euclid(TAU);
            let ray =
                rays.reach >= 4.5 && ((turn * rays.reach * 0.9).floor() as i32).rem_euclid(2) == 0;
            let color = match (rim, lit, ray) {
                (true, ..) if rays.reach < 3.5 => ramp.shadow,
                (true, ..) => ramp.edge,
                (false, true, true) => ramp.base,
                (false, true, false) => ramp.light,
                (false, false, true) => ramp.shadow,
                (false, false, false) => ramp.base,
            };
            put(s, x, y, Rgba::new(color.r, color.g, color.b, opacity));
        }
    }
}

/// A fin filling the polygon `outline`, its rays spreading from `from`.
fn fin(
    s: &mut Canvas,
    outline: &[(f32, f32)],
    from: (f32, f32),
    split: f32,
    ramps: (Ramp, Ramp),
    opacity: u8,
) {
    let count = outline.len() as f32;
    let middle = (
        outline.iter().map(|point| point.0).sum::<f32>() / count,
        outline.iter().map(|point| point.1).sum::<f32>() / count,
    );
    let rays = Rays {
        from,
        heading: (middle.1 - from.1).atan2(middle.0 - from.0),
        reach: outline
            .iter()
            .map(|&(x, y)| (x - from.0).hypot(y - from.1))
            .fold(1.0, f32::max),
    };
    let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in outline {
        (left, top, right, bottom) = (left.min(x), top.min(y), right.max(x), bottom.max(y));
    }
    let bounds = (
        left.floor() as i32,
        top.floor() as i32,
        right.ceil() as i32,
        bottom.ceil() as i32,
    );
    let inside = |x: i32, y: i32| within(outline, x as f32 + 0.5, y as f32 + 0.5);
    paint_fin(s, &inside, bounds, rays, split, ramps, opacity);
}

/// Which edge of the body a fin grows from.
#[derive(Clone, Copy)]
enum Edge {
    Back,
    Belly,
}

/// A fin growing from the back or the belly at `u`, its outline given in pixels from there:
/// along the body (negative is towards the tail) and out from its edge (negative is into the
/// body, which will cover it). Fins on the back catch the light; those underneath are in shade.
fn fin_on(s: &mut Canvas, body: &Body, edge: Edge, u: f32, points: &[(f32, f32)], ramp: Ramp) {
    let x0 = body.x(u);
    let (outward, split) = match edge {
        Edge::Back => (-1.0, f32::MAX),
        Edge::Belly => (1.0, f32::MIN),
    };
    let rim = |x: f32| {
        let u = body.along(x).clamp(0.0, 1.0);
        match edge {
            Edge::Back => body.top(u),
            Edge::Belly => body.bottom(u),
        }
    };
    let outline: Vec<(f32, f32)> = points
        .iter()
        .map(|&(along, out)| (x0 + along, rim(x0 + along) + outward * out))
        .collect();
    let from = (x0, rim(x0) - outward * 2.5);
    fin(s, &outline, from, split, (ramp, ramp), 255);
}

/// A pectoral fin lying against the flank behind the gill, from the body's `(u, v)`, its outline
/// in pixels from there. It is thin enough for the flank to show through.
fn pectoral(s: &mut Canvas, body: &Body, (u, v): (f32, f32), points: &[(f32, f32)], ramp: Ramp) {
    let (x, y) = body.point(u, v);
    let at = (x as f32 + 0.5, y as f32 + 0.5);
    let outline: Vec<(f32, f32)> = points
        .iter()
        .map(|&(dx, dy)| (at.0 + dx, at.1 + dy))
        .collect();
    fin(s, &outline, (at.0 + 1.0, at.1), f32::MIN, (ramp, ramp), 190);
}

/// How a tail's trailing edge runs.
#[derive(Clone, Copy)]
enum Tail {
    /// Forked, the fork running this share of the way in from the tips.
    Forked(f32),
    Square,
    Round,
}

/// The tail fin, from a pixel inside the body (which covers its root) back to its tips at the
/// left edge of the canvas, `spread` pixels from tip to tip and carrying on the body's flex. It
/// is built a column at a time, so each lobe of a fork keeps at least a pixel all the way to its
/// tip: at this size a lobe drawn as a thin triangle breaks up.
fn tail(s: &mut Canvas, body: &Body, spread: f32, shape: Tail, ramps: (Ramp, Ramp)) {
    let join = body.root + 1.0;
    let (top, bottom) = (body.top(0.0), body.bottom(0.0));
    let (middle, wrist) = ((top + bottom) / 2.0, (bottom - top) / 2.0);
    let lift = body.lift();
    let half = spread / 2.0;
    let inside = |x: i32, y: i32| {
        let across = x as f32 + 0.5;
        if !(0.0..join).contains(&across) {
            return false;
        }
        // From the join (0) to the tips (1).
        let t = 1.0 - across / join;
        let mut outer = wrist + (half - wrist) * t.powf(0.85);
        let mut inner = 0.0;
        match shape {
            Tail::Forked(fork) => {
                let start = 1.0 - fork;
                if t > start {
                    inner = (t - start) / fork * (outer - 1.2).max(0.0);
                }
            }
            Tail::Square => {}
            Tail::Round => {
                if t > 0.55 {
                    let k = (t - 0.55) / 0.5;
                    outer *= 0.35 + 0.65 * (1.0 - k * k).max(0.0).sqrt();
                }
            }
        }
        let off = (y as f32 + 0.5 - (middle - lift * (body.root - across).max(0.0))).abs();
        off <= outer + 0.15 && off >= inner
    };
    let tip = middle - lift * body.root;
    let bounds = (
        0,
        (tip - half - 1.0).floor() as i32,
        join.ceil() as i32,
        (middle + half + 1.0).ceil() as i32,
    );
    let rays = Rays {
        from: (join + 1.0, middle),
        heading: PI,
        reach: join + 1.0,
    };
    paint_fin(s, &inside, bounds, rays, tip + 0.5, ramps, 255);
}

/// The edge of the gill cover: a curve down the side of the head at `u`, from `across.0` to
/// `across.1`, bowed `bow` pixels towards the tail at its middle.
fn gill(s: &mut Canvas, body: &Body, u: f32, across: (f32, f32), bow: f32, color: Rgba) {
    let (x, top) = body.point(u, across.0);
    let (_, bottom) = body.point(u, across.1);
    for y in top..=bottom {
        let t = (y - top) as f32 / (bottom - top).max(1) as f32 * 2.0 - 1.0;
        let x = x - (bow * (1.0 - t * t)).round() as i32;
        if body.holds(x, y) && !body.rim(x, y) {
            put(s, x, y, color);
        }
    }
}

/// Paints rows of letters with their top-left at `at`, each letter a colour from `inks`; a
/// letter not in `inks` is left clear.
fn stamp(s: &mut Canvas, at: (i32, i32), rows: &[&str], inks: &[(u8, Rgba)]) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            if let Some(&(_, color)) = inks.iter().find(|(name, _)| *name == letter) {
                put(s, at.0 + dx as i32, at.1 + dy as i32, color);
            }
        }
    }
}

/// An eye stamped with its middle at the body's `(u, v)`.
fn eye(s: &mut Canvas, body: &Body, (u, v): (f32, f32), rows: &[&str], inks: &[(u8, Rgba)]) {
    let (x, y) = body.point(u, v);
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as i32;
    stamp(
        s,
        (x - (width - 1) / 2, y - (rows.len() as i32 - 1) / 2),
        rows,
        inks,
    );
}

/// A middling eye: the iris over the top and back of a pupil that catches the light.
const EYE: [&str; 3] = ["Ii.", "igp", ".pp"];

/// A big eye, the iris all round.
const BIG_EYE: [&str; 4] = [".ii.", "igpi", "ippi", ".II."];

/// The colours of an eye: its iris, the iris in shade, the glint and the pupil.
fn eye_inks(iris: Rgba, shade: Rgba) -> [(u8, Rgba); 4] {
    [(b'i', iris), (b'I', shade), (b'g', GLINT), (b'p', PUPIL)]
}

/// The line of the mouth, running back `length` pixels from the body's `(u, v)`.
fn mouth(s: &mut Canvas, body: &Body, (u, v): (f32, f32), length: i32, color: Rgba) {
    let (x, y) = body.point(u, v);
    for dx in 0..length {
        put(s, x - dx, y, color);
    }
}

/// Pixels along a curve from `from`, pulled towards `via`, to `to`, one step at a time.
fn curve(from: (f32, f32), via: (f32, f32), to: (f32, f32)) -> Vec<(i32, i32)> {
    let span = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
    let steps = ((span(from, via) + span(via, to)) * 2.0).ceil().max(1.0) as usize;
    let mut points: Vec<(i32, i32)> = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let w = 1.0 - t;
        let x = w * w * from.0 + 2.0 * w * t * via.0 + t * t * to.0;
        let y = w * w * from.1 + 2.0 * w * t * via.1 + t * t * to.1;
        let point = (x.floor() as i32, y.floor() as i32);
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

/// A barbel or whisker along a curve: `root` colour where it leaves the lip, `tip` beyond.
fn whisker(
    s: &mut Canvas,
    from: (f32, f32),
    via: (f32, f32),
    to: (f32, f32),
    root: Rgba,
    tip: Rgba,
) {
    let points = curve(from, via, to);
    let count = points.len().max(1) as f32;
    for (index, &(x, y)) in points.iter().enumerate() {
        let color = if (index as f32) < count * 0.45 {
            root
        } else {
            tip
        };
        put(s, x, y, color);
    }
}

/// Soft blotches: noise on a coarse grid, smoothed between its points, from 0 to 1.
fn blotch(x: i32, y: i32, size: f32, salt: u32) -> f32 {
    let (fx, fy) = (x as f32 / size, y as f32 / size);
    let (gx, gy) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - gx as f32, fy - gy as f32);
    let (tx, ty) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let at = |x: i32, y: i32| (noise(x, y, salt) & 0xff) as f32 / 255.0;
    let top = at(gx, gy) + (at(gx + 1, gy) - at(gx, gy)) * tx;
    let bottom = at(gx, gy + 1) + (at(gx + 1, gy + 1) - at(gx, gy + 1)) * tx;
    top + (bottom - top) * ty
}

/// Which part of a scale a pixel falls on.
#[derive(Clone, Copy, PartialEq)]
enum Scale {
    /// The free edge, which faces the tail.
    Rim,
    /// Just inside the rim at the top, where the light catches.
    Lit,
    Plain,
}

/// Where a pixel falls among round scales `size` pixels across and down, in columns that follow
/// the body's flex, each column half a scale lower than the last. Each scale overlaps the one
/// behind it, as a fish's do, so all that shows of its edge is the arc facing the tail, and the
/// arcs meet in a net. Returns the part and which scale it is.
fn scale(body: &Body, x: i32, y: i32, (wide, tall): (f32, f32)) -> (Scale, (i32, i32)) {
    let u = body.along(x as f32 + 0.5);
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5 - body.centre(u));
    // How far a pixel is from the middle of a scale, as shares of its size across and down.
    let from = |(column, row): (i32, i32)| {
        let shift = if column.rem_euclid(2) == 0 { 0.0 } else { tall };
        let (cx, cy) = (column as f32 * wide, row as f32 * 2.0 * tall + shift);
        ((px - cx) / wide, (py - cy) / tall)
    };
    // The scale on top here: of those this pixel is within, the one nearest the head.
    let mut top: Option<(i32, i32)> = None;
    let column0 = (px / wide).floor() as i32;
    for column in column0 - 2..=column0 + 2 {
        let shift = if column.rem_euclid(2) == 0 { 0.0 } else { tall };
        let row0 = ((py - shift) / (2.0 * tall)).floor() as i32;
        for row in row0 - 1..=row0 + 2 {
            let (dx, dy) = from((column, row));
            if dx.hypot(dy) <= 1.0 && top.is_none_or(|(best, _)| column > best) {
                top = Some((column, row));
            }
        }
    }
    let Some(which) = top else {
        return (Scale::Plain, (0, 0));
    };
    let (dx, dy) = from(which);
    // How far in from its edge, in pixels, roughly.
    let inset = (1.0 - dx.hypot(dy)) * wide.min(tall);
    let part = if inset < 0.8 && dx < -0.3 {
        Scale::Rim
    } else if inset < 1.8 && dy < -0.35 && dx < -0.1 {
        Scale::Lit
    } else {
        Scale::Plain
    };
    (part, which)
}

// ---------------------------------------------------------------------------------------------
// The fish
// ---------------------------------------------------------------------------------------------

const PUPIL: Rgba = rgb(0x1c1418);
const GLINT: Rgba = rgb(0xfbfaf2);

const OLIVE: Ramp = Ramp::new(0x3a4426, 0x55602f, 0x6e7a3c, 0x8a964e, 0xaab670);
const SILVER: Ramp = Ramp::new(0x5e6a74, 0x96a2ac, 0xbcc8ce, 0xe0e8ea, 0xffffff);
const PEARL: Ramp = Ramp::new(0x7e848c, 0xbcc2c6, 0xe2e6e4, 0xf4f6f2, 0xffffff);
const MINNOW_FIN: Ramp = Ramp::new(0x56603e, 0x7e8862, 0xa0a882, 0xc0c6a6, 0xdfe3cc);
const MINNOW_STRIPE: Rgba = rgb(0x3e4836);

/// The minnow, pixel by pixel: an olive back, the dark stripe down its side, a silver belly.
const MINNOW: [&str; 7] = [
    "............",
    ".....fF.....",
    "f..OllooO...",
    "ffOsssssEO..",
    ".fSSSSSSssN.",
    "ffWwwwwwwW..",
    "f..WWWWWW...",
];

fn minnow(length: i32) -> Canvas {
    let mut s = Canvas::new((length as u32 + 1).max(12), 7);
    let inks = [
        (b'f', MINNOW_FIN.base),
        (b'F', MINNOW_FIN.edge),
        (b'O', OLIVE.edge),
        (b'l', OLIVE.light),
        (b'o', OLIVE.base),
        (b'S', MINNOW_STRIPE),
        (b's', SILVER.light),
        (b'E', PUPIL),
        (b'N', SILVER.edge),
        (b'w', PEARL.base),
        (b'W', SILVER.edge),
    ];
    stamp(&mut s, (0, 0), &MINNOW, &inks);
    s
}

const ROACH_BACK: Ramp = Ramp::new(0x2c3c4c, 0x3c5266, 0x51697e, 0x6b8498, 0x90a8b8);
const ROACH_FIN: Ramp = Ramp::new(0x8a2c24, 0xb8443a, 0xd8604c, 0xee8668, 0xffb89c);
const ROACH_DORSAL: Ramp = Ramp::new(0x5a3a40, 0x7e5258, 0x9c6a6e, 0xbc8a88, 0xdcb0a8);
const ROACH_EYE: Rgba = rgb(0xe03c30);

fn roach(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.42;
    let body = Body {
        root: (l * 0.24).round(),
        nose: l,
        mid: 3.2 + depth * 0.53,
        depth,
        peak: 0.56,
        wrist: 0.3,
        snout: 0.36,
        above: 0.53,
        head: 1.8,
        chin: 0.3,
        flex: 0.6,
    };
    let mut s = Canvas::new(length as u32 + 1, (depth + 6.4).ceil() as u32);
    let coat = Coat {
        back: ROACH_BACK,
        flank: SILVER,
        belly: PEARL,
        upper: -0.4,
        lower: 0.5,
    };
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.6,
        &[
            (1.0, -1.0),
            (0.6, 1.0),
            (0.0, 3.0),
            (-1.0, 2.8),
            (-3.0, 1.2),
            (-4.5, 0.3),
            (-5.0, -1.0),
        ],
        ROACH_DORSAL,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.6,
        &[
            (1.0, -1.0),
            (0.5, 1.0),
            (-1.2, 2.6),
            (-2.4, 2.0),
            (-2.4, -1.0),
        ],
        ROACH_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.25,
        &[
            (1.0, -1.0),
            (0.5, 1.2),
            (-0.8, 2.4),
            (-3.0, 1.2),
            (-3.2, -1.0),
        ],
        ROACH_FIN,
    );
    tail(
        &mut s,
        &body,
        depth * 1.05,
        Tail::Forked(0.4),
        (ROACH_DORSAL, ROACH_FIN),
    );
    paint_body(&mut s, &body, &coat, 2);
    // Its scales just show on the silver.
    mark(&mut s, &body, |x, y, u, v, under| {
        let showing = u < 0.74 && v > -0.4 && scale(&body, x, y, (2.6, 2.0)).0 == Scale::Rim;
        showing.then(|| mix(under, SILVER.shadow, 0.45))
    });
    gleam(&mut s, &body, &coat, (0.3, 0.72), -0.3);
    gill(&mut s, &body, 0.76, (-0.5, 0.6), 0.8, SILVER.shadow);
    eye(
        &mut s,
        &body,
        (0.81, -0.2),
        &EYE,
        &eye_inks(ROACH_EYE, ROACH_FIN.edge),
    );
    mouth(&mut s, &body, (0.99, 0.25), 1, SILVER.edge);
    s
}

const PERCH_BACK: Ramp = Ramp::new(0x2c3c20, 0x40562a, 0x566e34, 0x6e8840, 0x92ac5c);
const PERCH_FLANK: Ramp = Ramp::new(0x5a6224, 0x8e9a3a, 0xb2b84c, 0xd2d46a, 0xf4f0a8);
const CREAM: Ramp = Ramp::new(0x8e8a5c, 0xcfcca0, 0xeceac6, 0xf8f6e2, 0xffffff);
const PERCH_FIN: Ramp = Ramp::new(0x46502e, 0x6e7a4c, 0x909a6c, 0xb4bc90, 0xd8dcbc);
const EMBER: Ramp = Ramp::new(0x8e3018, 0xc24a22, 0xe86c2e, 0xf8964c, 0xffc888);
const BARS: Rgba = rgb(0x26341c);
const PERCH_EYE: Rgba = rgb(0xf0a830);

fn perch(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.4;
    let body = Body {
        root: (l * 0.2).round(),
        nose: l,
        mid: 4.4 + depth * 0.56,
        depth,
        peak: 0.55,
        wrist: 0.3,
        snout: 0.36,
        above: 0.56,
        head: 1.6,
        chin: 0.4,
        flex: 0.6,
    };
    let mut s = Canvas::new(length as u32 + 1, (depth + 8.0).ceil() as u32);
    let coat = Coat {
        back: PERCH_BACK,
        flank: PERCH_FLANK,
        belly: CREAM,
        upper: -0.45,
        lower: 0.5,
    };
    // The spiny first dorsal, a spine and a dip of membrane over and over, then the soft second.
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.78,
        &[
            (0.6, -1.0),
            (0.3, 1.5),
            (0.0, 3.6),
            (-0.8, 2.0),
            (-2.0, 3.8),
            (-2.8, 2.0),
            (-4.0, 3.4),
            (-4.8, 1.8),
            (-6.0, 2.8),
            (-6.8, 1.4),
            (-7.6, 1.8),
            (-8.0, -1.0),
        ],
        PERCH_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.3,
        &[
            (0.5, -1.0),
            (0.2, 2.4),
            (-1.0, 2.8),
            (-3.0, 2.2),
            (-4.5, 1.0),
            (-5.0, -1.0),
        ],
        PERCH_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.68,
        &[
            (1.0, -1.0),
            (0.5, 1.0),
            (-1.4, 3.0),
            (-2.6, 2.4),
            (-2.6, -1.0),
        ],
        EMBER,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.27,
        &[
            (1.0, -1.0),
            (0.5, 1.4),
            (-1.0, 2.8),
            (-3.4, 1.4),
            (-3.6, -1.0),
        ],
        EMBER,
    );
    tail(
        &mut s,
        &body,
        depth * 1.1,
        Tail::Forked(0.3),
        (PERCH_FIN, EMBER),
    );
    paint_body(&mut s, &body, &coat, 3);
    // Bold bars from the back down the flank, broad above and tapering below.
    let bars = [0.14, 0.34, 0.54, 0.72];
    mark(&mut s, &body, |x, _, u, v, under| {
        let barred = bars.iter().any(|&bar| {
            let reach = 1.1 * (1.0 - (v + 1.0) / 2.4).max(0.0);
            (x as f32 + 0.5 - body.x(bar)).abs() < reach + 0.2
        });
        (barred && v < 0.5 && u < 0.8).then(|| mix(under, BARS, 0.75))
    });
    gleam(&mut s, &body, &coat, (0.62, 0.8), -0.45);
    gill(&mut s, &body, 0.8, (-0.45, 0.6), 0.8, PERCH_FLANK.shadow);
    pectoral(
        &mut s,
        &body,
        (0.76, 0.3),
        &[(0.5, -0.5), (-3.2, 1.0), (-1.6, 1.8), (0.5, 1.0)],
        PERCH_FIN,
    );
    eye(
        &mut s,
        &body,
        (0.82, -0.25),
        &EYE,
        &eye_inks(PERCH_EYE, EMBER.shadow),
    );
    mouth(&mut s, &body, (0.99, 0.3), 2, PERCH_FLANK.edge);
    s
}

const TROUT_BACK: Ramp = Ramp::new(0x3a2e18, 0x55442a, 0x705a34, 0x8c7244, 0xac9060);
const TROUT_FLANK: Ramp = Ramp::new(0x6a4a1c, 0x9a6e2c, 0xbe8e3e, 0xdcb05a, 0xf6d890);
const TROUT_BELLY: Ramp = Ramp::new(0x8e7840, 0xdcc47a, 0xf2e2a6, 0xfaf0cc, 0xffffff);
const TROUT_FIN: Ramp = Ramp::new(0x564020, 0x80602e, 0xa48046, 0xc4a466, 0xe2c890);
const HALO: Rgba = rgb(0xf6ead0);
const TROUT_SPOT: Rgba = rgb(0x2e2014);
const TROUT_RED: Rgba = rgb(0xc4302a);
const TROUT_EYE: Rgba = rgb(0xe8c050);
const TROUT_ADIPOSE: Ramp = Ramp::new(0x7a3a1c, 0xa85a2a, 0xc87438, 0xe0904a, 0xf4b070);

fn trout(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.33;
    let body = Body {
        root: (l * 0.2).round(),
        nose: l,
        mid: 3.4 + depth * 0.5,
        depth,
        peak: 0.5,
        wrist: 0.42,
        snout: 0.4,
        above: 0.5,
        head: 1.7,
        chin: 0.4,
        flex: 0.7,
    };
    let mut s = Canvas::new(length as u32 + 1, (depth + 7.0).ceil() as u32);
    let coat = Coat {
        back: TROUT_BACK,
        flank: TROUT_FLANK,
        belly: TROUT_BELLY,
        upper: -0.4,
        lower: 0.55,
    };
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.62,
        &[
            (1.0, -1.0),
            (0.6, 1.5),
            (-0.4, 3.2),
            (-2.0, 3.0),
            (-4.0, 2.0),
            (-5.0, 0.8),
            (-5.5, -1.0),
        ],
        TROUT_FIN,
    );
    // The little adipose fin near the tail, tinged orange.
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.16,
        &[(0.5, -1.0), (0.2, 1.4), (-1.2, 1.8), (-2.0, -1.0)],
        TROUT_ADIPOSE,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.5,
        &[
            (1.0, -1.0),
            (0.5, 1.0),
            (-1.4, 2.8),
            (-2.8, 2.2),
            (-2.8, -1.0),
        ],
        TROUT_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.27,
        &[
            (1.0, -1.0),
            (0.5, 1.4),
            (-1.0, 3.0),
            (-3.8, 1.8),
            (-4.2, -1.0),
        ],
        TROUT_FIN,
    );
    tail(
        &mut s,
        &body,
        depth * 1.05,
        Tail::Square,
        (TROUT_FIN, TROUT_FIN),
    );
    paint_body(&mut s, &body, &coat, 4);
    gleam(&mut s, &body, &coat, (0.45, 0.75), -0.35);
    gill(&mut s, &body, 0.79, (-0.5, 0.6), 0.8, TROUT_FLANK.shadow);
    // Spots, each in a pale halo: dark ones over the back and upper flank, a few red ones along
    // the middle of the side.
    let mut spots = Vec::new();
    for (row, (v, red)) in [(-0.55, false), (-0.1, false), (0.3, true)]
        .into_iter()
        .enumerate()
    {
        let row = row as i32;
        let step = if red { 4 } else { 3 };
        let mut x = body.root as i32 + 1 + row;
        while (x as f32) < body.x(0.74) {
            let roll = noise(x, row, 41);
            if roll & 3 != 0 {
                let wobble = ((roll >> 2) & 3) as f32 * 0.1 - 0.15;
                let (_, y) = body.point(body.along(x as f32 + 0.5), v + wobble);
                spots.push(((x, y), red));
            }
            x += step;
        }
    }
    for &((x, y), _) in &spots {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if body.holds(x + dx, y + dy) && !body.rim(x + dx, y + dy) {
                let under = s.get(x + dx, y + dy);
                put(&mut s, x + dx, y + dy, mix(under, HALO, 0.45));
            }
        }
    }
    for &((x, y), red) in &spots {
        if body.holds(x, y) && !body.rim(x, y) {
            put(&mut s, x, y, if red { TROUT_RED } else { TROUT_SPOT });
        }
    }
    pectoral(
        &mut s,
        &body,
        (0.75, 0.35),
        &[(0.5, -0.5), (-3.4, 1.2), (-1.6, 2.0), (0.5, 1.0)],
        TROUT_FIN,
    );
    eye(
        &mut s,
        &body,
        (0.83, -0.2),
        &EYE,
        &eye_inks(TROUT_EYE, TROUT_FLANK.shadow),
    );
    mouth(&mut s, &body, (0.99, 0.3), 2, TROUT_FLANK.edge);
    s
}

const CARP_BACK: Ramp = Ramp::new(0x3a2e16, 0x54421e, 0x6e5626, 0x8a6e32, 0xae9048);
const BRONZE: Ramp = Ramp::new(0x64421a, 0x926024, 0xb67e32, 0xd6a04a, 0xf2cc7a);
const GOLD: Ramp = Ramp::new(0x80601e, 0xc49a3a, 0xe2ba58, 0xf2d688, 0xfff0c0);
const CARP_FIN: Ramp = Ramp::new(0x4a3418, 0x6e4e2a, 0x8c6638, 0xaa824e, 0xc8a26e);
const CARP_LOWER: Ramp = Ramp::new(0x7a3818, 0xa45224, 0xc26e34, 0xdc8e4c, 0xf2b272);
const CARP_EYE: Rgba = rgb(0xe8b840);
const LIPS: Ramp = Ramp::new(0x7a4a34, 0xa06848, 0xc08a66, 0xdcaa86, 0xf0caa8);

fn carp(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.4;
    let body = Body {
        root: (l * 0.2).round(),
        nose: l,
        mid: 5.0 + depth * 0.58,
        depth,
        peak: 0.52,
        wrist: 0.3,
        snout: 0.32,
        above: 0.58,
        head: 2.0,
        chin: 1.0,
        flex: 0.9,
    };
    let mut s = Canvas::new(length as u32 + 2, (depth + 10.0).ceil() as u32);
    let coat = Coat {
        back: CARP_BACK,
        flank: BRONZE,
        belly: GOLD,
        upper: -0.38,
        lower: 0.45,
    };
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.66,
        &[
            (0.5, -1.0),
            (0.3, 2.0),
            (-0.4, 4.6),
            (-1.6, 3.8),
            (-4.0, 3.0),
            (-8.0, 2.4),
            (-11.5, 1.6),
            (-13.5, 0.5),
            (-14.0, -1.0),
        ],
        CARP_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.56,
        &[
            (1.0, -1.0),
            (0.6, 1.5),
            (-2.0, 4.0),
            (-3.8, 3.2),
            (-3.8, -1.0),
        ],
        CARP_LOWER,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.27,
        &[
            (1.0, -1.0),
            (0.6, 2.0),
            (-1.2, 4.0),
            (-4.2, 2.4),
            (-4.4, -1.0),
        ],
        CARP_LOWER,
    );
    tail(
        &mut s,
        &body,
        depth * 1.1,
        Tail::Forked(0.38),
        (CARP_FIN, CARP_LOWER),
    );
    paint_body(&mut s, &body, &coat, 5);
    // Big scales, each outlined, over all but the head.
    mark(&mut s, &body, |x, y, u, v, _| {
        if u > 0.76 {
            return None;
        }
        let ramp = coat.ramp(v);
        match scale(&body, x, y, (3.6, 2.7)).0 {
            Scale::Rim => Some(mix(ramp.shadow, ramp.edge, 0.45)),
            Scale::Lit => Some(ramp.light),
            Scale::Plain => None,
        }
    });
    gleam(&mut s, &body, &coat, (0.62, 0.86), -0.5);
    gill(&mut s, &body, 0.79, (-0.55, 0.65), 1.2, BRONZE.edge);
    pectoral(
        &mut s,
        &body,
        (0.77, 0.45),
        &[(0.5, -0.5), (-3.6, 1.2), (-2.2, 2.4), (0.5, 1.2)],
        CARP_FIN,
    );
    // The lips, pushed forward under the snout, a short barbel at the top of them and a longer
    // one from the corner.
    let (nx, ny) = body.point(0.99, 0.15);
    stamp(
        &mut s,
        (nx, ny),
        &["l", "s"],
        &[(b'l', LIPS.light), (b's', LIPS.shadow)],
    );
    whisker(
        &mut s,
        (nx as f32 + 1.5, ny as f32 + 0.5),
        (nx as f32 + 2.0, ny as f32 + 1.5),
        (nx as f32 + 1.5, ny as f32 + 2.5),
        LIPS.base,
        LIPS.light,
    );
    whisker(
        &mut s,
        (nx as f32 + 0.5, ny as f32 + 1.5),
        (nx as f32 + 0.5, ny as f32 + 3.0),
        (nx as f32 - 1.0, ny as f32 + 4.0),
        LIPS.base,
        LIPS.light,
    );
    eye(
        &mut s,
        &body,
        (0.88, -0.3),
        &BIG_EYE,
        &eye_inks(CARP_EYE, BRONZE.shadow),
    );
    s
}

const PIKE_BACK: Ramp = Ramp::new(0x2a3e22, 0x304626, 0x405c30, 0x547440, 0x749456);
const PIKE_FLANK: Ramp = Ramp::new(0x34441e, 0x52682e, 0x6c863c, 0x88a250, 0xaec474);
const PIKE_FIN: Ramp = Ramp::new(0x4a3a1a, 0x705a2a, 0x92783a, 0xb09650, 0xd0b878);
const PIKE_SPOT: Rgba = rgb(0xdcdc98);
const PIKE_EYE: Rgba = rgb(0xf0d848);

fn pike(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.18;
    let body = Body {
        root: (l * 0.14).round(),
        nose: l,
        mid: 3.6 + depth * 0.46,
        depth,
        peak: 0.45,
        wrist: 0.55,
        snout: 0.42,
        above: 0.46,
        head: 1.0,
        chin: 1.2,
        flex: 0.7,
    };
    let mut s = Canvas::new(length as u32 + 1, (depth + 7.5).ceil() as u32);
    let coat = Coat {
        back: PIKE_BACK,
        flank: PIKE_FLANK,
        belly: CREAM,
        upper: -0.35,
        lower: 0.5,
    };
    // The dorsal and anal fins sit far back, one over the other, to drive it like an arrow.
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.24,
        &[
            (1.0, -1.0),
            (0.6, 1.5),
            (-0.6, 3.4),
            (-2.6, 3.4),
            (-5.4, 2.2),
            (-7.0, 0.6),
            (-7.5, -1.0),
        ],
        PIKE_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.22,
        &[
            (1.0, -1.0),
            (0.6, 1.5),
            (-0.8, 3.2),
            (-2.8, 3.0),
            (-5.6, 1.6),
            (-6.4, -1.0),
        ],
        PIKE_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.5,
        &[
            (1.0, -1.0),
            (0.6, 1.0),
            (-1.8, 2.6),
            (-2.8, 2.4),
            (-2.0, 0.5),
            (-2.4, -1.0),
        ],
        PIKE_FIN,
    );
    tail(
        &mut s,
        &body,
        depth * 1.35,
        Tail::Forked(0.3),
        (PIKE_FIN, PIKE_FIN),
    );
    paint_body(&mut s, &body, &coat, 6);
    // Pale bean-shaped spots in staggered rows along the flank.
    for (row, v) in [-0.45, 0.0, 0.42].into_iter().enumerate() {
        let row = row as i32;
        let mut x = body.root as i32 + 2 + row * 2;
        while (x as f32) < body.x(0.7) {
            let roll = noise(x, row, 61);
            if roll & 7 != 0 {
                let long = 2 + (roll >> 3) as i32 % 2;
                for dx in 0..long {
                    let (_, y) = body.point(body.along((x + dx) as f32 + 0.5), v);
                    if body.holds(x + dx, y) && !body.rim(x + dx, y) {
                        let under = s.get(x + dx, y);
                        put(&mut s, x + dx, y, mix(under, PIKE_SPOT, 0.75));
                    }
                }
            }
            x += 5;
        }
    }
    gleam(&mut s, &body, &coat, (0.5, 0.78), -0.5);
    gill(&mut s, &body, 0.7, (-0.5, 0.6), 1.0, PIKE_FLANK.shadow);
    // The long mouth of the duck-bill snout.
    let (mx, my) = body.point(0.99, 0.35);
    for dx in 0..(depth * 0.9) as i32 {
        put(&mut s, mx - dx, my, PIKE_BACK.edge);
    }
    pectoral(
        &mut s,
        &body,
        (0.68, 0.45),
        &[(0.5, -0.5), (-3.4, 1.2), (-1.6, 2.0), (0.5, 1.0)],
        PIKE_FIN,
    );
    eye(
        &mut s,
        &body,
        (0.78, -0.4),
        &EYE,
        &eye_inks(PIKE_EYE, PIKE_FLANK.shadow),
    );
    s
}

const TENCH_BACK: Ramp = Ramp::new(0x7e3c10, 0xb05a16, 0xd47c20, 0xee9c30, 0xffc460);
const TENCH_FLANK: Ramp = Ramp::new(0x8e4a12, 0xca7a20, 0xeca032, 0xfcc452, 0xfff0a8);
const TENCH_BELLY: Ramp = Ramp::new(0x9a6418, 0xe0a83e, 0xf8cc64, 0xffe49a, 0xfff8e0);
const TENCH_FIN: Ramp = Ramp::new(0x8a3c0e, 0xbe601c, 0xe2842a, 0xf6a84a, 0xffd088);
const TENCH_EYE: Rgba = rgb(0xd02a1e);

fn tench(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.36;
    let body = Body {
        root: (l * 0.19).round(),
        nose: l,
        mid: 4.0 + depth * 0.52,
        depth,
        peak: 0.45,
        wrist: 0.45,
        snout: 0.4,
        above: 0.52,
        head: 2.0,
        chin: 0.6,
        flex: 0.7,
    };
    let mut s = Canvas::new(length as u32 + 2, (depth + 9.0).ceil() as u32);
    let coat = Coat {
        back: TENCH_BACK,
        flank: TENCH_FLANK,
        belly: TENCH_BELLY,
        upper: -0.4,
        lower: 0.5,
    };
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.6,
        &[
            (0.5, -1.0),
            (0.4, 1.4),
            (-0.2, 3.0),
            (-1.4, 3.6),
            (-3.0, 3.4),
            (-4.2, 2.2),
            (-4.8, -1.0),
        ],
        TENCH_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.3,
        &[
            (1.0, -1.0),
            (0.7, 1.5),
            (-0.4, 3.0),
            (-2.2, 3.4),
            (-4.0, 2.4),
            (-4.6, -1.0),
        ],
        TENCH_FIN,
    );
    // The big rounded pelvic fins a tench is known by.
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.58,
        &[
            (1.0, -1.0),
            (0.7, 1.6),
            (0.0, 3.2),
            (-1.4, 4.2),
            (-3.0, 4.2),
            (-4.2, 3.2),
            (-4.4, 1.2),
            (-4.0, -1.0),
        ],
        TENCH_FIN,
    );
    tail(
        &mut s,
        &body,
        depth * 1.0,
        Tail::Round,
        (TENCH_FIN, TENCH_FIN),
    );
    paint_body(&mut s, &body, &coat, 7);
    // Scales so small they show only as a fine shimmer.
    mark(&mut s, &body, |x, y, u, _, under| {
        if u > 0.78 || (x + y) % 2 != 0 || !chance(x, y, 71, 110) {
            return None;
        }
        Some(mix(under, rgb(0xfff2b0), 0.28))
    });
    // A soft sheen over the shoulder.
    gleam(&mut s, &body, &coat, (0.3, 0.8), -0.42);
    gleam(&mut s, &body, &coat, (0.45, 0.7), -0.2);
    gill(&mut s, &body, 0.8, (-0.5, 0.6), 1.0, TENCH_FLANK.shadow);
    pectoral(
        &mut s,
        &body,
        (0.76, 0.35),
        &[
            (0.5, -0.5),
            (-3.0, 0.6),
            (-3.6, 2.0),
            (-1.6, 2.6),
            (0.5, 1.2),
        ],
        TENCH_FIN,
    );
    let (nx, ny) = body.point(0.99, 0.2);
    whisker(
        &mut s,
        (nx as f32 + 0.5, ny as f32 + 1.5),
        (nx as f32 + 0.5, ny as f32 + 2.0),
        (nx as f32 - 0.5, ny as f32 + 2.5),
        TENCH_FIN.shadow,
        TENCH_FIN.base,
    );
    eye(
        &mut s,
        &body,
        (0.85, -0.25),
        &["gi", "ip"],
        &eye_inks(TENCH_EYE, TENCH_EYE),
    );
    s
}

const ANCIENT_BACK: Ramp = Ramp::new(0x2e3a22, 0x323e22, 0x404e2a, 0x526034, 0x6a7842);
const ANCIENT_FLANK: Ramp = Ramp::new(0x34321a, 0x4c4a22, 0x64622e, 0x7e7a3a, 0xa09a52);
const ANCIENT_BELLY: Ramp = Ramp::new(0x5c4a1c, 0x8a7232, 0xae944c, 0xcab068, 0xe8d090);
const ANCIENT_FIN: Ramp = Ramp::new(0x302e1c, 0x3a3820, 0x4e4a2a, 0x666038, 0x827a4e);
const LICHEN: Rgba = rgb(0xa8ac84);
const GILT: Rgba = rgb(0xd6b45a);
const AMBER: Rgba = rgb(0xe0a438);
const AMBER_DEEP: Rgba = rgb(0xa86a20);
const WHISKER: Ramp = Ramp::new(0x5a4a26, 0x7a6a3e, 0x9a8650, 0xb4a066, 0xd0bc84);

fn old_one(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.36;
    let body = Body {
        root: (l * 0.19).round(),
        nose: l,
        mid: 6.4 + depth * 0.58,
        depth,
        peak: 0.5,
        wrist: 0.3,
        snout: 0.34,
        above: 0.58,
        head: 2.0,
        chin: 1.8,
        flex: 1.2,
    };
    let mut s = Canvas::new(length as u32 + 2, (depth + 11.0).ceil() as u32);
    let coat = Coat {
        back: ANCIENT_BACK,
        flank: ANCIENT_FLANK,
        belly: ANCIENT_BELLY,
        upper: -0.3,
        lower: 0.42,
    };
    // A long dorsal, high at the front, with a notch bitten out of it long ago.
    fin_on(
        &mut s,
        &body,
        Edge::Back,
        0.68,
        &[
            (0.5, -1.0),
            (0.3, 2.5),
            (-0.6, 5.8),
            (-2.4, 5.0),
            (-5.0, 4.2),
            (-5.6, 4.0),
            (-6.2, 1.0),
            (-7.6, 0.8),
            (-8.4, 3.6),
            (-12.0, 3.0),
            (-17.0, 2.0),
            (-20.0, 0.8),
            (-20.5, -1.0),
        ],
        ANCIENT_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.56,
        &[
            (1.0, -1.0),
            (0.6, 2.0),
            (-2.6, 5.0),
            (-5.0, 4.0),
            (-5.0, -1.0),
        ],
        ANCIENT_FIN,
    );
    fin_on(
        &mut s,
        &body,
        Edge::Belly,
        0.27,
        &[
            (1.0, -1.0),
            (0.6, 2.4),
            (-1.6, 5.0),
            (-6.0, 3.0),
            (-6.4, -1.0),
        ],
        ANCIENT_FIN,
    );
    tail(
        &mut s,
        &body,
        depth * 1.05,
        Tail::Forked(0.4),
        (ANCIENT_FIN, ANCIENT_FIN),
    );
    paint_body(&mut s, &body, &coat, 8);
    // Mottled with age, darker in patches.
    mark(&mut s, &body, |x, y, _, _, under| {
        let patch = blotch(x, y, 6.0, 81);
        if patch > 0.66 {
            Some(mix(under, rgb(0x1e2412), 0.22))
        } else if patch < 0.26 {
            Some(mix(under, rgb(0xb0a868), 0.12))
        } else {
            None
        }
    });
    // Great weathered scales: some rims worn away, a few grown over pale, the high ones gilded
    // where the light catches them.
    mark(&mut s, &body, |x, y, u, v, under| {
        if u > 0.76 {
            return None;
        }
        let ramp = coat.ramp(v);
        let (part, (column, row)) = scale(&body, x, y, (3.8, 2.9));
        let worn = noise(column, row, 82).is_multiple_of(7);
        let grown = noise(column, row, 83).is_multiple_of(13);
        match part {
            Scale::Rim if worn => None,
            Scale::Rim if v < 0.1 && chance(x, y, 86, 90) => Some(mix(ramp.edge, GILT, 0.55)),
            Scale::Rim => Some(mix(ramp.shadow, ramp.edge, 0.7)),
            Scale::Lit if v < 0.0 && chance(x, y, 85, 18) => Some(GLINT),
            Scale::Lit if v < 0.3 => Some(mix(under, GILT, 0.75)),
            Scale::Lit => Some(ramp.light),
            Scale::Plain if grown => Some(mix(under, LICHEN, 0.4)),
            Scale::Plain => None,
        }
    });
    gleam(&mut s, &body, &coat, (0.68, 0.88), -0.55);
    gill(&mut s, &body, 0.79, (-0.55, 0.62), 2.0, ANCIENT_FLANK.edge);
    pectoral(
        &mut s,
        &body,
        (0.77, 0.45),
        &[(0.5, -0.5), (-4.6, 1.4), (-3.0, 2.6), (0.5, 1.2)],
        ANCIENT_FIN,
    );
    // Lips, and long whiskery barbels trailing down and back.
    let (nx, ny) = body.point(0.99, 0.15);
    stamp(
        &mut s,
        (nx, ny),
        &["l", "s"],
        &[(b'l', LIPS.base), (b's', LIPS.shadow)],
    );
    let (fx, fy) = (nx as f32 + 1.5, ny as f32 + 1.5);
    whisker(
        &mut s,
        (fx, fy),
        (fx + 0.5, fy + 3.5),
        (fx - 3.5, fy + 5.0),
        WHISKER.shadow,
        WHISKER.base,
    );
    whisker(
        &mut s,
        (fx, fy),
        (fx + 2.5, fy + 6.0),
        (fx - 2.0, fy + 9.5),
        WHISKER.base,
        WHISKER.light,
    );
    // A wise old eye, half hooded by a heavy lid, with a crease beneath.
    eye(
        &mut s,
        &body,
        (0.88, -0.3),
        &["LLL.", "Lgpi", "Ippi", ".II."],
        &[
            (b'L', ANCIENT_FLANK.edge),
            (b'g', GLINT),
            (b'I', AMBER_DEEP),
            (b'i', AMBER),
            (b'p', PUPIL),
        ],
    );
    let (ex, ey) = body.point(0.88, -0.3);
    for (dx, dy) in [(-2, 2), (-1, 3), (0, 3)] {
        let under = s.get(ex + dx, ey + dy);
        put(
            &mut s,
            ex + dx,
            ey + dy,
            mix(under, ANCIENT_FLANK.edge, 0.6),
        );
    }
    // Light catches it as nothing else in the pool: a few glints about its old gold.
    for (u, v, size) in [(0.36, -0.55, 2), (0.6, 0.05, 1), (0.14, 0.1, 1)] {
        sparkle(&mut s, body.point(u, v), size);
    }
    s
}

/// A glint of light: a bright point with rays fading out `size` pixels.
fn sparkle(s: &mut Canvas, at: (i32, i32), size: i32) {
    put(s, at.0, at.1, rgba(0xffffff, 240));
    for step in 1..=size {
        let alpha = (200 / step) as u8;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(s, at.0 + dx * step, at.1 + dy * step, rgba(0xfff6d8, alpha));
        }
    }
}

const PLAIN: Ramp = Ramp::new(0x3e4a52, 0x5a6a72, 0x7a8a92, 0x9eacb2, 0xc8d2d6);

/// A fish Hill doesn't know: grey and plain, but still a fish.
fn plain(length: i32) -> Canvas {
    let l = length as f32;
    let depth = l * 0.3;
    let body = Body {
        root: (l * 0.2).round(),
        nose: l,
        mid: 2.0 + depth * 0.5,
        depth,
        peak: 0.55,
        wrist: 0.35,
        snout: 0.4,
        above: 0.5,
        head: 1.8,
        chin: 0.0,
        flex: 0.0,
    };
    let mut s = Canvas::new(length as u32 + 1, (depth + 4.0).ceil() as u32);
    let coat = Coat {
        back: PLAIN,
        flank: PLAIN,
        belly: PLAIN,
        upper: -1.0,
        lower: 1.0,
    };
    tail(&mut s, &body, depth, Tail::Forked(0.3), (PLAIN, PLAIN));
    paint_body(&mut s, &body, &coat, 9);
    eye(&mut s, &body, (0.88, -0.2), &["p"], &[(b'p', PUPIL)]);
    s
}

// ---------------------------------------------------------------------------------------------
// Icons
// ---------------------------------------------------------------------------------------------

const MINNOW_ICON: [&str; 9] = [
    ".........",
    ".........",
    "....fF...",
    "f.OooooO.",
    "ffslllspO",
    ".fSSSSSsN",
    "ffWwwwwW.",
    "f..WWWW..",
    ".........",
];

fn minnow_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'f', MINNOW_FIN.base),
        (b'F', MINNOW_FIN.edge),
        (b'O', OLIVE.edge),
        (b'o', OLIVE.light),
        (b's', SILVER.light),
        (b'l', SILVER.shine),
        (b'S', MINNOW_STRIPE),
        (b'p', PUPIL),
        (b'N', SILVER.edge),
        (b'w', PEARL.base),
        (b'W', SILVER.edge),
    ]
}

const ROACH_ICON: [&str; 9] = [
    "....d....",
    "...ddD...",
    "r..BBBBB.",
    "rrBbbbbbB",
    ".rslllRpS",
    "rrsssssRS",
    "r.WwwwwW.",
    "...fWWf..",
    "....f....",
];

fn roach_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'd', ROACH_DORSAL.base),
        (b'D', ROACH_DORSAL.edge),
        (b'r', ROACH_FIN.base),
        (b'f', ROACH_FIN.base),
        (b'B', ROACH_BACK.edge),
        (b'b', ROACH_BACK.light),
        (b's', SILVER.base),
        (b'l', SILVER.light),
        (b'S', SILVER.edge),
        (b'R', ROACH_EYE),
        (b'p', PUPIL),
        (b'w', PEARL.base),
        (b'W', PEARL.edge),
    ]
}

const PERCH_ICON: [&str; 9] = [
    "...^.^.^.",
    "...^m^m^.",
    "f.BBBBBB.",
    "ffbgbgggB",
    ".fbgbgipB",
    "eebcbcccM",
    "e..CCCC..",
    "...ee.e..",
    ".........",
];

fn perch_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'^', PERCH_FIN.edge),
        (b'm', PERCH_FIN.light),
        (b'f', PERCH_FIN.base),
        (b'B', PERCH_BACK.edge),
        (b'b', BARS),
        (b'g', PERCH_FLANK.light),
        (b'i', PERCH_EYE),
        (b'p', PUPIL),
        (b'c', CREAM.base),
        (b'C', CREAM.edge),
        (b'M', PERCH_FLANK.edge),
        (b'e', EMBER.base),
    ]
}

const TROUT_ICON: [&str; 9] = [
    ".........",
    "....FF...",
    "TT.BBBBB.",
    "TTBbbkbbB",
    "TTgkgggip",
    "TTgrgrggm",
    "TT.wwwww.",
    "...F..F..",
    ".........",
];

fn trout_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'T', TROUT_FIN.base),
        (b'F', TROUT_FIN.shadow),
        (b'B', TROUT_BACK.edge),
        (b'b', TROUT_BACK.light),
        (b'k', TROUT_SPOT),
        (b'g', TROUT_FLANK.base),
        (b'r', TROUT_RED),
        (b'i', TROUT_EYE),
        (b'p', PUPIL),
        (b'm', TROUT_FLANK.edge),
        (b'w', TROUT_BELLY.base),
    ]
}

const CARP_ICON: [&str; 9] = [
    "....DD...",
    "...DddB..",
    "f.BbbbbB.",
    "ffRoRoRii",
    ".foRoRoip",
    "ffRoRooo#",
    "l.GgggggL",
    "...ll..w.",
    ".........",
];

fn carp_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'D', CARP_FIN.edge),
        (b'd', CARP_FIN.light),
        (b'f', CARP_FIN.base),
        (b'B', CARP_BACK.edge),
        (b'b', CARP_BACK.light),
        (b'o', BRONZE.light),
        (b'R', BRONZE.shadow),
        (b'i', CARP_EYE),
        (b'p', PUPIL),
        (b'#', BRONZE.edge),
        (b'G', GOLD.edge),
        (b'g', GOLD.base),
        (b'L', LIPS.base),
        (b'l', CARP_LOWER.base),
        (b'w', LIPS.light),
    ]
}

const PIKE_ICON: [&str; 9] = [
    ".........",
    ".........",
    "..FF.....",
    "f.BBBBB..",
    "ffgsgsiBB",
    ".fsgsgpjj",
    "ffFWWWWW.",
    "f........",
    ".........",
];

fn pike_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'f', PIKE_FIN.base),
        (b'F', PIKE_FIN.shadow),
        (b'B', PIKE_BACK.edge),
        (b'g', PIKE_FLANK.base),
        (b's', PIKE_SPOT),
        (b'i', PIKE_EYE),
        (b'p', PUPIL),
        (b'j', CREAM.base),
        (b'W', CREAM.edge),
    ]
}

const TENCH_ICON: [&str; 9] = [
    ".........",
    "....ff...",
    "f..BBBBB.",
    "ffBbbbbbB",
    "fflll*grB",
    "ffoooooo#",
    "f.GyyyyyG",
    "..ff.ff..",
    ".........",
];

fn tench_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'f', TENCH_FIN.base),
        (b'B', TENCH_BACK.edge),
        (b'b', TENCH_BACK.light),
        (b'o', TENCH_FLANK.base),
        (b'l', TENCH_FLANK.light),
        (b'*', TENCH_FLANK.shine),
        (b'g', GLINT),
        (b'r', TENCH_EYE),
        (b'#', TENCH_FLANK.edge),
        (b'G', TENCH_BELLY.edge),
        (b'y', TENCH_BELLY.base),
    ]
}

const OLD_ONE_ICON: [&str; 9] = [
    "..DD.D...",
    ".BBBBBBB.",
    "fbbRbbRbB",
    "fRoooRLLL",
    "ffoRooiip",
    "fRoooRooM",
    "f.GyyyyyG",
    "...D...ww",
    ".......w.",
];

fn old_one_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'D', ANCIENT_FIN.light),
        (b'f', ANCIENT_FIN.base),
        (b'B', ANCIENT_BACK.edge),
        (b'b', ANCIENT_BACK.light),
        (b'R', GILT),
        (b'o', ANCIENT_FLANK.base),
        (b'L', ANCIENT_FLANK.edge),
        (b'i', AMBER),
        (b'p', PUPIL),
        (b'M', LIPS.shadow),
        (b'G', ANCIENT_BELLY.edge),
        (b'y', ANCIENT_BELLY.base),
        (b'w', WHISKER.light),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::fish::CATALOGUE;

    #[test]
    fn every_fish_has_a_catch_and_an_icon() {
        for fish in &CATALOGUE {
            let icon = icon(fish.id);
            assert_eq!((icon.width(), icon.height()), (ICON, ICON));
            assert!(
                icon.pixels().iter().filter(|pixel| pixel.a > 0).count() >= 12,
                "{}",
                fish.id
            );
            let catch = catch(fish.id);
            assert!(
                catch.width() >= drawn_length(fish.length.1),
                "{} is drawn too short",
                fish.id
            );
            assert!(
                catch.width() <= 64 && catch.height() <= 32,
                "{} is too big to hold up",
                fish.id
            );
        }
    }
}
