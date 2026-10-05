//! What grows along the hedgerow, at each stage of ripeness: the look that is the whole of
//! foraging's reading. Each plant has its own tells. Blackberries go from hard and green to red,
//! and only once they are black and glossy are they ripe; then they dull and shrink. A field
//! mushroom is a closed button, then lifts its cap on its veil, and is ripe with its cap open and
//! its pink gills showing, until the cap flattens and browns. Hazelnuts loosen and brown in husks
//! that open; crab apples yellow and blush; rose hips go orange, then scarlet and glossy; elder
//! heads open from green buds into cream; thyme comes into flower; wild garlic's papery bud splits
//! into white stars; honeysuckle's long buds open into trumpets; a chanterelle grows from a pale
//! knob to a golden trumpet. And a four-leaf clover only unfolds its fourth leaf when it is ready.
//!
//! Ripe is always the brightest, fullest and shiniest a thing gets, and over always dull, small
//! and sagging, so the person can read a hedge at a glance and still have each plant's own look to
//! learn. Unripe is a hard yellow-green, paler and yellower than the leaves it grows among, and
//! over a dusty brown or grey, never as dark as the hedge's shade, so that something not ready, or
//! past it, still stands out from the leaves at a glance. Everything is drawn a little larger than
//! life, at the colony's scale, lit from the upper left and outlined in a darker shade of its own
//! colour.

use super::{Plant, Slot};
use crate::paint::{Ramp, put, rgb, rgba};
use crate::playground::Prop;
use formiga_art::{Canvas, Rgba};

/// Where something is in its ripening.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    /// Not ready: green, hard, closed.
    Unripe,
    /// Nearly: blushing, swelling, opening.
    Turning,
    /// Ready to pick.
    Ripe,
    /// Past its best: dull, shrunk, sagging. It will drop soon.
    Over,
}

impl Stage {
    pub const ALL: [Self; 4] = [Self::Unripe, Self::Turning, Self::Ripe, Self::Over];
}

/// Every sprite is drawn on a canvas this big, with its anchor at `ANCHOR`.
const SIZE: (u32, u32) = (13, 13);

/// Leaf and stalk tones shared by everything on the hedge.
const LEAF: Ramp = Ramp::new(0x22401f, 0x335a2a, 0x4a7834, 0x649444, 0x88b45c);
const DEAD_LEAF: Ramp = Ramp::new(0x4a3a22, 0x6a5430, 0x8a6e40, 0xa88a56, 0xc4a872);
const STALK: Rgba = rgb(0x4a5a2a);
const WOODY: Rgba = rgb(0x5a3a2a);

/// Hard and unripe: a yellow-green, yellower and paler than any leaf on the hedge.
const UNRIPE: Ramp = Ramp::new(0x485218, 0x788826, 0xa4b23c, 0xc8d468, 0xecf0b0);

// Blackberries.
const BERRY_GREEN: Ramp = UNRIPE;
const BERRY_RED: Ramp = Ramp::new(0x5a1420, 0x8a2230, 0xb83a3a, 0xd8604c, 0xf4a088);
const BERRY_BLACK: Ramp = Ramp::new(0x0e0818, 0x22143a, 0x36204e, 0x54386e, 0xe4dcff);
const BERRY_OVER: Ramp = Ramp::new(0x3a3038, 0x5a4c58, 0x766874, 0x8a7c88, 0x9a8e98);
// Strawberries.
const STRAW_WHITE: Ramp = Ramp::new(0x6a7a4a, 0xa8b888, 0xd0dcb0, 0xe8f0d0, 0xf8fcf0);
const STRAW_BLUSH: Ramp = Ramp::new(0x8a3a3a, 0xc06a5a, 0xe09a80, 0xf0c0a8, 0xfce4d8);
const STRAW_RED: Ramp = Ramp::new(0x6a0e14, 0xa81c22, 0xd8302c, 0xf05a44, 0xffd0c0);
const STRAW_OVER: Ramp = Ramp::new(0x3a1a1a, 0x5a2a26, 0x74382e, 0x84483a, 0x946050);
const SEED: Rgba = rgb(0xf0d060);
// Rose hips.
const HIP_GREEN: Ramp = UNRIPE;
const HIP_ORANGE: Ramp = Ramp::new(0x7a3410, 0xb0561a, 0xe07a24, 0xf4a040, 0xffd8a0);
const HIP_SCARLET: Ramp = Ramp::new(0x5a0a10, 0x9a121a, 0xd42020, 0xf04a34, 0xffe0d0);
const HIP_OVER: Ramp = Ramp::new(0x4a2a20, 0x7a4636, 0x9a6048, 0xb27c5e, 0xc89878);
// Crab apples.
const APPLE_GREEN: Ramp = Ramp::new(0x445618, 0x708c28, 0x9cb63e, 0xc2d666, 0xe8f2ae);
const APPLE_YELLOW: Ramp = Ramp::new(0x6a6a1a, 0x9a9a26, 0xc4c03a, 0xe0dc5e, 0xf8f4a8);
const APPLE_GOLD: Ramp = Ramp::new(0x7a5212, 0xb88a1e, 0xe8c034, 0xf8dc60, 0xfffcd8);
const BLUSH: Ramp = Ramp::new(0x6a1414, 0xa02420, 0xd03c2c, 0xe86440, 0xf89070);
const APPLE_OVER: Ramp = Ramp::new(0x3e2614, 0x64421e, 0x86602c, 0x9a7438, 0xaa8448);
// Hazelnuts.
const NUT_PALE: Ramp = Ramp::new(0x6a7044, 0x98a066, 0xc0c48a, 0xd8dcaa, 0xf0f0d4);
const NUT_TAN: Ramp = Ramp::new(0x6a4a24, 0x9a6e36, 0xbc9050, 0xd4ae6c, 0xecd4a0);
const NUT_BROWN: Ramp = Ramp::new(0x3a1e0e, 0x6a3a18, 0x965424, 0xb87436, 0xf4d8b0);
const NUT_OVER: Ramp = Ramp::new(0x3a2a1c, 0x5a4430, 0x725a40, 0x846a4c, 0x947a5a);
const HUSK: Ramp = Ramp::new(0x465818, 0x748e2a, 0xa0b840, 0xc4d868, 0xe8f2ae);
const HUSK_DRY: Ramp = Ramp::new(0x5a4220, 0x82602c, 0xa8823e, 0xc4a258, 0xdcc488);
// Elder.
const ELDER_BUD: Ramp = Ramp::new(0x4e5626, 0x7e8a3c, 0xb2bc6a, 0xd6dc98, 0xf2f2d0);
const ELDER_CREAM: Ramp = Ramp::new(0x8a8460, 0xc8c098, 0xece4c0, 0xf8f4dc, 0xffffff);
const ELDER_OVER: Ramp = Ramp::new(0x4e3e28, 0x7a6640, 0x968052, 0xaa9466, 0xbea87a);
// Thyme.
const THYME: Ramp = Ramp::new(0x24381e, 0x34502a, 0x486a36, 0x5e8444, 0x7aa05a);
const THYME_BUD: Ramp = Ramp::new(0x7a5068, 0xa87a90, 0xcca0b4, 0xe4c4d0, 0xf8e8f0);
const THYME_FLOWER: Ramp = Ramp::new(0x5a1e5a, 0x8a3088, 0xb84ab0, 0xd874cc, 0xf4b8ec);
const THYME_OVER: Ramp = Ramp::new(0x4a3a2a, 0x6a5440, 0x846c52, 0x9a8266, 0xae987c);
// Wild garlic.
const GARLIC_LEAF: Ramp = Ramp::new(0x1e4024, 0x2c5a30, 0x3e783e, 0x58944e, 0x7cb064);
const SPATHE: Ramp = Ramp::new(0x6a7a5a, 0x9aa888, 0xc4ceb0, 0xdce4cc, 0xf0f4e8);
const STAR: Ramp = Ramp::new(0x9aa0a4, 0xd4d8d8, 0xf4f4f0, 0xfcfcf8, 0xffffff);
const GARLIC_OVER: Ramp = Ramp::new(0x5a5a26, 0x7a7a34, 0x9a9446, 0xb0aa5a, 0xc8c078);
// Mushrooms.
const BUTTON: Ramp = Ramp::new(0x8a8478, 0xc4bcae, 0xe4ded2, 0xf4f0e8, 0xffffff);
const GILLS: Ramp = Ramp::new(0x6a3a3a, 0x9a5a56, 0xc48478, 0xd8a094, 0xecc4b8);
const VEIL: Rgba = rgb(0xf0e8d4);
const CAP_OVER: Ramp = Ramp::new(0x4a3a2c, 0x6e5a46, 0x8e7860, 0xa48e76, 0xb8a48c);
const GILLS_OVER: Rgba = rgb(0x3a2620);
// Chanterelle.
const CHANT_PALE: Ramp = Ramp::new(0x8a7a3a, 0xbcaa5a, 0xdcca7a, 0xece09a, 0xf8f0c8);
const CHANT_GOLD: Ramp = Ramp::new(0x7a4a08, 0xc07a10, 0xf0a818, 0xffc840, 0xfff4b0);
const CHANT_OVER: Ramp = Ramp::new(0x4a2a10, 0x6e4018, 0x8a5422, 0x9e6430, 0xb07a44);
// Honeysuckle.
const SUCKLE_BUD: Ramp = Ramp::new(0x4e5a26, 0x82923a, 0xb0c058, 0xd4e08a, 0xf2f6c8);
const SUCKLE_PINK: Ramp = Ramp::new(0x7a3a4a, 0xb05a6a, 0xd88094, 0xeca8b8, 0xfcd8e0);
const SUCKLE_CREAM: Ramp = Ramp::new(0x9a7a3a, 0xd8b85a, 0xf4dc84, 0xfcecb0, 0xfffcf0);
const SUCKLE_OVER: Ramp = Ramp::new(0x4a3022, 0x7a5236, 0x946a46, 0xa88058, 0xbc966c);
// Clover.
const CLOVER: Ramp = Ramp::new(0x1e4a22, 0x2e6a2e, 0x429040, 0x5eb050, 0xa0e080);
const CLOVER_DULL: Ramp = Ramp::new(0x2a4424, 0x3a5a30, 0x4e7240, 0x648a50, 0x7ca064);
const CLOVER_OVER: Ramp = Ramp::new(0x4a4a20, 0x6a6a2c, 0x8a843c, 0xa09a50, 0xb8b068);
const CHEVRON: Rgba = rgb(0xd8f4c8);

/// A rounded fruit's tone at a point `(u, v)` across it, each from -1 to 1: lit from the upper
/// left, with its shine a pixel or two at the top left and its own edge tone round its rim.
fn fruit_tone(ramp: Ramp, u: f32, v: f32, shiny: bool) -> Rgba {
    let lit = -0.62 * u - 0.7 * v;
    if shiny && (-0.75..-0.2).contains(&u) && (-0.8..-0.25).contains(&v) && lit > 0.55 {
        ramp.shine
    } else if lit > 0.35 {
        ramp.light
    } else if lit > -0.35 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// A rounded fruit centred on `(cx, cy)`, `(rx, ry)` across, outlined in its own edge.
fn fruit(s: &mut Canvas, (cx, cy): (f32, f32), (rx, ry): (f32, f32), ramp: Ramp, shiny: bool) {
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
        u * u + v * v <= 1.0
    };
    let (left, right) = ((cx - rx - 1.0) as i32, (cx + rx + 1.0) as i32);
    let (top, bottom) = ((cy - ry - 1.0) as i32, (cy + ry + 1.0) as i32);
    for y in top..=bottom {
        for x in left..=right {
            if !inside(x, y) {
                continue;
            }
            let rim =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let color = if rim {
                ramp.edge
            } else {
                fruit_tone(ramp, u, v, shiny)
            };
            s.set(x, y, color);
        }
    }
}

/// Where in every sprite's canvas the slot's point falls: for something that hangs, where it
/// hangs from; for something that stands, the middle of its foot.
pub fn anchor(plant: Plant) -> (i32, i32) {
    if hangs(plant) { (6, 1) } else { (6, 11) }
}

/// Whether a plant's produce hangs from where it grows, rather than standing on the ground.
fn hangs(plant: Plant) -> bool {
    matches!(
        plant,
        Plant::Bramble | Plant::Strawberry | Plant::Hazel | Plant::CrabApple | Plant::DogRose
    )
}

/// A plant's produce at a stage of ripeness. `glint` adds the twinkle a ripe thing gives off now
/// and then; reduced motion keeps it still.
pub fn sprite(plant: Plant, stage: Stage, glint: bool) -> Canvas {
    let mut s = Canvas::new(SIZE.0, SIZE.1);
    match plant {
        Plant::Bramble => blackberries(&mut s, stage),
        Plant::Strawberry => strawberry(&mut s, stage),
        Plant::Hazel => hazelnut(&mut s, stage),
        Plant::Mushroom => mushroom(&mut s, stage),
        Plant::WildGarlic => wild_garlic(&mut s, stage),
        Plant::Elder => elderflower(&mut s, stage),
        Plant::CrabApple => crab_apple(&mut s, stage),
        Plant::DogRose => rose_hip(&mut s, stage),
        Plant::Thyme => thyme(&mut s, stage),
        Plant::Chanterelle => chanterelle(&mut s, stage),
        Plant::Honeysuckle => honeysuckle(&mut s, stage),
        Plant::Clover => clover(&mut s, stage),
    }
    if glint && stage == Stage::Ripe {
        let (x, y) = glint_at(plant);
        put(&mut s, x, y, rgba(0xffffff, 230));
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            put(&mut s, x + dx, y + dy, rgba(0xfff8d8, 150));
        }
    }
    s
}

/// Where a ripe thing catches the light.
fn glint_at(plant: Plant) -> (i32, i32) {
    match plant {
        Plant::Bramble => (4, 5),
        Plant::Strawberry => (5, 5),
        Plant::Hazel => (5, 7),
        Plant::Mushroom => (4, 4),
        Plant::WildGarlic => (5, 1),
        Plant::Elder => (4, 2),
        Plant::CrabApple => (5, 5),
        Plant::DogRose => (5, 5),
        Plant::Thyme => (5, 6),
        Plant::Chanterelle => (5, 4),
        Plant::Honeysuckle => (4, 4),
        Plant::Clover => (5, 5),
    }
}

/// The produce growing at a slot, at a stage, as something standing among everyone on `base`.
/// Ripe things twinkle now and then; with reduced motion the twinkle is held still.
pub fn prop(slot: &Slot, base: f32, stage: Stage, now: f32, reduce_motion: bool) -> Prop {
    let phase = (slot.at.0 * 0.37 + slot.at.1 * 0.11).fract();
    let glint = reduce_motion || (now * 0.45 + phase).fract() < 0.12;
    let sprite = sprite(slot.plant, stage, glint);
    let (ax, ay) = anchor(slot.plant);
    Prop::new(sprite, (slot.at.0 as i32 - ax, slot.at.1 as i32 - ay), base)
}

/// The inks a sprite's rows are painted in: its main material by `#`, `s`, `o`, `l` and `*`, edge
/// to shine; a second material by `E`, `S`, `O`, `L` and `H`; leaves by `d`, `v`, `g` and `G`,
/// edge to light; a stalk by `k` and a woody one by `w`; and anything else by name. `.` is clear.
struct Inks<'a> {
    main: Ramp,
    second: Ramp,
    leaf: Ramp,
    extra: &'a [(char, Rgba)],
}

impl<'a> Inks<'a> {
    fn new(main: Ramp) -> Self {
        Self {
            main,
            second: main,
            leaf: LEAF,
            extra: &[],
        }
    }

    fn second(self, second: Ramp) -> Self {
        Self { second, ..self }
    }

    fn leaf(self, leaf: Ramp) -> Self {
        Self { leaf, ..self }
    }

    fn extra(self, extra: &'a [(char, Rgba)]) -> Self {
        Self { extra, ..self }
    }

    fn ink(&self, code: char) -> Option<Rgba> {
        Some(match code {
            '#' => self.main.edge,
            's' => self.main.shadow,
            'o' => self.main.base,
            'l' => self.main.light,
            '*' => self.main.shine,
            'E' => self.second.edge,
            'S' => self.second.shadow,
            'O' => self.second.base,
            'L' => self.second.light,
            'H' => self.second.shine,
            'd' => self.leaf.edge,
            'v' => self.leaf.shadow,
            'g' => self.leaf.base,
            'G' => self.leaf.light,
            'k' => STALK,
            'w' => WOODY,
            other => {
                return self
                    .extra
                    .iter()
                    .find(|(name, _)| *name == other)
                    .map(|(_, color)| *color);
            }
        })
    }
}

/// Paints rows of letters in `inks` with their top-left at `at`.
fn paint(s: &mut Canvas, at: (i32, i32), lines: &[&str], inks: &Inks) {
    for (dy, line) in lines.iter().enumerate() {
        for (dx, code) in line.chars().enumerate() {
            if let Some(color) = inks.ink(code) {
                put(s, at.0 + dx as i32, at.1 + dy as i32, color);
            }
        }
    }
}

/// A ramp a shade darker all through, for a berry behind another.
fn darker(ramp: Ramp) -> Ramp {
    Ramp {
        edge: ramp.edge,
        shadow: ramp.edge,
        base: ramp.shadow,
        light: ramp.base,
        shine: ramp.light,
    }
}

/// A sprig of blackberries hanging from the cane, a leaf at its stalk: one berry in front and one
/// behind. Hard and green, then red, then black and glossy; over, dull and shrunk, the one behind
/// already dropped.
fn blackberries(s: &mut Canvas, stage: Stage) {
    let ramp = match stage {
        Stage::Unripe => BERRY_GREEN,
        Stage::Turning => BERRY_RED,
        Stage::Ripe => BERRY_BLACK,
        Stage::Over => BERRY_OVER,
    };
    let leaf = if stage == Stage::Over {
        DEAD_LEAF
    } else {
        LEAF
    };
    let extra = [('m', rgb(0xb4aeb4))];
    let inks = Inks::new(ramp)
        .second(darker(ramp))
        .leaf(leaf)
        .extra(&extra);
    paint(
        s,
        (0, 0),
        &[
            "......k......",
            "..dd..k......",
            ".dGgd.k......",
            "..dvd.kkk....",
        ],
        &inks,
    );
    match stage {
        Stage::Unripe => {
            paint(s, (7, 4), &[".EE.", "ELOE", "EOSE", ".EE."], &inks);
            paint(
                s,
                (3, 5),
                &[".#.#.", "#lol#", "#olo#", "#sos#", ".###."],
                &inks,
            );
        }
        Stage::Turning | Stage::Ripe => {
            paint(
                s,
                (7, 3),
                &[".E.E.", "ELOLE", "EOLOE", "ESOSE", ".ESE."],
                &inks,
            );
            let front: &[&str] = if stage == Stage::Ripe {
                &[
                    "..#.#..", ".#*#l#.", "#*lolo#", "#lolos#", "#oloso#", ".#osos#", "..#s#..",
                ]
            } else {
                &[
                    "..#.#..", ".#l#l#.", "#llolo#", "#lolos#", "#oloso#", ".#osos#", "..#s#..",
                ]
            };
            paint(s, (2, 4), front, &inks);
        }
        Stage::Over => {
            paint(
                s,
                (3, 6),
                &[".#.#.", "#oom#", "#osm#", ".#s#.", "..#.."],
                &inks,
            );
        }
    }
}

/// A wild strawberry hanging under its star of a calyx, a cone seeded all over: small and white,
/// then blushing, then scarlet and glossy; over, dark and soft, with a grey bloom.
fn strawberry(s: &mut Canvas, stage: Stage) {
    let (ramp, seed) = match stage {
        Stage::Unripe => (STRAW_WHITE, rgb(0xa8c070)),
        Stage::Turning => (STRAW_BLUSH, rgb(0xf0e0a0)),
        Stage::Ripe => (STRAW_RED, SEED),
        Stage::Over => (STRAW_OVER, rgb(0x5a4030)),
    };
    let leaf = if stage == Stage::Over {
        DEAD_LEAF
    } else {
        LEAF
    };
    let extra = [('y', seed), ('m', rgb(0xb0aca8))];
    let inks = Inks::new(ramp).leaf(leaf).extra(&extra);
    paint(
        s,
        (0, 0),
        &["......k......", "......k......", ".....dkd....."],
        &inks,
    );
    let berry: &[&str] = match stage {
        Stage::Unripe => &[
            "....#lyl#....",
            "....#yoy#....",
            ".....#o#.....",
            "......#......",
        ],
        Stage::Turning => &[
            "...#llylo#...",
            "...#lyoly#...",
            "...#oloys#...",
            "....#yos#....",
            ".....#s#.....",
            "......#......",
        ],
        Stage::Ripe => &[
            "...#*lylo#...",
            "...#lyoly#...",
            "...#oloys#...",
            "....#yos#....",
            ".....#s#.....",
            "......#......",
        ],
        Stage::Over => &[
            "....#ooo#....",
            "....#oym#....",
            ".....#sm#....",
            "......##.....",
        ],
    };
    paint(s, (0, 4), berry, &inks);
    paint(s, (0, 3), &["...dgGgGgd..."], &inks);
}

/// A hazelnut hanging in its frilled husk: tight in a green husk, then tan and showing, then
/// glossy brown and loose in a husk gone dry and open; over, dark, with a weevil's hole.
fn hazelnut(s: &mut Canvas, stage: Stage) {
    let (nut, husk) = match stage {
        Stage::Unripe => (NUT_PALE, HUSK),
        Stage::Turning => (NUT_TAN, HUSK),
        Stage::Ripe => (NUT_BROWN, HUSK_DRY),
        Stage::Over => (NUT_OVER, HUSK_DRY),
    };
    let extra = [('x', rgb(0x120a06)), ('p', NUT_TAN.light)];
    let inks = Inks::new(nut).second(husk).extra(&extra);
    paint(s, (0, 0), &["......k......", "......k......"], &inks);
    let lines: &[&str] = match stage {
        Stage::Unripe => &[
            "....E.k.E....",
            "...ELHLOLE...",
            "...ELOLOSE...",
            "...EOLOOSE...",
            "....ESOSE....",
            ".....#l#.....",
            "......#......",
        ],
        Stage::Turning => &[
            "...E.EkE.E...",
            "...ELHLOLE...",
            "...ELOLOSE...",
            "...E#lol#E...",
            "....#lol#....",
            "....#oos#....",
            ".....###.....",
        ],
        Stage::Ripe => &[
            "...E.EkE.E...",
            "..ELHLOLOLE..",
            "..E.EpppE.E..",
            "...#*lolo#...",
            "...#lolos#...",
            "...#oloso#...",
            "....#oss#....",
            ".....###.....",
        ],
        Stage::Over => &[
            "....E.k.E....",
            "...ESOSOSE...",
            "...E#oso#E...",
            "...#oxos#....",
            "...#osss#....",
            "....#ss#.....",
            ".....##......",
        ],
    };
    paint(s, (0, 2), lines, &inks);
}

/// A field mushroom: a white button, then its cap lifting on its veil, then open with pink gills
/// showing under it and a ring on its stalk; over, flat and brown, gills gone dark, and bitten.
fn mushroom(s: &mut Canvas, stage: Stage) {
    let (cap, gills) = match stage {
        Stage::Over => (CAP_OVER, darker(CAP_OVER)),
        _ => (BUTTON, GILLS),
    };
    let extra = [('V', VEIL), ('x', GILLS_OVER)];
    let inks = Inks::new(cap).second(gills).extra(&extra);
    let lines: &[&str] = match stage {
        Stage::Unripe => &[
            ".....###.....",
            "....#*ll#....",
            "....#loo#....",
            ".....#s#.....",
        ],
        Stage::Turning => &[
            ".....###.....",
            "...##*ll##...",
            "..#llllooo#..",
            "..#oooooss#..",
            "...VVVVVVV...",
            ".....#l#.....",
        ],
        Stage::Ripe => &[
            "....#####....",
            "..##l*lll##..",
            ".#l*llllooo#.",
            "#lloooooosss#",
            ".ESOSOSOSOSE.",
            ".....#lo#....",
            "....VlloV....",
            ".....#ls#....",
        ],
        Stage::Over => &[
            ".#.........#.",
            ".#o###.###o#.",
            "..#ooo.ooss#.",
            "..xxxxxxxxx..",
            ".....#o#.....",
            ".....#o#.....",
            ".....#s#.....",
        ],
    };
    paint(s, (0, 12 - lines.len() as i32), lines, &inks);
}

/// Wild garlic: two broad leaves arching from its foot and a stem with its flower. A papery bud,
/// then the bud splitting white, then a ball of white stars; over, the leaves yellowing and the
/// flowers gone to seed.
fn wild_garlic(s: &mut Canvas, stage: Stage) {
    let leaf = if stage == Stage::Over {
        GARLIC_OVER
    } else {
        GARLIC_LEAF
    };
    let flower = match stage {
        Stage::Unripe => SPATHE,
        Stage::Over => GARLIC_OVER,
        _ => STAR,
    };
    let inks = Inks::new(flower).second(SPATHE).leaf(leaf);
    paint(
        s,
        (0, 4),
        &[
            "......k......",
            ".d....k....d.",
            "dGd...k...dGd",
            "dGgd..k..dgGd",
            ".dggd.k.dggd.",
            "..dggdkdggd..",
            "...dggkggd...",
            "....dgkgd....",
        ],
        &inks,
    );
    let head: &[&str] = match stage {
        Stage::Unripe => &[
            "......E......",
            ".....ELE.....",
            "....ELHOE....",
            "....EOOSE....",
            ".....ESE.....",
        ],
        Stage::Turning => &[
            "......#......",
            ".....#*#.....",
            "....#l*lE....",
            ".....SoE.....",
            "......k......",
        ],
        Stage::Ripe => &[
            "...l.*.l.....",
            "..*o*l*o*l...",
            "...l*o*o*o*..",
            "..*o*l*o*l...",
            "....l.k.l....",
        ],
        Stage::Over => &[
            "...o.o.o.....",
            "..o#o#o#o....",
            "...o#k#o.....",
            "......k......",
            "......k......",
        ],
    };
    paint(s, (0, 0), head, &inks);
}

/// A head of elderflower held up on its stalk from the bush: a little dome of tight green buds,
/// then cream buds, then open into a froth of cream florets with gold eyes; over, browning and
/// sagging, with its petals dropping.
fn elderflower(s: &mut Canvas, stage: Stage) {
    let ramp = match stage {
        Stage::Unripe => ELDER_BUD,
        Stage::Turning | Stage::Ripe => ELDER_CREAM,
        Stage::Over => ELDER_OVER,
    };
    let extra = [('y', rgb(0xe8c84a))];
    let inks = Inks::new(ramp).extra(&extra);
    let head: &[&str] = match stage {
        Stage::Unripe => &[
            ".............",
            ".....#.#.....",
            "....#lll#....",
            "...#lolol#...",
            "..#sosososs#.",
        ],
        Stage::Turning => &[
            ".............",
            "...l.l.l.l...",
            "..lolololol..",
            ".lolololololo",
            ".sososososos.",
        ],
        Stage::Ripe => &[
            "...*l*l*l*...",
            ".l*lylolyl*l.",
            "*lolololyolo*",
            "lylolylolylol",
            ".sososososos.",
        ],
        Stage::Over => &[
            ".............",
            ".............",
            "...#ososo#...",
            "..#s#oso#s#..",
            ".#s#.#s#.#s#.",
        ],
    };
    paint(s, (0, 1), head, &inks);
    paint(
        s,
        (0, 6),
        &[
            "..k.k.k.k.k..",
            "...k.k.k.k...",
            "....kkkkk....",
            "......k......",
            "......k......",
            "......k......",
        ],
        &inks,
    );
    if stage == Stage::Over {
        put(s, 2, 10, ELDER_OVER.light);
        put(s, 10, 8, ELDER_OVER.base);
    }
}

/// A crab apple on its stalk with a leaf: small and green, then yellowing, then gold with a red
/// blush on its sunny side; over, brown and bruised.
fn crab_apple(s: &mut Canvas, stage: Stage) {
    let (ramp, size, shiny) = match stage {
        Stage::Unripe => (APPLE_GREEN, 0.84, false),
        Stage::Turning => (APPLE_YELLOW, 0.92, false),
        Stage::Ripe => (APPLE_GOLD, 1.0, true),
        Stage::Over => (APPLE_OVER, 0.86, false),
    };
    let leaf = if stage == Stage::Over {
        DEAD_LEAF
    } else {
        LEAF
    };
    let inks = Inks::new(ramp).leaf(leaf);
    paint(
        s,
        (0, 0),
        &["......w.dd...", "......wdGgd..", ".......wvd..."],
        &inks,
    );
    fruit(s, (6.5, 6.8), (3.6 * size, 3.4 * size), ramp, shiny);
    let blush: &[(i32, i32)] = match stage {
        Stage::Ripe => &[(4, 6), (4, 7), (5, 8), (4, 8), (5, 7), (3, 7), (5, 6)],
        Stage::Turning => &[(4, 7), (4, 8)],
        _ => &[],
    };
    for &(x, y) in blush {
        let here = s.get(x, y);
        if here.a == 255 && here != ramp.shine && here != ramp.edge {
            s.set(x, y, if y < 7 { BLUSH.light } else { BLUSH.base });
        }
    }
    if stage == Stage::Over {
        paint(s, (7, 7), &["#s", "s."], &inks);
    }
    put(s, 6, 3, ramp.shadow);
}

/// A pair of rose hips, upright ovals with dried sepals at their feet: green, then orange, then
/// glossy scarlet; over, dark and wrinkled.
fn rose_hip(s: &mut Canvas, stage: Stage) {
    let (ramp, size, shiny) = match stage {
        Stage::Unripe => (HIP_GREEN, 0.82, false),
        Stage::Turning => (HIP_ORANGE, 0.92, false),
        Stage::Ripe => (HIP_SCARLET, 1.0, true),
        Stage::Over => (HIP_OVER, 0.84, false),
    };
    let extra = [('c', rgb(0x2a1a14))];
    let inks = Inks::new(ramp).extra(&extra);
    paint(
        s,
        (0, 0),
        &["......w......", "......ww.....", ".......w....."],
        &inks,
    );
    // The one behind, a little to the right and darker.
    fruit(s, (8.6, 6.4), (2.1 * size, 3.0 * size), darker(ramp), false);
    fruit(s, (5.6, 6.6), (2.7 * size, 3.8 * size), ramp, shiny);
    let foot = (6.6 + 3.8 * size).round() as i32;
    paint(s, (4, foot - 1), &["c.c", ".c."], &inks);
    if stage == Stage::Over {
        put(s, 5, 6, HIP_OVER.edge);
        put(s, 6, 8, HIP_OVER.edge);
    }
}

/// Wild thyme in a low cushion: green, then pink buds, then covered in purple flowers; over, gone
/// to brown seed heads.
fn thyme(s: &mut Canvas, stage: Stage) {
    let cushion = if stage == Stage::Over {
        DEAD_LEAF
    } else {
        THYME
    };
    let flower = match stage {
        Stage::Unripe => THYME,
        Stage::Turning => THYME_BUD,
        Stage::Ripe => THYME_FLOWER,
        Stage::Over => THYME_OVER,
    };
    let inks = Inks::new(flower).leaf(cushion);
    paint(
        s,
        (0, 7),
        &[
            "....dgGd.....",
            "..dGgGggGd...",
            ".dGgGggGggGd.",
            "dGgggggggvgvd",
            ".dvdvdvdvdvd.",
        ],
        &inks,
    );
    let flowers: &[&str] = match stage {
        Stage::Unripe => &[],
        Stage::Turning => &[
            ".............",
            ".............",
            "....l..o.....",
            "..o..l...l...",
            ".l...o.l..o..",
        ],
        Stage::Ripe => &[
            ".............",
            ".....*.......",
            "...*lol.*....",
            "..lolsol*ol..",
            ".lolsolsolol.",
        ],
        Stage::Over => &[
            ".............",
            ".............",
            "....s..s.....",
            "...s.o..s.o..",
            "..o....s.....",
        ],
    };
    paint(s, (0, 4), flowers, &inks);
}

/// A chanterelle: a pale knob, then a little pale funnel, then a golden trumpet with a wavy rim;
/// over, soggy and brown.
fn chanterelle(s: &mut Canvas, stage: Stage) {
    let ramp = match stage {
        Stage::Unripe | Stage::Turning => CHANT_PALE,
        Stage::Ripe => CHANT_GOLD,
        Stage::Over => CHANT_OVER,
    };
    let inks = Inks::new(ramp);
    let lines: &[&str] = match stage {
        Stage::Unripe => &[
            ".....##......",
            "....#lo#.....",
            "....#oo#.....",
            ".....ss......",
        ],
        Stage::Turning => &[
            "....#####....",
            "....#llloo#..",
            ".....#lo#....",
            ".....#oo#....",
            ".....#os#....",
            "......s......",
        ],
        Stage::Ripe => &[
            "..#.##.##.#..",
            ".#*l*llloo#..",
            ".#lllloooos#.",
            "..#sllloss#..",
            "...#llos#....",
            "....#los#....",
            "....#lo#.....",
            ".....ls......",
        ],
        Stage::Over => &[
            ".##.....##...",
            "#oo#####oo#..",
            ".#soooooss#..",
            "..#soos##....",
            "...#os#......",
            "....ss.......",
        ],
    };
    paint(s, (0, 12 - lines.len() as i32), lines, &inks);
}

/// A whorl of honeysuckle at the end of its twining stem, a pair of leaves under it: a tight crown
/// of green buds, then long pink buds, then the trumpets open, cream and gold from pink throats,
/// their lips curled back; over, browned and drooping.
fn honeysuckle(s: &mut Canvas, stage: Stage) {
    let ramp = match stage {
        Stage::Unripe => SUCKLE_BUD,
        Stage::Turning => SUCKLE_PINK,
        Stage::Ripe => SUCKLE_CREAM,
        Stage::Over => SUCKLE_OVER,
    };
    let extra = [('p', SUCKLE_PINK.base), ('P', SUCKLE_PINK.shadow)];
    let inks = Inks::new(ramp).extra(&extra);
    let whorl: &[&str] = match stage {
        Stage::Unripe => &[
            ".............",
            ".............",
            ".............",
            "....#.#.#....",
            "...#l#l#o#...",
            "...#lolos#...",
            "....#oss#....",
        ],
        Stage::Turning => &[
            ".............",
            "...l...l.....",
            "....o..o..l..",
            "..l..o.o.o...",
            "...o.ooo.o...",
            "....oooooo...",
            ".....sPs.....",
        ],
        Stage::Ripe => &[
            ".*l.....l*...",
            "..lo..*.ol...",
            "....o.lpo..*l",
            "*l..po.op.ol.",
            ".lopp.pp.p...",
            "....pppppP...",
            ".....PpP.....",
        ],
        Stage::Over => &[
            ".............",
            ".............",
            ".............",
            "....#s#s#....",
            "...#ososo#...",
            "..#o#sos#o#..",
            "..#s#.#.#s#..",
        ],
    };
    paint(s, (0, 0), whorl, &inks);
    paint(
        s,
        (0, 7),
        &[
            "...dGgkgGd...",
            "....dg.gd....",
            "......k......",
            ".....k.......",
        ],
        &inks,
    );
}

/// A clover leaf. Three leaflets folded, then three open with a fourth still curled, then all
/// four open, bright, with their pale chevrons; over, wilted and yellowing.
fn clover(s: &mut Canvas, stage: Stage) {
    let ramp = match stage {
        Stage::Unripe | Stage::Turning => CLOVER_DULL,
        Stage::Ripe => CLOVER,
        Stage::Over => CLOVER_OVER,
    };
    let extra = [('c', CHEVRON)];
    let inks = Inks::new(ramp).extra(&extra);
    let lines: &[&str] = match stage {
        Stage::Unripe => &[
            ".............",
            ".............",
            ".............",
            ".....#o#.....",
            "....#ol#.#...",
            ".....#o#ol#..",
            "......#k#....",
            "......k......",
            "......k......",
        ],
        Stage::Turning => &[
            ".............",
            "....##.##....",
            "...#lo#lo#...",
            "...#oo#os#...",
            "....#s#s#....",
            "...##.k......",
            "..#lo#k.#o#..",
            "...##.k.##...",
            "......k......",
        ],
        Stage::Ripe => &[
            "..##.....##..",
            ".#*l#...#lo#.",
            ".#lcl#.#lco#.",
            "..#ll#k#os#..",
            "...##.k.##...",
            "..#ll#k#os#..",
            ".#lco#.#sco#.",
            ".#lo#...#so#.",
            "..##..k..##..",
        ],
        Stage::Over => &[
            ".............",
            ".............",
            ".............",
            ".....#.......",
            "....#o#.##...",
            "....#s##os#..",
            "...##s.k##...",
            "..#os#.k.....",
            "...#....k....",
        ],
    };
    paint(s, (0, 2), lines, &inks);
    if stage != Stage::Over {
        put(s, 6, 11, STALK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque(canvas: &Canvas) -> usize {
        canvas.pixels().iter().filter(|pixel| pixel.a > 0).count()
    }

    #[test]
    fn every_stage_of_everything_is_drawn_and_each_looks_different() {
        for plant in Plant::ALL {
            let looks: Vec<Canvas> = Stage::ALL
                .iter()
                .map(|stage| sprite(plant, *stage, false))
                .collect();
            for (stage, look) in Stage::ALL.iter().zip(&looks) {
                assert!(opaque(look) >= 6, "{plant:?} {stage:?} is barely there");
            }
            for a in 0..looks.len() {
                for b in a + 1..looks.len() {
                    assert_ne!(
                        looks[a],
                        looks[b],
                        "{plant:?} looks the same {:?} and {:?}",
                        Stage::ALL[a],
                        Stage::ALL[b]
                    );
                }
            }
        }
    }

    #[test]
    fn a_ripe_thing_shines_and_one_gone_over_is_dull() {
        // The brightest point on it: the shine on a ripe thing, which nothing over has.
        let shine = |canvas: &Canvas| {
            canvas
                .pixels()
                .iter()
                .filter(|pixel| pixel.a == 255)
                .map(|pixel| u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b))
                .max()
                .unwrap_or(0)
        };
        for plant in Plant::ALL {
            let ripe = shine(&sprite(plant, Stage::Ripe, false));
            let over = shine(&sprite(plant, Stage::Over, false));
            assert!(
                ripe >= over + 90,
                "{plant:?} over ({over}) is nearly as bright as ripe ({ripe})"
            );
        }
    }

    #[test]
    fn with_reduced_motion_a_ripe_thing_keeps_its_glint_still() {
        let slot = Slot {
            plant: Plant::Bramble,
            at: (40.0, 100.0),
            reach: super::super::Reach::Within,
            hidden: false,
        };
        let still: Vec<Canvas> = (0..40)
            .map(|step| prop(&slot, 140.0, Stage::Ripe, step as f32 * 0.1, true).sprite)
            .collect();
        assert!(still.windows(2).all(|pair| pair[0] == pair[1]));
        let moving: Vec<Canvas> = (0..40)
            .map(|step| prop(&slot, 140.0, Stage::Ripe, step as f32 * 0.1, false).sprite)
            .collect();
        assert!(
            moving.windows(2).any(|pair| pair[0] != pair[1]),
            "it never glints"
        );
    }
}
