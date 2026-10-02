//! The pieces areas are built from: plastered walls in timber frames, tiled roofs, glazed
//! windows, window boxes, dressed stone, and bushes. Each is painted the same way wherever it
//! stands, so the Hill reads as one place.

use crate::materials::*;
use crate::paint::{
    bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgba, vline,
};
use formiga_art::Canvas;

pub fn bush(scene: &mut Canvas, cx: i32, cy: i32, radius: i32, salt: u32) {
    ellipse(scene, cx, cy + 1, radius, radius - 1, LEAF.edge);
    ellipse(scene, cx, cy, radius - 1, radius - 2, LEAF.shadow);
    ellipse(scene, cx - 1, cy - 1, radius - 2, radius - 3, LEAF.base);
    // Leaf clusters, lit from the upper left.
    for y in cy - radius..=cy + radius {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy > (radius - 2) * (radius - 2) {
                continue;
            }
            if dx + dy < -radius / 2 && chance(x, y, 40 + salt, 110) {
                put(scene, x, y, LEAF.light);
            } else if dx + dy < -radius && chance(x, y, 41 + salt, 60) {
                put(scene, x, y, LEAF.shine);
            } else if dx + dy > radius / 2 && chance(x, y, 42 + salt, 90) {
                put(scene, x, y, LEAF.shadow);
            }
        }
    }
    if salt % 3 != 1 {
        let blossom = BLOSSOMS[(salt as usize * 2 + 1) % BLOSSOMS.len()];
        for petal in 0..4 {
            let x = cx - radius / 2 + (noise(petal, cy, salt) % radius.max(1) as u32) as i32;
            let y = cy - radius / 2 + (noise(cx, petal, salt) % (radius as u32).max(1)) as i32;
            put(scene, x, y, blossom);
        }
    }
}
pub fn plaster(scene: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    for py in y..y + height {
        for px in x..x + width {
            // Soft blotches at two-pixel grain read as limewash rather than as noise.
            let grain = noise(px / 2, py / 2, 71) % 100;
            let color = match grain {
                0..=6 => PLASTER.light,
                7..=11 => mix(PLASTER.base, PLASTER.shadow, 0.5),
                _ => PLASTER.base,
            };
            scene.set(px, py, color);
        }
    }
}
pub fn timber(scene: &mut Canvas, x: i32, y: i32, width: i32, height: i32, upright: bool) {
    rect(scene, x, y, width, height, TIMBER.base);
    if upright {
        vline(scene, x, y, height, TIMBER.edge);
        vline(scene, x + 1, y, height, TIMBER.light);
        vline(scene, x + width - 1, y, height, TIMBER.shadow);
        for py in y..y + height {
            if chance(x, py, 72, 40) {
                put(scene, x + 2, py, TIMBER.shadow);
            }
        }
    } else {
        hline(scene, x, y, width, TIMBER.light);
        hline(scene, x, y + height - 1, width, TIMBER.edge);
        for px in x..x + width {
            if chance(px, y, 73, 30) {
                put(scene, px, y + 1, TIMBER.shadow);
            }
        }
    }
}
/// A tiled roof inside `outline`, courses running down from `top` to `bottom`.
pub fn roof(scene: &mut Canvas, outline: &[(i32, i32)], top: i32, bottom: i32) {
    polygon(scene, outline, |x, y| {
        let row = (y - top) / 4;
        let within = (y - top) % 4;
        let shifted = x + if row % 2 == 1 { 3 } else { 0 };
        let tile = shifted.div_euclid(6);
        let across = shifted.rem_euclid(6);
        // Lower courses fall a little into shade, as a roof curves away from the sky.
        let depth = (y - top) as f32 / (bottom - top).max(1) as f32;
        let base = match noise(tile, row, 81) % 6 {
            0 => TILE.light,
            1 => mix(TILE.base, TILE.shadow, 0.5),
            _ => TILE.base,
        };
        let base = mix(base, TILE.shadow, depth * 0.35);
        Some(match within {
            0 => TILE.edge,
            1 if across == 1 => TILE.shine,
            1 => mix(base, TILE.light, 0.5),
            3 if across == 0 || across == 5 => TILE.shadow,
            _ if across == 0 => mix(base, TILE.shadow, 0.6),
            _ => base,
        })
    });
    for pair in outline.windows(2) {
        line(scene, pair[0], pair[1], TILE.edge);
    }
}
pub fn ridge_tiles(scene: &mut Canvas, left: i32, right: i32, y: i32) {
    rect(scene, left, y - 2, right - left, 3, TILE.base);
    hline(scene, left, y - 3, right - left, TILE.edge);
    hline(scene, left, y - 2, right - left, TILE.light);
    for x in (left..right).step_by(6) {
        vline(scene, x, y - 2, 3, TILE.shadow);
        put(scene, x + 1, y - 2, TILE.shine);
    }
}
pub fn window(scene: &mut Canvas, x: i32, y: i32, width: i32, height: i32, columns: i32) {
    bevel(scene, x, y, width, height, TRIM);
    let (inner_x, inner_y, inner_w, inner_h) = (x + 2, y + 2, width - 4, height - 4);
    for py in inner_y..inner_y + inner_h {
        // The sky, reflected: lightest at the top.
        let t = (py - inner_y) as f32 / inner_h as f32;
        let pane = if t < 0.25 {
            GLASS[2]
        } else if t < 0.6 {
            GLASS[1]
        } else {
            mix(GLASS[1], GLASS[0], (t - 0.6) * 2.0)
        };
        hline(scene, inner_x, py, inner_w, pane);
    }
    hline(scene, inner_x, inner_y, inner_w, GLASS[0]);
    // Glazing bars.
    let mid_y = inner_y + inner_h / 2;
    hline(scene, inner_x, mid_y, inner_w, TRIM.base);
    hline(scene, inner_x, mid_y + 1, inner_w, TRIM.shadow);
    for column in 1..columns {
        let bar = inner_x + inner_w * column / columns;
        vline(scene, bar, inner_y, inner_h, TRIM.base);
    }
    // A glint across each pane.
    for column in 0..columns {
        let pane_x = inner_x + inner_w * column / columns + 2;
        for (dx, dy) in [(0, 3), (1, 2), (2, 1), (3, 0)] {
            put(scene, pane_x + dx, inner_y + 1 + dy, GLASS[3]);
        }
        put(scene, pane_x + 1, inner_y + 4, rgba(0xe9f6f7, 140));
    }
    // The sill.
    bevel(scene, x - 2, y + height - 1, width + 4, 3, TRIM);
}
pub fn flower_box(scene: &mut Canvas, x: i32, y: i32, width: i32) {
    for leaf in 0..width {
        let tall = 2 + (noise(leaf, y, 101) % 3) as i32;
        vline(scene, x + leaf, y - tall, tall, LEAF.base);
        put(scene, x + leaf, y - tall, LEAF.light);
    }
    for index in 0..7 {
        let fx = x + 2 + index * 3 + (noise(index, y, 102) % 2) as i32;
        let fy = y - 3 - (noise(index, y, 103) % 2) as i32;
        let blossom = BLOSSOMS[(index as usize * 3) % 4];
        put(scene, fx, fy, blossom);
        put(scene, fx + 1, fy, blossom);
        put(scene, fx, fy + 1, mix(blossom, LEAF.shadow, 0.4));
    }
    bevel(scene, x, y, width, 5, TIMBER);
    hline(scene, x + 1, y + 2, width - 2, TIMBER.shadow);
}

/// How a wall is laid: how tall each course of stone is, and how long each stone.
#[derive(Clone, Copy)]
pub struct Courses {
    pub tall: i32,
    pub long: i32,
}

/// Courses of dressed stone with mortar between, each stone a slightly different grey, filling
/// `(left, top, width, height)`.
pub fn stonework(scene: &mut Canvas, area: (i32, i32, i32, i32), courses: Courses, salt: u32) {
    let (left, top, width, height) = area;
    let (course, block) = (courses.tall, courses.long);
    for y in top..top + height {
        let row = (y - top) / course;
        let within = (y - top) % course;
        for x in left..left + width {
            let shifted = x + if row % 2 == 1 { block / 2 } else { 0 };
            let stone = shifted.div_euclid(block);
            let across = shifted.rem_euclid(block);
            let color = if within == course - 1 || across == 0 {
                STONE.edge
            } else {
                let base = match noise(stone, row, salt) % 4 {
                    0 => mix(STONE.base, STONE.shadow, 0.5),
                    1 => mix(STONE.base, STONE.light, 0.3),
                    _ => STONE.base,
                };
                if within == 0 || across == 1 {
                    mix(base, STONE.light, 0.6)
                } else if within == course - 2 || across == block - 1 {
                    mix(base, STONE.shadow, 0.6)
                } else if chance(x, y, salt + 1, 18) {
                    STONE.shadow
                } else {
                    base
                }
            };
            scene.set(x, y, color);
        }
    }
}
