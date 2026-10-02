//! The Village Green, painted back to front: sky and the Hill beyond, cottages over a dry-stone
//! wall, the well, the lawn with its blanket and toy chest, and the old oak with its swing.
//! A strip of long grass along the bottom is painted separately, to be drawn in front of everyone.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::kit::{bush, plaster, ridge_tiles, roof, window};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline,
};
use formiga_art::{Canvas, Rgba};

/// Where travellers may stand on the lawn, their feet anywhere within.
pub const WALK_LEFT: f32 = 26.0;
pub const WALK_RIGHT: f32 = 358.0;
pub const WALK_TOP: f32 = 152.0;
pub const WALK_BOTTOM: f32 = 202.0;
/// The picnic blanket, for sitting on: its back and front rows, and its edges at each.
pub const BLANKET: (i32, i32, (i32, i32), (i32, i32)) = (168, 190, (222, 290), (214, 300));
/// The oak's shade, for napping in.
pub const SHADE: (f32, f32, f32, f32) = (40.0, 154.0, 128.0, 166.0);

const HORIZON: i32 = 100;
const WALL_TOP: i32 = 106;
const LAWN_TOP: i32 = 118;

const BARK: Ramp = Ramp::new(0x3a2a20, 0x5a4232, 0x74583f, 0x8c6e50, 0xa48562);
const THATCH: Ramp = Ramp::new(0x6b4f22, 0x96733a, 0xc29a52, 0xd9b56a, 0xeacf8c);
const CANOPY: Ramp = Ramp::new(0x24452b, 0x335e3b, 0x447a4a, 0x5f9a5b, 0x86bd73);
const WICKER: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xbf9650, 0xd8b26a, 0xead08e);
const LAWN_LIGHT: Rgba = rgb(0x8cc178);
const LAWN_DARK: Rgba = rgb(0x7db36c);
const CHECK_RED: Rgba = rgb(0xd0574a);
const ROPE: Rgba = rgb(0xcbb38a);

pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    the_hill(&mut scene);
    village(&mut scene);
    dry_stone_wall(&mut scene);
    lawn(&mut scene);
    flower_bed(&mut scene);
    stepping_stones(&mut scene);
    well(&mut scene, 222);
    bunting_pole(&mut scene);
    toy_chest(&mut scene, 322);
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
    // A clump of flowers in each lower corner, framing the green.
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
// Far away
// ---------------------------------------------------------------------------------------------

fn sky(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for y in 0..HORIZON {
        let band = mix(SKY_TOP, SKY_LOW, (y * 6 / HORIZON) as f32 / 5.0);
        scene.fill_rect(0, y, width, 1, band);
    }
    // Down to the lawn, so wherever the hills and the village leave a gap there is still sky.
    scene.fill_rect(0, HORIZON, width, LAWN_TOP - HORIZON, SKY_LOW);
    for (x, y, size) in [(196, 24, 10), (340, 36, 8), (128, 12, 7)] {
        scene.fill_ellipse(x, y, size * 2, size / 2 + 2, CLOUD);
        scene.fill_ellipse(x - size / 2, y - 3, size, size / 2 + 1, CLOUD);
        scene.fill_ellipse(x + size / 2, y - 4, size, size / 2 + 2, CLOUD);
    }
}

/// The Hill itself, close now, with the path up it and the tree at the top.
fn the_hill(scene: &mut Canvas) {
    scene.fill_ellipse(150, 112, 110, 26, FAR_HILL);
    scene.fill_ellipse(366, 108, 80, 22, FAR_HILL);
    scene.fill_ellipse(268, 134, 118, 78, HILL);
    scene.fill_ellipse(304, 142, 80, 62, HILL_SHADE);
    // Gorse and grass tussocks on its flank.
    for index in 0..40 {
        let x = 170 + (noise(index, 0, 301) % 190) as i32;
        let y = 64 + (noise(index, 1, 302) % 40) as i32;
        if scene.get(x, y) == HILL || scene.get(x, y) == HILL_SHADE {
            put(scene, x, y, mix(scene.get(x, y), LEAF.shadow, 0.5));
            if index % 5 == 0 {
                put(scene, x + 1, y - 1, rgb(0xf5d25e));
            }
        }
    }
    let path = [
        (234, 106),
        (258, 96),
        (244, 86),
        (272, 76),
        (262, 66),
        (268, 58),
    ];
    for pair in path.windows(2) {
        scene.line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, 2, PATH);
    }
    scene.fill_rect(266, 46, 3, 12, TRUNK);
    scene.fill_circle(268, 42, 7, LEAVES);
    scene.fill_circle(262, 45, 5, LEAVES);
    scene.fill_circle(274, 45, 5, LEAVES);
}

// ---------------------------------------------------------------------------------------------
// The village behind the wall
// ---------------------------------------------------------------------------------------------

fn village(scene: &mut Canvas) {
    for (index, &(x, y, r)) in [
        (4, 100, 10),
        (22, 102, 8),
        (118, 100, 8),
        (198, 100, 8),
        (252, 101, 7),
        (284, 101, 7),
        (368, 99, 9),
    ]
    .iter()
    .enumerate()
    {
        bush(scene, x, y, r, 30 + index as u32);
    }
    tiled_cottage(scene, 136);
    thatched_cottage(scene, 300);
}

fn tiled_cottage(scene: &mut Canvas, left: i32) {
    let right = left + 48;
    // A brick chimney at the gable end.
    for y in 62..78 {
        for x in right - 14..right - 7 {
            let row = (y - 62) / 3;
            let brick = (x + if row % 2 == 1 { 2 } else { 0 }) % 4 == 0 || (y - 62) % 3 == 2;
            scene.set(x, y, if brick { BRICK.shadow } else { BRICK.base });
        }
    }
    bevel(scene, right - 15, 60, 9, 3, STONE);
    plaster(scene, left, 88, right - left, WALL_TOP - 88);
    vline(scene, left, 88, WALL_TOP - 88, TIMBER.base);
    vline(scene, right - 1, 88, WALL_TOP - 88, TIMBER.shadow);
    hline(scene, left, 88, right - left, TIMBER.base);
    window(scene, left + 6, 92, 12, 10, 2);
    window(scene, left + 28, 92, 12, 10, 2);
    roof(
        scene,
        &[
            (left - 6, 89),
            (left + 8, 72),
            (right - 8, 72),
            (right + 6, 89),
        ],
        72,
        89,
    );
    ridge_tiles(scene, left + 7, right - 7, 72);
    for (index, y) in (89..92).enumerate() {
        hline(
            scene,
            left,
            y,
            right - left,
            rgba(0x4a3040, 60 - index as u8 * 18),
        );
    }
}

fn thatched_cottage(scene: &mut Canvas, left: i32) {
    let right = left + 48;
    plaster(scene, left, 90, right - left, WALL_TOP - 90);
    vline(scene, left, 90, WALL_TOP - 90, TIMBER.base);
    vline(scene, right - 1, 90, WALL_TOP - 90, TIMBER.shadow);
    window(scene, left + 18, 94, 12, 9, 2);
    // Thatch: strands running down the roof, in layers, rounded over the top.
    let outline = [
        (left - 8, 93),
        (left + 2, 76),
        (left + 10, 70),
        (right - 10, 70),
        (right - 2, 76),
        (right + 8, 93),
    ];
    polygon(scene, &outline, |x, y| {
        let layer = (y - 70) % 5;
        let strand = noise(x, y / 3, 311) % 7;
        let depth = (y - 70) as f32 / 23.0;
        let base = match strand {
            0 => THATCH.shadow,
            1 | 2 => THATCH.light,
            _ => THATCH.base,
        };
        let shaded = mix(base, THATCH.shadow, depth * 0.4);
        Some(if layer == 4 {
            mix(shaded, THATCH.edge, 0.5)
        } else if layer == 0 && strand < 2 {
            THATCH.shine
        } else {
            shaded
        })
    });
    // The ridge, wrapped and pinned, and a heavy edge along the eaves.
    rect(scene, left + 10, 70, right - left - 20, 3, THATCH.shadow);
    hline(scene, left + 10, 70, right - left - 20, THATCH.light);
    for x in (left + 12..right - 10).step_by(4) {
        put(scene, x, 71, THATCH.edge);
    }
    for pair in outline.windows(2) {
        line(scene, pair[0], pair[1], THATCH.edge);
    }
    hline(scene, left - 7, 93, right - left + 14, THATCH.edge);
    hline(scene, left - 6, 94, right - left + 12, rgba(0x3a2a20, 90));
}

fn dry_stone_wall(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // A row of upright coping stones along the top.
    for x in 0..width {
        let stone = x.div_euclid(3);
        let tall = 3 + (noise(stone, 0, 321) % 2) as i32;
        let color = match x.rem_euclid(3) {
            0 => STONE.light,
            2 => STONE.edge,
            _ => STONE.base,
        };
        vline(scene, x, WALL_TOP + 4 - tall, tall, color);
    }
    // Courses of rough stones, each a different length, set without mortar.
    for course in 0..2 {
        let top = WALL_TOP + 4 + course * 4;
        let mut x = -((noise(course, 0, 322) % 6) as i32);
        let mut stone = 0;
        while x < width {
            let long = 4 + (noise(stone, course, 323) % 5) as i32;
            let tone = match noise(stone, course, 324) % 5 {
                0 => mix(STONE.base, STONE.shadow, 0.5),
                1 => mix(STONE.base, STONE.light, 0.4),
                2 => mix(STONE.base, LEAF.base, 0.3),
                _ => STONE.base,
            };
            rect(scene, x, top, long, 4, tone);
            hline(scene, x, top, long - 1, mix(tone, STONE.shine, 0.5));
            vline(scene, x, top, 3, mix(tone, STONE.light, 0.5));
            hline(scene, x, top + 3, long, STONE.edge);
            vline(scene, x + long - 1, top, 4, STONE.edge);
            x += long;
            stone += 1;
        }
    }
}

fn flower_bed(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for x in 0..width {
        if (204..240).contains(&x) {
            continue;
        }
        let tall = 3 + (noise(x, 0, 331) % 4) as i32;
        vline(scene, x, LAWN_TOP + 4 - tall, tall + 2, LEAF.shadow);
        put(scene, x, LAWN_TOP + 4 - tall, LEAF.base);
        if chance(x, 1, 332, 70) {
            let blossom = BLOSSOMS[(noise(x, 2, 333) % 5) as usize];
            let y = LAWN_TOP + 2 - tall;
            put(scene, x, y, blossom);
            put(scene, x, y - 1, mix(blossom, rgb(0xffffff), 0.35));
        }
    }
    hline(scene, 0, LAWN_TOP + 6, width, rgba(0x2c4a2e, 60));
}

// ---------------------------------------------------------------------------------------------
// The lawn and what is on it
// ---------------------------------------------------------------------------------------------

fn lawn(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    // Mown stripes, widening towards the front as a lawn seen from above does.
    let stripes = [LAWN_TOP, 124, 131, 140, 151, 164, 180, 198, height];
    for (index, pair) in stripes.windows(2).enumerate() {
        let color = if index % 2 == 0 {
            LAWN_LIGHT
        } else {
            LAWN_DARK
        };
        rect(scene, 0, pair[0], width, pair[1] - pair[0], color);
    }
    for y in LAWN_TOP..height {
        let near = (y - LAWN_TOP) as f32 / (height - LAWN_TOP) as f32;
        for x in 0..width {
            let roll = noise(x, y, 341) % 64;
            if roll < 4 {
                put(scene, x, y, mix(scene.get(x, y), GRASS_DARK, 0.7));
            } else if roll < 6 {
                put(scene, x, y, mix(scene.get(x, y), GRASS_LIGHT, 0.8));
            }
        }
        // Clover and daisies, larger and more often towards the front.
        for x in 0..width {
            if chance(x, y, 342, 2 + (near * 2.0) as u32) {
                put(scene, x, y, LEAF.light);
                put(scene, x + 1, y, LEAF.base);
                put(scene, x, y - 1, LEAF.light);
            }
            if chance(x, y, 343, if near > 0.45 { 1 } else { 0 })
                || chance(x / 2, y, 345, 1) && near < 0.45 && x % 3 == 0
            {
                put(scene, x, y, rgb(0xfbf6ee));
                if near > 0.45 {
                    put(scene, x - 1, y, rgb(0xf3ece0));
                    put(scene, x + 1, y, rgb(0xf3ece0));
                    put(scene, x, y - 1, rgb(0xfbf6ee));
                    put(scene, x, y + 1, rgb(0xe8e0d0));
                    put(scene, x, y, rgb(0xf5d25e));
                }
            } else if x % 2 == 0 && chance(x, y, 344, 1) {
                put(scene, x, y, rgb(0xf5d25e));
            }
        }
    }
}

fn stepping_stones(scene: &mut Canvas) {
    for step in 0..11 {
        let t = step as f32 / 10.0;
        let x = 380.0 - t * 150.0;
        let y = 176.0 - t * 34.0 + (t * 3.2).sin() * 4.0;
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (rx, ry) = (4 + ((1.0 - t) * 3.0) as i32, 2 + ((1.0 - t) * 1.2) as i32);
        ellipse(scene, cx + 1, cy + 1, rx, ry, rgba(0x2c4a2e, 70));
        ellipse(scene, cx, cy, rx, ry, STONE.edge);
        ellipse(scene, cx, cy, rx - 1, ry - 1, STONE.base);
        hline(scene, cx - rx + 2, cy - ry + 1, rx, STONE.light);
        put(scene, cx - rx / 2, cy - ry + 1, STONE.shine);
    }
}

fn well(scene: &mut Canvas, centre: i32) {
    let (left, right) = (centre - 16, centre + 16);
    // Shade cast on the lawn.
    ellipse(scene, centre + 6, 139, 19, 3, rgba(0x2c4a2e, 70));
    // Posts and the little roof over them.
    for post in [left + 2, right - 5] {
        rect(scene, post, 98, 3, 30, TIMBER.base);
        vline(scene, post, 98, 30, TIMBER.light);
        vline(scene, post + 2, 98, 30, TIMBER.edge);
    }
    roof(
        scene,
        &[(left - 4, 101), (centre, 88), (right + 4, 101)],
        88,
        101,
    );
    for offset in 0..2 {
        line(
            scene,
            (left - 4, 101 + offset),
            (centre, 88 + offset),
            TIMBER.light,
        );
        line(
            scene,
            (centre, 88 + offset),
            (right + 4, 101 + offset),
            TIMBER.base,
        );
    }
    // The windlass, its handle, the rope and the bucket.
    rect(scene, left + 3, 104, right - left - 6, 2, TIMBER.base);
    hline(scene, left + 3, 104, right - left - 6, TIMBER.light);
    line(scene, (right - 2, 105), (right + 2, 105), IRON.base);
    line(scene, (right + 2, 105), (right + 2, 109), IRON.base);
    vline(scene, centre, 106, 8, ROPE);
    bevel(scene, centre - 3, 113, 7, 6, PLANK);
    hline(scene, centre - 3, 115, 7, IRON.base);
    // The round stone wall, darker towards its sides as it turns away.
    for y in 124..139 {
        for x in left..right {
            let across = (x - centre) as f32 / 16.0;
            let course = (y - 124) / 4;
            let joint =
                (y - 124) % 4 == 3 || (x + if course % 2 == 1 { 3 } else { 0 }).rem_euclid(7) == 0;
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
    ellipse(scene, centre, 124, 16, 4, STONE.light);
    ellipse(scene, centre, 124, 15, 3, STONE.shine);
    ellipse(scene, centre, 124, 12, 2, rgb(0x2a3532));
    hline(scene, centre - 6, 124, 5, rgba(0x86aab5, 160));
    hline(scene, left, 139, right - left, rgba(0x2c4a2e, 90));
}

fn bunting_pole(scene: &mut Canvas) {
    let x = 374;
    rect(scene, x, 60, 3, 88, TRIM.base);
    vline(scene, x, 60, 88, TRIM.light);
    vline(scene, x + 2, 60, 88, TRIM.shadow);
    ellipse(scene, x + 1, 58, 2, 2, BRASS_LIKE);
    put(scene, x, 57, rgb(0xf6e3a2));
    hline(scene, x - 2, 148, 7, rgba(0x2c4a2e, 90));
}

const BRASS_LIKE: Rgba = rgb(0xc9a14e);

fn toy_chest(scene: &mut Canvas, left: i32) {
    let (width, top, bottom) = (28, 138, 150);
    ellipse(
        scene,
        left + width / 2 + 3,
        bottom,
        width / 2 + 2,
        2,
        rgba(0x2c4a2e, 80),
    );
    // The lid, thrown open behind.
    polygon(
        scene,
        &[
            (left, top),
            (left + 3, top - 10),
            (left + width + 3, top - 10),
            (left + width, top),
        ],
        |_, y| {
            Some(if y < top - 8 {
                PLANK.light
            } else {
                PLANK.shadow
            })
        },
    );
    line(scene, (left, top), (left + 3, top - 10), PLANK.edge);
    line(
        scene,
        (left + 3, top - 10),
        (left + width + 3, top - 10),
        PLANK.edge,
    );
    line(
        scene,
        (left + width + 3, top - 10),
        (left + width, top),
        PLANK.edge,
    );
    // What is inside, poking out: a ball, a block, a wooden boat's mast.
    rect(scene, left + 2, top - 2, width - 4, 3, rgb(0x2e2626));
    ellipse(scene, left + 8, top - 2, 3, 3, rgb(0xc74a3e));
    put(scene, left + 7, top - 4, rgb(0xf29a88));
    bevel(
        scene,
        left + 14,
        top - 6,
        6,
        6,
        Ramp::new(0x22406a, 0x2f5c96, 0x4a7fc4, 0x76a6e0, 0xb0d0f4),
    );
    vline(scene, left + 23, top - 9, 8, TIMBER.base);
    polygon(
        scene,
        &[
            (left + 24, top - 9),
            (left + 24, top - 4),
            (left + 28, top - 4),
        ],
        |_, _| Some(CREAM),
    );
    // The chest.
    bevel(scene, left, top, width, bottom - top, PLANK);
    hline(scene, left + 1, top + 4, width - 2, PLANK.shadow);
    hline(scene, left + 1, top + 8, width - 2, PLANK.shadow);
    for band in [left + 4, left + width - 6] {
        rect(scene, band, top, 2, bottom - top, IRON.base);
        vline(scene, band, top, bottom - top, IRON.light);
    }
    rect(scene, left + width / 2 - 1, top + 3, 3, 4, BRASS_LIKE);
    put(scene, left + width / 2, top + 4, rgb(0x6b4a24));
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
    ellipse(
        scene,
        (front_left + front_right) / 2 + 3,
        front + 1,
        (front_right - front_left) / 2,
        2,
        rgba(0x2c4a2e, 70),
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
            let check = ((u * 8.0) as i32 + (v * 4.0) as i32) % 2 == 0;
            let base = if check { CHECK_RED } else { CREAM };
            // A soft fold across the middle catches the light.
            let fold = ((u * 8.0 + v * 3.0) - 3.6).abs() < 0.15;
            Some(if fold {
                mix(base, rgb(0xffffff), 0.3)
            } else {
                mix(base, rgb(0x6a3a3a), (1.0 - v) * 0.12)
            })
        },
    );
    line(
        scene,
        (front_left, front),
        (front_right, front),
        mix(CHECK_RED, rgb(0x3a1418), 0.5),
    );
    // A basket of good things, and a jar of jam.
    let (bx, by) = (front_right - 18, back - 4);
    for step in 0..=12 {
        let t = step as f32 / 12.0;
        let x = bx + 1 + (t * 14.0) as i32;
        let y = by + 4 - ((t * std::f32::consts::PI).sin() * 7.0) as i32;
        put(scene, x, y, WICKER.shadow);
    }
    rect(scene, bx, by + 4, 16, 9, WICKER.base);
    for y in by + 4..by + 13 {
        for x in bx..bx + 16 {
            if ((x - bx) / 2 + (y - by) / 2) % 2 == 0 {
                put(scene, x, y, WICKER.light);
            }
        }
    }
    hline(scene, bx - 1, by + 4, 18, WICKER.edge);
    hline(scene, bx, by + 12, 16, WICKER.shadow);
    for x in bx + 2..bx + 9 {
        put(scene, x, by + 3, if x % 2 == 0 { CHECK_RED } else { CREAM });
    }
    ellipse(scene, bx + 11, by + 3, 2, 1, rgb(0xd8a060));
    put(scene, bx + 10, by + 2, rgb(0xf0c88a));
    let jar = bx - 8;
    rect(scene, jar, by + 8, 4, 5, rgb(0xb83a40));
    hline(scene, jar, by + 7, 4, CREAM);
    put(scene, jar, by + 9, rgb(0xf2a0a0));
}

// ---------------------------------------------------------------------------------------------
// The oak
// ---------------------------------------------------------------------------------------------

fn oak(scene: &mut Canvas) {
    // Dappled shade on the lawn beneath, with flecks of sun coming through.
    for y in 136..172 {
        for x in -10..170 {
            let (dx, dy) = ((x - 74) as f32 / 86.0, (y - 154) as f32 / 16.0);
            if dx * dx + dy * dy <= 1.0 && !chance(x / 2, y, 351, 40) {
                put(scene, x, y, rgba(0x24452b, 46));
            }
        }
    }
    // The swing, hanging from the long limb.
    ellipse(scene, 120, 152, 12, 2, rgba(0x24452b, 60));
    for rope in [112, 126] {
        for y in 64..128 {
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
    bevel(scene, 106, 128, 26, 3, PLANK);
    // The trunk: flared at the foot, furrowed, lit on its left.
    polygon(
        scene,
        &[(16, 150), (28, 130), (54, 130), (68, 150)],
        |_, _| Some(BARK.base),
    );
    rect(scene, 28, 40, 26, 92, BARK.base);
    for x in 16..68 {
        for y in 40..150 {
            if scene.get(x, y) != BARK.base {
                continue;
            }
            let across = (x - 41) as f32 / 13.0;
            let mut color = if across < -0.55 {
                BARK.light
            } else if across > 0.6 {
                BARK.shadow
            } else {
                BARK.base
            };
            let furrow = noise(x, y / 6, 352).is_multiple_of(9);
            if furrow {
                color = mix(color, BARK.edge, 0.7);
            } else if chance(x, y, 353, 20) {
                color = mix(color, BARK.shine, 0.4);
            }
            scene.set(x, y, color);
        }
    }
    vline(scene, 28, 40, 92, BARK.edge);
    vline(scene, 53, 40, 92, BARK.edge);
    ellipse(scene, 40, 104, 3, 4, BARK.edge);
    ellipse(scene, 40, 104, 2, 3, rgb(0x2a1e18));
    hline(scene, 38, 100, 4, BARK.light);
    // Roots into the grass.
    hline(scene, 14, 150, 56, rgba(0x2a1e18, 110));
    // The long limb the swing hangs from.
    polygon(
        scene,
        &[(50, 62), (140, 58), (140, 63), (50, 74)],
        |x, y| {
            let top = 62.0 - (x - 50) as f32 * 4.0 / 90.0;
            Some(if (y as f32) < top + 1.5 {
                BARK.light
            } else {
                BARK.base
            })
        },
    );
    line(scene, (50, 74), (140, 63), BARK.edge);
    // The crown: overlapping masses of leaves, lit from the upper left, darker underneath.
    let masses = [
        (-4, 6, 26),
        (30, -4, 28),
        (66, 8, 26),
        (100, 0, 24),
        (128, 20, 18),
        (18, 36, 24),
        (54, 40, 22),
        (90, 34, 20),
        (120, 46, 14),
        (148, 40, 12),
        (-8, 52, 18),
        (34, 62, 14),
        (70, 64, 12),
    ];
    for (index, &(cx, cy, r)) in masses.iter().enumerate() {
        leaf_mass(scene, cx, cy, r, 360 + index as u32);
    }
}

fn leaf_mass(scene: &mut Canvas, cx: i32, cy: i32, radius: i32, salt: u32) {
    // A ragged rim of leaf clumps, so no mass is a perfect circle.
    for step in 0..radius * 3 {
        let angle = step as f32 / (radius * 3) as f32 * std::f32::consts::TAU;
        let reach = radius as f32 + (noise(step, 0, salt) % 3) as f32 - 0.5;
        let (x, y) = (
            cx + (angle.cos() * reach).round() as i32,
            cy + 2 + (angle.sin() * (reach - 2.0)).round() as i32,
        );
        let clump = 1 + (noise(step, 1, salt) % 2) as i32;
        ellipse(scene, x, y, clump, clump, CANOPY.edge);
        if angle.sin() < -0.2 {
            put(scene, x, y - 1, CANOPY.shadow);
        }
    }
    ellipse(scene, cx, cy + 2, radius, radius - 2, CANOPY.edge);
    ellipse(scene, cx, cy, radius - 1, radius - 2, CANOPY.shadow);
    ellipse(scene, cx - 2, cy - 2, radius - 4, radius - 5, CANOPY.base);
    for y in cy - radius..=cy + radius {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy > (radius - 2) * (radius - 2) {
                continue;
            }
            // Leaves catch the light in clusters, not pixel by pixel.
            let cluster = noise(x / 2, y / 2, salt);
            if dx + dy < -radius / 3 && cluster.is_multiple_of(4) {
                put(scene, x, y, CANOPY.light);
            } else if dx + dy < -radius && cluster % 7 == 1 {
                put(scene, x, y, CANOPY.shine);
            } else if dx + dy > radius / 2 && cluster.is_multiple_of(3) {
                put(scene, x, y, CANOPY.edge);
            }
        }
    }
}

/// Flags strung from the oak's limb to the pole, sagging between.
fn bunting(scene: &mut Canvas) {
    let (from, to, sag) = ((138.0, 64.0), (375.0, 60.0), 16.0);
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
    for index in 0..28 {
        let (x, y) = at((index as f32 + 0.5) / 28.0);
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
        // Nothing in front of where anyone's head would be.
        let (_, top, _, _) = front.alpha_bounds().unwrap();
        assert!(top as f32 > WALK_BOTTOM - 10.0);
    }

    #[test]
    fn the_green_is_the_same_every_time() {
        assert_eq!(backdrop(), backdrop());
    }
}
