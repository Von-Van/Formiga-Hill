//! The Fairground at dusk, at the level of a grown-up's eye: a violet sky going amber at the
//! horizon, the Hill a silhouette behind, a big top, a carousel and a hoopla stall along the back
//! under strings of lights, and a sawdust ground scattered with things to hide behind.
//!
//! The hiding places are props, drawn among the colony in depth order, so whoever stands behind a
//! hay bale is hidden by it, ears and all if they crouch low enough.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::font::{GLYPH_HEIGHT, draw_text_shadowed, text_width};
use crate::kit::window;
use crate::materials::*;
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

const SKY: [Rgba; 4] = [rgb(0x2e2f5e), rgb(0x5a467c), rgb(0xb5708a), rgb(0xf0a878)];
const DUSK_HILLS: Rgba = rgb(0x3e3150);
const NEAR_HILLS: Rgba = rgb(0x4a3a58);
const SAWDUST: Ramp = Ramp::new(0x6b5034, 0x8f6c48, 0xb08a5c, 0xc8a274, 0xdcbb8c);
const STRAW: Ramp = Ramp::new(0x8a6a28, 0xb8903a, 0xd8b04e, 0xe8c86a, 0xf4e09a);
const CANVAS_RED: Ramp = Ramp::new(0x5e1a24, 0x8a2a30, 0xb83a3c, 0xd0584c, 0xe87a64);
const CANVAS_CREAM: Ramp = Ramp::new(0x8a7a6a, 0xc8b8a0, 0xe8dcc4, 0xf4ecd8, 0xffffff);
const TEAL: Ramp = Ramp::new(0x173c40, 0x22585c, 0x2f7a7a, 0x45999a, 0x6ab8b4);
const AWNING_BLUE: Ramp = Ramp::new(0x1e2c58, 0x2c4480, 0x3a5ea8, 0x5a7ec4, 0x86a6dc);
const GOLD: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const BULB: Rgba = rgb(0xffe7a0);
const GLOW: Rgba = rgba(0xffd08a, 22);
const SHADOW: Rgba = rgba(0x2a1e28, 80);

/// Everything behind the colony and its hiding places.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    ground(&mut scene);
    big_top(&mut scene, BIG_TOP.0, BIG_TOP.1);
    carousel(&mut scene, 196);
    hoopla_stall(&mut scene, 266);
    caravan(&mut scene, 336);
    festoons(&mut scene);
    scene
}

/// Bunting and bulbs nearest the eye, drawn over everyone.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    // Trampled grass along the very front.
    for x in 0..width {
        if chance(x, 0, 601, 120) {
            let tall = 2 + (noise(x, 1, 602) % 4) as i32;
            vline(&mut scene, x, height - tall, tall, rgb(0x4e6a3a));
            put(&mut scene, x, height - tall - 1, rgb(0x6a8a4a));
        }
    }
    scene
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
    add("the hay bales", hay_stack(22, 150), 22.0);
    add("the barrel", barrel(118, 134), 20.0);
    add("the crates", crates(170, 162), 24.0);
    add("the bass drum", drum(236, 132), 19.0);
    add("the prize sack", sack(292, 166), 18.0);
    add("the handcart", handcart(330, 130), 16.0);
    add("the hoopla counter", counter(278, 104), 22.0);
    add("the straw bale", straw_bale(84, 186), 14.0);
    (props, places)
}

// ---------------------------------------------------------------------------------------------
// Sky and ground
// ---------------------------------------------------------------------------------------------

fn sky(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for y in 0..HORIZON + 10 {
        let t = y as f32 / (HORIZON + 10) as f32 * 3.0;
        let band = (t.floor() as usize).min(2);
        let color = mix(
            SKY[band],
            SKY[band + 1],
            ((t - band as f32) * 4.0).floor() / 4.0,
        );
        scene.fill_rect(0, y, width, 1, color);
    }
    // The first stars, and a thin moon.
    for index in 0..26 {
        let x = (noise(index, 0, 611) % width as u32) as i32;
        let y = (noise(index, 1, 612) % 30) as i32;
        put(
            scene,
            x,
            y,
            rgba(0xfff8e0, 120 + (noise(index, 2, 613) % 120) as u8),
        );
    }
    ellipse(scene, 58, 18, 7, 7, rgb(0xf8f0d0));
    ellipse(scene, 61, 16, 6, 6, SKY[0]);
    // The Hill and its neighbours, dark against the glow, the Hill's tree on top.
    scene.fill_ellipse(80, HORIZON + 14, 120, 22, DUSK_HILLS);
    scene.fill_ellipse(300, HORIZON + 6, 90, 34, NEAR_HILLS);
    scene.fill_rect(298, HORIZON - 34, 2, 6, NEAR_HILLS);
    scene.fill_circle(299, HORIZON - 37, 4, NEAR_HILLS);
}

fn ground(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for y in HORIZON..height {
        for x in 0..width {
            let near = (y - HORIZON) as f32 / (height - HORIZON) as f32;
            let base = mix(SAWDUST.shadow, SAWDUST.base, near.min(1.0));
            // Soft clumps, bigger nearer the eye, rather than a speckle.
            let size = 1 + (near * 3.0) as i32;
            let color = match noise(x / (size * 2), y / size, 621) % 11 {
                0 => mix(base, SAWDUST.shadow, 0.3),
                1 => mix(base, SAWDUST.light, 0.22),
                _ => base,
            };
            scene.set(x, y, color);
        }
    }
    // Wisps of straw, a few short strands.
    for index in 0..70 {
        let x = (noise(index, 0, 623) % width as u32) as i32;
        let y = HORIZON + 14 + (noise(index, 1, 624) % (height - HORIZON - 14) as u32) as i32;
        let long = 2 + (noise(index, 2, 625) % 3) as i32;
        let lean = if noise(index, 3, 626).is_multiple_of(2) {
            1
        } else {
            -1
        };
        line(
            scene,
            (x, y),
            (x + long, y + lean),
            mix(STRAW.base, SAWDUST.base, 0.35),
        );
        put(scene, x, y, mix(STRAW.light, SAWDUST.light, 0.3));
    }
    // Trampled grass at the edges, in tufts, worn away where everyone walks.
    for y in HORIZON + 2..height {
        for x in 0..width {
            let edge = x.min(width - 1 - x) as f32;
            if edge < 20.0 && chance(x, y, 622, 40 - (edge * 2.0) as u32) {
                let tall = 1 + (y - HORIZON) / 40;
                vline(scene, x, y - tall, tall + 1, rgb(0x4e6a3a));
                put(scene, x, y - tall, rgb(0x6a8a4a));
            }
        }
    }
    // Pools of lamplight from the strings overhead.
    for (cx, cy) in [(70, 150), (160, 140), (250, 146), (330, 156), (190, 186)] {
        ellipse(scene, cx, cy, 46, 14, GLOW);
        ellipse(scene, cx, cy, 26, 8, GLOW);
    }
}

// ---------------------------------------------------------------------------------------------
// The attractions along the back
// ---------------------------------------------------------------------------------------------

/// A striped big top with its door open and a pennant on the pole.
fn big_top(scene: &mut Canvas, left: i32, base: i32) {
    let (width, eave, peak) = (96, base - 44, base - 78);
    let centre = left + width / 2;
    ellipse(scene, centre + 8, base + 1, width / 2 + 4, 5, SHADOW);
    // The roof: a cone of stripes, darker on its right.
    polygon(
        scene,
        &[(centre, peak), (left + width + 4, eave), (left - 4, eave)],
        |x, y| {
            let across = (x - centre) as f32 / (y - peak + 1) as f32;
            let stripe = ((across * 6.0).floor() as i32).rem_euclid(2) == 0;
            let ramp = if stripe { CANVAS_RED } else { CANVAS_CREAM };
            Some(if x > centre { ramp.shadow } else { ramp.base })
        },
    );
    // The walls, scalloped along the top.
    for y in eave..base {
        for x in left..left + width {
            let stripe = ((x - left) / 8) % 2 == 0;
            let ramp = if stripe { CANVAS_RED } else { CANVAS_CREAM };
            let shade = (x - left) as f32 / width as f32;
            scene.set(x, y, mix(ramp.base, ramp.shadow, shade * 0.7));
        }
    }
    for x in left - 4..left + width + 4 {
        let across = (x - left + 4).rem_euclid(8);
        let drop = if (2..6).contains(&across) { 4 } else { 2 };
        vline(
            scene,
            x,
            eave,
            drop,
            if ((x - left + 4) / 8) % 2 == 0 {
                CANVAS_RED.light
            } else {
                GOLD.base
            },
        );
    }
    // The door: a dark way in, lit from inside.
    let (door_left, door_width) = (centre - 10, 20);
    rect(scene, door_left, base - 28, door_width, 28, rgb(0x2a1a22));
    ellipse(scene, centre, base - 8, 8, 6, rgba(0xffb070, 60));
    polygon(
        scene,
        &[
            (door_left, base - 28),
            (centre, base - 34),
            (door_left + door_width, base - 28),
        ],
        |_, _| Some(CANVAS_RED.shadow),
    );
    // The pole and its pennant.
    vline(scene, centre, peak - 12, 12, GOLD.shadow);
    polygon(
        scene,
        &[
            (centre + 1, peak - 12),
            (centre + 11, peak - 9),
            (centre + 1, peak - 6),
        ],
        |_, _| Some(rgb(0xf5d25e)),
    );
    // A painted sign over the door.
    let sign = "BIG TOP";
    let (sign_width, sign_left) = (text_width(sign) + 6, centre - (text_width(sign) + 6) / 2);
    bevel(
        scene,
        sign_left,
        eave + 6,
        sign_width,
        GLYPH_HEIGHT + 5,
        GOLD,
    );
    rect(
        scene,
        sign_left + 2,
        eave + 8,
        sign_width - 4,
        GLYPH_HEIGHT + 1,
        CANVAS_RED.shadow,
    );
    draw_text_shadowed(scene, sign_left + 3, eave + 9, sign, CREAM, rgb(0x2a1a22));
}

/// A carousel, its striped roof hung with lights and its horses on their brass poles.
fn carousel(scene: &mut Canvas, centre: i32) {
    let (half, roof_top, eave, deck) = (52, 30, 54, 106);
    ellipse(scene, centre + 8, deck + 4, half + 6, 7, SHADOW);
    // The centre column, mirrored.
    rect(scene, centre - 7, eave, 14, deck - eave, rgb(0x6a3a4a));
    for y in (eave + 3..deck).step_by(5) {
        hline(scene, centre - 5, y, 10, rgba(0xfff0d0, 70));
    }
    // The deck.
    ellipse(scene, centre, deck, half, 6, GOLD.shadow);
    ellipse(scene, centre, deck - 1, half - 1, 5, rgb(0x8a5a3a));
    hline(scene, centre - half + 4, deck + 3, half * 2 - 8, GOLD.edge);
    // Poles and horses, the front ones in full view.
    for (index, offset) in [-38, -14, 14, 38].into_iter().enumerate() {
        let x = centre + offset;
        let rise = if index % 2 == 0 { 0 } else { 4 };
        vline(scene, x, eave + 2, deck - eave - 4, GOLD.light);
        vline(scene, x + 1, eave + 2, deck - eave - 4, GOLD.shadow);
        horse(scene, x, deck - 16 - rise, offset > 0);
    }
    // The roof: a cone of teal and cream with a gold scalloped edge and lights along it.
    polygon(
        scene,
        &[
            (centre, roof_top),
            (centre + half + 6, eave),
            (centre - half - 6, eave),
        ],
        |x, y| {
            let across = (x - centre) as f32 / (y - roof_top + 1) as f32;
            let stripe = ((across * 7.0).floor() as i32).rem_euclid(2) == 0;
            let ramp = if stripe { TEAL } else { CANVAS_CREAM };
            Some(if x > centre + 8 {
                ramp.shadow
            } else {
                ramp.base
            })
        },
    );
    for x in centre - half - 6..centre + half + 6 {
        let across = (x - centre + half + 6).rem_euclid(6);
        let drop = if (1..5).contains(&across) { 4 } else { 2 };
        vline(scene, x, eave, drop, GOLD.base);
        put(scene, x, eave, GOLD.light);
    }
    for x in (centre - half - 2..centre + half + 4).step_by(8) {
        bulb(scene, x, eave + 5);
    }
    vline(scene, centre, roof_top - 10, 10, GOLD.light);
    polygon(
        scene,
        &[
            (centre + 1, roof_top - 10),
            (centre + 9, roof_top - 7),
            (centre + 1, roof_top - 4),
        ],
        |_, _| Some(CANVAS_RED.base),
    );
}

/// A carousel horse in profile, saddled, facing whichever way its side of the ride turns.
fn horse(scene: &mut Canvas, x: i32, top: i32, facing_right: bool) {
    let flip = |dx: i32| if facing_right { x + dx } else { x - dx };
    let white = Ramp::new(0x8a8090, 0xc8c0c8, 0xeee8e8, 0xfaf6f4, 0xffffff);
    // Body, neck, head, legs mid-gallop, and a red saddle.
    for dy in 0..6 {
        for dx in -6..6 {
            put(
                scene,
                flip(dx),
                top + 6 + dy,
                if dy > 3 { white.shadow } else { white.base },
            );
        }
    }
    for (dx, dy) in [(5, 5), (6, 4), (7, 3), (8, 2), (8, 1), (9, 1), (10, 2)] {
        put(scene, flip(dx), top + dy, white.base);
        put(scene, flip(dx), top + dy + 1, white.base);
    }
    put(scene, flip(9), top + 1, rgb(0x2a1a22));
    for (dx, dy) in [(-7, 6), (-8, 7), (-9, 9)] {
        put(scene, flip(dx), top + dy, rgb(0xd8a060));
    }
    for (dx, length) in [(-5, 5), (-3, 4), (3, 5), (5, 4)] {
        vline(scene, flip(dx), top + 12, length, white.shadow);
    }
    // The saddle, centred on the pole.
    rect(scene, x - 2, top + 5, 5, 2, CANVAS_RED.base);
    put(scene, x, top + 7, GOLD.light);
}

/// A hoopla stall: a striped awning, prizes on the shelves behind, a painted sign. Its counter is
/// a prop, so someone can hide behind it.
fn hoopla_stall(scene: &mut Canvas, left: i32) {
    let (width, roof, base) = (72, 66, 104);
    ellipse(
        scene,
        left + width / 2 + 6,
        base + 2,
        width / 2 + 4,
        4,
        SHADOW,
    );
    rect(
        scene,
        left,
        roof + 10,
        width,
        base - roof - 10,
        rgb(0x3a2a3a),
    );
    // Shelves of prizes: little bears, bottles and pegs to throw rings at.
    for (row, y) in [roof + 16, roof + 26].into_iter().enumerate() {
        hline(scene, left + 3, y + 6, width - 6, PLANK.light);
        for slot in 0..8 {
            let x = left + 6 + slot * 8;
            match (slot + row as i32) % 3 {
                0 => {
                    ellipse(scene, x + 2, y + 3, 2, 3, rgb(0xc8905a));
                    ellipse(scene, x + 2, y, 2, 2, rgb(0xd8a06a));
                    put(scene, x + 1, y, rgb(0x2a1a22));
                }
                1 => {
                    rect(scene, x + 1, y, 3, 6, rgb(0x5a9a6a));
                    put(scene, x + 2, y - 1, rgb(0x5a9a6a));
                    put(scene, x + 1, y + 1, rgba(0xffffff, 140));
                }
                _ => {
                    vline(scene, x + 2, y + 1, 5, PLANK.base);
                    ellipse(scene, x + 2, y + 5, 3, 1, CANVAS_RED.light);
                }
            }
        }
    }
    // Posts and a striped, scalloped awning.
    for post in [left, left + width - 3] {
        rect(scene, post, roof, 3, base - roof, PLANK.base);
        vline(scene, post, roof, base - roof, PLANK.light);
    }
    for y in roof..roof + 10 {
        for x in left - 3..left + width + 3 {
            let stripe = ((x - left + 3) / 6) % 2 == 0;
            let ramp = if stripe { AWNING_BLUE } else { CANVAS_CREAM };
            let scallop = (x - left + 3).rem_euclid(6);
            if y >= roof + 8 && !(1..5).contains(&scallop) {
                continue;
            }
            scene.set(x, y, if y == roof { ramp.light } else { ramp.base });
        }
    }
    let sign = "HOOPLA";
    let sign_left = left + (width - text_width(sign) - 6) / 2;
    bevel(
        scene,
        sign_left,
        roof - 12,
        text_width(sign) + 6,
        GLYPH_HEIGHT + 5,
        GOLD,
    );
    rect(
        scene,
        sign_left + 2,
        roof - 10,
        text_width(sign) + 2,
        GLYPH_HEIGHT + 1,
        AWNING_BLUE.edge,
    );
    draw_text_shadowed(scene, sign_left + 3, roof - 9, sign, CREAM, rgb(0x0e1430));
    for x in (left..left + width).step_by(9) {
        bulb(scene, x + 2, roof + 12);
    }
}

/// A painted caravan at the far end, its window lit.
fn caravan(scene: &mut Canvas, left: i32) {
    let (width, top, base) = (44, 70, 110);
    ellipse(
        scene,
        left + width / 2 + 6,
        base + 2,
        width / 2 + 2,
        4,
        SHADOW,
    );
    polygon(
        scene,
        &[
            (left, top + 8),
            (left + width / 2, top),
            (left + width, top + 8),
        ],
        |_, _| Some(TEAL.shadow),
    );
    bevel(
        scene,
        left,
        top + 8,
        width,
        base - top - 14,
        Ramp::new(0x4a1e2a, 0x6a2a3a, 0x8a3a4a, 0xa85060, 0xc87080),
    );
    window(scene, left + 14, top + 14, 16, 12, 2);
    rect(scene, left + 16, top + 16, 12, 8, rgba(0xffc070, 120));
    for wheel in [left + 8, left + width - 8] {
        ellipse(scene, wheel, base - 4, 5, 5, GOLD.edge);
        ellipse(scene, wheel, base - 4, 4, 4, GOLD.base);
        put(scene, wheel, base - 4, GOLD.shine);
        line(
            scene,
            (wheel - 3, base - 4),
            (wheel + 3, base - 4),
            GOLD.shadow,
        );
        line(scene, (wheel, base - 7), (wheel, base - 1), GOLD.shadow);
    }
}

/// Strings of bulbs from the big top's pole to the carousel and on to the stall.
fn festoons(scene: &mut Canvas) {
    let strings = [
        ((60, 34), (196, 26), 10.0),
        ((196, 26), (300, 50), 12.0),
        ((300, 50), (384, 44), 8.0),
    ];
    for (from, to, sag) in strings {
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
                rgb(0x2a1e28),
            );
            previous = point;
        }
        for index in 1..12 {
            let (x, y) = at(index as f32 / 12.0);
            bulb(scene, x.round() as i32, y.round() as i32 + 2);
        }
    }
}

fn bulb(scene: &mut Canvas, x: i32, y: i32) {
    ellipse(scene, x, y, 4, 4, GLOW);
    put(scene, x, y, BULB);
    put(scene, x, y + 1, mix(BULB, GOLD.base, 0.4));
}

// ---------------------------------------------------------------------------------------------
// Hiding places
// ---------------------------------------------------------------------------------------------

/// A sprite of `width` by `height` with its top-left at `at`, meeting the ground `base_from_top`
/// rows down.
fn sprite(width: u32, height: u32) -> Canvas {
    Canvas::new(width, height)
}

/// The front of the big top around its door, with the flap let down: whoever slips inside is
/// hidden altogether, and only the flap stirring gives them away.
fn tent_door() -> (Canvas, (i32, i32), f32) {
    let (left, base) = BIG_TOP;
    let mut tent = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    big_top(&mut tent, left, base);
    // Cut along stripe edges, below the scallops, so a shake barely shows.
    let (crop_left, crop_top) = (left + 24, base - 39);
    let (width, height) = (48, 39);
    let mut s = sprite(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let pixel = tent.get(crop_left + x, crop_top + y);
            if pixel.a > 0 {
                s.set(x, y, pixel);
            }
        }
    }
    // The flap, hanging in folds across the doorway, a little open at the foot.
    let (door_left, door_top) = (left + 38 - crop_left, base - 28 - crop_top);
    for y in door_top..height {
        for x in door_left..door_left + 20 {
            if y > height - 6 && (door_left + 8..door_left + 12).contains(&x) {
                continue;
            }
            let ramp = if ((x - door_left) / 5) % 2 == 0 {
                CANVAS_RED
            } else {
                CANVAS_CREAM
            };
            put(
                &mut s,
                x,
                y,
                if (x - door_left) % 5 == 4 {
                    ramp.shadow
                } else {
                    ramp.base
                },
            );
        }
    }
    hline(&mut s, door_left, door_top, 20, GOLD.base);
    (s, (crop_left, crop_top), base as f32)
}

fn hay_stack(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(38, 24);
    ellipse(&mut s, 21, 22, 18, 2, SHADOW);
    for (x, y, w, h) in [(0, 10, 18, 12), (18, 10, 18, 12), (8, 0, 20, 11)] {
        bale(&mut s, x, y, w, h);
    }
    (s, (left, top), (top + 22) as f32)
}

fn straw_bale(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(30, 16);
    ellipse(&mut s, 16, 14, 14, 2, SHADOW);
    bale(&mut s, 0, 1, 28, 13);
    (s, (left, top), (top + 14) as f32)
}

fn bale(s: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    bevel(s, x, y, width, height, STRAW);
    for py in y + 1..y + height - 1 {
        for px in x + 1..x + width - 1 {
            if chance(px, py, 631, 50) {
                put(s, px, py, STRAW.shadow);
            } else if chance(px, py, 632, 30) {
                put(s, px, py, STRAW.shine);
            }
        }
    }
    for band in [x + width / 3, x + width * 2 / 3] {
        vline(s, band, y, height, rgb(0x7a4a2a));
    }
}

fn barrel(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(18, 22);
    ellipse(&mut s, 10, 20, 8, 2, SHADOW);
    for y in 2..20 {
        let bulge = if (6..15).contains(&y) { 1 } else { 0 };
        for x in 1 - bulge..17 + bulge {
            let across = (x - 8) as f32 / 9.0;
            let base = mix(PLANK.base, PLANK.shadow, across.max(0.0));
            let lit = if across < -0.4 {
                mix(base, PLANK.light, 0.5)
            } else {
                base
            };
            put(&mut s, x, y, if x % 4 == 0 { PLANK.shadow } else { lit });
        }
    }
    for hoop in [4, 15] {
        hline(&mut s, 0, hoop, 18, IRON.base);
        hline(&mut s, 0, hoop + 1, 18, IRON.edge);
    }
    ellipse(&mut s, 8, 2, 8, 2, PLANK.edge);
    ellipse(&mut s, 8, 2, 7, 1, PLANK.light);
    (s, (left, top), (top + 20) as f32)
}

fn crates(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(30, 26);
    ellipse(&mut s, 16, 24, 14, 2, SHADOW);
    for (x, y, w) in [(0, 12, 14), (14, 12, 14), (6, 0, 15)] {
        bevel(&mut s, x, y, w, 12, PLANK);
        line(&mut s, (x + 1, y + 1), (x + w - 2, y + 10), PLANK.shadow);
        hline(&mut s, x + 1, y + 6, w - 2, PLANK.shadow);
    }
    (s, (left, top), (top + 24) as f32)
}

fn drum(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(24, 22);
    ellipse(&mut s, 13, 20, 11, 2, SHADOW);
    // On its side: the shell, and the skin facing out with its painted star.
    rect(&mut s, 2, 2, 12, 17, CANVAS_RED.base);
    for x in (3..14).step_by(3) {
        line(&mut s, (x, 2), (x + 2, 18), GOLD.base);
    }
    ellipse(&mut s, 15, 10, 7, 9, GOLD.edge);
    ellipse(&mut s, 15, 10, 6, 8, CANVAS_CREAM.light);
    for (dx, dy) in [(0, -3), (-2, 1), (2, 1), (0, 0), (-1, -1), (1, -1)] {
        put(&mut s, 15 + dx, 10 + dy, rgb(0x3a5ea8));
    }
    (s, (left, top), (top + 20) as f32)
}

fn sack(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(26, 20);
    ellipse(&mut s, 13, 18, 12, 2, SHADOW);
    let burlap = Ramp::new(0x6a5030, 0x8a6a42, 0xb08a58, 0xc8a270, 0xdcbb8c);
    ellipse(&mut s, 12, 11, 11, 7, burlap.edge);
    ellipse(&mut s, 12, 11, 10, 6, burlap.base);
    ellipse(&mut s, 10, 9, 6, 3, burlap.light);
    // Prizes spilling from the top: a bear's ear, a ball.
    ellipse(&mut s, 7, 4, 3, 3, rgb(0xc8905a));
    put(&mut s, 6, 3, rgb(0x2a1a22));
    ellipse(&mut s, 17, 4, 3, 3, rgb(0x4a7fc4));
    put(&mut s, 16, 3, rgb(0xb0d0f4));
    (s, (left, top), (top + 18) as f32)
}

fn handcart(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    let mut s = sprite(36, 20);
    ellipse(&mut s, 18, 18, 16, 2, SHADOW);
    bevel(&mut s, 2, 4, 26, 9, PLANK);
    hline(&mut s, 3, 8, 24, PLANK.shadow);
    line(&mut s, (28, 6), (35, 2), PLANK.edge);
    for wheel in [8, 22] {
        ellipse(&mut s, wheel, 14, 4, 4, IRON.edge);
        ellipse(&mut s, wheel, 14, 3, 3, PLANK.base);
        put(&mut s, wheel, 14, GOLD.light);
    }
    (s, (left, top), (top + 18) as f32)
}

/// The hoopla stall's counter, at the front of the stall, with rings stacked ready on top.
fn counter(left: i32, top: i32) -> (Canvas, (i32, i32), f32) {
    const HEADROOM: i32 = 4;
    let mut s = sprite(62, 16 + HEADROOM as u32);
    bevel(&mut s, 0, HEADROOM, 62, 3, PLANK);
    for y in HEADROOM + 3..HEADROOM + 16 {
        for x in 1..61 {
            let stripe = (x / 6) % 2 == 0;
            put(
                &mut s,
                x,
                y,
                if stripe {
                    AWNING_BLUE.base
                } else {
                    CANVAS_CREAM.base
                },
            );
        }
    }
    hline(&mut s, 1, HEADROOM + 15, 60, AWNING_BLUE.edge);
    for ring in 0..3 {
        let y = HEADROOM - 1 - ring;
        hline(
            &mut s,
            9,
            y,
            7,
            if ring % 2 == 0 {
                CANVAS_RED.light
            } else {
                GOLD.light
            },
        );
        put(&mut s, 8, y, CANVAS_RED.shadow);
        put(&mut s, 16, y, CANVAS_RED.shadow);
    }
    (s, (left, top - HEADROOM), (top + 16) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(backdrop().pixels().iter().all(|pixel| pixel.a == 255));
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
}
