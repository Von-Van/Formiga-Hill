//! The hedgerow, at the eye level of someone standing in the lane: a summer sky over a high old
//! hedge, the hedge broken by a field gate with the Hill far off through it, a crab-apple tree
//! standing up out of the hedge, and the edge of the Woods on the right. In front of the hedge, a
//! grassy bank runs down to a sunken lane with ruts in it, and the near verge is thick with
//! clover; under the trees on the right, wild garlic carpets the bank and a ring of grass marks
//! where mushrooms come up.
//!
//! Along the hedge are its plants: the bramble heaped at the left end and spilling down the bank,
//! a dog rose arching out with honeysuckle through it, an elder, a hazel and the crab apple, and
//! wild strawberries and thyme on the bank. Nothing that can be picked is painted: whatever grows
//! at a patch's slots is drawn as it ripens (see `produce`), and the scenery gives each something
//! to hang from or stand on. The palette is the glade's and the meadow's, so the hedgerow belongs
//! to the same woods.

use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::materials::{FAR_HILL, PATH, SKY_LOW};
use crate::paint::{Ramp, blit, chance, ellipse, hline, line, mix, noise, put, rgb, rgba, vline};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

// The glade's and the meadow's materials, so the hedgerow is plainly at the edge of the same woods.
const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const BRAMBLE: Ramp = Ramp::new(0x1b3320, 0x284628, 0x385e32, 0x4e7840, 0x6a9450);
const SUNLIT_LEAF: Ramp = Ramp::new(0x1b3320, 0x3e6834, 0x568640, 0x74a24e, 0x9cc26a);
const CANE: Ramp = Ramp::new(0x3a1e22, 0x56292c, 0x723a36, 0x8c4e42, 0xa66a52);
const OAK: Ramp = Ramp::new(0x1a3428, 0x264a34, 0x356240, 0x4c7e4c, 0x6c9c5c);
const BEECH: Ramp = Ramp::new(0x22401f, 0x33582a, 0x4a7236, 0x648e42, 0x86ac58);
const HOLLY: Ramp = Ramp::new(0x122a24, 0x1a3a30, 0x24503e, 0x34664c, 0x4e8060);
const HAWTHORN: Ramp = Ramp::new(0x203c22, 0x30562c, 0x447038, 0x5c8c46, 0x7caa58);
const HAZEL_LEAF: Ramp = Ramp::new(0x29441f, 0x3a5c2a, 0x4f7834, 0x6a9442, 0x8cb058);
const SWARD: Ramp = Ramp::new(0x34502a, 0x4a6c34, 0x618a3e, 0x7ca64a, 0x9cc05e);
const CAMPION: Ramp = Ramp::new(0x7a2448, 0xa8345e, 0xcc4a74, 0xe06a8e, 0xf096ae);
const CLOUDS: Ramp = Ramp::new(0xc9d7e2, 0xdfe7ee, 0xf4f4f1, 0xfdfbf5, 0xffffff);
// The hedgerow's own.
const ELDER_LEAF: Ramp = Ramp::new(0x2c4a1e, 0x40662a, 0x568436, 0x70a044, 0x96c060);
const ROSE_LEAF: Ramp = Ramp::new(0x203a20, 0x30542c, 0x427038, 0x5a8a46, 0x7aa65a);
const ROSE: Ramp = Ramp::new(0xa0506a, 0xd27a92, 0xec9eb2, 0xf8c4d0, 0xfde8ee);
const APPLE_BARK: Ramp = Ramp::new(0x2a201c, 0x40322a, 0x58463a, 0x70604e, 0x8a7a64);
const STRAWBERRY_LEAF: Ramp = Ramp::new(0x1e3a1c, 0x2e5428, 0x427034, 0x5a8c42, 0x7aaa56);
const GARLIC: Ramp = Ramp::new(0x1a3a20, 0x28542c, 0x3a7038, 0x528c46, 0x72a85a);
const CLOVER_LEAF: Ramp = Ramp::new(0x23452a, 0x325f34, 0x447a40, 0x5c944e, 0x7cb064);
const CLOVER_HEAD: Ramp = Ramp::new(0x8a4a62, 0xb46a82, 0xd290a2, 0xe8b4c2, 0xf8dce4);
const GATE: Ramp = Ramp::new(0x4a4844, 0x6e6a62, 0x8e8a80, 0xaca69a, 0xc8c2b4);
const LICHEN: Rgba = rgb(0xc8c890);
const LANE: Ramp = Ramp::new(0x5a4630, 0x7a6244, 0x967a56, 0xb0966c, 0xc8b086);
const RUT: Ramp = Ramp::new(0x3a2c1e, 0x4e3c2a, 0x624c36, 0x7a6248, 0x92785a);
const PUDDLE: Ramp = Ramp::new(0x3a5058, 0x587078, 0x7a949a, 0xa2bcc0, 0xd8eaec);
const LITTER: Ramp = Ramp::new(0x3a2618, 0x583a22, 0x74502e, 0x8e6a3e, 0xaa8656);
const IRON: Ramp = Ramp::new(0x1c2224, 0x2e3436, 0x444a4c, 0x5e6466, 0x80888a);
const STITCHWORT: Rgba = rgb(0xf8f8f0);
const BLOSSOM: Rgba = rgb(0xf6eef0);
/// The shade under the trees.
const UNDERSTOREY: Rgba = rgb(0x16281f);
/// The air between the lane and anything far off.
const DISTANCE: Rgba = rgb(0xa8cac4);
const HIGH_SKY: Rgba = rgb(0x9ccce6);
const SHADE: Rgba = rgba(0x1c3020, 70);

/// The Hill's turf, and its old tree's leaves and bark, in the Hilltop's own tones.
const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const OLD_BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
const HILL_HAZE: f32 = 0.32;

/// Where the gate stands in the hedge, post to post, and the tops of its posts.
const GATE_POSTS: (i32, i32) = (168, 212);
const POST_TOP: i32 = 76;
/// The far edge of the field beyond the gate, and the Hill beyond that.
const FAR_EDGE: i32 = 84;
const HILL_MIDDLE: f32 = 192.0;
const HILL_TOP: f32 = 56.0;
const HILL_FALL: f32 = 30.0;
const HILL_BREADTH: f32 = 22.0;

/// The Hill's skyline through the gate: a broad flat summit, rounding over into its flanks.
fn hill_crest(x: f32) -> f32 {
    let off = (x - HILL_MIDDLE) / HILL_BREADTH;
    HILL_TOP + HILL_FALL * (1.0 - (-(off * off * off * off)).exp())
}

/// The Hill through the gate, with whatever stands on it.
const VISTA: Vista = Vista {
    tree: (181.0, 57.8),
    crest: hill_crest,
    spread: 12.0,
    shrink: 8,
    tint: Tint::Haze(DISTANCE, HILL_HAZE),
};

/// The lowest row of open sky anywhere, between the hedge and the far fields.
const SKY_FOOT: i32 = FAR_EDGE;
/// Where the lane's near edge runs, and where the bank comes down to it.
const LANE_TOP: i32 = 146;

/// Where the bank's foot meets the lane at `x`.
fn lane_top(x: i32) -> i32 {
    LANE_TOP + (patches(x, 0, (11, 1), 501) * 3.0) as i32 - 1
}

/// Where the lane's near edge meets the verge at `x`: straight across, then curving up and away
/// into the trees on the right.
fn lane_foot(x: i32) -> i32 {
    let bend = ((x - 286) as f32 / 98.0).clamp(0.0, 1.0);
    186 - (bend * bend * 24.0) as i32 + (patches(x, 1, (9, 1), 502) * 2.0) as i32
}

/// The top of the hedge at `x`: high and lumpy where the bramble, the rose and the elder are,
/// cut low either side of the gate, up again for the hazel, and down where the crab apple stands
/// over it, so its trunk shows.
fn hedge_top(x: i32) -> i32 {
    let lump =
        |middle: f32, wide: f32, high: f32| high * (-((x as f32 - middle) / wide).powi(2)).exp();
    let away = ((x as f32 - 190.0) / 34.0).powi(4);
    let high = 26.0 * (1.0 - (-away).exp());
    let lumps = lump(20.0, 14.0, 8.0)
        + lump(70.0, 16.0, 10.0)
        + lump(140.0, 18.0, 14.0)
        + lump(246.0, 16.0, 10.0)
        - lump(286.0, 14.0, 34.0);
    (88.0 - high - lumps) as i32 + (patches(x, 2, (5, 1), 503) * 4.0) as i32
}

/// The foot of the hedge at `x`, where it meets the top of the bank.
fn hedge_foot(x: i32) -> i32 {
    112 + (patches(x, 3, (13, 1), 504) * 4.0) as i32
}

/// Everything behind the companions, filling every pixel.
pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    far_country(&mut scene);
    skyline(&mut scene, hilltop, &VISTA);
    far_field(&mut scene);
    woods(&mut scene);
    crab_apple(&mut scene);
    hedge(&mut scene);
    gate(&mut scene);
    bank(&mut scene);
    lane(&mut scene);
    verge(&mut scene);
    under_the_trees(&mut scene);
    big_oak(&mut scene);
    bramble(&mut scene);
    dog_rose(&mut scene);
    elder(&mut scene);
    hazel(&mut scene);
    strawberries(&mut scene);
    scene
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

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

/// A number from the hash in `0..n`.
fn pick(index: i32, axis: i32, salt: u32, n: i32) -> i32 {
    (noise(index, axis, salt) % n.max(1) as u32) as i32
}

/// A ramp's tone by level, 0 its edge and 4 its shine.
fn tone(ramp: Ramp, level: i32) -> Rgba {
    match level {
        ..=0 => ramp.edge,
        1 => ramp.shadow,
        2 => ramp.base,
        3 => ramp.light,
        _ => ramp.shine,
    }
}

/// `color` as seen through `amount` of the distance.
fn hazed(color: Rgba, amount: f32) -> Rgba {
    mix(color, DISTANCE, amount)
}

/// A point along a quadratic curve from `from` through towards `bend` to `to`.
fn curve(from: (i32, i32), bend: (i32, i32), to: (i32, i32), t: f32) -> (i32, i32) {
    let along = |a: i32, b: i32, c: i32| {
        ((1.0 - t) * (1.0 - t) * a as f32 + 2.0 * (1.0 - t) * t * b as f32 + t * t * c as f32)
            .round() as i32
    };
    (along(from.0, bend.0, to.0), along(from.1, bend.1, to.1))
}

/// A clump of leaves, its rim scalloped into smaller clusters, lit from the upper left and lost
/// in the distance by `haze`.
fn clump(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    radius: i32,
    (ramp, haze): (Ramp, f32),
    salt: u32,
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
        ellipse(scene, x, y, size + 1, size + 1, hazed(ramp.edge, haze));
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
            put(scene, x, y, hazed(color, haze));
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
        ellipse(scene, x, y, size, size, hazed(color, haze));
        if light < -0.9 {
            put(scene, x - 1, y - 1, hazed(ramp.shine, haze));
        }
    }
}

/// A tree's crown from the side: a rounded mass of leaves, its rim scalloped and outlined in its
/// own darkest green, filled with clumps of leaves back to front, each with a dark underside and
/// a lit crescent on its upper left, brighter towards the upper left of the whole crown.
fn crown(
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

/// A small leaf, two by two with a tip, lit at `level` (1 to 3).
fn leaf(scene: &mut Canvas, (x, y): (i32, i32), ramp: Ramp, level: i32, tip_left: bool) {
    let (lit, dark) = (tone(ramp, level + 1), tone(ramp, level - 1));
    put(scene, x, y, lit);
    put(scene, x + 1, y, tone(ramp, level));
    put(scene, x, y + 1, tone(ramp, level));
    put(scene, x + 1, y + 1, dark);
    let tip = if tip_left { x - 1 } else { x + 2 };
    put(
        scene,
        tip,
        y + if tip_left { 0 } else { 1 },
        tone(ramp, level),
    );
    put(scene, x, y + 2, ramp.edge);
    put(scene, x + 1, y + 2, ramp.edge);
}

/// A stem or cane along a curve `(from, bend, to)`, lit along its top and edged below.
fn stem(scene: &mut Canvas, (from, bend, to): ((i32, i32), (i32, i32), (i32, i32)), ramp: Ramp) {
    let mut previous = from;
    for step in 1..=28 {
        let point = curve(from, bend, to, step as f32 / 28.0);
        line(scene, previous, point, ramp.base);
        put(scene, point.0, point.1 - 1, ramp.light);
        put(scene, point.0, point.1 + 1, ramp.edge);
        previous = point;
    }
}

/// A heaped mass of small leaves inside `inside`, lit from the upper left and darker deeper in:
/// a bush in a hedge, every leaf drawn.
fn leafy(
    scene: &mut Canvas,
    bounds: (i32, i32, i32, i32),
    inside: impl Fn(i32, i32) -> bool,
    ramp: Ramp,
    (step, salt): (i32, u32),
) {
    let (left, top, right, bottom) = bounds;
    // The dark inside of the bush, so nothing shows through between the leaves.
    for y in top..=bottom {
        for x in left..=right {
            if inside(x, y) {
                let deep = if chance(x, y, salt, 40) {
                    ramp.shadow
                } else {
                    mix(ramp.shadow, ramp.edge, 0.55)
                };
                scene.set(x, y, deep);
            }
        }
    }
    let height = (bottom - top).max(1) as f32;
    let width = (right - left).max(1) as f32;
    let mut row = 0;
    let mut y = top;
    while y <= bottom {
        let offset = if row % 2 == 0 { 0 } else { step / 2 + 1 };
        let mut x = left + offset;
        while x <= right {
            let (lx, ly) = (
                x + pick(x, y, salt + 1, 3) - 1,
                y + pick(x, y, salt + 2, 3) - 1,
            );
            if inside(lx, ly) && inside(lx + 1, ly + 1) && !chance(lx, ly, salt + 3, 50) {
                // How far into the bush from its top edge, and across it from the sun.
                let mut depth = 0;
                while depth < 12 && inside(lx, ly - depth - 1) {
                    depth += 1;
                }
                let across = (lx - left) as f32 / width;
                let down = (ly - top) as f32 / height;
                let light = 3.4 - depth as f32 / 5.0 - across * 0.6 - down * 0.4
                    + pick(lx, ly, salt + 4, 3) as f32 * 0.4;
                let level = light.round().clamp(1.0, 3.0) as i32;
                leaf(scene, (lx, ly), ramp, level, (lx + ly) % 3 == 0);
            }
            x += step;
        }
        y += 2;
        row += 1;
    }
}

// ---------------------------------------------------------------------------------------------
// The sky and the far country
// ---------------------------------------------------------------------------------------------

/// The sky on its own, clouds and all: deep overhead, paling to a warm haze low down, a big
/// fair-weather cloud over the left of the hedge and smaller ones drifting.
pub fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 9;
    let band = |y: i32| (y * BANDS / SKY_FOOT).min(BANDS - 1);
    for y in 0..SKY_FOOT + 30 {
        for x in 0..WIDTH {
            let mut index = band(y);
            if band(y + 1) != index && (x + y) % 2 == 0 {
                index += 1;
            }
            let t = (index as f32 / (BANDS - 1) as f32).min(1.0);
            scene.set(x, y, mix(HIGH_SKY, SKY_LOW, t));
        }
    }
    cloud(
        scene,
        (70, 16),
        &[
            (-24, 2, 6),
            (-12, -3, 8),
            (1, -6, 10),
            (14, -2, 8),
            (25, 2, 5),
        ],
    );
    cloud(scene, (196, 26), &[(-7, 0, 3), (0, -3, 5), (7, -1, 3)]);
    cloud(scene, (238, 10), &[(-5, 0, 3), (1, -2, 4), (7, 0, 2)]);
}

/// A heaped cloud from round puffs `(dx, dy, radius)`: flat underneath, sunlit on top and to the
/// left, cool grey below, and edged only along its underside.
fn cloud(scene: &mut Canvas, (cx, cy): (i32, i32), puffs: &[(i32, i32, i32)]) {
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

fn brightness(color: Rgba) -> u32 {
    u32::from(color.r) + u32::from(color.g) + u32::from(color.b)
}

/// `color` as the Hill's distance leaves it.
fn far(color: Rgba) -> Rgba {
    mix(color, DISTANCE, HILL_HAZE)
}

/// What shows over the gate: downs a long way off, the Hill in front of them with its path
/// winding up to the old tree on its summit, and the far hedgerow along the field's edge.
fn far_country(scene: &mut Canvas) {
    downs(scene);
    hill(scene);
    hill_path(scene);
    old_tree(scene);
}

/// Downs far behind the Hill, pale in the haze.
fn downs(scene: &mut Canvas) {
    for x in 120..270 {
        let ridge =
            (66.0 + (x as f32 * 0.06).sin() * 2.5 + (x as f32 * 0.19).sin() * 0.8).round() as i32;
        for y in ridge..FAR_EDGE + 4 {
            let color = if y == ridge {
                mix(FAR_HILL, TURF.shadow, 0.3)
            } else if y - ridge < 2 && x < HILL_MIDDLE as i32 {
                mix(FAR_HILL, TURF.shine, 0.3)
            } else {
                FAR_HILL
            };
            scene.set(x, y, mix(color, DISTANCE, 0.45));
        }
    }
}

/// The Hill: a dome of turf lit on its left flank and shaded on its right, outlined along its
/// crest, a hedgerow tree or two on its lower slopes.
fn hill(scene: &mut Canvas) {
    for x in 140..250 {
        let crest = hill_crest(x as f32 + 0.5).round() as i32;
        if crest >= FAR_EDGE {
            continue;
        }
        let across = (x as f32 + 0.5 - HILL_MIDDLE) / HILL_BREADTH;
        for y in crest..FAR_EDGE + 4 {
            let down = (y - crest) as f32;
            let dither = (noise(x, y, 511) % 100) as f32 / 100.0 * 0.16;
            let light = across * 0.9 + down * 0.01 + dither;
            let mut color = if light < -0.3 {
                TURF.light
            } else if light < 0.2 {
                TURF.base
            } else if light < 0.6 {
                TURF.shadow
            } else {
                mix(TURF.shadow, TURF.edge, 0.45)
            };
            match noise(x.div_euclid(2), y, 512) % 23 {
                0 => color = mix(color, TURF.edge, 0.35),
                1 => color = mix(color, TURF.shine, 0.35),
                _ => {}
            }
            scene.set(x, y, far(color));
        }
        let edge = if across < -0.2 {
            mix(TURF.shadow, TURF.edge, 0.3)
        } else {
            TURF.edge
        };
        put(scene, x, crest, far(edge));
    }
    for (cx, cy) in [(170, 74), (213, 72)] {
        ellipse(scene, cx + 1, cy + 2, 2, 1, far(TURF.edge));
        ellipse(scene, cx, cy, 2, 2, far(CROWN.shadow));
        put(scene, cx - 1, cy - 1, far(CROWN.light));
        put(scene, cx, cy - 1, far(CROWN.base));
        put(scene, cx + 1, cy + 1, far(CROWN.edge));
    }
}

/// The path up the Hill, winding across its face to the summit by the old tree.
fn hill_path(scene: &mut Canvas) {
    let tree = VISTA.tree.0.round() as i32;
    let summit = (tree - 2, hill_crest(tree as f32 - 1.5).round() as i32 + 1);
    let bends = [(200, FAR_EDGE), (190, 74), (202, 67), (188, 62), summit];
    for (index, pair) in bends.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        line(scene, from, to, far(mix(PATH, TURF.base, 0.25)));
        line(
            scene,
            (from.0, from.1 + 1),
            (to.0, to.1 + 1),
            far(mix(
                TURF.shadow,
                rgb(0x8e7553),
                if index < 2 { 0.4 } else { 0.25 },
            )),
        );
    }
}

/// The old tree's crown as overlapping masses `(x, y, radius)` from the foot of its trunk, as
/// the Hilltop has it in its own pixels.
const OLD_TREE_CROWN: [(f32, f32, f32); 8] = [
    (-3.0, -82.0, 27.0),
    (-27.0, -92.0, 16.0),
    (21.0, -90.0, 15.0),
    (-45.0, -76.0, 13.0),
    (-31.0, -62.0, 18.0),
    (27.0, -66.0, 15.0),
    (13.0, -58.0, 9.0),
    (-47.0, -56.0, 9.0),
];

fn old_tree_mass(hx: f32, hy: f32) -> Option<(f32, f32, f32)> {
    OLD_TREE_CROWN
        .iter()
        .copied()
        .filter(|&(cx, cy, r)| (hx - cx).powi(2) + (hy - cy).powi(2) <= (r + 3.0).powi(2))
        .min_by(|a, b| a.2.total_cmp(&b.2))
}

/// The old tree on the summit, at the same scale as the pieces standing about it.
fn old_tree(scene: &mut Canvas) {
    let scale = VISTA.shrink as f32;
    let foot_x = VISTA.tree.0.round() as i32;
    let foot_y = hill_crest(VISTA.tree.0).round() as i32 + 1;
    let at = |x: i32, y: i32| {
        (
            (x - foot_x) as f32 * scale + scale * 0.5,
            (y - foot_y) as f32 * scale + scale * 0.5,
        )
    };
    hline(scene, foot_x, foot_y, 4, rgba(0x2c4a2e, 70));
    for y in foot_y - 7..=foot_y {
        let shade = if foot_y - y >= 4 { 0.5 } else { 0.0 };
        put(
            scene,
            foot_x - 1,
            y,
            far(mix(OLD_BARK.light, OLD_BARK.edge, shade)),
        );
        put(
            scene,
            foot_x,
            y,
            far(mix(OLD_BARK.shadow, OLD_BARK.edge, shade)),
        );
    }
    put(scene, foot_x - 2, foot_y, far(OLD_BARK.base));
    put(scene, foot_x + 1, foot_y, far(OLD_BARK.edge));
    let mut layer = Canvas::new(36, 26);
    let origin = (foot_x - 18, foot_y - 23);
    for ly in 0..26 {
        for lx in 0..36 {
            let (x, y) = (origin.0 + lx, origin.1 + ly);
            let (hx, hy) = at(x, y);
            let Some((mx, my, r)) = old_tree_mass(hx, hy) else {
                continue;
            };
            let local = 0.5 - ((hx - mx) + (hy - my)) / (3.0 * r);
            let whole = 1.0 - (hx + 53.0) / 110.0 * 0.45 - (hy + 112.0) / 75.0 * 0.55;
            let fleck = (noise(x, y, 521) % 100) as f32 / 100.0 * 0.12;
            let lit = local * 0.45 + whole * 0.55 + fleck;
            let color = if lit > 0.62 {
                CROWN.light
            } else if lit > 0.44 {
                CROWN.base
            } else if lit > 0.28 {
                CROWN.shadow
            } else {
                mix(CROWN.edge, CROWN.shadow, 0.5)
            };
            layer.set(lx, ly, color);
        }
    }
    let mut edges = Vec::new();
    for ly in 0..26 {
        for lx in 0..36 {
            if layer.get(lx, ly).a == 0 {
                continue;
            }
            let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| layer.get(lx + dx, ly + dy).a == 0);
            if open {
                edges.push((lx, ly));
            }
        }
    }
    for (lx, ly) in edges {
        layer.set(lx, ly, CROWN.edge);
    }
    for ly in 0..26 {
        for lx in 0..36 {
            let pixel = layer.get(lx, ly);
            if pixel.a > 0 {
                scene.set(origin.0 + lx, origin.1 + ly, far(pixel));
            }
        }
    }
}

/// The field beyond the hedge, mown and pale, running back to a far hedgerow of rounded trees
/// along its edge: seen through the gate, and under the crab apple where the hedge is low.
fn far_field(scene: &mut Canvas) {
    for x in 0..WIDTH {
        for y in FAR_EDGE..124 {
            let near = ((y - FAR_EDGE) as f32 / 30.0).clamp(0.0, 1.0);
            // Swathes of mown grass running across, alternately lighter and darker.
            let swathe = ((y - FAR_EDGE) as f32 * 0.55 + (x as f32 * 0.02)).sin() > 0.2;
            let base = if swathe { SWARD.light } else { SWARD.base };
            let mut color = mix(base, rgb(0xc8c088), 0.35);
            if chance(x, y, 531, 30) {
                color = mix(color, SWARD.shine, 0.4);
            }
            scene.set(x, y, mix(color, DISTANCE, 0.4 * (1.0 - near)));
        }
    }
    for (index, x) in (-4..WIDTH + 4).step_by(5).enumerate() {
        let i = index as i32;
        let radius = 3 + pick(i, 0, 532, 3);
        let cx = x + pick(i, 1, 532, 3) - 1;
        let cy = FAR_EDGE - 1 - pick(i, 2, 532, 3);
        let ramp = if i % 3 == 0 { BEECH } else { OAK };
        clump(scene, (cx, cy), radius, (ramp, 0.5), 533 + index as u32 * 3);
    }
}

// ---------------------------------------------------------------------------------------------
// The woods on the right
// ---------------------------------------------------------------------------------------------

/// Where the woods begin, on the right.
const WOODS: i32 = 326;

/// The edge of the woods on the right: the shade under the trees deepening in bands down to the
/// bank, trunks going back into it, a rank of dark undergrowth along its foot, and the canopy over
/// it all, its leaves hanging down into the shade.
fn woods(scene: &mut Canvas) {
    let edge = |y: i32| WOODS + 4 + (patches(y, 0, (1, 6), 540) * 6.0) as i32;
    for y in 0..LANE_TOP {
        for x in edge(y)..WIDTH {
            // Stepped in bands with dithered edges, like paint rather than a smooth fade.
            let level = (y + pick(x, y, 541, 5) - 2).div_euclid(8) as f32 * 8.0;
            let deep = (level / 130.0).clamp(0.0, 1.0);
            scene.set(
                x,
                y,
                mix(mix(OAK.shadow, OAK.edge, 0.4), UNDERSTOREY, deep * 0.85),
            );
        }
    }
    // Trunks going back into the shade, fainter the further.
    for (index, x) in (WOODS + 2..WIDTH).step_by(8).enumerate() {
        let x = x + pick(index as i32, 0, 542, 4);
        let depth = 0.45 + pick(index as i32, 1, 542, 3) as f32 * 0.12;
        let width = 2 + pick(index as i32, 2, 542, 2);
        for y in 30..124 {
            for dx in 0..width {
                let color = if dx == 0 { BARK.light } else { BARK.shadow };
                scene.set(x + dx, y, mix(color, UNDERSTOREY, depth));
            }
        }
    }
    // Undergrowth along the foot of the trees.
    for (index, x) in (WOODS - 2..WIDTH + 6).step_by(9).enumerate() {
        let i = index as i32;
        clump(
            scene,
            (x + pick(i, 0, 543, 5), 112 + pick(i, 1, 543, 6)),
            6 + pick(i, 2, 543, 4),
            (
                Ramp::new(0x10221a, 0x1a3424, 0x24462e, 0x325a3a, 0x46704a),
                0.0,
            ),
            544 + index as u32 * 5,
        );
    }
    // The canopy over it all.
    for (index, &((cx, cy), (rx, ry), ramp)) in [
        ((364, 4), (26, 20), HOLLY),
        ((348, 30), (14, 16), OAK),
        ((384, 34), (20, 22), HOLLY),
        ((374, 58), (14, 10), OAK),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (ramp, 0.0),
            545 + index as u32 * 7,
        );
    }
    // The trees' edge, where the woods meet the open: dark leaves from the canopy down to the
    // undergrowth, so the shade inside has a boundary.
    for (index, &((cx, cy), (rx, ry))) in [
        ((332, 52), (8, 14)),
        ((330, 80), (8, 12)),
        ((334, 102), (9, 10)),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (HOLLY, 0.0),
            549 + index as u32 * 7,
        );
    }
    // Leaves hanging down from the canopy into the shade, catching a little light.
    for x in WOODS..WIDTH {
        let hang = 66 + (patches(x, 0, (5, 1), 546) * 10.0) as i32;
        for y in hang..hang + 3 + pick(x, 1, 547, 4) {
            if chance(x, y, 547, 140) {
                put(
                    scene,
                    x,
                    y,
                    if chance(x, y, 548, 60) {
                        OAK.base
                    } else {
                        OAK.shadow
                    },
                );
            }
        }
    }
}

/// The big oak at the very right, its trunk in the shade going up out of the picture and its
/// roots spreading over the bank and the floor.
fn big_oak(scene: &mut Canvas) {
    let left = 368;
    for y in 0..176 {
        let flare = ((y - 150).max(0) * 3) / 4;
        let from = left - flare;
        for x in from..WIDTH {
            let across = (x - from) as f32 / (WIDTH - from).max(1) as f32;
            let mut color = if x == from {
                BARK.edge
            } else if across < 0.25 {
                BARK.light
            } else if across < 0.6 {
                BARK.base
            } else {
                BARK.shadow
            };
            let streak = noise(x, (y + pick(x, 1, 551, 9)).div_euclid(4), 551);
            if x > from && streak.is_multiple_of(4) {
                color = mix(color, BARK.edge, 0.6);
            }
            if y > 130 && patches(x, y, (3, 4), 552) > 0.5 + across * 0.3 {
                color = if across < 0.4 { MOSS.light } else { MOSS.base };
            }
            let shaded = (0.6 - y as f32 / 300.0).clamp(0.15, 0.6);
            scene.set(x, y, mix(color, UNDERSTOREY, shaded));
        }
    }
    // Roots over the floor, thick where they leave the trunk and tapering, lit along their tops.
    for &(from, bend, to) in &[
        ((368, 170), (360, 167), (352, 171)),
        ((372, 175), (372, 188), (366, 200)),
        ((378, 176), (384, 188), (382, 206)),
    ] {
        let mut drawn: Vec<(i32, i32, i32)> = Vec::new();
        for step in 0..=60 {
            let t = step as f32 / 60.0;
            let (x, y) = curve(from, bend, to, t);
            let thick = (4.5 * (1.0 - t)).round() as i32 + 2;
            if drawn.iter().any(|&(px, py, _)| px == x && py == y) {
                continue;
            }
            drawn.push((x, y, thick));
        }
        for &(x, y, thick) in &drawn {
            for dy in -1..thick {
                let color = match dy {
                    -1 => mix(BARK.edge, UNDERSTOREY, 0.2),
                    0 => BARK.light,
                    d if d == thick - 1 => BARK.edge,
                    d if d < thick / 2 => BARK.base,
                    _ => BARK.shadow,
                };
                scene.set(x, y + dy, mix(color, UNDERSTOREY, 0.2));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The crab apple
// ---------------------------------------------------------------------------------------------

/// The crab apple standing up over the hedge where it dips: a short crooked trunk, boughs
/// spreading under a broad crown of fresh green, and the low branches the apples hang from,
/// against the field beyond.
fn crab_apple(scene: &mut Canvas) {
    const LEAVES: Ramp = Ramp::new(0x23422a, 0x335e34, 0x4a7e40, 0x66a050, 0x8ec068);
    // The trunk, leaning a little, its bark rough.
    for y in 50..116_i32 {
        let lean = ((116 - y) as f32 * 0.06) as i32;
        let flare = ((y - 100).max(0)) / 3;
        let (from, to) = (282 + lean - flare, 290 + lean + flare);
        for x in from..to {
            let across = (x - from) as f32 / (to - from) as f32;
            let color = if x == from || x == to - 1 {
                APPLE_BARK.edge
            } else if across < 0.35 {
                APPLE_BARK.light
            } else if across < 0.7 {
                APPLE_BARK.base
            } else {
                APPLE_BARK.shadow
            };
            let knot = noise(x, y.div_euclid(3), 561).is_multiple_of(7);
            scene.set(x, y, if knot { APPLE_BARK.edge } else { color });
        }
    }
    // Boughs out from the trunk, under the crown.
    for &(from, bend, to) in &[
        ((285, 70), (270, 62), (256, 66)),
        ((289, 68), (304, 60), (318, 66)),
        ((286, 60), (276, 48), (268, 40)),
        ((288, 58), (298, 46), (306, 38)),
    ] {
        for thick in 0..2 {
            stem(
                scene,
                (
                    (from.0, from.1 + thick),
                    (bend.0, bend.1 + thick),
                    (to.0, to.1 + thick),
                ),
                APPLE_BARK,
            );
        }
    }
    for (index, &((cx, cy), (rx, ry))) in [
        ((286, 32), (30, 20)),
        ((262, 48), (13, 10)),
        ((310, 48), (13, 10)),
        ((288, 14), (20, 11)),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (LEAVES, 0.0),
            563 + index as u32 * 5,
        );
    }
    // The low branches the apples hang from, out under the crown.
    for &(from, bend, to) in &[
        ((268, 64), (262, 74), (262, 82)),
        ((286, 74), (283, 82), (282, 88)),
        ((302, 64), (306, 76), (304, 84)),
    ] {
        stem(scene, (from, bend, to), APPLE_BARK);
        let (x, y) = to;
        leaf(scene, (x - 4, y - 3), LEAVES, 2, true);
        leaf(scene, (x + 2, y - 4), LEAVES, 3, false);
    }
    // Twigs out to the high apples.
    for &(x, y) in &[(276, 40), (292, 30), (306, 46)] {
        line(scene, (x, y - 4), (x, y), APPLE_BARK.base);
        leaf(scene, (x + 2, y - 3), LEAVES, 3, false);
    }
}

// ---------------------------------------------------------------------------------------------
// The hedge
// ---------------------------------------------------------------------------------------------

/// The hedge along the top of the bank: old hawthorn, its masses of small leaves heaped and lit
/// from the upper left, dark between them, with the hedge's bare stems showing at its foot; cut
/// low either side of the gate and under the crab apple. The shrubs that grow out of it are
/// painted over it after.
fn hedge(scene: &mut Canvas) {
    let in_gap = |x: i32| (GATE_POSTS.0 + 2..GATE_POSTS.1 + 1).contains(&x);
    // The dark inside of the hedge, so no sky shows between its masses.
    for x in 0..WOODS + 6 {
        if in_gap(x) {
            continue;
        }
        for y in hedge_top(x) + 6..=hedge_foot(x) {
            scene.set(x, y, mix(HAWTHORN.shadow, UNDERSTOREY, 0.5));
        }
    }
    // Back masses, then front ones lower down, each a heap of leaves.
    for (row, (step, down, size)) in [(15, 7, (10, 9)), (19, 17, (13, 11))]
        .into_iter()
        .enumerate()
    {
        for (index, x) in (-6..WOODS + 10).step_by(step).enumerate() {
            let i = index as i32;
            let cx = x + pick(i, row as i32, 571, 5) - 2;
            if in_gap(cx) || in_gap(cx - size.0 / 2) || in_gap(cx + size.0 / 2) {
                continue;
            }
            let cy = (hedge_top(cx) + down).min(hedge_foot(cx) - size.1 / 2);
            let rx = size.0 + pick(i, 3, 571, 4);
            let ry = size.1 + pick(i, 4, 571, 3);
            crown(
                scene,
                (cx, cy),
                (rx, ry),
                (HAWTHORN, 0.0),
                572 + (row * 100 + index) as u32,
            );
        }
    }
    // Its stems at the foot, in the shade under the leaves, and its shade on the bank.
    for x in 0..WOODS {
        if in_gap(x) {
            continue;
        }
        let foot = hedge_foot(x);
        let stem = chance(x, 0, 573, 22);
        for y in foot - 4..=foot {
            if stem && y > foot - 3 {
                put(scene, x, y, BARK.shadow);
            } else {
                put(scene, x, y, mix(HAWTHORN.edge, UNDERSTOREY, 0.4));
            }
        }
        for dy in 1..4 {
            put(scene, x, foot + dy, rgba(0x1c3020, (60 - dy * 15) as u8));
        }
    }
    // A few haws, gone dark red, along the lit tops.
    for index in 0..40 {
        let x = pick(index, 0, 574, WOODS);
        if in_gap(x) || (54..124).contains(&x) {
            continue;
        }
        let y = hedge_top(x) + 4 + pick(index, 1, 574, 10);
        put(scene, x, y, rgb(0x8a2a24));
        put(scene, x - 1, y - 1, rgb(0xc8564a));
    }
}

/// The field gate: two stout posts and five weathered bars with a brace, silver-grey and spotted
/// with lichen, the far field and the Hill showing through it, and an iron lantern on a hook on
/// the hanging post for the lane after dark.
fn gate(scene: &mut Canvas) {
    let (left, right) = GATE_POSTS;
    let foot = 116;
    let post = |scene: &mut Canvas, x: i32| {
        for y in POST_TOP..foot {
            for dx in 0..5 {
                let color = match dx {
                    0 => GATE.edge,
                    1 => GATE.light,
                    2 | 3 => GATE.base,
                    _ => GATE.shadow,
                };
                let grain = noise(x + dx, y.div_euclid(3), 581).is_multiple_of(6);
                scene.set(
                    x + dx,
                    y,
                    if grain {
                        mix(color, GATE.edge, 0.5)
                    } else {
                        color
                    },
                );
            }
            put(scene, x + 5, y, GATE.edge);
        }
        hline(scene, x, POST_TOP, 6, GATE.edge);
        hline(scene, x + 1, POST_TOP + 1, 3, GATE.shine);
        for (dx, dy) in [(2, 8), (3, 9), (1, 20), (3, 30)] {
            put(scene, x + dx, POST_TOP + dy, LICHEN);
        }
    };
    post(scene, left);
    post(scene, right);
    // The bars.
    for (index, y) in [84, 91, 98, 105, 111].into_iter().enumerate() {
        for x in left + 5..right {
            let color = if (x + index as i32 * 3) % 23 == 0 {
                mix(GATE.base, LICHEN, 0.6)
            } else {
                GATE.base
            };
            put(scene, x, y, GATE.light);
            put(scene, x, y + 1, color);
            put(scene, x, y + 2, GATE.edge);
        }
    }
    // The brace, from the foot of the hanging side up to the top of the latch side.
    for step in 0..=30 {
        let t = step as f32 / 30.0;
        let x = (left as f32 + 6.0 + t * (right - left - 8) as f32) as i32;
        let y = (110.0 - t * 24.0) as i32;
        put(scene, x, y, GATE.light);
        put(scene, x, y + 1, GATE.base);
        put(scene, x, y + 2, GATE.edge);
    }
    // The latch.
    scene.fill_rect(right - 3, 92, 3, 2, IRON.base);
    put(scene, right - 3, 92, IRON.light);
    // The lantern on its hook, unlit by day.
    let (lx, ly) = (left - 4, POST_TOP + 6);
    hline(scene, lx, ly - 2, 5, IRON.base);
    put(scene, lx, ly - 1, IRON.base);
    let lantern = [".###.", "#lol#", "#ogo#", "#oGo#", ".###.", "..#.."];
    for (dy, row) in lantern.iter().enumerate() {
        for (dx, code) in row.bytes().enumerate() {
            let color = match code {
                b'#' => IRON.edge,
                b'l' => IRON.light,
                b'o' => IRON.base,
                b'g' => rgb(0xb8c4b8),
                b'G' => rgb(0x9aa89c),
                _ => continue,
            };
            put(scene, lx - 2 + dx as i32, ly + dy as i32, color);
        }
    }
}

/// Where the lantern on the gate post hangs, for its light after dark.
const LANTERN: (i32, i32) = (GATE_POSTS.0 - 4, POST_TOP + 8);

// ---------------------------------------------------------------------------------------------
// The bank, the lane and the verge
// ---------------------------------------------------------------------------------------------

/// The bank from the hedge's foot down to the lane: grass on it, earth and the odd stone showing
/// through where it is steep, flowers along it, and the trodden path up to the gate. The
/// strawberries and the thyme are painted after.
fn bank(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let top = hedge_foot(x).min(118);
        let foot = lane_top(x);
        for y in top..foot {
            let down = (y - top) as f32 / (foot - top).max(1) as f32;
            // Earth shows where the bank is steepest, lower down.
            let bare = patches(x, y, (9, 5), 591) * 0.7 + down * 0.5 > 0.98;
            let color = if bare {
                let level = if patches(x, y, (3, 2), 592) > 0.55 {
                    3
                } else {
                    2
                };
                tone(EARTH, level - i32::from(chance(x, y, 593, 60)))
            } else {
                let light = 0.8 - down * 0.5 + patches(x, y, (6, 3), 594) * 0.5;
                let level = if light > 0.95 {
                    3
                } else if light > 0.55 {
                    2
                } else {
                    1
                };
                let mut color = tone(SWARD, level);
                if chance(x, y, 595, 40) {
                    color = tone(SWARD, level + 1);
                }
                color
            };
            scene.set(x, y, color);
        }
        // The bank's lip over the lane, catching the light, and its shadow on the lane.
        put(scene, x, foot - 1, SWARD.shadow);
        put(scene, x, foot - 2, mix(SWARD.base, SWARD.light, 0.5));
    }
    // Stones in the bank.
    for &(x, y, r) in &[(98, 138, 3), (262, 134, 2), (308, 140, 3), (164, 134, 2)] {
        stone(scene, (x, y), r);
    }
    // Sunny stones where the thyme grows.
    for &(x, y) in &[(201, 131), (214, 133), (227, 129), (240, 132)] {
        stone(scene, (x, y), 2);
    }
    gate_path(scene);
    // Flowers along the bank: red campion and stitchwort, and grass tufts.
    for index in 0..60 {
        let x = pick(index, 0, 596, WIDTH - 60) + 4;
        if (96..160).contains(&x) || (196..244).contains(&x) || (172..204).contains(&x) {
            continue;
        }
        let y = hedge_foot(x) + 4 + pick(index, 1, 596, 20);
        if y >= lane_top(x) - 3 {
            continue;
        }
        let tall = 3 + pick(index, 2, 596, 4);
        vline(scene, x, y - tall, tall, SWARD.shadow);
        if index % 3 == 0 {
            put(scene, x, y - tall, CAMPION.light);
            put(scene, x + 1, y - tall, CAMPION.base);
            put(scene, x, y - tall - 1, CAMPION.shine);
        } else if index % 3 == 1 {
            put(scene, x, y - tall, STITCHWORT);
            put(scene, x - 1, y - tall, mix(STITCHWORT, SWARD.light, 0.4));
            put(scene, x + 1, y - tall, mix(STITCHWORT, SWARD.light, 0.4));
            put(scene, x, y - tall - 1, mix(STITCHWORT, SWARD.light, 0.3));
        } else {
            put(scene, x - 1, y - tall + 1, SWARD.light);
            put(scene, x + 1, y - tall + 2, SWARD.base);
        }
    }
}

/// A stone half sunk in the bank, rounded and lit from the upper left, edged in its own darkest
/// grey.
fn stone(scene: &mut Canvas, (x, y): (i32, i32), r: i32) {
    for dy in -r..=0 {
        for dx in -r - 1..=r + 1 {
            let (u, v) = (dx as f32 / (r as f32 + 1.2), dy as f32 / (r as f32 + 0.4));
            if u * u + v * v > 1.0 {
                continue;
            }
            let rim = (dx.abs() == r + 1) || dy == -r;
            let lit = -u - v * 1.2;
            let color = if rim && lit < 0.6 {
                ROCK.edge
            } else if lit > 0.9 {
                ROCK.shine
            } else if lit > 0.3 {
                ROCK.light
            } else if lit > -0.3 {
                ROCK.base
            } else {
                ROCK.shadow
            };
            put(scene, x + dx, y + dy, color);
        }
    }
    hline(scene, x - r, y + 1, r * 2 + 1, rgba(0x1c3020, 80));
}

/// The path worn up the bank to the gate: bare earth, paler where it is trodden hardest, its
/// edges softened by grass, a root across it for a step.
fn gate_path(scene: &mut Canvas) {
    for y in 112..LANE_TOP + 2 {
        let t = (y - 112) as f32 / (LANE_TOP - 112) as f32;
        let middle = 190.0 - t * 2.0;
        let half = 10.0 + t * 6.0 + patches(y, 0, (5, 1), 601) * 2.0;
        for x in (middle - half) as i32..=(middle + half) as i32 {
            let off = (x as f32 - middle).abs() / half;
            let worn = patches(x, y, (4, 3), 602) * 0.6 + (1.0 - off) * 0.6;
            if off > 0.85 && chance(x, y, 603, 140) {
                continue;
            }
            let color = if off > 0.85 {
                mix(LANE.shadow, SWARD.base, 0.5)
            } else if worn > 0.75 {
                LANE.light
            } else if worn > 0.45 {
                LANE.base
            } else {
                LANE.shadow
            };
            scene.set(x, y, color);
        }
    }
    // A root across it, worn smooth, for a step.
    for x in 180..200 {
        let y = 128 + (x - 180) / 7;
        put(scene, x, y, BARK.light);
        put(scene, x, y + 1, BARK.base);
        put(scene, x, y + 2, mix(LANE.edge, BARK.edge, 0.5));
    }
}

/// The sunken lane: packed earth, paler where it is dry and darker where it holds the wet, a cart's
/// two ruts wandering along it with grass down the crown between them, grass creeping in at its
/// edges, stones, and a puddle in the far rut.
fn lane(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let (top, foot) = (lane_top(x), lane_foot(x));
        let width = (foot - top).max(1) as f32;
        let far_rut = top as f32 + width * 0.26 + (x as f32 * 0.045).sin() * 1.5;
        let near_rut = top as f32 + width * 0.72 + (x as f32 * 0.038 + 1.3).sin() * 1.5;
        let crown_line = (far_rut + near_rut) / 2.0 + (x as f32 * 0.07).sin();
        for y in top..foot {
            let grain = patches(x, y, (6, 2), 611);
            let level = if grain > 0.72 {
                4
            } else if grain > 0.42 {
                3
            } else {
                2
            };
            let mut color = tone(LANE, level);
            // The ruts: worn darker, with a lit lip on their far side and a dark one near.
            for (index, rut) in [far_rut, near_rut].into_iter().enumerate() {
                let off = y as f32 - rut;
                let broken = patches(x, 7 + index as i32, (7, 1), 612) > 0.72;
                if broken {
                    continue;
                }
                if (-1.0..1.0).contains(&off) {
                    color = mix(tone(RUT, 3), tone(LANE, level), 0.35);
                } else if (1.0..2.0).contains(&off) {
                    color = mix(tone(RUT, 2), color, 0.55);
                } else if (-2.0..-1.0).contains(&off) && chance(x, y, 618, 160) {
                    color = LANE.light;
                }
            }
            // Grass down the crown of the lane, in tufts.
            if (y as f32 - crown_line).abs() < 1.5 && patches(x, 9, (5, 1), 613) > 0.35 {
                color = tone(SWARD, 2 + i32::from(chance(x, y, 614, 90)));
            }
            // The bank's shade along the far side.
            if y - top < 4 {
                color = mix(color, LANE.edge, 0.32 - (y - top) as f32 * 0.07);
            }
            if chance(x, y, 615, 6) {
                color = LANE.shine;
            }
            scene.set(x, y, color);
        }
        // Grass creeping in from the near verge.
        if chance(x, 1, 616, 120) {
            let tall = 1 + pick(x, 2, 616, 3);
            vline(scene, x, foot - tall, tall, SWARD.base);
            put(scene, x, foot - tall, SWARD.light);
        }
    }
    // Stones and pebbles in the lane, kept off where anyone stands to pick.
    for index in 0..30 {
        let x = pick(index, 0, 617, WIDTH);
        let (top, foot) = (lane_top(x), lane_foot(x));
        let y = top + 8 + pick(index, 1, 617, (foot - top - 10).max(1));
        put(scene, x, y, ROCK.light);
        put(scene, x + 1, y, ROCK.base);
        put(scene, x + 1, y + 1, ROCK.edge);
        put(scene, x, y + 1, ROCK.shadow);
    }
    // A puddle in the far rut, the sky in it.
    for y in 152..158 {
        for x in 248..272 {
            let (u, v) = ((x - 260) as f32 / 12.0, (y - 155) as f32 / 2.6);
            if u * u + v * v > 1.0 {
                continue;
            }
            let rim = u * u + v * v > 0.68;
            let color = if rim {
                mix(PUDDLE.edge, RUT.shadow, 0.5)
            } else if v < -0.1 {
                mix(PUDDLE.shine, HIGH_SKY, 0.4)
            } else {
                PUDDLE.light
            };
            scene.set(x, y, color);
        }
    }
    hline(scene, 255, 154, 4, rgb(0xffffff));
    hline(scene, 264, 156, 2, rgb(0xe8f4f4));
}

/// The near verge: grass thick with clover, its round white and pink heads up out of it, and a
/// few dandelions. The four-leaf clover, when there is one, is drawn as it is found.
fn verge(scene: &mut Canvas) {
    for x in 0..WIDTH {
        for y in lane_foot(x)..HEIGHT {
            let near = (y - 186) as f32 / 30.0;
            let light = 0.6 + patches(x, y, (7, 3), 621) * 0.5 - near * 0.3;
            let level = if light > 0.9 {
                3
            } else if light > 0.5 {
                2
            } else {
                1
            };
            let mut color = tone(SWARD, level);
            if chance(x, y, 622, 50) {
                color = tone(SWARD, level + 1);
            }
            scene.set(x, y, color);
        }
    }
    // The clover, thick in the middle of the verge: trefoils lit from the upper left.
    for index in 0..150 {
        let x = 124 + pick(index, 0, 623, 152);
        let y = 191 + pick(index, 1, 623, 22);
        let spread = ((x - 200) as f32 / 76.0).powi(2) + ((y - 201) as f32 / 12.0).powi(2);
        if spread > 1.0 || (x - 200).abs() < 6 && (y - 199).abs() < 6 {
            continue;
        }
        trefoil(scene, (x, y), index);
    }
    for index in 0..16 {
        let x = 132 + pick(index, 0, 624, 136);
        let y = 192 + pick(index, 1, 624, 16);
        if (x - 200).abs() < 6 && (y - 199).abs() < 6 {
            continue;
        }
        let head = if index % 3 == 0 {
            Ramp::new(0x8a8a80, 0xc8c8bc, 0xe8e8de, 0xf8f8f0, 0xffffff)
        } else {
            CLOVER_HEAD
        };
        vline(scene, x, y - 2, 3, CLOVER_LEAF.shadow);
        put(scene, x, y - 3, head.light);
        put(scene, x + 1, y - 3, head.base);
        put(scene, x, y - 4, head.shine);
        put(scene, x + 1, y - 4, head.light);
        put(scene, x - 1, y - 3, head.base);
        put(scene, x + 1, y - 2, head.shadow);
        put(scene, x, y - 2, head.shadow);
    }
    // Dandelions.
    for &(x, y) in &[(40, 196), (96, 204), (262, 198), (290, 208), (18, 208)] {
        vline(scene, x, y - 3, 3, SWARD.shadow);
        put(scene, x, y - 4, rgb(0xf8d040));
        put(scene, x - 1, y - 4, rgb(0xf0b828));
        put(scene, x + 1, y - 4, rgb(0xf0b828));
        put(scene, x, y - 5, rgb(0xffe880));
    }
}

/// A sprig of three clover leaves, each with its pale chevron.
fn trefoil(scene: &mut Canvas, (x, y): (i32, i32), salt: i32) {
    let level = 1 + pick(salt, 2, 625, 3);
    let lit = tone(CLOVER_LEAF, level + 1);
    let mid = tone(CLOVER_LEAF, level);
    let dark = tone(CLOVER_LEAF, level - 1);
    // Top leaflet, left, and right, around the stalk.
    for (dx, dy, color) in [
        (0, -2, lit),
        (1, -2, mid),
        (-2, 0, lit),
        (-1, 0, mid),
        (2, 0, mid),
        (3, 0, dark),
    ] {
        put(scene, x + dx, y + dy, color);
    }
    put(scene, x, y - 1, mid);
    put(scene, x + 1, y - 1, dark);
    put(scene, x - 2, y + 1, dark);
    put(scene, x - 1, y + 1, CLOVER_LEAF.edge);
    put(scene, x + 2, y + 1, CLOVER_LEAF.edge);
    put(scene, x + 3, y + 1, CLOVER_LEAF.edge);
    put(scene, x, y + 1, CLOVER_LEAF.shadow);
    if salt % 3 == 0 {
        put(scene, x, y - 2, mix(lit, rgb(0xd8f4c8), 0.5));
    }
}

/// Under the trees on the right: wild garlic's broad leaves all over the shady bank, the floor of
/// leaf litter and moss with the lane running off into the trees, and a ring of lusher grass where
/// mushrooms come up.
fn under_the_trees(scene: &mut Canvas) {
    let start = WOODS - 36;
    // The floor below the lane on the right, going into the trees.
    for x in start..WIDTH {
        let into = ((x - start) as f32 / 36.0).clamp(0.0, 1.0);
        for y in lane_foot(x)..HEIGHT {
            if patches(x, y, (3, 3), 630) > into * 1.3 {
                continue;
            }
            let grain = patches(x, y, (4, 3), 631);
            let color = if grain > 0.62 {
                tone(MOSS, 2 + i32::from(chance(x, y, 632, 80)))
            } else {
                let level = if grain > 0.4 { 3 } else { 2 };
                tone(LITTER, level - i32::from(chance(x, y, 633, 50)))
            };
            scene.set(x, y, color);
        }
        // The bank, in the trees' shade.
        for y in 108..lane_top(x) {
            let here = scene.get(x, y);
            scene.set(x, y, mix(here, UNDERSTOREY, into * 0.55));
        }
    }
    // The ring in the grass, darker and lusher, where the mushrooms come up.
    for y in 168..198 {
        for x in 320..380 {
            let (u, v) = ((x - 348) as f32 / 22.0, (y - 183) as f32 / 10.0);
            let reach = (u * u + v * v).sqrt();
            if (reach - 1.0).abs() < 0.24 && !chance(x, y, 634, 30) {
                let level = if v < 0.0 { 4 } else { 3 };
                scene.set(x, y, tone(SWARD, level - i32::from(chance(x, y, 636, 90))));
            } else if (reach - 1.0).abs() < 0.32 && chance(x, y, 635, 120) {
                scene.set(x, y, SWARD.shadow);
            }
        }
    }
    // Wild garlic's leaves over the shady bank: broad, glossy, pointed, in clumps, the back ones
    // first.
    let mut clumps: Vec<(i32, i32, i32)> = (0..18)
        .map(|index| {
            let x = WOODS - 4 + pick(index, 0, 636, WIDTH - WOODS);
            let y = 126 + pick(index, 1, 636, 18);
            (x, y, index)
        })
        .filter(|&(x, y, _)| y < lane_top(x) - 1)
        .collect();
    clumps.sort_by_key(|&(_, y, _)| y);
    for (x, y, index) in clumps {
        ellipse(scene, x, y, 5, 1, rgba(0x0c1a12, 90));
        for (angle, length) in [(-2.3, 7.0), (-1.6, 9.0), (-0.85, 7.0)] {
            let angle = angle + (pick(index, 2, 636, 5) - 2) as f32 * 0.08;
            garlic_leaf(scene, (x, y), angle, length);
        }
    }
}

/// A wild garlic leaf from `from` along `angle`: broad in the middle, pointed, lit along its upper
/// side and edged in its own darkest green.
fn garlic_leaf(scene: &mut Canvas, from: (i32, i32), angle: f32, length: f32) {
    let (dx, dy) = (angle.cos(), angle.sin());
    let (nx, ny) = (-dy, dx);
    for step in 0..(length * 2.0) as i32 {
        let t = step as f32 / (length * 2.0);
        let along = t * length;
        let half = (std::f32::consts::PI * t).sin() * 1.6;
        let (cx, cy) = (from.0 as f32 + dx * along, from.1 as f32 + dy * along);
        for side in [-1.0_f32, 0.0, 1.0] {
            let reach = side * half;
            let (x, y) = (
                (cx + nx * reach).round() as i32,
                (cy + ny * reach).round() as i32,
            );
            let color = if side.abs() > 0.5 && half > 0.8 {
                if side * ny < 0.0 {
                    GARLIC.light
                } else {
                    GARLIC.edge
                }
            } else {
                GARLIC.base
            };
            put(scene, x, y, color);
        }
        // Its midrib, paler, and its outline in its own darkest green.
        if (0.15..0.8).contains(&t) {
            put(scene, cx.round() as i32, cy.round() as i32, GARLIC.light);
        }
    }
    let tip = (
        (from.0 as f32 + dx * length).round() as i32,
        (from.1 as f32 + dy * length).round() as i32,
    );
    put(scene, tip.0, tip.1, GARLIC.shine);
}

// ---------------------------------------------------------------------------------------------
// The plants of the hedgerow
// ---------------------------------------------------------------------------------------------

/// The bramble mound's top at a column, ragged; it runs off the left edge.
fn bramble_top(x: i32) -> i32 {
    let lobe =
        |middle: f32, wide: f32, high: f32| high * (-((x as f32 - middle) / wide).powi(2)).exp();
    let u = ((x - 30) as f32 / 44.0).max(-0.8);
    88 + (u.powi(4) * 56.0) as i32 + (patches(x, 0, (4, 1), 641) * 4.0) as i32
        - lobe(10.0, 12.0, 6.0) as i32
        - lobe(40.0, 9.0, 4.0) as i32
}

/// The bramble heaped at the left end of the hedge and spilling down the bank to the lane: a
/// tangle of small leaves, canes arching up out of it and down again, and its white flowers.
/// Its berries are drawn as they ripen.
fn bramble(scene: &mut Canvas) {
    const FOOT: i32 = 144;
    ellipse(scene, 34, FOOT + 1, 40, 3, SHADE);
    let inside = |x: i32, y: i32| (-8..76).contains(&x) && y >= bramble_top(x) && y <= FOOT;
    leafy(scene, (-8, 76, 76, FOOT), inside, BRAMBLE, (3, 642));
    let canes = [
        ((18, 104), (26, 66), (48, 80)),
        ((44, 110), (62, 70), (80, 98)),
        ((6, 124), (-4, 84), (-10, 102)),
        ((30, 136), (54, 104), (76, 132)),
    ];
    for &(from, bend, to) in &canes {
        stem(scene, (from, bend, to), CANE);
        for step in [8, 16, 22] {
            let point = curve(from, bend, to, step as f32 / 28.0);
            leaf(scene, (point.0 - 2, point.1 - 2), SUNLIT_LEAF, 3, true);
            leaf(scene, (point.0 + 2, point.1 - 1), BRAMBLE, 2, false);
        }
    }
    // Flowers, white with a blush of pink.
    for &(x, y) in &[
        (8, 88),
        (40, 84),
        (64, 100),
        (12, 130),
        (56, 122),
        (26, 106),
    ] {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            put(scene, x + dx, y + dy, BLOSSOM);
        }
        put(scene, x + 1, y + 1, mix(BLOSSOM, CAMPION.light, 0.4));
        put(scene, x, y, rgb(0xf4d444));
    }
    // Grass growing up against its foot.
    for x in -2..80 {
        if !chance(x, 0, 643, 140) {
            continue;
        }
        let tall = 2 + pick(x, 1, 643, 4);
        let lean = pick(x, 2, 643, 3) - 1;
        line(
            scene,
            (x, FOOT + 1),
            (x + lean, FOOT + 1 - tall),
            SWARD.base,
        );
        put(scene, x + lean, FOOT + 1 - tall, SWARD.light);
    }
}

/// The dog rose arching out of the hedge over the bank: masses of its small leaves, darker than
/// the hawthorn's, its thorny canes arching out of them, its flowers pale pink with gold middles,
/// and honeysuckle twining through it with its leaves in pairs. Its hips, and the honeysuckle's
/// flowers, are drawn as they ripen.
fn dog_rose(scene: &mut Canvas) {
    for (index, &((cx, cy), (rx, ry))) in [
        ((84, 78), (14, 12)),
        ((100, 92), (13, 12)),
        ((74, 98), (11, 10)),
        ((92, 106), (12, 8)),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (ROSE_LEAF, 0.0),
            651 + index as u32 * 7,
        );
    }
    let cane = Ramp::new(0x2a3a1c, 0x3e5228, 0x566e34, 0x6e8a42, 0x8aa858);
    for &(from, bend, to) in &[
        ((80, 110), (66, 62), (100, 60)),
        ((96, 110), (116, 70), (120, 96)),
        ((72, 110), (56, 84), (64, 102)),
    ] {
        stem(scene, (from, bend, to), cane);
        for step in (3..28).step_by(4) {
            let point = curve(from, bend, to, step as f32 / 28.0);
            put(scene, point.0 + 1, point.1 - 2, CANE.base);
            if step % 8 == 3 {
                leaf(scene, (point.0 - 3, point.1 - 1), ROSE_LEAF, 3, true);
                leaf(scene, (point.0 + 2, point.1), ROSE_LEAF, 2, false);
            }
        }
    }
    for &(x, y) in &[
        (70, 72),
        (104, 66),
        (114, 88),
        (62, 96),
        (90, 82),
        (108, 104),
        (80, 92),
    ] {
        rose(scene, (x, y));
    }
    // Honeysuckle twining through, leaves in pairs along its stem.
    let twine = Ramp::new(0x4a3a2a, 0x5e4a34, 0x76603e, 0x8e7850, 0xa89066);
    let leaves = Ramp::new(0x2a4a2a, 0x3a6636, 0x508446, 0x6ca05a, 0x90bc74);
    for &(from, bend, to) in &[
        ((90, 110), (110, 96), (96, 76)),
        ((98, 96), (116, 92), (106, 88)),
    ] {
        stem(scene, (from, bend, to), twine);
        for step in (3..28).step_by(6) {
            let point = curve(from, bend, to, step as f32 / 28.0);
            leaf(scene, (point.0 - 3, point.1), leaves, 3, true);
            leaf(scene, (point.0 + 2, point.1), leaves, 2, false);
        }
    }
}

/// A dog rose: five pale pink petals and a gold middle.
fn rose(scene: &mut Canvas, (x, y): (i32, i32)) {
    for (dx, dy, level) in [
        (-1, -1, 4),
        (1, -1, 3),
        (-2, 0, 3),
        (2, 0, 2),
        (-1, 1, 2),
        (1, 1, 1),
    ] {
        put(scene, x + dx, y + dy, tone(ROSE, level));
    }
    put(scene, x, y - 1, ROSE.shine);
    put(scene, x - 1, y, ROSE.light);
    put(scene, x + 1, y, ROSE.base);
    put(scene, x, y + 1, ROSE.shadow);
    put(scene, x, y, rgb(0xf0c840));
}

/// The elder standing up out of the hedge: a tall rounded shrub of pale yellow-green leaves, its
/// leaflets in feathery pairs at its edges, on straight pale stems. Its flower heads are drawn as
/// they ripen.
fn elder(scene: &mut Canvas) {
    let stems = Ramp::new(0x4a4232, 0x625a44, 0x7c7458, 0x968e6e, 0xb0a888);
    for &(from, bend, to) in &[
        ((134, 112), (132, 80), (128, 50)),
        ((144, 112), (146, 76), (150, 44)),
    ] {
        stem(scene, (from, bend, to), stems);
    }
    for (index, &((cx, cy), (rx, ry))) in [
        ((140, 58), (16, 16)),
        ((126, 78), (12, 12)),
        ((156, 82), (12, 12)),
        ((142, 96), (14, 10)),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (ELDER_LEAF, 0.0),
            661 + index as u32 * 7,
        );
    }
    // Feathery leaves out at its edge: leaflets in pairs along a stalk.
    for &(x, y, side) in &[
        (122, 56, -1),
        (156, 50, 1),
        (112, 80, -1),
        (168, 78, 1),
        (130, 40, -1),
    ] {
        for pair in 0..3 {
            let (px, py) = (x + side * pair * 2, y + pair);
            put(scene, px, py, stems.base);
            leaf(scene, (px - 2, py - 2), ELDER_LEAF, 3, true);
            leaf(scene, (px + 1, py + 1), ELDER_LEAF, 2, false);
        }
    }
}

/// The hazel in the hedge: many straight stems from its foot, and masses of round leaves, brighter
/// than the hawthorn's, with the biggest of them catching the sun at its edges. Its nuts are drawn
/// as they ripen.
fn hazel(scene: &mut Canvas) {
    let stems = Ramp::new(0x3a2a20, 0x543e2e, 0x6e543e, 0x8a6e52, 0xa68a6a);
    for (index, &((cx, cy), (rx, ry))) in [
        ((246, 62), (16, 14)),
        ((232, 82), (12, 12)),
        ((258, 84), (12, 12)),
        ((246, 98), (14, 10)),
    ]
    .iter()
    .enumerate()
    {
        crown(
            scene,
            (cx, cy),
            (rx, ry),
            (HAZEL_LEAF, 0.0),
            671 + index as u32 * 7,
        );
    }
    for &(from, bend, to) in &[
        ((238, 114), (234, 96), (232, 80)),
        ((246, 114), (246, 100), (248, 86)),
        ((254, 114), (258, 102), (260, 92)),
    ] {
        stem(scene, (from, bend, to), stems);
    }
    for &(x, y) in &[
        (232, 52),
        (252, 50),
        (220, 74),
        (268, 76),
        (226, 96),
        (264, 96),
        (240, 72),
    ] {
        ellipse(scene, x, y, 3, 2, HAZEL_LEAF.edge);
        ellipse(scene, x, y, 2, 1, HAZEL_LEAF.light);
        put(scene, x - 1, y - 1, HAZEL_LEAF.shine);
        put(scene, x + 1, y + 1, HAZEL_LEAF.base);
        put(scene, x + 2, y, HAZEL_LEAF.shadow);
    }
}

/// Wild strawberry plants along the bank: their leaves in threes, each leaflet toothed and lit
/// from the upper left, and white flowers here and there. Their berries are drawn as they ripen.
fn strawberries(scene: &mut Canvas) {
    for index in 0..16 {
        let x = 102 + pick(index, 0, 681, 54);
        let y = 124 + pick(index, 1, 681, 18);
        if y >= lane_top(x) - 2 {
            continue;
        }
        vline(scene, x, y - 2, 4, STRAWBERRY_LEAF.shadow);
        for (dx, dy) in [(-3, -3), (0, -5), (3, -3)] {
            let (lx, ly) = (x + dx - 1, y + dy);
            put(scene, lx, ly, STRAWBERRY_LEAF.light);
            put(scene, lx + 1, ly, STRAWBERRY_LEAF.light);
            put(scene, lx + 2, ly, STRAWBERRY_LEAF.base);
            put(scene, lx, ly + 1, STRAWBERRY_LEAF.base);
            put(scene, lx + 1, ly + 1, STRAWBERRY_LEAF.base);
            put(scene, lx + 2, ly + 1, STRAWBERRY_LEAF.shadow);
            put(scene, lx, ly - 1, STRAWBERRY_LEAF.edge);
            put(scene, lx + 2, ly - 1, STRAWBERRY_LEAF.edge);
            hline(scene, lx, ly + 2, 3, STRAWBERRY_LEAF.edge);
            put(scene, lx + 1, ly, STRAWBERRY_LEAF.shine);
        }
        if index % 4 == 0 {
            let (fx, fy) = (x + 4, y - 7);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                put(scene, fx + dx, fy + dy, STITCHWORT);
            }
            put(scene, fx, fy, rgb(0xf0d050));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// In front of everyone
// ---------------------------------------------------------------------------------------------

/// Long grass along the bottom corners, drawn over everyone: low, and nothing in the middle of the
/// bottom edge.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for layer in 0..2 {
        for x in (0..100).chain(WIDTH - 90..WIDTH) {
            let corner = if x < 100 {
                1.0 - x as f32 / 100.0
            } else {
                (x - (WIDTH - 90)) as f32 / 90.0
            };
            if !chance(x, layer, 691, 50 + (corner * 160.0) as u32) {
                continue;
            }
            let tall = (2 + (corner * 9.0) as i32 + pick(x, layer + 1, 691, 3)).min(12);
            let lean = pick(x, layer + 2, 691, 7) - 3;
            let ramp = if x > WIDTH / 2 { FERN } else { SWARD };
            let (body, tip) = match (layer, pick(x, layer + 3, 691, 3)) {
                (0, 0) => (ramp.edge, ramp.shadow),
                (0, _) => (mix(ramp.edge, ramp.shadow, 0.5), ramp.base),
                (_, 1) => (ramp.shadow, ramp.light),
                _ => (ramp.base, ramp.shine),
            };
            let mut previous = (x, HEIGHT);
            for step in 1..=tall {
                let t = step as f32 / tall as f32;
                let point = (x + (lean as f32 * t * t).round() as i32, HEIGHT - step);
                line(
                    &mut scene,
                    previous,
                    point,
                    if t > 0.8 { tip } else { body },
                );
                previous = point;
            }
        }
    }
    scene
}

// ---------------------------------------------------------------------------------------------
// After dark
// ---------------------------------------------------------------------------------------------

/// Where the glow-worms shine after dark: low on the bank, in the grass at the hedge's foot.
const GLOW_WORMS: [(i32, i32); 7] = [
    (82, 128),
    (166, 140),
    (232, 136),
    (258, 120),
    (12, 150),
    (306, 126),
    (114, 140),
];

/// The lantern on the gate lit, glow-worms on the bank, and fireflies under the trees.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let (lx, ly) = LANTERN;
    crate::daylight::glow(&mut lights, (lx, ly), 16, rgba(0xffd77a, 200));
    for (dx, dy) in [(0, 0), (0, 1), (-1, 0), (1, 1)] {
        put(&mut lights, lx + dx, ly + dy, rgb(0xfff0bd));
    }
    for &(x, y) in &GLOW_WORMS {
        crate::daylight::glow(&mut lights, (x, y), 5, rgb(0xb8f07a));
        put(&mut lights, x, y, rgb(0xeaffc0));
    }
    crate::daylight::fireflies(
        &mut lights,
        &[(320, 80, 60, 50, 9), (300, 150, 80, 30, 5)],
        697,
    );
    lights
}

/// Where the moon rises after dark: in open sky over the left of the hedge.
const MOON: (i32, i32) = (128, 14);

/// The sky after dark, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, SKY_FOOT, Some(MOON))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hedgerow::{PATCHES, PLACES, Reach, walkable};
    use crate::hilltop::SPOTS;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(
            backdrop(&Arrangement::new())
                .pixels()
                .iter()
                .all(|pixel| pixel.a == 255)
        );
    }

    #[test]
    fn the_hedgerow_is_the_same_every_time() {
        assert_eq!(backdrop(&Arrangement::new()), backdrop(&Arrangement::new()));
        assert_eq!(foreground(), foreground());
        assert_eq!(lamplight(), lamplight());
    }

    #[test]
    fn the_foreground_keeps_low_and_off_where_anyone_stands() {
        let front = foreground();
        let (_, top, _, _) = front.alpha_bounds().expect("no long grass at all");
        assert!(top as i32 >= HEIGHT - 14, "the grass reaches up to {top}");
        let stands = PATCHES
            .iter()
            .map(|patch| (patch.name, patch.stand))
            .chain(PLACES.iter().copied());
        for (name, (x, y)) in stands {
            for dy in -30..=0 {
                for dx in -8..=8 {
                    let (px, py) = (x as i32 + dx, y as i32 + dy);
                    assert!(
                        front.get(px, py).a < 128,
                        "grass covers {name} at {px}, {py}"
                    );
                }
            }
        }
    }

    #[test]
    fn whatever_stands_on_the_hilltop_shows_through_the_gate() {
        for find in crate::finds::CATALOGUE.iter().chain(&crate::finds::FORAGED) {
            let everywhere: Arrangement = (0..SPOTS.len() as u8)
                .map(|spot| (spot, find.id.to_owned()))
                .collect();
            let mut view = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
            skyline(&mut view, &everywhere, &VISTA);
            let scene = backdrop(&everywhere);
            let shown = (0..HEIGHT)
                .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
                .filter(|&(x, y)| view.get(x, y).a > 0 && scene.get(x, y) == view.get(x, y))
                .count();
            assert!(shown > 0, "{} can't be seen through the gate", find.id);
        }
    }

    #[test]
    fn the_old_tree_stands_on_the_hill_as_drawn() {
        assert!((VISTA.tree.1 - hill_crest(VISTA.tree.0)).abs() < 0.5);
    }

    #[test]
    fn everything_at_a_patch_has_something_to_grow_from_or_on() {
        let scene = backdrop(&Arrangement::new());
        for patch in &PATCHES {
            for slot in patch.slots {
                let (x, y) = (slot.at.0 as i32, slot.at.1 as i32);
                let sky = {
                    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
                    sky(&mut only);
                    only.get(x, y)
                };
                assert_ne!(
                    scene.get(x, y),
                    sky,
                    "{:?} at {x}, {y} in {} hangs in the open sky",
                    slot.plant,
                    patch.name
                );
                if slot.reach != Reach::High {
                    assert!(y < HEIGHT, "{:?} is off the bottom", slot.plant);
                }
            }
            assert!(walkable(patch.stand.0, patch.stand.1));
        }
    }

    #[test]
    fn the_lantern_and_the_glow_worms_shine_out_of_the_sky() {
        let lights = lamplight();
        for y in 0..40 {
            for x in 0..WOODS - 10 {
                assert_eq!(lights.get(x, y).a, 0, "a light in the sky at {x}, {y}");
            }
        }
        assert!(
            lights.get(LANTERN.0, LANTERN.1).a > 0,
            "the lantern is dark"
        );
    }

    #[test]
    fn the_moon_rises_in_open_sky() {
        let night = night_sky(&backdrop(&Arrangement::new()));
        assert_eq!(night.get(MOON.0, MOON.1).a, 255, "the moon is hidden");
        for y in SKY_FOOT + 30..HEIGHT {
            for x in 0..WIDTH {
                assert_eq!(night.get(x, y).a, 0, "night sky on the ground at {x}, {y}");
            }
        }
    }
}
