//! The Hilltop, painted from on top of the Hill: a broad dome of turf under a big sky, the old
//! tree that the station sees on the skyline standing at its back on the left, and beyond the brow
//! the land falling away to the valley far below, small and hazy with distance. Down there are the
//! places the colony knows: the station's red roof by the railway, the oak on the village green,
//! the big top's stripes, the dark of the Woods, the lane joining them, and hills along the
//! horizon.
//!
//! The summit begins nearly bare, as the colony first finds it: grass, the path coming up, and
//! flattened places in the turf where whatever the person brings back from the Woods will stand.

use super::{SPOTS, TREE};
use crate::materials::*;
use crate::paint::{Ramp, blit, chance, ellipse, line, mix, noise, put, rgb, rgba, vline};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

/// Where the plain below meets the far hills: low in the picture, as it is from somewhere high.
const HORIZON: i32 = 66;
/// The point on the horizon the plain's furrows run towards: straight ahead of the eye.
const VANISH: i32 = 210;
/// Where the station, the village green and the fairground stand down on the plain, each between
/// two of the back row's places, so a piece standing there hides as little of them as it can.
const STATION: (i32, i32) = (286, 95);
const GREEN: (i32, i32) = (232, 86);
const FAIR: (i32, i32) = (182, 79);
const WOODS: (i32, i32) = (132, 75);

const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const EARTH: Ramp = Ramp::new(0x8e7553, 0xb39a6f, 0xcdb688, 0xdcc89a, 0xece0bb);
const CLOUDS: Ramp = Ramp::new(0xc9d7e2, 0xdfe7ee, 0xf4f4f1, 0xfdfbf5, 0xffffff);
const HEDGE: Ramp = Ramp::new(0x2f5733, 0x3f6e40, 0x4f8250, 0x67995e, 0x85b277);
const HAZE: Rgba = rgb(0xcfe1dc);
const SHADE: Rgba = rgba(0x2c4a2e, 70);

/// Everything behind the colony and the pieces placed on the summit.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    plain(&mut scene);
    far_hills(&mut scene);
    lane(&mut scene);
    railway(&mut scene);
    woods(&mut scene);
    fairground(&mut scene);
    village_green(&mut scene);
    station(&mut scene);
    flanks(&mut scene);
    summit(&mut scene);
    path(&mut scene);
    places(&mut scene);
    grass(&mut scene);
    old_tree(&mut scene);
    // The bottom edge falls into a little shade, which holds the eye in the scene.
    for (index, y) in (HEIGHT - 3..HEIGHT).enumerate() {
        for x in 0..WIDTH {
            put(&mut scene, x, y, rgba(0x2c4a2e, 26 + index as u8 * 22));
        }
    }
    scene
}

/// A few tufts of grass along the very bottom edge, in front of everyone, too short to hide
/// where the front row's pieces meet the ground.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for x in 0..WIDTH {
        if !chance(x, 0, 901, 34) {
            continue;
        }
        for (dx, tall) in [(-1, 2), (0, 4), (1, 3)] {
            let tall = tall + (noise(x + dx, 1, 902) % 2) as i32;
            vline(&mut scene, x + dx, HEIGHT - tall, tall, TURF.edge);
            put(
                &mut scene,
                x + dx,
                HEIGHT - tall,
                if dx < 0 { TURF.base } else { TURF.shadow },
            );
        }
    }
    scene
}

// ---------------------------------------------------------------------------------------------
// The sky
// ---------------------------------------------------------------------------------------------

fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 8;
    let bottom = HORIZON + 6;
    // In bands, as pixel skies are, each dithered a row into the next.
    let band = |y: i32| (y * BANDS / bottom).min(BANDS - 1);
    for y in 0..bottom {
        for x in 0..WIDTH {
            let mut index = band(y);
            if band(y + 1) != index && (x + y) % 2 == 0 {
                index += 1;
            }
            let color = mix(SKY_TOP, SKY_LOW, index as f32 / (BANDS - 1) as f32);
            scene.set(x, y, color);
        }
    }
    // A big fair-weather cloud, smaller ones drifting, and long thin ones low over the hills.
    cloud(
        scene,
        (292, 30),
        &[
            (-24, 2, 7),
            (-12, -4, 10),
            (2, -8, 12),
            (16, -3, 10),
            (28, 2, 7),
        ],
    );
    cloud(scene, (176, 18), &[(-10, 0, 6), (0, -4, 8), (10, -1, 6)]);
    cloud(scene, (362, 56), &[(-6, 0, 4), (2, -2, 5), (9, 0, 3)]);
    cloud(scene, (120, 46), &[(-5, 0, 3), (2, -2, 4), (8, 0, 3)]);
    for (x, y, long) in [(140, 58, 34), (232, 54, 46), (318, 60, 28)] {
        for dx in 0..long {
            let fade = (dx.min(long - dx) * 40).min(150) as u8;
            put(scene, x + dx, y, rgba(0xfdfbf5, fade));
            if (4..long - 6).contains(&dx) {
                put(scene, x + dx + 3, y + 1, rgba(0xdfe7ee, fade / 2));
            }
        }
    }
}

/// A heaped cloud from round puffs `(dx, dy, radius)`: flat underneath, sunlit on top and to the
/// left, cool grey below, and edged only along its underside, which is the only hard edge a cloud
/// has.
fn cloud(scene: &mut Canvas, (cx, cy): (i32, i32), puffs: &[(i32, i32, i32)]) {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let floor = cy + 3;
    for &(dx, dy, radius) in puffs {
        let (px, py) = (cx + dx, cy + dy);
        for y in py - radius..=(py + radius).min(floor) {
            for x in px - radius * 3 / 2..=px + radius * 3 / 2 {
                let (ex, ey) = (
                    (x - px) as f32 / (radius as f32 * 1.4),
                    (y - py) as f32 / radius as f32,
                );
                let reach = ex * ex + ey * ey;
                if reach > 1.0 {
                    continue;
                }
                // Lit towards the upper left of each puff, shaded towards the flat base.
                let toward = ex * 0.7 + ey;
                let low = (y - (floor - radius / 2)) as f32 / (radius as f32 / 2.0 + 1.0);
                let color = if low > 0.5 {
                    CLOUDS.shadow
                } else if toward < -0.55 && reach > 0.35 {
                    CLOUDS.shine
                } else if toward > 0.45 || low > 0.0 {
                    CLOUDS.base
                } else {
                    CLOUDS.light
                };
                if layer.get(x, y).a == 0 || color_rank(color) > color_rank(layer.get(x, y)) {
                    layer.set(x, y, color);
                }
            }
        }
    }
    for x in cx - 60..cx + 60 {
        for y in cy - 30..=floor {
            if layer.get(x, y).a > 0 && layer.get(x, y + 1).a == 0 {
                layer.set(x, y, CLOUDS.edge);
            }
        }
    }
    blit(scene, &layer, 0, 0);
}

/// How light a cloud tone is, so overlapping puffs keep their lit sides.
fn color_rank(color: Rgba) -> u32 {
    u32::from(color.r) + u32::from(color.g) + u32::from(color.b)
}

// ---------------------------------------------------------------------------------------------
// The valley far below
// ---------------------------------------------------------------------------------------------

/// Where a pixel of the plain lies on the ground below: how far across from straight ahead, and
/// how far off, both in field lengths.
fn ground(x: i32, y: i32) -> (f32, f32) {
    let below = (y.max(HORIZON) - HORIZON) as f32 + 1.0;
    ((x - VANISH) as f32 * 0.8 / below, 36.0 / below)
}

/// How much the air between here and a row of the plain pales it.
fn haze(y: i32) -> f32 {
    // Even the foot of the Hill is a long way down.
    let (_, off) = ground(0, y);
    0.16 + (1.0 - (-(off - 0.6) * 0.26).exp()).clamp(0.0, 0.55)
}

/// A colour as it looks from up here, at a row of the plain.
fn far(color: Rgba, y: i32) -> Rgba {
    mix(color, HAZE, haze(y))
}

/// Which field a point of the plain falls in. Rows of fields grow deeper the further off they
/// are, run a little aslant, and each row is cut into fields of its own widths, so the hedges
/// never line up into a grid.
fn field(u: f32, v: f32) -> (i32, i32) {
    let aslant = v + u * 0.1 + (u * 1.7).sin() * 0.03;
    let row = (aslant.max(0.05).ln() / 1.24f32.ln()).floor() as i32;
    let width = 0.4 + (noise(row, 0, 701) % 50) as f32 / 100.0;
    let offset = (noise(row, 1, 702) % 100) as f32 / 100.0;
    (row, (u / width + offset).floor() as i32)
}

fn field_at(x: i32, y: i32) -> (i32, i32) {
    let (u, v) = ground(x, y);
    field(u, v)
}

/// Whether a pixel of the plain is hedge: along the far and left edges of each field, as long as
/// the fields there are big enough for their hedges to show.
fn hedge_at(x: i32, y: i32) -> bool {
    let below = y - HORIZON + 1;
    let here = field_at(x, y);
    (below > 6 && field_at(x, y - 1) != here) || (below > 13 && field_at(x - 1, y) != here)
}

/// What grows in a field, and how it looks at a pixel: pasture mostly, with hay, wheat, plough
/// and young crops among it, rows and furrows showing only where they are near enough to.
fn crop((row, column): (i32, i32), x: i32, y: i32) -> Rgba {
    let (u, v) = ground(x, y);
    let below = (y - HORIZON) as f32 + 1.0;
    let kind = noise(row, column, 711) % 16;
    let (a, b) = match kind {
        0..=4 => (rgb(0x8cc27a), rgb(0x7fb66f)),
        5..=7 => (rgb(0xa3cf8a), rgb(0x94c47e)),
        8 | 9 => (rgb(0x74a862), rgb(0x689b58)),
        10 | 11 => (rgb(0xdcc57c), rgb(0xc8b06a)),
        12 => (rgb(0xe8d898), rgb(0xd8c47e)),
        13 | 14 => (rgb(0xb39272), rgb(0x977657)),
        _ => (rgb(0x95ba66), rgb(0x7c9e56)),
    };
    let striped = match kind {
        // Furrows and drills run away from the eye, closing up with distance.
        13..=15 if below > 26.0 => Some(((u * 18.0).floor() as i32).rem_euclid(2) == 0),
        // Hay lies in mown rows across.
        10 | 11 if below > 28.0 => Some(((v * 12.0).floor() as i32).rem_euclid(2) == 0),
        _ => None,
    };
    match striped {
        Some(true) => a,
        Some(false) => b,
        None if matches!(kind, 10 | 11 | 13..=15) => mix(a, b, 0.5),
        None if noise(x, y, 712).is_multiple_of(11) => b,
        None => a,
    }
}

fn plain(scene: &mut Canvas) {
    let mut trees = Vec::new();
    for y in HORIZON..118 {
        let below = y - HORIZON + 1;
        for x in 0..WIDTH {
            let mut color = if hedge_at(x, y) {
                if below > 11 && chance(x, y, 721, 18) {
                    trees.push((x, y));
                }
                HEDGE.shadow
            } else if below > 10 && (hedge_at(x - 1, y) || hedge_at(x, y - 1)) {
                // The hedge's shadow, falling to the lower right.
                mix(crop(field_at(x, y), x, y), HEDGE.shadow, 0.3)
            } else {
                crop(field_at(x, y), x, y)
            };
            // The shadows of the clouds overhead, lying across the fields.
            let (u, v) = ground(x, y);
            for (cu, cv, ru, rv) in [(1.6, 2.4, 1.1, 0.42), (-3.4, 1.5, 0.9, 0.3)] {
                let (du, dv) = ((u - cu) / ru, (v - cv) / rv);
                if du * du + dv * dv <= 1.0 {
                    color = mix(color, rgb(0x3d5e58), 0.2);
                }
            }
            scene.set(x, y, far(color, y));
        }
    }
    for (x, y) in trees {
        let size = if y - HORIZON > 30 { 2 } else { 1 };
        hedgerow_tree(scene, x, y, size);
    }
}

/// A tree in a hedgerow, `size` 1 or 2: a dark round head lit on its upper left.
fn hedgerow_tree(scene: &mut Canvas, x: i32, y: i32, size: i32) {
    let tone = |color| far(color, y);
    ellipse(scene, x + 1, y, size, size, tone(HEDGE.edge));
    ellipse(scene, x, y - size, size, size, tone(HEDGE.base));
    put(scene, x - size / 2, y - size - size / 2, tone(HEDGE.light));
    if size > 1 {
        put(scene, x - 1, y - size - 1, tone(HEDGE.shine));
        put(scene, x + size, y - size + 1, tone(HEDGE.shadow));
    }
}

/// Two ranges of hills along the horizon, the further one bluer, each lit on its left-facing
/// slopes and edged along its crest.
fn far_hills(scene: &mut Canvas) {
    let ranges = [
        (HORIZON - 7, 6.0, rgb(0xa7c2cc), 0.0),
        (HORIZON - 1, 4.0, rgb(0xa9cba9), 2.1),
    ];
    for (base, rise, body, phase) in ranges {
        let top = |x: i32| {
            let x = x as f32;
            (base as f32
                - rise
                    * (1.0
                        + 0.55 * (x / 41.0 + phase).sin()
                        + 0.3 * (x / 19.0 + phase * 2.0).sin()
                        + 0.15 * (x / 8.0 + phase).sin()))
            .round() as i32
        };
        let edge = mix(body, rgb(0x5f7f7c), 0.22);
        for x in 0..WIDTH {
            let crest = top(x);
            // Slopes facing left catch the light; those facing right are in shade.
            let facing = top(x + 2) - top(x - 2);
            for y in crest..=HORIZON + 1 {
                let color = if y == crest || (y == crest + 1 && top(x - 1) > crest + 1) {
                    edge
                } else if facing > 0 && y < crest + 4 {
                    mix(body, rgb(0xf4f6ee), 0.3)
                } else if facing < 0 && y < crest + 3 {
                    mix(body, edge, 0.4)
                } else {
                    body
                };
                scene.set(x, y, color);
            }
        }
    }
}

/// A point along a smooth curve through `points`, `t` of the way from the first to the last.
fn along(points: &[(f32, f32)], t: f32) -> (f32, f32) {
    let spans = (points.len() - 1) as f32;
    let at = (t * spans).clamp(0.0, spans - 0.001);
    let index = at.floor() as usize;
    let local = at - index as f32;
    let get = |i: isize| points[i.clamp(0, points.len() as isize - 1) as usize];
    let (p0, p1, p2, p3) = (
        get(index as isize - 1),
        get(index as isize),
        get(index as isize + 1),
        get(index as isize + 2),
    );
    // Catmull-Rom, so the curve passes through every point.
    let blend = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (c - a) * local
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * local * local
            + (3.0 * b - a - 3.0 * c + d) * local * local * local)
    };
    (blend(p0.0, p1.0, p2.0, p3.0), blend(p0.1, p1.1, p2.1, p3.1))
}

/// The lane down on the plain, winding from the foot of the Hill past the station and through
/// the village, out by the fairground and away towards the hills.
fn lane(scene: &mut Canvas) {
    let points = [
        (318.0, 104.0),
        (300.0, 97.0),
        (270.0, 93.0),
        (252.0, 89.0),
        (240.0, 87.5),
        (218.0, 85.0),
        (198.0, 82.0),
        (178.0, 80.5),
        (160.0, 77.0),
        (164.0, 74.0),
        (150.0, 71.0),
        (140.0, 69.0),
    ];
    let steps = 600;
    for step in 0..=steps {
        let (x, y) = along(&points, step as f32 / steps as f32);
        let (x, y) = (x.round() as i32, y.round() as i32);
        let tone = |color| mix(far(color, y), color, 0.45);
        put(scene, x, y, tone(EARTH.shine));
        put(scene, x, y + 1, tone(EARTH.shadow));
    }
}

/// The railway, coming in from the right past the station and curving away across the plain:
/// a dark bed of ballast, its rails catching the light where they are near enough to.
fn railway(scene: &mut Canvas) {
    let points = [
        (392.0, 97.0),
        (330.0, 92.0),
        (280.0, 89.0),
        (220.0, 85.0),
        (160.0, 81.0),
        (110.0, 78.0),
        (60.0, 76.0),
        (0.0, 75.0),
    ];
    let steps = 900;
    let mut last = (i32::MIN, i32::MIN);
    for step in 0..=steps {
        let (x, y) = along(&points, step as f32 / steps as f32);
        let (x, y) = (x.round() as i32, y.round() as i32);
        if (x, y) == last {
            continue;
        }
        last = (x, y);
        let tone = |color| mix(far(color, y), color, 0.25);
        put(scene, x, y, tone(rgb(0x5f5752)));
        if y > 84 {
            put(
                scene,
                x,
                y - 1,
                tone(if x % 2 == 0 { RAIL.light } else { RAIL.base }),
            );
        }
    }
}

/// The Woods: a dark wood out on the plain, tree heads crowded together.
fn woods(scene: &mut Canvas) {
    let (cx, cy) = WOODS;
    for index in 0..34 {
        let x = cx - 18 + (noise(index, 0, 731) % 37) as i32;
        let spread = 3 - ((x - cx).abs() / 7).min(3);
        let y = cy - (noise(index, 1, 732) % (spread as u32 + 1)) as i32;
        let tone = |color| far(color, y + 3);
        ellipse(scene, x + 1, y + 1, 2, 2, tone(rgb(0x1c3424)));
        ellipse(scene, x, y, 2, 2, tone(rgb(0x2f5538)));
        put(scene, x - 1, y - 1, tone(rgb(0x4b7b52)));
    }
}

/// The fairground in its field: the big top's red and white stripes, the carousel, a caravan.
fn fairground(scene: &mut Canvas) {
    let (cx, base) = FAIR;
    let tone = |color| mix(far(color, base), color, 0.35);
    ellipse(scene, cx + 6, base, 16, 2, tone(rgb(0xc8a274)));
    // The big top: striped walls under a striped cone, its pennant flying.
    let red = rgb(0xc0403e);
    let cream = rgb(0xf4ecd8);
    for (row, half) in [4, 4, 4, 3, 2, 1, 0].into_iter().enumerate() {
        for dx in -half..=half {
            let mut color = if dx % 2 == 0 { red } else { cream };
            if dx > 0 {
                color = mix(color, rgb(0x5e1a24), 0.25);
            }
            put(scene, cx + dx, base - 1 - row as i32, tone(color));
        }
    }
    put(scene, cx, base - 1, tone(rgb(0x2a1a22)));
    put(scene, cx, base - 8, tone(rgb(0x6b4a24)));
    put(scene, cx + 1, base - 9, tone(rgb(0xf5d25e)));
    // The carousel: a teal and cream canopy over a dark deck.
    let teal = rgb(0x2f7a7a);
    for (row, half) in [(0, 3), (1, 3), (2, 2), (3, 1)] {
        for dx in -half..=half {
            let color = match row {
                0 => rgb(0x6a3a4a),
                _ if dx % 2 == 0 => teal,
                _ => cream,
            };
            put(scene, cx + 11 + dx, base - 1 - row, tone(color));
        }
    }
    // A painted caravan at the end of the field.
    for (dx, dy) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
        put(scene, cx + 17 + dx, base - 1 - dy, tone(rgb(0x8a3a4a)));
    }
}

/// The village around its green: the oak at one end of the lawn, and cottages about it.
fn village_green(scene: &mut Canvas) {
    let (cx, cy) = GREEN;
    let tone = |color, y| mix(far(color, y), color, 0.3);
    // The lawn, mown in stripes.
    for y in cy - 2..=cy + 2 {
        for x in cx - 10..=cx + 10 {
            let (dx, dy) = ((x - cx) as f32 / 10.5, (y - cy) as f32 / 2.6);
            if dx * dx + dy * dy <= 1.0 {
                let stripe = (x / 2 + y).rem_euclid(2) == 0;
                put(
                    scene,
                    x,
                    y,
                    tone(if stripe { rgb(0x9ccd84) } else { rgb(0x8cc178) }, y),
                );
            }
        }
    }
    // Cottages: whitewashed walls under red and brown roofs, a chimney or two.
    for (x, y, roof) in [
        (cx + 13, cy - 2, TILE.base),
        (cx - 16, cy + 1, rgb(0x8c6a4a)),
        (cx + 19, cy + 2, TILE.base),
        (cx + 4, cy + 4, rgb(0x8c6a4a)),
        (cx - 8, cy - 4, TILE.light),
    ] {
        for dx in 0..4 {
            put(scene, x + dx, y, tone(PLASTER.base, y));
        }
        put(scene, x + 3, y, tone(PLASTER.shadow, y));
        put(scene, x + 1, y, tone(rgb(0x4f6f80), y));
        for dx in -1..5 {
            put(scene, x + dx, y - 1, tone(roof, y));
        }
        for dx in 0..4 {
            put(
                scene,
                x + dx,
                y - 2,
                tone(mix(roof, rgb(0xffffff), 0.15), y),
            );
        }
        put(scene, x + 3, y - 3, tone(BRICK.base, y));
    }
    // The oak: a big dark head at the end of the lawn, lit on its upper left, with its shade.
    let (ox, oy) = (cx - 6, cy - 2);
    ellipse(scene, ox + 3, oy + 3, 4, 1, tone(rgb(0x5f8f55), oy));
    vline(scene, ox, oy + 1, 2, tone(BARK.shadow, oy));
    ellipse(scene, ox + 1, oy - 2, 4, 3, tone(rgb(0x1f3a26), oy));
    ellipse(scene, ox, oy - 3, 4, 3, tone(rgb(0x335e3b), oy));
    ellipse(scene, ox - 1, oy - 4, 2, 1, tone(rgb(0x447a4a), oy));
    put(scene, ox - 2, oy - 5, tone(rgb(0x5f9a5b), oy));
    // Bunting across the green, a dot of each colour.
    for (index, dx) in (-3..8).step_by(2).enumerate() {
        put(
            scene,
            cx + dx,
            cy - 3 + (dx - 2).abs() / 4,
            tone(BLOSSOMS[index % 4], cy),
        );
    }
}

/// The station at the foot of the Hill: the house's red roof and chimney, the platform beside
/// the line, and the garden fence along the bottom of the Hill.
fn station(scene: &mut Canvas) {
    let (cx, base) = STATION;
    let tone = |color| mix(far(color, base), color, 0.4);
    // The platform, pale stone along the line, and its canopy.
    for x in cx - 4..cx + 22 {
        put(scene, x, base - 5, tone(STONE.light));
        put(scene, x, base - 4, tone(STONE.shadow));
    }
    for x in cx + 6..cx + 16 {
        put(scene, x, base - 7, tone(rgb(0x5d626b)));
        put(scene, x, base - 6, tone(rgb(0x2f5d50)));
    }
    // The house: plastered walls, windows and the green door, under a red-tiled hipped roof.
    for y in base - 4..base {
        for x in cx - 6..cx + 6 {
            let color = if x == cx + 5 {
                PLASTER.shadow
            } else if y == base - 3 && (x == cx - 4 || x == cx + 3) {
                rgb(0x4f6f80)
            } else if x == cx && y > base - 3 {
                rgb(0x2f5d50)
            } else {
                PLASTER.base
            };
            put(scene, x, y, tone(color));
        }
    }
    for (row, (left, right)) in [(-8, 7), (-7, 6), (-6, 5), (-4, 3)].into_iter().enumerate() {
        let y = base - 5 - row as i32;
        for dx in left..=right {
            let color = if row == 0 {
                TILE.shadow
            } else if dx < 0 {
                TILE.light
            } else {
                TILE.base
            };
            put(scene, cx + dx, y, tone(color));
        }
    }
    vline(scene, cx - 3, base - 11, 3, tone(BRICK.base));
    put(scene, cx - 3, base - 12, tone(BRICK.shadow));
    // The garden fence between the station and the Hill, white pickets catching the sun.
    for x in cx - 20..cx + 30 {
        if x % 2 == 0 && !(cx - 8..cx + 8).contains(&x) {
            put(scene, x, base + 1, tone(TRIM.light));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The summit
// ---------------------------------------------------------------------------------------------

/// The summit's back edge, where it rounds over and the valley shows beyond: highest in the
/// middle, falling away a little to either side.
fn brow(x: i32) -> i32 {
    let t = (x - 210) as f32 / 200.0;
    (100.0 + 11.0 * t * t).round() as i32
}

/// How much of the Hill's own side shows beyond the summit: none straight ahead, where the brow
/// hides it, and more to either side, where the summit rounds away.
fn flank(x: i32) -> i32 {
    let t = (x - 210) as f32 / 200.0;
    ((t.abs() - 0.42).max(0.0) * 22.0).round() as i32
}

/// The Hill's sides, going down to the plain beyond the summit's edges: rough grass and gorse,
/// lit on the left, in shade on the right, with a hedge along the foot.
fn flanks(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let (foot, edge) = (brow(x) - flank(x), brow(x));
        let lit = x < VANISH;
        for y in foot..edge {
            let base = if lit {
                mix(TURF.base, TURF.light, 0.2)
            } else {
                mix(TURF.shadow, TURF.edge, 0.3)
            };
            let color = match noise(x.div_euclid(2), y, 741) % 9 {
                0 => mix(base, TURF.edge, 0.35),
                1 => mix(base, TURF.light, 0.3),
                _ => base,
            };
            scene.set(x, y, mix(color, HAZE, haze(y) * 0.5));
        }
        if flank(x) > 2 && chance(x, 0, 742, 70) {
            hedgerow_tree(scene, x, foot + 1, 1);
        }
        if flank(x) > 5 && chance(x, 1, 743, 14) {
            let y = foot + 3 + (noise(x, 2, 744) % (flank(x) as u32 - 4)) as i32;
            put(scene, x, y, mix(TURF.edge, HAZE, 0.2));
            put(scene, x - 1, y, mix(TURF.shadow, HAZE, 0.2));
            put(scene, x - 1, y - 1, mix(rgb(0xe8c860), HAZE, 0.2));
        }
    }
}

/// How lit the turf is at a point, 0 to 1: brighter to the left, where the summit rounds towards
/// the sun, and towards the back, falling into shade on the right and as it comes down to the
/// front, with broad soft patches where the grass lies differently.
fn light(x: i32, y: i32) -> f32 {
    let t = (x - 210) as f32 / 200.0;
    let (across, back) = turf_ground(x, y);
    let patches = smooth_noise(across / 30.0, back / 5.0, 751) * 0.65
        + smooth_noise(across / 11.0, back / 2.2, 752) * 0.35;
    0.64 - 0.3 * t - 0.3 * nearness(x, y) + (patches - 0.5) * 0.55
}

/// Where a point of the summit lies on the ground, so its texture can grow as the turf comes
/// nearer: across from the middle, and how far back.
fn turf_ground(x: i32, y: i32) -> (f32, f32) {
    let from_eye = (y.max(60) - 52) as f32;
    ((x - 192) as f32 * 48.0 / from_eye, 48.0 * from_eye.ln())
}

/// Noise that rolls smoothly between whole-numbered points, for soft patches rather than
/// speckle.
fn smooth_noise(x: f32, y: f32, salt: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy) = (ease(x - ix as f32), ease(y - iy as f32));
    let corner = |dx: i32, dy: i32| (noise(ix + dx, iy + dy, salt) % 1000) as f32 / 999.0;
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// How far down the summit a row is, 0 at the brow and 1 at the front.
fn nearness(x: i32, y: i32) -> f32 {
    let top = brow(x);
    ((y - top) as f32 / (HEIGHT - top) as f32).clamp(0.0, 1.0)
}

/// The turf's tone at a step of its ramp, from -1 (deep in the grass) to 4 (sunlit).
fn turf(step: i32) -> Rgba {
    match step {
        ..=-1 => mix(TURF.edge, TURF.shadow, 0.45),
        0 => TURF.shadow,
        1 => mix(TURF.shadow, TURF.base, 0.5),
        2 => TURF.base,
        3 => mix(TURF.base, TURF.light, 0.5),
        _ => TURF.light,
    }
}

/// Which step of the turf's ramp a pixel takes: its light, its edges broken a pixel at a time.
fn turf_step(x: i32, y: i32) -> i32 {
    let scatter = (noise(x, y, 753) % 100) as f32 / 100.0 - 0.5;
    (light(x, y) * 4.0 + scatter * 0.45).round().clamp(0.0, 4.0) as i32
}

fn summit(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let top = brow(x);
        for y in top..HEIGHT {
            scene.set(x, y, turf(turf_step(x, y)));
        }
        // The crest against the valley, outlined and catching the light, with grass standing up
        // along it.
        scene.set(x, top, TURF.shadow);
        put(scene, x, top + 1, turf(turf_step(x, top + 1) + 1));
        if chance(x, 0, 752, 90) {
            let tall = 1 + (noise(x, 1, 753) % 2) as i32;
            vline(scene, x, top - tall, tall, TURF.shadow);
        }
    }
}

/// The path's middle and half-width at `x`, from the left edge where the colony comes up, until
/// it wears away into the turf.
fn path_at(x: i32) -> Option<(f32, f32)> {
    const END: i32 = 112;
    if !(-8..END).contains(&x) {
        return None;
    }
    // Along the line the colony walks in by, from the entrance off the left edge.
    let along = (x + 8) as f32 / (END + 8) as f32;
    let middle = 195.0 - 13.0 * along + 3.0 * (along * 3.1).sin();
    // Wider where it comes up over the edge, narrowing as it goes, its edges never quite even.
    let wobble = smooth_noise(x as f32 / 6.0, 0.5, 767) - 0.5;
    let half = 7.5 - 5.0 * along + wobble * 1.6;
    Some((middle, half))
}

/// How much of the path there is at a point: 1 in the trodden middle, down to 0 at its edge and
/// where it fades into the grass at its far end.
fn path_wear(x: i32, y: i32) -> f32 {
    let Some((middle, half)) = path_at(x) else {
        return 0.0;
    };
    let across = (y as f32 + 0.5 - middle).abs() / half;
    let fade = ((112 - x) as f32 / 24.0).min(1.0);
    ((1.0 - across) * 4.0).min(1.0) * fade
}

/// The worn path up onto the summit at the front left, where the colony comes up from the
/// station.
fn path(scene: &mut Canvas) {
    for x in 0..116 {
        for y in 170..HEIGHT {
            let wear = path_wear(x, y);
            if wear <= 0.0 {
                continue;
            }
            // Where it wears away, it breaks into bare patches among the grass.
            let patchy = (noise(x.div_euclid(2), y, 761) % 100) as f32 / 100.0;
            if wear < 1.0 && patchy > wear {
                continue;
            }
            let Some((middle, half)) = path_at(x) else {
                continue;
            };
            // A shallow trodden hollow: its far bank in shade, the middle worn pale.
            let across = (y as f32 + 0.5 - middle) / half;
            let base = if across < -0.62 {
                EARTH.shadow
            } else if across < 0.3 {
                EARTH.light
            } else {
                EARTH.base
            };
            let color = match noise(x, y, 762) % 14 {
                0 => mix(base, EARTH.shadow, 0.7),
                1 | 2 => mix(base, EARTH.shine, 0.5),
                _ => base,
            };
            scene.set(x, y, color);
        }
    }
    for x in 0..116 {
        let Some((middle, half)) = path_at(x) else {
            continue;
        };
        // The far bank's lip, broken where the grass grows over it; the grass along the near
        // edge leaning across.
        let far_edge = (middle - half).ceil() as i32;
        if path_wear(x, far_edge + 1) >= 0.9 && !chance(x, 2, 768, 70) {
            put(scene, x, far_edge, mix(EARTH.edge, TURF.shadow, 0.35));
        }
        let near = (middle + half).floor() as i32;
        if path_wear(x, near - 2) >= 0.9 && chance(x, 0, 765, 150) {
            let tall = 1 + (noise(x, 1, 766) % 3) as i32;
            vline(scene, x, near - tall, tall + 1, TURF.shadow);
            put(scene, x, near - tall, TURF.base);
        }
    }
    // Pebbles kicked along it, lit on top.
    for index in 0..9 {
        let x = 4 + (noise(index, 0, 763) % 84) as i32;
        let Some((middle, half)) = path_at(x) else {
            continue;
        };
        let y = (middle - half * 0.4 + (noise(index, 1, 764) % 100) as f32 / 100.0 * half).round()
            as i32;
        put(scene, x, y, STONE.light);
        put(scene, x + 1, y, STONE.base);
        put(scene, x, y + 1, mix(EARTH.edge, STONE.shadow, 0.5));
        put(scene, x + 1, y + 1, EARTH.shadow);
    }
}

/// How far into one of the summit's places a point is, 0 at its middle and 1 at its edge, for the
/// nearest place.
fn place_reach(x: i32, y: i32) -> f32 {
    SPOTS
        .iter()
        .enumerate()
        .map(|(index, &(sx, sy))| {
            let wide = 13.0 + (noise(index as i32, 0, 771) % 3) as f32;
            let (dx, dy) = ((x as f32 + 0.5 - sx) / wide, (y as f32 + 0.5 - sy) / 3.2);
            dx * dx + dy * dy
        })
        .fold(f32::MAX, f32::min)
}

/// The flattened places where pieces will stand: shorter, paler grass in a shallow oval, its far
/// rim shaded by the longer grass behind and longer grass standing along its near edge. Each is a
/// little different, so the summit hints at them without looking ruled out.
fn places(scene: &mut Canvas) {
    for (index, &(sx, sy)) in SPOTS.iter().enumerate() {
        let wide = 13.0 + (noise(index as i32, 0, 771) % 3) as f32;
        let inside = |x: i32, y: i32| {
            let (dx, dy) = ((x as f32 + 0.5 - sx) / wide, (y as f32 + 0.5 - sy) / 3.3);
            let ragged = (noise(x, y, 772) % 100) as f32 / 100.0 * 0.16;
            dx * dx + dy * dy + ragged <= 1.0
        };
        let (cx, cy) = (sx as i32, sy as i32);
        for y in cy - 5..=cy + 5 {
            for x in cx - 18..=cx + 18 {
                let step = (light(x, y) * 4.0).round() as i32;
                if !inside(x, y) {
                    if inside(x, y - 1) && chance(x, y, 774, 110) {
                        vline(scene, x, y - 1, 2, turf(step - 1));
                    }
                    continue;
                }
                let color = if !inside(x, y - 1) {
                    turf(step - 1)
                } else if (!inside(x, y - 2) && x < cx) || chance(x, y, 773, 16) {
                    // Lit along the far edge on the sunny side, and here and there within.
                    turf(step + 2)
                } else {
                    turf(step + 1)
                };
                scene.set(x, y, color);
            }
        }
    }
}

/// Whether grass detail stays off a point: in a place, on the path, or under the old tree.
fn kept_clear(x: i32, y: i32, room: f32) -> bool {
    let (left, top, right, bottom) = TREE;
    place_reach(x, y) < room
        || path_wear(x, y) > 0.0
        || (x as f32 >= left - 2.0
            && x as f32 <= right + 2.0
            && y as f32 >= top
            && y as f32 <= bottom + 2.0)
}

/// Blades, a few tufts and wildflowers over the turf: sparse, so the summit stays open.
fn grass(scene: &mut Canvas) {
    for x in 0..WIDTH {
        for y in brow(x) + 2..HEIGHT {
            let scale = (y - 52) as f32 / 48.0;
            let per = (30.0 / (scale * scale)) as u32 + 3;
            if !chance(x, y, 781, per) || kept_clear(x, y, 1.4) {
                continue;
            }
            // A little fan of blades, darker than the turf around it, its tip lit.
            let step = turf_step(x, y);
            let tall = (scale * 0.9).round().max(1.0) as i32;
            vline(scene, x, y - tall + 1, tall, turf(step - 2));
            if scale > 1.7 {
                vline(scene, x + 1, y - tall + 2, tall - 1, turf(step - 1));
                put(scene, x - 1, y, turf(step - 1));
            }
            if chance(x, y, 782, 90) {
                put(scene, x, y - tall, turf(step + 1));
            }
        }
    }
    // Tufts, here and there.
    for index in 0..20 {
        let x = 6 + (noise(index, 0, 783) % (WIDTH as u32 - 12)) as i32;
        let y = brow(x) + 6 + (noise(index, 1, 784) % (HEIGHT - brow(x) - 10) as u32) as i32;
        if kept_clear(x, y, 2.2) {
            continue;
        }
        tuft(scene, x, y);
    }
    // Wildflowers in little scatters: daisies and buttercups mostly, a few others.
    for index in 0..16 {
        let x = 8 + (noise(index, 0, 785) % (WIDTH as u32 - 16)) as i32;
        let y = brow(x) + 8 + (noise(index, 1, 786) % (HEIGHT - brow(x) - 14) as u32) as i32;
        for floret in 0..3 + (noise(index, 2, 787) % 3) as i32 {
            let fx = x + (noise(index, floret, 788) % 9) as i32 - 4;
            let fy = y + (noise(floret, index, 789) % 5) as i32 - 2;
            if kept_clear(fx, fy, 2.0) {
                continue;
            }
            let blossom = match noise(index, floret, 790) % 8 {
                0..=3 => BLOSSOMS[3],
                4 | 5 => BLOSSOMS[2],
                6 => BLOSSOMS[1],
                _ => BLOSSOMS[4],
            };
            put(scene, fx, fy + 1, TURF.edge);
            put(scene, fx, fy, blossom);
            if blossom == BLOSSOMS[3] && noise(index, floret, 791).is_multiple_of(2) {
                put(scene, fx + 1, fy, mix(blossom, TURF.base, 0.5));
            }
        }
    }
}

/// A tuft of longer grass, its blades fanning out, lit on the left.
fn tuft(scene: &mut Canvas, x: i32, y: i32) {
    let near = nearness(x, y);
    let tall = 3 + (near * 3.0) as i32;
    for (dx, lean, length, color) in [
        (-2, -1, tall - 2, TURF.light),
        (-1, 0, tall - 1, TURF.base),
        (0, 0, tall, TURF.shadow),
        (1, 1, tall - 1, TURF.shadow),
        (2, 1, tall - 2, TURF.edge),
    ] {
        line(scene, (x + dx, y), (x + dx + lean, y - length + 1), color);
    }
    put(scene, x - 1, y - tall + 1, TURF.light);
}

// ---------------------------------------------------------------------------------------------
// The old tree
// ---------------------------------------------------------------------------------------------

/// The crown as overlapping masses `(x, y, radius)`: a great dome with shoulders either side and
/// the limbs showing underneath, as the station sees it on the skyline, kept in on the right low
/// down so the nearest place has room.
const CROWN_MASSES: [(i32, i32, i32); 8] = [
    (50, 30, 27),
    (26, 20, 16),
    (74, 22, 15),
    (8, 36, 13),
    (22, 50, 18),
    (80, 46, 15),
    (66, 54, 9),
    (6, 56, 9),
];

/// Where the trunk stands: its middle, and the row where it meets the ground.
fn trunk_foot() -> (i32, i32) {
    let (left, _, right, bottom) = TREE;
    (((left + right) / 2.0) as i32, bottom as i32 - 4)
}

fn in_crown(x: i32, y: i32, inset: i32) -> bool {
    CROWN_MASSES.iter().any(|&(cx, cy, radius)| {
        let reach = radius - inset;
        reach > 0 && (x - cx).pow(2) + (y - cy).pow(2) <= reach * reach
    })
}

fn old_tree(scene: &mut Canvas) {
    let (foot_x, foot_y) = trunk_foot();
    // Its shade on the turf, falling away to the right, with sun coming through.
    for y in foot_y - 4..foot_y + 6 {
        for x in foot_x - 20..foot_x + 46 {
            let (dx, dy) = (
                (x - foot_x - 12) as f32 / 34.0,
                (y - foot_y - 1) as f32 / 4.5,
            );
            if dx * dx + dy * dy <= 1.0 && !chance(x.div_euclid(2), y, 801, 50) {
                put(scene, x, y, SHADE);
            }
        }
    }
    trunk(scene, foot_x, foot_y);
    crown(scene);
}

/// The trunk: old and broad, a little crooked, flaring into roots at its foot, furrowed, lit from
/// the left, with a knot-hole partway up.
fn trunk(scene: &mut Canvas, foot_x: i32, foot_y: i32) {
    let middle = |y: i32| {
        let up = (foot_y - y) as f32;
        foot_x as f32 - up * 0.04 + (up / 14.0).sin() * 1.2
    };
    let half = |y: i32| {
        let flare = (y - (foot_y - 12)).max(0) as f32;
        let fork = (66 - y).max(0) as f32;
        6.5 + flare * flare * 0.07 + fork * 0.3
    };
    for y in 50..=foot_y + 1 {
        let (centre, width) = (middle(y), half(y));
        let (left, right) = (
            (centre - width).round() as i32,
            (centre + width).round() as i32,
        );
        for x in left..=right {
            let across = (x as f32 + 0.5 - centre) / width;
            let mut color = if x == left || x == right {
                BARK.edge
            } else if across < -0.55 {
                BARK.light
            } else if across > 0.45 {
                BARK.shadow
            } else {
                BARK.base
            };
            // Furrows running up the bark, and the odd lit ridge on the sunny side.
            if x != left && x != right {
                if noise(x, y.div_euclid(4), 811).is_multiple_of(6) {
                    color = mix(color, BARK.edge, 0.6);
                } else if across < 0.0 && chance(x, y, 812, 24) {
                    color = mix(color, BARK.shine, 0.5);
                }
            }
            // Under the crown, the trunk is in shade.
            if y < 72 {
                color = mix(color, BARK.edge, ((72 - y) as f32 / 22.0).min(0.7));
            }
            scene.set(x, y, color);
        }
    }
    // Roots, thick where they leave the trunk, sinking into the grass.
    for (from, to) in [
        ((foot_x - 13, foot_y), (foot_x - 19, foot_y + 2)),
        ((foot_x + 13, foot_y), (foot_x + 20, foot_y + 3)),
        ((foot_x + 3, foot_y + 1), (foot_x + 5, foot_y + 3)),
    ] {
        line(scene, (from.0, from.1 - 1), (to.0, to.1 - 1), BARK.light);
        line(scene, from, to, BARK.base);
        line(scene, (from.0, from.1 + 1), (to.0, to.1 + 1), BARK.shadow);
        line(scene, (from.0, from.1 + 2), (to.0, to.1 + 2), BARK.edge);
    }
    // A knot-hole, dark inside, its lower lip catching the light.
    let (kx, ky) = (foot_x - 3, foot_y - 22);
    ellipse(scene, kx, ky, 2, 3, BARK.edge);
    put(scene, kx, ky, rgb(0x241a16));
    put(scene, kx, ky + 1, rgb(0x241a16));
    put(scene, kx + 1, ky + 3, BARK.light);
    put(scene, kx - 1, ky + 3, BARK.light);
    // Grass growing up against the foot.
    for x in foot_x - 20..foot_x + 22 {
        if chance(x, 0, 813, 120) {
            let tall = 1 + (noise(x, 1, 814) % 3) as i32;
            vline(scene, x, foot_y + 4 - tall, tall, TURF.shadow);
            put(scene, x, foot_y + 3 - tall, TURF.base);
        }
    }
}

/// The crown: clumps of leaves, back to front, each lit on its upper left and darker the further
/// it sits to the lower right of its own mass and of the tree, so the great masses read through
/// the leaves; the limbs show among the lowest clumps, and one outline goes round it all.
fn crown(scene: &mut Canvas) {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in 0..80 {
        for x in -14..100 {
            if in_crown(x, y, 2) {
                layer.set(x, y, CROWN.edge);
            }
        }
    }
    let mut clumps = Vec::new();
    for gy in (-4..80).step_by(5) {
        for gx in (-14..104).step_by(6) {
            let x = gx + (noise(gx, gy, 821) % 7) as i32 - 3 + if gy % 10 == 0 { 3 } else { 0 };
            let y = gy + (noise(gx, gy, 822) % 5) as i32 - 2;
            if in_crown(x, y, 3) {
                // Bigger clumps up in the dome, smaller ones out at the edges.
                let radius = if in_crown(x, y, 12) { 5 } else { 4 } + (noise(x, y, 823) % 3) as i32;
                clumps.push((x, y, radius));
            }
        }
    }
    clumps.sort_by_key(|&(x, y, _)| (y, x));
    let mut front = Vec::new();
    for &(x, y, radius) in &clumps {
        let tones = clump_tones(x, y);
        clump(&mut layer, (x, y, radius), tones);
        if y > 50 && !noise(x, y, 826).is_multiple_of(3) {
            front.push((x, y, radius, tones));
        }
    }
    // The limbs, seen among the lowest leaves, some of which hang in front of them.
    limbs(&mut layer);
    for (x, y, radius, tones) in front {
        if !(38..66).contains(&x) || noise(x, y, 827).is_multiple_of(2) {
            clump(&mut layer, (x, y, radius - 1), tones);
        }
    }
    // One outline round the whole crown, in its own darkest green.
    let mut edges = Vec::new();
    for y in -2..84 {
        for x in -16..106 {
            if layer.get(x, y).a == 0 {
                continue;
            }
            let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| layer.get(x + dx, y + dy).a == 0);
            if open && x > 0 && y > 0 && !is_bark(layer.get(x, y)) {
                edges.push((x, y));
            }
        }
    }
    for (x, y) in edges {
        layer.set(x, y, CROWN.edge);
    }
    blit(scene, &layer, 0, 0);
}

/// Whether a colour is one of the limbs' shaded bark tones, which keep their own outline.
fn is_bark(color: Rgba) -> bool {
    color.r > color.g
}

/// The tones for a clump of leaves at a point: lit by where it sits on its own mass, and on the
/// tree as a whole.
fn clump_tones(x: i32, y: i32) -> ([Rgba; 3], Option<Rgba>) {
    let (mx, my, radius) = CROWN_MASSES
        .iter()
        .copied()
        .filter(|&(cx, cy, r)| (x - cx).pow(2) + (y - cy).pow(2) <= r * r)
        .min_by_key(|&(_, _, r)| r)
        .unwrap_or(CROWN_MASSES[0]);
    let local = 0.5 - ((x - mx) + (y - my)) as f32 / (3.0 * radius as f32);
    let whole = 1.0 - (x as f32 / 110.0) * 0.45 - (y as f32 / 75.0) * 0.55;
    let lit = local * 0.45 + whole * 0.55;
    if lit > 0.6 {
        ([CROWN.shadow, CROWN.base, CROWN.light], Some(CROWN.shine))
    } else if lit > 0.42 {
        ([CROWN.shadow, CROWN.base, CROWN.light], None)
    } else if lit > 0.26 {
        ([CROWN.edge, CROWN.shadow, CROWN.base], None)
    } else {
        (
            [CROWN.edge, mix(CROWN.edge, CROWN.shadow, 0.5), CROWN.shadow],
            None,
        )
    }
}

/// A clump of leaves: a dark underside, its body, and a lit crescent on its upper left.
fn clump(
    layer: &mut Canvas,
    (cx, cy, radius): (i32, i32, i32),
    ([dark, mid, bright], shine): ([Rgba; 3], Option<Rgba>),
) {
    ellipse(layer, cx + 1, cy + 1, radius, radius - 1, dark);
    ellipse(layer, cx, cy, radius - 1, radius - 2, mid);
    for y in cy - radius..=cy + radius {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = (x - cx, y - cy);
            let (rx, ry) = (radius - 1, (radius - 2).max(1));
            if dx * dx * ry * ry + dy * dy * rx * rx > rx * rx * ry * ry {
                continue;
            }
            // Ragged, leafy edges to the light, not a smooth arc.
            let ragged = (noise(x, y, 824) % 3) as i32;
            if dx + dy < -radius / 2 - ragged + 1 {
                layer.set(x, y, bright);
            } else if dx + dy > radius / 2 + ragged && noise(x, y, 825).is_multiple_of(3) {
                layer.set(x, y, dark);
            }
        }
    }
    if let Some(shine) = shine {
        layer.set(cx - radius / 2, cy - radius / 2, shine);
        layer.set(cx - radius / 2 + 1, cy - radius / 2 - 1, shine);
    }
}

/// The great limbs, forking from the top of the trunk up into the leaves, in the crown's shade.
fn limbs(layer: &mut Canvas) {
    let (foot_x, _) = trunk_foot();
    let shaded = |color| mix(color, BARK.edge, 0.3);
    for (from, to, thick) in [
        ((foot_x - 4, 68), (foot_x - 24, 48), 2),
        ((foot_x + 3, 67), (foot_x + 22, 50), 2),
        ((foot_x, 66), (foot_x - 1, 46), 1),
        ((foot_x - 17, 55), (foot_x - 28, 50), 1),
        ((foot_x + 14, 57), (foot_x + 21, 53), 1),
    ] {
        for offset in -thick..=thick + 1 {
            let color = match offset {
                o if o == -thick => BARK.light,
                o if o == thick + 1 => BARK.edge,
                o if o > 0 => BARK.shadow,
                _ => BARK.base,
            };
            line(
                layer,
                (from.0 + offset, from.1),
                (to.0 + offset, to.1),
                shaded(color),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(backdrop().pixels().iter().all(|pixel| pixel.a == 255));
    }

    #[test]
    fn the_foreground_keeps_off_where_pieces_stand() {
        let scene = foreground();
        for (x, y) in SPOTS {
            assert_eq!(scene.get(x as i32, y as i32).a, 0);
        }
    }

    #[test]
    fn the_old_tree_leaves_the_nearest_place_room() {
        let mut leaves = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        crown(&mut leaves);
        let (left, top) = (SPOTS[0].0 as i32 - 24, SPOTS[0].1 as i32 - 56);
        for y in top.max(70)..SPOTS[0].1 as i32 {
            for x in left..left + 48 {
                assert_eq!(leaves.get(x, y).a, 0, "the crown reaches ({x}, {y})");
            }
        }
    }
}
