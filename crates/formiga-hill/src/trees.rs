//! Trees as the hedgerow and the meadow paint them, far enough off to take the distance's haze:
//! a bough's curve, a clump of leaves and a crown. Both places paint them with the same code, so a
//! tree reads as one tree wherever it stands.

use crate::kit;
use crate::paint::{Ramp, ellipse, mix, pick, put, rgb};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

/// The colour the hedgerow and the meadow haze toward with distance.
pub const DISTANCE: Rgba = rgb(0xa8cac4);

/// `color` as seen through `amount` of the distance.
pub fn hazed(color: Rgba, amount: f32) -> Rgba {
    mix(color, DISTANCE, amount)
}

/// A point along a quadratic curve from `from` through towards `bend` to `to`.
pub fn curve(from: (i32, i32), bend: (i32, i32), to: (i32, i32), t: f32) -> (i32, i32) {
    let along = |a: i32, b: i32, c: i32| {
        ((1.0 - t) * (1.0 - t) * a as f32 + 2.0 * (1.0 - t) * t * b as f32 + t * t * c as f32)
            .round() as i32
    };
    (along(from.0, bend.0, to.0), along(from.1, bend.1, to.1))
}

/// A clump of leaves, its rim scalloped into smaller clusters, lit from the upper left and lost
/// in the distance by `haze`.
pub fn clump(
    scene: &mut Canvas,
    centre: (i32, i32),
    radius: i32,
    (ramp, haze): (Ramp, f32),
    salt: u32,
) {
    kit::clump(scene, centre, radius, ramp, salt, |color, _| {
        hazed(color, haze)
    });
}

/// A tree's crown from the side: a rounded mass of leaves, its rim scalloped and outlined in its
/// own darkest green, filled with clumps of leaves back to front, each with a dark underside and
/// a lit crescent on its upper left, brighter towards the upper left of the whole crown.
pub fn crown(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    (rx, ry): (i32, i32),
    (ramp, haze): (Ramp, f32),
    salt: u32,
) {
    let tint = |color: Rgba| hazed(color, haze);
    let rim = (rx + ry) * 3 / 2;
    for step in 0..rim {
        let angle = step as f32 / rim as f32 * TAU;
        let wobble = pick(step, 0, salt, 3) as f32;
        let size = 2 + pick(step, 1, salt, 2) + rx.min(ry) / 10;
        let (x, y) = (
            cx + (angle.cos() * (rx as f32 - 1.0 + wobble)).round() as i32,
            cy + (angle.sin() * (ry as f32 - 1.0 + wobble)).round() as i32,
        );
        ellipse(scene, x, y, size, size, tint(ramp.edge));
    }
    ellipse(scene, cx, cy, rx, ry, tint(ramp.shadow));
    let mut clumps = Vec::new();
    for (row, gy) in (cy - ry..=cy + ry + 1).step_by(3).enumerate() {
        let offset = if row % 2 == 0 { 0 } else { 2 };
        for gx in (cx - rx - 1..=cx + rx + 1).step_by(4) {
            let x = gx + offset + pick(gx, gy, salt + 1, 3) - 1;
            let y = gy + pick(gx, gy, salt + 2, 3) - 1;
            let (dx, dy) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            if dx * dx + dy * dy <= 0.92 {
                clumps.push((x, y, dx * 0.8 + dy * 1.1));
            }
        }
    }
    for (x, y, light) in clumps {
        let size = 2 + pick(x, y, salt + 3, 2);
        let fleck = (pick(x, y, salt + 4, 5) - 2) as f32 * 0.1;
        let (dark, mid, bright) = match light + fleck {
            l if l < -0.6 => (ramp.base, ramp.light, ramp.shine),
            l if l < 0.1 => (ramp.shadow, ramp.base, ramp.light),
            l if l < 0.7 => (mix(ramp.edge, ramp.shadow, 0.5), ramp.shadow, ramp.base),
            _ => (ramp.edge, mix(ramp.edge, ramp.shadow, 0.5), ramp.shadow),
        };
        ellipse(scene, x + 1, y + 1, size, size - 1, tint(dark));
        ellipse(scene, x, y, size - 1, size - 1, tint(mid));
        for py in y - size..=y + size {
            for px in x - size..=x + size {
                let (dx, dy) = (px - x, py - y);
                if dx * dx + dy * dy < size * size
                    && dx + dy < -size / 2 - pick(px, py, salt, 2) + 1
                {
                    put(scene, px, py, tint(bright));
                }
            }
        }
    }
}
