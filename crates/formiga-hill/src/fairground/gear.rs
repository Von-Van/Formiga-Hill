//! The games' gear, drawn as the Fairground is: the sacks the racers hop in, the chalk line and
//! the ribbon on its two posts, and the high striker's moving parts, its puck, its mallet and the
//! shine of its bell. (The striker itself stands painted in the scenery.)
//!
//! Every material is shaded with a ramp, lit from the upper left and outlined in a darker shade of
//! its own colour, never black, so the creatures' own near-black outlines read first; the sacking
//! has a weave to it, from `paint::noise`.

use super::scenery::{BELL, PAD, PUCK_RAIL, STRIKER};
use crate::paint::{Ramp, chance, ellipse, line, mix, noise, put, rgb, rgba};
use crate::playground::Prop;
use formiga_art::{Canvas, Rgba};

/// Brass for the bell, iron for the puck and the mallet's bands, hickory for its handle and oak
/// for its head, red satin for the ribbon and cream paint for the bands on its posts.
const BRASS: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const IRON: Ramp = Ramp::new(0x2e2628, 0x4a4240, 0x6a6058, 0x8c8276, 0xb4a898);
const HICKORY: Ramp = Ramp::new(0x5e4228, 0x8a6640, 0xae8656, 0xc8a26e, 0xe0c08e);
const OAK: Ramp = Ramp::new(0x3e2618, 0x5e3c26, 0x7e5434, 0x9c6e46, 0xb88a5e);
const SATIN: Ramp = Ramp::new(0x6a1424, 0x9a2034, 0xc8303e, 0xe8606a, 0xffb0b0);
const PAINT: Ramp = Ramp::new(0x7a6a5a, 0xc8b8a0, 0xe8dcc4, 0xf4ecd8, 0xffffff);
const CHALK: Rgba = rgb(0xf4f0e4);
/// How long the bell rings for.
pub const RING_SECS: f32 = 2.0;
/// How tall a ribbon post stands.
pub const POST_TALL: i32 = 21;

/// Undyed sacking, pale as oatmeal so it shows against the sawdust.
const SACKING: Ramp = Ramp::new(0x6a5438, 0xae9670, 0xd2bf98, 0xe8d8b4, 0xf8eed6);
/// The band printed round each sack, so racers can be told apart: one colour to each.
const BANDS: [Ramp; 6] = [
    Ramp::new(0x6a1e24, 0x9a2e34, 0xc0464a, 0xd8706a, 0xf0a090),
    Ramp::new(0x1e2c58, 0x2c4480, 0x3a5ea8, 0x5a7ec4, 0x86a6dc),
    Ramp::new(0x1e4a2a, 0x2a6a3a, 0x3e8a4e, 0x62aa6a, 0x96cc96),
    Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2),
    Ramp::new(0x3e2450, 0x5a3672, 0x7a4e96, 0x9a6eb4, 0xc4a0d6),
    Ramp::new(0x173c40, 0x22585c, 0x2f7a7a, 0x45999a, 0x6ab8b4),
];

/// How wide a companion's sack is, from a frame of it standing at rest facing right: as wide as
/// its body is low down (its feet, and whatever fills most of the way down from where the sack's
/// mouth comes, but not a tail or an arm held out). Worked out once, so the sack keeps its size
/// whatever the companion does in it. `None` if there is nothing drawn.
pub fn girth(resting: &Canvas) -> Option<(i32, i32)> {
    let (left, top, right, bottom) = resting.alpha_bounds()?;
    let (left, top, right, bottom) = (left as i32, top as i32, right as i32, bottom as i32);
    let depth = depth(bottom - top + 1);
    let mouth = bottom - depth + 1;
    let solid = |x: i32, y: i32| resting.get(x, y).a > 0;
    let body: Vec<i32> = (left..=right)
        .filter(|&x| {
            let feet = (bottom - 1..=bottom).any(|y| solid(x, y));
            let filled = (mouth..=bottom).filter(|&y| solid(x, y)).count() as i32;
            feet || filled * 10 >= depth * 7
        })
        .collect();
    Some((body.first()? - 1, body.last()? + 1))
}

/// How far up a companion `tall` pixels tall its sack comes: a little under halfway.
fn depth(tall: i32) -> i32 {
    ((tall as f32 * 0.48).round() as i32).clamp(7, 15)
}

/// A sack pulled up over a companion's lower body, for one frame of it: on a canvas the frame's
/// size, to lay over it. `girth` is how far across it goes (see [`girth`]); it comes up a little
/// under halfway, however tall the companion is in this frame, so it squashes as the companion
/// crouches and stretches as it springs. Its mouth is rolled over at the top, it is gathered under
/// that, its belly is full where the feet push at it, its corners stick out at the foot, and a
/// coloured band goes round it.
pub fn sack(frame: &Canvas, band: u8, (from, to): (i32, i32)) -> Canvas {
    let mut sack = Canvas::new(frame.width(), frame.height());
    let Some((_, top, _, bottom)) = frame.alpha_bounds() else {
        return sack;
    };
    let (top, bottom) = (top as i32, bottom as i32);
    let depth = depth(bottom - top + 1);
    let mouth = bottom - depth + 1;
    let width = (to - from).max(1);
    let ramp = BANDS[usize::from(band) % BANDS.len()];
    let stripe = mouth + 4 + (depth - 7) / 3;
    // How far in from `from` and `to` each row comes: out a little at the rolled mouth, gathered
    // under it, full through the middle, bellied lower down where the feet push at it, and its
    // corners sticking out at the foot.
    let inset = |row: i32| -> i32 {
        if row <= 1 {
            0
        } else if row <= 3 {
            1
        } else if row >= depth - 1 {
            0
        } else if row * 2 >= depth && row < depth - 2 {
            -1
        } else {
            0
        }
    };
    for y in mouth..=bottom {
        let row = y - mouth;
        let (l, r) = (from + inset(row), to - inset(row));
        for x in l..=r {
            let across = (x - from) as f32 / width as f32;
            let down = row as f32 / depth as f32;
            // The top of the roll is ragged where the cloth bunches.
            if row == 0
                && x != l
                && x != r
                && noise(x, mouth, 917 + u32::from(band)).is_multiple_of(4)
            {
                continue;
            }
            let outline = x == l
                || x == r
                || y == bottom
                || row == 0
                || (row == 1 && sack.get(x, y - 1).a == 0);
            let color = if outline {
                SACKING.edge
            } else if row == 1 {
                // The rolled mouth, catching the light along its top.
                if across < 0.35 {
                    SACKING.shine
                } else if across < 0.8 {
                    SACKING.light
                } else {
                    SACKING.base
                }
            } else if row == 2 || row == 3 {
                // Gathered in under the roll: pleats, in its shade.
                let pleat = (x - from) % 3 == 0;
                if row == 2 {
                    mix(SACKING.shadow, SACKING.edge, 0.3)
                } else if pleat {
                    SACKING.shadow
                } else if across < 0.4 {
                    SACKING.light
                } else {
                    SACKING.base
                }
            } else if y == stripe && y < bottom - 2 {
                if across < 0.25 {
                    ramp.light
                } else if across < 0.72 {
                    ramp.base
                } else {
                    ramp.shadow
                }
            } else {
                // Lit from the upper left, rounding away to the right and underneath.
                let lit = 0.75 - across - 0.45 * down;
                let mut tone = if lit > 0.45 {
                    SACKING.light
                } else if lit > -0.12 {
                    SACKING.base
                } else {
                    SACKING.shadow
                };
                // The weave, and a crease or two where it is pulled up.
                if (x + y) % 2 == 0 && chance(x, y, 911 + u32::from(band), 48) {
                    tone = mix(tone, SACKING.edge, 0.28);
                }
                let crease = x - from == width / 3 + (row - 4) / 3
                    || x - from == width * 2 / 3 - (row - 4) / 4;
                if crease && row > 4 && row < depth - 2 && !noise(x, y, 913).is_multiple_of(3) {
                    tone = mix(tone, SACKING.shadow, 0.7);
                }
                tone
            };
            sack.set(x, y, color);
        }
    }
    // The corners at the foot, poking out.
    for x in [from - 1, to + 1] {
        sack.set(x, bottom, SACKING.edge);
    }
    sack
}

/// Paints rows of letters with their top-left at `(x, y)`: `E`, `S`, `B`, `L` and `W` are
/// `ramp`'s edge, shadow, base, light and shine; anything else is left clear.
fn stamp(canvas: &mut Canvas, (x, y): (i32, i32), rows: &[&str], ramp: Ramp) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            let color = match letter {
                b'E' => ramp.edge,
                b'S' => ramp.shadow,
                b'B' => ramp.base,
                b'L' => ramp.light,
                b'W' => ramp.shine,
                _ => continue,
            };
            put(canvas, x + dx as i32, y + dy as i32, color);
        }
    }
}

/// A piece of gear standing on row `base`, so it is drawn among the colony in depth order. It is
/// painted on a canvas just big enough for it, `(left, top, width, height)` in the scene: `paint`
/// is given that canvas and its top-left, to take from anything placed in the scene.
fn piece(
    base: f32,
    (left, top, width, height): (i32, i32, i32, i32),
    paint: impl FnOnce(&mut Canvas, (i32, i32)),
) -> Prop {
    let mut canvas = Canvas::new(width.max(1) as u32, height.max(1) as u32);
    paint(&mut canvas, (left, top));
    Prop::new(canvas, (left, top), base)
}

/// The striker's brass bell, sitting on its cap. `rung` is how long ago it rang, if it did: for
/// a moment it shakes on its seat and shines, its rays flickering. With motion reduced it holds
/// still and shines steadily.
pub fn bell(rung: Option<f32>, reduce_motion: bool) -> Prop {
    let (cx, rim) = BELL;
    piece(
        STRIKER.1 as f32,
        (cx - 13, rim - 17, 27, 23),
        |canvas, (left, top)| {
            let (cx, rim) = (cx - left, rim - top);
            let ringing = rung.filter(|since| (0.0..RING_SECS).contains(since));
            let shake = match ringing {
                Some(since) if !reduce_motion && since < 0.7 => {
                    if (since * 24.0) as i32 % 2 == 0 {
                        -1
                    } else {
                        1
                    }
                }
                _ => 0,
            };
            stamp(
                canvas,
                (cx - 4 + shake, rim - 7),
                &[
                    "...EEE...",
                    "..ELBSE..",
                    "..EWBSE..",
                    ".ELWBBSE.",
                    ".ELWBBSE.",
                    ".ELLBBSE.",
                    "ELLBBBSSE",
                    "EEEEEEEEE",
                ],
                BRASS,
            );
            let Some(since) = ringing else {
                return;
            };
            let fade = if reduce_motion {
                1.0
            } else {
                1.0 - since / RING_SECS
            };
            // Only solid pixels: a soft halo would show the daytime sky through it after dark.
            let middle = (cx, rim - 4);
            // It rings out: a curl of sound either side while it shakes.
            if reduce_motion || since < 0.9 {
                let ring = rgba(0xfff8e0, (230.0 * fade) as u8);
                for (side, dx) in [(-1, -7), (1, 7)] {
                    put(canvas, cx + dx, middle.1 - 2, ring);
                    put(canvas, cx + dx + side, middle.1 - 1, ring);
                    put(canvas, cx + dx + side, middle.1, ring);
                    put(canvas, cx + dx, middle.1 + 1, ring);
                }
            }
            let beat = if reduce_motion {
                0
            } else {
                (since * 10.0) as i32
            };
            for ray in 0..8 {
                if !reduce_motion && (ray + beat) % 3 == 0 {
                    continue;
                }
                let angle = ray as f32 * std::f32::consts::FRAC_PI_4 + 0.2;
                let (dx, dy) = (angle.cos(), angle.sin() * 0.8);
                let reach = if ray % 2 == 0 { 11.0 } else { 9.0 };
                for step in 0..3 {
                    let distance = reach - 2.0 + step as f32;
                    let (x, y) = (
                        middle.0 + (dx * distance).round() as i32,
                        middle.1 + (dy * distance).round() as i32,
                    );
                    let color = if step == 1 {
                        rgba(0xffffff, (255.0 * fade) as u8)
                    } else {
                        rgba(0xffd860, (230.0 * fade) as u8)
                    };
                    put(canvas, x, y, color);
                }
            }
        },
    )
}

/// The puck on its rail, `height` of the way from rest to the bell.
pub fn puck(height: f32) -> Prop {
    let (rest, top) = PUCK_RAIL;
    let y = rest - ((rest - top) as f32 * height.clamp(0.0, 1.0)).round() as i32;
    piece(
        STRIKER.1 as f32 + 0.1,
        (STRIKER.0 - 2, y, 5, 4),
        |canvas, _| {
            stamp(canvas, (0, 0), &[".EEE.", "ELWBE", "ESBSE", ".EEE."], IRON);
            // A brass band round it, catching the light.
            put(canvas, 1, 2, BRASS.light);
            put(canvas, 2, 2, BRASS.base);
            put(canvas, 3, 2, BRASS.shadow);
        },
    )
}

/// Where the mallet is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mallet {
    /// Leaning against the striker's plinth, waiting.
    Leaning,
    /// In someone's paws: where they grip it, where its head is, and the row they stand on.
    Held {
        grip: (i32, i32),
        head: (i32, i32),
        feet: f32,
    },
}

impl Mallet {
    /// Held at the ready by someone whose head's top middle is at `crown`, its head on the
    /// ground in front.
    pub fn ready(crown: (f32, f32), feet: f32, facing_right: bool) -> Self {
        let ahead = if facing_right { 1 } else { -1 };
        let (x, y) = (crown.0.round() as i32, crown.1.round() as i32);
        Self::Held {
            grip: (x + ahead * 5, paws(y, feet)),
            head: (x + ahead * 12, feet.round() as i32 - 2),
            feet,
        }
    }

    /// Swung up overhead.
    pub fn raised(crown: (f32, f32), feet: f32, facing_right: bool) -> Self {
        let back = if facing_right { -2 } else { 2 };
        let (x, y) = (crown.0.round() as i32, crown.1.round() as i32);
        Self::Held {
            grip: (x, y + 4),
            head: (x + back, y - 12),
            feet,
        }
    }

    /// Brought down on the pad.
    pub fn down(crown: (f32, f32), feet: f32, facing_right: bool) -> Self {
        let ahead = if facing_right { 5 } else { -5 };
        let (x, y) = (crown.0.round() as i32, crown.1.round() as i32);
        Self::Held {
            grip: (x + ahead, paws(y, feet)),
            head: (PAD.0, PAD.1 - 3),
            feet,
        }
    }
}

/// The row where a companion whose head's top is at row `crown` and whose feet are at row `feet`
/// holds something low in front of it.
fn paws(crown: i32, feet: f32) -> i32 {
    (crown as f32 + (feet - crown as f32) * 0.66).round() as i32
}

/// The mallet: a hickory handle and a round oak head with iron bands at its ends, lit from the
/// upper left.
pub fn mallet(at: Mallet) -> Prop {
    let (grip, head, base) = match at {
        Mallet::Leaning => {
            let (cx, foot) = STRIKER;
            ((cx + 9, foot - 13), (cx + 14, foot - 2), foot as f32 + 0.2)
        }
        Mallet::Held { grip, head, feet } => (grip, head, feet + 0.5),
    };
    let (dx, dy) = (head.0 - grip.0, head.1 - grip.1);
    let long = ((dx * dx + dy * dy) as f32).sqrt().max(1.0);
    let tail = (
        grip.0 - (dx as f32 / long * 2.0).round() as i32,
        grip.1 - (dy as f32 / long * 2.0).round() as i32,
    );
    let left = tail.0.min(head.0) - 5;
    let top = tail.1.min(head.1) - 4;
    let area = (
        left,
        top,
        tail.0.max(head.0) + 6 - left,
        tail.1.max(head.1) + 4 - top,
    );
    piece(base, area, |canvas, (left, top)| {
        let at = |(x, y): (i32, i32)| (x - left, y - top);
        let (tail, head) = (at(tail), at(head));
        // The handle, lit along one side, from a little past the grip to the head.
        // Two pixels thick: lit down one side, in shade down the other, outlined at its end.
        line(
            canvas,
            (tail.0 + 1, tail.1),
            (head.0 + 1, head.1),
            HICKORY.shadow,
        );
        line(
            canvas,
            (tail.0 - 1, tail.1),
            (head.0 - 1, head.1),
            HICKORY.edge,
        );
        line(
            canvas,
            (tail.0 + 2, tail.1),
            (head.0 + 2, head.1),
            HICKORY.edge,
        );
        line(canvas, tail, head, HICKORY.light);
        put(canvas, tail.0, tail.1 + 1, HICKORY.edge);
        put(canvas, tail.0 + 1, tail.1 + 1, HICKORY.edge);
        // The head, a round of oak banded in iron at either end.
        stamp(
            canvas,
            (head.0 - 4, head.1 - 3),
            &[
                ".EEEEEEE.",
                "EWLLLBBSE",
                "ELLWLBBSE",
                "ELBBBBSSE",
                "ESSSSSSSE",
                ".EEEEEEE.",
            ],
            OAK,
        );
        for x in [head.0 - 3, head.0 + 3] {
            put(canvas, x, head.1 - 2, IRON.light);
            put(canvas, x, head.1 - 1, IRON.base);
            put(canvas, x, head.1, IRON.base);
            put(canvas, x, head.1 + 1, IRON.shadow);
        }
    })
}

/// A chalk line on the ground from `from` to `to` (the back of it to the front), slanting either
/// way: drawn under everyone, a little uneven, with chalk dust either side. The line the racers
/// start from, and the one across the middle of the tug-of-war.
pub fn chalk_line(from: (i32, i32), to: (i32, i32)) -> Prop {
    let left = from.0.min(to.0) - 3;
    let area = (left, from.1, (from.0 - to.0).abs() + 7, to.1 - from.1 + 1);
    piece(f32::MIN, area, |canvas, (left, top)| {
        let rows = (to.1 - from.1).max(1) as f32;
        let across = |y: i32| from.0 as f32 + (to.0 - from.0) as f32 * (y - from.1) as f32 / rows;
        for scene_y in from.1..=to.1 {
            let (here, next) = (
                across(scene_y).round() as i32,
                across(scene_y + 1).round() as i32,
            );
            let (start, end) = (here.min(next), here.max(next));
            for scene_x in start..end.max(start + 1) {
                let (x, y) = (scene_x - left, scene_y - top);
                if !chance(scene_x, scene_y, 933, 30) {
                    put(canvas, x, y, CHALK);
                }
                if chance(scene_x, scene_y, 935, 90) {
                    put(canvas, x + 1, y, rgba(0xf4f0e4, 140));
                }
                if chance(scene_x, scene_y, 937, 50) {
                    put(canvas, x - 1, y, rgba(0xf4f0e4, 90));
                }
            }
        }
    })
}

/// A post for the ribbon with its foot at `(x, foot)`: banded in red and cream paint, a brass
/// knob on top, its shadow on the ground. Its top is `POST_TALL` above its foot.
pub fn post(x: i32, foot: i32) -> Prop {
    let area = (x - 2, foot - POST_TALL - 2, 8, POST_TALL + 4);
    piece(foot as f32, area, |canvas, (left, top)| {
        let (x, foot) = (x - left, foot - top);
        ellipse(canvas, x + 2, foot, 3, 1, rgba(0x2a1e28, 70));
        for y in foot - POST_TALL + 2..=foot {
            let ramp = if (foot - y) / 4 % 2 == 0 {
                SATIN
            } else {
                PAINT
            };
            put(canvas, x - 1, y, ramp.edge);
            put(canvas, x, y, ramp.light);
            put(canvas, x + 1, y, ramp.shadow);
            put(canvas, x + 2, y, ramp.edge);
        }
        stamp(
            canvas,
            (x - 1, foot - POST_TALL - 1),
            &[".EE.", "EWLE", "ELSE", ".EE."],
            BRASS,
        );
    })
}

/// The ribbon across the finish between the tops of its two posts, whose feet are at `back` and
/// `front`: whole, sagging a little, or broken by the winner `broken` seconds ago and hanging
/// from each post, swinging a moment (not with motion reduced).
pub fn ribbon(
    back: (i32, i32),
    front: (i32, i32),
    broken: Option<f32>,
    reduce_motion: bool,
) -> Prop {
    let top_of = |(x, foot): (i32, i32)| (x + 1, foot - POST_TALL + 1);
    let (back, front) = (top_of(back), top_of(front));
    let left = back.0.min(front.0) - 4;
    let top = back.1.min(front.1) - 2;
    let area = (
        left,
        top,
        back.0.max(front.0) + 9 - left,
        back.1.max(front.1) + 13 - top,
    );
    piece(
        front.1 as f32 + POST_TALL as f32 - 0.5,
        area,
        |canvas, (left, top)| {
            let (back, front) = (
                (back.0 - left, back.1 - top),
                (front.0 - left, front.1 - top),
            );
            let mut strip = |from: (f32, f32), to: (f32, f32), sag: f32| {
                let at = |t: f32| {
                    (
                        from.0 + (to.0 - from.0) * t,
                        from.1 + (to.1 - from.1) * t + sag * 4.0 * t * (1.0 - t),
                    )
                };
                let mut previous = at(0.0);
                for step in 1..=24 {
                    let point = at(step as f32 / 24.0);
                    let (a, b) = (
                        (previous.0.round() as i32, previous.1.round() as i32),
                        (point.0.round() as i32, point.1.round() as i32),
                    );
                    line(canvas, (a.0 + 1, a.1), (b.0 + 1, b.1), SATIN.shadow);
                    line(canvas, a, b, SATIN.light);
                    previous = point;
                }
            };
            let float = |(x, y): (i32, i32)| (x as f32, y as f32);
            match broken {
                None => strip(float(back), float(front), 3.0),
                Some(since) => {
                    let swing = if reduce_motion {
                        0.0
                    } else {
                        (since * 7.0).sin() * (2.5 - since).max(0.0)
                    };
                    // Each end hangs from its post towards the other.
                    for (end, other) in [(back, front), (front, back)] {
                        let side = if other.0 >= end.0 { 1.0 } else { -1.0 };
                        let start = float(end);
                        let tip = (start.0 + side * 2.0 + swing, start.1 + 9.0);
                        strip(start, tip, 0.0);
                    }
                }
            }
        },
    )
}

// ---------------------------------------------------------------------------------------------
// Hoopla
// ---------------------------------------------------------------------------------------------

/// A hoopla ring, by its colour: each player's three are red, gold and blue, thrown in that order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hoop {
    Red,
    Gold,
    Blue,
}

/// Painted cane for the rings: red, gold and blue.
const HOOP_BLUE: Ramp = Ramp::new(0x1e2c58, 0x2c4480, 0x3a5ea8, 0x5a7ec4, 0x9ab8e4);

impl Hoop {
    pub const ALL: [Self; 3] = [Self::Red, Self::Gold, Self::Blue];

    fn ramp(self) -> Ramp {
        match self {
            Self::Red => SATIN,
            Self::Gold => BRASS,
            Self::Blue => HOOP_BLUE,
        }
    }
}

/// Paints rows of letters with their top-left at `(x, y)`, as `stamp` does, from a ring's ramp.
fn hoop_rows(canvas: &mut Canvas, (x, y): (i32, i32), rows: &[&str], hoop: Hoop) {
    stamp(canvas, (x, y), rows, hoop.ramp());
}

/// A ring lying flat, or seen flat in the air, about `(x, y)`: a hoop seen from a little above,
/// wide enough to go over a prize's block, its far side lit and its near side in shade.
fn hoop_flat(canvas: &mut Canvas, (x, y): (i32, i32), hoop: Hoop) {
    hoop_rows(
        canvas,
        (x - 4, y - 1),
        &[".LWLLLLB.", "L.......S", ".SSSSSSE."],
        hoop,
    );
}

/// A ring turning in the air, as it is seen through its spin: face on, then edge on.
fn hoop_turning(canvas: &mut Canvas, (x, y): (i32, i32), spin: f32, hoop: Hoop) {
    match (spin * 4.0).rem_euclid(4.0) as i32 {
        0 => hoop_flat(canvas, (x, y), hoop),
        2 => hoop_rows(
            canvas,
            (x - 1, y - 3),
            &["L", "LB", "LB", "BS", "BS", "S"],
            hoop,
        ),
        _ => hoop_rows(
            canvas,
            (x - 2, y - 2),
            &[".LWB.", "L...S", "L...S", "B...E", ".SSE."],
            hoop,
        ),
    }
}

/// A ring dropped over a prize's block whose foot is at `(x, foot)`, lying round it on the
/// counter: its far side showing either side of the block, its sides, and its near side in front.
fn hoop_round_block(canvas: &mut Canvas, (x, foot): (i32, i32), hoop: Hoop) {
    let ramp = hoop.ramp();
    put(canvas, x - 3, foot - 2, ramp.light);
    put(canvas, x + 3, foot - 2, ramp.base);
    put(canvas, x - 4, foot - 1, ramp.light);
    put(canvas, x + 4, foot - 1, ramp.shadow);
    for dx in -3..=3 {
        let color = match dx {
            -3 => ramp.base,
            3 => ramp.edge,
            _ => ramp.shadow,
        };
        put(canvas, x + dx, foot, color);
    }
}

/// The painted blocks the prizes stand on: blue, red and gold, nearest the line first.
const BLOCK_PAINT: [Ramp; 3] = [
    HOOP_BLUE,
    SATIN,
    Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2),
];

/// The hoopla counter's top, set out: the rings stacked at the end nearest the line, a peg either
/// side of each prize, each prize on its block (`on_show`, unless it has just been carried off),
/// shining a moment if it has just been rung, and the rings round the blocks or lying on the
/// counter where they came down. Everything stands on the counter's top row.
pub fn hoopla_counter(
    on_show: [bool; 3],
    shine: [Option<f32>; 3],
    round: &[(usize, Hoop)],
    lying: &[((i32, i32), Hoop)],
    stacked: usize,
    now: f32,
    still: bool,
) -> Prop {
    use super::hoopla::{BLOCKS, COUNTER_TOP, PEGS, STACK};
    use super::prizes::{Prize, standing};
    use super::scenery::COUNTER;
    let foot = COUNTER_TOP;
    let area = (COUNTER.0, foot - 28, 66, 30);
    piece(COUNTER.1 as f32 + 16.5, area, |canvas, (left, top)| {
        let at = |(x, y): (i32, i32)| (x - left, y - top);
        // The rings waiting, stacked flat at the end, as many as there are.
        for (ring, hoop) in Hoop::ALL.iter().take(stacked).enumerate() {
            let (x, y) = at((STACK.0 - 3, STACK.1 - ring as i32));
            let ramp = hoop.ramp();
            for dx in 0..8 {
                let color = match dx {
                    0 | 7 => ramp.edge,
                    1 => ramp.shine,
                    2..=4 => ramp.light,
                    _ => ramp.base,
                };
                put(canvas, x + dx, y, color);
            }
        }
        for &x in &PEGS {
            peg(canvas, at((x, foot)));
        }
        for (block, &x) in BLOCKS.iter().enumerate() {
            let (bx, by) = at((x, foot));
            if on_show[block] {
                standing(canvas, Prize::ALL[block], (bx, by - 4), now, still);
                if let Some(since) = shine[block] {
                    sparkle(canvas, (bx, by - 14), since, still);
                }
            }
            crate::paint::bevel(canvas, bx - 2, by - 3, 5, 4, BLOCK_PAINT[block]);
            for (_, hoop) in round.iter().filter(|(on, _)| *on == block) {
                hoop_round_block(canvas, (bx, by), *hoop);
            }
        }
        for &(spot, hoop) in lying {
            let (x, y) = at(spot);
            hoop_flat(canvas, (x, y - 1), hoop);
        }
    })
}

/// A turned hickory peg standing on `(x, foot)`, its knob at the top, lit from the left.
fn peg(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    use super::hoopla::PEG_TALL;
    let top = foot - PEG_TALL;
    stamp(canvas, (x - 1, top), &[".W.", "LBS", ".E."], HICKORY);
    for y in top + 3..=foot {
        put(canvas, x - 1, y, HICKORY.light);
        put(canvas, x, y, HICKORY.shadow);
    }
    put(canvas, x - 1, foot, HICKORY.base);
    put(canvas, x, foot, HICKORY.edge);
}

/// A burst of sparkle about `(x, y)` for a prize just rung, `since` seconds ago: its rays
/// flickering, or steady with motion reduced.
fn sparkle(canvas: &mut Canvas, (x, y): (i32, i32), since: f32, still: bool) {
    let beat = if still { 0 } else { (since * 10.0) as i32 };
    for (ray, (dx, dy)) in [(-5, -2), (5, -3), (-4, 4), (6, 3), (0, -7), (-7, 1)]
        .into_iter()
        .enumerate()
    {
        if !still && (ray as i32 + beat) % 3 == 0 {
            continue;
        }
        put(canvas, x + dx, y + dy, rgb(0xffffff));
        put(canvas, x + dx + 1, y + dy, rgb(0xffe48a));
        put(canvas, x + dx - 1, y + dy, rgb(0xffe48a));
        put(canvas, x + dx, y + dy - 1, rgb(0xffe48a));
        put(canvas, x + dx, y + dy + 1, rgb(0xffe48a));
    }
}

/// A ring lying flat in the sawdust about `at`, under everyone, with a crumb of shade beside it.
pub fn hoop_lying(at: (i32, i32), hoop: Hoop) -> Prop {
    let area = (at.0 - 4, at.1 - 2, 9, 4);
    piece(f32::MIN, area, |canvas, (left, top)| {
        let (x, y) = (at.0 - left, at.1 - top);
        hline_shade(canvas, x - 2, y + 2, 6);
        hoop_flat(canvas, (x, y), hoop);
    })
}

/// A row of soft shade.
fn hline_shade(canvas: &mut Canvas, x: i32, y: i32, width: i32) {
    for dx in 0..width {
        put(canvas, x + dx, y, rgba(0x2a1e28, 60));
    }
}

/// The rings a thrower still holds, hanging from its paw at `grip`, in front of it as it stands
/// on row `feet`.
pub fn hoops_held(grip: (i32, i32), hoops: &[Hoop], feet: f32) -> Prop {
    let area = (grip.0 - 4, grip.1 - 1, 9, 9);
    piece(feet + 0.5, area, |canvas, (left, top)| {
        let (x, y) = (grip.0 - left, grip.1 - top);
        for (index, hoop) in hoops.iter().enumerate().rev() {
            let shift = index as i32;
            hoop_rows(
                canvas,
                (x - 2 + shift, y + 1 - shift / 2),
                &[".LWB.", "L...S", "L...S", "B...E", ".SSE."],
                *hoop,
            );
        }
    })
}

/// The ring in the air, if it is, at `at` and so far through its spin; and the arc it has flown,
/// in dots, fading towards where it was thrown from. Drawn over everyone.
pub fn hoop_flying(at: Option<((i32, i32), f32)>, hoop: Hoop, trail: &[(i32, i32)]) -> Prop {
    let points: Vec<(i32, i32)> = trail
        .iter()
        .copied()
        .chain(at.map(|(point, _)| point))
        .collect();
    let left = points.iter().map(|p| p.0).min().unwrap_or(0) - 5;
    let top = points.iter().map(|p| p.1).min().unwrap_or(0) - 5;
    let right = points.iter().map(|p| p.0).max().unwrap_or(0) + 5;
    let bottom = points.iter().map(|p| p.1).max().unwrap_or(0) + 5;
    piece(
        f32::MAX,
        (left, top, right - left + 1, bottom - top + 1),
        |canvas, (left, top)| {
            // Each dot of the arc, and the ring, with a crumb of shade under it, so they read
            // against the stall and the sawdust alike.
            let count = trail.len().max(1);
            for (index, &(x, y)) in trail.iter().enumerate() {
                if index % 2 == 1 {
                    continue;
                }
                let fade = 140 + (115 * index / count) as u8;
                put(canvas, x - left + 1, y - top + 1, rgba(0x2a1e28, fade / 2));
                put(canvas, x - left, y - top, rgba(0xfff4d8, fade));
            }
            if let Some(((x, y), spin)) = at {
                let mut ring = Canvas::new(canvas.width(), canvas.height());
                hoop_turning(&mut ring, (x - left, y - top), spin, hoop);
                for py in 0..ring.height() as i32 {
                    for px in 0..ring.width() as i32 {
                        if ring.get(px, py).a > 0 && ring.get(px + 1, py + 1).a == 0 {
                            put(canvas, px + 1, py + 1, rgba(0x2a1e28, 150));
                        }
                    }
                }
                crate::paint::blit(canvas, &ring, 0, 0);
            }
        },
    )
}

// ---------------------------------------------------------------------------------------------
// The tug-of-war
// ---------------------------------------------------------------------------------------------

/// Hemp for the rope, its strands twisted so they catch the light by turns; hay for tumbling
/// into, greener and duller than straw.
const HEMP: Ramp = Ramp::new(0x6a5030, 0x8a6c40, 0xb8955a, 0xd4b47a, 0xe8d2a0);
const HAY: Ramp = Ramp::new(0x6a5c26, 0x96883c, 0xbcac54, 0xd4c670, 0xeae09c);

/// Hay strewn either side of the line at `(x, y)`, `spread` along the rope each way: a loose,
/// low bed of it under the rope, deepest and heaped highest on the line and thinning out towards
/// its ends, its top lit and its straws streaked, ragged at its edges with wisps sticking out.
/// Flat on the ground, under everyone.
pub fn hay((x, y): (i32, i32), spread: i32) -> Prop {
    let area = (x - spread - 6, y - 10, spread * 2 + 13, 24);
    piece(f32::MIN, area, |canvas, (left, top)| {
        let at = |px: i32, py: i32| (px - left, py - top);
        // How far above and below the row the bed reaches at `dx` from the line: deep in the
        // middle, thin at the ends, ragged all along.
        let reach = |dx: i32| -> (i32, i32) {
            let out = (dx.abs() as f32 / spread.max(1) as f32).min(1.0);
            let ragged = (noise(x + dx, y, 955) % 3) as i32;
            let heap = (5.0 * (1.0 - out * out)).round() as i32;
            (
                2 + heap + ragged / 2,
                5 + (2.0 * (1.0 - out)).round() as i32 - ragged / 2,
            )
        };
        for dx in -spread..=spread {
            let (above, below) = reach(dx);
            for dy in -above..=below {
                let streak = noise((x + dx).div_euclid(2), y + dy, 957) % 8;
                let lit = dy - (-above);
                let tone = if lit == 0 {
                    if streak < 4 { HAY.light } else { HAY.shine }
                } else if dy >= below - 1 {
                    if streak < 5 { HAY.shadow } else { HAY.base }
                } else if lit <= 2 {
                    if streak < 2 { HAY.base } else { HAY.light }
                } else {
                    match streak {
                        0 => HAY.shadow,
                        1..=4 => HAY.base,
                        _ => HAY.light,
                    }
                };
                let (px, py) = at(x + dx, y + dy);
                put(canvas, px, py, tone);
            }
            // Its edge in its own darkest tone where it lies on the sawdust, and a crumb of shade.
            let (px, py) = at(x + dx, y + below + 1);
            put(canvas, px, py, mix(HAY.edge, HAY.shadow, 0.4));
            put(canvas, px, py + 1, rgba(0x2e2024, 46));
        }
        // Straws sticking out of it, and a few strayed further.
        for index in 0..spread * 2 {
            let along = (noise(index, 0, 953) % (spread as u32 * 2 + 9)) as i32 - spread - 4;
            let (above, below) = reach(along.clamp(-spread, spread));
            let wy = if index % 2 == 0 {
                y - above - 1
            } else {
                y + below + 2
            };
            let long = 2 + (noise(index, 3, 953) % 3) as i32;
            let lean = (noise(index, 4, 953) % 3) as i32 - 1;
            let (px, py) = at(x + along, wy);
            line(canvas, (px, py), (px + long, py + lean), HAY.base);
            put(canvas, px, py, HAY.light);
        }
    })
}

/// The rope, held: through the paws of everyone on it, `paws`, front ones nearest the line, taut
/// between them, its ends trailing to the ground beyond the last on each side; the ribbon tied on
/// at `ribbon`, hanging from it. Dropped (`lying`), it lies along row `ground` between `ends`, its
/// ribbon in the sawdust beside it. Drawn over everyone holding it, or under everyone, dropped.
pub fn rope(
    paws: &[((i32, i32), super::tug_of_war::Side)],
    ribbon: i32,
    ground: i32,
    lying: bool,
    ends: (i32, i32),
) -> Prop {
    use super::tug_of_war::Side;
    let area = (ends.0 - 6, ground - 48, ends.1 - ends.0 + 13, 58);
    let base = if lying { f32::MIN } else { ground as f32 + 5.5 };
    piece(base, area, |canvas, (left, top)| {
        let at = |(x, y): (i32, i32)| (x - left, y - top);
        // The rope's line, as points walked straight between.
        let mut points: Vec<(i32, i32)> = Vec::new();
        if lying {
            let reach = 54;
            for step in -reach..=reach {
                let wave = ((step as f32 * 0.21).sin() * 1.2).round() as i32;
                points.push((ribbon + step, ground + 2 + wave));
            }
        } else {
            let mut left_side: Vec<(i32, i32)> = paws
                .iter()
                .filter(|(_, side)| *side == Side::Left)
                .map(|(paw, _)| *paw)
                .collect();
            let mut right_side: Vec<(i32, i32)> = paws
                .iter()
                .filter(|(_, side)| *side == Side::Right)
                .map(|(paw, _)| *paw)
                .collect();
            left_side.sort_by_key(|paw| paw.0);
            right_side.sort_by_key(|paw| paw.0);
            // Trailing to the ground beyond the last on the left, and on to its end there.
            if let Some(&(x, _)) = left_side.first() {
                points.push((x - 16, ground + 1));
                points.push((x - 9, ground - 2));
            }
            points.extend(&left_side);
            // Across the middle, sagging a little to the ribbon.
            if let (Some(&from), Some(&to)) = (left_side.last(), right_side.first()) {
                let lowest = from.1.max(to.1) + 1;
                points.push((ribbon, lowest));
            }
            points.extend(&right_side);
            if let Some(&(x, _)) = right_side.last() {
                points.push((x + 9, ground - 2));
                points.push((x + 16, ground + 1));
            }
        }
        // Two pixels thick: the strands catching the light by turns along its top, its underside
        // in shade.
        let mut along = 0;
        for pair in points.windows(2) {
            let (a, b) = (at(pair[0]), at(pair[1]));
            let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
            for step in 0..steps {
                let t = step as f32 / steps as f32;
                let x = a.0 + ((b.0 - a.0) as f32 * t).round() as i32;
                let y = a.1 + ((b.1 - a.1) as f32 * t).round() as i32;
                let strand = along % 3;
                put(
                    canvas,
                    x,
                    y,
                    match strand {
                        0 => HEMP.light,
                        1 => HEMP.shine,
                        _ => HEMP.base,
                    },
                );
                put(
                    canvas,
                    x,
                    y + 1,
                    if strand == 2 { HEMP.edge } else { HEMP.shadow },
                );
                along += 1;
            }
        }
        // The ribbon tied round it at the middle: a knot, and its two tails hanging, or lying
        // flat beside it, dropped.
        let knot = points
            .iter()
            .min_by_key(|(x, _)| (x - ribbon).abs())
            .copied()
            .unwrap_or((ribbon, ground));
        let (kx, ky) = at(knot);
        let satin = SATIN;
        put(canvas, kx - 1, ky, satin.light);
        put(canvas, kx, ky, satin.base);
        put(canvas, kx - 1, ky + 1, satin.base);
        put(canvas, kx, ky + 1, satin.edge);
        let tails: [(i32, i32); 2] = if lying {
            [(-3, 2), (3, 2)]
        } else {
            [(-2, 6), (2, 5)]
        };
        for (dx, long) in tails {
            for step in 1..=long {
                let x = kx
                    + if lying {
                        dx * step / 3
                    } else {
                        dx * step / long.max(1)
                    };
                let y = ky + 1 + if lying { 1 } else { step };
                put(
                    canvas,
                    x,
                    y,
                    if step == long {
                        satin.edge
                    } else {
                        satin.light
                    },
                );
                put(canvas, x + 1, y, satin.shadow);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use formiga_art::{CreatureRenderer, FRAME_SIZE};
    use formiga_core::ActionKind;

    #[test]
    fn a_sack_covers_the_lower_body_of_every_companion_and_nothing_else() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for member in &cast.members {
            let frame = CreatureRenderer::render_dressed_body_frame(
                member.genome(),
                member.dress,
                ActionKind::Idle,
                0,
                false,
            )
            .canvas;
            let (_, top, _, bottom) = frame.alpha_bounds().unwrap();
            let girth = girth(&frame).unwrap();
            let sack = sack(&frame, 0, girth);
            let (sack_left, sack_top, sack_right, sack_bottom) = sack.alpha_bounds().unwrap();
            assert_eq!(sack_bottom, bottom, "{} stands on its sack", member.name);
            assert!(
                sack_top > top + (bottom - top) / 3,
                "{}'s sack comes up over its head",
                member.name
            );
            assert!(
                sack_right - sack_left >= 6,
                "{}'s sack is a sliver",
                member.name
            );
            assert!(sack.width() == FRAME_SIZE && sack.height() == FRAME_SIZE);
            // Never black: outlined in its own colour.
            assert!(
                sack.pixels()
                    .iter()
                    .filter(|pixel| pixel.a > 0)
                    .all(
                        |pixel| u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b) > 120
                    )
            );
        }
    }
}
