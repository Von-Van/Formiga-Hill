//! What the colony builds on the Hilltop from its plans (`finds::plans`): each bigger and grander
//! than the finds it was made of, but still within one spot. Drawn as the finds are, with the same
//! ramps and brushwork, so the pebbles of a cairn are the pebbles found and the lanterns of the
//! lantern tree the lantern brought home. Several are made from the finds' own pieces: the
//! burrow house is the little door in the hill with more built round it.

use super::Piece;
use super::brush::*;
use super::earth::{self, PEBBLE, PEBBLE_WARM, POST, TURF};
use super::hollow::{self, BARK, CONE, GILT, IRON, LACQUER, Lie, MOSSY, NEST, PLUM, POLE, VELVET};
use super::undergrowth::{self, BERRY, BERRY_DARK, BERRY_RED, LEAF, STRING};
use super::water::{
    self, AMBER, BLUE_GLASS, BOTTLE_GREEN, COBALT, PALE_GLASS, SANDSTONE, SEA_GLASS, SLATE,
    TEAL_GLASS,
};
use crate::materials::{ENAMEL_DEEP, GLASS, GLOW, PLANK, STONE, TILE, TRIM};
use crate::paint::{Ramp, blit, chance, hline, mix, noise, polygon, put, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::PI;

/// The piece a plan builds, or `None` if it isn't drawn.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "grand_cairn" => grand_cairn(),
        "flower_bed" => flower_bed(),
        "picnic_table" => picnic_table(),
        "bird_table" => bird_table(),
        "snug_den" => snug_den(),
        "lantern_tree" => lantern_tree(),
        "wishing_well" => wishing_well(),
        "great_telescope" => great_telescope(),
        "bandstand" => bandstand(),
        "burrow_house" => burrow_house(),
        _ => return None,
    })
}

/// What shines from a built piece after dark: each light's colour, and where it is in the piece.
pub fn lights(id: &str) -> Vec<(Rgba, (i32, i32))> {
    let candle = rgb(0xffcf6a);
    match id {
        "lantern_tree" => LANTERNS
            .iter()
            .map(|&(x, y)| (candle, (x, y + 8)))
            .collect(),
        "bandstand" => vec![(candle, (24, 26))],
        "burrow_house" => vec![
            (candle, (11, 39)),
            (candle, BACK_WINDOW),
            (candle, (31, 31)),
        ],
        _ => Vec::new(),
    }
}

/// Willow withies, woven.
const WILLOW: Ramp = Ramp::new(0x4a3624, 0x6e5236, 0x8e6e48, 0xae8c60, 0xcaa87c);
/// A robin's back, and its breast.
const ROBIN: Ramp = Ramp::new(0x4a3424, 0x6a4a32, 0x8a6444, 0xa88058, 0xc49c70);
const BREAST: Rgba = rgb(0xe0703c);
/// Wooden shingles, weathered.
const SHINGLE: Ramp = Ramp::new(0x4a3424, 0x6c4c34, 0x8c6844, 0xaa865a, 0xc6a476);
/// Water deep down a well, nearly black but blue.
const DEEP_WATER: Ramp = Ramp::new(0x141c2a, 0x1e2a3c, 0x2a3a52, 0x3e5470, 0x6a86a6);
/// The dark inside of anything, a warm brown rather than black.
const INSIDE: Rgba = rgb(0x2e2018);

// ---------------------------------------------------------------------------------------------
// Commoner things
// ---------------------------------------------------------------------------------------------

/// The grand cairn: a waymark of a dozen stones on a footing of three, each settled a little off
/// the one below, moss on the shoulders that face the weather and pebbles about its foot.
fn grand_cairn() -> Piece {
    let mut s = Canvas::new(28, 42);
    let (cx, ground) = (14, 39);
    shadow(&mut s, cx, ground, 13, 2);
    // The footing, two at the back and one in front between them, then each stone up from it.
    let stones = [
        (7.5, 36.2, 6.0, 3.0, 0.08, PEBBLE),
        (20.5, 36.4, 6.0, 2.9, -0.1, PEBBLE_WARM),
        (14.0, 36.6, 6.8, 3.2, 0.0, STONE),
        (10.5, 31.0, 5.2, 2.8, -0.12, PEBBLE_WARM),
        (17.8, 30.8, 4.8, 2.7, 0.15, PEBBLE),
        (14.2, 25.8, 5.8, 3.0, 0.05, STONE),
        (13.6, 20.6, 4.8, 2.6, -0.14, PEBBLE),
        (14.4, 15.9, 4.0, 2.4, 0.18, PEBBLE_WARM),
        (13.8, 11.6, 3.2, 2.0, -0.1, STONE),
        (14.3, 8.0, 2.5, 1.7, 0.2, PEBBLE),
        (13.9, 5.2, 1.7, 1.3, -0.15, PEBBLE_WARM),
    ];
    for (index, &(x, y, rx, ry, tilt, ramp)) in stones.iter().enumerate() {
        if index > 2 {
            // Where it rests, the stone below is in its shade.
            tuck(&mut s, x + 1.0, y + 1.2, rx, ry, rgba(0x2a2420, 60));
        }
        tilted_lump(&mut s, (x, y), (rx, ry), tilt, ramp, 400 + index as u32);
    }
    moss(&mut s, 4.6, 34.4, 2.8, 1.2, 411);
    moss(&mut s, 7.8, 29.8, 2.2, 1.0, 412);
    moss(&mut s, 10.2, 24.6, 2.0, 1.0, 413);
    moss(&mut s, 11.2, 14.8, 1.4, 0.8, 414);
    lump(&mut s, 2.4, 38.6, 1.8, 1.2, PEBBLE, 415);
    lump(&mut s, 25.6, 38.8, 1.6, 1.1, STONE, 416);
    tuft(&mut s, 4, ground, 3, 417);
    tuft(&mut s, 24, ground, 3, 418);
    flowers(&mut s, 22, ground - 1, 2, 2, 419);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A bed of turned earth with a woven willow edge: two clumps of bluebells along the back, a patch
/// of dandelions in front.
fn flower_bed() -> Piece {
    let mut s = Canvas::new(46, 27);
    let (cx, ground) = (23, 25);
    shadow(&mut s, cx, ground, 22, 3);
    let (middle, size) = ((23, 19), (21.5, 5.0));
    let row = |x: i32, front: bool| {
        let dx = (x - middle.0) as f32 / size.0;
        let across = (1.0 - dx * dx).max(0.0).sqrt() * size.1;
        let y = if front {
            middle.1 as f32 + across
        } else {
            middle.1 as f32 - across
        };
        y.round() as i32
    };
    model(&mut s, TILTH, LYING, 40, 420, |x, y| {
        in_ellipse(x, y, middle, (21, 5))
    });
    // The far side of the edging, behind everything growing.
    for x in 2..=44 {
        let y = row(x, false);
        put(&mut s, x, y, WILLOW.shadow);
        put(
            &mut s,
            x,
            y - 1,
            if (x / 3) % 2 == 0 {
                WILLOW.base
            } else {
                WILLOW.edge
            },
        );
    }
    for x in [12, 33] {
        let clump = earth::bluebells();
        blit(
            &mut s,
            &clump.sprite,
            x - clump.anchor.0,
            18 - clump.anchor.1,
        );
    }
    let patch = undergrowth::dandelion_patch();
    blit(
        &mut s,
        &patch.sprite,
        23 - patch.anchor.0,
        23 - patch.anchor.1,
    );
    // The near side: withies woven between stakes, over and under, the stakes poking up.
    for x in 1..=45 {
        let front = row(x, true);
        for (step, y) in (front - 1..=front + 1).enumerate() {
            let over = (x / 3 + step as i32) % 2 == 0;
            let color = if step == 2 {
                WILLOW.edge
            } else if over && x < cx {
                WILLOW.light
            } else if over {
                WILLOW.base
            } else {
                WILLOW.shadow
            };
            put(&mut s, x, y, color);
        }
        if x % 7 == 3 {
            put(&mut s, x, front - 2, WILLOW.light);
            put(&mut s, x, front - 3, WILLOW.edge);
        }
    }
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A picnic table of flat stones: a broad one on two stout pebbles for the table, a smaller one
/// either side for seats, and berries heaped on a leaf for a plate.
fn picnic_table() -> Piece {
    let mut s = Canvas::new(44, 28);
    let (cx, ground) = (22, 25);
    shadow(&mut s, cx, ground, 21, 3);
    // The seats, each a flat stone on a pair of pebbles.
    for (seat, salt) in [(5, 430), (38, 434)] {
        lump(&mut s, seat as f32 - 2.5, 23.6, 1.5, 1.4, PEBBLE, salt);
        lump(
            &mut s,
            seat as f32 + 2.5,
            23.6,
            1.5,
            1.4,
            PEBBLE_WARM,
            salt + 1,
        );
        water::flat_stone(&mut s, (seat, 20), (5, 1), 2, SLATE, salt + 2);
    }
    // The table.
    lump(&mut s, 15.5, 20.2, 2.4, 4.0, PEBBLE_WARM, 440);
    lump(&mut s, 28.5, 20.2, 2.4, 4.0, PEBBLE, 441);
    tuck(&mut s, 15.5, 17.4, 2.4, 1.2, rgba(0x2a2420, 70));
    tuck(&mut s, 28.5, 17.4, 2.4, 1.2, rgba(0x2a2420, 70));
    water::flat_stone(&mut s, (22, 11), (13, 3), 2, SANDSTONE, 442);
    // The plate: a big leaf, its rib down the middle, heaped with berries.
    model(&mut s, LEAF, LYING, 0, 443, |x, y| {
        in_ellipse(x, y, (19, 10), (6, 2))
    });
    hline(&mut s, 14, 10, 10, LEAF.light);
    let red = letters(BERRY_RED, b"#sol*");
    let dark = letters(BERRY_DARK, b"#sol*");
    for (index, &(x, y)) in [(15, 8), (18, 9), (21, 8), (17, 7), (20, 6), (23, 9)]
        .iter()
        .enumerate()
    {
        stamp(
            &mut s,
            (x, y),
            &BERRY,
            if index % 3 == 1 { &dark } else { &red },
            false,
        );
    }
    stamp(&mut s, (28, 10), &BERRY, &red, false);
    stamp(&mut s, (30, 11), &BERRY, &dark, false);
    tuft(&mut s, 1, ground, 2, 444);
    tuft(&mut s, 42, ground, 2, 445);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// A robin, standing facing left: `b` its back, `r` its breast, `e` its eye, `k` its legs.
const ROBIN_ROWS: [&str; 5] = [".bb...", "beb...", "rrbbbt", ".rrbb.", "..k.k."];

/// A bird table: a tray on a braced post with a little roof over it on four corner posts, the nest
/// tucked under the roof, berries on the tray, a pinecone feeder hanging from its edge, and a robin
/// on the ridge.
fn bird_table() -> Piece {
    let mut s = Canvas::new(28, 48);
    let (cx, ground) = (14, 45);
    shadow(&mut s, cx, ground, 8, 2);
    // The post, braced at its foot.
    hollow::stick(&mut s, (14.0, 39.0), (8.5, 45.5), (1.8, 1.6), POLE);
    hollow::stick(&mut s, (14.0, 39.0), (19.5, 45.5), (1.8, 1.6), POLE);
    hollow::stick(&mut s, (14.0, 45.5), (14.0, 23.0), (3.4, 2.8), POLE);
    // The corner posts that hold the roof up.
    for x in [4.5, 23.5] {
        hollow::stick(&mut s, (x, 22.0), (x, 11.0), (1.6, 1.6), POLE);
    }
    // The nest on the tray at the back, lined with moss.
    model(&mut s, NEST, ROUND, 70, 450, |x, y| {
        in_ellipse(x, y, (14, 19), (5, 3)) && y >= 17
    });
    for x in 10..=18 {
        put(
            &mut s,
            x,
            17,
            if chance(x, 17, 451, 120) {
                MOSSY.light
            } else {
                MOSSY.base
            },
        );
    }
    // The tray: a board with a lip round it, lit along its top.
    for y in 20..=23 {
        for x in 2..=25 {
            let color = if y == 23 || x == 2 || x == 25 {
                POLE.edge
            } else if y == 20 {
                POLE.light
            } else if y == 21 {
                if x < 6 { POLE.shine } else { POLE.base }
            } else {
                POLE.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    put(&mut s, 2, 19, POLE.base);
    put(&mut s, 25, 19, POLE.shadow);
    let red = letters(BERRY_RED, b"#sol*");
    let dark = letters(BERRY_DARK, b"#sol*");
    for (x, inks) in [(4, &red), (7, &dark), (19, &red), (21, &red)] {
        stamp(&mut s, (x, 17), &BERRY, inks, false);
    }
    // The roof: two boards meeting over the tray, overhanging, moss along their tops.
    let apex = 14;
    for side in [-1, 1] {
        for step in 0..=13 {
            let (x, y) = (apex + side * step, 4 + step / 2 + step % 2);
            put(&mut s, x, y, POLE.light);
            put(&mut s, x, y + 1, POLE.base);
            put(&mut s, x, y + 2, POLE.edge);
            let bump = i32::from(noise(x, 0, 452).is_multiple_of(3)) + i32::from(step < 3);
            for dy in 1..=1 + bump {
                let color = if dy == 1 + bump {
                    MOSSY.edge
                } else if side < 0 {
                    MOSSY.light
                } else {
                    MOSSY.shadow
                };
                put(&mut s, x, y - dy, color);
            }
            put(
                &mut s,
                x,
                y,
                if side < 0 { MOSSY.base } else { MOSSY.shadow },
            );
        }
    }
    // The pinecone feeder, on a string from the tray's edge.
    vline(&mut s, 24, 24, 3, STRING);
    hollow::cone(&mut s, 22, 27, Lie::Up);
    // A robin on the ridge, come to see who is new.
    stamp(
        &mut s,
        (apex - 1, 0),
        &ROBIN_ROWS,
        &[
            (b'b', ROBIN.base),
            (b'e', ROBIN.edge),
            (b'r', BREAST),
            (b't', ROBIN.shadow),
            (b'k', ROBIN.edge),
        ],
        false,
    );
    put(&mut s, apex - 2, 2, rgb(0xd8b04e));
    tuft(&mut s, 9, ground, 2, 453);
    tuft(&mut s, 19, ground, 2, 454);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// Things from the hollows
// ---------------------------------------------------------------------------------------------

/// A snug den: a dome of pinecones heaped over a frame of twigs, the twigs poking out at the top,
/// moss grown over it, and a round doorway lined with the nest's weave.
fn snug_den() -> Piece {
    let mut s = Canvas::new(42, 32);
    let (cx, ground) = (21, 29);
    shadow(&mut s, cx, ground, 20, 3);
    let dome = |x: i32, y: i32| {
        let dx = (x as f32 + 0.5 - 21.0) / 19.0;
        let dy = (y as f32 + 0.5 - 29.5) / 20.0;
        y <= ground && dx * dx + dy * dy <= 1.0
    };
    bough(&mut s, &[(18, 12), (16, 6), (15, 3)], BARK);
    bough(&mut s, &[(23, 11), (26, 6), (28, 4)], BARK);
    // Under the cones, so no daylight shows between them.
    for y in 0..=ground {
        for x in 0..42 {
            if dome(x, y) {
                put(
                    &mut s,
                    x,
                    y,
                    if dome(x, y - 1) {
                        CONE.edge
                    } else {
                        CONE.shadow
                    },
                );
            }
        }
    }
    // Cones in rows, back to front, each lying its own way.
    for row in 0..7 {
        let y = 8 + row * 3;
        for column in 0..7 {
            let x = column * 6 - 2 + (row % 2) * 3;
            if !dome(x + 4, y + 3) {
                continue;
            }
            let lie = if noise(column, row, 460).is_multiple_of(2) {
                Lie::Left
            } else {
                Lie::Right
            };
            hollow::cone(&mut s, x, y, lie);
        }
    }
    moss(&mut s, 17.0, 12.0, 7.0, 2.2, 461);
    moss(&mut s, 8.0, 19.5, 3.0, 1.6, 462);
    moss(&mut s, 33.0, 21.0, 2.6, 1.4, 463);
    // The doorway: dark inside, woven round with the nest, moss on the threshold.
    let (door, rim) = (((21.0, 29.5), (5.0, 9.0)), ((21.0, 29.5), (7.0, 11.0)));
    let within = |((ox, oy), (rx, ry)): ((f32, f32), (f32, f32)), x: i32, y: i32| {
        let (dx, dy) = ((x as f32 + 0.5 - ox) / rx, (y as f32 + 0.5 - oy) / ry);
        y <= ground && dx * dx + dy * dy <= 1.0
    };
    for y in 16..=ground {
        for x in 12..31 {
            if within(door, x, y) {
                let far = within(door, x - 1, y - 1) && !within(door, x - 2, y - 2);
                put(&mut s, x, y, if far { rgb(0x4a3424) } else { INSIDE });
            } else if within(rim, x, y) {
                let over = (x + y / 2) % 3 == 0;
                let color = if over {
                    NEST.light
                } else if x < cx {
                    NEST.base
                } else {
                    NEST.shadow
                };
                put(&mut s, x, y, color);
            }
        }
    }
    for x in 17..26 {
        if chance(x, ground, 464, 150) {
            put(&mut s, x, ground, MOSSY.base);
            put(&mut s, x, ground - 1, MOSSY.light);
        }
    }
    put(&mut s, 25, 21, rgb(0xd8b04e));
    put(&mut s, 26, 22, rgb(0xe8c86a));
    hollow::toadstool(&mut s, 38, ground);
    tuft(&mut s, 3, ground, 2, 465);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// Where the lantern tree's two lanterns hang from, in its own pixels: bare twigs of the bottle
/// tree that never had a bottle.
const LANTERNS: [(i32, i32); 2] = [(10, 29), (32, 23)];

/// The lantern tree: the bottle tree with two lanterns hung on its bare twigs and a string of
/// glass beads looped between them, all in the lanterns' warm light.
fn lantern_tree() -> Piece {
    let mut s = Canvas::new(40, 54);
    let (cx, ground) = (20, 51);
    // The light first, so the tree is drawn in it.
    for (x, y) in LANTERNS {
        hollow::glow(&mut s, x, y + 8, 9.0, rgba(0xffe7a0, 40));
        hollow::glow(&mut s, x, y + 8, 6.0, rgba(0xffe7a0, 52));
    }
    let tree = water::bottle_tree();
    blit(
        &mut s,
        &tree.sprite,
        cx - tree.anchor.0,
        ground - tree.anchor.1,
    );
    // Beads strung between the lanterns, sagging across the trunk.
    let ((x0, y0), (x1, y1)) = (LANTERNS[0], LANTERNS[1]);
    let from = (x0 as f32, y0 as f32 + 1.0);
    let to = (x1 as f32, y1 as f32 + 1.0);
    let sag = ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0 + 9.0);
    let beads = [AMBER, COBALT, BOTTLE_GREEN, SEA_GLASS];
    for (index, (x, y)) in curve(from, sag, to).into_iter().enumerate() {
        if index % 3 == 1 {
            let bead = beads[(index / 3) % beads.len()];
            put(&mut s, x, y, bead.light);
            put(&mut s, x, y + 1, bead.base);
            put(&mut s, x + 1, y + 1, bead.edge);
        } else {
            put(&mut s, x, y, STRING);
        }
    }
    for (x, y) in LANTERNS {
        hollow::hung_lantern(&mut s, (x, y));
    }
    flowers(&mut s, 30, ground - 1, 3, 3, 470);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// Rarer things
// ---------------------------------------------------------------------------------------------

/// A wishing well: a round wall of pebbles, lichened, with dark water deep inside and the coin
/// glinting at the bottom; posts up either side to a red-tiled roof, the windlass under it with
/// its rope wound on and the bucket let down.
fn wishing_well() -> Piece {
    let mut s = Canvas::new(34, 48);
    let (cx, ground) = (17, 45);
    shadow(&mut s, cx, ground, 15, 3);
    let (left, right, rim) = (4, 30, 31);
    for x in [6.5, 27.5] {
        hollow::stick(&mut s, (x, rim as f32 + 1.0), (x, 14.0), (2.8, 2.6), POLE);
    }
    // The windlass: a log between the posts with the rope wound on it, a crank on the end.
    for x in 7..=28 {
        put(&mut s, x, 17, POLE.light);
        put(&mut s, x, 18, POLE.base);
        put(&mut s, x, 19, POLE.edge);
    }
    for x in 12..=21 {
        put(&mut s, x, 17 + (x % 3).min(2), STRING);
        put(
            &mut s,
            x,
            18,
            if x % 2 == 0 { rgb(0xb8a684) } else { STRING },
        );
    }
    for (x, y) in [(29, 18), (30, 18), (31, 19), (31, 20), (31, 21), (32, 21)] {
        put(&mut s, x, y, IRON.base);
    }
    put(&mut s, 30, 17, IRON.light);
    // The rope down, and the bucket on it.
    vline(&mut s, 17, 20, 5, STRING);
    paint_rows(
        &mut s,
        14,
        23,
        &["..ii..", ".i..i.", "#iiii#", "#loos#", "#iiii#", ".#os#."],
        POLE,
        &[('i', IRON.base)],
    );
    // The water down inside, the coin at the bottom of it.
    for y in rim - 3..=rim + 3 {
        for x in left..=right {
            let outer = in_ellipse(x, y, (cx, rim), (13, 3));
            let inner = in_ellipse(x, y, (cx, rim), (10, 2));
            if inner {
                let lit = in_ellipse(x, y + 1, (cx - 3, rim), (5, 1));
                put(
                    &mut s,
                    x,
                    y,
                    if lit {
                        DEEP_WATER.shadow
                    } else {
                        DEEP_WATER.edge
                    },
                );
            } else if outer {
                let stone = noise((x - left) / 3, y, 480) % 3;
                let ramp = [PEBBLE, PEBBLE_WARM, STONE][stone as usize];
                let color = if y < rim {
                    ramp.light
                } else if x < cx {
                    ramp.base
                } else {
                    ramp.shadow
                };
                put(&mut s, x, y, color);
            }
        }
    }
    put(&mut s, cx + 2, rim, GILT.light);
    put(&mut s, cx + 3, rim, GILT.base);
    water::sparkle(&mut s, (cx + 2, rim - 1), 1);
    // The wall, in courses of pebbles round the curve, lit from the left.
    for y in rim + 2..=ground {
        let course = (y - rim - 2) / 3;
        let across_row = (y - rim - 2) % 3;
        for x in left..=right {
            let shifted = x + (course % 2) * 3;
            let stone = (shifted / 5, course);
            let ramp = [PEBBLE, PEBBLE_WARM, STONE][(noise(stone.0, stone.1, 481) % 3) as usize];
            let across = (x - left) as f32 / (right - left) as f32;
            let mut color = if x == left || x == right || y == ground {
                ramp.edge
            } else if shifted % 5 == 0 || across_row == 2 {
                mix(cylinder(ramp, across), ramp.edge, 0.55)
            } else {
                cylinder(ramp, across)
            };
            if color != ramp.edge && chance(x, y, 482, 8) {
                color = rgb(0xc8c070);
            }
            put(&mut s, x, y, color);
        }
    }
    moss(&mut s, 7.0, 36.0, 3.0, 1.4, 483);
    moss(&mut s, 24.0, 43.0, 3.6, 1.4, 484);
    // The roof: courses of wooden shingles on two pitches, moss along the ridge, dark under the
    // eaves.
    for x in 1..=33 {
        put(&mut s, x, 16, IRON.edge);
    }
    polygon(&mut s, &[(0, 16), (17, 2), (34, 16)], |x, y| {
        let row = (y - 2) / 2;
        let joint = (x + (row % 2) * 2) % 4 == 0;
        let lit = x < cx;
        Some(if y == 15 || ((y - 2) % 2 == 1 && !lit) {
            SHINGLE.shadow
        } else if joint {
            SHINGLE.edge
        } else if lit && (y - 2) % 2 == 0 {
            SHINGLE.light
        } else if lit {
            SHINGLE.base
        } else {
            mix(SHINGLE.base, SHINGLE.shadow, 0.5)
        })
    });
    for step in 0..=17 {
        let y = 2 + (step * 14) / 17;
        put(&mut s, cx - step, y, SHINGLE.edge);
        put(&mut s, cx + step, y, SHINGLE.edge);
    }
    put(&mut s, cx, 1, SHINGLE.edge);
    put(&mut s, cx, 2, SHINGLE.shine);
    moss(&mut s, cx as f32 - 1.0, 4.5, 4.5, 2.2, 487);
    moss(&mut s, 7.0, 12.5, 3.0, 1.4, 488);
    flowers(&mut s, 3, ground - 1, 2, 2, 485);
    tuft(&mut s, 31, ground, 2, 486);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// The great telescope: a long brass tube in sections, leather-bound in the middle, with a finder
/// scope riding on it, held in a brass yoke on a turntable on a stone pier; a stool by the eyepiece
/// for whoever is looking.
fn great_telescope() -> Piece {
    let mut s = Canvas::new(46, 52);
    let (cx, ground) = (21, 49);
    shadow(&mut s, cx, ground, 19, 3);
    // The pier.
    earth::stone_block(&mut s, 12, 45, 18, 4, 490);
    earth::stone_block(&mut s, 15, 31, 12, 14, 491);
    for ring in [35, 41] {
        hline(&mut s, 16, ring, 10, STONE.shadow);
        hline(&mut s, 16, ring + 1, 10, STONE.light);
    }
    earth::stone_block(&mut s, 13, 28, 16, 3, 492);
    moss(&mut s, 18.0, 44.0, 3.0, 1.0, 493);
    // The turntable, and the yoke up from it.
    for x in 14..=28_i32 {
        let y = 27 + i32::from((x - 21).abs() > 5);
        put(
            &mut s,
            x,
            y,
            if x < 18 {
                BRASS_RAMP.shine
            } else {
                BRASS_RAMP.base
            },
        );
        put(&mut s, x, y + 1, BRASS_RAMP.edge);
    }
    hollow::stick(&mut s, (17.0, 27.0), (19.0, 21.0), (2.0, 1.6), BRASS_RAMP);
    hollow::stick(&mut s, (25.0, 27.0), (23.0, 21.0), (2.0, 1.6), BRASS_RAMP);
    // The tube, from the eyepiece low on the left to the dew cap high on the right.
    let (from, to) = ((3.0_f32, 37.0_f32), (44.5_f32, 4.0_f32));
    tube(
        &mut s,
        from,
        to,
        &[
            (0.06, 1.4, IRON),
            (0.2, 1.9, BRASS_RAMP),
            (0.28, 2.6, BRASS_RAMP),
            (0.6, 3.0, LEATHER),
            (0.86, 3.0, BRASS_RAMP),
            (1.0, 3.8, BRASS_RAMP),
        ],
    );
    // The finder scope, riding on top near the front on two little brackets.
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = (dy / length, -dx / length);
    let along = |t: f32, out: f32| (from.0 + dx * t + nx * out, from.1 + dy * t + ny * out);
    for t in [0.62, 0.74] {
        let (x0, y0) = along(t, 2.6);
        let (x1, y1) = along(t, 4.6);
        for (x, y) in trace(&[(x0 as i32, y0 as i32), (x1 as i32, y1 as i32)]) {
            put(&mut s, x, y, IRON.base);
        }
    }
    tube(
        &mut s,
        along(0.56, 5.6),
        along(0.8, 5.6),
        &[(0.15, 1.2, IRON), (1.0, 1.5, BRASS_RAMP)],
    );
    // The pivot it turns on.
    let (px, py) = along(0.434, 0.0);
    model(&mut s, BRASS_RAMP, ROUND, 0, 494, |x, y| {
        in_ellipse(x, y, (px as i32, py as i32), (1, 1))
    });
    // The great lens, catching the sky.
    let (lx, ly) = along(0.995, 0.0);
    put(&mut s, lx as i32, ly as i32 - 1, GLASS[3]);
    put(&mut s, lx as i32 + 1, ly as i32, GLASS[2]);
    put(&mut s, lx as i32 - 1, ly as i32 - 2, GLASS[1]);
    // A stool by the eyepiece.
    for x in 1..=9 {
        put(&mut s, x, 43, if x < 4 { PLANK.shine } else { PLANK.light });
        put(&mut s, x, 44, PLANK.base);
        put(&mut s, x, 45, PLANK.edge);
    }
    for x in [2, 8] {
        vline(&mut s, x, 46, 4, PLANK.shadow);
        vline(&mut s, x + 1, 46, 4, PLANK.edge);
    }
    tuft(&mut s, 33, ground, 3, 495);
    tuft(&mut s, 11, ground, 2, 496);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

const BRASS_RAMP: Ramp = earth::BRASS;
const LEATHER: Ramp = Ramp::new(0x3e2419, 0x5e3624, 0x7b4a31, 0x96603f, 0xb27b52);

/// A tube from `from` to `to` in sections: each runs up to its fraction of the way along, as wide
/// as its half-width either side, in its own material; rings of shine where one meets the next.
fn tube(s: &mut Canvas, from: (f32, f32), to: (f32, f32), sections: &[(f32, f32, Ramp)]) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt().max(0.01);
    let (ux, uy) = (dx / length, dy / length);
    let (nx, ny) = (-uy, ux);
    let reach = sections
        .iter()
        .map(|(_, half, _)| *half)
        .fold(0.0, f32::max)
        .ceil() as i32
        + 1;
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
            let Some(&(_, half, ramp)) = sections.iter().find(|(until, _, _)| t <= *until) else {
                continue;
            };
            let across = px * nx + py * ny;
            if across.abs() > half {
                continue;
            }
            let k = across / half;
            let joint = sections
                .iter()
                .any(|(until, _, _)| *until < 1.0 && (t - until).abs() * length < 0.7);
            let color = if half - across.abs() < 0.7 || t * length > length - 0.8 {
                ramp.edge
            } else if joint {
                BRASS_RAMP.shine
            } else if k < -0.45 {
                ramp.light
            } else if k > 0.35 {
                ramp.shadow
            } else {
                ramp.base
            };
            put(s, x, y, color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Landmarks
// ---------------------------------------------------------------------------------------------

/// The bandstand: a striped dome on slender white pillars, a scalloped valance round its eaves
/// with glass chimes hung from it, a lamp hung in the middle and the music box playing under it on
/// a little round table; a railed, latticed platform with steps up at the front.
fn bandstand() -> Piece {
    let mut s = Canvas::new(48, 54);
    let (cx, ground) = (24, 51);
    shadow(&mut s, cx, ground, 23, 3);
    // The far side, in the roof's shade: its rail and pillars seen between the near ones.
    for x in 6..=42 {
        put(&mut s, x, 34, TRIM.shadow);
        put(&mut s, x, 35, TRIM.edge);
        if x % 3 == 0 {
            vline(&mut s, x, 36, 5, TRIM.edge);
        }
    }
    for x in [11, 37] {
        vline(&mut s, x, 22, 19, TRIM.shadow);
        vline(&mut s, x + 1, 22, 19, TRIM.edge);
    }
    // The floor.
    for x in 3..=45 {
        put(&mut s, x, 41, TRIM.light);
        put(
            &mut s,
            x,
            42,
            if x % 4 == 0 { TRIM.shadow } else { TRIM.base },
        );
        put(&mut s, x, 43, TRIM.edge);
    }
    // The skirt below it: white lattice over the dark beneath.
    for y in 44..=ground {
        for x in 3..=45 {
            let color = if x == 3 || x == 45 || y == ground {
                TRIM.edge
            } else if (x + y) % 4 == 0 || (x - y).rem_euclid(4) == 0 {
                if x < 10 { TRIM.light } else { TRIM.base }
            } else {
                ENAMEL_DEEP
            };
            put(&mut s, x, y, color);
        }
    }
    // Steps up the front.
    for (step, (y, half)) in [(48, 8), (46, 7), (44, 6)].into_iter().enumerate() {
        for x in cx - half..=cx + half {
            let edge = x == cx - half || x == cx + half;
            put(&mut s, x, y, if edge { TRIM.edge } else { TRIM.light });
            put(&mut s, x, y + 1, if edge { TRIM.edge } else { TRIM.shadow });
            if step == 0 {
                put(&mut s, x, y + 2, TRIM.edge);
                put(&mut s, x, y + 3, TRIM.edge);
            }
        }
    }
    // The music box on its little round table.
    vline(&mut s, cx, 39, 2, PLANK.shadow);
    hline(&mut s, cx - 3, 41, 7, PLANK.edge);
    for x in cx - 5..=cx + 5 {
        put(
            &mut s,
            x,
            38,
            if x < cx - 2 { PLANK.shine } else { PLANK.light },
        );
        put(&mut s, x, 39, PLANK.edge);
    }
    music_box(&mut s, (cx - 4, 28));
    // The lamp, hung from the middle of the roof.
    vline(&mut s, cx, 23, 2, IRON.light);
    paint_rows(
        &mut s,
        cx - 2,
        24,
        &[".###.", "#yGy#", "#yfy#", "#####", "..#.."],
        IRON,
        &[('y', GLOW[0]), ('G', GLOW[1]), ('f', rgb(0xfffbe8))],
    );
    // The near pillars, and the rail between them either side of the steps.
    for x in [5, 16, 31, 42] {
        for y in 22..=41 {
            put(&mut s, x, y, TRIM.shine);
            put(&mut s, x + 1, y, TRIM.shadow);
        }
        hline(&mut s, x - 1, 22, 4, TRIM.light);
        hline(&mut s, x - 1, 23, 4, TRIM.edge);
        hline(&mut s, x - 1, 40, 4, TRIM.base);
    }
    for (from, to) in [(6, 16), (33, 42)] {
        for x in from..to {
            put(&mut s, x, 34, TRIM.light);
            put(&mut s, x, 35, TRIM.edge);
            if x % 2 == 0 {
                vline(&mut s, x, 36, 4, TRIM.base);
            }
        }
    }
    // The tune, drifting out between the pillars.
    for (x, y) in [(20, 27), (30, 29)] {
        paint_rows(
            &mut s,
            x,
            y,
            &[".#.", ".##", ".#.", "l#.", "##."],
            PLUM,
            &[],
        );
    }
    // The roof: a dome of red and cream panels coming together at the top.
    let half = |y: i32| 22.5 * ((y - 2) as f32 / 18.0).clamp(0.0, 1.0).powf(0.55);
    let on_roof =
        |x: i32, y: i32| (3..=20).contains(&y) && (x as f32 + 0.5 - 24.0).abs() <= half(y);
    for y in 3..=20 {
        for x in 0..48 {
            if !on_roof(x, y) {
                continue;
            }
            let u = (x as f32 + 0.5 - 24.0) / half(y);
            let panel = ((u.clamp(-1.0, 1.0).asin() / PI + 0.5) * 8.0) as i32;
            let ramp = if panel % 2 == 0 { TILE } else { STRIPE };
            let rim = !on_roof(x - 1, y) || !on_roof(x + 1, y) || !on_roof(x, y - 1);
            let jitter = (noise(x, y, 500) & 0xff) as f32 / 255.0 - 0.5;
            let lit = -0.8 * u - 0.4 * ((20 - y) as f32 / 18.0) + 0.25 * jitter + 0.2;
            let color = if rim {
                ramp.edge
            } else if lit > 0.55 {
                ramp.light
            } else if lit > -0.1 {
                ramp.base
            } else {
                ramp.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    // The eaves, and the valance scalloped under them.
    for x in 0..48 {
        put(&mut s, x, 20, TRIM.light);
        put(&mut s, x, 21, TRIM.edge);
        let scallop = (x - 1).rem_euclid(4);
        let ramp = if ((x - 1) / 4) % 2 == 0 { TILE } else { STRIPE };
        put(&mut s, x, 22, ramp.base);
        if (1..=2).contains(&scallop) {
            put(&mut s, x, 23, ramp.shadow);
        }
    }
    // The finial.
    model(&mut s, GILT, ROUND, 0, 501, |x, y| {
        in_ellipse(x, y, (24, 2), (1, 1))
    });
    put(&mut s, 24, 0, GILT.base);
    // Glass chimes hung along the eaves.
    for (x, drop, glass) in [
        (8, 3, SEA_GLASS),
        (13, 1, BLUE_GLASS),
        (35, 2, TEAL_GLASS),
        (40, 3, PALE_GLASS),
    ] {
        vline(&mut s, x, 23, drop, STRING);
        stamp(
            &mut s,
            (x - 1, 23 + drop),
            &[".#.", "#*#", "#o#", ".#."],
            &water::glass_inks(glass),
            false,
        );
    }
    tuft(&mut s, 2, ground, 2, 502);
    tuft(&mut s, 46, ground, 2, 503);
    Piece {
        sprite: s,
        anchor: (cx, ground),
    }
}

/// The bandstand roof's pale stripes.
const STRIPE: Ramp = Ramp::new(0xb8a07a, 0xdcc8a4, 0xf0e2c2, 0xfaf2dc, 0xffffff);

/// The music box, small, with its top-left at `(x, y)`: the lid thrown back, velvet and a glint of
/// mirror in it, and the lacquered box in front with its gilt rim.
fn music_box(s: &mut Canvas, (x, y): (i32, i32)) {
    for row in 2..5 {
        for column in 1..8 {
            let color = if row == 2 || column == 1 || column == 7 {
                LACQUER.edge
            } else if column == 4 && row == 3 {
                GLASS[3]
            } else if row == 3 {
                VELVET.light
            } else {
                VELVET.base
            };
            put(s, x + column, y + row, color);
        }
    }
    for row in 5..10 {
        for column in 0..9 {
            let color = if row == 9 || column == 0 || column == 8 {
                LACQUER.edge
            } else if row == 5 {
                GILT.light
            } else if column == 1 || row == 6 {
                LACQUER.light
            } else if column >= 7 {
                LACQUER.shadow
            } else {
                LACQUER.base
            };
            put(s, x + column, y + row, color);
        }
    }
    put(s, x + 4, y + 7, GILT.shine);
}

/// Where the burrow house's back room has its round window, in the hump of turf behind.
const BACK_WINDOW: (i32, i32) = (13, 21);

/// The burrow house: the little door in the hill, grown into a home. A back room in a second hump
/// of turf behind, with its own round window; a porch over the door roofed in pinecone scales on
/// two posts, with a lantern under it; a pebble doorstep; and smoke from the chimney.
fn burrow_house() -> Piece {
    let mut s = Canvas::new(48, 55);
    let ground = 49;
    // The back room's hump, lit from the upper left, grass standing up along its top.
    let (hx, hy, rx, ry) = (14.0, 34.0, 13.0, 22.0);
    let hump = |x: i32, y: i32| {
        let (dx, dy) = ((x as f32 + 0.5 - hx) / rx, (y as f32 + 0.5 - hy) / ry);
        y <= hy as i32 && dx * dx + dy * dy <= 1.0
    };
    for y in 0..=hy as i32 {
        for x in 0..28 {
            if !hump(x, y) {
                continue;
            }
            let edge = !(hump(x - 1, y) && hump(x + 1, y) && hump(x, y - 1));
            let (u, v) = ((x as f32 + 0.5 - hx) / rx, (y as f32 + 0.5 - hy) / ry);
            let mut color = if edge {
                TURF.edge
            } else {
                lit(TURF, u, v, noise(x, y, 511))
            };
            if !edge && chance(x, y, 512, 40) {
                color = if chance(x, y, 513, 128) {
                    TURF.shadow
                } else {
                    TURF.light
                };
            }
            put(&mut s, x, y, color);
        }
    }
    for x in 3..26 {
        if let Some(top) = (0..=hy as i32).find(|&y| hump(x, y))
            && chance(x, 0, 514, 80)
        {
            put(&mut s, x, top - 1, TURF.edge);
            put(&mut s, x, top, TURF.base);
        }
    }
    let (wx, wy) = BACK_WINDOW;
    for (dx, dy) in earth::ring_offsets(3.0) {
        put(&mut s, wx + dx, wy + dy, POST.edge);
    }
    for (dx, dy) in earth::ring_offsets(2.4) {
        put(&mut s, wx + dx, wy + dy, POST.base);
    }
    for dy in -1..=1 {
        for dx in -1..=1 {
            put(
                &mut s,
                wx + dx,
                wy + dy,
                if dx + dy < 0 { GLOW[2] } else { GLOW[1] },
            );
        }
    }
    put(&mut s, wx, wy, POST.shadow);
    // The front room: the little door's own hill, in front.
    let door = earth::hill_door();
    blit(
        &mut s,
        &door.sprite,
        24 - door.anchor.0,
        ground - door.anchor.1,
    );
    // The porch: two posts, and a roof of pinecone scales over the door.
    for x in [13.5, 34.5] {
        hollow::stick(
            &mut s,
            (x, 27.0),
            (x, ground as f32 + 0.5),
            (1.6, 1.6),
            POLE,
        );
    }
    hline(&mut s, 12, 27, 25, POLE.edge);
    for y in 19..=26_i32 {
        let half = (y - 19) * 13 / 7;
        for x in 24 - half..=24 + half {
            let row = y - 19;
            let scale = (x + (row % 2) * 2).rem_euclid(4);
            let lit = x < 24;
            let color = if scale == 3 || x == 24 - half || x == 24 + half {
                CONE.edge
            } else if scale == 0 && lit {
                CONE.shine
            } else if lit {
                CONE.light
            } else if scale == 0 {
                CONE.base
            } else {
                CONE.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    // A little lantern hung under the porch.
    vline(&mut s, 31, 28, 1, IRON.light);
    paint_rows(
        &mut s,
        30,
        29,
        &[".#.", "#y#", "#f#", ".#."],
        IRON,
        &[('y', GLOW[1]), ('f', GLOW[2])],
    );
    // The pebble doorstep.
    lump(
        &mut s,
        24.0,
        ground as f32 + 0.6,
        4.2,
        1.6,
        PEBBLE_WARM,
        510,
    );
    // Smoke from the chimney, drifting up and away.
    for (index, &(x, y, r)) in [
        (35.0, 19.5, 1.4),
        (36.0, 15.5, 2.0),
        (35.0, 10.5, 2.4),
        (37.0, 5.5, 2.0),
        (38.5, 1.8, 1.4),
    ]
    .iter()
    .enumerate()
    {
        let alpha = 190 - 30 * index as u8;
        let reach = (r as i32) + 1;
        for py in (y as i32) - reach..=(y as i32) + reach {
            for px in (x as i32) - reach..=(x as i32) + reach {
                let (dx, dy) = (px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let color = if dx + dy < -0.5 {
                    rgb(0xf8f6f2)
                } else {
                    rgb(0xd6d2cc)
                };
                put(&mut s, px, py, Rgba::new(color.r, color.g, color.b, alpha));
            }
        }
    }
    Piece {
        sprite: s,
        anchor: (24, ground),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_light_shines_from_inside_its_piece() {
        for plan in &crate::finds::plans::PLANS {
            let drawn = piece(plan.id).expect("every plan is drawn");
            for (_, (x, y)) in lights(plan.id) {
                assert!(
                    drawn.sprite.get(x, y).a > 0,
                    "{}'s light at ({x}, {y}) shines from nothing",
                    plan.id
                );
            }
        }
    }
}
