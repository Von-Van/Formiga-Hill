//! The Far Falls, from the shingle at the foot of the gorge: the water pouring out from under an
//! old stone arch high between two crags, falling the height of the picture into a deep green
//! plunge pool, the spray off it catching a rainbow. The crags are jointed blocks of rock, the
//! left one in the gorge's shade and the right one catching the light from the upper left, with
//! ferns and moss on every ledge, trees along their tops and ivy hanging from the arch. A slice of
//! sky shows between their tops, over the arch.
//!
//! The eddy the pool turns in is left clear (see `EDDY`), and the stones and shingle where the
//! party stands are kept free of clutter. Nothing is blown out to white, so the falls can be
//! tinted towards dusk as the day's light runs out.

use super::{EDDY, FOOT, LIP};
use crate::paint::{
    Ramp, chance, ellipse, hline, line, looped, mix, noise, patches, pick, put, rgb, rgba, tone,
};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::{PI, TAU};

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

const CRAG: Ramp = Ramp::new(0x3c3832, 0x5a544c, 0x787066, 0x968c7e, 0xb6ac9a);
const SHADED_CRAG: Ramp = Ramp::new(0x262624, 0x383632, 0x4c4842, 0x645e56, 0x7e776c);
const WET_ROCK: Ramp = Ramp::new(0x1c2422, 0x2a3431, 0x3a4642, 0x4e5c56, 0x687870);
const ARCH: Ramp = Ramp::new(0x5e584e, 0x847c6e, 0xa49a88, 0xc0b6a2, 0xdcd2bc);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const IVY: Ramp = Ramp::new(0x16301c, 0x224428, 0x305c34, 0x447a44, 0x62985a);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const CANOPY: Ramp = Ramp::new(0x203a2c, 0x2c4c36, 0x3c6444, 0x527e54, 0x6e9a66);
const FAR_CANOPY: Ramp = Ramp::new(0x3e5a4c, 0x4c6a5a, 0x5c7c68, 0x708e78, 0x86a28a);
const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const FALLS: Ramp = Ramp::new(0x2c5450, 0x4a7a72, 0x6a9a8e, 0x92bcae, 0xb4d4c6);
const POOL: Ramp = Ramp::new(0x14302e, 0x204442, 0x2e5a55, 0x44766c, 0x66968a);
const SHINGLE: Ramp = Ramp::new(0x4a4238, 0x6a6052, 0x8a7e6c, 0xa89c86, 0xc6baa2);
const SLATE: Ramp = Ramp::new(0x3c4446, 0x56605e, 0x737c78, 0x939a92, 0xb4b8ae);
const SAND: Ramp = Ramp::new(0x4e4434, 0x665a46, 0x807258, 0x9a8c6e, 0xb4a686);
const FOAM: Rgba = rgb(0xc8dcd2);
const SKY_TOP: Rgba = rgb(0xa8d0e4);
const SKY_LOW: Rgba = rgb(0xe8eedc);

/// How far down the sky shows, at the most.
const SKY_FOOT: i32 = 60;
/// The arch: its middle, how far its opening reaches either side, how high its opening rises
/// above where it springs, and where it springs from (the lip the water comes over).
const ARCH_MIDDLE: i32 = 192;
const ARCH_SPAN: i32 = 30;
const ARCH_RISE: i32 = 20;
const ARCH_SPRING: i32 = 40;
/// The falls: where they are at the lip and at the foot, either side.
const FALLS_TOP: (i32, i32) = (178, 206);
const FALLS_FOOT: (i32, i32) = (172, 212);
/// The back of the pool, at the foot of the crags.
const POOL_BACK: i32 = 116;

/// Everything behind the party, filling every pixel.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    rim_woods(&mut scene);
    upstream(&mut scene);
    back_wall(&mut scene);
    crag(&mut scene, true);
    crag(&mut scene, false);
    arch(&mut scene);
    ledges(&mut scene);
    pool(&mut scene);
    falls(&mut scene);
    foot(&mut scene);
    rainbow(&mut scene);
    boulders(&mut scene);
    shore(&mut scene);
    ferns(&mut scene);
    scene
}

/// What is nearer the eye than anyone: ferns up out of the bottom corners and a bough of
/// leaves over the top left. Transparent everywhere else.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for (index, &(x, angles)) in [
        (4, [-70.0, -42.0, -100.0, -18.0]),
        (20, [-82.0, -55.0, -112.0, -32.0]),
        (WIDTH - 6, [-110.0, -138.0, -80.0, -162.0]),
        (WIDTH - 22, [-98.0, -125.0, -68.0, -148.0]),
    ]
    .iter()
    .enumerate()
    {
        for (step, angle) in angles.iter().enumerate() {
            let length = 18.0 + pick(index as i32, step as i32, 1901, 9) as f32;
            frond(
                &mut scene,
                (x, HEIGHT + 2),
                (*angle, length),
                1 + (step as i32 % 3),
            );
        }
    }
    bough(&mut scene);
    scene
}

/// What glows after dark: glow-worms in the ferns on the ledges, and fireflies over the water.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for &(x, y) in &[
        (58, 84),
        (96, 64),
        (128, 98),
        (300, 74),
        (334, 98),
        (262, 102),
        (30, 150),
        (356, 150),
    ] {
        crate::daylight::glow(&mut lights, (x, y), 5, rgb(0xb8f07a));
        put(&mut lights, x, y, rgb(0xeaffc0));
    }
    crate::daylight::fireflies(
        &mut lights,
        &[(70, 120, 90, 30, 6), (230, 118, 100, 30, 6)],
        1907,
    );
    lights
}

/// Where the moon rises after dark: in the slice of sky, beside the arch.
const MOON: (i32, i32) = (262, 12);

/// The sky after dark, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, SKY_FOOT, Some(MOON))
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

/// The broken outline of an ellipse, for ripples and rings of foam.
fn ring(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), color: Rgba, salt: u32) {
    let steps = (rx + ry) * 4;
    for step in 0..steps {
        if chance(step, rx, salt, 80) {
            continue;
        }
        let angle = step as f32 / steps as f32 * TAU;
        put(
            scene,
            cx + (angle.cos() * rx as f32).round() as i32,
            cy + (angle.sin() * ry as f32).round() as i32,
            color,
        );
    }
}

/// A clump of leaves lit from the upper left, its rim broken into smaller clusters.
fn clump(scene: &mut Canvas, (cx, cy): (i32, i32), radius: i32, ramp: Ramp, salt: u32) {
    let ry = ((radius as f32 * 0.8) as i32).max(1);
    for step in 0..radius * 3 {
        let angle = step as f32 / (radius * 3) as f32 * TAU;
        let reach = radius as f32 - 1.0 + pick(step, 0, salt, 3) as f32;
        let (x, y) = (
            cx + (angle.cos() * reach).round() as i32,
            cy + (angle.sin() * reach * 0.8).round() as i32,
        );
        let size = 1 + pick(step, 1, salt, 2);
        ellipse(scene, x, y, size + 1, size + 1, ramp.edge);
        let lit = angle.cos() + angle.sin();
        let color = if lit < -0.5 {
            ramp.light
        } else if lit < 0.6 {
            ramp.base
        } else {
            ramp.shadow
        };
        ellipse(
            scene,
            x - (x - cx).signum(),
            y - (y - cy).signum(),
            size,
            size,
            color,
        );
    }
    for y in cy - ry..=cy + ry {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = ((x - cx) as f32 / radius as f32, (y - cy) as f32 / ry as f32);
            if dx * dx + dy * dy > 0.8 {
                continue;
            }
            let lit = dx + dy;
            let cell = noise(x.div_euclid(2), y.div_euclid(2), salt + 7) % 8;
            let color = if lit < -0.5 {
                if cell < 2 { ramp.shine } else { ramp.light }
            } else if lit < 0.3 {
                if cell < 3 { ramp.light } else { ramp.base }
            } else if cell < 3 {
                ramp.base
            } else {
                ramp.shadow
            };
            put(scene, x, y, color);
        }
    }
}

/// A fern frond from `from` out along `angle` degrees for `length` pixels, its leaflets in pairs
/// along it, `level` setting how lit it is.
fn frond(scene: &mut Canvas, from: (i32, i32), (angle, length): (f32, f32), level: i32) {
    let (sin, cos) = angle.to_radians().sin_cos();
    let steps = length as i32;
    for step in 0..steps {
        let t = step as f32 / steps as f32;
        // A frond arches over as it goes.
        let droop = t * t * length * 0.25;
        let (x, y) = (
            from.0 as f32 + cos * step as f32,
            from.1 as f32 + sin * step as f32 + droop,
        );
        put(scene, x as i32, y as i32, tone(FERN, level - 1));
        if step % 2 == 0 && step > 1 {
            let leaf = ((1.0 - t) * 5.0 + 1.0) as i32;
            for side in [-1.0f32, 1.0] {
                for along in 1..=leaf {
                    let (lx, ly) = (
                        x + (-sin) * side * along as f32 + cos * along as f32 * 0.4,
                        y + cos * side * along as f32 + sin * along as f32 * 0.4,
                    );
                    let lit = if side < 0.0 { level + 1 } else { level };
                    put(
                        scene,
                        lx as i32,
                        ly as i32,
                        tone(FERN, if along == leaf { lit - 1 } else { lit }),
                    );
                }
            }
        }
    }
}

/// A block of rock lit from the upper left: a rounded slab with a lumpy outline and a flat
/// underside, pitted and faceted, moss on its crown, drawn only where `clip` allows.
#[allow(clippy::too_many_arguments)]
fn rock(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    (rx, ry): (i32, i32),
    square: f32,
    moss: f32,
    ramp: Ramp,
    salt: u32,
    clip: &dyn Fn(i32, i32) -> bool,
) {
    let (frx, fry) = (rx.max(1) as f32, ry.max(1) as f32);
    let reach = |u: f32, v: f32| u.abs().powf(square) + v.abs().powf(square);
    let inside = |x: i32, y: i32| {
        let (u, v) = ((x - cx) as f32 / frx, (y - cy) as f32 / fry);
        let around = (v.atan2(u) / TAU + 0.5) * 9.0;
        let lump = 0.86 + 0.18 * looped(around, 9, salt);
        reach(u, v) <= lump.powf(square) && v < 0.92 && clip(x, y)
    };
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
            let crown = v + (patches(x, y, (3, 2), salt + 3) - 0.5) * 0.8 + u * 0.2;
            if crown < -1.05 + moss * 1.4 {
                color = if rim {
                    MOSS.edge
                } else {
                    tone(MOSS, (level - 1).clamp(1, 4))
                };
            } else if !rim && chance(x, y, salt + 4, 12) {
                color = mix(color, rgb(0xb4b67c), 0.4);
            }
            put(scene, x, y, color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The sky, the woods along the tops, and the stream above the falls
// ---------------------------------------------------------------------------------------------

/// The sky over the gorge: in bands, dithered where they meet, with a cloud or two.
fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 7;
    let band = |y: i32| (y * BANDS / SKY_FOOT).min(BANDS - 1);
    for y in 0..SKY_FOOT {
        for x in 0..WIDTH {
            let mut index = band(y);
            if band(y + 1) != index && (x + y) % 2 == 0 {
                index += 1;
            }
            scene.set(
                x,
                y,
                mix(SKY_TOP, SKY_LOW, index as f32 / (BANDS - 1) as f32),
            );
        }
    }
    for (cx, cy, rx, ry) in [(236, 9, 16, 3), (248, 7, 10, 3), (146, 16, 10, 2)] {
        ellipse(scene, cx, cy, rx, ry, rgba(0xfdfbf5, 210));
        hline(scene, cx - rx + 3, cy + ry, rx * 2 - 5, rgba(0xc8d8e4, 170));
    }
}

/// The top of a crag at column `x`: rising out of the picture towards the corners, lowest where
/// it meets the arch.
fn crag_top(x: i32) -> i32 {
    let off = if x < ARCH_MIDDLE {
        (150 - x).max(0)
    } else {
        (x - 234).max(0)
    } as f32;
    let wobble = (looped(x as f32 / 7.0, 64, 1700) - 0.5) * 8.0;
    (52.0 - off * 0.3 + wobble).round() as i32
}

/// Trees along the tops of the crags against the sky: the far ones hazy, the near ones darker,
/// leaning out over the gorge.
fn rim_woods(scene: &mut Canvas) {
    for (index, x) in (-12..WIDTH + 12).step_by(9).enumerate() {
        let i = index as i32;
        if (150..234).contains(&x) {
            continue;
        }
        let top = crag_top(x);
        let radius = 7 + pick(i, 1, 1702, 5);
        clump(
            scene,
            (x + pick(i, 0, 1702, 5) - 2, top - radius / 2),
            radius,
            FAR_CANOPY,
            1703 + i as u32,
        );
    }
    for (index, x) in (-6..WIDTH + 6).step_by(14).enumerate() {
        let i = index as i32;
        if (140..244).contains(&x) {
            continue;
        }
        let top = crag_top(x);
        let radius = 8 + pick(i, 2, 1704, 4);
        clump(
            scene,
            (x + pick(i, 3, 1704, 7) - 3, top + 2),
            radius,
            CANOPY,
            1705 + i as u32,
        );
    }
}

/// Through the arch: far off, the woods upstream in the sun; nearer, the stream coming down to
/// the lip between mossy banks, in the shade under the arch.
fn upstream(scene: &mut Canvas) {
    for y in ARCH_SPRING - ARCH_RISE..=ARCH_SPRING {
        let Some(reach) = opening(y) else {
            continue;
        };
        let depth = (y - (ARCH_SPRING - ARCH_RISE)) as f32 / ARCH_RISE as f32;
        for x in ARCH_MIDDLE - ARCH_SPAN..=ARCH_MIDDLE + ARCH_SPAN {
            let off = (x - ARCH_MIDDLE) as f32;
            if off.abs() > reach {
                continue;
            }
            let stream = 1.5 + (depth - 0.4).max(0.0) * 26.0;
            let mut color = if depth < 0.4 {
                // The woods beyond, sunlit and hazy with the distance.
                let leaf = patches(x, y, (3, 2), 1710);
                mix(mix(FAR_CANOPY.base, FAR_CANOPY.light, leaf), SKY_LOW, 0.35)
            } else if off.abs() < stream {
                match noise(x, y.div_euclid(2), 1711) % 9 {
                    0 => FALLS.shine,
                    1 | 2 => FALLS.light,
                    _ => FALLS.base,
                }
            } else {
                // The banks, mossy, darker away from the water.
                let away = ((off.abs() - stream) / 10.0).clamp(0.0, 1.0);
                mix(
                    MOSS.base,
                    MOSS.edge,
                    away * 0.7 + patches(x, y, (3, 2), 1712) * 0.2,
                )
            };
            // The shade under the arch's curve, deepest at its edge.
            let inside = reach - off.abs();
            if inside < 4.0 {
                color = mix(color, rgb(0x141c18), (1.0 - inside / 4.0) * 0.6);
            }
            if depth < 0.12 {
                color = mix(color, rgb(0x141c18), 0.3);
            }
            scene.set(x, y, color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The rock
// ---------------------------------------------------------------------------------------------

/// The inner edge of a crag at row `y`: where the left one's face ends and the right one's
/// begins, closing in on the falls as they go down, and stepping back at the foot.
fn crag_edge(y: i32, left: bool) -> i32 {
    let t = (y as f32 / POOL_BACK as f32).clamp(0.0, 1.0);
    let wobble = (looped(y as f32 / 6.0, 64, if left { 1721 } else { 1722 }) - 0.5) * 8.0;
    let base = if left {
        150.0 + 18.0 * (t * PI).sin() - 130.0 * (t - 0.8).max(0.0)
    } else {
        234.0 - 18.0 * (t * PI).sin() + 130.0 * (t - 0.8).max(0.0)
    };
    (base + wobble).round() as i32
}

/// Whether a point is on one of the crags.
fn on_crag(x: i32, y: i32, left: bool) -> bool {
    let edge = crag_edge(y, left);
    let side = if left { x < edge } else { x > edge };
    side && y >= crag_top(x) && y < POOL_BACK + 4
}

/// The wall of wet rock at the back of the gorge, behind the falls: dark, glistening, mossy where
/// the spray reaches. It starts below the lip, so the stream under the arch runs right up to the
/// falls without a band of rock across it.
fn back_wall(scene: &mut Canvas) {
    let behind = |x: i32, y: i32| {
        (ARCH_SPRING + 1..POOL_BACK + 2).contains(&y)
            && !on_crag(x, y, true)
            && !on_crag(x, y, false)
    };
    for y in ARCH_SPRING + 1..POOL_BACK + 2 {
        for x in 100..290 {
            if behind(x, y) {
                let color = if chance(x, y, 1731, 50) {
                    WET_ROCK.shadow
                } else {
                    WET_ROCK.edge
                };
                scene.set(x, y, color);
            }
        }
    }
    let mut index = 0;
    let mut y = ARCH_SPRING + 4;
    while y < POOL_BACK + 6 {
        let mut x = 120 + pick(index, 0, 1732, 10);
        while x < 280 {
            let (rx, ry) = (8 + pick(index, 1, 1732, 6), 5 + pick(index, 2, 1732, 3));
            rock(
                scene,
                (x, y),
                (rx, ry),
                3.0,
                0.2,
                WET_ROCK,
                1733 + index as u32 * 3,
                &behind,
            );
            x += rx * 2 - 1;
            index += 1;
        }
        y += 9;
    }
}

/// One of the crags, built of jointed blocks of rock in rough courses, the dark of the cracks
/// between them behind: the left crag in the gorge's shade and the right one lit, both duller
/// lower down, where less of the sky reaches.
fn crag(scene: &mut Canvas, left: bool) {
    let salt = if left { 1740 } else { 1760 };
    let ramp = if left { SHADED_CRAG } else { CRAG };
    let on = |x: i32, y: i32| on_crag(x, y, left);
    let (from, to) = if left { (0, 200) } else { (184, WIDTH) };
    for y in 0..POOL_BACK + 4 {
        for x in from..to {
            if on(x, y) {
                scene.set(x, y, mix(ramp.edge, rgb(0x0e100e), 0.3));
            }
        }
    }
    // Rough courses of blocks, each course its own depth, each block its own length, set a
    // little up or down, some sunk back into the crag and darker for it, and here and there
    // one missing, leaving a cleft.
    let mut row = 0;
    let mut y = -10;
    while y < POOL_BACK + 8 {
        let tall = 6 + pick(row, 0, salt, 9);
        let mut x = from - 14 + pick(row, 1, salt, 16);
        let mut index = 0;
        while x < to + 14 {
            let key = row * 31 + index;
            let wide = 8 + pick(key, 2, salt, 22) + if pick(key, 3, salt, 5) == 0 { 14 } else { 0 };
            if pick(key, 4, salt, 11) == 0 {
                // A cleft where a block has gone.
                x += wide / 2 + 1;
                index += 1;
                continue;
            }
            let gloom = (y as f32 / POOL_BACK as f32).clamp(0.0, 1.0) * 0.25
                + if pick(key, 5, salt, 4) == 0 { 0.3 } else { 0.0 };
            let shaded = Ramp {
                edge: ramp.edge,
                shadow: mix(ramp.shadow, ramp.edge, gloom),
                base: mix(ramp.base, ramp.shadow, gloom),
                light: mix(ramp.light, ramp.base, gloom),
                shine: mix(ramp.shine, ramp.light, gloom),
            };
            let moss = match pick(key, 6, salt, 4) {
                0 => 0.6,
                1 => 0.4,
                _ => 0.18,
            };
            let lift = pick(key, 7, salt, 5) - 2;
            rock(
                scene,
                (x + wide / 2, y + tall / 2 + lift),
                (wide / 2 + 1, tall / 2 + 1),
                2.6 + pick(key, 8, salt, 3) as f32 * 0.5,
                moss,
                shaded,
                salt + 2 + key as u32,
                &on,
            );
            x += wide + 1;
            index += 1;
        }
        y += tall;
        row += 1;
    }
    // The shade deepening into the gorge, towards the falls and down to the water.
    for y in 0..POOL_BACK + 4 {
        let edge = crag_edge(y, left);
        for x in from..to {
            if !on(x, y) {
                continue;
            }
            let into = 1.0 - ((x - edge).abs() as f32 / 70.0).clamp(0.0, 1.0);
            let low = (y as f32 / POOL_BACK as f32).clamp(0.0, 1.0);
            let shade = into * 0.28 + low * low * 0.12;
            let pixel = scene.get(x, y);
            scene.set(x, y, mix(pixel, rgb(0x101614), shade));
        }
    }
    // Tufts of grass along the crowns of the blocks, and bushes rooted in the clefts.
    for index in 0..90 {
        let x = from + pick(index, 0, salt + 900, to - from);
        let y = crag_top(x) + 4 + pick(index, 1, salt + 900, POOL_BACK - crag_top(x).max(0));
        if !on(x, y) || on(x, y - 1) && on(x, y - 2) && pick(index, 2, salt + 900, 3) != 0 {
            continue;
        }
        for dx in -2..=2 {
            let tall = 1 + pick(index, dx + 3, salt + 901, 3);
            for step in 0..tall {
                put(
                    scene,
                    x + dx,
                    y - step,
                    tone(FERN, if step == tall - 1 { 3 } else { 2 }),
                );
            }
        }
    }
    for index in 0..7 {
        let x = from + 16 + pick(index, 0, salt + 902, (to - from - 32).max(1));
        let y = crag_top(x) + 14 + pick(index, 1, salt + 902, 60);
        if on(x, y) && on(x, y + 6) {
            ellipse(scene, x + 1, y + 4, 6, 2, rgba(0x0e100e, 90));
            clump(
                scene,
                (x, y),
                4 + pick(index, 2, salt + 902, 3),
                CANOPY,
                salt + 903 + index as u32,
            );
        }
    }
    // The crag's face where it turns into the gorge: a dark seam down its inner edge.
    for y in crag_top(if left { 150 } else { 234 })..POOL_BACK + 2 {
        let edge = crag_edge(y, left);
        let x = if left { edge - 1 } else { edge + 1 };
        if on(x, y) {
            put(scene, x, y, ramp.edge);
            put(scene, x + if left { -1 } else { 1 }, y, rgba(0x0e100e, 90));
        }
    }
}

/// Ferns and hanging moss on the crags' ledges, and the trees growing out of them.
fn ledges(scene: &mut Canvas) {
    // Small trees clinging to the crags, leaning out over the gorge.
    for &(root, top, radius, salt) in &[
        ((112, 48), (122, 36), 8, 1752),
        ((272, 46), (262, 34), 8, 1753),
    ] {
        line(scene, root, top, BARK.edge);
        line(scene, (root.0 + 1, root.1), (top.0 + 1, top.1), BARK.light);
        line(scene, (root.0 + 2, root.1), (top.0 + 2, top.1), BARK.shadow);
        clump(scene, top, radius, CANOPY, salt);
    }
    // Fern sprays on the ledges.
    for (index, &(x, y, lean)) in [
        (40, 76, -1.0),
        (106, 92, 1.0),
        (134, 104, 1.0),
        (64, 104, -1.0),
        (258, 94, -1.0),
        (306, 70, 1.0),
        (332, 104, 1.0),
        (246, 110, -1.0),
        (16, 112, 1.0),
        (368, 86, -1.0),
        (120, 62, 1.0),
        (270, 60, -1.0),
    ]
    .iter()
    .enumerate()
    {
        for (step, offset) in [-50.0, -10.0, 30.0].iter().enumerate() {
            let angle = -90.0 + offset * lean + 25.0 * lean;
            let length = 7.0 + pick(index as i32, step as i32, 1755, 5) as f32;
            frond(scene, (x, y), (angle, length), 1 + (step as i32 % 3));
        }
    }
    // Moss hanging in curtains from the rock either side of the falls, where the spray keeps it
    // wet.
    for (index, &(x, y, long)) in [
        (168, 54, 8),
        (171, 70, 6),
        (165, 86, 9),
        (215, 50, 6),
        (219, 66, 9),
        (224, 84, 7),
        (160, 100, 5),
        (228, 102, 6),
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
            if dy < long - 2 && pick(index as i32, dy, 1756, 3) == 0 {
                put(scene, x + 1, y + dy, MOSS.shadow);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The arch
// ---------------------------------------------------------------------------------------------

/// How far from the arch's middle its opening reaches at row `y`, if the row is within it.
fn opening(y: i32) -> Option<f32> {
    let rise = (ARCH_SPRING - y) as f32 / ARCH_RISE as f32;
    if !(0.0..=1.0).contains(&rise) {
        return None;
    }
    Some(ARCH_SPAN as f32 * (1.0 - rise * rise).sqrt())
}

/// The old arch across the gorge, the water coming out from under it: dressed stone gone grey
/// and green, its voussoirs radiating round the opening and lit along their upper left edges,
/// its ends set into the tops of the crags, moss along the top and ivy hanging down its face.
fn arch(scene: &mut Canvas) {
    let (left, right) = (ARCH_MIDDLE - ARCH_SPAN - 26, ARCH_MIDDLE + ARCH_SPAN + 26);
    let top = ARCH_SPRING - ARCH_RISE - 9;
    // How low each end goes: down to where it rests on the crag.
    let foot = |x: i32| crag_top(x).max(ARCH_SPRING + 2);
    for x in left..=right {
        for y in top..=foot(x) {
            let off = (x - ARCH_MIDDLE) as f32;
            if let Some(reach) = opening(y)
                && off.abs() < reach
            {
                continue;
            }
            if y > ARCH_SPRING + 2 && off.abs() < ARCH_SPAN as f32 + 4.0 {
                continue;
            }
            let (dx, dy) = (off, (ARCH_SPRING - y) as f32);
            let around = dy.atan2(dx);
            let reach = (dx * dx / (ARCH_SPAN as f32).powi(2)
                + dy * dy / (ARCH_RISE as f32).powi(2))
            .sqrt();
            let in_ring = reach < 1.45 && dy >= 0.0;
            let color = if y == top {
                ARCH.edge
            } else if y == top + 1 {
                ARCH.shine
            } else if y <= top + 3 {
                // The parapet's coping, a course along the top.
                if (x - left) % 9 == 0 {
                    ARCH.shadow
                } else {
                    ARCH.light
                }
            } else if in_ring {
                // The voussoirs: a ring of wedges round the opening.
                let wedge = (around / PI * 13.0).floor() as i32;
                let within = (around / PI * 13.0).fract();
                let joint = !(0.08..0.92).contains(&within) || (reach - 1.0).abs() < 0.04;
                if joint {
                    ARCH.edge
                } else if within < 0.3 {
                    ARCH.light
                } else if pick(wedge, 0, 1761, 4) == 0 {
                    ARCH.shadow
                } else {
                    ARCH.base
                }
            } else {
                // The spandrels and the ends: coursed blocks.
                let course = (y - top - 4).div_euclid(5);
                let shifted = x + if course % 2 == 0 { 0 } else { 5 };
                let joint = (y - top - 4).rem_euclid(5) == 4 || shifted.rem_euclid(10) == 0;
                if joint {
                    ARCH.shadow
                } else if (y - top - 4).rem_euclid(5) == 0 {
                    ARCH.light
                } else {
                    ARCH.base
                }
            };
            let weathered = if chance(x, y, 1762, 26) {
                mix(color, ARCH.edge, 0.35)
            } else {
                color
            };
            // In the gorge's shade on its right-hand end.
            let shaded = if x > ARCH_MIDDLE + ARCH_SPAN + 12 {
                mix(weathered, ARCH.shadow, 0.25)
            } else {
                weathered
            };
            scene.set(x, y, shaded);
        }
        put(scene, x, foot(x) + 1, rgba(0x0e100e, 120));
    }
    // The ends, where they meet the air, outlined.
    for y in top..=foot(left) {
        put(scene, left, y, ARCH.edge);
    }
    for y in top..=foot(right) {
        put(scene, right, y, ARCH.edge);
    }
    // The opening's own edge: in shade on the right of the soffit, lit on the left.
    for y in ARCH_SPRING - ARCH_RISE..=ARCH_SPRING {
        if let Some(reach) = opening(y) {
            let (l, r) = (
                ARCH_MIDDLE - reach.round() as i32,
                ARCH_MIDDLE + reach.round() as i32,
            );
            put(scene, l - 1, y, ARCH.edge);
            put(scene, l - 2, y, ARCH.light);
            put(scene, r, y, ARCH.edge);
            put(scene, r + 1, y, ARCH.shadow);
        }
    }
    // Moss along the top and lichen on the stones.
    for x in left + 1..right {
        if patches(x, 0, (6, 1), 1763) > 0.35 {
            put(scene, x, top, MOSS.light);
            put(scene, x, top - 1, MOSS.base);
            if chance(x, top, 1764, 90) {
                put(scene, x, top - 2, MOSS.light);
            }
        }
    }
    for index in 0..24 {
        let x = left + 3 + pick(index, 0, 1765, right - left - 6);
        let y = top + 5 + pick(index, 1, 1765, ARCH_SPRING - top - 4);
        if opening(y).is_some_and(|reach| ((x - ARCH_MIDDLE) as f32).abs() < reach + 1.0) {
            continue;
        }
        put(scene, x, y, rgb(0xb8b878));
        put(scene, x + 1, y, rgb(0x9a9a62));
    }
    // Ivy trailing down its face and the crag beside it, in strands of leaves.
    for (index, &(x, long)) in [
        (left + 6, 30),
        (left + 15, 16),
        (right - 8, 34),
        (right - 18, 14),
        (ARCH_MIDDLE - ARCH_SPAN - 8, 9),
        (ARCH_MIDDLE + ARCH_SPAN + 6, 11),
    ]
    .iter()
    .enumerate()
    {
        let sway = |dy: i32| ((dy as f32 / 5.0 + index as f32).sin() * 1.5).round() as i32;
        for dy in 0..long {
            let (px, py) = (x + sway(dy), top + 2 + dy);
            put(scene, px, py, IVY.shadow);
            if dy % 3 == 0 {
                let side = if dy % 6 == 0 { -1 } else { 1 };
                put(scene, px + side, py, IVY.light);
                put(scene, px + side * 2, py, IVY.base);
                put(scene, px + side, py + 1, IVY.edge);
                if pick(index as i32, dy, 1766, 3) == 0 {
                    put(scene, px + side, py - 1, IVY.shine);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The falls and the pool
// ---------------------------------------------------------------------------------------------

/// The edges of the falls at row `y`, wavering as the sheet of water twists.
fn falls_edges(y: i32) -> (i32, i32) {
    let t = ((y - LIP.1 as i32) as f32 / (FOOT.1 - LIP.1)).clamp(0.0, 1.0);
    let waver = |salt: u32| ((looped(y as f32 / 6.0, 64, salt) - 0.5) * 3.0 * t).round() as i32;
    (
        (FALLS_TOP.0 as f32 + (FALLS_FOOT.0 - FALLS_TOP.0) as f32 * t).round() as i32 + waver(1771),
        (FALLS_TOP.1 as f32 + (FALLS_FOOT.1 - FALLS_TOP.1) as f32 * t).round() as i32 + waver(1772),
    )
}

/// The falls: a sheet of water bending over the lip, glassy there, breaking into white streaks
/// as it falls, lit on its left and in shade on its right, and its shade on the rock behind.
fn falls(scene: &mut Canvas) {
    let (top, foot) = (LIP.1 as i32, FOOT.1 as i32);
    for y in top..foot {
        let t = (y - top) as f32 / (foot - top) as f32;
        let (left, right) = falls_edges(y);
        put(scene, right + 1, y, rgba(0x101a18, 110));
        put(scene, right + 2, y, rgba(0x101a18, 60));
        for x in left..=right {
            let color = if x == left {
                FALLS.shadow
            } else if x == right {
                FALLS.edge
            } else if y < top + 2 {
                FALLS.shine
            } else if y < top + 6 {
                if x < left + 4 {
                    FALLS.shine
                } else if x > right - 4 {
                    FALLS.base
                } else {
                    FALLS.light
                }
            } else {
                let fall = (y + pick(x, 0, 1773, 11)).div_euclid(6);
                let mut level = match noise(x, fall, 1774) % 7 {
                    0 => 5,
                    1 | 2 => 4,
                    3 | 4 => 3,
                    _ => 2,
                } + i32::from(x < left + 5)
                    - i32::from(x > right - 5);
                if chance(x, fall, 1775, (t * 90.0) as u32) {
                    level += 1;
                }
                if level >= 5 { FOAM } else { tone(FALLS, level) }
            };
            scene.set(x, y, color);
        }
    }
    // The lip itself: a wet ledge of rock the water bends over.
    for x in FALLS_TOP.0 - 8..=FALLS_TOP.1 + 8 {
        if !(FALLS_TOP.0..=FALLS_TOP.1).contains(&x) {
            put(scene, x, top, WET_ROCK.light);
            put(scene, x, top + 1, WET_ROCK.edge);
        }
    }
}

/// Whether a pixel is on the plunge pool: everything from the foot of the crags to the shore.
fn is_pool(x: i32, y: i32) -> bool {
    y >= POOL_BACK - 2 && y < shore_line(x)
}

/// Where the shore begins at column `x`: the water's edge, coming forward towards the middle,
/// where the wading stone juts out.
fn shore_line(x: i32) -> i32 {
    let from_middle = ((x - ARCH_MIDDLE) as f32 / 190.0).clamp(-1.0, 1.0);
    let curve = 176.0 - 30.0 * from_middle * from_middle;
    let wobble = (looped(x as f32 / 11.0, 64, 1781) - 0.5) * 5.0;
    (curve + wobble).round() as i32
}

/// The plunge pool: deep green under the falls and out in the middle, shallower and lighter
/// towards the shore, the crags dark in it, lit along its ripples.
fn pool(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = EDDY;
    for y in POOL_BACK - 2..HEIGHT {
        for x in 0..WIDTH {
            if !is_pool(x, y) {
                continue;
            }
            let shore = shore_line(x);
            let shallow = 1.0 - ((shore - y) as f32 / 18.0).clamp(0.0, 1.0);
            let (u, v) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
            let deep = (1.0 - (u * u + v * v).sqrt() * 0.5).clamp(0.0, 1.0);
            // Reflections of the crags, dark, rippled.
            let mirrored = POOL_BACK - (y - POOL_BACK);
            let reflected =
                y < POOL_BACK + 16 && (on_crag(x, mirrored, true) || on_crag(x, mirrored, false));
            let mut level = 2.0 + shallow * 1.6 - deep * 0.9;
            if reflected {
                level -= 0.8;
            }
            let ripple = (y + pick(x.div_euclid(7), y, 1782, 2)).rem_euclid(4) == 0;
            if ripple && chance(x.div_euclid(2), y, 1783, 90) {
                level += 0.8;
            }
            let mut color = tone(POOL, level.round() as i32);
            if shallow > 0.6 && chance(x, y, 1784, 30) {
                color = mix(color, SAND.base, 0.3);
            }
            scene.set(x, y, color);
        }
    }
    // The waterline at the foot of the crags.
    for x in 0..WIDTH {
        put(scene, x, POOL_BACK - 2, rgba(0xc8dcd2, 70));
    }
    // Rings of foam spreading across the pool from the falls, fainter as they go: the eddy.
    for (index, (rx, ry)) in [(30, 6), (46, 9), (64, 13), (84, 17)]
        .into_iter()
        .enumerate()
    {
        ring(
            scene,
            (FOOT.0 as i32, FOOT.1 as i32 + 6 + index as i32 * 2),
            (rx, ry),
            rgba(0xa4c8bc, 120 - index as u8 * 22),
            1785 + index as u32,
        );
    }
    // Streaks of foam carried round on the current.
    for index in 0..40 {
        let angle = pick(index, 0, 1786, 360) as f32 / 360.0 * TAU;
        let (x, y) = (
            cx + angle.sin() * (rx - 6.0 + pick(index, 1, 1786, 14) as f32),
            cy - angle.cos() * (ry - 2.0 + pick(index, 2, 1786, 5) as f32),
        );
        let long = 2 + pick(index, 3, 1786, 4);
        if is_pool(x as i32, y as i32) {
            hline(scene, x as i32, y as i32, long, rgba(0xc8dcd2, 140));
        }
    }
}

/// Where the falls land: a churning mound of foam, lit on top, and the spray rising off it in
/// soft billows.
fn foot(scene: &mut Canvas) {
    let (fx, fy) = (FOOT.0 as i32, FOOT.1 as i32);
    let (rx, ry) = (24.0f32, 6.0f32);
    hline(scene, fx - 20, fy + 6, 41, rgba(0x14302e, 80));
    for y in fy - 7..=fy + 6 {
        for x in fx - 26..=fx + 26 {
            let (u, v) = ((x - fx) as f32 / rx, (y - fy) as f32 / ry);
            let lumps = (noise(x.div_euclid(3), 0, 1791) % 100) as f32 / 140.0 * (-v).max(0.0);
            let r = u * u + v * v + lumps;
            if r > 1.0 {
                continue;
            }
            let clot = noise(x.div_euclid(2), y, 1792) % 10;
            let lit = v - u * 0.4;
            let color = if r > 0.8 && v > 0.2 {
                FALLS.shadow
            } else if lit < -0.3 {
                if clot < 7 { FOAM } else { FALLS.shine }
            } else if lit < 0.4 {
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
    // The spray, billowing up over the foot and drifting right on the air.
    for (dx, dy, rx, ry, alpha) in [
        (0, -8, 30, 10, 26),
        (10, -18, 26, 9, 18),
        (22, -28, 22, 8, 12),
        (-12, -14, 18, 7, 14),
    ] {
        ellipse(scene, fx + dx, fy + dy, rx, ry, rgba(0xdce8e2, alpha));
    }
    for index in 0..60 {
        let x = fx - 26 + pick(index, 0, 1793, 60);
        let y = fy - 4 - pick(index, 1, 1793, 30);
        put(scene, x, y, rgba(0xeef4ee, 110));
    }
}

/// A rainbow in the spray, to the right of the falls where the sun from the upper left comes
/// through it: faint bands, red outermost.
fn rainbow(scene: &mut Canvas) {
    const BANDS: [u32; 6] = [0xe06050, 0xf0a048, 0xf0e070, 0x80c870, 0x6090e0, 0x9070c8];
    let (cx, cy) = (FOOT.0 as i32 + 30, FOOT.1 as i32 + 10);
    for (index, color) in BANDS.iter().enumerate() {
        let radius = 30.0 - index as f32 * 1.4;
        for step in 0..160 {
            let angle = PI + step as f32 / 160.0 * PI * 0.62;
            let (x, y) = (
                cx + (angle.cos() * radius).round() as i32,
                cy + (angle.sin() * radius * 0.95).round() as i32,
            );
            // Strongest where it crosses the spray, fading at its ends.
            let fade = (step as f32 / 160.0 * PI).sin();
            let alpha = (fade * 80.0) as u8;
            if alpha > 6 && y < FOOT.1 as i32 + 2 {
                put(scene, x, y, rgba(*color, alpha));
            }
        }
    }
}

/// Mossy boulders breaking the pool's surface at its edges, each with a ripple round its foot.
fn boulders(scene: &mut Canvas) {
    let anywhere = |_: i32, _: i32| true;
    for &((cx, cy), (rx, ry), salt) in &[
        ((44, 134), (16, 9), 1801),
        ((74, 124), (9, 6), 1802),
        ((342, 130), (15, 10), 1803),
        ((312, 121), (8, 5), 1804),
        ((114, 121), (6, 4), 1805),
        ((272, 120), (7, 4), 1806),
    ] {
        ring(
            scene,
            (cx, cy + ry - 1),
            (rx + 3, 2),
            rgba(0xa4c8bc, 120),
            salt,
        );
        rock(
            scene,
            (cx, cy),
            (rx, ry),
            2.4,
            0.6,
            SLATE,
            salt + 10,
            &anywhere,
        );
        // Dark and wet at the waterline.
        for x in cx - rx..=cx + rx {
            let pixel = scene.get(x, cy + ry - 2);
            scene.set(x, cy + ry - 2, mix(pixel, POOL.edge, 0.4));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The shore
// ---------------------------------------------------------------------------------------------

/// The shingle along the front: damp sand packed with rounded pebbles of every size, each lit
/// on its upper left, darker and wet towards the water, with flat stones set in it and the big
/// wading stone jutting into the pool where the eddy comes past.
fn shore(scene: &mut Canvas) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let line = shore_line(x);
            if y < line {
                continue;
            }
            let wet = (1.0 - (y - line) as f32 / 8.0).clamp(0.0, 1.0);
            let level = if patches(x, y, (4, 3), 1810) > 0.6 {
                1
            } else {
                2
            };
            let color = mix(tone(SAND, level), POOL.shadow, wet * 0.5);
            scene.set(x, y, color);
        }
    }
    // Pebbles, packed in rows, each its own size and stone.
    for row in 0..((HEIGHT - 130) / 3) {
        let y = 130 + row * 3;
        let mut x = pick(row, 0, 1811, 4);
        let mut index = 0;
        while x < WIDTH {
            let (rx, ry) = (
                1 + pick(row * 97 + index, 1, 1811, 3),
                1 + pick(row, index, 1812, 2),
            );
            let at = (x + rx, y + pick(index, row, 1813, 2));
            let line = shore_line(at.0);
            if at.1 > line + 1 {
                let ramp = match pick(index, row, 1814, 5) {
                    0 => SLATE,
                    1 => SAND,
                    _ => SHINGLE,
                };
                pebble(scene, at, (rx, ry), ramp, (at.1 - line) < 6);
            }
            x += rx * 2 + 1 + pick(index, row, 1815, 2);
            index += 1;
        }
    }
    // The foam lapping at the shingle.
    for x in 0..WIDTH {
        let line = shore_line(x);
        if chance(x, line, 1816, 150) {
            put(scene, x, line, rgba(0xdce8e2, 180));
        }
        put(scene, x, line - 1, rgba(0x14302e, 60));
    }
    // Flat stones set in the shingle, the wading stone biggest, jutting into the water.
    for &((cx, cy), (rx, ry), thick, salt) in &[
        ((192, 191), (20, 6), 3, 1820),
        ((120, 196), (9, 3), 2, 1821),
        ((276, 199), (10, 3), 2, 1822),
        ((84, 182), (8, 3), 2, 1823),
        ((304, 184), (9, 3), 2, 1824),
        ((50, 204), (12, 4), 2, 1825),
        ((338, 205), (10, 3), 2, 1826),
    ] {
        flat_stone(scene, (cx, cy), (rx, ry), thick, salt);
    }
}

/// One pebble: a little rounded stone lit from the upper left, outlined in its own darkest
/// tone, darker if it is wet.
fn pebble(scene: &mut Canvas, (cx, cy): (i32, i32), (rx, ry): (i32, i32), ramp: Ramp, wet: bool) {
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (u, v) = (
                (x - cx) as f32 / (rx as f32 + 0.5),
                (y - cy) as f32 / (ry as f32 + 0.5),
            );
            let r = u * u + v * v;
            if r > 1.0 {
                continue;
            }
            let lit = u + v;
            let mut color = if r > 0.65 && (u > -0.2 || v > 0.0) {
                ramp.edge
            } else if lit < -0.6 {
                ramp.light
            } else if lit < 0.4 {
                ramp.base
            } else {
                ramp.shadow
            };
            if wet {
                color = mix(color, POOL.shadow, 0.35);
            }
            put(scene, x, y, color);
        }
    }
    if rx > 1 {
        put(scene, cx - 1, cy - ry + 1, ramp.shine);
    }
}

/// A flat stone in the shingle: its face lit from the upper left, `thick` rows of its side in
/// shade beneath, outlined in its own darkest tone.
fn flat_stone(scene: &mut Canvas, at: (i32, i32), size: (i32, i32), thick: i32, salt: u32) {
    let face = |x: i32, y: i32| {
        let (u, v) = (
            (x - at.0) as f32 / (size.0 as f32 + 0.4),
            (y - at.1) as f32 / (size.1 as f32 + 0.4),
        );
        u * u + v * v <= 1.0
    };
    let solid = |x: i32, y: i32| (0..=thick).any(|down| face(x, y - down));
    // Its shadow on the shingle, down and to the right.
    for y in at.1..=at.1 + size.1 + thick + 2 {
        for x in at.0 - size.0..=at.0 + size.0 + 3 {
            if solid(x - 2, y - 1) && !solid(x, y) {
                put(scene, x, y, rgba(0x1a1a14, 80));
            }
        }
    }
    for y in at.1 - size.1 - 1..=at.1 + size.1 + thick + 1 {
        for x in at.0 - size.0 - 1..=at.0 + size.0 + 1 {
            if !solid(x, y) {
                continue;
            }
            let rim = !solid(x - 1, y) || !solid(x + 1, y) || !solid(x, y - 1) || !solid(x, y + 1);
            let color = if rim {
                SLATE.edge
            } else if !face(x, y) {
                SLATE.shadow
            } else {
                let (u, v) = (
                    (x - at.0) as f32 / size.0 as f32,
                    (y - at.1) as f32 / size.1 as f32,
                );
                let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
                let lit = -0.6 * u - 0.5 * v + 0.5 * jitter;
                if chance(x, y, salt + 1, 30) {
                    SLATE.shadow
                } else if lit > 0.75 {
                    SLATE.shine
                } else if lit > 0.25 {
                    SLATE.light
                } else {
                    SLATE.base
                }
            };
            put(scene, x, y, color);
        }
    }
}

/// Ferns at the feet of the crags either side of the beach.
fn ferns(scene: &mut Canvas) {
    for (index, &(x, y)) in [
        (10, 150),
        (28, 156),
        (46, 162),
        (352, 150),
        (370, 156),
        (336, 162),
        (8, 184),
        (376, 180),
    ]
    .iter()
    .enumerate()
    {
        let lean = if x < ARCH_MIDDLE { 1.0 } else { -1.0 };
        for (step, offset) in [-60.0, -25.0, 10.0, 40.0].iter().enumerate() {
            let angle = -90.0 + offset * lean;
            let length = 10.0 + pick(index as i32, step as i32, 1840, 6) as f32;
            frond(
                scene,
                (x, y),
                (angle, length),
                1 + ((step + index) as i32 % 3),
            );
        }
    }
}

/// A bough of leaves reaching in over the top left corner.
fn bough(scene: &mut Canvas) {
    let points = [(-4, 12), (18, 8), (42, 12), (60, 6)];
    for pair in points.windows(2) {
        line(scene, pair[0], pair[1], BARK.edge);
        line(
            scene,
            (pair[0].0, pair[0].1 - 1),
            (pair[1].0, pair[1].1 - 1),
            BARK.light,
        );
    }
    for (index, &(x, y)) in [(4, 16), (16, 4), (30, 14), (46, 4), (60, 12)]
        .iter()
        .enumerate()
    {
        clump(
            scene,
            (x, y),
            6 + index as i32 % 3,
            CANOPY,
            1850 + index as u32,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::falls::SHORE;

    #[test]
    fn the_backdrop_fills_every_pixel_and_the_eddy_is_on_the_water() {
        let scene = backdrop();
        assert!(scene.pixels().iter().all(|pixel| pixel.a == 255));
        let (cx, cy, rx, ry) = EDDY;
        for step in 0..48 {
            let angle = step as f32 / 48.0 * TAU;
            let (x, y) = (cx + angle.sin() * rx, cy - angle.cos() * ry);
            assert!(
                is_pool(x as i32, y as i32),
                "the eddy runs aground at {x}, {y}"
            );
        }
        // Where the party stands is shingle, not water.
        let (left, top, right, bottom) = SHORE;
        for x in [left, (left + right) / 2.0, right] {
            for y in [top + 4.0, bottom] {
                assert!(
                    !is_pool(x as i32, y as i32),
                    "the shore is under water at {x}, {y}"
                );
            }
        }
    }
}
