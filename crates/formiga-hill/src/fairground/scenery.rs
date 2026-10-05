//! The Fairground on a sunny afternoon, at the level of a grown-up's eye: a summer sky with the
//! Hill behind, a big top, a carousel and a hoopla stall along the back under strings of bulbs,
//! and a sawdust ground scattered with things to hide behind.
//!
//! Like every place it is painted by day, and the hour's light is laid over it (`daylight`). What
//! shines of its own after dark is painted separately, in `lamplight`: every bulb lit with its
//! halo, the carousel and the stall lit from within, the big top's door, the caravan's window and
//! the lantern on the barrel, and their light lying in pools on the sawdust. Both are painted by
//! the same brushwork in one pass (`paint`), so a light is always where its bulb is.
//!
//! The hiding places are props, drawn among the colony in depth order, so whoever stands behind a
//! hay bale is hidden by it, ears and all if they crouch low enough.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::font::{GLYPH_HEIGHT, draw_text_shadowed, text_width};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::kit::window;
use crate::materials::{
    CLOUD, CREAM, FAR_HILL, GLASS, GLOW, HILL, HILL_SHADE, LEAVES, PLANK, SKY_LOW, SKY_TOP, TRIM,
    TRUNK,
};
use crate::paint::{
    Ramp, bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline,
};
use crate::playground::Prop;
use formiga_art::{Canvas, Rgba};

pub const HORIZON: i32 = 66;
/// The big top's left edge and the row it stands on.
const BIG_TOP: (i32, i32) = (12, 112);
/// Where the colony can stand: the sawdust in front of the attractions.
pub const GROUND: (f32, f32, f32, f32) = (16.0, 122.0, 368.0, 204.0);

const SAWDUST: Ramp = Ramp::new(0x6b5034, 0x8f6c48, 0xb08a5c, 0xc8a274, 0xdcbb8c);
const STRAW: Ramp = Ramp::new(0x8a6a28, 0xb8903a, 0xd8b04e, 0xe8c86a, 0xf4e09a);
/// Hay is greener than straw, and a little duller.
const HAY: Ramp = Ramp::new(0x6a5c26, 0x96883c, 0xbcac54, 0xd4c670, 0xeae09c);
const TWINE: Ramp = Ramp::new(0x4a2418, 0x6e3622, 0x8e4a2e, 0xa8643e, 0xc0805a);
/// Grass at the edges of the field, where nobody walks.
const GRASS: Ramp = Ramp::new(0x2f5a2e, 0x447a3c, 0x5e9450, 0x7db36c, 0x9fd08a);
const CANVAS_RED: Ramp = Ramp::new(0x5e1a24, 0x8a2a30, 0xb83a3c, 0xd0584c, 0xe87a64);
const CANVAS_CREAM: Ramp = Ramp::new(0x8a7a6a, 0xc8b8a0, 0xe8dcc4, 0xf4ecd8, 0xffffff);
const TEAL: Ramp = Ramp::new(0x173c40, 0x22585c, 0x2f7a7a, 0x45999a, 0x6ab8b4);
const AWNING_BLUE: Ramp = Ramp::new(0x1e2c58, 0x2c4480, 0x3a5ea8, 0x5a7ec4, 0x86a6dc);
const GOLD: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const CARAVAN_RED: Ramp = Ramp::new(0x4a1e2a, 0x6a2a3a, 0x8a3a4a, 0xa85060, 0xc87080);
/// The boards at the back of the stall.
const BOARDS: Ramp = Ramp::new(0x2e1c1e, 0x44282a, 0x5a3830, 0x6e4a3a, 0x86604a);
/// Pine for the cart, oak for the barrel, iron for the hoops round it and the tyres of the
/// wheels.
const PINE: Ramp = Ramp::new(0x5e4228, 0x8a6640, 0xae8656, 0xc8a26e, 0xe0c08e);
const OAK: Ramp = Ramp::new(0x3e2618, 0x5e3c26, 0x7e5434, 0x9c6e46, 0xb88a5e);
const IRON: Ramp = Ramp::new(0x241e20, 0x3e3836, 0x5a524c, 0x7c7268, 0xa49888);
const BURLAP: Ramp = Ramp::new(0x5a4428, 0x80653e, 0xa48456, 0xc0a06e, 0xd8bc8a);
const BEAR: Ramp = Ramp::new(0x5a3420, 0x8a5634, 0xb07a4a, 0xc8945e, 0xdcb07a);
const HONEY_BEAR: Ramp = Ramp::new(0x7a5a30, 0xb08a50, 0xd4b070, 0xe8cc94, 0xf8e8c0);
const PINK_BEAR: Ramp = Ramp::new(0x6a3a4a, 0xa05a70, 0xc87890, 0xe09aaa, 0xf4c4cc);
const COCONUT: Ramp = Ramp::new(0x2e1c12, 0x4a2e1c, 0x684228, 0x845838, 0xa0744e);
const BOTTLE: Ramp = Ramp::new(0x1e4a2a, 0x2a6a3a, 0x3e8a4e, 0x6ab878, 0xd8f4d8);
/// The mirrors round the middle of the carousel, full of the sky.
const MIRROR: Ramp = Ramp::new(0x3e4c58, 0x5e7280, 0x82a0ac, 0xb2ccd4, 0xf4fbfc);
/// The carousel's deck, its boards worn by feet.
const DECK: Ramp = Ramp::new(0x3e2420, 0x5e3628, 0x7e4c34, 0x9a6442, 0xb47e56);
/// The underside of the carousel's canopy, in its own shade: lighter just inside the valance,
/// darker in towards the middle.
const CEILING: Rgba = rgb(0x24363a);
const CEILING_NEAR: Rgba = rgb(0x5e7472);
/// The shade inside the carousel, beyond its middle, that the far horses are lost in.
const INSIDE: Rgba = rgb(0x2e2234);
/// The horses' coats, and their manes and tails.
const WHITE_COAT: Ramp = Ramp::new(0x6e6478, 0xb4aab8, 0xe2dade, 0xf6f2ee, 0xffffff);
const DAPPLE_COAT: Ramp = Ramp::new(0x4a4656, 0x7a7688, 0xa4a0b0, 0xc8c4d0, 0xe8e6ee);
const CHESTNUT_COAT: Ramp = Ramp::new(0x4a2418, 0x7a3e24, 0xa45a34, 0xc47a4a, 0xdc9a68);
const CREAM_COAT: Ramp = Ramp::new(0x6e5430, 0xb8964e, 0xdcbc74, 0xf0d898, 0xfff0c8);
const DARK_MANE: Ramp = Ramp::new(0x2a2430, 0x3e3646, 0x564c5e, 0x6e6476, 0x8a8090);
const FLAXEN_MANE: Ramp = Ramp::new(0x8a7048, 0xc8aa70, 0xe8d098, 0xf6e6b8, 0xfff8e0);
/// The colour lamplight lends whatever it falls on after dark.
const LAMP: Rgba = rgb(0xffc070);
/// Bulbs: the hot core of each, and the colour of its glass.
const BULBS: [(Rgba, Rgba); 3] = [
    (rgb(0xfffbe6), rgb(0xffd88a)),
    (rgb(0xfff2c4), rgb(0xffaa48)),
    (rgb(0xffeedc), rgb(0xff8064)),
];
const WIRE: Rgba = rgb(0x2a1e28);
const ROPE: Rgba = rgb(0x7a6a64);
const INK: Rgba = rgb(0x2a1a22);
const SHADOW: Rgba = rgba(0x2a1e28, 80);
/// The Hill on the skyline, its tree on top, and whatever stands there now, dark against the
/// bright sky behind it.
const VISTA: Vista = Vista {
    tree: (299.0, (HORIZON - 28) as f32),
    crest: |x| (HORIZON + 6) as f32 - 34.0 * (1.0 - ((x - 300.0) / 90.0).powi(2)).max(0.0).sqrt(),
    spread: 8.0,
    shrink: 7,
    tint: Tint::Silhouette(rgb(0x52705c)),
};
/// The barrel's left edge and top, and where the lantern standing on it has its top-left, from
/// there.
const BARREL: (i32, i32) = (118, 134);
const BARREL_LANTERN: (i32, i32) = (4, -5);
/// Where the other props stand, each by its sprite's top-left: the hay and the straw, and the
/// handcart, in the front corners, and the prize sack by the hoopla stall, so the front of the
/// field is clear sawdust for the sack race and the tug-of-war, the ground before the big top is
/// clear for the striker's queue, and the ground beside the stall for whoever is throwing.
const HAY_STACK: (i32, i32) = (18, 176);
const STRAW_BALE: (i32, i32) = (58, 188);
const HANDCART: (i32, i32) = (338, 176);
const PRIZE_SACK: (i32, i32) = (342, 100);
/// The hoopla counter's left edge, and the row its top is on.
pub const COUNTER: (i32, i32) = (278, 104);
/// The high striker, between the big top and the carousel: the middle of its tower, and the row
/// its plinth stands on.
pub const STRIKER: (i32, i32) = (129, 117);
/// The rows the striker's puck runs between up its rail: where its top is at rest, and where it
/// is when it strikes the bell.
pub const PUCK_RAIL: (i32, i32) = (104, 61);
/// The pad the mallet comes down on: its middle, and its top row.
pub const PAD: (i32, i32) = (129, 117);
/// The bell on top of the striker: its middle, and the row of its rim.
pub const BELL: (i32, i32) = (129, 55);

/// Everything behind the colony and its hiding places, by day.
pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut lamps = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    paint(&mut scene, &mut lamps, hilltop);
    scene
}

/// What shines here after dark, at full strength, on a clear canvas: every bulb lit in its
/// colour with its halo, the big top's door, the carousel and the stall lit from within, the
/// caravan's window and the lanterns, and the pools of light they throw on the sawdust. Nothing
/// that isn't a light is painted.
pub fn lamplight() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut lamps = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    paint(&mut scene, &mut lamps, &Arrangement::new());
    lamps
}

/// The sky after dark, deepened, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, HORIZON, Some((150, 14)))
}

/// The place by day into `scene`, and whatever shines in it after dark into `lamps`, by the same
/// brushwork: so a light is always exactly where its bulb is painted.
fn paint(scene: &mut Canvas, lamps: &mut Canvas, hilltop: &Arrangement) {
    sky(scene);
    hills(scene);
    skyline(scene, hilltop, &VISTA);
    ground(scene, lamps);
    big_top(scene, lamps, BIG_TOP.0, BIG_TOP.1);
    carousel(scene, lamps, 196);
    hoopla_stall(scene, lamps, 266);
    caravan(scene, lamps, 336);
    festoons(scene, lamps);
    high_striker(scene, lamps);
    near_strings(scene, lamps);
    lantern_light(
        lamps,
        BARREL.0 + BARREL_LANTERN.0,
        BARREL.1 + BARREL_LANTERN.1,
    );
}

/// The grass along the very front, nearest the eye, drawn over everyone.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for x in (0..width).step_by(2) {
        if chance(x, 0, 601, 150) {
            let spread = 1 + pick(x, 1, 602, 2);
            tuft(&mut scene, x, height - 1, spread, 3, 603);
        }
    }
    scene
}

/// Strings of bunting and bulbs nearest the eye, hung across the top corners of the picture.
/// Nobody ever stands so high, so they are painted with the backdrop, where their bulbs can be
/// lit after dark.
const NEAR_STRINGS: [Span; 2] = [((-8, 8), (46, -6), 11.0), ((318, -6), (392, 8), 12.0)];

fn near_strings(scene: &mut Canvas, lamps: &mut Canvas) {
    for (string, &(from, to, sag)) in NEAR_STRINGS.iter().enumerate() {
        let at = |t: f32| {
            (
                from.0 as f32 + (to.0 - from.0) as f32 * t,
                from.1 as f32 + (to.1 - from.1) as f32 * t + sag * 4.0 * t * (1.0 - t),
            )
        };
        let count = ((to.0 - from.0) as f32 / 13.0).round().max(3.0) as usize;
        let mut previous = at(0.0);
        for step in 1..=80 {
            let point = at(step as f32 / 80.0);
            line(
                scene,
                (previous.0.round() as i32, previous.1.round() as i32),
                (point.0.round() as i32, point.1.round() as i32),
                WIRE,
            );
            previous = point;
        }
        for index in 0..count {
            let (x, y) = at((index as f32 + 0.5) / count as f32);
            let flag = [CANVAS_RED, GOLD, TEAL, CANVAS_CREAM][(index + string) % 4];
            stamp(
                scene,
                (x.round() as i32 - 3, y.round() as i32 + 1),
                &["#####", "#loo#", "#los#", ".#s#.", ".#s#.", "..#.."],
                &[
                    (b'#', flag.edge),
                    (b'l', flag.light),
                    (b'o', flag.base),
                    (b's', flag.shadow),
                ],
                false,
            );
        }
        for index in 1..count {
            let (x, y) = at(index as f32 / count as f32);
            let (x, y) = (x.round() as i32, y.round() as i32 + 1);
            let (core, glass) = BULBS[(index + string) % BULBS.len()];
            // By day, glass catching the sun.
            hline(scene, x, y, 2, IRON.base);
            hline(scene, x, y + 1, 2, mix(glass, rgb(0xffffff), 0.6));
            put(scene, x, y + 2, mix(glass, rgb(0xffffff), 0.3));
            put(scene, x + 1, y + 2, glass);
            hline(scene, x, y + 3, 2, mix(glass, INK, 0.3));
            // After dark, lit, and its light on the air about it.
            let middle = (x as f32 + 1.0, y as f32 + 3.0);
            halo(lamps, middle, (12.0, 12.0), LAMP, 0.3);
            halo(lamps, middle, (4.5, 4.5), core, 0.5);
            hline(lamps, x, y + 1, 2, core);
            put(lamps, x, y + 2, core);
            put(lamps, x + 1, y + 2, glass);
            put(lamps, x, y + 3, glass);
            put(lamps, x + 1, y + 3, mix(glass, INK, 0.2));
        }
    }
}

/// A place to hide: the prop's index among the props, where the hider stands, and how much of
/// a creature it can cover.
#[derive(Clone, Copy, Debug)]
pub struct HidingPlace {
    pub name: &'static str,
    /// Where a hider's feet go, just behind the prop.
    pub behind: (f32, f32),
    /// Where they come out to when found, just in front of it.
    pub out_front: (f32, f32),
    /// How tall the prop stands, in pixels: how much of someone it hides.
    pub cover: f32,
}

/// The props, and the hiding place each one makes, in the same order.
pub fn props() -> (Vec<Prop>, Vec<HidingPlace>) {
    let mut props = Vec::new();
    let mut places = Vec::new();
    let mut add = |name, (sprite, at, base): (Canvas, (i32, i32), f32), cover: f32| {
        let width = sprite.width() as f32;
        let centre = at.0 as f32 + width / 2.0;
        places.push(HidingPlace {
            name,
            behind: (centre, base - 3.0),
            out_front: (centre, (base + 12.0).min(GROUND.3 + 2.0)),
            cover,
        });
        props.push(Prop::new(sprite, at, base));
    };
    add("the tent door", tent_door(), 40.0);
    add("the hay bales", hay_stack(HAY_STACK.0, HAY_STACK.1), 22.0);
    add("the barrel", barrel(BARREL.0, BARREL.1), 20.0);
    add("the prize sack", sack(PRIZE_SACK.0, PRIZE_SACK.1), 18.0);
    add("the handcart", handcart(HANDCART.0, HANDCART.1), 16.0);
    add("the hoopla counter", counter(COUNTER.0, COUNTER.1), 22.0);
    add(
        "the straw bale",
        straw_bale(STRAW_BALE.0, STRAW_BALE.1),
        14.0,
    );
    (props, places)
}

// ---------------------------------------------------------------------------------------------
// Light
// ---------------------------------------------------------------------------------------------

/// How much of a light reaches `(x, y)`, from 0 to 1: falling away from the middle of an
/// ellipse in four steps, dithered where one step meets the next, as light lies in pixel art.
fn falloff(x: i32, y: i32, (cx, cy): (f32, f32), (rx, ry): (f32, f32)) -> f32 {
    let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
    let reach = (u * u + v * v).sqrt();
    if reach >= 1.0 {
        return 0.0;
    }
    let jitter = (noise(x, y, 641) & 0xff) as f32 / 255.0 - 0.5;
    ((1.0 - reach) * 4.0 + jitter * 0.8).ceil().clamp(0.0, 4.0) / 4.0
}

/// Light about `middle` reaching `size` either way, as `color` laid over the dark: as strong as
/// `strength` in the middle and falling away in soft steps.
fn halo(lamps: &mut Canvas, middle: (f32, f32), size: (f32, f32), color: Rgba, strength: f32) {
    let (cx, cy) = middle;
    let (rx, ry) = size;
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let amount = falloff(x, y, middle, size) * strength;
            if amount > 0.0 {
                let alpha = (amount * 255.0).min(255.0) as u8;
                put(lamps, x, y, Rgba { a: alpha, ..color });
            }
        }
    }
}

/// A bulb hung from a string, `hue` choosing its glass. By day, only the glass catching the sun;
/// after dark, its hot core over its coloured glass, a bright halo close about it and a wide faint
/// one beyond.
fn bulb(scene: &mut Canvas, lamps: &mut Canvas, x: i32, y: i32, hue: usize) {
    let (core, glass) = BULBS[hue % BULBS.len()];
    put(scene, x, y, mix(glass, rgb(0xffffff), 0.6));
    put(scene, x, y + 1, mix(glass, INK, 0.25));
    let middle = (x as f32 + 0.5, y as f32 + 1.0);
    halo(lamps, middle, (9.0, 8.0), glass, 0.32);
    halo(lamps, middle, (3.5, 3.5), core, 0.55);
    put(lamps, x, y, core);
    put(lamps, x, y + 1, glass);
}

/// A small bulb set into a sign or a board, with only a little halo after dark.
fn stud(scene: &mut Canvas, lamps: &mut Canvas, x: i32, y: i32, hue: usize) {
    let (core, glass) = BULBS[hue % BULBS.len()];
    put(scene, x, y, mix(glass, rgb(0xffffff), 0.55));
    halo(
        lamps,
        (x as f32 + 0.5, y as f32 + 0.5),
        (4.0, 4.0),
        glass,
        0.45,
    );
    put(lamps, x, y, core);
}

// ---------------------------------------------------------------------------------------------
// Sky and ground
// ---------------------------------------------------------------------------------------------

/// The sky on its own, and its clouds: painted first, and again by itself, so the sky can be
/// told from everything in front of it.
fn sky(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // In bands rather than a smooth ramp, as pixel skies are.
    for y in 0..HORIZON {
        let band = mix(SKY_TOP, SKY_LOW, (y * 6 / HORIZON) as f32 / 5.0);
        scene.fill_rect(0, y, width, 1, band);
    }
    for (middle, size) in [((104, 16), 1.0), ((238, 8), 1.3), ((356, 30), 0.8)] {
        cloud(scene, middle, size);
    }
}

/// A fair-weather cloud, its top lit, greying a little underneath.
fn cloud(scene: &mut Canvas, (cx, cy): (i32, i32), size: f32) {
    let scaled = |n: i32| (n as f32 * size).round() as i32;
    let under = mix(CLOUD, SKY_TOP, 0.45);
    for &(dx, dy, rx, ry) in &[(0, 1, 9, 3), (-5, 0, 5, 3), (3, -1, 6, 3), (9, 1, 4, 2)] {
        ellipse(scene, cx + scaled(dx), cy + dy + 1, scaled(rx), ry, under);
    }
    for &(dx, dy, rx, ry) in &[(0, 0, 8, 2), (-5, -1, 4, 2), (3, -2, 5, 3), (9, 0, 3, 1)] {
        ellipse(scene, cx + scaled(dx), cy + dy, scaled(rx), ry, CLOUD);
    }
}

/// The downs far off, pale in the haze, and the Hill nearer at the right with its tree on top, lit
/// on its left flank and shaded on its right.
fn hills(scene: &mut Canvas) {
    let haze = |color: Rgba, amount: f32| mix(color, SKY_LOW, amount);
    let width = SCENE_WIDTH as i32;
    for x in 0..width {
        // The downs, a long low ridge.
        let ridge = (HORIZON as f32 - 9.0 + (x as f32 * 0.03).sin() * 3.0 + (x as f32 * 0.11).sin())
            .round() as i32;
        for y in ridge..HORIZON {
            let color = if y == ridge {
                mix(FAR_HILL, HILL_SHADE, 0.4)
            } else if y < ridge + 2 {
                mix(FAR_HILL, rgb(0xffffff), 0.15)
            } else {
                FAR_HILL
            };
            scene.set(x, y, haze(color, 0.45));
        }
        // The Hill.
        let across = (x as f32 + 0.5 - 300.0) / 90.0;
        if across.abs() < 1.0 {
            let crest = (VISTA.crest)(x as f32).round() as i32;
            for y in crest..HORIZON {
                let color = if y == crest {
                    HILL_SHADE
                } else if across < -0.35 && y < crest + 6 {
                    mix(HILL, rgb(0xffffff), 0.12)
                } else if across > 0.4 || chance(x, y, 655, 24) {
                    HILL_SHADE
                } else {
                    HILL
                };
                scene.set(x, y, haze(color, 0.3));
            }
        }
    }
    // The Hill's old tree.
    let (tx, ty) = (VISTA.tree.0 as i32, VISTA.tree.1 as i32);
    rect(scene, tx - 1, ty - 6, 2, 7, haze(TRUNK, 0.3));
    ellipse(scene, tx, ty - 9, 5, 4, haze(mix(LEAVES, INK, 0.2), 0.3));
    ellipse(scene, tx - 1, ty - 10, 4, 3, haze(LEAVES, 0.3));
    ellipse(
        scene,
        tx - 2,
        ty - 11,
        2,
        1,
        haze(mix(LEAVES, rgb(0xffffff), 0.2), 0.3),
    );
}

/// A pool of light on the ground: its middle, how far it reaches either way, and how bright it
/// is at its brightest.
type Pool = ((f32, f32), (f32, f32), f32);

/// Where the lamps' light falls on the sawdust after dark: in front of the carousel, out of the
/// hoopla stall, from the big top's door and the caravan's window, about the lantern on the
/// barrel (and on the air round the lantern itself), and under the strings.
const POOLS: [Pool; 8] = [
    ((196.0, 118.0), (86.0, 13.0), 0.4),
    ((304.0, 122.0), (52.0, 10.0), 0.42),
    ((60.0, 117.0), (24.0, 6.0), 0.35),
    ((358.0, 116.0), (26.0, 6.0), 0.28),
    ((126.0, 153.0), (30.0, 9.0), 0.38),
    ((124.5, 133.0), (9.0, 9.0), 0.35),
    ((116.0, 124.0), (32.0, 8.0), 0.16),
    ((258.0, 128.0), (24.0, 7.0), 0.16),
];

/// The sawdust: paler far off in the haze, raked into soft clumps, trodden flatter where everyone
/// walks, with prints, straw and dropped tickets in it and grass at its edges; and after dark,
/// the lamps' light lying on it.
fn ground(scene: &mut Canvas, lamps: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for y in HORIZON..height {
        let near = nearness(y);
        let size = 1 + (near * 3.0) as i32;
        let base = mix(mix(SAWDUST.light, SKY_LOW, 0.15), SAWDUST.base, near.sqrt());
        for x in 0..width {
            // Soft clumps, bigger nearer the eye, rather than a speckle.
            let mut color = match noise(x / (size * 2), y / size, 621) % 11 {
                0 => mix(base, SAWDUST.shadow, 0.3),
                1 => mix(base, SAWDUST.light, 0.2),
                _ => base,
            };
            let wear = patches(x, y, (30, 8), 627);
            if wear > 0.66 {
                color = mix(color, SAWDUST.edge, 0.14);
            } else if wear < 0.3 {
                color = mix(color, SAWDUST.light, 0.1);
            }
            scene.set(x, y, color);
        }
    }
    shavings(scene);
    let clear = kept_clear();
    footprints(scene);
    straw(scene, &clear);
    edge_grass(scene);
    tickets(scene);
    for &(middle, size, strength) in &POOLS {
        halo(lamps, middle, size, LAMP, strength);
    }
}

/// How near the eye a row of the ground is: 0 at the horizon, 1 at the foot of the picture.
fn nearness(y: i32) -> f32 {
    ((y - HORIZON) as f32 / (SCENE_HEIGHT as i32 - HORIZON) as f32).clamp(0.0, 1.0)
}

/// Flakes of sawdust lying on the rest, each lit along its top with a crumb of shade under it,
/// bigger and further apart nearer the eye.
fn shavings(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    let mut row = HORIZON + 2;
    while row < height {
        let near = nearness(row);
        let (step_x, step_y) = (4 + (near * 4.0) as i32, 2 + (near * 2.0) as i32);
        for (index, gx) in (0..width).step_by(step_x as usize).enumerate() {
            let index = index as i32;
            let x = gx + pick(index, row, 633, step_x);
            let y = row + pick(index, row + 1, 633, step_y);
            if !chance(x, y, 634, 60) {
                continue;
            }
            let long = 1 + (near * 1.6) as i32 + pick(index, row + 2, 633, 2);
            let tone = match pick(index, row + 3, 633, 5) {
                0 => SAWDUST.shine,
                1 | 2 => SAWDUST.light,
                3 => SAWDUST.base,
                _ => SAWDUST.shadow,
            };
            hline(scene, x, y, long, tone);
            if long > 1 {
                hline(scene, x, y + 1, long, rgba(0x2e2024, 34));
            }
        }
        row += step_y;
    }
}

/// Where someone stands, and so where nothing is dropped: the fairground's spots, and either side
/// of each hiding place.
fn kept_clear() -> Vec<(f32, f32)> {
    let (_, places) = props();
    super::SPOTS
        .iter()
        .map(|&(_, at)| at)
        .chain(
            places
                .iter()
                .flat_map(|place| [place.behind, place.out_front]),
        )
        .collect()
}

fn by_a_spot(clear: &[(f32, f32)], x: i32, y: i32) -> bool {
    clear
        .iter()
        .any(|&(sx, sy)| (sx - x as f32).abs() < 12.0 && (sy - y as f32).abs() < 6.0)
}

/// A curve from where it starts, bent towards a point, to where it ends.
type Bend = ((f32, f32), (f32, f32), (f32, f32));

/// Trails of prints in the sawdust: in from the lane at the front right, out of the big top, along
/// the front of the rides, and round past the bales.
const TRAILS: [Bend; 4] = [
    ((396.0, 202.0), (300.0, 176.0), (236.0, 124.0)),
    ((62.0, 116.0), (84.0, 168.0), (150.0, 220.0)),
    ((120.0, 128.0), (200.0, 142.0), (296.0, 126.0)),
    ((4.0, 196.0), (70.0, 214.0), (166.0, 176.0)),
];

fn footprints(scene: &mut Canvas) {
    for &(from, via, to) in &TRAILS {
        let at = |t: f32| {
            let u = 1.0 - t;
            (
                u * u * from.0 + 2.0 * u * t * via.0 + t * t * to.0,
                u * u * from.1 + 2.0 * u * t * via.1 + t * t * to.1,
            )
        };
        let (mut t, mut side) = (0.0, 1.0);
        while t < 1.0 {
            let (x, y) = at(t);
            let ahead = at(t + 0.01);
            let (dx, dy) = (ahead.0 - x, ahead.1 - y);
            let step = (dx * dx + dy * dy).sqrt().max(0.01);
            let near = nearness(y.round() as i32);
            // Left foot, right foot, either side of the way.
            let (px, py) = (x - dy / step * side, y + dx / step * side);
            footprint(scene, px.round() as i32, py.round() as i32, near);
            side = -side;
            t += (4.0 + near * 4.0) / (step * 100.0);
        }
    }
}

/// One print, pressed into the sawdust: a dent, and the sawdust pushed up and catching the light
/// along its near edge. Bigger nearer the eye.
fn footprint(scene: &mut Canvas, x: i32, y: i32, near: f32) {
    let dent = rgba(0x2e2024, 64);
    let rim = rgba(0xf4d4a4, 50);
    if near < 0.45 {
        hline(scene, x, y, 2, dent);
        hline(scene, x, y + 1, 2, rim);
    } else {
        hline(scene, x, y, 3, dent);
        hline(scene, x, y + 1, 3, dent);
        put(scene, x + 1, y + 1, dent);
        hline(scene, x, y + 2, 3, rim);
    }
}

/// Wisps of straw everywhere, and more round the feet of the bales.
fn straw(scene: &mut Canvas, clear: &[(f32, f32)]) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    let mut wisps: Vec<(i32, i32)> = (0..90)
        .map(|index| {
            (
                pick(index, 0, 623, width),
                HORIZON + 12 + pick(index, 1, 624, height - HORIZON - 12),
            )
        })
        .collect();
    for (heap, (x, y, spread)) in [
        (HAY_STACK.0, HAY_STACK.1 + 18, 40),
        (STRAW_BALE.0 - 2, STRAW_BALE.1 + 10, 36),
        (118, 152, 22),
    ]
    .into_iter()
    .enumerate()
    {
        for index in 0..26 {
            wisps.push((
                x + pick(index, heap as i32, 630, spread),
                y + pick(index, heap as i32 + 7, 631, 9),
            ));
        }
    }
    for (index, &(x, y)) in wisps.iter().enumerate() {
        if by_a_spot(clear, x, y) {
            continue;
        }
        let index = index as i32;
        let long = 2 + pick(index, 2, 625, 3);
        let lean = if pick(index, 3, 626, 2) == 0 { 1 } else { -1 };
        let dusty = mix(STRAW.base, SAWDUST.base, 0.45 - nearness(y) * 0.2);
        line(scene, (x, y), (x + long, y + lean), dusty);
        put(scene, x, y, mix(STRAW.light, SAWDUST.light, 0.3));
        put(scene, x + long, y + lean + 1, rgba(0x2e2024, 40));
    }
}

/// Grass in clumps along the edges of the field, thinning out where everyone walks.
fn edge_grass(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for y in (HORIZON + 3..height).step_by(3) {
        let near = nearness(y);
        for x in (0..width).step_by(2) {
            let edge = x.min(width - 1 - x) as f32;
            if edge < 26.0 && chance(x, y, 622, (90.0 - edge * 3.6).max(0.0) as u32) {
                let spread = 1 + (near * 2.0) as i32 - i32::from(edge > 12.0);
                tuft(scene, x, y, spread, 2 + (near * 3.0) as i32, 632);
            }
        }
    }
}

/// A tuft of grass blades about `(x, y)` on the ground, `spread` pixels either side and up to
/// `tall` high, lit at their tips.
fn tuft(scene: &mut Canvas, x: i32, y: i32, spread: i32, tall: i32, salt: u32) {
    for dx in -spread..=spread {
        if !chance(x + dx, y, salt, 180) {
            continue;
        }
        let high = 1 + pick(x + dx, y, salt + 1, tall) + (spread - dx.abs()) / 2;
        let lean = dx.signum() * pick(dx, y, salt + 2, 2);
        for step in 0..high {
            let (color, px) = if step == high - 1 {
                (GRASS.light, x + dx + lean)
            } else if step == 0 {
                (GRASS.shadow, x + dx)
            } else {
                (GRASS.base, x + dx)
            };
            put(scene, px, y - step, color);
        }
    }
}

/// Tickets dropped about the place: where each lies, whether it lies askew, and its colour.
const TICKETS: [(i32, i32, bool, usize); 6] = [
    (150, 172, false, 0),
    (232, 192, true, 1),
    (322, 170, false, 2),
    (78, 140, true, 0),
    (266, 136, false, 1),
    (134, 206, true, 2),
];

fn tickets(scene: &mut Canvas) {
    let papers = [CANVAS_RED, CANVAS_CREAM, AWNING_BLUE];
    let under = rgba(0x2a1e28, 64);
    for &(x, y, askew, paper) in &TICKETS {
        let paper = papers[paper];
        if askew {
            hline(scene, x + 1, y, 2, paper.light);
            put(scene, x + 3, y, paper.base);
            hline(scene, x, y + 1, 2, paper.base);
            put(scene, x + 2, y + 1, paper.shadow);
            hline(scene, x, y + 2, 2, under);
            put(scene, x + 2, y + 2, under);
        } else {
            hline(scene, x, y, 3, paper.light);
            hline(scene, x, y + 1, 3, paper.base);
            // The stub, torn off along its perforation.
            put(scene, x + 3, y, paper.shadow);
            put(scene, x + 3, y + 1, paper.edge);
            hline(scene, x + 1, y + 2, 4, under);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The attractions along the back
// ---------------------------------------------------------------------------------------------

/// A cone of striped canvas from its point at `peak` down to `eave`, `reach` either side of the
/// middle there, `stripes` panels to each step across: lit on its left flank and turning away
/// into shade on its right, outlined down its slopes in each panel's own darkest tone.
fn cone(
    scene: &mut Canvas,
    peak: (i32, i32),
    eave: i32,
    reach: i32,
    stripes: f32,
    ramps: [Ramp; 2],
) {
    let (centre, top) = peak;
    let panel = |x: i32, y: i32| {
        let across = (x - centre) as f32 / (y - top + 1) as f32;
        ramps[((across * stripes).floor() as i32).rem_euclid(2) as usize]
    };
    let mut rows = vec![(i32::MAX, i32::MIN); (eave - top) as usize];
    polygon(
        scene,
        &[
            (centre, top),
            (centre + reach, eave),
            (centre - reach, eave),
        ],
        |x, y| {
            let row = &mut rows[(y - top) as usize];
            *row = (row.0.min(x), row.1.max(x));
            let ramp = panel(x, y);
            let spread = reach as f32 * (y - top) as f32 / (eave - top) as f32 + 0.5;
            let u = (x - centre) as f32 / spread;
            let lit = -u;
            let mut color = if lit > 0.5 {
                ramp.light
            } else if lit > -0.2 {
                ramp.base
            } else if lit > -0.65 {
                mix(ramp.base, ramp.shadow, 0.55)
            } else {
                ramp.shadow
            };
            // The weave of the canvas.
            if chance(x, y, 652, 10) {
                color = mix(color, ramp.edge, 0.15);
            }
            Some(color)
        },
    );
    for (index, &(first, last)) in rows.iter().enumerate() {
        if first <= last {
            let y = top + index as i32;
            put(scene, first, y, panel(first, y).edge);
            put(scene, last, y, panel(last, y).edge);
        }
    }
}

/// A striped big top with its door hung with a flap, a lit sign over it, bulbs along its eave,
/// a pennant on its pole and guy ropes out to pegs.
fn big_top(scene: &mut Canvas, lamps: &mut Canvas, left: i32, base: i32) {
    let (width, eave, peak) = (96, base - 44, base - 78);
    let centre = left + width / 2;
    ellipse(scene, centre + 8, base + 1, width / 2 + 4, 5, SHADOW);
    cone(
        scene,
        (centre, peak),
        eave,
        width / 2 + 4,
        6.0,
        [CANVAS_RED, CANVAS_CREAM],
    );
    // Strings of bulbs down the roof from the pole to the eave.
    for (string, reach) in [-44, -17, 17, 44].into_iter().enumerate() {
        let (to_x, to_y) = (centre + reach, eave - 1);
        line(scene, (centre, peak + 1), (to_x, to_y), rgba(0x2a1e28, 110));
        for step in 1..5 {
            let t = step as f32 / 5.0;
            let x = centre + (reach as f32 * t).round() as i32;
            let y = peak + 1 + ((to_y - peak - 1) as f32 * t).round() as i32;
            stud(scene, lamps, x, y + 1, step + string);
        }
    }
    // The walls, a drum of stripes: lit towards the left, in shade towards the right, each panel
    // hanging in a shallow fold and darker at the foot where the canvas meets the ground.
    for y in eave..base {
        for x in left..left + width {
            let column = x - left;
            let ramp = if (column / 8) % 2 == 0 {
                CANVAS_RED
            } else {
                CANVAS_CREAM
            };
            let u = (column as f32 + 0.5) / width as f32 * 2.0 - 1.0;
            let fold = match column % 8 {
                0 => 0.25,
                7 => -0.3,
                _ => 0.0,
            };
            let lit = -u * 1.1 + fold;
            let mut color = if lit > 0.6 {
                ramp.light
            } else if lit > -0.25 {
                ramp.base
            } else if lit > -0.75 {
                mix(ramp.base, ramp.shadow, 0.55)
            } else {
                ramp.shadow
            };
            if column == 0 || column == width - 1 {
                color = ramp.edge;
            } else if y >= base - 2 {
                color = mix(color, ramp.edge, 0.4);
            }
            scene.set(x, y, color);
        }
    }
    // Guy ropes out to pegs in the sawdust.
    for (from, to) in [
        ((left - 3, eave + 1), (left - 11, base + 2)),
        ((left + width + 2, eave + 1), (left + width + 12, base + 3)),
    ] {
        line(scene, from, to, ROPE);
        vline(scene, to.0, to.1 - 2, 3, PLANK.shadow);
        put(scene, to.0, to.1 - 2, PLANK.light);
    }
    // The door: a way into the ring, dim inside by day. The flap let down over it is a prop.
    let (door_left, door_width) = (centre - 10, 20);
    for y in base - 28..base {
        let t = (y - base + 28) as f32 / 28.0;
        hline(
            scene,
            door_left,
            y,
            door_width,
            mix(rgb(0x2a1a22), rgb(0x4a3028), t),
        );
    }
    polygon(
        scene,
        &[
            (door_left - 1, base - 28),
            (centre, base - 34),
            (door_left + door_width + 1, base - 28),
        ],
        |_, _| Some(CANVAS_RED.shadow),
    );
    // After dark, the ring's lamps show through the gap where the flap parts at its foot, glow
    // through the cloth, and fan out over the sawdust.
    for y in base - 8..base {
        let gap = parted(base - 1 - y);
        let t = (y - base + 8) as f32 / 8.0;
        hline(
            lamps,
            centre - gap,
            y,
            gap * 2,
            mix(rgb(0xa85a30), rgb(0xffc878), t),
        );
    }
    halo(
        lamps,
        (centre as f32, base as f32),
        (11.0, 15.0),
        LAMP,
        0.35,
    );
    for dy in 0..7 {
        let spread = 2.5 + dy as f32 * 0.8;
        for x in centre - 12..centre + 12 {
            let off = (x as f32 + 0.5 - centre as f32).abs();
            if off > spread || (off > spread - 1.0 && chance(x, dy, 654, 128)) {
                continue;
            }
            let alpha = (110.0 * (1.0 - dy as f32 / 7.0)) as u8;
            put(lamps, x, base + dy, Rgba { a: alpha, ..LAMP });
        }
    }
    // A scalloped valance round the eave, red and gold, its shade on the wall beneath.
    for x in left - 4..left + width + 4 {
        let across = (x - left + 4).rem_euclid(8);
        let drop = [2, 3, 4, 4, 4, 4, 3, 2][across as usize];
        let ramp = if ((x - left + 4) / 8) % 2 == 0 {
            CANVAS_RED
        } else {
            GOLD
        };
        for dy in 0..drop {
            let color = if dy == drop - 1 {
                ramp.edge
            } else if dy == 0 {
                ramp.light
            } else {
                ramp.base
            };
            put(scene, x, eave + dy, color);
        }
        if (left..left + width).contains(&x) {
            put(scene, x, eave + drop, rgba(0x2a1e28, 70));
        }
    }
    // Bulbs along the eave, one at each scallop's point, lighting the canvas under them.
    for (index, x) in (left..left + width).step_by(8).enumerate() {
        bulb(scene, lamps, x, eave + 2, index);
    }
    // A sign over the door, framed in bulbs.
    let sign = "BIG TOP";
    let sign_width = text_width(sign) + 4;
    let (sign_left, sign_top, sign_height) = (centre - sign_width / 2, eave + 6, GLYPH_HEIGHT + 3);
    rect(
        scene,
        sign_left,
        sign_top,
        sign_width,
        sign_height,
        GOLD.base,
    );
    hline(scene, sign_left, sign_top, sign_width, GOLD.light);
    hline(
        scene,
        sign_left,
        sign_top + sign_height - 1,
        sign_width,
        GOLD.edge,
    );
    vline(scene, sign_left, sign_top, sign_height, GOLD.light);
    vline(
        scene,
        sign_left + sign_width - 1,
        sign_top,
        sign_height,
        GOLD.edge,
    );
    rect(
        scene,
        sign_left + 1,
        sign_top + 1,
        sign_width - 2,
        sign_height - 2,
        CANVAS_RED.edge,
    );
    draw_text_shadowed(scene, sign_left + 2, sign_top + 1, sign, CREAM, INK);
    for (index, x) in (sign_left + 1..sign_left + sign_width - 1)
        .step_by(3)
        .enumerate()
    {
        stud(scene, lamps, x, sign_top, index);
        stud(scene, lamps, x + 1, sign_top + sign_height - 1, index + 1);
    }
    // The pole and its pennant.
    vline(scene, centre, peak - 12, 12, GOLD.light);
    put(scene, centre, peak - 13, GOLD.shine);
    pennant(scene, centre + 1, peak - 12, 10, rgb(0xf5d25e));
}

/// How far either side of the middle the big top's flap is parted, `rows_up` rows above its
/// foot: drawn back a little at the bottom, closed from eight rows up.
fn parted(rows_up: i32) -> i32 {
    ((8 - rows_up) / 2).max(0)
}

/// A pennant flying from a pole at `x`, its top at `top`, `long` from the pole to its tip.
fn pennant(scene: &mut Canvas, x: i32, top: i32, long: i32, color: Rgba) {
    polygon(
        scene,
        &[(x, top), (x + long, top + 3), (x, top + 6)],
        |_, y| {
            Some(if y < top + 2 {
                mix(color, rgb(0xffffff), 0.25)
            } else if y >= top + 4 {
                mix(color, INK, 0.25)
            } else {
                color
            })
        },
    );
}

/// How far round a circle seen edge on `x` is from its side, as a height: 1 in its middle, 0 at
/// its edges, `reach` either side of `centre`.
fn arc(x: i32, centre: i32, reach: i32) -> f32 {
    let u = (x - centre) as f32 / reach as f32;
    (1.0 - u * u).max(0.0).sqrt()
}

/// A carousel: a striped canopy with a rounding board of bulbs and a fringed valance, horses going
/// round on twisted brass poles, a painted drum set with mirrors in its middle and a painted deck
/// underfoot. After dark, its underside and everything under it are warm with its bulbs.
fn carousel(scene: &mut Canvas, lamps: &mut Canvas, centre: i32) {
    let (half, roof_top, eave, deck) = (52, 30, 54, 106);
    // The canopy's reach either side at its eave, the foot of its rounding board, and the far
    // rim of its underside, lower in the middle where it is farthest off.
    let reach = half + 6;
    let board = eave + 7;
    let far_rim = |x: i32| board + 3 + (6.0 * arc(x, centre, reach)).round() as i32;
    let far_deck = |x: i32| deck - (5.0 * arc(x, centre, half)).round() as i32;
    let near_deck = |x: i32| deck + (5.0 * arc(x, centre, half)).round() as i32;
    ellipse(scene, centre + 8, deck + 6, half + 8, 7, SHADOW);
    // The deck: boards laid in rings, and its painted edge towards us.
    for y in deck - 5..=deck + 5 {
        for x in centre - half..=centre + half {
            let (u, v) = ((x - centre) as f32 / half as f32, (y - deck) as f32 / 5.5);
            let r = (u * u + v * v).sqrt();
            if r > 1.0 {
                continue;
            }
            let mut color = if ((r * 6.0) as i32) % 2 == 0 {
                DECK.base
            } else {
                DECK.shadow
            };
            if chance(x / 2, y, 661, 14) {
                color = DECK.light;
            }
            scene.set(x, y, color);
        }
    }
    for x in centre - half..=centre + half {
        let rim = near_deck(x);
        let u = (x - centre) as f32 / half as f32;
        put(scene, x, rim, GOLD.light);
        let side = if u < -0.4 {
            CANVAS_RED.light
        } else if u < 0.45 {
            CANVAS_RED.base
        } else {
            CANVAS_RED.shadow
        };
        put(scene, x, rim + 1, side);
        put(
            scene,
            x,
            rim + 2,
            if (x - centre).rem_euclid(6) < 2 {
                GOLD.base
            } else {
                side
            },
        );
        put(scene, x, rim + 3, CANVAS_RED.edge);
    }
    // The underside of the canopy, from under the valance back to its far rim, in its own shade,
    // its sweeps running in to the middle with a bulb on each. After dark, the bulbs light it.
    for x in centre - reach + 2..centre + reach - 2 {
        for y in board..far_rim(x) {
            let deep = ((y - board) as f32 / 8.0).min(1.0);
            scene.set(x, y, mix(CEILING_NEAR, CEILING, deep));
            let alpha = (150.0 * (1.0 - deep * 0.5)) as u8;
            put(lamps, x, y, Rgba { a: alpha, ..LAMP });
        }
        // The far side of the valance, seen from within.
        let rim = far_rim(x);
        let across = (x - centre).rem_euclid(6);
        put(scene, x, rim, mix(CANVAS_CREAM.shadow, CEILING, 0.3));
        if (1..5).contains(&across) {
            put(scene, x, rim + 1, mix(CANVAS_RED.base, CEILING, 0.3));
        }
    }
    let hub = (centre, far_rim(centre) - 1);
    for (index, sweep) in [-3, -2, -1, 1, 2, 3].into_iter().enumerate() {
        let x = centre + sweep * reach / 4;
        line(
            scene,
            (x, board + 2),
            hub,
            mix(GOLD.shadow, CEILING_NEAR, 0.3),
        );
        let (sx, sy) = ((x + hub.0) / 2, (board + 2 + hub.1) / 2);
        stud(scene, lamps, sx, sy, index);
    }
    // The far side, in the shade under the canopy: poles and horses going the other way, dim
    // beyond the middle.
    for x in centre - half..=centre + half {
        for y in far_rim(x) + 2..far_deck(x) {
            put(scene, x, y, Rgba { a: 90, ..INSIDE });
        }
    }
    for (offset, rise, horse_kind) in [(-30, 3, 4), (30, 0, 5)] {
        let x = centre + offset;
        let floor = far_deck(x);
        pole(scene, x, far_rim(x) + 1, floor, true);
        horse(scene, x, floor - 17 - rise, true, HORSES[horse_kind], 0.55);
    }
    centre_column(scene, centre, far_rim(centre) + 1, deck - 2);
    // The near side: twisted brass poles, and the horses on them, rising and falling.
    for (index, offset) in [-39, -13, 13, 39].into_iter().enumerate() {
        let x = centre + offset;
        let floor = near_deck(x) - 1;
        let rise = if index % 2 == 0 { 0 } else { 5 };
        pole(scene, x, board, floor, false);
        horse(scene, x, floor - 18 - rise, false, HORSES[index], 0.0);
    }
    // After dark, all of it lit from above by the bulbs round the canopy.
    halo(
        lamps,
        (centre as f32, (deck - 22) as f32),
        ((half + 6) as f32, 34.0),
        LAMP,
        0.3,
    );
    // The canopy: a cone of teal and cream, a gilt knop at its point and a pennant above.
    cone(
        scene,
        (centre, roof_top),
        eave,
        reach,
        7.0,
        [TEAL, CANVAS_CREAM],
    );
    vline(scene, centre, roof_top - 10, 9, GOLD.light);
    pennant(scene, centre + 1, roof_top - 10, 8, CANVAS_RED.base);
    ellipse(scene, centre, roof_top - 1, 2, 2, GOLD.edge);
    put(scene, centre - 1, roof_top - 2, GOLD.shine);
    put(scene, centre, roof_top - 1, GOLD.light);
    // The rounding board: gilt framed panels, red and teal, with a mirror in each, and a bulb
    // on every pilaster between them.
    for x in centre - reach..centre + reach {
        let slot = (x - centre + reach).rem_euclid(12);
        let ramp = if ((x - centre + reach) / 12) % 2 == 0 {
            CANVAS_RED
        } else {
            TEAL
        };
        for y in eave..board {
            let row = y - eave;
            let color = if row == 0 {
                GOLD.light
            } else if row == board - eave - 1 {
                GOLD.edge
            } else if slot == 0 {
                GOLD.light
            } else if slot == 1 {
                GOLD.shadow
            } else if row == 1 || row == board - eave - 2 {
                GOLD.base
            } else if (5..9).contains(&slot) && row == 3 {
                MIRROR.light
            } else if (5..9).contains(&slot) {
                MIRROR.base
            } else if row == 2 {
                ramp.base
            } else {
                ramp.shadow
            };
            scene.set(x, y, color);
        }
    }
    // The valance: scallops of cream and red, fringed in gold with a tassel to each.
    for x in centre - reach..centre + reach {
        let across = (x - centre + reach).rem_euclid(6);
        let drop = [2, 3, 4, 4, 3, 2][across as usize];
        let ramp = if ((x - centre + reach) / 6) % 2 == 0 {
            CANVAS_CREAM
        } else {
            CANVAS_RED
        };
        for dy in 0..drop {
            let color = if dy == drop - 1 {
                ramp.edge
            } else if dy == 0 {
                ramp.light
            } else {
                ramp.base
            };
            put(scene, x, board + dy, color);
        }
        if across == 2 || across == 3 {
            put(scene, x, board + drop, GOLD.base);
        }
    }
    for (index, x) in (centre - reach + 1..centre + reach).step_by(12).enumerate() {
        bulb(scene, lamps, x, eave + 2, index);
    }
    for (index, x) in (centre - reach + 7..centre + reach).step_by(12).enumerate() {
        stud(scene, lamps, x, eave, index + 1);
    }
}

/// A brass pole from `top` down to `bottom`, twisted like barley sugar so its light and shade
/// wind round it. One on the far side of the ride is thinner and dimmer.
fn pole(scene: &mut Canvas, x: i32, top: i32, bottom: i32, far: bool) {
    for y in top..bottom {
        let twist = y.rem_euclid(4) < 2;
        if far {
            let color = if twist { GOLD.base } else { GOLD.shadow };
            put(scene, x, y, mix(color, INSIDE, 0.35));
        } else {
            put(scene, x, y, if twist { GOLD.shine } else { GOLD.base });
            put(scene, x + 1, y, if twist { GOLD.base } else { GOLD.shadow });
        }
    }
    if !far {
        hline(scene, x - 1, bottom - 1, 4, GOLD.edge);
        hline(scene, x - 1, bottom - 2, 4, GOLD.light);
    }
}

/// The carousel's middle, round which it turns: painted panels between gilt pilasters, a mirror
/// in each with the bulbs round it shining back out of the glass, and a frieze along its top.
fn centre_column(scene: &mut Canvas, centre: i32, top: i32, bottom: i32) {
    let half = 9;
    let middle = (top + bottom) / 2;
    for y in top..bottom {
        for x in centre - half..centre + half {
            let column = x - centre + half;
            let across = column as f32 / (half * 2) as f32;
            let panel = column / 6;
            let ramp = if panel % 2 == 0 { CANVAS_RED } else { TEAL };
            let in_mirror = in_ellipse(
                column,
                y,
                (panel * 6 + 3, middle),
                (1, (bottom - top) / 2 - 5),
            );
            let color = if y < top + 3 {
                if y == top + 2 {
                    GOLD.base
                } else {
                    round_tone(CANVAS_RED, across)
                }
            } else if y >= bottom - 2 || column % 6 == 0 {
                round_tone(GOLD, across)
            } else if in_mirror && chance(x, y, 671, 30) {
                MIRROR.shine
            } else if in_mirror {
                round_tone(MIRROR, across.max(0.3))
            } else {
                mix(round_tone(ramp, across), ramp.edge, 0.35)
            };
            scene.set(x, y, color);
        }
    }
    vline(scene, centre - half, top, bottom - top, GOLD.edge);
    vline(scene, centre + half - 1, top, bottom - top, GOLD.edge);
}

/// A tone across something round lit from the left, `across` from 0 at its left to 1 at its
/// right: a light edge, its brightest a little in, then turning away into shade.
fn round_tone(ramp: Ramp, across: f32) -> Rgba {
    if across < 0.1 {
        ramp.light
    } else if across < 0.28 {
        ramp.shine
    } else if across < 0.45 {
        ramp.light
    } else if across < 0.75 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// A carousel horse in profile at a gallop, facing right, as rows: `E` its outline, `s`, `o`
/// and `l` its coat from shade to light, `M`, `m` and `n` its mane and tail, `k` its eye, `R`,
/// `r` and `q` its saddle, `g` the gilt on the saddle, `h` its gilded hooves.
const HORSE: [&str; 15] = [
    ".............E.E...",
    "............EnElE..",
    "...........EmnlllE.",
    "..........EmmlkollE",
    ".........EmmllEsloE",
    "........EMmloE.EEE.",
    "...ERRRREmlloE.....",
    ".EnERgqrgRlloE.....",
    "EnmElllllllooE.....",
    "EmMElllooooooE.....",
    "EnmEoooooooosE.....",
    ".EMEsssssssssEEEE..",
    "..E.Es.EsEEEsEsssE.",
    "...EsE..Es.EsE..EsE",
    "...Eh...Eh.Eh....Eh",
];
/// The column of `HORSE` its pole runs through, under the saddle.
const HORSE_POLE: i32 = 7;

/// Each horse's coat, mane and saddle: the four on the near side, then the two on the far.
const HORSES: [(Ramp, Ramp, Ramp); 6] = [
    (WHITE_COAT, GOLD, CANVAS_RED),
    (DAPPLE_COAT, DARK_MANE, TEAL),
    (CHESTNUT_COAT, FLAXEN_MANE, AWNING_BLUE),
    (WHITE_COAT, DARK_MANE, GOLD),
    (CREAM_COAT, FLAXEN_MANE, TEAL),
    (WHITE_COAT, GOLD, AWNING_BLUE),
];

/// A horse with its pole through its saddle at `x` and its ears at `top`, facing whichever way
/// its side of the ride is going; `dim` is how far it is lost in the shade.
fn horse(
    scene: &mut Canvas,
    x: i32,
    top: i32,
    facing_right: bool,
    (coat, mane, saddle): (Ramp, Ramp, Ramp),
    dim: f32,
) {
    let width = HORSE[0].len() as i32;
    let left = if facing_right {
        x - HORSE_POLE
    } else {
        x - (width - 1 - HORSE_POLE)
    };
    let inks = [
        (b'E', coat.edge),
        (b's', coat.shadow),
        (b'o', coat.base),
        (b'l', coat.light),
        (b'M', mane.shadow),
        (b'm', mane.base),
        (b'n', mane.light),
        (b'k', INK),
        (b'R', saddle.shadow),
        (b'r', saddle.base),
        (b'q', saddle.light),
        (b'g', GOLD.light),
        (b'h', GOLD.base),
    ]
    .map(|(letter, color)| (letter, mix(color, INSIDE, dim)));
    stamp(scene, (left, top), &HORSE, &inks, !facing_right);
}

/// A hoopla stall: a striped awning, prizes on the shelves behind in its shade, bulbs under it
/// and a sign framed in bulbs, which light it all after dark. Its counter is a prop, so someone
/// can hide behind it.
fn hoopla_stall(scene: &mut Canvas, lamps: &mut Canvas, left: i32) {
    let (width, roof, base) = (72, 66, 104);
    let opening = roof + 10;
    let middle = left + width / 2;
    ellipse(scene, middle + 6, base + 2, width / 2 + 4, 4, SHADOW);
    // The back wall, of boards.
    for y in opening..base {
        for x in left..left + width {
            let board = (x - left).rem_euclid(6);
            let mut color = match board {
                0 => BOARDS.edge,
                1 => BOARDS.light,
                _ => BOARDS.base,
            };
            if board > 1 && chance(x, y / 3, 672, 50) {
                color = BOARDS.shadow;
            }
            scene.set(x, y, color);
        }
    }
    // Little prizes hung on a line along the top: stars and fish.
    hline(scene, left + 3, opening + 2, width - 6, rgb(0x8a7a6a));
    for (index, x) in (left + 7..left + width - 5).step_by(9).enumerate() {
        let y = opening + 3;
        if index % 2 == 0 {
            // A star.
            put(scene, x, y, GOLD.light);
            hline(scene, x - 1, y + 1, 3, GOLD.base);
            put(scene, x - 1, y + 2, GOLD.shadow);
            put(scene, x + 1, y + 2, GOLD.shadow);
        } else {
            // A fish, in a bag of water.
            ellipse(scene, x, y + 1, 2, 2, rgba(0xd0e4ec, 130));
            hline(scene, x - 1, y + 1, 2, rgb(0xff8a3a));
            put(scene, x + 1, y + 1, rgb(0xd85a1a));
            put(scene, x - 1, y, rgba(0xffffff, 200));
        }
    }
    // A shelf of prizes up high, teddy bears and bottles on show, and below it the boards left
    // bare, so the pegs and prizes set out along the counter stand out against them.
    let shelf = opening + 12;
    for slot in 0..8 {
        let x = left + 5 + slot * 8;
        let kind = slot % 6;
        if kind % 2 == 0 {
            let coat = [BEAR, HONEY_BEAR, PINK_BEAR][(kind / 2) as usize];
            stamp(scene, (x, shelf - 7), &TEDDY, &teddy_inks(coat), false);
        } else {
            bottle(scene, x, shelf - 7, kind == 3);
        }
    }
    hline(scene, left + 2, shelf, width - 4, PLANK.light);
    hline(scene, left + 2, shelf + 1, width - 4, PLANK.edge);
    hline(scene, left + 2, shelf + 2, width - 4, rgba(0x1a1014, 90));
    // The back of the stall, in the shade of the counter towards its foot.
    for y in shelf + 3..base {
        let deep = (y - shelf - 3) as f32 / (base - shelf - 3) as f32;
        hline(scene, left, y, width, rgba(0x1a1014, (deep * 70.0) as u8));
    }
    // After dark, the light of the bulbs under the awning, filling the stall.
    halo(
        lamps,
        (middle as f32, opening as f32 + 2.0),
        (width as f32 * 0.62, 32.0),
        LAMP,
        0.4,
    );
    // Posts, lit on the side facing in after dark.
    for (post, inner) in [(left, 2), (left + width - 3, 0)] {
        rect(scene, post, roof, 3, base - roof, PLANK.base);
        vline(scene, post, roof, base - roof, PLANK.edge);
        vline(scene, post + 2, roof, base - roof, PLANK.edge);
        vline(scene, post + 1, roof, base - roof, PLANK.light);
        vline(
            lamps,
            post + inner,
            opening,
            base - opening,
            Rgba { a: 140, ..LAMP },
        );
    }
    // The lining of the awning, in its own shade by day, and warm with the bulbs under it after
    // dark.
    for x in left..left + width {
        let ramp = if ((x - left + 3) / 6) % 2 == 0 {
            AWNING_BLUE
        } else {
            CANVAS_CREAM
        };
        put(scene, x, opening, ramp.shadow);
        put(scene, x, opening + 1, mix(ramp.shadow, ramp.edge, 0.4));
        put(
            lamps,
            x,
            opening,
            Rgba {
                a: 200,
                ..mix(ramp.light, LAMP, 0.5)
            },
        );
        put(
            lamps,
            x,
            opening + 1,
            Rgba {
                a: 150,
                ..mix(ramp.base, LAMP, 0.4)
            },
        );
    }
    // The striped, scalloped awning, lit along its top.
    for y in roof..roof + 10 {
        for x in left - 3..left + width + 3 {
            let ramp = if ((x - left + 3) / 6) % 2 == 0 {
                AWNING_BLUE
            } else {
                CANVAS_CREAM
            };
            let scallop = (x - left + 3).rem_euclid(6);
            if y >= roof + 8 && !(1..5).contains(&scallop) {
                continue;
            }
            let row = y - roof;
            let color = if row == 0 {
                ramp.edge
            } else if row == 1 {
                ramp.light
            } else if y == roof + 9 || (y == roof + 7 && !(1..5).contains(&scallop)) {
                ramp.edge
            } else if row >= 6 {
                ramp.shadow
            } else {
                ramp.base
            };
            scene.set(x, y, color);
        }
    }
    vline(scene, left - 3, roof, 8, AWNING_BLUE.edge);
    vline(scene, left + width + 2, roof, 8, AWNING_BLUE.edge);
    // The sign, framed in bulbs, on a little pole for the string of lights.
    let sign = "HOOPLA";
    let sign_width = text_width(sign) + 6;
    let (sign_left, sign_top) = (middle - sign_width / 2, roof - 12);
    let pole = STRINGS[2].1.0;
    vline(scene, pole, sign_top - 7, 7, GOLD.light);
    put(scene, pole, sign_top - 8, GOLD.shine);
    bevel(
        scene,
        sign_left,
        sign_top,
        sign_width,
        GLYPH_HEIGHT + 5,
        GOLD,
    );
    rect(
        scene,
        sign_left + 2,
        sign_top + 2,
        sign_width - 4,
        GLYPH_HEIGHT + 1,
        AWNING_BLUE.edge,
    );
    draw_text_shadowed(
        scene,
        sign_left + 3,
        sign_top + 3,
        sign,
        CREAM,
        rgb(0x0e1430),
    );
    for (index, x) in (sign_left + 1..sign_left + sign_width - 1)
        .step_by(3)
        .enumerate()
    {
        stud(scene, lamps, x, sign_top, index);
        stud(scene, lamps, x + 1, sign_top + GLYPH_HEIGHT + 4, index + 1);
    }
    for (index, x) in (left + 2..left + width).step_by(9).enumerate() {
        bulb(scene, lamps, x, opening + 1, index);
    }
}

/// A teddy bear sitting up, as rows: `o`, `l` and `s` its fur, `k` its eyes and nose, `r` the
/// ribbon round its neck.
const TEDDY: [&str; 7] = [
    "o...s", "ollls", "lkokl", ".lks.", "orrrs", "ollls", "os.ss",
];

fn teddy_inks(coat: Ramp) -> [(u8, Rgba); 5] {
    [
        (b'o', coat.base),
        (b'l', coat.light),
        (b's', coat.shadow),
        (b'k', INK),
        (b'r', CANVAS_RED.light),
    ]
}

/// A bottle to throw a ring over, its top-left at `(x, y)`, with a ring round its neck if
/// someone has won it.
fn bottle(scene: &mut Canvas, x: i32, y: i32, ringed: bool) {
    put(scene, x + 2, y, PLANK.base);
    put(scene, x + 2, y + 1, BOTTLE.base);
    for dy in 2..7 {
        put(scene, x + 1, y + dy, BOTTLE.light);
        put(scene, x + 2, y + dy, BOTTLE.base);
        put(scene, x + 3, y + dy, BOTTLE.edge);
    }
    put(scene, x + 1, y + 3, BOTTLE.shine);
    if ringed {
        hline(scene, x, y + 3, 5, CANVAS_RED.light);
        put(scene, x, y + 3, CANVAS_RED.shadow);
    }
}

/// A painted caravan at the far end, its bow top over it, a lantern at its door and a curl of
/// smoke from its chimney; after dark, its window and the lantern lit.
fn caravan(scene: &mut Canvas, lamps: &mut Canvas, left: i32) {
    let (width, top, base) = (44, 70, 110);
    let (body_top, body_bottom) = (top + 8, base - 6);
    let middle = left + width / 2;
    ellipse(scene, middle + 6, base + 2, width / 2 + 2, 4, SHADOW);
    // The chimney behind the crown of the roof, and its smoke.
    rect(scene, left + 30, top - 6, 3, 8, IRON.base);
    vline(scene, left + 30, top - 6, 8, IRON.light);
    hline(scene, left + 29, top - 7, 5, IRON.edge);
    for (dx, dy, alpha) in [(1, -10, 90), (0, -13, 70), (2, -16, 50), (1, -19, 34)] {
        ellipse(scene, left + 31 + dx, top + dy, 2, 2, rgba(0xb8a8c0, alpha));
    }
    // The bow top, painted canvas over hoops, lit along its crown.
    for x in left - 1..=left + width {
        let u = (x - middle) as f32 / (width / 2 + 1) as f32;
        let crown = body_top - (8.0 * (1.0 - u * u).max(0.0).sqrt()).round() as i32;
        for y in crown..=body_top {
            // Lit where it faces the sky, turning away into shade towards its eaves.
            let high = (body_top - y) as f32 / 8.0;
            let lit = high * 0.9 - u * 0.35 - 0.25;
            let mut color = if y == crown || y == body_top {
                TEAL.edge
            } else if y == crown + 1 && u < 0.2 {
                TEAL.shine
            } else if lit > 0.35 {
                TEAL.light
            } else if lit > -0.1 {
                TEAL.base
            } else {
                TEAL.shadow
            };
            if (x - left).rem_euclid(9) == 4 && y > crown {
                color = mix(color, TEAL.edge, 0.4);
            }
            put(scene, x, y, color);
        }
    }
    // The body, painted boards lined in gilt.
    bevel(
        scene,
        left,
        body_top,
        width,
        body_bottom - body_top,
        CARAVAN_RED,
    );
    hline(scene, left - 1, body_top, width + 2, CARAVAN_RED.edge);
    let (lining_left, lining_top) = (left + 3, body_top + 3);
    let (lining_width, lining_height) = (width - 6, body_bottom - body_top - 6);
    hline(scene, lining_left, lining_top, lining_width, GOLD.base);
    hline(
        scene,
        lining_left,
        lining_top + lining_height - 1,
        lining_width,
        GOLD.shadow,
    );
    vline(scene, lining_left, lining_top, lining_height, GOLD.base);
    vline(
        scene,
        lining_left + lining_width - 1,
        lining_top,
        lining_height,
        GOLD.shadow,
    );
    // The window, its curtains drawn back. After dark, warm with the lamp inside, its curtains lit
    // from behind, and its light on the paint round it.
    let (wx, wy, ww, wh) = (left + 14, body_top + 4, 16, 12);
    window(scene, wx, wy, ww, wh, 2);
    halo(
        lamps,
        ((wx + ww / 2) as f32, (wy + wh / 2) as f32),
        (18.0, 14.0),
        LAMP,
        0.3,
    );
    for y in wy + 2..wy + wh - 2 {
        let t = (y - wy - 2) as f32 / (wh - 4) as f32;
        let pane = if t < 0.3 {
            GLOW[2]
        } else if t < 0.7 {
            GLOW[1]
        } else {
            GLOW[0]
        };
        for x in wx + 2..wx + ww - 2 {
            let pixel = scene.get(x, y);
            if pixel != TRIM.base && pixel != TRIM.shadow {
                lamps.set(x, y, pane);
            }
        }
    }
    for (x, way) in [(wx + 2, 1), (wx + ww - 3, -1)] {
        for y in wy + 2..wy + wh - 2 {
            let reach = if y < wy + 6 { 2 } else { 1 };
            for step in 0..reach {
                let cloth = if step == 0 {
                    CANVAS_RED.base
                } else {
                    CANVAS_RED.light
                };
                put(scene, x + way * step, y, cloth);
                lamps.set(x + way * step, y, mix(cloth, LAMP, 0.35));
            }
        }
    }
    // The undercarriage, and its wheels.
    rect(scene, left + 2, body_bottom, width - 4, 2, OAK.edge);
    for (wheel, radius) in [(left + 9, 6.0), (left + width - 9, 6.0)] {
        cart_wheel(scene, (wheel as f32, (base - 6) as f32), radius);
    }
    // A lantern hung from the edge of the roof by the door at its far end.
    let hook = left + width - 4;
    vline(scene, hook + 2, body_top, 2, IRON.base);
    lantern(scene, hook, body_top + 2);
    lantern_light(lamps, hook, body_top + 2);
}

/// A lantern of tin and glass with a candle in it, its top-left at `(x, y)`: `#`, `l` and `s` its
/// tin, `y` and `G` its glass, `f` the candle.
const LANTERN: [&str; 8] = [
    "..#..", ".#l#.", "#####", "#yfy#", "#yGy#", "#yGy#", "#####", ".#s#.",
];

/// The lantern by day, its candle out and the sky in its glass.
fn lantern(s: &mut Canvas, x: i32, y: i32) {
    let inks = [
        (b'#', IRON.edge),
        (b'l', IRON.light),
        (b's', IRON.shadow),
        (b'y', GLASS[1]),
        (b'G', GLASS[2]),
        (b'f', CREAM),
    ];
    stamp(s, (x, y), &LANTERN, &inks, false);
}

/// The lantern at `(x, y)` after dark: its glass full of candlelight, and the light about it.
fn lantern_light(lamps: &mut Canvas, x: i32, y: i32) {
    halo(
        lamps,
        (x as f32 + 2.5, y as f32 + 4.5),
        (9.0, 9.0),
        LAMP,
        0.4,
    );
    let inks = [(b'y', GLOW[0]), (b'G', GLOW[1]), (b'f', rgb(0xfffbe8))];
    stamp(lamps, (x, y), &LANTERN, &inks, false);
}

/// A cart wheel about `middle`: an iron tyre, a wooden felloe, six spokes and a brass hub, lit
/// from the upper left, with what is behind it showing between the spokes.
fn cart_wheel(s: &mut Canvas, middle: (f32, f32), radius: f32) {
    let reach = radius.ceil() as i32 + 1;
    let (cx, cy) = (middle.0.round() as i32, middle.1.round() as i32);
    for y in cy - reach..=cy + reach {
        for x in cx - reach..=cx + reach {
            let (dx, dy) = (x as f32 + 0.5 - middle.0, y as f32 + 0.5 - middle.1);
            let d = (dx * dx + dy * dy).sqrt();
            if d > radius {
                continue;
            }
            let lit = dx + dy < -radius * 0.4;
            let color = if d > radius - 1.0 {
                if lit { IRON.light } else { IRON.edge }
            } else if d > radius - 2.0 {
                if lit { OAK.light } else { OAK.shadow }
            } else if d < 1.2 {
                if lit { GOLD.shine } else { GOLD.base }
            } else {
                let angle = dy.atan2(dx).rem_euclid(std::f32::consts::FRAC_PI_3);
                let off = angle.min(std::f32::consts::FRAC_PI_3 - angle) * d;
                if off > 0.55 {
                    continue;
                }
                if lit { OAK.light } else { OAK.base }
            };
            put(s, x, y, color);
        }
    }
}

/// A string of lights: where it is hung from and to, and how far it sags between.
type Span = ((i32, i32), (i32, i32), f32);

/// Strings of bulbs from the edge of the picture to the big top's pole, on to the carousel's
/// crown, down to the stall's sign and away off the right: where each runs from and to, and how
/// far it sags.
const STRINGS: [Span; 4] = [
    ((-6, 52), (60, 34), 7.0),
    ((60, 34), (196, 26), 10.0),
    ((196, 26), (300, 47), 12.0),
    ((300, 47), (390, 42), 8.0),
];

/// The strings of lights, with bunting between the bulbs.
fn festoons(scene: &mut Canvas, lamps: &mut Canvas) {
    let flags = [CANVAS_RED, CANVAS_CREAM, TEAL, GOLD];
    for (string, &(from, to, sag)) in STRINGS.iter().enumerate() {
        let at = |t: f32| {
            (
                from.0 as f32 + (to.0 - from.0) as f32 * t,
                from.1 as f32 + (to.1 - from.1) as f32 * t + sag * 4.0 * t * (1.0 - t),
            )
        };
        let mut previous = at(0.0);
        for step in 1..=60 {
            let point = at(step as f32 / 60.0);
            line(
                scene,
                (previous.0.round() as i32, previous.1.round() as i32),
                (point.0.round() as i32, point.1.round() as i32),
                WIRE,
            );
            previous = point;
        }
        let count = ((to.0 - from.0) as f32 / 11.0).round() as i32;
        for index in 0..count {
            let (x, y) = at((index as f32 + 0.5) / count as f32);
            let flag = flags[(index as usize + string) % flags.len()];
            let (x, y) = (x.round() as i32, y.round() as i32 + 1);
            hline(scene, x - 1, y, 3, flag.light);
            put(scene, x - 1, y + 1, flag.base);
            put(scene, x, y + 1, flag.shadow);
            put(scene, x, y + 2, flag.edge);
        }
        for index in 1..count {
            let (x, y) = at(index as f32 / count as f32);
            bulb(
                scene,
                lamps,
                x.round() as i32,
                y.round() as i32 + 1,
                index as usize + string,
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The high striker
// ---------------------------------------------------------------------------------------------

/// The high striker: a tall board striped red and cream, a mark to each stripe with a gilt tick
/// either side, a brass rail up its middle for the puck, a gilt cap on top for the bell to sit on
/// (the bell, the puck and the mallet move, so they are drawn with the game: see `gear`), little
/// bulbs up its edges that light after dark, a plinth painted with a star, and the pad at its
/// foot that the mallet comes down on.
fn high_striker(scene: &mut Canvas, lamps: &mut Canvas) {
    let (cx, base) = STRIKER;
    let (left, right) = (cx - 4, cx + 4);
    let (rest, bell) = PUCK_RAIL;
    let foot = base - 9;
    let top = bell - 1;
    let band = (rest + 2 - bell) as f32 / 10.0;
    ellipse(scene, cx + 3, base, 13, 2, SHADOW);
    // The board, one stripe to each mark, each stripe shaded in its own colour.
    for y in top..=foot {
        let mark = (((rest + 2 - y) as f32 / band).floor() as i32).clamp(0, 9);
        let ramp = if mark % 2 == 0 {
            CANVAS_RED
        } else {
            CANVAS_CREAM
        };
        for x in left..=right {
            let color = if x == left || x == right {
                ramp.edge
            } else if x == left + 1 {
                ramp.light
            } else if x >= right - 1 || chance(x, y, 721, 30) {
                ramp.shadow
            } else {
                ramp.base
            };
            put(scene, x, y, color);
        }
    }
    // A gilt tick either side at every mark, and a bulb at every other one.
    for mark in 1..10 {
        let y = rest + 2 - (mark as f32 * band).round() as i32;
        put(scene, left - 1, y, GOLD.light);
        put(scene, right + 1, y, GOLD.base);
        hline(
            scene,
            left + 1,
            y,
            2,
            mix(GOLD.light, CANVAS_CREAM.base, 0.4),
        );
        hline(
            scene,
            right - 2,
            y,
            2,
            mix(GOLD.shadow, CANVAS_RED.base, 0.4),
        );
        if mark % 2 == 1 {
            stud(scene, lamps, left - 1, y - 2, mark as usize);
            stud(scene, lamps, right + 1, y - 2, mark as usize + 1);
        }
    }
    // The rail, brass in a groove, with a stop at the foot for the puck to rest on.
    vline(scene, cx, top + 1, foot - top - 1, GOLD.light);
    vline(
        scene,
        cx + 1,
        top + 1,
        foot - top - 1,
        mix(GOLD.shadow, INK, 0.3),
    );
    hline(scene, cx - 1, rest + 3, 3, IRON.base);
    hline(scene, cx - 1, rest + 4, 3, IRON.edge);
    // A star at the top mark, for the bell.
    let star = top + 2;
    put(scene, cx - 2, star + 1, GOLD.light);
    put(scene, cx + 2, star + 1, GOLD.base);
    put(scene, cx - 1, star, GOLD.shine);
    put(scene, cx + 1, star, GOLD.light);
    // The gilt cap the bell sits on.
    bevel(scene, left - 2, bell, 13, 4, GOLD);
    hline(scene, left - 1, bell + 4, 11, mix(GOLD.edge, INK, 0.2));
    // The plinth, painted, with a star on its front.
    bevel(scene, cx - 8, foot + 1, 17, base - foot - 1, OAK);
    rect(
        scene,
        cx - 5,
        foot + 3,
        11,
        base - foot - 6,
        CANVAS_RED.base,
    );
    hline(scene, cx - 5, foot + 3, 11, CANVAS_RED.light);
    hline(scene, cx - 5, base - 4, 11, CANVAS_RED.shadow);
    put(scene, cx, foot + 4, GOLD.shine);
    hline(scene, cx - 1, foot + 5, 3, GOLD.light);
    put(scene, cx - 1, foot + 6, GOLD.base);
    put(scene, cx + 1, foot + 6, GOLD.shadow);
    // The pad on its lever, out in front of the plinth: red leather, plump, lit along its top,
    // on an oak block.
    let (pad_x, pad_top) = PAD;
    ellipse(scene, pad_x + 1, pad_top + 5, 6, 1, SHADOW);
    bevel(scene, pad_x - 3, pad_top + 2, 7, 3, OAK);
    for y in pad_top..pad_top + 3 {
        for x in pad_x - 4..=pad_x + 4 {
            let row = y - pad_top;
            let corner = (x == pad_x - 4 || x == pad_x + 4) && row == 0;
            if corner {
                continue;
            }
            let edge = x == pad_x - 4 || x == pad_x + 4 || row == 0 || row == 2;
            let color = if edge {
                CANVAS_RED.edge
            } else if x < pad_x - 1 {
                CANVAS_RED.light
            } else {
                CANVAS_RED.base
            };
            put(scene, x, y, color);
        }
    }
    put(scene, pad_x - 2, pad_top + 1, CANVAS_RED.shine);
}

// ---------------------------------------------------------------------------------------------
// Hiding places
// ---------------------------------------------------------------------------------------------

/// The front of the big top around its door, with the flap let down: whoever slips inside is
/// hidden altogether, and only the flap stirring gives them away.
fn tent_door() -> (Canvas, (i32, i32), f32) {
    let (left, base) = BIG_TOP;
    let (mut tent, mut lamps) = (
        Canvas::new(SCENE_WIDTH, SCENE_HEIGHT),
        Canvas::new(SCENE_WIDTH, SCENE_HEIGHT),
    );
    big_top(&mut tent, &mut lamps, left, base);
    // Cut along stripe edges, below the scallops, so a shake barely shows.
    let (crop_left, crop_top) = (left + 24, base - 39);
    let (width, height) = (48, 39);
    let mut s = Canvas::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let pixel = tent.get(crop_left + x, crop_top + y);
            if pixel.a > 0 {
                s.set(x, y, pixel);
            }
        }
    }
    // The flap: a heavy red curtain hanging in folds across the doorway, edged in gold, its two
    // halves parted a little at the foot where the ring inside shows (and its lamps, after dark).
    let (door_left, door_top, door_width) = (left + 38 - crop_left, base - 28 - crop_top, 20);
    let parting = door_left + door_width / 2;
    for y in door_top..height {
        let gap = parted(height - 1 - y);
        for x in door_left - 1..door_left + door_width + 1 {
            let from_parting = if x < parting {
                parting - 1 - x
            } else {
                x - parting
            };
            let column = x - door_left;
            let color = if column < 0 || column == door_width {
                // The edge of the doorway, in shadow either side of the flap.
                mix(s.get(x, y), CANVAS_RED.edge, 0.7)
            } else if from_parting < gap {
                // The ring inside, glimpsed, dim out of the sun.
                let t = (y - (height - 8)) as f32 / 8.0;
                mix(rgb(0x2a1a22), rgb(0x4a3028), t)
            } else if gap > 0 && from_parting == gap {
                CANVAS_RED.edge
            } else if column == 0 || column == door_width - 1 {
                GOLD.base
            } else if column == 1 || column == door_width - 2 {
                GOLD.shadow
            } else if y == height - 1 {
                GOLD.light
            } else {
                match (column - 2) % 4 {
                    0 => CANVAS_RED.light,
                    1 => CANVAS_RED.base,
                    2 => CANVAS_RED.shadow,
                    _ => mix(CANVAS_RED.shadow, CANVAS_RED.edge, 0.5),
                }
            };
            s.set(x, y, color);
        }
    }
    // The rod it hangs from, and its rings.
    hline(&mut s, door_left - 1, door_top, door_width + 2, GOLD.base);
    put(&mut s, door_left - 1, door_top, GOLD.edge);
    put(&mut s, door_left + door_width, door_top, GOLD.edge);
    for x in (door_left + 2..door_left + door_width).step_by(5) {
        put(&mut s, x, door_top, GOLD.shine);
        put(&mut s, x, door_top + 1, GOLD.shadow);
    }
    (s, (crop_left, crop_top), base as f32)
}

/// Three bales of hay, two on the ground and one on top.
fn hay_stack(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = Canvas::new(38, 24);
    ellipse(&mut s, 19, 22, 19, 2, SHADOW);
    bale(&mut s, (0, 10), (18, 12), HAY, 661);
    bale(&mut s, (18, 10), (18, 12), HAY, 662);
    // Where the top bale rests, the ones under it are in its shade.
    for x in 8..28 {
        put(&mut s, x, 11, rgba(0x2a1e18, 110));
        put(&mut s, x, 12, rgba(0x2a1e18, 60));
    }
    bale(&mut s, (8, 0), (20, 11), HAY, 663);
    loose_straw(&mut s, 22, HAY, 664);
    (s, (left, top), (top + 22) as f32)
}

/// A single bale of straw, low enough that only someone crouching is hidden by it.
fn straw_bale(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = Canvas::new(30, 16);
    ellipse(&mut s, 15, 14, 15, 2, SHADOW);
    bale(&mut s, (1, 1), (28, 13), STRAW, 671);
    loose_straw(&mut s, 14, STRAW, 672);
    (s, (left, top), (top + 14) as f32)
}

/// A bale lying on its long side with its top-left at `(x, y)`: its top lit, the straw running
/// along it in streaks, two bands of twine round it pinching it in, and straws sticking out
/// ragged along its edges.
fn bale(s: &mut Canvas, (x, y): (i32, i32), (width, height): (i32, i32), ramp: Ramp, salt: u32) {
    let top = 3;
    let bands = [x + width / 3, x + width * 2 / 3];
    for py in y..y + height {
        let shift = pick(salt as i32, py, salt, 5);
        for px in x..x + width {
            let (dx, dy) = (px - x, py - y);
            let streak = noise((px + shift).div_euclid(3), py, salt + 1) % 8;
            let color = if dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1 {
                ramp.edge
            } else if dy < top {
                // The top, open to the sky.
                match streak {
                    0 => ramp.shine,
                    1 | 2 => ramp.base,
                    _ => ramp.light,
                }
            } else if dy == top {
                // The rounded edge where top turns to side.
                if streak < 3 { ramp.base } else { ramp.light }
            } else {
                // The side, lit from the left and shaded towards its far end and its foot.
                let across = dx as f32 / width as f32;
                let down = (dy - top) as f32 / (height - top) as f32;
                let lit = 0.45 - across * 0.6 - down * 0.7 + (streak as f32 - 3.5) * 0.12;
                if bands.iter().any(|band| (px - band).abs() == 1) {
                    ramp.shadow
                } else if lit > 0.35 {
                    ramp.light
                } else if lit > -0.2 {
                    ramp.base
                } else {
                    ramp.shadow
                }
            };
            put(s, px, py, color);
        }
    }
    for band in bands {
        for py in y..y + height {
            let dy = py - y;
            let color = if dy == 0 || dy == height - 1 {
                TWINE.edge
            } else if dy < top {
                TWINE.light
            } else if dy < top + 3 {
                TWINE.base
            } else {
                TWINE.shadow
            };
            put(s, band, py, color);
        }
    }
    // Straws sticking out along the top and at the ends.
    for px in x + 1..x + width - 1 {
        if chance(px, y, salt + 2, 70) {
            put(s, px, y - 1, ramp.light);
        }
    }
    for py in y + 1..y + height - 1 {
        if chance(x, py, salt + 3, 80) {
            put(s, x - 1, py, ramp.light);
        }
        if chance(x + width, py, salt + 4, 80) {
            put(s, x + width, py, ramp.shadow);
        }
    }
}

/// A few straws fallen on the ground at a bale's foot, along row `y` of its sprite.
fn loose_straw(s: &mut Canvas, y: i32, ramp: Ramp, salt: u32) {
    for index in 0..6 {
        let x = 2 + pick(index, 0, salt, s.width() as i32 - 6);
        let long = 2 + pick(index, 1, salt, 3);
        let lean = pick(index, 2, salt, 3) - 1;
        line(s, (x, y), (x + long, y + lean), ramp.base);
        put(s, x, y, ramp.light);
    }
}

/// An oak barrel bound in iron, with a lantern left standing on its head: the one light out among
/// the props after dark, so the middle of the field isn't all in the dark.
fn barrel(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    /// Room above the barrel for the lantern standing on it.
    const HEADROOM: i32 = 8;
    let h = HEADROOM;
    let mut s = Canvas::new(18, (22 + h) as u32);
    ellipse(&mut s, 10, h + 20, 8, 2, SHADOW);
    // The staves: bellied in the middle, lit from the left, the joins between them dark, the
    // grain running down them.
    for y in h + 3..h + 20 {
        let row = y - h;
        let belly = i32::from((6..15).contains(&row));
        let (first, last) = (1 - belly, 16 + belly);
        for x in first..=last {
            let across = (x - first) as f32 / (last - first) as f32;
            let next = (x + 1 - first) as f32 / (last - first) as f32;
            let mut color = round_tone(OAK, across);
            if x == first || x == last || row == 19 {
                color = OAK.edge;
            } else if (across * 6.0) as i32 != (next * 6.0) as i32 {
                color = mix(color, OAK.edge, 0.55);
            } else if noise(x, y / 3, 691).is_multiple_of(5) {
                color = mix(color, OAK.shadow, 0.4);
            }
            put(&mut s, x, y, color);
        }
    }
    // Iron hoops round it, a rivet in each.
    for hoop in [h + 5, h + 15] {
        for x in 0..18 {
            let across = x as f32 / 17.0;
            if s.get(x, hoop).a == 0 {
                continue;
            }
            let edge = x == 0 || x == 17 || s.get(x - 1, hoop).a == 0 || s.get(x + 1, hoop).a == 0;
            put(
                &mut s,
                x,
                hoop,
                if edge {
                    IRON.edge
                } else {
                    round_tone(IRON, across)
                },
            );
            put(
                &mut s,
                x,
                hoop + 1,
                if edge { IRON.edge } else { IRON.shadow },
            );
        }
        put(&mut s, 4, hoop, IRON.shine);
    }
    // The head, seen from a little above: its rim, and its boards.
    for y in h..h + 6 {
        for x in 0..18 {
            if !in_ellipse(x, y, (8, h + 3), (8, 2)) {
                continue;
            }
            let rim = !in_ellipse(x, y, (8, h + 3), (7, 1));
            let color = if rim {
                if y <= h + 3 { OAK.light } else { OAK.edge }
            } else if (x - 2) % 4 == 0 {
                OAK.base
            } else if x < 8 {
                OAK.shine
            } else {
                OAK.light
            };
            put(&mut s, x, y, color);
        }
    }
    // A lantern left standing on it, lit after dark (see `lantern_light`).
    lantern(&mut s, BARREL_LANTERN.0, h + BARREL_LANTERN.1);
    (s, (left, top - h), (top + 20) as f32)
}

/// A burlap sack of prizes, open at the top with a bear and a ball looking out of it.
fn sack(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = Canvas::new(26, 20);
    ellipse(&mut s, 13, 18, 12, 2, SHADOW);
    // The far side of its rolled rim, and the dark inside its mouth.
    hline(&mut s, 5, 3, 15, BURLAP.shadow);
    ellipse(&mut s, 12, 4, 6, 1, BURLAP.edge);
    // Prizes standing up out of it: a bear and a ball.
    stamp(&mut s, (6, 1), &TEDDY[..4], &teddy_inks(BEAR), false);
    stamp(
        &mut s,
        (13, 0),
        &[".eoe.", "e*loe", "wwwww", "eooss", ".ese."],
        &[
            (b'e', AWNING_BLUE.edge),
            (b'o', AWNING_BLUE.base),
            (b'l', AWNING_BLUE.light),
            (b'*', AWNING_BLUE.shine),
            (b's', AWNING_BLUE.shadow),
            (b'w', CANVAS_CREAM.light),
        ],
        false,
    );
    // The sack itself: full and slumped at its foot, gathered in towards its mouth, lit from
    // the upper left, the weave of the burlap showing.
    let body = |x: i32, y: i32| {
        in_ellipse(x, y, (12, 12), (10, 5))
            || ((6..=11).contains(&y) && (x - 12).abs() <= 6 + (y - 6) / 2)
    };
    model(&mut s, BURLAP, (0.55, 0.45), 30, 711, body);
    for y in 0..20 {
        for x in 0..26 {
            if body(x, y) && (x + y) % 2 == 0 && chance(x, y, 712, 120) {
                let pixel = s.get(x, y);
                s.set(x, y, mix(pixel, BURLAP.edge, 0.22));
            }
        }
    }
    // Creases running down from the neck, and a patch sewn on.
    line(&mut s, (9, 8), (7, 14), BURLAP.shadow);
    line(&mut s, (16, 8), (18, 13), BURLAP.shadow);
    put(&mut s, 8, 9, BURLAP.light);
    rect(&mut s, 5, 12, 4, 3, mix(BURLAP.base, TWINE.light, 0.45));
    for (x, y) in [(5, 12), (7, 12), (8, 13), (6, 14), (8, 14)] {
        put(&mut s, x, y, TWINE.shadow);
    }
    // The rim, rolled down over itself in front of the prizes.
    for x in 4..=20 {
        let lit = if x < 9 { BURLAP.shine } else { BURLAP.light };
        put(
            &mut s,
            x,
            5,
            if x == 4 || x == 20 { BURLAP.edge } else { lit },
        );
        put(
            &mut s,
            x,
            6,
            if x == 4 || x == 20 {
                BURLAP.edge
            } else {
                BURLAP.base
            },
        );
        put(&mut s, x, 7, mix(BURLAP.edge, BURLAP.shadow, 0.3));
    }
    // A rosette pinned on it, for the best prize of all.
    ellipse(&mut s, 16, 11, 2, 2, CANVAS_RED.edge);
    ellipse(&mut s, 16, 11, 1, 1, CANVAS_RED.light);
    put(&mut s, 16, 11, GOLD.shine);
    put(&mut s, 15, 14, CANVAS_RED.base);
    put(&mut s, 17, 14, CANVAS_RED.shadow);
    put(&mut s, 15, 15, CANVAS_RED.shadow);
    put(&mut s, 17, 15, CANVAS_RED.edge);
    (s, (left, top), (top + 18) as f32)
}

/// A handcart of coconuts for the shy, resting on its wheels with its handles up.
fn handcart(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = Canvas::new(36, 20);
    ellipse(&mut s, 18, 18, 17, 2, SHADOW);
    // The far handle, in shade.
    line(&mut s, (27, 6), (35, 2), PINE.edge);
    // Coconuts for the shy, heaped in the back.
    for (cx, cy, salt) in [(8.0, 3.5, 721), (13.0, 3.0, 722), (18.0, 3.5, 723)] {
        lump(&mut s, cx, cy, 2.6, 2.4, COCONUT, salt);
    }
    // The bed: boards along its side, a rail lit along its top, posts at its corners.
    for y in 4..13 {
        for x in 2..29 {
            let (dx, dy) = (x - 2, y - 4);
            let grain = noise(x / 2, y, 724) % 5;
            let color = if dx == 0 || dx == 26 || dy == 8 {
                PINE.edge
            } else if dy == 0 {
                PINE.shine
            } else if dy == 1 || dx == 1 {
                PINE.light
            } else if dx == 25 || (dx == 2 && dy > 1) {
                PINE.shadow
            } else if dy % 3 == 1 {
                mix(PINE.shadow, PINE.edge, 0.3)
            } else if grain == 0 {
                PINE.shadow
            } else {
                PINE.base
            };
            put(&mut s, x, y, color);
        }
    }
    for x in [2, 27] {
        vline(&mut s, x + 1, 3, 9, PINE.light);
        put(&mut s, x + 1, 3, PINE.shine);
    }
    // The near handle.
    line(&mut s, (28, 8), (35, 4), PINE.base);
    line(&mut s, (28, 9), (35, 5), PINE.edge);
    put(&mut s, 35, 4, PINE.light);
    for wheel in [8.0, 22.0] {
        cart_wheel(&mut s, (wheel + 0.5, 14.0), 4.6);
    }
    (s, (left, top), (top + 18) as f32)
}

/// The hoopla stall's counter, at the front of the stall: its top is where the game's pegs, prizes
/// and rings are set out.
fn counter(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    /// Room above the counter's top, where the game's gear stands.
    const HEADROOM: i32 = 4;
    let h = HEADROOM;
    let mut s = Canvas::new(62, (16 + h + 2) as u32);
    ellipse(&mut s, 31, h + 16, 31, 1, SHADOW);
    // The skirt: blue and cream cloth hanging in folds, darker towards the ground.
    for y in h + 3..h + 16 {
        let down = (y - h - 3) as f32 / 13.0;
        for x in 0..62 {
            let stripe = x / 6;
            let ramp = if stripe % 2 == 0 {
                AWNING_BLUE
            } else {
                CANVAS_CREAM
            };
            let mut color = match x % 6 {
                0 => ramp.light,
                5 => ramp.shadow,
                _ => ramp.base,
            };
            if x == 0 || x == 61 || y == h + 15 {
                color = ramp.edge;
            } else if down > 0.6 {
                color = mix(color, ramp.shadow, (down - 0.6) * 1.5);
            }
            put(&mut s, x, y, color);
        }
    }
    // A gilt valance along the top of the skirt.
    for x in 0..62 {
        let drop = if (1..4).contains(&(x % 4)) { 2 } else { 1 };
        for dy in 0..drop {
            put(
                &mut s,
                x,
                h + 3 + dy,
                if dy == drop - 1 {
                    GOLD.shadow
                } else {
                    GOLD.base
                },
            );
        }
    }
    // The counter top: a lit plank with its grain.
    bevel(&mut s, 0, h, 62, 3, PLANK);
    for x in 2..60 {
        if noise(x / 3, 0, 731).is_multiple_of(4) {
            put(&mut s, x, h + 1, PLANK.base);
        }
    }
    // The rings, pegs and prizes on top are the game's own, set out with it (see `gear`).
    (s, (left, top - h), (top + 16) as f32)
}

// ---------------------------------------------------------------------------------------------
// Brushwork, after the finds' (see `finds::art::brush`)
// ---------------------------------------------------------------------------------------------

/// A number from the hash in `0..n`.
fn pick(index: i32, axis: i32, salt: u32, n: i32) -> i32 {
    (noise(index, axis, salt) % n.max(1) as u32) as i32
}

/// Smooth noise in 0..1 that changes over cells of `size`: patches rather than speckle.
fn patches(x: i32, y: i32, size: (i32, i32), salt: u32) -> f32 {
    let (cell_x, cell_y) = (x.div_euclid(size.0), y.div_euclid(size.1));
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let sx = smooth(x.rem_euclid(size.0) as f32 / size.0 as f32);
    let sy = smooth(y.rem_euclid(size.1) as f32 / size.1 as f32);
    let corner = |dx: i32, dy: i32| (noise(cell_x + dx, cell_y + dy, salt) % 1024) as f32 / 1023.0;
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * sx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// Whether `(x, y)` is inside the ellipse, measured a little generously so that small ones come
/// out round.
fn in_ellipse(x: i32, y: i32, centre: (i32, i32), size: (i32, i32)) -> bool {
    let (rx, ry) = (size.0 as f32 + 0.4, size.1 as f32 + 0.4);
    let (dx, dy) = ((x - centre.0) as f32 / rx, (y - centre.1) as f32 / ry);
    size.0 > 0 && size.1 > 0 && dx * dx + dy * dy <= 1.0
}

/// Paints rows of letters with their top-left at `at`, each letter a colour from `inks`; a
/// letter not in `inks` is left clear. `flip` mirrors it left to right.
fn stamp(s: &mut Canvas, at: (i32, i32), rows: &[&str], inks: &[(u8, Rgba)], flip: bool) {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as i32;
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            let Some(&(_, color)) = inks.iter().find(|(name, _)| *name == letter) else {
                continue;
            };
            let dx = if flip {
                width - 1 - dx as i32
            } else {
                dx as i32
            };
            put(s, at.0 + dx, at.1 + dy as i32, color);
        }
    }
}

/// A tone from `ramp` for a point `(u, v)` on a rounded surface, each from -1 to 1 across it,
/// lit from the upper left; `grain` jitters it so the surface isn't smooth.
fn brush_lit(ramp: Ramp, u: f32, v: f32, grain: u32) -> Rgba {
    let depth = (1.0 - u * u - v * v).max(0.0).sqrt();
    let light = -0.55 * u - 0.62 * v + 0.56 * depth + ((grain % 64) as f32 / 64.0 - 0.5) * 0.18;
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

/// A rounded lump lit from the upper left and outlined in its own edge tone: a coconut, a bear,
/// a ball.
fn lump(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, ramp: Ramp, salt: u32) {
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
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
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let color = if edge {
                ramp.edge
            } else {
                brush_lit(ramp, u, v, noise(x, y, salt))
            };
            put(s, x, y, color);
        }
    }
}

/// Shades the solid that `inside` describes as something rounded, lit from the upper left: how
/// far across and down it a pixel sits stands in for the way the surface there faces, `light`
/// weighing the two. Noise dithers the bands, `grain` in every 256 pixels are a shade darker for
/// texture, and the outermost pixels take the edge tone.
fn model(
    s: &mut Canvas,
    ramp: Ramp,
    light: (f32, f32),
    grain: u32,
    salt: u32,
    inside: impl Fn(i32, i32) -> bool,
) {
    let tones = [ramp.edge, ramp.shadow, ramp.base, ramp.light, ramp.shine];
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !inside(x, y) {
                continue;
            }
            let reach = |dx: i32, dy: i32| {
                let mut n = 0;
                while n < 64 && inside(x + dx * (n + 1), y + dy * (n + 1)) {
                    n += 1;
                }
                n
            };
            let (left, right, up, down) = (reach(-1, 0), reach(1, 0), reach(0, -1), reach(0, 1));
            if left.min(right).min(up).min(down) == 0 {
                put(s, x, y, ramp.edge);
                continue;
            }
            let across = (left - right) as f32 / (left + right) as f32;
            let along = (up - down) as f32 / (up + down) as f32;
            let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
            let lit = -light.0 * across - light.1 * along + 0.3 * jitter;
            let mut level: usize = if lit > 0.75 {
                4
            } else if lit > 0.3 {
                3
            } else if lit > -0.3 {
                2
            } else {
                1
            };
            if chance(x, y, salt.wrapping_add(1), grain) {
                level = (level - 1).max(1);
            }
            put(s, x, y, tones[level]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(
            backdrop(&Arrangement::new())
                .pixels()
                .iter()
                .all(|pixel| pixel.a == 255)
        );
    }

    #[test]
    fn every_hiding_place_has_its_prop() {
        let (props, places) = props();
        assert_eq!(props.len(), places.len());
        for (prop, place) in props.iter().zip(&places) {
            assert!(
                prop.sprite.alpha_bounds().is_some(),
                "{} is invisible",
                place.name
            );
            assert!(
                place.behind.1 < prop.base,
                "{} hides nobody behind it",
                place.name
            );
            assert!(
                place.out_front.1 > prop.base,
                "{} lets nobody out in front",
                place.name
            );
        }
    }

    #[test]
    fn the_tent_door_hides_whoever_slips_inside_altogether() {
        let (door, _, _) = tent_door();
        assert!(door.pixels().iter().all(|pixel| pixel.a == 255));
    }

    #[test]
    fn the_sky_painted_on_its_own_is_the_backdrops_sky() {
        let painted = backdrop(&Arrangement::new());
        let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        sky(&mut only);
        // High over the middle, where no pole, string or roof reaches.
        for y in 0..12 {
            for x in 60..300 {
                assert_eq!(painted.get(x, y), only.get(x, y), "at {x}, {y}");
            }
        }
    }

    #[test]
    fn only_what_shines_is_in_the_lamplight() {
        let lamps = lamplight();
        let lit = lamps.pixels().iter().filter(|pixel| pixel.a > 0).count();
        assert!(lit > 0, "nothing shines");
        assert!(
            lamps.pixels().iter().any(|pixel| pixel.a == 255),
            "no bulb is lit"
        );
        assert!(
            lit < lamps.pixels().len() / 2,
            "{lit} pixels lit: the lamps are lighting everything"
        );
        // Clear sky, and the open sawdust at the front, where nothing hangs.
        assert_eq!(lamps.get(150, 14).a, 0);
        assert_eq!(lamps.get(192, 206).a, 0);
    }
}
