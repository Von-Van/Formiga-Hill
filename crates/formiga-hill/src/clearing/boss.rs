//! The Cursor Sovereign as it is drawn: an enormous mouse pointer, crowned, with a glare, looming
//! over the clearing. It is the one thing in Hill outlined in true black: it is the desktop's own
//! pointer, come to the Hill, and it should look like it doesn't belong. While it hauls someone
//! across the stage it is the closed hand a desktop shows for dragging, crown, glare and all.

use crate::cast::Id;
use crate::paint::{Ramp, line, mix, put, rect, rgb, rgba};
use crate::playground::Patch;
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;
use std::sync::OnceLock;

/// Where the pointer's tip rests while it hovers.
pub const HOME: (f32, f32) = (262.0, 46.0);
/// How many scene pixels one unit of the pointer's outline is.
const SCALE: f32 = 6.0;

/// The pointer's outline, in units, tip first, as the classic arrow is drawn.
const ARROW: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 16.0),
    (4.0, 12.6),
    (6.8, 18.6),
    (9.4, 17.4),
    (6.6, 11.6),
    (11.6, 11.6),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// Hovering, bobbing, menacing.
    Hover,
    /// Pressing down where something stands.
    Click { at: (f32, f32) },
    /// Hauling one of them across the stage, and dropping it.
    Drag {
        who: Id,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// Dragging a selection out around them all.
    Select { area: Patch },
    /// Knocked back by a blow.
    Reel,
    /// Breaking into pieces.
    Shatter,
    /// What is left of it, a very ordinary little pointer, scurrying off.
    Flee,
    /// Gone.
    Gone,
}

impl Move {
    /// How long the move takes before the stage carries on.
    pub fn secs(self) -> f32 {
        match self {
            Self::Hover | Self::Gone => 0.0,
            Self::Click { .. } => 1.2,
            Self::Drag { .. } => 2.6,
            Self::Select { .. } => 2.4,
            Self::Reel => 0.8,
            Self::Shatter => 1.8,
            Self::Flee => 2.4,
        }
    }
}

/// Where the tip is, `t` seconds into a move.
/// When, into a drag, the cursor takes hold of whoever it drags, and when it lets go.
pub const HAUL: (f32, f32) = (0.5, 1.9);

pub fn tip(step: Move, t: f32) -> (f32, f32) {
    let ease = |a: (f32, f32), b: (f32, f32), u: f32| {
        let u = u.clamp(0.0, 1.0);
        let u = u * u * (3.0 - 2.0 * u);
        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
    };
    match step {
        Move::Hover | Move::Shatter | Move::Gone => HOME,
        Move::Click { at } => {
            // Swoop over, press, come back.
            let over = (at.0, at.1 - 34.0);
            if t < 0.45 {
                ease(HOME, over, t / 0.45)
            } else if t < 0.7 {
                (over.0, over.1 + 6.0)
            } else {
                ease(over, HOME, (t - 0.7) / 0.5)
            }
        }
        Move::Drag { from, to, .. } => {
            let grab = (from.0, from.1 - 30.0);
            let drop = (to.0, to.1 - 30.0);
            let (hold, release) = HAUL;
            if t < hold {
                ease(HOME, grab, t / hold)
            } else if t < release {
                // Hauled across, with a wobble.
                let along = ease(grab, drop, (t - hold) / (release - hold));
                (along.0, along.1 - ((t - hold) * 7.0).sin().abs() * 10.0)
            } else {
                ease(drop, HOME, (t - release) / 0.7)
            }
        }
        Move::Select { area } => {
            let (left, top, _, _) = area;
            ease(HOME, (left - 4.0, top - 4.0), t / 0.4)
        }
        Move::Reel => {
            // Knocked back and up, settling home again.
            let left = (1.0 - t / 0.8).max(0.0);
            (HOME.0 + 10.0 * left, HOME.1 - 4.0 * left)
        }
        Move::Flee => ease((HOME.0 + 20.0, 150.0), (420.0, 190.0), t / 2.4),
    }
}

/// Draws the Sovereign as it is `t` seconds into its move. `now` keeps it bobbing.
pub fn draw(scene: &mut Canvas, step: Move, t: f32, now: f32, reduce_motion: bool) {
    match step {
        Move::Gone => {}
        Move::Shatter => shatter(scene, t, reduce_motion),
        Move::Flee => {
            let at = if reduce_motion {
                (330.0, 180.0)
            } else {
                tip(step, t)
            };
            little_pointer(scene, at, now, reduce_motion);
        }
        Move::Select { area } => {
            selection(scene, area, t, now, reduce_motion);
            let at = if reduce_motion {
                // A cut: it is simply there at the corner, the selection already made.
                (area.0 - 4.0, area.1 - 4.0)
            } else {
                tip(step, t)
            };
            pointer(scene, at, false, now, reduce_motion);
        }
        Move::Drag { to, .. } => {
            if reduce_motion {
                hand(scene, (to.0, to.1 - 30.0), now, true);
            } else {
                let at = tip(step, t);
                let before = tip(step, (t - 0.06).max(0.0));
                haste(scene, at, before);
                hand(scene, at, now, false);
            }
        }
        _ => {
            let mut at = tip(step, t);
            if !reduce_motion && step == Move::Hover {
                at.1 += (now * 1.6).sin() * 3.0;
            }
            if reduce_motion {
                // A cut, not a swoop: it is simply there; knocked back, it simply flushes.
                at = match step {
                    Move::Click { at } => (at.0, at.1 - 30.0),
                    _ => HOME,
                };
            }
            if let Move::Click { at: target } = step
                && (0.45..1.0).contains(&t)
            {
                shockwave(
                    scene,
                    (target.0, target.1 - 22.0),
                    (t - 0.45) / 0.55,
                    reduce_motion,
                );
            }
            pointer(scene, at, step == Move::Reel, now, reduce_motion);
        }
    }
}

/// True black, as only the desktop's own things are drawn in Hill.
const INK: Rgba = rgb(0x0c0a10);
const WHITE: Ramp = Ramp::new(0x6a6280, 0xb4acc8, 0xe6e0f0, 0xf6f2fc, 0xffffff);
const CROWN: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xd4a842, 0xecc864, 0xfaeaa8);
/// The glow behind it, thrown back up onto its shaded side.
const BOUNCE: Rgba = rgb(0xe8a6e6);
/// The violet light it gives off.
const AURA: u32 = 0xa860e0;
/// How far round a figure its outline and the thin glow beyond it reach.
const REACH: u8 = 3;
/// How many soft rings of light it gives off, each the figure grown a little more.
const RINGS: usize = 6;

/// What one pixel of a figure is.
#[derive(Clone, Copy, PartialEq)]
enum Mark {
    /// Outside, this many pixels from the figure: the outline, then a thin glow.
    Away(u8),
    /// The black outline's inner half, or a crease where two parts meet.
    Ink,
    /// A part of the figure, and how its edge catches the light.
    Part(u8, Edge),
}

#[derive(Clone, Copy, PartialEq)]
enum Edge {
    Plain,
    /// Its upper left, catching the light.
    Lit,
    /// Its lower right, catching the glow from behind.
    Bounced,
}

/// One of the Sovereign's figures, worked out relative to where it is anchored: what each pixel
/// is, so it can be shaded part by part, creased where parts meet, and outlined.
struct Figure {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    marks: Vec<Mark>,
}

/// How a figure is finished.
#[derive(Clone, Copy)]
struct Style {
    /// The outline runs a pixel inside the figure as well as outside it, as the pointer's does.
    thick: bool,
    /// Its edges catch the light at the upper left and the glow behind at the lower right.
    bevel: bool,
}

impl Figure {
    /// Works out every pixel from `(left, top)` to `(right, bottom)`, and a margin round them:
    /// `part_at` says which part (0 for none) is there, `creased` whether two parts that meet are
    /// creased in black.
    fn new(
        (left, top): (i32, i32),
        (right, bottom): (i32, i32),
        style: Style,
        part_at: impl Fn(i32, i32) -> u8,
        creased: impl Fn(u8, u8) -> bool,
    ) -> Self {
        let margin = i32::from(REACH) + 1;
        let (left, top) = (left - margin, top - margin);
        let (width, height) = (right + margin - left + 1, bottom + margin - top + 1);
        let at = |x: i32, y: i32| (y * width + x) as usize;
        let mut parts = vec![0_u8; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                parts[at(x, y)] = part_at(left + x, top + y);
            }
        }
        let part = |x: i32, y: i32| {
            if (0..width).contains(&x) && (0..height).contains(&y) {
                parts[at(x, y)]
            } else {
                0
            }
        };
        // How far each pixel outside is from the figure, counting diagonal steps as one: a sweep
        // forward and a sweep back.
        let mut away: Vec<u8> = parts
            .iter()
            .map(|&part| if part == 0 { u8::MAX } else { 0 })
            .collect();
        let sweeps: [(bool, [(i32, i32); 4]); 2] = [
            (false, [(-1, 0), (-1, -1), (0, -1), (1, -1)]),
            (true, [(1, 0), (1, 1), (0, 1), (-1, 1)]),
        ];
        for (backwards, steps) in sweeps {
            for row in 0..height {
                for column in 0..width {
                    let (x, y) = if backwards {
                        (width - 1 - column, height - 1 - row)
                    } else {
                        (column, row)
                    };
                    for (dx, dy) in steps {
                        let (nx, ny) = (x + dx, y + dy);
                        if (0..width).contains(&nx) && (0..height).contains(&ny) {
                            let through = away[at(nx, ny)].saturating_add(1);
                            away[at(x, y)] = away[at(x, y)].min(through);
                        }
                    }
                }
            }
        }
        let mut marks = Vec::with_capacity(parts.len());
        for y in 0..height {
            for x in 0..width {
                let here = part(x, y);
                if here == 0 {
                    marks.push(Mark::Away(away[at(x, y)]));
                    continue;
                }
                let open = |dx: i32, dy: i32| part(x + dx, y + dy) == 0;
                let edge = style.thick && (open(-1, 0) || open(1, 0) || open(0, -1) || open(0, 1));
                let crease = [(1, 0), (0, 1)].iter().any(|&(dx, dy)| {
                    let other = part(x + dx, y + dy);
                    other != 0 && other != here && creased(here, other)
                });
                marks.push(if edge || crease {
                    Mark::Ink
                } else if style.bevel && (open(-2, 0) || open(0, -2)) {
                    Mark::Part(here, Edge::Lit)
                } else if style.bevel && (open(2, 0) || open(0, 2) || open(3, 3)) {
                    Mark::Part(here, Edge::Bounced)
                } else {
                    Mark::Part(here, Edge::Plain)
                });
            }
        }
        Self {
            left,
            top,
            width,
            height,
            marks,
        }
    }

    fn mark(&self, x: i32, y: i32) -> Mark {
        let (x, y) = (x - self.left, y - self.top);
        if (0..self.width).contains(&x) && (0..self.height).contains(&y) {
            self.marks[(y * self.width + x) as usize]
        } else {
            Mark::Away(u8::MAX)
        }
    }

    fn covers(&self, x: i32, y: i32) -> bool {
        !matches!(self.mark(x, y), Mark::Away(_))
    }

    /// Paints the figure anchored at `(x, y)`: each part toned by `tone` (given where it is
    /// relative to the anchor), lit and bounced along its edges, outlined in black and edged
    /// outside that with a thin violet glow as strong as `glow`. Flushed red if `hit`.
    fn paint(
        &self,
        scene: &mut Canvas,
        (x, y): (i32, i32),
        glow: f32,
        hit: bool,
        tone: impl Fn(i32, i32, u8) -> Rgba,
    ) {
        for row in 0..self.height {
            for column in 0..self.width {
                let (lx, ly) = (self.left + column, self.top + row);
                let color = match self.marks[(row * self.width + column) as usize] {
                    Mark::Away(1) => INK,
                    Mark::Away(2) if glow > 0.0 => rgba(0xf0c8ff, (130.0 * glow) as u8),
                    Mark::Away(3) if glow > 0.0 => rgba(AURA, (70.0 * glow) as u8),
                    Mark::Away(_) => continue,
                    Mark::Ink => INK,
                    Mark::Part(part, edge) => {
                        let color = tone(lx, ly, part);
                        let color = match edge {
                            Edge::Plain => color,
                            Edge::Lit => mix(color, WHITE.shine, 0.8),
                            Edge::Bounced => mix(color, BOUNCE, 0.55),
                        };
                        if hit {
                            mix(color, rgb(0xe04040), 0.45)
                        } else {
                            color
                        }
                    }
                };
                put(scene, x + lx, y + ly, color);
            }
        }
    }
}

/// The soft rings of light round a figure: for each pixel about it, which ring (counting from 1,
/// nearest) first takes it in, each ring the figure grown a little more about its middle.
struct Glow {
    left: i32,
    top: i32,
    width: i32,
    rings: Vec<u8>,
}

impl Glow {
    fn new(figure: &Figure, centre: (f32, f32)) -> Self {
        let grows: [f32; RINGS] = std::array::from_fn(|ring| 1.0 + (ring + 1) as f32 * 0.07);
        let widest = grows[RINGS - 1];
        let span = |from: i32, to: i32, middle: f32| {
            (
                (middle + (from as f32 - middle) * widest).floor() as i32,
                (middle + (to as f32 - middle) * widest).ceil() as i32,
            )
        };
        let (left, right) = span(figure.left, figure.left + figure.width, centre.0);
        let (top, bottom) = span(figure.top, figure.top + figure.height, centre.1);
        let mut rings = Vec::new();
        for y in top..bottom {
            for x in left..right {
                let (dx, dy) = (x as f32 + 0.5 - centre.0, y as f32 + 0.5 - centre.1);
                let ring = if figure.covers(x, y) {
                    None
                } else {
                    grows.iter().position(|grow| {
                        figure.covers(
                            (centre.0 + dx / grow).floor() as i32,
                            (centre.1 + dy / grow).floor() as i32,
                        )
                    })
                };
                rings.push(ring.map_or(0, |ring| ring as u8 + 1));
            }
        }
        Self {
            left,
            top,
            width: right - left,
            rings,
        }
    }

    /// Lays the rings round the figure anchored at `(x, y)`, brighter as it breathes in.
    fn paint(&self, scene: &mut Canvas, (x, y): (i32, i32), breath: f32) {
        // Where rings overlap their light builds up: how much each pixel lets through.
        let mut through = [1.0_f32; RINGS + 1];
        for ring in (0..RINGS).rev() {
            let alpha = f32::from((14.0 + 12.0 * breath) as u8 / (ring + 1) as u8 + 4) / 255.0;
            through[ring] = through[ring + 1] * (1.0 - alpha);
        }
        for (index, &ring) in self.rings.iter().enumerate() {
            if ring == 0 {
                continue;
            }
            let (column, row) = (index as i32 % self.width, index as i32 / self.width);
            let alpha = ((1.0 - through[usize::from(ring) - 1]) * 255.0) as u8;
            put(
                scene,
                x + self.left + column,
                y + self.top + row,
                rgba(AURA, alpha),
            );
        }
    }
}

/// Whether a point lies inside a closed outline.
fn inside(points: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut within = false;
    for (index, &(x0, y0)) in points.iter().enumerate() {
        let (x1, y1) = points[(index + 1) % points.len()];
        if (y0 > y) != (y1 > y) && x < x0 + (y - y0) / (y1 - y0) * (x1 - x0) {
            within = !within;
        }
    }
    within
}

/// The four steps of white a lit surface is shaded in, from `lit` 1 (full on) down.
fn white(lit: f32) -> Rgba {
    if lit > 0.78 {
        WHITE.shine
    } else if lit > 0.5 {
        WHITE.light
    } else if lit > 0.42 {
        WHITE.base
    } else if lit > 0.24 && lit < 0.28 {
        // A streak of shine across the shaded side, as glossy things have in anime.
        WHITE.light
    } else {
        WHITE.shadow
    }
}

/// How brightly it breathes: steady with reduced motion.
fn breath(now: f32, reduce_motion: bool) -> f32 {
    if reduce_motion {
        0.5
    } else {
        (now * 2.0).sin() * 0.5 + 0.5
    }
}

/// Where a figure anchored at `at` falls on whole pixels.
fn anchor(at: (f32, f32)) -> (i32, i32) {
    (at.0.round() as i32, at.1.round() as i32)
}

/// The arrow, worked out once, its tip at the origin.
fn arrow() -> &'static (Figure, Glow) {
    static ARROW_SHAPE: OnceLock<(Figure, Glow)> = OnceLock::new();
    ARROW_SHAPE.get_or_init(|| {
        let points: Vec<(f32, f32)> = ARROW.iter().map(|&(x, y)| (x * SCALE, y * SCALE)).collect();
        let figure = Figure::new(
            (0, 0),
            ((11.6 * SCALE) as i32 + 1, (18.6 * SCALE) as i32 + 1),
            Style {
                thick: true,
                bevel: true,
            },
            |x, y| u8::from(inside(&points, x as f32 + 0.5, y as f32 + 0.5)),
            |_, _| false,
        );
        let glow = Glow::new(&figure, (4.5 * SCALE, 11.0 * SCALE));
        (figure, glow)
    })
}

/// The great pointer, its tip at `at`: a violet aura, the white arrow outlined in black, a glare,
/// a crown. Flushed red when struck.
fn pointer(scene: &mut Canvas, at: (f32, f32), hit: bool, now: f32, reduce_motion: bool) {
    let (figure, glow) = arrow();
    let (x, y) = anchor(at);
    let breath = breath(now, reduce_motion);
    glow.paint(scene, (x, y), breath);
    let (wide, tall) = (11.6 * SCALE, 18.6 * SCALE);
    figure.paint(scene, (x, y), 0.5 + breath * 0.5, hit, |lx, ly, _| {
        white(1.0 - ly as f32 / tall * 0.7 - lx as f32 / wide * 0.4)
    });
    // A gleam down the left edge, as anime has it, broken once.
    line(scene, (x + 4, y + 12), (x + 4, y + 56), rgba(0xffffff, 230));
    line(scene, (x + 4, y + 62), (x + 4, y + 70), rgba(0xffffff, 230));
    sparkle(
        scene,
        (x + (11.6 * SCALE) as i32, y + (11.6 * SCALE) as i32),
        now,
        reduce_motion,
    );
    eye(scene, (x + 6, y + 40), -1, hit);
    eye(scene, (x + 18, y + 48), 1, hit);
    crown(scene, (x as f32 + 3.0, y as f32 - 1.0), -0.14);
}

/// A four-pointed glint on a corner, flaring now and then; held small with reduced motion.
fn sparkle(scene: &mut Canvas, (x, y): (i32, i32), now: f32, reduce_motion: bool) {
    let size = if reduce_motion {
        2
    } else {
        let phase = (now * 0.7).fract();
        if phase > 0.3 {
            return;
        }
        (1.0 + (phase / 0.3 * std::f32::consts::PI).sin() * 5.0) as i32
    };
    let x = x + 1;
    for step in -size..=size {
        let alpha = 255 - (step.abs() * 150 / size.max(1)) as u8;
        put(scene, x + step, y, rgba(0xffffff, alpha));
        put(scene, x, y + step, rgba(0xffffff, alpha));
    }
    put(scene, x, y, rgb(0xffffff));
}

/// One narrowed eye glaring out of the Sovereign, its brow scowling: `slant` tips the brow down
/// towards the other eye.
fn eye(scene: &mut Canvas, (x, y): (i32, i32), slant: i32, hit: bool) {
    let (iris, deep) = if hit {
        (rgb(0xffe080), rgb(0xd0a030))
    } else {
        (rgb(0xff3a48), rgb(0xa01828))
    };
    rect(scene, x, y, 7, 4, INK);
    rect(scene, x + 1, y + 1, 5, 1, deep);
    rect(scene, x + 1, y + 2, 5, 1, iris);
    // The lid narrows it from the inner corner.
    let inner = if slant < 0 { x + 5 } else { x + 1 };
    put(scene, inner, y + 1, INK);
    put(scene, x + 2, y + 2, rgb(0xffffff));
    for row in [2, 3] {
        line(
            scene,
            (x - 1, y - row + slant),
            (x + 7, y - row - slant),
            INK,
        );
    }
}

/// The crown's outline, a pixel to a unit, from the left of its foot: five points, the middle
/// one tallest.
const CROWN_SHAPE: [(f32, f32); 11] = [
    (0.0, 0.0),
    (0.0, -10.0),
    (3.5, -5.0),
    (6.0, -8.0),
    (8.5, -5.0),
    (11.0, -12.0),
    (13.5, -5.0),
    (16.0, -8.0),
    (18.5, -5.0),
    (22.0, -10.0),
    (22.0, 0.0),
];
const CROWN_WIDTH: f32 = 22.0;

/// A gold crown, its foot centred at `foot` and tipped by `tilt` radians, askew as a crown on a
/// pointer has to be: points lit from the left, a band set with jewels, a pearl on every point.
fn crown(scene: &mut Canvas, foot: (f32, f32), tilt: f32) {
    let (sin, cos) = tilt.sin_cos();
    let place = |(u, v): (f32, f32)| {
        let x = u - CROWN_WIDTH / 2.0;
        (foot.0 + x * cos - v * sin, foot.1 + x * sin + v * cos)
    };
    let local = |x: f32, y: f32| {
        let (dx, dy) = (x - foot.0, y - foot.1);
        (
            dx * cos + dy * sin + CROWN_WIDTH / 2.0,
            -dx * sin + dy * cos,
        )
    };
    let points: Vec<(f32, f32)> = CROWN_SHAPE.iter().map(|&point| place(point)).collect();
    let (left, top) = (foot.0 as i32 - 14, foot.1 as i32 - 16);
    let figure = Figure::new(
        (left, top),
        (left + 28, top + 18),
        Style {
            thick: false,
            bevel: false,
        },
        |x, y| {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if !inside(&points, px, py) {
                0
            } else if local(px, py).1 > -4.0 {
                2
            } else {
                1
            }
        },
        |_, _| false,
    );
    figure.paint(scene, (0, 0), 0.0, false, |x, y, part| {
        let (u, v) = local(x as f32 + 0.5, y as f32 + 0.5);
        if part == 2 {
            return if v < -3.0 {
                CROWN.light
            } else if v > -1.0 {
                CROWN.shadow
            } else {
                CROWN.base
            };
        }
        let lit = 1.0 - u / CROWN_WIDTH * 0.9 - (v + 12.0) / 12.0 * 0.2;
        if lit > 0.78 {
            CROWN.shine
        } else if lit > 0.55 {
            CROWN.light
        } else if lit > 0.3 {
            CROWN.base
        } else {
            CROWN.shadow
        }
    });
    for (u, color) in [(5.0, 0x3a6ad0), (11.0, 0xd02840), (17.0, 0x3a6ad0)] {
        let (x, y) = place((u, -2.0));
        let (x, y) = (x.round() as i32, y.round() as i32);
        rect(scene, x - 1, y - 1, 2, 2, rgb(color));
        put(scene, x - 1, y - 1, rgba(0xffffff, 180));
    }
    for (u, v) in [
        (0.5, -10.0),
        (6.0, -8.0),
        (11.0, -12.0),
        (16.0, -8.0),
        (21.5, -10.0),
    ] {
        let (x, y) = place((u, v));
        let (x, y) = (x.round() as i32, y.round() as i32);
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0)] {
            put(scene, x + dx, y + dy, INK);
        }
        put(scene, x, y, rgb(0xfff6e0));
    }
}

/// The closed hand's knuckles, the ends of its curled fingers, and its thumb, in units from the
/// middle of its grip.
const KNUCKLES: [(f32, f32); 4] = [(-2.75, -5.9), (-0.95, -6.35), (0.85, -6.3), (2.65, -5.8)];
const KNUCKLE_SIZE: f32 = 1.15;
const FINGERTIPS: [(f32, f32); 4] = [(-2.4, -0.8), (-0.8, -0.85), (0.8, -0.85), (2.4, -0.8)];
const FINGERTIP_SIZE: f32 = 0.78;
const THUMB: (f32, f32) = (-4.0, -2.9);
/// How the thumb leans: the sine and cosine of its tilt.
const THUMB_LEAN: (f32, f32) = (0.435, 0.900);
/// The back of the hand: left, top, right, bottom, and how round its corners are.
const PALM: (f32, f32, f32, f32, f32) = (-3.9, -6.0, 4.1, -0.9, 1.4);
/// How far the middle of the grip sits below the tip it hangs whoever is held from.
const GRIP: i32 = 5;

/// Which part of the closed hand a point (in units from the middle of its grip) is on: 1 the back
/// of the hand, 2 the thumb, 3 to 6 the knuckles, 7 to 10 the curled fingertips, 0 none.
fn fist(x: f32, y: f32) -> u8 {
    let (dx, dy) = (x - THUMB.0, y - THUMB.1);
    let (sin, cos) = THUMB_LEAN;
    let (u, v) = (dx * cos + dy * sin, -dx * sin + dy * cos);
    if (u / 1.05).powi(2) + (v / 1.75).powi(2) <= 1.0 {
        return 2;
    }
    let nearest = |centres: &[(f32, f32); 4], size: f32| {
        centres
            .iter()
            .map(|&(cx, cy)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
            .enumerate()
            .filter(|&(_, distance)| distance <= size)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index as u8)
    };
    if let Some(index) = nearest(&KNUCKLES, KNUCKLE_SIZE) {
        return 3 + index;
    }
    if let Some(index) = nearest(&FINGERTIPS, FINGERTIP_SIZE) {
        return 7 + index;
    }
    let (left, top, right, bottom, round) = PALM;
    let (cx, cy) = (
        x.clamp(left + round, right - round),
        y.clamp(top + round, bottom - round),
    );
    u8::from((x - cx).powi(2) + (y - cy).powi(2) <= round * round)
}

/// The middle of each round part of the closed hand, and its size, for shading it as a ball.
fn knob(part: u8) -> Option<((f32, f32), f32)> {
    match part {
        2 => Some((THUMB, 1.4)),
        3..=6 => Some((KNUCKLES[usize::from(part - 3)], KNUCKLE_SIZE)),
        7..=10 => Some((FINGERTIPS[usize::from(part - 7)], FINGERTIP_SIZE)),
        _ => None,
    }
}

/// A pixel of the closed hand, relative to the middle of its grip, in units.
fn units(x: i32, y: i32) -> (f32, f32) {
    ((x as f32 + 0.5) / SCALE, (y as f32 + 0.5) / SCALE)
}

/// The closed hand, worked out once, the middle of its grip at the origin.
fn closed_hand() -> &'static (Figure, Glow) {
    static HAND_SHAPE: OnceLock<(Figure, Glow)> = OnceLock::new();
    HAND_SHAPE.get_or_init(|| {
        let figure = Figure::new(
            ((-5.2 * SCALE) as i32, (-7.6 * SCALE) as i32),
            ((4.2 * SCALE) as i32, 1),
            Style {
                thick: true,
                bevel: true,
            },
            |x, y| {
                let (ux, uy) = units(x, y);
                fist(ux, uy)
            },
            |a, b| {
                // Knuckles run smoothly into the back of the hand; everything else is creased.
                let (low, high) = (a.min(b), a.max(b));
                !(low == 1 && (3..=6).contains(&high))
            },
        );
        let glow = Glow::new(&figure, (0.0, -3.8 * SCALE));
        (figure, glow)
    })
}

/// The Sovereign as the closed hand a desktop shows while dragging, gripping whoever hangs from
/// `at`: the same aura, outline, glare and crown as the pointer.
fn hand(scene: &mut Canvas, at: (f32, f32), now: f32, reduce_motion: bool) {
    let (figure, glow) = closed_hand();
    // The fingertips close just over the top of whoever is held.
    let (x, y) = anchor((at.0, at.1 + GRIP as f32));
    let breath = breath(now, reduce_motion);
    glow.paint(scene, (x, y), breath);
    let (left, top, right, bottom, _) = PALM;
    figure.paint(scene, (x, y), 0.5 + breath * 0.5, false, |lx, ly, part| {
        let (ux, uy) = units(lx, ly);
        let down = (uy - top) / (bottom - top);
        let across = (ux - left) / (right - left);
        let mut lit = 1.0 - down * 0.55 - across * 0.45;
        if let Some(((cx, cy), size)) = knob(part) {
            let round = ((ux - cx) + (uy - cy)) / (size * 1.6);
            lit = lit * 0.55 + (0.62 - round * 0.5) * 0.45 + 0.1;
        }
        white(lit)
    });
    eye(scene, (x - 13, y - 25), -1, false);
    eye(scene, (x + 3, y - 24), 1, false);
    crown(scene, (x as f32 - 6.0, y as f32 - 7.5 * SCALE), -0.16);
}

/// Streaks left behind a hand hauling something fast across the stage, along the way it goes.
fn haste(scene: &mut Canvas, at: (f32, f32), before: (f32, f32)) {
    let speed = at.0 - before.0;
    if speed.abs() < 1.5 {
        return;
    }
    let back = -speed.signum();
    for (index, rise) in [40, 30, 19, 8].iter().enumerate() {
        let from = at.0 + back * (34.0 + (index % 2) as f32 * 6.0);
        let length = 10.0 + index as f32 * 4.0 + speed.abs() * 3.0;
        let y = at.1 as i32 - rise;
        line(
            scene,
            (from as i32, y),
            ((from + back * length) as i32, y),
            rgba(0xf0e0ff, 140 - index as u8 * 20),
        );
    }
}

/// The ring a click sends out, `progress` of the way spread.
fn shockwave(scene: &mut Canvas, at: (f32, f32), progress: f32, reduce_motion: bool) {
    let rings: &[f32] = if reduce_motion {
        &[0.6]
    } else {
        &[progress, progress * 0.6]
    };
    for (index, spread) in rings.iter().enumerate() {
        let radius = 6.0 + spread * 34.0;
        let alpha = ((1.0 - spread) * 230.0) as u8;
        for step in 0..72 {
            let angle = step as f32 / 72.0 * TAU;
            let x = at.0 + angle.cos() * radius;
            let y = at.1 + angle.sin() * radius * 0.45;
            let color = if index == 0 {
                rgba(0xfff4c0, alpha)
            } else {
                rgba(0xb48af0, alpha)
            };
            put(scene, x as i32, y as i32, color);
        }
    }
}

/// The selection: dragged out with marching ants round a pale wash, then, as it closes on them,
/// a flash, a ring of light pulsing out from its edges, and everyone inside turned the blue of a
/// selected icon, with handles at its corners. Reduced motion has it simply made, and held.
fn selection(scene: &mut Canvas, area: Patch, t: f32, now: f32, reduce_motion: bool) {
    const CLOSES: f32 = 1.2;
    let grow = if reduce_motion {
        1.0
    } else {
        let grow = ((t - 0.3) / (CLOSES - 0.3)).clamp(0.0, 1.0);
        1.0 - (1.0 - grow) * (1.0 - grow)
    };
    if grow <= 0.0 {
        return;
    }
    let (left, top, right, bottom) = area;
    let (width, height) = (
        (((right - left) * grow) as i32).max(2),
        (((bottom - top) * grow) as i32).max(2),
    );
    let (x, y) = (left as i32, top as i32);
    let closed = reduce_motion || t >= CLOSES;
    let since = t - CLOSES;
    if closed {
        let glow = if reduce_motion {
            0.5
        } else {
            (now * 3.0).sin() * 0.5 + 0.5
        };
        rect(
            scene,
            x,
            y,
            width,
            height,
            rgba(0x2a6ae8, (70.0 + 30.0 * glow) as u8),
        );
        rect(scene, x + 1, y + 1, width - 2, 1, rgba(0xa8d0ff, 140));
        rect(scene, x + 1, y + 1, 1, height - 2, rgba(0xa8d0ff, 140));
    } else {
        rect(scene, x, y, width, height, rgba(0x6aa0f0, 50));
    }
    if !reduce_motion && (0.0..0.35).contains(&since) {
        let fade = 1.0 - since / 0.35;
        rect(
            scene,
            x,
            y,
            width,
            height,
            rgba(0xf4f0ff, (fade * fade * 220.0) as u8),
        );
    }
    if !reduce_motion {
        for delay in [0.0, 0.14] {
            let pulse = (since - delay) / 0.5;
            if (0.0..1.0).contains(&pulse) {
                let out = (pulse * 20.0) as i32;
                let alpha = ((1.0 - pulse) * 230.0) as u8;
                for inner in 0..2 {
                    frame(
                        scene,
                        (x - out + inner, y - out + inner),
                        (width + (out - inner) * 2, height + (out - inner) * 2),
                        rgba(0xe8d8ff, alpha),
                    );
                }
            }
        }
    }
    let march = if reduce_motion {
        0
    } else {
        (now * 12.0) as i32
    };
    let ant = |i: i32| {
        if ((i + march) / 3) % 2 == 0 {
            INK
        } else {
            rgb(0xffffff)
        }
    };
    for dx in 0..width {
        put(scene, x + dx, y, ant(dx));
        put(scene, x + dx, y + height - 1, ant(dx + 1));
    }
    for dy in 0..height {
        put(scene, x, y + dy, ant(dy + 1));
        put(scene, x + width - 1, y + dy, ant(dy));
    }
    if closed {
        let (right, bottom) = (x + width - 1, y + height - 1);
        let (middle, centre) = ((x + right) / 2, (y + bottom) / 2);
        for (hx, hy) in [
            (x, y),
            (middle, y),
            (right, y),
            (x, centre),
            (right, centre),
            (x, bottom),
            (middle, bottom),
            (right, bottom),
        ] {
            rect(scene, hx - 2, hy - 2, 5, 5, INK);
            rect(scene, hx - 1, hy - 1, 3, 3, rgb(0xffffff));
        }
    }
}

/// A one-pixel rectangle outline.
fn frame(scene: &mut Canvas, (x, y): (i32, i32), (width, height): (i32, i32), color: Rgba) {
    rect(scene, x, y, width, 1, color);
    rect(scene, x, y + height - 1, width, 1, color);
    rect(scene, x, y + 1, 1, height - 2, color);
    rect(scene, x + width - 1, y + 1, 1, height - 2, color);
}

/// The pointer breaking apart: shards flying out from where it hung, fading.
fn shatter(scene: &mut Canvas, t: f32, reduce_motion: bool) {
    let progress = if reduce_motion {
        0.5
    } else {
        (t / 1.8).clamp(0.0, 1.0)
    };
    let centre = (HOME.0 + 30.0, HOME.1 + 56.0);
    for shard in 0..18 {
        let angle = shard as f32 / 18.0 * TAU + 0.3;
        let distance = progress * (40.0 + (shard % 5) as f32 * 14.0);
        let (x, y) = (
            centre.0 + angle.cos() * distance,
            centre.1 + angle.sin() * distance + progress * progress * 30.0,
        );
        let alpha = ((1.0 - progress) * 255.0) as u8;
        let size = 2 + (shard % 3);
        let tone = if shard % 4 == 0 {
            rgba(0x0c0a10, alpha)
        } else {
            rgba(0xf6f2fc, alpha)
        };
        rect(scene, x as i32, y as i32, size, size, tone);
    }
    if progress < 0.3 {
        let alpha = ((0.3 - progress) / 0.3 * 200.0) as u8;
        rect(
            scene,
            0,
            0,
            scene.width() as i32,
            scene.height() as i32,
            rgba(0xffffff, alpha),
        );
    }
}

/// What is left: a very ordinary little pointer, hurrying off on tiny legs.
fn little_pointer(scene: &mut Canvas, at: (f32, f32), now: f32, reduce_motion: bool) {
    const SMALL: [&str; 11] = [
        "#.......", "##......", "#o#.....", "#oo#....", "#ooo#...", "#oooo#..", "#ooooo#.",
        "#oo####.", "#o#o#...", "##.#o#..", "#...##..",
    ];
    let (x, y) = (at.0 as i32, at.1 as i32 - 14);
    for (dy, row) in SMALL.iter().enumerate() {
        for (dx, cell) in row.bytes().enumerate() {
            let color = match cell {
                b'#' => INK,
                b'o' => rgb(0xffffff),
                _ => continue,
            };
            put(scene, x + dx as i32, y + dy as i32, color);
        }
    }
    let stride = if reduce_motion {
        0
    } else {
        ((now * 16.0).sin() * 1.5) as i32
    };
    line(scene, (x + 2, y + 11), (x + 1 + stride, y + 14), INK);
    line(scene, (x + 4, y + 11), (x + 5 - stride, y + 14), INK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_move_comes_back_home_or_goes() {
        for step in [
            Move::Click { at: (120.0, 186.0) },
            Move::Drag {
                who: 1,
                from: (90.0, 186.0),
                to: (170.0, 190.0),
            },
            Move::Reel,
        ] {
            let end = tip(step, step.secs());
            assert!(
                (end.0 - HOME.0).abs() < 1.0 && (end.1 - HOME.1).abs() < 1.0,
                "{step:?} ends at {end:?}"
            );
        }
        let fled = tip(Move::Flee, Move::Flee.secs());
        assert!(fled.0 > 384.0, "it scurries off the edge");
    }

    const DRAG: Move = Move::Drag {
        who: 1,
        from: (90.0, 186.0),
        to: (170.0, 190.0),
    };

    #[test]
    fn the_sovereign_is_drawn_in_every_move() {
        for step in [
            Move::Hover,
            Move::Click { at: (120.0, 186.0) },
            DRAG,
            Move::Select {
                area: (40.0, 150.0, 200.0, 204.0),
            },
            Move::Reel,
            Move::Shatter,
            Move::Flee,
        ] {
            let mut scene = Canvas::new(384, 216);
            draw(&mut scene, step, step.secs() * 0.5, 1.0, false);
            assert!(scene.alpha_bounds().is_some(), "{step:?} drew nothing");
        }
        let mut gone = Canvas::new(384, 216);
        draw(&mut gone, Move::Gone, 0.0, 1.0, false);
        assert!(gone.alpha_bounds().is_none());
    }

    #[test]
    fn with_reduced_motion_every_move_holds_still() {
        for step in [
            Move::Hover,
            Move::Click { at: (120.0, 186.0) },
            DRAG,
            Move::Select {
                area: (60.0, 150.0, 172.0, 206.0),
            },
            Move::Reel,
            Move::Shatter,
            Move::Flee,
        ] {
            let frame = |t: f32, now: f32| {
                let mut scene = Canvas::new(384, 216);
                draw(&mut scene, step, t, now, true);
                scene
            };
            let t = step.secs() * 0.5;
            assert_eq!(frame(t, 3.0), frame(t + 0.07, 3.07), "{step:?} moved");
        }
    }

    #[test]
    fn whoever_is_dragged_still_shows_its_face() {
        for t in [0.6, 1.2, 1.8] {
            let mut scene = Canvas::new(384, 216);
            draw(&mut scene, DRAG, t, 1.0, false);
            // Whoever is held hangs with its feet 30 below the tip, its face between.
            let (x, y) = anchor(tip(DRAG, t));
            for row in y + 12..y + 30 {
                for column in x - 10..=x + 10 {
                    let covered = scene.get(column, row).a;
                    assert!(covered < 32, "covered at {column}, {row}, {t}s in");
                }
            }
        }
    }
}
