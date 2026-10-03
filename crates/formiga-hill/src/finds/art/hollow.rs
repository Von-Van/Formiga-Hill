//! The finds in a hollow: the finds reached for (Kind::Reach): their icons and their Hilltop
//! pieces.

use super::brush::{icon_from, lit, lump, moss, mound, paint_rows, shadow, tuft};
use super::{ICON, Piece};
use crate::materials::{GLASS, GLOW, STONE};
use crate::paint::{Ramp, chance, hline, line, mix, noise, put, rect, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};

pub(super) const CONE: Ramp = Ramp::new(0x4a2c1a, 0x6e4426, 0x93602f, 0xb57d42, 0xd49e5e);
const OAK_LEAF: Ramp = Ramp::new(0x2e5426, 0x3e7230, 0x56923e, 0x76b250, 0x9ccc6c);
pub(super) const BARK: Ramp = Ramp::new(0x3a2a20, 0x5a4232, 0x76583e, 0x927252, 0xae8e6a);
const ACORN: Ramp = Ramp::new(0x5a3a1e, 0x80562a, 0xa6743a, 0xc4924e, 0xdeb46e);
const CUP: Ramp = Ramp::new(0x3e3020, 0x5a4630, 0x786040, 0x947a54, 0xb09a70);
const JAY: Ramp = Ramp::new(0x1e3a6e, 0x2c5ea8, 0x3a86d0, 0x6ab0e8, 0xa8dcf8);
const NAVY: Rgba = rgb(0x22264a);
const QUILL: Ramp = Ramp::new(0x6a5a48, 0x9a8a72, 0xc8b89a, 0xe2d6bc, 0xf6eedc);
pub(super) const POLE: Ramp = Ramp::new(0x3e2a20, 0x5a3e2e, 0x76543c, 0x92704e, 0xae8c66);
const BOX: Ramp = Ramp::new(0x6a4a30, 0x8e6a44, 0xb08a5a, 0xc8a472, 0xdcbe8c);
pub(super) const NEST: Ramp = Ramp::new(0x4a3420, 0x6e5034, 0x92704a, 0xb08e62, 0xcaa87c);
pub(super) const MOSSY: Ramp = Ramp::new(0x34512a, 0x4a6e34, 0x638c40, 0x80aa52, 0xa4c872);
pub(super) const IRON: Ramp = Ramp::new(0x38302e, 0x504644, 0x6a5e5a, 0x887a74, 0xa89a92);
const BRASS: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const LEATHER: Ramp = Ramp::new(0x3e2419, 0x5e3624, 0x7b4a31, 0x96603f, 0xb27b52);
pub(super) const VELVET: Ramp = Ramp::new(0x173c40, 0x22585c, 0x2f7a7a, 0x45999a, 0x6ab8b4);
pub(super) const LACQUER: Ramp = Ramp::new(0x5e1a24, 0x8a2a30, 0xb83a3c, 0xd0584c, 0xe87a64);
pub(super) const GILT: Ramp = Ramp::new(0x7a5420, 0xa87c2c, 0xd4a842, 0xecc864, 0xfaeaa8);
const RINGS: Ramp = Ramp::new(0x7a5a3a, 0xa07a50, 0xc09a68, 0xd6b484, 0xe8cca0);
const TOADSTOOL: Ramp = Ramp::new(0x6e2a22, 0x9a3a2e, 0xc4503c, 0xdc6e52, 0xf09a7a);
const CANDLE: Rgba = rgb(0xf3e9cf);
/// The red a young oak's shoot comes up.
const SHOOT: Rgba = rgb(0x9a5a3a);
pub(super) const PLUM: Ramp = Ramp::new(0x3e2448, 0x56325e, 0x6e4278, 0x8a5a94, 0xb088b8);

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    Some(match id {
        // A pinecone, closed up tight, its scales in rows.
        "pinecone" => icon_from(
            [
                "....#....",
                "...#l#...",
                "..#lso#..",
                ".#*osls#.",
                ".#lsoso#.",
                ".#olsos#.",
                ".#sosss#.",
                "..#sss#..",
                "...###...",
            ],
            CONE,
            &[],
        ),
        // Two acorns in their cups.
        "acorn_stash" => icon_from(
            [
                "......k..",
                ".....kcCk",
                ".k...kCdk",
                "kcCk.#*o#",
                "kCdk.#lo#",
                "#*o#.#os#",
                "#lo#..##.",
                "#os#.....",
                ".##......",
            ],
            ACORN,
            &[
                ('k', CUP.edge),
                ('c', CUP.light),
                ('C', CUP.base),
                ('d', CUP.shadow),
            ],
        ),
        // A jay's wing feather, barred blue and navy.
        "jay_feather" => icon_from(
            [
                "......##.",
                ".....#*l#",
                "....#lon#",
                "...#lno#.",
                "..#onl#..",
                "..#no#...",
                ".#lo#....",
                ".q##.....",
                "q........",
            ],
            JAY,
            &[('n', NAVY), ('q', QUILL.shadow)],
        ),
        // A little bowl of a nest, woven tight, moss inside.
        "old_nest" => icon_from(
            [
                ".........",
                "...k.....",
                "..#l###..",
                ".#lmMmo#.",
                "#lmddmmo#",
                "#looooos#",
                "#lososos#",
                ".#ossss#.",
                "..#####..",
            ],
            NEST,
            &[
                ('k', NEST.shadow),
                ('m', MOSSY.base),
                ('M', MOSSY.light),
                ('d', MOSSY.edge),
            ],
        ),
        // A small lantern, a stub of candle still alight in it.
        "lost_lantern" => icon_from(
            [
                "....l....",
                "...#s#...",
                "..#lls#..",
                ".#######.",
                ".#yGfGy#.",
                ".#yGcGy#.",
                ".#yGcGy#.",
                ".#######.",
                "..#sss#..",
            ],
            IRON,
            &[
                ('y', GLOW[0]),
                ('G', GLOW[1]),
                ('f', rgb(0xffffff)),
                ('c', CANDLE),
            ],
        ),
        "brass_lens" => lens_icon(),
        // A music box, open, a mirror in its lid.
        "music_box" => icon_from(
            [
                "..#####..",
                "..#mmw#..",
                "..#mwm#..",
                "#ggggggg#",
                "#lloooos#",
                "#lloGoos#",
                "#looooss#",
                "#########",
                ".g.....g.",
            ],
            LACQUER,
            &[
                ('g', GILT.base),
                ('G', GILT.shine),
                ('m', GLASS[2]),
                ('w', GLASS[3]),
            ],
        ),
        _ => return None,
    })
}

/// A ring of brass round a disc of clear glass, a glint across it.
fn lens_icon() -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    for y in 0..ICON as i32 {
        for x in 0..ICON as i32 {
            let (dx, dy) = (x as f32 - 4.0, y as f32 - 4.0);
            let d = (dx * dx + dy * dy).sqrt();
            let lit = dx + dy;
            let color = if d > 4.4 {
                continue;
            } else if d > 3.5 {
                BRASS.edge
            } else if d > 2.4 {
                if lit < -1.5 {
                    BRASS.shine
                } else if lit < 0.5 {
                    BRASS.light
                } else if lit < 2.5 {
                    BRASS.base
                } else {
                    BRASS.shadow
                }
            } else if (-2.6..-1.0).contains(&lit) && (dx - dy).abs() < 1.6 {
                GLASS[3]
            } else if lit < 0.5 {
                GLASS[2]
            } else {
                GLASS[1]
            };
            canvas.set(x, y, color);
        }
    }
    canvas
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "pinecone" => pinecone_pile(),
        "acorn_stash" => young_oak(),
        "jay_feather" => windvane(),
        "old_nest" => nest_box(),
        "lost_lantern" => lantern_post(),
        "brass_lens" => telescope(),
        "music_box" => music_box(),
        _ => return None,
    })
}

/// One of these finds planted, at `stage` of its growing, or `None` if it isn't drawn.
pub fn stage(id: &str, stage: u8) -> Option<Piece> {
    Some(match (id, stage) {
        ("acorn_stash", 0) => sprouting_acorn(),
        ("acorn_stash", 1) => oak_seedling(),
        ("acorn_stash", _) => oak_sapling(),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// The pieces
// ---------------------------------------------------------------------------------------------

/// A heap of pinecones, every one lying its own way, one stood on end at the top.
fn pinecone_pile() -> Piece {
    let mut s = Canvas::new(22, 16);
    let ground = 14;
    shadow(&mut s, 11, ground, 10, 2);
    // Back and top first, so the ones in front lie over them.
    for (x, y, lie) in [
        (8, 0, Lie::Up),
        (2, 6, Lie::Right),
        (11, 5, Lie::Left),
        (13, 10, Lie::Right),
        (0, 10, Lie::Left),
        (6, 10, Lie::Left),
    ] {
        cone(&mut s, x, y, lie);
    }
    Piece {
        sprite: s,
        anchor: (11, ground),
    }
}

/// Which way a pinecone lies: its tip to the right or left, or stood up on its stalk end.
#[derive(Clone, Copy)]
pub(super) enum Lie {
    Right,
    Left,
    Up,
}

/// One pinecone with its top-left at `(x, y)`: rows of scales, each row set half a scale over
/// from the next, lit along its top (or its left, stood up).
pub(super) fn cone(s: &mut Canvas, x: i32, y: i32, lie: Lie) {
    const LYING: [&str; 5] = [
        "..#####..",
        ".#l*l*l#.",
        "k#ososos#",
        ".#sosos#.",
        "..#####..",
    ];
    for (row, line) in LYING.iter().enumerate() {
        for (column, code) in line.chars().enumerate() {
            let color = match code {
                '#' | 'k' => CONE.edge,
                's' => CONE.shadow,
                'o' => CONE.base,
                'l' => CONE.light,
                '*' => CONE.shine,
                _ => continue,
            };
            let (row, column) = (row as i32, column as i32);
            let (dx, dy) = match lie {
                Lie::Right => (column, row),
                Lie::Left => (8 - column, row),
                Lie::Up => (row, 8 - column),
            };
            put(s, x + dx, y + dy, color);
        }
    }
}

/// A young oak grown from the stash: a slim trunk, a few branches, a loose crown of lobed
/// leaves, and the acorns it came from still lying at its foot.
fn young_oak() -> Piece {
    let mut s = Canvas::new(26, 44);
    let ground = 42;
    shadow(&mut s, 13, ground, 9, 2);
    // Trunk and branches.
    stick(&mut s, (12.5, 21.0), (6.0, 12.0), (2.0, 1.2), BARK);
    stick(&mut s, (13.0, 24.0), (20.0, 13.0), (2.0, 1.2), BARK);
    stick(&mut s, (13.0, 22.0), (13.0, 6.0), (2.0, 1.2), BARK);
    stick(&mut s, (9.5, 21.0), (8.5, 19.0), (1.6, 1.2), BARK);
    stick(
        &mut s,
        (13.0, ground as f32 + 0.5),
        (12.8, 18.0),
        (3.4, 2.4),
        BARK,
    );
    for (from, to) in [((11, 41), (9, 42)), ((15, 41), (17, 42))] {
        line(&mut s, from, to, BARK.shadow);
    }
    // The crown, in clusters of leaves, the back ones first.
    for (cx, cy, r, salt) in [
        (19.5, 13.0, 4.0, 101),
        (6.0, 12.0, 4.2, 102),
        (13.0, 5.5, 5.0, 103),
        (8.5, 19.5, 3.4, 104),
        (18.5, 20.0, 3.4, 105),
        (13.0, 12.5, 3.6, 106),
    ] {
        leaves(&mut s, cx, cy, r, salt);
    }
    // Single leaves at the edges of the crown, their lobes showing.
    let leaf = [".o.", "olo", ".o.", "oos", ".s."];
    for (x, y) in [(1, 10), (22, 10), (9, 0), (15, 22), (4, 17), (21, 16)] {
        paint_rows(&mut s, x, y, &leaf, OAK_LEAF, &[]);
    }
    // The acorns it came from, and grass about its foot.
    tuft(&mut s, 13, ground, 5, 107);
    let acorn = [".k..", "kcCk", "kCdk", "#*o#", "#lo#", "#os#", ".##."];
    let cup = [
        ('k', CUP.edge),
        ('c', CUP.light),
        ('C', CUP.base),
        ('d', CUP.shadow),
    ];
    for (x, y) in [(3, 37), (17, 38)] {
        paint_rows(&mut s, x, y, &acorn, ACORN, &cup);
    }
    // One lying on its side.
    paint_rows(
        &mut s,
        7,
        39,
        &["..###.", "kk*lo#", "kCoos#", "..###."],
        ACORN,
        &cup,
    );
    Piece {
        sprite: s,
        anchor: (13, ground),
    }
}

/// One oak leaf, its lobes showing, standing up from its stalk.
const OAK: [&str; 5] = [".o.", "olo", ".o.", "oos", ".s."];

/// An acorn lying on its side, its cup to the left.
const LYING_ACORN: [&str; 4] = ["..###.", "kk*lo#", "kCoos#", "..###."];

fn cup_inks() -> [(char, Rgba); 4] {
    [
        ('k', CUP.edge),
        ('c', CUP.light),
        ('C', CUP.base),
        ('d', CUP.shadow),
    ]
}

/// Just planted: an acorn lying split on a mound of turned earth, and beside it the shoot it
/// sent up, reddish as young oaks are, with its first two leaves.
fn sprouting_acorn() -> Piece {
    let mut s = Canvas::new(16, 17);
    let (cx, ground) = (8, 15);
    mound(&mut s, cx, ground, (6, 4), 111);
    paint_rows(&mut s, 1, 9, &LYING_ACORN, ACORN, &cup_inks());
    for y in 6..ground - 3 {
        let color = if y < 9 { SHOOT } else { OAK_LEAF.shadow };
        put(&mut s, cx + 1, y, color);
    }
    paint_rows(&mut s, cx - 2, 2, &OAK, OAK_LEAF, &[]);
    paint_rows(&mut s, cx + 2, 1, &OAK, OAK_LEAF, &[]);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A visit on: a thin stem a hand high with a few leaves off it, and the empty husk at its foot.
fn oak_seedling() -> Piece {
    let mut s = Canvas::new(18, 26);
    let (cx, ground) = (9, 24);
    shadow(&mut s, cx, ground, 6, 2);
    mound(&mut s, cx, ground, (6, 2), 112);
    stick(&mut s, (9.0, 16.0), (5.0, 13.0), (1.2, 1.0), BARK);
    stick(&mut s, (9.5, 13.0), (13.0, 10.0), (1.2, 1.0), BARK);
    stick(
        &mut s,
        (9.5, ground as f32 + 0.5),
        (9.5, 7.0),
        (2.0, 1.2),
        BARK,
    );
    for (x, y) in [(3, 9), (13, 6), (7, 3), (10, 2), (5, 13)] {
        paint_rows(&mut s, x, y, &OAK, OAK_LEAF, &[]);
    }
    paint_rows(&mut s, 11, ground - 3, &LYING_ACORN, CUP, &cup_inks());
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// Another visit on: a sapling with a slim trunk, three branches and a small crown, the young
/// oak it will be the next time the train comes.
fn oak_sapling() -> Piece {
    let mut s = Canvas::new(22, 36);
    let (cx, ground) = (11, 34);
    shadow(&mut s, cx, ground, 8, 2);
    mound(&mut s, cx, ground, (6, 1), 113);
    stick(&mut s, (10.5, 21.0), (5.0, 13.0), (1.6, 1.0), BARK);
    stick(&mut s, (11.0, 19.0), (17.0, 11.0), (1.6, 1.0), BARK);
    stick(&mut s, (11.0, 16.0), (11.0, 5.0), (1.6, 1.0), BARK);
    stick(
        &mut s,
        (11.0, ground as f32 + 0.5),
        (11.0, 14.0),
        (2.8, 1.8),
        BARK,
    );
    for (x, y, r, salt) in [
        (16.5, 10.5, 3.2, 114),
        (5.0, 11.5, 3.2, 115),
        (11.0, 5.5, 3.8, 116),
        (10.5, 12.5, 2.8, 117),
    ] {
        leaves(&mut s, x, y, r, salt);
    }
    for (x, y) in [(0, 9), (19, 8), (9, 0), (14, 15)] {
        paint_rows(&mut s, x, y, &OAK, OAK_LEAF, &[]);
    }
    tuft(&mut s, cx, ground, 4, 118);
    paint_rows(&mut s, 2, ground - 3, &LYING_ACORN, ACORN, &cup_inks());
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A cluster of oak leaves: a lobed outline, lit from the upper left, the leaves picked out.
fn leaves(s: &mut Canvas, cx: f32, cy: f32, r: f32, salt: u32) {
    let reach = |x: i32, y: i32| {
        let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
        let angle = dy.atan2(dx);
        let lobe = (angle * 4.0 + salt as f32).sin() * 0.9 + (noise(x, y, salt) % 3) as f32 * 0.25;
        ((dx * dx + dy * dy).sqrt(), r + lobe)
    };
    let inside = |x: i32, y: i32| {
        let (d, edge) = reach(x, y);
        d <= edge
    };
    let span = r.ceil() as i32 + 2;
    let (mx, my) = (cx.round() as i32, cy.round() as i32);
    for y in my - span..=my + span {
        for x in mx - span..=mx + span {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (u, v) = ((x as f32 + 0.5 - cx) / r, (y as f32 + 0.5 - cy) / r);
            let mut color = if edge {
                OAK_LEAF.edge
            } else {
                lit(OAK_LEAF, u, v, noise(x, y, salt + 1))
            };
            // Each leaf a little patch, with darker gaps between.
            if !edge {
                let leaf = noise(x / 2, (y + x % 2) / 2, salt + 2) % 6;
                if leaf == 0 {
                    color = OAK_LEAF.shadow;
                } else if leaf == 1 && u + v < 0.2 {
                    color = OAK_LEAF.light;
                }
            }
            put(s, x, y, color);
        }
    }
}

/// A slim pole with a jay's feather for a vane, turning on a brass cap, arms below it to say
/// which way is which.
fn windvane() -> Piece {
    let mut s = Canvas::new(25, 46);
    let ground = 44;
    shadow(&mut s, 12, ground, 5, 2);
    stick(
        &mut s,
        (12.0, ground as f32 + 0.5),
        (12.0, 11.0),
        (3.0, 2.4),
        POLE,
    );
    // The arms, each finished with a brass ball.
    hline(&mut s, 5, 21, 15, IRON.light);
    hline(&mut s, 5, 22, 15, IRON.edge);
    for x in [4, 19] {
        paint_rows(&mut s, x, 20, &[".#.", "#*#", ".#."], BRASS, &[]);
    }
    rect(&mut s, 10, 20, 5, 3, BRASS.base);
    hline(&mut s, 10, 20, 5, BRASS.light);
    hline(&mut s, 10, 22, 5, BRASS.edge);
    put(&mut s, 10, 20, BRASS.shine);
    // The feather, tip to the wind.
    let (tip, end) = (1, 20);
    let shaft = |x: i32| 6.0 + (x - tip) as f32 * 0.16 - ((x - tip) as f32 * 0.16).powi(2) * 0.25;
    for x in tip..=end {
        let t = (x - tip) as f32 / (end - tip) as f32;
        let swell = (t * 2.4).min(1.0) * (1.0 - (t - 0.78).max(0.0) * 3.4);
        let (up, down) = ((4.2 * swell).round() as i32, (3.2 * swell).round() as i32);
        let middle = shaft(x).round() as i32;
        for y in middle - up..=middle + down {
            let off = (y - middle).abs();
            let color = if y == middle - up || y == middle + down {
                if t > 0.8 { QUILL.edge } else { JAY.edge }
            } else if y == middle {
                QUILL.light
            } else if t > 0.8 {
                if y < middle { QUILL.base } else { QUILL.shadow }
            } else {
                // Barred across, the bars swept back from the shaft.
                let band = ((x as f32 - off as f32 * 0.9) / 1.4).floor() as i32;
                match band.rem_euclid(4) {
                    0 => NAVY,
                    1 if y < middle => JAY.shine,
                    1 => JAY.light,
                    2 => JAY.light,
                    _ if y > middle => JAY.shadow,
                    _ => JAY.base,
                }
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, tip - 1, shaft(tip).round() as i32, JAY.edge);
    for x in end + 1..end + 4 {
        put(&mut s, x, shaft(x).round() as i32, QUILL.shadow);
    }
    // The brass cap it turns on.
    paint_rows(&mut s, 10, 9, &[".###.", "#*lo#", ".#s#."], BRASS, &[]);
    tuft(&mut s, 12, ground, 3, 111);
    Piece {
        sprite: s,
        anchor: (12, ground),
    }
}

/// A wooden nest box up a pole: a round door, a perch, a gable roof gone green with moss, and a
/// wisp of somebody's nesting straw.
fn nest_box() -> Piece {
    let mut s = Canvas::new(21, 46);
    let ground = 44;
    shadow(&mut s, 10, ground, 5, 2);
    stick(
        &mut s,
        (10.0, ground as f32 + 0.5),
        (10.0, 24.0),
        (3.2, 3.0),
        POLE,
    );
    // The bracket it sits on.
    line(&mut s, (8, 30), (6, 27), POLE.base);
    line(&mut s, (12, 30), (14, 27), POLE.shadow);
    // The box: planks, its gabled front, lit from the left.
    let (left, right, eaves, bottom): (i32, i32, i32, i32) = (3, 17, 13, 27);
    let apex = (left + right) / 2;
    for y in 5..=bottom {
        for x in left..right {
            let gable = y < eaves && (x - apex).abs() > (y - 6);
            if gable {
                continue;
            }
            let plank = (x - left) % 4;
            let mut color = if x == left || x == right - 1 || y == bottom {
                BOX.edge
            } else if plank == 0 {
                BOX.shadow
            } else if x == left + 1 {
                BOX.light
            } else if x >= right - 3 {
                BOX.shadow
            } else {
                BOX.base
            };
            if color == BOX.base && chance(x, y, 121, 30) {
                color = BOX.light;
            }
            put(&mut s, x, y, color);
        }
    }
    // The round door, dark inside but for the lit far rim, and the straw.
    let (hx, hy) = (10.0, 16.5);
    for y in 13..21 {
        for x in 6..14 {
            let (dx, dy) = (x as f32 + 0.5 - hx, y as f32 + 0.5 - hy);
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 2.3 {
                let rim = d > 1.4 && dx + dy > 0.8;
                put(&mut s, x, y, if rim { BOX.base } else { rgb(0x2e2018) });
            } else if d <= 3.1 {
                put(&mut s, x, y, BOX.edge);
            }
        }
    }
    put(&mut s, 9, 18, rgb(0xd8b04e));
    put(&mut s, 8, 19, rgb(0xe8c86a));
    // The perch, end on, and its little shadow.
    put(&mut s, 9, 21, BOX.shine);
    put(&mut s, 10, 21, BOX.light);
    put(&mut s, 9, 22, BOX.base);
    put(&mut s, 10, 22, BOX.edge);
    hline(&mut s, 9, 23, 3, BOX.shadow);
    // The roof: two boards meeting over the gable, overhanging, moss thick along their tops.
    for side in [-1, 1] {
        for step in 0..=9 {
            let x = apex + side * step;
            let y = 4 + step;
            put(&mut s, x, y, POLE.light);
            put(&mut s, x, y + 1, POLE.base);
            put(&mut s, x, y + 2, POLE.edge);
            if step < 9 {
                put(&mut s, x + side, y, POLE.base);
            }
        }
    }
    // Moss in a cushion along the top of each board, thicker at the ridge, hanging over at
    // the eaves.
    for side in [-1, 1] {
        for step in 0..=9 {
            let x = apex + side * step;
            let y = 4 + step;
            let bump = i32::from(noise(x, 0, 122).is_multiple_of(3)) + i32::from(step < 2);
            let lit_side = side < 0;
            for dy in 0..=1 + bump {
                let py = y - dy;
                let color = if dy == 1 + bump {
                    MOSSY.edge
                } else if dy == bump && (lit_side || chance(x, py, 123, 90)) {
                    MOSSY.light
                } else if lit_side {
                    MOSSY.base
                } else {
                    MOSSY.shadow
                };
                put(&mut s, x, py, color);
            }
            if lit_side && chance(x, y, 124, 50) {
                put(&mut s, x, y - bump, MOSSY.shine);
            }
        }
        put(&mut s, apex + side * 9, 15, MOSSY.shadow);
        put(&mut s, apex + side * 7, 13, MOSSY.base);
    }
    tuft(&mut s, 10, ground, 3, 127);
    Piece {
        sprite: s,
        anchor: (10, ground),
    }
}

/// A crook-topped post with a lantern hung from it, its candle still burning and its light
/// warm on the air about it.
fn lantern_post() -> Piece {
    let mut s = Canvas::new(27, 51);
    let ground = 48;
    shadow(&mut s, 9, ground, 6, 2);
    // The glow first, so everything else is drawn in it.
    let (lx, ly) = (18, 19);
    glow(&mut s, lx, ly, 8.5, rgba(0xffe7a0, 44));
    glow(&mut s, lx, ly, 5.5, rgba(0xffe7a0, 56));
    // The post, and its crook curling over to the right.
    stick(
        &mut s,
        (9.0, ground as f32 + 0.5),
        (9.0, 8.0),
        (3.4, 3.0),
        POLE,
    );
    let (cx, cy, r) = (13.5, 8.5, 4.5);
    for y in 1..10 {
        for x in 5..21 {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let d = (dx * dx + dy * dy).sqrt();
            if dy > 0.0 || !(r - 1.5..=r + 1.5).contains(&d) {
                continue;
            }
            let outside = d - r;
            let color = if outside > 0.8 {
                POLE.edge
            } else if outside < -0.8 {
                POLE.shadow
            } else if dx < 0.0 {
                POLE.light
            } else {
                POLE.base
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, 18, 9, POLE.base);
    put(&mut s, 19, 9, POLE.edge);
    hung_lantern(&mut s, (lx, 10));
    // Grass, a stone and a toadstool about the foot.
    lump(&mut s, 5.0, ground as f32 - 0.6, 2.6, 1.7, STONE, 131);
    tuft(&mut s, 9, ground, 4, 132);
    toadstool(&mut s, 13, ground);
    Piece {
        sprite: s,
        anchor: (9, ground),
    }
}

/// The lost lantern hung by its hook from `(x, y)`: a cap, glass all round the burning candle, a
/// base, seven pixels wide and fourteen down from the hook.
pub(super) fn hung_lantern(s: &mut Canvas, (x, y): (i32, i32)) {
    // The hook and ring it hangs by.
    vline(s, x, y, 2, IRON.light);
    put(s, x + 1, y + 1, IRON.edge);
    paint_rows(
        s,
        x - 3,
        y + 2,
        &[
            "...#...", "..#l#..", ".#lls#.", "#######", "#yGfGy#", "#yGfGy#", "#yGcGy#", "#yGcGy#",
            "#yGcGy#", "#######", ".#sss#.", "..#s#..",
        ],
        IRON,
        &[
            ('y', GLOW[0]),
            ('G', GLOW[1]),
            ('f', rgb(0xfffbe8)),
            ('c', CANDLE),
        ],
    );
    put(s, x - 2, y + 6, GLOW[2]);
    put(s, x - 2, y + 7, GLOW[2]);
}

/// A small brass telescope on a wooden tripod, tilted up at the sky.
fn telescope() -> Piece {
    let mut s = Canvas::new(32, 38);
    let ground = 36;
    shadow(&mut s, 16, ground, 13, 2);
    // The tripod: the back leg first, darker, then the two in front.
    let head = (16.0, 19.0);
    stick(&mut s, head, (18.0, ground as f32 - 2.5), (2.0, 1.6), POLE);
    for foot in [(6.0, ground as f32 + 0.5), (26.0, ground as f32 + 0.5)] {
        stick(&mut s, head, foot, (2.6, 2.0), POLE);
    }
    for x in [6, 25] {
        put(&mut s, x, ground, POLE.edge);
    }
    paint_rows(
        &mut s,
        14,
        16,
        &["#####", "#*los", "#loo#", ".#s#."],
        BRASS,
        &[],
    );
    // The tube, from eyepiece to lens, in sections.
    let (from, to) = ((6.0_f32, 24.0_f32), (28.5_f32, 5.0_f32));
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = (dx / length, dy / length);
    let (nx, ny) = (-uy, ux);
    for y in 0..32 {
        for x in 0..32 {
            let (px, py) = (x as f32 + 0.5 - from.0, y as f32 + 0.5 - from.1);
            let t = (px * ux + py * uy) / length;
            let across = px * nx + py * ny;
            // Eyepiece, draw tube, leather-wrapped body, the wider dew cap at the far end.
            let (half, ramp) = match t {
                t if !(0.0..=1.0).contains(&t) => continue,
                t if t < 0.1 => (1.3, IRON),
                t if t < 0.34 => (1.7, BRASS),
                t if t < 0.42 => (2.2, BRASS),
                t if t < 0.64 => (2.3, LEATHER),
                t if t < 0.82 => (2.3, BRASS),
                _ => (2.9, BRASS),
            };
            if across.abs() > half {
                continue;
            }
            let k = across / half;
            let ring = [0.1, 0.34, 0.42, 0.64, 0.82]
                .iter()
                .any(|joint| (t - joint).abs() * length < 0.7);
            let color = if half - across.abs() < 0.7 || t * length > length - 0.8 {
                ramp.edge
            } else if ring {
                BRASS.shine
            } else if k < -0.45 {
                ramp.light
            } else if k > 0.35 {
                ramp.shadow
            } else {
                ramp.base
            };
            put(&mut s, x, y, color);
        }
    }
    // The lens catching the sky at the far end.
    put(&mut s, 28, 4, GLASS[3]);
    put(&mut s, 29, 5, GLASS[2]);
    put(&mut s, 27, 3, GLASS[1]);
    tuft(&mut s, 6, ground, 2, 141);
    tuft(&mut s, 26, ground, 2, 142);
    Piece {
        sprite: s,
        anchor: (16, ground),
    }
}

/// An old stump with a music box set on it, open, a little dancer turning in front of the
/// mirror in its lid and the tune drifting off.
fn music_box() -> Piece {
    let mut s = Canvas::new(36, 40);
    let ground = 37;
    shadow(&mut s, 18, ground, 16, 2);
    // The stump, furrowed bark lit from the left, roots spreading at its foot.
    let (cx, top, rx, ry) = (18.0, 24.0, 12.5, 3.6);
    for y in top as i32..=ground {
        let depth = (y as f32 - top) / (ground as f32 - top);
        let half = rx + (depth - 0.6).max(0.0) * 7.0;
        for x in 0..36 {
            let dx = x as f32 + 0.5 - cx;
            if dx.abs() > half {
                continue;
            }
            let k = dx / half;
            let mut color = if dx.abs() > half - 1.0 {
                BARK.edge
            } else if k < -0.55 {
                BARK.light
            } else if k > 0.45 {
                BARK.shadow
            } else {
                BARK.base
            };
            if dx.abs() <= half - 1.0 {
                if noise(x, y / 5, 151).is_multiple_of(4) {
                    color = mix(color, BARK.edge, 0.55);
                } else if k < 0.0 && chance(x, y, 152, 26) {
                    color = BARK.shine;
                }
            }
            if y >= ground - 1 {
                color = mix(color, BARK.edge, 0.35);
            }
            put(&mut s, x, y, color);
        }
    }
    moss(&mut s, 8.0, 34.5, 3.0, 2.0, 153);
    // The cut top, ring on ring, a crack from the heart.
    for y in (top - ry).floor() as i32..=(top + ry).ceil() as i32 {
        for x in 0..36 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - top) / ry);
            let d = (u * u + v * v).sqrt();
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.9 {
                BARK.edge
            } else if d > 0.78 {
                BARK.light
            } else if (d * 5.5) as i32 % 2 == 0 {
                if u + v < -0.3 {
                    RINGS.shine
                } else {
                    RINGS.light
                }
            } else if u + v < -0.3 {
                RINGS.light
            } else {
                RINGS.base
            };
            put(&mut s, x, y, color);
        }
    }
    line(&mut s, (19, 24), (27, 26), RINGS.shadow);
    // The box, a little from one side: its front, its shaded side going back, the open top.
    let (left, right, rim, base, depth) = (10, 23, 15, 22, 3);
    for k in 1..=depth {
        let x = right + k;
        for y in rim - k..=base - k {
            let color = if k == depth || y == rim - k || y == base - k {
                LACQUER.edge
            } else {
                LACQUER.shadow
            };
            put(&mut s, x, y, color);
        }
        let back = rim - k;
        for x in left + k..=right + k {
            let color = if k == depth || x == left + k {
                LACQUER.edge
            } else if k == 1 {
                rgb(0x4a1420)
            } else {
                VELVET.shadow
            };
            put(&mut s, x, back, color);
        }
    }
    // The comb and drum that play the tune, glinting inside.
    hline(&mut s, left + 3, rim - 2, 5, GILT.base);
    put(&mut s, left + 3, rim - 2, GILT.shine);
    // The lid, thrown back on its hinges: velvet inside, and a mirror.
    let (lid_left, lid_right, lid_top, hinge) = (left + depth, right + depth, 5, rim - depth);
    for y in lid_top..hinge {
        for x in lid_left..=lid_right {
            let color = if x == lid_left || x == lid_right || y == lid_top {
                LACQUER.edge
            } else if y == lid_top + 1 || x == lid_left + 1 {
                VELVET.light
            } else {
                VELVET.base
            };
            put(&mut s, x, y, color);
        }
    }
    vline(
        &mut s,
        lid_right + 1,
        lid_top + 1,
        hinge - lid_top - 1,
        LACQUER.base,
    );
    hline(
        &mut s,
        lid_left,
        lid_top,
        lid_right - lid_left + 1,
        GILT.light,
    );
    for y in lid_top + 2..hinge - 1 {
        for x in lid_left + 3..lid_right - 2 {
            let corner =
                (y == lid_top + 2 || y == hinge - 2) && (x == lid_left + 3 || x == lid_right - 3);
            if corner {
                continue;
            }
            let glint = x - y == lid_left - lid_top + 1 || x - y == lid_left - lid_top + 2;
            put(&mut s, x, y, if glint { GLASS[3] } else { GLASS[2] });
        }
    }
    // The dancer on her spindle, turning in front of the mirror.
    paint_rows(
        &mut s,
        16,
        2,
        &[
            "..h..", ".h.h.", ".hfh.", "..b..", ".tTt.", "tttTT", "..l..", "..l..", "..l..",
            "..s..", ".lo#.",
        ],
        GILT,
        &[
            ('h', rgb(0xf4d4b8)),
            ('f', rgb(0xe8b898)),
            ('b', rgb(0xd0587a)),
            ('t', rgb(0xf5bfcd)),
            ('T', rgb(0xe08aa0)),
            ('l', rgb(0xfbf6ee)),
        ],
    );
    // The front, lacquered, with a gilt rim and trim.
    for y in rim..=base {
        for x in left..=right {
            let across = (x - left) as f32 / (right - left) as f32;
            let color = if x == left || x == right || y == base {
                LACQUER.edge
            } else if y == rim {
                GILT.light
            } else if x == left + 1 || y == rim + 1 {
                LACQUER.light
            } else if across > 0.8 {
                LACQUER.shadow
            } else {
                LACQUER.base
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, left, rim, GILT.shine);
    for x in left + 2..=right - 2 {
        put(&mut s, x, rim + 2, GILT.shadow);
        put(&mut s, x, base - 2, GILT.shadow);
    }
    vline(&mut s, left + 2, rim + 2, base - rim - 3, GILT.shadow);
    vline(&mut s, right - 2, rim + 2, base - rim - 3, GILT.shadow);
    paint_rows(&mut s, 15, rim + 3, &[".#.", "#*o", ".s."], GILT, &[]);
    // Gilt feet, and the key in its side.
    for x in [left, right - 1, right + depth - 1] {
        let y = if x > right {
            base - depth + 1
        } else {
            base + 1
        };
        paint_rows(&mut s, x, y, &["lo"], GILT, &[]);
    }
    paint_rows(
        &mut s,
        right + depth + 1,
        rim - 1,
        &[".l", "lo", "#s", ".s"],
        GILT,
        &[],
    );
    // The tune, drifting up and away.
    for (x, y) in [(29, 6), (32, 1)] {
        paint_rows(
            &mut s,
            x,
            y,
            &[".#.", ".##", ".#.", "l#.", "##."],
            PLUM,
            &[],
        );
    }
    // A toadstool and some grass at the stump's foot.
    toadstool(&mut s, 30, ground);
    tuft(&mut s, 5, ground, 3, 154);
    tuft(&mut s, 27, ground, 2, 155);
    Piece {
        sprite: s,
        anchor: (18, ground),
    }
}

/// A little red toadstool with white spots, standing at `(x, ground)`.
pub(super) fn toadstool(s: &mut Canvas, x: i32, ground: i32) {
    paint_rows(
        s,
        x - 2,
        ground - 5,
        &[".##.", "#*wo#", "#wos#", ".ccd.", "..cd."],
        TOADSTOOL,
        &[
            ('w', rgb(0xfbf6ee)),
            ('c', rgb(0xf3e9cf)),
            ('d', rgb(0xc8b8a0)),
        ],
    );
}

/// A tapering stick from `from` to `to`, `widths` across at each end, lit from the left and
/// outlined in its own edge tone: a trunk, a leg, a pole.
pub(super) fn stick(
    s: &mut Canvas,
    from: (f32, f32),
    to: (f32, f32),
    widths: (f32, f32),
    ramp: Ramp,
) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / length, dy / length);
    // Across the stick, pointing away from the light.
    let (mut nx, mut ny) = (-uy, ux);
    if nx + ny * 0.3 < 0.0 {
        (nx, ny) = (-nx, -ny);
    }
    let reach = widths.0.max(widths.1).ceil() as i32 + 1;
    let (x0, x1) = (
        from.0.min(to.0) as i32 - reach,
        from.0.max(to.0) as i32 + reach,
    );
    let (y0, y1) = (
        from.1.min(to.1) as i32 - reach,
        from.1.max(to.1) as i32 + reach,
    );
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f32 + 0.5 - from.0, y as f32 + 0.5 - from.1);
            let t = (px * ux + py * uy) / length;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let across = px * nx + py * ny;
            let half = (widths.0 + (widths.1 - widths.0) * t) / 2.0;
            if across.abs() > half {
                continue;
            }
            // Counted in pixels from the lit side: thin sticks get a lit side and an edge, thick
            // ones an edge all round.
            let (width, from_lit) = (half * 2.0, across + half);
            let color = if width < 2.6 {
                if from_lit < width / 2.0 {
                    ramp.light
                } else {
                    ramp.edge
                }
            } else if width < 3.6 {
                if from_lit < 1.0 {
                    ramp.light
                } else if from_lit < width - 1.0 {
                    ramp.base
                } else {
                    ramp.edge
                }
            } else if from_lit < 0.9 || from_lit > width - 0.9 {
                ramp.edge
            } else if from_lit < 1.9 {
                ramp.light
            } else if from_lit > width - 1.9 {
                ramp.shadow
            } else {
                ramp.base
            };
            put(s, x, y, color);
        }
    }
}

/// Warm light on the air in a disc about `(cx, cy)`, laid over whatever is there.
pub(super) fn glow(s: &mut Canvas, cx: i32, cy: i32, radius: f32, color: Rgba) {
    let reach = radius.ceil() as i32;
    for y in cy - reach..=cy + reach {
        for x in cx - reach..=cx + reach {
            let (dx, dy) = ((x - cx) as f32, (y - cy) as f32);
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            if s.get(x, y).a == 0 {
                s.set(x, y, color);
            } else {
                put(s, x, y, color);
            }
        }
    }
}
