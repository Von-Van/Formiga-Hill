//! The Clubhouse from inside, seen from just within the door and a little above, as the green is.
//! The back wall fills the top of the picture, papered above a panelled dado: the bookshelf, the
//! hearth, the window onto the Hill, the notice board where stories are pinned, and pegs for
//! scarves. Floorboards fill the rest, with the rag rug before the fire. The armchairs, the table
//! and the toy box stand on the floor as props, so anyone can be behind them.

use crate::daylight::{Daylight, glow, light, night_sky};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, blit, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba,
    vline,
};
use crate::playground::{Patch, Prop};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};

/// Where the back wall meets the floor.
const WALL_FOOT: i32 = 96;
/// Where anyone may stand: the floor, short of the wall and the bottom edge.
pub const GROUND: Patch = (22.0, 104.0, 362.0, 204.0);
/// The rug before the fire, where anyone who wants to sit down goes.
pub const SEATS: Patch = (108.0, 136.0, 204.0, 154.0);
/// The warm boards in front of the hearth, for napping.
pub const HEARTHSIDE: Patch = (118.0, 104.0, 166.0, 112.0);
/// The notice board: left, top, width, height.
pub const BOARD: (i32, i32, i32, i32) = (288, 22, 50, 38);
/// How many stories the board has room to show.
pub const BOARD_ROOM: usize = 6;
/// The rag rug: centre and radii.
const RUG: (i32, i32, i32, i32) = (156, 146, 72, 22);
/// The window: left, top, width, height, frame included.
const WINDOW: (i32, i32, i32, i32) = (214, 20, 56, 42);
/// How thick the window's frame is.
const SASH: i32 = 3;
/// The chimney breast's sides.
const BREAST: (i32, i32) = (100, 184);
/// The fire's opening: its middle, its half-width, the top of its arch and its floor.
const FIREBOX: (i32, i32, i32, i32) = (142, 19, 60, 94);
/// The bookshelf: left, top, width, height.
const SHELF: (i32, i32, i32, i32) = (22, 14, 50, 82);
/// Where the lamp hangs from the beam: far enough from the window that its shade stays clear of
/// the curtain rod's end.
const LAMP_X: i32 = 192;
/// How tall the bookshelf's plinth is, taller than the skirting so their lines never meet.
const PLINTH: i32 = 9;
/// How many pictures the fire has, flickering between them.
pub const FLICKERS: usize = 4;

const WALLPAPER: Ramp = Ramp::new(0x5d6a50, 0x7d8a66, 0x98a57c, 0xb0bb92, 0xcdd5ae);
const PANEL: Ramp = Ramp::new(0x3f2a1f, 0x61402d, 0x7c5539, 0x966b48, 0xb3865c);
const FLOOR: Ramp = Ramp::new(0x553a2a, 0x7a5638, 0x956b45, 0xae8455, 0xc9a06c);
const BRASS: Ramp = Ramp::new(0x6b4e1c, 0x9a7330, 0xc49a45, 0xe0bd62, 0xf6e09a);
const CORK: Ramp = Ramp::new(0x6e4a2c, 0x9a6c40, 0xb98a55, 0xcfa46c, 0xe0bd86);
const CURTAIN: Ramp = Ramp::new(0x6a3326, 0x934a33, 0xb5623f, 0xcf8352, 0xe4a872);
const ROSE: Ramp = Ramp::new(0x542233, 0x7a3346, 0x9a4859, 0xb76571, 0xd38b8f);
const MUSTARD: Ramp = Ramp::new(0x5e4418, 0x8a6526, 0xb08636, 0xcca24c, 0xe5c470);
const TOYBOX: Ramp = Ramp::new(0x223a4a, 0x31546a, 0x436f88, 0x5e8ea6, 0x86b0c4);
const BARK: Ramp = Ramp::new(0x2e2019, 0x4a3426, 0x634632, 0x7c5b41, 0x987456);
const BINDINGS: [Ramp; 6] = [
    Ramp::new(0x4a1f22, 0x6e2a2e, 0x8f3a3a, 0xad5148, 0xc87060),
    Ramp::new(0x1e2a44, 0x2c3d5e, 0x3e5478, 0x587096, 0x7a92b4),
    Ramp::new(0x1f3a2a, 0x2d5238, 0x3f6b48, 0x598a5e, 0x7aaa78),
    Ramp::new(0x5a4416, 0x846426, 0xa9823a, 0xc9a052, 0xe2c27a),
    Ramp::new(0x3e2236, 0x5a3250, 0x77476a, 0x936287, 0xb083a6),
    Ramp::new(0x6e6250, 0x9a8c74, 0xbcae92, 0xd6caae, 0xece2c8),
];
/// The rug's braided rings, from the edge in.
const RINGS: [Ramp; 11] = [
    Ramp::new(0x4b2a20, 0x6e3a2a, 0x93503a, 0xae674a, 0xc8835e),
    Ramp::new(0x5e4418, 0x8a6526, 0xb08636, 0xcca24c, 0xe5c470),
    Ramp::new(0x3c4a32, 0x566a46, 0x6f8a5a, 0x89a46f, 0xa9c08a),
    Ramp::new(0x7a6a50, 0xa8957a, 0xcab696, 0xe0d0b0, 0xf2e6cc),
    Ramp::new(0x2f3f52, 0x46607a, 0x5f7d96, 0x7c98ae, 0x9fb6c6),
    Ramp::new(0x4b2a20, 0x6e3a2a, 0x93503a, 0xae674a, 0xc8835e),
    Ramp::new(0x7a6a50, 0xa8957a, 0xcab696, 0xe0d0b0, 0xf2e6cc),
    Ramp::new(0x5e4418, 0x8a6526, 0xb08636, 0xcca24c, 0xe5c470),
    Ramp::new(0x3c4a32, 0x566a46, 0x6f8a5a, 0x89a46f, 0xa9c08a),
    Ramp::new(0x542233, 0x7a3346, 0x9a4859, 0xb76571, 0xd38b8f),
    Ramp::new(0x7a6a50, 0xa8957a, 0xcab696, 0xe0d0b0, 0xf2e6cc),
];
const SOOT: Rgba = rgb(0x231c1a);
const FLAME: [Rgba; 4] = [rgb(0xc8452c), rgb(0xec7a30), rgb(0xf9bd4c), rgb(0xfff2c4)];
const INK: Rgba = rgba(0x3a3550, 170);
const SHADOW: Rgba = rgba(0x2a1a14, 72);
const WARM: Rgba = rgba(0xffb35a, 28);

/// The Hill through the window, small, with whatever stands on it. In the window's own picture.
const VISTA: Vista = Vista {
    tree: (30.0, 19.0),
    crest: |x| 35.0 - 16.0 * (1.0 - ((x - 30.0) / 26.0).powi(2)).max(0.0).sqrt(),
    spread: 12.0,
    shrink: 10,
    tint: Tint::Haze(HILL_SHADE, 0.35),
};

/// The room with everything that never moves, the fire out: the hearth is lit in `with_fire`.
/// `pinned` has one card per story on the board, true where the colony has finished it. The
/// window shows the Hill at the hour `outside`, and the sun comes in only by day.
pub fn room(hilltop: &Arrangement, pinned: &[bool], outside: Daylight) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    wall(&mut scene);
    lamp(&mut scene);
    bookshelf(&mut scene);
    hearth(&mut scene);
    window(&mut scene, hilltop, outside);
    board(&mut scene, pinned);
    pegs(&mut scene);
    floor(&mut scene);
    hearthstone(&mut scene);
    light_through_the_window(&mut scene, 1.0 - outside.lamps());
    rug(&mut scene);
    log_basket(&mut scene, 82, WALL_FOOT + 2);
    // Where the furniture stands, soft shade on the boards.
    for (sprite, (x, y), _) in furniture() {
        let (left, right) = (x, x + sprite.width() as i32);
        let foot = y + sprite.height() as i32;
        ellipse(
            &mut scene,
            (left + right) / 2 + 2,
            foot - 1,
            (right - left) / 2,
            4,
            SHADOW,
        );
    }
    scene
}

/// The room with the fire at one of its flickers.
pub fn with_fire(room: &Canvas, flicker: usize) -> Canvas {
    let mut scene = room.clone();
    fire(&mut scene, flicker % FLICKERS);
    scene
}

/// The armchairs, the table and the toy box, each standing on the floor.
pub fn props() -> Vec<Prop> {
    furniture()
        .into_iter()
        .map(|(sprite, at, base)| Prop::new(sprite, at, base))
        .collect()
}

fn furniture() -> Vec<(Canvas, (i32, i32), f32)> {
    vec![
        (armchair(ROSE, false), (52, 108), 144.0),
        (armchair(MUSTARD, true), (226, 108), 144.0),
        (table(), (281, 141), 176.0),
        (toy_box(), (26, 168), 194.0),
    ]
}

/// A fern in a pot at each bottom corner, drawn over the travellers.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let height = SCENE_HEIGHT as i32;
    potted_fern(&mut scene, 10, height, 11);
    potted_fern(&mut scene, SCENE_WIDTH as i32 - 12, height, 23);
    scene
}

// ---------------------------------------------------------------------------------------------
// The back wall
// ---------------------------------------------------------------------------------------------

fn wall(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // Sage paper in broad and narrow stripes, a sprig in each broad one, darker up under the
    // beam and in the corners.
    for y in 8..62 {
        for x in 0..width {
            let stripe = x.rem_euclid(14);
            let mut color = if stripe < 9 {
                WALLPAPER.base
            } else {
                mix(WALLPAPER.base, WALLPAPER.light, 0.45)
            };
            if stripe == 9 || stripe == 13 {
                color = mix(WALLPAPER.base, WALLPAPER.shadow, 0.35);
            }
            if chance(x / 2, y / 2, 401, 14) {
                color = mix(color, WALLPAPER.shadow, 0.3);
            }
            let ceiling = (1.0 - (y - 8) as f32 / 14.0).max(0.0) * 0.45;
            let corner = (1.0 - x.min(width - 1 - x) as f32 / 24.0).max(0.0) * 0.35;
            scene.set(x, y, mix(color, WALLPAPER.edge, ceiling + corner));
        }
    }
    for row in 0..4 {
        for column in 0..width / 14 + 1 {
            let x = column * 14 + 4 + if row % 2 == 1 { 0 } else { 1 };
            let y = 16 + row * 12 + (column % 2) * 6;
            if y > 56 {
                continue;
            }
            sprig(scene, x, y);
        }
    }
    // The beam along the top, and its shadow on the paper.
    rect(scene, 0, 0, width, 8, TIMBER.base);
    hline(scene, 0, 0, width, TIMBER.shadow);
    hline(scene, 0, 1, width, TIMBER.light);
    hline(scene, 0, 7, width, TIMBER.edge);
    for x in 0..width {
        if chance(x / 3, 3, 402, 70) {
            put(scene, x, 3 + (noise(x, 0, 403) % 3) as i32, TIMBER.shadow);
        }
    }
    hline(scene, 0, 8, width, rgba(0x1e2418, 90));
    hline(scene, 0, 9, width, rgba(0x1e2418, 40));
    // The dado: a rail, upright boards, and the skirting.
    rect(scene, 0, 60, width, 3, PANEL.base);
    hline(scene, 0, 60, width, PANEL.shine);
    hline(scene, 0, 62, width, PANEL.edge);
    for y in 63..90 {
        for x in 0..width {
            let board = x.div_euclid(9);
            let across = x.rem_euclid(9);
            let tone = match noise(board, 0, 404) % 4 {
                0 => mix(PANEL.base, PANEL.light, 0.3),
                1 => mix(PANEL.base, PANEL.shadow, 0.3),
                _ => PANEL.base,
            };
            let color = match across {
                0 => PANEL.edge,
                1 => mix(tone, PANEL.light, 0.6),
                8 => mix(tone, PANEL.shadow, 0.6),
                _ if chance(x, y / 3, 405 + board as u32, 40) => mix(tone, PANEL.shadow, 0.4),
                _ => tone,
            };
            scene.set(x, y, color);
        }
    }
    hline(scene, 0, 63, width, rgba(0x1e140e, 80));
    rect(scene, 0, 90, width, 6, PANEL.shadow);
    hline(scene, 0, 90, width, PANEL.light);
    hline(scene, 0, 91, width, PANEL.base);
    hline(scene, 0, 95, width, PANEL.edge);
}

/// A little cream flower on two leaves.
fn sprig(scene: &mut Canvas, x: i32, y: i32) {
    let leaf = mix(WALLPAPER.shadow, LEAF.shadow, 0.4);
    put(scene, x - 1, y + 1, leaf);
    put(scene, x + 1, y + 1, leaf);
    put(scene, x, y + 2, leaf);
    let petal = rgb(0xe8e2c6);
    put(scene, x, y - 1, petal);
    put(scene, x - 1, y, petal);
    put(scene, x + 1, y, petal);
    put(scene, x, y, rgb(0xd9a85a));
}

/// The lamp hanging from the beam between the hearth and the window, and its glow on the paper.
fn lamp(scene: &mut Canvas) {
    let cx = LAMP_X;
    for (radius, alpha) in [(34, 14), (24, 18), (14, 24)] {
        ellipse(scene, cx, 22, radius, radius * 3 / 4, rgba(0xffe2a0, alpha));
    }
    vline(scene, cx, 8, 6, IRON.shadow);
    for link in (8..14).step_by(2) {
        put(scene, cx, link, IRON.light);
    }
    polygon(
        scene,
        &[(cx - 5, 14), (cx + 6, 14), (cx + 9, 20), (cx - 8, 20)],
        |x, _| {
            Some(if x < cx - 2 {
                BRASS.light
            } else if x > cx + 4 {
                BRASS.shadow
            } else {
                BRASS.base
            })
        },
    );
    hline(scene, cx - 5, 14, 11, BRASS.edge);
    hline(scene, cx - 8, 20, 17, BRASS.edge);
    put(scene, cx - 3, 15, BRASS.shine);
    hline(scene, cx - 4, 21, 9, GLOW[2]);
    hline(scene, cx - 2, 22, 5, GLOW[1]);
}

/// Tall shelves of books, a jar, and a little fern on top.
fn bookshelf(scene: &mut Canvas) {
    let (left, top, width, height) = SHELF;
    let right = left + width;
    let foot = top + height;
    // The case: dark inside, sides and crown in timber, standing on a plinth as wide as the
    // crown. The plinth is taller than the skirting behind it, so none of the skirting's lines
    // run on through the case's foot, and its top is the bottom shelf.
    let plinth = foot - PLINTH;
    rect(scene, left, top, width, height, rgb(0x2a1c16));
    bevel(scene, left - 2, top - 3, width + 4, 5, TIMBER);
    for side in [left - 1, right - 3] {
        rect(scene, side, top, 4, plinth - top, TIMBER.base);
        vline(scene, side, top, plinth - top, TIMBER.edge);
        vline(scene, side + 1, top, plinth - top, TIMBER.light);
        vline(scene, side + 3, top, plinth - top, TIMBER.shadow);
    }
    bevel(scene, left - 2, plinth, width + 4, PLINTH, TIMBER);
    hline(scene, left - 1, plinth + 1, width + 2, TIMBER.shine);
    hline(scene, left - 1, plinth + 2, width + 2, TIMBER.shadow);
    let shelves = [top + 18, top + 37, top + 56, plinth];
    let mut floor_of = top + 2;
    for (index, &board) in shelves.iter().enumerate() {
        let inside_top = floor_of;
        // A row of books standing on the board, with a gap or a leaner now and then.
        let mut x = left + 3;
        let mut book = 0;
        while x < right - 4 {
            let salt = (index * 31 + book) as u32;
            let wide = 2 + (noise(book as i32, index as i32, 410) % 3) as i32;
            if x + wide > right - 4 {
                break;
            }
            let tall = (board - inside_top - 1)
                .min(9 + (noise(book as i32, index as i32, 411) % 6) as i32);
            if noise(salt as i32, 7, 412).is_multiple_of(9) {
                x += 3;
                book += 1;
                continue;
            }
            let ramp = BINDINGS[(noise(salt as i32, 3, 413) % BINDINGS.len() as u32) as usize];
            spine(scene, x, board - tall, wide, tall, ramp);
            x += wide;
            book += 1;
        }
        if index == 1 {
            // A glass jar of buttons where a few books would be.
            let jx = right - 12;
            rect(scene, jx, board - 7, 6, 7, rgba(0xdcecef, 110));
            hline(scene, jx, board - 8, 6, BRASS.base);
            for (dx, dy, color) in [(1, 3, 0xd0574a), (3, 4, 0xf5d25e), (2, 5, 0x5f7d96)] {
                put(scene, jx + dx, board - 8 + dy, rgb(color));
            }
            vline(scene, jx, board - 7, 7, rgba(0xffffff, 120));
        }
        if board == plinth {
            break;
        }
        // The board itself, lit along its lip.
        rect(scene, left + 2, board, width - 4, 3, TIMBER.base);
        hline(scene, left + 2, board, width - 4, TIMBER.shine);
        hline(scene, left + 2, board + 2, width - 4, TIMBER.edge);
        hline(scene, left + 2, board + 3, width - 4, rgba(0x000000, 60));
        floor_of = board + 3;
    }
    // A fern on top, and the case's shadow on the paper to its right.
    vline(
        scene,
        right + 1,
        top - 2,
        plinth - top + 2,
        rgba(0x1e2418, 50),
    );
    vline(scene, right + 2, top, plinth - top, rgba(0x1e2418, 25));
    vline(scene, right + 3, plinth + 1, PLINTH - 1, rgba(0x1e2418, 50));
    potted_fern(scene, left + 12, top - 3, 5);
}

/// One book on a shelf, its spine towards the room.
fn spine(scene: &mut Canvas, x: i32, y: i32, wide: i32, tall: i32, ramp: Ramp) {
    rect(scene, x, y, wide, tall, ramp.base);
    vline(scene, x, y, tall, ramp.light);
    if wide > 2 {
        vline(scene, x + wide - 1, y, tall, ramp.shadow);
    }
    hline(scene, x, y, wide, ramp.shine);
    // A band and a title, in gilt.
    hline(
        scene,
        x,
        y + tall / 3,
        wide,
        mix(ramp.base, BRASS.light, 0.55),
    );
    if tall > 10 {
        hline(
            scene,
            x,
            y + tall - 3,
            wide,
            mix(ramp.base, BRASS.base, 0.4),
        );
    }
}

/// The chimney breast in dressed stone, the mantel and what stands on it, and the fire's
/// opening, dark until `fire` lights it.
fn hearth(scene: &mut Canvas) {
    let (left, right) = BREAST;
    crate::kit::stonework(
        scene,
        (left, 0, right - left, WALL_FOOT),
        crate::kit::Courses { tall: 6, long: 10 },
        420,
    );
    // The breast stands out from the wall: light down its left, shade down its right.
    vline(scene, left, 0, WALL_FOOT, STONE.edge);
    vline(scene, left + 1, 0, WALL_FOOT, rgba(0xffffff, 50));
    vline(scene, right - 1, 0, WALL_FOOT, STONE.edge);
    for x in right..right + 4 {
        vline(
            scene,
            x,
            8,
            WALL_FOOT - 8,
            rgba(0x1e2418, (70 - (x - right) * 16) as u8),
        );
    }
    // A painting above the mantel: the Hill, and the train that brings everyone.
    painting(scene, 122, 13);
    // The mantel shelf on two corbels.
    bevel(scene, left - 4, 44, right - left + 8, 5, TIMBER);
    hline(scene, left - 4, 49, right - left + 8, rgba(0x1e140e, 90));
    for corbel in [left + 2, right - 8] {
        rect(scene, corbel, 49, 6, 5, TIMBER.shadow);
        hline(scene, corbel, 49, 6, TIMBER.base);
        vline(scene, corbel, 49, 5, TIMBER.light);
        put(scene, corbel + 5, 53, TIMBER.edge);
    }
    clock(scene, 143, 44);
    candlestick(scene, left + 6, 44);
    candlestick(scene, right - 7, 44);
    // A jug of daisies.
    rect(scene, 160, 37, 6, 7, CREAM);
    vline(scene, 165, 37, 7, mix(CREAM, STONE.shadow, 0.4));
    hline(scene, 160, 36, 6, mix(CREAM, STONE.shadow, 0.2));
    for (dx, dy) in [(0, -3), (2, -5), (4, -4), (6, -2), (3, -2)] {
        put(scene, 160 + dx, 36 + dy + 1, LEAF.base);
        put(scene, 160 + dx, 36 + dy, BLOSSOMS[3]);
        put(scene, 160 + dx, 36 + dy - 1, rgb(0xf5d25e));
    }

    // The opening: a dressed arch of lighter stone, soot inside, and logs on the dogs.
    let (cx, half, top, foot) = FIREBOX;
    let spring = top + half;
    let inside = |x: i32, y: i32, grow: i32| -> bool {
        let h = half + grow;
        if y < top - grow || y > foot {
            return false;
        }
        if y >= spring {
            return (x - cx).abs() <= h;
        }
        let (dx, dy) = (x - cx, y - spring);
        dx * dx + dy * dy <= h * h
    };
    for y in top - 4..=foot {
        for x in cx - half - 4..=cx + half + 4 {
            if inside(x, y, 3) && !inside(x, y, 0) {
                let light = (x - cx) + (y - spring) < 0;
                put(scene, x, y, if light { STONE.light } else { STONE.base });
                if chance(x, y, 421, 30) {
                    put(scene, x, y, STONE.shadow);
                }
            } else if inside(x, y, 0) {
                let depth = (y - (top - half)) as f32 / (foot - top + half) as f32;
                let back = mix(SOOT, rgb(0x4a2a20), depth.powf(1.5) * 0.8);
                let brick = (y.rem_euclid(4) == 0)
                    || ((x + if (y / 4) % 2 == 0 { 0 } else { 3 }).rem_euclid(7) == 0);
                put(scene, x, y, if brick { mix(back, SOOT, 0.5) } else { back });
            }
        }
    }
    for y in top - 4..=foot {
        for x in cx - half - 4..=cx + half + 4 {
            if inside(x, y, 4) && !inside(x, y, 3) {
                put(scene, x, y, STONE.edge);
            }
        }
    }
    // A keystone.
    rect(scene, cx - 3, top - 5, 7, 5, STONE.light);
    hline(scene, cx - 3, top - 5, 7, STONE.shine);
    vline(scene, cx + 3, top - 5, 5, STONE.shadow);
    // Firedogs.
    for dog in [cx - 13, cx + 12] {
        vline(scene, dog, foot - 9, 9, IRON.edge);
        vline(scene, dog + 1, foot - 9, 9, IRON.shadow);
        rect(scene, dog - 1, foot - 11, 3, 2, IRON.base);
        put(scene, dog - 1, foot - 11, IRON.light);
    }
    // Two logs, one across the other.
    log(scene, cx - 15, foot - 6, 30, 5);
    log(scene, cx - 9, foot - 10, 20, 4);
}

fn log(scene: &mut Canvas, x: i32, y: i32, long: i32, thick: i32) {
    rect(scene, x, y, long, thick, BARK.base);
    hline(scene, x, y, long, BARK.light);
    hline(scene, x, y + thick - 1, long, BARK.edge);
    for px in x..x + long {
        if chance(px / 2, y, 430, 70) {
            put(
                scene,
                px,
                y + 1 + (noise(px, y, 431) % (thick as u32 - 1).max(1)) as i32,
                BARK.shadow,
            );
        }
    }
    // The cut end, rings and all.
    ellipse(scene, x, y + thick / 2, 2, thick / 2, PLANK.light);
    put(scene, x, y + thick / 2, PLANK.shadow);
}

/// A gilt-framed picture of the Hill with the train coming round it.
fn painting(scene: &mut Canvas, x: i32, y: i32) {
    let (wide, tall) = (42, 24);
    bevel(scene, x, y, wide, tall, BRASS);
    rect(scene, x + 1, y + 1, wide - 2, tall - 2, BRASS.base);
    // The picture is painted on its own canvas the size of the opening, so its hills end at
    // the frame instead of spilling over it.
    let (ix, iy, iw, ih) = (x + 3, y + 3, wide - 6, tall - 6);
    let mut picture = Canvas::new(iw as u32, ih as u32);
    let canvas = &mut picture;
    for py in 0..ih {
        let band = mix(SKY_TOP, SKY_LOW, py as f32 / ih as f32);
        hline(canvas, 0, py, iw, band);
    }
    ellipse(canvas, 22, ih + 4, 18, 12, HILL);
    ellipse(canvas, 28, ih + 5, 12, 9, HILL_SHADE);
    ellipse(canvas, 6, ih + 3, 12, 6, FAR_HILL);
    rect(canvas, 21, 2, 1, 3, TRUNK);
    ellipse(canvas, 21, 2, 2, 2, LEAVES);
    // The train along the bottom, with a puff of steam.
    hline(canvas, 0, ih - 2, iw, rgb(0x6b625d));
    rect(canvas, 6, ih - 6, 7, 4, ENAMEL);
    rect(canvas, 11, ih - 8, 2, 2, ENAMEL_DEEP);
    rect(canvas, 14, ih - 5, 6, 3, rgb(0xb05445));
    put(canvas, 7, ih - 2, rgb(0x2b2b2b));
    put(canvas, 11, ih - 2, rgb(0x2b2b2b));
    ellipse(canvas, 11, ih - 11, 2, 1, CLOUD);
    ellipse(canvas, 8, ih - 13, 2, 1, rgba(0xfdfbf5, 180));
    blit(scene, &picture, ix, iy);
    // The frame's shadow on the stone.
    hline(scene, x + 1, y + tall, wide, rgba(0x1e140e, 70));
    vline(scene, x + wide, y + 1, tall, rgba(0x1e140e, 70));
}

/// A brass carriage clock with a cream face.
fn clock(scene: &mut Canvas, cx: i32, shelf: i32) {
    bevel(scene, cx - 5, shelf - 10, 11, 10, BRASS);
    rect(scene, cx - 6, shelf - 11, 13, 2, BRASS.shadow);
    hline(scene, cx - 6, shelf - 11, 13, BRASS.light);
    ellipse(scene, cx, shelf - 5, 3, 3, CREAM);
    put(scene, cx, shelf - 6, INK);
    put(scene, cx, shelf - 7, INK);
    put(scene, cx + 1, shelf - 5, INK);
    put(scene, cx - 2, shelf - 7, rgba(0xffffff, 160));
}

fn candlestick(scene: &mut Canvas, x: i32, shelf: i32) {
    hline(scene, x - 2, shelf - 1, 5, BRASS.shadow);
    hline(scene, x - 1, shelf - 2, 3, BRASS.base);
    vline(scene, x, shelf - 6, 4, BRASS.light);
    hline(scene, x - 1, shelf - 6, 3, BRASS.base);
    rect(scene, x - 1, shelf - 12, 2, 6, CREAM);
    vline(scene, x, shelf - 12, 6, mix(CREAM, STONE.shadow, 0.3));
    put(scene, x, shelf - 13, rgb(0x3a2a20));
    put(scene, x, shelf - 14, GLOW[1]);
    put(scene, x, shelf - 15, rgba(0xfff0bd, 170));
    ellipse(scene, x, shelf - 14, 3, 3, rgba(0xffd77a, 30));
}

/// The fire's flames and embers at one flicker, and its glow on the stone.
fn fire(scene: &mut Canvas, flicker: usize) {
    let (cx, half, top, foot) = FIREBOX;
    let salt = flicker as u32 * 17 + 440;
    let lift = [0, 2, 1, 3][flicker];
    ellipse(
        scene,
        cx,
        foot - 12,
        half + 10,
        18,
        rgba(0xff9a40, 26 + lift as u8 * 4),
    );
    ellipse(scene, cx, foot - 10, half - 2, 12, rgba(0xffb050, 36));
    // Tongues of flame, tallest in the middle, swaying a pixel or two from one flicker to the
    // next: red outside, orange, then yellow, and a pale heart.
    for tongue in 0..6 {
        let x = cx - 12 + tongue * 5 + (noise(tongue, 0, salt) % 3) as i32 - 1;
        let middle = 3 - (tongue - 2).abs().min(3);
        let tall = 7 + middle * 3 + (noise(tongue, 1, salt) % 5) as i32;
        let lean = (noise(tongue, 2, salt) % 3) as i32 - 1;
        let base = foot - 9;
        for (layer, color) in FLAME.iter().enumerate() {
            let shrink = layer as i32 * 2;
            let height = (tall - shrink * 2).max(0);
            for dy in 0..height {
                let t = dy as f32 / height.max(1) as f32;
                let w = ((1.0 - t) * (3.2 - layer as f32 * 0.7)).round() as i32;
                let sway = (t * t * lean as f32 * 2.0).round() as i32;
                for dx in -w..=w {
                    put(scene, x + dx + sway, base - dy, *color);
                }
            }
        }
    }
    // A spark or two above.
    for spark in 0..2 {
        let x = cx - 6 + (noise(spark, 3, salt) % 13) as i32;
        let y = top + 6 + (noise(spark, 4, salt) % 10) as i32;
        put(scene, x, y, FLAME[2]);
    }
    // Embers glowing under the logs.
    for x in cx - 15..cx + 15 {
        if chance(x, flicker as i32, 450, 120) {
            put(
                scene,
                x,
                foot - 1,
                if chance(x, 1, salt, 128) {
                    FLAME[1]
                } else {
                    FLAME[0]
                },
            );
        }
    }
}

/// The window, and the Hill through it.
fn window(scene: &mut Canvas, hilltop: &Arrangement, outside: Daylight) {
    let (x, y, wide, tall) = WINDOW;
    let (vw, vh) = (wide - SASH * 2, tall - SASH * 2);
    let mut sky = Canvas::new(vw as u32, vh as u32);
    for py in 0..vh {
        let band = mix(SKY_TOP, SKY_LOW, (py as f32 / vh as f32).powf(1.3));
        hline(&mut sky, 0, py, vw, band);
    }
    ellipse(&mut sky, 10, 7, 5, 2, CLOUD);
    ellipse(&mut sky, 14, 6, 4, 2, CLOUD);
    ellipse(&mut sky, 41, 10, 4, 1, rgba(0xfdfbf5, 200));
    let mut view = sky.clone();
    for px in 0..vw {
        let crest = 26 + ((px as f32 / 7.0).sin() * 1.5) as i32;
        vline(&mut view, px, crest, vh - crest, FAR_HILL);
    }
    view.fill_ellipse(30, 36, 26, 16, HILL);
    view.fill_ellipse(38, 38, 18, 12, HILL_SHADE);
    view.line(30, 33, 26, 26, 1, PATH);
    view.line(26, 26, 30, 20, 1, PATH);
    view.fill_rect(29, 13, 2, 6, TRUNK);
    view.fill_circle(30, 11, 4, LEAVES);
    skyline(&mut view, hilltop, &VISTA);
    // The hedge outside the window.
    for px in 0..vw {
        let top = vh - 4 - (noise(px / 2, 0, 460) % 3) as i32;
        vline(&mut view, px, top, vh - top, LEAF.shadow);
        if chance(px, top, 461, 120) {
            put(&mut view, px, top, LEAF.base);
        }
    }
    // The hour outside, before the glass.
    let painted = view.clone();
    let night = night_sky(&painted, &sky, vh, Some((40, 7)));
    light(
        &mut view,
        outside.ambient(),
        &painted,
        &[(&night, outside.stars())],
    );
    // The glass: a little cooler at the bottom, with a glint across each pane.
    for py in 0..vh {
        hline(&mut view, 0, py, vw, rgba(0x7499a8, (py * 40 / vh) as u8));
    }
    for (pane_x, pane_y) in [
        (2, 2),
        (vw / 2 + 2, 2),
        (2, vh / 2 + 2),
        (vw / 2 + 2, vh / 2 + 2),
    ] {
        for step in 0..4 {
            put(
                &mut view,
                pane_x + step,
                pane_y + 3 - step,
                rgba(0xe9f6f7, 170),
            );
        }
    }
    // A deep reveal, shaded along its top and left as the light comes from there.
    rect(
        scene,
        x - 3,
        y - 3,
        wide + 6,
        tall + 6,
        mix(WALLPAPER.base, WALLPAPER.shadow, 0.5),
    );
    hline(scene, x - 3, y - 3, wide + 6, WALLPAPER.edge);
    blit(scene, &view, x + SASH, y + SASH);
    // The frame and glazing bars.
    for (fx, fy, fw, fh) in [
        (x, y, wide, SASH),
        (x, y + tall - SASH, wide, SASH),
        (x, y, SASH, tall),
        (x + wide - SASH, y, SASH, tall),
        (x + wide / 2 - 1, y, 2, tall),
        (x, y + tall / 2 - 1, wide, 2),
    ] {
        rect(scene, fx, fy, fw, fh, TRIM.base);
    }
    hline(scene, x, y, wide, TRIM.shine);
    vline(scene, x, y, tall, TRIM.light);
    hline(scene, x, y + tall - 1, wide, TRIM.shadow);
    vline(scene, x + wide - 1, y, tall, TRIM.shadow);
    hline(
        scene,
        x + SASH,
        y + tall / 2 + 1,
        wide - SASH * 2,
        TRIM.shadow,
    );
    vline(scene, x + wide / 2, y + SASH, tall - SASH * 2, TRIM.shadow);
    rect(scene, x - 1, y - 1, wide + 2, 1, TRIM.edge);
    // The sill, deep enough for a pot of geraniums.
    bevel(scene, x - 4, y + tall, wide + 8, 4, TRIM);
    hline(scene, x - 4, y + tall + 4, wide + 8, rgba(0x1e2418, 80));
    geranium(scene, x + wide - 12, y + tall);
    // The curtains, hung from a brass rod and tied back.
    hline(scene, x - 12, y - 6, wide + 24, BRASS.base);
    hline(scene, x - 12, y - 5, wide + 24, BRASS.shadow);
    for end in [x - 13, x + wide + 12] {
        ellipse(scene, end, y - 6, 1, 1, BRASS.light);
    }
    curtain(scene, x - 11, y - 4, false);
    curtain(scene, x + wide + 1, y - 4, true);
}

/// One curtain, gathered at a tie-back two-thirds of the way down.
fn curtain(scene: &mut Canvas, left: i32, top: i32, right_hand: bool) {
    let full = 11;
    let tie = top + 30;
    let foot = WINDOW.1 + WINDOW.3 + 10;
    for y in top..foot {
        let wide = if y < tie {
            full - (y - top) * 4 / (tie - top)
        } else {
            7 + (y - tie) * 3 / (foot - tie)
        };
        for i in 0..wide {
            let x = if right_hand {
                left + i
            } else {
                left + full - wide + i
            };
            let fold = (i + (y - top) / 9).rem_euclid(4);
            let color = match fold {
                0 => CURTAIN.light,
                1 => CURTAIN.base,
                2 => CURTAIN.shadow,
                _ => CURTAIN.base,
            };
            put(scene, x, y, color);
        }
        let edge_x = if right_hand {
            left + wide - 1
        } else {
            left + full - wide
        };
        put(scene, edge_x, y, CURTAIN.edge);
    }
    hline(
        scene,
        left + if right_hand { 0 } else { 3 },
        tie,
        8,
        BRASS.base,
    );
    hline(
        scene,
        left + if right_hand { 0 } else { 3 },
        foot - 1,
        if right_hand { 10 } else { 11 },
        CURTAIN.edge,
    );
}

fn geranium(scene: &mut Canvas, x: i32, sill: i32) {
    polygon(
        scene,
        &[
            (x, sill - 6),
            (x + 8, sill - 6),
            (x + 7, sill),
            (x + 1, sill),
        ],
        |px, _| Some(if px < x + 3 { TILE.light } else { TILE.base }),
    );
    hline(scene, x, sill - 6, 8, TILE.shine);
    hline(scene, x + 1, sill - 1, 6, TILE.shadow);
    for (dx, dy) in [
        (-1, -8),
        (2, -9),
        (5, -8),
        (8, -7),
        (3, -11),
        (6, -10),
        (0, -10),
    ] {
        ellipse(scene, x + dx + 1, sill + dy, 1, 1, LEAF.base);
        put(scene, x + dx, sill + dy - 1, LEAF.light);
    }
    for (dx, dy) in [(1, -12), (5, -12), (8, -10), (3, -10)] {
        put(scene, x + dx, sill + dy, rgb(0xe0605a));
        put(scene, x + dx + 1, sill + dy, rgb(0xd0574a));
        put(scene, x + dx, sill + dy - 1, rgb(0xf19bb0));
    }
}

/// The cork board where stories are pinned, one card each, a star on those finished.
fn board(scene: &mut Canvas, pinned: &[bool]) {
    let (x, y, wide, tall) = BOARD;
    bevel(scene, x, y, wide, tall, TIMBER);
    for py in y + 3..y + tall - 3 {
        for px in x + 3..x + wide - 3 {
            let speck = noise(px, py, 470) % 9;
            let color = match speck {
                0 => CORK.shadow,
                1 => CORK.light,
                2 if chance(px, py, 471, 90) => CORK.edge,
                _ => CORK.base,
            };
            scene.set(px, py, color);
        }
    }
    hline(scene, x + 3, y + 3, wide - 6, CORK.shadow);
    vline(scene, x + 3, y + 3, tall - 6, CORK.shadow);
    // A child's drawing of the Hill, pinned in the corner, whatever else is up.
    let (dx, dy) = (x + wide - 15, y + tall - 14);
    rect(scene, dx, dy, 11, 9, rgb(0xfbf6ee));
    polygon(
        scene,
        &[(dx + 1, dy + 8), (dx + 5, dy + 3), (dx + 10, dy + 8)],
        |_, _| Some(rgb(0x6fa866)),
    );
    put(scene, dx + 5, dy + 2, rgb(0x7a5a3a));
    put(scene, dx + 5, dy + 1, rgb(0x518a55));
    put(scene, dx + 8, dy + 2, rgb(0xf5d25e));
    hline(scene, dx, dy + 9, 11, rgba(0x3e2a1c, 80));
    for (index, finished) in pinned.iter().take(BOARD_ROOM).enumerate() {
        let column = (index % 3) as i32;
        let row = (index / 3) as i32;
        let cx = x + 6 + column * 14 + (noise(index as i32, 0, 472) % 2) as i32;
        let cy = y + 6 + row * 15 + (noise(index as i32, 1, 473) % 2) as i32;
        card(scene, cx, cy, index, *finished);
    }
}

/// A story's card: a few lines of writing, a pin, and a gold star once it is finished.
fn card(scene: &mut Canvas, x: i32, y: i32, index: usize, finished: bool) {
    let paper = [rgb(0xfbf6ee), rgb(0xf6e7c4), rgb(0xe4eef0), rgb(0xf3dfe2)][index % 4];
    rect(scene, x + 1, y + 1, 11, 12, rgba(0x3e2a1c, 80));
    rect(scene, x, y, 11, 12, paper);
    hline(scene, x, y, 11, mix(paper, rgb(0xffffff), 0.6));
    vline(scene, x + 10, y, 12, mix(paper, rgb(0x9a8c74), 0.3));
    hline(scene, x + 2, y + 3, 6, rgba(0x3a3550, 190));
    for line_y in [y + 6, y + 8, y + 10] {
        let long = 3 + (noise(index as i32, line_y, 474) % 5) as i32;
        hline(scene, x + 2, line_y, long, INK);
    }
    let pin = [rgb(0xd0574a), rgb(0x5f7d96), rgb(0xf5d25e), rgb(0x6fa866)][index % 4];
    put(scene, x + 5, y, pin);
    put(scene, x + 4, y, mix(pin, rgb(0xffffff), 0.5));
    put(scene, x + 5, y + 1, mix(pin, rgb(0x000000), 0.3));
    if finished {
        let (sx, sy) = (x + 8, y + 9);
        for (dx, dy) in [(0, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (1, 1)] {
            put(scene, sx + dx, sy + dy, BRASS.light);
        }
        put(scene, sx, sy, BRASS.shine);
    }
}

/// A rail of pegs by the board: a long striped scarf, a straw hat, a satchel.
fn pegs(scene: &mut Canvas) {
    let (left, rail) = (346, 30);
    bevel(scene, left, rail, 32, 4, TIMBER);
    for peg in [left + 5, left + 15, left + 25] {
        rect(scene, peg, rail + 3, 2, 3, TIMBER.light);
        put(scene, peg + 1, rail + 5, TIMBER.edge);
    }
    // The scarf, red and cream, hanging in two tails.
    for (tail, long) in [(left + 3, 30), (left + 7, 26)] {
        for y in rail + 6..rail + 6 + long {
            let red = ((y - rail) / 3) % 2 == 0;
            let color = if red { rgb(0xc04a3c) } else { rgb(0xf0e2c4) };
            hline(scene, tail, y, 4, color);
            put(scene, tail + 3, y, mix(color, rgb(0x3a1a14), 0.3));
        }
        for fringe in 0..4 {
            vline(scene, tail + fringe, rail + 6 + long, 2, rgb(0xc04a3c));
        }
    }
    // The hat.
    ellipse(scene, left + 16, rail + 10, 7, 2, rgb(0xc9a14e));
    ellipse(scene, left + 16, rail + 7, 4, 3, rgb(0xddb862));
    hline(scene, left + 12, rail + 8, 9, rgb(0x5f7d96));
    put(scene, left + 13, rail + 5, rgb(0xf0d58a));
    // The satchel, strap over the last peg.
    line(
        scene,
        (left + 22, rail + 6),
        (left + 26, rail + 18),
        LEATHER.shadow,
    );
    line(
        scene,
        (left + 28, rail + 6),
        (left + 30, rail + 18),
        LEATHER.shadow,
    );
    bevel(scene, left + 21, rail + 18, 11, 9, LEATHER);
    hline(scene, left + 22, rail + 21, 9, LEATHER.shadow);
    put(scene, left + 26, rail + 22, BRASS.light);
    // Their shadows on the paper.
    vline(scene, left + 33, rail + 4, 26, rgba(0x1e2418, 30));
}

// ---------------------------------------------------------------------------------------------
// The floor
// ---------------------------------------------------------------------------------------------

/// Boards running across the room, each row a little deeper as it comes nearer.
fn floor(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    let height = SCENE_HEIGHT as i32;
    let mut top = WALL_FOOT;
    let mut row = 0;
    while top < height {
        let deep = 5 + row / 3;
        let offset = (noise(row, 0, 480) % 70) as i32;
        let long = 56 + (noise(row, 1, 481) % 30) as i32;
        for y in top..(top + deep).min(height) {
            let within = y - top;
            for x in 0..width {
                let shifted = x + offset;
                let board = shifted.div_euclid(long);
                let along = shifted.rem_euclid(long);
                let tone = match noise(board, row, 482) % 6 {
                    0 => mix(FLOOR.base, FLOOR.light, 0.4),
                    1 => mix(FLOOR.base, FLOOR.shadow, 0.35),
                    2 => mix(FLOOR.base, FLOOR.light, 0.15),
                    _ => FLOOR.base,
                };
                let color = if within == 0 || along == 0 {
                    FLOOR.edge
                } else if within == 1 || along == 1 {
                    mix(tone, FLOOR.light, 0.5)
                } else if (along == 4 || along == long - 4) && within == deep / 2 {
                    FLOOR.shadow
                } else if chance(x / 5, y, 483 + board as u32, 45) {
                    mix(tone, FLOOR.shadow, 0.45)
                } else if chance(x / 3, y, 484, 18) {
                    mix(tone, FLOOR.light, 0.35)
                } else {
                    tone
                };
                scene.set(x, y, color);
            }
        }
        top += deep;
        row += 1;
    }
    // Shade where the boards meet the wall, and towards the corners nearest the viewer.
    for (step, alpha) in [(0, 90), (1, 70), (2, 50), (3, 34), (4, 20), (5, 10)] {
        hline(scene, 0, WALL_FOOT + step, width, rgba(0x1e140e, alpha));
    }
    for y in WALL_FOOT..height {
        for x in 0..width {
            let from_side = x.min(width - 1 - x) as f32 / 40.0;
            let near = (y - WALL_FOOT) as f32 / (height - WALL_FOOT) as f32;
            let dark = ((1.0 - from_side).max(0.0) * near * 70.0) as u8;
            if dark > 0 {
                put(scene, x, y, rgba(0x1e140e, dark));
            }
        }
    }
    // Firelight on the boards in front of the hearth.
    for (rx, ry, alpha) in [(84, 30, 18), (62, 22, 22), (40, 14, 26)] {
        ellipse(
            scene,
            FIREBOX.0,
            WALL_FOOT + 6,
            rx,
            ry,
            rgba(0xffa850, alpha),
        );
    }
}

/// The stone slab the fire stands on, reaching out onto the floor.
fn hearthstone(scene: &mut Canvas) {
    let (left, right) = (BREAST.0 + 4, BREAST.1 - 4);
    polygon(
        scene,
        &[
            (left, WALL_FOOT),
            (right, WALL_FOOT),
            (right + 4, WALL_FOOT + 8),
            (left - 4, WALL_FOOT + 8),
        ],
        |x, y| {
            Some(if y == WALL_FOOT {
                STONE.edge
            } else if chance(x, y, 490, 30) {
                STONE.shadow
            } else if (x - left).rem_euclid(22) == 0 {
                STONE.edge
            } else {
                mix(STONE.base, rgb(0xffb35a), 0.15)
            })
        },
    );
    hline(scene, left - 4, WALL_FOOT + 8, right - left + 8, STONE.edge);
    hline(
        scene,
        left - 3,
        WALL_FOOT + 9,
        right - left + 6,
        rgba(0x1e140e, 90),
    );
    hline(
        scene,
        left - 3,
        WALL_FOOT + 1,
        right - left + 6,
        mix(STONE.light, rgb(0xffb35a), 0.2),
    );
}

/// The patch of daylight from the window, falling down and to the right across the boards.
fn light_through_the_window(scene: &mut Canvas, strength: f32) {
    if strength <= 0.0 {
        return;
    }
    let (x, _, wide, _) = WINDOW;
    polygon(
        scene,
        &[
            (x + 2, WALL_FOOT + 1),
            (x + wide - 2, WALL_FOOT + 1),
            (x + wide + 22, WALL_FOOT + 40),
            (x + 18, WALL_FOOT + 40),
        ],
        |px, py| {
            // Softer the further it falls, and crossed by the glazing bars' shadow.
            let bar = px - (py - WALL_FOOT) * 24 / 39 == x + wide / 2;
            let fade = 1.0 - (py - WALL_FOOT) as f32 / 40.0;
            let alpha = if bar { 0.0 } else { 32.0 * fade * strength };
            Some(rgba(0xfff3d0, alpha as u8))
        },
    );
}

/// The braided rag rug, ring on ring, lit from the upper left.
fn rug(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = RUG;
    ellipse(scene, cx + 2, cy + 2, rx, ry, rgba(0x1e140e, 70));
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (dx, dy) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            let reach = (dx * dx + dy * dy).sqrt();
            if reach > 1.0 {
                continue;
            }
            let depth = (1.0 - reach) * RINGS.len() as f32;
            let ring = (depth as usize).min(RINGS.len() - 1);
            let ramp = RINGS[ring];
            // The braid: short diagonal stitches running round each ring.
            let stitch = (x / 2 + y + ring as i32).rem_euclid(3);
            let mut color = match stitch {
                0 => ramp.light,
                1 => ramp.base,
                _ => ramp.shadow,
            };
            if depth.fract() < 0.12 {
                color = ramp.edge;
            }
            let lit = dx + dy;
            if lit < -0.7 {
                color = mix(color, ramp.shine, 0.35);
            } else if lit > 0.8 {
                color = mix(color, ramp.edge, 0.3);
            }
            if reach > 0.97 {
                color = RINGS[0].edge;
            }
            put(scene, x, y, color);
        }
    }
    // The fire warms the rug's far side.
    ellipse(scene, cx - 14, cy - 12, 40, 10, WARM);
}

/// A wicker basket of split logs by the hearth.
fn log_basket(scene: &mut Canvas, x: i32, foot: i32) {
    // Ten rows of wicker, so the rim stands above the skirting's top edge instead of running on
    // along it.
    let rim = foot - 10;
    for (dx, dy) in [(1, -14), (4, -16), (7, -15), (10, -14)] {
        rect(scene, x + dx, foot + dy, 3, 6, BARK.base);
        put(scene, x + dx, foot + dy, PLANK.light);
        put(scene, x + dx + 2, foot + dy + 5, BARK.edge);
    }
    for y in rim..foot {
        for px in x..x + 15 {
            let weave = (px + y).rem_euclid(3) == 0;
            put(
                scene,
                px,
                y,
                if weave { rgb(0x9a7434) } else { rgb(0xbf9650) },
            );
        }
        put(scene, x, y, rgb(0x6b4a24));
        put(scene, x + 14, y, rgb(0x6b4a24));
    }
    hline(scene, x, rim, 15, rgb(0xd8b26a));
    hline(scene, x, foot - 1, 15, rgb(0x6b4a24));
}

// ---------------------------------------------------------------------------------------------
// Furniture, standing among everyone
// ---------------------------------------------------------------------------------------------

/// A buttoned wing armchair seen from the front, one cushion, a throw over one arm if `throw`.
fn armchair(cloth: Ramp, throw: bool) -> Canvas {
    let (wide, tall) = (40, 36);
    let mut chair = Canvas::new(wide as u32, tall as u32);
    // The back, rounded at the shoulders.
    for y in 0..22 {
        for x in 5..wide - 5 {
            let (dx, dy) = ((x - 5).min(wide - 6 - x), y);
            if dx + dy < 3 {
                continue;
            }
            let color = if x < 9 {
                cloth.light
            } else if x > wide - 10 {
                cloth.shadow
            } else {
                cloth.base
            };
            put(&mut chair, x, y, color);
        }
    }
    // Buttons, each with a dimple of shade.
    for row in 0..3 {
        for column in 0..4 {
            let bx = 11 + column * 6 + if row % 2 == 1 { 3 } else { 0 };
            let by = 5 + row * 5;
            if bx > wide - 11 {
                continue;
            }
            put(&mut chair, bx, by, cloth.edge);
            put(&mut chair, bx - 1, by - 1, cloth.light);
            put(&mut chair, bx + 1, by + 1, cloth.shadow);
        }
    }
    // The seat cushion, and the skirt below it, between the arms.
    rect(&mut chair, 9, 20, wide - 18, 7, cloth.base);
    hline(&mut chair, 9, 20, wide - 18, cloth.shine);
    hline(&mut chair, 9, 21, wide - 18, cloth.light);
    hline(&mut chair, 9, 26, wide - 18, cloth.shadow);
    rect(&mut chair, 9, 27, wide - 18, 5, cloth.shadow);
    hline(&mut chair, 9, 27, wide - 18, cloth.base);
    // Arms rolled over at the top and running down to the floor in one piece, over the seat's
    // ends, so nothing crosses them: the lit arm faces the light, the far one is in shade. Only
    // the side against the seat is drawn in, the outline does the rest.
    for (left, lit) in [(0, true), (wide - 9, false)] {
        let (roll, face, crease) = if lit {
            (cloth.light, cloth.light, cloth.base)
        } else {
            (cloth.base, cloth.shadow, cloth.edge)
        };
        let inner = if lit { left + 8 } else { left };
        for y in 12..32 {
            for x in left..left + 9 {
                // The roll's top corners are left off, so the outline rounds them.
                if y == 12 && (x == left || x == left + 8) {
                    continue;
                }
                let color = if x == inner {
                    cloth.edge
                } else if y < 15 {
                    roll
                } else if y == 15 {
                    crease
                } else if lit && x < left + 3 {
                    cloth.shine
                } else {
                    face
                };
                put(&mut chair, x, y, color);
            }
        }
        put(
            &mut chair,
            left + 2,
            12,
            if lit { cloth.shine } else { cloth.light },
        );
    }
    // A cushion of the other colour, plumped against the back.
    let pillow = if throw { ROSE } else { MUSTARD };
    ellipse(&mut chair, 20, 16, 6, 4, pillow.base);
    ellipse(&mut chair, 19, 15, 4, 2, pillow.light);
    put(&mut chair, 17, 14, pillow.shine);
    hline(&mut chair, 15, 20, 11, pillow.shadow);
    if throw {
        // A knitted throw over the left arm.
        for y in 10..30 {
            for x in 0..8 {
                let row = (y - 10) / 2;
                let color = if row % 3 == 0 {
                    CREAM
                } else {
                    mix(CREAM, rgb(0x5f7d96), 0.25)
                };
                if (x + y) % 2 == 0 || y < 12 {
                    put(&mut chair, x, y, color);
                } else {
                    put(&mut chair, x, y, mix(color, rgb(0x6e6250), 0.2));
                }
            }
        }
        for tassel in [1, 4, 7] {
            vline(&mut chair, tassel, 30, 2, CREAM);
        }
    }
    // Turned legs.
    for leg in [4, wide - 6] {
        rect(&mut chair, leg, 32, 2, 4, TIMBER.base);
        put(&mut chair, leg, 32, TIMBER.light);
        put(&mut chair, leg + 1, 35, TIMBER.edge);
    }
    // An outline in the cloth's own darkest tone.
    outline(&mut chair, cloth.edge);
    chair
}

/// The round table under a gingham cloth, with the teapot, a cup and a plate of buns.
fn table() -> Canvas {
    let (wide, tall) = (50, 36);
    let mut table = Canvas::new(wide as u32, tall as u32);
    let (cx, cy, rx, ry) = (25, 10, 22, 7);
    let hem = 23;
    // The pedestal and its three feet, below the cloth.
    rect(&mut table, cx - 2, hem, 5, tall - hem - 4, TIMBER.base);
    vline(&mut table, cx - 2, hem, tall - hem - 4, TIMBER.light);
    vline(&mut table, cx + 2, hem, tall - hem - 4, TIMBER.shadow);
    for (to_x, to_y) in [(cx - 11, tall - 2), (cx + 11, tall - 2), (cx + 1, tall - 1)] {
        line(&mut table, (cx, tall - 6), (to_x, to_y), TIMBER.base);
        line(&mut table, (cx, tall - 7), (to_x, to_y - 1), TIMBER.light);
        put(&mut table, to_x, to_y, TIMBER.edge);
    }
    // The cloth: red and cream checks where the threads cross, flat across the top and falling
    // in folds over the front, shaded away from the light.
    let check = |x: i32, y: i32, tall: i32| -> Rgba {
        let red_across = (x.div_euclid(4)) % 2 == 0;
        let red_down = (y.div_euclid(tall)) % 2 == 0;
        match (red_across, red_down) {
            (true, true) => rgb(0xc04a40),
            (false, false) => rgb(0xf6ecd4),
            _ => rgb(0xe2928a),
        }
    };
    for y in cy..hem {
        // How far round the front the skirt reaches at this height: it flares a little.
        let flare = (y - cy) / 4;
        for x in cx - rx - flare..=cx + rx + flare {
            let fold = (x - cx + 40).rem_euclid(7);
            let mut color = check(x, y - cy, 3);
            if fold == 0 {
                color = mix(color, rgb(0x5a2420), 0.35);
            } else if fold == 1 {
                color = mix(color, rgb(0xffffff), 0.2);
            }
            let away = (x - cx) as f32 / rx as f32;
            color = mix(color, rgb(0x5a2420), (away.max(0.0) * 0.3) + 0.08);
            put(&mut table, x, y, color);
        }
    }
    // A scalloped hem.
    for x in cx - rx - 3..=cx + rx + 3 {
        let dip = if (x - cx).rem_euclid(7) < 4 { 1 } else { 0 };
        put(&mut table, x, hem + dip - 1, rgb(0x8a3a32));
        if dip == 1 {
            put(&mut table, x, hem - 1, check(x, hem - cy, 3));
        }
    }
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (dx, dy) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            let mut color = check(x, y - cy + ry, 2);
            if dx + dy < -0.9 {
                color = mix(color, rgb(0xffffff), 0.25);
            }
            put(&mut table, x, y, color);
        }
    }
    // The teapot: blue and white, spout to the right.
    ellipse(&mut table, cx - 7, cy - 2, 5, 4, rgb(0xe4eef0));
    ellipse(&mut table, cx - 8, cy - 3, 3, 2, rgb(0xffffff));
    hline(&mut table, cx - 11, cy - 1, 9, rgb(0x46607a));
    rect(&mut table, cx - 9, cy - 7, 4, 2, rgb(0xc2cfd6));
    put(&mut table, cx - 7, cy - 8, rgb(0x46607a));
    line(
        &mut table,
        (cx - 2, cy - 2),
        (cx + 1, cy - 5),
        rgb(0xc2cfd6),
    );
    line(&mut table, (cx - 12, cy - 4), (cx - 13, cy), rgb(0xc2cfd6));
    hline(&mut table, cx - 11, cy + 2, 9, rgb(0x7c98ae));
    // A cup on its saucer.
    ellipse(&mut table, cx + 3, cy + 3, 3, 1, rgb(0xfbf6ee));
    rect(&mut table, cx + 2, cy, 3, 3, rgb(0xfbf6ee));
    hline(&mut table, cx + 2, cy, 3, rgb(0x7a5a3a));
    put(&mut table, cx + 5, cy + 1, rgb(0xe4eef0));
    // A plate of buns.
    ellipse(&mut table, cx + 13, cy + 1, 6, 2, rgb(0xf6f2ea));
    for (dx, dy) in [(-3, 0), (1, 0), (-1, -2), (3, -1)] {
        ellipse(&mut table, cx + 13 + dx, cy + dy, 2, 1, rgb(0xc98a44));
        put(&mut table, cx + 12 + dx, cy - 1 + dy, rgb(0xe4b36c));
    }
    outline(&mut table, rgb(0x6e2a26));
    table
}

/// The toy box, lid propped open: a ball, a top and a block peeping out.
fn toy_box() -> Canvas {
    let (wide, tall) = (34, 26);
    let mut chest = Canvas::new(wide as u32, tall as u32);
    // The lid, open and leaning back.
    polygon(&mut chest, &[(2, 0), (32, 0), (31, 6), (3, 6)], |x, y| {
        Some(if y == 0 {
            TOYBOX.light
        } else if x < 6 {
            TOYBOX.base
        } else {
            mix(TOYBOX.base, TOYBOX.shadow, 0.4)
        })
    });
    hline(&mut chest, 3, 6, 28, TOYBOX.edge);
    // What is inside.
    ellipse(&mut chest, 9, 9, 4, 4, rgb(0xd0574a));
    put(&mut chest, 7, 7, rgb(0xf19bb0));
    hline(&mut chest, 6, 9, 7, rgb(0xfbf6ee));
    polygon(&mut chest, &[(17, 5), (23, 5), (20, 11)], |_, _| {
        Some(rgb(0xf5d25e))
    });
    put(&mut chest, 20, 4, rgb(0x9a7434));
    put(&mut chest, 18, 6, rgb(0xfff0bd));
    bevel(
        &mut chest,
        24,
        6,
        6,
        6,
        Ramp::new(0x2f5d50, 0x3f7a68, 0x52967f, 0x6fb09a, 0x93cbb6),
    );
    // The box.
    bevel(&mut chest, 0, 10, wide, tall - 12, TOYBOX);
    // The band under the rim stops at the box's lit and shaded sides, so they run unbroken.
    hline(&mut chest, 2, 13, wide - 4, TOYBOX.shadow);
    hline(&mut chest, 2, 14, wide - 4, TOYBOX.shine);
    // Painted stars along the front.
    for (sx, color) in [(7, 0xf5d25e), (16, 0xfbf6ee), (25, 0xf19bb0)] {
        let sy = 19;
        for (dx, dy) in [(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1)] {
            put(&mut chest, sx + dx, sy + dy, rgb(color));
        }
    }
    // Feet.
    for foot in [1, wide - 4] {
        rect(&mut chest, foot, tall - 2, 3, 2, TIMBER.shadow);
    }
    outline(&mut chest, TOYBOX.edge);
    chest
}

/// A fern in a terracotta pot, standing with its foot at `(cx, foot)`.
fn potted_fern(scene: &mut Canvas, cx: i32, foot: i32, salt: u32) {
    for frond in 0..9 {
        let angle = -2.6 + frond as f32 * 0.32 + (noise(frond, 0, salt) % 10) as f32 * 0.02;
        let long = 9.0 + (noise(frond, 1, salt) % 6) as f32;
        let (sx, sy) = (cx as f32, (foot - 8) as f32);
        for step in 0..long as i32 {
            let t = step as f32;
            let droop = t * t * 0.03;
            let x = (sx + angle.cos() * t).round() as i32;
            let y = (sy + angle.sin() * t + droop).round() as i32;
            let color = if step % 3 == 0 { LEAF.light } else { LEAF.base };
            put(scene, x, y, color);
            if step > 1 && step % 2 == 0 {
                put(scene, x, y - 1, LEAF.shadow);
            }
        }
    }
    polygon(
        scene,
        &[
            (cx - 5, foot - 9),
            (cx + 6, foot - 9),
            (cx + 4, foot),
            (cx - 3, foot),
        ],
        |x, _| Some(if x < cx - 1 { TILE.light } else { TILE.base }),
    );
    hline(scene, cx - 6, foot - 9, 13, TILE.shine);
    hline(scene, cx - 6, foot - 8, 13, TILE.shadow);
    hline(scene, cx - 3, foot - 1, 7, TILE.edge);
}

/// Rings a sprite's opaque pixels with `color`, so it reads against the floor.
fn outline(sprite: &mut Canvas, color: Rgba) {
    let (width, height) = (sprite.width() as i32, sprite.height() as i32);
    let solid = |x: i32, y: i32, sprite: &Canvas| {
        x >= 0 && y >= 0 && x < width && y < height && sprite.get(x, y).a > 0
    };
    let mut edges = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if !solid(x, y, sprite) {
                continue;
            }
            let open = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| !solid(x + dx, y + dy, sprite));
            if open {
                edges.push((x, y));
            }
        }
    }
    for (x, y) in edges {
        sprite.set(x, y, color);
    }
}

// ---------------------------------------------------------------------------------------------
// After dark
// ---------------------------------------------------------------------------------------------

/// What keeps the room bright when it is dark outside: the hanging lamp, the fire and the
/// firelight on the floor before it, and the candles on the mantel.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    glow(&mut lights, (LAMP_X, 24), 56, rgb(0xffdc96));
    glow(&mut lights, (FIREBOX.0, FIREBOX.3 - 12), 70, rgb(0xffa04a));
    glow(
        &mut lights,
        (FIREBOX.0, WALL_FOOT + 10),
        84,
        rgba(0xff9a40, 200),
    );
    for candle in [BREAST.0 + 6, BREAST.1 - 7] {
        glow(&mut lights, (candle, 30), 10, rgb(0xffd77a));
    }
    lights
}
