//! The pool in the Woods, close up from its near bank and a little from above, so the water fills
//! most of the picture: the mossy rock ledge across the far side with the waterfall coming down
//! its middle, the woods rising behind it into the canopy with the sun slanting through from the
//! upper left, lily pads out to the left, reeds and bulrushes up the right shore, the shallows
//! shelving up to a pebbly beach at the front left, the deep water dark off to the right, and the
//! grassy near bank where a companion sits on a flat stone to fish.
//!
//! The water is kept a calm mid-tone, the far bank mirrored in it and the light soft on it, so
//! the fish, drawn over it as dark shadows, always show; no fish are painted here. It is the
//! glade's palette, so the pool belongs to the same woods, and nothing is blown out to white, so
//! it can be tinted towards dusk with the rest.

use super::{Haunt, NET, PLACES, SEAT, WATER};
use crate::kit;
use crate::materials::LEATHER;
use crate::paint::{
    Ramp, chance, ellipse, hline, line, looped, mix, noise, patches, pick, polygon, put, rgb, rgba,
    tone, vline,
};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

/// A point, or a size, in whole pixels.
type Pixels = (i32, i32);

// The glade's materials, so the pool is plainly in the same woods.
const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const CANOPY: Ramp = Ramp::new(0x142a24, 0x1d3a30, 0x28503c, 0x3a6848, 0x56865a);
const UNDERGROWTH: Ramp = Ramp::new(0x1c3628, 0x284a32, 0x36603c, 0x4c7a48, 0x6a9658);
const HAZEL_LEAF: Ramp = Ramp::new(0x29441f, 0x3a5c2a, 0x4f7834, 0x6a9442, 0x8cb058);
const LITTER: Ramp = Ramp::new(0x382a1c, 0x54402e, 0x6e5238, 0x886a48, 0xa2845c);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const GRASS: Ramp = Ramp::new(0x2a4224, 0x3a5a2e, 0x507438, 0x6a8e46, 0x88a85a);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const REED: Ramp = Ramp::new(0x34442a, 0x4c5e36, 0x667a44, 0x829654, 0xa2b06c);
const BULRUSH: Ramp = Ramp::new(0x2a1a12, 0x43291c, 0x5c3a26, 0x765032, 0x8f6640);

// The pool's own.
/// The water, a mid-tone at its base so a dark shadow beneath it always shows.
const POOL: Ramp = Ramp::new(0x1e3a3c, 0x2d5554, 0x41706b, 0x5a8b83, 0x7ea99c);
/// Falling water, lighter than the pool for the air churned into it.
const FALLS: Ramp = Ramp::new(0x2c5450, 0x4a7a72, 0x6a9a8e, 0x92bcae, 0xb4d4c6);
/// Stone the falls keep wet: darker, and glistening.
const WET_ROCK: Ramp = Ramp::new(0x262f2d, 0x384441, 0x4e5a55, 0x68746c, 0x8a968a);
const LILY: Ramp = Ramp::new(0x24461e, 0x35602a, 0x4c7c34, 0x68983f, 0x8cb658);
const PETAL: Ramp = Ramp::new(0x9a6474, 0xcc96a4, 0xe6bec2, 0xf0d8d4, 0xf2e2da);
const WICKER: Ramp = Ramp::new(0x5a4026, 0x7e5c36, 0x9e7a48, 0xbc9a5e, 0xd6b67a);
const JAR: Ramp = Ramp::new(0x3e5a66, 0x587a86, 0x7c9ea6, 0xa6c4c8, 0xcfe4e2);
const FOAM: Rgba = rgb(0xc8dcd2);
const SAND: Rgba = rgb(0x8c8462);
const SUN: u32 = 0xffe2a2;
const SHADE: Rgba = rgba(0x14241c, 72);

/// Shafts of sun: where each crosses the top of the picture, its half-width, where it comes out
/// of the canopy, and where it lands. One comes down on the water by the lily pads, the other on
/// the ledge beside the falls.
const BEAMS: [(f32, f32, i32, i32); 2] = [(50.0, 8.0, 8, 126), (146.0, 6.0, 8, 52)];
/// How far right a sunbeam travels for every pixel it falls.
const SLANT: f32 = 0.4;
/// Where the falls come over the ledge, and how wide they are at the top and where they land.
const FALLS_TOP: i32 = 20;
const FALLS_FOOT: i32 = 68;
const FALLS_SPAN: (Pixels, Pixels) = ((197, 211), (194, 214));
/// The foot of the tackle basket, and of the jar of worms beside it.
const BASKET: Pixels = (240, 184);
const WORMS: Pixels = (257, 187);

/// Everything behind the companions and the fish, filling every pixel.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    woods(&mut scene);
    ground(&mut scene);
    far_bank(&mut scene);
    outcrop(&mut scene);
    let land = scene.clone();
    water(&mut scene, &land);
    shallows(&mut scene);
    waterline(&mut scene);
    tree_roots(&mut scene);
    falls_foot(&mut scene);
    lilies(&mut scene);
    shore(&mut scene);
    near_bank(&mut scene);
    seat(&mut scene);
    tackle(&mut scene);
    reeds(&mut scene);
    sunbeams(&mut scene);
    scene
}

/// What is nearer the eye than anyone: leaves hanging into the top corners and grass standing up
/// in the bottom ones. Transparent everywhere else, and always clear of the water, the seat and
/// the net.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    leaf_spray(
        &mut scene,
        (-8, -6),
        &[(10, 5), (26, 9), (42, 6), (56, 1)],
        900,
    );
    leaf_spray(
        &mut scene,
        (WIDTH + 8, -6),
        &[(WIDTH - 12, 6), (WIDTH - 28, 8), (WIDTH - 44, 3)],
        920,
    );
    front_grass(&mut scene);
    scene
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

// The air, fronds and leaves are drawn as the glade draws them, so the pool matches the woods
// round it; they are the glade's own, kept private there.

/// The air between the trees at height `y`: shade under the canopy, brightening to a haze just
/// above the far bank, where the woods go out of sight.
fn air(y: i32) -> Rgba {
    const STOPS: [(i32, u32); 5] = [
        (0, 0x1e382f),
        (14, 0x335a4b),
        (32, 0x5f8a7c),
        (48, 0x8fb0a2),
        (60, 0x7a9c8e),
    ];
    let mut previous = STOPS[0];
    for stop in STOPS {
        if y <= stop.0 {
            let t = (y - previous.0) as f32 / (stop.0 - previous.0).max(1) as f32;
            return mix(rgb(previous.1), rgb(stop.1), t.clamp(0.0, 1.0));
        }
        previous = stop;
    }
    rgb(previous.1)
}

/// `color` as seen through `depth` of the haze (0 is nearby, 1 lost in it).
fn hazed(color: Rgba, y: i32, depth: f32) -> Rgba {
    mix(color, air(y), depth)
}

/// A ramp's tone at a level that may fall between two of its steps, mixed in proportion.
fn tone_between(ramp: Ramp, level: f32) -> Rgba {
    let level = level.clamp(0.0, 4.0);
    let low = level.floor() as i32;
    mix(tone(ramp, low), tone(ramp, low + 1), level - low as f32)
}

/// A haunt's middle and reach, in whole pixels.
fn haunt(haunt: Haunt) -> (i32, i32, i32) {
    let ((x, y), r) = haunt.area();
    (x as i32, y as i32, r as i32)
}

/// The broken outline of an ellipse, for ripples.
fn ring(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), color: Rgba, salt: u32) {
    let mut done: Vec<(i32, i32)> = Vec::new();
    let steps = (rx + ry) * 4;
    for step in 0..steps {
        let angle = step as f32 / steps as f32 * TAU;
        let point = (
            cx + (angle.cos() * rx as f32).round() as i32,
            cy + (angle.sin() * ry as f32).round() as i32,
        );
        if done.contains(&point) || chance(step, rx, salt, 90) {
            continue;
        }
        done.push(point);
        put(scene, point.0, point.1, color);
    }
}

/// A clump of leaves, its rim scalloped into smaller clusters, lit from the upper left and lost
/// in the haze by `depth`.
fn clump(scene: &mut Canvas, centre: (i32, i32), radius: i32, ramp: Ramp, depth: f32, salt: u32) {
    kit::clump(scene, centre, radius, ramp, salt, |color, y| {
        hazed(color, y, depth)
    });
}

/// A stone lit from the upper left, its outline a little lumpy and flat underneath where it sits.
/// `square` rounds it (2) or squares it into a slab with a flat lit top and a seam across its
/// face (3 or more); `moss` covers its crown (0 bare, 1 smothered); the falls keep it `wet`,
/// darker and glistening.
fn stone(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    (rx, ry): (i32, i32),
    square: f32,
    moss: f32,
    wet: bool,
    salt: u32,
) {
    let (frx, fry) = (rx.max(1) as f32, ry.max(1) as f32);
    let reach = |u: f32, v: f32| u.abs().powf(square) + v.abs().powf(square);
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x - cx) as f32 / frx, (y - cy) as f32 / fry);
        let around = (v.atan2(u) / TAU + 0.5) * 9.0;
        let lump = 0.88 + 0.16 * looped(around, 9, salt);
        reach(u, v) <= lump.powf(square) && v < 0.9
    };
    let ramp = if wet { WET_ROCK } else { ROCK };
    // Where the seam runs across a slab's face.
    let seam = cy + ry / 3 + pick(cx, cy, salt + 6, 3) - 1;
    for y in cy - ry - 1..=cy + ry + 1 {
        for x in cx - rx - 1..=cx + rx + 1 {
            if !inside(x, y) {
                continue;
            }
            let rim =
                !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            let (u, v) = ((x - cx) as f32 / frx, (y - cy) as f32 / fry);
            let height = (1.0 - reach(u, v).min(1.0)).powf(1.0 / square);
            let bend = |t: f32| t.signum() * t.abs().powf(square - 1.0);
            let lit = -0.6 * bend(u) - 0.8 * bend(v) + 0.5 * height;
            let mut level = if lit > 0.95 {
                4
            } else if lit > 0.45 {
                3
            } else if lit > -0.05 {
                2
            } else {
                1
            };
            // Facets and pits, so it reads as stone rather than a ball.
            match noise(x.div_euclid(2), y.div_euclid(2), salt + 1) % 9 {
                0 => level += 1,
                1 | 2 => level -= 1,
                _ => {}
            }
            if level >= 4 && !chance(x, y, salt + 2, 80) {
                level = 3;
            }
            let mut color = if rim {
                ramp.edge
            } else {
                tone(ramp, level.clamp(1, 4))
            };
            if square > 2.5
                && !rim
                && y == seam + i32::from(u > 0.3)
                && patches(x, 0, (4, 1), salt + 7) > 0.3
            {
                color = mix(color, ramp.edge, 0.6);
            }
            // Moss on the crown, ragged at its edge, thickest where the light falls.
            let crown = v + (patches(x, y, (3, 2), salt + 3) - 0.5) * 0.8 + u * 0.2;
            if crown < -1.05 + moss * 1.4 {
                color = if rim {
                    MOSS.edge
                } else {
                    tone(MOSS, (level - 1).clamp(1, 4))
                };
            } else if !rim && chance(x, y, salt + 4, 12) {
                // Lichen.
                color = mix(color, rgb(0xb4b67c), 0.45);
            }
            if wet && !rim && level >= 3 && chance(x, y, salt + 5, 14) {
                color = FALLS.light;
            }
            put(scene, x, y, color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The woods behind the pool
// ---------------------------------------------------------------------------------------------

/// The air, the trees in four ranks from the haze forwards, each rank with its own canopy and
/// undergrowth, and the near canopy with the sun breaking through it.
fn woods(scene: &mut Canvas) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // Stepped in bands with dithered edges, like paint rather than a smooth fade.
            let level = (y + pick(x, y, 11, 5) - 2).div_euclid(4) * 4;
            scene.set(x, y, air(level));
        }
    }
    // Pockets of sunlit haze, where the beams come down.
    for (cx, cy, rx, ry) in [(70, 42, 44, 16), (200, 40, 60, 18), (316, 44, 34, 12)] {
        ellipse(scene, cx, cy, rx, ry, rgba(0xdce4b0, 12));
        ellipse(
            scene,
            cx,
            cy + 3,
            rx * 3 / 5,
            ry * 3 / 5,
            rgba(0xdce4b0, 14),
        );
    }
    for (index, x) in (-4..WIDTH + 4).step_by(11).enumerate() {
        let i = index as i32;
        trunk(
            scene,
            x + pick(i, 0, 21, 7) - 3,
            2 + pick(i, 1, 21, 2),
            50 + pick(i, 2, 21, 3),
            0.72,
            200 + index as u32,
        );
    }
    canopy_row(scene, 14, 17, (7, 10), 0.64, 300);
    undergrowth_row(scene, 47, 8, (3, 5), 0.6, 310);
    let middling = [
        (4, 4),
        (52, 5),
        (74, 4),
        (120, 5),
        (162, 4),
        (186, 5),
        (238, 4),
        (266, 5),
        (324, 4),
        (370, 5),
    ];
    for (index, &(x, width)) in middling.iter().enumerate() {
        trunk(
            scene,
            x,
            width,
            51 + index as i32 % 3,
            0.48,
            400 + index as u32,
        );
    }
    canopy_row(scene, 10, 21, (9, 12), 0.44, 500);
    undergrowth_row(scene, 50, 10, (4, 6), 0.4, 510);
    for (index, &(x, width)) in [(38, 8), (136, 7), (222, 8), (312, 9)].iter().enumerate() {
        trunk(
            scene,
            x,
            width,
            53 + index as i32 % 2,
            0.26,
            600 + index as u32,
        );
    }
    canopy_row(scene, 6, 25, (11, 15), 0.22, 700);
    undergrowth_row(scene, 53, 12, (4, 7), 0.2, 710);
    for (index, &(x, width)) in [(98, 12), (284, 13), (350, 15)].iter().enumerate() {
        trunk(scene, x, width, 58, 0.06, 800 + index as u32);
    }
    near_canopy(scene);
}

/// One tree's trunk, rising out of the top of the picture to stand at `base`, lit from the left,
/// its bark furrowed and mossy towards the foot.
fn trunk(scene: &mut Canvas, left: i32, width: i32, base: i32, depth: f32, salt: u32) {
    let lean = pick(left, 0, salt, 3) - 1;
    for y in 0..base {
        let shift = lean * (base - y) / 50;
        let flare = ((y - (base - 5)).max(0) * width) / 12;
        let (from, to) = (left + shift - flare, left + shift + width + flare);
        let span = (to - from).max(1);
        let height = (base - y) as f32 / base as f32;
        for x in from..to {
            let across = (x - from) as f32 / span as f32;
            let mut color = if x == from || x == to - 1 {
                BARK.edge
            } else if across < 0.28 {
                BARK.light
            } else if across < 0.62 {
                BARK.base
            } else {
                BARK.shadow
            };
            if width >= 6 && x > from && x < to - 1 {
                // Furrows run up the bark in broken streaks.
                let streak = noise(x, (y + pick(x, 1, salt, 9)).div_euclid(5), salt);
                if streak.is_multiple_of(6) {
                    color = mix(color, BARK.edge, 0.7);
                } else if streak % 11 == 1 && across < 0.6 {
                    color = mix(color, BARK.shine, 0.6);
                }
                // Moss creeping up from the foot, thickest on the lit side.
                if height < 0.5
                    && patches(x, y, (3, 6), salt + 1) > 0.36 + height * 1.0 + across * 0.3
                {
                    color = if across < 0.3 {
                        MOSS.light
                    } else if across < 0.65 {
                        MOSS.base
                    } else {
                        MOSS.shadow
                    };
                }
            }
            scene.set(x, y, hazed(color, y, depth));
        }
    }
}

/// A rank of the canopy, its underside at about `y`, filled solid above so no air shows through.
fn canopy_row(
    scene: &mut Canvas,
    y: i32,
    spacing: usize,
    radius: (i32, i32),
    depth: f32,
    salt: u32,
) {
    for py in 0..y - radius.0 + 2 {
        for x in 0..WIDTH {
            let color = if chance(x.div_euclid(2), py.div_euclid(2), salt, 60) {
                CANOPY.base
            } else {
                CANOPY.shadow
            };
            scene.set(x, py, hazed(color, py, depth));
        }
    }
    for (index, x) in (-20..WIDTH + 20).step_by(spacing).enumerate() {
        let i = index as i32;
        let r = radius.0 + pick(i, 0, salt, radius.1 - radius.0 + 1);
        clump(
            scene,
            (x + pick(i, 1, salt, 7) - 3, y + pick(i, 2, salt, 7) - 4),
            r,
            CANOPY,
            depth,
            salt + index as u32 * 7,
        );
    }
}

/// A rank of low bushes along the woods floor, and the shade beneath them.
fn undergrowth_row(
    scene: &mut Canvas,
    y: i32,
    spacing: usize,
    radius: (i32, i32),
    depth: f32,
    salt: u32,
) {
    for py in y + 1..y + 6 {
        for x in 0..WIDTH {
            scene.set(x, py, hazed(UNDERGROWTH.edge, py, depth));
        }
    }
    for (index, x) in (-8..WIDTH + 8).step_by(spacing).enumerate() {
        let i = index as i32;
        let r = radius.0 + pick(i, 0, salt, radius.1 - radius.0 + 1);
        clump(
            scene,
            (x + pick(i, 1, salt, 5) - 2, y + pick(i, 2, salt, 5) - 2),
            r,
            UNDERGROWTH,
            depth,
            salt + index as u32 * 5,
        );
    }
}

/// The canopy nearest the eye, across the top of the picture, broken where the sun comes through.
fn near_canopy(scene: &mut Canvas) {
    let gaps: Vec<i32> = BEAMS
        .iter()
        .map(|&(origin, _, top, _)| (origin + SLANT * top as f32) as i32)
        .collect();
    for (index, x) in (-16..WIDTH + 16).step_by(22).enumerate() {
        let i = index as i32;
        let cx = x + pick(i, 1, 1001, 7) - 3;
        if gaps.iter().any(|gap| (cx - gap).abs() < 12) {
            continue;
        }
        clump(
            scene,
            (cx, pick(i, 2, 1001, 5) - 1),
            13 + pick(i, 0, 1001, 6),
            CANOPY,
            0.0,
            1010 + index as u32 * 7,
        );
    }
    // Sunlit leaves high up, glimpsed in flecks through the gaps, with the near leaves dark
    // against them.
    for (index, &gap) in gaps.iter().enumerate() {
        let salt = 1100 + index as u32 * 10;
        ellipse(scene, gap, 4, 16, 8, rgba(0xe8e2a0, 16));
        ellipse(scene, gap, 3, 10, 5, rgba(0xe8e2a0, 18));
        for fleck in 0..14 {
            let spread = 5 + fleck / 2;
            let x = gap + pick(fleck, 0, salt, spread * 2 + 1) - spread;
            let y = pick(fleck, 1, salt, 5 + fleck / 3);
            let (rx, ry) = (1 + pick(fleck, 2, salt, 3), 1 + pick(fleck, 3, salt, 2));
            let warm = fleck < 6;
            ellipse(
                scene,
                x,
                y,
                rx,
                ry,
                if warm { rgb(0xd4d496) } else { rgb(0xa4bc7c) },
            );
            if warm {
                put(scene, x, y, rgb(0xe6e2aa));
            }
        }
        for leaf in 0..6 {
            let x = gap + pick(leaf, 4, salt, 19) - 9;
            let y = 1 + pick(leaf, 5, salt, 8);
            ellipse(scene, x, y, 2, 1, CANOPY.shadow);
            put(scene, x - 1, y - 1, CANOPY.base);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The ground round the pool
// ---------------------------------------------------------------------------------------------

/// Where the ground begins at the back, in front of the undergrowth.
fn bank_top(x: i32) -> i32 {
    let fx = x as f32;
    (56.5 + 1.4 * (fx * 0.05).sin() + 0.8 * (fx * 0.13 + 1.0).sin()).round() as i32
}

/// How near the eye a row of the ground is: 0 at the back of the pool, 1 at the bottom of the
/// picture.
fn nearness(y: i32) -> f32 {
    ((y - 56) as f32 / 160.0).clamp(0.0, 1.0)
}

/// Whether the ground is mossy at `(x, y)`: soft cushions, ragged at their edges.
fn mossy(x: i32, y: i32) -> bool {
    patches(x, y, (28, 9), 702) + (noise(x.div_euclid(2), y, 703) % 100) as f32 / 800.0 > 0.72
}

/// Whether the grass gives way to bare earth and fallen leaves at `(x, y)`: here and there, more
/// of it nearer the eye.
fn bare(x: i32, y: i32) -> bool {
    patches(x, y, (30, 9), 709) + nearness(y) * 0.2 + (noise(x, y, 712) % 100) as f32 / 1400.0
        > 0.88
}

/// The ground all round the pool: a sward of woodland grass, lit and shaded in soft patches,
/// cushions of moss and bare earth strewn with leaves in it, going into the haze at the back.
/// The water is laid over it.
fn ground(scene: &mut Canvas) {
    for x in 0..WIDTH {
        for y in bank_top(x)..HEIGHT {
            let near = nearness(y);
            let sward = patches(x, y, (24, 8), 704) * 0.6
                + patches(x, y, (7, 3), 705) * 0.4
                + (noise(x, y, 706) % 100) as f32 / 700.0
                - 0.1 * near;
            let mut color = if sward < 0.34 {
                GRASS.shadow
            } else if sward < 0.5 {
                mix(GRASS.shadow, GRASS.base, 0.5)
            } else if sward < 0.68 {
                GRASS.base
            } else {
                mix(GRASS.base, GRASS.light, 0.5)
            };
            if bare(x, y) {
                let soil = patches(x, y, (9, 4), 713) + (noise(x, y, 714) % 100) as f32 / 600.0;
                color = if !bare(x, y - 1) {
                    mix(GRASS.shadow, LITTER.shadow, 0.5)
                } else if soil < 0.42 {
                    LITTER.shadow
                } else if soil < 0.74 {
                    mix(LITTER.shadow, LITTER.base, 0.6)
                } else {
                    LITTER.base
                };
            }
            if mossy(x, y) {
                let cushion = patches(x, y, (4, 3), 707);
                color = if !mossy(x, y - 1) && chance(x, y, 708, 150) {
                    MOSS.light
                } else if !mossy(x, y + 1) {
                    MOSS.edge
                } else if !mossy(x, y + 2) || cushion < 0.25 {
                    MOSS.shadow
                } else if cushion > 0.72 {
                    mix(MOSS.base, MOSS.light, 0.6)
                } else {
                    MOSS.base
                };
            }
            // The far ground goes into the haze.
            let haze = (1.0 - near * 3.5).max(0.0) * 0.3;
            scene.set(x, y, mix(color, air(50), haze));
        }
    }
    // Blades standing up out of the sward: taller and further apart nearer the eye.
    let mut row = 58;
    while row < HEIGHT + 4 {
        let near = nearness(row);
        let (step_x, step_y) = (2 + (near * 3.0) as i32, 1 + (near * 2.5) as i32);
        for (index, gx) in (-2..WIDTH + 2).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 710, step_x);
            let y = row + pick(i, row + 1, 710, step_y);
            let sparse = if bare(x, y) { 30 } else { 130 };
            if y < bank_top(x) + 1 || !chance(x, y, 711, sparse) || mossy(x, y) {
                continue;
            }
            let tall = 1 + (near * 3.2) as i32 + pick(i, row + 2, 710, 2);
            let lean = if tall > 2 {
                pick(i, row + 3, 710, 3) - 1
            } else {
                0
            };
            let (body, tip) = match pick(i, row + 4, 710, 3) {
                0 => (GRASS.edge, GRASS.shadow),
                1 => (GRASS.shadow, GRASS.light),
                _ => (GRASS.light, GRASS.shine),
            };
            let haze = (1.0 - near * 3.5).max(0.0) * 0.3;
            line(
                scene,
                (x, y),
                (x + lean, y - tall + 1),
                mix(body, air(50), haze),
            );
            put(scene, x + lean, y - tall + 1, mix(tip, air(50), haze));
        }
        row += step_y;
    }
    // Fallen leaves on the bare earth, bigger nearer the eye.
    let fallen = [
        LITTER.base,
        rgb(0x7a5832),
        rgb(0x6a4430),
        rgb(0x84703e),
        rgb(0x5e4430),
    ];
    let mut row = 90;
    while row < HEIGHT + 2 {
        let near = nearness(row);
        let (step_x, step_y) = (3 + (near * 4.0) as i32, 2 + (near * 1.6) as i32);
        for (index, gx) in (-4..WIDTH + 4).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 715, step_x);
            let y = row + pick(i, row + 1, 715, step_y);
            if !bare(x, y) || !chance(x, y, 716, 170) {
                continue;
            }
            let long = 1 + (near * 1.8) as i32 + pick(i, row + 2, 715, 2);
            let color = fallen[pick(i, row + 3, 715, fallen.len() as i32) as usize];
            hline(scene, x - long + 1, y, long * 2 - 1, color);
            if long > 1 {
                hline(
                    scene,
                    x - long + 2,
                    y - 1,
                    long * 2 - 3,
                    mix(color, LITTER.shine, 0.3),
                );
            }
            hline(
                scene,
                x - long + 2,
                y + 1,
                (long * 2 - 2).max(1),
                rgba(0x1e140e, 100),
            );
        }
        row += step_y;
    }
}

/// The far bank: bushes along its top, ferns among them, and the low ledge of slabs it makes
/// along the water either side of the outcrop the falls come down.
fn far_bank(scene: &mut Canvas) {
    let mut x = -6;
    let mut index = 0;
    while x < WIDTH + 6 {
        let radius = 4 + pick(index, 0, 1201, 6);
        let lift = pick(index, 2, 1201, 3) + radius / 3;
        let ramp = if pick(index, 3, 1201, 3) == 0 {
            HAZEL_LEAF
        } else {
            UNDERGROWTH
        };
        if pick(index, 4, 1201, 5) != 0 {
            clump(
                scene,
                (x, bank_top(x) - lift + 2),
                radius,
                ramp,
                0.1,
                1210 + index as u32 * 3,
            );
        }
        if index % 3 == 1 {
            for (angle, length) in [(-130.0, 9.0), (-90.0, 11.0), (-50.0, 9.0)] {
                frond(scene, (x + radius + 2, bank_top(x) + 2), (angle, length), 2);
            }
        }
        x += radius + 3 + pick(index, 1, 1201, 5);
        index += 1;
    }
    // The low ledge, back slabs first: (centre, half-size, moss).
    let slabs: [(Pixels, Pixels, f32); 16] = [
        ((50, 63), (10, 5), 0.6),
        ((74, 61), (12, 6), 0.5),
        ((100, 60), (11, 5), 0.6),
        ((124, 60), (12, 6), 0.4),
        ((284, 60), (12, 6), 0.5),
        ((308, 61), (12, 6), 0.6),
        ((332, 64), (10, 6), 0.5),
        ((38, 72), (11, 7), 0.5),
        ((62, 71), (13, 7), 0.4),
        ((88, 68), (12, 6), 0.5),
        ((112, 67), (11, 6), 0.3),
        ((134, 66), (12, 7), 0.5),
        ((274, 66), (12, 7), 0.4),
        ((300, 68), (12, 6), 0.5),
        ((324, 71), (12, 7), 0.4),
        ((348, 76), (11, 7), 0.6),
    ];
    for (index, (centre, size, moss)) in slabs.into_iter().enumerate() {
        ellipse(scene, centre.0 + 2, centre.1 + size.1 - 1, size.0, 2, SHADE);
        stone(
            scene,
            centre,
            size,
            3.0,
            moss,
            false,
            1230 + index as u32 * 3,
        );
    }
    // Ferns between the slabs, spilling forward.
    for (index, (x, y)) in [
        (32, 66),
        (62, 60),
        (112, 58),
        (296, 59),
        (322, 63),
        (44, 80),
    ]
    .into_iter()
    .enumerate()
    {
        let lean = if index % 2 == 0 { -1.0 } else { 1.0 };
        for (angle, length, level) in [
            (-90.0 - 40.0 * lean, 10.0, 1),
            (-90.0 + 10.0 * lean, 12.0, 2),
            (-90.0 + 50.0 * lean, 10.0, 2),
            (-90.0 - 70.0 * lean, 8.0, 3),
        ] {
            frond(scene, (x, y), (angle, length), level);
        }
    }
    left_tree(scene);
}

/// Where the near tree on the left shore stands on the ground.
const TREE_FOOT: i32 = 98;

/// The near tree on the left shore, standing right at the water: its trunk rising out of the top
/// of the picture, mostly in its own shade as the light comes from beyond it, its bark furrowed
/// and mossy towards the foot.
fn left_tree(scene: &mut Canvas) {
    let left = -14;
    for y in 0..=TREE_FOOT {
        let fy = y as f32;
        let flare = ((fy - 60.0).max(0.0) / 38.0).powi(3) * 18.0;
        let right = (15.0 + (fy * 0.06).sin() * 1.2 + flare).round() as i32;
        let height = (TREE_FOOT - y) as f32 / TREE_FOOT as f32;
        for x in 0..=right {
            let across = (x - left) as f32 / (right - left) as f32;
            let mut color = if x == right {
                BARK.edge
            } else if across < 0.6 {
                BARK.base
            } else if across < 0.85 {
                BARK.shadow
            } else {
                mix(BARK.shadow, BARK.edge, 0.5)
            };
            if x < right {
                let streak = noise(x, (y + pick(x, 1, 1260, 9)).div_euclid(5), 1260);
                if streak.is_multiple_of(5) {
                    color = mix(color, BARK.edge, 0.7);
                } else if streak % 13 == 1 && across < 0.75 {
                    color = BARK.light;
                }
                if patches(x, y, (3, 6), 1261) > 0.3 + height * 1.5 + across * 0.25 {
                    color = if across < 0.6 {
                        MOSS.base
                    } else if across < 0.85 {
                        MOSS.shadow
                    } else {
                        MOSS.edge
                    };
                }
            }
            scene.set(x, y, color);
        }
    }
    ellipse(scene, 18, TREE_FOOT + 1, 16, 3, SHADE);
}

/// The tree's roots, swelling out of its foot and gripping the bank before they go under it,
/// the lowest reaching down to the end of the pool.
fn tree_roots(scene: &mut Canvas) {
    for (index, &(from, bend, to, thick)) in [
        ((16, 90), (26, 88), (32, 92), 7.0),
        ((10, 96), (18, 100), (21, 106), 8.0),
    ]
    .iter()
    .enumerate()
    {
        root(scene, from, bend, to, (thick, 2.0), 1262 + index as u32 * 2);
    }
}

/// A root swelling from `from`, curving by way of `bend` to `to`, `thick` pixels deep where it
/// leaves the trunk and `thin` at its end: lit along its top and mossy here and there, and where
/// it dips into the water, the water folding round it.
fn root(
    scene: &mut Canvas,
    from: (i32, i32),
    bend: (i32, i32),
    to: (i32, i32),
    (thick, thin): (f32, f32),
    salt: u32,
) {
    let along = |t: f32, a: i32, b: i32, c: i32| {
        (1.0 - t) * (1.0 - t) * a as f32 + 2.0 * (1.0 - t) * t * b as f32 + t * t * c as f32
    };
    // Its middle and half-thickness every half pixel or so along it.
    let spine: Vec<(f32, f32, f32)> = (0..=80)
        .map(|step| {
            let t = step as f32 / 80.0;
            (
                along(t, from.0, bend.0, to.0),
                along(t, from.1, bend.1, to.1),
                (thick + (thin - thick) * t.sqrt()) / 2.0,
            )
        })
        .collect();
    let covers = |x: i32, y: i32, (cx, cy, r): (f32, f32, f32)| {
        let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
        dx * dx + dy * dy <= r * r
    };
    // Outlined all round, then filled, lit along its top and towards the upper left.
    for &(cx, cy, r) in &spine {
        let reach = r.ceil() as i32 + 1;
        for y in cy as i32 - reach..=cy as i32 + reach {
            for x in cx as i32 - reach..=cx as i32 + reach {
                if covers(x, y, (cx, cy, r)) {
                    scene.set(x, y, BARK.edge);
                }
            }
        }
    }
    for &(cx, cy, r) in &spine {
        let reach = r.ceil() as i32;
        for y in cy as i32 - reach..=cy as i32 + reach {
            for x in cx as i32 - reach..=cx as i32 + reach {
                if !covers(x, y, (cx, cy, r - 1.0)) {
                    continue;
                }
                let lit = ((x as f32 + 0.5 - cx) * 0.5 + (y as f32 + 0.5 - cy)) / r.max(1.0);
                let mut color = if lit < -0.25 {
                    BARK.light
                } else if lit < 0.45 {
                    BARK.base
                } else {
                    BARK.shadow
                };
                if lit < -0.25 && chance(x, y, salt, 60) {
                    color = MOSS.base;
                } else if chance(x.div_euclid(3), y, salt + 1, 26) {
                    color = mix(color, BARK.edge, 0.5);
                }
                scene.set(x, y, color);
            }
        }
    }
    // Where it dips into the water, the water folds round it.
    for &(cx, cy, r) in spine.iter().step_by(4) {
        let (x, below) = (cx as i32, (cy + r).round() as i32 + 1);
        if is_water(x, below) {
            put(scene, x, below, rgba(0x14282a, 110));
            if chance(x, 0, salt + 2, 120) {
                hline(scene, x + 1, below + 1, 2, rgba(0xa4c8bc, 110));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The outcrop and the falls
// ---------------------------------------------------------------------------------------------

/// The rocky outcrop in the middle of the far bank: boulders and slabs heaped up to a long lip
/// the stream comes over, a big rounded boulder on the left and the rest stepping down to the
/// right, wet and dark beside the falls, moss on their tops and hanging from their lips, ferns
/// spilling over the top, and the falls coming down its face.
fn outcrop(scene: &mut Canvas) {
    // The dark mass behind the stones, so the gaps between them read as crevices.
    polygon(
        scene,
        &[
            (124, 70),
            (132, 50),
            (146, 32),
            (158, 22),
            (172, 15),
            (222, 17),
            (236, 21),
            (250, 29),
            (266, 40),
            (282, 70),
        ],
        |x, y| {
            Some(if chance(x, y, 1301, 40) {
                ROCK.shadow
            } else {
                mix(ROCK.edge, EARTH.edge, 0.3)
            })
        },
    );
    // Back to front: (centre, half-size, squareness, moss, wet).
    let stones: [(Pixels, Pixels, f32, f32, bool); 21] = [
        ((236, 24), (12, 6), 3.2, 0.8, false),
        ((256, 32), (9, 5), 3.0, 0.7, false),
        ((171, 23), (14, 11), 2.3, 0.9, false),
        ((205, 19), (19, 5), 3.6, 0.5, false),
        ((154, 39), (11, 9), 2.6, 0.6, false),
        ((182, 39), (8, 10), 2.4, 0.3, true),
        ((227, 37), (10, 7), 2.6, 0.3, true),
        ((249, 43), (13, 7), 3.2, 0.6, false),
        ((268, 47), (9, 6), 3.0, 0.5, false),
        ((141, 52), (12, 8), 3.0, 0.5, false),
        ((166, 54), (10, 7), 2.8, 0.4, false),
        ((188, 56), (6, 7), 2.4, 0.2, true),
        ((221, 54), (8, 8), 2.4, 0.2, true),
        ((239, 57), (10, 7), 3.0, 0.4, false),
        ((262, 58), (12, 7), 3.2, 0.5, false),
        ((130, 62), (11, 6), 3.0, 0.4, false),
        ((153, 64), (13, 6), 3.0, 0.4, false),
        ((172, 66), (8, 5), 2.6, 0.3, false),
        ((232, 66), (9, 5), 2.6, 0.3, false),
        ((251, 65), (10, 5), 3.0, 0.4, false),
        ((276, 64), (12, 6), 3.0, 0.5, false),
    ];
    for (index, (centre, size, square, moss, wet)) in stones.into_iter().enumerate() {
        stone(
            scene,
            centre,
            size,
            square,
            moss,
            wet,
            1310 + index as u32 * 3,
        );
    }
    // The stream coming to the lip along the channel it has worn, narrowing away into the
    // shade under the trees, smooth and quickening as it comes.
    let ((top_left, top_right), _) = FALLS_SPAN;
    for y in FALLS_TOP - 5..FALLS_TOP {
        let inset = (FALLS_TOP - 2 - y).max(0);
        let (left, right) = (top_left + inset, top_right - inset);
        for x in left..=right {
            let color = if x == left {
                ROCK.edge
            } else if x == right {
                WET_ROCK.edge
            } else if y == FALLS_TOP - 5 {
                mix(POOL.edge, ROCK.edge, 0.4)
            } else if x == left + 2 + (FALLS_TOP - y) / 2 {
                FALLS.light
            } else if y < FALLS_TOP - 3 {
                POOL.shadow
            } else if y < FALLS_TOP - 1 {
                POOL.base
            } else {
                FALLS.base
            };
            scene.set(x, y, color);
        }
    }
    // Ferns spilling over the top, either side of where the water comes over.
    for (index, &(x, y)) in [(160, 14), (178, 13), (226, 18), (244, 19)]
        .iter()
        .enumerate()
    {
        let lean = if x < 204 { -1.0 } else { 1.0 };
        for (angle, length, level) in [
            (-90.0 + 60.0 * lean, 9.0 + index as f32 % 2.0, 1),
            (-90.0 + 20.0 * lean, 11.0, 2),
            (-90.0 + 100.0 * lean, 9.0, 3),
        ] {
            frond(scene, (x, y), (angle, length), level);
        }
    }
    // Moss hanging in curtains from the lips of the stones.
    for (index, &(x, y, long)) in [
        (160, 31, 6),
        (166, 33, 9),
        (176, 33, 5),
        (232, 30, 5),
        (240, 30, 3),
        (148, 47, 5),
        (152, 47, 3),
        (252, 49, 6),
        (170, 60, 4),
        (236, 62, 4),
    ]
    .iter()
    .enumerate()
    {
        for dy in 0..long {
            let shade = if dy == long - 1 {
                MOSS.edge
            } else if dy < 2 {
                MOSS.light
            } else {
                MOSS.base
            };
            put(scene, x, y + dy, shade);
            if dy < long - 2 && pick(index as i32, dy, 1340, 3) == 0 {
                put(scene, x + 1, y + dy, MOSS.shadow);
            }
        }
    }
    curtain(scene, FALLS_TOP, FALLS_FOOT);
}

/// The edges of the falls at row `y`, wavering a little as the sheet of water twists.
fn falls_edges(y: i32) -> (i32, i32) {
    let ((top_left, top_right), (foot_left, foot_right)) = FALLS_SPAN;
    let t = (y - FALLS_TOP) as f32 / (FALLS_FOOT - FALLS_TOP) as f32;
    let waver = |salt: u32| ((looped(y as f32 / 5.0, 64, salt) - 0.5) * 2.4 * t).round() as i32;
    (
        (top_left as f32 + (foot_left - top_left) as f32 * t).round() as i32 + waver(1604),
        (top_right as f32 + (foot_right - top_right) as f32 * t).round() as i32 + waver(1605),
    )
}

/// The falls between rows `from` and `to`: a sheet of water coming over the lip, glassy at the
/// top, breaking into white streaks as it falls, lit on its left and in shade on its right.
fn curtain(scene: &mut Canvas, from: i32, to: i32) {
    for y in from..to {
        let t = (y - FALLS_TOP) as f32 / (FALLS_FOOT - FALLS_TOP) as f32;
        let (left, right) = falls_edges(y);
        // Its shade on the wet rock behind, to the right.
        put(scene, right + 1, y, rgba(0x101a18, 90));
        for x in left..=right {
            let color = if x == left {
                FALLS.shadow
            } else if x == right {
                FALLS.edge
            } else if y == FALLS_TOP {
                // Smooth and bright where it bends over the lip.
                FALLS.shine
            } else if y < FALLS_TOP + 4 {
                if x < left + 3 {
                    FALLS.shine
                } else if x > right - 3 {
                    FALLS.base
                } else {
                    FALLS.light
                }
            } else {
                let fall = (y + pick(x, 0, 1601, 9)).div_euclid(5);
                let mut level = match noise(x, fall, 1602) % 7 {
                    0 => 5,
                    1 | 2 => 4,
                    3 | 4 => 3,
                    _ => 2,
                } + i32::from(x < left + 4)
                    - i32::from(x > right - 4);
                // More of it is white the further it has fallen.
                if chance(x, fall, 1603, (t * 80.0) as u32) {
                    level += 1;
                }
                if level >= 5 { FOAM } else { tone(FALLS, level) }
            };
            scene.set(x, y, color);
        }
    }
}

/// Where the falls land: the curtain's foot in the pool, a mound of foam churning there, lit on
/// its top, the spray over it, and the rings and streaks of foam it sends out across the pool.
fn falls_foot(scene: &mut Canvas) {
    let (fx, fy, _) = haunt(Haunt::Falls);
    // Rings spreading out across the pool, fainter as they go.
    for (index, (rx, ry)) in [(26, 5), (35, 7), (45, 9), (56, 11)]
        .into_iter()
        .enumerate()
    {
        ring(
            scene,
            (fx, fy - 3 + index as i32),
            (rx, ry),
            rgba(0xa4c8bc, 110 - index as u8 * 22),
            1610 + index as u32,
        );
    }
    curtain(scene, FALLS_TOP + 30, FALLS_FOOT);
    // The mound of foam, in clots, lit on top and shaded where it meets the water, which it
    // darkens a little beneath it.
    let (cx, cy, rx, ry) = (fx, FALLS_FOOT, 15.0f32, 4.5f32);
    hline(scene, cx - 12, cy + 5, 25, rgba(0x1e3a3c, 70));
    for y in cy - 5..=cy + 5 {
        for x in cx - 16..=cx + 16 {
            let (u, v) = ((x - cx) as f32 / rx, (y - cy) as f32 / ry);
            // Lumpy along its top, flatter beneath.
            let lumps = (noise(x.div_euclid(3), 0, 1621) % 100) as f32 / 160.0 * (-v).max(0.0);
            let r = u * u + v * v + lumps;
            if r > 1.0 {
                continue;
            }
            let clot = noise(x.div_euclid(2), y, 1622) % 10;
            let lit = v - u * 0.4;
            let color = if r > 0.8 && v > 0.2 {
                FALLS.shadow
            } else if lit < -0.35 {
                match clot {
                    0..=6 => FOAM,
                    _ => FALLS.shine,
                }
            } else if lit < 0.35 {
                match clot {
                    0..=3 => FOAM,
                    4..=6 => FALLS.shine,
                    _ => FALLS.light,
                }
            } else {
                match clot {
                    0 => FALLS.shine,
                    1..=5 => FALLS.light,
                    _ => FALLS.base,
                }
            };
            put(scene, x, y, color);
        }
    }
    // Clots of foam breaking away from it, and streaks carried off on the current.
    for index in 0..26 {
        let spread = 14 + index / 2;
        let x = fx + pick(index, 0, 1630, spread * 2 + 1) - spread;
        let y = FALLS_FOOT - 1 + pick(index, 1, 1630, 6 + index / 6);
        let long = 1 + pick(index, 2, 1630, 3);
        hline(
            scene,
            x,
            y,
            long,
            if index % 3 == 0 { FALLS.shine } else { FOAM },
        );
        hline(scene, x, y + 1, long, rgba(0x2c5450, 60));
    }
    for index in 0..12 {
        let side = if index % 2 == 0 { -1 } else { 1 };
        let x = fx + side * (18 + pick(index, 0, 1631, 24));
        let y = fy - 1 + pick(index, 1, 1631, 8);
        hline(
            scene,
            x,
            y,
            3 + pick(index, 2, 1631, 5),
            rgba(0xc8dcd2, 100),
        );
    }
    // Spray drifting up over the foot of the falls.
    ellipse(scene, fx, FALLS_FOOT - 5, 16, 6, rgba(0xc8dcd2, 22));
    for index in 0..22 {
        let x = fx + pick(index, 0, 1640, 33) - 16;
        let y = FALLS_FOOT - 3 - pick(index, 1, 1640, 12);
        put(scene, x, y, rgba(0xdce8e2, 100));
    }
}

// ---------------------------------------------------------------------------------------------
// The water
// ---------------------------------------------------------------------------------------------

/// How far out from the middle of the pool the middle of pixel `(x, y)` is, with the rim of the
/// open water at 1, and its angle round from the right.
fn polar(x: i32, y: i32) -> (f32, f32) {
    let (cx, cy, rx, ry) = WATER;
    let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
    ((u * u + v * v).sqrt(), v.atan2(u))
}

/// How far out the shore is at `angle`: always a little beyond the open water, in coves and
/// points along the near and far sides, and close in at the ends so the pool stays in the picture.
fn shore_reach(angle: f32) -> f32 {
    let around = (angle / TAU + 0.5) * 120.0;
    let side = angle.sin().powi(2);
    1.02 + (0.025 + 0.05 * side) * looped(around / 5.0, 24, 31) + 0.014 * looped(around, 120, 32)
}

fn is_water(x: i32, y: i32) -> bool {
    let (reach, angle) = polar(x, y);
    reach < shore_reach(angle)
}

/// The first and last rows of water in each column, for the columns that have any.
fn columns() -> Vec<Option<(i32, i32)>> {
    (0..WIDTH).map(columns_at).collect()
}

/// The first and last rows of water in column `x`, if it has any.
fn columns_at(x: i32) -> Option<(i32, i32)> {
    let mut rows = (0..HEIGHT).filter(|&y| is_water(x, y));
    let first = rows.next()?;
    Some((first, rows.next_back().unwrap_or(first)))
}

/// How light the pool is at `(x, y)`, in steps of `POOL`: lighter far off, where it mirrors the
/// haze, and darker near the eye, darkest over the deep, lightest over the shallows and in the
/// churn below the falls, with soft light drifting across it.
fn pool_light(x: i32, y: i32) -> f32 {
    let (fx, fy) = (x as f32, y as f32);
    let mut level = 3.0 - (fy - 62.0) / 100.0 * 0.8;
    level += (patches(x, y, (56, 18), 801) - 0.5) * 0.6;
    // The pool is a bowl, shelving up towards its rim.
    let (reach, _) = polar(x, y);
    level -= (1.0 - reach * reach).max(0.0) * 0.35;
    level -= deepness(x, y) * 0.85;
    let (sx, sy, sr) = haunt(Haunt::Shallows);
    let (u, v) = (
        (fx - sx as f32) / (sr as f32 * 1.8),
        (fy - sy as f32 - 4.0) / (sr as f32 * 0.8),
    );
    let shallow = 1.0 - (u * u + v * v).sqrt();
    if shallow > 0.0 {
        level += (shallow * 1.8).min(1.0) * 0.9;
    }
    let (fx0, fy0, fr) = haunt(Haunt::Falls);
    let (u, v) = (
        (fx - fx0 as f32) / (fr as f32 * 1.4),
        (fy - fy0 as f32) / (fr as f32 * 0.5),
    );
    let churn = 1.0 - (u * u + v * v).sqrt();
    if churn > 0.0 {
        level += churn * 0.9;
    }
    level
}

/// How far into the deep water `(x, y)` is, from 0 at its edge to 1, an uneven hollow in the bed
/// rather than a ring.
fn deepness(x: i32, y: i32) -> f32 {
    let (dx, dy, dr) = haunt(Haunt::Deep);
    let (u, v) = (
        (x - dx) as f32 / (dr as f32 * 1.9),
        (y - dy) as f32 / (dr as f32 * 0.72),
    );
    let deep = 1.0 - (u * u + v * v).sqrt() - (patches(x, y, (24, 9), 806) - 0.5) * 0.6;
    let fall = (deep * 1.5).clamp(0.0, 1.0);
    fall * fall * (3.0 - 2.0 * fall)
}

/// The open water: stepped into bands with broken edges, the far bank and the falls mirrored in
/// it, wavering, wavelets on its calm surface, and a few leaves riding it.
fn water(scene: &mut Canvas, land: &Canvas) {
    for (x, column) in (0..WIDTH).zip(columns()) {
        let Some((far, near)) = column else {
            continue;
        };
        for y in far..=near {
            if !is_water(x, y) {
                continue;
            }
            // Stepped in quarters of the ramp, the steps' edges broken a couple of pixels at a
            // time along the water.
            let scatter = (noise(x.div_euclid(2), y, 802) % 100) as f32 / 100.0 - 0.5;
            let level = ((pool_light(x, y) + scatter * 0.32) * 4.0).round() / 4.0;
            let mut color = tone_between(POOL, level.clamp(0.75, 3.5));
            color = mix(color, rgb(0x1c3a48), deepness(x, y) * 0.22);
            // The far bank mirrored, strongest at the far shore, where the water is seen most
            // nearly edge on, the trees above it faint further out, and gone by the middle;
            // broken by the faintest ripples.
            let d = (y - far) as f32;
            let mut strength = 0.62 * (1.0 - d / 26.0).max(0.0).powf(1.3)
                + 0.22 * (1.0 - d / 64.0).max(0.0).powf(1.5);
            if noise((x + y * 3).div_euclid(9), y, 803).is_multiple_of(5) {
                strength *= 0.35;
            }
            if strength > 0.01 {
                let sway =
                    ((y as f32 * 0.9 + x as f32 * 0.04).sin() * (0.5 + d / 9.0)).round() as i32;
                let mirrored = land.get(x + sway, far - 1 - (y - far));
                if mirrored.a > 0 {
                    color = mix(color, mix(mirrored, POOL.shadow, 0.3), strength);
                }
            }
            scene.set(x, y, color);
        }
        // A thin bright line where the water laps the far bank.
        if chance(x, 0, 805, 150) {
            put(scene, x, far, rgba(0xa4c8bc, 90));
        }
    }
    wavelets(scene);
    // A few leaves riding the water, apart from one another and away from where the fish keep.
    let mut floating: Vec<Pixels> = Vec::new();
    for index in 0..40 {
        let (x, y) = (
            40 + pick(index, 0, 807, WIDTH - 80),
            70 + pick(index, 1, 807, 90),
        );
        let clear = Haunt::ALL.iter().all(|&kind| {
            let (hx, hy, reach) = haunt(kind);
            let (u, v) = (
                (x - hx) as f32 / reach as f32,
                (y - hy) as f32 / reach as f32,
            );
            u * u + v * v * 4.0 > 1.6
        });
        let apart = floating
            .iter()
            .all(|&(fx, fy)| (fx - x).abs() > 30 || (fy - y).abs() > 14);
        if floating.len() == 5 || !clear || !apart || !is_water(x - 2, y) || !is_water(x + 5, y + 2)
        {
            continue;
        }
        floating.push((x, y));
        let color = [rgb(0x9a6a34), rgb(0x84482c), rgb(0xa8843e)][index as usize % 3];
        hline(scene, x, y, 3, color);
        put(scene, x, y, mix(color, rgb(0xd6b07a), 0.4));
        hline(scene, x + 1, y + 1, 2, rgba(0x1a3034, 90));
        put(scene, x + 3, y, rgba(0xa4c4b8, 120));
    }
}

/// The calm surface: long low wavelets catching the light, each a bright dash with its shade
/// beneath it, more of them where the light is and fewer over the deep.
fn wavelets(scene: &mut Canvas) {
    let mut row = 60;
    while row < HEIGHT {
        let near = nearness(row);
        let step_x = 9 + (near * 10.0) as i32;
        for (index, gx) in (0..WIDTH).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 811, step_x);
            let y = row + pick(i, row + 1, 811, 2);
            let long = 2 + pick(i, row + 2, 811, 3) + (near * 4.0) as i32;
            if !is_water(x - 2, y) || !is_water(x + long + 2, y + 2) {
                continue;
            }
            let odds = ((pool_light(x, y) - 1.7) * 70.0).clamp(12.0, 120.0) as u32;
            if !chance(x, y, 812, odds) {
                continue;
            }
            hline(scene, x, y, long, rgba(0x8cb4a6, 130));
            put(scene, x + long / 2, y, rgba(0xa4c8bc, 120));
            hline(scene, x + 1, y + 1, long - 1, rgba(0x1e3a3c, 55));
        }
        row += 3;
    }
}

/// The shallows: pebbles on the bed showing through, clearer towards the shore, and stones
/// breaking the surface with the water folding round them.
fn shallows(scene: &mut Canvas) {
    let (sx, sy, sr) = haunt(Haunt::Shallows);
    let pebbles = [
        rgb(0x7a7466),
        rgb(0x8c7a5e),
        rgb(0x6a706a),
        rgb(0x9a8e74),
        rgb(0x5e5a52),
    ];
    // The sandy bed shelving up to the shore.
    for y in sy - sr..sy + sr + 12 {
        for x in sx - sr * 2..sx + sr * 2 {
            if !is_water(x, y) {
                continue;
            }
            let (u, v) = (
                (x - sx) as f32 / (sr as f32 * 1.9),
                (y - sy - 6) as f32 / (sr as f32 * 0.9),
            );
            let near = 1.0 - (u * u + v * v).sqrt();
            if near <= 0.0 {
                continue;
            }
            let amount = (near * 1.4).min(1.0) * 0.36;
            if !chance(x.div_euclid(2), y, 1501, ((1.0 - near) * 120.0) as u32) {
                put(scene, x, y, rgba(0x8c8462, (amount * 255.0) as u8));
            }
        }
    }
    for index in 0..70 {
        let x = sx - sr * 2 + 4 + pick(index, 0, 1502, sr * 4 - 8);
        let y = sy - sr / 2 + pick(index, 1, 1502, sr + 14);
        if !is_water(x, y) || !is_water(x + 2, y + 1) {
            continue;
        }
        let (u, v) = (
            (x - sx) as f32 / (sr as f32 * 1.9),
            (y - sy - 6) as f32 / (sr as f32 * 0.9),
        );
        let near = 1.0 - (u * u + v * v).sqrt();
        if near < 0.1 {
            continue;
        }
        let water = scene.get(x, y);
        let color = mix(
            pebbles[pick(index, 2, 1502, 5) as usize],
            water,
            (0.6 - near * 0.5).clamp(0.15, 0.55),
        );
        let long = 2 + pick(index, 3, 1502, 2);
        hline(scene, x, y, long, color);
        hline(scene, x + 1, y - 1, long - 2, mix(color, POOL.shine, 0.35));
        put(scene, x, y - 1, mix(color, POOL.shine, 0.2));
        hline(scene, x + 1, y + 1, long - 1, mix(water, POOL.edge, 0.3));
    }
    // Stones breaking the surface near the shore.
    for (index, (centre, size)) in [
        ((34, 138), (4, 3)),
        ((88, 152), (3, 2)),
        ((46, 147), (3, 2)),
    ]
    .into_iter()
    .enumerate()
    {
        hline(
            scene,
            centre.0 - size.0,
            centre.1 + size.1,
            size.0 * 2 + 1,
            rgba(0x1a3034, 100),
        );
        stone(scene, centre, size, 2.0, 0.3, false, 1510 + index as u32);
        hline(
            scene,
            centre.0 + size.0 + 1,
            centre.1 + 1,
            4,
            rgba(0xc8dcd2, 140),
        );
        hline(
            scene,
            centre.0 - size.0 - 3,
            centre.1 + 1,
            2,
            rgba(0xc8dcd2, 100),
        );
    }
}

/// Stones at the water's edge, half in and half out, breaking the line of the shore: in front of
/// the ledge, at the ends and along the near bank, clear of the seat and the net.
fn waterline(scene: &mut Canvas) {
    let stones: [((i32, i32), (i32, i32)); 7] = [
        ((122, 70), (6, 4)),
        ((284, 71), (7, 4)),
        ((50, 86), (5, 3)),
        ((340, 84), (6, 4)),
        ((236, 166), (7, 4)),
        ((292, 160), (5, 3)),
        ((26, 130), (5, 3)),
    ];
    for (index, (centre, size)) in stones.into_iter().enumerate() {
        // The water folding round its foot, and its dark reflection.
        hline(
            scene,
            centre.0 - size.0,
            centre.1 + size.1,
            size.0 * 2 + 1,
            rgba(0x14282a, 110),
        );
        hline(
            scene,
            centre.0 - size.0 + 1,
            centre.1 + size.1 + 1,
            size.0 * 2 - 1,
            rgba(0x14282a, 60),
        );
        stone(
            scene,
            centre,
            size,
            2.2,
            0.5,
            false,
            1520 + index as u32 * 3,
        );
        hline(
            scene,
            centre.0 + size.0 + 1,
            centre.1 + size.1 - 1,
            3,
            rgba(0xb8d4c8, 120),
        );
        hline(
            scene,
            centre.0 - size.0 - 3,
            centre.1 + size.1 - 1,
            2,
            rgba(0xb8d4c8, 90),
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The lily pads
// ---------------------------------------------------------------------------------------------

/// The lily pads out on the water to the left, a flower open among them and one in bud.
fn lilies(scene: &mut Canvas) {
    let (lx, ly, _) = haunt(Haunt::Lilies);
    // (offset, half-width, which way the notch points), back to front.
    let pads: [((i32, i32), i32, f32); 11] = [
        ((-6, -9), 5, 0.6),
        ((7, -8), 6, 2.4),
        ((-17, -5), 6, 1.2),
        ((16, -3), 5, 3.6),
        ((-3, -3), 7, 4.4),
        ((-14, 3), 7, 0.2),
        ((9, 3), 8, 5.2),
        ((-2, 7), 6, 2.0),
        ((20, 7), 4, 1.0),
        ((-26, 1), 4, 4.0),
        ((32, -6), 3, 2.8),
    ];
    for (index, ((dx, dy), half, notch)) in pads.into_iter().enumerate() {
        pad(scene, (lx + dx, ly + dy), half, notch, 1700 + index as u32);
    }
    flower(scene, (lx + 11, ly + 1));
    // A bud on its stalk, still closed.
    let (bx, by) = (lx - 12, ly + 1);
    put(scene, bx, by, PETAL.light);
    put(scene, bx + 1, by, PETAL.shadow);
    put(scene, bx, by - 1, PETAL.base);
    put(scene, bx + 1, by - 1, PETAL.shadow);
    put(scene, bx, by - 2, PETAL.edge);
    put(scene, bx, by + 1, LILY.shadow);
    put(scene, bx + 1, by + 1, LILY.edge);
}

/// One lily pad lying on the water, foreshortened, lit from the upper left, with its notch.
fn pad(scene: &mut Canvas, (cx, cy): (i32, i32), half: i32, notch: f32, salt: u32) {
    let (rx, ry) = (half as f32 + 0.4, (half as f32 * 0.45).max(1.6));
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x - cx) as f32 / rx, (y - cy) as f32 / ry);
        let r = (u * u + v * v).sqrt();
        if r > 1.0 {
            return false;
        }
        let angle = v.atan2(u);
        let off = (angle - notch).rem_euclid(TAU);
        let off = off.min(TAU - off);
        r < 0.2 || off > 0.32
    };
    // Sitting on the water: a little shade cast down and to the right.
    for y in cy - half..=cy + half {
        for x in cx - half - 1..=cx + half + 1 {
            if inside(x - 1, y - 1) && !inside(x, y) {
                put(scene, x, y, rgba(0x14282a, 90));
            }
        }
    }
    for y in cy - half..=cy + half {
        for x in cx - half - 1..=cx + half + 1 {
            if !inside(x, y) {
                continue;
            }
            let rim =
                !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            let (u, v) = ((x - cx) as f32 / rx, (y - cy) as f32 / ry);
            let angle = v.atan2(u);
            // Veins running out from the middle.
            let vein = ((angle * 7.0 / TAU + 0.5).rem_euclid(1.0) - 0.5).abs() < 0.08;
            let lit = u + v * 0.8 + (noise(x, y, salt) % 100) as f32 / 400.0;
            let color = if rim {
                if lit < -0.4 { LILY.shadow } else { LILY.edge }
            } else if vein && (u * u + v * v) > 0.1 {
                LILY.shadow
            } else if lit < -0.55 {
                LILY.light
            } else if lit < 0.35 {
                LILY.base
            } else {
                LILY.shadow
            };
            put(scene, x, y, color);
        }
    }
    // Wet shine on the lit side.
    put(scene, cx - half / 2, cy - (ry * 0.5) as i32, LILY.shine);
}

/// A water lily, open: pale pink petals round a yellow heart, lit from the upper left.
fn flower(scene: &mut Canvas, (x, y): (i32, i32)) {
    let heart = rgb(0xe8c048);
    let heart_light = rgb(0xf4dc78);
    // Rows of petals, back to front: (dx, dy, tone).
    let petals: [(i32, i32, Rgba); 19] = [
        (-1, -3, PETAL.light),
        (1, -3, PETAL.base),
        (-3, -2, PETAL.light),
        (-2, -2, PETAL.shine),
        (0, -2, PETAL.base),
        (2, -2, PETAL.base),
        (3, -2, PETAL.shadow),
        (-4, -1, PETAL.light),
        (-3, -1, PETAL.base),
        (3, -1, PETAL.shadow),
        (4, -1, PETAL.edge),
        (-3, 0, PETAL.base),
        (-2, 0, PETAL.light),
        (-1, 0, PETAL.base),
        (0, 0, PETAL.base),
        (1, 0, PETAL.shadow),
        (2, 0, PETAL.shadow),
        (3, 0, PETAL.edge),
        (-2, 1, PETAL.shadow),
    ];
    hline(scene, x - 3, y + 1, 7, rgba(0x14282a, 70));
    for (dx, dy, color) in petals {
        put(scene, x + dx, y + dy, color);
    }
    hline(scene, x - 1, y + 1, 3, PETAL.edge);
    put(scene, x - 1, y - 1, heart_light);
    put(scene, x, y - 1, heart);
    put(scene, x + 1, y - 1, heart);
    put(scene, x + 2, y - 1, PETAL.shadow);
    put(scene, x - 2, y - 1, PETAL.light);
}

// ---------------------------------------------------------------------------------------------
// The shore and the near bank
// ---------------------------------------------------------------------------------------------

/// The water's edge: along the near bank the lip of the grass, its shade on the water and blades
/// and tufts of sedge leaning out over it, and at the shallows a beach of wet sand and pebbles
/// shelving up into the grass.
fn shore(scene: &mut Canvas) {
    let (sx, _, sr) = haunt(Haunt::Shallows);
    let pebbles = [rgb(0x7a7466), rgb(0x9a8e74), rgb(0x6a706a), rgb(0x8c7a5e)];
    for (x, column) in (0..WIDTH).zip(columns()) {
        let Some((_, near)) = column else {
            continue;
        };
        if near < 113 {
            continue;
        }
        // How much of a beach there is here, most in front of the shallows.
        let beach = (1.0 - ((x - sx) as f32 / (sr as f32 * 2.1)).abs()).max(0.0);
        let wide = (beach.sqrt() * 10.0).round() as i32 - 1;
        for dy in 1..=wide {
            let y = near + dy;
            let t = dy as f32 / wide as f32;
            // Wet and dark at the water, drying paler, then giving way to the grass.
            if t > 0.75 && chance(x, y, 1806, ((t - 0.75) * 900.0) as u32) {
                continue;
            }
            let color = if t < 0.3 {
                mix(SAND, EARTH.shadow, 0.4)
            } else if t < 0.55 {
                mix(SAND, EARTH.shadow, 0.15)
            } else {
                SAND
            };
            put(scene, x, y, color);
        }
        if wide > 0 && chance(x, 0, 1807, 110) {
            put(scene, x, near + 1, rgba(0xb8d4c8, 120));
        }
        // The shade of the bank on the water just below it.
        put(scene, x, near, rgba(0x14282a, 120));
        put(scene, x, near - 1, rgba(0x14282a, 50));
        // The lip of the bank, catching the light, with grass leaning out over the water.
        let lip = near + 1 + wide.max(0);
        if wide < 2 {
            put(
                scene,
                x,
                lip,
                if chance(x, lip, 1804, 120) {
                    MOSS.light
                } else {
                    GRASS.light
                },
            );
            put(scene, x, lip + 1, GRASS.base);
        }
        if chance(x, 0, 1805, 90) {
            let tall = 2 + pick(x, 1, 1805, 3);
            let lean = pick(x, 2, 1805, 3) - 1;
            line(scene, (x, lip), (x + lean, lip - tall), GRASS.base);
            put(scene, x + lean, lip - tall, GRASS.light);
        }
    }
    // Pebbles on the beach, each lit on top.
    for index in 0..60 {
        let x = sx - sr * 2 + pick(index, 0, 1808, sr * 4);
        let Some((_, near)) = columns_at(x) else {
            continue;
        };
        let beach = (1.0 - ((x - sx) as f32 / (sr as f32 * 2.1)).abs()).max(0.0);
        let wide = (beach.sqrt() * 10.0).round() as i32 - 1;
        if wide < 3 {
            continue;
        }
        let y = near + 2 + pick(index, 1, 1808, wide - 2);
        let color = pebbles[pick(index, 2, 1808, 4) as usize];
        let long = 1 + pick(index, 3, 1808, 2);
        hline(scene, x, y, long, color);
        put(scene, x, y - 1, mix(color, rgb(0xd8d0b8), 0.4));
        hline(scene, x, y + 1, long, rgba(0x2a2418, 80));
    }
    // Tufts of sedge at the water's edge, leaning out over it; none in front of the seat.
    for (index, &x) in [26, 182, 254, 318].iter().enumerate() {
        let Some((_, near)) = columns_at(x) else {
            continue;
        };
        sedge(
            scene,
            (x, near + 3),
            9 + index as i32 % 2 * 3,
            1810 + index as u32,
        );
    }
}

/// A tuft of sedge: blades fanning up and out from one root, the outer ones arching over, lit on
/// the left.
fn sedge(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32, salt: u32) {
    for blade in 0..9i32 {
        let spread = blade - 4;
        let long = tall - spread.abs() + pick(blade, 0, salt, 3);
        let lean = spread * 2 + pick(blade, 1, salt, 3) - 1;
        let level = if spread < -1 {
            3
        } else if spread < 2 {
            2
        } else {
            1
        };
        let mut previous = (x + spread / 2, foot);
        for step in 1..=long * 2 {
            let t = step as f32 / (long * 2) as f32;
            let point = (
                x + spread / 2 + (lean as f32 * t * t).round() as i32,
                foot - (long as f32 * (2.0 * t - t * t * 0.8)).round() as i32,
            );
            let color = if t > 0.8 {
                tone(GRASS, level + 1)
            } else if t < 0.3 {
                tone(GRASS, level - 1)
            } else {
                tone(GRASS, level)
            };
            line(scene, previous, point, color);
            previous = point;
        }
    }
}

/// Whether `(x, y)` is close to where the one fishing sits, the one with the net stands, anyone
/// else waits, or the tackle is: kept clear of twigs, stones and flowers.
fn kept_clear(x: i32, y: i32) -> bool {
    let near = |(sx, sy): Pixels, (w, h): Pixels| (sx - x).abs() < w && (sy - y).abs() < h;
    near((SEAT.0 as i32, SEAT.1 as i32), (24, 12))
        || near((NET.0 as i32, NET.1 as i32), (18, 12))
        || PLACES
            .iter()
            .any(|&(_, (px, py))| near((px as i32, py as i32), (14, 8)))
        || near(BASKET, (14, 12))
        || near(WORMS, (8, 8))
}

/// The near bank: the path coming in from the left to the worn patch where the fishing is done,
/// stones, twigs and wildflowers, and the dapple of the canopy over it all.
fn near_bank(scene: &mut Canvas) {
    path(scene);
    fern_clump(scene, (30, 170), 0.55);
    fern_clump(scene, (318, 206), 0.85);
    fern_clump(scene, (6, 150), 0.6);
    mushrooms(scene, (16, 122));
    // Twigs and stones, kept away from wherever anyone sits or stands, and the tackle.
    for index in 0..22 {
        let x = 10 + pick(index, 0, 1901, WIDTH - 20);
        let y = 168 + pick(index, 1, 1901, HEIGHT - 172);
        if kept_clear(x, y) || is_water(x, y - 4) {
            continue;
        }
        let long = 3 + pick(index, 2, 1901, 5);
        let rise = pick(index, 3, 1901, 3) - 1;
        line(scene, (x, y), (x + long, y + rise), BARK.shadow);
        put(scene, x, y - 1, BARK.light);
        put(scene, x + long, y + rise + 1, rgba(0x14241c, 90));
    }
    for index in 0..10 {
        let x = 12 + pick(index, 0, 1902, WIDTH - 24);
        let y = 170 + pick(index, 1, 1902, HEIGHT - 176);
        if kept_clear(x, y) || is_water(x, y - 4) {
            continue;
        }
        ellipse(scene, x + 1, y + 2, 3, 1, rgba(0x14241c, 80));
        stone(
            scene,
            (x, y),
            (2 + pick(index, 2, 1902, 2), 2),
            2.0,
            0.3,
            false,
            1903 + index as u32,
        );
    }
    // Wildflowers in little scatters: forget-me-nots, buttercups and wood sorrel.
    for index in 0..22 {
        let x = 8 + pick(index, 0, 1910, WIDTH - 16);
        let y = 164 + pick(index, 1, 1910, HEIGHT - 168);
        let kind = pick(index, 2, 1910, 3);
        for floret in 0..2 + pick(index, 3, 1910, 3) {
            let fx = x + pick(index, floret, 1911, 7) - 3;
            let fy = y + pick(floret, index, 1911, 4) - 2;
            if kept_clear(fx, fy) || is_water(fx, fy) || is_water(fx, fy + 3) {
                continue;
            }
            let (petal, heart) = match kind {
                0 => (rgb(0x7aa2d6), rgb(0xe8d070)),
                1 => (rgb(0xe8c84a), rgb(0xf4e08a)),
                _ => (rgb(0xe8dcd8), rgb(0xd8a8b4)),
            };
            put(scene, fx, fy + 1, GRASS.edge);
            put(scene, fx - 1, fy, petal);
            put(scene, fx + 1, fy, mix(petal, GRASS.shadow, 0.3));
            put(scene, fx, fy - 1, petal);
            put(scene, fx, fy, heart);
        }
    }
    // Shade where the canopy is thickest, and flecks of sun through the leaves.
    for y in 150..HEIGHT {
        for x in 0..WIDTH {
            if !is_water(x, y) && patches(x, y, (44, 14), 1920) > 0.68 {
                put(scene, x, y, rgba(0x14241c, 40));
            }
        }
    }
    for index in 0..40 {
        let x = pick(index, 0, 1921, WIDTH);
        let y = 160 + pick(index, 1, 1921, HEIGHT - 160);
        if is_water(x, y) {
            continue;
        }
        let rx = 1 + pick(index, 2, 1921, 4);
        ellipse(scene, x, y, rx, 1, rgba(SUN, 36));
        if rx > 2 {
            hline(scene, x - rx / 2, y, rx, rgba(SUN, 32));
        }
    }
}

/// A clump of ferns, fronds arching out from one crown: the back ones darker and more upright,
/// the near ones lighter and spreading wider, a fiddlehead still uncurling in the middle.
fn fern_clump(scene: &mut Canvas, (cx, base): (i32, i32), size: f32) {
    ellipse(scene, cx + 3, base, (22.0 * size) as i32, 3, SHADE);
    let fronds = [
        (-100.0, 34.0, 1),
        (-132.0, 30.0, 1),
        (-68.0, 30.0, 1),
        (-158.0, 26.0, 2),
        (-30.0, 24.0, 2),
        (-116.0, 32.0, 3),
        (-82.0, 30.0, 3),
        (-172.0, 18.0, 3),
        (-14.0, 16.0, 3),
    ];
    for (angle, length, level) in fronds {
        frond(scene, (cx, base), (angle, length * size), level);
    }
    for (dx, dy) in [(0, 0), (1, -1), (2, -1), (3, 0), (3, 1), (2, 2), (1, 1)] {
        put(scene, cx + 2 + dx, base - 6 + dy, FERN.light);
    }
}

/// A few small brown mushrooms in the moss, caps lit from the upper left.
fn mushrooms(scene: &mut Canvas, (x, y): (i32, i32)) {
    let cap = Ramp::new(0x5a3424, 0x83502f, 0xa66e42, 0xc08c5a, 0xd6aa7a);
    let stalk = Ramp::new(0x7a6a56, 0xa89878, 0xc4b694, 0xd6caa8, 0xe2d8bc);
    for &(dx, dy, tall, wide) in &[(0, 0, 4, 2), (5, 2, 3, 1), (-4, 3, 2, 1)] {
        let (mx, my) = (x + dx, y + dy);
        ellipse(scene, mx + 1, my, wide + 1, 1, SHADE);
        vline(scene, mx, my - tall + 1, tall, stalk.light);
        vline(scene, mx + 1, my - tall + 2, tall - 1, stalk.shadow);
        let under = my - tall;
        hline(scene, mx - wide, under, wide * 2 + 2, cap.edge);
        hline(scene, mx - wide, under - 1, wide * 2 + 2, cap.base);
        put(scene, mx - wide, under - 1, cap.light);
        put(scene, mx + wide + 1, under - 1, cap.shadow);
        hline(scene, mx - wide + 1, under - 2, wide * 2, cap.light);
        put(scene, mx - wide + 1, under - 2, cap.shine);
        hline(scene, mx - wide + 1, under - 3, wide * 2, cap.edge);
        put(scene, mx - wide - 1, under - 1, cap.edge);
        put(scene, mx + wide + 2, under - 1, cap.edge);
        put(scene, mx - wide, under - 2, cap.edge);
        put(scene, mx + wide + 1, under - 2, cap.edge);
    }
}

/// The path trodden round the pool from the left, ending in the worn patch on the bank where
/// the fishing is done.
fn path(scene: &mut Canvas) {
    let points: [(f32, f32); 6] = [
        (-4.0, 206.0),
        (30.0, 202.0),
        (64.0, 199.0),
        (98.0, 200.0),
        (128.0, 197.0),
        (152.0, 192.0),
    ];
    for x in 0..160 {
        let fx = x as f32;
        let Some(pair) = points
            .windows(2)
            .find(|pair| fx >= pair[0].0 && fx <= pair[1].0)
        else {
            continue;
        };
        let t = (fx - pair[0].0) / (pair[1].0 - pair[0].0);
        let centre = pair[0].1 + (pair[1].1 - pair[0].1) * t;
        let half = 3.0 + (centre - 186.0) * 0.18;
        for y in (centre - half - 1.0) as i32..=(centre + half + 1.0) as i32 {
            let off = (y as f32 - centre).abs() / half;
            let edge = off + (noise(x, y, 1930) % 100) as f32 / 260.0;
            if edge > 1.0 {
                continue;
            }
            let color = if edge > 0.8 {
                rgba(0x6e5238, 110)
            } else if chance(x, y, 1931, 26) {
                rgba(0xa2845c, 170)
            } else {
                rgba(0x886a48, 150)
            };
            put(scene, x, y, color);
        }
    }
    // The worn patch where the fishing is done, in front of the seat and round the net.
    let (cx, cy) = ((SEAT.0 + NET.0) as i32 / 2, NET.1 as i32 + 1);
    for y in cy - 8..=cy + 8 {
        for x in cx - 30..=cx + 30 {
            let (u, v) = ((x - cx) as f32 / 28.0, (y - cy) as f32 / 7.0);
            let r = (u * u + v * v).sqrt() + (noise(x, y, 1932) % 100) as f32 / 300.0;
            if r > 1.0 {
                continue;
            }
            let color = if r > 0.85 {
                rgba(0x6e5238, 90)
            } else if chance(x, y, 1933, 24) {
                rgba(0xa2845c, 150)
            } else {
                rgba(0x886a48, 130)
            };
            put(scene, x, y, color);
        }
    }
    for (index, &(x, y)) in [(4, 207), (40, 204), (78, 197), (100, 203), (166, 195)]
        .iter()
        .enumerate()
    {
        put(scene, x, y, ROCK.light);
        put(scene, x + 1, y, ROCK.shadow);
        put(scene, x + 1, y + 1, rgba(0x14241c, 80));
        if index % 2 == 0 {
            put(scene, x + 3, y + 1, ROCK.base);
        }
    }
}

/// The flat mossy stone at the water's edge where the one fishing sits: its top lit and flat
/// enough to sit on, its front in shade, cracked, moss across its back.
fn seat(scene: &mut Canvas) {
    let (sx, sy) = (SEAT.0 as i32, SEAT.1 as i32);
    let (half, top_depth, face) = (19, 5, 8);
    let cy = sy - 3;
    ellipse(scene, sx + 4, cy + face + 3, half + 2, 3, SHADE);
    for x in sx - half..=sx + half {
        let u = (x - sx) as f32 / half as f32;
        let round = (1.0 - u * u).max(0.0).sqrt();
        let back = cy - (top_depth as f32 * round).round() as i32;
        let front = cy + (top_depth as f32 * 0.7 * round).round() as i32;
        let bottom = front + (face as f32 * (0.5 + 0.5 * round)).round() as i32;
        for y in back..=bottom {
            let on_top = y < front;
            let edge = y == back || y == bottom || x == sx - half || x == sx + half;
            let mut color = if edge {
                ROCK.edge
            } else if on_top {
                // The top, flat and lit, brightest towards the back left.
                let v = (y - back) as f32 / (front - back).max(1) as f32;
                if u + v * 0.6 < -0.35 {
                    ROCK.shine
                } else if u + v * 0.6 < 0.4 {
                    ROCK.light
                } else {
                    ROCK.base
                }
            } else if y == front {
                // The rounded front edge of the top catches the light.
                if u < 0.3 { ROCK.light } else { ROCK.base }
            } else if u < -0.5 {
                ROCK.base
            } else if y > bottom - 2 || u > 0.55 {
                ROCK.edge
            } else {
                ROCK.shadow
            };
            if !edge && chance(x.div_euclid(2), y, 2001, 30) {
                color = mix(color, ROCK.edge, 0.35);
            }
            // Moss across its back and down its left end.
            let moss = patches(x, y, (4, 3), 2002) - (y - back) as f32 * 0.06 - u * 0.25;
            if moss > 0.62 {
                color = if edge {
                    MOSS.edge
                } else if on_top {
                    if moss > 0.8 { MOSS.light } else { MOSS.base }
                } else {
                    MOSS.shadow
                };
            }
            put(scene, x, y, color);
        }
    }
    // A crack across its face, and lichen on top.
    let crack_y = cy + 6;
    line(
        scene,
        (sx - 4, crack_y - 1),
        (sx + 6, crack_y + 1),
        ROCK.edge,
    );
    line(
        scene,
        (sx - 4, crack_y),
        (sx + 6, crack_y + 2),
        mix(ROCK.shadow, ROCK.light, 0.4),
    );
    for (dx, dy) in [(-8, -2), (6, -1), (11, 0)] {
        put(scene, sx + dx, cy + dy, rgb(0xb4b67c));
        put(scene, sx + dx + 1, cy + dy, rgb(0x9a9c66));
    }
    // Grass growing up round its foot.
    for x in sx - half - 2..=sx + half + 2 {
        if chance(x, 0, 2003, 110) {
            let u = (x - sx) as f32 / half as f32;
            let foot = cy
                + (top_depth as f32 * 0.7 * (1.0 - u * u).max(0.0).sqrt()) as i32
                + (face as f32 * (0.5 + 0.5 * (1.0 - u * u).max(0.0).sqrt())) as i32;
            let tall = 2 + pick(x, 1, 2003, 3);
            vline(scene, x, foot - tall + 2, tall, GRASS.base);
            put(scene, x, foot - tall + 2, GRASS.light);
        }
    }
}

/// The tackle off to one side: a wicker creel with its lid shut and its strap, and a jar of worms.
fn tackle(scene: &mut Canvas) {
    let (bx, by) = BASKET;
    let (left, right) = (bx - 8, bx + 8);
    let (lid_back, lid_front, bottom) = (by - 14, by - 10, by);
    ellipse(scene, bx + 3, bottom, 12, 2, SHADE);
    // The body, woven: rows of withies over and under, lit on the left.
    for y in lid_front..=bottom {
        for x in left..=right {
            let edge = x == left || x == right || y == bottom;
            let row = (y - lid_front).div_euclid(2);
            let over = (x + row * 2).div_euclid(2).rem_euclid(2) == 0;
            let across = (x - left) as f32 / (right - left) as f32;
            let mut level = if over { 3 } else { 2 };
            if across > 0.7 {
                level -= 1;
            }
            if (y - lid_front).rem_euclid(2) == 1 {
                level -= 1;
            }
            put(
                scene,
                x,
                y,
                if edge {
                    WICKER.edge
                } else {
                    tone(WICKER, level)
                },
            );
        }
    }
    // A band round its rim.
    hline(scene, left, lid_front, right - left + 1, WICKER.edge);
    hline(
        scene,
        left + 1,
        lid_front + 1,
        right - left - 1,
        WICKER.light,
    );
    // The lid, seen from a little above: its top lit, a hole for the catch.
    for y in lid_back..lid_front {
        let inset = (lid_front - y) / 2;
        for x in left + inset..=right - inset {
            let edge = y == lid_back || x == left + inset || x == right - inset;
            let color = if edge {
                WICKER.edge
            } else if (x + y).rem_euclid(3) == 0 {
                WICKER.base
            } else if x < bx {
                WICKER.light
            } else {
                mix(WICKER.light, WICKER.base, 0.5)
            };
            put(scene, x, y, color);
        }
    }
    put(scene, left + 3, lid_back + 1, WICKER.shine);
    hline(scene, bx - 3, lid_back + 2, 4, WICKER.edge);
    hline(scene, bx - 3, lid_back + 3, 4, rgb(0x2a1e14));
    // The strap, from one side over the lid to the other.
    for x in left - 1..=right + 1 {
        let u = (x - bx) as f32 / 9.0;
        let y = lid_back - 4 + (u * u * 6.0).round() as i32;
        put(scene, x, y, LEATHER.base);
        put(scene, x, y + 1, LEATHER.edge);
        if x < bx - 3 {
            put(scene, x, y, LEATHER.light);
        }
    }
    for (x, color) in [(left - 1, LEATHER.shadow), (right + 1, LEATHER.edge)] {
        vline(scene, x, lid_back + 1, lid_front - lid_back + 2, color);
    }
    put(scene, left - 1, lid_front + 2, rgb(0xc9a14e));
    put(scene, right + 1, lid_front + 2, rgb(0x9a7434));

    // The jar of worms: earth inside, a worm against the glass, a cloth tied over the top.
    let (jx, jy) = WORMS;
    ellipse(scene, jx + 2, jy, 5, 1, SHADE);
    let (left, right, top) = (jx - 3, jx + 3, jy - 9);
    for y in top..=jy {
        for x in left..=right {
            let edge = x == left || x == right || y == jy;
            let color = if edge {
                JAR.shadow
            } else if y > top + 3 {
                let soil = if chance(x, y, 2101, 60) {
                    EARTH.light
                } else {
                    EARTH.base
                };
                mix(soil, JAR.light, 0.2)
            } else {
                mix(JAR.base, air(48), 0.3)
            };
            put(scene, x, y, color);
        }
    }
    vline(scene, left + 1, top + 1, 7, JAR.shine);
    for (dx, dy) in [(0, 5), (1, 6), (1, 7), (0, 8)] {
        put(scene, jx + dx, top + dy, rgb(0xd88a8a));
    }
    put(scene, jx + 2, top + 6, rgb(0xb06a6a));
    hline(scene, left - 1, top, 9, rgb(0xb05445));
    hline(scene, left, top - 1, 7, rgb(0xc96c53));
    put(scene, left + 1, top - 1, rgb(0xf3e9cf));
    put(scene, left + 4, top - 1, rgb(0xf3e9cf));
    put(scene, left + 2, top, rgb(0xf3e9cf));
    put(scene, left + 5, top, rgb(0xf3e9cf));
    hline(scene, left, top + 1, 7, rgb(0x8a6a48));
}

// ---------------------------------------------------------------------------------------------
// The reeds
// ---------------------------------------------------------------------------------------------

/// One stem or leaf of the reed beds: its foot, how tall it stands, how far it leans out, and
/// what it is.
#[derive(Clone, Copy)]
struct Blade {
    foot: (i32, i32),
    tall: i32,
    lean: i32,
    kind: Stem,
}

#[derive(Clone, Copy, PartialEq)]
enum Stem {
    /// A reed, nearly upright.
    Reed,
    /// A leaf arching out and over at its tip.
    Leaf,
    /// A bulrush, stiff, its brown head at the top.
    Rush,
}

/// Reeds and bulrushes at the right of the water: beds of them standing in it round the reed bed
/// and running up the right shore and onto the near bank, and a dragonfly resting on one.
fn reeds(scene: &mut Canvas) {
    let (rx, ry, _) = haunt(Haunt::Reeds);
    // Beds: (foot centre, how many, how far they spread across and back, how many rushes).
    let beds = [
        ((rx - 32, ry - 40), 6, (6, 2), 0),
        ((rx + 12, ry - 36), 9, (10, 3), 2),
        ((rx + 34, ry - 40), 14, (14, 5), 0),
        ((rx + 48, ry - 22), 12, (9, 6), 1),
        ((rx + 36, ry + 2), 10, (7, 6), 0),
        ((rx + 44, ry + 24), 12, (9, 5), 2),
        ((rx + 40, ry + 46), 16, (14, 5), 1),
    ];
    let mut blades: Vec<Blade> = Vec::new();
    for (index, ((cx, cy), count, (across, back), rushes)) in beds.into_iter().enumerate() {
        for blade in 0..count + rushes {
            let i = index as i32 * 32 + blade;
            let foot = (
                cx + pick(i, 0, 2201, across * 2 + 1) - across,
                cy + pick(i, 1, 2201, back * 2 + 1) - back,
            );
            let near = nearness(foot.1);
            let kind = if blade >= count {
                Stem::Rush
            } else if pick(i, 4, 2201, 3) == 0 {
                Stem::Leaf
            } else {
                Stem::Reed
            };
            let tall = match kind {
                Stem::Reed => (14.0 + near * 34.0) as i32 + pick(i, 2, 2201, 10),
                Stem::Leaf => (8.0 + near * 18.0) as i32 + pick(i, 2, 2201, 6),
                Stem::Rush => (20.0 + near * 30.0) as i32 + pick(i, 2, 2201, 16),
            };
            let lean = match kind {
                Stem::Leaf => (pick(i, 3, 2201, 2) * 2 - 1) * (4 + pick(i, 5, 2201, 5)),
                _ => pick(i, 3, 2201, 7) - 3,
            };
            blades.push(Blade {
                foot,
                tall,
                lean,
                kind,
            });
        }
        // The dark crowd of stems at the bed's foot.
        if count > 4 {
            for x in cx - across - 2..=cx + across + 2 {
                let u = (x - cx) as f32 / (across + 2) as f32;
                let depth = (2.0 * (1.0 - u * u).max(0.0).sqrt()).round() as i32;
                for y in cy - depth..=cy + back {
                    if chance(x, y, 2204, 170) {
                        put(scene, x, y, if y < cy { REED.shadow } else { REED.edge });
                    }
                }
            }
        }
    }
    blades.sort_by_key(|blade| blade.foot.1);
    for (index, blade) in blades.iter().enumerate() {
        let (x, foot) = blade.foot;
        if is_water(x, foot + 1) {
            // Its reflection, broken by ripples, and the ripple round its foot.
            for dy in (1..6).step_by(2) {
                put(scene, x, foot + dy, rgba(0x34442a, 80));
            }
            hline(scene, x - 1, foot + 1, 3, rgba(0xa4c4b8, 70));
        }
        match blade.kind {
            Stem::Rush => bulrush(scene, blade.foot, blade.tall, blade.lean),
            Stem::Reed | Stem::Leaf => stem(scene, *blade, index),
        }
    }
    // A dragonfly resting on a reed, its wings catching the light.
    let (fx, fy) = (rx + 18, ry - 52);
    hline(scene, fx, fy, 7, rgb(0x2a5a66));
    put(scene, fx, fy, rgb(0x4a8a96));
    put(scene, fx - 1, fy, rgb(0x1e3a44));
    for (wx, wy) in [
        (fx + 1, fy - 2),
        (fx + 3, fy - 2),
        (fx + 1, fy + 1),
        (fx + 3, fy + 1),
    ] {
        hline(scene, wx, wy, 3, rgba(0xd8e8e2, 120));
    }
}

/// One reed or leaf: rising from its foot and bending out by its lean, a leaf arching right over
/// at its tip; two pixels wide nearer the eye, lit on the left.
fn stem(scene: &mut Canvas, blade: Blade, index: usize) {
    let (x0, foot) = blade.foot;
    let (tall, lean) = (blade.tall.max(2), blade.lean);
    let wide = nearness(foot) > 0.5;
    let level = match index % 3 {
        0 => 1,
        1 => 2,
        _ => 3,
    };
    let steps = tall * 2;
    let mut previous = (x0, foot);
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let (dx, rise) = match blade.kind {
            // Rises and arches over, its tip running out level.
            Stem::Leaf => (lean as f32 * t * t, tall as f32 * (2.0 * t - t * t)),
            _ => (lean as f32 * t * t, tall as f32 * t),
        };
        let point = (x0 + dx.round() as i32, foot - rise.round() as i32);
        let near_tip = t > 0.85;
        let color = if near_tip {
            tone(REED, level + 1)
        } else if t < 0.25 {
            tone(REED, level - 1)
        } else {
            tone(REED, level)
        };
        line(scene, previous, point, color);
        if wide && t < 0.7 {
            put(scene, point.0 + 1, point.1, tone(REED, level - 1));
        }
        if !near_tip && chance(point.0, point.1, 2203, 24) {
            put(scene, point.0, point.1, REED.light);
        }
        previous = point;
    }
}

/// One bulrush: a stiff stem and its brown velvet head, lit on the left, a spike above it.
fn bulrush(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32, lean: i32) {
    let top = foot - tall;
    let head = (5 + tall / 7).min(10);
    let wide = tall > 40;
    line(scene, (x, foot), (x + lean, top), REED.shadow);
    put(scene, x + lean - 1, top + head + 3, REED.base);
    vline(scene, x + lean, top - 4, 4, REED.base);
    put(scene, x + lean, top - 4, REED.light);
    for y in top + 1..top + 1 + head {
        let left = if wide { x + lean - 1 } else { x + lean };
        put(scene, left - 1, y, BULRUSH.edge);
        put(
            scene,
            left,
            y,
            if y == top + 1 {
                BULRUSH.shine
            } else {
                BULRUSH.light
            },
        );
        put(scene, left + 1, y, BULRUSH.base);
        if wide {
            put(scene, left + 2, y, BULRUSH.shadow);
        }
        put(scene, left + 2 + i32::from(wide), y, BULRUSH.edge);
    }
    hline(scene, x + lean - 1, top, 2 + i32::from(wide), BULRUSH.edge);
    hline(
        scene,
        x + lean - 1,
        top + 1 + head,
        2 + i32::from(wide),
        BULRUSH.edge,
    );
}

// ---------------------------------------------------------------------------------------------
// The light
// ---------------------------------------------------------------------------------------------

/// Shafts of sun slanting down from the gaps in the canopy, motes drifting in them, and the
/// warm pools where they land: softly, so the water stays a mid-tone under them.
fn sunbeams(scene: &mut Canvas) {
    for (index, &(origin, half, top, end)) in BEAMS.iter().enumerate() {
        let salt = 2500 + index as u32 * 10;
        for y in top..end {
            let centre = origin + SLANT * y as f32;
            let strength = ((y - top) as f32 / 14.0).min(1.0) * ((end - y) as f32 / 30.0).min(1.0);
            for x in (centre - half - 2.0) as i32..=(centre + half + 2.0) as i32 {
                let off = (x as f32 - centre).abs() / half
                    + (noise(x, y, salt) % 100) as f32 / 100.0 * 0.14;
                let alpha = if off < 0.35 {
                    58.0
                } else if off < 0.7 {
                    38.0
                } else if off < 1.0 {
                    18.0
                } else {
                    0.0
                } * strength
                    * if is_water(x, y) { 0.55 } else { 1.0 };
                if alpha >= 1.0 {
                    put(scene, x, y, rgba(SUN, alpha as u8));
                }
            }
        }
        // Where it lands.
        let (lx, ly) = ((origin + SLANT * end as f32) as i32, end - 2);
        let half = half as i32;
        for (rx, ry, alpha) in [(half * 2 + 6, 5, 18), (half * 2, 3, 22), (half, 1, 26)] {
            ellipse(scene, lx, ly, rx, ry, rgba(SUN, alpha));
        }
        // On the water, the light breaks into warm glints on the ripples.
        if is_water(lx, ly) {
            for glint in 0..9 {
                let x = lx + pick(glint, 0, salt + 2, half * 4 + 1) - half * 2;
                let y = ly + pick(glint, 1, salt + 2, 9) - 4;
                if is_water(x, y) && is_water(x + 4, y) {
                    hline(scene, x, y, 2 + pick(glint, 2, salt + 2, 3), rgba(SUN, 96));
                }
            }
        }
        // Motes caught in the light.
        for mote in 0..8 {
            let y = top + 14 + pick(mote, 0, salt + 1, (end - top - 24).max(1));
            let x = (origin + SLANT * y as f32) as i32 + pick(mote, 1, salt + 1, half * 2) - half;
            put(scene, x, y, rgba(0xfff0c0, 110));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// In front of everyone
// ---------------------------------------------------------------------------------------------

/// A spray of the nearest leaves of all, reaching into the picture from a corner: twigs fanning
/// out from `root` off the edge, each ending in a few pointed leaves hanging from it.
fn leaf_spray(scene: &mut Canvas, root: (i32, i32), tips: &[(i32, i32)], salt: u32) {
    let near = Ramp::new(0x22401e, 0x36602a, 0x4c7c36, 0x689846, 0x8cb45c);
    for &tip in tips {
        line(scene, root, tip, BARK.edge);
        line(scene, (root.0, root.1 + 1), (tip.0, tip.1 + 1), BARK.shadow);
    }
    for (index, &tip) in tips.iter().enumerate() {
        let i = index as i32;
        let (dx, dy) = (tip.0 - root.0, tip.1 - root.1);
        for leaf in 0..3 {
            let t = 0.55 + leaf as f32 * 0.2;
            let stalk = (
                root.0 + (dx as f32 * t) as i32,
                root.1 + (dy as f32 * t) as i32 + 1,
            );
            let side = if (leaf + i) % 2 == 0 { -1 } else { 1 };
            let reach = 10 + pick(i * 3 + leaf, 0, salt, 5);
            let end = if leaf == 2 {
                (stalk.0 + dx.signum() * reach / 2, stalk.1 + reach - 2)
            } else {
                (stalk.0 + side * (2 + reach / 3), stalk.1 + reach)
            };
            hanging_leaf(scene, stalk, end, near);
        }
    }
}

/// One pointed leaf from its stalk to its tip, lit on the side towards the upper left.
fn hanging_leaf(scene: &mut Canvas, from: (i32, i32), to: (i32, i32), ramp: Ramp) {
    let (dx, dy) = ((to.0 - from.0) as f32, (to.1 - from.1) as f32);
    let length = (dx * dx + dy * dy).sqrt().max(1.0);
    let (nx, ny) = (-dy / length * 2.4, dx / length * 2.4);
    let at = |t: f32, n: f32| {
        (
            (from.0 as f32 + dx * t + nx * n).round() as i32,
            (from.1 as f32 + dy * t + ny * n).round() as i32,
        )
    };
    let outline = [from, at(0.45, 1.0), to, at(0.45, -1.0)];
    let normal_lit = dy - dx > 0.0;
    polygon(scene, &outline, |x, y| {
        let on_normal_side = (y - from.1) as f32 * dx - (x - from.0) as f32 * dy > 0.0;
        Some(if on_normal_side == normal_lit {
            ramp.light
        } else {
            ramp.shadow
        })
    });
    for index in 0..4 {
        line(scene, outline[index], outline[(index + 1) % 4], ramp.edge);
    }
    line(scene, from, at(0.75, 0.0), ramp.base);
}

/// Grass standing up along the bottom corners, tallest right in them, and a reed or two at the
/// right; nothing in the middle, where the seat and the net are.
fn front_grass(scene: &mut Canvas) {
    for x in (0..72).chain(WIDTH - 84..WIDTH) {
        let corner = if x < 72 {
            1.0 - x as f32 / 72.0
        } else {
            (x - (WIDTH - 84)) as f32 / 84.0
        };
        if !chance(x, 0, 2701, 60 + (corner * 120.0) as u32) {
            continue;
        }
        let tall = 2 + (corner * 12.0) as i32 + pick(x, 1, 2701, 4);
        let lean = pick(x, 2, 2701, 5) - 2;
        let (body, tip) = match pick(x, 3, 2701, 3) {
            0 => (GRASS.edge, GRASS.shadow),
            1 => (GRASS.shadow, GRASS.base),
            _ => (GRASS.base, GRASS.light),
        };
        line(scene, (x, HEIGHT - 1), (x + lean, HEIGHT - tall), body);
        put(scene, x + lean, HEIGHT - tall, tip);
    }
    for (x, tall, lean) in [
        (WIDTH - 22, 34, -3),
        (WIDTH - 14, 40, -1),
        (WIDTH - 6, 30, 2),
    ] {
        for k in 0..tall {
            let px = x + lean * k * k / (tall * tall);
            put(
                scene,
                px,
                HEIGHT - 1 - k,
                tone(REED, if k > tall - 4 { 3 } else { 1 }),
            );
            if k < tall / 2 {
                put(scene, px + 1, HEIGHT - 1 - k, REED.edge);
            }
        }
    }
}

/// One fern frond from the crown at `from`, arching out at `angle` degrees and drooping at its
/// tip, with leaflets either side that shorten towards the tip.
fn frond(scene: &mut Canvas, from: (i32, i32), (angle, length): (f32, f32), level: i32) {
    let radians = f32::to_radians(angle);
    let (dx, dy) = (radians.cos(), radians.sin());
    let droop = length * (0.25 + 0.4 * dx.abs());
    let spine = |t: f32| {
        (
            from.0 as f32 + dx * length * t,
            from.1 as f32 + dy * length * t + droop * t * t,
        )
    };
    let steps = length as i32;
    let mut previous = spine(0.0);
    let round = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let point = spine(t);
        if t > 0.18 && step % 2 == 0 {
            let (ax, ay) = (point.0 - previous.0, point.1 - previous.1);
            let norm = (ax * ax + ay * ay).sqrt().max(0.01);
            let (ax, ay) = (ax / norm, ay / norm);
            let reach = (1.0 - t) * 3.0 + 1.0;
            for side in [-1.0f32, 1.0] {
                let (nx, ny) = (-ay * side, ax * side);
                let tip = (
                    point.0 + nx * reach + ax * reach * 0.5,
                    point.1 + ny * reach + ay * reach * 0.5 + 0.6,
                );
                let upper = ny < 0.0;
                line(
                    scene,
                    round(point),
                    round(tip),
                    tone(FERN, level + i32::from(upper)),
                );
                if upper && reach > 3.0 {
                    put(scene, round(tip).0, round(tip).1, tone(FERN, level + 2));
                }
            }
        }
        line(scene, round(previous), round(point), tone(FERN, level - 1));
        previous = point;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::in_water;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(backdrop().pixels().iter().all(|pixel| pixel.a == 255));
    }

    #[test]
    fn the_pool_is_the_same_every_time() {
        assert_eq!(backdrop(), backdrop());
        assert_eq!(foreground(), foreground());
    }

    #[test]
    fn the_shore_lies_beyond_everywhere_a_fish_can_swim() {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if in_water(x as f32 + 0.5, y as f32 + 0.5) {
                    assert!(
                        is_water(x, y),
                        "({x}, {y}) is open water but painted as land"
                    );
                }
            }
        }
        for (x, y) in [SEAT, NET] {
            assert!(
                !is_water(x as i32, y as i32 - 10),
                "({x}, {y}) is in the pool"
            );
        }
    }

    #[test]
    fn the_foreground_leaves_the_water_the_seat_and_the_net_in_sight() {
        let front = foreground();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if is_water(x, y) {
                    assert_eq!(
                        front.get(x, y).a,
                        0,
                        "the foreground covers the water at ({x}, {y})"
                    );
                }
            }
        }
        for (x, y) in [SEAT, NET] {
            for dy in -36..=4 {
                for dx in -18..=18 {
                    let (px, py) = (x as i32 + dx, y as i32 + dy);
                    assert_eq!(front.get(px, py).a, 0, "the foreground covers ({px}, {py})");
                }
            }
        }
    }

    #[test]
    fn every_haunt_is_a_mid_tone_a_shadow_shows_on() {
        let scene = backdrop();
        for haunt in Haunt::ALL {
            let ((cx, cy), r) = haunt.area();
            let mut brightness: Vec<u32> = (-(r as i32)..=r as i32)
                .flat_map(|dy| (-(r as i32)..=r as i32).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| (cx + dx as f32, cy + dy as f32 * 0.4))
                .filter(|&(x, y)| in_water(x, y))
                .map(|(x, y)| {
                    let pixel = scene.get(x as i32, y as i32);
                    u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b)
                })
                .collect();
            brightness.sort_unstable();
            let middle = brightness[brightness.len() / 2];
            assert!(
                (200..=460).contains(&middle),
                "{} is {middle} bright, too dark or too light for a shadow to show",
                haunt.name()
            );
        }
    }
}
