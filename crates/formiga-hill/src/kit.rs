//! The pieces areas are built from: plastered walls in timber frames, tiled roofs, glazed
//! windows, window boxes, dressed stone, bushes, clumps of leaves and clouds. Each is painted the
//! same way wherever it stands, so the Hill reads as one place.

use crate::materials::*;
use crate::paint::{
    Ramp, bevel, blit, chance, ellipse, hline, line, mix, noise, pick, polygon, put, rect, rgba,
    vline,
};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

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

/// A clump of leaves, its rim scalloped into smaller clusters and lit from the upper left. `haze`
/// gives each colour as it is seen at its row, so a clump far off is lost in the distance.
pub fn clump(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    radius: i32,
    ramp: Ramp,
    salt: u32,
    haze: impl Fn(Rgba, i32) -> Rgba,
) {
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
        ellipse(scene, x, y, size + 1, size + 1, haze(ramp.edge, y));
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
            put(scene, x, y, haze(color, y));
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
        ellipse(scene, x, y, size, size, haze(color, y));
        if light < -0.9 {
            put(scene, x - 1, y - 1, haze(ramp.shine, y));
        }
    }
}

const CLOUDS: Ramp = Ramp::new(0xc9d7e2, 0xdfe7ee, 0xf4f4f1, 0xfdfbf5, 0xffffff);

/// A heaped cloud from round puffs `(dx, dy, radius)`: flat underneath, sunlit on top and to the
/// left, cool grey below, and edged only along its underside.
pub fn cloud(scene: &mut Canvas, (cx, cy): (i32, i32), puffs: &[(i32, i32, i32)]) {
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
                let here = layer.get(x, y);
                if here.a == 0 || brightness(color) > brightness(here) {
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

/// How light a colour is, so overlapping puffs keep their lit sides.
fn brightness(color: Rgba) -> u32 {
    u32::from(color.r) + u32::from(color.g) + u32::from(color.b)
}
