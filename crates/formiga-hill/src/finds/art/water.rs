//! The finds in the water: the finds scooped up (Kind::Scoop): their icons and their Hilltop pieces.
//! Stream things, mostly stone and glass: cool greys and sea greens, wet-looking, with one warm
//! light among them where the fallen star stands.

use super::brush::*;
use super::{ICON, Piece};
use crate::materials::STONE;
use crate::paint::{Ramp, chance, ellipse, mix, noise, polygon, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    let (rows, inks): (&[&str; 9], Vec<(u8, Rgba)>) = match id {
        "skimming_stone" => (&SKIMMING_STONE, letters(SLATE, b"#sol*").to_vec()),
        "river_glass" => (&RIVER_GLASS, letters(SEA_GLASS, b"#sol*").to_vec()),
        "mussel_shell" => (
            &MUSSEL_SHELL,
            [letters(MUSSEL, b"#mMb."), letters(NACRE, b".pnN*")].concat(),
        ),
        "bottle_message" => (
            &BOTTLE_MESSAGE,
            [
                &letters(CORK, b"C.c..")[..],
                &letters(SEA_GLASS, b"Gdgh*"),
                &letters(PAPER, b".Nn.."),
                &[(b'r', rgb(0xc8483e))],
            ]
            .concat(),
        ),
        "wooden_duck" => (&DUCK, duck_inks()),
        "geode" => (
            &GEODE,
            [
                &letters(GEODE_SHELL, b"#QrR.")[..],
                &letters(AMETHYST, b".Ppv*"),
                &[(b'w', AGATE)],
            ]
            .concat(),
        ),
        "fallen_star" => (&STAR, star_inks()),
        _ => return None,
    };
    let mut icon = Canvas::new(ICON, ICON);
    stamp(&mut icon, (0, 0), rows, &inks, false);
    Some(icon)
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "skimming_stone" => stone_ring(),
        "river_glass" => wind_chime(),
        "mussel_shell" => birdbath(),
        "bottle_message" => bottle_tree(),
        "wooden_duck" => pond(),
        "geode" => geode_plinth(),
        "fallen_star" => star_stone(),
        _ => return None,
    })
}

const SLATE: Ramp = Ramp::new(0x4a535c, 0x69747d, 0x8b959b, 0xafb8bb, 0xdde3e2);
const SANDSTONE: Ramp = Ramp::new(0x6a5d50, 0x8d7f70, 0xab9e8c, 0xc8bca8, 0xe6dccb);
const DRIFTWOOD: Ramp = Ramp::new(0x564a40, 0x786a5c, 0x9a8a78, 0xbcad98, 0xdcd0bc);
const WATER: Ramp = Ramp::new(0x2c5a74, 0x3f7a96, 0x5c9cb6, 0x8cc4d6, 0xe2f6fa);
const SEA_GLASS: Ramp = Ramp::new(0x3d7a60, 0x67a888, 0x8fc8a8, 0xb8e2c8, 0xf0fbf4);
const BLUE_GLASS: Ramp = Ramp::new(0x34607e, 0x5a8cb0, 0x84b2d2, 0xb0d4ea, 0xf0f8ff);
const PALE_GLASS: Ramp = Ramp::new(0x4a7a7a, 0x8abcb8, 0xb4dcd6, 0xd6eeea, 0xffffff);
const TEAL_GLASS: Ramp = Ramp::new(0x225e62, 0x3a8a8a, 0x5cb0aa, 0x8ad0c8, 0xe4fbf6);
const MUSSEL: Ramp = Ramp::new(0x161c2e, 0x232c48, 0x34406a, 0x4c5c8c, 0x7c8cb8);
const NACRE: Ramp = Ramp::new(0x8a88b0, 0xaeb0d0, 0xcdd0e6, 0xe6e8f4, 0xffffff);
const COBALT: Ramp = Ramp::new(0x1a2a6a, 0x24409a, 0x3a5ec4, 0x6a8ee0, 0xd0e0ff);
const BOTTLE_GREEN: Ramp = Ramp::new(0x1e4a2a, 0x2a6a3a, 0x3e8a4e, 0x6ab878, 0xd8f4d8);
const AMBER: Ramp = Ramp::new(0x6a3a10, 0x9a5a18, 0xc8822a, 0xe8aa52, 0xfff0c8);
const CORK: Ramp = Ramp::new(0x5a3a20, 0x7a5232, 0xa07048, 0xc09068, 0xdcb48c);
const PAPER: Ramp = Ramp::new(0x9a845a, 0xc8b48a, 0xe8dab4, 0xf6eed4, 0xffffff);
const DRAKE: Ramp = Ramp::new(0x14402c, 0x1d5a3e, 0x2c7a52, 0x48a070, 0x90d8a8);
const CHESTNUT: Ramp = Ramp::new(0x4a2418, 0x6e3624, 0x8e4a30, 0xaa6440, 0xc88a60);
const DECOY: Ramp = Ramp::new(0x6a6458, 0x989080, 0xbab2a0, 0xd8d0be, 0xf4eee0);
const GEODE_SHELL: Ramp = Ramp::new(0x4e4a52, 0x6a6670, 0x88848c, 0xa8a4aa, 0xc8c4c8);
const AMETHYST: Ramp = Ramp::new(0x3a1e5a, 0x5a2e8a, 0x8048b8, 0xa878d8, 0xf0e0ff);
const AGATE: Rgba = rgb(0xece6f2);
const PLINTH: Ramp = Ramp::new(0x8a7458, 0xb09a78, 0xcdb894, 0xe2d2b0, 0xf4ead2);
const MOONSTONE: Ramp = Ramp::new(0x6a6680, 0x928ea6, 0xb6b2c6, 0xd2d0de, 0xf0eef6);
const STARLIGHT: Ramp = Ramp::new(0xc08a2e, 0xeab450, 0xffd66e, 0xfff0b0, 0xffffff);
const REED: Ramp = Ramp::new(0x2e5426, 0x3f6e32, 0x5a8a44, 0x7aa858, 0xa4c878);
const CATTAIL: Ramp = Ramp::new(0x3e2416, 0x5e3620, 0x7a4a2a, 0x9a6238, 0xbc8452);
const LILY: Ramp = Ramp::new(0x2a5a2e, 0x3a7a3a, 0x52964a, 0x74b45e, 0x9cd07c);
const MOSS: Rgba = rgb(0x6f8f3e);
const LEAFY: Rgba = rgb(0x8eae4c);
const LICHEN: Rgba = rgb(0xc4cc8a);
const TWINE: Rgba = rgb(0x7a6a56);

// ---------------------------------------------------------------------------------------------
// Icons
// ---------------------------------------------------------------------------------------------

/// Flat as a biscuit, its thin edge showing underneath.
const SKIMMING_STONE: [&str; 9] = [
    ".........",
    ".........",
    "..#####..",
    ".#*lloo#.",
    "#lloooos#",
    "#oosooss#",
    ".#sssss#.",
    "..#####..",
    ".........",
];

/// Frosted, rounded off by the stream.
const RIVER_GLASS: [&str; 9] = [
    ".........",
    "..####...",
    ".#*llo#..",
    ".#llooo#.",
    "#lloloos#",
    "#oooooss#",
    ".#ossss#.",
    "..#####..",
    ".........",
];

/// One half of a mussel, hinge at the top: pearly inside, its blue-black lip round the edge.
const MUSSEL_SHELL: [&str; 9] = [
    "##.......",
    "#b##.....",
    "#bMM##...",
    ".#bMMm#..",
    ".#bMMMm#.",
    "..#nbMmm#",
    "..#nnMMm#",
    "...#nnNm#",
    "....####.",
];

/// Corked, with the note rolled up inside and tied.
const BOTTLE_MESSAGE: [&str; 9] = [
    "...CCC...",
    "...CcC...",
    "...GhG...",
    "..GhgdG..",
    ".GhnrnNG.",
    ".G*NrNdG.",
    ".GhgggdG.",
    ".GhgggdG.",
    "..GGGGG..",
];

/// A painted decoy, a drake, facing left.
const DUCK: [&str; 9] = [
    ".........",
    ".HHH.....",
    "HgegH....",
    "YYggH...T",
    ".HwwH..TT",
    "CcclllbbT",
    "Cccbbbbb#",
    ".#######.",
    ".........",
];

fn duck_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'H', DRAKE.edge),
        (b'g', DRAKE.base),
        (b'e', rgb(0x10141a)),
        (b'Y', rgb(0xe0a830)),
        (b'w', rgb(0xf4f0e6)),
        (b'T', rgb(0x2a2a30)),
        (b'C', CHESTNUT.edge),
        (b'c', CHESTNUT.base),
        (b'l', DECOY.light),
        (b'b', DECOY.base),
        (b'#', DECOY.edge),
    ]
}

/// Cracked open: grey and plain outside, a cave of purple crystal within.
const GEODE: [&str; 9] = [
    ".........",
    "..#####..",
    ".#RwwwR#.",
    "#RwPPpwr#",
    "#wPPpvvw#",
    "#wPpv*vw#",
    "#rwvvvwQ#",
    ".#QrrrQ#.",
    "..#####..",
];

/// A five-pointed star, still glowing.
const STAR: [&str; 9] = [
    "....#....",
    "...#*#...",
    "...#l#...",
    "####l####",
    "#*lllooo#",
    ".#lllos#.",
    "..#loo#..",
    ".#lo#os#.",
    ".##...##.",
];

// ---------------------------------------------------------------------------------------------
// Pieces
// ---------------------------------------------------------------------------------------------

/// A stone lying in the grass: where its middle is, how big it is, and what stone it is.
type Stone = ((i32, i32), (i32, i32), Ramp);

/// Where each stone of the ring lies, as a fraction of the way round, how big it is, and what
/// stone it is. The near ones are seen more fully, the far ones nearly edge on.
const RING: [(f32, (i32, i32), Ramp); 8] = [
    (0.00, (4, 2), SANDSTONE),
    (0.13, (4, 2), STONE),
    (0.25, (5, 2), SLATE),
    (0.37, (4, 2), STONE),
    (0.50, (4, 2), SANDSTONE),
    (0.63, (3, 1), SLATE),
    (0.75, (4, 1), STONE),
    (0.88, (3, 1), SLATE),
];

/// A ring of flat stones lying in the grass, a little circle to sit on.
fn stone_ring() -> Piece {
    let mut s = Canvas::new(42, 17);
    let centre = (21, 7);
    let mut stones: Vec<Stone> = RING
        .iter()
        .map(|&(turn, size, ramp)| {
            let (sin, cos) = (turn * TAU).sin_cos();
            let at = (
                centre.0 + (cos * 15.0).round() as i32,
                centre.1 + (sin * 4.5).round() as i32,
            );
            (at, size, ramp)
        })
        .collect();
    stones.sort_by_key(|(at, _, _)| at.1);
    // Worn a little in the middle, where everyone's feet go.
    for y in centre.1 - 3..=centre.1 + 3 {
        for x in centre.0 - 11..=centre.0 + 11 {
            if in_ellipse(x, y, centre, (11, 3)) && chance(x, y, 41, 150) {
                put(&mut s, x, y, rgba(0xb8d890, 70));
            }
        }
    }
    for &(at, (rx, ry), _) in &stones {
        ellipse(
            &mut s,
            at.0 + 1,
            at.1 + ry + 1,
            rx + 1,
            ry.max(2) - 1,
            SHADOW,
        );
    }
    // Grass grown up between the stones, and daisies.
    for (x, y, flip) in [(9, 1, false), (28, 0, true), (2, 8, false), (36, 9, true)] {
        stamp(&mut s, (x, y), &TUFT, &tuft_inks(), flip);
    }
    stamp(&mut s, (15, 5), &DAISY, &daisy_inks(), false);
    stamp(&mut s, (25, 7), &DAISY, &daisy_inks(), false);
    for (index, &(at, size, ramp)) in stones.iter().enumerate() {
        let thick = if size.1 > 1 { 2 } else { 1 };
        flat_stone(&mut s, at, size, thick, ramp, 40 + index as u32);
    }
    Piece {
        sprite: s,
        anchor: (centre.0, centre.1 + 3),
    }
}

/// Each string of the chime: where it hangs from the bar, how long it is, and the glass on it
/// (how far down, which shape, which glass).
type Chime = (i32, i32, &'static [(i32, usize, Ramp)]);
const CHIMES: [Chime; 6] = [
    (2, 9, &[(9, 2, PALE_GLASS)]),
    (5, 15, &[(3, 1, BLUE_GLASS), (15, 0, SEA_GLASS)]),
    (8, 21, &[(8, 3, SEA_GLASS), (21, 1, TEAL_GLASS)]),
    (15, 19, &[(4, 2, TEAL_GLASS), (19, 3, BLUE_GLASS)]),
    (18, 13, &[(13, 0, SEA_GLASS)]),
    (21, 8, &[(2, 1, SEA_GLASS), (8, 3, BLUE_GLASS)]),
];

/// Glass worn smooth, in four shapes, each hung from the top middle.
const SHARDS: [&[&str]; 4] = [
    &[".##.", "#*l#", "#lo#", "#oo#", "#os#", ".##."],
    &["###", "#*#", "#o#", "#s#", ".#."],
    &[".#..", "#*#.", "#lo#", "#os#", ".##."],
    &[".##.", "#*o#", "#os#", ".##."],
];

/// A short driftwood post with a crossbar, and river glass hung from it on strings.
fn wind_chime() -> Piece {
    let mut s = Canvas::new(24, 42);
    let ground = 39;
    ellipse(&mut s, 12, ground, 6, 2, SHADOW);
    for &(x, length, _) in &CHIMES {
        for y in 10..10 + length {
            put(&mut s, x, y, TWINE);
        }
    }
    model(&mut s, DRIFTWOOD, UPRIGHT, 30, 51, |x, y| {
        (10..=13).contains(&x) && (5..=ground).contains(&y)
    });
    model(&mut s, DRIFTWOOD, LYING, 30, 52, |x, y| {
        (0..=23).contains(&x) && (6..=9).contains(&y) && !((x == 0 || x == 23) && y == 9)
    });
    model(&mut s, DRIFTWOOD, ROUND, 0, 53, |x, y| {
        in_ellipse(x, y, (11, 3), (2, 2)) || in_ellipse(x, y, (12, 3), (2, 2))
    });
    for &(x, _, _) in &CHIMES {
        put(&mut s, x, 9, DRIFTWOOD.edge);
    }
    for &(x, _, glass) in &CHIMES {
        for &(down, shape, ramp) in glass {
            let rows = SHARDS[shape];
            let width = rows[0].len() as i32;
            stamp(
                &mut s,
                (x - (width - 1) / 2, 10 + down),
                rows,
                &glass_inks(ramp),
                false,
            );
        }
    }
    // The glass catching the sun.
    sparkle(&mut s, (5, 14), 1);
    sparkle(&mut s, (21, 19), 1);
    stamp(&mut s, (7, ground - 3), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (13, ground - 2), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (12, ground),
    }
}

/// A stone birdbath, brimming, with a mussel shell left on its rim.
fn birdbath() -> Piece {
    let mut s = Canvas::new(24, 28);
    let (cx, ground) = (12, 25);
    ellipse(&mut s, cx, ground, 9, 2, SHADOW);
    // The foot: a low drum of stone.
    model(&mut s, STONE, UPRIGHT, 36, 61, |x, y| {
        in_ellipse(x, y, (cx, ground - 1), (6, 2))
            || in_ellipse(x, y, (cx, ground - 4), (6, 2))
            || ((cx - 6..=cx + 6).contains(&x) && (ground - 4..=ground - 1).contains(&y))
    });
    for y in ground - 5..=ground - 3 {
        for x in cx - 5..=cx + 5 {
            if in_ellipse(x, y, (cx, ground - 4), (5, 1)) {
                put(&mut s, x, y, speckled(STONE.light, STONE.base, x, y, 62));
            }
        }
    }
    // The stem, waisted, with a ring where it meets the foot.
    model(&mut s, STONE, UPRIGHT, 36, 63, |x, y| {
        let half = match y {
            11..=13 => 3,
            14..=18 => 2,
            19..=21 => 3,
            _ => return false,
        };
        (cx - half..=cx + half).contains(&x)
    });
    // The bowl, its underside curving in to the stem, its rim lit, and the water in it.
    model(&mut s, STONE, ROUND, 36, 64, |x, y| {
        y >= 7 && in_ellipse(x, y, (cx, 7), (10, 5))
    });
    model(&mut s, STONE, LYING, 20, 65, |x, y| {
        in_ellipse(x, y, (cx, 6), (10, 3))
    });
    for y in 3..=9 {
        for x in cx - 9..=cx + 9 {
            if !in_ellipse(x, y, (cx, 6), (8, 2)) {
                continue;
            }
            let color = if !in_ellipse(x, y - 1, (cx, 6), (8, 2)) {
                // The inside of the far wall, in its own shade.
                STONE.shadow
            } else if y <= 5 {
                WATER.shadow
            } else if chance(x, y, 66, 90) {
                WATER.light
            } else {
                WATER.base
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, cx - 3, 6, WATER.shine);
    put(&mut s, cx - 2, 6, WATER.light);
    put(&mut s, cx + 4, 7, WATER.light);
    // The mussel shell, open on the rim.
    stamp(&mut s, (cx + 6, 2), &RIM_MUSSEL, &mussel_inks(), false);
    // Moss creeping up out of the grass.
    for y in ground - 6..=ground {
        for x in cx - 6..=cx + 6 {
            let solid = s.get(x, y).a == 255;
            if solid && chance(x, y, 67, 40 + 12 * (y - ground + 6) as u32) {
                let mossy = mix(MOSS, s.get(x, y), 0.3);
                put(&mut s, x, y, mossy);
            }
        }
    }
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A mussel shell lying open, as small as it can be and still read.
const RIM_MUSSEL: [&str; 3] = [".##..", "#*nM.", ".#MM#"];

fn mussel_inks() -> Vec<(u8, Rgba)> {
    [letters(MUSSEL, b"#.M.."), letters(NACRE, b"..n.*")].concat()
}

/// Where each bottle on the bottle tree hangs: the path of its branch from the trunk, the bottle
/// upended on the end of it.
type Branch = (&'static [(i32, i32)], Ramp);
const BRANCHES: [Branch; 6] = [
    (&[(16, 39), (11, 36), (7, 33), (5, 31), (5, 29)], COBALT),
    (&[(18, 36), (23, 33), (27, 31), (29, 29), (29, 27)], AMBER),
    (&[(16, 30), (12, 26), (9, 23), (8, 20)], BOTTLE_GREEN),
    (&[(18, 26), (22, 22), (25, 18), (26, 16)], COBALT),
    (&[(16, 20), (13, 16), (11, 14), (11, 12)], AMBER),
    (&[(17, 14), (17, 11)], BOTTLE_GREEN),
];

/// A bottle upended, its neck pushed down over a branch tip, lit on its left.
const BOTTLE: [&str; 9] = [
    ".###.", "#*os#", "#los#", "#los#", "#los#", "#oos#", ".#o#.", ".#l#.", ".###.",
];

/// A bare branching stick with coloured glass bottles on the ends of its branches.
fn bottle_tree() -> Piece {
    let mut s = Canvas::new(34, 50);
    let (cx, ground) = (17, 47);
    ellipse(&mut s, cx, ground, 10, 2, SHADOW);
    // Twigs that never got a bottle, behind the rest.
    bough(&mut s, &[(16, 34), (13, 32), (12, 30)], DRIFTWOOD);
    bough(&mut s, &[(18, 42), (21, 40), (22, 38)], DRIFTWOOD);
    bough(&mut s, &[(23, 21), (27, 21), (29, 19)], DRIFTWOOD);
    bough(&mut s, &[(17, 17), (21, 12), (22, 9)], DRIFTWOOD);
    bough(&mut s, &[(10, 24), (7, 25)], DRIFTWOOD);
    for &(path, _) in &BRANCHES {
        bough(&mut s, path, DRIFTWOOD);
    }
    model(&mut s, DRIFTWOOD, UPRIGHT, 40, 71, |x, y| {
        let (left, right) = match y {
            12..=21 => (16, 17),
            22..=43 => (16, 18),
            44..=47 => (15, 19),
            _ => return false,
        };
        (left..=right).contains(&x)
    });
    for &(path, ramp) in &BRANCHES {
        if let Some(&tip) = path.last() {
            stamp(
                &mut s,
                (tip.0 - 2, tip.1 - 9),
                &BOTTLE,
                &glass_inks(ramp),
                false,
            );
        }
    }
    stamp(&mut s, (11, ground - 3), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (19, ground - 2), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A small pond with a stone rim, reeds at the back, and the wooden duck afloat.
fn pond() -> Piece {
    let mut s = Canvas::new(46, 26);
    let (cx, cy) = (23, 17);
    ellipse(&mut s, cx + 1, cy + 2, 22, 5, SHADOW);
    // Reeds at the back, rising from behind the rim.
    reed_clump(
        &mut s,
        (36, cy - 5),
        &[
            (-3, -5, 10),
            (-1, -2, 14),
            (1, 0, 16),
            (3, 4, 12),
            (4, 6, 7),
        ],
    );
    cattail(&mut s, (34, cy - 5), 15);
    cattail(&mut s, (38, cy - 5), 12);
    reed_clump(&mut s, (8, cy - 4), &[(-1, -4, 8), (0, -1, 11), (2, 2, 7)]);
    // The water: the far bank darkening it at the back, the sky lightening it at the front, and
    // the reeds standing in it upside down.
    for y in cy - 5..=cy + 5 {
        for x in cx - 18..=cx + 18 {
            if !in_ellipse(x, y, (cx, cy), (18, 4)) {
                continue;
            }
            let depth = (y - cy + 5) as f32 + (noise(x, y, 81) & 0xff) as f32 / 128.0 - 1.0;
            let reflected = (32..=40).contains(&x) && y < cy + 1 && chance(x, 0, 84, 140);
            let color = match depth {
                _ if reflected => mix(WATER.shadow, REED.shadow, 0.5),
                d if d < 2.5 => WATER.shadow,
                d if d < 6.5 => WATER.base,
                _ => WATER.light,
            };
            put(&mut s, x, y, color);
        }
    }
    for (x, y, length) in [
        (25, cy - 1, 4),
        (30, cy + 2, 3),
        (9, cy + 1, 3),
        (18, cy + 3, 5),
    ] {
        for dx in 0..length {
            put(&mut s, x + dx, y, WATER.light);
        }
        put(&mut s, x + 1, y, WATER.shine);
    }
    stamp(&mut s, (26, cy - 3), &LILY_PAD, &lily_inks(), false);
    stamp(&mut s, (31, cy + 1), &LILY_PAD, &lily_inks(), true);
    stamp(
        &mut s,
        (28, cy - 5),
        &[".p.", "pPp"],
        &[(b'p', rgb(0xe88aa0)), (b'P', rgb(0xfbd0d8))],
        false,
    );
    // The rim, back to front.
    let mut stones: Vec<Stone> = (0..16)
        .map(|index| {
            let turn = index as f32 / 16.0 + (noise(index, 0, 82) & 0xff) as f32 / 255.0 / 40.0;
            let (sin, cos) = (turn * TAU).sin_cos();
            let at = (
                cx + (cos * 19.5).round() as i32,
                cy + (sin * 5.0).round() as i32,
            );
            let size = (3 - index % 2, if sin < -0.3 { 1 } else { 2 });
            let ramp = [STONE, SANDSTONE, STONE][(noise(index, 1, 83) % 3) as usize];
            (at, size, ramp)
        })
        .collect();
    stones.sort_by_key(|(at, _, _)| at.1);
    for (index, &(at, size, ramp)) in stones.iter().enumerate() {
        model(&mut s, ramp, ROUND, 30, 90 + index as u32, |x, y| {
            in_ellipse(x, y, at, size)
        });
    }
    // The duck, afloat, with the water rippling round it.
    stamp(&mut s, (12, cy - 7), &DUCK[1..7], &duck_inks(), false);
    for x in 11..=22 {
        put(
            &mut s,
            x,
            cy,
            if x % 4 == 1 { WATER.shine } else { WATER.light },
        );
    }
    Piece {
        sprite: s,
        anchor: (cx, cy + 2),
    }
}

const LILY_PAD: [&str; 2] = [".#lol#", "#sooos#"];

fn lily_inks() -> Vec<(u8, Rgba)> {
    letters(LILY, b"#sol*").to_vec()
}

/// A split geode on a stone plinth, its purple crystals catching the light.
fn geode_plinth() -> Piece {
    let mut s = Canvas::new(24, 30);
    let (cx, ground) = (12, 27);
    ellipse(&mut s, cx, ground, 10, 2, SHADOW);
    slab(&mut s, (2, 23), (20, 5), 2, PLINTH, 101);
    slab(&mut s, (5, 16), (14, 8), 0, PLINTH, 102);
    slab(&mut s, (3, 12), (18, 5), 2, PLINTH, 103);
    // A panel carved into the front: shaded at its top and left, lit at its bottom and right.
    for x in 8..=15 {
        put(&mut s, x, 18, PLINTH.shadow);
        put(&mut s, x, 21, PLINTH.light);
    }
    for y in 18..=21 {
        put(&mut s, 8, y, PLINTH.shadow);
        put(&mut s, 15, y, PLINTH.light);
    }
    // The geode, sitting on its rounded back with the face it was split along tipped up to
    // show: first the rough underside, then the face.
    let (gx, gy) = (cx, 6);
    model(&mut s, GEODE_SHELL, ROUND, 70, 104, |x, y| {
        y >= gy && in_ellipse(x, y, (gx, gy + 3), (7, 5))
    });
    let (rx, ry) = (7.0, 5.0);
    let inside = |x: i32, y: i32| in_ellipse(x, y, (gx, gy), (7, 5));
    for y in gy - 6..=gy + 6 {
        for x in gx - 8..=gx + 8 {
            if !inside(x, y) {
                continue;
            }
            let rim =
                !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            let (nx, ny) = ((x - gx) as f32 / rx, (y - gy) as f32 / ry);
            let reach = (nx * nx + ny * ny).sqrt();
            let jitter = (noise(x, y, 105) & 0xff) as f32 / 255.0 - 0.5;
            let color = if rim {
                GEODE_SHELL.edge
            } else if reach > 0.8 {
                speckled(GEODE_SHELL.light, GEODE_SHELL.base, x, y, 106)
            } else if reach > 0.66 {
                AGATE
            } else if reach > 0.56 {
                mix(AGATE, AMETHYST.light, 0.6)
            } else {
                // Hollow, so lit the other way about: the far wall catches the light.
                let lit = 0.6 * nx + 0.5 * ny + 0.5 * jitter;
                if chance(x, y, 107, 40) {
                    AMETHYST.shine
                } else if lit > 0.25 {
                    AMETHYST.light
                } else if lit > -0.2 {
                    AMETHYST.base
                } else {
                    AMETHYST.shadow
                }
            };
            put(&mut s, x, y, color);
        }
    }
    sparkle(&mut s, (gx + 2, gy + 1), 2);
    sparkle(&mut s, (gx - 2, gy + 2), 1);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// The star stone: a pale standing stone with the fallen star set glowing in its heart.
fn star_stone() -> Piece {
    let mut s = Canvas::new(42, 50);
    let (cx, ground) = (21, 46);
    let heart = (21, 20);
    // The glow, spread round the heart on the air.
    for (radius, alpha) in [(20, 16), (16, 20), (12, 28), (8, 40)] {
        for y in heart.1 - radius..=heart.1 + radius {
            for x in heart.0 - radius..=heart.0 + radius {
                if in_ellipse(x, y, heart, (radius, radius)) {
                    put(&mut s, x, y, rgba(0xffe7a0, alpha));
                }
            }
        }
    }
    ellipse(&mut s, cx + 1, ground, 12, 3, SHADOW);
    // The stone: broad shouldered, a little lopsided, wider at the foot.
    let mut outline = Canvas::new(42, 50);
    polygon(
        &mut outline,
        &[
            (13, 47),
            (12, 38),
            (13, 27),
            (14, 17),
            (15, 11),
            (18, 7),
            (22, 5),
            (26, 6),
            (28, 9),
            (30, 15),
            (30, 27),
            (31, 38),
            (31, 47),
        ],
        |_, _| Some(rgb(0xffffff)),
    );
    let stone = |x: i32, y: i32| outline.get(x, y).a > 0;
    model(&mut s, MOONSTONE, (0.75, 0.3), 30, 111, stone);
    // A ring carved round the heart long ago, and cup marks below it: each groove in shade on
    // its upper side and catching the light on its lower.
    for step in 0..48 {
        let (sin, cos) = (step as f32 / 48.0 * TAU).sin_cos();
        let (x, y) = (
            heart.0 + (cos * 7.0).round() as i32,
            heart.1 + (sin * 7.0).round() as i32,
        );
        if stone(x, y) && stone(x, y + 1) {
            put(&mut s, x, y, MOONSTONE.shadow);
            if sin > -0.2 {
                put(&mut s, x, y + 1, MOONSTONE.shine);
            }
        }
    }
    for (x, y) in [(18, 32), (24, 35), (20, 39)] {
        put(&mut s, x, y, MOONSTONE.edge);
        put(&mut s, x + 1, y, MOONSTONE.shadow);
        put(&mut s, x, y + 1, MOONSTONE.light);
        put(&mut s, x + 1, y + 1, MOONSTONE.shine);
    }
    // Its face warmed by the star, more so nearer it.
    for y in heart.1 - 10..=heart.1 + 10 {
        for x in heart.0 - 10..=heart.0 + 10 {
            let far = (((x - heart.0).pow(2) + (y - heart.1).pow(2)) as f32).sqrt();
            if stone(x, y) && far < 10.0 {
                let warmth = (1.0 - far / 10.0) * 0.65;
                let warmed = mix(s.get(x, y), STARLIGHT.light, warmth);
                put(&mut s, x, y, warmed);
            }
        }
    }
    // Lichen in patches on the shaded side, and moss where it meets the grass.
    for (px, py, r) in [(27, 30, 2), (28, 40, 3), (25, 13, 1)] {
        for y in py - r..=py + r {
            for x in px - r..=px + r {
                let edge = s.get(x, y) == MOONSTONE.edge;
                let patch = in_ellipse(x, y, (px, py), (r, r));
                if stone(x, y) && !edge && patch && chance(x, y, 113, 190) {
                    put(&mut s, x, y, LICHEN);
                }
            }
        }
    }
    for y in 41..=47 {
        for x in 12..=31 {
            let edge = s.get(x, y) == MOONSTONE.edge;
            if stone(x, y) && !edge && chance(x, y, 114, 40 + 25 * (y - 41) as u32) {
                put(
                    &mut s,
                    x,
                    y,
                    if chance(x, y, 115, 90) { LEAFY } else { MOSS },
                );
            }
        }
    }
    stamp(
        &mut s,
        (heart.0 - 4, heart.1 - 4),
        &STAR,
        &star_inks(),
        false,
    );
    // Stones at its foot, and grass and flowers growing up round it.
    flat_stone(&mut s, (9, 45), (3, 1), 1, SLATE, 116);
    flat_stone(&mut s, (34, 46), (2, 1), 1, MOONSTONE, 117);
    stamp(&mut s, (14, ground - 2), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (26, ground - 1), &TUFT, &tuft_inks(), true);
    stamp(&mut s, (10, ground - 5), &DAISY, &daisy_inks(), false);
    stamp(&mut s, (31, ground - 4), &DAISY, &daisy_inks(), false);
    // Sparkles in the air about it.
    for (at, size) in [
        ((6, 13), 2),
        ((35, 8), 2),
        ((35, 26), 1),
        ((8, 29), 1),
        ((15, 3), 1),
    ] {
        sparkle(&mut s, at, size);
    }
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

fn star_inks() -> Vec<(u8, Rgba)> {
    letters(STARLIGHT, b"#sol*").to_vec()
}

// ---------------------------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------------------------

/// A flat stone lying in the grass: its face lit from the upper left, `thick` rows of its side
/// in shade beneath, outlined in its own darkest tone.
fn flat_stone(s: &mut Canvas, at: (i32, i32), size: (i32, i32), thick: i32, ramp: Ramp, salt: u32) {
    let face = |x: i32, y: i32| in_ellipse(x, y, at, size);
    let solid = |x: i32, y: i32| (0..=thick).any(|down| in_ellipse(x, y - down, at, size));
    for y in at.1 - size.1 - 1..=at.1 + size.1 + thick + 1 {
        for x in at.0 - size.0 - 1..=at.0 + size.0 + 1 {
            if !solid(x, y) {
                continue;
            }
            let rim = !solid(x - 1, y) || !solid(x + 1, y) || !solid(x, y - 1) || !solid(x, y + 1);
            let color = if rim {
                ramp.edge
            } else if !face(x, y) {
                ramp.shadow
            } else {
                let (nx, ny) = (
                    (x - at.0) as f32 / size.0 as f32,
                    (y - at.1) as f32 / size.1 as f32,
                );
                let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
                let lit = -0.6 * nx - 0.5 * ny + 0.5 * jitter;
                if chance(x, y, salt + 1, 40) {
                    ramp.shadow
                } else if lit > 0.75 {
                    ramp.shine
                } else if lit > 0.3 {
                    ramp.light
                } else {
                    ramp.base
                }
            };
            put(s, x, y, color);
        }
    }
}

/// A box of dressed stone seen from the front, its top face showing `top` rows deep.
fn slab(s: &mut Canvas, at: (i32, i32), size: (i32, i32), top: i32, ramp: Ramp, salt: u32) {
    let (x0, y0, w, h) = (at.0, at.1, size.0, size.1);
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let rim = x == x0 || y == y0 || x == x0 + w - 1 || y == y0 + h - 1;
            let color = if rim {
                ramp.edge
            } else if y <= y0 + top {
                speckled(ramp.light, ramp.shine, x, y, salt)
            } else if x == x0 + w - 2 || y == y0 + h - 2 {
                ramp.shadow
            } else if x == x0 + 1 {
                ramp.light
            } else {
                speckled(ramp.base, ramp.shadow, x, y, salt)
            };
            put(s, x, y, color);
        }
    }
}

/// A clump of reeds from `base`: each blade's lean at the foot, lean at the tip, and height.
fn reed_clump(s: &mut Canvas, base: (i32, i32), blades: &[(i32, i32, i32)]) {
    for &(foot, tip, height) in blades {
        let points = trace(&[(base.0 + foot, base.1), (base.0 + tip, base.1 - height)]);
        for (index, &(x, y)) in points.iter().enumerate() {
            let up = index as f32 / points.len() as f32;
            put(s, x + 1, y, REED.edge);
            put(
                s,
                x,
                y,
                if up > 0.75 {
                    REED.light
                } else if up > 0.35 {
                    REED.base
                } else {
                    REED.shadow
                },
            );
        }
    }
}

/// A bulrush: a stalk with its brown head near the top.
fn cattail(s: &mut Canvas, base: (i32, i32), height: i32) {
    for y in base.1 - height..=base.1 {
        put(s, base.0, y, REED.shadow);
        put(s, base.0 + 1, y, REED.edge);
    }
    stamp(
        s,
        (base.0 - 1, base.1 - height + 2),
        &[".#.", "#l#", "#o#", "#o#", "#s#", ".#."],
        &letters(CATTAIL, b"#sol*"),
        false,
    );
}

/// A four-pointed glint, its arms fading out `size` pixels.
fn sparkle(s: &mut Canvas, at: (i32, i32), size: i32) {
    put(s, at.0, at.1, rgba(0xffffff, 240));
    for step in 1..=size {
        let alpha = (200 / step) as u8;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(s, at.0 + dx * step, at.1 + dy * step, rgba(0xfff6d8, alpha));
        }
    }
}

/// `base`, with about one pixel in five swapped for `fleck`.
fn speckled(base: Rgba, fleck: Rgba, x: i32, y: i32, salt: u32) -> Rgba {
    if chance(x, y, salt, 48) { fleck } else { base }
}

/// A daisy in the grass.
const DAISY: [&str; 3] = [".w.", "wyw", ".w."];

fn daisy_inks() -> Vec<(u8, Rgba)> {
    vec![(b'w', rgb(0xfbf6ee)), (b'y', rgb(0xf0c040))]
}

/// Glass: its outline solid, the rest a little see-through.
fn glass_inks(ramp: Ramp) -> Vec<(u8, Rgba)> {
    let clear = |color: Rgba| Rgba::new(color.r, color.g, color.b, 225);
    vec![
        (b'#', ramp.edge),
        (b's', clear(ramp.shadow)),
        (b'o', clear(ramp.base)),
        (b'l', clear(ramp.light)),
        (b'*', ramp.shine),
    ]
}
