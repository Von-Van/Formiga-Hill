//! The meadow at the Woods' edge, at the eye level of someone standing in it: a summer sky with
//! fair-weather clouds, the edge of the Woods across the back in ranks going off into the haze,
//! and through a gap in the trees the Hill far off, with its old tree and whatever stands on the
//! Hilltop. In front is the open meadow in full sun: the bramble thicket on the left where the
//! trees give way, the fallen log and the old stump along the back, the wildflowers, the long
//! grass, the boggy pond in its reeds on the right, and the path the colony walks in by, out of
//! the trees on the left.
//!
//! Every perch in `Haunt::perches` has something painted at it, a flower head, a seed head, a
//! leaf, the top of the log or the stump, a reed's head, or open water under a hovering
//! dragonfly, with a little contrast behind it so that a bug a few pixels across settled there
//! shows. It is the glade's palette in sunlight, so the meadow belongs to the same woods.

use super::{Haunt, PLACES, POND};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::kit::{self, cloud};
use crate::materials::{FAR_HILL, PATH, SKY_LOW};
use crate::paint::{
    Ramp, blit, chance, ellipse, hline, line, mix, noise, patches, pick, put, rgb, rgba, tone,
    vline,
};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

// The glade's materials, so the meadow is plainly at the edge of the same woods.
const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const UNDERGROWTH: Ramp = Ramp::new(0x1c3628, 0x284a32, 0x36603c, 0x4c7a48, 0x6a9658);
const HAZEL_LEAF: Ramp = Ramp::new(0x29441f, 0x3a5c2a, 0x4f7834, 0x6a9442, 0x8cb058);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const REED: Ramp = Ramp::new(0x34442a, 0x4c5e36, 0x667a44, 0x829654, 0xa2b06c);
const BULRUSH: Ramp = Ramp::new(0x2a1a12, 0x43291c, 0x5c3a26, 0x765032, 0x8f6640);
const BRAMBLE: Ramp = Ramp::new(0x1b3320, 0x284628, 0x385e32, 0x4e7840, 0x6a9450);
/// A bramble leaf with the sun full on it.
const SUNLIT_LEAF: Ramp = Ramp::new(0x1b3320, 0x3e6834, 0x568640, 0x74a24e, 0x9cc26a);
const CANE: Ramp = Ramp::new(0x3a1e22, 0x56292c, 0x723a36, 0x8c4e42, 0xa66a52);
const BERRY: Ramp = Ramp::new(0x1a0f1e, 0x2b1832, 0x42244a, 0x5e3666, 0x86608a);
const UNRIPE: Ramp = Ramp::new(0x5a1a22, 0x7a2a30, 0x9a3a3a, 0xb85448, 0xd07a62);
const CAP: Ramp = Ramp::new(0x5a3424, 0x83502f, 0xa66e42, 0xc08c5a, 0xd6aa7a);
const TOADSTOOL: Ramp = Ramp::new(0x561c1c, 0x842a26, 0xa83c30, 0xc45a44, 0xd8826a);
const STALK: Ramp = Ramp::new(0x7a6a56, 0xa89878, 0xc4b694, 0xd6caa8, 0xe2d8bc);
const WOOD: Ramp = Ramp::new(0x4e3626, 0x6c4e36, 0x8c6a4a, 0xa88660, 0xbea078);
const POOL: Ramp = Ramp::new(0x1e3a3c, 0x2d5554, 0x41706b, 0x5a8b83, 0x7ea99c);
const LILY: Ramp = Ramp::new(0x24461e, 0x35602a, 0x4c7c34, 0x68983f, 0x8cb658);
const SUN: u32 = 0xffe2a2;

// The meadow's own: the woods' trees with the sun full on them, the sward and its flowers.
const OAK: Ramp = Ramp::new(0x1a3428, 0x264a34, 0x356240, 0x4c7e4c, 0x6c9c5c);
const BEECH: Ramp = Ramp::new(0x22401f, 0x33582a, 0x4a7236, 0x648e42, 0x86ac58);
const HOLLY: Ramp = Ramp::new(0x122a24, 0x1a3a30, 0x24503e, 0x34664c, 0x4e8060);
const HAWTHORN: Ramp = Ramp::new(0x203c22, 0x30562c, 0x447038, 0x5c8c46, 0x7caa58);
const KINDS: [Ramp; 3] = [OAK, BEECH, HOLLY];
const SWARD: Ramp = Ramp::new(0x34502a, 0x4a6c34, 0x618a3e, 0x7ca64a, 0x9cc05e);
const SEED: Ramp = Ramp::new(0x6a5a36, 0x8c7a4a, 0xae9a62, 0xc8b67c, 0xe0d29c);
const DRY_EARTH: Ramp = Ramp::new(0x4a3a28, 0x6a563c, 0x86704e, 0xa08a64, 0xb8a47e);
const WEATHERED: Ramp = Ramp::new(0x544a42, 0x766a5c, 0x948876, 0xb0a48e, 0xc8bea6);
const PLUME: Ramp = Ramp::new(0x6a5444, 0x927a62, 0xb49c80, 0xcfba9c, 0xe6d6bc);
const DAISY: Ramp = Ramp::new(0x8a8c94, 0xc6c8d0, 0xe6e6e8, 0xf6f4ee, 0xffffff);
const KNAPWEED: Ramp = Ramp::new(0x4a2448, 0x6e3468, 0x8e4a86, 0xac66a2, 0xc888bc);
const BUTTERCUP: Ramp = Ramp::new(0x8a6418, 0xc0921e, 0xe2b828, 0xf4d444, 0xfcec8a);
const CAMPION: Ramp = Ramp::new(0x7a2448, 0xa8345e, 0xcc4a74, 0xe06a8e, 0xf096ae);
const CLOVER: Ramp = Ramp::new(0x7a3a52, 0xa4566e, 0xc47a8c, 0xdc9eaa, 0xf0c4c8);
const FOXGLOVE: Ramp = Ramp::new(0x5a2450, 0x82346e, 0xa64a8c, 0xc46aa6, 0xdc92c0);
const BLOSSOM: Rgba = rgb(0xf4ecec);
const ELDER: Rgba = rgb(0xeee6c6);
/// The shade under the trees, deepest at the foot of the wood.
const UNDERSTOREY: Rgba = rgb(0x16281f);
/// The air between the meadow and anything far off, which everything fades towards.
const DISTANCE: Rgba = rgb(0xa8cac4);
/// The sky overhead, deeper than it is low down.
const HIGH_SKY: Rgba = rgb(0xa2d0e8);
const SHADE: Rgba = rgba(0x1c3020, 70);

/// The Hill's turf, and its old tree's leaves and bark, in the Hilltop's own tones.
const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const OLD_BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
const EARTH_SHADE: Rgba = rgb(0x8e7553);
/// How far the Hill is lost in the distance.
const HILL_HAZE: f32 = 0.3;

/// The gap in the trees, through which the Hill shows: its left and right edges.
const GAP: (i32, i32) = (186, 274);
/// The far edge of the meadow, beyond which the land drops away out of sight.
const BROW: i32 = 76;

/// The middle of the Hill across the gap, the row its summit reaches, how far its face falls
/// to where the brow hides it, and how broad its summit is.
const HILL_MIDDLE: f32 = 232.0;
const HILL_TOP: f32 = 45.0;
const HILL_FALL: f32 = 30.0;
const HILL_BREADTH: f32 = 31.0;

/// The Hill's skyline across the gap: a broad flat summit, rounding over into its flanks.
fn hill_crest(x: f32) -> f32 {
    let off = (x - HILL_MIDDLE) / HILL_BREADTH;
    HILL_TOP + HILL_FALL * (1.0 - (-(off * off * off * off)).exp())
}

/// The Hill through the gap, with whatever stands on it: the old tree on the left of the summit,
/// and the summit running right from it.
const VISTA: Vista = Vista {
    tree: (216.0, 47.1),
    crest: hill_crest,
    spread: 9.4,
    shrink: 7,
    tint: Tint::Haze(DISTANCE, HILL_HAZE),
};

/// The lowest row of sky anywhere, at the brow in the gap.
const SKY_FOOT: i32 = BROW;

/// Everything behind the companions and the bugs, filling every pixel.
pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    far_country(&mut scene);
    skyline(&mut scene, hilltop, &VISTA);
    woods(&mut scene);
    meadow(&mut scene);
    woods_edge(&mut scene);
    path(&mut scene);
    pond(&mut scene);
    for haunt in [
        Haunt::Stump,
        Haunt::Log,
        Haunt::Reeds,
        Haunt::Bramble,
        Haunt::Flowers,
        Haunt::Grass,
    ] {
        paint_haunt(&mut scene, haunt);
    }
    scene
}

/// The scenery a haunt is, with something at each of its perches.
fn paint_haunt(scene: &mut Canvas, haunt: Haunt) {
    match haunt {
        Haunt::Flowers => wildflowers(scene),
        Haunt::Grass => long_grass(scene),
        Haunt::Bramble => bramble(scene),
        Haunt::Log => log(scene),
        Haunt::Stump => stump(scene),
        Haunt::Reeds => reeds(scene),
    }
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

/// `color` as seen through `amount` of the distance.
fn hazed(color: Rgba, amount: f32) -> Rgba {
    mix(color, DISTANCE, amount)
}

/// How near the eye a row of the meadow is: 0 at the brow, 1 at the bottom of the picture.
fn nearness(y: i32) -> f32 {
    ((y - BROW) as f32 / (HEIGHT - BROW) as f32).clamp(0.0, 1.0)
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
    centre: (i32, i32),
    radius: i32,
    (ramp, haze): (Ramp, f32),
    salt: u32,
) {
    kit::clump(scene, centre, radius, ramp, salt, |color, _| {
        hazed(color, haze)
    });
}

// ---------------------------------------------------------------------------------------------
// The sky
// ---------------------------------------------------------------------------------------------

/// The sky on its own, clouds and all: deep overhead, paling in bands to a warm haze low down,
/// with a big fair-weather cloud over the trees on the right, a smaller one drifting, and a
/// little one over the Hill.
pub fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 9;
    let band = |y: i32| (y * BANDS / SKY_FOOT).min(BANDS - 1);
    for y in 0..SKY_FOOT {
        for x in 0..WIDTH {
            let mut index = band(y);
            if band(y + 1) != index && (x + y) % 2 == 0 {
                index += 1;
            }
            let color = mix(HIGH_SKY, SKY_LOW, index as f32 / (BANDS - 1) as f32);
            scene.set(x, y, color);
        }
    }
    cloud(
        scene,
        (324, 18),
        &[
            (-22, 2, 6),
            (-11, -3, 8),
            (2, -6, 10),
            (15, -2, 8),
            (26, 2, 5),
        ],
    );
    cloud(scene, (150, 12), &[(-8, 0, 4), (0, -3, 6), (8, -1, 4)]);
    cloud(scene, (252, 28), &[(-5, 0, 3), (1, -2, 4), (7, 0, 2)]);
}

// ---------------------------------------------------------------------------------------------
// Through the gap
// ---------------------------------------------------------------------------------------------

/// `color` as the Hill's distance leaves it.
fn far(color: Rgba) -> Rgba {
    mix(color, DISTANCE, HILL_HAZE)
}

/// What shows through the gap in the trees beyond the brow: downs a long way off, the Hill in
/// front of them, its path winding up to the old tree on its summit, and the tops of woods and
/// hedgerows in the valley between, which is out of sight below the brow.
fn far_country(scene: &mut Canvas) {
    downs(scene);
    hill(scene);
    hill_path(scene);
    old_tree(scene);
    valley(scene);
}

/// Downs far behind the Hill, pale in the haze, showing either side of it.
fn downs(scene: &mut Canvas) {
    for x in GAP.0 - 8..GAP.1 + 8 {
        let ridge =
            (57.0 + (x as f32 * 0.07).sin() * 2.0 + (x as f32 * 0.21).sin() * 0.8).round() as i32;
        for y in ridge..BROW {
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

/// The Hill: a broad dome of turf lit on its left flank and shaded on its right, outlined along
/// its crest, with a hedgerow tree or two on its lower slopes.
fn hill(scene: &mut Canvas) {
    for x in GAP.0 - 8..GAP.1 + 8 {
        let crest = hill_crest(x as f32 + 0.5).round() as i32;
        if crest >= BROW {
            continue;
        }
        let across = (x as f32 + 0.5 - HILL_MIDDLE) / HILL_BREADTH;
        for y in crest..BROW {
            let down = (y - crest) as f32;
            let dither = (noise(x, y, 161) % 100) as f32 / 100.0 * 0.16;
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
            match noise(x.div_euclid(2), y, 162) % 23 {
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
    // Hedgerow trees on its lower slopes, small and far off.
    for (cx, cy) in [(205, 64), (259, 62)] {
        ellipse(scene, cx + 1, cy + 2, 2, 1, far(TURF.edge));
        ellipse(scene, cx, cy, 2, 2, far(CROWN.shadow));
        put(scene, cx - 1, cy - 1, far(CROWN.light));
        put(scene, cx, cy - 1, far(CROWN.base));
        put(scene, cx + 1, cy + 1, far(CROWN.edge));
    }
}

/// The path up the Hill: out of sight below the brow, winding across its face, narrowing as it
/// climbs, to the summit by the old tree.
fn hill_path(scene: &mut Canvas) {
    let tree = VISTA.tree.0.round() as i32;
    let summit = (tree - 2, hill_crest(tree as f32 - 1.5).round() as i32 + 1);
    let bends = [(240, BROW), (228, 64), (242, 57), (226, 51), summit];
    for (index, pair) in bends.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        line(scene, from, to, far(mix(PATH, TURF.base, 0.25)));
        line(
            scene,
            (from.0, from.1 + 1),
            (to.0, to.1 + 1),
            far(mix(
                TURF.shadow,
                EARTH_SHADE,
                if index < 2 { 0.4 } else { 0.25 },
            )),
        );
    }
}

/// The valley between the meadow and the Hill, out of sight below the brow but for the tops of
/// its woods and hedgerow trees, rounded and blue with distance, standing along the Hill's foot.
fn valley(scene: &mut Canvas) {
    for (index, x) in (GAP.0 - 6..GAP.1 + 6).step_by(5).enumerate() {
        let i = index as i32;
        let radius = 3 + pick(i, 0, 181, 3);
        let cx = x + pick(i, 1, 181, 3) - 1;
        let cy = BROW - 2 - pick(i, 2, 181, 3) + i32::from(i % 4 == 0) * 2;
        let ramp = if i % 3 == 0 { BEECH } else { OAK };
        clump(
            scene,
            (cx, cy),
            radius,
            (ramp, 0.55),
            182 + index as u32 * 3,
        );
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
    let at = |x: i32, y: i32| {
        (
            (x - foot_x) as f32 * scale + scale * 0.5,
            (y - foot_y) as f32 * scale + scale * 0.5,
        )
    };
    hline(scene, foot_x, foot_y, 5, rgba(0x2c4a2e, 70));
    hline(scene, foot_x + 1, foot_y + 1, 3, rgba(0x2c4a2e, 40));
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
            let fleck = (noise(x, y, 171) % 100) as f32 / 100.0 * 0.12;
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
// The edge of the woods
// ---------------------------------------------------------------------------------------------

/// Where the foot of the woods meets the meadow at `x`.
fn wood_foot(x: i32) -> i32 {
    BROW + 3 + (patches(x, 0, (13, 1), 201) * 3.0) as i32
}

/// Whether a column is in the gap, its edges a little ragged at `y`.
fn in_gap(x: i32, y: i32) -> bool {
    let ragged = pick(y.div_euclid(2), 0, 202, 3) - 1;
    x >= GAP.0 + ragged && x < GAP.1 - ragged
}

/// A rank of the woods going back from its edge: where its crowns sit and how far up or down
/// they wander, how far apart and how big they are, and how far off they are.
struct Rank {
    row: i32,
    wander: i32,
    spacing: i32,
    radius: (i32, i32),
    haze: f32,
    salt: u32,
}

const RANKS: [Rank; 2] = [
    Rank {
        row: 40,
        wander: 5,
        spacing: 11,
        radius: (8, 11),
        haze: 0.5,
        salt: 210,
    },
    Rank {
        row: 47,
        wander: 4,
        spacing: 15,
        radius: (10, 13),
        haze: 0.28,
        salt: 240,
    },
];

/// An edge tree: the middle of its crown, its reach across and up, and which of `KINDS`.
type EdgeTree = ((i32, i32), (i32, i32), usize);

/// The trees at the very edge of the woods: the middle of each crown, its reach across and up,
/// and which kind it is. Their crowns run together into one wall of leaves, tallest on the left
/// where the colony comes out of the deep woods, and the two either side of the gap frame it.
const EDGE_TREES: [EdgeTree; 13] = [
    ((-8, 16), (24, 24), 2),
    ((30, 24), (21, 21), 0),
    ((64, 18), (23, 23), 1),
    ((98, 32), (18, 18), 0),
    ((126, 38), (16, 16), 2),
    ((152, 34), (17, 17), 1),
    ((177, 28), (18, 19), 0),
    ((285, 26), (19, 20), 1),
    ((316, 38), (16, 16), 0),
    ((344, 30), (19, 19), 2),
    ((374, 22), (20, 21), 1),
    ((402, 30), (18, 18), 0),
    ((112, 44), (12, 12), 1),
];

/// Where the edge trees' trunks show under the canopy, and how wide each is: in the shade
/// between the leaves and the bushes along the foot.
const TRUNKS: [(i32, i32); 11] = [
    (8, 7),
    (36, 5),
    (70, 7),
    (101, 5),
    (131, 4),
    (156, 5),
    (178, 6),
    (283, 6),
    (318, 4),
    (347, 5),
    (370, 6),
];

/// Where the shade under the canopy begins, deepening to the foot of the woods.
const CANOPY_FOOT: i32 = 50;

/// The woods along the back: two ranks of crowns going back into the haze, the edge trees' wall
/// of leaves in front of them, its underside going into the deep shade under the trees, and
/// trunks standing in the shade; all broken by the gap where the Hill shows.
fn woods(scene: &mut Canvas) {
    for rank in &RANKS {
        // Solid shade under the rank's crowns, so no sky shows between them lower down.
        for x in 0..WIDTH {
            for y in rank.row..wood_foot(x) {
                if in_gap(x, y) {
                    continue;
                }
                let color = if chance(x.div_euclid(2), y.div_euclid(2), rank.salt, 50) {
                    OAK.base
                } else {
                    OAK.shadow
                };
                scene.set(x, y, hazed(color, rank.haze));
            }
        }
        for (index, x) in (-12..WIDTH + 12).step_by(rank.spacing as usize).enumerate() {
            let i = index as i32;
            let radius = rank.radius.0 + pick(i, 0, rank.salt, rank.radius.1 - rank.radius.0 + 1);
            let cx = x + pick(i, 1, rank.salt, 7) - 3;
            let cy = rank.row + pick(i, 2, rank.salt, rank.wander * 2 + 1) - rank.wander;
            if cx + radius >= GAP.0 && cx - radius < GAP.1 {
                continue;
            }
            let kind = KINDS[pick(i, 3, rank.salt, KINDS.len() as i32) as usize];
            crown(
                scene,
                (cx, cy),
                (radius, radius * 9 / 10),
                (kind, rank.haze),
                rank.salt + index as u32 * 7,
            );
        }
    }
    // Below their tops, the ranks behind go into the shade under the trees.
    for x in 0..WIDTH {
        for y in 36..CANOPY_FOOT + 4 {
            if in_gap(x, y) {
                continue;
            }
            let into = ((y - 36) as f32 / 16.0 + pick(x, y, 225, 3) as f32 * 0.05).min(1.0);
            scene.set(x, y, mix(scene.get(x, y), UNDERSTOREY, into * 0.7));
        }
    }
    // Each edge tree's crown is a great mass with a shoulder either side, lower down, so the
    // treeline's top is lumpy as real trees' are rather than a row of balls.
    for (index, &((cx, cy), (rx, ry), kind)) in EDGE_TREES.iter().enumerate() {
        let salt = 270 + index as u32 * 9;
        let i = index as i32;
        let lean = pick(i, 0, 271, 5) - 2;
        let masses = [
            (
                (cx - rx / 2 + lean, cy + ry / 4 + pick(i, 1, 271, 4)),
                (rx * 3 / 5, ry * 3 / 5),
            ),
            (
                (cx + rx / 2 + lean, cy + ry / 3 + pick(i, 2, 271, 4)),
                (rx * 3 / 5, ry * 3 / 5),
            ),
            ((cx + lean / 2, cy - ry / 8), (rx * 4 / 5, ry * 4 / 5)),
        ];
        for (mass, &(centre, reach)) in masses.iter().enumerate() {
            crown(scene, centre, reach, (KINDS[kind], 0.0), salt + mass as u32);
        }
    }
    understorey(scene);
    for (index, &(x, width)) in TRUNKS.iter().enumerate() {
        trunk(scene, (x, CANOPY_FOOT + 2), width, 260 + index as u32);
    }
}

/// The underside of the canopy going into shade, and the shade under the trees, deepening
/// towards their foot, with the trunks of the wood behind faint in it.
fn understorey(scene: &mut Canvas) {
    for x in 0..WIDTH {
        let foot = wood_foot(x);
        for y in CANOPY_FOOT..foot {
            if in_gap(x, y) {
                continue;
            }
            // Leaves hang down unevenly from the canopy into the shade.
            let hang = patches(x, 0, (5, 1), 221) * 9.0 + pick(x, 1, 221, 2) as f32;
            let into = (y - CANOPY_FOOT) as f32 - hang;
            if into < 0.0 {
                let darken = ((into + 9.0) / 9.0).clamp(0.0, 1.0) * 0.5;
                scene.set(x, y, mix(scene.get(x, y), UNDERSTOREY, darken));
                continue;
            }
            let deep = (into / (foot - CANOPY_FOOT) as f32 + pick(x, y, 222, 3) as f32 * 0.06)
                .clamp(0.0, 1.0);
            let color = if into < 1.0 {
                mix(OAK.edge, UNDERSTOREY, 0.3)
            } else {
                mix(mix(OAK.edge, UNDERSTOREY, 0.6), UNDERSTOREY, deep)
            };
            scene.set(x, y, color);
        }
    }
    for (index, x) in (-3..WIDTH).step_by(6).enumerate() {
        let x = x + pick(index as i32, 0, 223, 5) - 2;
        let top = CANOPY_FOOT + 6 + pick(index as i32, 1, 223, 5);
        if in_gap(x, top) || in_gap(x + 1, top) {
            continue;
        }
        let color = mix(BARK.shadow, UNDERSTOREY, 0.55);
        for y in top..wood_foot(x) {
            put(scene, x, y, color);
            if index % 3 == 0 {
                put(scene, x + 1, y, mix(BARK.edge, UNDERSTOREY, 0.5));
            }
        }
    }
}

/// One of the edge trees' trunks, from up under the canopy down to the meadow, lit on the left
/// where the sun reaches in under the leaves, its foot flaring and mossy.
fn trunk(scene: &mut Canvas, (left, top): (i32, i32), width: i32, salt: u32) {
    let foot = wood_foot(left + width / 2) + 1;
    for y in top..foot {
        let flare = ((y - (foot - 5)).max(0) * width) / 8;
        let (from, to) = (left - flare, left + width + flare);
        let span = (to - from).max(1);
        // Deep in the canopy's shade at the top, a little sun reaching in lower down.
        let shaded = (0.75 - (y - top) as f32 / 30.0).clamp(0.3, 0.75);
        for x in from..to {
            if in_gap(x, y) {
                continue;
            }
            let across = (x - from) as f32 / span as f32;
            let mut color = if x == from || x == to - 1 {
                BARK.edge
            } else if across < 0.3 {
                BARK.light
            } else if across < 0.62 {
                BARK.base
            } else {
                BARK.shadow
            };
            if x > from && x < to - 1 {
                let streak = noise(x, (y + pick(x, 1, salt, 9)).div_euclid(4), salt);
                if streak.is_multiple_of(5) {
                    color = mix(color, BARK.edge, 0.6);
                }
                if foot - y < 7 && patches(x, y, (3, 4), salt + 1) > 0.45 + across * 0.3 {
                    color = if across < 0.4 {
                        MOSS.light
                    } else {
                        MOSS.shadow
                    };
                }
            }
            scene.set(x, y, mix(color, UNDERSTOREY, shaded));
        }
    }
}

/// A tree's crown from the side: a rounded mass of leaves, its rim scalloped and outlined in its
/// own darkest green, filled with clumps of leaves back to front. Each clump has a dark underside
/// and a lit crescent on its upper left, and the clumps are brighter towards the crown's upper
/// left, where the sun catches it, so the mass reads through the leaves. `haze` fades it all into
/// the distance.
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

/// The bushes along the foot of the woods: where each stands, how big it is, and what it is.
/// Bigger ones stand between the trunks, with gaps here and there where the shade under the
/// trees shows down to the grass.
const BUSHES: [(i32, i32, usize); 22] = [
    (-2, 9, 0),
    (14, 7, 2),
    (48, 10, 1),
    (60, 6, 0),
    (84, 8, 2),
    (96, 6, 1),
    (114, 9, 0),
    (140, 7, 1),
    (150, 5, 2),
    (166, 8, 0),
    (182, 9, 1),
    (278, 9, 0),
    (292, 6, 2),
    (304, 8, 1),
    (326, 10, 2),
    (338, 6, 0),
    (356, 7, 1),
    (368, 9, 0),
    (384, 8, 2),
    (24, 5, 1),
    (128, 5, 0),
    (266, 5, 2),
];

/// The woods' edge where it meets the meadow: bushes in the sun along the foot of the trees,
/// elderflower and dog-roses in them, foxgloves and cow parsley in front, and the trees' shade
/// on the grass. In the gap, a hawthorn or two out on the brow.
fn woods_edge(scene: &mut Canvas) {
    let ramps = [HAZEL_LEAF, HAWTHORN, UNDERGROWTH];
    for (index, &(x, radius, kind)) in BUSHES.iter().enumerate() {
        let foot = wood_foot(x);
        clump(
            scene,
            (x, foot - radius * 2 / 3),
            radius,
            (ramps[kind], 0.0),
            310 + index as u32 * 5,
        );
        match index % 5 {
            1 => elderflower(
                scene,
                (x, foot - radius * 3 / 2),
                radius,
                320 + index as u32,
            ),
            3 => {
                for rose in 0..3 {
                    let rx = x + pick(rose, index as i32, 321, radius) - radius / 2;
                    let ry = foot - radius + pick(index as i32, rose, 322, radius) - 1;
                    put(scene, rx, ry, rgb(0xf2b6c4));
                    put(scene, rx + 1, ry, rgb(0xd88a9c));
                    put(scene, rx, ry - 1, rgb(0xfad8de));
                    put(scene, rx, ry, rgb(0xf6e07a));
                }
            }
            _ => {}
        }
    }
    // A hawthorn or two out on the brow in the gap, small in the distance.
    for (cx, radius) in [(GAP.0 + 9, 4), (GAP.1 - 12, 3)] {
        clump(
            scene,
            (cx, BROW - 1),
            radius,
            (HAWTHORN, 0.3),
            330 + cx as u32,
        );
    }
    // Foxgloves and cow parsley out in front of the bushes.
    for (index, x) in [76, 90, 134, 310, 350, 377].into_iter().enumerate() {
        foxglove(
            scene,
            (x, wood_foot(x) + 3),
            13 + pick(index as i32, 0, 331, 7),
        );
    }
    for (index, x) in [4, 30, 104, 120, 160, 296, 332, 362]
        .into_iter()
        .enumerate()
    {
        cow_parsley(
            scene,
            (x, wood_foot(x) + 3),
            7 + pick(index as i32, 0, 332, 7),
        );
    }
    // The shade of the woods, falling out onto the meadow to the lower right.
    for x in 0..WIDTH {
        if in_gap(x, BROW) {
            continue;
        }
        let foot = wood_foot(x);
        let reach = 4 + pick(x.div_euclid(4), 0, 333, 3);
        for dy in 0..reach {
            if chance(x, foot + dy, 334, 230 - dy as u32 * 30) {
                put(
                    scene,
                    x,
                    foot + dy,
                    rgba(0x1c3020, (70 - dy * 12).max(10) as u8),
                );
            }
        }
    }
}

/// A flat head of elderflower or two on a bush, creamy in the sun.
fn elderflower(scene: &mut Canvas, (cx, top): (i32, i32), radius: i32, salt: u32) {
    for head in 0..2 {
        let x = cx + pick(head, 0, salt, radius * 2) - radius;
        let y = top + 2 + pick(head, 1, salt, radius);
        hline(scene, x - 1, y, 4, ELDER);
        hline(scene, x, y - 1, 2, mix(ELDER, rgb(0xffffff), 0.5));
        hline(scene, x - 1, y + 1, 4, mix(ELDER, HAZEL_LEAF.shadow, 0.5));
    }
}

/// A foxglove: a tall spike of purple bells hanging from one side, buds at its tip.
fn foxglove(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32) {
    vline(scene, x, foot - tall, tall, HAZEL_LEAF.shadow);
    for (dx, dy) in [(-2, -1), (2, -1), (-1, 0), (1, 0)] {
        put(scene, x + dx, foot + dy, HAZEL_LEAF.base);
    }
    put(scene, x - 2, foot - 2, HAZEL_LEAF.light);
    for bell in 0..tall * 2 / 3 / 2 {
        let y = foot - tall / 3 - bell * 2;
        let side = if bell % 2 == 0 { 1 } else { -1 };
        let lit = side < 0;
        put(
            scene,
            x + side,
            y,
            if lit { FOXGLOVE.light } else { FOXGLOVE.base },
        );
        put(scene, x + side, y + 1, FOXGLOVE.shadow);
        put(scene, x + side * 2, y + 1, FOXGLOVE.edge);
    }
    put(scene, x, foot - tall - 1, HAZEL_LEAF.light);
    put(scene, x, foot - tall, FOXGLOVE.shine);
}

/// Cow parsley: a slender stem and a lacy white umbel.
fn cow_parsley(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32) {
    vline(scene, x, foot - tall, tall, HAZEL_LEAF.base);
    put(scene, x - 1, foot - tall / 2, HAZEL_LEAF.light);
    let top = foot - tall;
    hline(scene, x - 2, top, 5, rgb(0xf4f0e2));
    hline(scene, x - 1, top - 1, 3, rgb(0xffffff));
    put(scene, x + 2, top, rgb(0xd6d4c4));
    put(scene, x - 3, top + 1, rgb(0xe8e4d4));
    put(scene, x + 3, top + 1, rgb(0xc8c6b6));
}

// ---------------------------------------------------------------------------------------------
// The meadow
// ---------------------------------------------------------------------------------------------

/// The sward from the brow to the bottom of the picture: grass in the sun, greener and paler in
/// broad patches, with drifts of seeding grass, blades standing up out of it, taller nearer the
/// eye, and wildflowers scattered through it.
fn meadow(scene: &mut Canvas) {
    for x in 0..WIDTH {
        for y in BROW..HEIGHT {
            scene.set(x, y, sward(x, y));
        }
    }
    // Blades standing up out of the sward, each a shade darker than the grass it grows from and
    // lit at its tip: hardly more than a texture far off, longer and further apart nearer.
    let mut row = BROW + 3;
    while row < HEIGHT + 4 {
        let near = nearness(row);
        let (step_x, step_y) = (3 + (near * 3.0) as i32, 2 + (near * 2.0) as i32);
        for (index, gx) in (-2..WIDTH + 2).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 401, step_x);
            let y = row + pick(i, row + 1, 401, step_y);
            if !chance(x, y, 402, 60 + (near * 120.0) as u32) {
                continue;
            }
            let ground = sward(x, y);
            let tall = 1 + (near * 3.4) as i32 + pick(i, row + 2, 401, 2);
            let lean = if tall > 2 {
                pick(i, row + 3, 401, 3) - 1
            } else {
                0
            };
            let dark = 0.25 + near * 0.2;
            line(
                scene,
                (x, y),
                (x + lean, y - tall + 1),
                mix(ground, SWARD.edge, dark),
            );
            if tall > 1 {
                let tip = if pick(i, row + 4, 401, 9) == 0 {
                    SEED.light
                } else {
                    SWARD.shine
                };
                put(scene, x + lean, y - tall + 1, mix(ground, tip, 0.45));
            }
        }
        row += step_y;
    }
    tussocks(scene);
    meadow_flowers(scene);
}

/// Tussocks of coarser grass dotted about the open sward, longer than the grass round them and
/// lit on their left: none where anyone stands, or in the haunts.
fn tussocks(scene: &mut Canvas) {
    for index in 0..60 {
        let x = 8 + pick(index, 0, 441, WIDTH - 16);
        let y = BROW + 16 + pick(index, 1, 441, HEIGHT - BROW - 16);
        let busy = Haunt::ALL.iter().any(|haunt| {
            let (left, top, right, bottom) = haunt.area();
            (left - 8.0..=right + 8.0).contains(&(x as f32))
                && (top - 4.0..=bottom + 8.0).contains(&(y as f32))
        });
        if busy || kept_clear(x, y) || kept_clear(x, y - 4) {
            continue;
        }
        let near = nearness(y);
        let tall = 3 + (near * 5.0) as i32 + pick(index, 2, 441, 3);
        let ground = sward(x, y);
        ellipse(
            scene,
            x + 1,
            y,
            3 + (near * 2.0) as i32,
            1,
            rgba(0x1c3020, 50),
        );
        for blade in -3..=3i32 {
            let long = tall - blade.abs() / 2 + pick(blade, index, 442, 2);
            let lean = blade + pick(blade, index, 443, 3) - 1;
            let color = match blade {
                ..=-2 => mix(ground, SWARD.shine, 0.35),
                -1..=1 => mix(ground, SWARD.edge, 0.3),
                _ => mix(ground, SWARD.edge, 0.5),
            };
            line(scene, (x + blade / 2, y), (x + lean, y - long), color);
        }
        if index % 4 == 0 {
            // A dandelion standing up out of it, or its clock gone to seed.
            let (fx, fy) = (x + 1, y - tall - 2);
            vline(scene, fx, fy + 1, tall, SWARD.shadow);
            if index % 8 == 0 {
                put(scene, fx, fy, BUTTERCUP.light);
                put(scene, fx - 1, fy, BUTTERCUP.shine);
                put(scene, fx + 1, fy, BUTTERCUP.base);
                put(scene, fx, fy - 1, BUTTERCUP.light);
            } else {
                put(scene, fx, fy, DAISY.light);
                put(scene, fx - 1, fy, DAISY.shine);
                put(scene, fx + 1, fy, DAISY.shadow);
                put(scene, fx, fy - 1, DAISY.shine);
                put(scene, fx, fy + 1, DAISY.base);
            }
        }
    }
}

/// The sward's own colour at a point: four tones in broad soft patches, their edges dithered,
/// greener where it grows lush and paler where it has gone to seed, and paling into the
/// distance towards the brow.
fn sward(x: i32, y: i32) -> Rgba {
    let near = nearness(y);
    let lie = patches(x, y, (38, 10), 411) * 0.7
        + patches(x, y, (11, 4), 412) * 0.3
        + (pick(x, y, 413, 5) - 2) as f32 * 0.025;
    let mut color = if lie < 0.34 {
        SWARD.shadow
    } else if lie < 0.48 {
        mix(SWARD.shadow, SWARD.base, 0.5)
    } else if lie < 0.66 {
        SWARD.base
    } else {
        mix(SWARD.base, SWARD.light, 0.55)
    };
    // Drifts of grass gone to seed, paler and warmer.
    if patches(x, y, (56, 14), 414) + (pick(x, y, 415, 3) as f32) * 0.02 > 0.66 {
        color = mix(color, SEED.light, 0.2);
    }
    let haze = (1.0 - near * 4.0).max(0.0) * 0.22;
    mix(color, DISTANCE, haze)
}

/// Whether a point is somewhere someone might stand or walk in, kept clear of anything small
/// that might look like it is underfoot: the named places, and the path in.
fn kept_clear(x: i32, y: i32) -> bool {
    PLACES
        .iter()
        .any(|&(_, (px, py))| (px as i32 - x).abs() < 18 && (py as i32 - y).abs() < 9)
        || path_reach(x, y) < 1.6
}

/// Buttercups, clover, self-heal and the odd daisy dotted through the sward, more of them in
/// drifts, none where anyone stands.
fn meadow_flowers(scene: &mut Canvas) {
    for y in BROW + 4..HEIGHT {
        let near = nearness(y);
        for x in 0..WIDTH {
            let drift = patches(x, y, (40, 12), 421);
            let rate = if drift > 0.7 { 3 } else { 0 };
            if !chance(x, y, 422, rate) || kept_clear(x, y) {
                continue;
            }
            let kind = pick(x, y, 423, 4);
            let (petal, heart) = match kind {
                0 => (BUTTERCUP.light, BUTTERCUP.base),
                1 => (CLOVER.light, CLOVER.base),
                2 => (KNAPWEED.light, KNAPWEED.base),
                _ => (DAISY.light, BUTTERCUP.light),
            };
            let haze = (1.0 - near * 4.0).max(0.0) * 0.25;
            if near < 0.35 {
                put(scene, x, y, hazed(petal, haze));
                continue;
            }
            put(scene, x, y + 1, SWARD.edge);
            put(scene, x - 1, y, hazed(petal, haze));
            put(scene, x + 1, y, hazed(heart, haze));
            put(scene, x, y - 1, hazed(petal, haze));
            put(scene, x, y, hazed(heart, haze));
        }
    }
}

/// The trodden way through the meadow, out of the trees at the left to the middle.
const PATH_LINE: [(f32, f32); 6] = [
    (-8.0, 194.0),
    (36.0, 190.0),
    (80.0, 182.0),
    (124.0, 173.0),
    (166.0, 166.0),
    (206.0, 160.0),
];

/// How far a point is from the middle of the path, in the path's half-widths; large off it.
fn path_reach(x: i32, y: i32) -> f32 {
    let fx = x as f32;
    let Some(pair) = PATH_LINE
        .windows(2)
        .find(|pair| fx >= pair[0].0 && fx <= pair[1].0)
    else {
        return f32::MAX;
    };
    let t = (fx - pair[0].0) / (pair[1].0 - pair[0].0);
    let centre = pair[0].1 + (pair[1].1 - pair[0].1) * t;
    let half = 2.5 + nearness(centre as i32) * 4.0;
    (y as f32 - centre).abs() / half
}

/// The path: bare trodden earth in the middle, flattened pale grass either side, fading out as
/// it comes into the middle of the meadow.
fn path(scene: &mut Canvas) {
    for x in 0..206 {
        let fade = (1.0 - (x as f32 - 150.0).max(0.0) / 56.0).max(0.0);
        for y in 150..HEIGHT {
            let reach = path_reach(x, y) + (noise(x, y, 431) % 100) as f32 / 300.0;
            if reach > 1.35 {
                continue;
            }
            if reach > 0.9 {
                put(scene, x, y, rgba(0xc8c890, (60.0 * fade) as u8));
                continue;
            }
            let color = match noise(x, y, 432) % 11 {
                0 => DRY_EARTH.shadow,
                1 | 2 => DRY_EARTH.light,
                3 => DRY_EARTH.shine,
                _ => DRY_EARTH.base,
            };
            let color = if reach > 0.7 {
                mix(color, SWARD.shadow, 0.4)
            } else {
                color
            };
            put(
                scene,
                x,
                y,
                Rgba::new(color.r, color.g, color.b, (230.0 * fade) as u8),
            );
        }
    }
    // Pebbles in the path, lit on top.
    for index in 0..14 {
        let x = 4 + pick(index, 0, 433, 170);
        let y = 160 + pick(index, 1, 433, 40);
        if path_reach(x, y) > 0.8 {
            continue;
        }
        let color = [ROCK.base, ROCK.light, DRY_EARTH.shadow][pick(index, 2, 433, 3) as usize];
        hline(scene, x, y, 2, color);
        put(scene, x, y - 1, mix(color, rgb(0xe8e0c8), 0.4));
        hline(scene, x, y + 1, 2, rgba(0x2a2418, 80));
    }
}

// ---------------------------------------------------------------------------------------------
// The pond
// ---------------------------------------------------------------------------------------------

/// Whether a point is open water.
fn in_pond(x: i32, y: i32) -> bool {
    let (cx, cy, rx, ry) = POND;
    let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
    u * u + v * v <= 1.0
}

/// The boggy pond: a margin of wet earth and moss, the water dark with the reflections of the
/// reeds along its far side and pale with the sky nearer, ripples, lily pads, and marsh
/// marigolds at its edge.
fn pond(scene: &mut Canvas) {
    let (cx, cy, rx, ry) = POND;
    let (cx, cy, rx, ry) = (cx as i32, cy as i32, rx as i32, ry as i32);
    // The bog round it, soft and dark, ragged at its edge.
    for y in cy - ry - 5..=cy + ry + 5 {
        for x in cx - rx - 7..=cx + rx + 7 {
            let (u, v) = (
                (x - cx) as f32 / (rx as f32 + 6.0),
                (y - cy) as f32 / (ry as f32 + 4.0),
            );
            let reach = u * u + v * v + (noise(x, y, 501) % 100) as f32 / 400.0;
            if reach > 1.0 {
                continue;
            }
            let wet = patches(x, y, (5, 3), 502);
            let color = if wet < 0.35 {
                EARTH.base
            } else if wet < 0.6 {
                MOSS.shadow
            } else {
                MOSS.base
            };
            scene.set(x, y, color);
        }
    }
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            if !in_pond(x, y) {
                continue;
            }
            scene.set(x, y, water(x, y));
        }
    }
    // The near lip of the bank, catching the light, and its shade on the water just below the
    // far bank.
    for x in cx - rx..=cx + rx {
        let mut far_edge = None;
        let mut near_edge = None;
        for y in cy - ry - 1..=cy + ry + 1 {
            if in_pond(x, y) {
                far_edge.get_or_insert(y);
                near_edge = Some(y);
            }
        }
        if let (Some(top), Some(bottom)) = (far_edge, near_edge) {
            put(scene, x, top, rgba(0x14282a, 110));
            put(scene, x, top + 1, rgba(0x14282a, 50));
            put(scene, x, bottom + 1, MOSS.light);
            put(scene, x, bottom + 2, MOSS.base);
            if chance(x, 0, 503, 70) {
                put(scene, x, bottom, rgba(0xc8dcd2, 90));
            }
        }
    }
    // Lily pads out on the water towards the near side, one in flower.
    for (index, &(x, y, half)) in [(306, 140, 4), (322, 146, 3), (350, 144, 4), (362, 138, 3)]
        .iter()
        .enumerate()
    {
        lily_pad(scene, (x, y), half, 510 + index as u32);
    }
    let (fx, fy) = (350, 142);
    put(scene, fx, fy, rgb(0xfdf6f0));
    put(scene, fx - 1, fy, rgb(0xf2e2da));
    put(scene, fx + 1, fy, rgb(0xd8c4c4));
    put(scene, fx, fy - 1, rgb(0xffffff));
    put(scene, fx, fy + 1, rgb(0xf4d25e));
    // Marsh marigolds at the water's edge.
    for (index, &(x, y)) in [(296, 146), (300, 150), (368, 142), (372, 146)]
        .iter()
        .enumerate()
    {
        ellipse(scene, x, y + 1, 2, 1, LILY.shadow);
        put(scene, x - 1, y, LILY.light);
        let flower = (x + pick(index as i32, 0, 520, 3) - 1, y - 2);
        put(scene, flower.0, flower.1, BUTTERCUP.light);
        put(scene, flower.0 + 1, flower.1, BUTTERCUP.base);
        put(scene, flower.0, flower.1 - 1, BUTTERCUP.shine);
    }
}

/// The water at a point: by the far bank, the reeds standing in it upside down and dark; further
/// out, the sky pale in it; nearer, the water's own deeper colour; ripples across it all, and
/// the sun glinting on them on the left.
fn water(x: i32, y: i32) -> Rgba {
    let (cx, cy, _, ry) = POND;
    let v = (y as f32 + 0.5 - cy) / ry;
    let mut level: f32 = if v < -0.55 {
        1.2
    } else if v < 0.0 {
        2.6
    } else if v < 0.5 {
        2.2
    } else {
        1.6
    };
    // The reeds' reflections, broken by ripples.
    if v < -0.3 && reed_column(x) && (y + x / 3) % 3 != 0 {
        level -= 0.8;
    }
    // The sky in it, in broad soft patches.
    if (-0.4..0.4).contains(&v) && patches(x, y, (14, 4), 531) > 0.6 {
        level += 0.6;
    }
    let ripple = patches(x, y, (6, 2), 532);
    if ripple > 0.78 {
        level += 0.6;
    } else if ripple < 0.15 {
        level -= 0.4;
    }
    let low = level.clamp(0.0, 4.0).floor() as i32;
    let mut color = mix(
        tone(POOL, low),
        tone(POOL, low + 1),
        level.clamp(0.0, 4.0) - low as f32,
    );
    // Glints of sun on the ripples, on the left where it comes from.
    if (x as f32) < cx - 10.0 && ripple > 0.82 && chance(x, y, 533, 90) {
        color = mix(color, rgb(SUN), 0.5);
    }
    color
}

/// Whether a reed stands in the water's reflection at a column, along the far side.
fn reed_column(x: i32) -> bool {
    patches(x, 0, (3, 1), 541) > 0.45
}

/// A lily pad floating flat: a disc seen almost edge on, its notch towards the eye, lit on its
/// upper left.
fn lily_pad(scene: &mut Canvas, (cx, cy): (i32, i32), half: i32, salt: u32) {
    ellipse(scene, cx + 1, cy + 1, half, 1, rgba(0x14282a, 90));
    for y in cy - 1..=cy + 1 {
        for x in cx - half..=cx + half {
            let (u, v) = ((x - cx) as f32 / half as f32, (y - cy) as f32 / 1.5);
            if u * u + v * v > 1.0 || (y == cy + 1 && (x - cx - 1).abs() < 1) {
                continue;
            }
            let color = if u + v < -0.5 {
                LILY.light
            } else if u + v > 0.6 {
                LILY.shadow
            } else {
                LILY.base
            };
            put(scene, x, y, color);
        }
    }
    put(scene, cx - half + 1, cy - 1, LILY.shine);
    if chance(cx, cy, salt, 128) {
        put(scene, cx + half, cy, LILY.edge);
    }
}

// ---------------------------------------------------------------------------------------------
// The haunts
// ---------------------------------------------------------------------------------------------

/// The old stump: the middle of its sawn top, how far that reaches either way, and the row its
/// foot meets the grass.
const STUMP_TOP: (i32, i32) = (284, 84);
const STUMP_REACH: (i32, i32) = (15, 4);
const STUMP_FOOT: i32 = 105;

/// The old stump, back right of the log: thick bark furrowed up its sides, lit on the left,
/// flaring into roots at its foot; its sawn top gone silver with weather, ringed and split, moss
/// creeping over its back edge; lichen crusting its front, toadstools at its foot.
fn stump(scene: &mut Canvas) {
    let (cx, top) = STUMP_TOP;
    let (rx, ry) = STUMP_REACH;
    // Its shade on the grass, to the lower right.
    ellipse(scene, cx + 9, STUMP_FOOT + 2, rx + 12, 3, SHADE);
    ellipse(scene, cx + 5, STUMP_FOOT + 1, rx + 4, 2, SHADE);
    // Roots, thick where they leave it and sinking into the grass, behind and either side.
    root(
        scene,
        (cx + rx - 2, STUMP_FOOT - 7),
        (cx + rx + 12, STUMP_FOOT + 3),
        5,
    );
    root(
        scene,
        (cx - rx + 2, STUMP_FOOT - 7),
        (cx - rx - 11, STUMP_FOOT + 2),
        5,
    );
    // The sides, from the front rim of the top down to the roots.
    for x in cx - rx - 4..=cx + rx + 4 {
        let u = (x - cx) as f32 / rx as f32;
        let rim = if u.abs() <= 1.0 {
            top + (ry as f32 * (1.0 - u * u).sqrt()).round() as i32
        } else {
            top
        };
        for y in top..=STUMP_FOOT {
            // Wider at the foot, where the roots go down.
            let flare = ((y - (STUMP_FOOT - 7)).max(0) as f32 * 0.5) as i32;
            let half = rx + flare;
            if (x - cx).abs() > half || y < rim {
                continue;
            }
            let across = (x - cx) as f32 / half as f32;
            let outline = (x - cx).abs() == half;
            let mut level = if across < -0.6 {
                3
            } else if across < 0.1 {
                2
            } else if across < 0.6 {
                1
            } else {
                0
            };
            // Ridges of bark up its sides, broken into plates.
            let ridge = (x + pick(y.div_euclid(5), x.div_euclid(3), 601, 2)).rem_euclid(3);
            if ridge == 0 {
                level -= 1;
            } else if ridge == 1 && chance(x, y.div_euclid(2), 602, 70) {
                level += 1;
            }
            // Just under the rim, in the shade of its own lip; and darker at the foot.
            if y <= rim + 1 || y >= STUMP_FOOT - 1 {
                level -= 1;
            }
            let color = if outline {
                BARK.edge
            } else {
                tone(BARK, level.clamp(0, 4))
            };
            scene.set(x, y, color);
        }
    }
    // A root in front, coming down the near side into the grass.
    root(scene, (cx - 3, STUMP_FOOT - 4), (cx - 7, STUMP_FOOT + 3), 4);
    // Moss up the shady foot, ragged at its top.
    for x in cx + 3..cx + rx + 3 {
        let high = 1 + (patches(x, 0, (3, 1), 607) * 5.0) as i32;
        for y in STUMP_FOOT - high..STUMP_FOOT {
            if scene.get(x, y) == BARK.edge {
                continue;
            }
            let color = if y == STUMP_FOOT - high {
                MOSS.light
            } else if chance(x, y, 608, 90) {
                MOSS.base
            } else {
                MOSS.shadow
            };
            put(scene, x, y, color);
        }
    }
    // A long scar up its front where the bark has split and come away, the wood beneath
    // gone silver and grained, so that something crawling there shows up against it.
    for y in 87..105 {
        let t = (y - 87) as f32 / 18.0;
        let half = (3.4 * (t * std::f32::consts::PI).sin()).round() as i32 + pick(1, y, 603, 2);
        let middle = cx - 1 + (t * 2.0).round() as i32;
        if half < 1 {
            continue;
        }
        for x in middle - half..=middle + half {
            let across = (x - middle) as f32 / half as f32;
            let color = if x == middle - half {
                BARK.edge
            } else if x == middle + half {
                BARK.shadow
            } else if (x + pick(x, y.div_euclid(3), 604, 2)).rem_euclid(3) == 0 {
                WEATHERED.shadow
            } else if across < -0.3 {
                WEATHERED.light
            } else if across > 0.4 {
                WEATHERED.shadow
            } else {
                WEATHERED.base
            };
            scene.set(x, y, color);
        }
    }
    // The sawn top, gone silver, ringed and split, lit as a flat face in the sun.
    for y in top - ry..=top + ry {
        for x in cx - rx..=cx + rx {
            let (u, v) = ((x - cx) as f32 / rx as f32, (y - top) as f32 / ry as f32);
            let r = (u * u + v * v).sqrt();
            if r > 1.0 {
                continue;
            }
            let color = if r > 0.86 {
                // The bark round the rim, its back edge in the light.
                if v < -0.2 {
                    BARK.light
                } else if v < 0.5 {
                    BARK.base
                } else {
                    BARK.edge
                }
            } else if r > 0.76 || (((r * 6.0) as i32) % 2 == 1 && chance(x, y, 605, 210)) {
                WEATHERED.base
            } else if r < 0.18 {
                WEATHERED.shadow
            } else if u + v < -0.5 {
                WEATHERED.shine
            } else {
                WEATHERED.light
            };
            scene.set(x, y, color);
        }
    }
    // Cracks split out from the heart.
    for (dx, dy) in [(10, 2), (-8, 2), (3, -3)] {
        line(scene, (cx, top), (cx + dx, top + dy), WEATHERED.shadow);
    }
    put(scene, cx, top, WEATHERED.edge);
    // Moss creeping over the back of the rim.
    for x in cx - rx + 1..cx - 3 {
        let u = (x - cx) as f32 / rx as f32;
        let back = top - (ry as f32 * (1.0 - u * u).sqrt()).round() as i32;
        let thick = 1 + (patches(x, 0, (3, 1), 606) * 2.4) as i32;
        for dy in 0..thick {
            put(
                scene,
                x,
                back + dy,
                if dy == 0 { MOSS.light } else { MOSS.base },
            );
        }
        if chance(x, 0, 609, 110) {
            put(scene, x, back - 1, MOSS.light);
        }
    }
    // Toadstools at its foot, on the shady side.
    for (x, y, size) in [
        (cx + rx + 6, STUMP_FOOT + 3, 2),
        (cx + rx + 10, STUMP_FOOT + 4, 1),
    ] {
        vline(scene, x, y - size - 1, size + 1, STALK.light);
        hline(scene, x - size, y - size - 2, size * 2 + 1, TOADSTOOL.base);
        put(scene, x - size, y - size - 2, TOADSTOOL.light);
        hline(
            scene,
            x - size + 1,
            y - size - 3,
            (size * 2 - 1).max(1),
            TOADSTOOL.light,
        );
        put(scene, x, y - size - 3, STALK.shine);
        hline(scene, x - size, y - size - 1, size * 2 + 1, TOADSTOOL.edge);
    }
}

/// A root from where it leaves the trunk at `from` to where it sinks into the grass at `to`,
/// `thick` pixels deep at its start and tapering: lit along its top, outlined underneath.
fn root(scene: &mut Canvas, from: (i32, i32), to: (i32, i32), thick: i32) {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = from.0 + ((to.0 - from.0) as f32 * t).round() as i32;
        // Humped a little as it goes, the way roots arch over the ground.
        let y = from.1 + ((to.1 - from.1) as f32 * t * t).round() as i32;
        let deep = ((thick as f32) * (1.0 - t)).round() as i32 + 1;
        for dy in 0..=deep {
            let color = if dy == 0 {
                BARK.light
            } else if dy == deep {
                BARK.edge
            } else if dy * 2 < deep {
                BARK.base
            } else {
                BARK.shadow
            };
            scene.set(x, y + dy, color);
        }
    }
}

/// The fallen log: its ends, how thick it is at each, and the row its underside rests on.
const LOG_ENDS: (i32, i32) = (146, 254);
const LOG_THICK: (f32, f32) = (13.0, 11.0);
const LOG_FOOT: i32 = 109;

/// The top of the log's bark at a column.
fn log_top(x: i32) -> i32 {
    let t = ((x - LOG_ENDS.0) as f32 / (LOG_ENDS.1 - LOG_ENDS.0) as f32).clamp(0.0, 1.0);
    LOG_FOOT - (LOG_THICK.0 + (LOG_THICK.1 - LOG_THICK.0) * t).round() as i32
}

/// The fallen log across the back: a long trunk lying in the grass, round and lit along its
/// top, its bark in ridges and fallen away in places, its butt end sawn and pale on the left and
/// broken off at the right; moss along its top in cushions, one under each perch; fungi on its
/// side, and grass and a fern growing up at its foot.
fn log(scene: &mut Canvas) {
    let (left, right) = LOG_ENDS;
    // Its shade on the grass, deepest right under it.
    for x in left - 2..right + 14 {
        for dy in 0..4 {
            let fade = 1.0 - ((x - right).max(0) as f32 / 14.0);
            let alpha = ((90 - dy * 22) as f32 * fade).max(0.0) as u8;
            put(scene, x + dy, LOG_FOOT + dy - 1, rgba(0x1c3020, alpha));
        }
    }
    for x in left + 3..right {
        let from_end = right - 1 - x;
        // Broken off at the right: a jagged end, highest at the bottom.
        let jag = [7, 3, 5, 1, 2].get(from_end as usize).copied().unwrap_or(0);
        let top = log_top(x);
        let span = (LOG_FOOT - top) as f32;
        let waver = ((x as f32 * 0.19).sin() * 1.3 + (x as f32 * 0.06).sin()).round() as i32;
        // Where the bark has fallen away, the wood shows pale beneath it.
        let bare = patches(x, 0, (9, 1), 707) > 0.78 && x > left + 30;
        for y in top + jag..=LOG_FOOT {
            let v = (y - top) as f32 / span;
            let mut level = if v < 0.1 {
                2
            } else if v < 0.38 {
                3
            } else if v < 0.6 {
                2
            } else if v < 0.84 {
                1
            } else {
                0
            };
            // Brighter towards the butt end, which the light comes from, and the odd sunlit
            // fleck along the top of a ridge.
            if (x < left + 20 && v < 0.5 && chance(x, y, 708, 120))
                || (level == 3 && chance(x, y, 702, 14))
            {
                level += 1;
            }
            // Furrows running along it between the ridges of bark, long and wavering.
            let groove = (y - top + waver).rem_euclid(4);
            if groove == 0 && v > 0.15 && patches(x, y, (7, 1), 701) > 0.3 {
                level -= 1;
            }
            // A crack across the bark between two plates, now and then.
            if pick(x, 0, 709, 29) == 0 && v > 0.2 && v < 0.8 {
                level -= 1;
            }
            let color = if y == top + jag || y == LOG_FOOT || x == right - 1 {
                BARK.edge
            } else if from_end < 5 && y < top + jag + 2 {
                if y == top + jag + 1 {
                    WOOD.light
                } else {
                    WOOD.base
                }
            } else if bare && (0.22..0.6).contains(&v) {
                tone(WOOD, (level - 1).clamp(1, 4))
            } else {
                tone(BARK, level.clamp(0, 4))
            };
            scene.set(x, y, color);
        }
    }
    // A branch broken off short, sticking up from it near the right-hand end.
    for (step, x) in (243..247).enumerate() {
        let tall = 8 - step as i32 * 2;
        for y in log_top(x) - tall..log_top(x) + 1 {
            let color = match step {
                1 => BARK.light,
                2 => BARK.base,
                _ => BARK.edge,
            };
            scene.set(x, y, color);
        }
        put(scene, x, log_top(x) - tall, WOOD.light);
    }
    // The butt end, sawn across: pale rings, lit from the left, which it faces.
    let (ex, ey) = (left + 3, (log_top(left) + LOG_FOOT) / 2);
    let half = (LOG_FOOT - log_top(left)) / 2;
    for y in ey - half..=ey + half {
        for x in ex - 4..=ex + 3 {
            let (u, v) = ((x - ex) as f32 / 3.5, (y - ey) as f32 / (half as f32 + 0.5));
            let r = (u * u + v * v).sqrt();
            if r > 1.0 {
                continue;
            }
            let color = if r > 0.84 {
                if v < 0.0 { BARK.base } else { BARK.edge }
            } else if ((r * 5.0) as i32) % 2 == 1 {
                WOOD.base
            } else if u + v < -0.4 {
                WOOD.shine
            } else {
                WOOD.light
            };
            scene.set(x, y, color);
        }
    }
    line(scene, (ex, ey), (ex - 2, ey + 3), WOOD.shadow);
    put(scene, ex, ey, WOOD.edge);
    // A skin of moss over its top, and cushions of it: one under each perch, standing up to it,
    // lower ones between, draping down its side here and there. It is outlined against the
    // meadow behind in its own darkest green, as moss and sward are much the same green.
    let mut moss = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for x in left + 6..right - 6 {
        let top = log_top(x);
        if patches(x, 0, (7, 1), 710) > 0.55 {
            moss.set(x, top, MOSS.light);
            moss.set(
                x,
                top + 1,
                if chance(x, 1, 711, 120) {
                    MOSS.base
                } else {
                    MOSS.shadow
                },
            );
        }
    }
    let mut cushions: Vec<(i32, i32, i32)> = Haunt::Log
        .perches()
        .iter()
        .enumerate()
        .map(|(index, &(x, y))| (x as i32, y as i32 + 1, 5 + pick(index as i32, 0, 703, 4)))
        .collect();
    for x in [189, 226] {
        cushions.push((x, log_top(x), 3));
    }
    for &(px, peak, half) in &cushions {
        let base = log_top(px) + 2;
        for x in px - half..=px + half {
            let u = (x - px) as f32 / (half as f32 + 0.5);
            let rise = ((base - peak) as f32 * (1.0 - u * u).max(0.0).sqrt()).round() as i32;
            let surface = base - rise;
            for y in surface..=base {
                let color = if y == surface {
                    if u < 0.1 { MOSS.shine } else { MOSS.light }
                } else if y == base {
                    MOSS.edge
                } else if y == surface + 1 && u < 0.5 {
                    MOSS.light
                } else if u > 0.4 || y == base - 1 {
                    MOSS.shadow
                } else {
                    MOSS.base
                };
                moss.set(x, y, color);
            }
            if pick(x, 1, 705, 9) == 0 {
                let long = 1 + pick(x, 2, 705, 3);
                vline(&mut moss, x, base + 1, long, MOSS.base);
                moss.set(x, base + long + 1, MOSS.edge);
            }
        }
    }
    let mut outline = Vec::new();
    for x in left..right + 1 {
        for y in log_top(x) - 8..log_top(x) {
            let beside = [(0, 1), (1, 0), (-1, 0)]
                .iter()
                .any(|&(dx, dy)| moss.get(x + dx, y + dy).a > 0);
            if moss.get(x, y).a == 0 && beside {
                outline.push((x, y));
            }
        }
    }
    for (x, y) in outline {
        moss.set(x, y, MOSS.edge);
    }
    blit(scene, &moss, 0, 0);
    // Bracket fungi stepping along its side.
    for (cx, cy, half) in [(168, 104, 3), (173, 102, 2), (224, 105, 3)] {
        hline(scene, cx - half + 1, cy - 1, half * 2 - 1, CAP.light);
        put(scene, cx - half + 1, cy - 1, CAP.shine);
        hline(scene, cx - half, cy, half * 2 + 1, CAP.base);
        put(scene, cx - half, cy, CAP.light);
        hline(scene, cx - half, cy + 1, half * 2 + 1, CAP.edge);
        put(scene, cx + half, cy - 1, CAP.edge);
    }
    // Grass and a fern growing up against its foot.
    for x in left - 2..right + 6 {
        if chance(x, 0, 706, 60) {
            let tall = 2 + pick(x, 1, 706, 4);
            let lean = pick(x, 2, 706, 3) - 1;
            let foot = LOG_FOOT + 2;
            line(scene, (x, foot), (x + lean, foot - tall), SWARD.shadow);
            put(scene, x + lean, foot - tall, SWARD.light);
        }
    }
    for (angle, length, level) in [(-150.0, 9.0, 2), (-118.0, 10.0, 3), (-172.0, 7.0, 3)] {
        frond(scene, (left + 2, LOG_FOOT + 3), (angle, length), level);
    }
}

/// One fern frond from the crown at `from`, arching out at `angle` degrees and drooping at its
/// tip, with leaflets either side that shorten towards the tip.
fn frond(scene: &mut Canvas, from: (i32, i32), (angle, length): (f32, f32), level: i32) {
    let radians = f32::to_radians(angle);
    let (dx, dy) = (radians.cos(), radians.sin());
    let droop = length * (0.25 + 0.4 * dx.abs());
    let spine = |t: f32| {
        (
            from.0 as f32 + dx * length * t,
            from.1 as f32 + dy * length * t + droop * t * t,
        )
    };
    let steps = length as i32;
    let mut previous = spine(0.0);
    let round = |p: (f32, f32)| (p.0.round() as i32, p.1.round() as i32);
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let point = spine(t);
        if t > 0.18 && step % 2 == 0 {
            let (ax, ay) = (point.0 - previous.0, point.1 - previous.1);
            let norm = (ax * ax + ay * ay).sqrt().max(0.01);
            let (ax, ay) = (ax / norm, ay / norm);
            let reach = (1.0 - t) * 3.0 + 1.0;
            for side in [-1.0f32, 1.0] {
                let (nx, ny) = (-ay * side, ax * side);
                let tip = (
                    point.0 + nx * reach + ax * reach * 0.5,
                    point.1 + ny * reach + ay * reach * 0.5 + 0.6,
                );
                let upper = ny < 0.0;
                line(
                    scene,
                    round(point),
                    round(tip),
                    tone(FERN, level + i32::from(upper)),
                );
            }
        }
        line(scene, round(previous), round(point), tone(FERN, level - 1));
        previous = point;
    }
}

/// The reed heads at the first five perches: the feathery plumes of the common reed, or
/// bulrushes' brown velvet.
const PLUMES: [bool; 5] = [true, false, true, false, true];

/// The row of the pond's far shore at a column, where the reeds stand.
fn far_shore(x: i32) -> i32 {
    let (cx, cy, rx, ry) = POND;
    let u = ((x as f32 + 0.5 - cx) / rx).clamp(-1.0, 1.0);
    (cy - ry * (1.0 - u * u).sqrt()).round() as i32
}

/// Reeds and bulrushes in a bed along the far side of the pond and thick up its right end: a
/// mass of stems standing close, striped light and dark and darkest at their foot; leaves
/// arching out of it; and a head at each of the first five perches, standing up above the bed.
/// The rest of the perches are open water, which the reeds keep clear of.
fn reeds(scene: &mut Canvas) {
    let (cx, cy, rx, _) = POND;
    let (cx, cy, rx) = (cx as i32, cy as i32, rx as i32);
    let perches = Haunt::Reeds.perches();
    // How high the bed stands at a column: in broad stands, lower round each head so the
    // heads rise above it, and tallest up the right end.
    let height = |x: i32| -> i32 {
        let mut high = 6 + (patches(x, 0, (9, 1), 803) * 9.0) as i32;
        if x > cx + rx - 8 {
            high += x - (cx + rx - 8);
        }
        for &(px, py) in &perches[..5] {
            if (x - px as i32).abs() <= 4 {
                high = high.min(far_shore(x) - (py as i32 + 5));
            }
        }
        high.max(3)
    };
    let foot_at = |x: i32| {
        if x > cx + rx - 4 {
            // Round the right end the bed comes forward onto the near bank.
            cy + 22 - (x - (cx + rx)).max(0)
        } else {
            far_shore(x) + 1
        }
    };
    for x in cx - rx - 3..WIDTH {
        let foot = foot_at(x);
        let top = far_shore(x.min(cx + rx)) - height(x);
        let ragged = pick(x, 0, 805, 5) - 1;
        for y in top + ragged..foot {
            // Leave the water in front of the right end open.
            if in_pond(x, y) || in_pond(x - 1, y + 2) {
                continue;
            }
            let down = (y - top) as f32 / (foot - top).max(1) as f32;
            // Stems in vertical stripes, lit and shaded, deepening to the foot.
            let mut level = match pick(x, y.div_euclid(5 + pick(x, 1, 807, 4)), 806, 6) {
                0 | 1 => 3,
                2 | 3 => 2,
                4 => 1,
                _ => 0,
            };
            if down > 0.55 {
                level -= 1;
            }
            if down > 0.8 {
                level -= 1;
            }
            if y == top + ragged && level >= 2 {
                level += 1;
            }
            scene.set(x, y, tone(REED, level.clamp(0, 4)));
        }
    }
    // Leaves and stems standing out of the bed: (foot, how tall, how far it leans, whether it
    // arches over).
    let mut blades: Vec<((i32, i32), i32, i32, bool)> = Vec::new();
    for index in 0..90 {
        let x = cx - rx - 2 + pick(index, 0, 801, WIDTH - (cx - rx) + 2);
        let foot = foot_at(x) - 2 - pick(index, 1, 801, 4);
        let arch = pick(index, 2, 801, 2) == 0;
        let tall = height(x) + 1 + pick(index, 3, 801, 6);
        let lean = if arch {
            (pick(index, 4, 801, 2) * 2 - 1) * (3 + pick(index, 5, 801, 4))
        } else {
            pick(index, 4, 801, 5) - 2
        };
        blades.push(((x, foot), tall, lean, arch));
    }
    // None of them reaches up past a head, or over the open water a dragonfly hovers above.
    for blade in &mut blades {
        let ((x, foot), tall, lean, _) = *blade;
        for &(px, py) in &perches[..5] {
            let (px, py) = (px as i32, py as i32);
            if (x + lean - px).abs() <= 3 || (x + lean / 2 - px).abs() <= 2 {
                blade.1 = tall.min(foot - (py + 4));
            }
        }
    }
    blades.retain(|&((x, foot), tall, lean, _)| {
        tall >= 2
            && perches[5..].iter().all(|&(px, py)| {
                let (px, py) = (px as i32, py as i32);
                let reach = (x - px).abs().min((x + lean - px).abs());
                reach > 5 || foot - tall > py + 4 || foot < py - 8
            })
    });
    blades.sort_by_key(|blade| blade.0.1);
    for (index, &(foot, tall, lean, arch)) in blades.iter().enumerate() {
        reed_blade(scene, foot, (tall, lean), arch, index);
    }
    for (index, &(px, py)) in perches[..5].iter().enumerate() {
        let (px, py) = (px as i32, py as i32);
        let foot = (px + 1 - (index as i32 % 2) * 2, far_shore(px) - 1);
        if PLUMES[index] {
            plume(scene, foot, (px, py + 1));
        } else {
            bulrush(scene, foot, (px, py + 1));
        }
    }
}

/// One reed leaf or stem, rising from its foot and leaning out, an arching leaf curling over at
/// its tip; lit on the left.
fn reed_blade(
    scene: &mut Canvas,
    (x0, foot): (i32, i32),
    (tall, lean): (i32, i32),
    arch: bool,
    index: usize,
) {
    let level = 1 + (index % 3) as i32;
    let steps = tall * 2;
    let mut previous = (x0, foot);
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let (dx, rise) = if arch {
            (lean as f32 * t * t, tall as f32 * (2.0 * t - t * t))
        } else {
            (lean as f32 * t * t, tall as f32 * t)
        };
        let point = (x0 + dx.round() as i32, foot - rise.round() as i32);
        let color = if t > 0.85 {
            tone(REED, level + 1)
        } else if t < 0.25 {
            tone(REED, level - 1)
        } else {
            tone(REED, level)
        };
        line(scene, previous, point, color);
        previous = point;
    }
}

/// A common reed's plume on its stem from `foot`, its topmost tuft at `top`: a soft, nodding
/// panicle gone pale, hanging to the right, lit on its left and outlined in its own shade.
fn plume(scene: &mut Canvas, foot: (i32, i32), top: (i32, i32)) {
    let (tx, ty) = top;
    line(scene, foot, (tx - 1, ty + 2), REED.light);
    line(scene, (foot.0 + 1, foot.1), (tx, ty + 3), REED.shadow);
    // A leaf off the stem partway up, arching away.
    let mid = ((foot.0 + tx) / 2, (foot.1 + ty) / 2 + 3);
    line(scene, mid, (mid.0 - 3, mid.1 - 2), REED.light);
    line(scene, (mid.0 - 3, mid.1 - 2), (mid.0 - 6, mid.1), REED.base);
    // Its rows, top to bottom: where each starts and how wide it is.
    let rows = [
        (-1, 3),
        (-2, 5),
        (-2, 5),
        (-1, 5),
        (0, 4),
        (0, 4),
        (1, 3),
        (2, 2),
    ];
    for (row, &(from, width)) in rows.iter().enumerate() {
        let y = ty + row as i32;
        for dx in 0..width {
            let x = tx + from + dx;
            let outside = dx == 0 || dx == width - 1 || row == rows.len() - 1;
            let color = if row == 0 && dx > 0 && dx < width - 1 {
                PLUME.shine
            } else if outside && (dx > 0 || row > 2) {
                PLUME.edge
            } else if dx <= 1 {
                PLUME.light
            } else if (row + dx as usize).is_multiple_of(3) {
                PLUME.shadow
            } else {
                PLUME.base
            };
            scene.set(x, y, color);
        }
    }
    hline(scene, tx - 1, ty - 1, 3, PLUME.edge);
}

/// A bulrush on its stem from `foot`, the top of its brown velvet head at `top`, a thin spike
/// standing up from it.
fn bulrush(scene: &mut Canvas, foot: (i32, i32), top: (i32, i32)) {
    let (tx, ty) = top;
    let head = 6;
    line(scene, foot, (tx, ty + head), REED.shadow);
    vline(scene, tx, ty - 3, 3, REED.base);
    put(scene, tx, ty - 3, REED.light);
    for y in ty..ty + head {
        put(
            scene,
            tx - 1,
            y,
            if y <= ty + 1 {
                BULRUSH.shine
            } else {
                BULRUSH.light
            },
        );
        put(scene, tx, y, BULRUSH.base);
        put(scene, tx + 1, y, BULRUSH.edge);
    }
    put(scene, tx - 2, ty + 2, BULRUSH.edge);
    put(scene, tx - 2, ty + 3, BULRUSH.edge);
    hline(scene, tx - 1, ty - 1, 2, BULRUSH.edge);
    hline(scene, tx - 1, ty + head, 3, BULRUSH.edge);
}

/// A small leaf five pixels across, lit on its upper left whichever way its tip points, its
/// lower edge in the leaf's own darkest tone. `level` is how much light it catches.
fn leaf(scene: &mut Canvas, (x, y): (i32, i32), ramp: Ramp, level: i32, tip_left: bool) {
    put(scene, x - 1, y - 1, tone(ramp, level + 1));
    put(scene, x, y - 1, tone(ramp, level + 1));
    put(scene, x - 2, y, tone(ramp, level + i32::from(tip_left)));
    put(scene, x - 1, y, tone(ramp, level));
    put(scene, x, y, tone(ramp, level));
    put(scene, x + 1, y, tone(ramp, level - 1));
    put(
        scene,
        x + 2,
        y,
        if tip_left {
            ramp.edge
        } else {
            tone(ramp, level - 1)
        },
    );
    put(scene, x - 1, y + 1, tone(ramp, level - 1));
    put(scene, x, y + 1, ramp.edge);
    put(scene, x + 1, y + 1, ramp.edge);
}

/// The bramble mound's top at a column, ragged; it runs off the left edge.
fn bramble_top(x: i32) -> i32 {
    let u = ((x - 26) as f32 / 52.0).max(-0.8);
    // Heaped higher in a lobe or two, where old canes have arched over and rooted.
    let lobe =
        |middle: f32, wide: f32, high: f32| high * (-((x as f32 - middle) / wide).powi(2)).exp();
    let lobes = lobe(6.0, 10.0, 6.0) + lobe(38.0, 8.0, 4.0) + lobe(58.0, 6.0, 3.0);
    72 + (u.powi(4) * 54.0) as i32 + (patches(x, 0, (4, 1), 905) * 5.0) as i32 - lobes as i32
}

/// The bramble thicket on the left edge, where the trees give way: a mound of small leaves in
/// a dark tangle, canes arching up out of it and down again, flowers and blackberries ripe and
/// not among them, and a leaf turned up to the light at each perch.
fn bramble(scene: &mut Canvas) {
    const FOOT: i32 = 125;
    ellipse(scene, 38, FOOT + 1, 42, 3, SHADE);
    let inside = |x: i32, y: i32| (-8..80).contains(&x) && y >= bramble_top(x) && y <= FOOT;
    // The tangle inside, dark and deepening downwards.
    for x in -8..80 {
        let top = bramble_top(x);
        for y in top..=FOOT {
            let deep = (y - top) as f32 / (FOOT - top).max(1) as f32;
            let color = if y == FOOT {
                BRAMBLE.edge
            } else if chance(x, y, 906, 50) {
                BRAMBLE.shadow
            } else {
                mix(BRAMBLE.shadow, BRAMBLE.edge, 0.4 + deep * 0.6)
            };
            scene.set(x, y, color);
        }
    }
    // Canes behind the leaves, showing between them.
    let canes = [
        ((22, 96), (30, 54), (50, 66)),
        ((44, 100), (64, 58), (82, 86)),
        ((6, 118), (-6, 78), (-12, 96)),
        ((30, 124), (56, 92), (78, 122)),
        ((12, 110), (20, 72), (36, 78)),
    ];
    for (index, &(from, bend, to)) in canes.iter().enumerate().skip(2) {
        cane(scene, (from, bend, to), 910 + index as u32);
    }
    // Leaves over it all, top to bottom, brighter up on its sunny top and towards the left.
    let mut leaves = Vec::new();
    for (row, gy) in (66..FOOT).step_by(2).enumerate() {
        for gx in (-8..82).step_by(3) {
            let x = gx + if row % 2 == 0 { 0 } else { 2 } + pick(gx, gy, 907, 3) - 1;
            let y = gy + pick(gx, gy, 908, 3) - 1;
            if !inside(x, y) || !inside(x, y + 1) {
                continue;
            }
            let depth = (y - bramble_top(x)) as f32;
            let light = 3.6 - depth / 12.0 - x as f32 / 110.0 + pick(x, y, 909, 3) as f32 * 0.45;
            let level = light.round().clamp(1.0, 3.0) as i32;
            if chance(x, y, 911, 70) {
                continue;
            }
            leaves.push((x, y, level));
        }
    }
    for &(x, y, level) in &leaves {
        leaf(scene, (x, y), BRAMBLE, level, (x + y) % 3 == 0);
    }
    // Canes arching up out of it, thorny, with leaves along them.
    for (index, &(from, bend, to)) in canes.iter().enumerate().take(2) {
        cane(scene, (from, bend, to), 910 + index as u32);
    }
    // Flowers, white with a blush of pink, and the berries coming after them.
    for &(x, y) in &[
        (10, 82),
        (40, 80),
        (64, 92),
        (16, 112),
        (30, 96),
        (56, 100),
        (4, 98),
    ] {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            put(scene, x + dx, y + dy, BLOSSOM);
        }
        put(scene, x + 1, y + 1, mix(BLOSSOM, CAMPION.light, 0.4));
        put(scene, x + 1, y, mix(BLOSSOM, CAMPION.light, 0.25));
        put(scene, x, y, BUTTERCUP.light);
    }
    for (index, &(x, y)) in [
        (20, 90),
        (22, 92),
        (44, 96),
        (46, 98),
        (8, 106),
        (34, 108),
        (36, 110),
        (62, 86),
        (14, 96),
        (44, 114),
        (28, 100),
        (54, 106),
    ]
    .iter()
    .enumerate()
    {
        let ramp = match index % 4 {
            0 | 1 => BERRY,
            2 => UNRIPE,
            _ => mix_ramp(BERRY, UNRIPE),
        };
        put(scene, x, y, ramp.base);
        put(scene, x + 1, y, ramp.shadow);
        put(scene, x, y + 1, ramp.shadow);
        put(scene, x + 1, y + 1, ramp.edge);
        put(scene, x, y - 1, ramp.light);
        put(scene, x - 1, y, ramp.light);
        put(scene, x - 1, y - 1, ramp.shine);
    }
    for &(x, y) in Haunt::Bramble.perches() {
        bramble_leaf(scene, (x as i32, y as i32 + 1));
    }
    // Grass growing up against its foot, so it sits in the meadow.
    for x in -2..82 {
        let foot = FOOT + 1;
        if bramble_top(x) >= FOOT || !chance(x, 0, 912, 150) {
            continue;
        }
        let tall = 2 + pick(x, 1, 912, 4);
        let lean = pick(x, 2, 912, 3) - 1;
        line(scene, (x, foot), (x + lean, foot - tall), SWARD.base);
        put(scene, x + lean, foot - tall, SWARD.light);
    }
}

/// A ramp halfway between two others, for berries ripening.
fn mix_ramp(a: Ramp, b: Ramp) -> Ramp {
    Ramp {
        edge: mix(a.edge, b.edge, 0.5),
        shadow: mix(a.shadow, b.shadow, 0.5),
        base: mix(a.base, b.base, 0.5),
        light: mix(a.light, b.light, 0.5),
        shine: mix(a.shine, b.shine, 0.5),
    }
}

/// A bramble cane along a curve `(from, bend, to)`: dark red, lit along its top, thorny, with
/// leaves in threes along it.
fn cane(scene: &mut Canvas, (from, bend, to): ((i32, i32), (i32, i32), (i32, i32)), salt: u32) {
    let mut previous = from;
    for step in 1..=32 {
        let point = curve(from, bend, to, step as f32 / 32.0);
        line(scene, previous, point, CANE.base);
        put(scene, point.0, point.1 - 1, CANE.light);
        put(scene, point.0, point.1 + 1, CANE.edge);
        if chance(point.0, point.1, salt, 50) {
            put(scene, point.0 + 1, point.1 - 1, CANE.shine);
        }
        if step % 8 == 5 {
            leaf(scene, (point.0 - 2, point.1 - 2), BRAMBLE, 3, true);
            leaf(scene, (point.0 + 3, point.1 - 1), BRAMBLE, 2, false);
        }
        previous = point;
    }
}

/// A bramble leaf turned up to the sun, its upper face flat enough for something to settle on
/// at `top`: bigger and brighter than the leaves round it, lit along its top and left, a midrib
/// down it, its tip hanging, and its toothed edge outlined in the bramble's darkest green so it
/// stands out from the tangle.
fn bramble_leaf(scene: &mut Canvas, (x, top): (i32, i32)) {
    let rows: [(i32, i32); 5] = [(-3, 3), (-4, 4), (-4, 4), (-3, 3), (-1, 1)];
    for (row, &(from, to)) in rows.iter().enumerate() {
        let y = top + row as i32;
        put(scene, x + from - 1, y, BRAMBLE.edge);
        put(scene, x + to + 1, y, BRAMBLE.edge);
        for px in x + from..=x + to {
            let left = px < x;
            let color = match row {
                0 => {
                    if left || px == x {
                        SUNLIT_LEAF.shine
                    } else {
                        SUNLIT_LEAF.light
                    }
                }
                1 if px == x => SUNLIT_LEAF.base,
                1 | 2 if left => SUNLIT_LEAF.light,
                1 => SUNLIT_LEAF.base,
                2 if px == x => SUNLIT_LEAF.shadow,
                2 => SUNLIT_LEAF.base,
                3 if left => SUNLIT_LEAF.base,
                _ => SUNLIT_LEAF.shadow,
            };
            scene.set(px, y, color);
        }
    }
    hline(scene, x - 3, top - 1, 7, BRAMBLE.edge);
    hline(scene, x - 1, top + 5, 3, BRAMBLE.edge);
    // Its teeth, and its stalk going back into the tangle.
    put(scene, x - 5, top + 1, SUNLIT_LEAF.base);
    put(scene, x - 6, top + 1, BRAMBLE.edge);
    put(scene, x + 5, top + 3, BRAMBLE.edge);
    put(scene, x + 4, top + 4, BRAMBLE.edge);
    put(scene, x - 4, top + 4, BRAMBLE.edge);
    put(scene, x + 5, top + 1, CANE.shadow);
    put(scene, x + 6, top + 2, CANE.base);
}

/// What flower stands at each perch of the wildflowers, in order.
#[derive(Clone, Copy)]
enum Bloom {
    Daisy,
    Knapweed,
    Campion,
    Buttercup,
    Clover,
}

const BLOOMS: [Bloom; 10] = [
    Bloom::Daisy,
    Bloom::Knapweed,
    Bloom::Campion,
    Bloom::Buttercup,
    Bloom::Daisy,
    Bloom::Clover,
    Bloom::Knapweed,
    Bloom::Buttercup,
    Bloom::Campion,
    Bloom::Clover,
];

/// How far into the wildflower patch a point is: under 1 inside it, its edge ragged.
fn in_wildflowers(x: i32, y: i32) -> f32 {
    let (u, v) = ((x - 118) as f32 / 56.0, (y - 131) as f32 / 21.0);
    u * u + v * v + (patches(x, y, (7, 4), 1001) - 0.5) * 0.6
}

/// The wildflower patch, left of the middle: a lush tangle of leaves and stems, darker than the
/// sward so the flowers stand out against it, small flowers all through it, and at each perch a
/// flower standing up taller than the rest on its own stem: ox-eye daisies, knapweed, red
/// campion, buttercups and clover.
fn wildflowers(scene: &mut Canvas) {
    // The shade the plants make among themselves, close to the ground.
    for y in 110..158 {
        for x in 58..182 {
            let reach = in_wildflowers(x, y);
            if reach < 1.0 && chance(x, y, 1002, (200.0 * (1.0 - reach * 0.6)) as u32) {
                put(scene, x, y, rgba(0x1c3020, 60));
            }
        }
    }
    // Leaves, back to front, lit on the tops of the plants and darker down among them.
    for (row, gy) in (110..158).step_by(2).enumerate() {
        for gx in (58..182).step_by(3) {
            let x = gx + if row % 2 == 0 { 0 } else { 1 } + pick(gx, gy, 1003, 3) - 1;
            let y = gy + pick(gx, gy, 1004, 2);
            let reach = in_wildflowers(x, y);
            if reach > 1.0 || !chance(x, y, 1005, (240.0 * (1.0 - reach * 0.7)) as u32) {
                continue;
            }
            let ramp = if pick(x, y, 1006, 3) == 0 {
                SWARD
            } else {
                HAZEL_LEAF
            };
            let level = 1 + pick(x, y, 1007, 3) - i32::from(reach > 0.7);
            leaf(scene, (x, y), ramp, level, (x + y) % 2 == 0);
        }
    }
    // Grass stems and leaves standing up out of it.
    for index in 0..140 {
        let x = 60 + pick(index, 0, 1008, 120);
        let y = 116 + pick(index, 1, 1008, 40);
        if in_wildflowers(x, y) > 1.0 {
            continue;
        }
        let tall = 4 + pick(index, 2, 1008, 6);
        let lean = pick(index, 3, 1008, 5) - 2;
        let (body, tip) = match pick(index, 4, 1008, 3) {
            0 => (SWARD.shadow, SWARD.base),
            1 => (SWARD.base, SWARD.light),
            _ => (SWARD.shadow, SEED.light),
        };
        line(scene, (x, y), (x + lean, y - tall + 1), body);
        put(scene, x + lean, y - tall, tip);
    }
    // Small flowers all through it, on short stalks.
    for index in 0..110 {
        let x = 62 + pick(index, 0, 1009, 116);
        let y = 114 + pick(index, 1, 1009, 40);
        if in_wildflowers(x, y) > 0.95 {
            continue;
        }
        let (petal, heart, shade) = match pick(index, 2, 1009, 5) {
            0 => (DAISY.light, BUTTERCUP.base, DAISY.shadow),
            1 => (BUTTERCUP.light, BUTTERCUP.base, BUTTERCUP.shadow),
            2 => (KNAPWEED.light, KNAPWEED.base, KNAPWEED.shadow),
            3 => (CAMPION.light, CAMPION.base, CAMPION.shadow),
            _ => (CLOVER.light, CLOVER.base, CLOVER.shadow),
        };
        put(scene, x, y + 1, SWARD.shadow);
        put(scene, x, y + 2, SWARD.edge);
        put(scene, x - 1, y, petal);
        put(scene, x, y, heart);
        put(scene, x + 1, y, shade);
        if index % 3 == 0 {
            put(scene, x, y - 1, petal);
        }
    }
    // The perch flowers, each on its own stem, back to front.
    let mut flowers: Vec<((i32, i32), Bloom, usize)> = Haunt::Flowers
        .perches()
        .iter()
        .zip(BLOOMS)
        .enumerate()
        .map(|(index, (&(x, y), bloom))| ((x as i32, y as i32 + 1), bloom, index))
        .collect();
    flowers.sort_by_key(|((_, y), _, _)| *y);
    for ((x, top), bloom, index) in flowers {
        let tall = 10 + pick(index as i32, 0, 1010, 5);
        let lean = pick(index as i32, 1, 1010, 3) - 1;
        let stem = (x + lean, top + tall);
        line(scene, stem, (x, top + 2), SWARD.shadow);
        line(
            scene,
            (stem.0 + 1, stem.1),
            (x + 1, top + 5),
            mix(SWARD.shadow, SWARD.edge, 0.5),
        );
        // A leaf either side partway up the stem.
        let (lx, ly) = (x + lean / 2, top + tall / 2 + 2);
        leaf(scene, (lx - 3, ly), SWARD, 3, true);
        leaf(scene, (lx + 3, ly + 2), SWARD, 2, false);
        flower(scene, (x, top), bloom);
    }
}

/// One flower head seen from the side, its topmost point at `top`, where something settles.
fn flower(scene: &mut Canvas, (x, top): (i32, i32), bloom: Bloom) {
    match bloom {
        Bloom::Daisy => {
            // A raised yellow eye on a flat ring of white rays, the lower ones in shade.
            hline(scene, x - 1, top, 3, BUTTERCUP.light);
            put(scene, x - 1, top, BUTTERCUP.shine);
            hline(scene, x - 1, top + 1, 3, BUTTERCUP.base);
            put(scene, x + 1, top + 1, BUTTERCUP.shadow);
            hline(scene, x - 4, top + 1, 3, DAISY.shine);
            hline(scene, x + 2, top + 1, 3, DAISY.light);
            hline(scene, x - 4, top + 2, 9, DAISY.light);
            put(scene, x - 4, top + 2, DAISY.shine);
            hline(scene, x - 3, top + 3, 7, DAISY.shadow);
            put(scene, x - 5, top + 2, DAISY.edge);
            put(scene, x + 5, top + 2, DAISY.edge);
            put(scene, x + 4, top + 1, DAISY.base);
            hline(scene, x - 2, top + 4, 5, DAISY.edge);
        }
        Bloom::Knapweed => {
            // A tuft of purple florets on a hard dark knob of bracts.
            for (dx, color) in [
                (-2, KNAPWEED.light),
                (-1, KNAPWEED.shine),
                (0, KNAPWEED.light),
                (1, KNAPWEED.base),
                (2, KNAPWEED.shadow),
            ] {
                put(scene, x + dx, top + 1, color);
                if dx.abs() < 2 {
                    put(
                        scene,
                        x + dx,
                        top,
                        if dx < 1 {
                            KNAPWEED.light
                        } else {
                            KNAPWEED.base
                        },
                    );
                }
            }
            put(scene, x - 3, top + 1, KNAPWEED.edge);
            put(scene, x + 3, top + 1, KNAPWEED.edge);
            hline(scene, x - 2, top + 2, 5, BULRUSH.light);
            put(scene, x - 2, top + 2, BULRUSH.shine);
            hline(scene, x - 2, top + 3, 5, BULRUSH.base);
            put(scene, x + 2, top + 3, BULRUSH.edge);
            hline(scene, x - 1, top + 4, 3, BULRUSH.edge);
        }
        Bloom::Campion => {
            // Five notched pink petals opening flat, on a swollen dark calyx.
            hline(scene, x - 3, top + 1, 7, CAMPION.base);
            hline(scene, x - 2, top, 5, CAMPION.light);
            put(scene, x - 2, top, CAMPION.shine);
            put(scene, x, top, CAMPION.shadow);
            put(scene, x - 3, top + 1, CAMPION.light);
            put(scene, x + 3, top + 1, CAMPION.shadow);
            put(scene, x - 4, top + 1, CAMPION.edge);
            put(scene, x + 4, top + 1, CAMPION.edge);
            hline(scene, x - 2, top + 2, 5, CAMPION.edge);
            put(scene, x, top + 1, rgb(0xf8d8e0));
            vline(
                scene,
                x - 1,
                top + 3,
                2,
                mix(CAMPION.shadow, SWARD.shadow, 0.5),
            );
            vline(scene, x, top + 3, 2, mix(CAMPION.edge, SWARD.edge, 0.5));
            put(scene, x + 1, top + 3, mix(CAMPION.edge, SWARD.edge, 0.6));
        }
        Bloom::Buttercup => {
            // A glossy yellow cup, shining where the sun catches its rim.
            hline(scene, x - 2, top, 5, BUTTERCUP.light);
            put(scene, x - 2, top, BUTTERCUP.shine);
            put(scene, x, top, BUTTERCUP.base);
            put(scene, x - 3, top, BUTTERCUP.edge);
            put(scene, x + 3, top, BUTTERCUP.edge);
            hline(scene, x - 2, top + 1, 5, BUTTERCUP.base);
            put(scene, x - 2, top + 1, BUTTERCUP.shine);
            put(scene, x + 2, top + 1, BUTTERCUP.shadow);
            hline(scene, x - 1, top + 2, 3, BUTTERCUP.shadow);
            put(scene, x + 1, top + 2, BUTTERCUP.edge);
            put(scene, x, top + 3, SWARD.edge);
        }
        Bloom::Clover => {
            // A round head of pink florets, paler at the top where they open first.
            hline(scene, x - 1, top, 3, CLOVER.light);
            put(scene, x - 1, top, CLOVER.shine);
            hline(scene, x - 2, top + 1, 5, CLOVER.base);
            put(scene, x - 2, top + 1, CLOVER.light);
            hline(scene, x - 2, top + 2, 5, CLOVER.shadow);
            put(scene, x - 2, top + 2, CLOVER.base);
            hline(scene, x - 1, top + 3, 3, CLOVER.edge);
            put(scene, x - 3, top + 1, CLOVER.edge);
            put(scene, x + 3, top + 1, CLOVER.edge);
            put(scene, x + 2, top + 2, CLOVER.edge);
            // A trefoil leaf just under it.
            put(scene, x - 2, top + 4, SWARD.light);
            put(scene, x + 2, top + 4, SWARD.base);
        }
    }
}

/// How far into the stand of long grass a point is: under 1 inside it, its edge ragged.
fn in_long_grass(x: i32, y: i32) -> f32 {
    let (u, v) = ((x - 291) as f32 / 66.0, (y - 186) as f32 / 23.0);
    u * u + v * v + (patches(x, y, (9, 4), 1101) - 0.5) * 0.5
}

/// The long grass, front right: a stand of meadow grass left uncut and gone to seed, green
/// down in it and straw-gold at the top, so it stands apart from the sward round it; thin
/// stems carrying seed heads above it; and at each perch a stem taller than those round it,
/// ending in a seed head outlined against the grass behind.
fn long_grass(scene: &mut Canvas) {
    let perches = Haunt::Grass.perches();
    // The shade down in the stand, where the grass grows thick.
    for y in 160..214 {
        for x in 220..362 {
            let reach = in_long_grass(x, y);
            if reach < 1.0 && chance(x, y, 1102, (230.0 * (1.0 - reach * 0.5)) as u32) {
                put(scene, x, y, rgba(0x1c3020, 90));
            }
        }
    }
    // The blades, back to front: taller in the middle of the stand, shorter at its edges.
    let mut blades: Vec<((i32, i32), i32, i32)> = Vec::new();
    for index in 0..650 {
        let x = 222 + pick(index, 0, 1103, 138);
        let y = 164 + pick(index, 1, 1103, 48);
        let reach = in_long_grass(x, y);
        if reach > 1.0 {
            continue;
        }
        let tall = (5.0 + (1.0 - reach) * 12.0) as i32 + pick(index, 2, 1103, 6);
        let lean = pick(index, 3, 1103, 7) - 3;
        blades.push(((x, y), tall, lean));
    }
    // Nothing reaches up past a perch's seed head, so each stands clear.
    for blade in &mut blades {
        let ((x, y), _, lean) = *blade;
        for &(px, py) in perches {
            let (px, py) = (px as i32, py as i32);
            if (x + lean - px).abs() <= 3 || (x + lean / 2 - px).abs() <= 2 {
                blade.1 = blade.1.min(y - (py + 8));
            }
        }
    }
    blades.retain(|blade| blade.1 >= 2);
    blades.sort_by_key(|blade| blade.0.1);
    for (index, &((x, y), tall, lean)) in blades.iter().enumerate() {
        // Lit on the blades leaning into the light, gold at the top where it has dried.
        let level = match index % 4 {
            0 => 3,
            1 | 2 => 2,
            _ => 1,
        };
        let mut previous = (x, y);
        for step in 1..=tall {
            let t = step as f32 / tall as f32;
            let point = (x + (lean as f32 * t * t).round() as i32, y - step);
            let color = if t > 0.8 && index % 3 == 0 {
                tone(SEED, level + 1)
            } else if t > 0.6 {
                mix(tone(SWARD, level + 1), tone(SEED, level), 0.45)
            } else if t > 0.3 {
                tone(SWARD, level)
            } else {
                tone(SWARD, level - 1)
            };
            line(scene, previous, point, color);
            previous = point;
        }
    }
    // Stems standing up out of it, each with a small seed head of its own.
    for index in 0..40 {
        let x = 226 + pick(index, 0, 1106, 130);
        let y = 172 + pick(index, 1, 1106, 36);
        if in_long_grass(x, y) > 0.8 {
            continue;
        }
        let high = 14 + pick(index, 2, 1106, 8);
        let clear = perches
            .iter()
            .all(|&(px, py)| (x - px as i32).abs() > 4 || y - high > py as i32 + 6);
        if clear {
            vline(scene, x, y - high + 3, high - 3, SEED.shadow);
            vline(scene, x, y - high, 3, SEED.light);
            put(scene, x + 1, y - high + 1, SEED.edge);
            put(scene, x, y - high - 1, SEED.shine);
        }
    }
    for (index, &(px, py)) in perches.iter().enumerate() {
        let (px, py) = (px as i32, py as i32);
        let tall = 13 + pick(index as i32, 0, 1107, 6);
        let lean = pick(index as i32, 1, 1107, 5) - 2;
        let foot = (px - lean, py + tall);
        let mut previous = foot;
        for step in 1..=tall {
            let t = step as f32 / tall as f32;
            let point = (foot.0 + (lean as f32 * t).round() as i32, foot.1 - step);
            let color = if t > 0.5 { SEED.shadow } else { SWARD.shadow };
            line(scene, previous, point, color);
            previous = point;
        }
        seed_head(scene, (px, py + 1), index);
    }
}

/// A grass seed head with its top at `top`, outlined in its own shade so it shows against the
/// grass behind: a tight spike of timothy, or the looser, pinker plume of Yorkshire fog.
fn seed_head(scene: &mut Canvas, (x, top): (i32, i32), index: usize) {
    if index.is_multiple_of(2) {
        for y in top..top + 6 {
            put(scene, x - 2, y, SEED.edge);
            put(
                scene,
                x - 1,
                y,
                if y <= top + 1 { SEED.shine } else { SEED.light },
            );
            put(scene, x, y, if y % 2 == 0 { SEED.base } else { SEED.light });
            put(scene, x + 1, y, SEED.edge);
        }
        hline(scene, x - 1, top - 1, 2, SEED.edge);
        hline(scene, x - 1, top + 6, 2, SEED.edge);
    } else {
        let fog = Ramp::new(0x6a4a4a, 0x9a7a74, 0xc0a09a, 0xd8bcb4, 0xeedad2);
        let rows = [(-1, 3), (-2, 5), (-2, 5), (-1, 4), (0, 2)];
        for (row, &(from, width)) in rows.iter().enumerate() {
            for dx in 0..width {
                let px = x + from + dx;
                let color = if dx == width - 1 || (dx == 0 && row > 2) || row == rows.len() - 1 {
                    fog.edge
                } else if dx <= 1 && row < 3 {
                    fog.shine
                } else if row < 2 {
                    fog.light
                } else {
                    fog.base
                };
                scene.set(px, top + row as i32, color);
            }
        }
        hline(scene, x - 1, top - 1, 3, fog.edge);
        put(scene, x - 3, top + 1, fog.edge);
        put(scene, x - 3, top + 2, fog.edge);
    }
}

// ---------------------------------------------------------------------------------------------
// In front of everyone
// ---------------------------------------------------------------------------------------------

/// Long grass and a few flowers along the bottom corners, drawn over everyone: low, and nothing
/// in the middle of the bottom edge.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    // Two layers of blades, the back ones darker, thickest and tallest right in the corners and
    // thinning out towards the middle, where there are none.
    for layer in 0..2 {
        for x in (0..112).chain(WIDTH - 104..WIDTH) {
            let corner = if x < 112 {
                1.0 - x as f32 / 112.0
            } else {
                (x - (WIDTH - 104)) as f32 / 104.0
            };
            if !chance(x, layer, 1201, 50 + (corner * 160.0) as u32) {
                continue;
            }
            let tall = (2 + (corner * 9.0) as i32 + pick(x, layer + 1, 1201, 3)).min(12);
            let lean = pick(x, layer + 2, 1201, 7) - 3;
            let (body, tip) = match (layer, pick(x, layer + 3, 1201, 3)) {
                (0, 0) => (SWARD.edge, SWARD.shadow),
                (0, _) => (mix(SWARD.edge, SWARD.shadow, 0.5), SWARD.base),
                (_, 0) => (SWARD.shadow, SEED.base),
                (_, 1) => (SWARD.shadow, SWARD.light),
                _ => (SWARD.base, SWARD.shine),
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
    // A buttercup and a daisy or two among it, and a seed head at the right.
    for (x, tall, bloom) in [
        (14, 11, Bloom::Buttercup),
        (30, 8, Bloom::Daisy),
        (WIDTH - 20, 10, Bloom::Clover),
        (WIDTH - 40, 7, Bloom::Buttercup),
    ] {
        let top = HEIGHT - tall;
        vline(&mut scene, x, top + 2, tall - 2, SWARD.shadow);
        flower(&mut scene, (x, top), bloom);
    }
    seed_head(&mut scene, (WIDTH - 8, HEIGHT - 12), 0);
    vline(&mut scene, WIDTH - 8, HEIGHT - 5, 5, SEED.shadow);
    scene
}

// ---------------------------------------------------------------------------------------------
// After dark
// ---------------------------------------------------------------------------------------------

/// Where the glow-worms shine after dark: low in the grass at the meadow's edges, under the
/// bramble, along the foot of the log and round the stump.
const GLOW_WORMS: [(i32, i32); 9] = [
    (12, 128),
    (64, 127),
    (158, 109),
    (204, 110),
    (246, 109),
    (306, 108),
    (112, 86),
    (176, 84),
    (378, 156),
];

/// Glow-worms, each a steady point of green light low in the grass with a soft glow about it.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for &(x, y) in &GLOW_WORMS {
        crate::daylight::glow(&mut lights, (x, y), 5, rgb(0xb8f07a));
        put(&mut lights, x, y, rgb(0xeaffc0));
    }
    lights
}

/// Where the moon rises after dark: in the open sky between the treetops on the left.
const MOON: (i32, i32) = (116, 12);

/// The sky after dark, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, SKY_FOOT, Some(MOON))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn the_meadow_is_the_same_every_time() {
        assert_eq!(backdrop(&Arrangement::new()), backdrop(&Arrangement::new()));
        assert_eq!(foreground(), foreground());
        assert_eq!(lamplight(), lamplight());
    }

    #[test]
    fn every_perch_has_something_to_settle_on() {
        for haunt in Haunt::ALL {
            let mut alone = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
            paint_haunt(&mut alone, haunt);
            let perches = haunt.perches();
            // Over the pond, the last of the reeds' perches are open air for a dragonfly.
            let settled = if haunt == Haunt::Reeds {
                5
            } else {
                perches.len()
            };
            for &(x, y) in &perches[..settled] {
                let (x, y) = (x as i32, y as i32);
                assert!(
                    alone.get(x, y).a == 255 || alone.get(x, y + 1).a == 255,
                    "nothing of {} at the perch at {x}, {y}",
                    haunt.name()
                );
            }
            for &(x, y) in &perches[settled..] {
                let (x, y) = (x as i32, y as i32);
                assert!(
                    in_pond(x, y) && in_pond(x, y + 6),
                    "{x}, {y} is not over water"
                );
                for dy in -6..=6 {
                    for dx in -3..=3 {
                        assert_eq!(
                            alone.get(x + dx, y + dy).a,
                            0,
                            "a reed stands in the open air at {}, {}",
                            x + dx,
                            y + dy
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_foreground_keeps_low_and_off_where_anyone_stands() {
        let front = foreground();
        let (_, top, _, _) = front.alpha_bounds().expect("no long grass at all");
        assert!(
            top as i32 >= HEIGHT - 14,
            "the long grass reaches up to {top}"
        );
        for y in 0..HEIGHT {
            for x in WIDTH / 3..WIDTH * 2 / 3 {
                assert_eq!(front.get(x, y).a, 0, "grass in the middle at {x}, {y}");
            }
        }
        for &(name, (x, y)) in &PLACES {
            for dy in -30..=0 {
                for dx in -10..=10 {
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
    fn the_old_tree_stands_on_the_hill_as_drawn() {
        assert!((VISTA.tree.1 - hill_crest(VISTA.tree.0)).abs() < 0.5);
    }

    #[test]
    fn whatever_stands_on_the_hilltop_shows_through_the_gap() {
        for standing in crate::hilltop::Standing::every() {
            let everywhere: Arrangement = (0..SPOTS.len() as u8)
                .map(|spot| (spot, standing.clone()))
                .collect();
            let mut view = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
            skyline(&mut view, &everywhere, &VISTA);
            let scene = backdrop(&everywhere);
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let piece = view.get(x, y);
                    if piece.a > 0 {
                        assert_eq!(scene.get(x, y), piece, "{standing:?} is hidden at {x}, {y}");
                    }
                }
            }
        }
    }

    #[test]
    fn glow_worms_shine_low_in_the_grass_away_from_the_fireflies() {
        let lights = lamplight();
        let (left, top, right, bottom) = Haunt::Grass.area();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if lights.get(x, y).a == 0 {
                    continue;
                }
                assert!(y >= BROW, "a glow-worm in the sky at {x}, {y}");
                let (fx, fy) = (x as f32, y as f32);
                assert!(
                    !(left..=right).contains(&fx) || !(top..=bottom).contains(&fy),
                    "a glow-worm in the long grass at {x}, {y}, where the fireflies are"
                );
            }
        }
    }

    #[test]
    fn the_moon_rises_in_open_sky() {
        let night = night_sky(&backdrop(&Arrangement::new()));
        assert_eq!(night.get(MOON.0, MOON.1).a, 255, "the moon is hidden");
        for y in SKY_FOOT..HEIGHT {
            for x in 0..WIDTH {
                assert_eq!(night.get(x, y).a, 0, "night sky on the ground at {x}, {y}");
            }
        }
    }
}
