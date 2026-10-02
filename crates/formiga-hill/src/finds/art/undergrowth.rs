//! The finds in the undergrowth: the finds shaken out (Kind::Shake): their icons and their Hilltop pieces.
//! Growing things mostly, in leaf greens with a berry, a flower or a butterfly for colour, and
//! two made things the brambles caught: a kite, and an old copper fox.

use super::brush::*;
use super::{ICON, Piece};
use crate::paint::{Ramp, blit, chance, ellipse, mix, noise, polygon, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::{PI, TAU};

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    let (rows, inks): (&[&str; 9], Vec<(u8, Rgba)>) = match id {
        "sycamore_key" => (
            &SYCAMORE_KEY,
            [
                &letters(KEY_WING, b"#sol*")[..],
                &letters(KEY_SEED, b"S.k.."),
            ]
            .concat(),
        ),
        "wild_berries" => (
            &WILD_BERRIES,
            [
                &letters(BERRY_RED, b"Rrbh*")[..],
                &letters(LEAF, b"Lgfl."),
                &[(b'K', STALK)],
            ]
            .concat(),
        ),
        "dandelion_clock" => (
            &DANDELION_CLOCK,
            [&letters(CLOCK, b"#sol*")[..], &letters(STEM, b"Gg...")].concat(),
        ),
        "strange_seed" => (
            &STRANGE_SEED,
            [
                &letters(ODD_TEAL, b"T.tu*")[..],
                &letters(ODD_VIOLET, b"V.vx."),
                &[(b'g', rgba(0x8af0e0, 80))],
            ]
            .concat(),
        ),
        "tangled_kite" => (
            &TANGLED_KITE,
            [
                &letters(KITE_RED, b"R.r..")[..],
                &letters(KITE_YELLOW, b"Y.yh."),
                &letters(BOW_BLUE, b"B.b.."),
                &[(b'k', STRING)],
            ]
            .concat(),
        ),
        "silk_cocoon" => (
            &SILK_COCOON,
            [
                &letters(SILK, b"#sol*")[..],
                &[(b't', SILK_THREAD), (b'k', STALK)],
            ]
            .concat(),
        ),
        "weathervane" => (
            &FOX_ICON,
            [
                &letters(COPPER, b"#scC.")[..],
                &[
                    (b'v', VERDIGRIS.base),
                    (b'V', VERDIGRIS.light),
                    (b'e', COPPER.edge),
                ],
            ]
            .concat(),
        ),
        _ => return None,
    };
    let mut icon = Canvas::new(ICON, ICON);
    stamp(&mut icon, (0, 0), rows, &inks, false);
    Some(icon)
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "sycamore_key" => sycamore_sapling(),
        "wild_berries" => berry_bush(),
        "dandelion_clock" => dandelion_patch(),
        "strange_seed" => strange_sapling(),
        "tangled_kite" => kite_post(),
        "silk_cocoon" => butterfly_bush(),
        "weathervane" => fox_weathervane(),
        _ => return None,
    })
}

const LEAF: Ramp = Ramp::new(0x2c5233, 0x3d6e45, 0x518a55, 0x6fa866, 0x93c67e);
const DEEP_LEAF: Ramp = Ramp::new(0x22422a, 0x30583a, 0x41704a, 0x588c58, 0x7aa86c);
const SYCAMORE: Ramp = Ramp::new(0x2a5a2a, 0x3c7638, 0x529444, 0x74b456, 0x9ed078);
const SAGE: Ramp = Ramp::new(0x34503e, 0x4a6a52, 0x648a6a, 0x84a884, 0xa8c8a4);
const BARK: Ramp = Ramp::new(0x4a4038, 0x6a5c50, 0x8a7a6a, 0xa89888, 0xc4b6a4);
const KEY_WING: Ramp = Ramp::new(0x7a4a2a, 0xa8683a, 0xc89058, 0xe0b47a, 0xf4dcb0);
const KEY_SEED: Ramp = Ramp::new(0x5a3a1e, 0x7a5028, 0x96683a, 0xb08250, 0xc8a070);
const BERRY_RED: Ramp = Ramp::new(0x6a1a22, 0x9a2430, 0xd03a3a, 0xf06a5a, 0xffd0c0);
const BERRY_DARK: Ramp = Ramp::new(0x1e1430, 0x2e2048, 0x46306a, 0x6a5090, 0xc0b0e0);
const CLOCK: Ramp = Ramp::new(0xa8a49a, 0xd4d0c4, 0xece8de, 0xf8f6f0, 0xffffff);
const STEM: Ramp = Ramp::new(0x3e6a32, 0x4e8040, 0x62984e, 0x80b466, 0xa4d088);
const PETAL: Ramp = Ramp::new(0xb07a18, 0xd89a20, 0xf5c430, 0xffe070, 0xfff4c0);
const ODD_TEAL: Ramp = Ramp::new(0x1a4a50, 0x226a6e, 0x2e9090, 0x5cbcb4, 0xc0fff0);
const ODD_VIOLET: Ramp = Ramp::new(0x3a2058, 0x54307e, 0x7448a8, 0x9a70cc, 0xe0c8ff);
const ODD_BARK: Ramp = Ramp::new(0x3e2e3e, 0x5a4058, 0x7a5a74, 0x9a7a90, 0xc0a2b4);
const KITE_RED: Ramp = Ramp::new(0x7a1e24, 0xa82e30, 0xd04a40, 0xe87058, 0xffb8a0);
const KITE_YELLOW: Ramp = Ramp::new(0xa07418, 0xd0a020, 0xf2c838, 0xffe27a, 0xfff6c8);
const BOW_BLUE: Ramp = Ramp::new(0x1e3a78, 0x2c54a0, 0x3e70c4, 0x6a96dc, 0xb0ccf4);
const POST: Ramp = Ramp::new(0x5a3e2a, 0x7a5638, 0x9a7048, 0xb88c60, 0xd4ac80);
const SILK: Ramp = Ramp::new(0xa89a7a, 0xd8ccaa, 0xece2c6, 0xf8f2e2, 0xffffff);
const BUDDLEIA: Ramp = Ramp::new(0x4e2c78, 0x7448a8, 0x9a6ac8, 0xc09ae0, 0xe6d4fa);
const VERDIGRIS: Ramp = Ramp::new(0x24503f, 0x3a7a64, 0x58a088, 0x84c4a8, 0xbce8d4);
const COPPER: Ramp = Ramp::new(0x5a2a14, 0x8a4220, 0xb86430, 0xd8884a, 0xf4c088);
const WROUGHT: Ramp = Ramp::new(0x2e3a3c, 0x46545a, 0x5e6e74, 0x7c8c90, 0xa4b2b4);
const FOOTING: Ramp = Ramp::new(0x605a5d, 0x857d7c, 0xa69d94, 0xc2baae, 0xdcd5c8);
const STALK: Rgba = rgb(0x5a6a34);
const STRING: Rgba = rgb(0x8a7a64);
const SILK_THREAD: Rgba = rgba(0xd8d0bc, 200);

// ---------------------------------------------------------------------------------------------
// Icons
// ---------------------------------------------------------------------------------------------

/// A pair of winged seeds, joined where they grew.
const SYCAMORE_KEY: [&str; 9] = [
    "###...###",
    "#*l#.#lo#",
    "#lo#.#os#",
    ".#lo#os#.",
    ".#lo#os#.",
    "..#SkS#..",
    "..#kSk#..",
    "...###...",
    ".........",
];

/// A sprig of berries with a leaf on its stalk.
const WILD_BERRIES: [&str; 9] = [
    ".....LLL.",
    "....LfggL",
    "...K.LLL.",
    "..K......",
    ".RRKRR...",
    "RhrRhrR..",
    "RrbRrbR..",
    ".RRhrRR..",
    "..RRbR...",
];

/// A clock of seeds on its stalk, not blown yet.
const DANDELION_CLOCK: [&str; 9] = [
    "..#.#.#..",
    ".#*lllo#.",
    "#llloooo#",
    ".lloooos.",
    "#looooss#",
    ".#ooss#..",
    "..#.g.#..",
    "....g....",
    "...Gg....",
];

/// A seed in stripes, faintly glowing.
const STRANGE_SEED: [&str; 9] = [
    "...ggg...",
    "..gTtTg..",
    ".gVxvvVg.",
    ".Tu*ttTg.",
    ".VxvvvVg.",
    ".TutttTg.",
    ".gVxvVg..",
    "..gTtTg..",
    "...ggg...",
];

/// A diamond kite with a bow on its tail.
const TANGLED_KITE: [&str; 9] = [
    ".....Y...",
    "....YhR..",
    "...YhyrR.",
    "..YyyrrrR",
    "...RrrYy.",
    "....RrY..",
    ".....k...",
    "..BbBk...",
    "...k.....",
];

/// A silk cocoon hanging from its twig.
const SILK_COCOON: [&str; 9] = [
    "kkkkkkkk.",
    "....t....",
    "....#....",
    "...#l#...",
    "..#*lo#t.",
    "..#lso#..",
    ".t#los#..",
    "..#oss#..",
    "...###...",
];

/// The copper fox off the weathervane, green with age.
const FOX_ICON: [&str; 9] = [
    ".........",
    "...#...#.",
    "..#c#.#V#",
    ".#cCc##c#",
    "#Ceccccv#",
    ".##cccvs#",
    "..#c##c#.",
    ".##...##.",
    ".........",
];

// ---------------------------------------------------------------------------------------------
// Pieces
// ---------------------------------------------------------------------------------------------

/// A pair of keys hanging by their stalk, wings down.
const HANGING_KEYS: [&str; 5] = ["..k..", ".SkS.", "#l.o#", "#o.s#", "##.##"];

/// Where each leaf of the sapling grows, how big it is, which way it points, and whether it is
/// one of the far ones, darker for being in the others' shade.
const SYCAMORE_LEAVES: [((i32, i32), f32, f32, bool); 9] = [
    ((5, 15), 4.0, -0.33, true),
    ((22, 12), 4.0, -0.17, true),
    ((13, 4), 4.0, -0.25, true),
    ((6, 22), 5.0, -0.37, false),
    ((22, 20), 5.0, -0.13, false),
    ((8, 10), 5.0, -0.31, false),
    ((19, 7), 5.0, -0.2, false),
    ((14, 16), 5.0, -0.27, false),
    ((14, 9), 4.0, -0.24, false),
];

/// A sycamore sapling: broad leaves on a thin trunk, a couple of keys hanging under them.
fn sycamore_sapling() -> Piece {
    let mut s = Canvas::new(28, 44);
    let (cx, ground) = (14, 41);
    ellipse(&mut s, cx, ground, 8, 2, SHADOW);
    ellipse(&mut s, cx + 1, ground, 5, 1, SHADOW);
    // The twigs that carry the leaves, then the trunk, with a slight lean.
    for path in [
        &[(13, 27), (9, 24), (6, 22)][..],
        &[(14, 25), (19, 22), (22, 20)],
        &[(13, 20), (10, 15), (8, 10)],
        &[(14, 17), (17, 11), (19, 7)],
        &[(13, 18), (7, 16), (5, 15)],
        &[(14, 15), (20, 13), (22, 12)],
    ] {
        bough(&mut s, path, BARK);
    }
    model(&mut s, BARK, UPRIGHT, 30, 201, |x, y| {
        let lean = if y < 26 { 1 } else { 0 };
        let width = if y > 36 { 3 } else { 2 };
        (11..=41).contains(&y) && (cx - 1 + lean..cx - 1 + lean + width).contains(&x)
    });
    for (index, &(at, size, turn, far)) in SYCAMORE_LEAVES.iter().enumerate() {
        let ramp = if far { DEEP_LEAF } else { SYCAMORE };
        palmate(&mut s, at, size, turn, ramp, 202 + index as u32);
    }
    // Keys, hanging under the leaves.
    let keys = [
        &letters(KEY_WING, b"#sol*")[..],
        &letters(KEY_SEED, b"S.k.."),
    ]
    .concat();
    stamp(&mut s, (3, 26), &HANGING_KEYS, &keys, false);
    stamp(&mut s, (19, 25), &HANGING_KEYS, &keys, true);
    stamp(&mut s, (9, ground - 2), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (15, ground - 1), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A broad leaf of five pointed lobes spread like a hand from its stalk, the middle one pointing
/// towards `turn`: lit from the upper left, its veins paler, outlined in its own darkest green.
fn palmate(s: &mut Canvas, centre: (i32, i32), size: f32, turn: f32, ramp: Ramp, salt: u32) {
    let base = (centre.0 as f32 + 0.5, centre.1 as f32 + 0.5);
    let lobes: Vec<Blade> = [
        (0.0, 1.0, 0.5),
        (-0.12, 0.85, 0.45),
        (0.12, 0.85, 0.45),
        (-0.25, 0.6, 0.38),
        (0.25, 0.6, 0.38),
    ]
    .iter()
    .map(|&(offset, reach, width)| Blade::new(base, turn + offset, size * reach, size * width))
    .collect();
    let shape = |t: f32| (1.0 - t).sqrt() * (0.5 + t).min(1.0);
    let on = |x: i32, y: i32| {
        (x - centre.0).abs() + (y - centre.1).abs() <= 1
            || lobes.iter().any(|lobe| lobe.at(x, y, &shape).is_some())
    };
    let reach = size.ceil() as i32 + 1;
    for y in centre.1 - reach..=centre.1 + reach {
        for x in centre.0 - reach..=centre.0 + reach {
            if !on(x, y) {
                continue;
            }
            let rim = !on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1);
            if rim {
                put(s, x, y, ramp.edge);
                continue;
            }
            let lit = -((x - centre.0) + (y - centre.1)) as f32 / (size * 1.2);
            let vein = lobes.iter().any(|lobe| {
                lobe.at(x, y, &shape)
                    .is_some_and(|(t, across, _)| across.abs() < 0.3 && t > 0.1 && t < 0.8)
            });
            let color = tone(ramp, lit, x, y, salt);
            put(
                s,
                x,
                y,
                if vein {
                    mix(color, ramp.shine, 0.35)
                } else {
                    color
                },
            );
        }
    }
}

/// A round bush with red berries and dark ones.
fn berry_bush() -> Piece {
    let mut s = Canvas::new(32, 26);
    let (cx, ground) = (16, 22);
    ellipse(&mut s, cx, ground, 14, 3, SHADOW);
    foliage(
        &mut s,
        &[
            ((8, 16), (7, 5)),
            ((24, 16), (7, 5)),
            ((12, 9), (7, 6)),
            ((21, 9), (6, 6)),
            ((16, 15), (10, 6)),
        ],
        LEAF,
        211,
    );
    // Berries, mostly on the sunny side, in little bunches.
    let red = letters(BERRY_RED, b"#sol*");
    let dark = letters(BERRY_DARK, b"#sol*");
    for (index, &(x, y)) in [
        (7, 9),
        (13, 6),
        (5, 15),
        (11, 13),
        (19, 7),
        (23, 13),
        (16, 17),
        (27, 16),
        (9, 19),
        (21, 19),
    ]
    .iter()
    .enumerate()
    {
        let inks = if index % 3 == 2 { &dark } else { &red };
        stamp(&mut s, (x, y), &BERRY, inks, false);
        if index % 2 == 0 {
            stamp(&mut s, (x + 2, y + 1), &BERRY, inks, false);
        }
    }
    // A couple dropped in the grass.
    stamp(&mut s, (4, ground - 1), &BERRY, &red, false);
    stamp(&mut s, (25, ground), &BERRY, &dark, false);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// One berry, round and shiny.
const BERRY: [&str; 3] = [".#.", "#*o", ".os"];

/// A low patch of dandelions: flowers open, and clocks gone to seed.
fn dandelion_patch() -> Piece {
    let mut s = Canvas::new(30, 20);
    let (cx, ground) = (15, 16);
    ellipse(&mut s, cx, ground, 13, 2, SHADOW);
    // The toothed leaves, lying out flat from each plant.
    let teeth = |t: f32| {
        let saw = if (t * 9.0) as i32 % 2 == 0 { 1.0 } else { 0.55 };
        (PI * t).sin().powf(0.6) * saw
    };
    for &(base, turn, length) in &[
        ((8.0, 15.5), 0.47, 7.0),
        ((8.0, 15.5), 0.03, 6.0),
        ((8.0, 15.5), 0.58, 5.0),
        ((20.0, 15.5), 0.52, 6.0),
        ((20.0, 15.5), 0.97, 8.0),
        ((20.0, 15.5), 0.9, 5.0),
        ((14.0, 16.5), 0.45, 5.0),
        ((14.0, 16.5), 0.05, 5.0),
    ] {
        blade(
            &mut s,
            Blade::new(base, turn, length, 2.2),
            teeth,
            |_, facing, x, y| tone(STEM, 0.4 * facing + 0.15, x, y, 221),
        );
    }
    // Stalks, then the flowers and clocks on them.
    let stalks = [
        (5, 11, false),
        (10, 9, false),
        (13, 5, true),
        (18, 12, false),
        (22, 6, true),
        (25, 10, false),
    ];
    for &(x, top, _) in &stalks {
        for y in top..ground - 1 {
            put(
                &mut s,
                x,
                y,
                if y % 3 == 0 { STEM.base } else { STEM.shadow },
            );
        }
    }
    let petals = letters(PETAL, b"#sol*");
    for &(x, top, clock) in &stalks {
        if clock {
            seed_clock(&mut s, (x, top - 1), x == 22);
        } else {
            stamp(&mut s, (x - 2, top - 2), &FLOWER, &petals, false);
        }
    }
    // A few seeds already away on the breeze.
    for &(x, y) in &[(26, 1), (28, 4), (18, 0)] {
        stamp(&mut s, (x, y), &[".w.", "w.w", ".k."], &seed_inks(), false);
    }
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A dandelion flower head, open to the sun.
const FLOWER: [&str; 4] = [".#*#..", "#lolo#", "#oslo#", ".###.."];

fn seed_inks() -> Vec<(u8, Rgba)> {
    vec![(b'w', rgba(0xffffff, 210)), (b'k', CLOCK.edge)]
}

/// A dandelion clock: a ball of seeds, fluffy round the edge; `blown` leaves a bite out of it.
fn seed_clock(s: &mut Canvas, centre: (i32, i32), blown: bool) {
    for y in centre.1 - 5..=centre.1 + 5 {
        for x in centre.0 - 5..=centre.0 + 5 {
            let (dx, dy) = ((x - centre.0) as f32, (y - centre.1) as f32);
            let reach = (dx * dx + dy * dy).sqrt();
            if reach > 4.2 || (blown && dx + dy * 0.4 > 1.4) {
                continue;
            }
            let color = if reach > 3.5 {
                // The fluff at the edge, thinning out.
                if chance(x, y, 231, 140) {
                    continue;
                }
                rgba(0xe4e0d4, 200)
            } else if reach > 2.8 {
                CLOCK.shadow
            } else if reach < 0.8 {
                rgb(0xb8a878)
            } else {
                let lit = -(dx + dy) / 4.0;
                if lit > 0.35 {
                    CLOCK.shine
                } else if lit > -0.2 {
                    CLOCK.light
                } else {
                    CLOCK.shadow
                }
            };
            put(s, x, y, color);
        }
    }
}

/// A whimsical little tree: a twisting trunk in two colours, striped leaves in teal and violet,
/// and a curl at the top; faintly glowing, as the seed was.
fn strange_sapling() -> Piece {
    let mut s = Canvas::new(28, 44);
    let (cx, ground) = (14, 41);
    ellipse(&mut s, cx, ground, 7, 2, SHADOW);
    // A faint glow about the crown.
    for (radius, alpha) in [(12, 14), (8, 20)] {
        for y in 18 - radius..=18 + radius {
            for x in cx - radius..=cx + radius {
                if in_ellipse(x, y, (cx, 18), (radius, radius)) {
                    put(&mut s, x, y, rgba(0x9af4e4, alpha));
                }
            }
        }
    }
    // The trunk, twisting: its two colours wind up it in stripes.
    let middle = |y: i32| cx - 1 + (1.5 * ((ground - y) as f32 * 0.4).sin()).round() as i32;
    for y in 14..=ground {
        let left = middle(y);
        let width = if y < 21 { 2 } else { 3 };
        for x in left - 1..=left + width {
            let ramp = if (y + x - left).rem_euclid(4) < 2 {
                ODD_BARK
            } else {
                ODD_TEAL
            };
            let color = match x - left {
                -1 => ramp.edge,
                0 => ramp.light,
                across if across == width => ramp.edge,
                across if across == width - 1 => ramp.shadow,
                _ => ramp.base,
            };
            put(&mut s, x, y, color);
        }
    }
    // Leaves in stripes, in pairs up the trunk.
    for &(from, turn, length) in &[
        ((13.0, 31.0), 0.47, 9.0),
        ((16.0, 30.0), 0.03, 9.0),
        ((13.0, 25.0), 0.58, 11.0),
        ((16.0, 24.0), 0.92, 11.0),
        ((13.0, 19.0), 0.66, 9.0),
        ((16.0, 18.0), 0.84, 9.0),
    ] {
        blade(
            &mut s,
            Blade::new(from, turn, length, 2.8),
            |t: f32| (PI * t).sin().powf(0.6),
            |t, facing, x, y| {
                let ramp = if (t * length / 2.0) as i32 % 2 == 1 {
                    ODD_VIOLET
                } else {
                    ODD_TEAL
                };
                tone(ramp, 0.5 * facing + 0.15, x, y, 241)
            },
        );
    }
    // The curl at the top, like a fern unrolling.
    bough(
        &mut s,
        &[
            (15, 15),
            (15, 10),
            (16, 7),
            (18, 5),
            (21, 5),
            (22, 7),
            (21, 9),
            (19, 9),
            (18, 8),
            (19, 7),
        ],
        ODD_TEAL,
    );
    // Motes of its glow, drifting.
    for &(x, y) in &[(5, 12), (23, 15), (8, 29), (21, 6), (25, 27)] {
        put(&mut s, x, y, rgba(0xd8fff4, 220));
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            put(&mut s, x + dx, y + dy, rgba(0x8af0e0, 90));
        }
    }
    stamp(&mut s, (8, ground - 2), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (16, ground - 1), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A slim post with a kite tied to it, flying up and out on its string, bows on its tail.
fn kite_post() -> Piece {
    let mut s = Canvas::new(38, 56);
    let (cx, ground) = (12, 53);
    ellipse(&mut s, cx, ground, 6, 2, SHADOW);
    // The string first, sagging a little on its way up to the kite.
    let (from, bridle) = ((13.0, 27.0), (29.0, 10.0));
    let sag = (24.0, 24.0);
    let mut last = (from.0 as i32, from.1 as i32);
    for step in 1..=40 {
        let t = step as f32 / 40.0;
        let u = 1.0 - t;
        let point = (
            (u * u * from.0 + 2.0 * u * t * sag.0 + t * t * bridle.0).round() as i32,
            (u * u * from.1 + 2.0 * u * t * sag.1 + t * t * bridle.1).round() as i32,
        );
        for (x, y) in trace(&[last, point]) {
            put(&mut s, x, y, STRING);
        }
        last = point;
    }
    // The post, with a cap, and the winder hooked on it with the rest of the string.
    model(&mut s, POST, UPRIGHT, 30, 251, |x, y| {
        (cx - 1..=cx + 1).contains(&x) && (26..=ground).contains(&y)
    });
    model(&mut s, POST, ROUND, 0, 252, |x, y| {
        in_ellipse(x, y, (cx, 25), (2, 1))
    });
    model(&mut s, POST, LYING, 0, 253, |x, y| {
        (cx - 3..=cx + 3).contains(&x) && (y == 31 || y == 36)
    });
    for y in 32..=35 {
        for x in cx - 2..=cx + 2 {
            put(
                &mut s,
                x,
                y,
                if (y + x) % 3 == 0 {
                    STRING
                } else {
                    rgb(0xe8dcc0)
                },
            );
        }
    }
    // The tail, streaming out with its bows.
    let tail = trace(&[(27, 22), (28, 27), (31, 31), (33, 36), (32, 41)]);
    for &(x, y) in &tail {
        put(&mut s, x, y, STRING);
    }
    let bows = [
        (&letters(BOW_BLUE, b"#.o.*"), (27, 26)),
        (&letters(KITE_YELLOW, b"#.o.*"), (30, 31)),
        (&letters(KITE_RED, b"#.o.*"), (32, 36)),
    ];
    for (inks, (x, y)) in bows {
        stamp(&mut s, (x - 2, y - 1), &BOW, inks, false);
    }
    // The kite: four panels round its spars, tipped a little in the wind.
    let (top, left, right, bottom) = ((32, 0), (21, 7), (38, 9), (27, 23));
    let cross = (30, 8);
    let panels = [
        ([top, left, cross], KITE_YELLOW),
        ([top, cross, right], KITE_RED),
        ([left, bottom, cross], KITE_RED),
        ([cross, bottom, right], KITE_YELLOW),
    ];
    let mut cloth = Canvas::new(38, 56);
    for (index, (corners, ramp)) in panels.iter().enumerate() {
        polygon(&mut cloth, corners, |x, y| {
            let lit = if index == 0 {
                0.4
            } else if index == 3 {
                -0.3
            } else {
                0.0
            };
            Some(tone(*ramp, lit, x, y, 254 + index as u32))
        });
    }
    let on = |x: i32, y: i32| cloth.get(x, y).a > 0;
    for y in 0..24 {
        for x in 18..38 {
            if !on(x, y) {
                continue;
            }
            let color = cloth.get(x, y);
            let ramp = if color == KITE_RED.base
                || color == KITE_RED.shadow
                || color == KITE_RED.light
                || color == KITE_RED.shine
            {
                KITE_RED
            } else {
                KITE_YELLOW
            };
            let rim = !on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1);
            put(&mut s, x, y, if rim { ramp.edge } else { color });
        }
    }
    for (x, y) in trace(&[top, bottom])
        .into_iter()
        .chain(trace(&[left, right]))
    {
        if on(x - 1, y) && on(x + 1, y) && on(x, y - 1) && on(x, y + 1) {
            let spar = mix(s.get(x, y), POST.base, 0.6);
            put(&mut s, x, y, spar);
        }
    }
    stamp(&mut s, (cx - 6, ground - 2), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (cx + 2, ground - 1), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A bow tied on a kite's tail.
const BOW: [&str; 3] = ["##.##", "#o*o#", "##.##"];

/// A buddleia: grey-green leaves, arching purple cones of flowers, and butterflies on them.
fn butterfly_bush() -> Piece {
    let mut s = Canvas::new(32, 36);
    let (cx, ground) = (16, 33);
    ellipse(&mut s, cx, ground, 12, 3, SHADOW);
    // Long leaves poking out of the bush, behind it.
    for &(from, turn, length) in &[
        ((7.0, 24.0), 0.55, 7.0),
        ((25.0, 24.0), 0.95, 7.0),
        ((10.0, 20.0), 0.65, 6.0),
        ((22.0, 20.0), 0.85, 6.0),
    ] {
        blade(
            &mut s,
            Blade::new(from, turn, length, 1.6),
            |t: f32| (PI * t).sin().powf(0.7),
            |_, facing, x, y| tone(SAGE, 0.5 * facing, x, y, 261),
        );
    }
    foliage(
        &mut s,
        &[
            ((10, 29), (6, 3)),
            ((22, 29), (6, 3)),
            ((16, 27), (7, 4)),
            ((16, 30), (9, 3)),
        ],
        SAGE,
        262,
    );
    // The flowers: cones of tiny florets on arching stems, nodding over at their tips.
    for (index, &(from, turn, length)) in [
        ((8.0, 26.0), 0.655, 15.0),
        ((24.0, 26.0), 0.845, 15.0),
        ((12.0, 25.0), 0.7, 18.0),
        ((20.0, 25.0), 0.8, 18.0),
        ((16.0, 25.0), 0.752, 21.0),
    ]
    .iter()
    .enumerate()
    {
        panicle(&mut s, from, turn, length, 264 + index as u32);
    }
    // Leaves along the stems, in front of where the flowers start.
    for &(from, turn, length) in &[
        ((9.0, 26.0), 0.47, 6.0),
        ((23.0, 26.0), 0.03, 6.0),
        ((13.0, 24.0), 0.56, 6.0),
        ((19.0, 24.0), 0.94, 6.0),
        ((16.0, 25.0), 0.68, 5.0),
    ] {
        blade(
            &mut s,
            Blade::new(from, turn, length, 1.5),
            |t: f32| (PI * t).sin().powf(0.7),
            |_, facing, x, y| tone(SAGE, 0.5 * facing + 0.2, x, y, 269),
        );
    }
    // Butterflies: two settled on the flowers, one about to.
    stamp(
        &mut s,
        (3, 6),
        &BUTTERFLY,
        &butterfly_inks(0xe07a2a, 0x6a2a14),
        false,
    );
    stamp(
        &mut s,
        (23, 9),
        &BUTTERFLY,
        &butterfly_inks(0xf2e27a, 0x8a7a22),
        false,
    );
    stamp(
        &mut s,
        (20, 0),
        &BUTTERFLY,
        &butterfly_inks(0x7aa8f0, 0x2c4a90),
        false,
    );
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A cone of buddleia flowers out from `from` towards `turn`, nodding over towards its tip: a
/// tapering run of florets, each lit from the upper left, with an orange eye here and there,
/// and the whole outlined in the flowers' own darkest purple.
fn panicle(s: &mut Canvas, from: (f32, f32), turn: f32, length: f32, salt: u32) {
    let mut layer = Canvas::new(s.width(), s.height());
    let (sin, cos) = (turn * TAU).sin_cos();
    // A bare green stalk out of the bush, then the flowers along the rest of it, the ones
    // leaning furthest out nodding over the most.
    let droop = 0.5 + 3.0 * cos.abs();
    let at = |t: f32| {
        (
            from.0 + cos * length * t,
            from.1 + sin * length * t + droop * t * t,
        )
    };
    let stalk = at(0.2);
    for (x, y) in trace(&[
        (from.0 as i32, from.1 as i32),
        (stalk.0 as i32, stalk.1 as i32),
    ]) {
        put(s, x, y, SAGE.shadow);
    }
    let steps = (length * 2.0) as i32;
    for step in 0..=steps {
        let t = 0.2 + 0.8 * step as f32 / steps as f32;
        let (px, py) = at(t);
        let along = (t - 0.2) / 0.8;
        let radius = 2.1 * (PI * (0.2 + 0.8 * along)).sin().powf(0.8) + 0.5;
        let reach = radius.ceil() as i32;
        for y in py as i32 - reach..=py as i32 + reach {
            for x in px as i32 - reach..=px as i32 + reach {
                let (dx, dy) = (x as f32 + 0.5 - px, y as f32 + 0.5 - py);
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                let color = if chance(x, y, salt, 22) && along < 0.85 {
                    rgb(0xf0a040)
                } else if chance(x, y, salt + 2, 50) {
                    BUDDLEIA.shine
                } else {
                    tone(BUDDLEIA, -(dx + dy) / radius + 0.45, x, y, salt + 1)
                };
                layer.set(x, y, color);
            }
        }
    }
    let on = |x: i32, y: i32| layer.get(x, y).a > 0;
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if on(x, y) {
                let rim = !on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1);
                put(s, x, y, if rim { BUDDLEIA.edge } else { layer.get(x, y) });
            }
        }
    }
}

/// A butterfly with its wings open: `#` for the wings' edges, `o` and `l` for the wings, `b` its
/// body.
const BUTTERFLY: [&str; 4] = ["#o...o#", "olobolo", ".ooboo.", ".#...#."];

fn butterfly_inks(wing: u32, edge: u32) -> Vec<(u8, Rgba)> {
    vec![
        (b'#', rgb(edge)),
        (b'o', rgb(wing)),
        (b'l', mix(rgb(wing), rgb(0xffffff), 0.45)),
        (b'b', rgb(0x3a2a2a)),
    ]
}

/// The fox, as a shape: `x` where it is, running to the left with its brush out behind.
const FOX: [&str; 11] = [
    "...x.x..................",
    "...xxxx.................",
    "..xxxxxx................",
    "xxxxxxxxx...........xxx.",
    ".xxxxxxxxxxx......xxxxxx",
    "....xxxxxxxxxxxxxxxxxxxx",
    "....xxxxxxxxxxxxxxxxxxx.",
    "...xxxxxxxxxxxxxxx.xx...",
    "..xx.......xxxx.........",
    ".xx..........xxx........",
    "xx.............xx.......",
];

/// Letters for the compass points, raised on the ends of the arms.
const NORTH: [&str; 5] = ["#..#", "##.#", "#.##", "#..#", "#..#"];
const EAST: [&str; 5] = ["###", "#..", "##.", "#..", "###"];
const SOUTH: [&str; 5] = [".##", "#..", ".#.", "..#", "##."];
const WEST: [&str; 5] = ["#...#", "#...#", "#.#.#", "#.#.#", ".#.#."];

/// The fox weathervane: a tall pole with the compass points on their arms, and the copper fox on
/// top, green with age, running into the wind.
fn fox_weathervane() -> Piece {
    let mut s = Canvas::new(30, 56);
    let (cx, ground) = (15, 53);
    ellipse(&mut s, cx, ground, 8, 2, SHADOW);
    // The pole, set in a stone footing.
    model(&mut s, WROUGHT, UPRIGHT, 20, 271, |x, y| {
        (cx - 1..=cx + 1).contains(&x) && (14..=ground - 3).contains(&y)
    });
    model(&mut s, FOOTING, ROUND, 40, 272, |x, y| {
        (cx - 5..=cx + 5).contains(&x)
            && (ground - 5..=ground).contains(&y)
            && !((x == cx - 5 || x == cx + 5) && y == ground - 5)
    });
    for x in cx - 4..=cx + 4 {
        put(&mut s, x, ground - 4, FOOTING.light);
    }
    // The arms: east and west across, north and south seen end on, running back and forth.
    let arm_y = 27;
    for x in 5..=25 {
        put(&mut s, x, arm_y, WROUGHT.base);
        put(&mut s, x, arm_y + 1, WROUGHT.edge);
    }
    for (x, y) in trace(&[(9, 23), (21, 31)]) {
        put(&mut s, x, y, WROUGHT.light);
        put(&mut s, x, y + 1, WROUGHT.edge);
    }
    ellipse(&mut s, cx, arm_y, 1, 1, COPPER.base);
    put(&mut s, cx - 1, arm_y - 1, COPPER.light);
    let raised = [(b'#', COPPER.light)];
    for (rows, at) in [
        (&WEST[..], (0, arm_y - 2)),
        (&EAST[..], (27, arm_y - 2)),
        (&NORTH[..], (5, 18)),
        (&SOUTH[..], (22, 31)),
    ] {
        stamp(
            &mut s,
            (at.0 + 1, at.1 + 1),
            rows,
            &[(b'#', COPPER.edge)],
            false,
        );
        stamp(&mut s, at, rows, &raised, false);
    }
    // The arrow the fox runs along, and the ball it turns on.
    for x in 2..=27 {
        put(&mut s, x, 12, COPPER.light);
        put(&mut s, x, 13, COPPER.edge);
    }
    stamp(
        &mut s,
        (0, 10),
        &["..#", ".#l", "#lc", ".#c", "..#"],
        &arrow_inks(),
        false,
    );
    stamp(
        &mut s,
        (25, 9),
        &["#..#", "#c.#", ".#c#", "#c.#", "#..#"],
        &arrow_inks(),
        false,
    );
    model(&mut s, COPPER, ROUND, 0, 273, |x, y| {
        in_ellipse(x, y, (cx, 15), (1, 1))
    });
    // The fox, green with age where the weather has had it, copper still where it hasn't.
    let fox = |x: i32, y: i32| {
        let (fx, fy) = (x - 3, y - 1);
        fy >= 0
            && fx >= 0
            && FOX
                .get(fy as usize)
                .and_then(|row| row.as_bytes().get(fx as usize))
                .is_some_and(|&letter| letter == b'x')
    };
    model(&mut s, VERDIGRIS, ROUND, 0, 274, fox);
    for y in 1..12 {
        for x in 3..27 {
            let color = s.get(x, y);
            if fox(x, y) && color != VERDIGRIS.edge && chance(x / 2, y / 2, 275, 90) {
                let copper = if color == VERDIGRIS.light || color == VERDIGRIS.shine {
                    COPPER.light
                } else {
                    COPPER.base
                };
                put(&mut s, x, y, copper);
            }
        }
    }
    // Its eye, and the tip of its nose.
    put(&mut s, 7, 3, VERDIGRIS.edge);
    put(&mut s, 3, 4, VERDIGRIS.edge);
    stamp(&mut s, (cx - 9, ground - 2), &TUFT, &tuft_inks(), false);
    stamp(&mut s, (cx + 5, ground - 1), &TUFT, &tuft_inks(), true);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

fn arrow_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'#', COPPER.edge),
        (b'c', COPPER.base),
        (b'l', COPPER.light),
    ]
}

// ---------------------------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------------------------

/// A ramp's tone for something facing the light by `lit` (about -1 to 1), dithered.
fn tone(ramp: Ramp, lit: f32, x: i32, y: i32, salt: u32) -> Rgba {
    let lit = lit + 0.35 * ((noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5);
    if lit > 0.62 {
        ramp.shine
    } else if lit > 0.22 {
        ramp.light
    } else if lit > -0.25 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// A clump of leaves in a bush: its middle and its size.
type Clump = ((i32, i32), (i32, i32));

/// A bush's worth of leaves: rounded clumps, each lit from the upper left, the clumps parted by
/// shade and the whole outlined in the leaves' own darkest green, with leaves catching the light
/// all over and poking out round the edge.
fn foliage(s: &mut Canvas, clumps: &[Clump], ramp: Ramp, salt: u32) {
    let mut layer = Canvas::new(s.width(), s.height());
    for (index, &(centre, size)) in clumps.iter().enumerate() {
        model(&mut layer, ramp, ROUND, 40, salt + index as u32, |x, y| {
            in_ellipse(x, y, centre, size)
        });
    }
    let leafy = |x: i32, y: i32| layer.get(x, y).a > 0;
    let mut bush = layer.clone();
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !leafy(x, y) {
                // A leaf tip out past the edge, here and there.
                let beside = leafy(x + 1, y) || leafy(x - 1, y) || leafy(x, y + 1);
                if beside && !leafy(x, y - 1) && chance(x, y, salt + 50, 60) {
                    bush.set(x, y, ramp.edge);
                }
                continue;
            }
            let inner = leafy(x - 1, y) && leafy(x + 1, y) && leafy(x, y - 1) && leafy(x, y + 1);
            let color = layer.get(x, y);
            if inner && color == ramp.edge {
                bush.set(x, y, ramp.shadow);
            } else if inner && color != ramp.shadow && chance(x, y, salt + 51, 34) {
                bush.set(x, y, ramp.light);
                if leafy(x + 1, y + 1) {
                    bush.set(x + 1, y + 1, ramp.base);
                }
            }
        }
    }
    blit(s, &bush, 0, 0);
}

/// A long leaf or a cone of flowers, along a line from `base` at `turn` (a fraction of a full
/// turn: none points right, a quarter down).
struct Blade {
    base: (f32, f32),
    turn: f32,
    length: f32,
    width: f32,
}

impl Blade {
    fn new(base: (f32, f32), turn: f32, length: f32, width: f32) -> Self {
        Self {
            base,
            turn,
            length,
            width,
        }
    }

    /// Where `(x, y)` falls on the blade if it does: how far along it (0 to 1), how far across
    /// it from its rib (-1 to 1), and how far towards the lit side of it (-1 to 1).
    fn at(&self, x: i32, y: i32, shape: &impl Fn(f32) -> f32) -> Option<(f32, f32, f32)> {
        let (sin, cos) = (self.turn * TAU).sin_cos();
        let (dx, dy) = (x as f32 + 0.5 - self.base.0, y as f32 + 0.5 - self.base.1);
        let along = dx * cos + dy * sin;
        let across = dy * cos - dx * sin;
        if !(0.0..=self.length).contains(&along) {
            return None;
        }
        let t = along / self.length;
        let half = (self.width * shape(t)).max(0.5);
        if across.abs() > half {
            return None;
        }
        // Which way across faces up and to the left, where the light comes from.
        let facing = (sin - cos) * across / half;
        Some((t, across / half, facing.clamp(-1.0, 1.0)))
    }
}

/// Draws a blade whose width along it follows `shape` (0 to 1), each pixel painted by `paint`
/// from how far along it is, how much it faces the light, and where it is; its outermost pixels
/// in the edge tone of whatever was painted there.
fn blade(
    s: &mut Canvas,
    blade: Blade,
    shape: impl Fn(f32) -> f32,
    paint: impl Fn(f32, f32, i32, i32) -> Rgba,
) {
    let on = |x: i32, y: i32| blade.at(x, y, &shape).is_some();
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            let Some((t, _, facing)) = blade.at(x, y, &shape) else {
                continue;
            };
            let color = paint(t, facing, x, y);
            let rim = !on(x - 1, y) || !on(x + 1, y) || !on(x, y - 1) || !on(x, y + 1);
            put(s, x, y, if rim { darker(color) } else { color });
        }
    }
}

/// A colour's own darker shade, for outlining it.
fn darker(color: Rgba) -> Rgba {
    let shade = |channel: u8| (f32::from(channel) * 0.55) as u8;
    Rgba::new(shade(color.r), shade(color.g), shade(color.b), color.a)
}
