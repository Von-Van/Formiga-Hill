//! The landmarks a map names, as a kit of props set out round a fork: the split oak and the oak
//! that is not split, the mossy stone and the grey one, the hollow stump and the plain one, the
//! holly, the birch, the woodpile and the old gatepost. Each is painted once, near, and the same
//! drawing is made small and hazy for far off down a way, so what stands far off is the same
//! thing and can be told for it, by someone sharp-eyed. And the dig: the old milestone, the
//! places by it that might be dug, what each is by, and the chest.

use super::routes::{Clue, Dig, Dir, Feature, Fork, Landmark, MILESTONE, Side, Spot};
use crate::paint::{Ramp, chance, ellipse, hline, line, mix, noise, put, rgb, rgba};
use crate::playground::Prop;
use formiga_art::{Canvas, Rgba};
use std::f32::consts::{PI, TAU};

const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const OAK_LEAF: Ramp = Ramp::new(0x1a3428, 0x264a34, 0x356240, 0x4c7e4c, 0x6c9c5c);
const BIRCH_LEAF: Ramp = Ramp::new(0x2c4a1e, 0x40662a, 0x568436, 0x74a248, 0x9cc266);
const BIRCH_BARK: Ramp = Ramp::new(0x6e6a62, 0xb4b0a6, 0xd8d4ca, 0xeeebe2, 0xfbf9f2);
const HOLLY: Ramp = Ramp::new(0x0c2018, 0x143024, 0x1e4432, 0x2c5c44, 0x4a8462);
const BERRY: Rgba = rgb(0xd4282a);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const LICHEN: Ramp = Ramp::new(0x5a5428, 0x7a7234, 0x9a9042, 0xb8ac56, 0xd0c470);
const WOOD: Ramp = Ramp::new(0x5a4630, 0x7a6244, 0x9a7e58, 0xb89a70, 0xd0b48a);
const POST: Ramp = Ramp::new(0x4e4a42, 0x6e685c, 0x8e8676, 0xaaa290, 0xc4bca8);
const IRON: Ramp = Ramp::new(0x1c2224, 0x2e3436, 0x444a4c, 0x5e6466, 0x80888a);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const GRASS: Ramp = Ramp::new(0x2a4224, 0x3a5a2e, 0x507438, 0x6a8e46, 0x88a85a);
const FOXGLOVE: Ramp = Ramp::new(0x5a2450, 0x82346e, 0xa64a8c, 0xc46aa6, 0xdc92c0);
const CHEST: Ramp = Ramp::new(0x3a2414, 0x5a3820, 0x7a4e2c, 0x98663c, 0xb48250);
const GOLD: Ramp = Ramp::new(0x7a4a08, 0xc07a10, 0xf0a818, 0xffc840, 0xfff4b0);
const HOLLOW: Rgba = rgb(0x18120e);
const SHADE: Rgba = rgba(0x14241c, 72);
/// The air far off down a way, that something far off fades into.
const FAR_AIR: Rgba = rgb(0x6a9286);

/// A drawing on its own canvas, with the point in it that stands on the ground.
pub struct Sprite {
    pub canvas: Canvas,
    pub foot: (i32, i32),
}

/// Where each way runs at the middle distance, across the picture.
fn across(dir: Dir) -> f32 {
    match dir {
        Dir::Left => 92.0,
        Dir::Ahead => 196.0,
        Dir::Right => 300.0,
    }
}

/// The wedge between two ways that part at the fork, where something standing between them is
/// clear of both: the ways fan out from the fork, so halfway across at a height would be on the
/// slanting one.
fn between(left: Dir, right: Dir) -> (f32, f32) {
    match (left, right) {
        (Dir::Left, Dir::Ahead) => (174.0, 138.0),
        (Dir::Ahead, Dir::Right) => (218.0, 138.0),
        _ => (196.0, 146.0),
    }
}

/// Where a landmark stands at a fork, as the point its foot is on.
pub fn spot_point(fork: &Fork, spot: Spot) -> (f32, f32) {
    match spot {
        Spot::Fork => {
            let ahead = fork.ways.contains(&Dir::Ahead);
            match fork.clue {
                Clue::Turn(Side::Left, _) if ahead => between(Dir::Left, Dir::Ahead),
                Clue::Turn(Side::Right, _) if ahead => between(Dir::Ahead, Dir::Right),
                _ => (196.0, 152.0),
            }
        }
        Spot::Far(way) => far_point(fork.ways.len(), fork.ways[way]),
        Spot::Beside(region) => {
            let count = fork.ways.len();
            if region == 0 {
                (across(fork.ways[0]) - 52.0, 146.0)
            } else if region == count {
                (across(fork.ways[count - 1]) + 52.0, 146.0)
            } else {
                between(fork.ways[region - 1], fork.ways[region])
            }
        }
    }
}

/// Where something far down a way stands, small in the distance.
pub fn far_point(_count: usize, dir: Dir) -> (f32, f32) {
    match dir {
        Dir::Left => (30.0, 116.0),
        Dir::Ahead => (214.0, 108.0),
        Dir::Right => (362.0, 116.0),
    }
}

/// The landmarks at a fork, each standing among everyone on its own row: near ones as they are,
/// far ones small and hazy.
pub fn props(fork: &Fork) -> Vec<Prop> {
    fork.landmarks
        .iter()
        .map(|placed| {
            let (x, y) = spot_point(fork, placed.spot);
            let sprite = match placed.spot {
                Spot::Far(_) => far(&near(placed.landmark)),
                _ => near(placed.landmark),
            };
            let at = (x as i32 - sprite.foot.0, y as i32 - sprite.foot.1);
            Prop::new(sprite.canvas, at, y)
        })
        .collect()
}

/// A landmark as it stands nearby.
pub fn near(landmark: Landmark) -> Sprite {
    let mut canvas = Canvas::new(40, 44);
    let foot = (20, 42);
    match landmark {
        Landmark::SplitOak => split_oak(&mut canvas, foot),
        Landmark::Oak => oak(&mut canvas, foot),
        Landmark::MossyStone => stone(&mut canvas, foot, true),
        Landmark::GreyStone => stone(&mut canvas, foot, false),
        Landmark::HollowStump => stump(&mut canvas, foot, true),
        Landmark::Stump => stump(&mut canvas, foot, false),
        Landmark::Holly => holly(&mut canvas, foot),
        Landmark::Birch => birch(&mut canvas, foot),
        Landmark::Woodpile => woodpile(&mut canvas, foot),
        Landmark::Gatepost => gatepost(&mut canvas, foot),
    }
    Sprite { canvas, foot }
}

/// A landmark made small and hazy, as it stands far off: each block of two by two that is mostly
/// there becomes one pixel of its colours, faded into the air.
pub fn far(near: &Sprite) -> Sprite {
    let (w, h) = (
        near.canvas.width().div_ceil(2),
        near.canvas.height().div_ceil(2),
    );
    let mut small = Canvas::new(w, h);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut sum = [0u32; 3];
            let mut opaque = 0;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let pixel = near.canvas.get(x * 2 + dx, y * 2 + dy);
                if pixel.a >= 200 {
                    opaque += 1;
                    sum[0] += u32::from(pixel.r);
                    sum[1] += u32::from(pixel.g);
                    sum[2] += u32::from(pixel.b);
                }
            }
            if opaque < 2 {
                continue;
            }
            let average = Rgba::new(
                (sum[0] / opaque) as u8,
                (sum[1] / opaque) as u8,
                (sum[2] / opaque) as u8,
                255,
            );
            small.set(x, y, mix(average, FAR_AIR, 0.3));
        }
    }
    Sprite {
        canvas: small,
        foot: (near.foot.0 / 2, near.foot.1 / 2),
    }
}

// ---------------------------------------------------------------------------------------------
// Brushes
// ---------------------------------------------------------------------------------------------

/// A clump of leaves, lit from the upper left, its rim scalloped and edged in its darkest green.
fn leaves(canvas: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), ramp: Ramp, salt: u32) {
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
        let angle = v.atan2(u);
        let scallop = 1.0 + 0.12 * (angle * 7.0 + salt as f32).sin();
        u * u + v * v <= scallop * scallop
    };
    for y in cy - ry - 2..=cy + ry + 2 {
        for x in cx - rx - 2..=cx + rx + 2 {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
            let (u, v) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            let light =
                u + v + (noise(x.div_euclid(2), y.div_euclid(2), salt) % 100) as f32 / 160.0 - 0.3;
            let color = if edge {
                ramp.edge
            } else if light < -0.7 {
                ramp.shine
            } else if light < -0.1 {
                ramp.light
            } else if light < 0.6 {
                ramp.base
            } else {
                ramp.shadow
            };
            put(canvas, x, y, color);
        }
    }
    // Leaf clusters catching the light.
    for index in 0..(rx * ry / 6) {
        let x = cx - rx + 1 + (noise(index, 0, salt) % (rx * 2 - 1).max(1) as u32) as i32;
        let y = cy - ry + 1 + (noise(index, 1, salt) % (ry * 2 - 1).max(1) as u32) as i32;
        if inside(x, y) && inside(x + 1, y + 1) && x + y < cx + cy + 2 {
            put(canvas, x, y, ramp.light);
            put(canvas, x + 1, y + 1, ramp.base);
        }
    }
}

/// A trunk from `foot` up to `top`, `width` across, lit from the left, its foot flared.
fn bole(canvas: &mut Canvas, (x, foot): (i32, i32), top: i32, width: i32, ramp: Ramp, salt: u32) {
    for y in top..=foot {
        let flare = ((y - (foot - 3)).max(0) * 2) / 2;
        let (from, to) = (x - width / 2 - flare, x + (width + 1) / 2 + flare);
        for px in from..to {
            let across = (px - from) as f32 / (to - from - 1).max(1) as f32;
            let mut color = if px == from || px == to - 1 {
                ramp.edge
            } else if across < 0.35 {
                ramp.light
            } else if across < 0.7 {
                ramp.base
            } else {
                ramp.shadow
            };
            if px > from && px < to - 1 && noise(px, y.div_euclid(3), salt).is_multiple_of(5) {
                color = mix(color, ramp.edge, 0.6);
            }
            put(canvas, px, y, color);
        }
    }
}

/// Grass round something's foot.
fn grass_round(canvas: &mut Canvas, (x, foot): (i32, i32), spread: i32, salt: u32) {
    for dx in -spread..=spread {
        if !chance(x + dx, foot, salt, 120) {
            continue;
        }
        let tall = 1 + (noise(dx, foot, salt + 1) % 3) as i32;
        line(
            canvas,
            (x + dx, foot),
            (x + dx + dx.signum(), foot - tall),
            GRASS.base,
        );
        put(canvas, x + dx + dx.signum(), foot - tall, GRASS.light);
    }
}

// ---------------------------------------------------------------------------------------------
// The kit
// ---------------------------------------------------------------------------------------------

/// An oak split down its middle by lightning long ago: two halves leaning apart from one foot,
/// the bare wood pale on their inner faces and scorched along the split, each with its own
/// crown.
fn split_oak(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 10, 2, SHADE);
    leaves(canvas, (x - 9, foot - 30), (8, 7), OAK_LEAF, 1401);
    leaves(canvas, (x + 10, foot - 27), (8, 7), OAK_LEAF, 1402);
    for (side, lean) in [(-1, -7), (1, 8)] {
        for y in 0..24 {
            let t = y as f32 / 24.0;
            let cx = x + side * 2 + (lean as f32 * t * t) as i32;
            for dx in 0..4 {
                let inner = (side < 0 && dx == 3) || (side > 0 && dx == 0);
                let color = if inner {
                    if y % 5 == 2 {
                        rgb(0x2a1c14)
                    } else {
                        WOOD.light
                    }
                } else if dx == if side < 0 { 0 } else { 3 } {
                    BARK.edge
                } else if side < 0 {
                    BARK.light
                } else {
                    BARK.base
                };
                let px = cx + if side < 0 { dx - 3 } else { dx };
                put(canvas, px, foot - y, color);
            }
        }
    }
    // The dark of the split between them at the foot.
    for y in 0..7 {
        put(canvas, x, foot - y, HOLLOW);
    }
    grass_round(canvas, (x, foot), 7, 1403);
}

/// An oak that is not split: one stout trunk, roots flaring, a big round crown.
fn oak(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 10, 2, SHADE);
    bole(canvas, (x, foot), foot - 22, 5, BARK, 1410);
    leaves(canvas, (x - 6, foot - 26), (9, 7), OAK_LEAF, 1411);
    leaves(canvas, (x + 7, foot - 25), (8, 7), OAK_LEAF, 1412);
    leaves(canvas, (x, foot - 32), (10, 8), OAK_LEAF, 1413);
    grass_round(canvas, (x, foot), 6, 1414);
}

/// A big stone, half sunk: furred with moss over its top and down one side, or bare and grey with
/// lichen and a crack.
fn stone(canvas: &mut Canvas, (x, foot): (i32, i32), mossy: bool) {
    let (rx, ry) = (10.0, 7.0);
    let (cx, cy) = (x as f32, foot as f32 - 6.0);
    ellipse(canvas, x + 3, foot, 12, 2, SHADE);
    let inside = |px: i32, py: i32| {
        let (u, v) = ((px as f32 + 0.5 - cx) / rx, (py as f32 + 0.5 - cy) / ry);
        u.abs().powf(2.3) + v.abs().powf(2.3) <= 1.0 && py <= foot
    };
    for py in foot - 15..=foot {
        for px in x - 12..=x + 12 {
            if !inside(px, py) {
                continue;
            }
            let edge =
                !inside(px - 1, py) || !inside(px + 1, py) || !inside(px, py - 1) || py == foot;
            let (u, v) = ((px as f32 + 0.5 - cx) / rx, (py as f32 + 0.5 - cy) / ry);
            let lit =
                -0.6 * u - 0.7 * v + ((noise(px, py, 1420) % 100) as f32 / 100.0 - 0.5) * 0.25;
            let moss = mossy && v < -0.25 + (noise(px, 0, 1421) % 4) as f32 * 0.12 - u * 0.25;
            let ramp = if moss { MOSS } else { ROCK };
            let color = if edge {
                ramp.edge
            } else if lit > 0.7 {
                ramp.shine
            } else if lit > 0.25 {
                ramp.light
            } else if lit > -0.3 {
                ramp.base
            } else {
                ramp.shadow
            };
            put(canvas, px, py, color);
        }
    }
    if mossy {
        // Moss standing up off its crown like fur, and hanging over its side.
        for px in x - 8..=x + 8 {
            if chance(px, 0, 1422, 140) {
                let top = (cy - ry * (1.0 - ((px - x) as f32 / rx).powi(2)).max(0.0).sqrt()) as i32;
                put(canvas, px, top - 1, MOSS.light);
            }
        }
    } else {
        for (dx, dy) in [(-4, -3), (3, -1), (-1, 1)] {
            put(canvas, x + dx, foot - 7 + dy, LICHEN.light);
            put(canvas, x + dx + 1, foot - 7 + dy, LICHEN.base);
        }
        line(canvas, (x + 2, foot - 12), (x + 5, foot - 5), ROCK.edge);
    }
    grass_round(canvas, (x, foot), 10, 1423);
}

/// A tree stump: bark round its sides, its sawn top showing rings; or a hollow one, rotted away
/// to a dark hole in its side, with a fern growing by it.
fn stump(canvas: &mut Canvas, (x, foot): (i32, i32), hollow: bool) {
    let (w, h) = (14, 11);
    let left = x - w / 2;
    ellipse(canvas, x + 2, foot, 9, 2, SHADE);
    for py in foot - h..=foot {
        let flare = (py - (foot - 3)).max(0);
        for px in left - flare..left + w + flare {
            let across = (px - left + flare) as f32 / (w + flare * 2 - 1) as f32;
            let color = if px == left - flare || px == left + w + flare - 1 {
                BARK.edge
            } else if across < 0.3 {
                BARK.light
            } else if across < 0.7 {
                BARK.base
            } else {
                BARK.shadow
            };
            put(canvas, px, py, color);
        }
    }
    // The top, seen a little from above.
    let top = foot - h;
    for py in top - 3..=top + 1 {
        for px in left..left + w {
            let (u, v) = (
                (px - x) as f32 / (w as f32 / 2.0),
                (py - top + 1) as f32 / 2.5,
            );
            let ring = (u * u + v * v).sqrt();
            if ring > 1.0 {
                continue;
            }
            let color = if hollow {
                if ring > 0.75 { BARK.base } else { HOLLOW }
            } else if ring > 0.85 {
                BARK.edge
            } else if ((ring * 4.0) as i32) % 2 == 0 {
                WOOD.light
            } else {
                WOOD.base
            };
            put(canvas, px, py, color);
        }
    }
    if hollow {
        // The hole rotted into its side, dark, with soft rotten wood round its lip.
        ellipse(canvas, x + 1, foot - 4, 3, 4, WOOD.shadow);
        ellipse(canvas, x + 1, foot - 4, 2, 3, HOLLOW);
        put(canvas, x - 1, foot - 7, WOOD.light);
        for (index, angle) in [-2.4f32, -2.0, -1.6].iter().enumerate() {
            let (dx, dy) = (angle.cos(), angle.sin());
            for step in 0..8 {
                let (px, py) = (
                    x - 7 + (dx * step as f32) as i32,
                    foot + (dy * step as f32 + step as f32 * step as f32 * 0.06) as i32,
                );
                put(
                    canvas,
                    px,
                    py,
                    if index == 1 { FERN.light } else { FERN.base },
                );
                if step % 2 == 0 {
                    put(canvas, px - 1, py + 1, FERN.shadow);
                }
            }
        }
    }
    grass_round(canvas, (x, foot), 9, 1430);
}

/// A holly bush: dark glossy leaves, spiky at its edge, and red berries in clusters.
fn holly(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 11, 2, SHADE);
    leaves(canvas, (x, foot - 10), (11, 9), HOLLY, 1440);
    leaves(canvas, (x - 4, foot - 17), (6, 5), HOLLY, 1441);
    // Spikes round its edge.
    for step in 0..28 {
        let angle = step as f32 / 28.0 * TAU;
        let (px, py) = (
            x + (angle.cos() * 12.5) as i32,
            foot - 10 + (angle.sin() * 10.0) as i32,
        );
        if py < foot && step % 2 == 0 {
            put(canvas, px, py, HOLLY.edge);
        }
    }
    // Glossy highlights, and berries.
    for (dx, dy) in [(-6, -14), (-3, -9), (2, -15), (5, -8), (-8, -6)] {
        put(canvas, x + dx, foot + dy, HOLLY.shine);
    }
    for (dx, dy) in [(-2, -11), (4, -12), (-6, -8), (6, -6), (1, -5)] {
        put(canvas, x + dx, foot + dy, BERRY);
        put(canvas, x + dx + 1, foot + dy, rgb(0x9a1a1c));
        put(canvas, x + dx, foot + dy - 1, rgb(0xf06a5a));
    }
}

/// A birch: a slender white trunk with black marks across it, and a light, airy crown.
fn birch(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 7, 2, SHADE);
    leaves(canvas, (x - 4, foot - 32), (6, 6), BIRCH_LEAF, 1450);
    leaves(canvas, (x + 5, foot - 36), (6, 5), BIRCH_LEAF, 1451);
    leaves(canvas, (x + 1, foot - 26), (7, 5), BIRCH_LEAF, 1452);
    bole(canvas, (x, foot), foot - 28, 3, BIRCH_BARK, 1453);
    for y in (foot - 26..foot).step_by(3) {
        if noise(x, y, 1454).is_multiple_of(2) {
            hline(canvas, x - 1, y, 2, rgb(0x2a2622));
        }
    }
    grass_round(canvas, (x, foot), 5, 1455);
}

/// A woodpile: logs stacked three, two and one, their sawn ends towards the eye.
fn woodpile(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 12, 2, SHADE);
    const LOG: [&str; 5] = [".bBb.", "bwlwb", "Blrwb", "bwwsb", ".bbb."];
    for (row, count) in [3, 2, 1].iter().enumerate() {
        for column in 0..*count {
            let left = x - count * 3 + column * 6 + 1;
            let top = foot - 5 - row as i32 * 5;
            for (dy, line_of) in LOG.iter().enumerate() {
                for (dx, code) in line_of.bytes().enumerate() {
                    let color = match code {
                        b'b' => BARK.shadow,
                        b'B' => BARK.base,
                        b'w' => WOOD.light,
                        b'l' => WOOD.shine,
                        b'r' => WOOD.shadow,
                        b's' => WOOD.base,
                        _ => continue,
                    };
                    put(canvas, left + dx as i32, top + dy as i32, color);
                }
            }
        }
    }
    grass_round(canvas, (x, foot), 11, 1460);
}

/// An old stone gatepost standing on its own where the gate and the wall are long gone, leaning,
/// its iron hinge pins rusted, moss on its top.
fn gatepost(canvas: &mut Canvas, (x, foot): (i32, i32)) {
    ellipse(canvas, x + 2, foot, 7, 2, SHADE);
    let (w, h) = (8, 24);
    for py in 0..h {
        let lean = py / 8;
        let left = x - w / 2 + lean;
        for px in left..left + w {
            let across = (px - left) as f32 / (w - 1) as f32;
            let mut color = if px == left || px == left + w - 1 || py == h - 1 {
                POST.edge
            } else if across < 0.3 {
                POST.light
            } else if across < 0.7 {
                POST.base
            } else {
                POST.shadow
            };
            if py > h - 4 {
                color = if across < 0.5 { MOSS.light } else { MOSS.base };
            }
            if chance(px, py, 1470, 24) {
                color = mix(color, POST.edge, 0.3);
            }
            put(canvas, px, foot - py, color);
        }
    }
    for py in [6, 16] {
        let lean = py / 8;
        put(canvas, x + w / 2 + lean, foot - py, IRON.base);
        put(canvas, x + w / 2 + lean + 1, foot - py, rgb(0x8a4a2a));
        put(canvas, x + w / 2 + lean + 1, foot - py - 1, IRON.light);
    }
    grass_round(canvas, (x, foot), 6, 1471);
}

// ---------------------------------------------------------------------------------------------
// The dig
// ---------------------------------------------------------------------------------------------

/// The milestone and the places that might be dug, those dug already turned over, and the chest,
/// `opening` of the way open, once it is found.
pub fn dig_props(dig: &Dig, dug: &[bool], opening: Option<f32>) -> Vec<Prop> {
    let mut props = Vec::new();
    let mut stone = Canvas::new(30, 30);
    super::scenery::milestone(&mut stone, (15, 28));
    props.push(Prop::new(
        stone,
        (MILESTONE.0 as i32 - 15, MILESTONE.1 as i32 - 28),
        MILESTONE.1,
    ));
    for (index, spot) in dig.spots.iter().enumerate() {
        let done = dug.get(index).copied().unwrap_or(false);
        let mut canvas = Canvas::new(40, 30);
        let foot = (20, 24);
        place_to_dig(&mut canvas, foot, done);
        feature(&mut canvas, (foot.0 + 9, foot.1), spot.feature);
        let at = (spot.at.0 as i32 - foot.0, spot.at.1 as i32 - foot.1);
        props.push(Prop::new(canvas, at, spot.at.1));
    }
    if let Some(open) = opening {
        let spot = dig.spots[dig.right].at;
        let mut canvas = Canvas::new(34, 30);
        chest(&mut canvas, (17, 25), open);
        props.push(Prop::new(
            canvas,
            (spot.0 as i32 - 17, spot.1 as i32 - 25),
            spot.1 + 0.5,
        ));
    }
    props
}

/// A place that might be dug: a patch of soft earth, or, dug, a hole with the spoil heaped beside
/// it.
fn place_to_dig(canvas: &mut Canvas, (x, foot): (i32, i32), dug: bool) {
    if dug {
        ellipse(canvas, x, foot - 1, 7, 3, EARTH.edge);
        ellipse(canvas, x, foot - 1, 5, 2, HOLLOW);
        hline(canvas, x - 4, foot - 3, 8, EARTH.shadow);
        for (dx, dy) in [(-8, -2), (-9, -3), (-7, -4), (-10, -1), (-6, -3)] {
            put(canvas, x + dx, foot + dy, EARTH.light);
            put(canvas, x + dx, foot + dy + 1, EARTH.base);
        }
    } else {
        for py in foot - 3..=foot {
            for px in x - 7..=x + 7 {
                let (u, v) = ((px - x) as f32 / 7.0, (py - foot + 1) as f32 / 2.5);
                if u * u + v * v > 1.0 || !chance(px, py, 1480, 200) {
                    continue;
                }
                let color = if v < -0.2 { EARTH.light } else { EARTH.base };
                put(canvas, px, py, color);
            }
        }
    }
}

/// What a place that might be dug is by, standing on `(x, foot)`: each big enough to tell from
/// the others at a glance, and nothing else in the clearing looks like any of them.
fn feature(canvas: &mut Canvas, (x, foot): (i32, i32), feature: Feature) {
    match feature {
        Feature::TwistedRoot => {
            // Two strands of root arching up out of the ground and back in, twisted round each
            // other, lit along their tops.
            for (strand, phase) in [(0, 0.0f32), (1, PI)] {
                for step in 0..=16 {
                    let t = step as f32 / 16.0;
                    let px = x - 3 + step;
                    let arch = (t * PI).sin() * 6.0;
                    let twist = (t * 9.0 + phase).sin() * 1.2;
                    let py = foot - (arch + twist).round() as i32;
                    let front = (t * 9.0 + phase).cos() > 0.0;
                    if strand == 1 && !front {
                        continue;
                    }
                    put(
                        canvas,
                        px,
                        py - 1,
                        if front { BARK.light } else { BARK.base },
                    );
                    put(canvas, px, py, BARK.base);
                    put(canvas, px, py + 1, BARK.edge);
                }
            }
            // Where it goes into the ground, earth heaped.
            for end in [x - 3, x + 13] {
                put(canvas, end - 1, foot + 1, EARTH.light);
                put(canvas, end, foot + 1, EARTH.base);
                put(canvas, end + 1, foot + 1, EARTH.shadow);
            }
        }
        Feature::FlatStone => {
            // A slab lying in the grass: its top lit, its front edge in shade, moss on it.
            let (cx, cy) = (x + 5, foot - 2);
            for py in cy - 3..=cy + 3 {
                for px in cx - 8..=cx + 8 {
                    let (u, v) = ((px - cx) as f32 / 8.4, (py - cy) as f32 / 2.6);
                    let lower = (py - cy - 2) as f32 / 2.6;
                    let top = u * u + v * v <= 1.0;
                    if !top && u * u + lower * lower > 1.0 {
                        continue;
                    }
                    let rim = top && u * u + v * v > 0.78;
                    let color = if !top {
                        if py >= cy + 3 { ROCK.edge } else { ROCK.shadow }
                    } else if rim {
                        ROCK.edge
                    } else if u + v < -0.4 {
                        ROCK.light
                    } else if chance(px, py, 1482, 40) {
                        ROCK.shadow
                    } else {
                        ROCK.base
                    };
                    put(canvas, px, py, color);
                }
            }
            put(canvas, cx - 4, cy - 1, ROCK.shine);
            for (dx, dy) in [(3, 0), (4, 0), (4, -1), (5, 1)] {
                put(canvas, cx + dx, cy + dy, MOSS.light);
            }
        }
        Feature::Foxgloves => {
            // A clump of foxgloves: broad leaves at the foot, and tall spires hung with bells
            // down one side, buds at their tips.
            for (dx, lean) in [(-3, -2), (0, -1), (3, 2), (5, 1)] {
                for step in 0..4 {
                    put(
                        canvas,
                        x + dx + lean * step / 3,
                        foot - step / 2,
                        GRASS.base,
                    );
                }
                put(canvas, x + dx + lean, foot - 2, GRASS.light);
            }
            for (dx, tall) in [(-1, 19), (3, 15), (6, 12)] {
                let px = x + dx;
                for step in 0..tall {
                    put(canvas, px, foot - step, GRASS.shadow);
                }
                put(canvas, px, foot - tall, GRASS.light);
                put(canvas, px, foot - tall + 1, FOXGLOVE.shadow);
                for bell in 0..(tall - 5) / 2 {
                    let py = foot - tall + 3 + bell * 2;
                    put(canvas, px + 1, py, FOXGLOVE.light);
                    put(canvas, px + 2, py, FOXGLOVE.base);
                    put(canvas, px + 1, py + 1, FOXGLOVE.base);
                    put(canvas, px + 2, py + 1, FOXGLOVE.edge);
                    if bell % 2 == 0 {
                        put(canvas, px - 1, py + 1, FOXGLOVE.shine);
                    }
                }
            }
        }
        Feature::Molehill => {
            // A heap of crumbly earth thrown up fresh, darker than the floor, crumbs rolled off it.
            let (cx, w, h) = (x + 5, 7.0, 6.0);
            for py in foot - 7..=foot {
                for px in cx - 8..=cx + 8 {
                    let u = (px - cx) as f32 / w;
                    let rise = (foot - py) as f32 / h;
                    if u * u + rise > 1.0 {
                        continue;
                    }
                    let light = u - rise * 0.6;
                    let color = if u * u + rise > 0.82 || chance(px, py, 1481, 50) {
                        EARTH.edge
                    } else if light < -0.35 {
                        EARTH.light
                    } else if light < 0.25 {
                        EARTH.base
                    } else {
                        EARTH.shadow
                    };
                    put(canvas, px, py, color);
                }
            }
            for (dx, dy) in [(-9, 0), (9, 0), (-7, 1), (8, 1), (10, -1)] {
                put(canvas, cx + dx, foot + dy, EARTH.base);
            }
        }
        Feature::Ferns => {
            // A clump of fern fronds arching out every way from the middle.
            for (index, angle) in [-2.75f32, -2.35, -1.95, -1.55, -1.2, -0.8, -0.4]
                .iter()
                .enumerate()
            {
                let length = 9 + (index as i32 % 3) * 2;
                frond(canvas, (x + 4, foot), *angle, length);
            }
        }
    }
}

/// A fern frond from `from`, reaching out at `angle` and drooping, `length` long: a dark stem
/// with leaflets either side, lit along the upper.
fn frond(canvas: &mut Canvas, from: (i32, i32), angle: f32, length: i32) {
    let (dx, dy) = (angle.cos(), angle.sin());
    for step in 0..length {
        let t = step as f32 / length as f32;
        let droop = t * t * length as f32 * 0.45;
        let x = (from.0 as f32 + dx * step as f32).round() as i32;
        let y = (from.1 as f32 + dy * step as f32 + droop).round() as i32;
        put(canvas, x, y, FERN.shadow);
        let leaf = ((1.0 - t) * 2.6) as i32 + 1;
        if step % 2 == 1 {
            for reach in 1..=leaf {
                put(canvas, x - reach / 2, y - reach.min(2) + 1, FERN.light);
                put(canvas, x + reach / 2 + 1, y - reach.min(2) + 1, FERN.base);
            }
        }
    }
}

/// Warm light laid over whatever is painted in a disc about `(cx, cy)`.
fn glow(canvas: &mut Canvas, (cx, cy): (i32, i32), radius: f32, color: Rgba) {
    let reach = radius.ceil() as i32;
    for y in cy - reach..=cy + reach {
        for x in cx - reach..=cx + reach {
            let (dx, dy) = ((x - cx) as f32, (y - cy) as f32 * 1.6);
            if dx * dx + dy * dy <= radius * radius {
                put(canvas, x, y, color);
            }
        }
    }
}

/// The chest, iron-bound, sitting in the hole it was dug from, `open` of the way open: its lid
/// tipping back on its hinges, and the gold heaped inside shining out as it does.
fn chest(canvas: &mut Canvas, (x, foot): (i32, i32), open: f32) {
    let (w, h) = (18, 9);
    let left = x - w / 2;
    let top = foot - h;
    let open = open.clamp(0.0, 1.0);
    // The lid, tipped back: drawn first, behind the box, as a board rising from the hinge.
    let tilt = (open * 7.0).round() as i32;
    if tilt > 0 {
        for row in 0..=tilt {
            let y = top - row;
            for px in left..left + w {
                let edge = px == left || px == left + w - 1 || row == tilt;
                let color = if edge {
                    CHEST.edge
                } else if row % 3 == 1 {
                    CHEST.light
                } else {
                    CHEST.shadow
                };
                put(canvas, px, y, color);
            }
        }
        hline(canvas, left, top - tilt, w, IRON.base);
        glow(
            canvas,
            (x, top),
            10.0 * open,
            rgba(0xffe7a0, (60.0 * open) as u8),
        );
    }
    // The box, banded with iron.
    for py in top..foot {
        for px in left..left + w {
            let across = (px - left) as f32 / (w - 1) as f32;
            let mut color = if px == left || px == left + w - 1 || py == foot - 1 {
                CHEST.edge
            } else if py == top {
                if tilt > 0 { GOLD.light } else { CHEST.light }
            } else if across < 0.2 {
                CHEST.light
            } else if (py - top) % 3 == 0 {
                CHEST.shadow
            } else {
                CHEST.base
            };
            if (px == left + 3 || px == left + w - 4) && py > top {
                color = if py == top + 1 { IRON.light } else { IRON.base };
            }
            put(canvas, px, py, color);
        }
    }
    if tilt > 0 {
        // The gold heaped up in the mouth of it.
        for (dx, dy, color) in [
            (-6, -1, GOLD.base),
            (-4, -2, GOLD.light),
            (-2, -2, GOLD.shine),
            (0, -3, GOLD.light),
            (2, -2, GOLD.base),
            (4, -2, GOLD.light),
            (6, -1, GOLD.shadow),
            (-1, -1, GOLD.base),
            (3, -1, GOLD.shadow),
        ] {
            put(canvas, x + dx, top + dy, color);
            put(canvas, x + dx + 1, top + dy, color);
        }
        put(canvas, x - 2, top - 3, GOLD.shine);
    } else {
        // Shut, the lid is a rounded top over it.
        for row in 0..4 {
            let y = top - 1 - row;
            let inset = row / 2;
            for px in left + inset..left + w - inset {
                let edge = row == 3 || px == left + inset || px == left + w - inset - 1;
                let color = if edge {
                    CHEST.edge
                } else if row == 2 {
                    CHEST.light
                } else {
                    CHEST.base
                };
                put(canvas, px, y, color);
            }
        }
        hline(canvas, left + 1, top - 1, w - 2, IRON.base);
    }
    // The lock plate.
    put(canvas, x, top + 1, GOLD.light);
    put(canvas, x - 1, top + 1, GOLD.base);
    put(canvas, x, top + 2, GOLD.shadow);
    // The near lip of the hole over its foot, and earth thrown up either side.
    for px in left - 3..left + w + 3 {
        let dip = ((px - x) as f32 / (w as f32 / 2.0 + 3.0)).powi(2);
        let y = foot - 1 + (dip * 1.5) as i32;
        put(canvas, px, y, EARTH.base);
        put(canvas, px, y + 1, EARTH.shadow);
    }
    for (dx, dy) in [
        (-12, -2),
        (-13, -1),
        (-11, -3),
        (12, -2),
        (13, -1),
        (11, -1),
        (-12, 0),
        (12, 0),
    ] {
        put(canvas, x + dx, foot + dy, EARTH.light);
        put(canvas, x + dx, foot + dy + 1, EARTH.base);
    }
}

/// Every landmark, near and far, on a strip of woodland floor; and the dig's features, a place
/// dug and not, and the chest shut and open: for review.
pub fn sheet() -> Canvas {
    const CELL: (i32, i32) = (48, 56);
    let mut sheet = Canvas::new(
        (CELL.0 * Landmark::ALL.len() as i32) as u32,
        (CELL.1 * 3) as u32,
    );
    for row in 0..3 {
        for column in 0..Landmark::ALL.len() as i32 {
            let (left, top) = (column * CELL.0, row * CELL.1);
            for y in 0..CELL.1 {
                let color = if y < CELL.1 - 10 {
                    mix(rgb(0x547e70), rgb(0x6a9286), y as f32 / CELL.1 as f32)
                } else {
                    mix(
                        rgb(0x6e5238),
                        rgb(0x54402e),
                        (y - CELL.1 + 10) as f32 / 10.0,
                    )
                };
                sheet.fill_rect(left, top + y, CELL.0, 1, color);
            }
        }
    }
    for row in 0..2 {
        for (column, landmark) in Landmark::ALL.into_iter().enumerate() {
            let (left, top) = (column as i32 * CELL.0, row * CELL.1);
            let sprite = if row == 0 {
                near(landmark)
            } else {
                far(&near(landmark))
            };
            let foot = (left + CELL.0 / 2, top + CELL.1 - 6);
            crate::paint::blit(
                &mut sheet,
                &sprite.canvas,
                foot.0 - sprite.foot.0,
                foot.1 - sprite.foot.1,
            );
        }
    }
    // The dig: each feature by its place, a place dug, and the chest shut, half open and open.
    let top = 2 * CELL.1;
    for (column, feature_by) in Feature::ALL.into_iter().enumerate() {
        let foot = (column as i32 * CELL.0 + CELL.0 / 2 - 6, top + CELL.1 - 6);
        place_to_dig(&mut sheet, foot, false);
        feature(&mut sheet, (foot.0 + 9, foot.1), feature_by);
    }
    let column = Feature::ALL.len() as i32;
    place_to_dig(
        &mut sheet,
        (column * CELL.0 + CELL.0 / 2, top + CELL.1 - 6),
        true,
    );
    for (index, open) in [0.0, 0.5, 1.0].into_iter().enumerate() {
        let x = (column + 1 + index as i32) * CELL.0 + CELL.0 / 2;
        chest(&mut sheet, (x, top + CELL.1 - 6), open);
    }
    sheet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::routes::Route;

    #[test]
    fn every_landmark_is_drawn_near_and_far_and_its_look_alike_looks_different() {
        for landmark in Landmark::ALL {
            let sprite = near(landmark);
            let (left, top, right, bottom) = sprite.canvas.alpha_bounds().expect("nothing drawn");
            assert!(
                bottom as i32 <= sprite.foot.1 + 2,
                "{landmark:?} sinks below its foot"
            );
            assert!((left as i32) < sprite.foot.0 && (right as i32) > sprite.foot.0);
            assert!(top as i32 >= 0);
            let small = far(&sprite);
            assert!(
                small.canvas.alpha_bounds().is_some(),
                "{landmark:?} vanishes far off"
            );
            if let Some(alike) = landmark.lookalike() {
                assert_ne!(
                    sprite.canvas,
                    near(alike).canvas,
                    "{landmark:?} is drawn as {alike:?}"
                );
            }
        }
    }

    #[test]
    fn every_landmark_at_every_fork_stands_on_the_ground_and_off_the_ways() {
        for map in 0..300 {
            for fork in Route::of(map).forks {
                for placed in &fork.landmarks {
                    let (x, y) = spot_point(&fork, placed.spot);
                    assert!((0.0..384.0).contains(&x) && (100.0..200.0).contains(&y));
                    if matches!(placed.spot, Spot::Far(_)) {
                        continue;
                    }
                    for dir in &fork.ways {
                        let near = super::super::scenery::way_path(*dir)
                            .iter()
                            .map(|point| crate::playground::distance(*point, (x, y)))
                            .fold(f32::MAX, f32::min);
                        assert!(
                            near > 9.0,
                            "map {map}: {:?} stands on {dir:?}",
                            placed.landmark
                        );
                    }
                }
            }
        }
    }
}
