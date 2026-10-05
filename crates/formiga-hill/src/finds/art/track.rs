//! The finds of the old track: their icons and their Hilltop pieces. What the woodcutter and the
//! carter left behind, and what each becomes: a cracked teacup planted with pansies, the copper
//! kettle hung over a fire ring, a sentry box with the tin soldier in it, a scarecrow in the old
//! straw hat, a tower of the good china, a bell under its own little roof, a signpost, a lantern
//! by a log seat, a compass rose set in the turf, a cuckoo clock on a post; and the two finest, a
//! ship in its bottle on a cradle, and the golden axe struck in a stump.
//!
//! The kettle, the lantern, the carved board and the teacups are drawn by helpers the woodshed and
//! the tea party use too, so what is built from them is the same things.

#[cfg(test)]
use super::ICON;
use super::Piece;
use super::brush::*;
use super::earth::BRASS;
use super::hollow::{BARK, CONE, IRON, glow, stick, toadstool};
use super::undergrowth::{BERRY_DARK, BERRY_RED};
use super::water::{BOTTLE_GREEN, SLATE, sparkle};
use crate::materials::{GLOW, STONE, TRIM};
use crate::paint::{Ramp, chance, ellipse, hline, line, mix, noise, put, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::{PI, TAU};

/// White china, and the blue birds painted round it.
pub(super) const CHINA: Ramp = Ramp::new(0x8a8e9c, 0xc4c8d4, 0xe2e6ee, 0xf2f4f8, 0xffffff);
const WILLOW: Ramp = Ramp::new(0x22357a, 0x2f4a9a, 0x4064b8, 0x6888d0, 0x9cb4e4);
pub(super) const GOLD: Ramp = Ramp::new(0x7a4a08, 0xc07a10, 0xf0a818, 0xffc840, 0xfff4b0);
pub(super) const COPPER: Ramp = Ramp::new(0x5a2814, 0x8a4220, 0xb8642e, 0xda8c4c, 0xf6c08a);
const VERDIGRIS: Rgba = rgb(0x5c9e86);
const TIN: Ramp = Ramp::new(0x464c56, 0x6a727e, 0x929aa6, 0xb8c0ca, 0xe2e8ee);
const REDCOAT: Ramp = Ramp::new(0x5a1616, 0x8a2222, 0xb8342e, 0xd6584a, 0xf08c76);
const NAVY: Ramp = Ramp::new(0x262c48, 0x323a5a, 0x434c72, 0x5a6690, 0x7a86ae);
const SKIN: Rgba = rgb(0xf0c8a0);
const STRAW: Ramp = Ramp::new(0x8a7438, 0xb89a4e, 0xd6ba68, 0xe8d08a, 0xf6e6b4);
const SACKING: Ramp = Ramp::new(0x6a5838, 0x8e7a50, 0xb09a6c, 0xc8b486, 0xe0cea4);
const COAT: Ramp = Ramp::new(0x2a3448, 0x3e4c66, 0x566a86, 0x7488a4, 0x98aac0);
const PATCH: Ramp = Ramp::new(0x5a2a1e, 0x82402a, 0xa85a3a, 0xc47a52, 0xdc9c72);
pub(super) const WOOD: Ramp = Ramp::new(0x4e3626, 0x6c4e36, 0x8c6a4a, 0xa88660, 0xbea078);
pub(super) const WEATHERED: Ramp = Ramp::new(0x3c3832, 0x57524a, 0x746d62, 0x928a7c, 0xb0a898);
pub(super) const SHINGLE: Ramp = Ramp::new(0x4a3424, 0x6c4c34, 0x8c6844, 0xaa865a, 0xc6a476);
const WALNUT: Ramp = Ramp::new(0x2e1a10, 0x4a2c1a, 0x6a4026, 0x8a5a36, 0xaa7a4e);
const BOX_PAINT: Ramp = Ramp::new(0x1e3a36, 0x2a524c, 0x3a6e64, 0x548a7e, 0x76a89a);
const FACE: Ramp = Ramp::new(0x9a8e70, 0xd0c4a4, 0xeee4c8, 0xf8f2e0, 0xffffff);
const SAIL: Ramp = Ramp::new(0x988e76, 0xc8bea2, 0xe8e0c6, 0xf6f0dc, 0xffffff);
const HULL: Ramp = Ramp::new(0x3a2216, 0x52321e, 0x6e4628, 0x8e6038, 0xac7c4e);
const SEA: Ramp = Ramp::new(0x1c3858, 0x2a5280, 0x3c70a6, 0x5c92c6, 0x9cc4e6);
const CORK: Ramp = Ramp::new(0x6a4a2a, 0x8e6a40, 0xb08a58, 0xc8a472, 0xdcc08e);
pub(super) const RINGS: Ramp = Ramp::new(0x7a5a3a, 0xa07a50, 0xc09a68, 0xd6b484, 0xe8cca0);
const SOIL: Ramp = Ramp::new(0x2e2018, 0x46301f, 0x5c402a, 0x74543a, 0x8c6a4c);
const IVY: Ramp = Ramp::new(0x183a22, 0x24522e, 0x346c3c, 0x4a8a4c, 0x6aa866);
const PANSY: Ramp = Ramp::new(0x3a1e5a, 0x56307e, 0x7448a4, 0x9670c4, 0xc0a4e4);
const PANSY_GOLD: Rgba = rgb(0xf0c840);
const ROPE: Ramp = Ramp::new(0x6a5434, 0x8e7448, 0xb0965e, 0xc8b078, 0xdcc894);
const ROBIN: Ramp = Ramp::new(0x4a3424, 0x6a4a32, 0x8a6444, 0xa88058, 0xc49c70);
const BREAST: Rgba = rgb(0xe0703c);
const LEAVES: Ramp = Ramp::new(0x22401f, 0x335a2a, 0x4a7834, 0x649444, 0x88b45c);
/// The embers of a small fire, darkest first.
const EMBER: [Rgba; 3] = [rgb(0xb8401a), rgb(0xf07a2a), rgb(0xffc860)];
const ASH: Rgba = rgb(0x8a847c);
const CHAR: Rgba = rgb(0x3a3028);
/// The dark inside of anything, a warm brown rather than black.
const INSIDE: Rgba = rgb(0x2e2018);
const CANDLE: Rgba = rgb(0xf3e9cf);
const CANDLELIGHT: Rgba = rgb(0xffcf6a);

/// Where the woodcutter's lantern hangs in its piece, and the embers glow in the camp kettle's.
const LANTERN_SEAT_HOOK: (i32, i32) = (27, 9);
const CAMP_FIRE: (i32, i32) = (17, 37);

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    let china = [
        ('b', WILLOW.base),
        ('B', WILLOW.light),
        ('k', rgb(0x6c6878)),
        ('g', GOLD.light),
        ('G', GOLD.base),
    ];
    Some(match id {
        // A teacup with a chip out of its rim and a crack down its side.
        "cracked_teacup" => icon_from(
            [
                ".........",
                "#.#####..",
                "#*.llo##.",
                "#lbkbo#.#",
                "#lokBs#.#",
                ".#ooks##.",
                "..#oss#..",
                "..#####..",
                ".........",
            ],
            CHINA,
            &china,
        ),
        // The copper kettle: a spout green at its tip, and its handle arched over the lid.
        "copper_kettle" => icon_from(
            [
                "...iii...",
                "..i...i..",
                "..##k##..",
                "v.#*lo#..",
                ".##lloo#.",
                ".#llooos#",
                ".#loooss#",
                "..#ssss#.",
                "...####..",
            ],
            COPPER,
            &[('i', IRON.light), ('k', COPPER.shine), ('v', VERDIGRIS)],
        ),
        // The tin soldier, saluting, on his stand.
        "tin_soldier" => icon_from(
            [
                "...hh....",
                "..hHhh...",
                "..hBhhf..",
                "...ffr...",
                "..lowr...",
                "..lwoo...",
                "..#oo#...",
                "...tT....",
                "..gGgg...",
            ],
            REDCOAT,
            &[
                ('h', NAVY.base),
                ('H', NAVY.light),
                ('B', BRASS.light),
                ('f', SKIN),
                ('r', REDCOAT.shadow),
                ('w', TRIM.light),
                ('t', TIN.base),
                ('T', TIN.light),
                ('g', rgb(0x3e6e3a)),
                ('G', rgb(0x5a8e4c)),
            ],
        ),
        // A straw hat, its band round it and a feather in the band.
        "straw_hat" => icon_from(
            [
                ".........",
                "......f..",
                "...###fF.",
                "..#*lo#F.",
                "..#loos#.",
                "..#bbbb#.",
                "#*llloos#",
                ".#ssss##.",
                ".........",
            ],
            STRAW,
            &[
                ('b', REDCOAT.base),
                ('f', rgb(0xeae6da)),
                ('F', rgb(0x9a9488)),
            ],
        ),
        // The good china: gilt round the rim, birds round the cup, and its saucer.
        "china_teacup" => icon_from(
            [
                ".........",
                "#######..",
                "#gGgGg##.",
                "#*bBbo#.#",
                "#lobos#.#",
                ".#ooss##.",
                "..#oo#...",
                "#*llooss#",
                ".#######.",
            ],
            CHINA,
            &china,
        ),
        // A brass bell on its strap, the clapper hanging under.
        "brass_bell" => icon_from(
            [
                "...###...",
                "...#.#...",
                "..##o##..",
                "..#*lo#..",
                "..#loo#..",
                ".#llooo#.",
                "#lloooss#",
                "#########",
                "....c....",
            ],
            BRASS,
            &[('c', IRON.base)],
        ),
        // A board cut to a point, a hand carved on it pointing the way.
        "carved_sign" => icon_from(
            [
                ".........",
                "#######..",
                "#*llllo#.",
                "#lhhhhoo#",
                "#lhhoos#.",
                "#######..",
                "...pP....",
                "...pP....",
                "..gpPg...",
            ],
            WEATHERED,
            &[
                ('h', WEATHERED.edge),
                ('p', WOOD.base),
                ('P', WOOD.edge),
                ('g', LEAVES.light),
            ],
        ),
        // A tin lantern with its hook, the flame inside a guard of wires.
        "woodcutters_lantern" => icon_from(
            [
                "....i....",
                "...i.i...",
                "..#####..",
                "..#*lo#..",
                ".#wGfGw#.",
                ".#wycyw#.",
                ".#######.",
                "..#sss#..",
                "...###...",
            ],
            TIN,
            &[
                ('i', IRON.light),
                ('w', IRON.shadow),
                ('y', GLOW[0]),
                ('G', GLOW[1]),
                ('f', GLOW[2]),
                ('c', CANDLE),
            ],
        ),
        // A pocket compass in its brass case, the needle's red end pointing north.
        "pocket_compass" => icon_from(
            [
                "....#....",
                "...#l#...",
                "..#####..",
                ".#*WWwo#.",
                ".#lWRwo#.",
                ".#lwnws#.",
                ".#owwws#.",
                "..#####..",
                ".........",
            ],
            BRASS,
            &[
                ('w', FACE.base),
                ('W', FACE.shine),
                ('R', REDCOAT.light),
                ('n', NAVY.base),
            ],
        ),
        // A carved cuckoo clock: the bird out of its door, the face, the cones on their chains.
        "cuckoo_clock" => icon_from(
            [
                "....#....",
                "...#*#...",
                "..#lll#..",
                ".#lllos#.",
                "..#oBo#..",
                "..#fff#..",
                "..#fhf#..",
                "..#####..",
                "..c...c..",
            ],
            WALNUT,
            &[
                ('B', ROBIN.light),
                ('f', FACE.base),
                ('h', WALNUT.edge),
                ('c', CONE.base),
            ],
        ),
        // A ship in full sail in a bottle lying on its side, the cork in its neck.
        "ship_in_a_bottle" => icon_from(
            [
                ".........",
                ".#####...",
                "#*.S.S#..",
                "#.SsSs###",
                "#.hhhh.#k",
                "#wWwwwW#k",
                ".#####...",
                ".........",
                ".........",
            ],
            BOTTLE_GREEN,
            &[
                ('S', SAIL.shine),
                ('s', SAIL.base),
                ('h', HULL.base),
                ('w', SEA.base),
                ('W', SEA.light),
                ('k', CORK.light),
            ],
        ),
        // The golden axe, gold from its blade to the end of its handle.
        "golden_axe" => icon_from(
            [
                ".###.....",
                "#*lo##...",
                "#llooHh..",
                "#los##...",
                ".##.Hh...",
                "....Hh...",
                "....Hh...",
                "....Hh...",
                "....HH...",
            ],
            GOLD,
            &[('h', GOLD.light), ('H', GOLD.shadow)],
        ),
        _ => return None,
    })
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "cracked_teacup" => teacup_planter(),
        "copper_kettle" => camp_kettle(),
        "tin_soldier" => sentry_box(),
        "straw_hat" => scarecrow(),
        "china_teacup" => teacup_tower(),
        "brass_bell" => bell_frame(),
        "carved_sign" => signpost(),
        "woodcutters_lantern" => lantern_by_a_log_seat(),
        "pocket_compass" => compass_rose(),
        "cuckoo_clock" => cuckoo_clock_on_a_post(),
        "ship_in_a_bottle" => bottled_ship(),
        "golden_axe" => golden_axe_in_its_stump(),
        _ => return None,
    })
}

/// What shines from one of these pieces after dark, and where in it: the woodcutter's lantern,
/// and the embers under the camp kettle.
pub fn lights(id: &str) -> Vec<(Rgba, (i32, i32))> {
    match id {
        "woodcutters_lantern" => {
            vec![(CANDLELIGHT, (LANTERN_SEAT_HOOK.0, LANTERN_SEAT_HOOK.1 + 10))]
        }
        "copper_kettle" => vec![(rgb(0xffa050), CAMP_FIRE)],
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------------------------

/// Something round standing upright, from `top` to `foot`, as wide at each height as `half` says
/// (from 0 at its top to 1 at its foot): shaded as a cylinder lit from the left, outlined all
/// round in its own darkest tone.
pub(super) fn turned(
    s: &mut Canvas,
    cx: f32,
    (top, foot): (i32, i32),
    ramp: Ramp,
    half: impl Fn(f32) -> f32,
) {
    let width = |y: i32| {
        if y < top || y > foot {
            return -1.0;
        }
        half((y - top) as f32 / (foot - top).max(1) as f32)
    };
    let inside = |x: i32, y: i32| (x as f32 + 0.5 - cx).abs() <= width(y);
    for y in top..=foot {
        let reach = width(y);
        if reach <= 0.0 {
            continue;
        }
        for x in (cx - reach).floor() as i32..=(cx + reach).ceil() as i32 {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let across = ((x as f32 + 0.5 - cx) / reach + 1.0) / 2.0;
            put(
                s,
                x,
                y,
                if edge {
                    ramp.edge
                } else {
                    cylinder(ramp, across)
                },
            );
        }
    }
}

/// A saucer about `(cx, cy)`, `rx` either side: china seen a little from above, its rim lit along
/// the back and its well in shade, the thickness of it showing under its front; gilt round the rim
/// if it is the good china.
pub(super) fn saucer(s: &mut Canvas, (cx, cy): (i32, i32), rx: i32, gilt: bool) {
    let ry = (rx as f32 / 3.6).max(1.6);
    let rx = rx as f32 + 0.4;
    let top = |x: i32, y: i32| {
        let (u, v) = (
            (x as f32 + 0.5 - cx as f32 - 0.5) / rx,
            (y as f32 - cy as f32) / ry,
        );
        u * u + v * v
    };
    let solid = |x: i32, y: i32| top(x, y) <= 1.0 || top(x, y - 1) <= 1.0;
    for y in cy - ry.ceil() as i32 - 1..=cy + ry.ceil() as i32 + 2 {
        for x in cx - rx.ceil() as i32 - 1..=cx + rx.ceil() as i32 + 1 {
            if !solid(x, y) {
                continue;
            }
            let rim = !(solid(x - 1, y) && solid(x + 1, y) && solid(x, y - 1) && solid(x, y + 1));
            let d = top(x, y);
            let back = y < cy;
            let color = if rim {
                CHINA.edge
            } else if d > 1.0 {
                CHINA.shadow
            } else if d > 0.62 {
                match (gilt, back) {
                    (true, _) if d > 0.8 => {
                        if x < cx {
                            GOLD.light
                        } else {
                            GOLD.base
                        }
                    }
                    (_, true) => CHINA.shine,
                    _ => CHINA.light,
                }
            } else if d > 0.28 {
                CHINA.base
            } else {
                CHINA.shadow
            };
            put(s, x, y, color);
        }
    }
}

/// A teacup standing with its foot at `(cx, foot)`, `rim` pixels either side across its mouth and
/// `tall` high: white china lit from the left, blue birds flying round it under its rim, its
/// handle on the right. The good china has gilt round its rim; the other a chip out of the rim
/// and a crack down its side. `inside` is what shows in its mouth.
pub(super) fn teacup(
    s: &mut Canvas,
    (cx, foot): (i32, i32),
    (rim, tall): (i32, i32),
    good: bool,
    inside: Rgba,
) {
    let top = foot - tall;
    let centre = cx as f32 + 0.5;
    // The handle, a loop off its right side, drawn first so the cup covers where it joins.
    let (hx, h0, h1) = (cx + rim, top + 2, top + 2 + (tall * 3 / 5).max(3));
    for y in h0..=h1 {
        let out = if y == h0 || y == h1 { 1 } else { 2 };
        put(s, hx + out, y, CHINA.edge);
        if y == h0 {
            put(s, hx + 1, y + 1, CHINA.light);
        }
    }
    put(s, hx, h0, CHINA.edge);
    put(s, hx, h1, CHINA.edge);
    // The cup, wide at the mouth and rounding in to its foot.
    let r = rim as f32 + 0.5;
    turned(s, centre, (top + 1, foot - 1), CHINA, |t| {
        r * (1.0 - 0.34 * t * t)
    });
    let foot_half = (rim * 3 / 5).max(1);
    hline(s, cx - foot_half, foot, foot_half * 2 + 1, CHINA.edge);
    hline(
        s,
        cx - foot_half + 1,
        foot - 1,
        foot_half * 2 - 1,
        CHINA.shadow,
    );
    // Blue birds round it, a band under the rim, where the china is.
    let band = top + 3;
    let china = |s: &Canvas, x: i32, y: i32| {
        let pixel = s.get(x, y);
        pixel.a == 255 && pixel != CHINA.edge
    };
    for x in cx - rim + 1..cx + rim {
        if china(s, x, band + 2) && (x - cx).rem_euclid(4) != 2 {
            put(s, x, band + 2, WILLOW.light);
        }
    }
    for x in (cx - rim + 2..cx + rim - 1).step_by(4) {
        for (dx, dy, color) in [
            (0, 0, WILLOW.base),
            (1, 1, WILLOW.shadow),
            (2, 0, WILLOW.base),
        ] {
            if china(s, x + dx, band + dy) {
                put(s, x + dx, band + dy, color);
            }
        }
    }
    // The mouth: the far wall in shade over what's in it, the near rim lit.
    let mouth = |x: i32, y: i32| {
        let (u, v) = (
            (x as f32 + 0.5 - centre) / (r - 0.3),
            (y - top - 1) as f32 / 1.4,
        );
        u * u + v * v <= 1.0
    };
    for y in top - 1..=top + 2 {
        for x in cx - rim - 1..=cx + rim + 1 {
            if !mouth(x, y) {
                continue;
            }
            let rim_pixel =
                !(mouth(x - 1, y) && mouth(x + 1, y) && mouth(x, y - 1) && mouth(x, y + 1));
            let color = if rim_pixel && y <= top {
                CHINA.edge
            } else if rim_pixel {
                if good {
                    if x < cx { GOLD.light } else { GOLD.base }
                } else if x < cx {
                    CHINA.shine
                } else {
                    CHINA.light
                }
            } else if y <= top {
                CHINA.shadow
            } else {
                inside
            };
            put(s, x, y, color);
        }
    }
    if !good {
        // A chip out of the rim on the left, and a crack running down from it.
        put(s, cx - rim + 1, top, CHINA.shadow);
        put(s, cx - rim + 1, top + 1, CHINA.edge);
        put(s, cx - rim + 2, top + 1, CHINA.shadow);
        let crack = rgb(0x6c6878);
        let mut x = cx - rim / 3;
        for y in top + 2..foot - 1 {
            put(s, x, y, crack);
            if (y - top) % 3 == 0 {
                x += if (y / 3) % 2 == 0 { 1 } else { -1 };
            }
        }
    }
}

/// The copper kettle with its foot at `(cx, foot)`: a squat round body lit from the upper left,
/// its spout reaching out to the left and green at the tip, a lid with a knob, and its iron
/// handle arched over the top. Thirteen pixels across and fourteen high.
pub(super) fn kettle(s: &mut Canvas, (cx, foot): (i32, i32)) {
    let (x, y) = (cx as f32 + 0.5, foot as f32);
    // The spout first, reaching up and out to the left from low on the body.
    stick(
        s,
        (x - 3.5, y - 3.0),
        (x - 8.6, y - 7.6),
        (2.6, 1.6),
        COPPER,
    );
    put(s, cx - 9, foot - 8, VERDIGRIS);
    put(s, cx - 8, foot - 8, VERDIGRIS);
    put(s, cx - 9, foot - 7, mix(VERDIGRIS, COPPER.edge, 0.5));
    // The body, a little flattened, and its foot.
    lump(s, x, y - 4.6, 6.4, 4.8, COPPER, 4101);
    hline(s, cx - 4, foot, 9, COPPER.edge);
    // A dent, and verdigris creeping round from the spout.
    put(s, cx + 2, foot - 4, COPPER.shadow);
    put(s, cx + 3, foot - 3, COPPER.light);
    put(s, cx - 5, foot - 3, VERDIGRIS);
    // The lid and its knob.
    ellipse(s, cx, foot - 9, 3, 1, COPPER.edge);
    hline(s, cx - 2, foot - 9, 5, COPPER.light);
    put(s, cx - 1, foot - 10, COPPER.shine);
    put(s, cx, foot - 11, COPPER.edge);
    put(s, cx, foot - 10, COPPER.light);
    // The handle, an iron hoop over the lid.
    for step in 0..=10 {
        let t = step as f32 / 10.0;
        let (hx, hy) = (x - 5.0 + t * 10.0, y - 8.5 - (t * PI).sin() * 5.0);
        put(
            s,
            hx as i32,
            hy as i32,
            if t < 0.5 { IRON.light } else { IRON.base },
        );
        put(s, hx as i32, hy as i32 + 1, IRON.edge);
    }
}

/// The woodcutter's lantern hung by its hook from `(x, y)`: a tin cap and base, glass all round
/// a candle burning inside a guard of wires; nine pixels across and fifteen down from the hook.
pub(super) fn tin_lantern(s: &mut Canvas, (x, y): (i32, i32)) {
    glow(s, x, y + 9, 7.5, rgba(0xffe7a0, 36));
    glow(s, x, y + 9, 4.5, rgba(0xffe7a0, 50));
    paint_rows(
        s,
        x - 4,
        y,
        &[
            "....i....",
            "...i.i...",
            "....i....",
            "...###...",
            "..#*lo#..",
            ".#lloos#.",
            ".#######.",
            ".#wGfGw#.",
            ".#wGFGw#.",
            ".#wwwww#.",
            ".#wycyw#.",
            ".#wycyw#.",
            ".#######.",
            "..#sos#..",
            "...###...",
        ],
        TIN,
        &[
            ('i', IRON.light),
            ('w', IRON.shadow),
            ('y', GLOW[0]),
            ('G', GLOW[1]),
            ('f', GLOW[2]),
            ('F', rgb(0xfffbe8)),
            ('c', CANDLE),
        ],
    );
}

/// The carved sign as a board on a signpost or over a door: weathered wood with its grain in
/// streaks, its end cut to a point, a hand carved on it pointing the way and letters worn too
/// smooth to read. `at` is its top-left, `length` long and seven high; `left` points it the
/// other way.
pub(super) fn carved_board(s: &mut Canvas, at: (i32, i32), length: i32, left: bool, salt: u32) {
    let (x0, y0) = at;
    let height: i32 = 7;
    for row in 0..height {
        // The pointed end narrows over its last three pixels.
        let taper = (row - height / 2).abs();
        let reach = length - 3 + (3 - taper).max(0);
        for col in 0..reach {
            let x = if left {
                x0 + length - 1 - col
            } else {
                x0 + col
            };
            let y = y0 + row;
            let end = col == reach - 1;
            let mut color = if row == 0 || row == height - 1 || col == 0 || end {
                WEATHERED.edge
            } else if row == 1 {
                WEATHERED.light
            } else if row == height - 2 {
                WEATHERED.shadow
            } else {
                WEATHERED.base
            };
            if row > 1
                && row < height - 2
                && !end
                && noise((col + 1).div_euclid(4), row, salt).is_multiple_of(4)
            {
                color = WEATHERED.shadow;
            }
            put(s, x, y, color);
        }
    }
    // The hand, carved near the point: a cuff, a fist, and a finger pointing on.
    let hand: [&str; 3] = ["#.###.", "######", "#.##.."];
    let start = length - 12;
    for (row, line) in hand.iter().enumerate() {
        for (col, code) in line.bytes().enumerate() {
            if code != b'#' {
                continue;
            }
            let col = start + col as i32;
            let x = if left {
                x0 + length - 1 - col
            } else {
                x0 + col
            };
            put(s, x, y0 + 2 + row as i32, WEATHERED.edge);
            put(s, x, y0 + 3 + row as i32, WEATHERED.light);
        }
    }
    // Letters worn almost away.
    for col in (2..start - 1).step_by(2) {
        let x = if left {
            x0 + length - 1 - col
        } else {
            x0 + col
        };
        if !noise(col, 0, salt + 1).is_multiple_of(3) {
            put(s, x, y0 + 3, WEATHERED.shadow);
        }
        if noise(col, 1, salt + 1).is_multiple_of(2) {
            put(s, x, y0 + 4, WEATHERED.shadow);
        }
    }
}

/// The cut end of a log at `(cx, cy)`, `size` across: its rings, darker towards the heart, a
/// crack from the heart out, and the bark round it.
pub(super) fn log_end(s: &mut Canvas, (cx, cy): (f32, f32), size: (f32, f32), salt: u32) {
    let (rx, ry) = size;
    for y in (cy - ry).floor() as i32 - 1..=(cy + ry).ceil() as i32 + 1 {
        for x in (cx - rx).floor() as i32 - 1..=(cx + rx).ceil() as i32 + 1 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let d = (u * u + v * v).sqrt();
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.82 {
                if u + v < 0.0 { BARK.base } else { BARK.edge }
            } else {
                let ring = (d * 4.0 + (noise(x, y, salt) % 64) as f32 / 160.0) as i32;
                match ring {
                    0 => RINGS.shadow,
                    1 => RINGS.light,
                    2 => RINGS.base,
                    _ => RINGS.light,
                }
            };
            put(s, x, y, color);
        }
    }
    line(
        s,
        (cx as i32, cy as i32),
        ((cx + rx * 0.6) as i32, (cy - ry * 0.5) as i32),
        RINGS.edge,
    );
}

/// A pansy at `(x, y)`: purple petals, palest at the top, a gold eye.
fn pansy(s: &mut Canvas, (x, y): (i32, i32)) {
    put(s, x - 1, y - 1, PANSY.light);
    put(s, x, y - 1, PANSY.shine);
    put(s, x + 1, y - 1, PANSY.base);
    put(s, x - 1, y, PANSY.base);
    put(s, x + 1, y, PANSY.shadow);
    put(s, x, y + 1, PANSY.shadow);
    put(s, x, y, PANSY_GOLD);
}

/// An ivy leaf at `(x, y)`: three lobes, lit on the left.
fn ivy_leaf(s: &mut Canvas, (x, y): (i32, i32)) {
    put(s, x, y - 1, IVY.light);
    put(s, x - 1, y, IVY.light);
    put(s, x, y, IVY.base);
    put(s, x + 1, y, IVY.shadow);
    put(s, x, y + 1, IVY.edge);
}

/// A stone set in the grass, lit from the upper left.
fn stone(s: &mut Canvas, (x, y): (f32, f32), (rx, ry): (f32, f32), salt: u32) {
    tuck(
        s,
        x + 0.6,
        y + ry * 0.7,
        rx + 0.8,
        ry * 0.6,
        rgba(0x2a3a2a, 60),
    );
    lump(s, x, y, rx, ry, STONE, salt);
}

/// A robin, perched with its feet at `(x, y)`, facing left.
fn robin(s: &mut Canvas, (x, y): (i32, i32)) {
    paint_rows(
        s,
        x - 3,
        y - 5,
        &[
            "..##..", ".#lo#.", "y#*rs#", ".#rrso#", "..#ss##", "...k.k.",
        ],
        ROBIN,
        &[('r', BREAST), ('y', rgb(0x6a4a22)), ('k', rgb(0x5a3a2a))],
    );
}

// ---------------------------------------------------------------------------------------------
// The commoner pieces
// ---------------------------------------------------------------------------------------------

/// The cracked teacup on its saucer, planted: soil to the rim, pansies come up in it, and ivy
/// trailing over the chipped side down to the saucer.
fn teacup_planter() -> Piece {
    let mut s = Canvas::new(28, 28);
    let ground = 25;
    shadow(&mut s, 14, ground, 12, 2);
    saucer(&mut s, (13, ground - 2), 11, false);
    teacup(&mut s, (13, ground - 3), (7, 10), false, SOIL.base);
    // Leaves round the rim, and the pansies standing up out of them.
    let top = ground - 13;
    for (x, y) in [
        (8, top),
        (10, top - 1),
        (13, top - 1),
        (16, top),
        (18, top - 1),
    ] {
        ivy_leaf(&mut s, (x, y));
    }
    for (x, from, to) in [(10, top, top - 5), (14, top, top - 7), (17, top, top - 4)] {
        vline(&mut s, x, to + 1, from - to, LEAVES.shadow);
        pansy(&mut s, (x, to));
    }
    // Ivy trailing down over the chip, to the saucer.
    let trail = curve(
        (6.5, top as f32 + 1.0),
        (3.0, top as f32 + 6.0),
        (5.0, ground as f32 - 3.0),
    );
    for (index, &(x, y)) in trail.iter().enumerate() {
        put(&mut s, x, y, IVY.shadow);
        if index % 4 == 1 {
            ivy_leaf(&mut s, (x - 1, y));
        }
    }
    tuft(&mut s, 24, ground, 2, 4102);
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// The copper kettle hung from a tripod of poles over a ring of stones, a little fire of sticks
/// burning down to embers under it and steam curling from its spout.
fn camp_kettle() -> Piece {
    let mut s = Canvas::new(34, 42);
    let ground = 39;
    shadow(&mut s, 17, ground, 14, 2);
    let (fx, fy) = CAMP_FIRE;
    // The back leg of the tripod, and the stones at the back of the ring.
    stick(&mut s, (20.5, 35.0), (17.5, 5.0), (2.2, 1.4), BARK);
    let stones: Vec<(f32, f32)> = (0..9)
        .map(|index| {
            let angle = index as f32 / 9.0 * TAU + 0.2;
            (
                fx as f32 + angle.cos() * 10.0,
                fy as f32 + angle.sin() * 3.0,
            )
        })
        .collect();
    for (index, &(x, y)) in stones.iter().enumerate() {
        if y < fy as f32 {
            stone(&mut s, (x, y), (2.2, 1.5), 4110 + index as u32);
        }
    }
    // Ash, sticks burnt through, and the embers glowing among them.
    ellipse(&mut s, fx, fy, 7, 2, ASH);
    ellipse(&mut s, fx, fy, 5, 1, mix(ASH, CHAR, 0.4));
    line(&mut s, (fx - 6, fy + 1), (fx + 3, fy - 2), CHAR);
    line(&mut s, (fx - 2, fy - 2), (fx + 6, fy + 1), WOOD.edge);
    line(&mut s, (fx - 4, fy - 1), (fx + 4, fy), CHAR);
    for (dx, dy, glow_at) in [
        (-2, -1, 2),
        (0, 0, 1),
        (2, -1, 2),
        (-1, 0, 0),
        (1, 1, 1),
        (3, 0, 0),
    ] {
        put(&mut s, fx + dx, fy + dy, EMBER[glow_at]);
    }
    glow(&mut s, fx, fy - 2, 4.0, rgba(0xffb060, 40));
    // A little flame licking up, and smoke.
    put(&mut s, fx, fy - 2, EMBER[1]);
    put(&mut s, fx, fy - 3, EMBER[2]);
    put(&mut s, fx + 1, fy - 2, EMBER[2]);
    // The front of the ring.
    for (index, &(x, y)) in stones.iter().enumerate() {
        if y >= fy as f32 {
            stone(&mut s, (x, y), (2.4, 1.6), 4120 + index as u32);
        }
    }
    // The two front legs, splayed, lashed where they cross at the top.
    stick(&mut s, (5.5, ground as f32), (16.5, 4.0), (2.6, 1.6), WOOD);
    stick(&mut s, (29.5, ground as f32), (18.5, 4.0), (2.6, 1.6), WOOD);
    for (x, y) in [(16, 6), (17, 7), (18, 6), (17, 5), (16, 8), (18, 8)] {
        put(
            &mut s,
            x,
            y,
            if (x + y) % 2 == 0 {
                ROPE.light
            } else {
                ROPE.shadow
            },
        );
    }
    // The chain down to the kettle's handle, and the kettle.
    for y in 9..17 {
        put(
            &mut s,
            17,
            y,
            if y % 2 == 0 { IRON.light } else { IRON.edge },
        );
    }
    kettle(&mut s, (17, 31));
    // Steam curling up from the spout.
    for (x, y, alpha) in [
        (7, 21, 130),
        (6, 20, 110),
        (6, 19, 90),
        (7, 18, 70),
        (7, 17, 50),
        (6, 16, 34),
    ] {
        put(&mut s, x, y, rgba(0xffffff, alpha));
    }
    tuft(&mut s, 4, ground, 2, 4103);
    tuft(&mut s, 30, ground, 2, 4104);
    Piece {
        sprite: s,
        anchor: (17, ground),
    }
}

/// The tin soldier in his sentry box: a narrow box painted green with white trim, a pitched roof
/// with a brass ball on top, standing on a stone step, and him inside it at attention, saluting.
fn sentry_box() -> Piece {
    let mut s = Canvas::new(22, 40);
    let ground = 38;
    shadow(&mut s, 11, ground, 9, 2);
    // The step it stands on.
    for y in ground - 3..=ground {
        for x in 2..20 {
            let rim = y == ground - 3 || y == ground || x == 2 || x == 19;
            let color = if rim {
                STONE.edge
            } else if y == ground - 2 {
                STONE.light
            } else {
                STONE.base
            };
            put(&mut s, x, y, color);
        }
    }
    // The box: its back wall dark inside, its sides and lintel painted, trimmed in white.
    let (left, right, top, floor) = (3, 18, 13, ground - 3);
    for y in top..floor {
        for x in left..=right {
            let pillar = x <= left + 2 || x >= right - 2;
            let lintel = y <= top + 2;
            let color = if x == left || x == right {
                BOX_PAINT.edge
            } else if pillar || lintel {
                if x == left + 1 || y == top {
                    BOX_PAINT.light
                } else if x == right - 1 {
                    BOX_PAINT.shadow
                } else {
                    BOX_PAINT.base
                }
            } else if y == top + 3 {
                rgb(0x1e1610)
            } else {
                INSIDE
            };
            put(&mut s, x, y, color);
        }
    }
    // White trim round the doorway.
    for y in top + 3..floor {
        put(&mut s, left + 3, y, TRIM.light);
        put(&mut s, right - 3, y, TRIM.base);
    }
    hline(&mut s, left + 3, top + 3, right - left - 5, TRIM.light);
    // The roof, pitched, overhanging, a ball on its point.
    for row in 0..9 {
        let y = top - row;
        let half = 10 - row;
        for x in 11 - half..=10 + half {
            let edge = x == 11 - half || x == 10 + half || row == 0;
            let color = if edge {
                REDCOAT.edge
            } else if x < 11 {
                if row % 3 == 1 {
                    REDCOAT.base
                } else {
                    REDCOAT.light
                }
            } else if row % 3 == 1 {
                REDCOAT.shadow
            } else {
                REDCOAT.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 1, top, 20, REDCOAT.edge);
    lump(&mut s, 11.0, top as f32 - 9.5, 1.6, 1.6, BRASS, 4105);
    // The soldier, standing in the doorway.
    paint_rows(
        &mut s,
        7,
        top + 4,
        &[
            "..##....", ".#HH#...", ".#BH#...", ".#hh#f..", "..ff.r..", ".#lowr..", ".#lwoo#.",
            ".#wloo#.", ".#oooo#.", "..#oo#..", "..tTtT..", "..tTtT..", "..nn.n..", ".gGGGGg.",
        ],
        REDCOAT,
        &[
            ('h', NAVY.base),
            ('H', NAVY.light),
            ('B', BRASS.light),
            ('f', SKIN),
            ('r', REDCOAT.shadow),
            ('w', TRIM.light),
            ('t', TIN.base),
            ('T', TIN.light),
            ('n', NAVY.edge),
            ('g', rgb(0x3e6e3a)),
            ('G', rgb(0x5a8e4c)),
        ],
    );
    tuft(&mut s, 20, ground, 1, 4106);
    Piece {
        sprite: s,
        anchor: (11, ground),
    }
}

/// A scarecrow in the old straw hat: a sacking head with a stitched smile, an old coat on the
/// cross-pole, patched, with straw poking out at the cuffs and the hem, and a robin on his arm.
fn scarecrow() -> Piece {
    let mut s = Canvas::new(34, 50);
    let ground = 48;
    shadow(&mut s, 17, ground, 8, 2);
    // The pole and the cross-pole.
    stick(
        &mut s,
        (17.0, ground as f32 + 0.5),
        (17.0, 14.0),
        (2.8, 2.4),
        WOOD,
    );
    stick(&mut s, (3.0, 22.5), (31.0, 22.0), (2.4, 2.2), WOOD);
    // Straw out of the cuffs and from under the hem, drawn first so the coat covers its roots.
    for (x0, y0, dir) in [(6, 23, -1), (28, 23, 1)] {
        for (index, (dx, dy)) in [(2, -2), (3, 0), (3, 2), (2, 3), (1, 4)].iter().enumerate() {
            let tip = (x0 + dir * dx, y0 + dy);
            line(
                &mut s,
                (x0, y0),
                tip,
                if index % 2 == 0 {
                    STRAW.light
                } else {
                    STRAW.base
                },
            );
        }
    }
    for (index, x) in (11..24).step_by(2).enumerate() {
        let long = 3 + (index as i32 % 3);
        line(
            &mut s,
            (x, 36),
            (x + (index as i32 % 3) - 1, 36 + long),
            if index % 2 == 0 {
                STRAW.light
            } else {
                STRAW.base
            },
        );
    }
    // The coat: shoulders hung on the cross-pole, sleeves along it, its body down to the hem.
    let coat = |x: i32, y: i32| {
        let sleeve = (6..=28).contains(&x) && (20..=25).contains(&y);
        let body = {
            let flare = (y - 20) as f32 * 0.12;
            (19..=37).contains(&y) && (x as f32) >= 10.5 - flare && (x as f32) <= 23.5 + flare
        };
        sleeve || body
    };
    model(&mut s, COAT, UPRIGHT, 40, 4107, coat);
    // Its opening down the front, buttons, and patches.
    for y in 22..37 {
        put(&mut s, 17, y, COAT.edge);
    }
    for y in [25, 29, 33] {
        put(&mut s, 18, y, BRASS.light);
    }
    for (x0, y0, w, h) in [(8, 21, 3, 3), (20, 30, 3, 4)] {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let rim = x == x0 || y == y0 || x == x0 + w - 1 || y == y0 + h - 1;
                put(&mut s, x, y, if rim { PATCH.shadow } else { PATCH.light });
            }
        }
        put(&mut s, x0 + 1, y0 + h - 1, rgb(0xe8dcc0));
    }
    // A tuft of straw at the neck, and the sacking head on it.
    for (dx, dy) in [(-2, 0), (-1, 1), (1, 1), (2, 0), (0, 1)] {
        put(&mut s, 17 + dx, 19 + dy, STRAW.light);
    }
    lump(&mut s, 17.0, 14.0, 5.2, 4.8, SACKING, 4108);
    for (x, y) in [(15, 11), (19, 12), (16, 16), (14, 14)] {
        put(&mut s, x, y, SACKING.shadow);
    }
    // Button eyes and a stitched smile.
    for x in [15, 19] {
        put(&mut s, x, 13, WALNUT.edge);
        put(&mut s, x, 12, WALNUT.light);
    }
    for (x, y) in [
        (14, 15),
        (15, 16),
        (16, 16),
        (17, 17),
        (18, 16),
        (19, 16),
        (20, 15),
    ] {
        put(
            &mut s,
            x,
            y,
            if x % 2 == 0 {
                WALNUT.base
            } else {
                SACKING.shadow
            },
        );
    }
    // The hat: the brim wide over the head, the crown, the band and the feather.
    for y in 7..11 {
        for x in 5..30 {
            let (u, v) = ((x as f32 + 0.5 - 17.5) / 12.4, (y as f32 + 0.5 - 9.0) / 1.9);
            let d = u * u + v * v;
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.78 {
                STRAW.edge
            } else if v < 0.0 {
                if u < 0.0 { STRAW.light } else { STRAW.base }
            } else {
                STRAW.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    turned(&mut s, 17.5, (2, 8), STRAW, |t| 4.4 + t * 1.6);
    for x in 12..23 {
        put(
            &mut s,
            x,
            7,
            if x < 17 { REDCOAT.light } else { REDCOAT.base },
        );
    }
    line(&mut s, (22, 6), (26, 0), rgb(0xeae6da));
    line(&mut s, (23, 6), (27, 1), rgb(0x9a9488));
    // Straw-weave in the crown.
    for (x, y) in [(15, 4), (18, 3), (16, 6), (20, 5)] {
        put(&mut s, x, y, STRAW.shadow);
    }
    robin(&mut s, (29, 21));
    tuft(&mut s, 17, ground, 4, 4109);
    flowers(&mut s, 23, ground - 1, 3, 2, 4111);
    Piece {
        sprite: s,
        anchor: (17, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// The less common pieces
// ---------------------------------------------------------------------------------------------

/// The good china built into a tower: saucer, cup, saucer, cup, each balanced a little off the
/// one below, gilt-rimmed with blue birds on, and tea steaming in the cup at the top.
fn teacup_tower() -> Piece {
    let mut s = Canvas::new(26, 44);
    let ground = 42;
    shadow(&mut s, 13, ground, 10, 2);
    saucer(&mut s, (13, ground - 2), 10, true);
    teacup(&mut s, (13, ground - 3), (7, 8), true, CHINA.shadow);
    saucer(&mut s, (14, ground - 12), 8, true);
    teacup(&mut s, (14, ground - 13), (6, 7), true, CHINA.shadow);
    saucer(&mut s, (12, ground - 21), 7, true);
    let tea = rgb(0x9a6a3a);
    teacup(&mut s, (12, ground - 22), (5, 6), true, tea);
    // Steam off the tea.
    for (x, y, alpha) in [
        (11, 12, 120),
        (12, 11, 100),
        (11, 10, 80),
        (12, 9, 60),
        (14, 12, 90),
        (15, 11, 60),
    ] {
        put(&mut s, x, y, rgba(0xffffff, alpha));
    }
    flowers(&mut s, 4, ground, 2, 2, 4112);
    tuft(&mut s, 22, ground, 2, 4113);
    Piece {
        sprite: s,
        anchor: (13, ground),
    }
}

/// The brass bell hung in a frame of two posts and a beam, under a little shingled roof of its
/// own, braced at the corners, with a rope to ring it by.
fn bell_frame() -> Piece {
    let mut s = Canvas::new(32, 46);
    let ground = 44;
    shadow(&mut s, 16, ground, 13, 2);
    // The posts, and the braces from them up to the beam.
    stick(
        &mut s,
        (6.0, ground as f32 + 0.5),
        (6.0, 13.0),
        (3.6, 3.2),
        WOOD,
    );
    stick(
        &mut s,
        (26.0, ground as f32 + 0.5),
        (26.0, 13.0),
        (3.6, 3.2),
        WOOD,
    );
    stick(&mut s, (7.5, 22.0), (12.5, 15.5), (1.8, 1.8), WOOD);
    stick(&mut s, (24.5, 22.0), (19.5, 15.5), (1.8, 1.8), WOOD);
    // The beam.
    for y in 13..16 {
        for x in 2..30 {
            let color = if y == 13 || x == 2 || x == 29 {
                WOOD.edge
            } else if y == 14 {
                WOOD.light
            } else {
                WOOD.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 2, 16, 28, WOOD.edge);
    // The roof over it, shingled in courses.
    for row in 0..10 {
        let y = 12 - row;
        let half = 15 - row * 3 / 2;
        for x in 16 - half..16 + half {
            let edge = x == 16 - half || x == 16 + half - 1 || row == 0;
            let course = (row + i32::from((x / 3) % 2 == 0)) % 2 == 0;
            let color = if edge {
                SHINGLE.edge
            } else if x < 16 {
                if course { SHINGLE.light } else { SHINGLE.base }
            } else if course {
                SHINGLE.base
            } else {
                SHINGLE.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    // The bell on its strap, flaring to its lip, the clapper showing under it.
    vline(&mut s, 16, 17, 2, IRON.base);
    turned(&mut s, 16.0, (18, 29), BRASS, |t| {
        2.4 + 3.6 * t.powf(1.6) + if t > 0.9 { 1.0 } else { 0.0 }
    });
    hline(&mut s, 11, 29, 10, BRASS.edge);
    put(&mut s, 14, 21, BRASS.shine);
    lump(&mut s, 16.0, 30.6, 1.4, 1.2, IRON, 4114);
    // The rope hanging from the clapper, knotted at the end.
    for y in 32..39 {
        put(
            &mut s,
            16,
            y,
            if y % 2 == 0 { ROPE.light } else { ROPE.base },
        );
        put(&mut s, 17, y, ROPE.edge);
    }
    lump(&mut s, 16.5, 39.5, 1.4, 1.2, ROPE, 4115);
    tuft(&mut s, 6, ground, 3, 4116);
    tuft(&mut s, 26, ground, 3, 4117);
    Piece {
        sprite: s,
        anchor: (16, ground),
    }
}

/// A signpost of weathered grey wood: the carved sign on top pointing on, two more arms worn
/// blank, moss on its cap and ivy up its post, and a stone at its foot.
fn signpost() -> Piece {
    let mut s = Canvas::new(36, 50);
    let ground = 48;
    shadow(&mut s, 18, ground, 7, 2);
    // The post, and its cap.
    stick(
        &mut s,
        (17.5, ground as f32 + 0.5),
        (17.5, 5.0),
        (4.2, 3.8),
        WEATHERED,
    );
    for (row, half) in [(2, 1), (3, 2), (4, 3)] {
        hline(
            &mut s,
            18 - half,
            row,
            half * 2,
            if row == 2 {
                WEATHERED.edge
            } else {
                WEATHERED.light
            },
        );
    }
    hline(&mut s, 14, 5, 8, WEATHERED.edge);
    moss(&mut s, 17.0, 4.0, 3.5, 1.6, 4118);
    // The arms: the carved sign pointing right, a blank one left, and a lower one right again
    // with a corner gone.
    carved_board(&mut s, (19, 8), 16, false, 4119);
    blank_arm(&mut s, (2, 17), 15, true, 4120);
    blank_arm(&mut s, (19, 26), 13, false, 4121);
    moss(&mut s, 25.0, 8.5, 4.0, 1.2, 4122);
    // Ivy climbing the post.
    let climb = curve((16.0, ground as f32), (20.0, 40.0), (16.0, 31.0));
    for (index, &(x, y)) in climb.iter().enumerate() {
        put(&mut s, x, y, IVY.shadow);
        if index % 3 == 0 {
            ivy_leaf(&mut s, (x + if index % 2 == 0 { -1 } else { 1 }, y));
        }
    }
    stone(&mut s, (24.0, ground as f32 - 1.5), (3.6, 2.4), 4123);
    tuft(&mut s, 12, ground, 3, 4124);
    tuft(&mut s, 21, ground, 2, 4125);
    Piece {
        sprite: s,
        anchor: (18, ground),
    }
}

/// An arm of a signpost worn blank: a board cut to a point, its grain in streaks.
fn blank_arm(s: &mut Canvas, at: (i32, i32), length: i32, left: bool, salt: u32) {
    let height: i32 = 6;
    for row in 0..height {
        let taper = (row * 2 - (height - 1)).abs() / 2;
        let reach = length - 2 + (2 - taper).max(0);
        for col in 0..reach {
            let x = if left {
                at.0 + length - 1 - col
            } else {
                at.0 + col
            };
            let end = col == reach - 1;
            let mut color = if row == 0 || row == height - 1 || col == 0 || end {
                WEATHERED.edge
            } else if row == 1 {
                WEATHERED.light
            } else {
                WEATHERED.base
            };
            if row > 1 && !end && noise((col + 2).div_euclid(3), row, salt).is_multiple_of(3) {
                color = WEATHERED.shadow;
            }
            put(s, x, at.1 + row, color);
        }
    }
}

/// The woodcutter's lantern hung from a crooked stick pushed into the ground by a log seat: a
/// log lying in the grass, its end sawn and its back mossy, chips of wood about it.
fn lantern_by_a_log_seat() -> Piece {
    let mut s = Canvas::new(38, 40);
    let ground = 37;
    shadow(&mut s, 18, ground, 16, 2);
    // The stick, crooked over at the top where the lantern hangs.
    let (hx, hy) = LANTERN_SEAT_HOOK;
    stick(
        &mut s,
        (33.5, ground as f32 + 0.5),
        (32.0, 8.0),
        (2.6, 2.0),
        BARK,
    );
    stick(
        &mut s,
        (32.5, 8.5),
        (hx as f32 - 0.5, hy as f32 - 1.5),
        (2.0, 1.6),
        BARK,
    );
    // The log, lying along the grass, lit along its back.
    let (left, right, middle, radius): (i32, i32, f32, f32) = (4, 30, 31.0, 5.0);
    for y in 25..=ground {
        for x in left..=right {
            let v = (y as f32 + 0.5 - middle) / radius;
            if v.abs() > 1.0 {
                continue;
            }
            let rim = v.abs() > 0.82;
            let mut color = if rim {
                BARK.edge
            } else if v < -0.45 {
                BARK.light
            } else if v < 0.1 {
                BARK.base
            } else {
                BARK.shadow
            };
            if !rim && noise(x.div_euclid(3), y, 4126).is_multiple_of(4) {
                color = mix(color, BARK.edge, 0.5);
            }
            put(&mut s, x, y, color);
        }
    }
    moss(&mut s, 14.0, 27.0, 7.0, 1.8, 4127);
    moss(&mut s, 25.0, 27.5, 3.0, 1.4, 4128);
    log_end(&mut s, (4.5, middle), (3.0, radius), 4129);
    // Chips of wood in the grass.
    for (x, y) in [(9, 37), (12, 38), (22, 38), (26, 37), (19, 38)] {
        put(&mut s, x, y, WOOD.light);
        put(&mut s, x + 1, y, WOOD.shadow);
    }
    tin_lantern(&mut s, (hx, hy));
    tuft(&mut s, 34, ground, 2, 4130);
    tuft(&mut s, 16, ground + 1, 3, 4131);
    Piece {
        sprite: s,
        anchor: (18, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// The rare pieces
// ---------------------------------------------------------------------------------------------

/// A compass rose set in the turf, seen from a little above: a ring of dressed stone round a
/// floor of slate, an eight-pointed star inlaid in it, cream and slate by halves, the north point
/// red, and the pocket compass set in brass at its heart.
fn compass_rose() -> Piece {
    let mut s = Canvas::new(46, 22);
    let (cx, cy) = (23.0, 11.0);
    let (rx, ry) = (21.5, 8.0);
    for y in 0..22 {
        for x in 0..46 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let d = (u * u + v * v).sqrt();
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.93 {
                STONE.edge
            } else if d > 0.76 {
                // The ring of stones, a joint every so often.
                let angle = v.atan2(u);
                let joint = ((angle + PI) / TAU * 18.0).fract() < 0.07;
                if joint || d > 0.9 {
                    STONE.shadow
                } else if v < 0.0 {
                    STONE.light
                } else {
                    STONE.base
                }
            } else {
                rose_point(u / 0.76, v / 0.76).unwrap_or(if chance(x, y, 4132, 30) {
                    SLATE.edge
                } else {
                    SLATE.shadow
                })
            };
            put(&mut s, x, y, color);
        }
    }
    // The lip of turf over the ring's far edge, and grass and moss at its joints.
    for x in 4..42 {
        let (u, back) = (
            (x as f32 + 0.5 - cx) / rx,
            cy - ry * (1.0 - ((x as f32 + 0.5 - cx) / rx).powi(2)).max(0.0).sqrt(),
        );
        if u.abs() < 0.95 && chance(x, 0, 4133, 120) {
            put(&mut s, x, back as i32, LEAVES.light);
        }
    }
    moss(&mut s, 7.0, 13.0, 2.5, 1.2, 4134);
    moss(&mut s, 37.0, 8.0, 2.5, 1.0, 4135);
    // The compass at its heart.
    ellipse(&mut s, 23, 11, 3, 2, BRASS.edge);
    ellipse(&mut s, 23, 11, 2, 1, BRASS.light);
    put(&mut s, 23, 11, FACE.shine);
    put(&mut s, 22, 11, FACE.base);
    put(&mut s, 24, 11, FACE.base);
    put(&mut s, 23, 10, REDCOAT.light);
    put(&mut s, 21, 10, BRASS.shine);
    // An N cut beyond the north point, on the ring.
    for (x, y) in [(22, 1), (22, 2), (23, 1), (24, 2), (24, 1)] {
        put(&mut s, x, y, REDCOAT.shadow);
    }
    tuft(&mut s, 2, 13, 1, 4136);
    tuft(&mut s, 44, 12, 1, 4137);
    Piece {
        sprite: s,
        anchor: (23, 18),
    }
}

/// Which colour of the compass rose's star covers `(u, v)` on its floor, from -1 to 1 across, or
/// `None` if the floor shows there: four long points to the quarters and four short ones
/// between, each cream on its lit half and slate on the other, the north one red.
fn rose_point(u: f32, v: f32) -> Option<Rgba> {
    let reach = (u * u + v * v).sqrt();
    let angle = v.atan2(u);
    // Which point this is nearest, and how far round from its middle.
    let eighth = TAU / 8.0;
    let index = ((angle + PI + eighth / 2.0) / eighth).floor() as i32 % 8;
    let middle = index as f32 * eighth - PI;
    let off = (angle - middle + PI).rem_euclid(TAU) - PI;
    let long = index % 2 == 0;
    // Each point runs from its tip down to the hollow it shares with the next.
    let tip = if long { 0.98 } else { 0.64 };
    let hollow = 0.3;
    let edge = tip - (tip - hollow) * (off.abs() / (eighth / 2.0));
    if reach > edge {
        return None;
    }
    // North is up the picture: an angle of -90 degrees.
    let north = index == 2;
    let lit = off < 0.0;
    Some(match (north, long, lit) {
        (true, _, true) => REDCOAT.light,
        (true, _, false) => REDCOAT.shadow,
        (false, true, true) => FACE.light,
        (false, true, false) => NAVY.base,
        (false, false, true) => BRASS.light,
        (false, false, false) => BRASS.shadow,
    })
}

/// The cuckoo clock up on a post: a steep carved roof with oak leaves along its eaves, the cuckoo
/// out of its door under the gable looking surprised, the face, the pendulum, and the pine-cone
/// weights hanging on their chains.
fn cuckoo_clock_on_a_post() -> Piece {
    let mut s = Canvas::new(28, 54);
    let ground = 52;
    shadow(&mut s, 14, ground, 5, 2);
    // The post and the shelf it carries the clock on.
    stick(
        &mut s,
        (14.0, ground as f32 + 0.5),
        (14.0, 31.0),
        (3.6, 3.2),
        WOOD,
    );
    hline(&mut s, 9, 31, 10, WOOD.light);
    hline(&mut s, 9, 32, 10, WOOD.edge);
    // The chains, and the cones on them.
    for (x, end) in [(9, 42), (19, 46)] {
        for y in 30..end {
            put(
                &mut s,
                x,
                y,
                if y % 2 == 0 {
                    BRASS.light
                } else {
                    BRASS.shadow
                },
            );
        }
        paint_rows(
            &mut s,
            x - 2,
            end,
            &[".#l#.", "#lso#", "#oso#", "#sos#", ".#s#.", "..#.."],
            CONE,
            &[],
        );
    }
    // The case.
    for y in 14..31_i32 {
        for x in 6..22 {
            let color = if x == 6 || x == 21 || y == 30 {
                WALNUT.edge
            } else if x == 7 {
                WALNUT.light
            } else if x == 20 || noise(x, y.div_euclid(3), 4138).is_multiple_of(5) {
                WALNUT.shadow
            } else {
                WALNUT.base
            };
            put(&mut s, x, y, color);
        }
    }
    // Carved leaves along the bottom of the case.
    for x in (8..21).step_by(3) {
        put(&mut s, x, 29, WALNUT.light);
        put(&mut s, x + 1, 28, WALNUT.light);
        put(&mut s, x + 1, 29, WALNUT.shadow);
    }
    // The roof, steep, its eaves carved with oak leaves, a little bird carved on the ridge.
    for row in 0..12 {
        let y = 15 - row;
        let half = 12 - row;
        for x in 14 - half..14 + half {
            let edge = x == 14 - half || x == 14 + half - 1;
            let color = if edge {
                WALNUT.edge
            } else if x < 14 {
                if row % 2 == 0 {
                    WALNUT.light
                } else {
                    WALNUT.base
                }
            } else if row % 2 == 0 {
                WALNUT.base
            } else {
                WALNUT.shadow
            };
            put(&mut s, x, y, color);
        }
    }
    for (x, y) in [(3, 15), (5, 14), (7, 12), (22, 15), (20, 14), (24, 15)] {
        put(&mut s, x, y, LEAVES.light);
        put(&mut s, x + 1, y, LEAVES.base);
        put(&mut s, x, y + 1, LEAVES.shadow);
    }
    paint_rows(&mut s, 12, 1, &["..#..", ".#l#.", "#lo#."], WALNUT, &[]);
    // The door under the gable, open, and the cuckoo out of it.
    for y in 6..11 {
        for x in 12..17 {
            put(
                &mut s,
                x,
                y,
                if x == 12 || x == 16 || y == 6 {
                    WALNUT.edge
                } else {
                    INSIDE
                },
            );
        }
    }
    paint_rows(
        &mut s,
        9,
        6,
        &["..##..", ".#eW#.", "yy#oo#", "..#ss#", "...##."],
        ROBIN,
        &[('e', NAVY.edge), ('W', rgb(0xffffff)), ('y', rgb(0xe8b040))],
    );
    // The face.
    for y in 16..27 {
        for x in 8..20 {
            let (u, v) = ((x as f32 + 0.5 - 14.0) / 5.6, (y as f32 + 0.5 - 21.5) / 5.4);
            let d = u * u + v * v;
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.8 {
                BRASS.base
            } else if u + v < -0.6 {
                FACE.shine
            } else {
                FACE.base
            };
            put(&mut s, x, y, color);
        }
    }
    for hour in 0..12 {
        let angle = hour as f32 / 12.0 * TAU;
        let (x, y) = (14.0 + angle.sin() * 3.9, 21.5 - angle.cos() * 3.8);
        put(&mut s, x as i32, y as i32, WALNUT.base);
    }
    line(&mut s, (14, 21), (12, 19), WALNUT.edge);
    line(&mut s, (14, 21), (16, 18), WALNUT.edge);
    put(&mut s, 14, 21, BRASS.light);
    // The pendulum, swinging under the case.
    line(&mut s, (14, 31), (13, 36), BRASS.shadow);
    ellipse(&mut s, 13, 37, 2, 2, BRASS.edge);
    ellipse(&mut s, 13, 37, 1, 1, BRASS.light);
    put(&mut s, 12, 36, BRASS.shine);
    tuft(&mut s, 14, ground, 3, 4139);
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// The finest pieces
// ---------------------------------------------------------------------------------------------

/// The bottled ship: a great bottle lying on a polished cradle, and in it a tall ship in full
/// sail, three masts and every rope, on a sea of blue with white caps; the glass catching the
/// light along its top, a cork in its neck, and a brass plate on the cradle.
fn bottled_ship() -> Piece {
    let mut s = Canvas::new(48, 38);
    let ground = 35;
    shadow(&mut s, 24, ground, 21, 2);
    // The cradle: a plinth and two curved rests.
    for y in ground - 3..=ground {
        for x in 4..44 {
            let color = if y == ground - 3 || y == ground || x == 4 || x == 43 {
                WALNUT.edge
            } else if y == ground - 2 {
                WALNUT.light
            } else {
                WALNUT.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 21, ground - 2, 6, GOLD.light);
    hline(&mut s, 21, ground - 1, 6, GOLD.base);
    for x0 in [9, 29] {
        for y in 24..ground - 3 {
            for x in x0..x0 + 7 {
                let edge = x == x0 || x == x0 + 6;
                put(
                    &mut s,
                    x,
                    y,
                    if edge {
                        WALNUT.edge
                    } else if x == x0 + 1 {
                        WALNUT.light
                    } else {
                        WALNUT.base
                    },
                );
            }
        }
    }
    // The bottle: its body, shoulder and neck.
    let (cy, body_end, neck_end) = (17.0, 35.0, 43.0);
    let half = |x: f32| -> f32 {
        if x < 3.0 {
            -1.0
        } else if x < 5.0 {
            8.0 - (5.0 - x) * 1.5
        } else if x < body_end {
            9.0
        } else if x < body_end + 5.0 {
            9.0 - (x - body_end) * 1.2
        } else if x < neck_end {
            3.0
        } else {
            -1.0
        }
    };
    let inside = |x: i32, y: i32| {
        let reach = half(x as f32 + 0.5);
        reach > 0.0 && (y as f32 + 0.5 - cy).abs() <= reach
    };
    for y in 0..ground {
        for x in 0..48 {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let across = (y as f32 + 0.5 - cy) / half(x as f32 + 0.5);
            let color = if edge {
                BOTTLE_GREEN.edge
            } else if across > 0.7 {
                mix(BOTTLE_GREEN.base, rgb(0xd8eee0), 0.35)
            } else {
                mix(
                    rgb(0xcfe6d8),
                    BOTTLE_GREEN.light,
                    0.25 + across.max(0.0) * 0.3,
                )
            };
            put(&mut s, x, y, color);
        }
    }
    // The sea, along the bottom of the bottle.
    for x in 5..body_end as i32 {
        let crest = 21 + ((x as f32 * 0.7).sin() * 1.0) as i32;
        for y in crest..26 {
            if !inside(x, y) || !inside(x, y + 1) {
                continue;
            }
            let color = if y == crest {
                if x % 4 == 0 { SEA.shine } else { SEA.light }
            } else if y > 23 {
                SEA.shadow
            } else {
                SEA.base
            };
            put(&mut s, x, y, color);
        }
    }
    // The hull, riding the sea.
    for y in 18..22 {
        let inset = (y - 18) * 2;
        for x in 11 + inset / 2..31 - inset {
            let color = if y == 18 {
                GOLD.base
            } else if y == 21 || x == 11 + inset / 2 {
                HULL.edge
            } else if y == 19 {
                HULL.light
            } else {
                HULL.base
            };
            put(&mut s, x, y, color);
        }
    }
    for x in [15, 19, 23, 27] {
        put(&mut s, x, 20, GOLD.light);
    }
    // Three masts and their sails, the bowsprit and its jib, a pennant on the mainmast.
    for (mast, top) in [(15, 10), (21, 9), (27, 11)] {
        vline(&mut s, mast, top, 18 - top, HULL.shadow);
        for (index, (from, to)) in [(top + 1, top + 3), (top + 4, top + 6), (top + 7, top + 9)]
            .iter()
            .enumerate()
        {
            let spread = 2 + index as i32;
            for y in *from..=*to {
                for x in mast - spread..=mast + spread {
                    let billow = y == *to && (x == mast - spread || x == mast + spread);
                    if billow || !inside(x, y) {
                        continue;
                    }
                    let color = if x < mast {
                        SAIL.shine
                    } else if x == mast + spread {
                        SAIL.shadow
                    } else {
                        SAIL.base
                    };
                    put(&mut s, x, y, color);
                }
            }
        }
    }
    put(&mut s, 21, 8, REDCOAT.light);
    put(&mut s, 22, 8, REDCOAT.base);
    line(&mut s, (31, 18), (35, 15), HULL.shadow);
    for (y, from) in [(12, 28), (13, 28), (14, 28), (15, 28)] {
        hline(&mut s, from, y, 1 + (y - 11), SAIL.base);
    }
    line(&mut s, (15, 10), (11, 17), HULL.shadow);
    line(&mut s, (27, 11), (34, 15), HULL.shadow);
    // The glass over it all: a long gleam along the top, a lesser one under, the green deeper
    // at the bottom.
    for x in 6..34 {
        put(
            &mut s,
            x,
            10,
            rgba(0xffffff, if x < 20 { 200 } else { 140 }),
        );
    }
    for x in 7..15 {
        put(&mut s, x, 11, rgba(0xffffff, 110));
    }
    for x in 8..30 {
        put(&mut s, x, 25, rgba(0x1e4a2a, 60));
    }
    put(&mut s, 5, 12, rgba(0xffffff, 220));
    put(&mut s, 37, 13, rgba(0xffffff, 180));
    // The cork.
    for y in 15..20 {
        for x in neck_end as i32..neck_end as i32 + 4 {
            let edge = x == neck_end as i32 + 3 || y == 15 || y == 19;
            put(
                &mut s,
                x,
                y,
                if edge {
                    CORK.edge
                } else if y == 16 {
                    CORK.light
                } else {
                    CORK.base
                },
            );
        }
    }
    sparkle(&mut s, (9, 9), 2);
    Piece {
        sprite: s,
        anchor: (24, ground),
    }
}

/// The head of the golden axe, set across its handle at `eye` (the handle running off along
/// `along`): the poll a little way behind the eye, the cheek widening to the blade, and nothing
/// drawn below `buried`, where the blade has gone into the wood.
fn axe_head(s: &mut Canvas, eye: (f32, f32), along: (f32, f32), buried: i32) {
    // Across the handle, towards the blade.
    let across = (-along.1, along.0);
    let across = if across.1 < 0.0 {
        (-across.0, -across.1)
    } else {
        across
    };
    let inside = |x: i32, y: i32| {
        let (dx, dy) = (x as f32 + 0.5 - eye.0, y as f32 + 0.5 - eye.1);
        let (a, b) = (dx * across.0 + dy * across.1, dx * along.0 + dy * along.1);
        // From the poll at -3 to the blade at 9, widening from 3 to 8 across.
        let half = 1.6 + (a + 3.0).max(0.0) * 0.3;
        (-3.0..=9.0).contains(&a) && b.abs() <= half && y <= buried
    };
    for y in (eye.1 as i32 - 8)..=buried {
        for x in (eye.0 as i32 - 8)..=(eye.0 as i32 + 12) {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (dx, dy) = (x as f32 + 0.5 - eye.0, y as f32 + 0.5 - eye.1);
            let (a, b) = (dx * across.0 + dy * across.1, dx * along.0 + dy * along.1);
            let color = if edge && y < buried {
                GOLD.edge
            } else if a < -1.5 || b > 1.8 {
                GOLD.light
            } else if b < -1.6 {
                GOLD.shadow
            } else if (b - 0.5).abs() < 0.6 && a > 0.0 {
                GOLD.shine
            } else {
                GOLD.base
            };
            put(s, x, y, color);
        }
    }
}

/// The golden axe struck in an old stump: the stump broad, its roots spreading into the grass,
/// moss up its shaded side, the rings on its top; the axe sunk in it to the cheek, gold from its
/// blade to the end of its handle, and glinting.
fn golden_axe_in_its_stump() -> Piece {
    let mut s = Canvas::new(44, 46);
    let ground = 43;
    shadow(&mut s, 22, ground, 19, 3);
    // The roots, spreading out from its foot.
    for (from, to) in [
        ((12.0, 36.0), (3.0, 43.0)),
        ((16.0, 38.0), (11.0, 44.0)),
        ((31.0, 36.0), (41.0, 43.0)),
        ((27.0, 38.0), (32.0, 44.0)),
    ] {
        stick(&mut s, from, to, (4.0, 1.6), BARK);
    }
    // The stump: bark in ridges running up it, lit from the left.
    turned(&mut s, 22.0, (22, 40), BARK, |t| 13.0 + t * t * 2.5);
    for y in 24..40 {
        for x in 10..35 {
            if s.get(x, y) == BARK.edge || s.get(x, y).a < 255 {
                continue;
            }
            let ridge =
                (x + noise(x.div_euclid(3), y.div_euclid(5), 4140) as i32 % 2).rem_euclid(3);
            if ridge == 0 {
                let under = s.get(x, y);
                put(&mut s, x, y, mix(under, BARK.edge, 0.6));
            } else if ridge == 1 && chance(x, y, 4145, 60) {
                put(&mut s, x, y, BARK.light);
            }
        }
    }
    moss(&mut s, 31.0, 33.0, 4.0, 6.0, 4141);
    moss(&mut s, 12.0, 38.0, 3.0, 2.0, 4142);
    // The top, sawn flat long ago: rings, and a crack from the heart.
    let (tx, ty, trx, try_) = (22.0, 22.0, 13.0, 4.0);
    for y in 17..27 {
        for x in 8..37 {
            let (u, v) = ((x as f32 + 0.5 - tx) / trx, (y as f32 + 0.5 - ty) / try_);
            let d = (u * u + v * v).sqrt();
            if d > 1.0 {
                continue;
            }
            let color = if d > 0.9 {
                BARK.edge
            } else if d > 0.8 {
                BARK.light
            } else {
                match ((d * 5.0) as i32 + i32::from(chance(x, y, 4143, 40))) % 2 {
                    0 => RINGS.light,
                    _ => RINGS.base,
                }
            };
            put(&mut s, x, y, color);
        }
    }
    // The axe: the handle rising out of the top, and the head across the end of it, its poll
    // standing up out of the wood and its blade sunk in.
    stick(&mut s, (20.0, 15.5), (35.0, 3.0), (2.6, 2.2), GOLD);
    axe_head(&mut s, (19.0, 15.0), (0.77, -0.64), 22);
    // Where the blade bit in, the wood split either side of it.
    hline(&mut s, 12, 22, 14, RINGS.edge);
    line(&mut s, (11, 22), (9, 24), RINGS.edge);
    line(&mut s, (26, 22), (29, 24), RINGS.edge);
    put(&mut s, 35, 3, GOLD.shine);
    sparkle(&mut s, (16, 16), 3);
    sparkle(&mut s, (32, 6), 2);
    sparkle(&mut s, (25, 11), 1);
    toadstool(&mut s, 7, ground);
    tuft(&mut s, 38, ground, 2, 4144);
    Piece {
        sprite: s,
        anchor: (22, ground),
    }
}

// ---------------------------------------------------------------------------------------------
// What is built from them
// ---------------------------------------------------------------------------------------------

/// Where the lantern hangs in front of the woodshed.
const WOODSHED_HOOK: (i32, i32) = (5, 22);

/// What shines from something built from these finds after dark, and where in it.
pub(super) fn built_lights(plan: &str) -> Vec<(Rgba, (i32, i32))> {
    match plan {
        "woodshed" => vec![(CANDLELIGHT, (WOODSHED_HOOK.0, WOODSHED_HOOK.1 + 10))],
        _ => Vec::new(),
    }
}

/// The woodshed: a lean-to with its roof shingled, logs stacked inside with their sawn ends out,
/// the carved sign nailed over the opening pointing in, the woodcutter's lantern hung by it, and
/// two pinecones for kindling at its foot.
pub(super) fn woodshed() -> Piece {
    let mut s = Canvas::new(48, 48);
    let ground = 45;
    shadow(&mut s, 24, ground, 22, 3);
    // The back wall, boards in shade.
    for y in 15..ground {
        for x in 5..43 {
            let board = (x - 5) / 4;
            let mut color = if (x - 5) % 4 == 0 {
                mix(WOOD.edge, INSIDE, 0.5)
            } else if board % 2 == 0 {
                mix(WOOD.shadow, INSIDE, 0.35)
            } else {
                mix(WOOD.base, INSIDE, 0.45)
            };
            if noise(x, y.div_euclid(3), 4150).is_multiple_of(7) {
                color = mix(color, INSIDE, 0.5);
            }
            put(&mut s, x, y, color);
        }
    }
    // The logs, stacked in rows with their sawn ends out, each row settled into the one below.
    for (row, y) in [(0, 41.0), (1, 36.0), (2, 31.0), (3, 26.5)] {
        let offset = if row % 2 == 0 { 0.0 } else { 3.2 };
        let mut x = 10.5 + offset;
        let mut index = 0;
        while x < 39.0 {
            log_end(&mut s, (x, y), (3.2, 2.9), 4151 + row * 10 + index);
            x += 6.4;
            index += 1;
        }
    }
    // The corner posts at the front.
    stick(
        &mut s,
        (5.5, ground as f32 + 0.5),
        (5.5, 15.0),
        (3.4, 3.2),
        WOOD,
    );
    stick(
        &mut s,
        (42.5, ground as f32 + 0.5),
        (42.5, 15.0),
        (3.4, 3.2),
        WOOD,
    );
    // The roof, sloping down to the front, shingled in courses, and its fascia board.
    for y in 4..15_i32 {
        let inset = (14 - y) / 4;
        for x in 1 + inset..47 - inset {
            let course = (y - 4) / 2;
            let shingle = (x + course * 2).rem_euclid(5) == 0;
            let color = if y == 4 || x == 1 + inset || x == 46 - inset {
                SHINGLE.edge
            } else if shingle || (y - 4) % 2 == 1 {
                if x < 24 { SHINGLE.base } else { SHINGLE.shadow }
            } else if x < 24 {
                SHINGLE.light
            } else {
                SHINGLE.base
            };
            put(&mut s, x, y, color);
        }
    }
    moss(&mut s, 12.0, 7.0, 5.0, 1.6, 4152);
    for y in 15..18 {
        for x in 1..47 {
            let color = if y == 17 || x == 1 || x == 46 {
                WOOD.edge
            } else if y == 15 {
                WOOD.light
            } else {
                WOOD.base
            };
            put(&mut s, x, y, color);
        }
    }
    // The carved sign over the opening, and the lantern on its bracket.
    carved_board(&mut s, (14, 18), 22, false, 4153);
    let (hx, hy) = WOODSHED_HOOK;
    hline(&mut s, hx, hy - 1, 3, IRON.base);
    tin_lantern(&mut s, (hx, hy));
    // Pinecones at its foot, and grass grown up round it.
    super::hollow::cone(&mut s, 30, ground - 4, super::hollow::Lie::Right);
    super::hollow::cone(&mut s, 37, ground - 3, super::hollow::Lie::Left);
    tuft(&mut s, 12, ground, 3, 4154);
    tuft(&mut s, 45, ground, 2, 4155);
    Piece {
        sprite: s,
        anchor: (24, ground),
    }
}

/// A tea party: a stump for a table under a gingham cloth, the copper kettle on it, the good
/// china out on its saucers, a bowl of berries, and two log stools drawn up.
pub(super) fn tea_party() -> Piece {
    let mut s = Canvas::new(48, 34);
    let ground = 31;
    shadow(&mut s, 24, ground, 22, 3);
    // The stools.
    for cx in [6.5_f32, 41.5] {
        turned(&mut s, cx, (24, ground), BARK, |_| 4.6);
        log_end(&mut s, (cx, 24.0), (4.6, 1.8), 4160 + cx as u32);
    }
    // The stump, and the cloth over it, red and white checks to a scalloped hem.
    turned(&mut s, 24.0, (19, ground), BARK, |t| 10.5 + t * t * 1.5);
    let (tx, ty, rx, ry) = (24.0, 19.0, 12.6, 4.0);
    let hem = |x: i32| 25 + i32::from((x - 13).rem_euclid(4) < 2);
    for y in 14..27 {
        for x in 10..39 {
            let (u, v) = ((x as f32 + 0.5 - tx) / rx, (y as f32 + 0.5 - ty) / ry);
            let top = u * u + v * v <= 1.0;
            let drape = u.abs() <= 1.0 && y as f32 > ty && y <= hem(x);
            if !top && !drape {
                continue;
            }
            let check = (x.div_euclid(2) + y.div_euclid(2)).rem_euclid(2) == 0;
            let red = x.div_euclid(2).rem_euclid(2) == 0 || y.div_euclid(2).rem_euclid(2) == 0;
            let mut color = match (check && red, red) {
                (true, _) => REDCOAT.base,
                (false, true) => mix(REDCOAT.light, TRIM.light, 0.45),
                _ => TRIM.light,
            };
            let rim = !top && (y == hem(x) || u.abs() > 0.95);
            if rim {
                color = mix(color, REDCOAT.edge, 0.6);
            } else if !top || u > 0.5 {
                color = mix(color, REDCOAT.edge, 0.22);
            }
            put(&mut s, x, y, color);
        }
    }
    // The kettle at the back, a cup on its saucer beside it, and a bowl of berries and the other
    // cup in front.
    kettle(&mut s, (16, 18));
    saucer(&mut s, (31, 16), 5, true);
    teacup(&mut s, (31, 15), (3, 4), true, rgb(0x9a6a3a));
    turned(&mut s, 30.5, (20, 23), CHINA, |t| 3.8 - t * 1.4);
    for (x, y, dark) in [
        (28, 19, false),
        (30, 18, true),
        (32, 19, false),
        (29, 20, true),
        (31, 20, false),
    ] {
        let ramp = if dark { BERRY_DARK } else { BERRY_RED };
        put(&mut s, x, y, ramp.light);
        put(&mut s, x + 1, y, ramp.base);
        put(&mut s, x, y + 1, ramp.shadow);
    }
    saucer(&mut s, (21, 22), 5, true);
    teacup(&mut s, (21, 21), (3, 4), true, rgb(0x9a6a3a));
    tuft(&mut s, 2, ground, 2, 4161);
    tuft(&mut s, 46, ground, 2, 4162);
    flowers(&mut s, 24, ground + 1, 4, 2, 4163);
    Piece {
        sprite: s,
        anchor: (24, ground),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::SCAVENGED;

    #[test]
    fn every_find_from_the_old_track_is_drawn_and_its_lights_fall_on_it() {
        for find in &SCAVENGED {
            let drawn = icon(find.id).unwrap_or_else(|| panic!("{} has no icon", find.id));
            assert_eq!((drawn.width(), drawn.height()), (ICON, ICON));
            let piece = piece(find.id).unwrap_or_else(|| panic!("{} has no piece", find.id));
            for (_, (x, y)) in lights(find.id) {
                assert!(
                    piece.sprite.get(x, y).a > 0,
                    "{}'s light shines from nothing",
                    find.id
                );
            }
        }
    }
}
