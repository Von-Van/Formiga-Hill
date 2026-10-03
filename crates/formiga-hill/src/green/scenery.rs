//! The Village Green, seen from above and a little to one side, as if from a branch of the oak:
//! the lawn fills the picture, a treeline and hedge close it in along the top, and through a gap
//! in them the path climbs away towards the Hill. Unlike the station's level view, everything
//! here is laid out on the ground: the oak and its swing, the well, the toy chest, the blanket.
//! A strip of long grass along the bottom is painted separately, to be drawn in front of everyone.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::kit::{bush, roof};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba, vline,
};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

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
/// The gap in the trees where the path leaves, its left and right edges.
const GAP: (i32, i32) = (282, 336);

const BARK: Ramp = Ramp::new(0x3a2a20, 0x5a4232, 0x74583f, 0x8c6e50, 0xa48562);
const CANOPY: Ramp = Ramp::new(0x24452b, 0x335e3b, 0x447a4a, 0x5f9a5b, 0x86bd73);
const WOODS: Ramp = Ramp::new(0x1c3424, 0x284a33, 0x355f40, 0x4b7b52, 0x6a9a69);
/// The yellower green of a beech, and the deep blue-green of a yew, so the woods along the top
/// are not all one tree.
const BEECH: Ramp = Ramp::new(0x2b4526, 0x3e6230, 0x557e3c, 0x729c4a, 0x98bc66);
const YEW: Ramp = Ramp::new(0x16302a, 0x1f4236, 0x2b5646, 0x3c6e58, 0x588a6e);
const KINDS: [Ramp; 4] = [WOODS, CANOPY, BEECH, YEW];
/// The shade under the trees, deepest at the foot of the wood.
const UNDERSTOREY: Rgba = rgb(0x172a21);
const WICKER: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xbf9650, 0xd8b26a, 0xead08e);
const LAWN_LIGHT: Rgba = rgb(0x8cc178);
const LAWN_DARK: Rgba = rgb(0x7fb46d);
const CLOVER: Ramp = Ramp::new(0x3f7442, 0x4f8a4e, 0x63a05c, 0x78b26a, 0xf4efe6);
const GRAVEL: Ramp = Ramp::new(0x9c8a6a, 0xc4b08a, 0xdcc9a0, 0xe9dab6, 0xf4ead0);
const CHECK_RED: Rgba = rgb(0xd0574a);
const ROPE: Rgba = rgb(0xcbb38a);
const BRASS_LIKE: Rgba = rgb(0xc9a14e);
const SHADOW: Rgba = rgba(0x24452b, 64);

/// The Hill's turf, and its old tree's leaves and bark, in the Hilltop's own tones.
const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const OLD_BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
/// Worn earth in the shade of a path's lower edge.
const EARTH_SHADE: Rgba = rgb(0x8e7553);
/// The air between the green and anything far off: the Hill, and the farthest of the woods.
const DISTANCE: Rgba = rgb(0xb8d6d2);
/// How far the Hill is lost in it.
const HILL_HAZE: f32 = 0.15;

/// The middle of the Hill across the gap, the row its summit reaches, and how far its dome runs
/// either way and down to its foot.
const HILL_MIDDLE: f32 = 311.0;
const HILL_TOP: f32 = 14.0;
const HILL_REACH: (f32, f32) = (31.0, 14.0);

/// The Hill's skyline across the gap: a broad summit, falling away to either side into the trees.
fn hill_crest(x: f32) -> f32 {
    let across = ((x - HILL_MIDDLE) / HILL_REACH.0).clamp(-1.0, 1.0);
    HILL_TOP + HILL_REACH.1 * (1.0 - (1.0 - across * across).sqrt())
}

/// The Hill through the gap in the trees, with whatever stands on it: the old tree on its left
/// shoulder, and the summit running right from it across the top of the dome.
const VISTA: Vista = Vista {
    tree: (296.0, 15.75),
    crest: hill_crest,
    spread: 9.4,
    shrink: 7,
    tint: Tint::Haze(DISTANCE, 0.15),
};

pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    beyond_the_gap(&mut scene);
    skyline(&mut scene, hilltop, &VISTA);
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
// Tools
// ---------------------------------------------------------------------------------------------

/// Smooth noise in 0..1 that changes over cells of `size`: broad patches rather than speckle.
/// Cells are found with `div_euclid`, so they keep their size either side of zero.
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

// ---------------------------------------------------------------------------------------------
// Beyond the gap
// ---------------------------------------------------------------------------------------------

/// The row where the green's lawn begins at `x`: its far brow across the gap, beyond which the
/// ground drops out of sight and the Hill rises. Everywhere else it is under the hedge.
fn far_brow(x: i32) -> i32 {
    let across = ((x as f32 - 309.0) / 27.0).abs().min(1.0);
    LAWN_TOP + 2 + (across * across * 2.0).round() as i32
}

/// What shows through the gap in the trees: sky with a cloud in it, and the Hill some way off,
/// its path winding up to the old tree on its summit.
fn beyond_the_gap(scene: &mut Canvas) {
    sky(scene);
    downs(scene);
    hill(scene);
    hill_path(scene);
    old_tree(scene);
}

/// The sky, in bands, paling towards the horizon behind the Hill, and a cloud.
fn sky(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for x in 0..width {
        for y in 0..far_brow(x) {
            let band = mix(SKY_TOP, SKY_LOW, (y * 5 / 26).min(4) as f32 / 7.0);
            scene.set(x, y, band);
        }
    }
    cloud(scene, (324, 5));
}

/// A small fair-weather cloud, lit from above and greying a little underneath.
fn cloud(scene: &mut Canvas, (cx, cy): (i32, i32)) {
    let under = mix(CLOUD, SKY_TOP, 0.45);
    for &(dx, dy, rx, ry) in &[(0, 1, 7, 2), (-3, 0, 3, 2), (2, -1, 4, 2), (6, 1, 3, 1)] {
        ellipse(scene, cx + dx, cy + dy + 1, rx, ry, under);
    }
    for &(dx, dy, rx, ry) in &[(0, 0, 6, 2), (-3, -1, 3, 2), (2, -2, 4, 2), (6, 0, 2, 1)] {
        ellipse(scene, cx + dx, cy + dy, rx, ry, CLOUD);
    }
}

/// `color` as the Hill's distance leaves it.
fn far(color: Rgba) -> Rgba {
    mix(color, DISTANCE, HILL_HAZE)
}

/// Downs farther off behind the Hill, pale in the haze, showing either side of it.
fn downs(scene: &mut Canvas) {
    for x in GAP.0 - 12..GAP.1 + 12 {
        let ridge =
            (20.0 + (x as f32 * 0.09).sin() * 1.5 + (x as f32 * 0.23).sin() * 0.7).round() as i32;
        for y in ridge..far_brow(x) {
            let color = if y == ridge {
                mix(FAR_HILL, TURF.shadow, 0.3)
            } else if (y - ridge) < 2 && x < HILL_MIDDLE as i32 {
                mix(FAR_HILL, TURF.shine, 0.3)
            } else {
                FAR_HILL
            };
            scene.set(x, y, mix(color, DISTANCE, 0.35));
        }
    }
}

/// The Hill: a broad dome of turf lit on its left flank and shaded on its right, outlined along
/// its crest, with a hedgerow tree or two on its lower slopes and its foot in the dip below the
/// green's brow.
fn hill(scene: &mut Canvas) {
    for x in GAP.0 - 12..GAP.1 + 12 {
        let across = (x as f32 + 0.5 - HILL_MIDDLE) / HILL_REACH.0;
        if across.abs() >= 1.0 {
            continue;
        }
        let crest = hill_crest(x as f32 + 0.5).round() as i32;
        let brow = far_brow(x);
        for y in crest..brow {
            let down = (y - crest) as f32;
            // Lit from the upper left: the shade turns down the right flank, ragged at its edge.
            let dither = (noise(x, y, 451) % 100) as f32 / 100.0 * 0.16;
            let light = across * 1.2 - down * 0.02 + dither;
            let mut color = if light < -0.35 && down < 4.0 {
                TURF.light
            } else if light < 0.32 {
                TURF.base
            } else if light < 0.7 {
                TURF.shadow
            } else {
                mix(TURF.shadow, TURF.edge, 0.5)
            };
            // Grass in tussocks, at two-pixel grain.
            match noise(x.div_euclid(2), y, 452) % 23 {
                0 => color = mix(color, TURF.edge, 0.35),
                1 => color = mix(color, TURF.shine, 0.35),
                _ => {}
            }
            // The foot of the Hill, down in the dip below the brow, in the green's own shadow.
            if y >= brow - 2 {
                color = mix(color, TURF.edge, if y == brow - 1 { 0.45 } else { 0.25 });
            }
            scene.set(x, y, far(color));
        }
        // The crest outlined against the sky in the turf's own darkest tone, softer where lit.
        let edge = if across < -0.2 {
            mix(TURF.shadow, TURF.edge, 0.3)
        } else {
            TURF.edge
        };
        put(scene, x, crest, far(edge));
    }
    // A hedgerow tree on either flank, small and far off, for the size of it.
    for (cx, cy) in [(286, 22), (331, 21)] {
        ellipse(scene, cx + 1, cy + 2, 2, 1, far(TURF.edge));
        ellipse(scene, cx, cy, 2, 2, far(CROWN.shadow));
        put(scene, cx - 1, cy - 1, far(CROWN.light));
        put(scene, cx, cy - 1, far(CROWN.base));
        put(scene, cx + 1, cy + 1, far(CROWN.edge));
    }
}

/// The path up the Hill: out of the dip below the green, winding across the face of the Hill,
/// narrowing as it climbs, to the summit by the old tree.
fn hill_path(scene: &mut Canvas) {
    let tree = VISTA.tree.0.round() as i32;
    let summit = (tree - 2, hill_crest(tree as f32 - 1.5).round() as i32 + 1);
    let bends = [
        (306, far_brow(306) - 1),
        (297, 23),
        (315, 20),
        (302, 18),
        summit,
    ];
    for (index, pair) in bends.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        // Two pixels wide on the lower bends, one up on the summit.
        let wide = index < 2;
        line(scene, from, to, far(PATH));
        if wide {
            line(scene, (from.0, from.1 + 1), (to.0, to.1 + 1), far(PATH));
            line(
                scene,
                (from.0, from.1 + 2),
                (to.0, to.1 + 2),
                far(mix(TURF.shadow, EARTH_SHADE, 0.4)),
            );
        } else {
            line(
                scene,
                (from.0, from.1 + 1),
                (to.0, to.1 + 1),
                far(mix(TURF.shadow, EARTH_SHADE, 0.3)),
            );
        }
    }
}

/// The old tree's crown as overlapping masses `(x, y, radius)` from the foot of its trunk, as
/// the Hilltop has it in its own pixels: a great dome with shoulders either side and smaller
/// masses under, a little to the left of the trunk.
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

/// Which of the old tree's crown masses a point (in the Hilltop's pixels, from the trunk's
/// foot) is in, the smallest if several, or `None` outside them all. Each is taken a little
/// fuller than the Hilltop's, so the crown keeps its shape this small.
fn old_tree_mass(hx: f32, hy: f32) -> Option<(f32, f32, f32)> {
    OLD_TREE_CROWN
        .iter()
        .copied()
        .filter(|&(cx, cy, r)| (hx - cx).powi(2) + (hy - cy).powi(2) <= (r + 3.0).powi(2))
        .min_by(|a, b| a.2.total_cmp(&b.2))
}

/// The old tree on the summit, drawn at the same scale as the pieces standing about it: a stout
/// trunk flaring at its foot, and the Hilltop's broad crown lit from the upper left, darker to its
/// lower right, with one outline round it in its own darkest green.
fn old_tree(scene: &mut Canvas) {
    let scale = VISTA.shrink as f32;
    let foot_x = VISTA.tree.0.round() as i32;
    let foot_y = hill_crest(VISTA.tree.0).round() as i32 + 1;
    // In the Hilltop's own pixels, from the trunk's foot.
    let at = |x: i32, y: i32| {
        (
            (x - foot_x) as f32 * scale + scale * 0.5,
            (y - foot_y) as f32 * scale + scale * 0.5,
        )
    };
    // Its shade on the turf, falling away to the right.
    hline(scene, foot_x, foot_y, 5, rgba(0x2c4a2e, 70));
    hline(scene, foot_x + 1, foot_y + 1, 3, rgba(0x2c4a2e, 40));
    // The trunk, two pixels wide, lit on its left and in the crown's shade at the top, flaring
    // into roots at its foot.
    for y in foot_y - 8..=foot_y {
        let shade = if foot_y - y >= 5 { 0.5 } else { 0.0 };
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
    put(
        scene,
        foot_x - 2,
        foot_y - 1,
        far(mix(OLD_BARK.light, TURF.base, 0.5)),
    );
    // The crown, pixel by pixel from the Hilltop's masses.
    let mut layer = Canvas::new(40, 30);
    let origin = (foot_x - 20, foot_y - 26);
    for ly in 0..30 {
        for lx in 0..40 {
            let (x, y) = (origin.0 + lx, origin.1 + ly);
            let (hx, hy) = at(x, y);
            let Some((mx, my, r)) = old_tree_mass(hx, hy) else {
                continue;
            };
            // Lit by where it sits on its own mass, and on the tree as a whole, as the Hilltop
            // lights it (where the trunk's foot stands 53 across and 112 down).
            let local = 0.5 - ((hx - mx) + (hy - my)) / (3.0 * r);
            let whole = 1.0 - (hx + 53.0) / 110.0 * 0.45 - (hy + 112.0) / 75.0 * 0.55;
            let fleck = (noise(x, y, 471) % 100) as f32 / 100.0 * 0.12;
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
    // One outline round it all.
    let mut edges = Vec::new();
    for ly in 0..30 {
        for lx in 0..40 {
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
    for ly in 0..30 {
        for lx in 0..40 {
            let pixel = layer.get(lx, ly);
            if pixel.a > 0 {
                scene.set(origin.0 + lx, origin.1 + ly, far(pixel));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The edges of the green
// ---------------------------------------------------------------------------------------------

/// A rank of the woods along the top: where its crowns sit and how far up or down they wander,
/// how far apart and how big they are, how far off (lost in the distance by `haze`), and whether
/// their trunks show beneath them.
struct Rank {
    row: i32,
    wander: i32,
    spacing: i32,
    radius: (i32, i32),
    haze: f32,
    trunks: bool,
    salt: u32,
}

const RANKS: [Rank; 3] = [
    Rank {
        row: -4,
        wander: 2,
        spacing: 13,
        radius: (8, 10),
        haze: 0.4,
        trunks: false,
        salt: 410,
    },
    Rank {
        row: 2,
        wander: 2,
        spacing: 16,
        radius: (9, 11),
        haze: 0.2,
        trunks: true,
        salt: 440,
    },
    Rank {
        row: 9,
        wander: 3,
        spacing: 27,
        radius: (10, 13),
        haze: 0.0,
        trunks: true,
        salt: 470,
    },
];

/// Woods closing in along the top, a gap where the path leaves, and a flowering hedge in front.
/// The woods go back in ranks: the farthest crowns fading into the distance, nearer ones each in
/// its own green, the nearest standing on trunks in the shade of the wood behind them. Two trees
/// either side of the gap frame the Hill.
fn treeline(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    // The canopy going back into the woods, and the shade under the nearest trees, so only the
    // gap ever shows sky.
    for y in 0..LAWN_TOP + 4 {
        for x in 0..width {
            if x < GAP.0 - 7 || x >= GAP.1 + 7 {
                scene.set(x, y, under_the_trees(x, y));
            }
        }
    }
    // Far trunks, faint in the shade.
    for (index, x) in (-3..width).step_by(8).enumerate() {
        let x = x + pick(index as i32, 0, 401, 5) - 2;
        if x >= GAP.0 - 9 && x < GAP.1 + 9 {
            continue;
        }
        let color = mix(BARK.shadow, UNDERSTOREY, 0.55);
        vline(scene, x, 8, LAWN_TOP + 4 - 8, color);
        if index % 3 == 0 {
            vline(
                scene,
                x + 1,
                8,
                LAWN_TOP + 4 - 8,
                mix(BARK.edge, UNDERSTOREY, 0.5),
            );
        }
    }
    for rank in &RANKS {
        for (index, x) in (-12..width + 12).step_by(rank.spacing as usize).enumerate() {
            let i = index as i32;
            let radius = rank.radius.0 + pick(i, 0, rank.salt, rank.radius.1 - rank.radius.0 + 1);
            let cx = x + pick(i, 1, rank.salt, 7) - 3;
            let cy = rank.row + pick(i, 2, rank.salt, rank.wander * 2 + 1) - rank.wander;
            // The gap is kept clear, and framed by the two trees beside it.
            if cx + radius + 1 >= GAP.0 - 6 && cx - radius - 1 <= GAP.1 + 6 {
                continue;
            }
            let kind = KINDS[pick(i, 3, rank.salt, KINDS.len() as i32) as usize];
            tree(
                scene,
                (cx, cy),
                radius,
                (kind, rank.haze),
                rank.trunks,
                rank.salt + index as u32 * 7,
            );
        }
    }
    tree(scene, (GAP.0 - 14, 8), 12, (CANOPY, 0.0), true, 497);
    tree(scene, (GAP.1 + 13, 10), 13, (BEECH, 0.0), true, 503);
    hedge(scene);
}

/// The wood behind the nearest trees: its canopy going back at the top, and the shade under the
/// trees deepening to their foot, in soft bands.
fn under_the_trees(x: i32, y: i32) -> Rgba {
    let deep = ((y + pick(x, y, 403, 3) - 1) as f32 / 18.0).clamp(0.0, 1.0);
    let canopy = if chance(x.div_euclid(2), y.div_euclid(2), 404, 70) {
        WOODS.base
    } else {
        WOODS.shadow
    };
    let canopy = mix(canopy, DISTANCE, 0.3);
    if deep < 0.4 {
        canopy
    } else if deep < 0.6 {
        mix(WOODS.edge, canopy, 0.4)
    } else {
        mix(UNDERSTOREY, WOODS.edge, 1.0 - deep)
    }
}

/// One tree of the woods along the top: its trunk down into the shade if it shows, and its
/// crown over it.
fn tree(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    radius: i32,
    (kind, haze): (Ramp, f32),
    trunk: bool,
    salt: u32,
) {
    if trunk {
        let width = 2 + radius / 5;
        let left = cx - width / 2 + pick(cx, cy, salt, 3) - 1;
        let foot = LAWN_TOP + 4;
        for y in cy..foot {
            // In the shade of its own crown at the top, a little sun on its lit side lower down.
            let shaded = if y < cy + radius { 0.55 } else { 0.25 };
            for dx in 0..width {
                let tone = if dx == 0 {
                    BARK.light
                } else if dx == width - 1 {
                    BARK.edge
                } else if dx * 2 < width {
                    BARK.base
                } else {
                    BARK.shadow
                };
                let mut color = mix(tone, UNDERSTOREY, shaded);
                if dx > 0
                    && dx < width - 1
                    && noise(left + dx, y.div_euclid(3), salt).is_multiple_of(5)
                {
                    color = mix(color, BARK.edge, 0.5);
                }
                scene.set(left + dx, y, mix(color, UNDERSTOREY, haze * 1.5));
            }
        }
    }
    crown(scene, (cx, cy), radius, (kind, haze), salt);
}

/// A tree's crown seen from above: a round mass of leaves, its rim scalloped and outlined in its
/// own darkest green, filled with clumps of leaves back to front. Each clump has a dark underside
/// and a lit crescent on its upper left, and the clumps are brighter towards the crown's upper left,
/// where the sun catches the top, so the mass reads through the leaves. `haze` fades it all into
/// the distance.
fn crown(
    scene: &mut Canvas,
    (cx, cy): (i32, i32),
    radius: i32,
    (ramp, haze): (Ramp, f32),
    salt: u32,
) {
    let tint = |color: Rgba| mix(color, DISTANCE, haze);
    let ry = (radius * 7 / 8).max(1);
    // The silhouette: a scalloped rim in the edge tone, the body in shade inside it.
    let rim = radius * 3;
    for step in 0..rim {
        let angle = step as f32 / rim as f32 * TAU;
        let reach = radius as f32 - 1.0 + pick(step, 0, salt, 3) as f32;
        let size = 2 + pick(step, 1, salt, 2) + radius / 10;
        let (x, y) = (
            cx + (angle.cos() * reach).round() as i32,
            cy + (angle.sin() * reach * ry as f32 / radius as f32).round() as i32,
        );
        ellipse(scene, x, y, size, size, tint(ramp.edge));
    }
    ellipse(scene, cx, cy, radius, ry, tint(ramp.shadow));
    // Clumps of leaves on a jittered grid, top to bottom so the lower ones overlap the higher.
    let mut clumps = Vec::new();
    for (row, gy) in (cy - ry..=cy + ry + 1).step_by(3).enumerate() {
        let offset = if row % 2 == 0 { 0 } else { 2 };
        for gx in (cx - radius - 1..=cx + radius + 1).step_by(4) {
            let x = gx + offset + pick(gx, gy, salt + 1, 3) - 1;
            let y = gy + pick(gx, gy, salt + 2, 3) - 1;
            let (dx, dy) = ((x - cx) as f32 / radius as f32, (y - cy) as f32 / ry as f32);
            if dx * dx + dy * dy <= 0.92 {
                clumps.push((x, y, dx + dy));
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

/// The flowering hedge along the foot of the woods, ending either side of the gap, and its shade
/// on the lawn.
fn hedge(scene: &mut Canvas) {
    let width = SCENE_WIDTH as i32;
    for (index, x) in (-4..width + 6).step_by(11).enumerate() {
        let i = index as i32;
        let radius = 7 + pick(i, 0, 405, 3);
        let y = LAWN_TOP + 6 + pick(i, 1, 405, 3) - 1;
        if x + radius >= GAP.0 && x - radius <= GAP.1 {
            continue;
        }
        bush(scene, x, y, radius, (2000 + x) as u32);
    }
    // The hedge's ends, rounded off where the path goes through.
    bush(scene, GAP.0 - 5, LAWN_TOP + 7, 7, 2301);
    bush(scene, GAP.1 + 5, LAWN_TOP + 7, 7, 2303);
    for x in 0..width {
        if (GAP.0 - 1..GAP.1 + 1).contains(&x) {
            continue;
        }
        hline(scene, x, LAWN_TOP + 14, 1, rgba(0x24452b, 70));
        hline(scene, x, LAWN_TOP + 15, 1, rgba(0x24452b, 34));
    }
}

// ---------------------------------------------------------------------------------------------
// The ground
// ---------------------------------------------------------------------------------------------

/// Where clover has crept into the lawn, and where daisies have come up, as the mower left them.
const CLOVER_PATCHES: [(i32, i32, i32); 5] = [
    (48, 152, 12),
    (168, 196, 11),
    (204, 122, 10),
    (298, 196, 10),
    (112, 178, 8),
];
const DAISY_DRIFTS: [(i32, i32, i32); 5] = [
    (64, 196, 16),
    (150, 158, 14),
    (262, 116, 14),
    (210, 206, 12),
    (24, 108, 10),
];

fn lawn(scene: &mut Canvas) {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    for x in 0..width {
        for y in far_brow(x)..height {
            scene.set(x, y, mown(x, y));
        }
    }
    // Grass standing up out of the cut here and there, darker at the root and lit at the tip.
    for x in 0..width {
        for y in far_brow(x) + 1..height {
            if chance(x, y, 342, 3) {
                put(scene, x, y, rgba(0x4f7f48, 70));
                put(scene, x, y - 1, LEAF.light);
            } else if chance(x, y, 346, 2) {
                put(scene, x, y, LEAF.base);
                put(scene, x + 1, y, rgba(0x4f7f48, 60));
            }
        }
    }
    // The brow across the gap, the grass along it standing against the dip beyond.
    for x in GAP.0..GAP.1 {
        let brow = far_brow(x);
        if chance(x, brow, 347, 110) {
            put(scene, x, brow - 1, far(GRASS_DARK));
        }
        put(scene, x, brow, mix(LAWN_DARK, GRASS_LIGHT, 0.5));
    }
    for (index, &(cx, cy, reach)) in CLOVER_PATCHES.iter().enumerate() {
        clover(scene, (cx, cy), reach, 360 + index as u32 * 5);
    }
    for (index, &(cx, cy, reach)) in DAISY_DRIFTS.iter().enumerate() {
        daisies(scene, (cx, cy), reach, 380 + index as u32 * 3);
    }
    // And the odd daisy anywhere.
    for y in LAWN_TOP + 16..height {
        for x in 0..width {
            if chance(x.div_euclid(3), y.div_euclid(2), 343, 1) && x % 3 == 0 && y % 2 == 0 {
                daisy(scene, x, y, false);
            }
        }
    }
}

/// The lawn's own colour at a point: mown in diagonal stripes, as a lawn seen from above is,
/// their edges a little ragged where the mower wandered, each stripe its own shade, and greener or
/// paler in broad patches where the grass grows unevenly.
fn mown(x: i32, y: i32) -> Rgba {
    let wander = pick(y.div_euclid(3), x.div_euclid(40), 344, 3) - 1;
    let along = x + y * 2 + wander;
    let stripe = along.div_euclid(30);
    let mut color = if stripe.rem_euclid(2) == 0 {
        LAWN_LIGHT
    } else {
        LAWN_DARK
    };
    // No two passes of the mower quite alike.
    color = match pick(stripe, 0, 348, 4) {
        0 => mix(color, GRASS_LIGHT, 0.18),
        1 => mix(color, rgb(0x78ad6a), 0.25),
        _ => color,
    };
    let patch = patches(x, y, (52, 30), 345) + (noise(x, y, 349) % 100) as f32 / 100.0 * 0.08;
    if patch > 0.66 {
        color = mix(color, GRASS_LIGHT, 0.3);
    } else if patch < 0.3 {
        color = mix(color, GRASS_DARK, 0.28);
    }
    match noise(x, y, 341) % 64 {
        0..=3 => mix(color, GRASS_DARK, 0.6),
        4 | 5 => mix(color, GRASS_LIGHT, 0.7),
        _ => color,
    }
}

/// A patch of clover in the lawn: trefoils of a bluer green, lit on their upper leaf, with a white
/// head or two among them.
fn clover(scene: &mut Canvas, (cx, cy): (i32, i32), reach: i32, salt: u32) {
    let ry = (reach * 3 / 5).max(2);
    let inside = |x: i32, y: i32| {
        let (dx, dy) = ((x - cx) as f32 / reach as f32, (y - cy) as f32 / ry as f32);
        // A ragged edge, as clover creeps rather than grows in rings.
        dx * dx + dy * dy + (noise(x.div_euclid(2), y, salt) % 100) as f32 / 300.0
    };
    // The ground under it a deeper green.
    for y in cy - ry - 1..=cy + ry + 1 {
        for x in cx - reach - 1..=cx + reach + 1 {
            if inside(x, y) <= 1.0 {
                put(scene, x, y, rgba(0x3f7442, 56));
            }
        }
    }
    // Trefoils every few pixels, each row a step out from the last.
    for y in (cy - ry..=cy + ry).step_by(2) {
        let step = (y - cy).div_euclid(2) * 2;
        for x in cx - reach..=cx + reach {
            if (x + step).rem_euclid(4) != 0 || inside(x, y) > 0.95 || chance(x, y, salt + 1, 60) {
                continue;
            }
            put(scene, x, y, CLOVER.shadow);
            put(scene, x - 1, y, CLOVER.base);
            put(scene, x, y - 1, CLOVER.light);
            put(scene, x + 1, y, CLOVER.base);
        }
    }
    for head in 0..2 {
        let x = cx - reach / 2 + pick(head, 0, salt, reach.max(1));
        let y = cy - ry / 2 + pick(head, 1, salt, ry.max(1));
        put(scene, x, y, CLOVER.shine);
        put(scene, x + 1, y, mix(CLOVER.shine, rgb(0xd8a0b8), 0.4));
        put(scene, x, y + 1, mix(CLOVER.shine, rgb(0xd8a0b8), 0.5));
        put(scene, x + 1, y + 1, CLOVER.edge);
    }
}

/// A drift of daisies, thickest in its middle.
fn daisies(scene: &mut Canvas, (cx, cy): (i32, i32), reach: i32, salt: u32) {
    let ry = (reach * 3 / 5).max(2);
    for y in cy - ry..=cy + ry {
        for x in cx - reach..=cx + reach {
            let (dx, dy) = ((x - cx) as f32 / reach as f32, (y - cy) as f32 / ry as f32);
            let r = dx * dx + dy * dy;
            if r > 1.0 || !chance(x, y, salt, (14.0 * (1.0 - r)) as u32 + 1) {
                continue;
            }
            daisy(scene, x, y, chance(x, y, salt + 1, 70));
        }
    }
}

/// A daisy from above: a yellow eye in white petals, those on its lower right in shade; or, if
/// not `open`, just a white bud.
fn daisy(scene: &mut Canvas, x: i32, y: i32, open: bool) {
    let petal = rgb(0xfbf6ee);
    let shaded = rgb(0xd9dbe0);
    if open {
        put(scene, x, y - 1, petal);
        put(scene, x - 1, y, petal);
        put(scene, x + 1, y, shaded);
        put(scene, x, y + 1, shaded);
        put(scene, x, y, rgb(0xf5c94e));
    } else {
        put(scene, x, y, petal);
        put(scene, x + 1, y + 1, rgba(0x3f6a3a, 80));
    }
}

/// The gravel path in from the lane at the bottom corner, up through the gap and over the brow
/// out of sight, to come out again on the Hill.
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
        let brow = far_brow(middle.round() as i32);
        if y < brow {
            continue;
        }
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
            // Going over the brow, the far end of it catches the light.
            let color = if y == brow {
                mix(color, GRAVEL.shine, 0.4)
            } else {
                color
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

// ---------------------------------------------------------------------------------------------
// After dark
// ---------------------------------------------------------------------------------------------

/// Fireflies: along the hedge, round the oak and over the flower bed, each a point of green-gold
/// light with a little glow about it.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    crate::daylight::fireflies(
        &mut lights,
        &[
            (60, 40, 300, 20, 14),
            (40, 70, 130, 60, 10),
            (150, 66, 60, 26, 5),
            (330, 60, 40, 60, 5),
        ],
        900,
    );
    lights
}

/// The sky through the gap after dark, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, 40, Some((328, 13)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(backdrop(&Arrangement::new()), backdrop(&Arrangement::new()));
    }

    #[test]
    fn the_old_tree_stands_on_the_hill_as_drawn() {
        assert!((VISTA.tree.1 - hill_crest(VISTA.tree.0)).abs() < 0.5);
    }

    #[test]
    fn whatever_stands_on_the_hilltop_shows_through_the_gap() {
        for find in crate::finds::CATALOGUE {
            let everywhere: Arrangement = (0..crate::hilltop::SPOTS.len() as u8)
                .map(|spot| (spot, find.id.to_owned()))
                .collect();
            let mut view = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
            skyline(&mut view, &everywhere, &VISTA);
            let (left, top, right, bottom) = view.alpha_bounds().expect("nothing showed");
            assert!(
                left as i32 >= GAP.0 && (right as i32) < GAP.1,
                "{} spills into the trees: {left}..{right}",
                find.id
            );
            assert!(
                top > 0 && (bottom as i32) < far_brow(HILL_MIDDLE as i32),
                "{} is off the summit: {top}..{bottom}",
                find.id
            );
        }
    }

    #[test]
    fn nobody_can_stand_in_the_oak() {
        assert!(!walkable(56.0, 110.0));
        assert!(walkable(56.0, 140.0));
        assert!(walkable(200.0, 120.0));
    }
}
