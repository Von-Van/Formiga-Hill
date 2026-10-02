//! The finds in the earth: the finds dug up (Kind::Dig): their icons and their Hilltop pieces.
//!
//! Also the few painting helpers the hollows share: pictures from rows of codes, a lump lit from
//! the upper left, a soft shadow on the grass, curves to draw stems and blades along.

use super::{ICON, Piece};
use crate::materials::{BLOSSOMS, GLOW, STONE};
use crate::paint::{Ramp, chance, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};

const PEBBLE: Ramp = Ramp::new(0x5a524e, 0x7a716b, 0x999088, 0xb6aea4, 0xd4cdc2);
const PEBBLE_WARM: Ramp = Ramp::new(0x5e4f45, 0x80695b, 0xa18977, 0xbea692, 0xd8c5b0);
const MOSS: Ramp = Ramp::new(0x34512a, 0x4a6e34, 0x638c40, 0x80aa52, 0xa4c872);
const BULB: Ramp = Ramp::new(0x7a5a3e, 0xae8c66, 0xd2b88e, 0xe8d6b0, 0xf8eed8);
const BLUEBELL: Ramp = Ramp::new(0x2c2f74, 0x3f4aa0, 0x5668c8, 0x7d8ee0, 0xb0bef2);
const STEM: Ramp = Ramp::new(0x2f5a2e, 0x3f7a3a, 0x58964a, 0x76b25c, 0x9ccc7e);
const STRAP: Ramp = Ramp::new(0x24482c, 0x346838, 0x488a46, 0x66a858, 0x8cc672);
const SHOE: Ramp = Ramp::new(0x3e322c, 0x5a4a42, 0x76645a, 0x988474, 0xbaa694);
const RUST: Ramp = Ramp::new(0x4a2a1e, 0x6e3c26, 0x925232, 0xb06c42, 0xcc8c5c);
const WEATHERED: Ramp = Ramp::new(0x56504a, 0x7a726a, 0x9a9288, 0xb8b0a4, 0xd4ccc0);
const BRASS: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const GATE: Ramp = Ramp::new(0x3c5a56, 0x5a807a, 0x7ca69c, 0xa0c4b8, 0xc8e0d6);
const POST: Ramp = Ramp::new(0x3e2a20, 0x5a3e2e, 0x76543c, 0x92704e, 0xae8c66);
const GLAZE: Ramp = Ramp::new(0x9a8868, 0xcbbc98, 0xeadfc0, 0xf6eed8, 0xfffaf0);
const DELFT: Ramp = Ramp::new(0x22357a, 0x2f4a9a, 0x4064b8, 0x6888d0, 0x9cb4e4);
const MEND: Ramp = Ramp::new(0x8a5e14, 0xb8861e, 0xdcac30, 0xf2cc58, 0xfff0a8);
const SOIL: Ramp = Ramp::new(0x2e2018, 0x46301f, 0x5c402a, 0x74543a, 0x8c6a4c);
const BASIL: Ramp = Ramp::new(0x2a5228, 0x3a7232, 0x509640, 0x70b654, 0x9cd47a);
const ROSEMARY: Ramp = Ramp::new(0x2a4238, 0x3a5a4a, 0x4e7460, 0x6a8e78, 0x8eac96);
const TURF: Ramp = Ramp::new(0x2e5630, 0x44763c, 0x5e944c, 0x7cb262, 0xa2cc80);
const OAK_DOOR: Ramp = Ramp::new(0x4a2a1a, 0x6c3e24, 0x8a5430, 0xa86e42, 0xc48c5c);
const HINGE: Ramp = Ramp::new(0x2e2624, 0x463a36, 0x5e504a, 0x7a6a62, 0x9a8a80);
const CLAY: Rgba = rgb(0xb0684a);

/// The soft shade every piece casts on the summit grass.
const SHADOW: Rgba = rgba(0x2a4a2a, 70);

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    Some(match id {
        // A round grey pebble, polished by the earth.
        "smooth_pebble" => icon_from(
            [
                ".........",
                "..#####..",
                ".#l*loo#.",
                "#llooooo#",
                "#loooooo#",
                "#oooooss#",
                ".#oossss#",
                "..######.",
                ".........",
            ],
            PEBBLE,
            &[],
        ),
        // A papery bulb with a green tip and a beard of roots.
        "bluebell_bulb" => icon_from(
            [
                "....g....",
                "...gG....",
                "...#G#...",
                "..#l*o#..",
                ".#lloos#.",
                ".#looos#.",
                ".#oosss#.",
                "..#####..",
                "..r.r.r..",
            ],
            BULB,
            &[('g', STEM.light), ('G', STEM.base), ('r', BULB.shadow)],
        ),
        // A horseshoe gone to rust, its nail holes showing.
        "old_horseshoe" => icon_from(
            [
                ".##...##.",
                "#*l#.#os#",
                "#ln#.#ns#",
                "#lo#.#os#",
                "#ln#.#ns#",
                "#loo#oos#",
                ".#loooss#",
                "..#####..",
                ".........",
            ],
            RUST,
            &[('n', RUST.edge)],
        ),
        // A brass key: a ring of a bow, a long shank, two teeth.
        "brass_key" => icon_from(
            [
                ".........",
                ".###.....",
                "#l*o#....",
                "#l.s#####",
                "#osoollo#",
                ".####o#o#",
                "....#s#s#",
                ".....#.#.",
                ".........",
            ],
            BRASS,
            &[],
        ),
        // A curve of cream pot with a blue flower on it, the clay showing where it broke.
        "pot_shard" => icon_from(
            [
                ".........",
                ".tt......",
                ".#*tt....",
                ".#lbltt..",
                ".#bwboot.",
                "#llbooos#",
                "#looooss#",
                ".##ssss#.",
                "...####..",
            ],
            GLAZE,
            &[('b', DELFT.base), ('w', DELFT.shine), ('t', CLAY)],
        ),
        // A gold coin with a sun stamped on it.
        "sun_coin" => icon_from(
            [
                "..#####..",
                ".#*llll#.",
                "#ldldlds#",
                "#llowoos#",
                "#ldwwwds#",
                "#loowoss#",
                "#odsdsds#",
                ".#ossss#.",
                "..#####..",
            ],
            MEND,
            &[('d', MEND.edge), ('w', MEND.shine)],
        ),
        // A little arched door with its brass knob.
        "tiny_door" => icon_from(
            [
                "...###...",
                "..#los#..",
                ".#l|o|s#.",
                ".#l|o|s#.",
                ".#l|o|k#.",
                ".#l|o|s#.",
                ".#l|o|s#.",
                ".#######.",
                ".........",
            ],
            OAK_DOOR,
            &[('|', OAK_DOOR.edge), ('k', BRASS.light)],
        ),
        _ => return None,
    })
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "smooth_pebble" => cairn(),
        "bluebell_bulb" => bluebells(),
        "old_horseshoe" => horseshoe_post(),
        "brass_key" => gate(),
        "pot_shard" => herb_pot(),
        "sun_coin" => sundial(),
        "tiny_door" => hill_door(),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// The pieces
// ---------------------------------------------------------------------------------------------

/// A cairn of five rounded stones, each settled a little off the one below, moss on the
/// shoulders that face the weather.
fn cairn() -> Piece {
    let mut s = Canvas::new(18, 24);
    let ground = 21;
    shadow(&mut s, 9, ground, 8, 2);
    let stones = [
        (9.0, 18.2, 7.8, 4.0, 0.0, PEBBLE_WARM),
        (8.6, 12.8, 6.0, 3.4, -0.1, PEBBLE),
        (9.6, 8.1, 4.8, 2.9, 0.14, STONE),
        (8.8, 4.2, 3.6, 2.4, -0.12, PEBBLE_WARM),
        (9.4, 1.6, 2.2, 1.7, 0.2, PEBBLE),
    ];
    for (index, &(cx, cy, rx, ry, tilt, ramp)) in stones.iter().enumerate() {
        if index > 0 {
            // Where it rests, the stone below is in its shade.
            tuck(&mut s, cx + 1.0, cy + 1.2, rx, ry, rgba(0x2a2420, 60));
        }
        tilted_lump(&mut s, (cx, cy), (rx, ry), tilt, ramp, 10 + index as u32);
    }
    moss(&mut s, 5.0, 16.4, 2.6, 1.2, 20);
    moss(&mut s, 5.4, 11.2, 1.8, 0.9, 21);
    moss(&mut s, 13.0, 16.0, 1.4, 0.8, 22);
    Piece {
        sprite: s,
        anchor: (9, ground),
    }
}

/// A clump of bluebells: glossy strappy leaves fanned out over the grass, and stems nodding
/// over at the top with their bells all hanging down one side.
fn bluebells() -> Piece {
    let mut s = Canvas::new(26, 19);
    let ground = 17;
    shadow(&mut s, 13, ground, 11, 2);
    for (tip, via) in [
        ((1.0, 15.0), (6.0, 11.0)),
        ((25.0, 15.0), (20.0, 11.0)),
        ((4.0, 10.0), (8.0, 9.0)),
        ((22.0, 10.0), (18.0, 9.0)),
        ((7.0, 17.0), (9.0, 14.0)),
        ((19.0, 17.0), (17.0, 14.0)),
        ((8.0, 7.0), (11.0, 11.0)),
        ((18.0, 6.0), (15.0, 10.0)),
    ] {
        blade(&mut s, (13.0, ground as f32), via, tip, STRAP);
    }
    // Each stem: where it rises, the row it arches over at, which way it nods, how far, and
    // where along the arch its bells hang.
    let stems: [(i32, i32, i32, i32, &[i32]); 5] = [
        (6, 11, -1, 4, &[2]),
        (19, 13, 1, 4, &[2]),
        (10, 5, -1, 7, &[2, 5]),
        (16, 8, 1, 7, &[2, 5]),
        (13, 1, 1, 8, &[2, 5]),
    ];
    for (x, top, way, reach, bells) in stems {
        // Up, over, and down at the tip to a bud.
        vline(&mut s, x, top + 2, ground - top - 2, STEM.base);
        put(&mut s, x + way, top + 1, STEM.base);
        for step in 2..reach {
            put(&mut s, x + way * step, top, STEM.light);
        }
        put(&mut s, x + way * reach, top + 1, STEM.base);
        put(&mut s, x + way * reach, top + 2, BLUEBELL.shadow);
        put(&mut s, x + way * reach, top + 3, BLUEBELL.edge);
        let left = |at: i32| if way > 0 { x + at } else { x - at - 1 };
        for &at in bells {
            bell(&mut s, left(at), top + 1);
        }
        // One more, lower, hanging off the upright.
        if top + 8 < ground {
            bell(&mut s, left(1), top + 4);
        }
    }
    Piece {
        sprite: s,
        anchor: (13, ground),
    }
}

/// One nodding bell two pixels wide, its top-left at `(x, y)`, its lip curled back.
fn bell(s: &mut Canvas, x: i32, y: i32) {
    put(s, x, y, BLUEBELL.base);
    put(s, x + 1, y, BLUEBELL.shadow);
    put(s, x, y + 1, BLUEBELL.light);
    put(s, x + 1, y + 1, BLUEBELL.base);
    put(s, x, y + 2, BLUEBELL.shine);
    put(s, x + 1, y + 2, BLUEBELL.shadow);
}

/// A short weathered post with an old horseshoe nailed to it, open end up to keep the luck in,
/// and a four-leaf clover at its foot.
fn horseshoe_post() -> Piece {
    let mut s = Canvas::new(16, 31);
    let ground = 29;
    shadow(&mut s, 8, ground, 6, 2);
    // The post, silvered by years of rain, its grain running down it.
    let (left, top) = (5, 4);
    for y in top..=ground {
        for x in left..left + 6 {
            let column = x - left;
            let mut color = match column {
                0 | 5 => WEATHERED.edge,
                1 => WEATHERED.light,
                4 => WEATHERED.shadow,
                _ => WEATHERED.base,
            };
            if (1..5).contains(&column) && noise(x, y / 4, 31).is_multiple_of(5) {
                color = mix(color, WEATHERED.edge, 0.45);
            } else if column < 3 && chance(x, y, 32, 30) {
                color = WEATHERED.shine;
            }
            put(&mut s, x, y, color);
        }
    }
    // The cut top, its end grain catching the light, and a split down from it.
    hline(&mut s, left + 1, top - 1, 4, WEATHERED.edge);
    hline(&mut s, left + 1, top, 4, WEATHERED.shine);
    put(&mut s, left + 4, top, WEATHERED.light);
    vline(&mut s, left + 3, top + 1, 3, WEATHERED.edge);
    // Damp and moss where it goes into the ground.
    for y in ground - 3..=ground {
        for x in left + 1..left + 5 {
            if chance(x, y, 33, 90 + (y - ground + 3) as u32 * 40) {
                put(
                    &mut s,
                    x,
                    y,
                    if x < left + 3 { MOSS.light } else { MOSS.base },
                );
            }
        }
    }
    // The shoe, nailed on with its heels up, its shade on the post beneath it.
    let shoe = [
        "####...####",
        "#l*#...#os#",
        "#lo#...#os#",
        "#ln#...#ns#",
        "#lo#...#os#",
        "#lo#...#os#",
        "#ln##.##ns#",
        "#loo###oos#",
        ".#looooos#.",
        "..#oosss#..",
        "...#####...",
    ];
    let (shoe_x, shoe_y) = (3, 8);
    for (y, row) in shoe.iter().enumerate() {
        for (x, code) in row.chars().enumerate() {
            let (px, py) = (shoe_x + 1 + x as i32, shoe_y + 1 + y as i32);
            if code != '.' && s.get(px, py).a == 255 {
                put(&mut s, px, py, rgba(0x2a1e18, 110));
            }
        }
    }
    paint_rows(&mut s, shoe_x, shoe_y, &shoe, RUST, &[('n', RUST.edge)]);
    // Bare iron where the rust has flaked away.
    for (y, row) in shoe.iter().enumerate() {
        for (x, code) in row.chars().enumerate() {
            let (px, py) = (shoe_x + x as i32, shoe_y + y as i32);
            if matches!(code, 'o' | 's') && chance(px, py, 34, 60) {
                put(
                    &mut s,
                    px,
                    py,
                    if code == 'o' { SHOE.light } else { SHOE.base },
                );
            }
        }
    }
    // Grass about the foot, and the clover.
    tuft(&mut s, 8, ground, 6, 35);
    clover(&mut s, 3, 26);
    Piece {
        sprite: s,
        anchor: (8, ground),
    }
}

/// A four-leaf clover with its centre at `(x, y)`.
fn clover(s: &mut Canvas, x: i32, y: i32) {
    vline(s, x, y + 1, 3, STEM.shadow);
    for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (lx, ly) = (x + dx, y + dy);
        put(s, lx, ly, STEM.base);
        put(s, lx + dx, ly, STEM.shadow);
        put(s, lx, ly + dy, STEM.shadow);
    }
    put(s, x - 1, y - 1, STEM.shine);
    put(s, x - 2, y - 1, STEM.light);
    put(s, x, y, STEM.edge);
}

/// A picket gate in a garden that isn't there, hung between two posts and left a little open,
/// with a brass keyhole plate and flowers growing at its foot.
fn gate() -> Piece {
    let mut s = Canvas::new(38, 33);
    let ground = 31;
    shadow(&mut s, 19, ground, 17, 2);
    // Posts with capped tops and round finials, the gate hung on the left one.
    for left in [2, 31] {
        for y in 7..=ground {
            for x in left..left + 5 {
                let column = x - left;
                let mut color = match column {
                    0 | 4 => POST.edge,
                    1 => POST.light,
                    3 => POST.shadow,
                    _ => POST.base,
                };
                if (1..4).contains(&column) && noise(x, y / 3, 41).is_multiple_of(6) {
                    color = mix(color, POST.edge, 0.4);
                }
                put(&mut s, x, y, color);
            }
        }
        hline(&mut s, left - 1, 5, 7, POST.edge);
        hline(&mut s, left, 5, 5, POST.light);
        hline(&mut s, left - 1, 6, 7, POST.edge);
        put(&mut s, left, 5, POST.shine);
        paint_rows(
            &mut s,
            left,
            1,
            &[".###.", "#*lo#", "#los#", ".###."],
            POST,
            &[],
        );
    }
    // The gate swings towards us from its hinges, so its far end stands a touch lower.
    let (hinge, latch) = (8, 27);
    let sag = |x: i32| ((x - hinge) as f32 / (latch - hinge) as f32 * 2.0).round() as i32;
    // Rails and a brace, seen between the pickets.
    for rail in [12, 25] {
        for x in hinge..=latch {
            let y = rail + sag(x);
            put(&mut s, x, y - 1, GATE.base);
            put(&mut s, x, y, GATE.shadow);
            put(&mut s, x, y + 1, GATE.edge);
        }
    }
    for x in hinge + 1..latch {
        let t = (x - hinge) as f32 / (latch - hinge) as f32;
        let y = (24.0 - t * 11.0).round() as i32 + sag(x);
        put(&mut s, x, y - 1, GATE.base);
        put(&mut s, x, y, GATE.shadow);
        put(&mut s, x, y + 1, GATE.edge);
    }
    // Pickets, pointed, the paint worn through to grey wood here and there.
    for (index, x) in [hinge, 13, 18, 23].into_iter().enumerate() {
        let drop = sag(x + 1);
        let (top, bottom) = (7 + drop + (index as i32 % 2), 28 + drop);
        for y in top..=bottom {
            for dx in 0..4 {
                let color = match (dx, y - top) {
                    (1, 0) | (2, 0) => GATE.edge,
                    (_, 0) => continue,
                    (0, 1) | (3, 1) => GATE.edge,
                    (1, 1) => GATE.shine,
                    (2, 1) => GATE.light,
                    (0, _) | (3, _) => GATE.edge,
                    (1, _) if chance(x, y, 43, 26) => WEATHERED.light,
                    (1, _) => GATE.light,
                    _ if chance(x + 1, y, 44, 26) => WEATHERED.base,
                    _ => GATE.base,
                };
                put(&mut s, x + dx, y, color);
            }
        }
        hline(&mut s, x, bottom + 1, 4, GATE.edge);
    }
    // The latch stile, and iron hinges across to the post.
    let stile = latch - 1;
    for y in 9 + sag(stile)..=29 + sag(stile) {
        put(&mut s, stile, y, GATE.base);
        put(&mut s, stile + 1, y, GATE.edge);
    }
    put(&mut s, stile, 9 + sag(stile), GATE.light);
    for rail in [12, 25] {
        hline(&mut s, 5, rail - 1, 6, HINGE.light);
        hline(&mut s, 5, rail, 6, HINGE.edge);
        put(&mut s, 10, rail - 1, HINGE.shine);
    }
    // The keyhole plate on the latch side, brass gone soft with handling.
    let plate = 16 + sag(23);
    rect(&mut s, 23, plate, 4, 6, BRASS.edge);
    rect(&mut s, 24, plate + 1, 2, 4, BRASS.base);
    put(&mut s, 24, plate + 1, BRASS.shine);
    vline(&mut s, 24, plate + 2, 2, BRASS.light);
    put(&mut s, 25, plate + 2, BRASS.edge);
    put(&mut s, 25, plate + 3, BRASS.edge);
    // Flowers at the foot of the gate and about the far post.
    for (cx, salt) in [(10, 44), (29, 45), (19, 46)] {
        tuft(&mut s, cx, ground, 3, salt);
        flowers(&mut s, cx, ground - 2, 3, 3, salt);
    }
    Piece {
        sprite: s,
        anchor: (19, ground),
    }
}

/// A cream pot painted with blue flowers, broken once and mended with gold along the crack, full
/// of herbs: basil, rosemary in flower, and thyme.
fn herb_pot() -> Piece {
    let mut s = Canvas::new(22, 27);
    let ground = 25;
    shadow(&mut s, 11, ground, 9, 2);
    // The soil inside the rim, and the back of the rim behind it.
    let (rim_top, rim_left, rim_width) = (11, 2, 18);
    hline(&mut s, rim_left + 1, rim_top - 1, rim_width - 2, GLAZE.edge);
    hline(&mut s, rim_left + 1, rim_top, rim_width - 2, SOIL.base);
    // Herbs: rosemary in flower at the back, thyme between, basil at the front.
    for (tip, via) in [
        ((15.0, 0.0), (14.0, 6.0)),
        ((18.0, 3.0), (16.0, 7.0)),
        ((12.0, 2.0), (13.0, 6.0)),
        ((20.0, 6.0), (17.0, 8.0)),
    ] {
        let points = curve((14.0, 11.0), via, tip);
        for (index, &(x, y)) in points.iter().enumerate() {
            put(&mut s, x, y, ROSEMARY.shadow);
            if index % 2 == 0 {
                put(&mut s, x - 1, y, ROSEMARY.light);
                put(&mut s, x + 1, y, ROSEMARY.base);
            }
        }
        let (x, y) = points[points.len() - 1];
        put(&mut s, x, y, rgb(0xa8b8e8));
        put(&mut s, x, y + 2, rgb(0x8898d0));
    }
    lump(&mut s, 11.5, 8.4, 3.6, 2.8, ROSEMARY, 51);
    for (x, y) in [(10, 6), (12, 7), (10, 8), (13, 9), (11, 5), (12, 6)] {
        put(&mut s, x, y, rgb(0xc8a0d8));
    }
    lump(&mut s, 6.4, 8.6, 4.2, 3.2, BASIL, 52);
    lump(&mut s, 8.4, 5.6, 2.8, 2.4, BASIL, 53);
    lump(&mut s, 4.0, 6.6, 2.0, 1.8, BASIL, 54);
    // The pot: a rolled rim, then a body narrowing to its foot, lit from the left.
    let body = |y: i32| {
        let t = (y - (rim_top + 3)) as f32 / 10.0;
        let half = 7.0 - t * 2.0;
        ((11.0 - half).round() as i32, (11.0 + half).round() as i32)
    };
    for y in rim_top + 3..=ground - 1 {
        let (left, right) = body(y);
        for x in left..right {
            let across = (x - left) as f32 / (right - left - 1) as f32;
            let color = if x == left || x == right - 1 || y == ground - 1 {
                GLAZE.edge
            } else {
                cylinder(GLAZE, across)
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 5, rim_top + 3, 12, GLAZE.shadow);
    for y in rim_top + 1..rim_top + 4 {
        for x in rim_left..rim_left + rim_width {
            let across = (x - rim_left) as f32 / (rim_width - 1) as f32;
            let color = if x == rim_left || x == rim_left + rim_width - 1 || y == rim_top + 3 {
                GLAZE.edge
            } else {
                cylinder(GLAZE, across)
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, rim_left + 1, rim_top + 2, rim_width - 2, DELFT.base);
    // Blue flowers painted round the body, with a sprig between.
    for (x, y) in [(6, 18), (15, 20)] {
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            put(&mut s, x + dx, y + dy, DELFT.base);
        }
        put(&mut s, x, y, DELFT.shine);
    }
    line(&mut s, (8, 20), (12, 17), DELFT.light);
    put(&mut s, 10, 18, DELFT.base);
    put(&mut s, 11, 19, DELFT.base);
    hline(&mut s, 7, 23, 8, DELFT.light);
    // The mended crack, from the rim down, in gold.
    let crack = [(13, 12), (12, 15), (14, 17), (13, 19), (14, 22), (13, 24)];
    for pair in crack.windows(2) {
        line(&mut s, pair[0], pair[1], MEND.base);
    }
    put(&mut s, 12, 15, MEND.shine);
    put(&mut s, 14, 17, MEND.light);
    put(&mut s, 14, 22, MEND.light);
    Piece {
        sprite: s,
        anchor: (11, ground),
    }
}

/// A sundial: a stone pedestal, lichened, with a brass dial on top and its gnomon throwing a
/// shadow across the hours.
fn sundial() -> Piece {
    let mut s = Canvas::new(22, 31);
    let ground = 29;
    shadow(&mut s, 11, ground, 10, 2);
    stone_block(&mut s, 2, 26, 18, 4, 61);
    stone_block(&mut s, 5, 24, 12, 2, 62);
    stone_block(&mut s, 7, 12, 8, 12, 63);
    for ring in [14, 21] {
        hline(&mut s, 8, ring, 6, STONE.shadow);
        hline(&mut s, 8, ring + 1, 6, STONE.light);
    }
    stone_block(&mut s, 3, 9, 16, 3, 64);
    // The capital's top, seen a little from above.
    polygon(&mut s, &[(4, 6), (18, 6), (19, 9), (3, 9)], |x, y| {
        Some(if y == 6 {
            STONE.edge
        } else if x < 7 {
            STONE.shine
        } else {
            STONE.light
        })
    });
    // Lichen on the stone, and moss at the foot of the column.
    for y in 9..ground {
        for x in 2..20 {
            if s.get(x, y).a == 255 && chance(x, y, 65, 10) {
                put(&mut s, x, y, rgb(0xc8c070));
            }
        }
    }
    moss(&mut s, 9.0, 23.5, 3.5, 1.2, 66);
    // The dial plate, its hours marked round the edge.
    let (cx, cy) = (11.0, 7.5);
    for y in 5..11 {
        for x in 3..20 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / 6.4, (y as f32 + 0.5 - cy) / 1.9);
            let d = dx * dx + dy * dy;
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.8 {
                BRASS.edge
            } else if dx < -0.3 {
                BRASS.light
            } else {
                BRASS.base
            };
            put(&mut s, x, y, color);
        }
    }
    for x in [7, 9, 13, 15] {
        put(&mut s, x, 6, BRASS.shadow);
    }
    put(&mut s, 6, 7, BRASS.shine);
    // The gnomon's shadow falls across the afternoon hours, and the gnomon stands over it.
    hline(&mut s, 13, 8, 3, BRASS.edge);
    for x in 9..=13 {
        let top = 7 - (x - 9);
        vline(&mut s, x, top, 8 - top, BRASS.base);
        put(&mut s, x, top, BRASS.shine);
    }
    vline(&mut s, 13, 3, 5, BRASS.shadow);
    put(&mut s, 13, 2, BRASS.edge);
    put(&mut s, 14, 3, BRASS.edge);
    // Grass and a few flowers about the plinth.
    tuft(&mut s, 3, ground, 3, 67);
    tuft(&mut s, 19, ground, 3, 68);
    flowers(&mut s, 18, ground - 2, 2, 2, 69);
    Piece {
        sprite: s,
        anchor: (11, ground),
    }
}

/// A block of dressed stone lit from the upper left, speckled.
fn stone_block(s: &mut Canvas, x: i32, y: i32, width: i32, height: i32, salt: u32) {
    for py in y..y + height {
        for px in x..x + width {
            let (col, row) = (px - x, py - y);
            let mut color = if col == 0 || col == width - 1 || row == height - 1 {
                STONE.edge
            } else if row == 0 || col == 1 {
                STONE.light
            } else if col >= width - 3 {
                STONE.shadow
            } else {
                STONE.base
            };
            if col > 0 && col < width - 1 && row > 0 && row < height - 1 {
                if chance(px, py, salt, 28) {
                    color = mix(color, STONE.edge, 0.35);
                } else if chance(px, py, salt + 100, 18) {
                    color = mix(color, STONE.shine, 0.5);
                }
            }
            put(s, px, py, color);
        }
    }
    if height > 1 {
        put(s, x + 1, y, STONE.shine);
    }
}

/// A little door in a grassy mound, for somebody very small: an arch of stones, a round window
/// lit within, stepping stones up to the threshold and flowers all over.
fn hill_door() -> Piece {
    let mut s = Canvas::new(48, 43);
    let ground = 38;
    shadow(&mut s, 24, ground, 23, 3);
    // The mound, lit from the upper left, grass blades standing up along its back.
    let (cx, cy, rx, ry) = (24.0, 39.0, 22.0, 26.0);
    let inside = |x: i32, y: i32| {
        let wobble = (noise(x / 3, 0, 71) % 3) as f32 * 0.6;
        let (dx, dy) = (
            (x as f32 + 0.5 - cx) / rx,
            (y as f32 + 0.5 - cy) / (ry + wobble),
        );
        y <= ground && dx * dx + dy * dy <= 1.0
    };
    for y in 0..=ground {
        for x in 0..48 {
            if !inside(x, y) {
                continue;
            }
            let edge = !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1));
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let mut color = if edge {
                TURF.edge
            } else {
                lit(TURF, u, v, noise(x, y, 72))
            };
            if !edge && chance(x, y, 73, 40) {
                color = if chance(x, y, 74, 128) {
                    TURF.shadow
                } else {
                    TURF.light
                };
            }
            if y >= ground - 1 && !edge {
                color = mix(color, TURF.edge, 0.4);
            }
            put(&mut s, x, y, color);
        }
    }
    for x in 4..44 {
        let top = (0..=ground).find(|&y| inside(x, y));
        if let Some(top) = top
            && chance(x, 0, 75, 70)
        {
            put(&mut s, x, top - 1, TURF.edge);
            put(&mut s, x, top, TURF.base);
        }
    }
    // A chimney pot poking out of the turf at the back.
    stone_block(&mut s, 33, 13, 4, 5, 77);
    rect(&mut s, 33, 11, 4, 2, rgb(0xb05445));
    hline(&mut s, 32, 11, 6, rgb(0x612a2b));
    put(&mut s, 33, 12, rgb(0xe08c69));
    for x in 32..38 {
        if chance(x, 18, 78, 160) {
            put(&mut s, x, 18, TURF.base);
        }
    }
    // A round window, lit within.
    let (wx, wy) = (11, 28);
    for (dx, dy) in ring_offsets(3.0) {
        put(&mut s, wx + dx, wy + dy, POST.base);
    }
    for (dx, dy) in ring_offsets(3.6) {
        if s.get(wx + dx, wy + dy) != POST.base {
            put(&mut s, wx + dx, wy + dy, POST.edge);
        }
    }
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx * dx + dy * dy <= 5 {
                put(
                    &mut s,
                    wx + dx,
                    wy + dy,
                    if dx + dy < 0 { GLOW[2] } else { GLOW[1] },
                );
            }
        }
    }
    hline(&mut s, wx - 2, wy, 5, POST.shadow);
    vline(&mut s, wx, wy - 2, 5, POST.shadow);
    // The arch of stones around the door.
    let (door_x, door_w, spring) = (18, 12, 26);
    let half = door_w as f32 / 2.0;
    let mid = door_x as f32 + half;
    for y in spring - 9..=ground {
        for x in door_x - 3..door_x + door_w + 3 {
            let dx = x as f32 + 0.5 - mid;
            let dy = (y as f32 + 0.5 - spring as f32).min(0.0);
            let d = (dx * dx + dy * dy).sqrt();
            if !(half..half + 2.6).contains(&d) {
                continue;
            }
            let angle = dy.atan2(dx);
            let block = ((angle * 4.0).round() as i32, y / 3);
            let joint = noise(block.0, if dy < 0.0 { 0 } else { block.1 }, 79) % 4;
            let face = [STONE.base, STONE.light, STONE.base, STONE.shadow][joint as usize];
            let color = if d > half + 2.0 {
                STONE.edge
            } else if dx < -half {
                mix(face, STONE.light, 0.3)
            } else {
                face
            };
            put(&mut s, x, y, color);
        }
    }
    // The door: oak planks, iron straps and a brass knob.
    for y in spring - 6..ground {
        for x in door_x..door_x + door_w {
            let dx = x as f32 + 0.5 - mid;
            let dy = (y as f32 + 0.5 - spring as f32).min(0.0);
            if dx * dx + dy * dy > half * half {
                continue;
            }
            let plank = (x - door_x) % 3;
            let mut color = match plank {
                0 => OAK_DOOR.edge,
                1 => OAK_DOOR.light,
                _ => OAK_DOOR.base,
            };
            if plank != 0 && chance(x, y / 2, 80, 40) {
                color = OAK_DOOR.shadow;
            }
            if x >= door_x + door_w - 2 {
                color = mix(color, OAK_DOOR.edge, 0.5);
            }
            put(&mut s, x, y, color);
        }
    }
    for strap in [spring - 2, ground - 5] {
        hline(&mut s, door_x, strap, 8, HINGE.base);
        hline(&mut s, door_x, strap + 1, 8, HINGE.edge);
        put(&mut s, door_x + 1, strap, HINGE.shine);
        put(&mut s, door_x + 7, strap, HINGE.light);
    }
    put(&mut s, door_x + 9, spring + 4, BRASS.shine);
    put(&mut s, door_x + 10, spring + 4, BRASS.base);
    put(&mut s, door_x + 9, spring + 5, BRASS.base);
    put(&mut s, door_x + 10, spring + 5, BRASS.edge);
    // The threshold and the stepping stones down to the grass.
    hline(&mut s, door_x - 2, ground, door_w + 4, STONE.light);
    hline(&mut s, door_x - 2, ground + 1, door_w + 4, STONE.edge);
    lump(&mut s, 21.0, 40.4, 3.6, 1.5, STONE, 81);
    lump(&mut s, 28.0, 41.4, 3.0, 1.3, STONE, 82);
    // Flowers on the turf, and clumps either side of the door.
    for y in 12..ground - 1 {
        for x in 3..45 {
            let near_door = (door_x - 4..door_x + door_w + 4).contains(&x) && y > spring - 10;
            let near_window = (x - wx).abs() <= 4 && (y - wy).abs() <= 4;
            if near_door || near_window || !inside(x, y) || !inside(x, y - 1) {
                continue;
            }
            if chance(x, y, 83, 9) {
                let blossom = BLOSSOMS[(noise(x, y, 84) % 5) as usize];
                put(&mut s, x, y, blossom);
                put(&mut s, x + 1, y + 1, TURF.shadow);
            }
        }
    }
    for (x, salt) in [(door_x - 4, 85), (door_x + door_w + 3, 86)] {
        tuft(&mut s, x, ground, 3, salt);
        flowers(&mut s, x, ground - 2, 3, 3, salt);
    }
    Piece {
        sprite: s,
        anchor: (24, ground),
    }
}

/// The pixels about a circle of `radius` round the origin.
fn ring_offsets(radius: f32) -> Vec<(i32, i32)> {
    let reach = radius.ceil() as i32 + 1;
    let mut points = Vec::new();
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if (d - radius).abs() < 0.55 {
                points.push((dx, dy));
            }
        }
    }
    points
}

// ---------------------------------------------------------------------------------------------
// Painting helpers, shared with the hollows
// ---------------------------------------------------------------------------------------------

/// An icon painted from nine rows of codes, as `paint_rows` reads them.
pub(super) fn icon_from(rows: [&str; 9], ramp: Ramp, extras: &[(char, Rgba)]) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    for row in rows {
        debug_assert_eq!(row.chars().count(), ICON as usize, "{row:?}");
    }
    paint_rows(&mut canvas, 0, 0, &rows, ramp, extras);
    canvas
}

/// Paints rows of codes with their top-left at `(x, y)`: `#`, `s`, `o`, `l` and `*` for the main
/// material's edge, shadow, base, light and shine, `extras` for anything else, `.` for clear.
pub(super) fn paint_rows(
    s: &mut Canvas,
    x: i32,
    y: i32,
    rows: &[&str],
    ramp: Ramp,
    extras: &[(char, Rgba)],
) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, code) in row.chars().enumerate() {
            let color = match code {
                '#' => ramp.edge,
                's' => ramp.shadow,
                'o' => ramp.base,
                'l' => ramp.light,
                '*' => ramp.shine,
                other => match extras.iter().find(|(c, _)| *c == other) {
                    Some(&(_, color)) => color,
                    None => continue,
                },
            };
            put(s, x + dx as i32, y + dy as i32, color);
        }
    }
}

/// The soft shade under a piece, where it stands on the grass. Drawn first, onto clear canvas.
pub(super) fn shadow(s: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32) {
    for y in -ry..=ry {
        for x in -rx..=rx {
            if (x * x * ry * ry + y * y * rx * rx) <= rx * rx * ry * ry {
                s.set(cx + x, cy + y, SHADOW);
            }
        }
    }
}

/// A tone from `ramp` for a point `(u, v)` on a rounded surface, each from -1 to 1 across it,
/// lit from the upper left; `grain` jitters it so the surface isn't smooth.
pub(super) fn lit(ramp: Ramp, u: f32, v: f32, grain: u32) -> Rgba {
    let depth = (1.0 - u * u - v * v).max(0.0).sqrt();
    let light = -0.55 * u - 0.62 * v + 0.56 * depth;
    let light = light + ((grain % 64) as f32 / 64.0 - 0.5) * 0.18;
    if light > 0.9 {
        ramp.shine
    } else if light > 0.58 {
        ramp.light
    } else if light > 0.2 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// A rounded lump lit from the upper left and outlined in its own edge tone: a pebble, a cone, a
/// clump of leaves.
pub(super) fn lump(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, ramp: Ramp, salt: u32) {
    tilted_lump(s, (cx, cy), (rx, ry), 0.0, ramp, salt);
}

/// A lump turned `tilt` radians clockwise, still lit from the upper left: a stone set down a
/// little askew, a cone lying on the slant.
pub(super) fn tilted_lump(
    s: &mut Canvas,
    (cx, cy): (f32, f32),
    (rx, ry): (f32, f32),
    tilt: f32,
    ramp: Ramp,
    salt: u32,
) {
    let (sin, cos) = tilt.sin_cos();
    // Where a pixel falls across the lump, as it lies and as the light sees it.
    let at = |x: i32, y: i32| {
        let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
        let (u, v) = ((dx * cos + dy * sin) / rx, (dy * cos - dx * sin) / ry);
        ((u, v), (u * cos - v * sin, u * sin + v * cos))
    };
    let inside = |x: i32, y: i32| {
        let ((u, v), _) = at(x, y);
        u * u + v * v <= 1.0
    };
    let reach = rx.max(ry).ceil() as i32 + 1;
    let (mx, my) = (cx.round() as i32, cy.round() as i32);
    for y in my - reach..=my + reach {
        for x in mx - reach..=mx + reach {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (_, (u, v)) = at(x, y);
            let color = if edge {
                ramp.edge
            } else {
                lit(ramp, u, v, noise(x, y, salt))
            };
            put(s, x, y, color);
        }
    }
}

/// Darkens whatever is already painted within an ellipse: where one thing rests on another.
pub(super) fn tuck(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgba) {
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if u * u + v * v <= 1.0 && s.get(x, y).a == 255 {
                put(s, x, y, color);
            }
        }
    }
}

/// Moss in a patch over whatever is already painted there.
pub(super) fn moss(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, salt: u32) {
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let d = u * u + v * v;
            let thick = (230.0 * (1.0 - d)) as u32 + 40;
            if d > 1.0 || s.get(x, y).a < 255 || !chance(x, y, salt, thick) {
                continue;
            }
            let color = if v < -0.2 && chance(x, y, salt + 1, 140) {
                MOSS.light
            } else if u + v > 0.6 {
                MOSS.shadow
            } else {
                MOSS.base
            };
            put(s, x, y, color);
        }
    }
}

/// A tone across a cylinder lit from the left, `across` from 0 at its left to 1 at its right.
pub(super) fn cylinder(ramp: Ramp, across: f32) -> Rgba {
    if across < 0.12 {
        ramp.light
    } else if across < 0.3 {
        ramp.shine
    } else if across < 0.45 {
        ramp.light
    } else if across < 0.78 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// Pixels along a curve from `from`, pulled towards `via`, to `to`, one step at a time.
pub(super) fn curve(from: (f32, f32), via: (f32, f32), to: (f32, f32)) -> Vec<(i32, i32)> {
    let span = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let steps = ((span(from, via) + span(via, to)) * 2.0).ceil().max(1.0) as usize;
    let mut points: Vec<(i32, i32)> = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * from.0 + 2.0 * u * t * via.0 + t * t * to.0;
        let y = u * u * from.1 + 2.0 * u * t * via.1 + t * t * to.1;
        let point = (x.round() as i32, y.round() as i32);
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

/// A strappy leaf or grass blade from `from` to `to`, two pixels wide for most of its length,
/// lit along its upper side.
pub(super) fn blade(s: &mut Canvas, from: (f32, f32), via: (f32, f32), to: (f32, f32), ramp: Ramp) {
    let points = curve(from, via, to);
    let count = points.len();
    for (index, &(x, y)) in points.iter().enumerate() {
        let t = index as f32 / count as f32;
        if t < 0.75 {
            put(s, x, y + 1, ramp.edge);
        }
        put(s, x, y, if t < 0.4 { ramp.base } else { ramp.light });
    }
    if let Some(&(x, y)) = points.last() {
        put(s, x, y, ramp.shadow);
    }
}

/// A tuft of grass blades about `(x, y)` on the ground, `spread` pixels either side.
pub(super) fn tuft(s: &mut Canvas, x: i32, y: i32, spread: i32, salt: u32) {
    for dx in -spread..=spread {
        if !chance(x + dx, y, salt, 170) {
            continue;
        }
        let tall = 1 + (noise(x + dx, y, salt + 1) % 3) as i32 + (spread - dx.abs()) / 2;
        let lean = (dx.signum() * (noise(dx, y, salt + 2) % 2) as i32).clamp(-1, 1);
        for step in 0..tall {
            let color = if step == tall - 1 {
                STEM.light
            } else if step == 0 {
                STEM.shadow
            } else {
                STEM.base
            };
            let px = x + dx + if step == tall - 1 { lean } else { 0 };
            put(s, px, y - step, color);
        }
    }
}

/// A few flowers on short stems around `(x, y)`.
pub(super) fn flowers(s: &mut Canvas, x: i32, y: i32, spread: i32, count: u32, salt: u32) {
    for index in 0..count {
        let fx = x - spread + (noise(index as i32, x, salt) % (spread * 2 + 1) as u32) as i32;
        let fy = y - (noise(x, index as i32, salt + 1) % 3) as i32;
        let blossom = BLOSSOMS[(noise(fx, fy, salt + 2) % 5) as usize];
        put(s, fx, fy + 1, STEM.shadow);
        put(s, fx, fy, blossom);
        put(s, fx - 1, fy, mix(blossom, rgb(0xffffff), 0.35));
        put(s, fx + 1, fy, mix(blossom, rgb(0x000000), 0.15));
        put(s, fx, fy - 1, mix(blossom, rgb(0xffffff), 0.2));
    }
}
