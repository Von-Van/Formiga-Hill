//! The station's scenery, painted back to front: sky, the Hill, the garden behind the fence, the
//! platform and the line, then the station house, the canopy, and the things standing about.
//!
//! Everything here is painted once into the backdrop, except the chimney smoke, which drifts.

use crate::font::{GLYPH_HEIGHT, draw_text_shadowed, text_width};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::kit::{Courses, bush, flower_box, plaster, ridge_tiles, roof, stonework, timber};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline,
};
use formiga_art::Canvas;

use super::{SCENE_HEIGHT, SCENE_WIDTH};

/// The back edge of the platform, where the house, the fence and the lamp stand.
pub const PLATFORM_BACK: i32 = 146;
/// Where feet meet the boards: well in from either edge, so everyone is plainly on them.
pub const STAND_Y: i32 = 160;
const SAFETY_LINE: i32 = 166;
const COPING: i32 = 168;
const FACE_TOP: i32 = 173;
const TRACK_TOP: i32 = 187;
const FAR_RAIL: i32 = 190;
const NEAR_RAIL: i32 = 199;
const BANK_TOP: i32 = 206;
const HORIZON: i32 = 132;

/// Where the chimney breathes from, for the smoke.
const CHIMNEY_TOP: (i32, i32) = (101, 52);

/// The fixed part of the scene.
/// The display case of kept souvenirs, set into the station house's left wall: its left, top,
/// right and bottom edges, for knowing when the pointer is over it.
pub const CASE: (i32, i32, i32, i32) = (15, 101, 43, 134);

/// The Hill as the station sees it: the tree at the top of the near hill, a little way off.
const VISTA: Vista = Vista {
    tree: (307.0, 68.0),
    crest: |x| 160.0 - 92.0 * (1.0 - ((x - 312.0) / 112.0).powi(2)).max(0.0).sqrt(),
    spread: 6.0,
    shrink: 5,
    tint: Tint::Haze(HILL_SHADE, 0.35),
};

/// The scene behind everyone, with whatever souvenirs the colony has kept in the display case
/// and whatever stands on the Hilltop, up on the skyline.
pub fn backdrop(keepsakes: &[String], hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    hills(&mut scene);
    skyline(&mut scene, hilltop, &VISTA);
    garden(&mut scene);
    platform(&mut scene);
    track(&mut scene);
    bank(&mut scene);
    canopy_shade(&mut scene);
    station_house(&mut scene);
    canopy(&mut scene);
    nameboard(&mut scene);
    lamp(&mut scene, 356);
    display_case(&mut scene, keepsakes);
    bench(&mut scene, 186);
    planter(&mut scene, 130);
    luggage(&mut scene, 284);
    potted_fern(&mut scene, 368);
    scene
}

/// Smoke from the chimney, `elapsed` seconds in. Still when motion is reduced: three puffs held
/// where they would be partway up.
pub fn smoke(scene: &mut Canvas, elapsed: f32, reduce_motion: bool) {
    const PUFFS: usize = 4;
    for index in 0..PUFFS {
        let offset = index as f32 / PUFFS as f32;
        let age = if reduce_motion {
            offset * 0.8 + 0.1
        } else {
            (elapsed * 0.22 + offset).fract()
        };
        let x = CHIMNEY_TOP.0 as f32 + age * 22.0 + (age * 9.0).sin() * 1.5;
        let y = CHIMNEY_TOP.1 as f32 - age * 34.0;
        let radius = 2.0 + age * 4.5;
        let fade = ((1.0 - age) * 150.0) as u8;
        let (cx, cy, r) = (x.round() as i32, y.round() as i32, radius.round() as i32);
        ellipse(scene, cx + 1, cy + 1, r, r - 1, rgba(0xc9c4c4, fade / 2));
        ellipse(scene, cx, cy, r, r - 1, rgba(0xf4f2ef, fade));
    }
}

/// How often the smoke changes enough to be worth drawing again.
pub const SMOKE_STEPS_PER_SECOND: f32 = 8.0;

// ---------------------------------------------------------------------------------------------
// Far away
// ---------------------------------------------------------------------------------------------

fn sky(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // In bands rather than a smooth ramp, as pixel skies are.
    for y in 0..HORIZON {
        let band = mix(SKY_TOP, SKY_LOW, (y * 6 / HORIZON) as f32 / 5.0);
        scene.fill_rect(0, y, width, 1, band);
    }
    for (x, y, size) in [(58, 30, 9), (300, 22, 11), (236, 56, 7)] {
        scene.fill_ellipse(x, y, size * 2, size / 2 + 2, CLOUD);
        scene.fill_ellipse(x - size / 2, y - 3, size, size / 2 + 1, CLOUD);
        scene.fill_ellipse(x + size / 2, y - 4, size, size / 2 + 2, CLOUD);
    }
}

fn hills(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    scene.fill_ellipse(70, 150, 120, 42, FAR_HILL);
    scene.fill_ellipse(200, 156, 140, 34, FAR_HILL);
    scene.fill_ellipse(312, 160, 112, 92, HILL);
    scene.fill_ellipse(340, 168, 70, 70, HILL_SHADE);
    // The way up: from the garden gate to the tree at the top.
    let path = [
        (252, 146),
        (250, 134),
        (292, 122),
        (276, 106),
        (314, 92),
        (300, 80),
        (306, 72),
    ];
    for pair in path.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        scene.line(x0, y0, x1, y1, 2, PATH);
    }
    scene.fill_rect(306, 62, 3, 10, TRUNK);
    scene.fill_circle(307, 58, 8, LEAVES);
    scene.fill_circle(301, 61, 5, LEAVES);
    scene.fill_circle(313, 61, 5, LEAVES);

    scene.fill_rect(0, HORIZON, width, PLATFORM_BACK - HORIZON, GRASS);
    for x in (0..width).step_by(7) {
        scene.set(x, HORIZON + 3 + (x % 5), GRASS_DARK);
        scene.set(x + 3, HORIZON + 9 + (x % 3), GRASS_DARK);
    }
    // The path again where it crosses the grass, so it reaches the gate.
    scene.line(252, PLATFORM_BACK, 250, HORIZON, 2, PATH);
}

// ---------------------------------------------------------------------------------------------
// The garden behind the platform
// ---------------------------------------------------------------------------------------------

fn garden(scene: &mut Canvas) {
    for (index, &(x, r)) in [
        (142, 7),
        (160, 6),
        (184, 8),
        (206, 6),
        (226, 7),
        (278, 7),
        (298, 6),
        (322, 8),
        (346, 6),
        (378, 8),
    ]
    .iter()
    .enumerate()
    {
        bush(scene, x, 136, r, index as u32);
    }
    fence(scene);
}

/// White pickets right of the house, with a gate where the path starts up the Hill.
fn fence(scene: &mut Canvas) {
    const TOP: i32 = 133;
    const GATE: (i32, i32) = (244, 260);
    let right = SCENE_WIDTH as i32;
    for x in (130..right).step_by(5) {
        if x + 3 > GATE.0 && x < GATE.1 {
            continue;
        }
        // A picket with a pointed top, lit on the left.
        put(scene, x + 1, TOP, TRIM.base);
        rect(scene, x, TOP + 1, 3, PLATFORM_BACK - TOP - 1, TRIM.base);
        vline(scene, x, TOP + 1, PLATFORM_BACK - TOP - 1, TRIM.light);
        vline(scene, x + 2, TOP + 1, PLATFORM_BACK - TOP - 1, TRIM.shadow);
        vline(
            scene,
            x + 3,
            TOP + 2,
            PLATFORM_BACK - TOP - 2,
            rgba(0x2c5233, 70),
        );
    }
    for rail in [TOP + 4, TOP + 9] {
        for x in 130..right {
            if x >= GATE.0 && x < GATE.1 {
                continue;
            }
            put(scene, x, rail, TRIM.light);
            put(scene, x, rail + 1, TRIM.shadow);
        }
    }
    for post in [GATE.0 - 3, GATE.1] {
        bevel(scene, post, TOP - 1, 4, PLATFORM_BACK - TOP + 1, TRIM);
        ellipse(scene, post + 1, TOP - 2, 2, 2, TRIM.light);
        put(scene, post + 1, TOP - 3, TRIM.shine);
    }
}

// ---------------------------------------------------------------------------------------------
// The platform and the line
// ---------------------------------------------------------------------------------------------

fn platform(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // Boards laid along the platform, four pixels to a row, joints staggered row by row.
    for y in PLATFORM_BACK..SAFETY_LINE {
        let row = (y - PLATFORM_BACK) / 4;
        let within = (y - PLATFORM_BACK) % 4;
        for x in 0..width {
            let shifted = x + row * 23;
            let plank = shifted.div_euclid(41);
            let joint = shifted.rem_euclid(41) == 0;
            let tone = noise(plank, row, 7) % 5;
            let mut color = match tone {
                0 => mix(PLANK.base, PLANK.shadow, 0.35),
                1 => mix(PLANK.base, PLANK.light, 0.4),
                _ => PLANK.base,
            };
            if within == 0 {
                color = mix(color, PLANK.light, 0.6);
            } else if within == 3 {
                color = PLANK.shadow;
            } else if chance(x / 3, y, 8 + plank as u32, 14) {
                color = mix(color, PLANK.shadow, 0.6);
            }
            if joint {
                color = if within == 3 {
                    PLANK.edge
                } else {
                    PLANK.shadow
                };
            }
            scene.set(x, y, color);
            // Two nails beside every joint.
            if within == 1 && matches!(shifted.rem_euclid(41), 2 | 39) {
                scene.set(x, y, mix(PLANK.shadow, STONE.shadow, 0.5));
            }
        }
    }
    // Back rows sit a little in the shade of everything along the back edge.
    for x in 0..width {
        put(scene, x, PLATFORM_BACK, rgba(0x3a2a30, 70));
        put(scene, x, PLATFORM_BACK + 1, rgba(0x3a2a30, 30));
    }

    // The painted edge, then the coping stones, then the face of the platform down to the line.
    hline(scene, 0, SAFETY_LINE, width, CREAM);
    hline(
        scene,
        0,
        SAFETY_LINE + 1,
        width,
        mix(CREAM, PLANK.shadow, 0.35),
    );
    for y in COPING..FACE_TOP {
        for x in 0..width {
            let block = (x + 5).rem_euclid(18);
            let color = match y - COPING {
                0 => STONE.shine,
                1 | 2 => STONE.light,
                3 => STONE.base,
                _ => STONE.shadow,
            };
            scene.set(x, y, if block == 0 { STONE.shadow } else { color });
        }
    }
    hline(scene, 0, FACE_TOP, width, STONE.edge);
    // Right down to the ballast: the shade line below only darkens the last course.
    let face = (0, FACE_TOP + 1, width, TRACK_TOP - FACE_TOP - 1);
    stonework(scene, face, Courses { tall: 4, long: 14 }, 21);
    hline(scene, 0, TRACK_TOP - 1, width, rgba(0x2a2226, 150));
    weathering(scene);
}

/// Moss creeping up the foot of the platform wall where it stays damp, and weeds at its base.
fn weathering(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for x in 0..width {
        // Patches rather than an even fringe: moss grows where the wall stays wettest.
        let patch = noise(x / 9, 0, 141).is_multiple_of(4);
        let reach = if patch {
            2 + (noise(x, 1, 142) % 4) as i32
        } else {
            (noise(x, 2, 143) % 2) as i32
        };
        for step in 0..reach {
            let y = TRACK_TOP - 2 - step;
            let tone = if step + 1 == reach {
                LEAF.light
            } else {
                LEAF.shadow
            };
            if chance(x, y, 144, 200) {
                put(scene, x, y, mix(tone, STONE.shadow, 0.35));
            }
        }
        if chance(x, 3, 145, 22) {
            let tall = 2 + (noise(x, 4, 146) % 3) as i32;
            vline(scene, x, TRACK_TOP + 1, tall, LEAF.base);
            put(scene, x, TRACK_TOP, LEAF.light);
            put(scene, x + 1, TRACK_TOP + 1, LEAF.shadow);
        }
    }
}

fn track(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for y in TRACK_TOP..BANK_TOP {
        for x in 0..width {
            // Mostly middling greys, so the sleepers and rails stand out of it.
            let stone = match noise(x, y, 31) % 16 {
                0 | 1 => BALLAST[0],
                2..=8 => BALLAST[1],
                9..=12 => BALLAST[2],
                13 | 14 => BALLAST[4],
                _ => BALLAST[3],
            };
            scene.set(x, y, stone);
        }
    }
    // Shade near the foot of the platform.
    for y in TRACK_TOP..TRACK_TOP + 3 {
        hline(
            scene,
            0,
            y,
            width,
            rgba(0x2a2226, 70 - (y - TRACK_TOP) as u8 * 20),
        );
    }

    for x in (2..width).step_by(13) {
        let (top, bottom) = (FAR_RAIL - 2, BANK_TOP - 1);
        rect(scene, x, top, 7, bottom - top, SLEEPER.base);
        hline(scene, x, top, 7, SLEEPER.light);
        vline(scene, x, top, bottom - top, SLEEPER.light);
        vline(scene, x + 6, top, bottom - top, SLEEPER.shadow);
        hline(scene, x, bottom - 1, 7, SLEEPER.edge);
        // Grain, and a crack in the odd one.
        put(scene, x + 2, top + 5, SLEEPER.shadow);
        put(scene, x + 4, top + 10, SLEEPER.shadow);
        if noise(x, 0, 32).is_multiple_of(3) {
            vline(scene, x + 3, top + 6, 3, SLEEPER.edge);
        }
        // Each sleeper sits down in the stones.
        vline(scene, x + 7, top + 1, bottom - top, rgba(0x2a2226, 90));
        vline(scene, x - 1, top + 1, bottom - top, rgba(0x2a2226, 40));
        // The chairs that hold the rails down.
        for rail in [FAR_RAIL, NEAR_RAIL] {
            rect(scene, x + 1, rail + 2, 5, 2, RAIL.edge);
            hline(scene, x + 1, rail + 2, 5, RAIL.shadow);
        }
    }

    for (rail, depth) in [(FAR_RAIL, 3), (NEAR_RAIL, 4)] {
        hline(scene, 0, rail, width, RAIL.shine);
        hline(scene, 0, rail + 1, width, RAIL.light);
        for row in 2..depth {
            hline(scene, 0, rail + row, width, RAIL.base);
        }
        hline(scene, 0, rail + depth, width, RAIL.shadow);
        hline(scene, 0, rail + depth + 1, width, rgba(0x2a2226, 80));
    }
}

/// The grass bank in front of the line.
fn bank(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    rect(scene, 0, BANK_TOP, width, height - BANK_TOP, GRASS);
    hline(scene, 0, BANK_TOP, width, GRASS_LIGHT);
    for y in BANK_TOP + 1..height {
        for x in 0..width {
            if chance(x, y, 51, 26) {
                put(scene, x, y, GRASS_DARK);
            } else if chance(x, y, 52, 14) {
                put(scene, x, y, GRASS_LIGHT);
            }
        }
    }
    // Tufts reaching up into the ballast, and a few flowers among them.
    for x in 0..width {
        if chance(x, 0, 53, 70) {
            let tall = 1 + (noise(x, 1, 54) % 3) as i32;
            vline(scene, x, BANK_TOP - tall, tall, GRASS_DARK);
            put(scene, x, BANK_TOP - tall, GRASS);
        }
        if chance(x, 2, 55, 9) {
            let y = BANK_TOP + 2 + (noise(x, 3, 56) % 6) as i32;
            let blossom = BLOSSOMS[(noise(x, 4, 57) % 4) as usize + 1];
            put(scene, x, y, blossom);
            put(scene, x - 1, y, mix(blossom, GRASS, 0.5));
            put(scene, x + 1, y, mix(blossom, GRASS, 0.5));
            put(scene, x, y - 1, mix(blossom, GRASS, 0.3));
            put(scene, x, y + 1, GRASS_DARK);
        }
    }
    // The bottom edge falls into a little shade, which holds the eye in the scene.
    for (index, y) in (height - 3..height).enumerate() {
        hline(scene, 0, y, width, rgba(0x2c4a2e, 30 + index as u8 * 25));
    }
}

// ---------------------------------------------------------------------------------------------
// The station house
// ---------------------------------------------------------------------------------------------

const HOUSE_LEFT: i32 = 10;
const HOUSE_RIGHT: i32 = 126;
const RIDGE: i32 = 70;
const EAVES: i32 = 96;
const WALL_TOP: i32 = 98;
const PLINTH: i32 = 136;

fn station_house(scene: &mut Canvas) {
    chimney(scene);
    roof(
        scene,
        &[
            (HOUSE_LEFT - 8, EAVES),
            (HOUSE_LEFT + 18, RIDGE),
            (HOUSE_RIGHT - 18, RIDGE),
            (HOUSE_RIGHT + 8, EAVES),
        ],
        RIDGE,
        EAVES,
    );
    ridge_tiles(scene, HOUSE_LEFT + 17, HOUSE_RIGHT - 17, RIDGE);
    dormer(scene, 64);

    // Fascia board under the eaves.
    rect(
        scene,
        HOUSE_LEFT - 8,
        EAVES,
        HOUSE_RIGHT - HOUSE_LEFT + 16,
        2,
        TIMBER.base,
    );
    hline(
        scene,
        HOUSE_LEFT - 8,
        EAVES,
        HOUSE_RIGHT - HOUSE_LEFT + 16,
        TIMBER.light,
    );
    hline(
        scene,
        HOUSE_LEFT - 8,
        EAVES + 2,
        HOUSE_RIGHT - HOUSE_LEFT + 16,
        TIMBER.edge,
    );

    plaster(
        scene,
        HOUSE_LEFT,
        WALL_TOP,
        HOUSE_RIGHT - HOUSE_LEFT,
        PLINTH - WALL_TOP,
    );
    // The timber frame: posts, the top plate and the sill.
    for post in [HOUSE_LEFT, 44, 92, HOUSE_RIGHT - 4] {
        timber(scene, post, WALL_TOP, 4, PLINTH - WALL_TOP, true);
    }
    timber(
        scene,
        HOUSE_LEFT,
        WALL_TOP,
        HOUSE_RIGHT - HOUSE_LEFT,
        3,
        false,
    );
    timber(
        scene,
        HOUSE_LEFT,
        PLINTH - 3,
        HOUSE_RIGHT - HOUSE_LEFT,
        3,
        false,
    );
    // The eaves throw a band of shade down the wall.
    for (index, y) in (WALL_TOP + 3..WALL_TOP + 7).enumerate() {
        hline(
            scene,
            HOUSE_LEFT,
            y,
            HOUSE_RIGHT - HOUSE_LEFT,
            rgba(0x4a3040, 70 - index as u8 * 16),
        );
    }

    flower_box(scene, 16, 136, 26);
    door(scene, 52, 106);
    timetable(scene, 76, 109);
    ticket_window(scene, 98, 110);

    let plinth = (
        HOUSE_LEFT - 2,
        PLINTH,
        HOUSE_RIGHT - HOUSE_LEFT + 4,
        PLATFORM_BACK - PLINTH,
    );
    stonework(scene, plinth, Courses { tall: 5, long: 11 }, 61);
    hline(
        scene,
        HOUSE_LEFT - 2,
        PLINTH,
        HOUSE_RIGHT - HOUSE_LEFT + 4,
        STONE.shine,
    );
}

fn chimney(scene: &mut Canvas) {
    let (left, top, width, bottom): (i32, i32, i32, i32) = (95, 58, 12, 80);
    for y in top..bottom {
        let row = (y - top) / 3;
        let within = (y - top) % 3;
        for x in left..left + width {
            let shifted = x + if row % 2 == 1 { 2 } else { 0 };
            let color = if within == 2 || shifted.rem_euclid(4) == 0 {
                BRICK.shadow
            } else if noise(shifted / 4, row, 91).is_multiple_of(4) {
                BRICK.light
            } else {
                BRICK.base
            };
            scene.set(x, y, color);
        }
    }
    vline(scene, left, top, bottom - top, BRICK.edge);
    vline(scene, left + width - 1, top, bottom - top, BRICK.edge);
    vline(
        scene,
        left + width - 2,
        top,
        bottom - top,
        rgba(0x3a1a1a, 70),
    );
    bevel(scene, left - 1, top - 3, width + 2, 4, STONE);
    // Two pots.
    for pot in [left + 2, left + 7] {
        bevel(scene, pot, top - 7, 3, 5, BRICK);
    }
}

/// A little gabled dormer with the station clock in it.
fn dormer(scene: &mut Canvas, centre: i32) {
    let (left, right) = (centre - 10, centre + 10);
    plaster(scene, left, 82, right - left, 14);
    vline(scene, left, 82, 14, TIMBER.base);
    vline(scene, right - 1, 82, 14, TIMBER.shadow);
    hline(scene, left, 95, right - left, TIMBER.edge);
    // It throws a little shade across the roof to its right.
    for y in 76..96 {
        hline(scene, right, y, 4 - (y - 76) / 6, rgba(0x3a1a28, 70));
    }
    roof(
        scene,
        &[(left - 3, 84), (centre, 72), (right + 3, 84)],
        72,
        84,
    );
    // Timber bargeboards along the gable, lit on the left slope, shaded on the right.
    for offset in 0..2 {
        line(
            scene,
            (left - 3, 84 + offset),
            (centre, 72 + offset),
            TIMBER.light,
        );
        line(
            scene,
            (centre, 72 + offset),
            (right + 3, 84 + offset),
            TIMBER.base,
        );
    }
    line(scene, (left - 3, 86), (centre, 74), TIMBER.edge);
    line(scene, (centre, 74), (right + 3, 86), TIMBER.edge);
    put(scene, centre, 71, TIMBER.shine);
    // Under the gable, the face sits in shade.
    hline(scene, left + 1, 82, right - left - 2, rgba(0x4a3040, 60));

    // The clock: a brass rim, a cream face, the quarters marked, and ten past ten.
    let (cx, cy) = (centre, 89);
    ellipse(scene, cx, cy, 6, 6, rgb(0x6b4a24));
    ellipse(scene, cx, cy, 5, 5, rgb(0xc9a14e));
    put(scene, cx - 3, cy - 4, rgb(0xf1d58a));
    ellipse(scene, cx, cy, 4, 4, CREAM);
    for (dx, dy) in [(0, -3), (3, 0), (0, 3), (-3, 0)] {
        put(scene, cx + dx, cy + dy, TIMBER.shadow);
    }
    line(scene, (cx, cy), (cx - 2, cy - 1), TIMBER.edge);
    line(scene, (cx, cy), (cx + 2, cy - 2), TIMBER.edge);
    put(scene, cx, cy, rgb(0x8c3d39));
}

fn door(scene: &mut Canvas, x: i32, y: i32) {
    let (width, height) = (22, PLINTH - y);
    bevel(scene, x, y, width, height, TRIM);
    let (inner_x, inner_y, inner_w, inner_h) = (x + 2, y + 2, width - 4, height - 2);
    // A fanlight of glass over the door.
    rect(scene, inner_x, inner_y, inner_w, 5, GLASS[1]);
    hline(scene, inner_x, inner_y, inner_w, GLASS[0]);
    for bar in [inner_x + inner_w / 3, inner_x + inner_w * 2 / 3] {
        vline(scene, bar, inner_y, 5, TRIM.base);
    }
    put(scene, inner_x + 2, inner_y + 2, GLASS[3]);
    hline(scene, inner_x, inner_y + 5, inner_w, TRIM.base);
    // Two leaves of panelled door.
    let leaf_top = inner_y + 6;
    let leaf_h = inner_h - 6;
    rect(scene, inner_x, leaf_top, inner_w, leaf_h, DOOR.base);
    vline(scene, inner_x + inner_w / 2, leaf_top, leaf_h, DOOR.edge);
    for leaf in [inner_x, inner_x + inner_w / 2 + 1] {
        for (top, tall) in [(leaf_top + 2, 8), (leaf_top + 12, leaf_h - 14)] {
            let panel_w = inner_w / 2 - 3;
            rect(scene, leaf + 1, top, panel_w, tall, DOOR.shadow);
            hline(scene, leaf + 1, top, panel_w, DOOR.edge);
            vline(scene, leaf + 1, top, tall, DOOR.edge);
            hline(scene, leaf + 1, top + tall - 1, panel_w, DOOR.light);
            vline(scene, leaf + panel_w, top, tall, DOOR.light);
        }
    }
    vline(scene, inner_x, leaf_top, leaf_h, DOOR.light);
    // Brass knobs.
    for knob in [inner_x + inner_w / 2 - 2, inner_x + inner_w / 2 + 2] {
        put(scene, knob, leaf_top + 11, rgb(0xc9a14e));
        put(scene, knob, leaf_top + 10, rgb(0xf1d58a));
    }
    // The step.
    bevel(scene, x - 2, PLINTH - 1, width + 4, 3, STONE);
}

/// A board of departures: illegible at this size, as a real one is from across the platform.
fn timetable(scene: &mut Canvas, x: i32, y: i32) {
    let (width, height) = (14, 18);
    bevel(scene, x, y, width, height, TIMBER);
    rect(scene, x + 2, y + 2, width - 4, height - 4, rgb(0x2a3532));
    hline(scene, x + 2, y + 3, width - 4, CREAM);
    for row in 0..5 {
        let line_y = y + 6 + row * 2;
        let length = 4 + (noise(row, x, 111) % 5) as i32;
        hline(scene, x + 3, line_y, length, rgba(0xf3e9cf, 190));
        put(scene, x + width - 4, line_y, rgb(0xf5d25e));
    }
}

fn ticket_window(scene: &mut Canvas, x: i32, y: i32) {
    let (width, height) = (22, 17);
    // A striped awning, scalloped along its lower edge.
    let awning_top = y - 6;
    for py in awning_top..awning_top + 5 {
        for px in x - 2..x + width + 2 {
            let stripe = (px - x + 2).div_euclid(3) % 2 == 0;
            let color = if stripe { rgb(0xc75a4a) } else { CREAM };
            let scallop = (px - x + 2).rem_euclid(3);
            if py == awning_top + 4 && scallop != 1 {
                continue;
            }
            let shaded = if py == awning_top {
                mix(color, rgb(0xffffff), 0.2)
            } else {
                color
            };
            scene.set(px, py, shaded);
        }
    }
    hline(scene, x - 2, awning_top - 1, width + 4, TIMBER.edge);
    for (index, py) in (awning_top + 5..awning_top + 8).enumerate() {
        hline(scene, x, py, width, rgba(0x4a3040, 80 - index as u8 * 25));
    }

    bevel(scene, x, y, width, height, TIMBER);
    // A dim office inside, with the clerk's lamp.
    for py in y + 2..y + height - 2 {
        let t = (py - y) as f32 / height as f32;
        hline(
            scene,
            x + 2,
            py,
            width - 4,
            mix(rgb(0x5a4a44), rgb(0x2e2628), t),
        );
    }
    // Pigeonholes of tickets along the back wall.
    for row in 0..2 {
        for column in 0..4 {
            let (hx, hy) = (x + 4 + column * 4, y + 6 + row * 4);
            rect(scene, hx, hy, 3, 3, rgb(0x241d1f));
            let ticket = BLOSSOMS[(noise(column, row, 112) % 4) as usize];
            hline(scene, hx, hy + 2, 2, mix(ticket, rgb(0x5a4a44), 0.35));
        }
    }
    // A lamp on a cord, and the pool of light it makes.
    let lamp_x = x + width / 2;
    vline(scene, lamp_x, y + 2, 2, rgb(0x241d1f));
    hline(scene, lamp_x - 1, y + 4, 3, rgb(0x3b5a50));
    put(scene, lamp_x, y + 5, GLOW[2]);
    ellipse(scene, lamp_x, y + 9, 6, 4, rgba(0xffd77a, 34));
    // The glass, catching the sky along its top.
    hline(scene, x + 2, y + 2, width - 4, rgba(0xa6c8d2, 150));
    put(scene, x + 3, y + 3, rgba(0xe9f6f7, 170));
    // The counter.
    bevel(scene, x - 1, y + height - 3, width + 2, 3, PLANK);
}

/// A glass-fronted case in the wall, in the station's green enamel, lined in velvet, with a place
/// on its two shelves for each of Hill's souvenirs. Those kept sit in their places; the rest are
/// empty cushions, waiting, never a list of what is missing.
fn display_case(scene: &mut Canvas, keepsakes: &[String]) {
    let (left, top, right, bottom) = CASE;
    let width = right - left;
    let (glass_top, sill) = (top + 3, bottom - 4);
    // Shade where it stands out from the wall.
    vline(scene, right, top + 2, bottom - top - 2, rgba(0x4a3040, 80));
    // The cornice, with a brass finial, and the stone sill below.
    bevel(scene, left - 1, top, width + 2, 4, IRON);
    put(scene, left + width / 2, top - 1, rgb(0xc9a14e));
    put(scene, left + width / 2, top, rgb(0xf1d58a));
    bevel(scene, left - 2, sill, width + 4, 4, STONE);
    // The frame, and the velvet behind the glass.
    rect(scene, left, glass_top, width, sill - glass_top, IRON.base);
    vline(scene, left, glass_top, sill - glass_top, IRON.light);
    vline(scene, right - 1, glass_top, sill - glass_top, IRON.edge);
    let (inner_left, inner_top) = (left + 2, glass_top + 2);
    let (inner_width, inner_height) = (width - 4, sill - glass_top - 3);
    for y in inner_top..inner_top + inner_height {
        let t = (y - inner_top) as f32 / inner_height as f32;
        hline(
            scene,
            inner_left,
            y,
            inner_width,
            mix(rgb(0x7a2c3a), rgb(0x3e1620), t),
        );
    }
    // Two shelves, three places on each, in the catalogue's order.
    let shelves = [inner_top + 8, inner_top + 20];
    for shelf in shelves {
        hline(scene, inner_left, shelf, inner_width, PLANK.light);
        hline(scene, inner_left, shelf + 1, inner_width, PLANK.shadow);
    }
    for (index, id) in crate::story::souvenirs::ids().into_iter().enumerate() {
        let shelf = shelves[index / 3];
        let x = inner_left + (index % 3) as i32 * (crate::keepsake_art::ICON + 1);
        if keepsakes.iter().any(|kept| kept == id) {
            crate::keepsake_art::draw_souvenir(scene, id, x, shelf - crate::keepsake_art::ICON);
        } else {
            // An empty cushion, waiting.
            ellipse(scene, x + 3, shelf - 1, 3, 1, rgb(0x5c2030));
            hline(scene, x + 1, shelf - 2, 5, rgb(0x96404e));
        }
    }
    // The glass: a faint sheen, two glints, and the line where the doors meet.
    rect(
        scene,
        inner_left,
        inner_top,
        inner_width,
        inner_height,
        rgba(0xa6c8d2, 26),
    );
    vline(
        scene,
        left + width / 2,
        glass_top + 1,
        sill - glass_top - 1,
        IRON.edge,
    );
    for (dx, dy) in [(0, 2), (1, 1), (2, 0)] {
        put(
            scene,
            right - 6 + dx,
            inner_top + 1 + dy,
            rgba(0xffffff, 120),
        );
    }
    for (dx, dy) in [(0, 2), (1, 1), (2, 0)] {
        put(
            scene,
            left + width / 2 + 3 + dx,
            inner_top + 9 + dy,
            rgba(0xffffff, 90),
        );
    }
    for handle in [left + width / 2 - 2, left + width / 2 + 1] {
        put(scene, handle, glass_top + 12, rgb(0xf1d58a));
    }
}

// ---------------------------------------------------------------------------------------------
// The canopy and the nameboard
// ---------------------------------------------------------------------------------------------

const CANOPY_LEFT: i32 = HOUSE_RIGHT;
const CANOPY_RIGHT: i32 = 246;
const CANOPY_ROOF: i32 = 99;
const VALANCE: i32 = 104;
const COLUMNS: [i32; 2] = [168, 228];

/// The shade under the canopy, laid down before anything stands in it.
fn canopy_shade(scene: &mut Canvas) {
    let width = CANOPY_RIGHT - CANOPY_LEFT;
    let (top, bottom) = (VALANCE + 6, PLATFORM_BACK);
    for y in top..bottom {
        let depth = 1.0 - (y - top) as f32 / (bottom - top) as f32;
        let alpha = (46.0 + depth * 44.0) as u8;
        hline(scene, CANOPY_LEFT, y, width, rgba(0x28303c, alpha));
    }
    for (index, y) in (PLATFORM_BACK..PLATFORM_BACK + 8).enumerate() {
        hline(
            scene,
            CANOPY_LEFT,
            y,
            width,
            rgba(0x28303c, 46 - index as u8 * 5),
        );
    }
}

fn canopy(scene: &mut Canvas) {
    for column in COLUMNS {
        iron_column(scene, column);
    }
    platform_plate(scene, COLUMNS[0]);

    // Corrugated roof, seen a little from above.
    for y in CANOPY_ROOF..VALANCE {
        for x in CANOPY_LEFT..CANOPY_RIGHT {
            let ridge = (x / 2) % 2 == 0;
            let color = match y - CANOPY_ROOF {
                0 => RAIL.shine,
                1 if ridge => RAIL.light,
                4 => RAIL.shadow,
                _ if ridge => RAIL.base,
                _ => mix(RAIL.base, RAIL.shadow, 0.5),
            };
            scene.set(x, y, color);
        }
    }
    vline(
        scene,
        CANOPY_RIGHT - 1,
        CANOPY_ROOF,
        VALANCE - CANOPY_ROOF,
        RAIL.edge,
    );

    // The valance: cream boards with a green band, cut into scallops along the bottom.
    for y in VALANCE..VALANCE + 7 {
        for x in CANOPY_LEFT..CANOPY_RIGHT {
            let across = (x - CANOPY_LEFT).rem_euclid(6);
            match y - VALANCE {
                0 => scene.set(x, y, IRON.base),
                1 => scene.set(x, y, IRON.shadow),
                5 if !(1..=4).contains(&across) => continue,
                6 if !(2..=3).contains(&across) => continue,
                row => {
                    let board = if across == 0 {
                        PLASTER.shadow
                    } else {
                        PLASTER.base
                    };
                    let lit = if row == 2 { PLASTER.light } else { board };
                    scene.set(x, y, lit);
                }
            }
        }
    }
    bevel(
        scene,
        CANOPY_RIGHT - 2,
        CANOPY_ROOF - 1,
        3,
        VALANCE - CANOPY_ROOF + 7,
        IRON,
    );
    // Outline the scallops so they hold against the sky behind.
    for x in CANOPY_LEFT..CANOPY_RIGHT {
        let across = (x - CANOPY_LEFT).rem_euclid(6);
        let lowest = match across {
            2 | 3 => VALANCE + 6,
            1 | 4 => VALANCE + 5,
            _ => VALANCE + 4,
        };
        put(scene, x, lowest + 1, rgba(0x6e5a48, 170));
    }
}

/// The platform's number, on an enamel plate on the first column.
fn platform_plate(scene: &mut Canvas, x: i32) {
    let (left, top) = (x - 3, 118);
    rect(scene, left, top, 9, 11, IRON.edge);
    rect(scene, left + 1, top + 1, 7, 9, CREAM);
    rect(scene, left + 2, top + 2, 5, 7, ENAMEL);
    crate::font::draw_text(scene, left + 2, top + 2, "1", CREAM);
    put(scene, left + 1, top + 1, rgba(0xffffff, 160));
}

fn iron_column(scene: &mut Canvas, x: i32) {
    let top = VALANCE + 6;
    rect(scene, x, top, 3, PLATFORM_BACK - top, IRON.base);
    vline(scene, x, top, PLATFORM_BACK - top, IRON.light);
    vline(scene, x + 2, top, PLATFORM_BACK - top, IRON.edge);
    // Fluting.
    for y in (top + 4..PLATFORM_BACK - 6).step_by(3) {
        put(scene, x + 1, y, IRON.shine);
    }
    // Capital and brackets.
    bevel(scene, x - 2, top, 7, 3, IRON);
    for side in [-1, 1] {
        let reach = 7;
        let (inner, outer) = if side < 0 {
            (x - 1, x - reach)
        } else {
            (x + 3, x + 2 + reach)
        };
        line(scene, (outer, top), (inner, top + reach), IRON.base);
        line(scene, (outer, top + 1), (inner, top + reach + 1), IRON.edge);
        ellipse(scene, (outer + inner) / 2, top + 2, 1, 1, IRON.light);
    }
    // Base.
    bevel(scene, x - 2, PLATFORM_BACK - 5, 7, 5, IRON);
    hline(scene, x - 3, PLATFORM_BACK, 9, rgba(0x2a2226, 90));
}

/// The station's name, in enamel, standing on the canopy against the sky.
fn nameboard(scene: &mut Canvas) {
    const NAME: &str = "FORMIGA HILL";
    let (left, top, width, height) = (143, 82, 86, 15);
    for leg in [left + 10, left + width - 13] {
        rect(
            scene,
            leg,
            top + height,
            3,
            CANOPY_ROOF - top - height + 1,
            IRON.base,
        );
        vline(
            scene,
            leg,
            top + height,
            CANOPY_ROOF - top - height + 1,
            IRON.light,
        );
    }
    rect(scene, left, top, width, height, IRON.edge);
    rect(scene, left + 1, top + 1, width - 2, height - 2, CREAM);
    rect(scene, left + 2, top + 2, width - 4, height - 4, ENAMEL);
    hline(
        scene,
        left + 2,
        top + 2,
        width - 4,
        mix(ENAMEL, CREAM, 0.25),
    );
    hline(scene, left + 2, top + height - 3, width - 4, ENAMEL_DEEP);
    for (x, y) in [
        (left + 1, top + 1),
        (left + width - 2, top + 1),
        (left + 1, top + height - 2),
        (left + width - 2, top + height - 2),
    ] {
        put(scene, x, y, rgb(0xc9a14e));
    }
    let text_x = left + (width - text_width(NAME)) / 2;
    let text_y = top + (height - GLYPH_HEIGHT) / 2;
    draw_text_shadowed(scene, text_x, text_y, NAME, CREAM, ENAMEL_DEEP);
    // A glint off the enamel.
    for (dx, dy) in [(0, 2), (1, 1), (2, 0)] {
        put(
            scene,
            left + width - 9 + dx,
            top + 2 + dy,
            rgba(0xffffff, 120),
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Things standing about
// ---------------------------------------------------------------------------------------------

fn lamp(scene: &mut Canvas, x: i32) {
    let base = PLATFORM_BACK;
    // A warm halo, faint by day.
    ellipse(scene, x + 1, 92, 11, 11, rgba(0xfff0bd, 34));
    ellipse(scene, x + 1, 92, 7, 7, rgba(0xfff0bd, 40));
    // Stepped foot, fluted post, a collar halfway up, and the ladder bar.
    bevel(scene, x - 3, base - 4, 9, 4, IRON);
    bevel(scene, x - 2, base - 8, 7, 4, IRON);
    rect(scene, x, 101, 3, base - 109, IRON.base);
    vline(scene, x, 101, base - 109, IRON.light);
    vline(scene, x + 2, 101, base - 109, IRON.edge);
    bevel(scene, x - 1, 122, 5, 3, IRON);
    bevel(scene, x - 5, 100, 13, 2, IRON);
    // The lantern: a cap, four panes of glowing glass, a finial.
    let (lx, ly) = (x - 4, 86);
    polygon(
        scene,
        &[
            (lx - 1, ly),
            (lx + 5, ly - 5),
            (lx + 6, ly - 5),
            (lx + 12, ly),
        ],
        |_, _| Some(IRON.base),
    );
    line(scene, (lx - 1, ly), (lx + 5, ly - 5), IRON.light);
    rect(scene, lx + 5, ly - 8, 2, 3, IRON.base);
    put(scene, lx + 5, ly - 9, rgb(0xc9a14e));
    rect(scene, lx, ly, 11, 12, IRON.edge);
    for (pane, glow) in [(lx + 1, GLOW[1]), (lx + 6, GLOW[0])] {
        rect(scene, pane, ly + 1, 4, 10, glow);
        vline(scene, pane, ly + 1, 10, GLOW[2]);
        put(scene, pane + 1, ly + 2, rgb(0xffffff));
    }
    hline(scene, lx - 1, ly + 12, 13, IRON.base);
    hline(scene, lx, ly + 13, 11, IRON.edge);
}

fn bench(scene: &mut Canvas, x: i32) {
    let width = 30;
    let seat = PLATFORM_BACK - 6;
    // Cast-iron ends.
    for end in [x + 2, x + width - 4] {
        rect(scene, end, seat - 7, 2, 13, IRON.base);
        vline(scene, end, seat - 7, 13, IRON.light);
        put(scene, end - 1, PLATFORM_BACK - 1, IRON.edge);
        put(scene, end + 2, PLATFORM_BACK - 1, IRON.edge);
    }
    // Back slats and seat slats.
    for slat in [seat - 7, seat - 4] {
        bevel(scene, x, slat, width, 2, PLANK);
    }
    bevel(scene, x - 1, seat, width + 2, 3, PLANK);
    hline(scene, x - 1, seat + 3, width + 2, rgba(0x2a2226, 80));
    hline(scene, x, PLATFORM_BACK, width, rgba(0x2a2226, 70));
}

fn planter(scene: &mut Canvas, x: i32) {
    let (width, top) = (12, PLATFORM_BACK - 9);
    // Flowers heaped above the barrel.
    ellipse(scene, x + width / 2, top - 2, 7, 4, LEAF.shadow);
    ellipse(scene, x + width / 2 - 1, top - 3, 6, 3, LEAF.base);
    for index in 0..9 {
        let fx = x + 1 + (noise(index, x, 121) % width as u32) as i32;
        let fy = top - 5 + (noise(x, index, 122) % 4) as i32;
        let blossom = BLOSSOMS[(index as usize) % 3];
        put(scene, fx, fy, blossom);
        put(scene, fx + 1, fy, mix(blossom, rgb(0xffffff), 0.3));
    }
    // A half barrel, staves and hoops.
    bevel(scene, x, top, width, 9, PLANK);
    for stave in (x + 3..x + width - 1).step_by(3) {
        vline(scene, stave, top + 1, 7, PLANK.shadow);
    }
    for hoop in [top + 2, top + 6] {
        hline(scene, x, hoop, width, IRON.base);
    }
    hline(scene, x, PLATFORM_BACK, width, rgba(0x2a2226, 80));
}

fn luggage(scene: &mut Canvas, x: i32) {
    // A travelling trunk with straps and brass corners.
    let (trunk_w, trunk_top) = (22, PLATFORM_BACK - 10);
    bevel(scene, x, trunk_top, trunk_w, 10, LEATHER);
    hline(scene, x + 1, trunk_top + 3, trunk_w - 2, LEATHER.shadow);
    for strap in [x + 5, x + trunk_w - 7] {
        vline(scene, strap, trunk_top + 1, 9, LEATHER.edge);
        vline(scene, strap + 1, trunk_top + 1, 9, rgb(0x2e1a12));
        put(scene, strap, trunk_top + 5, rgb(0xc9a14e));
    }
    for (cx, cy) in [
        (x, trunk_top),
        (x + trunk_w - 2, trunk_top),
        (x, trunk_top + 8),
        (x + trunk_w - 2, trunk_top + 8),
    ] {
        rect(scene, cx, cy, 2, 2, rgb(0xc9a14e));
    }
    // A small case on top, and a hatbox beside.
    let case = Ramp::new(0x1f3b44, 0x2e5562, 0x3f7180, 0x5a8f9c, 0x7fb0ba);
    bevel(scene, x + 3, trunk_top - 6, 14, 6, case);
    hline(scene, x + 8, trunk_top - 8, 4, LEATHER.base);
    put(scene, x + 7, trunk_top - 7, LEATHER.base);
    put(scene, x + 12, trunk_top - 7, LEATHER.base);
    let hatbox = Ramp::new(0x8a4a5e, 0xc4708a, 0xe79ab0, 0xf5bfcd, 0xffe3ea);
    bevel(scene, x + trunk_w + 1, PLATFORM_BACK - 7, 10, 7, hatbox);
    for stripe in (x + trunk_w + 3..x + trunk_w + 10).step_by(3) {
        vline(scene, stripe, PLATFORM_BACK - 6, 5, CREAM);
    }
    hline(
        scene,
        x - 1,
        PLATFORM_BACK,
        trunk_w + 13,
        rgba(0x2a2226, 80),
    );
}

fn potted_fern(scene: &mut Canvas, x: i32) {
    let top = PLATFORM_BACK - 6;
    for (dx, dy) in [
        (-5, -5),
        (-3, -8),
        (0, -9),
        (3, -8),
        (5, -5),
        (-1, -6),
        (2, -6),
    ] {
        line(scene, (x + 3, top), (x + 3 + dx, top + dy), LEAF.base);
        put(scene, x + 3 + dx, top + dy, LEAF.light);
    }
    let pot = Ramp::new(0x6a2f22, 0x96452f, 0xb75e40, 0xcf7a55, 0xe59a72);
    bevel(scene, x, top, 7, 6, pot);
    hline(scene, x - 1, top, 9, pot.light);
    hline(scene, x, PLATFORM_BACK, 7, rgba(0x2a2226, 80));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        let scene = backdrop(&[], &Arrangement::new());
        assert!(scene.pixels().iter().all(|pixel| pixel.a == 255));
    }

    #[test]
    fn the_backdrop_is_the_same_every_time() {
        assert_eq!(
            backdrop(&[], &Arrangement::new()),
            backdrop(&[], &Arrangement::new())
        );
    }

    #[test]
    fn kept_souvenirs_show_in_the_case_and_nothing_else_changes() {
        let empty = backdrop(&[], &Arrangement::new());
        let kept = backdrop(
            &["picnic_ribbon".to_owned(), "oak_acorn".to_owned()],
            &Arrangement::new(),
        );
        let (left, top, right, bottom) = CASE;
        for y in 0..SCENE_HEIGHT as i32 {
            for x in 0..SCENE_WIDTH as i32 {
                let inside = (left..right).contains(&x) && (top..bottom).contains(&y);
                if !inside {
                    assert_eq!(empty.get(x, y), kept.get(x, y), "({x}, {y}) changed");
                }
            }
        }
        assert_ne!(empty, kept);
    }

    #[test]
    fn smoke_drifts_unless_motion_is_reduced() {
        let draw = |elapsed, reduce| {
            let mut scene = backdrop(&[], &Arrangement::new());
            smoke(&mut scene, elapsed, reduce);
            scene
        };
        assert_ne!(draw(0.0, false), draw(1.5, false));
        assert_eq!(draw(0.0, true), draw(1.5, true));
    }
}
