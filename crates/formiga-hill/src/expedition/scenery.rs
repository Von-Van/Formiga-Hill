//! The map of the Woods, painted, and the picture of the picnic pinned over it.
//!
//! The map is a sheet of parchment lying on a table, drawn in inks and washes: the woods as
//! little trees, each shaded from the upper left and outlined in a deeper green, the meadow in a
//! pale wash, the stream and the pool in blue, the crag along the top hatched in grey. Each place
//! has its picture where the party stands at it, and the paths are inked between them, the ways
//! only some company takes in a finer, redder hand. Where nobody has been, the Woods further in
//! are lost in mist. Every place and path is painted where `map` puts it.

use super::map::{
    EDGE, FAR_FALLS, GLADE, HEDGEROW, LOG, MEADOW, OLD_TRACK, Opener, PATHS, PLACES, POOL,
    SIGNPOST, Way,
};
use crate::font::{GLYPH_WIDTH, draw_text};
use crate::paint::{
    Ramp, chance, ellipse, hline, line, mix, noise, polygon, put, rgb, rgba, vline,
};
use crate::playground::distance;
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

const PARCHMENT: Ramp = Ramp::new(0x8a7452, 0xb6a07a, 0xd8c69c, 0xe6d8b2, 0xf2e8cc);
const TABLE: Ramp = Ramp::new(0x241812, 0x34241a, 0x4a3424, 0x604630, 0x7a5c40);
const TREE: Ramp = Ramp::new(0x2c4628, 0x426036, 0x5a7c46, 0x76985a, 0x98b676);
const PINE: Ramp = Ramp::new(0x223a26, 0x324e34, 0x446642, 0x5c8052, 0x7a9c6a);
const WATER: Ramp = Ramp::new(0x3a6474, 0x56828e, 0x76a2aa, 0x98c0c2, 0xc0dad8);
const ROCK: Ramp = Ramp::new(0x5e5446, 0x7e7262, 0x9e927e, 0xbab09a, 0xd4cab4);
const WOOD: Ramp = Ramp::new(0x4a3020, 0x6a4a30, 0x8a6644, 0xa6825a, 0xc0a074);
const HEDGE: Ramp = Ramp::new(0x284224, 0x3a5a30, 0x50743e, 0x6a9050, 0x8aac68);
/// The inks: sepia for lettering and the open paths, a redder hand for the ways only some
/// company takes, neither of them black.
const INK: Rgba = rgb(0x5a3c28);
const PATH_INK: Rgba = rgb(0x7a5234);
const PATH_SHADE: Rgba = rgba(0x4a2e1c, 120);
const SHORTCUT_INK: Rgba = rgb(0x8a2e1e);
const SHORTCUT_SHADE: Rgba = rgba(0x4a1810, 150);
const FIELD_WASH: Rgba = rgb(0xc4c88a);
const LANE: Ramp = Ramp::new(0x8a6a48, 0xa88a62, 0xc4a87e, 0xd6be96, 0xe6d2ae);

/// The parchment's edge, inside the table's border.
const MARGIN: i32 = 3;
/// The pond the pool is, on the map: its middle and reach.
const POND: (i32, i32, i32, i32) = (300, 166, 20, 11);
/// The Far Falls on the map: where the water comes over, and the plunge pool at its foot.
const FALLS_TOP: (i32, i32) = (326, 36);
const PLUNGE: (i32, i32, i32, i32) = (326, 72, 10, 4);
/// The stream, from the plunge pool down into the pond, and out of it off the map.
const STREAM: [(f32, f32); 6] = [
    (332.0, 74.0),
    (338.0, 100.0),
    (332.0, 124.0),
    (318.0, 146.0),
    (306.0, 158.0),
    (302.0, 162.0),
];
const OUTFLOW: [(f32, f32); 4] = [
    (312.0, 174.0),
    (318.0, 192.0),
    (322.0, 206.0),
    (324.0, 220.0),
];
/// The meadow's open ground: its middle and reach.
const FIELD: (f32, f32, f32, f32) = (64.0, 116.0, 38.0, 24.0);
/// The hedge, along the lane's far side, and the lane under it.
const HEDGE_ROW: (i32, i32, i32) = (40, 182, 188);
const LANE_ROW: i32 = 198;
/// The title, top left, and the compass, bottom right.
const CARTOUCHE: (i32, i32, i32, i32) = (8, 7, 84, 18);
const COMPASS: (i32, i32) = (360, 196);
/// Where the crag runs along the top right: its foot at each column.
fn crag_foot(x: i32) -> i32 {
    if x < 140 {
        return 0;
    }
    let t = (x - 140) as f32 / (WIDTH - 140) as f32;
    let wobble = (noise(x.div_euclid(6), 0, 2001) % 7) as i32 - 3;
    (14.0 + 34.0 * t.sqrt()).round() as i32 + wobble
}

/// What the map shows: which places are in sight, and which paths are drawn on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub places: [bool; PLACES.len()],
    pub paths: [bool; PATHS.len()],
}

/// The map, filling every pixel.
pub fn map(shown: &Shown) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    table(&mut scene);
    parchment(&mut scene);
    field(&mut scene);
    crag(&mut scene);
    water(&mut scene, shown.places[FAR_FALLS]);
    lane_and_hedge(&mut scene);
    trees(&mut scene, shown);
    for (index, path) in PATHS.iter().enumerate() {
        if shown.paths[index] {
            ink_path(&mut scene, path, shown);
        }
    }
    edge(&mut scene);
    hedgerow(&mut scene);
    meadow(&mut scene);
    glade(&mut scene);
    log(&mut scene);
    signpost(&mut scene);
    pool(&mut scene);
    old_track(&mut scene);
    if shown.places[FAR_FALLS] {
        falls(&mut scene);
    } else {
        mist(&mut scene, PLACES[FAR_FALLS].at);
    }
    names(&mut scene, shown);
    cartouche(&mut scene);
    compass(&mut scene);
    folds(&mut scene);
    scene
}

// ---------------------------------------------------------------------------------------------
// Tools
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

/// How far a point is from a line drawn through `points`.
fn from_line(at: (f32, f32), points: &[(f32, f32)]) -> f32 {
    points
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let length = dx * dx + dy * dy;
            let t = if length > 0.0 {
                (((at.0 - a.0) * dx + (at.1 - a.1) * dy) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            distance(at, (a.0 + dx * t, a.1 + dy * t))
        })
        .fold(f32::MAX, f32::min)
}

/// Every pixel along a line through `points`, one step at a time.
fn traced(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let steps = distance(a, b).ceil().max(1.0) as i32;
        for step in 0..steps {
            let t = step as f32 / steps as f32;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    if let Some(last) = points.last() {
        out.push(*last);
    }
    out
}

/// Whether a point is on the parchment rather than the table.
fn on_paper(x: i32, y: i32) -> bool {
    let ragged = |along: i32, salt: u32| (noise(along.div_euclid(3), 0, salt) % 3) as i32;
    x >= MARGIN + ragged(y, 2002)
        && x < WIDTH - MARGIN - ragged(y, 2003)
        && y >= MARGIN + ragged(x, 2004)
        && y < HEIGHT - MARGIN - ragged(x, 2005)
}

/// A wash of colour laid over the parchment, as watercolour is: `amount` of it, mottled.
fn wash(scene: &mut Canvas, x: i32, y: i32, color: Rgba, amount: f32) {
    let mottle = 0.8 + 0.4 * patches(x, y, (5, 4), 2006);
    let pixel = scene.get(x, y);
    scene.set(x, y, mix(pixel, color, (amount * mottle).min(1.0)));
}

// ---------------------------------------------------------------------------------------------
// The table and the parchment
// ---------------------------------------------------------------------------------------------

/// The table the map lies on: dark wood in long planks, its grain running across.
fn table(scene: &mut Canvas) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let plank = y.div_euclid(9);
            let grain = noise(x.div_euclid(5), y + plank * 3, 2010) % 6;
            let mut level = match grain {
                0 => 1,
                1 => 3,
                _ => 2,
            };
            if y.rem_euclid(9) == 0 {
                level = 0;
            }
            let color = match level {
                0 => TABLE.edge,
                1 => TABLE.shadow,
                3 => TABLE.light,
                _ => TABLE.base,
            };
            scene.set(x, y, color);
        }
    }
}

/// The sheet: warm cream, mottled and fibrous, browning towards its ragged edges, which cast a
/// shadow on the table down and to the right.
fn parchment(scene: &mut Canvas) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if on_paper(x - 2, y - 2) && !on_paper(x, y) {
                let pixel = scene.get(x, y);
                scene.set(x, y, mix(pixel, rgb(0x100a06), 0.5));
            }
        }
    }
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if !on_paper(x, y) {
                continue;
            }
            let edge = x.min(y).min(WIDTH - 1 - x).min(HEIGHT - 1 - y) as f32;
            let browned = (1.0 - edge / 26.0).clamp(0.0, 1.0);
            let mottle = patches(x, y, (13, 9), 2011);
            let mut color = mix(PARCHMENT.light, PARCHMENT.base, mottle * 0.8);
            color = mix(color, PARCHMENT.shadow, browned * browned * 0.7);
            // Fibres in the paper, and the odd fleck.
            if chance(x.div_euclid(3), y, 2012, 18) {
                color = mix(color, PARCHMENT.shine, 0.6);
            } else if chance(x, y, 2013, 6) {
                color = mix(color, PARCHMENT.shadow, 0.6);
            }
            let rim = !on_paper(x - 1, y) || !on_paper(x, y - 1);
            let under = !on_paper(x + 1, y) || !on_paper(x, y + 1);
            if rim {
                color = PARCHMENT.shine;
            } else if under {
                color = PARCHMENT.edge;
            }
            scene.set(x, y, color);
        }
    }
    // A tea ring and a stain or two: the map has been used.
    for (cx, cy, r) in [(352, 30, 9), (60, 60, 5)] {
        for step in 0..(r * 8) {
            let angle = step as f32 / (r * 8) as f32 * TAU;
            let (x, y) = (
                cx + (angle.cos() * r as f32).round() as i32,
                cy + (angle.sin() * r as f32 * 0.9).round() as i32,
            );
            if !chance(step, r, 2014, 70) {
                let pixel = scene.get(x, y);
                scene.set(x, y, mix(pixel, PARCHMENT.edge, 0.35));
            }
        }
    }
}

/// The folds the map has been kept in: a crease across and one down, each lit on one side.
fn folds(scene: &mut Canvas) {
    let across = HEIGHT / 2;
    let down = WIDTH / 2;
    for x in 0..WIDTH {
        if on_paper(x, across) {
            let pixel = scene.get(x, across);
            scene.set(x, across, mix(pixel, PARCHMENT.edge, 0.22));
            let below = scene.get(x, across + 1);
            scene.set(x, across + 1, mix(below, PARCHMENT.shine, 0.3));
        }
    }
    for y in 0..HEIGHT {
        if on_paper(down, y) {
            let pixel = scene.get(down, y);
            scene.set(down, y, mix(pixel, PARCHMENT.edge, 0.22));
            let beside = scene.get(down + 1, y);
            scene.set(down + 1, y, mix(beside, PARCHMENT.shine, 0.3));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The land
// ---------------------------------------------------------------------------------------------

/// The meadow's open ground in a pale wash, its edge ragged.
fn field(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = FIELD;
    for y in (cy - ry - 4.0) as i32..=(cy + ry + 4.0) as i32 {
        for x in (cx - rx - 4.0) as i32..=(cx + rx + 4.0) as i32 {
            let (u, v) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
            let r = u * u + v * v + (patches(x, y, (5, 5), 2020) - 0.5) * 0.4;
            if r < 1.0 && on_paper(x, y) {
                wash(scene, x, y, FIELD_WASH, 0.55);
            }
        }
    }
}

/// The crag along the top right: grey rock hatched in strokes, lighter on its upper faces, its
/// foot ragged against the woods.
fn crag(scene: &mut Canvas) {
    for x in 140..WIDTH {
        let foot = crag_foot(x);
        for y in 0..foot {
            if !on_paper(x, y) {
                continue;
            }
            let depth = (foot - y) as f32 / foot.max(1) as f32;
            let hatch = (x + y).rem_euclid(5) == 0 && chance(x, y, 2021, 180);
            let color = if hatch {
                ROCK.shadow
            } else if depth < 0.25 {
                ROCK.base
            } else {
                ROCK.light
            };
            wash(scene, x, y, color, 0.75);
        }
        // Its foot, inked.
        put(scene, x, foot, ROCK.edge);
        if chance(x, foot, 2022, 120) {
            put(scene, x, foot - 1, ROCK.shadow);
        }
    }
    // Boulders along the foot of the crag.
    for index in 0..10 {
        let x = 150 + pick(index, 0, 2023, WIDTH - 170);
        let y = crag_foot(x) + 1;
        ellipse(scene, x, y, 3, 2, ROCK.edge);
        ellipse(scene, x, y, 2, 1, ROCK.light);
        put(scene, x - 1, y - 1, ROCK.shine);
    }
}

/// The stream, the pond and, if they are in sight, the plunge pool under the falls: blue
/// washes with deeper edges, and ripple strokes on the water.
fn water(scene: &mut Canvas, falls_seen: bool) {
    let mut course: Vec<(f32, f32)> = Vec::new();
    if falls_seen {
        course.push((PLUNGE.0 as f32, PLUNGE.1 as f32));
    } else {
        // Out of the mist.
        course.push((340.0, 60.0));
    }
    course.extend_from_slice(&STREAM);
    for (points, wide) in [(course.as_slice(), 3.0), (&OUTFLOW[..], 2.6)] {
        for (x, y) in traced(points) {
            let (x, y) = (x as i32, y as i32);
            for dy in -4..=4 {
                for dx in -4..=4 {
                    let at = (x as f32 + dx as f32, y as f32 + dy as f32);
                    let off = from_line(at, points);
                    let (px, py) = (x + dx, y + dy);
                    if off <= wide && on_paper(px, py) {
                        let color = if off > wide - 1.0 {
                            WATER.shadow
                        } else if dx < 0 {
                            WATER.light
                        } else {
                            WATER.base
                        };
                        scene.set(px, py, color);
                    }
                }
            }
        }
    }
    // Ripple strokes along it.
    for (index, (x, y)) in traced(&STREAM).into_iter().enumerate().step_by(7) {
        put(scene, x as i32 - 1, y as i32, WATER.shine);
        put(scene, x as i32, y as i32, WATER.shine);
        let _ = index;
    }
}

/// The lane in from the Hill along the bottom, and the hedge along its far side, with the gate.
fn lane_and_hedge(scene: &mut Canvas) {
    for x in 0..190 {
        let wobble = (patches(x, 0, (13, 1), 2030) * 3.0) as i32;
        let top = LANE_ROW - 3 + wobble;
        let fade = ((x - 150) as f32 / 40.0).clamp(0.0, 1.0);
        for y in top..top + 6 {
            if !on_paper(x, y) {
                continue;
            }
            let color = if y == top {
                LANE.shadow
            } else if y == top + 5 {
                LANE.edge
            } else if chance(x, y, 2031, 30) {
                LANE.light
            } else {
                LANE.base
            };
            let pixel = scene.get(x, y);
            scene.set(x, y, mix(color, pixel, fade));
        }
    }
    // The hedge: a row of bushes along the lane's far side, berries in it here and there.
    let (from, top, foot) = HEDGE_ROW;
    let mut x = from;
    let mut index = 0;
    while x < 176 {
        // The gap a little one squeezes through, kept open.
        if (110..116).contains(&x) {
            x += 6;
            continue;
        }
        let radius = 3 + pick(index, 0, 2032, 2);
        bush(
            scene,
            (x, foot - radius + 1),
            radius,
            HEDGE,
            2033 + index as u32,
        );
        if pick(index, 1, 2032, 3) == 0 {
            put(scene, x + 1, foot - radius, rgb(0x6a2a48));
        }
        x += radius + 2;
        index += 1;
    }
    let _ = top;
}

/// A rounded bush or tree crown, shaded from the upper left, outlined in its own darkest green.
fn bush(scene: &mut Canvas, (cx, cy): (i32, i32), radius: i32, ramp: Ramp, salt: u32) {
    let ry = radius;
    for y in cy - ry - 1..=cy + ry + 1 {
        for x in cx - radius - 1..=cx + radius + 1 {
            let (u, v) = (
                (x - cx) as f32 / (radius as f32 + 0.4),
                (y - cy) as f32 / (ry as f32 + 0.4),
            );
            let lumpy = (noise(x, y, salt) % 100) as f32 / 400.0;
            let r = u * u + v * v - lumpy;
            if r > 1.0 {
                continue;
            }
            let lit = u + v;
            let color = if r > 0.72 {
                ramp.edge
            } else if lit < -0.75 {
                ramp.shine
            } else if lit < -0.15 {
                ramp.light
            } else if lit < 0.6 {
                ramp.base
            } else {
                ramp.shadow
            };
            put(scene, x, y, color);
        }
    }
}

/// A little conifer: a stack of shaded tiers on a short trunk.
fn pine(scene: &mut Canvas, (cx, foot): (i32, i32), tall: i32) {
    vline(scene, cx, foot - 2, 3, WOOD.shadow);
    for row in 0..tall {
        let y = foot - 2 - row;
        let half = ((tall - row) as f32 * 0.45).round() as i32 + i32::from(row % 3 == 0);
        for dx in -half..=half {
            let color = if dx == -half || dx == half {
                PINE.edge
            } else if dx < 0 {
                PINE.light
            } else if dx == 0 {
                PINE.base
            } else {
                PINE.shadow
            };
            put(scene, cx + dx, y, color);
        }
    }
    put(scene, cx, foot - 2 - tall, PINE.edge);
}

/// The woods: little trees everywhere the paths, the places, the water and the open ground
/// leave room, broad-leaved near the edge and darker pines deeper in.
fn trees(scene: &mut Canvas, shown: &Shown) {
    let mut planted: Vec<(i32, i32)> = Vec::new();
    for gy in (20..HEIGHT - 8).step_by(9) {
        for gx in (4..WIDTH - 4).step_by(10) {
            let row = gy / 9;
            let x = gx + pick(gx, gy, 2040, 7) - 3 + if row % 2 == 0 { 5 } else { 0 };
            let y = gy + pick(gx, gy, 2041, 5) - 2;
            let at = (x as f32, y as f32);
            if !on_paper(x - 6, y - 8) || !on_paper(x + 6, y + 2) {
                continue;
            }
            if y < crag_foot(x) + 6 {
                continue;
            }
            let near_place = PLACES
                .iter()
                .any(|place| distance(at, (place.at.0, place.at.1 - 10.0)) < 24.0);
            let near_path = PATHS.iter().enumerate().any(|(index, path)| {
                shown.paths[index] && from_line(at, &path.walk_from(path.ends.0)) < 7.0
            });
            let (fx, fy, frx, fry) = FIELD;
            let in_field =
                ((at.0 - fx) / (frx + 6.0)).powi(2) + ((at.1 - fy) / (fry + 6.0)).powi(2) < 1.0;
            let wet = from_line(at, &STREAM) < 7.0
                || from_line(at, &OUTFLOW) < 6.0
                || distance(at, (POND.0 as f32, POND.1 as f32)) < 28.0;
            let lane = y > HEDGE_ROW.1 - 6 && x < 190;
            let (cl, ct, cw, ch) = CARTOUCHE;
            let titled = x > cl - 6 && x < cl + cw + 6 && y > ct - 4 && y < ct + ch + 10;
            let compass = distance(at, (COMPASS.0 as f32, COMPASS.1 as f32)) < 22.0;
            if near_place || near_path || in_field || wet || lane || titled || compass {
                continue;
            }
            planted.push((x, y));
        }
    }
    // Little clearings among them, here and there.
    planted.retain(|(x, y)| patches(*x, *y, (30, 22), 2044) > 0.16);
    // The woods' own wash under them, deeper further in.
    for &(x, y) in &planted {
        let deep = x as f32 / WIDTH as f32 * 0.5 + (1.0 - y as f32 / HEIGHT as f32) * 0.3;
        let wood = mix(rgb(0xb8c48e), rgb(0x8a9a6a), deep);
        for dy in -9..=3 {
            for dx in -6..=6 {
                if dx * dx + dy * dy * 2 <= 40 && on_paper(x + dx, y + dy) {
                    let pixel = scene.get(x + dx, y + dy);
                    scene.set(x + dx, y + dy, mix(pixel, wood, 0.08));
                }
            }
        }
    }
    // Back to front, so nearer trees overlap those behind.
    planted.sort_by_key(|(_, y)| *y);
    for (index, (x, y)) in planted.into_iter().enumerate() {
        let i = index as i32;
        let deep = x as f32 / WIDTH as f32 * 0.6 + (1.0 - y as f32 / HEIGHT as f32) * 0.4;
        match pick(i, 3, 2042, 12) {
            // Now and then a bush rather than a tree.
            0 => bush(scene, (x, y - 2), 2, HEDGE, 2045 + index as u32),
            _ if pick(i, 0, 2042, 10) as f32 / 10.0 < deep - 0.35 => {
                pine(scene, (x, y), 6 + pick(i, 1, 2042, 5));
            }
            _ => {
                // A broad-leaved tree, now and then a big old one.
                let radius = if pick(i, 4, 2042, 9) == 0 {
                    5
                } else {
                    3 + pick(i, 2, 2042, 2)
                };
                vline(scene, x, y - 1, 3, WOOD.shadow);
                put(scene, x - 1, y + 1, WOOD.edge);
                put(scene, x + 1, y + 1, rgba(0x2a1c10, 60));
                bush(
                    scene,
                    (x, y - radius - 2),
                    radius,
                    TREE,
                    2043 + index as u32,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The paths
// ---------------------------------------------------------------------------------------------

/// A path inked along its bends: an open one in dashes, a way only some company takes dotted
/// in a redder hand with a mark of what it is. A path into the mist fades as it goes.
fn ink_path(scene: &mut Canvas, path: &super::map::Path, shown: &Shown) {
    let (a, b) = path.ends;
    let points = path.walk_from(a);
    let length: f32 = points.windows(2).map(|p| distance(p[0], p[1])).sum();
    let into_mist = !shown.places[a] || !shown.places[b];
    let toward_a = !shown.places[a];
    let shortcut = matches!(path.way, Way::Shortcut(..));
    let mut walked = 0.0;
    for (x, y) in traced(&points) {
        walked += 1.0;
        let along = walked / length.max(1.0);
        // Faded into the mist at the hidden end.
        let fade = if into_mist {
            let toward = if toward_a { 1.0 - along } else { along };
            (1.0 - (toward - 0.55).max(0.0) * 2.4).clamp(0.0, 1.0)
        } else {
            1.0
        };
        if fade <= 0.05 {
            continue;
        }
        let step = walked as i32;
        let (on, ink) = if shortcut {
            (step % 4 < 2, SHORTCUT_INK)
        } else {
            (step % 5 < 3, PATH_INK)
        };
        if !on {
            continue;
        }
        let alpha = (fade * 255.0) as u8;
        put(
            scene,
            x as i32,
            y as i32,
            Rgba::new(ink.r, ink.g, ink.b, alpha),
        );
        let shade = if shortcut { SHORTCUT_SHADE } else { PATH_SHADE };
        put(
            scene,
            x as i32,
            y as i32 + 1,
            Rgba::new(shade.r, shade.g, shade.b, alpha / 2),
        );
    }
    if let Way::Shortcut(_, opener) = path.way {
        let mid = points[points.len() / 2];
        mark(scene, opener, (mid.0 as i32, mid.1 as i32), &points);
    }
}

/// What a way only some company takes is marked with, at its middle: paw prints along the deer
/// track, stepping stones up the stream, a zigzag up the steep way, the old way's milestones, and
/// a gap in the hedge.
fn mark(scene: &mut Canvas, opener: Opener, (x, y): (i32, i32), points: &[(f32, f32)]) {
    match opener {
        Opener::Explorer => {
            for (dx, dy) in [(-6, 1), (-2, -1), (2, 1), (6, -1)] {
                put(scene, x + dx, y + dy - 3, SHORTCUT_INK);
                put(scene, x + dx + 1, y + dy - 3, SHORTCUT_INK);
                put(scene, x + dx, y + dy - 4, rgba(0x9a4a32, 140));
            }
        }
        Opener::ClosePair => {
            // Stones across the stream, stepped from one to the next up its bed.
            for (sx, sy) in traced(&STREAM)
                .into_iter()
                .filter(|(_, sy)| (100.0..150.0).contains(sy))
                .step_by(7)
            {
                let (sx, sy) = (sx as i32, sy as i32);
                ellipse(scene, sx, sy + 1, 3, 2, rgba(0x1a2a30, 90));
                ellipse(scene, sx, sy, 3, 2, ROCK.edge);
                ellipse(scene, sx, sy, 2, 1, ROCK.base);
                hline(scene, sx - 1, sy - 1, 2, ROCK.light);
                put(scene, sx - 1, sy - 1, ROCK.shine);
            }
        }
        Opener::Bold => {
            for (index, (sx, sy)) in traced(points).into_iter().enumerate().step_by(4) {
                let side = if index % 8 == 0 { -2 } else { 2 };
                put(scene, sx as i32 + side, sy as i32 - 1, SHORTCUT_INK);
            }
        }
        Opener::Reader => {
            for (sx, sy) in traced(points).into_iter().step_by(10).skip(1) {
                let (sx, sy) = (sx as i32, sy as i32 - 3);
                vline(scene, sx, sy, 3, ROCK.edge);
                put(scene, sx - 1, sy, ROCK.light);
            }
        }
        Opener::LittleOne => {
            ellipse(scene, x, y, 2, 2, rgba(0x9a4a32, 90));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The places
// ---------------------------------------------------------------------------------------------

/// The edge of the Woods: where the lane comes in from the Hill, which shows in the margin, with
/// a stile in the hedge and the first trees.
fn edge(scene: &mut Canvas) {
    let (x, y) = (PLACES[EDGE].at.0 as i32, PLACES[EDGE].at.1 as i32);
    // The Hill, small in the margin, with the Hilltop's old tree.
    for py in 150..186 {
        for px in 4..26 {
            let (u, v) = ((px as f32 - 8.0) / 20.0, (py as f32 - 186.0) / 30.0);
            if u * u + v * v <= 1.0 && on_paper(px, py) {
                let lit = u * 0.6 + v * 0.4;
                let color = if u * u + v * v > 0.86 {
                    HEDGE.edge
                } else if lit < -0.45 {
                    HEDGE.light
                } else {
                    HEDGE.base
                };
                scene.set(px, py, color);
            }
        }
    }
    vline(scene, 9, 154, 5, WOOD.shadow);
    bush(scene, (9, 151), 3, TREE, 2050);
    // The stile: two posts and a bar.
    for px in [x - 7, x + 7] {
        vline(scene, px, y - 12, 8, WOOD.edge);
        vline(scene, px - 1, y - 12, 8, WOOD.light);
    }
    hline(scene, x - 7, y - 9, 15, WOOD.base);
    hline(scene, x - 7, y - 8, 15, WOOD.edge);
}

/// The hedgerow: the gate in the hedge, and blackberries along it.
fn hedgerow(scene: &mut Canvas) {
    let (x, y) = (PLACES[HEDGEROW].at.0 as i32, PLACES[HEDGEROW].at.1 as i32);
    let top = HEDGE_ROW.2 - 9;
    // A field gate in the hedge, a little way along from where the party stands.
    let gx = x + 18;
    for px in [gx, gx + 12] {
        vline(scene, px, top, 10, WOOD.edge);
        vline(scene, px + 1, top, 10, WOOD.light);
    }
    for row in [top + 2, top + 5, top + 8] {
        hline(scene, gx + 1, row, 11, WOOD.base);
    }
    line(scene, (gx + 1, top + 8), (gx + 11, top + 2), WOOD.shadow);
    for (dx, dy) in [(-10, -4), (-6, -6), (-14, -5)] {
        put(scene, x + dx, y + dy - 12, rgb(0x3a1a3a));
        put(scene, x + dx + 1, y + dy - 12, rgb(0x6a2a5a));
    }
}

/// The meadow: grass and flowers in its wash, and a butterfly over it.
fn meadow(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = FIELD;
    for index in 0..70 {
        let angle = pick(index, 0, 2060, 360) as f32 / 360.0 * TAU;
        let reach = (pick(index, 1, 2060, 100) as f32 / 100.0).sqrt();
        let (x, y) = (
            (cx + angle.cos() * rx * reach * 0.9) as i32,
            (cy + angle.sin() * ry * reach * 0.85) as i32,
        );
        if index % 3 == 0 {
            let flower = [0xe0605a, 0xfbf6ee, 0xf5d25e, 0xb79be0][index as usize % 4];
            put(scene, x, y, rgb(flower));
            put(scene, x, y + 1, TREE.shadow);
        } else {
            put(scene, x, y, TREE.light);
            put(scene, x + 1, y - 1, TREE.base);
        }
    }
    // An old stump by the place, and a butterfly.
    let (x, y) = (
        PLACES[MEADOW].at.0 as i32 + 14,
        PLACES[MEADOW].at.1 as i32 - 14,
    );
    ellipse(scene, x, y, 4, 2, WOOD.edge);
    ellipse(scene, x, y - 1, 3, 1, WOOD.light);
    for (dx, dy, color) in [
        (-18, -14, 0xf5d25e),
        (-17, -15, 0xf5d25e),
        (-16, -14, 0x5a3c28),
    ] {
        put(scene, x + dx, y + dy, rgb(color));
    }
}

/// The glade: a clearing under the great oak, with the hollow log lying in it.
fn glade(scene: &mut Canvas) {
    let (x, y) = (PLACES[GLADE].at.0 as i32, PLACES[GLADE].at.1 as i32);
    for py in y - 22..y - 2 {
        for px in x - 22..x + 22 {
            let (u, v) = ((px - x) as f32 / 22.0, (py - (y - 12)) as f32 / 10.0);
            if u * u + v * v < 1.0 {
                wash(scene, px, py, rgb(0xd8d4a0), 0.5);
            }
        }
    }
    // The oak.
    let (ox, oy) = (x - 12, y - 18);
    for dx in 0..3 {
        vline(
            scene,
            ox + dx,
            oy,
            10,
            if dx == 0 { WOOD.light } else { WOOD.shadow },
        );
    }
    bush(scene, (ox + 1, oy - 6), 8, TREE, 2070);
    bush(scene, (ox - 5, oy - 2), 4, TREE, 2071);
    bush(scene, (ox + 7, oy - 2), 5, TREE, 2072);
    // The hollow log.
    let (lx, ly) = (x + 9, y - 8);
    for dx in 0..12 {
        put(scene, lx + dx, ly - 2, WOOD.light);
        put(scene, lx + dx, ly - 1, WOOD.base);
        put(scene, lx + dx, ly, WOOD.shadow);
        put(scene, lx + dx, ly + 1, WOOD.edge);
    }
    ellipse(scene, lx, ly - 1, 1, 2, WOOD.edge);
    put(scene, lx, ly - 1, rgb(0x2a1a10));
}

/// The fallen log: a great trunk lying in its own little clearing, mushrooms on it.
fn log(scene: &mut Canvas) {
    let (x, y) = (PLACES[LOG].at.0 as i32, PLACES[LOG].at.1 as i32);
    for py in y - 20..y - 2 {
        for px in x - 20..x + 20 {
            let (u, v) = ((px - x) as f32 / 20.0, (py - (y - 11)) as f32 / 9.0);
            if u * u + v * v < 1.0 {
                wash(scene, px, py, rgb(0xd8d4a0), 0.45);
            }
        }
    }
    let (from, to, ly) = (x - 14, x + 13, y - 12);
    for px in from..to {
        let slope = (px - from) / 9;
        let top = ly - 2 - slope / 2;
        put(scene, px, top, WOOD.edge);
        put(scene, px, top + 1, WOOD.light);
        put(scene, px, top + 2, WOOD.base);
        put(scene, px, top + 3, WOOD.base);
        put(scene, px, top + 4, WOOD.shadow);
        put(scene, px, top + 5, WOOD.edge);
        if chance(px, top, 2080, 50) {
            put(scene, px, top + 1, TREE.light);
        }
    }
    ellipse(scene, from, ly + 1, 2, 3, WOOD.edge);
    ellipse(scene, from, ly + 1, 1, 2, WOOD.light);
    put(scene, from, ly + 1, WOOD.shadow);
    for (dx, color) in [(4, 0xc84a3a), (8, 0xd8c8a8)] {
        put(scene, from + dx, ly - 3, rgb(color));
        put(scene, from + dx + 1, ly - 3, rgb(color));
        put(scene, from + dx, ly - 2, rgb(0xe8dcc0));
    }
}

/// The old signpost at the fork: a weathered post with two boards on it, one pointing on further
/// in, and its shadow on the ground.
fn signpost(scene: &mut Canvas) {
    let (x, foot) = (
        PLACES[SIGNPOST].at.0 as i32,
        PLACES[SIGNPOST].at.1 as i32 - 6,
    );
    ellipse(scene, x + 2, foot, 4, 1, rgba(0x2a1c10, 70));
    // The post, lit on its left, with a cap.
    for y in foot - 16..foot {
        put(scene, x - 1, y, WOOD.edge);
        put(scene, x, y, WOOD.light);
        put(scene, x + 1, y, WOOD.shadow);
        put(scene, x + 2, y, WOOD.edge);
    }
    hline(scene, x - 1, foot - 17, 4, WOOD.edge);
    // Two boards, each with a point at the end it shows the way to.
    for &(top, from, to, right) in &[
        (foot - 15, x + 3, x + 13, true),
        (foot - 10, x - 11, x - 1, false),
    ] {
        for y in top..top + 4 {
            for px in from..to {
                let tip = if right { to - 1 - px } else { px - from };
                let middle = (top + 1..top + 3).contains(&y);
                if tip == 0 && !middle {
                    continue;
                }
                let color = if y == top || y == top + 3 || tip == 0 {
                    WOOD.edge
                } else if y == top + 1 {
                    WOOD.light
                } else {
                    WOOD.base
                };
                put(scene, px, y, color);
            }
        }
        // Lettering, worn to a few strokes.
        for dx in [3, 5, 6] {
            let lx = if right { from + dx } else { to - 1 - dx };
            put(scene, lx, top + 2, WOOD.shadow);
        }
    }
}

/// The pool: the pond, with lilies on it and a little fall where the stream comes in.
fn pool(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = POND;
    for y in cy - ry - 1..=cy + ry + 1 {
        for x in cx - rx - 1..=cx + rx + 1 {
            let (u, v) = (
                (x - cx) as f32 / (rx as f32 + 0.4),
                (y - cy) as f32 / (ry as f32 + 0.4),
            );
            let r = u * u + v * v;
            if r > 1.0 {
                continue;
            }
            let color = if r > 0.8 {
                WATER.edge
            } else if v < -0.4 {
                WATER.shadow
            } else if u + v < -0.5 {
                WATER.light
            } else {
                WATER.base
            };
            put(scene, x, y, color);
        }
    }
    for (dx, dy) in [(-10, 2), (-6, 4), (8, -2)] {
        ellipse(scene, cx + dx, cy + dy, 2, 1, TREE.base);
        put(scene, cx + dx - 1, cy + dy, TREE.light);
    }
    put(scene, cx - 6, cy + 3, rgb(0xf0c8d0));
    for dx in -1..=1 {
        put(scene, cx + 6 + dx, cy - ry - 1, WATER.shine);
    }
    // Ripples.
    for (dx, dy) in [(2, 3), (10, 5), (-2, -4)] {
        hline(scene, cx + dx, cy + dy, 3, WATER.shine);
    }
}

/// The old track: an overgrown cart track running off between the trees, worn into two ruts with
/// grass down the middle, the woodcutter's tumbledown hut beside it with its roof fallen in, the
/// old milestone, a run of tumbled wall, and the broken cart that never got any further.
fn old_track(scene: &mut Canvas) {
    let (x, y) = (PLACES[OLD_TRACK].at.0 as i32, PLACES[OLD_TRACK].at.1 as i32);
    // Its own open ground, in a wash.
    for py in y - 38..y - 2 {
        for px in x - 28..x + 30 {
            let (u, v) = ((px - x - 1) as f32 / 29.0, (py - (y - 20)) as f32 / 17.0);
            if u * u + v * v < 1.0 {
                wash(scene, px, py, rgb(0xd8cca0), 0.45);
            }
        }
    }
    // The track, from where the party stands away up between the trees, narrowing as it goes:
    // worn earth, a rut down each side of it, grass along the middle, and lost in the grass at
    // its far end.
    let (fx, fy) = (x as f32, y as f32);
    let way = [
        (fx - 8.0, fy - 5.0),
        (fx + 1.0, fy - 15.0),
        (fx + 12.0, fy - 25.0),
        (fx + 26.0, fy - 35.0),
    ];
    let total: f32 = way.windows(2).map(|pair| distance(pair[0], pair[1])).sum();
    let nearest = |at: (f32, f32)| {
        let (mut best, mut along, mut walked) = (f32::MAX, 0.0, 0.0);
        for pair in way.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let length = (dx * dx + dy * dy).sqrt();
            let t = (((at.0 - a.0) * dx + (at.1 - a.1) * dy) / (length * length)).clamp(0.0, 1.0);
            let off = distance(at, (a.0 + dx * t, a.1 + dy * t));
            if off < best {
                best = off;
                along = (walked + t * length) / total;
            }
            walked += length;
        }
        (best, along)
    };
    for py in y - 40..y {
        for px in x - 16..x + 32 {
            let (off, along) = nearest((px as f32 + 0.5, py as f32 + 0.5));
            let half = 5.2 - 2.8 * along;
            if off > half {
                continue;
            }
            let r = off / half;
            let color = if r > 0.86 {
                // The verge, grassing over.
                if chance(px, py, 2122, 90) {
                    TREE.light
                } else {
                    LANE.base
                }
            } else if (0.42..0.7).contains(&r) {
                LANE.shadow
            } else if r < 0.24 {
                if chance(px, py, 2120, 110) {
                    TREE.light
                } else {
                    TREE.base
                }
            } else if chance(px, py, 2121, 40) {
                LANE.base
            } else {
                LANE.light
            };
            let fade = ((1.0 - along) / 0.22).clamp(0.0, 1.0);
            let pixel = scene.get(px, py);
            scene.set(px, py, mix(pixel, color, fade));
        }
    }
    // The woodcutter's hut, on the left: plank walls, a dark doorway, and its shingled roof sagging
    // with a hole fallen in it, the rafters showing; lit from the upper left.
    stamp_rows(
        scene,
        (x - 27, y - 34),
        &[
            ".....ee.......",
            "....ellee.....",
            "...ellbb.e....",
            "..ellbb.r.e...",
            ".ellbbb...se..",
            "eeeeeeeeeeeeee",
            ".ellbsbsbsbse.",
            ".elbkkbsbsbse.",
            ".elbkkbsblbse.",
            ".elbkkbsbsbse.",
            ".eeeeeeeeeeee.",
        ],
        &[
            ('e', WOOD.edge),
            ('s', WOOD.shadow),
            ('b', WOOD.base),
            ('l', WOOD.light),
            ('r', WOOD.shadow),
            ('k', rgb(0x2a1a10)),
        ],
    );
    for (dx, dy, color) in [(5, 1, TREE.light), (4, 2, TREE.base), (6, 1, TREE.base)] {
        put(scene, x - 27 + dx, y - 34 + dy, color);
    }
    // The old milestone by the track, lichen on its rounded top.
    stamp_rows(
        scene,
        (x - 11, y - 19),
        &[".ee.", "elge", "elbe", "elbe", "eese"],
        &[
            ('e', ROCK.edge),
            ('s', ROCK.shadow),
            ('b', ROCK.base),
            ('l', ROCK.light),
            ('g', rgb(0xc4c87a)),
        ],
    );
    // A run of dry-stone wall along the right of the track: stones lit on top with dark joints,
    // and a stretch fallen in, its stones lying where they came down.
    let wall = traced(&[(fx + 9.0, fy - 4.0), (fx + 26.0, fy - 17.0)]);
    for (index, &(sx, sy)) in wall.iter().enumerate() {
        if (9..14).contains(&index) {
            continue;
        }
        let (sx, sy) = (sx as i32, sy as i32);
        put(scene, sx, sy - 2, ROCK.edge);
        put(scene, sx, sy - 1, ROCK.light);
        let joint = index % 3 == 0;
        put(scene, sx, sy, if joint { ROCK.edge } else { ROCK.base });
        put(scene, sx, sy + 1, ROCK.edge);
    }
    for (dx, dy) in [(16, -11), (19, -12), (18, -8)] {
        put(scene, x + dx, y + dy, ROCK.light);
        put(scene, x + dx + 1, y + dy, ROCK.base);
        put(scene, x + dx, y + dy + 1, ROCK.edge);
        put(scene, x + dx + 1, y + dy + 1, ROCK.edge);
    }
    // The broken cart, stuck on the track: its plank bed, a spoked wheel still on, and the shafts
    // down in the grass where the other wheel came off.
    let (bx, by) = (x + 11, y - 38);
    for px in bx..bx + 12 {
        let end = px == bx || px == bx + 11;
        put(scene, px, by, WOOD.edge);
        put(scene, px, by + 1, if end { WOOD.edge } else { WOOD.light });
        let joint = (px - bx) % 4 == 3;
        put(
            scene,
            px,
            by + 2,
            if end || joint { WOOD.edge } else { WOOD.base },
        );
        put(scene, px, by + 3, if end { WOOD.edge } else { WOOD.shadow });
        put(scene, px, by + 4, WOOD.edge);
    }
    line(scene, (bx, by + 3), (bx - 6, by + 8), WOOD.shadow);
    line(scene, (bx + 1, by + 4), (bx - 5, by + 9), WOOD.edge);
    let (wx, wy) = (bx + 7, by + 6);
    for spoke in 0..4 {
        let angle = spoke as f32 * TAU / 8.0;
        let (dx, dy) = (angle.cos() * 3.0, angle.sin() * 3.0);
        line(
            scene,
            (
                (wx as f32 - dx).round() as i32,
                (wy as f32 - dy).round() as i32,
            ),
            (
                (wx as f32 + dx).round() as i32,
                (wy as f32 + dy).round() as i32,
            ),
            WOOD.shadow,
        );
    }
    for step in 0..40 {
        let angle = step as f32 / 40.0 * TAU;
        let (rx, ry) = (
            (wx as f32 + angle.cos() * 4.0).round() as i32,
            (wy as f32 + angle.sin() * 4.0).round() as i32,
        );
        // The rim, lit along its upper left.
        let lit = angle.cos() + angle.sin() < -0.6;
        put(scene, rx, ry, if lit { WOOD.light } else { WOOD.edge });
    }
    put(scene, wx, wy, WOOD.light);
}

/// Paints rows of letters with their top-left at `at`, each letter a colour from `inks`; a
/// letter not among them is left as it was.
fn stamp_rows(scene: &mut Canvas, (x, y): (i32, i32), rows: &[&str], inks: &[(char, Rgba)]) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.chars().enumerate() {
            if let Some(&(_, color)) = inks.iter().find(|(name, _)| *name == letter) {
                put(scene, x + dx as i32, y + dy as i32, color);
            }
        }
    }
}

/// The Far Falls, once in sight: the crag with the falls coming down it under the little arch,
/// the plunge pool at its foot, and spray.
fn falls(scene: &mut Canvas) {
    let (fx, top) = FALLS_TOP;
    let (px, py, prx, pry) = PLUNGE;
    // The crag face behind the falls, in shade.
    polygon(
        scene,
        &[
            (fx - 20, py - 2),
            (fx - 18, top - 2),
            (fx - 10, top - 10),
            (fx + 12, top - 10),
            (fx + 20, top - 2),
            (fx + 22, py - 2),
        ],
        |x, y| {
            Some(if (x + y).rem_euclid(4) == 0 {
                ROCK.shadow
            } else if x < fx - 8 {
                ROCK.light
            } else {
                ROCK.base
            })
        },
    );
    // The little arch over the top.
    for dx in -8..=8 {
        let lift = 4 - (dx * dx) / 16;
        put(scene, fx + dx, top - 6 - lift, ROCK.edge);
        put(scene, fx + dx, top - 5 - lift, ROCK.shine);
    }
    vline(scene, fx - 8, top - 6, 6, ROCK.edge);
    vline(scene, fx + 8, top - 6, 6, ROCK.edge);
    // The water coming down.
    for y in top..py {
        for dx in -3..=3 {
            let color = if dx == -3 {
                WATER.base
            } else if dx == 3 {
                WATER.shadow
            } else if (y + dx * 3).rem_euclid(5) == 0 {
                WATER.light
            } else {
                WATER.shine
            };
            put(scene, fx + dx, y, color);
        }
    }
    ellipse(scene, px, py, prx, pry, WATER.edge);
    ellipse(scene, px, py, prx - 1, pry - 1, WATER.base);
    hline(scene, px - 5, py - 1, 10, WATER.shine);
    for (dx, dy) in [(-6, -3), (5, -4), (-2, -6), (8, -2), (-9, -1)] {
        put(scene, px + dx, py + dy, rgba(0xffffff, 200));
    }
}

/// Mist over somewhere nobody has found yet: soft white billows, and a question mark.
fn mist(scene: &mut Canvas, at: (f32, f32)) {
    let (cx, cy) = (at.0 as i32 + 6, at.1 as i32 - 30);
    for y in cy - 40..cy + 40 {
        for x in cx - 54..cx + 54 {
            if !on_paper(x, y) {
                continue;
            }
            let (u, v) = ((x - cx) as f32 / 50.0, (y - cy) as f32 / 34.0);
            let billow = patches(x, y, (9, 7), 2090) * 0.5;
            let r = u * u + v * v - billow;
            if r < 0.75 {
                let thick = ((0.75 - r) * 2.2).clamp(0.0, 0.85);
                let pixel = scene.get(x, y);
                scene.set(x, y, mix(pixel, rgb(0xf4f0e4), thick));
            }
        }
    }
    draw_text(scene, cx - GLYPH_WIDTH / 2, cy - 3, "?", INK);
}

/// Lettering for the names of places, five pixels high: smaller than a sign's, as a map's is.
/// Each letter is three pixels wide but M and W, which need five to read apart from H. Its width,
/// and its rows with the leftmost pixel in the highest bit.
fn small_glyph(letter: char) -> Option<(i32, [u8; 5])> {
    let wide = match letter {
        'M' => Some([0b10001, 0b11011, 0b10101, 0b10001, 0b10001]),
        'W' => Some([0b10001, 0b10001, 0b10101, 0b10101, 0b01010]),
        _ => None,
    };
    if let Some(rows) = wide {
        return Some((5, rows));
    }
    Some((
        3,
        match letter {
            'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
            'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
            'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
            'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
            'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
            'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
            'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
            'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
            'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
            'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
            'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
            'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
            'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
            'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
            'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
            'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
            'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
            'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
            'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
            'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
            'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
            'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
            'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
            'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
            _ => return None,
        },
    ))
}

/// A place's name lettered on the map, centred on `x` with its top at `y`: in ink, with the
/// paper showing round it so it reads over trees and paths.
fn label(scene: &mut Canvas, text: &str, (x, y): (i32, i32)) {
    // Anything without a letter, a space say, is a gap as wide as one.
    let glyphs: Vec<(i32, [u8; 5])> = text
        .chars()
        .map(|letter| small_glyph(letter.to_ascii_uppercase()).unwrap_or((3, [0; 5])))
        .collect();
    let width = glyphs.iter().map(|(wide, _)| wide + 1).sum::<i32>() - 1;
    let mut left = x - width / 2;
    let mut inked: Vec<(i32, i32)> = Vec::new();
    for (wide, rows) in glyphs {
        for (row, bits) in rows.iter().enumerate() {
            for column in 0..wide {
                if bits & (1 << (wide - 1 - column)) != 0 {
                    inked.push((left + column, y + row as i32));
                }
            }
        }
        left += wide + 1;
    }
    for &(px, py) in &inked {
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ] {
            if !inked.contains(&(px + dx, py + dy)) {
                let pixel = scene.get(px + dx, py + dy);
                scene.set(px + dx, py + dy, mix(pixel, PARCHMENT.light, 0.8));
            }
        }
    }
    for &(px, py) in &inked {
        put(scene, px, py, INK);
    }
}

/// The names of the places in sight, under where the party stands at each.
fn names(scene: &mut Canvas, shown: &Shown) {
    for (index, name, nudge) in [
        (HEDGEROW, "HEDGEROW", 0),
        (MEADOW, "MEADOW", 0),
        (GLADE, "GLADE", 0),
        (LOG, "FALLEN LOG", 0),
        (SIGNPOST, "SIGNPOST", 0),
        (POOL, "POOL", 0),
        (OLD_TRACK, "OLD TRACK", 0),
        (FAR_FALLS, "FAR FALLS", -6),
    ] {
        if !shown.places[index] {
            continue;
        }
        let (x, y) = PLACES[index].at;
        label(scene, name, (x as i32 + nudge, y as i32 + 6));
    }
}

/// The title, top left: lettered on a little scroll.
fn cartouche(scene: &mut Canvas) {
    let (x, y, width, height) = CARTOUCHE;
    // The scroll, its ends curled under.
    for py in y..y + height {
        for px in x..x + width {
            let color = if py == y || py == y + height - 1 {
                PARCHMENT.edge
            } else if py == y + 1 {
                PARCHMENT.shine
            } else if py == y + height - 2 {
                PARCHMENT.shadow
            } else {
                PARCHMENT.light
            };
            scene.set(px, py, color);
        }
    }
    for (end, dir) in [(x, -1), (x + width - 1, 1)] {
        for dy in 2..height - 2 {
            put(scene, end + dir, y + dy, PARCHMENT.shadow);
            put(scene, end + dir * 2, y + dy + 1, PARCHMENT.edge);
        }
    }
    let text = "THE WOODS";
    let left = x + (width - crate::font::text_width(text)) / 2;
    draw_text(scene, left + 1, y + 6, text, rgba(0x2a1a10, 70));
    draw_text(scene, left, y + 5, text, INK);
}

/// The compass, bottom right: a star of eight points, the north one marked.
fn compass(scene: &mut Canvas) {
    let (cx, cy) = COMPASS;
    for step in 0..8 {
        let angle = step as f32 / 8.0 * TAU - TAU / 4.0;
        let long = if step % 2 == 0 { 12.0 } else { 7.0 };
        for along in 0..long as i32 {
            let (x, y) = (
                cx as f32 + angle.cos() * along as f32,
                cy as f32 + angle.sin() * along as f32,
            );
            put(
                scene,
                x as i32,
                y as i32,
                if step == 0 { SHORTCUT_INK } else { INK },
            );
        }
    }
    ellipse(scene, cx, cy, 2, 2, PARCHMENT.light);
    put(scene, cx, cy, INK);
    draw_text(scene, cx - 2, cy - 21, "N", INK);
}

// ---------------------------------------------------------------------------------------------
// The picnic
// ---------------------------------------------------------------------------------------------

/// The picture of the picnic on the fallen log, pinned over `map`: a sunny clearing in the
/// woods, the great log lying across it, a cloth spread in front with the picnic on it, in a
/// cream mount with the map darkened a little round it.
pub fn picnic(map: &Canvas) -> Canvas {
    let mut scene = map.clone();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let pixel = scene.get(x, y);
            scene.set(x, y, mix(pixel, rgb(0x1a1410), 0.25));
        }
    }
    let (left, top, width, height) = super::picnic::PICTURE;
    // Its shadow on the map, the mount, and the picture inside it.
    for y in top + 3..top + height + 3 {
        for x in left + 3..left + width + 3 {
            let pixel = scene.get(x, y);
            scene.set(x, y, mix(pixel, rgb(0x0e0a08), 0.45));
        }
    }
    for y in top..top + height {
        for x in left..left + width {
            let rim = x == left || y == top || x == left + width - 1 || y == top + height - 1;
            let color = if rim {
                PARCHMENT.edge
            } else if x == left + 1 || y == top + 1 {
                PARCHMENT.shine
            } else {
                PARCHMENT.light
            };
            scene.set(x, y, color);
        }
    }
    let inner = (left + 5, top + 5, width - 10, height - 10);
    clearing(&mut scene, inner);
    // A pin at the top.
    let (pin_x, pin_y) = (left + width / 2, top + 2);
    ellipse(&mut scene, pin_x, pin_y, 2, 2, rgb(0x7a1a1a));
    put(&mut scene, pin_x, pin_y, rgb(0xd04a40));
    put(&mut scene, pin_x - 1, pin_y - 1, rgb(0xf08070));
    scene
}

/// The clearing in the picture: trees behind in a sunny haze, the fallen log across the middle,
/// the floor in front with the cloth and the picnic on it.
fn clearing(scene: &mut Canvas, (left, top, width, height): (i32, i32, i32, i32)) {
    let log_top = super::picnic::SEATS[0].1 as i32 - 2;
    for y in top..top + height {
        for x in left..left + width {
            let t = (y - top) as f32 / height as f32;
            let color = if y < log_top - 14 {
                // Woods behind, hazy, trunks and leaves.
                let trunk =
                    (x + pick(x.div_euclid(14), 0, 2100, 6)).rem_euclid(14) < 3 && y > top + 10;
                if trunk {
                    mix(WOOD.shadow, rgb(0xb8c8a8), 0.35)
                } else {
                    let leaf = patches(x, y, (6, 5), 2101);
                    mix(
                        mix(TREE.base, TREE.light, leaf),
                        rgb(0xe8e8c0),
                        0.25 + (1.0 - t) * 0.2,
                    )
                }
            } else {
                // The floor: grass and leaf litter, mottled, sunlit in patches.
                let sun = patches(x, y, (11, 6), 2102);
                let base = mix(
                    rgb(0x6a8a46),
                    rgb(0x8a7a4a),
                    patches(x, y, (4, 3), 2103) * 0.6,
                );
                mix(base, rgb(0xd8d890), sun * 0.35)
            };
            scene.set(x, y, color);
        }
    }
    // Shafts of sun slanting down from the upper left.
    for start in [left + 20, left + 90, left + 150] {
        for y in top..log_top {
            let x = start + (y - top) * 2 / 5;
            for dx in 0..6 {
                if x + dx < left + width {
                    let pixel = scene.get(x + dx, y);
                    scene.set(x + dx, y, mix(pixel, rgb(0xfff4c8), 0.18));
                }
            }
        }
    }
    // The fallen log, lying across, mossy on top, its cut end towards the left.
    let (from, to) = (left + 18, left + width - 16);
    for x in from..to {
        let sag = ((x - from) as f32 / (to - from) as f32 * std::f32::consts::PI).sin() * 1.5;
        let crown = log_top + sag.round() as i32;
        for (dy, color) in [
            (0, WOOD.edge),
            (1, WOOD.light),
            (2, WOOD.light),
            (3, WOOD.base),
            (4, WOOD.base),
            (5, WOOD.base),
            (6, WOOD.shadow),
            (7, WOOD.shadow),
            (8, WOOD.edge),
        ] {
            let mut color = color;
            if dy > 1 && dy < 7 && chance(x.div_euclid(3), dy, 2104, 30) {
                color = WOOD.shadow;
            }
            if dy <= 2 && patches(x, 0, (5, 1), 2105) > 0.55 {
                color = if dy == 0 { TREE.edge } else { TREE.light };
            }
            scene.set(x, crown + dy, color);
        }
        // Its shadow on the floor.
        for dy in 9..12 {
            let pixel = scene.get(x + 1, crown + dy);
            scene.set(x + 1, crown + dy, mix(pixel, rgb(0x2a2414), 0.35));
        }
    }
    for dy in 0..9 {
        for dx in -2..=1 {
            let r = ((dy - 4) * (dy - 4)) as f32 / 16.0 + (dx * dx) as f32 / 4.0;
            if r <= 1.0 {
                let ring = (((dy - 4) * (dy - 4) + dx * dx * 4) as f32).sqrt() as i32 % 2 == 0;
                put(
                    scene,
                    from + dx,
                    log_top + dy,
                    if r > 0.7 {
                        WOOD.edge
                    } else if ring {
                        WOOD.shine
                    } else {
                        WOOD.light
                    },
                );
            }
        }
    }
    // Mushrooms along the log's foot, and ferns at either end.
    for (index, dx) in [24, 60, 118, 150].iter().enumerate() {
        let (x, y) = (from + dx, log_top + 9);
        let cap = if index % 2 == 0 { 0xc8503a } else { 0xd8c8a0 };
        hline(scene, x - 1, y - 2, 3, rgb(cap));
        put(scene, x, y - 3, rgb(cap));
        put(scene, x - 1, y - 2, rgb(0xf0e0c8));
        put(scene, x, y - 1, rgb(0xe8dcc0));
    }
    // The cloth spread on the floor in front, checked red and white, with the picnic on it.
    let (cx, cy) = (left + width / 2 + 4, log_top + 22);
    for y in cy - 5..cy + 6 {
        for x in cx - 30..cx + 30 {
            let skew = (y - cy) / 2;
            let x = x + skew;
            let check = ((x - cx).div_euclid(4) + (y - cy).div_euclid(3)).rem_euclid(2) == 0;
            let color = if y == cy + 5 {
                rgb(0x8a2a24)
            } else if check {
                rgb(0xd04a40)
            } else {
                rgb(0xf4ece0)
            };
            scene.set(x, y, color);
        }
    }
    // A basket, a loaf, a bowl of berries, and cups.
    let (bx, by) = (cx - 16, cy - 2);
    for dx in -5i32..=5 {
        for dy in -3i32..=2 {
            let color = if dy == -3 || dx.abs() == 5 {
                rgb(0x5a4026)
            } else if (dx + dy).rem_euclid(2) == 0 {
                rgb(0xbc9a5e)
            } else {
                rgb(0x9e7a48)
            };
            put(scene, bx + dx, by + dy, color);
        }
    }
    line(scene, (bx - 4, by - 3), (bx, by - 7), rgb(0x5a4026));
    line(scene, (bx, by - 7), (bx + 4, by - 3), rgb(0x5a4026));
    ellipse(scene, cx + 2, cy - 1, 5, 2, rgb(0x8a5a2a));
    ellipse(scene, cx + 1, cy - 2, 4, 1, rgb(0xc8904a));
    put(scene, cx - 1, cy - 2, rgb(0xe8b878));
    ellipse(scene, cx + 16, cy, 3, 2, rgb(0xf4ece0));
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (0, -2)] {
        put(scene, cx + 16 + dx, cy + dy, rgb(0x4a2050));
    }
    for dx in [24, 28] {
        vline(scene, cx + dx, cy - 3, 3, rgb(0xe8e0d0));
        put(scene, cx + dx, cy - 3, rgb(0xfaf6ee));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_fills_every_pixel_and_shows_only_what_is_in_sight() {
        let all = Shown {
            places: [true; PLACES.len()],
            paths: [true; PATHS.len()],
        };
        let seen = map(&all);
        assert!(seen.pixels().iter().all(|pixel| pixel.a == 255));
        let mut misty = all.clone();
        misty.places[FAR_FALLS] = false;
        let unseen = map(&misty);
        let (x, y) = (FALLS_TOP.0, FALLS_TOP.1 + 10);
        assert_ne!(
            seen.get(x, y),
            unseen.get(x, y),
            "the falls show through the mist"
        );
        // Nor is a place out of sight named: the lettering under it is only there once seen.
        let (fx, fy) = (PLACES[FAR_FALLS].at.0 as i32, PLACES[FAR_FALLS].at.1 as i32);
        let lettered = |scene: &Canvas| {
            (fx - 26..fx + 16)
                .flat_map(|x| (fy + 4..fy + 12).map(move |y| (x, y)))
                .filter(|&(x, y)| scene.get(x, y) == INK)
                .count()
        };
        assert!(lettered(&seen) > 20, "the falls aren't named once seen");
        assert_eq!(lettered(&unseen), 0, "the falls are named through the mist");
    }
}
