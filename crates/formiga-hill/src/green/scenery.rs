//! The Village Green, seen from above and a little to one side, as if from a branch of the oak:
//! the lawn fills the picture, a treeline and hedge close it in along the top, and through a gap
//! in them the path climbs away towards the Hill. Unlike the station's level view, everything
//! here is laid out on the ground: the oak and its swing, the well, the toy chest, the blanket.
//! A strip of long grass along the bottom is painted separately, to be drawn in front of everyone.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::kit::{bush, roof};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline,
};
use formiga_art::{Canvas, Rgba};

/// Where travellers may stand on the lawn, their feet anywhere within (see `walkable`).
pub const WALK_LEFT: f32 = 24.0;
pub const WALK_RIGHT: f32 = 360.0;
pub const WALK_TOP: f32 = 98.0;
pub const WALK_BOTTOM: f32 = 202.0;
/// The picnic blanket: its back and front rows, and its edges at each.
pub const BLANKET: (i32, i32, (i32, i32), (i32, i32)) = (148, 176, (238, 302), (228, 296));
/// The oak's shade, for napping in.
pub const SHADE: (f32, f32, f32, f32) = (78.0, 120.0, 150.0, 134.0);
/// The foot of the oak, which nobody walks through.
const TRUNK_CLEAR: (f32, f32) = (86.0, 120.0);

/// Whether a traveller can stand here: on the lawn, and not in the trunk of the oak.
pub fn walkable(x: f32, y: f32) -> bool {
    (WALK_LEFT..=WALK_RIGHT).contains(&x)
        && (WALK_TOP..=WALK_BOTTOM).contains(&y)
        && !(x < TRUNK_CLEAR.0 && y < TRUNK_CLEAR.1)
}

const LAWN_TOP: i32 = 24;
const GAP: (i32, i32) = (290, 326);

const BARK: Ramp = Ramp::new(0x3a2a20, 0x5a4232, 0x74583f, 0x8c6e50, 0xa48562);
const CANOPY: Ramp = Ramp::new(0x24452b, 0x335e3b, 0x447a4a, 0x5f9a5b, 0x86bd73);
const WOODS: Ramp = Ramp::new(0x1c3424, 0x284a33, 0x355f40, 0x4b7b52, 0x6a9a69);
const WICKER: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xbf9650, 0xd8b26a, 0xead08e);
const LAWN_LIGHT: Rgba = rgb(0x8cc178);
const LAWN_DARK: Rgba = rgb(0x7fb46d);
const GRAVEL: Ramp = Ramp::new(0x9c8a6a, 0xc4b08a, 0xdcc9a0, 0xe9dab6, 0xf4ead0);
const CHECK_RED: Rgba = rgb(0xd0574a);
const ROPE: Rgba = rgb(0xcbb38a);
const BRASS_LIKE: Rgba = rgb(0xc9a14e);
const SHADOW: Rgba = rgba(0x24452b, 64);

pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    beyond_the_gap(&mut scene);
    lawn(&mut scene);
    path(&mut scene);
    treeline(&mut scene);
    flower_bed(&mut scene, 176, 78);
    well(&mut scene, 232, 70);
    toy_chest(&mut scene, 322, 94);
    blanket(&mut scene);
    oak(&mut scene);
    bunting(&mut scene);
    scene
}

/// Long grass and flowers along the bottom edge, drawn over the travellers.
pub fn foreground() -> Canvas {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for x in 0..width {
        if chance(x, 0, 201, 150) {
            let tall = 2 + (noise(x, 1, 202) % 5) as i32;
            let lean = if noise(x, 2, 203).is_multiple_of(3) {
                1
            } else {
                0
            };
            vline(&mut scene, x, height - tall, tall, GRASS_DARK);
            put(&mut scene, x + lean, height - tall - 1, LEAF.base);
        }
    }
    for (cx, salt) in [(10, 210), (width - 14, 220)] {
        for stem in 0..9 {
            let x = cx - 6 + (noise(stem, 0, salt) % 14) as i32;
            let tall = 6 + (noise(stem, 1, salt) % 7) as i32;
            vline(&mut scene, x, height - tall, tall, LEAF.shadow);
            put(&mut scene, x - 1, height - tall / 2, LEAF.base);
            let blossom = BLOSSOMS[(noise(stem, 2, salt) % 5) as usize];
            put(&mut scene, x, height - tall - 1, blossom);
            put(
                &mut scene,
                x - 1,
                height - tall - 1,
                mix(blossom, LEAF.base, 0.4),
            );
            put(
                &mut scene,
                x + 1,
                height - tall - 1,
                mix(blossom, LEAF.base, 0.4),
            );
            put(
                &mut scene,
                x,
                height - tall - 2,
                mix(blossom, rgb(0xffffff), 0.3),
            );
        }
    }
    scene
}

// ---------------------------------------------------------------------------------------------
// The edges of the green
// ---------------------------------------------------------------------------------------------

/// What shows through the gap in the trees: sky, and the Hill some way off with its tree.
fn beyond_the_gap(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for y in 0..LAWN_TOP + 14 {
        let band = mix(SKY_TOP, SKY_LOW, (y * 4 / (LAWN_TOP + 14)) as f32 / 3.0);
        scene.fill_rect(0, y, width, 1, band);
    }
    scene.fill_ellipse(308, 44, 44, 26, HILL);
    scene.fill_ellipse(320, 48, 30, 20, HILL_SHADE);
    scene.line(306, 36, 300, 26, 1, PATH);
    scene.line(300, 26, 308, 20, 1, PATH);
    scene.fill_rect(307, 12, 2, 6, TRUNK);
    scene.fill_circle(308, 10, 4, LEAVES);
}

/// Woods closing in along the top, a gap where the path leaves, and a flowering hedge in front.
fn treeline(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // The shade under the trees, so only the gap ever shows sky.
    for y in 0..LAWN_TOP + 4 {
        for x in 0..width {
            if !(GAP.0 + 4..GAP.1 - 4).contains(&x) {
                scene.set(x, y, WOODS.shadow);
            }
        }
    }
    for x in (-8..width + 8).step_by(13) {
        if (GAP.0 - 10..GAP.1 + 10).contains(&x) {
            continue;
        }
        let radius = 12 + (noise(x, 0, 401) % 6) as i32;
        let lift = (noise(x, 1, 402) % 8) as i32;
        leaf_mass(scene, x, 4 - lift, radius, WOODS, (1000 + x) as u32);
    }
    for x in (-4..width + 6).step_by(11) {
        if (GAP.0 - 6..GAP.1 + 6).contains(&x) {
            continue;
        }
        bush(scene, x, LAWN_TOP + 6, 8, (2000 + x) as u32);
    }
    // The hedge's shade on the lawn.
    for x in 0..width {
        if (GAP.0..GAP.1).contains(&x) {
            continue;
        }
        hline(scene, x, LAWN_TOP + 14, 1, rgba(0x24452b, 70));
        hline(scene, x, LAWN_TOP + 15, 1, rgba(0x24452b, 34));
    }
}

// ---------------------------------------------------------------------------------------------
// The ground
// ---------------------------------------------------------------------------------------------

fn lawn(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for y in LAWN_TOP..height {
        for x in 0..width {
            // Mown in diagonal stripes, as a lawn seen from above is.
            let stripe = (x + y * 2).div_euclid(30).rem_euclid(2) == 0;
            let base = if stripe { LAWN_LIGHT } else { LAWN_DARK };
            let roll = noise(x, y, 341) % 64;
            let color = if roll < 4 {
                mix(base, GRASS_DARK, 0.7)
            } else if roll < 6 {
                mix(base, GRASS_LIGHT, 0.8)
            } else {
                base
            };
            scene.set(x, y, color);
        }
    }
    for y in LAWN_TOP..height {
        for x in 0..width {
            if chance(x, y, 342, 2) {
                put(scene, x, y, LEAF.light);
                put(scene, x + 1, y, LEAF.base);
                put(scene, x, y - 1, LEAF.light);
            }
            if chance(x / 2, y, 343, 1) && x % 3 == 0 {
                put(scene, x, y, rgb(0xfbf6ee));
                put(scene, x, y + 1, rgb(0xf5d25e));
            }
        }
    }
}

/// The gravel path in from the lane at the bottom corner, up through the gap and away.
fn path(scene: &mut Canvas) {
    let points: [(f32, f32); 7] = [
        (398.0, 222.0),
        (362.0, 190.0),
        (336.0, 152.0),
        (322.0, 112.0),
        (314.0, 72.0),
        (308.0, 36.0),
        (306.0, 20.0),
    ];
    let centre = |y: f32| -> f32 {
        for pair in points.windows(2) {
            let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
            if y <= y0 && y >= y1 {
                return x0 + (x1 - x0) * (y - y0) / (y1 - y0);
            }
        }
        points[0].0
    };
    for y in 20..SCENE_HEIGHT as i32 {
        // Wider towards the front, as anything flat is from above.
        let half = 3.0 + y as f32 * 0.035;
        let middle = centre(y as f32);
        let (left, right) = (
            (middle - half).round() as i32,
            (middle + half).round() as i32,
        );
        for x in left..=right {
            let color = match noise(x, y, 421) % 9 {
                0 => GRAVEL.shadow,
                1 | 2 => GRAVEL.light,
                3 => GRAVEL.shine,
                _ => GRAVEL.base,
            };
            scene.set(x, y, color);
        }
        put(scene, left - 1, y, mix(GRAVEL.edge, LAWN_DARK, 0.4));
        put(scene, right + 1, y, rgba(0x24452b, 60));
    }
}

/// A round bed of flowers with a birdbath in the middle.
fn flower_bed(scene: &mut Canvas, cx: i32, cy: i32) {
    ellipse(scene, cx + 2, cy + 3, 22, 8, SHADOW);
    ellipse(scene, cx, cy, 22, 8, rgb(0x6b4e36));
    ellipse(scene, cx, cy, 21, 7, rgb(0x7a5a3e));
    for y in cy - 7..=cy + 7 {
        for x in cx - 21..=cx + 21 {
            let (dx, dy) = ((x - cx) as f32 / 20.0, (y - cy) as f32 / 6.5);
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            match noise(x, y, 431) % 7 {
                0 | 1 => put(scene, x, y, LEAF.base),
                2 => put(scene, x, y, LEAF.light),
                3 => put(scene, x, y, BLOSSOMS[(noise(x, y, 432) % 5) as usize]),
                _ => {}
            }
        }
    }
    // The birdbath: a stone bowl on a stem, with a little water in it.
    ellipse(scene, cx + 2, cy + 1, 5, 2, rgba(0x24452b, 80));
    rect(scene, cx - 1, cy - 8, 3, 9, STONE.base);
    vline(scene, cx - 1, cy - 8, 9, STONE.light);
    ellipse(scene, cx, cy - 9, 6, 2, STONE.edge);
    ellipse(scene, cx, cy - 9, 5, 2, STONE.light);
    ellipse(scene, cx, cy - 9, 3, 1, rgb(0x86aab5));
    put(scene, cx - 1, cy - 10, rgb(0xe9f6f7));
}

// ---------------------------------------------------------------------------------------------
// Things on the lawn
// ---------------------------------------------------------------------------------------------

/// The well, from above: its stone wall, the water, and a little tiled roof over it.
fn well(scene: &mut Canvas, cx: i32, rim: i32) {
    let (rx, ry, wall) = (15, 6, 12);
    ellipse(scene, cx + 9, rim + wall + 3, 20, 5, SHADOW);
    for x in cx - rx..=cx + rx {
        let across = (x - cx) as f32 / rx as f32;
        let curve = ((1.0 - across * across).max(0.0)).sqrt() * ry as f32;
        let top = rim + curve.round() as i32;
        for y in top..top + wall {
            let course = (y - top) / 4;
            let joint =
                (y - top) % 4 == 3 || (x + if course % 2 == 1 { 3 } else { 0 }).rem_euclid(7) == 0;
            let base = if joint { STONE.edge } else { STONE.base };
            let shaded = mix(base, STONE.shadow, across.abs().powf(1.6) * 0.8);
            let lit = if across < -0.3 && !joint {
                mix(shaded, STONE.light, 0.35)
            } else {
                shaded
            };
            scene.set(x, y, lit);
        }
    }
    ellipse(scene, cx, rim, rx, ry, STONE.light);
    ellipse(scene, cx, rim, rx - 1, ry - 1, STONE.shine);
    ellipse(scene, cx, rim, rx - 4, ry - 2, rgb(0x2a3532));
    hline(scene, cx - 6, rim - 1, 5, rgba(0x86aab5, 160));
    // Posts up to the roof, the windlass, and the bucket on its rope.
    for post in [cx - rx + 1, cx + rx - 3] {
        rect(scene, post, rim - 20, 3, 21, TIMBER.base);
        vline(scene, post, rim - 20, 21, TIMBER.light);
        vline(scene, post + 2, rim - 20, 21, TIMBER.edge);
    }
    hline(scene, cx - rx + 3, rim - 13, rx * 2 - 6, TIMBER.base);
    hline(scene, cx - rx + 3, rim - 14, rx * 2 - 6, TIMBER.light);
    vline(scene, cx, rim - 12, 6, ROPE);
    bevel(scene, cx - 3, rim - 6, 7, 5, PLANK);
    roof(
        scene,
        &[
            (cx - rx - 6, rim - 18),
            (cx - rx + 2, rim - 30),
            (cx + rx - 2, rim - 30),
            (cx + rx + 6, rim - 18),
        ],
        rim - 30,
        rim - 18,
    );
    hline(scene, cx - rx + 2, rim - 31, rx * 2 - 4, TILE.shadow);
    hline(scene, cx - rx - 6, rim - 18, rx * 2 + 12, TIMBER.edge);
    hline(
        scene,
        cx - rx - 5,
        rim - 17,
        rx * 2 + 10,
        rgba(0x2a1e18, 90),
    );
}

/// The toy chest, its lid thrown back and its toys showing.
fn toy_chest(scene: &mut Canvas, left: i32, base: i32) {
    let (width, front, top) = (28, 11, 6);
    ellipse(
        scene,
        left + width / 2 + 6,
        base + 1,
        width / 2 + 4,
        3,
        SHADOW,
    );
    // The lid, open behind.
    polygon(
        scene,
        &[
            (left + 2, base - front - top),
            (left + 4, base - front - top - 9),
            (left + width + 4, base - front - top - 9),
            (left + width + 2, base - front - top),
        ],
        |_, y| {
            Some(if y < base - front - top - 6 {
                PLANK.light
            } else {
                PLANK.shadow
            })
        },
    );
    // The top, seen from above: dark inside, with a ball, a block and a boat's mast.
    polygon(
        scene,
        &[
            (left, base - front),
            (left + 2, base - front - top),
            (left + width + 2, base - front - top),
            (left + width, base - front),
        ],
        |_, _| Some(rgb(0x2e2626)),
    );
    ellipse(scene, left + 8, base - front - 3, 3, 2, rgb(0xc74a3e));
    put(scene, left + 7, base - front - 4, rgb(0xf29a88));
    bevel(
        scene,
        left + 14,
        base - front - 6,
        6,
        5,
        Ramp::new(0x22406a, 0x2f5c96, 0x4a7fc4, 0x76a6e0, 0xb0d0f4),
    );
    vline(scene, left + 23, base - front - 10, 8, TIMBER.base);
    polygon(
        scene,
        &[
            (left + 24, base - front - 10),
            (left + 24, base - front - 5),
            (left + 28, base - front - 5),
        ],
        |_, _| Some(CREAM),
    );
    // The front.
    bevel(scene, left, base - front, width, front, PLANK);
    hline(scene, left + 1, base - front + 4, width - 2, PLANK.shadow);
    for band in [left + 4, left + width - 6] {
        rect(scene, band, base - front, 2, front, IRON.base);
        vline(scene, band, base - front, front, IRON.light);
    }
    rect(
        scene,
        left + width / 2 - 1,
        base - front + 2,
        3,
        4,
        BRASS_LIKE,
    );
}

fn blanket(scene: &mut Canvas) {
    let (back, front, (back_left, back_right), (front_left, front_right)) = BLANKET;
    let edge = |y: i32| {
        let t = (y - back) as f32 / (front - back) as f32;
        (
            back_left as f32 + (front_left - back_left) as f32 * t,
            back_right as f32 + (front_right - back_right) as f32 * t,
        )
    };
    polygon(
        scene,
        &[
            (back_left + 4, back + 3),
            (back_right + 4, back + 3),
            (front_right + 4, front + 3),
            (front_left + 4, front + 3),
        ],
        |_, _| Some(SHADOW),
    );
    polygon(
        scene,
        &[
            (back_left, back),
            (back_right, back),
            (front_right, front),
            (front_left, front),
        ],
        |x, y| {
            let (left, right) = edge(y);
            let u = ((x as f32 - left) / (right - left)).clamp(0.0, 0.999);
            let v = ((y - back) as f32 / (front - back) as f32).clamp(0.0, 0.999);
            let check = ((u * 8.0) as i32 + (v * 5.0) as i32) % 2 == 0;
            let base = if check { CHECK_RED } else { CREAM };
            let fold = ((u * 8.0 + v * 3.0) - 3.6).abs() < 0.15;
            Some(if fold {
                mix(base, rgb(0xffffff), 0.3)
            } else {
                base
            })
        },
    );
    line(
        scene,
        (front_left, front),
        (front_right, front),
        mix(CHECK_RED, rgb(0x3a1418), 0.5),
    );
    // A basket of good things, from above: its weave, its handle, a cloth over the top.
    let (bx, by) = (back_right - 22, back + 6);
    ellipse(scene, bx + 9, by + 7, 9, 3, SHADOW);
    rect(scene, bx, by, 16, 8, WICKER.base);
    for y in by..by + 8 {
        for x in bx..bx + 16 {
            if ((x - bx) / 2 + (y - by) / 2) % 2 == 0 {
                put(scene, x, y, WICKER.light);
            }
        }
    }
    rect(scene, bx + 2, by + 1, 12, 3, CREAM);
    for x in (bx + 2..bx + 14).step_by(2) {
        put(scene, x, by + 2, CHECK_RED);
    }
    hline(scene, bx, by + 7, 16, WICKER.shadow);
    line(scene, (bx + 1, by), (bx + 8, by - 5), WICKER.edge);
    line(scene, (bx + 8, by - 5), (bx + 15, by), WICKER.edge);
    ellipse(scene, bx + 11, by + 2, 2, 1, rgb(0xd8a060));
    let jar = bx - 8;
    rect(scene, jar, by + 2, 4, 5, rgb(0xb83a40));
    hline(scene, jar, by + 1, 4, CREAM);
}

// ---------------------------------------------------------------------------------------------
// The oak
// ---------------------------------------------------------------------------------------------

fn oak(scene: &mut Canvas) {
    // Its shade falls away down the lawn, with flecks of sun coming through.
    for y in 100..150 {
        for x in 10..200 {
            let (dx, dy) = ((x - 104) as f32 / 90.0, (y - 124) as f32 / 18.0);
            if dx * dx + dy * dy <= 1.0 && !chance(x / 2, y, 351, 40) {
                put(scene, x, y, rgba(0x24452b, 50));
            }
        }
    }
    // The swing, hanging from the long branch.
    ellipse(scene, 124, 118, 10, 2, rgba(0x24452b, 60));
    for rope in [116, 130] {
        for y in 58..106 {
            put(
                scene,
                rope,
                y,
                if y % 2 == 0 {
                    ROPE
                } else {
                    mix(ROPE, BARK.shadow, 0.3)
                },
            );
        }
    }
    bevel(scene, 112, 106, 22, 3, PLANK);
    // The trunk: flared at the foot into roots across the grass, lit on its left.
    polygon(
        scene,
        &[(34, 116), (46, 98), (66, 98), (80, 116)],
        |_, _| Some(BARK.base),
    );
    rect(scene, 46, 56, 20, 44, BARK.base);
    for x in 34..80 {
        for y in 56..116 {
            if scene.get(x, y) != BARK.base {
                continue;
            }
            let across = (x - 56) as f32 / 10.0;
            let mut color = if across < -0.55 {
                BARK.light
            } else if across > 0.6 {
                BARK.shadow
            } else {
                BARK.base
            };
            if noise(x, y / 5, 352).is_multiple_of(9) {
                color = mix(color, BARK.edge, 0.7);
            } else if chance(x, y, 353, 20) {
                color = mix(color, BARK.shine, 0.4);
            }
            scene.set(x, y, color);
        }
    }
    // Roots, thick where they leave the trunk and sinking into the grass.
    for (from, to) in [((38, 115), (30, 119)), ((74, 115), (84, 119))] {
        line(scene, from, to, BARK.base);
        line(scene, (from.0, from.1 - 1), (to.0, to.1 - 1), BARK.light);
        line(scene, (from.0, from.1 + 1), (to.0, to.1 + 1), BARK.edge);
    }
    hline(scene, 30, 117, 52, rgba(0x2a1e18, 100));
    // The long branch the swing hangs from.
    polygon(
        scene,
        &[(60, 50), (140, 54), (140, 58), (60, 60)],
        |x, y| {
            let top = 50.0 + (x - 60) as f32 * 4.0 / 80.0;
            Some(if (y as f32) < top + 1.5 {
                BARK.light
            } else {
                BARK.base
            })
        },
    );
    line(scene, (60, 60), (140, 58), BARK.edge);
    // The crown, seen from above: one great mass of leaves, lit from the upper left.
    let masses = [
        (20, 8, 30),
        (58, -4, 30),
        (92, 12, 24),
        (8, 40, 22),
        (44, 30, 26),
        (80, 38, 20),
        (112, 34, 14),
        (28, 58, 14),
        (66, 54, 12),
    ];
    for (index, &(cx, cy, r)) in masses.iter().enumerate() {
        leaf_mass(scene, cx, cy, r, CANOPY, 360 + index as u32);
    }
}

fn leaf_mass(scene: &mut Canvas, cx: i32, cy: i32, radius: i32, ramp: Ramp, salt: u32) {
    for step in 0..radius * 3 {
        let angle = step as f32 / (radius * 3) as f32 * std::f32::consts::TAU;
        let reach = radius as f32 + (noise(step, 0, salt) % 3) as f32 - 0.5;
        let (x, y) = (
            cx + (angle.cos() * reach).round() as i32,
            cy + 2 + (angle.sin() * (reach - 2.0)).round() as i32,
        );
        let clump = 1 + (noise(step, 1, salt) % 2) as i32;
        ellipse(scene, x, y, clump, clump, ramp.edge);
        if angle.sin() < -0.2 {
            put(scene, x, y - 1, ramp.shadow);
        }
    }
    ellipse(scene, cx, cy + 2, radius, radius - 2, ramp.edge);
    ellipse(scene, cx, cy, radius - 1, radius - 2, ramp.shadow);
    ellipse(scene, cx - 2, cy - 2, radius - 4, radius - 5, ramp.base);
    for y in cy - radius..=cy + radius {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy > (radius - 2) * (radius - 2) {
                continue;
            }
            let cluster = noise(x / 2, y / 2, salt);
            if dx + dy < -radius / 3 && cluster.is_multiple_of(4) {
                put(scene, x, y, ramp.light);
            } else if dx + dy < -radius && cluster % 7 == 1 {
                put(scene, x, y, ramp.shine);
            } else if dx + dy > radius / 2 && cluster.is_multiple_of(3) {
                put(scene, x, y, ramp.edge);
            }
        }
    }
}

/// Flags strung from the oak's branch to a pole by the path, sagging between.
fn bunting(scene: &mut Canvas) {
    let pole = 352;
    rect(scene, pole, 34, 3, 64, TRIM.base);
    vline(scene, pole, 34, 64, TRIM.light);
    vline(scene, pole + 2, 34, 64, TRIM.shadow);
    ellipse(scene, pole + 1, 32, 2, 2, BRASS_LIKE);
    ellipse(scene, pole + 6, 98, 6, 2, SHADOW);
    let (from, to, sag) = ((138.0, 56.0), (353.0, 36.0), 18.0);
    let flags = [
        CHECK_RED,
        rgb(0xf5d25e),
        rgb(0x8fc6e0),
        rgb(0xf19bb0),
        rgb(0x9bd38a),
    ];
    let at = |t: f32| {
        (
            from.0 + (to.0 - from.0) * t,
            from.1 + (to.1 - from.1) * t + sag * 4.0 * t * (1.0 - t),
        )
    };
    let mut previous = at(0.0);
    for step in 1..=120 {
        let point = at(step as f32 / 120.0);
        line(
            scene,
            (previous.0.round() as i32, previous.1.round() as i32),
            (point.0.round() as i32, point.1.round() as i32),
            rgb(0x5a4a44),
        );
        previous = point;
    }
    for index in 0..24 {
        let (x, y) = at((index as f32 + 0.5) / 24.0);
        let (x, y) = (x.round() as i32, y.round() as i32 + 1);
        let color = flags[index % flags.len()];
        polygon(scene, &[(x - 3, y), (x + 3, y), (x, y + 6)], |px, _| {
            Some(if px > x {
                mix(color, rgb(0x2a2030), 0.25)
            } else {
                color
            })
        });
        put(scene, x - 2, y, mix(color, rgb(0xffffff), 0.4));
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
    fn the_foreground_leaves_the_lawn_open() {
        let front = foreground();
        let covered = front.pixels().iter().filter(|pixel| pixel.a > 0).count();
        let total = (SCENE_WIDTH * SCENE_HEIGHT) as usize;
        assert!(
            covered < total / 40,
            "{covered} pixels of long grass is a hedge"
        );
        let (_, top, _, _) = front.alpha_bounds().unwrap();
        assert!(top as f32 > WALK_BOTTOM - 10.0);
    }

    #[test]
    fn the_green_is_the_same_every_time() {
        assert_eq!(backdrop(), backdrop());
    }

    #[test]
    fn nobody_can_stand_in_the_oak() {
        assert!(!walkable(56.0, 110.0));
        assert!(walkable(56.0, 140.0));
        assert!(walkable(200.0, 120.0));
    }
}
