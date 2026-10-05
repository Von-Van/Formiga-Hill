//! The finds from afar (see `finds::afar`): their icons and their Hilltop pieces. Old things and
//! falls-water things, each made into something like a landmark: an arch, a bell under its own
//! roof, a grotto, a statue, a crystal on a post that throws a rainbow on the grass.

use super::Piece;
use super::brush::icon_from;
use super::brush::{ROUND, TUFT, in_ellipse, lump, model, moss, shadow, stamp, tuft, tuft_inks};
use super::water::{SLATE, sparkle};
use crate::paint::{Ramp, chance, hline, line, mix, put, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::PI;

/// Old dressed stone, gone grey-cream with the weather.
const OLD_STONE: Ramp = Ramp::new(0x6e6656, 0x948a76, 0xb6ac94, 0xd0c6ae, 0xe8e0ca);
/// Ivy, carved and growing.
const IVY: Ramp = Ramp::new(0x1c3a22, 0x2a5430, 0x3c7040, 0x568e52, 0x7aac6a);
/// Bronze gone green, and the bronze under it where it is rubbed.
const VERDIGRIS: Ramp = Ramp::new(0x22463e, 0x346656, 0x4c8a72, 0x6eaa8e, 0xa0d0b4);
const BRONZE: Ramp = Ramp::new(0x5a3a1a, 0x8a5a26, 0xb47c34, 0xd49c4e, 0xf0c47a);
const OAK: Ramp = Ramp::new(0x3a2618, 0x583a24, 0x765034, 0x926a48, 0xae8862);
const ROOF: Ramp = Ramp::new(0x4a2420, 0x6a3428, 0x8a4834, 0xa66244, 0xc28058);
const PEARL: Ramp = Ramp::new(0x9a8a96, 0xc8b8c2, 0xe6d8dc, 0xf4ecee, 0xffffff);
const SHELL: Ramp = Ramp::new(0x2a2236, 0x3e3450, 0x58506e, 0x7a7090, 0xa69cb8);
const NACRE: Ramp = Ramp::new(0x8a8aaa, 0xaeb0cc, 0xcdd0e4, 0xe6e8f4, 0xffffff);
const POOL: Ramp = Ramp::new(0x22404a, 0x33606a, 0x4a8290, 0x6aa2ae, 0x9cc8cf);
const HARE: Ramp = Ramp::new(0x5c5650, 0x7e776e, 0xa0988c, 0xbcb4a6, 0xd8d0c2);
/// The hollow of the hare's eye, deeper than its outline.
const HARE_EYE: Rgba = rgb(0x3a3430);
const CRYSTAL: Ramp = Ramp::new(0x6a8aa0, 0x9ab8cc, 0xc4dce8, 0xe2f0f6, 0xffffff);
const LICHEN: Rgba = rgb(0xc4c87a);
const ROPE: Rgba = rgb(0x9a8a6a);
/// The rainbow, outermost first.
const BANDS: [u32; 6] = [0xe06050, 0xf0a048, 0xf0e070, 0x80c870, 0x6090e0, 0x9070c8];

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    Some(match id {
        // A wedge of stone with an ivy leaf carved in it.
        "carved_keystone" => icon_from(
            [
                "#########",
                "#*llllos#",
                ".#lgGlo#.",
                ".#lGgos#.",
                "..#lgo#..",
                "..#los#..",
                "..#oss#..",
                "...#s#...",
                "...###...",
            ],
            OLD_STONE,
            &[('g', IVY.base), ('G', IVY.light)],
        ),
        // A little bell, green with age, its mouth dark and its clapper hanging.
        "bronze_bell" => icon_from(
            [
                "...#.#...",
                "....#....",
                "...#l#...",
                "..#llo#..",
                "..#loo#..",
                ".#l*oos#.",
                ".#loooS#.",
                "#########",
                "....c....",
            ],
            VERDIGRIS,
            &[('S', VERDIGRIS.shadow), ('c', BRONZE.base)],
        ),
        // A pearl, pink-white and shining, in its open shell.
        "falls_pearl" => {
            let mut icon = icon_from(
                [
                    ".........",
                    "...###...",
                    "..#*pp#..",
                    "..#pPP#..",
                    "..#PPq#..",
                    ".#n###n#.",
                    "#nNNNNNs#",
                    ".#sssss#.",
                    "..#####..",
                ],
                SHELL,
                &[
                    ('p', PEARL.light),
                    ('P', PEARL.base),
                    ('q', PEARL.shadow),
                    ('n', NACRE.light),
                    ('N', NACRE.base),
                ],
            );
            icon.set(3, 2, PEARL.shine);
            icon.set(4, 2, PEARL.shine);
            icon.set(3, 1, PEARL.edge);
            icon.set(4, 1, PEARL.edge);
            icon.set(5, 1, PEARL.edge);
            icon.set(2, 2, PEARL.edge);
            icon.set(6, 2, PEARL.edge);
            icon.set(2, 3, PEARL.edge);
            icon.set(6, 3, PEARL.edge);
            icon.set(2, 4, PEARL.edge);
            icon.set(6, 4, PEARL.edge);
            icon
        }
        // A stone hare sitting up, facing left, its ears laid back and its nose rubbed bright.
        "stone_hare" => icon_from(
            [
                "......##.",
                "....##l*#",
                "..###ll#.",
                ".#l*e##..",
                "#*lloo#..",
                ".##loo##.",
                "..#looos#",
                "..#lsoss#",
                ".########",
            ],
            HARE,
            &[('e', HARE_EYE)],
        ),
        // A clear crystal, a rainbow caught in it.
        "rainbow_prism" => {
            let mut icon = icon_from(
                [
                    "....#....",
                    "...#*#...",
                    "..#*ll#..",
                    ".#*llol#.",
                    "#llloooo#",
                    ".#loooo#.",
                    "..#oos#..",
                    "...#s#...",
                    "....#....",
                ],
                CRYSTAL,
                &[],
            );
            for (index, band) in BANDS.iter().enumerate() {
                let x = 2 + index as i32;
                if x < 7 {
                    icon.set(x, 5, rgb(*band));
                }
            }
            icon
        }
        _ => return None,
    })
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "carved_keystone" => old_arch(),
        "bronze_bell" => bell_post(),
        "falls_pearl" => pearl_grotto(),
        "stone_hare" => hare_stone(),
        "rainbow_prism" => rainbow_prism(),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------------------------

/// A block of old dressed stone, lit from the upper left and weathered, lichen in its corners.
fn block(s: &mut Canvas, (x, y): (i32, i32), (width, height): (i32, i32), salt: u32) {
    for py in y..y + height {
        for px in x..x + width {
            let (col, row) = (px - x, py - y);
            let mut color = if col == 0 || col == width - 1 || row == height - 1 || row == 0 {
                OLD_STONE.edge
            } else if row == 1 || col == 1 {
                OLD_STONE.light
            } else if col >= width - 2 || row == height - 2 {
                OLD_STONE.shadow
            } else {
                OLD_STONE.base
            };
            if col > 1 && col < width - 2 && row > 1 && row < height - 2 {
                if chance(px, py, salt, 26) {
                    color = mix(color, OLD_STONE.edge, 0.35);
                } else if chance(px, py, salt + 1, 14) {
                    color = LICHEN;
                }
            }
            put(s, px, py, color);
        }
    }
    put(s, x + 1, y + 1, OLD_STONE.shine);
}

/// A strand of ivy climbing from `from` to `to`, its leaves in pairs, lit on their upper left.
fn ivy(s: &mut Canvas, from: (i32, i32), to: (i32, i32), salt: u32) {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let wave = ((t * 9.0 + salt as f32).sin() * 1.5).round() as i32;
        let (x, y) = (
            from.0 + ((to.0 - from.0) as f32 * t).round() as i32 + wave,
            from.1 + ((to.1 - from.1) as f32 * t).round() as i32,
        );
        put(s, x, y, IVY.shadow);
        if step % 3 == 0 {
            let side = if (step / 3) % 2 == 0 { -1 } else { 1 };
            put(s, x + side, y, IVY.light);
            put(s, x + side * 2, y, IVY.base);
            put(s, x + side, y + 1, IVY.edge);
            if chance(x, y, salt, 120) {
                put(s, x + side, y - 1, IVY.shine);
            }
        }
    }
}

/// The old arch: a little ruin of an arch on two piers of old stone, the carved keystone at its
/// crown, ivy up one side and moss along its top, grass and flowers at its feet.
fn old_arch() -> Piece {
    let mut s = Canvas::new(44, 50);
    let ground = 47;
    let (cx, spring) = (22, 26);
    shadow(&mut s, cx + 1, ground, 20, 3);
    // The piers, in courses.
    for (left, salt) in [(4, 3101), (31, 3102)] {
        let mut y = ground - 1;
        let mut course = 0;
        while y > spring {
            let tall = 5 + (course % 2);
            let wide = 9 - (course % 2);
            let shift = if course % 2 == 0 { 0 } else { 1 };
            block(
                &mut s,
                (left + shift, y - tall + 1),
                (wide, tall),
                salt + course as u32,
            );
            y -= tall;
            course += 1;
        }
    }
    // The ring of voussoirs round the opening, each a wedge lit along its upper left edge.
    let (inner, outer) = (9.5f32, 17.0f32);
    for y in 4..=spring + 1 {
        for x in 4..40 {
            let (dx, dy) = (x as f32 + 0.5 - cx as f32, spring as f32 - (y as f32 + 0.5));
            let reach = (dx * dx + dy * dy).sqrt();
            if dy < -1.0 || reach < inner || reach > outer {
                continue;
            }
            let around = dy.atan2(dx);
            let wedge = (around / PI * 9.0).floor() as i32;
            let within = (around / PI * 9.0).fract();
            let keystone = wedge == 4;
            let joint = !(0.1..0.9).contains(&within);
            let rim = reach > outer - 1.0 || reach < inner + 1.0;
            let color = if joint || rim {
                OLD_STONE.edge
            } else if keystone {
                // The keystone, a little paler and proud of the rest.
                if reach > outer - 3.0 {
                    OLD_STONE.shine
                } else {
                    OLD_STONE.light
                }
            } else if within < 0.32 {
                OLD_STONE.light
            } else if dx > 4.0 {
                OLD_STONE.shadow
            } else {
                OLD_STONE.base
            };
            put(&mut s, x, y, color);
        }
    }
    // The ivy leaf carved in the keystone.
    stamp(
        &mut s,
        (cx - 2, 7),
        &[".gG.", "gGGg", ".gg.", "..g."],
        &[(b'g', IVY.shadow), (b'G', IVY.light)],
        false,
    );
    // The opening's shade on the inside of the piers.
    for y in spring..ground - 1 {
        put(&mut s, 13, y, mix(OLD_STONE.shadow, OLD_STONE.edge, 0.5));
        put(&mut s, 30, y, OLD_STONE.edge);
    }
    // Moss along its top and ivy up the left pier and over the shoulder.
    moss(&mut s, cx as f32 - 6.0, 6.0, 6.0, 2.0, 3103);
    moss(&mut s, 7.0, 27.0, 3.0, 1.5, 3104);
    ivy(&mut s, (6, ground - 1), (9, 18), 3105);
    ivy(&mut s, (9, 18), (17, 8), 3106);
    // Grass and flowers at its feet.
    stamp(&mut s, (2, ground - 3), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (36, ground - 3), &TUFT, &tuft_inks(), true);
    stamp(&mut s, (19, ground - 2), &TUFT, &tuft_inks(), false);
    for (x, color) in [(15, 0xf19bb0), (28, 0xfbf6ee), (25, 0xf5d25e)] {
        put(&mut s, x, ground - 3, rgb(color));
        put(&mut s, x, ground - 2, IVY.base);
    }
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// The bell post: an oak frame under a little shingled roof, the bronze bell hung from its beam
/// with a rope down from it to ring it by.
fn bell_post() -> Piece {
    let mut s = Canvas::new(32, 50);
    let ground = 47;
    shadow(&mut s, 16, ground, 13, 2);
    // The posts, braced.
    for x in [6, 23] {
        for dx in 0..3 {
            let color = match dx {
                0 => OAK.light,
                1 => OAK.base,
                _ => OAK.shadow,
            };
            vline(&mut s, x + dx, 12, ground - 12, color);
        }
        vline(&mut s, x - 1, 12, ground - 12, OAK.edge);
        vline(&mut s, x + 3, 12, ground - 12, OAK.edge);
    }
    line(&mut s, (9, 22), (14, 15), OAK.base);
    line(&mut s, (9, 23), (14, 16), OAK.edge);
    line(&mut s, (22, 22), (17, 15), OAK.base);
    line(&mut s, (22, 23), (17, 16), OAK.edge);
    // The beam.
    for x in 4..28 {
        put(&mut s, x, 12, OAK.edge);
        put(&mut s, x, 13, OAK.light);
        put(&mut s, x, 14, OAK.base);
        put(&mut s, x, 15, OAK.edge);
    }
    // The roof: two pitches of shingles, lit on the left.
    for y in 1..12 {
        let half = 3 + (y - 1) * 14 / 10;
        for x in 16 - half..=16 + half {
            let left = x < 16;
            let course = (y + if (x / 3) % 2 == 0 { 0 } else { 1 }) % 3 == 0;
            let color = if x == 16 - half || x == 16 + half || y == 11 {
                ROOF.edge
            } else if course {
                ROOF.shadow
            } else if left {
                ROOF.light
            } else {
                ROOF.base
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, 16, 0, ROOF.edge);
    put(&mut s, 15, 1, ROOF.shine);
    // The bell, hung from the beam: crown, waist and a flared lip, green with the rubbed bronze
    // showing through, its mouth dark, the clapper within.
    let (bx, top) = (16.0f32, 17.0f32);
    for y in 16..32 {
        for x in 6..27 {
            let depth = y as f32 + 0.5 - top;
            if depth < 0.0 {
                continue;
            }
            let half = if depth < 2.0 {
                2.5
            } else {
                3.0 + depth * 0.32 + (depth - 9.0).max(0.0) * 0.9
            };
            let dx = x as f32 + 0.5 - bx;
            if dx.abs() > half || depth > 13.0 {
                continue;
            }
            let k = dx / half;
            let color = if dx.abs() > half - 0.9 || depth > 12.0 {
                VERDIGRIS.edge
            } else if depth > 11.0 {
                VERDIGRIS.shadow
            } else if k < -0.5 {
                VERDIGRIS.light
            } else if k < -0.25 && depth > 3.0 {
                BRONZE.light
            } else if k > 0.45 {
                VERDIGRIS.shadow
            } else {
                VERDIGRIS.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 11, 29, 11, rgb(0x14201c));
    put(&mut s, 16, 30, BRONZE.shadow);
    put(&mut s, 16, 31, BRONZE.base);
    put(&mut s, 12, 21, VERDIGRIS.shine);
    // The rope, down to where anyone can reach it, knotted at the end.
    vline(&mut s, 20, 30, 10, ROPE);
    put(&mut s, 20, 40, mix(ROPE, OAK.edge, 0.4));
    put(&mut s, 19, 40, ROPE);
    put(&mut s, 21, 40, ROPE);
    stamp(&mut s, (3, ground - 3), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (23, ground - 3), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (16, ground),
    }
}

/// The pearl grotto: mossy rocks heaped into a little grotto with a pool inside, where the
/// pearl lies shining in its open shell, ferns round the mouth.
fn pearl_grotto() -> Piece {
    let mut s = Canvas::new(44, 40);
    let ground = 37;
    shadow(&mut s, 22, ground, 20, 3);
    // The dark of the grotto behind.
    for y in 10..ground {
        for x in 8..36 {
            if in_ellipse(x, y, (22, ground), (13, 25)) {
                put(&mut s, x, y, rgb(0x1a1c22));
            }
        }
    }
    // The pool inside, catching the light.
    for y in ground - 7..ground {
        for x in 9..36 {
            if in_ellipse(x, y, (22, ground - 3), (12, 4)) {
                let color = if !in_ellipse(x, y, (22, ground - 3), (11, 3)) {
                    POOL.edge
                } else if x < 19 {
                    POOL.light
                } else {
                    POOL.base
                };
                put(&mut s, x, y, color);
            }
        }
    }
    // The shell, open, nacre within, and the pearl on it.
    for (x, y, color) in [
        (17, 31, SHELL.edge),
        (18, 32, SHELL.base),
        (19, 32, SHELL.base),
        (20, 32, SHELL.base),
        (21, 32, SHELL.base),
        (22, 32, SHELL.base),
        (23, 32, SHELL.base),
        (24, 32, SHELL.shadow),
        (25, 31, SHELL.edge),
    ] {
        put(&mut s, x, y, color);
    }
    hline(&mut s, 18, 31, 7, NACRE.light);
    for y in 27..31 {
        for x in 19..24 {
            if in_ellipse(x, y, (21, 29), (2, 2)) {
                let lit = (x - 21) + (y - 29);
                let color = if lit < -1 {
                    PEARL.shine
                } else if lit < 1 {
                    PEARL.light
                } else {
                    PEARL.shadow
                };
                put(&mut s, x, y, color);
            }
        }
    }
    put(&mut s, 20, 28, rgb(0xffffff));
    // Its soft light on the water and the rock about it.
    for y in 22..ground {
        for x in 12..32 {
            let d = (((x - 21) * (x - 21) + (y - 29) * (y - 29)) as f32).sqrt();
            if d < 10.0 && s.get(x, y).a == 255 && !(19..24).contains(&x) {
                let pixel = s.get(x, y);
                put(
                    &mut s,
                    x,
                    y,
                    mix(pixel, rgb(0xf4e0e8), (1.0 - d / 10.0) * 0.25),
                );
            }
        }
    }
    // The rocks heaped round, the biggest over the top: lumps of old grey stone, mossy.
    for &((x, y), (rx, ry), salt) in &[
        ((8.0, 30.0), (7.5, 6.5), 3201),
        ((35.0, 31.0), (8.0, 6.0), 3202),
        ((10.0, 19.0), (6.5, 6.0), 3203),
        ((34.0, 19.0), (7.0, 6.5), 3204),
        ((22.0, 10.0), (12.0, 6.5), 3205),
        ((14.0, 9.0), (5.0, 4.0), 3206),
        ((30.0, 8.0), (6.0, 4.5), 3207),
    ] {
        lump(&mut s, x, y, rx, ry, SLATE, salt);
        moss(&mut s, x - 1.0, y - ry * 0.6, rx * 0.7, 1.6, salt + 20);
    }
    // Ferns at its mouth.
    for (root, tips) in [
        ((5, ground), [(-4, -9), (0, -11), (3, -8)]),
        ((39, ground), [(4, -9), (0, -11), (-3, -8)]),
    ] {
        for tip in tips {
            line(&mut s, root, (root.0 + tip.0, root.1 + tip.1), IVY.base);
            put(&mut s, root.0 + tip.0, root.1 + tip.1, IVY.light);
        }
    }
    sparkle(&mut s, (16, 25), 1);
    sparkle(&mut s, (27, 24), 1);
    Piece {
        sprite: s,
        anchor: (22, ground),
    }
}

/// The hare stone: a statue of a hare sitting up on a mossy plinth, in profile and facing left,
/// its ears laid back along its shoulders, worn soft by the water, with lichen on it and its
/// nose rubbed bright for luck.
fn hare_stone() -> Piece {
    let mut s = Canvas::new(34, 52);
    let ground = 49;
    shadow(&mut s, 17, ground, 14, 3);
    // The plinth.
    block(&mut s, (4, ground - 9), (26, 9), 3301);
    block(&mut s, (7, ground - 13), (20, 5), 3302);
    moss(&mut s, 10.0, ground as f32 - 9.5, 5.0, 1.5, 3303);
    // The hare, each part of it a rounded shape placed from the front of the plinth's top: its
    // head and muzzle, the two ears, the body rising from a big haunch, the chest, a foreleg
    // straight down to the paw, the long hind foot along the plinth and the tail.
    let base = ground - 13;
    let at = |x: f32, y: f32| (4.0 + x, base as f32 + y);
    let head =
        |x, y| oval(x, y, at(8.5, -24.0), (5.4, 4.6)) || oval(x, y, at(4.1, -22.6), (3.0, 2.6));
    let far_ear = |x, y| limb(x, y, at(10.5, -27.5), at(21.0, -35.5), (2.7, 1.4));
    let near_ear = |x, y| limb(x, y, at(11.5, -26.0), at(23.5, -32.0), (3.0, 1.5));
    let body = |x, y| oval(x, y, at(13.0, -13.0), (6.5, 9.0)) && y < base;
    let chest = |x, y| oval(x, y, at(8.0, -16.0), (4.2, 6.0));
    let haunch = |x, y| oval(x, y, at(16.0, -7.0), (7.6, 7.2)) && y < base;
    let foreleg = |x: i32, y: i32| {
        let (left, top) = at(5.0, -12.0);
        (left..=left + 3.2).contains(&(x as f32 + 0.5)) && y as f32 >= top && y < base
    };
    let feet = |x, y| {
        (oval(x, y, at(5.0, -1.0), (3.0, 1.6)) || oval(x, y, at(11.0, -1.0), (8.5, 1.7)))
            && y < base
    };
    let tail = |x, y| oval(x, y, at(23.5, -8.0), (1.8, 2.0));
    let solid = |x, y| {
        head(x, y)
            || far_ear(x, y)
            || near_ear(x, y)
            || body(x, y)
            || chest(x, y)
            || haunch(x, y)
            || foreleg(x, y)
            || feet(x, y)
            || tail(x, y)
    };
    model(&mut s, HARE, ROUND, 20, 3304, solid);
    // Where one part meets another the carving goes in: the front of the haunch, the back of
    // the foreleg, under the jaw, the far ear behind the near, and the near ear's hollow.
    let (front, _) = at(0.0, 0.0);
    for y in 0..base {
        for x in 0..34 {
            if !solid(x, y) || s.get(x, y) == HARE.edge {
                continue;
            }
            let groove = (haunch(x, y) && !haunch(x - 1, y) && y > base - 13)
                || (foreleg(x, y) && !foreleg(x + 1, y) && y > base - 10)
                || (head(x, y) && !head(x, y + 1) && x as f32 > front + 6.0)
                || (near_ear(x, y)
                    && near_ear(x, y - 2)
                    && near_ear(x, y + 2)
                    && x as f32 > front + 13.0
                    && !head(x, y));
            if far_ear(x, y) && !near_ear(x, y) && near_ear(x, y + 1) {
                put(&mut s, x, y, HARE.edge);
            } else if groove {
                put(&mut s, x, y, HARE.shadow);
            } else if y > base - 16 && chance(x, y, 3305, 9) {
                // Lichen, on its back and haunch.
                put(&mut s, x, y, LICHEN);
            }
        }
    }
    // Its eye, a little hollow, and its nose rubbed bright.
    let (eye, nose) = (at(6.0, -25.0), at(1.0, -22.0));
    put(&mut s, eye.0 as i32, eye.1 as i32, HARE_EYE);
    put(&mut s, eye.0 as i32 + 1, eye.1 as i32, HARE.edge);
    put(&mut s, eye.0 as i32, eye.1 as i32 - 1, HARE.shadow);
    put(&mut s, nose.0 as i32, nose.1 as i32, HARE.shine);
    stamp(&mut s, (1, ground - 3), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (28, ground - 3), &TUFT, &tuft_inks(), true);
    for (x, color) in [(3, 0xfbf6ee), (31, 0xf19bb0)] {
        put(&mut s, x, ground - 4, rgb(color));
    }
    Piece {
        sprite: s,
        anchor: (17, ground),
    }
}

/// Whether a pixel's middle is inside the ellipse about `centre`, measured in fractions of a
/// pixel so a shape can sit between pixels.
fn oval(x: i32, y: i32, centre: (f32, f32), (rx, ry): (f32, f32)) -> bool {
    let (dx, dy) = (
        (x as f32 + 0.5 - centre.0) / rx,
        (y as f32 + 0.5 - centre.1) / ry,
    );
    dx * dx + dy * dy <= 1.0
}

/// Whether a pixel's middle is in a limb running from `root` to `tip`, rounded at both ends and
/// narrowing from one thickness to the other along it: an ear, say.
fn limb(x: i32, y: i32, root: (f32, f32), tip: (f32, f32), (thick, thin): (f32, f32)) -> bool {
    let (dx, dy) = (tip.0 - root.0, tip.1 - root.1);
    let (px, py) = (x as f32 + 0.5 - root.0, y as f32 + 0.5 - root.1);
    let t = ((px * dx + py * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    let (ox, oy) = (px - t * dx, py - t * dy);
    (ox * ox + oy * oy).sqrt() <= thick + (thin - thick) * t
}

/// The rainbow prism: the clear crystal set on a carved post, throwing a rainbow in an arc onto
/// the grass beside it, sparkling.
fn rainbow_prism() -> Piece {
    let mut s = Canvas::new(46, 54);
    let ground = 51;
    shadow(&mut s, 14, ground, 10, 2);
    // The rainbow it throws, arching onto the grass on the right.
    let (cx, cy) = (30.0f32, ground as f32 + 2.0);
    for (index, band) in BANDS.iter().enumerate() {
        let radius = 14.0 - index as f32 * 1.4;
        for step in 0..120 {
            let angle = PI + step as f32 / 120.0 * PI;
            let (x, y) = (cx + angle.cos() * radius, cy + angle.sin() * radius * 1.2);
            let fade = (step as f32 / 120.0 * PI).sin();
            put(
                &mut s,
                x as i32,
                y as i32,
                rgba(*band, (60.0 + fade * 110.0) as u8),
            );
        }
    }
    // The post: carved oak, with a band of rings round it and a cap.
    for y in 22..ground {
        for x in 10..19 {
            let across = (x - 10) as f32 / 8.0;
            let ring = (y - 22) % 7 == 0;
            let color = if x == 10 || x == 18 {
                OAK.edge
            } else if ring {
                OAK.shadow
            } else if across < 0.3 {
                OAK.light
            } else if across < 0.65 {
                OAK.base
            } else {
                OAK.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 8, 21, 13, OAK.edge);
    hline(&mut s, 8, 22, 13, OAK.light);
    hline(&mut s, 9, 23, 11, OAK.shadow);
    // The crystal on top: a long prism of clear stone, faceted, lit from the upper left.
    let facets = [(14, 4), (19, 9), (19, 17), (14, 21), (9, 17), (9, 9)];
    for y in 3..22 {
        for x in 8..21 {
            let inside = point_in(&facets, (x, y));
            if !inside {
                continue;
            }
            let edge = !point_in(&facets, (x - 1, y))
                || !point_in(&facets, (x + 1, y))
                || !point_in(&facets, (x, y - 1))
                || !point_in(&facets, (x, y + 1));
            let color = if edge {
                CRYSTAL.edge
            } else if x < 12 && y < 15 {
                CRYSTAL.shine
            } else if x < 14 {
                CRYSTAL.light
            } else if x < 17 {
                CRYSTAL.base
            } else {
                CRYSTAL.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    // A facet line down its middle, and the rainbow caught inside it.
    vline(&mut s, 14, 6, 14, CRYSTAL.base);
    for (index, band) in BANDS.iter().enumerate() {
        put(&mut s, 15, 11 + index as i32, rgb(*band));
        put(&mut s, 16, 12 + index as i32, rgba(*band, 160));
    }
    // Light thrown off it.
    for (at, size) in [((6, 6), 2), ((22, 4), 1), ((24, 14), 2), ((5, 16), 1)] {
        sparkle(&mut s, at, size);
    }
    stamp(&mut s, (6, ground - 3), &TUFT, &tuft_inks(), false);
    tuft(&mut s, 18, ground, 2, 3401);
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// Whether a pixel's middle is inside a convex outline.
fn point_in(outline: &[(i32, i32)], (x, y): (i32, i32)) -> bool {
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    let mut sign = 0.0f32;
    for (index, &(ax, ay)) in outline.iter().enumerate() {
        let (bx, by) = outline[(index + 1) % outline.len()];
        let cross = (bx - ax) as f32 * (py - ay as f32) - (by - ay) as f32 * (px - ax as f32);
        if cross.abs() < f32::EPSILON {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::AFAR;
    use crate::finds::art::{ICON, PIECE_MAX};
    use crate::hilltop::{self, Arrangement, Standing};

    #[test]
    fn every_find_from_afar_is_drawn_and_fits_any_spot() {
        for find in &AFAR {
            let drawn = icon(find.id).unwrap_or_else(|| panic!("{} has no icon", find.id));
            assert_eq!((drawn.width(), drawn.height()), (ICON, ICON));
            let inked = drawn.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(inked >= 12, "{}'s icon is barely there", find.id);
            let piece = piece(find.id).unwrap_or_else(|| panic!("{} has no piece", find.id));
            let (width, height) = (piece.sprite.width(), piece.sprite.height());
            assert!(
                width <= PIECE_MAX.0 && height <= PIECE_MAX.1,
                "{} is {width}x{height}",
                find.id
            );
            let (x, y) = piece.anchor;
            assert!(x >= 0 && y >= 0 && x < width as i32 && y < height as i32);
            // Big enough to be a landmark, as anything from so far should be.
            let (left, top, right, bottom) = piece.sprite.alpha_bounds().unwrap();
            assert!(
                (right - left + 1) * (bottom - top + 1) >= 600,
                "{} is too slight for a landmark",
                find.id
            );
            for spot in 0..hilltop::SPOTS.len() as u8 {
                let arrangement = Arrangement::from([(spot, Standing::from(find.id))]);
                let props = hilltop::props(&arrangement);
                assert_eq!(props.len(), 1);
                let (left, top, right, bottom) = props[0].bounds();
                assert!(
                    left >= -4 && right < 388 && top >= -8 && bottom < 216,
                    "{} spills off spot {spot}",
                    find.id
                );
            }
        }
    }
}
