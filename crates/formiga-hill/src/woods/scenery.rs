//! The Woods, at the eye level of someone standing in them: trunks rising out of the top of the
//! picture on every side and receding into a green-blue haze, the canopy closing in overhead with
//! the sun slanting down through it from the upper left, a stream winding across the back of the
//! glade, and a floor of leaf litter and moss. Unlike the station's side-on platform, the Green
//! seen from a branch, or the Fairground's open field, the Woods close in around whoever is in
//! them: the oak and a near trunk hem in the sides, and the far trees fade away rather than end.
//!
//! Every place worth searching is painted where `SPOTS` puts it, and the ground where a companion
//! stands to search is left clear. Nothing is blown out to white, so the glade can be tinted
//! towards dusk as the light runs out.

use super::{LOG, OAK};
use crate::paint::{
    Ramp, chance, ellipse, hline, line, mix, noise, polygon, put, rgb, rgba, vline,
};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const OAK_BARK: Ramp = Ramp::new(0x2b221d, 0x44372d, 0x5d4c3e, 0x786652, 0x93816a);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const CANOPY: Ramp = Ramp::new(0x142a24, 0x1d3a30, 0x28503c, 0x3a6848, 0x56865a);
const UNDERGROWTH: Ramp = Ramp::new(0x1c3628, 0x284a32, 0x36603c, 0x4c7a48, 0x6a9658);
const LITTER: Ramp = Ramp::new(0x382a1c, 0x54402e, 0x6e5238, 0x886a48, 0xa2845c);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const WATER: Ramp = Ramp::new(0x1a3034, 0x25464a, 0x335e5e, 0x4a7a74, 0x74a094);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const GRASS: Ramp = Ramp::new(0x2a4224, 0x3a5a2e, 0x507438, 0x6a8e46, 0x88a85a);
const HAZEL_LEAF: Ramp = Ramp::new(0x29441f, 0x3a5c2a, 0x4f7834, 0x6a9442, 0x8cb058);
const HAZEL_BARK: Ramp = Ramp::new(0x3a302a, 0x55483e, 0x6e6052, 0x877a68, 0xa0947e);
const WOOD: Ramp = Ramp::new(0x4e3626, 0x6c4e36, 0x8c6a4a, 0xa88660, 0xbea078);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const REED: Ramp = Ramp::new(0x34442a, 0x4c5e36, 0x667a44, 0x829654, 0xa2b06c);
const BULRUSH: Ramp = Ramp::new(0x2a1a12, 0x43291c, 0x5c3a26, 0x765032, 0x8f6640);
const BRAMBLE: Ramp = Ramp::new(0x1b3320, 0x284628, 0x385e32, 0x4e7840, 0x6a9450);
const CANE: Ramp = Ramp::new(0x3a1e22, 0x56292c, 0x723a36, 0x8c4e42, 0xa66a52);
const BERRY: Ramp = Ramp::new(0x1a0f1e, 0x2b1832, 0x42244a, 0x5e3666, 0x86608a);
const UNRIPE: Ramp = Ramp::new(0x5a1a22, 0x7a2a30, 0x9a3a3a, 0xb85448, 0xd07a62);
const CAP: Ramp = Ramp::new(0x5a3424, 0x83502f, 0xa66e42, 0xc08c5a, 0xd6aa7a);
const TOADSTOOL: Ramp = Ramp::new(0x561c1c, 0x842a26, 0xa83c30, 0xc45a44, 0xd8826a);
const STALK: Ramp = Ramp::new(0x7a6a56, 0xa89878, 0xc4b694, 0xd6caa8, 0xe2d8bc);
const FOAM: Rgba = rgb(0xc8dcd2);
/// The inside of a hole: dark, but brown rather than black, so a creature's outline still reads
/// against it.
const HOLLOW: Rgba = rgb(0x18120e);
const HOLLOW_FLOOR: Rgba = rgb(0x2c2018);
const SUN: u32 = 0xffe2a2;
const SHADE: Rgba = rgba(0x14241c, 72);

/// The trees on the far bank nearest the eye: their left edges and widths.
const NEAR_TRUNKS: [(i32, i32); 3] = [(16, 15), (100, 12), (250, 14)];
/// Shafts of sun: where each crosses the top of the picture, its half-width, where it comes out
/// of the canopy, and where it lands.
const BEAMS: [(f32, f32, i32, i32); 3] = [
    (126.0, 9.0, 8, 160),
    (10.0, 7.0, 10, 194),
    (190.0, 8.0, 8, 154),
];
/// How far right a sunbeam travels for every pixel it falls.
const SLANT: f32 = 0.4;

/// Everything behind the companions, filling every pixel.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    deep_woods(&mut scene);
    far_bank(&mut scene);
    ledge(&mut scene);
    stream(&mut scene);
    shallows(&mut scene);
    waterfall(&mut scene);
    floor(&mut scene);
    oak(&mut scene);
    root_tangle(&mut scene);
    fox_hole(&mut scene);
    mossy_bank(&mut scene);
    hollow_log(&mut scene);
    hazel(&mut scene);
    mushroom_ring(&mut scene);
    bramble(&mut scene);
    ferns(&mut scene);
    reeds(&mut scene);
    sunbeams(&mut scene);
    scene
}

/// What is nearer the eye than anyone: the edge of a trunk at the left, leaves hanging into the
/// top corners, and litter along the bottom. Transparent everywhere else.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    near_trunk(&mut scene);
    leaf_spray(
        &mut scene,
        (-8, -6),
        &[(8, 4), (24, 8), (40, 6), (54, 1)],
        900,
    );
    leaf_spray(
        &mut scene,
        (WIDTH + 8, -6),
        &[(WIDTH - 10, 5), (WIDTH - 26, 7), (WIDTH - 42, 3)],
        920,
    );
    front_litter(&mut scene);
    scene
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

/// The air between the trees at height `y`: shade under the canopy, brightening to a haze where
/// the woods recede out of sight just above the far bank.
fn air(y: i32) -> Rgba {
    const STOPS: [(i32, u32); 5] = [
        (0, 0x1e382f),
        (30, 0x335a4b),
        (64, 0x5f8a7c),
        (96, 0x8fb0a2),
        (118, 0x7a9c8e),
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

/// A number from the hash in `0..n`.
fn pick(index: i32, axis: i32, salt: u32, n: i32) -> i32 {
    (noise(index, axis, salt) % n.max(1) as u32) as i32
}

/// The broken outline of an ellipse, for ripples and rings of growth.
fn ring(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), color: Rgba, salt: u32) {
    let mut done: Vec<(i32, i32)> = Vec::new();
    let steps = (rx + ry) * 4;
    for step in 0..steps {
        let angle = step as f32 / steps as f32 * TAU;
        let point = (
            cx + (angle.cos() * rx as f32).round() as i32,
            cy + (angle.sin() * ry as f32).round() as i32,
        );
        if done.contains(&point) || chance(step, rx, salt, 70) {
            continue;
        }
        done.push(point);
        put(scene, point.0, point.1, color);
    }
}

/// A clump of leaves, its rim scalloped into smaller clusters, lit from the upper left and lost
/// in the haze by `depth`.
fn clump(scene: &mut Canvas, (cx, cy): (i32, i32), radius: i32, ramp: Ramp, depth: f32, salt: u32) {
    let ry = ((radius as f32 * 0.8) as i32).max(1);
    let rim = radius * 3;
    let bumps: Vec<(i32, i32, i32, f32)> = (0..rim)
        .map(|step| {
            let angle = step as f32 / rim as f32 * TAU;
            let reach = radius as f32 - 1.0 + pick(step, 0, salt, 3) as f32;
            let size = 1 + pick(step, 1, salt, 2) + radius / 9;
            (
                cx + (angle.cos() * reach).round() as i32,
                cy + (angle.sin() * reach * 0.8).round() as i32,
                size,
                angle.cos() + angle.sin(),
            )
        })
        .collect();
    for &(x, y, size, _) in &bumps {
        ellipse(scene, x, y, size + 1, size + 1, hazed(ramp.edge, y, depth));
    }
    for y in cy - ry..=cy + ry {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = ((x - cx) as f32 / radius as f32, (y - cy) as f32 / ry as f32);
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            let light = dx + dy;
            let cell = noise(x.div_euclid(2), y.div_euclid(2), salt + 7) % 8;
            let color = if light < -0.6 {
                match cell {
                    0 => ramp.shine,
                    1..=4 => ramp.light,
                    _ => ramp.base,
                }
            } else if light < 0.2 {
                match cell {
                    0 | 1 => ramp.light,
                    2..=5 => ramp.base,
                    _ => ramp.shadow,
                }
            } else if light < 0.8 {
                if cell < 2 { ramp.base } else { ramp.shadow }
            } else if cell < 3 {
                ramp.shadow
            } else {
                ramp.edge
            };
            put(scene, x, y, hazed(color, y, depth));
        }
    }
    for &(x, y, size, light) in &bumps {
        let (x, y) = (x - (x - cx).signum(), y - (y - cy).signum());
        let color = if light < -0.5 {
            ramp.light
        } else if light < 0.6 {
            ramp.base
        } else {
            ramp.shadow
        };
        ellipse(scene, x, y, size, size, hazed(color, y, depth));
        if light < -0.9 {
            put(scene, x - 1, y - 1, hazed(ramp.shine, y, depth));
        }
    }
}

/// A rounded stone lit from the upper left, with moss on its crown.
fn boulder(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), salt: u32) {
    ellipse(scene, cx, cy, rx, ry, ROCK.edge);
    for y in cy - ry + 1..cy + ry {
        for x in cx - rx + 1..cx + rx {
            let (dx, dy) = (
                (x - cx) as f32 / (rx - 1).max(1) as f32,
                (y - cy) as f32 / (ry - 1).max(1) as f32,
            );
            let r = dx * dx + dy * dy;
            if r > 1.0 {
                continue;
            }
            let light = dx + dy;
            let mut color = if light < -0.9 && r > 0.3 {
                ROCK.shine
            } else if light < -0.3 {
                ROCK.light
            } else if light < 0.6 {
                ROCK.base
            } else {
                ROCK.shadow
            };
            if chance(x, y, salt, 30) {
                color = mix(color, ROCK.edge, 0.4);
            }
            put(scene, x, y, color);
        }
    }
    // Moss along the crown, where the damp sits.
    for x in cx - rx + 2..cx + rx - 1 {
        let across = (x - cx) as f32 / rx as f32;
        let top = cy - ((1.0 - across * across).max(0.0).sqrt() * ry as f32).round() as i32;
        if chance(x, top, salt + 1, 150) {
            put(scene, x, top, MOSS.base);
            put(scene, x, top + 1, MOSS.shadow);
            if across < 0.0 {
                put(scene, x, top, MOSS.light);
            }
        }
    }
}

/// A ramp's tone by level, 0 its edge and 4 its shine.
fn tone(ramp: Ramp, level: i32) -> Rgba {
    match level {
        ..=0 => ramp.edge,
        1 => ramp.shadow,
        2 => ramp.base,
        3 => ramp.light,
        _ => ramp.shine,
    }
}

// ---------------------------------------------------------------------------------------------
// The woods beyond the stream
// ---------------------------------------------------------------------------------------------

/// The air, the trees in four ranks from the haze forwards, each rank with its own canopy and
/// undergrowth, and the near canopy with the sun breaking through it.
fn deep_woods(scene: &mut Canvas) {
    for y in 0..124 {
        for x in 0..WIDTH {
            // Stepped in bands with dithered edges, like paint rather than a smooth fade.
            let level = (y + pick(x, y, 11, 5) - 2).div_euclid(5) * 5;
            scene.set(x, y, air(level));
        }
    }
    // Pockets of sunlit haze in the distance, where the beams come down.
    for (cx, cy, rx, ry) in [(60, 76, 46, 22), (200, 70, 54, 26), (318, 82, 30, 16)] {
        ellipse(scene, cx, cy, rx, ry, rgba(0xdce4b0, 12));
        ellipse(
            scene,
            cx,
            cy + 4,
            rx * 3 / 5,
            ry * 3 / 5,
            rgba(0xdce4b0, 14),
        );
    }
    // The farthest trees, barely darker than the haze they stand in.
    for (index, x) in (-4..WIDTH + 4).step_by(13).enumerate() {
        let i = index as i32;
        let width = 2 + pick(i, 1, 21, 3);
        trunk(
            scene,
            x + pick(i, 0, 21, 7) - 3,
            width,
            96 + pick(i, 2, 21, 3),
            0.74,
            200 + index as u32,
        );
    }
    canopy_row(scene, 42, 17, (8, 11), 0.66, 300);
    undergrowth_row(scene, 94, 9, (4, 6), 0.62, 310);
    let middling = [
        (2, 5),
        (66, 4),
        (86, 6),
        (124, 5),
        (164, 4),
        (198, 6),
        (236, 5),
        (274, 4),
        (318, 6),
    ];
    for (index, &(x, width)) in middling.iter().enumerate() {
        trunk(
            scene,
            x,
            width,
            100 + index as i32 % 3,
            0.5,
            400 + index as u32,
        );
    }
    canopy_row(scene, 30, 21, (10, 13), 0.46, 500);
    undergrowth_row(scene, 99, 11, (4, 7), 0.42, 510);
    for (index, &(x, width)) in [(52, 9), (140, 8), (182, 9), (292, 10)].iter().enumerate() {
        trunk(
            scene,
            x,
            width,
            104 + index as i32 % 2,
            0.28,
            600 + index as u32,
        );
    }
    canopy_row(scene, 17, 25, (12, 16), 0.24, 700);
    undergrowth_row(scene, 103, 12, (5, 8), 0.2, 710);
    for (index, &(x, width)) in NEAR_TRUNKS.iter().enumerate() {
        trunk(scene, x, width, 110, 0.06, 800 + index as u32);
    }
    near_canopy(scene);
}

/// One tree's trunk, rising out of the top of the picture to stand at `base`, lit from the left,
/// its bark furrowed and mossy towards the foot.
fn trunk(scene: &mut Canvas, left: i32, width: i32, base: i32, depth: f32, salt: u32) {
    let lean = pick(left, 0, salt, 3) - 1;
    for y in 0..base {
        let shift = lean * (base - y) / 70;
        let flare = ((y - (base - 6)).max(0) * width) / 14;
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
                if height < 0.4
                    && patches(x, y, (3, 7), salt + 1) > 0.38 + height * 1.2 + across * 0.3
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
            (x + pick(i, 1, salt, 7) - 3, y + pick(i, 2, salt, 9) - 4),
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
    for py in y + 1..y + 8 {
        for x in 0..WIDTH {
            scene.set(x, py, hazed(UNDERGROWTH.edge, py, depth));
        }
    }
    for (index, x) in (-8..WIDTH + 8).step_by(spacing).enumerate() {
        let i = index as i32;
        let r = radius.0 + pick(i, 0, salt, radius.1 - radius.0 + 1);
        clump(
            scene,
            (x + pick(i, 1, salt, 5) - 2, y + pick(i, 2, salt, 7) - 3),
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
    for (index, x) in (-16..WIDTH + 16).step_by(23).enumerate() {
        let i = index as i32;
        let cx = x + pick(i, 1, 1001, 7) - 3;
        if gaps.iter().any(|gap| (cx - gap).abs() < 12) {
            continue;
        }
        clump(
            scene,
            (cx, 2 + pick(i, 2, 1001, 7)),
            15 + pick(i, 0, 1001, 6),
            CANOPY,
            0.0,
            1010 + index as u32 * 7,
        );
    }
    // Sunlit leaves high up, glimpsed in flecks through the gaps, with the near leaves dark
    // against them.
    for (index, &gap) in gaps.iter().enumerate() {
        let salt = 1100 + index as u32 * 10;
        ellipse(scene, gap, 5, 18, 10, rgba(0xe8e2a0, 16));
        ellipse(scene, gap, 4, 11, 6, rgba(0xe8e2a0, 18));
        for fleck in 0..16 {
            let spread = 6 + fleck / 2;
            let x = gap + pick(fleck, 0, salt, spread * 2 + 1) - spread;
            let y = 1 + pick(fleck, 1, salt, 6 + fleck / 3);
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
        for leaf in 0..7 {
            let x = gap + pick(leaf, 4, salt, 21) - 10;
            let y = 2 + pick(leaf, 5, salt, 9);
            ellipse(scene, x, y, 2, 1, CANOPY.shadow);
            put(scene, x - 1, y - 1, CANOPY.base);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The stream
// ---------------------------------------------------------------------------------------------

/// How much the pool widens the stream at `x`: 1 in its middle, 0 beyond it.
fn pool_bulge(x: f32) -> f32 {
    let t = (x - 212.0) / 40.0;
    (1.0 - t * t).max(0.0)
}

/// The far edge of the stream's water at column `x`.
fn far_edge(x: i32) -> i32 {
    let fx = x as f32;
    let wind = 111.5 + 1.6 * (fx * 0.045).sin() + 0.8 * (fx * 0.13 + 1.0).sin();
    (wind - 3.0 * pool_bulge(fx)).round() as i32
}

/// The near edge of the stream: where the floor of the glade stops.
fn near_edge(x: i32) -> i32 {
    let fx = x as f32;
    let wind = 130.5 + 1.2 * (fx * 0.037 + 2.0).sin() + 0.6 * (fx * 0.11).sin();
    (wind + 1.5 * pool_bulge(fx)).round() as i32
}

/// The top of the far bank's earthen face.
fn bank_top(x: i32) -> i32 {
    far_edge(x) - 5 - pick(x.div_euclid(4), 0, 41, 2)
}

/// The far bank: a row of bushes along its top, then its face of earth and roots down to the
/// water, edged with moss.
fn far_bank(scene: &mut Canvas) {
    let mut x = -6;
    let mut index = 0;
    while x < WIDTH + 6 {
        let radius = 3 + pick(index, 0, 1201, 7);
        let lift = pick(index, 2, 1201, 3) + radius / 3;
        // Bushes of more than one kind, with gaps where only ferns grow.
        let ramp = if pick(index, 3, 1201, 3) == 0 {
            HAZEL_LEAF
        } else {
            UNDERGROWTH
        };
        if pick(index, 4, 1201, 5) != 0 {
            clump(
                scene,
                (x, bank_top(x) - 1 - lift),
                radius,
                ramp,
                0.12,
                1210 + index as u32 * 3,
            );
        }
        if index % 3 == 1 {
            for (angle, length) in [(-130.0, 9.0), (-90.0, 11.0), (-50.0, 9.0)] {
                frond(scene, (x + radius + 2, bank_top(x)), (angle, length), 2);
            }
        }
        x += radius + 3 + pick(index, 1, 1201, 5);
        index += 1;
    }
    for x in 0..WIDTH {
        let (top, water) = (bank_top(x), far_edge(x));
        for y in top..water {
            let down = y - top;
            let color = match down {
                0 => MOSS.light,
                1 => {
                    if chance(x, y, 1202, 120) {
                        MOSS.base
                    } else {
                        EARTH.light
                    }
                }
                _ if y == water - 1 => EARTH.edge,
                _ => {
                    // Strata of earth, darker towards the water.
                    let band = (y + pick(x.div_euclid(6), 0, 1203, 2)) % 3;
                    let base = if down > 3 { EARTH.shadow } else { EARTH.base };
                    if band == 0 {
                        mix(base, EARTH.edge, 0.4)
                    } else {
                        base
                    }
                }
            };
            scene.set(x, y, color);
        }
        // Roots dangling from the bank, and grass on its lip.
        if pick(x.div_euclid(3), 1, 1204, 7) == 0 && x % 3 == 1 {
            for y in top + 2..water - 1 {
                put(scene, x + (y - top) / 3 % 2, y, BARK.shadow);
            }
            put(scene, x, top + 2, BARK.light);
        }
        if chance(x, top, 1205, 110) {
            let tall = 1 + pick(x, 2, 1205, 3);
            for dy in 1..=tall {
                put(
                    scene,
                    x,
                    top - dy,
                    if dy == tall { GRASS.light } else { GRASS.base },
                );
            }
        }
    }
    for index in 0..9 {
        let x = 8 + pick(index, 0, 1206, WIDTH - 16);
        if (186..244).contains(&x) {
            continue;
        }
        let y = bank_top(x) + 3;
        boulder(
            scene,
            (x, y),
            (2 + pick(index, 1, 1206, 2), 2),
            1207 + index as u32,
        );
    }
}

/// The rocky step in the far bank that the waterfall comes down.
fn ledge(scene: &mut Canvas) {
    polygon(
        scene,
        &[
            (184, 112),
            (190, 96),
            (198, 87),
            (226, 86),
            (236, 95),
            (242, 112),
        ],
        |x, y| {
            Some(if chance(x, y, 1301, 40) {
                ROCK.shadow
            } else {
                ROCK.edge
            })
        },
    );
    let stones = [
        ((199, 90), (6, 5)),
        ((225, 90), (6, 5)),
        ((193, 100), (6, 6)),
        ((231, 100), (6, 6)),
        ((205, 99), (5, 5)),
        ((219, 98), (5, 5)),
        ((189, 108), (6, 5)),
        ((201, 108), (5, 4)),
        ((224, 108), (5, 4)),
        ((236, 108), (7, 5)),
    ];
    for (index, (centre, size)) in stones.into_iter().enumerate() {
        boulder(scene, centre, size, 1310 + index as u32);
    }
    // Ferns spilling over the top of the ledge, either side of where the water comes over.
    for (index, x) in [194, 200, 226, 232].into_iter().enumerate() {
        let lean = if x < 212 { -1 } else { 1 };
        for frond in 0..3 {
            let reach = 4 + frond;
            line(
                scene,
                (x, 87),
                (
                    x + lean * reach,
                    82 + frond * 2 - pick(index as i32, frond, 1320, 2),
                ),
                if frond == 0 { FERN.light } else { FERN.base },
            );
        }
    }
}

/// The colour of the stream at `(x, y)`, between its far and near edges.
fn water_color(x: i32, y: i32, far: i32, near: i32) -> Rgba {
    let wobble = (noise(x, y, 51) % 100) as f32 / 100.0 - 0.5;
    let t = (y - far) as f32 / (near - far).max(1) as f32 + wobble * 0.1;
    // The far bank reflected, then the haze, then the bed showing through nearest the eye.
    let mut color = if t < 0.14 {
        mix(WATER.edge, EARTH.shadow, 0.35)
    } else if t < 0.27 {
        WATER.shadow
    } else if t < 0.5 {
        mix(WATER.light, air(98), 0.3)
    } else if t < 0.74 {
        WATER.base
    } else {
        mix(WATER.shadow, EARTH.base, 0.2)
    };
    // The pool is deeper, and darker towards its middle.
    let (px, py) = ((x - 212) as f32 / 34.0, (y - 121) as f32 / 10.0);
    let deep = 1.0 - (px * px + py * py);
    if deep > 0.0 {
        color = mix(color, WATER.edge, (deep * 3.0).ceil() / 3.0 * 0.5);
    }
    // Glints and dark ripples, drawn out along the current.
    match noise((x + y * 5).div_euclid(6), y, 52) % 15 {
        0 => mix(color, WATER.shine, 0.55),
        1 => mix(color, WATER.edge, 0.3),
        _ => color,
    }
}

/// The stream from edge to edge, the near trunks reflected in it and leaves riding it.
fn stream(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let (far, near) = (far_edge(x), near_edge(x));
        for y in far..near {
            scene.set(x, y, water_color(x, y, far, near));
        }
        // The near trunks, reflected and wavering.
        for &(left, width) in &NEAR_TRUNKS {
            for y in far..near {
                let t = (y - far) as f32 / (near - far) as f32;
                let sway = ((y as f32 * 1.3).sin() * 1.2).round() as i32;
                if t < 0.55 && (left..left + width).contains(&(x + sway)) {
                    put(scene, x, y, rgba(0x2a221e, 80));
                }
            }
        }
    }
    // Leaves riding the current.
    for index in 0..10 {
        let x = 8 + pick(index, 0, 1401, WIDTH - 16);
        if (190..236).contains(&x) {
            continue;
        }
        let (far, near) = (far_edge(x), near_edge(x));
        let y = far + 3 + pick(index, 1, 1401, (near - far - 5).max(1));
        let color = [rgb(0x9a6a34), rgb(0x84482c), rgb(0xa8843e)][pick(index, 2, 1401, 3) as usize];
        hline(scene, x, y, 2, color);
        put(scene, x + 2, y, rgba(0x74a094, 120));
    }
}

/// The shallows: the stream running over a bed of pebbles, with ripples where stones break the
/// surface.
fn shallows(scene: &mut Canvas) {
    for x in 50..136 {
        let s = (1.0 - ((x - 92) as f32 / 42.0).abs()).max(0.0);
        let (far, near) = (far_edge(x), near_edge(x));
        for y in far + 2..near {
            let t = (y - far) as f32 / (near - far) as f32;
            let amount = (s * 1.6).min(1.0) * (0.1 + t * 0.4);
            put(scene, x, y, rgba(0x8c8a62, (amount * 255.0) as u8));
        }
    }
    let pebbles = [
        rgb(0x7a7466),
        rgb(0x8c7a5e),
        rgb(0x6a706a),
        rgb(0x9a8e74),
        rgb(0x5e5a52),
    ];
    for index in 0..46 {
        let x = 60 + pick(index, 0, 1501, 66);
        let (far, near) = (far_edge(x), near_edge(x));
        let y = far + 4 + pick(index, 1, 1501, (near - far - 5).max(1));
        let t = (y - far) as f32 / (near - far) as f32;
        let water = mix(WATER.base, WATER.light, 0.4);
        let color = mix(
            pebbles[pick(index, 2, 1501, 5) as usize],
            water,
            0.55 - t * 0.35,
        );
        let rx = 1 + pick(index, 3, 1501, 2);
        ellipse(scene, x, y, rx, 1, color);
        put(scene, x - rx + 1, y - 1, mix(color, WATER.shine, 0.5));
        put(scene, x + rx, y, mix(color, WATER.edge, 0.4));
    }
    // Stones breaking the surface, the water folding round them.
    for (index, (centre, size)) in [
        ((74, 123), (4, 3)),
        ((110, 119), (3, 2)),
        ((66, 128), (3, 2)),
    ]
    .into_iter()
    .enumerate()
    {
        boulder(scene, centre, size, 1510 + index as u32);
        hline(scene, centre.0 + size.0, centre.1, 4, rgba(0xc8dcd2, 150));
        hline(
            scene,
            centre.0 + size.0 + 5,
            centre.1,
            2,
            rgba(0xc8dcd2, 90),
        );
        hline(
            scene,
            centre.0 - size.0,
            centre.1 + size.1,
            size.0 * 2,
            rgba(0x1a3034, 110),
        );
    }
    for (index, y) in [117, 121, 125, 128].into_iter().enumerate() {
        let x = 64 + pick(index as i32, 0, 1520, 50);
        hline(
            scene,
            x,
            y,
            4 + pick(index as i32, 1, 1520, 4),
            rgba(0xb8d0c4, 110),
        );
    }
}

/// The little waterfall coming down the ledge into the pool, its foam and spray, and the rings
/// it sends out across the pool.
fn waterfall(scene: &mut Canvas) {
    let (top, bottom) = (91, 115);
    for y in top..bottom {
        let spread = (y - top) / 8;
        let (left, right) = (206 - spread, 218 + spread);
        for x in left..=right {
            let color = if x == left {
                WATER.shadow
            } else if x == right {
                WATER.edge
            } else {
                let fall = (y + pick(x, 0, 1601, 8)).div_euclid(4);
                let level = match noise(x, fall, 1602) % 6 {
                    0 => 5,
                    1 | 2 => 4,
                    3 | 4 => 3,
                    _ => 2,
                } + i32::from(x < left + 4)
                    - i32::from(x > right - 3);
                if level >= 5 { FOAM } else { tone(WATER, level) }
            };
            scene.set(x, y, color);
        }
    }
    // The water bending over the lip.
    hline(scene, 206, top - 1, 13, WATER.light);
    hline(scene, 207, top - 2, 11, WATER.shine);
    put(scene, 208, top - 2, FOAM);
    // Foam where it lands, and a breath of spray.
    ellipse(scene, 212, bottom, 18, 6, rgba(0xc8dcd2, 26));
    ellipse(scene, 212, bottom - 1, 11, 4, rgba(0xc8dcd2, 30));
    for index in 0..34 {
        let x = 212 + pick(index, 0, 1603, 25) - 12;
        let y = bottom - 1 + pick(index, 1, 1603, 5);
        let size = 1 + pick(index, 2, 1603, 2);
        let color = if index % 3 == 0 { WATER.shine } else { FOAM };
        ellipse(scene, x, y, size, 1, color);
    }
    for (index, (rx, ry)) in [(14, 2), (21, 3), (28, 4)].into_iter().enumerate() {
        ring(
            scene,
            (212, 118 + index as i32),
            (rx, ry),
            rgba(0xa4c4b8, 130 - index as u8 * 30),
            1610 + index as u32,
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The floor of the glade
// ---------------------------------------------------------------------------------------------

/// Whether `(x, y)` is close to a spot, where something shows itself or a companion stands to
/// search: kept clear of twigs and stones.
fn by_a_spot(x: i32, y: i32) -> bool {
    let near = |(sx, sy): (f32, f32)| (sx - x as f32).abs() < 14.0 && (sy - y as f32).abs() < 8.0;
    super::SPOTS
        .iter()
        .any(|spot| near(spot.stand) || near(spot.sign))
}

/// How near the eye a row of the floor is: 0 at the stream, 1 at the bottom of the picture.
fn nearness(y: i32) -> f32 {
    ((y - 132) as f32 / 84.0).clamp(0.0, 1.0)
}

/// Whether the floor is mossy at `(x, y)`: soft patches, ragged at their edges.
fn mossy(x: i32, y: i32) -> bool {
    patches(x, y, (22, 8), 502) + (noise(x, y, 515) % 100) as f32 / 1400.0 > 0.67
}

/// The floor of the glade: earth, moss and fallen leaves, the lip of the bank, the path, twigs
/// and stones, and flecks of sun.
fn floor(scene: &mut Canvas) {
    // The earth under the litter, darker and lighter in soft patches.
    for x in 0..WIDTH {
        for y in near_edge(x)..HEIGHT {
            let soil = patches(x, y, (18, 7), 501) + (noise(x, y, 511) % 100) as f32 / 600.0;
            let color = if soil < 0.4 {
                LITTER.shadow
            } else if soil < 0.72 {
                mix(LITTER.shadow, LITTER.base, 0.6)
            } else {
                LITTER.base
            };
            scene.set(x, y, color);
        }
    }
    // Moss in raised cushions, lit along their tops and shadowed beneath.
    for x in 0..WIDTH {
        for y in near_edge(x)..HEIGHT {
            if !mossy(x, y) {
                continue;
            }
            let cushion = patches(x, y, (4, 3), 503);
            let color = if !mossy(x, y - 1) && chance(x, y, 516, 150) {
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
            scene.set(x, y, color);
        }
    }
    // Fallen leaves, scattered over it all, bigger and further apart nearer the eye.
    let fallen = [
        LITTER.light,
        rgb(0x8e6434),
        rgb(0x7a4630),
        rgb(0x987a40),
        LITTER.base,
        rgb(0x6e4a2c),
        mix(LITTER.base, MOSS.shadow, 0.4),
    ];
    let mut row = 132;
    while row < HEIGHT + 2 {
        let near = nearness(row);
        let (step_x, step_y) = (3 + (near * 4.0) as i32, 2 + (near * 1.6) as i32);
        for (index, gx) in (-4..WIDTH + 4).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 512, step_x);
            let y = row + pick(i, row + 1, 512, step_y);
            if y <= near_edge(x) || !chance(x, y, 514, 210) {
                continue;
            }
            if mossy(x, y) && !chance(x, y, 513, 40) {
                continue;
            }
            let long = 1 + (near * 1.8) as i32 + pick(i, row + 2, 512, 2);
            let color = fallen[pick(i, row + 3, 512, fallen.len() as i32) as usize];
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
            put(scene, x - long + 1, y, mix(color, LITTER.shine, 0.35));
            hline(
                scene,
                x - long + 2,
                y + 1,
                (long * 2 - 2).max(1),
                rgba(0x1e140e, 110),
            );
        }
        row += step_y;
    }
    // The back of the glade softens into the haze.
    for x in 0..WIDTH {
        for y in near_edge(x)..150 {
            let amount = (1.0 - nearness(y)).powi(3) * 0.24;
            scene.set(x, y, mix(scene.get(x, y), air(112), amount));
        }
    }
    for x in 0..WIDTH {
        let lip = near_edge(x);
        // The lip of the bank catches the light, with grass leaning out over the water.
        put(
            scene,
            x,
            lip,
            if chance(x, lip, 504, 120) {
                MOSS.light
            } else {
                MOSS.base
            },
        );
        put(scene, x, lip + 1, MOSS.shadow);
        put(scene, x, lip - 1, rgba(0x1a3034, 110));
        if chance(x, 0, 505, 80) {
            let tall = 1 + pick(x, 1, 505, 3);
            for dy in 1..=tall {
                put(
                    scene,
                    x,
                    lip - dy,
                    if dy == tall { GRASS.light } else { GRASS.base },
                );
            }
        }
    }
    path(scene);
    // Shade where the canopy is thickest.
    for y in 134..HEIGHT {
        for x in 0..WIDTH {
            if patches(x, y, (40, 14), 506) > 0.66 {
                put(scene, x, y, rgba(0x14241c, 40));
            }
        }
    }
    // Twigs and stones, kept away from wherever anyone stands.
    for index in 0..40 {
        let x = 10 + pick(index, 0, 507, WIDTH - 20);
        let y = 140 + pick(index, 1, 507, HEIGHT - 144);
        if by_a_spot(x, y) {
            continue;
        }
        let long = 3 + pick(index, 2, 507, 4);
        let rise = pick(index, 3, 507, 3) - 1;
        line(scene, (x, y), (x + long, y + rise), BARK.shadow);
        put(scene, x, y - 1, BARK.light);
        put(scene, x + long, y + rise + 1, rgba(0x14241c, 90));
    }
    for index in 0..12 {
        let x = 10 + pick(index, 0, 508, WIDTH - 20);
        let y = 142 + pick(index, 1, 508, HEIGHT - 150);
        if by_a_spot(x, y) {
            continue;
        }
        ellipse(scene, x + 1, y + 2, 3, 1, rgba(0x14241c, 80));
        boulder(
            scene,
            (x, y),
            (2 + pick(index, 2, 508, 2), 2),
            509 + index as u32,
        );
    }
    // Flecks of sun through the leaves.
    for index in 0..70 {
        let x = pick(index, 0, 510, WIDTH);
        let y = 136 + pick(index, 1, 510, HEIGHT - 136);
        let rx = 1 + pick(index, 2, 510, 4);
        ellipse(scene, x, y, rx, 1, rgba(SUN, 40));
        if rx > 2 {
            hline(scene, x - rx / 2, y, rx, rgba(SUN, 36));
        }
    }
}

/// A faint path trodden in from the front left, where the companions come in from the Green.
fn path(scene: &mut Canvas) {
    let points: [(f32, f32); 6] = [
        (-4.0, 208.0),
        (40.0, 203.0),
        (84.0, 194.0),
        (122.0, 188.0),
        (160.0, 184.0),
        (188.0, 178.0),
    ];
    for x in 0..190 {
        let fx = x as f32;
        let Some(pair) = points
            .windows(2)
            .find(|pair| fx >= pair[0].0 && fx <= pair[1].0)
        else {
            continue;
        };
        let t = (fx - pair[0].0) / (pair[1].0 - pair[0].0);
        let centre = pair[0].1 + (pair[1].1 - pair[0].1) * t;
        let half = 2.0 + (centre - 176.0) * 0.16;
        let fade = (1.0 - (fx - 130.0).max(0.0) / 60.0).max(0.0);
        for y in (centre - half - 1.0) as i32..=(centre + half + 1.0) as i32 {
            let off = (y as f32 - centre).abs() / half;
            let edge = off + (noise(x, y, 520) % 100) as f32 / 260.0;
            if edge > 1.0 {
                continue;
            }
            let strength = if edge > 0.75 { 50.0 } else { 90.0 } * fade;
            put(scene, x, y, rgba(0x8e7658, strength as u8));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The old oak and its roots
// ---------------------------------------------------------------------------------------------

/// The left edge of the oak's trunk at height `y`: flaring out at its foot and spreading a little
/// towards its crown.
fn oak_edge(y: i32) -> i32 {
    let fy = y as f32;
    let wobble = (fy * 0.06).sin() * 1.5 + (fy * 0.17).sin() * 0.6;
    let foot = ((fy - 150.0).max(0.0) / 26.0).powi(2) * 12.0;
    let crown = ((24.0 - fy).max(0.0) / 24.0).powi(2) * 6.0;
    (340.0 + wobble - foot - crown).round() as i32
}

/// Where the `k`th furrow of the oak's bark runs at height `y`: roughly upright, wandering.
fn furrow(k: i32, y: i32) -> f32 {
    k as f32 * 8.0 + pick(k, 0, 1700, 3) as f32 + (patches(k * 40, y, (40, 20), 1701) - 0.5) * 6.0
}

/// The old oak at the right, too big to fit the picture: its furrowed trunk, a limb reaching
/// into the canopy, its hollow and the ivy climbing it.
fn oak(scene: &mut Canvas) {
    let base = OAK.3 as i32;
    ellipse(scene, 352, base, 34, 4, SHADE);
    for y in 0..=base {
        let left = oak_edge(y);
        for x in left..WIDTH {
            let inset = x - left;
            // The nearest furrow, and which side of it this is: plates of bark between deep
            // fissures, each plate catching the light on its left as it rises out of one.
            let k = x.div_euclid(8);
            let offset = (k - 1..=k + 2)
                .map(|k| x as f32 + 0.5 - furrow(k, y))
                .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                .unwrap_or(4.0);
            let bridged = patches(k * 23, y, (23, 6), 1702) > 0.8;
            let mut level = if offset.abs() < 1.0 && !bridged {
                0
            } else if offset > 0.0 && offset < 2.4 {
                3
            } else if offset < 0.0 && offset > -2.0 {
                1
            } else {
                2
            };
            // The whole trunk rounds away from the light on its right.
            if inset < 5 && level > 0 {
                level += 1;
            } else if inset > 30 && level > 1 {
                level -= 1;
            }
            let mut color = if inset == 0 {
                OAK_BARK.edge
            } else {
                tone(OAK_BARK, level.min(3))
            };
            if level >= 3 && inset < 10 && chance(x, y, 1703, 40) {
                color = OAK_BARK.shine;
            }
            // Moss up the foot and along the lit side, and in the furrows higher up.
            let height = (base - y) as f32;
            if inset > 0 && patches(x, y, (4, 8), 1704) > 0.42 + height / 90.0 + inset as f32 / 60.0
            {
                color = tone(MOSS, level.clamp(1, 3));
            } else if level == 0 && y > 50 && chance(x, y, 1705, 30) {
                color = MOSS.shadow;
            }
            scene.set(x, y, color);
        }
    }
    // A great limb leaving the trunk for the canopy, mossy along its top.
    root(
        scene,
        &[(346, 40), (333, 27), (319, 18), (305, 12), (290, 8)],
        (17.0, 7.0),
        1706,
    );
    for (index, (centre, radius)) in [((290, 4), 12), ((306, 3), 9), ((276, 8), 8)]
        .into_iter()
        .enumerate()
    {
        clump(scene, centre, radius, CANOPY, 0.0, 1707 + index as u32 * 5);
    }
    oak_hollow(scene, (354, 146));
    ivy(scene);
}

/// The dark hollow low in the oak, its rim rolled where the bark grew round it.
fn oak_hollow(scene: &mut Canvas, (cx, cy): (i32, i32)) {
    ellipse(scene, cx, cy, 8, 11, OAK_BARK.edge);
    for y in cy - 10..=cy + 10 {
        for x in cx - 7..=cx + 7 {
            let (dx, dy) = ((x - cx) as f32 / 7.0, (y - cy) as f32 / 10.0);
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            let light = dx + dy;
            put(
                scene,
                x,
                y,
                if light < -0.5 {
                    OAK_BARK.shine
                } else if light < 0.3 {
                    OAK_BARK.light
                } else {
                    OAK_BARK.shadow
                },
            );
        }
    }
    ellipse(scene, cx, cy, 5, 8, OAK_BARK.edge);
    ellipse(scene, cx, cy, 4, 7, HOLLOW);
    // The inside of the far wall catches a little of the light from the upper left.
    for y in cy..=cy + 6 {
        for x in cx + 1..=cx + 4 {
            let (dx, dy) = ((x - cx) as f32 / 4.0, (y - cy) as f32 / 7.0);
            let r = dx * dx + dy * dy;
            if (0.55..=1.0).contains(&r) {
                put(scene, x, y, HOLLOW_FLOOR);
            }
        }
    }
    // Moss on the sill.
    for x in cx - 4..=cx + 4 {
        if chance(x, cy, 1710, 170) {
            put(scene, x, cy + 8, MOSS.base);
            put(scene, x, cy + 9, MOSS.shadow);
        }
    }
    hline(scene, cx - 3, cy + 8, 3, MOSS.light);
}

/// Ivy climbing the right of the oak, clear of the hollow.
fn ivy(scene: &mut Canvas) {
    let leaf = Ramp::new(0x16301e, 0x1f4228, 0x2c5832, 0x3e7040, 0x5a8a50);
    let mut previous = (372, 176);
    for y in (40..176).rev() {
        let x = 372 + ((y as f32 * 0.09).sin() * 4.0) as i32;
        line(scene, previous, (x, y), leaf.edge);
        previous = (x, y);
        if y % 6 == 0 {
            let side = if (y / 6) % 2 == 0 { -1 } else { 1 };
            let (lx, ly) = (x + side * 2, y);
            ellipse(scene, lx, ly, 2, 1, leaf.edge);
            put(scene, lx, ly, leaf.base);
            put(scene, lx - 1, ly - 1, leaf.light);
            put(scene, lx + side, ly, leaf.shadow);
        }
    }
}

/// One of the oak's roots lying on the ground, or a limb reaching up from it, along `points`:
/// `thick` pixels deep where it leaves the trunk and `thin` at its end, lit along its top.
fn root(scene: &mut Canvas, points: &[(i32, i32)], (thick, thin): (f32, f32), salt: u32) {
    let lengths: Vec<f32> = points
        .windows(2)
        .map(|pair| {
            let (dx, dy) = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            ((dx * dx + dy * dy) as f32).sqrt()
        })
        .collect();
    let total: f32 = lengths.iter().sum();
    let mut walked = 0.0;
    for (pair, length) in points.windows(2).zip(&lengths) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        let steps = (length * 2.0).ceil() as i32;
        for step in 0..=steps {
            let t = step as f32 / steps.max(1) as f32;
            let x = x0 as f32 + (x1 - x0) as f32 * t;
            let y = y0 as f32 + (y1 - y0) as f32 * t;
            let along = (walked + length * t) / total;
            let half = (thick + (thin - thick) * along) / 2.0;
            let (px, top, bottom) = (
                x.round() as i32,
                (y - half).round() as i32,
                (y + half).round() as i32,
            );
            for py in top..=bottom {
                // Where it overlaps the trunk it grows out of it, so it has no outline there.
                let in_trunk = py <= OAK.3 as i32 && px > oak_edge(py);
                if in_trunk && (py == top || py == bottom) {
                    continue;
                }
                let mut color = if py == top || py == bottom {
                    OAK_BARK.edge
                } else if py == top + 1 {
                    OAK_BARK.light
                } else if py >= bottom - 1 {
                    OAK_BARK.shadow
                } else {
                    OAK_BARK.base
                };
                if py == top + 1 && chance(px, py, salt, 70) {
                    color = MOSS.base;
                } else if py > top + 1 && chance(px.div_euclid(3), py, salt + 1, 30) {
                    color = mix(color, OAK_BARK.edge, 0.5);
                }
                scene.set(px, py, color);
            }
        }
        walked += length;
    }
}

/// One of the oak's roots: the points it runs along, how thick it is where it leaves the trunk
/// and at its end, and its salt.
type Root = (&'static [(i32, i32)], (f32, f32), u32);

/// The roots lying on the soil, drawn in this order.
const ROOTS: [Root; 5] = [
    (
        &[(342, 176), (328, 181), (316, 183), (304, 184), (291, 187)],
        (11.0, 3.0),
        1801,
    ),
    (&[(386, 174), (380, 184), (374, 191)], (12.0, 4.0), 1802),
    SETT_ROOT,
    (
        &[(350, 175), (343, 185), (333, 193), (321, 197)],
        (12.0, 3.0),
        1804,
    ),
    (&[(334, 186), (326, 188), (316, 187)], (4.0, 2.0), 1805),
];

/// The root that ends beside the badger's sett, which is dug in under its tip.
const SETT_ROOT: Root = (&[(366, 175), (360, 186), (349, 194)], (11.0, 3.0), 1803);

/// The oak's roots sprawling forward and to the left over the soil, a dark nook among them.
fn root_tangle(scene: &mut Canvas) {
    // Bare soil between the roots, and the nook where something might be dug up.
    ellipse(scene, 320, 186, 30, 9, rgba(0x241a16, 90));
    ellipse(scene, 316, 190, 7, 3, EARTH.shadow);
    ellipse(scene, 316, 190, 5, 2, EARTH.edge);
    put(scene, 313, 189, EARTH.light);
    for (points, widths, salt) in ROOTS {
        root(scene, points, widths, salt);
    }
    // Pebbles turned up among them.
    for (x, y) in [(328, 189), (306, 189), (340, 196)] {
        boulder(scene, (x, y), (2, 1), 1806 + x as u32);
    }
}

// ---------------------------------------------------------------------------------------------
// The places to search on the near bank
// ---------------------------------------------------------------------------------------------

/// A dome rising from the ground, `half` wide each side of `cx` and `tall` above `base`, its
/// surface shaded from the upper left. Calls `paint` with the pixel and how lit it is (lower is
/// brighter) and how far down the dome it is.
fn dome(
    scene: &mut Canvas,
    (cx, base): (i32, i32),
    (half, tall): (i32, i32),
    power: f32,
    mut paint: impl FnMut(i32, i32, f32, bool) -> Rgba,
) {
    for x in cx - half..=cx + half {
        let u = (x - cx) as f32 / half as f32;
        let h = tall as f32 * (1.0 - u * u).max(0.0).powf(power);
        let top = base - h.round() as i32;
        for y in top..base {
            let v = (y - top) as f32 / h.max(1.0);
            let lit = u * 0.9 + v * 1.1 - 0.6;
            let color = paint(x, y, lit, y == top);
            put(scene, x, y, color);
        }
    }
}

/// A grassy hummock at the left with a fox's burrow dug into its front.
fn fox_hole(scene: &mut Canvas) {
    let (cx, base) = (41, 152);
    ellipse(scene, cx + 4, base, 30, 3, SHADE);
    dome(scene, (cx, base), (27, 24), 0.6, |x, y, lit, top| {
        if top {
            return GRASS.edge;
        }
        let cell = noise(x, y.div_euclid(2), 1901) % 6;
        let level = if lit < -0.45 {
            3
        } else if lit < 0.25 {
            2
        } else if lit < 0.75 {
            1
        } else {
            0
        } + match cell {
            0 => 1,
            1 => -1,
            _ => 0,
        };
        tone(GRASS, level.clamp(0, 4))
    });
    // Grass standing up along the top of it.
    for x in cx - 25..=cx + 25 {
        let u = (x - cx) as f32 / 27.0;
        let top = base - (24.0 * (1.0 - u * u).max(0.0).powf(0.6)).round() as i32;
        if chance(x, 0, 1902, 140) {
            let tall = 1 + pick(x, 1, 1902, 4);
            let lean = if u < 0.0 { -1 } else { 1 } * i32::from(tall > 2);
            line(scene, (x, top), (x + lean, top - tall), GRASS.base);
            put(scene, x + lean, top - tall, GRASS.light);
        }
    }
    // Bare earth worn round the mouth, ragged where the grass gives way to it.
    let (mx, floor_y) = (40, 149);
    for y in 134..=floor_y {
        for x in 28..=52 {
            let (dx, dy) = ((x - mx) as f32 / 12.0, (y - floor_y) as f32 / 14.0);
            let r = dx * dx + dy * dy + (noise(x, y, 1903) % 100) as f32 / 500.0;
            if r > 1.0 {
                continue;
            }
            let light = dx + dy * 0.6;
            let level = if r > 0.85 {
                0
            } else if light < -0.7 {
                3
            } else if light < -0.1 {
                2
            } else {
                1
            };
            put(scene, x, y, tone(EARTH, level));
        }
    }
    // The mouth: an arch going back into the dark, its floor worn smooth.
    for y in 137..=floor_y {
        let rise = (floor_y - y) as f32 / 12.0;
        let half = if rise < 0.45 {
            7.0
        } else {
            7.0 * (1.0 - ((rise - 0.45) / 0.55).powi(2)).max(0.0).sqrt()
        };
        let (from, to) = (
            (mx as f32 - half).round() as i32,
            (mx as f32 + half).round() as i32,
        );
        for x in from..=to {
            let color = if y > floor_y - 3 && x > mx - 3 {
                HOLLOW_FLOOR
            } else if x >= to - 1 && y > floor_y - 7 {
                mix(HOLLOW, HOLLOW_FLOOR, 0.6)
            } else {
                HOLLOW
            };
            put(scene, x, y, color);
        }
    }
    // Grass and roots hanging over the top of it.
    for x in 34..=46 {
        if chance(x, 0, 1904, 150) {
            let long = 1 + pick(x, 1, 1904, 3);
            vline(scene, x, 136, long, GRASS.base);
            put(scene, x, 136, GRASS.light);
        }
    }
    for (x, long) in [(37, 4), (42, 5)] {
        line(scene, (x, 137), (x + 1, 137 + long), BARK.base);
        put(scene, x + 1, 137 + long, BARK.light);
    }
    // The spoil kicked out in front, a low fan of loose earth and pebbles.
    for y in floor_y..floor_y + 6 {
        let half = 14 - (y - floor_y) * 2;
        for x in mx - half..=mx + half {
            if chance(x, y, 1905, 200) {
                let color = if y == floor_y {
                    EARTH.light
                } else if chance(x, y, 1906, 70) {
                    EARTH.shine
                } else {
                    EARTH.base
                };
                put(scene, x, y, color);
            }
        }
    }
    for (x, y) in [(31, 151), (47, 152), (36, 153)] {
        put(scene, x, y, ROCK.light);
        put(scene, x + 1, y, ROCK.shadow);
    }
}

/// A low hump of moss, cushioned and soft, with a stone or two showing through.
fn mossy_bank(scene: &mut Canvas) {
    let (cx, base) = (252, 160);
    ellipse(scene, cx + 4, base, 26, 3, SHADE);
    dome(scene, (cx, base), (22, 17), 0.55, |x, y, lit, top| {
        if top {
            return MOSS.edge;
        }
        // Cushions: their crowns lighter, the creases between them darker.
        let cushion = patches(x, y, (5, 3), 1949);
        let bump = if cushion > 0.66 {
            1
        } else if cushion < 0.3 {
            -1
        } else {
            0
        };
        let level = if lit < -0.45 {
            3
        } else if lit < 0.3 {
            2
        } else if lit < 0.8 {
            1
        } else {
            0
        } + bump;
        tone(MOSS, level.clamp(0, 4))
    });
    // Earth showing at its foot, a stone half buried, and a sprig of wood sorrel.
    for x in cx - 18..=cx + 18 {
        if chance(x, 0, 1950, 140) {
            put(scene, x, base - 1, EARTH.base);
        }
    }
    boulder(scene, (cx + 12, base - 3), (3, 2), 1951);
    boulder(scene, (cx - 15, base - 2), (2, 2), 1952);
    for (x, y) in [(cx - 8, 147), (cx + 4, 145), (cx + 14, 151)] {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1)] {
            put(scene, x + dx, y + dy, GRASS.shine);
        }
        put(scene, x, y + 1, GRASS.shadow);
    }
    put(scene, cx - 3, 149, rgb(0xd8c8cc));
    put(scene, cx + 9, 153, rgb(0xd8c8cc));
}

/// The fallen log, lying across `LOG` with its hollow end open to the left.
fn hollow_log(scene: &mut Canvas) {
    let (left, top, right, bottom) = (LOG.0 as i32, LOG.1 as i32, LOG.2 as i32, LOG.3 as i32);
    let (end_x, cy) = (left + 8, (top + bottom) / 2 + 1);
    ellipse(scene, (left + right) / 2 + 4, bottom, 38, 3, SHADE);
    // The body, a cylinder lit from above and to the left, broken off ragged at its far end.
    for x in end_x..right {
        let from_end = right - 1 - x;
        // Splintered where it snapped: a jagged end, highest at the bottom.
        let shrink_top = [12, 6, 9, 3, 5, 1]
            .get(from_end as usize)
            .copied()
            .unwrap_or(0);
        let shrink_bottom = [3, 1].get(from_end as usize).copied().unwrap_or(0);
        let (t, b) = (top + shrink_top, bottom - shrink_bottom);
        // Its bark runs in ridges along its length, wavering.
        let waver = ((x as f32 * 0.23).sin() * 1.3 + (x as f32 * 0.07).sin()).round() as i32;
        for y in t..=b {
            let v = (y - top) as f32 / (bottom - top) as f32;
            let mut level = if v < 0.14 {
                2
            } else if v < 0.38 {
                3
            } else if v < 0.62 {
                2
            } else if v < 0.86 {
                1
            } else {
                0
            };
            let groove = (y + waver).rem_euclid(4);
            if groove == 0 && chance(x.div_euclid(3), y, 2001, 190) {
                level -= 1;
            } else if groove == 1 && level >= 2 && chance(x.div_euclid(2), y, 2002, 90) {
                level += 1;
            }
            let color = if y == t || y == b || x == right - 1 {
                OAK_BARK.edge
            } else if from_end < 6 && y < t + 3 {
                // The broken wood showing pale at the splinters' tips.
                if y == t + 1 { WOOD.light } else { WOOD.base }
            } else {
                tone(OAK_BARK, level.clamp(0, 4))
            };
            scene.set(x, y, color);
        }
    }
    // A knot where a branch once grew.
    hline(scene, 165, 167, 3, OAK_BARK.edge);
    hline(scene, 164, 168, 5, OAK_BARK.shadow);
    put(scene, 164, 168, OAK_BARK.edge);
    put(scene, 168, 168, OAK_BARK.edge);
    put(scene, 165, 168, OAK_BARK.light);
    hline(scene, 165, 169, 3, OAK_BARK.edge);
    // Moss in lumpy cushions along its top, draping over its side here and there, with a
    // seedling growing out of it.
    for x in end_x + 3..right - 4 {
        let thick = 2 + (patches(x, 0, (5, 1), 2003) * 3.0) as i32;
        for dy in -1..thick {
            let y = top + dy;
            let color = if dy == -1 {
                if chance(x, y, 2004, 140) {
                    MOSS.light
                } else {
                    continue;
                }
            } else if dy == 0 {
                MOSS.light
            } else if dy == thick - 1 {
                MOSS.shadow
            } else if chance(x, y, 2005, 50) {
                MOSS.light
            } else {
                MOSS.base
            };
            put(scene, x, y, color);
        }
        put(scene, x, top + thick, MOSS.edge);
        if pick(x.div_euclid(4), 1, 2006, 5) == 0 && x % 4 == 1 {
            let long = 2 + pick(x, 2, 2006, 4);
            vline(scene, x, top + thick, long, MOSS.base);
            put(scene, x, top + thick + long, MOSS.shadow);
        }
    }
    vline(scene, 170, top - 5, 5, GRASS.base);
    put(scene, 169, top - 5, GRASS.light);
    put(scene, 171, top - 4, GRASS.light);
    put(scene, 168, top - 6, GRASS.shine);
    // Bracket fungi stepping along its side, each a shelf lit on top and shadowed beneath.
    for (cx, cy, half) in [(149, 171, 4), (155, 166, 3), (178, 170, 3)] {
        hline(scene, cx - half + 2, cy - 2, half * 2 - 3, CAP.edge);
        hline(scene, cx - half + 1, cy - 1, half * 2 - 1, CAP.light);
        put(scene, cx - half + 1, cy - 1, CAP.shine);
        hline(scene, cx - half, cy, half * 2 + 1, CAP.base);
        put(scene, cx - half, cy, CAP.light);
        hline(scene, cx - half, cy + 1, half * 2 + 1, CAP.edge);
        for x in (cx - half + 2..cx + half).step_by(2) {
            put(scene, x, cy, CAP.light);
        }
        put(scene, cx + half, cy - 1, CAP.edge);
        put(scene, cx - half, cy - 1, CAP.edge);
    }
    // The open end: a ring of pale wood round the dark inside.
    ellipse(scene, end_x, cy, 8, 11, OAK_BARK.edge);
    for y in cy - 10..=cy + 10 {
        for x in end_x - 7..=end_x + 7 {
            let (dx, dy) = ((x - end_x) as f32 / 7.0, (y - cy) as f32 / 10.0);
            let r = (dx * dx + dy * dy).sqrt();
            if r > 1.0 {
                continue;
            }
            let growth = ((r * 9.0) as i32) % 2 == 0;
            let light = dx + dy;
            let level = if light < -0.5 {
                3
            } else if light < 0.4 {
                2
            } else {
                1
            } - i32::from(!growth);
            put(scene, x, y, tone(WOOD, level));
        }
    }
    ellipse(scene, end_x + 1, cy - 1, 5, 8, WOOD.edge);
    ellipse(scene, end_x + 1, cy - 1, 4, 7, HOLLOW);
    for y in cy..=cy + 5 {
        for x in end_x + 2..=end_x + 5 {
            let (dx, dy) = ((x - end_x - 1) as f32 / 4.0, (y - cy + 1) as f32 / 7.0);
            let r = dx * dx + dy * dy;
            if (0.5..=1.0).contains(&r) {
                put(scene, x, y, HOLLOW_FLOOR);
            }
        }
    }
    for x in end_x - 6..=end_x - 1 {
        if chance(x, cy, 2004, 160) {
            put(scene, x, cy + 9 - (end_x - x) / 3, MOSS.base);
        }
    }
}

/// A low hazel on the near bank: several stems from one root, a rounded crown, and nuts in their
/// frilled husks.
fn hazel(scene: &mut Canvas) {
    let (cx, base) = (152, 136);
    ellipse(scene, cx + 4, base, 20, 3, SHADE);
    let stems = [(136, 116), (144, 106), (153, 102), (162, 107), (169, 116)];
    let stem = |scene: &mut Canvas, below: i32| {
        for &(tx, ty) in &stems {
            let from = (cx + (tx - cx) / 5, base);
            let to = (tx, ty);
            let mut previous = from;
            for step in 1..=16 {
                let t = step as f32 / 16.0;
                let point = (
                    (from.0 as f32 + (to.0 - from.0) as f32 * t) as i32,
                    (from.1 as f32 + (to.1 - from.1) as f32 * t) as i32,
                );
                if point.1 >= below {
                    line(scene, previous, point, HAZEL_BARK.base);
                    put(scene, point.0 + 1, point.1, HAZEL_BARK.shadow);
                    put(scene, point.0 - 1, point.1, HAZEL_BARK.edge);
                    if chance(point.0, point.1, 2101, 50) {
                        put(scene, point.0, point.1, HAZEL_BARK.shine);
                    }
                }
                previous = point;
            }
        }
    };
    stem(scene, 0);
    let crown = [
        ((142, 113), 9),
        ((161, 110), 9),
        ((152, 106), 8),
        ((168, 120), 7),
        ((136, 123), 7),
        ((148, 120), 9),
        ((160, 124), 8),
    ];
    for (index, (centre, radius)) in crown.into_iter().enumerate() {
        clump(
            scene,
            centre,
            radius,
            HAZEL_LEAF,
            0.0,
            2110 + index as u32 * 5,
        );
    }
    stem(scene, 129);
    // Nuts in pairs and threes, peeping from their frilled husks.
    for (x, y) in [(139, 126), (158, 128), (167, 122), (146, 112)] {
        for (dx, dy) in [(0, 0), (2, 1)] {
            put(scene, x + dx - 1, y + dy - 1, rgb(0xa8b866));
            put(scene, x + dx, y + dy - 1, rgb(0x8a9c50));
            put(scene, x + dx, y + dy, rgb(0x9a7444));
            put(scene, x + dx + 1, y + dy, rgb(0x6e5030));
        }
    }
}

/// A fairy ring of small mushrooms, open at the front where a companion steps in.
fn mushroom_ring(scene: &mut Canvas) {
    let (cx, cy) = (204, 187);
    // The grass grows greener and longer in a ring where the mushrooms are.
    for step in 0..48 {
        let angle = step as f32 / 48.0 * TAU;
        let (x, y) = (
            cx + (angle.cos() * 10.0).round() as i32,
            cy + (angle.sin() * 4.0).round() as i32,
        );
        put(scene, x, y, MOSS.base);
        put(scene, x + 1, y, MOSS.shadow);
        if chance(step, 0, 2201, 120) {
            put(scene, x, y - 1, GRASS.light);
        }
    }
    let mut caps: Vec<(i32, i32, i32)> = (0..13)
        .filter_map(|index| {
            let angle = index as f32 / 13.0 * TAU - 0.4;
            let (x, y) = (
                cx + (angle.cos() * 9.0).round() as i32,
                cy + (angle.sin() * 4.0).round() as i32,
            );
            // Open at the front, where a companion steps in.
            if angle.sin() > 0.8 {
                return None;
            }
            let tall = (4 + (angle.sin() * 1.6).round() as i32).max(3) + pick(index, 0, 2203, 2);
            Some((x, y, tall))
        })
        .collect();
    caps.sort_by_key(|&(_, y, _)| y);
    for (x, y, tall) in caps {
        ellipse(scene, x + 1, y, 3, 1, rgba(0x14241c, 90));
        // The stalk, lit on its left.
        let stalk = tall - 2;
        vline(scene, x, y - stalk + 1, stalk, STALK.light);
        vline(scene, x + 1, y - stalk + 1, stalk, STALK.shadow);
        // The cap: a little red dome flecked with white, gills beneath, lit from the upper left.
        let wide = if tall > 5 { 2 } else { 1 };
        let (left, right, under) = (x - wide, x + wide + 1, y - stalk);
        hline(scene, left + 1, under, right - left - 1, STALK.edge);
        put(scene, left, under, TOADSTOOL.edge);
        put(scene, right, under, TOADSTOOL.edge);
        for px in left..=right {
            let color = if px == left {
                TOADSTOOL.light
            } else if px == right {
                TOADSTOOL.shadow
            } else {
                TOADSTOOL.base
            };
            put(scene, px, under - 1, color);
        }
        if wide > 1 {
            hline(
                scene,
                left + 1,
                under - 2,
                right - left - 1,
                TOADSTOOL.light,
            );
            put(scene, right - 1, under - 2, TOADSTOOL.base);
            put(scene, left, under - 2, TOADSTOOL.edge);
            put(scene, right, under - 2, TOADSTOOL.edge);
            put(scene, left + 1, under - 2, TOADSTOOL.shine);
            put(scene, right - 1, under - 1, STALK.shine);
        }
        hline(
            scene,
            left + 1,
            under - 1 - wide,
            right - left - 1,
            TOADSTOOL.edge,
        );
        put(scene, x, under - wide, STALK.shine);
    }
}

/// A bramble patch of arching canes and dark leaves, a few blackberries ripe and a few not.
fn bramble(scene: &mut Canvas) {
    let base = 196;
    ellipse(scene, 264, base, 24, 3, SHADE);
    for (index, (centre, radius)) in [
        ((250, 191), 6),
        ((274, 191), 6),
        ((262, 186), 8),
        ((256, 192), 5),
        ((269, 193), 5),
    ]
    .into_iter()
    .enumerate()
    {
        clump(scene, centre, radius, BRAMBLE, 0.0, 2300 + index as u32 * 5);
    }
    // Canes arching up out of the patch and down again.
    let canes = [
        ((246, 195), (250, 172), (262, 182)),
        ((258, 195), (266, 170), (280, 184)),
        ((270, 195), (276, 176), (284, 190)),
        ((252, 195), (242, 178), (240, 190)),
    ];
    for (index, (from, peak, to)) in canes.into_iter().enumerate() {
        let mut previous = from;
        for step in 1..=24 {
            let t = step as f32 / 24.0;
            let along = |a: i32, b: i32, c: i32| {
                ((1.0 - t) * (1.0 - t) * a as f32
                    + 2.0 * (1.0 - t) * t * b as f32
                    + t * t * c as f32)
                    .round() as i32
            };
            let point = (along(from.0, peak.0, to.0), along(from.1, peak.1, to.1));
            line(scene, previous, point, CANE.base);
            put(scene, point.0, point.1 - 1, CANE.light);
            if chance(point.0, point.1, 2310 + index as u32, 40) {
                put(scene, point.0 + 1, point.1 - 1, CANE.shine);
            }
            if step % 6 == 3 {
                // Leaves in threes along the cane.
                for (dx, dy) in [(-2, -1), (2, -1), (0, -2)] {
                    let (lx, ly) = (point.0 + dx, point.1 + dy);
                    ellipse(scene, lx, ly, 1, 1, BRAMBLE.edge);
                    put(scene, lx, ly, BRAMBLE.light);
                }
            }
            previous = point;
        }
    }
    for (index, (x, y)) in [
        (252, 184),
        (259, 179),
        (266, 183),
        (273, 186),
        (248, 189),
        (262, 190),
        (278, 191),
    ]
    .into_iter()
    .enumerate()
    {
        let ripe = index % 3 != 2;
        let ramp = if ripe { BERRY } else { UNRIPE };
        put(scene, x, y, ramp.base);
        put(scene, x + 1, y, ramp.shadow);
        put(scene, x, y + 1, ramp.shadow);
        put(scene, x + 1, y + 1, ramp.edge);
        put(scene, x, y, ramp.light);
        put(scene, x, y - 1, ramp.shine);
    }
}

/// A clump of ferns at the front left, fronds arching out from one crown.
fn ferns(scene: &mut Canvas) {
    let (cx, base) = (24, 196);
    ellipse(scene, cx + 3, base, 22, 3, SHADE);
    // Back fronds first, darker and more upright; the near ones lighter, spreading wider.
    let fronds = [
        (-100.0, 34.0, 1),
        (-132.0, 30.0, 1),
        (-68.0, 30.0, 1),
        (-158.0, 26.0, 2),
        (-44.0, 22.0, 2),
        (-116.0, 32.0, 3),
        (-82.0, 30.0, 3),
        (-172.0, 18.0, 3),
    ];
    for (angle, length, level) in fronds {
        frond(scene, (cx, base), (angle, length), level);
    }
    // A fiddlehead still uncurling at the crown.
    for (dx, dy) in [(0, 0), (1, -1), (2, -1), (3, 0), (3, 1), (2, 2), (1, 1)] {
        put(scene, cx + 2 + dx, base - 6 + dy, FERN.light);
    }
}

/// One fern frond from the crown at `from`, arching out at `angle` degrees and drooping at its
/// tip, with leaflets either side that shorten towards the tip.
fn frond(scene: &mut Canvas, from: (i32, i32), (angle, length): (f32, f32), level: i32) {
    let radians = f32::to_radians(angle);
    let (dx, dy) = (radians.cos(), radians.sin());
    // The further a frond leans out, the more its tip droops.
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
                // Each leaflet sweeps a little towards the tip.
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

/// Reeds and bulrushes standing in the water at the right.
fn reeds(scene: &mut Canvas) {
    let mut blades: Vec<(i32, i32, i32, i32)> = (0..44)
        .filter_map(|index| {
            let x = 283 + pick(index, 0, 2401, 38);
            let foot = near_edge(x) - 1 - pick(index, 1, 2401, 13);
            if (297..=303).contains(&x) && foot < near_edge(x) - 3 {
                return None;
            }
            let tall = 16 + pick(index, 2, 2401, 18) - (near_edge(x) - foot) / 2;
            let lean = pick(index, 3, 2401, 9) - 4;
            Some((x, foot, tall, lean))
        })
        .collect();
    blades.sort_by_key(|&(_, foot, _, _)| foot);
    for (index, &(x, foot, tall, lean)) in blades.iter().enumerate() {
        // Its reflection, broken by ripples.
        for dy in 1..4 {
            if dy % 2 == 1 {
                put(scene, x, foot + dy, rgba(0x34442a, 90));
            }
        }
        hline(scene, x - 1, foot + 1, 3, rgba(0xa4c4b8, 60));
        for k in 0..tall {
            let bend = lean * k * k / (tall * tall).max(1);
            let px = x + bend;
            let y = foot - k;
            let level = if k > tall - 3 {
                3
            } else if index % 3 == 0 {
                1
            } else {
                2
            };
            put(scene, px, y, tone(REED, level));
            if k < tall / 3 {
                put(scene, px + 1, y, REED.shadow);
            }
            if k > tall / 2 && chance(px, y, 2402, 40) {
                put(scene, px, y, REED.light);
            }
        }
    }
    // A dragonfly holding still over the water, its wings catching the light.
    let (fx, fy) = (272, 106);
    hline(scene, fx, fy, 6, rgb(0x2a5a66));
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
    // Bulrushes, their brown heads up above the reeds.
    for (index, x) in [287, 293, 306, 312, 318].into_iter().enumerate() {
        let i = index as i32;
        let foot = near_edge(x) - 2 - pick(i, 0, 2403, 6);
        let tall = 26 + pick(i, 1, 2403, 10);
        let top = foot - tall;
        vline(scene, x, top, tall, REED.shadow);
        vline(scene, x, top - 4, 4, REED.base);
        for y in top + 2..top + 9 {
            put(
                scene,
                x - 1,
                y,
                if y == top + 2 {
                    BULRUSH.shine
                } else {
                    BULRUSH.light
                },
            );
            put(scene, x, y, BULRUSH.base);
            put(scene, x + 1, y, BULRUSH.shadow);
        }
        put(scene, x, top + 1, BULRUSH.edge);
        put(scene, x, top + 9, BULRUSH.edge);
    }
}

// ---------------------------------------------------------------------------------------------
// The places only some company finds
// ---------------------------------------------------------------------------------------------
//
// These are painted over the finished glade only when a party opens them (see `EXTRAS`), so
// each lays the sun back over itself wherever a beam crosses it. None of them reads what is
// already there, so they could as well be painted onto a clear canvas and laid over the glade.

/// Yellow lichen, in rosettes on bare stone.
const LICHEN: Ramp = Ramp::new(0x5a5428, 0x7a7234, 0x9a9042, 0xb8ac56, 0xd0c470);
/// Stone that has lain in the earth: darker, browner and damp.
const DAMP_ROCK: Ramp = Ramp::new(0x2a2a24, 0x3c3a30, 0x504c3e, 0x68624e, 0x847e66);
/// Grass and moss that grew on under a stone's edge: pale, yellowed and flattened.
const BLANCHED: Ramp = Ramp::new(0x56583a, 0x707246, 0x8a8a56, 0xa2a068, 0xb8b47e);
/// Earth a badger has dug out from deep down: paler than the floor it is thrown over.
const SPOIL: Ramp = Ramp::new(0x3a2a1c, 0x5a4430, 0x786046, 0x967c5a, 0xb09872);
/// Old bedding a badger has dragged out: dry grass and bracken.
const BEDDING: Ramp = Ramp::new(0x4a3e24, 0x6a5a32, 0x8a7642, 0xa69254, 0xbeaa6a);
const WOODLOUSE: Ramp = Ramp::new(0x2e2e34, 0x46464e, 0x5e5e66, 0x7a7a82, 0x9898a0);
/// Where the light comes from, for a surface shaded by which way it faces: up and to the left,
/// and towards the eye.
const LIGHT: [f32; 3] = [-0.505, -0.589, 0.631];

/// A badger's sett, dug in under the tip of one of the oak's roots: a wide, low mouth going back
/// into the dark, in a bank of earth the badgers have thrown up, the root lying across the top
/// of it with hair roots trailing, and in front a spoil heap of earth from deep down and old
/// bedding, a trough worn through it where they drag things out.
pub fn badger_sett(scene: &mut Canvas, sign: (f32, f32)) {
    let (mx, floor_y) = (sign.0 as i32 + 5, sign.1 as i32 + 3);
    // The bank, its crown still under the leaf litter, its face dug bare.
    let (bx, foot) = (mx + 4, floor_y + 1);
    ellipse(scene, bx + 4, foot, 13, 2, SHADE);
    for x in bx - 11..=bx + 11 {
        let u = (x - bx) as f32 / 11.0;
        let rise = 9.0 * (1.0 - u * u).max(0.0).powf(0.6);
        let top = foot - rise.round() as i32;
        let litter = top + 1 + pick(x, 0, 2901, 3);
        for y in top..foot {
            let v = (y - top) as f32 / rise.max(1.0);
            let lit = u * 0.9 + v * 1.1 - 0.6;
            let color = if y == top {
                LITTER.edge
            } else if y <= litter {
                tone(
                    LITTER,
                    2 + i32::from(chance(x, y, 2902, 100)) - i32::from(u > 0.5),
                )
            } else {
                let level = if lit < -0.2 {
                    3
                } else if lit < 0.6 {
                    2
                } else {
                    1
                } + match noise(x, y.div_euclid(2), 2903) % 7 {
                    0 => 1,
                    1 => -1,
                    _ => 0,
                };
                tone(EARTH, level.clamp(1, 4))
            };
            put(scene, x, y, color);
        }
    }
    // The mouth: wider than it is high, the way a badger is, its rim dug clean and the lip over
    // it catching the light.
    let mouth = |y: i32, grow: f32| -> (i32, i32) {
        let rise = (floor_y - y) as f32 / (7.0 + grow);
        let half = 7.0 + grow;
        let half = if rise < 0.4 {
            half
        } else {
            half * (1.0 - ((rise - 0.4) / 0.6).powi(2)).max(0.0).sqrt()
        };
        (
            (mx as f32 - half).round() as i32,
            (mx as f32 + half).round() as i32,
        )
    };
    let (from, to) = mouth(floor_y, 1.0);
    for x in from..=to {
        let Some(lip) = (floor_y - 8..=floor_y).find(|&y| {
            let (from, to) = mouth(y, 1.0);
            (from..=to).contains(&x)
        }) else {
            continue;
        };
        vline(scene, x, lip, floor_y + 1 - lip, EARTH.edge);
        put(scene, x, lip - 1, EARTH.light);
    }
    for y in floor_y - 7..=floor_y {
        let (from, to) = mouth(y, 0.0);
        for x in from..=to {
            let color = if y > floor_y - 2 && x > mx - 3 {
                HOLLOW_FLOOR
            } else if x >= to - 1 && y > floor_y - 6 {
                mix(HOLLOW, HOLLOW_FLOOR, 0.6)
            } else {
                HOLLOW
            };
            put(scene, x, y, color);
        }
    }
    // The root it is dug in under, lying across the top of the mouth, and hair roots hanging
    // from it into the dark.
    let (points, widths, salt) = SETT_ROOT;
    root(scene, points, widths, salt);
    for (x, y, long) in [(mx - 4, floor_y - 3, 2), (mx - 1, floor_y - 5, 3)] {
        vline(scene, x, y, long, OAK_BARK.base);
        put(scene, x, y + long, OAK_BARK.light);
    }
    // The spoil heap, with the trough worn down the middle of it from the mouth.
    let (hx, hy) = (mx + 4, floor_y + 7);
    ellipse(scene, hx + 3, hy, 15, 2, SHADE);
    for x in hx - 13..=hx + 13 {
        let u = (x - hx) as f32 / 13.0;
        let rise = 6.0 * (1.0 - u * u).max(0.0).powf(0.9) + patches(x, 0, (3, 1), 2904);
        let top = hy - rise.round() as i32;
        for y in top..hy {
            let v = (y - top) as f32 / rise.max(1.0);
            let lit = u * 0.8 + v * 1.2 - 0.5;
            let trough = x - (mx + (y - floor_y) * 5 / 7);
            let level = if y == top {
                1
            } else if y == top + 1 {
                4
            } else if lit < -0.2 {
                3
            } else if lit < 0.6 {
                2
            } else {
                1
            } + match trough {
                -1..=1 => -1,
                2 => 1,
                _ => 0,
            } + match noise(x, y, 2905) % 7 {
                0 => 1,
                1 => -1,
                _ => 0,
            };
            put(scene, x, y, tone(SPOIL, level.clamp(0, 4)));
        }
    }
    boulder(scene, (hx + 6, hy - 3), (2, 1), 2906);
    boulder(scene, (hx - 8, hy - 2), (1, 1), 2907);
    // Old bedding dragged out with it: wisps of dry grass caught on the lip and lying on the
    // heap.
    for (x, y, dx, dy) in [
        (mx + 6, floor_y - 1, 3, -1),
        (mx + 8, floor_y + 1, 2, 1),
        (hx + 1, hy - 4, -3, 1),
        (hx + 8, hy - 2, 3, 0),
    ] {
        line(scene, (x, y), (x + dx, y + dy), BEDDING.base);
        put(scene, x, y, BEDDING.light);
        put(scene, x + dx, y + dy + 1, rgba(0x14241c, 60));
    }
}

/// A crevice where a root parts from the oak's foot, just wide enough for small paws: a dark
/// slot narrowing up into the bark, its lips rolled in towards it, the far side catching a
/// little light, litter spilling in at its foot, a cobweb strung across it and a pair of tiny
/// toadstools growing beside it.
pub fn root_crevice(scene: &mut Canvas, sign: (f32, f32)) {
    let (top, bottom) = (sign.1 as i32 - 10, sign.1 as i32 + 5);
    let span = (bottom - top) as f32;
    // The first and last columns of the slot in a row, the first being its dark edge, and how
    // far down it the row is: a pixel wide at the top, widening to its foot, and leaning a
    // little with the flare of the trunk.
    let row = |y: i32| -> (i32, i32, f32) {
        let t = (y - top) as f32 / span;
        let width = 1.0 + 6.2 * t.powf(1.3);
        let centre = sign.0 + 0.5 + (0.5 - t) * 1.2;
        let from = (centre - width / 2.0).round() as i32;
        (from, from + width.round() as i32 - 1, t)
    };
    for y in top..=bottom {
        let (from, to, t) = row(y);
        // The bark rolled in round it, lit on its left lip and shaded on its right.
        if chance(from, y, 2950, 140) {
            put(scene, from - 2, y, OAK_BARK.light);
        }
        put(scene, from - 1, y, OAK_BARK.light);
        put(scene, from, y, OAK_BARK.edge);
        put(scene, to + 1, y, OAK_BARK.edge);
        put(scene, to + 2, y, OAK_BARK.shadow);
        for x in from + 1..=to {
            let color = if x == to && t > 0.4 {
                HOLLOW_FLOOR
            } else if x == to {
                mix(HOLLOW, HOLLOW_FLOOR, 0.4)
            } else if y == bottom {
                HOLLOW_FLOOR
            } else {
                HOLLOW
            };
            put(scene, x, y, color);
        }
    }
    // The split running on up the bark above it.
    let (tip, _, _) = row(top);
    for (dx, dy) in [(0, -1), (0, -2), (1, -3)] {
        put(scene, tip + dx, top + dy, OAK_BARK.edge);
        put(scene, tip + dx - 1, top + dy, OAK_BARK.light);
    }
    // Leaf litter spilling in over its foot, and moss either side of it.
    let (from, to, _) = row(bottom);
    for x in from..=to + 1 {
        if chance(x, bottom, 2951, 150) {
            put(scene, x, bottom + 1, LITTER.light);
            put(scene, x, bottom, LITTER.shadow);
        } else {
            put(scene, x, bottom + 1, EARTH.shadow);
        }
    }
    for (x, y) in [
        (from - 2, bottom),
        (from - 1, bottom + 1),
        (to + 2, bottom + 1),
    ] {
        put(scene, x, y, MOSS.base);
        put(scene, x, y - 1, MOSS.light);
        put(scene, x + 1, y + 1, MOSS.shadow);
    }
    // A cobweb strung across its upper part.
    let (from, to, _) = row(top + 5);
    line(
        scene,
        (from - 1, top + 4),
        (to + 1, top + 6),
        rgba(0xdce4d8, 80),
    );
    // Two tiny toadstools growing out of the bark beside its foot.
    let (_, to, _) = row(bottom - 1);
    for (x, foot, tall) in [(to + 4, bottom - 1, 2), (to + 6, bottom + 1, 1)] {
        vline(scene, x, foot - tall + 1, tall, STALK.light);
        let under = foot - tall;
        put(scene, x - 1, under, CAP.edge);
        put(scene, x, under, STALK.edge);
        put(scene, x + 1, under, CAP.edge);
        put(scene, x - 1, under - 1, CAP.light);
        put(scene, x, under - 1, CAP.base);
        put(scene, x + 1, under - 1, CAP.shadow);
        put(scene, x, under - 2, CAP.shine);
    }
}

/// The mossy boulder a close pair can heave over: sunk a little into the floor in the sunbeam
/// that comes down past the ferns, lichen on its flanks and moss over its crown. Once searched
/// it lies rolled over towards the ferns, its damp underside to the front, and where it lay is
/// a bare dip of earth with blanched grass round it and woodlice caught out in the light.
pub fn mossy_boulder(scene: &mut Canvas, sign: (f32, f32), rolled: bool) {
    let base = sign.1 as i32 + 6;
    let lying = Stone {
        centre: (sign.0 + 0.5, sign.1 - 2.0),
        radii: (13.0, 10.0),
        turn: 0.0,
        sunk: base,
        buried: 0.78,
        salt: 2801,
    };
    if !rolled {
        stone(scene, &lying);
        return;
    }
    bare_patch(scene, (sign.0 as i32, base - 2), (12, 4));
    // Moss torn up as it went over.
    for (dx, dy) in [(-15, -3), (-12, 0), (-17, 1)] {
        let (x, y) = (sign.0 as i32 + dx, base + dy);
        hline(scene, x, y, 2, MOSS.base);
        put(scene, x, y - 1, MOSS.light);
        put(scene, x + 1, y + 1, MOSS.edge);
    }
    // Over on its side, settled a little into the moss, showing what lay deepest.
    let (centre, radii) = ((sign.0 - 22.0, sign.1 - 10.0), (12.0, 9.2));
    stone(
        scene,
        &Stone {
            centre,
            radii,
            turn: -0.75,
            sunk: (centre.1 + radii.1 * 0.95) as i32,
            buried: 0.45,
            ..lying
        },
    );
}

/// A stone big enough that it takes two to shift: lumpy, chipped into faces that each catch the
/// light their own way, moss over whichever side was uppermost and earth caked on whichever lay
/// in the ground.
struct Stone {
    /// Its middle, in the glade.
    centre: (f32, f32),
    /// Half its width and half its height as it lay before anyone moved it.
    radii: (f32, f32),
    /// How far it has been turned over from how it lay, clockwise, in radians.
    turn: f32,
    /// The lowest row of it that shows: below that it is sunk in the floor.
    sunk: i32,
    /// How far down it, as it lay, the earth came up: damp below that.
    buried: f32,
    salt: u32,
}

/// What a stone's surface is at a point.
#[derive(Clone, Copy, PartialEq)]
enum Face {
    Rock,
    Moss,
    /// What lay in the earth.
    Underside,
}

/// How many chipped faces a stone has.
const FACETS: i32 = 6;
/// How square a stone is: 2 would be an ellipse.
const SQUARENESS: f32 = 2.15;

impl Stone {
    /// Where `(x, y)` falls on the stone in its own frame as it lay, scaled so its outline is
    /// about 1 away from its middle, and how far out that is; `None` if it is off the stone.
    fn at(&self, x: i32, y: i32) -> Option<(f32, f32, f32)> {
        if y > self.sunk {
            return None;
        }
        let (dx, dy) = (
            x as f32 + 0.5 - self.centre.0,
            y as f32 + 0.5 - self.centre.1,
        );
        let (sin, cos) = self.turn.sin_cos();
        let u = (dx * cos + dy * sin) / self.radii.0;
        let v = (dy * cos - dx * sin) / self.radii.1;
        let angle = v.atan2(u);
        let lumps = 1.0 + 0.07 * (3.0 * angle + 0.6).sin() + 0.04 * (5.0 * angle + 2.1).sin();
        let out =
            (u.abs().powf(SQUARENESS) + v.abs().powf(SQUARENESS)).powf(1.0 / SQUARENESS) / lumps;
        (out <= 1.0).then_some((u, v, out))
    }

    /// Whether `(u, v)` is moss, bare rock, or the side that lay in the earth. Moss lies over
    /// the crown in cushions and further down the left, dripping down here and there.
    fn face(&self, u: f32, v: f32) -> Face {
        let column = (u * self.radii.0).round() as i32;
        let ragged = (noise(column, 0, self.salt) % 100) as f32 / 100.0 * 0.12;
        let drip = if noise(column, 1, self.salt).is_multiple_of(4) {
            0.16
        } else {
            0.0
        };
        let cushions = 0.12 * (u * 6.5 + 1.3).sin().abs();
        if v < -0.4 - 0.32 * u + cushions + ragged + drip {
            Face::Moss
        } else if v > self.buried - ragged {
            Face::Underside
        } else {
            Face::Rock
        }
    }

    /// Which chipped face `(u, v)` is on: whichever of the stone's scattered seeds is nearest.
    fn facet(&self, u: f32, v: f32) -> i32 {
        (0..FACETS)
            .min_by(|&a, &b| {
                let (au, av) = self.seed(a);
                let (bu, bv) = self.seed(b);
                let distance = |su: f32, sv: f32| (u - su).powi(2) + (v - sv).powi(2) * 1.4;
                distance(au, av).total_cmp(&distance(bu, bv))
            })
            .unwrap_or(0)
    }

    fn seed(&self, index: i32) -> (f32, f32) {
        (
            pick(index, 0, self.salt + 1, 161) as f32 / 100.0 - 0.8,
            pick(index, 1, self.salt + 1, 161) as f32 / 100.0 - 0.8,
        )
    }

    /// Which way the stone's surface faces at `(u, v)`, turned into the glade: rounded like a
    /// cushion, flat across its middle and turning away at its edges.
    fn normal(&self, u: f32, v: f32, out: f32) -> [f32; 3] {
        let along = |w: f32, radius: f32| w.signum() * w.abs().powf(SQUARENESS - 1.0) / radius;
        let (gu, gv) = (along(u, self.radii.0), along(v, self.radii.1));
        let length = (gu * gu + gv * gv).sqrt().max(1e-6);
        let (sin, cos) = self.turn.sin_cos();
        let (gx, gy) = (
            (gu * cos - gv * sin) / length,
            (gu * sin + gv * cos) / length,
        );
        let tilt = out.powf(1.8).min(0.97);
        [gx * tilt, gy * tilt, (1.0 - tilt * tilt).sqrt()]
    }

    /// How brightly the surface at `(u, v)` is lit, about -1 to 1. Each chipped face is nearly
    /// flat, leaning its own way, so it takes the light all of a piece.
    fn lit(&self, u: f32, v: f32, out: f32) -> f32 {
        let facet = self.facet(u, v);
        let (su, sv) = self.seed(facet);
        let seed_out = (su.abs().powf(SQUARENESS) + sv.abs().powf(SQUARENESS))
            .powf(1.0 / SQUARENESS)
            .min(1.0);
        let flat = self.normal(su, sv, seed_out);
        let round = self.normal(u, v, out);
        let lean = |axis: i32| (pick(facet, axis, self.salt + 2, 25) - 12) as f32 / 100.0;
        let normal = [
            flat[0] * 0.35 + round[0] * 0.65 + lean(0),
            flat[1] * 0.35 + round[1] * 0.65 + lean(1),
            flat[2] * 0.35 + round[2] * 0.65,
        ];
        let length = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
        normal.iter().zip(LIGHT).map(|(n, l)| n / length * l).sum()
    }
}

/// Paints a stone where it lies, with the shade it casts and the sun that falls on it.
fn stone(scene: &mut Canvas, stone: &Stone) {
    let reach = stone.radii.0.max(stone.radii.1) * 1.12;
    let (left, right) = (
        (stone.centre.0 - reach).floor() as i32,
        (stone.centre.0 + reach).ceil() as i32,
    );
    let (top, bottom) = (
        (stone.centre.1 - reach).floor() as i32,
        ((stone.centre.1 + reach).ceil() as i32).min(stone.sunk),
    );
    let on = |x: i32, y: i32| stone.at(x, y).is_some();
    let face = |x: i32, y: i32| stone.at(x, y).map(|(u, v, _)| stone.face(u, v));
    let foot = (left..=right)
        .filter_map(|x| (top..=bottom).rev().find(|&y| on(x, y)))
        .max()
        .unwrap_or(bottom);
    let middle = stone.centre.0.round() as i32;
    let half = stone.radii.0.round() as i32;
    ellipse(scene, middle + 4, foot + 1, half + 2, 2, SHADE);
    ellipse(scene, middle + 1, foot, half, 1, rgba(0x14241c, 50));
    let mut painted = Vec::new();
    for y in top..=bottom {
        for x in left..=right {
            let Some((u, v, out)) = stone.at(x, y) else {
                continue;
            };
            let here = stone.face(u, v);
            let outline = [(0, -1), (0, 1), (-1, 0), (1, 0)]
                .iter()
                .any(|&(dx, dy)| !on(x + dx, y + dy));
            let shade = stone.lit(u, v, out)
                + (noise(x, y, stone.salt + 3) % 100) as f32 / 100.0 * 0.12
                - 0.06;
            let mut level = if shade > 0.84 {
                4
            } else if shade > 0.66 {
                3
            } else if shade > 0.3 {
                2
            } else {
                1
            };
            // A ridge where one chipped face meets the next catches the light.
            let ridge = stone
                .at(x + 1, y + 1)
                .is_some_and(|(nu, nv, _)| stone.facet(nu, nv) != stone.facet(u, v));
            if ridge && here == Face::Rock && (2..4).contains(&level) {
                level += 1;
            }
            let above = face(x, y - 1);
            let color = match here {
                Face::Moss if outline || face(x, y + 1) != Some(Face::Moss) => MOSS.edge,
                Face::Moss => {
                    let cushion = patches(
                        (u * stone.radii.0) as i32 + 40,
                        (v * stone.radii.1) as i32 + 40,
                        (3, 2),
                        stone.salt + 4,
                    );
                    let bump = if cushion > 0.66 {
                        1
                    } else if cushion < 0.3 {
                        -1
                    } else {
                        0
                    };
                    tone(MOSS, (level + bump).clamp(1, 4))
                }
                Face::Rock if outline => ROCK.edge,
                // Shaded under the moss hanging over it.
                Face::Rock if above == Some(Face::Moss) => ROCK.shadow,
                Face::Rock => {
                    let color = tone(ROCK, level);
                    if chance(x, y, stone.salt + 5, 26) {
                        mix(color, ROCK.edge, 0.35)
                    } else {
                        color
                    }
                }
                Face::Underside if outline => DAMP_ROCK.edge,
                Face::Underside if above == Some(Face::Rock) => DAMP_ROCK.shadow,
                Face::Underside => {
                    let clod = patches(
                        (u * stone.radii.0) as i32 + 40,
                        (v * stone.radii.1) as i32 + 40,
                        (3, 2),
                        stone.salt + 6,
                    );
                    if clod > 0.62 && stone.turn != 0.0 {
                        tone(EARTH, level)
                    } else if level >= 3 && chance(x, y, stone.salt + 7, 50) {
                        // Still wet from the ground.
                        DAMP_ROCK.shine
                    } else {
                        tone(DAMP_ROCK, level)
                    }
                }
            };
            put(scene, x, y, color);
            painted.push((x, y));
        }
    }
    // Moss standing up off the crown like fur, and a few of its stalks with their capsules.
    for x in left..=right {
        let Some(crown) = (top..=bottom).find(|&y| on(x, y)) else {
            continue;
        };
        if face(x, crown) != Some(Face::Moss) {
            continue;
        }
        if chance(x, crown, stone.salt + 8, 130) {
            put(
                scene,
                x,
                crown - 1,
                if x < middle { MOSS.light } else { MOSS.base },
            );
            painted.push((x, crown - 1));
        }
        if stone.turn == 0.0 && chance(x, crown, stone.salt + 9, 40) {
            vline(scene, x, crown - 3, 2, BULRUSH.light);
            put(scene, x, crown - 4, CAP.light);
            painted.extend([(x, crown - 3), (x, crown - 2), (x, crown - 4)]);
        }
    }
    // Rosettes of lichen on the bare stone.
    for index in 0..5 {
        let x = left + 3 + pick(index, 0, stone.salt + 10, right - left - 5);
        let y = top + 3 + pick(index, 1, stone.salt + 10, bottom - top - 5);
        let bare = |x: i32, y: i32| face(x, y) == Some(Face::Rock);
        if !(bare(x, y) && bare(x - 1, y) && bare(x + 1, y) && bare(x, y + 1)) {
            continue;
        }
        put(scene, x, y, LICHEN.base);
        put(scene, x - 1, y, LICHEN.light);
        match pick(index, 2, stone.salt + 10, 3) {
            0 => put(scene, x, y + 1, LICHEN.shadow),
            1 => put(scene, x + 1, y, LICHEN.shadow),
            _ => put(scene, x - 1, y + 1, LICHEN.shadow),
        }
    }
    // Grass growing up round its foot where it is sunk.
    if stone.turn == 0.0 {
        for x in [left + 3, left + 5, right - 4, right - 2] {
            let tall = 2 + pick(x, 0, stone.salt + 12, 3);
            let lean = if x < middle { -1 } else { 1 };
            line(
                scene,
                (x, bottom + 1),
                (x + lean, bottom + 1 - tall),
                GRASS.base,
            );
            put(scene, x + lean, bottom + 1 - tall, GRASS.light);
            painted.push((x + lean, bottom + 1 - tall));
        }
    }
    for (x, y) in painted {
        sunlit(scene, x, y, false);
    }
}

/// The bare patch where a stone lay: a shallow dip of damp earth, the grass that grew in under
/// its edge blanched and flattened round it, worm holes, a pale root, and woodlice caught out.
fn bare_patch(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32)) {
    let mut painted = Vec::new();
    for y in cy - ry - 1..=cy + ry + 1 {
        for x in cx - rx - 1..=cx + rx + 1 {
            let (dx, dy) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            let out = (dx * dx + dy * dy).sqrt() + (noise(x, y, 2810) % 100) as f32 / 100.0 * 0.16;
            if out > 1.08 {
                continue;
            }
            let color = if out > 0.84 {
                // Ragged: the floor's own moss shows through it here and there.
                if !chance(x, y, 2811, 200) {
                    continue;
                }
                if chance(x, y, 2812, 50) {
                    MOSS.light
                } else {
                    tone(BLANCHED, 2 + i32::from(dy < 0.0))
                }
            } else if out > 0.7 && dy < 0.0 {
                // The rim of the dip, nearest the eye at the back.
                EARTH.edge
            } else {
                // The dip's near wall, up and to the left, in its own shadow; its far wall lit.
                let wall = dx + dy * 0.8;
                let level = if out > 0.5 && wall > 0.4 {
                    3
                } else if out < 0.5 && patches(x, y, (4, 2), 2813) > 0.55 {
                    2
                } else {
                    1
                };
                tone(EARTH, level)
            };
            put(scene, x, y, color);
            painted.push((x, y));
        }
    }
    // Wet glints, worm holes, a pale root and two woodlice.
    for index in 0..4 {
        let x = cx - rx / 2 + pick(index, 0, 2814, rx);
        let y = cy - ry / 2 + pick(index, 1, 2814, ry.max(1));
        put(scene, x, y, EARTH.shine);
    }
    for (dx, dy) in [(-6, 0), (3, -1), (7, 1)] {
        put(scene, cx + dx, cy + dy, HOLLOW);
        put(scene, cx + dx + 1, cy + dy, EARTH.light);
    }
    line(
        scene,
        (cx - 9, cy + 1),
        (cx - 4, cy + 2),
        mix(STALK.shadow, EARTH.base, 0.3),
    );
    put(scene, cx - 3, cy + 1, STALK.shadow);
    for (x, y) in [(cx + 1, cy + 1), (cx - 3, cy - 1)] {
        hline(scene, x, y, 3, WOODLOUSE.base);
        put(scene, x, y, WOODLOUSE.light);
        put(scene, x + 1, y - 1, WOODLOUSE.shine);
        hline(scene, x, y + 1, 3, WOODLOUSE.edge);
    }
    for (x, y) in painted {
        sunlit(scene, x, y, true);
    }
}

// ---------------------------------------------------------------------------------------------
// The light
// ---------------------------------------------------------------------------------------------

/// Shafts of sun slanting down from the gaps in the canopy, motes drifting in them, and the
/// warm pools where they land.
fn sunbeams(scene: &mut Canvas) {
    for (index, &(origin, half, top, end)) in BEAMS.iter().enumerate() {
        let salt = 2500 + index as u32 * 10;
        for y in top..end {
            let centre = origin + SLANT * y as f32;
            for x in (centre - half - 2.0) as i32..=(centre + half + 2.0) as i32 {
                let alpha = shaft(index, x, y);
                if alpha > 0 {
                    put(scene, x, y, rgba(SUN, alpha));
                }
            }
        }
        // Where it lands.
        let (lx, ly) = landing(index);
        for (rx, ry, alpha) in pool(index) {
            ellipse(scene, lx, ly, rx, ry, rgba(SUN, alpha));
        }
        // Motes caught in the light.
        let half = half as i32;
        for mote in 0..9 {
            let y = top + 20 + pick(mote, 0, salt + 1, end - top - 40);
            let x = (origin + SLANT * y as f32) as i32 + pick(mote, 1, salt + 1, half * 2) - half;
            put(scene, x, y, rgba(0xfff0c0, 110));
        }
    }
}

/// How strongly the `index`th beam's shaft falls on `(x, y)`: the alpha of the sun laid over it.
fn shaft(index: usize, x: i32, y: i32) -> u8 {
    let (origin, half, top, end) = BEAMS[index];
    let centre = origin + SLANT * y as f32;
    if y < top || y >= end || x < (centre - half - 2.0) as i32 || x > (centre + half + 2.0) as i32 {
        return 0;
    }
    let strength = ((y - top) as f32 / 18.0).min(1.0) * ((end - y) as f32 / 40.0).min(1.0);
    let salt = 2500 + index as u32 * 10;
    let off = (x as f32 - centre).abs() / half + (noise(x, y, salt) % 100) as f32 / 100.0 * 0.14;
    let alpha = if off < 0.35 {
        52.0
    } else if off < 0.7 {
        34.0
    } else if off < 1.0 {
        16.0
    } else {
        0.0
    } * strength;
    // Anything fainter than one step is no light at all.
    alpha as u8
}

/// Where the `index`th beam lands on the floor.
fn landing(index: usize) -> (i32, i32) {
    let (origin, _, _, end) = BEAMS[index];
    ((origin + SLANT * end as f32) as i32, end - 2)
}

/// The warm pool where the `index`th beam lands: rings of sun, widest and faintest first.
fn pool(index: usize) -> [(i32, i32, u8); 3] {
    let half = BEAMS[index].1 as i32;
    [(half * 2 + 4, 5, 22), (half * 2, 3, 26), (half, 1, 30)]
}

/// The sun that falls on `(x, y)`, laid over whatever has just been painted there as the
/// sunbeams were laid over the glade, so something added to it sits in the same light. Only the
/// `floor` takes the pools where the beams land; anything standing up off it only the shafts.
fn sunlit(scene: &mut Canvas, x: i32, y: i32, floor: bool) {
    for index in 0..BEAMS.len() {
        let alpha = shaft(index, x, y);
        if alpha > 0 {
            put(scene, x, y, rgba(SUN, alpha));
        }
        if !floor {
            continue;
        }
        let (lx, ly) = landing(index);
        let (dx, dy) = (i64::from(x - lx), i64::from(y - ly));
        for (rx, ry, alpha) in pool(index) {
            // The same test `ellipse` makes.
            let (rx2, ry2) = (i64::from(rx * rx), i64::from(ry * ry));
            if dx * dx * ry2 + dy * dy * rx2 <= rx2 * ry2 {
                put(scene, x, y, rgba(SUN, alpha));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// In front of everyone
// ---------------------------------------------------------------------------------------------

/// The edge of a trunk nearer than anyone, down the far left: in shade, its bark only just made
/// out.
fn near_trunk(scene: &mut Canvas) {
    for y in 0..HEIGHT {
        let fy = y as f32;
        let flare = ((fy - 190.0).max(0.0) / 26.0).powi(2) * 7.0;
        let right = (6.0 + (fy * 0.05).sin() * 1.2 + flare).round() as i32;
        for x in 0..=right {
            let color = if x == right {
                mix(BARK.edge, rgb(0x0e0c0a), 0.3)
            } else if x == right - 1 {
                BARK.shadow
            } else if (x + pick(y.div_euclid(4), x, 2601, 3)) % 4 == 0 {
                mix(BARK.edge, BARK.shadow, 0.3)
            } else if y > 186 && patches(x, y, (3, 5), 2602) > 0.5 {
                MOSS.shadow
            } else {
                mix(BARK.shadow, BARK.edge, 0.45)
            };
            scene.set(x, y, color);
        }
    }
}

/// A spray of the nearest leaves of all, reaching into the picture from a corner: twigs fanning
/// out from `root` off the edge, each ending in a few pointed leaves hanging from it, the sun
/// glowing through them against the shade of the canopy.
fn leaf_spray(scene: &mut Canvas, root: (i32, i32), tips: &[(i32, i32)], salt: u32) {
    let near = Ramp::new(0x22401e, 0x36602a, 0x4c7c36, 0x689846, 0x8cb45c);
    for &tip in tips {
        line(scene, root, tip, BARK.edge);
        line(scene, (root.0, root.1 + 1), (tip.0, tip.1 + 1), BARK.shadow);
    }
    for (index, &tip) in tips.iter().enumerate() {
        let i = index as i32;
        let (dx, dy) = (tip.0 - root.0, tip.1 - root.1);
        // Leaves along the last part of each twig, hanging either side and one off its end.
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
    // The half of the leaf on the side its normal points is lit if that side faces up and left.
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

/// Leaf litter and grass along the very bottom edge, nearer than anyone stands.
fn front_litter(scene: &mut Canvas) {
    for x in 0..WIDTH {
        if chance(x, 0, 2701, 90) {
            let tall = 2 + pick(x, 1, 2701, 4);
            let lean = pick(x, 2, 2701, 3) - 1;
            line(
                scene,
                (x, HEIGHT - 1),
                (x + lean, HEIGHT - tall),
                GRASS.shadow,
            );
            put(scene, x + lean, HEIGHT - tall, GRASS.base);
        } else if chance(x, 3, 2702, 30) {
            hline(scene, x, HEIGHT - 2, 3, LITTER.shadow);
            hline(scene, x, HEIGHT - 1, 3, LITTER.edge);
            put(scene, x, HEIGHT - 2, LITTER.light);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::woods::{EXTRAS, Extra, Opener, SPOTS};

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(backdrop().pixels().iter().all(|pixel| pixel.a == 255));
    }

    #[test]
    fn the_foreground_leaves_every_spot_in_sight() {
        let front = foreground();
        for spot in SPOTS {
            for (x, y) in [spot.sign, spot.stand] {
                for dy in -24..=2 {
                    for dx in -10..=10 {
                        assert_eq!(
                            front.get(x as i32 + dx, y as i32 + dy).a,
                            0,
                            "the foreground covers {}",
                            spot.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_hollows_are_dark_where_their_finds_show() {
        let scene = backdrop();
        for name in ["the hollow log", "the old oak's hollow", "the fox hole"] {
            let spot = SPOTS.iter().find(|spot| spot.name == name).unwrap();
            let pixel = scene.get(spot.sign.0 as i32, spot.sign.1 as i32);
            let brightness = u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b);
            assert!(brightness < 150, "{name} is not dark inside");
        }
    }

    #[test]
    fn the_glade_is_the_same_every_time() {
        assert_eq!(backdrop(), backdrop());
        assert_eq!(foreground(), foreground());
    }

    /// The glade with an extra spot painted over it, as an outing that opened it draws it.
    fn opened(glade: &Canvas, extra: &Extra, searched: bool) -> Canvas {
        let mut scene = glade.clone();
        let sign = extra.spot.sign;
        match extra.opener {
            Opener::Explorer => badger_sett(&mut scene, sign),
            Opener::LittleOne => root_crevice(&mut scene, sign),
            Opener::ClosePair => mossy_boulder(&mut scene, sign, searched),
        }
        scene
    }

    #[test]
    fn the_extra_spots_leave_room_to_stand() {
        let glade = backdrop();
        for extra in &EXTRAS {
            for searched in [false, true] {
                let scene = opened(&glade, extra, searched);
                let (x, y) = (extra.spot.stand.0 as i32, extra.spot.stand.1 as i32);
                for dy in -2..=1 {
                    for dx in -4..=4 {
                        assert_eq!(
                            scene.get(x + dx, y + dy),
                            glade.get(x + dx, y + dy),
                            "{} is painted where its companion stands",
                            extra.spot.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_crevice_is_dark_where_its_glint_shows() {
        let glade = backdrop();
        let extra = EXTRAS
            .iter()
            .find(|extra| extra.opener == Opener::LittleOne)
            .unwrap();
        let scene = opened(&glade, extra, false);
        let (x, y) = (extra.spot.sign.0 as i32, extra.spot.sign.1 as i32);
        // The glint is a cross two pixels each way.
        for (dx, dy) in [(0, 0), (-2, 0), (2, 0), (0, -2), (0, 2)] {
            let pixel = scene.get(x + dx, y + dy);
            let brightness = u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b);
            assert!(brightness < 150, "the crevice is not dark at {dx},{dy}");
        }
    }

    #[test]
    fn the_boulder_is_rolled_aside_once_searched() {
        let glade = backdrop();
        let extra = EXTRAS
            .iter()
            .find(|extra| extra.opener == Opener::ClosePair)
            .unwrap();
        let (lying, rolled) = (opened(&glade, extra, false), opened(&glade, extra, true));
        // Its crown was over the sign; once it has gone over, the glade shows there again.
        let (x, y) = (extra.spot.sign.0 as i32, extra.spot.sign.1 as i32 - 8);
        assert_ne!(lying.get(x, y), glade.get(x, y));
        assert_eq!(rolled.get(x, y), glade.get(x, y));
        // And where it lay is bare earth: browner than the stone was.
        let (x, y) = (extra.spot.sign.0 as i32, extra.spot.sign.1 as i32 + 4);
        let (stone, earth) = (lying.get(x, y), rolled.get(x, y));
        assert!(i32::from(earth.r) - i32::from(earth.b) > i32::from(stone.r) - i32::from(stone.b));
    }

    #[test]
    fn the_extra_spots_are_the_same_every_time() {
        let glade = backdrop();
        for extra in &EXTRAS {
            for searched in [false, true] {
                assert_eq!(
                    opened(&glade, extra, searched),
                    opened(&glade, extra, searched)
                );
            }
        }
    }
}
