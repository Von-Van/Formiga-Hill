//! The old track, at the eye level of someone standing on it: a ride cut long ago through the
//! deep woods, the trees closing in on either side and leaning over, and the sky a wedge over the
//! track where it runs away to the far end, where the Hill shows small and hazy over a rise. The
//! track itself is two old wheel ruts, puddled, with grass grown up between and over them.
//!
//! Along it, everything left behind: on the left the woodcutter's hut, its roof fallen in at one
//! end and its woodshed collapsed against it; on the right a dry-stone wall that has fallen in
//! along its middle, and in front of it the broken cart that never got any further, tipped on its
//! nose with one wheel off and its load spilled across the track; and a mossy milestone at the
//! front. The heaps lie where `SITES` puts them, on beds of whatever they shed, and are drawn as
//! they stand (see `stuff`); everyone's standing places are kept clear. The palette is the
//! glade's, so the old track is plainly deeper in the same woods.

use super::routes::{Dig, Dir, Fork};
use super::{NOOK, SITES, Site};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::materials::{FAR_HILL, PATH, SKY_LOW};
use crate::paint::{Ramp, blit, chance, ellipse, hline, line, mix, noise, put, rgb, rgba, vline};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;

// The glade's materials, so the old track is deeper in the same woods.
const BARK: Ramp = Ramp::new(0x2a221e, 0x43362c, 0x5b4a3b, 0x75624d, 0x8f7c62);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const UNDERGROWTH: Ramp = Ramp::new(0x1c3628, 0x284a32, 0x36603c, 0x4c7a48, 0x6a9658);
const LITTER: Ramp = Ramp::new(0x382a1c, 0x54402e, 0x6e5238, 0x886a48, 0xa2845c);
const EARTH: Ramp = Ramp::new(0x241a16, 0x372820, 0x4c382b, 0x65503c, 0x80684e);
const GRASS: Ramp = Ramp::new(0x2a4224, 0x3a5a2e, 0x507438, 0x6a8e46, 0x88a85a);
const FERN: Ramp = Ramp::new(0x1e3a1e, 0x2e5a2a, 0x447a36, 0x60984a, 0x84b45e);
const OAK: Ramp = Ramp::new(0x1a3428, 0x264a34, 0x356240, 0x4c7e4c, 0x6c9c5c);
const BEECH: Ramp = Ramp::new(0x22401f, 0x33582a, 0x4a7236, 0x648e42, 0x86ac58);
const CLOUDS: Ramp = Ramp::new(0xc9d7e2, 0xdfe7ee, 0xf4f4f1, 0xfdfbf5, 0xffffff);
const LICHEN: Ramp = Ramp::new(0x5a5428, 0x7a7234, 0x9a9042, 0xb8ac56, 0xd0c470);
// The old track's own.
const RUT: Ramp = Ramp::new(0x2a1e16, 0x3c2c20, 0x52402e, 0x6a5440, 0x846c54);
const PUDDLE: Ramp = Ramp::new(0x2a3a3c, 0x3e5458, 0x5a7676, 0x86a2a0, 0xc4d8d2);
const DRYSTONE: Ramp = Ramp::new(0x3a3c38, 0x55574f, 0x6f7066, 0x8c8b7e, 0xaaa898);
const SILVER_OAK: Ramp = Ramp::new(0x3a3530, 0x56504a, 0x736b62, 0x91887c, 0xaea496);
const DAUB: Ramp = Ramp::new(0x7a7060, 0x9c9180, 0xb8ad98, 0xcec4ae, 0xe2d8c4);
const SHINGLE: Ramp = Ramp::new(0x2e2a26, 0x46403a, 0x5e564c, 0x786e60, 0x928878);
const CART_PAINT: Ramp = Ramp::new(0x2a3a40, 0x3e5560, 0x557480, 0x7090a0, 0x8eaab8);
const IRON: Ramp = Ramp::new(0x1c2224, 0x2e3436, 0x444a4c, 0x5e6466, 0x80888a);
const IVY: Ramp = Ramp::new(0x10281c, 0x1a3c28, 0x255234, 0x356a42, 0x4e8656);
const NETTLE: Ramp = Ramp::new(0x1e3a20, 0x2c5228, 0x3e6c32, 0x56883e, 0x72a24e);
const FOXGLOVE: Ramp = Ramp::new(0x5a2450, 0x82346e, 0xa64a8c, 0xc46aa6, 0xdc92c0);
const STRAW: Ramp = Ramp::new(0x5e4c2c, 0x7e6a3e, 0xa08a54, 0xbca66c, 0xd4c088);
const CHIPS: Ramp = Ramp::new(0x5a4630, 0x7a6244, 0x9a7e58, 0xb89a70, 0xd0b48a);
/// The inside of somewhere dark: brown rather than black, so a creature's outline still reads.
const HOLLOW: Rgba = rgb(0x18120e);
/// The air between the trees, and anything far off.
const DISTANCE: Rgba = rgb(0xa8cac4);
const HIGH_SKY: Rgba = rgb(0x9ccce6);
const SUN: u32 = 0xffe2a2;
const SHADE: Rgba = rgba(0x14241c, 72);

/// The Hill's turf, and its old tree's leaves and bark, in the Hilltop's own tones.
const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const OLD_BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
const HILL_HAZE: f32 = 0.34;

/// Where the track goes out of sight over the rise at the far end, and the far end's horizon.
const FAR_END: (f32, f32) = (208.0, 96.0);
const HORIZON: i32 = 96;
/// The Hill beyond: its middle, how high its summit reaches, how far its face falls to the
/// horizon, and how broad its summit is.
const HILL_MIDDLE: f32 = 210.0;
const HILL_TOP: f32 = 62.0;
const HILL_FALL: f32 = 34.0;
const HILL_BREADTH: f32 = 20.0;

/// The Hill's skyline at the end of the ride: a broad flat summit, rounding over into its flanks.
fn hill_crest(x: f32) -> f32 {
    let off = (x - HILL_MIDDLE) / HILL_BREADTH;
    HILL_TOP + HILL_FALL * (1.0 - (-(off * off * off * off)).exp())
}

/// The Hill at the end of the ride, with whatever stands on it.
const VISTA: Vista = Vista {
    tree: (199.0, 65.0),
    crest: hill_crest,
    spread: 12.0,
    shrink: 8,
    tint: Tint::Haze(DISTANCE, HILL_HAZE),
};

/// Where the trees on either side of the ride stand against the sky: their edge at height `y`,
/// left and right, closing in on the far end.
fn ride_edges(y: i32) -> (i32, i32) {
    let t = (y as f32 / HORIZON as f32).clamp(0.0, 1.0);
    let scallop = |salt: u32| (patches(y, 0, (1, 7), salt) * 8.0) as i32;
    let left = (120.0 + (FAR_END.0 - 30.0 - 120.0) * t.powf(0.7)) as i32 + scallop(901);
    let right = (304.0 + (FAR_END.0 + 30.0 - 304.0) * t.powf(0.7)) as i32 - scallop(902);
    (left, right)
}

/// The middle of the track at `y`, and how far its edges are from the middle.
fn track_middle(y: i32) -> f32 {
    let t = ((y - HORIZON) as f32 / (HEIGHT - HORIZON) as f32).clamp(0.0, 1.0);
    FAR_END.0 + (192.0 - FAR_END.0) * t
}

fn track_half(y: i32) -> f32 {
    3.0 + (y - HORIZON).max(0) as f32 * 0.78
}

/// How near the eye a row of the ground is: 0 at the far end, 1 at the bottom of the picture.
fn nearness(y: i32) -> f32 {
    ((y - HORIZON) as f32 / (HEIGHT - HORIZON) as f32).clamp(0.0, 1.0)
}

/// Everything behind the companions, filling every pixel.
pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    far_country(&mut scene);
    skyline(&mut scene, hilltop, &VISTA);
    woods(&mut scene);
    ground(&mut scene);
    track(&mut scene);
    wall(&mut scene);
    hut(&mut scene);
    woodshed(&mut scene);
    cart(&mut scene);
    for site in &SITES {
        bed(&mut scene, site);
    }
    milestone(&mut scene, MILESTONE_AT);
    plants(&mut scene);
    sunbeams(&mut scene);
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

/// The air in the woods at height `y`: deep shade up under the canopy, lightening down the ride.
fn air(y: i32) -> Rgba {
    const STOPS: [(i32, u32); 4] = [
        (0, 0x1e382f),
        (40, 0x2e5244),
        (80, 0x547e70),
        (110, 0x6a9286),
    ];
    let mut previous = STOPS[0];
    for stop in STOPS {
        if y <= stop.0 {
            let t = (y - previous.0) as f32 / (stop.0 - previous.0).max(1) as f32;
            return mix(rgb(previous.1), rgb(stop.1), t.clamp(0.0, 1.0));
        }
        previous = stop;
    }
    rgb(previous.1)
}

/// `color` as seen through `depth` of the air among the trees (0 nearby, 1 lost in it).
fn hazed(color: Rgba, y: i32, depth: f32) -> Rgba {
    mix(color, air(y), depth)
}

/// `color` as the Hill's distance leaves it.
fn far(color: Rgba) -> Rgba {
    mix(color, DISTANCE, HILL_HAZE)
}

/// A clump of leaves, its rim scalloped into smaller clusters, lit from the upper left and lost
/// in the air by `depth`.
fn clump(scene: &mut Canvas, (cx, cy): (i32, i32), radius: i32, ramp: Ramp, depth: f32, salt: u32) {
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
        ellipse(scene, x, y, size + 1, size + 1, hazed(ramp.edge, y, depth));
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
            put(scene, x, y, hazed(color, y, depth));
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
        ellipse(scene, x, y, size, size, hazed(color, y, depth));
        if light < -0.9 {
            put(scene, x - 1, y - 1, hazed(ramp.shine, y, depth));
        }
    }
}

/// One tree's trunk, rising out of the top of the picture to stand at `base`, lit from the left,
/// its bark furrowed and mossy towards the foot, lost in the air by `depth`.
fn trunk(
    scene: &mut Canvas,
    (left, width): (i32, i32),
    base: i32,
    (depth, sky): (f32, bool),
    salt: u32,
) {
    let lean = pick(left, 0, salt, 3) - 1;
    for y in 0..base {
        let shift = lean * (base - y) / 70;
        let flare = ((y - (base - 5)).max(0) * width) / 12;
        let (from, to) = (left + shift - flare, left + shift + width + flare);
        let span = (to - from).max(1);
        let height = (base - y) as f32 / base as f32;
        let (sky_left, sky_right) = ride_edges(y);
        for x in from..to {
            if sky && y < HORIZON && x > sky_left && x < sky_right {
                continue;
            }
            let across = (x - from) as f32 / span as f32;
            let mut color = if x == from || x == to - 1 {
                BARK.edge
            } else if across < 0.28 {
                BARK.light
            } else if across < 0.62 {
                BARK.base
            } else {
                BARK.shadow
            };
            if width >= 5 && x > from && x < to - 1 {
                let streak = noise(x, (y + pick(x, 1, salt, 9)).div_euclid(5), salt);
                if streak.is_multiple_of(6) {
                    color = mix(color, BARK.edge, 0.7);
                } else if streak % 11 == 1 && across < 0.6 {
                    color = mix(color, BARK.shine, 0.6);
                }
                if height < 0.35
                    && patches(x, y, (3, 7), salt + 1) > 0.38 + height * 1.3 + across * 0.3
                {
                    color = if across < 0.3 {
                        MOSS.light
                    } else if across < 0.65 {
                        MOSS.base
                    } else {
                        MOSS.shadow
                    };
                }
            }
            scene.set(x, y, hazed(color, y, depth));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The sky and the far end
// ---------------------------------------------------------------------------------------------

/// The sky on its own, clouds and all: deep overhead, paling to a warm haze low down, and a
/// fair-weather cloud or two drifting over the ride.
pub fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 8;
    let band = |y: i32| (y * BANDS / HORIZON).min(BANDS - 1);
    for y in 0..HORIZON + 4 {
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
        (196, 22),
        &[(-12, 1, 5), (-3, -3, 7), (8, -1, 6), (17, 2, 4)],
    );
    cloud(scene, (246, 40), &[(-4, 0, 3), (2, -2, 4), (8, 0, 2)]);
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

/// What shows at the far end of the ride: downs a long way off, the Hill in front of them with
/// its path winding up to the old tree on its summit.
fn far_country(scene: &mut Canvas) {
    for x in 150..270 {
        let ridge =
            (74.0 + (x as f32 * 0.07).sin() * 2.5 + (x as f32 * 0.21).sin() * 0.8).round() as i32;
        for y in ridge..HORIZON + 4 {
            let color = if y == ridge {
                mix(FAR_HILL, TURF.shadow, 0.3)
            } else {
                FAR_HILL
            };
            scene.set(x, y, mix(color, DISTANCE, 0.5));
        }
    }
    hill(scene);
    hill_path(scene);
    old_tree(scene);
}

/// The Hill: a dome of turf lit on its left flank and shaded on its right, outlined along its
/// crest.
fn hill(scene: &mut Canvas) {
    for x in 160..262 {
        let crest = hill_crest(x as f32 + 0.5).round() as i32;
        if crest >= HORIZON {
            continue;
        }
        let across = (x as f32 + 0.5 - HILL_MIDDLE) / HILL_BREADTH;
        for y in crest..HORIZON + 4 {
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
}

/// The path up the Hill, winding across its face to the summit by the old tree.
fn hill_path(scene: &mut Canvas) {
    let tree = VISTA.tree.0.round() as i32;
    let summit = (tree + 2, hill_crest(tree as f32 + 2.5).round() as i32 + 1);
    let bends = [(214, HORIZON), (204, 84), (216, 74), (203, 68), summit];
    for pair in bends.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        line(scene, from, to, far(mix(PATH, TURF.base, 0.25)));
        line(
            scene,
            (from.0, from.1 + 1),
            (to.0, to.1 + 1),
            far(mix(TURF.shadow, rgb(0x8e7553), 0.3)),
        );
    }
}

/// The old tree's crown as overlapping masses `(x, y, radius)` from the foot of its trunk, as the
/// Hilltop has it in its own pixels.
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

/// The old tree on the summit, at the same scale as the pieces standing about it.
fn old_tree(scene: &mut Canvas) {
    let scale = VISTA.shrink as f32;
    let foot_x = VISTA.tree.0.round() as i32;
    let foot_y = hill_crest(VISTA.tree.0).round() as i32 + 1;
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
    let mut layer = Canvas::new(36, 26);
    let origin = (foot_x - 18, foot_y - 23);
    for ly in 0..26 {
        for lx in 0..36 {
            let (hx, hy) = (
                (origin.0 + lx - foot_x) as f32 * scale + scale * 0.5,
                (origin.1 + ly - foot_y) as f32 * scale + scale * 0.5,
            );
            let mass = OLD_TREE_CROWN
                .iter()
                .copied()
                .filter(|&(cx, cy, r)| (hx - cx).powi(2) + (hy - cy).powi(2) <= (r + 3.0).powi(2))
                .min_by(|a, b| a.2.total_cmp(&b.2));
            let Some((mx, my, r)) = mass else {
                continue;
            };
            let local = 0.5 - ((hx - mx) + (hy - my)) / (3.0 * r);
            let whole = 1.0 - (hx + 53.0) / 110.0 * 0.45 - (hy + 112.0) / 75.0 * 0.55;
            let fleck = (noise(origin.0 + lx, origin.1 + ly, 521) % 100) as f32 / 100.0 * 0.12;
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

// ---------------------------------------------------------------------------------------------
// The woods on either side
// ---------------------------------------------------------------------------------------------

/// Where the trees stand along either side of the ride, as the row their feet are on at `x`:
/// low and near at the sides of the picture, rising towards the far end, where the ride runs out
/// of sight over the rise.
pub fn treeline(x: i32) -> i32 {
    const LEFT_END: f32 = 184.0;
    const RIGHT_END: f32 = 232.0;
    let x = x as f32;
    let row = if x < LEFT_END {
        HORIZON as f32 + 2.0 + (LEFT_END - x) * 0.17
    } else if x > RIGHT_END {
        HORIZON as f32 + 2.0 + (x - RIGHT_END) * 0.19
    } else {
        HORIZON as f32
    };
    row.round() as i32
}

/// How deep in the distance a point on the treeline is: 0 at the sides of the picture, 1 at the
/// far end of the ride.
fn far_along(x: i32) -> f32 {
    let off = (x as f32 - FAR_END.0).abs() / 200.0;
    (1.0 - off).clamp(0.0, 1.0)
}

/// The woods closing in on the ride from both sides: the air under the trees, trunks going back
/// along the treeline towards the far end, smaller and hazier the further they are, the
/// undergrowth along their feet, and the canopy over it all, its edges scalloped against the sky.
fn woods(scene: &mut Canvas) {
    // Under the trees, everything above their feet but the wedge of sky over the ride.
    for x in 0..WIDTH {
        let foot = treeline(x);
        for y in 0..foot + 2 {
            let (left, right) = ride_edges(y);
            if x > left && x < right && y < HORIZON {
                continue;
            }
            let level = (y + pick(x, y, 911, 5) - 2).div_euclid(6) * 6;
            scene.set(x, y, air(level));
        }
    }
    // Trunks deep in the woods, then nearer ones along the treeline, back to front.
    for rank in 0..3 {
        let deep = 0.62 - rank as f32 * 0.22;
        for index in 0..26 {
            let i = index + rank * 40;
            let x = pick(i, 0, 912 + rank as u32, WIDTH + 20) - 10;
            if (x - FAR_END.0 as i32).abs() < 28 {
                continue;
            }
            let along = far_along(x);
            let foot = treeline(x) - (2 - rank) * 3 - pick(i, 1, 912, 3);
            let width = ((1.0 - along) * (4.0 + rank as f32 * 3.0)) as i32 + 1 + pick(i, 2, 912, 2);
            let depth = (deep + along * 0.35).min(0.85);
            trunk(scene, (x, width), foot, (depth, true), 920 + i as u32);
        }
    }
    // The near trunks at either side, rising out of the top of the picture.
    for (index, &(x, width)) in [(-6, 16), (30, 9), (346, 11), (372, 18)].iter().enumerate() {
        trunk(
            scene,
            (x, width),
            treeline(x + width / 2) + 2,
            (0.04, true),
            980 + index as u32,
        );
    }
    // Undergrowth along the feet of the trees, bigger nearer the eye.
    let mut x = -10;
    let mut index = 0;
    while x < WIDTH + 10 {
        let along = far_along(x);
        let r = (2.0 + (1.0 - along) * 9.0) as i32 + pick(index, 0, 913, 3);
        if (x - FAR_END.0 as i32).abs() > 22 {
            clump(
                scene,
                (x, treeline(x) - r / 3 + pick(index, 1, 913, 3)),
                r,
                UNDERGROWTH,
                along * 0.5,
                930 + index as u32 * 5,
            );
        }
        x += (r * 3 / 2).max(3);
        index += 1;
    }
    // The canopy over the woods, leaning in over the ride from both sides.
    for (index, &(x, y, r)) in [
        (14, 4, 30),
        (58, -6, 26),
        (64, 34, 22),
        (104, 4, 26),
        (128, -6, 20),
        (138, 26, 16),
        (154, 48, 11),
        (168, 68, 7),
        (180, 82, 4),
        (318, -6, 22),
        (296, 8, 20),
        (280, 30, 14),
        (264, 52, 10),
        (250, 70, 7),
        (238, 84, 4),
        (346, 20, 26),
        (384, 0, 28),
        (380, 44, 18),
        (8, 46, 18),
    ]
    .iter()
    .enumerate()
    {
        let ramp = if index % 3 == 0 { BEECH } else { OAK };
        let depth = (far_along(x) * 0.3).clamp(0.0, 0.3);
        clump(scene, (x, y), r, ramp, depth, 950 + index as u32 * 7);
    }
}

// ---------------------------------------------------------------------------------------------
// The ground and the track
// ---------------------------------------------------------------------------------------------

/// The woods floor either side of the track: earth under leaf litter, moss in cushions, and
/// fallen leaves bigger nearer the eye; softening into the air towards the far end.
fn ground(scene: &mut Canvas) {
    for y in HORIZON..HEIGHT {
        for x in 0..WIDTH {
            if y <= treeline(x) && on_track(x, y) <= 0.0 {
                continue;
            }
            let soil = patches(x, y, (18, 6), 961) + (noise(x, y, 962) % 100) as f32 / 600.0;
            let mut color = if soil < 0.4 {
                LITTER.shadow
            } else if soil < 0.72 {
                mix(LITTER.shadow, LITTER.base, 0.6)
            } else {
                LITTER.base
            };
            let mossy = patches(x, y, (22, 7), 963) > 0.62;
            if mossy {
                let cushion = patches(x, y, (4, 3), 964);
                color = if cushion > 0.7 {
                    MOSS.light
                } else if cushion > 0.35 {
                    MOSS.base
                } else {
                    MOSS.shadow
                };
            }
            let amount = (1.0 - nearness(y)).powi(3) * 0.5;
            scene.set(x, y, mix(color, air(HORIZON + 8), amount));
        }
    }
    // Fallen leaves, scattered, bigger and further apart nearer the eye.
    let fallen = [
        LITTER.light,
        rgb(0x8e6434),
        rgb(0x7a4630),
        rgb(0x987a40),
        LITTER.base,
        mix(LITTER.base, MOSS.shadow, 0.4),
    ];
    let mut row = HORIZON + 8;
    while row < HEIGHT + 2 {
        let near = nearness(row);
        let (step_x, step_y) = (3 + (near * 4.0) as i32, 2 + (near * 1.6) as i32);
        for (index, gx) in (-4..WIDTH + 4).step_by(step_x as usize).enumerate() {
            let i = index as i32;
            let x = gx + pick(i, row, 965, step_x);
            let y = row + pick(i, row + 1, 965, step_y);
            if !chance(x, y, 966, 150) || on_track(x, y) > 0.0 || y <= treeline(x) + 1 {
                continue;
            }
            let long = 1 + (near * 1.6) as i32 + pick(i, row + 2, 965, 2);
            let color = fallen[pick(i, row + 3, 965, fallen.len() as i32) as usize];
            hline(scene, x - long + 1, y, long * 2 - 1, color);
            put(scene, x - long + 1, y, mix(color, LITTER.shine, 0.35));
            hline(
                scene,
                x - long + 2,
                y + 1,
                (long * 2 - 2).max(1),
                rgba(0x1e140e, 100),
            );
        }
        row += step_y;
    }
}

/// How far into the track a point is: 0 off it, rising to 1 well inside its edges.
fn on_track(x: i32, y: i32) -> f32 {
    if y < HORIZON {
        return 0.0;
    }
    let off = (x as f32 - track_middle(y)).abs() / track_half(y);
    ((1.15 - off) * 4.0).clamp(0.0, 1.0)
}

/// Where across the track a wheel rut runs, as a share of the half-width out from the middle.
const RUTS: f32 = 0.55;

/// How far into a rut a point is: 0 out of it, 1 in its bottom.
fn in_rut(x: i32, y: i32) -> f32 {
    if y < HORIZON {
        return 0.0;
    }
    let half = track_half(y);
    let width = 0.8 + (y - HORIZON) as f32 * 0.075;
    let middle = track_middle(y);
    [-1.0, 1.0]
        .iter()
        .map(|side| {
            let at = middle + side * half * RUTS;
            let wobble = (patches(y, side.to_bits() as i32 & 7, (1, 9), 971) - 0.5) * width * 0.5;
            (1.0 - (x as f32 - at - wobble).abs() / width).clamp(0.0, 1.0)
        })
        .fold(0.0, f32::max)
}

/// The old track: two wheel ruts of bare earth, puddled here and there, with grass grown up
/// between them and along their edges, running away to the far end.
fn track(scene: &mut Canvas) {
    for y in HORIZON..HEIGHT {
        for x in 0..WIDTH {
            let inside = on_track(x, y);
            if inside <= 0.0 {
                continue;
            }
            let near = nearness(y);
            let rut = in_rut(x, y);
            // Trodden earth, grass coming through it, the ruts worn down to mud.
            let turf = patches(x, y, (7, 3), 972) + (noise(x, y, 973) % 100) as f32 / 500.0;
            let mut color = if rut > 0.0 {
                let level = if rut > 0.75 {
                    1
                } else if rut > 0.4 {
                    2
                } else {
                    3
                };
                tone(RUT, level)
            } else if turf > 0.52 - (1.0 - inside) * 0.3 {
                let blade = noise(x, y.div_euclid(2), 974) % 6;
                match blade {
                    0 => GRASS.light,
                    1 | 2 => GRASS.base,
                    _ => mix(GRASS.base, GRASS.shadow, 0.5),
                }
            } else if turf > 0.34 {
                mix(EARTH.light, LITTER.base, 0.5)
            } else {
                EARTH.light
            };
            // The near lip of a rut catches the light; its far side is in shadow.
            if rut > 0.0 && rut < 0.4 && in_rut(x, y - 1) > rut {
                color = RUT.shine;
            }
            let amount = (1.0 - near).powi(3) * 0.5;
            put(scene, x, y, mix(color, air(HORIZON + 8), amount));
        }
    }
    // Puddles in the ruts, the sky in them.
    for &(y, side, long) in &[
        (132, -1.0, 7),
        (168, 1.0, 12),
        (194, -1.0, 15),
        (150, 1.0, 5),
    ] {
        let middle = track_middle(y) + side * track_half(y) * RUTS;
        let half = long / 2;
        for dy in -1..=1 {
            for dx in -half..=half {
                let (x, py) = (middle as i32 + dx, y + dy);
                let edge =
                    (dx.abs() as f32 / half as f32).powi(2) + (dy.abs() as f32).powi(2) * 0.5;
                if edge > 1.0 {
                    continue;
                }
                let color = if dy < 0 {
                    PUDDLE.edge
                } else if dx < -half / 3 && dy == 0 {
                    PUDDLE.light
                } else {
                    PUDDLE.base
                };
                put(scene, x, py, color);
            }
        }
        put(scene, middle as i32 - half / 2, y, PUDDLE.shine);
    }
}

// ---------------------------------------------------------------------------------------------
// Dry stone
// ---------------------------------------------------------------------------------------------

/// Dry stone laid in courses from `foot` up to `top` at each column across `from..to`, the
/// courses `course` pixels tall at that column: flat stones of every length with dark gaps
/// between them and no mortar, each lit along its top and left and shaded along its bottom and
/// right, with lichen here and there. `top` and `foot` say where the stonework is at a column.
fn drystone(
    scene: &mut Canvas,
    (from, to): (i32, i32),
    top: impl Fn(i32) -> i32,
    foot: impl Fn(i32) -> i32,
    course: impl Fn(i32) -> i32,
    salt: u32,
) {
    for x in from..to {
        let (top, foot, tall) = (top(x), foot(x), course(x).max(2));
        for y in top..foot {
            let up = foot - 1 - y;
            let row = up / tall;
            let within = up % tall;
            // Each course's stones are their own lengths, staggered from the course below.
            let long = tall * 2 + pick(row, 0, salt, tall + 2);
            let shifted = x + row * 5 + pick(row, 1, salt, long);
            let stone = shifted.div_euclid(long);
            let across = shifted.rem_euclid(long);
            let gap = within == tall - 1 || across == 0;
            let color = if gap {
                if chance(x, y, salt + 2, 40) {
                    MOSS.shadow
                } else {
                    DRYSTONE.edge
                }
            } else {
                let base = match noise(stone, row, salt + 3) % 5 {
                    0 => mix(DRYSTONE.base, DRYSTONE.shadow, 0.5),
                    1 => mix(DRYSTONE.base, DRYSTONE.light, 0.35),
                    2 => mix(DRYSTONE.base, LITTER.base, 0.25),
                    _ => DRYSTONE.base,
                };
                if within == tall - 2 || across == 1 {
                    if within == tall - 2 {
                        mix(base, DRYSTONE.light, 0.55)
                    } else {
                        mix(base, DRYSTONE.light, 0.3)
                    }
                } else if within == 0 || across == long - 1 {
                    mix(base, DRYSTONE.shadow, 0.6)
                } else if chance(x, y, salt + 4, 14) {
                    LICHEN.light
                } else {
                    base
                }
            };
            put(scene, x, y, color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The wall on the right
// ---------------------------------------------------------------------------------------------

/// Where the wall's foot runs along the right of the track at `x`, and how tall it stands there:
/// low and far off near the end of the ride, tall nearest the eye.
fn wall_foot(x: i32) -> i32 {
    (104.0 + (x - 238) as f32 * (52.0 / 146.0)).round() as i32
}

fn wall_height(x: i32) -> i32 {
    (5.0 + (x - 238) as f32 * (17.0 / 146.0)).round() as i32
}

/// Where the wall has fallen in, behind the heap of what came down.
const FALLEN: (i32, i32) = (268, 346);

/// How much of the wall still stands at `x`: all of it, or along the fallen stretch a ragged
/// stump with gaps.
fn wall_top(x: i32) -> i32 {
    let (foot, tall) = (wall_foot(x), wall_height(x));
    let ragged = if (FALLEN.0..FALLEN.1).contains(&x) {
        let into = ((x - FALLEN.0).min(FALLEN.1 - x) as f32 / 10.0).clamp(0.0, 1.0);
        let stump = 0.3 + patches(x, 0, (5, 1), 1011) * 0.25;
        1.0 - into * (1.0 - stump)
    } else {
        1.0
    };
    foot - (tall as f32 * ragged).round() as i32
}

/// A dry-stone wall running along the right of the track towards the far end, its coping stones
/// set on edge along its top and furred with moss, fallen in along its middle where the heap is.
fn wall(scene: &mut Canvas) {
    let from = 238;
    // The shade it casts along its foot, on the track side.
    for x in from..WIDTH {
        for dy in 0..3 {
            put(
                scene,
                x - dy,
                wall_foot(x) + dy,
                rgba(0x14241c, 60 - dy as u8 * 15),
            );
        }
    }
    drystone(
        scene,
        (from, WIDTH),
        wall_top,
        wall_foot,
        |x| (wall_height(x) / 5).max(2),
        1001,
    );
    // Coping stones on edge along the top where it still stands, and moss over them.
    for x in from..WIDTH {
        let fallen = wall_top(x) > wall_foot(x) - wall_height(x);
        let top = wall_top(x);
        if !fallen {
            let tall = (wall_height(x) / 6).clamp(1, 4);
            let column = x.div_euclid(2);
            for dy in 0..tall {
                let color = if x % 2 == 0 {
                    DRYSTONE.light
                } else if noise(column, dy, 1002).is_multiple_of(3) {
                    DRYSTONE.shadow
                } else {
                    DRYSTONE.base
                };
                put(scene, x, top - 1 - dy, color);
            }
            if chance(x, top, 1003, 150) {
                put(scene, x, top - tall - 1, MOSS.light);
                put(scene, x, top - tall, MOSS.base);
            } else {
                put(scene, x, top - tall - 1, DRYSTONE.edge);
            }
        } else if chance(x, top, 1004, 120) {
            put(scene, x, top - 1, MOSS.base);
            put(scene, x, top, MOSS.shadow);
        }
    }
    // Ferns and grass at its foot, here and there.
    for (index, x) in (from + 4..WIDTH).step_by(17).enumerate() {
        if (FALLEN.0 - 6..FALLEN.1 + 6).contains(&x) {
            continue;
        }
        let near = (x - from) as f32 / 146.0;
        let foot = wall_foot(x) + 1;
        let size = (4.0 + near * 7.0) as i32;
        for frond in 0..4 {
            let angle =
                -1.9 + frond as f32 * 0.4 + pick(index as i32, frond, 1005, 3) as f32 * 0.05;
            stem_frond(scene, (x + frond * 2 - 3, foot), angle, size);
        }
    }
}

/// A fern frond from `from`, leaning out at `angle`, `length` long: a stem with leaflets either
/// side, lit along its upper edge.
fn stem_frond(scene: &mut Canvas, from: (i32, i32), angle: f32, length: i32) {
    let (dx, dy) = (angle.cos(), angle.sin());
    for step in 0..length {
        let t = step as f32 / length as f32;
        let droop = t * t * length as f32 * 0.35;
        let x = from.0 as f32 + dx * step as f32;
        let y = from.1 as f32 + dy * step as f32 + droop;
        let (x, y) = (x.round() as i32, y.round() as i32);
        put(scene, x, y, FERN.shadow);
        let leaf = ((1.0 - t) * 3.0) as i32 + 1;
        if step % 2 == 0 {
            for side in [-1, 1] {
                for reach in 1..=leaf {
                    let color = if side < 0 { FERN.light } else { FERN.base };
                    put(scene, x + side * reach / 2, y - reach.min(2) + 1, color);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The hut on the left
// ---------------------------------------------------------------------------------------------

/// The hut's walls, left to right, the row its footing stands on, and where its eaves and ridge
/// are.
const HUT: (i32, i32) = (2, 80);
const HUT_FOOT: i32 = 151;
const EAVES: i32 = 108;
const RIDGE: i32 = 80;

/// A timber of the hut's frame, upright or across, silvered with age, lit from the left and top.
fn timber(scene: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32) {
    for py in y..y + h {
        for px in x..x + w {
            let upright = h > w;
            let (along, across) = if upright {
                (py - y, px - x)
            } else {
                (px - x, py - y)
            };
            let thick = if upright { w } else { h };
            let mut color = if across == thick - 1 {
                SILVER_OAK.edge
            } else if across == 0 {
                SILVER_OAK.light
            } else {
                SILVER_OAK.base
            };
            if across > 0
                && across < thick - 1
                && noise(along.div_euclid(3), across, salt).is_multiple_of(4)
            {
                color = SILVER_OAK.shadow;
            }
            put(scene, px, py, color);
        }
    }
}

/// The woodcutter's hut, long abandoned: a stone footing, a silvered timber frame with its daub
/// cracked and fallen out in places to show the wattle, the door hanging open from one hinge, a
/// broken window with one shutter left, a shingled roof furred with moss and fallen in at its
/// right end, its rafters showing, a stone chimney leaning, and ivy over it all.
fn hut(scene: &mut Canvas) {
    let (left, right) = HUT;
    // The shade it casts behind it, falling to the right.
    for y in EAVES..HUT_FOOT + 2 {
        for x in right..right + 10 - (HUT_FOOT - y) / 5 {
            put(scene, x, y, rgba(0x14241c, 50));
        }
    }
    // The footing.
    drystone(
        scene,
        (left, right),
        |_| HUT_FOOT - 8,
        |_| HUT_FOOT,
        |_| 4,
        1101,
    );
    // The walls: daub panels in the timber frame.
    let posts = [left, 26, 52, right - 3];
    for y in EAVES + 3..HUT_FOOT - 8 {
        for x in left..right {
            let blotch = patches(x, y, (3, 3), 1102);
            let mut color = if blotch > 0.7 {
                DAUB.light
            } else if blotch < 0.25 {
                DAUB.shadow
            } else {
                DAUB.base
            };
            // Damp coming up from the footing, and weathering down from the eaves.
            if y > HUT_FOOT - 14 && chance(x, y, 1103, 90) {
                color = mix(color, MOSS.shadow, 0.5);
            }
            if y < EAVES + 7 {
                color = mix(color, DAUB.shadow, 0.4);
            }
            put(scene, x, y, color);
        }
    }
    // Daub fallen out of two panels, showing the woven hazel behind: upright stakes with withies
    // woven in and out of them, and the dark between.
    for &(cx, cy, rx, ry) in &[(14, 125, 6, 6), (64, 137, 5, 4)] {
        for y in cy - ry..=cy + ry {
            for x in cx - rx..=cx + rx {
                let (u, v) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
                let ragged = (noise(x, y, 1104) % 100) as f32 / 260.0;
                let out = u * u + v * v;
                if out > 1.0 - ragged {
                    continue;
                }
                let stake = (x - cx).rem_euclid(4) == 0;
                let withy = (y - cy).rem_euclid(2) == 0
                    && ((x - cx).div_euclid(4) + (y - cy).div_euclid(2)).rem_euclid(2) == 0;
                let color = if out > 0.8 - ragged {
                    // The broken edge of the daub, its thickness showing.
                    DAUB.shadow
                } else if stake {
                    LITTER.light
                } else if withy {
                    if (x - cx).rem_euclid(4) == 1 {
                        CHIPS.light
                    } else {
                        CHIPS.base
                    }
                } else {
                    HOLLOW
                };
                put(scene, x, y, color);
            }
        }
    }
    // Cracks in the daub.
    for &(from, to) in &[
        ((38, 112), (42, 120)),
        ((70, 112), (66, 118)),
        ((10, 132), (16, 138)),
    ] {
        line(scene, from, to, DAUB.edge);
    }
    // The frame: posts, the wall plate and the sill, and a brace.
    for (index, &x) in posts.iter().enumerate() {
        timber(
            scene,
            (x, EAVES),
            (3, HUT_FOOT - 8 - EAVES),
            1110 + index as u32,
        );
    }
    timber(scene, (left, EAVES), (right - left, 3), 1115);
    timber(scene, (left, HUT_FOOT - 10), (right - left, 3), 1116);
    for step in 0..20 {
        let (x, y) = (posts[0] + 3 + step, HUT_FOOT - 11 - step * 3 / 2);
        put(scene, x, y, SILVER_OAK.light);
        put(scene, x + 1, y, SILVER_OAK.base);
        put(scene, x + 2, y, SILVER_OAK.edge);
    }
    hut_door(scene);
    hut_window(scene);
    hut_roof(scene);
    chimney(scene);
    ivy(
        scene,
        &[(4, 150), (3, 136), (6, 122), (4, 110), (8, 98), (12, 86)],
        1120,
    );
}

/// The doorway, dark, and the plank door hanging open from its upper hinge, swung out and askew.
fn hut_door(scene: &mut Canvas) {
    let (left, right, top) = (32, 47, 116);
    let bottom = HUT_FOOT - 10;
    for y in top..bottom {
        for x in left..right {
            let deep = if y < top + 2 || x < left + 2 {
                mix(HOLLOW, LITTER.shadow, 0.2)
            } else {
                HOLLOW
            };
            put(scene, x, y, deep);
        }
    }
    hline(scene, left - 1, top - 1, right - left + 2, SILVER_OAK.edge);
    // Something glimpsed inside: the edge of a table, and a cobweb.
    hline(scene, right - 6, top + 14, 5, LITTER.shadow);
    for (dx, dy) in [(1, 1), (2, 2), (3, 3), (1, 3), (3, 1)] {
        put(scene, left + 1 + dx, top + 1 + dy, rgba(0xd8d8d0, 70));
    }
    // The door, swung out towards the eye on its left side, foreshortened and dropped at its
    // free corner.
    for x in 0..7 {
        let drop = x / 2;
        let (column, from, to) = (left - 6 + x, top + 1 + drop, bottom + drop.min(1));
        for y in from..to {
            let plank = x % 3 == 2;
            let mut color = if plank {
                PLANKED.shadow
            } else if x == 0 {
                PLANKED.light
            } else {
                PLANKED.base
            };
            if (y - from) == 4 || (y - from) == (to - from) - 5 {
                color = PLANKED.edge;
            }
            if noise(column, y.div_euclid(3), 1130).is_multiple_of(7) {
                color = mix(color, MOSS.base, 0.5);
            }
            put(scene, column, y, color);
        }
        put(scene, column, to, PLANKED.edge);
    }
    // The hinge it hangs from, and the one it has come away from.
    put(scene, left - 1, top + 5, IRON.light);
    put(scene, left, top + 5, IRON.base);
    put(scene, left, bottom - 5, IRON.shadow);
}

/// The planks of the hut's door and its woodshed.
const PLANKED: Ramp = Ramp::new(0x3a2e26, 0x56463a, 0x6e5c4a, 0x8a765e, 0xa69278);

/// The window: dark inside, its lattice broken, one shutter hanging from a hinge.
fn hut_window(scene: &mut Canvas) {
    let (left, top, w, h) = (58, 117, 14, 11);
    for y in top..top + h {
        for x in left..left + w {
            put(scene, x, y, HOLLOW);
        }
    }
    // What is left of the lattice.
    for step in 0..h {
        put(scene, left + step, top + step, SILVER_OAK.shadow);
        if step < 6 {
            put(scene, left + w - 1 - step, top + step, SILVER_OAK.shadow);
        }
    }
    vline(scene, left + w / 2, top, h, SILVER_OAK.base);
    timber(scene, (left - 2, top + h), (w + 4, 2), 1131);
    timber(scene, (left - 1, top - 2), (w + 2, 2), 1132);
    // The shutter, hanging from its top hinge, swung down and out at an angle.
    for x in 0..6 {
        for y in 0..h + 2 {
            let (px, py) = (left + w + 1 + x, top + y + x / 2);
            let color = if x == 5 || y == h + 1 {
                PLANKED.edge
            } else if x == 0 {
                PLANKED.light
            } else if x == 3 {
                PLANKED.shadow
            } else {
                PLANKED.base
            };
            put(scene, px, py, color);
        }
    }
}

/// The roof: wooden shingles in courses, furred with moss, with barge boards along the ends; at
/// its right end it has fallen in, the rafters showing against the dark inside and the shingles
/// slid down into a ragged lip.
fn hut_roof(scene: &mut Canvas) {
    let (left, right) = (HUT.0 - 4, HUT.1 + 4);
    let at = |y: i32| {
        let t = (y - RIDGE) as f32 / (EAVES - RIDGE) as f32;
        let inset = ((1.0 - t) * 10.0) as i32;
        (left + inset, right - inset)
    };
    // Fallen in at the right end: from a ragged edge just under the ridge, widening down to the
    // eaves, where the roof has dropped.
    let fallen = |x: i32, y: i32| {
        let down = (y - (RIDGE + 5)) as f32 / (EAVES - RIDGE - 5) as f32;
        if down < 0.0 {
            return false;
        }
        let ragged = (patches(x, y, (2, 3), 1140) - 0.5) * 5.0;
        let (from, to) = (
            58.0 - down * 10.0 + ragged,
            72.0 + down * 2.0 + ragged * 0.5,
        );
        (x as f32) > from && (x as f32) < to && y < EAVES - 1
    };
    for y in RIDGE..EAVES + 2 {
        let (from, to) = at(y);
        let course = (y - RIDGE).div_euclid(4);
        let within = (y - RIDGE).rem_euclid(4);
        for x in from..to {
            if fallen(x, y) {
                continue;
            }
            let shifted = x + if course % 2 == 1 { 2 } else { 0 };
            let across = shifted.rem_euclid(5);
            let mut color = if within == 3 {
                SHINGLE.edge
            } else if across == 0 {
                SHINGLE.shadow
            } else if within == 0 {
                SHINGLE.light
            } else {
                SHINGLE.base
            };
            // Lit from the left along the roof, shaded under the ridge.
            if x < from + 3 && within != 3 {
                color = mix(color, SHINGLE.light, 0.4);
            }
            // Moss in cushions, thickest low down where the damp sits.
            let damp = (y - RIDGE) as f32 / (EAVES - RIDGE) as f32;
            if patches(x, y, (6, 3), 1141) > 0.75 - damp * 0.25 {
                color = if within == 0 {
                    MOSS.light
                } else if within == 3 {
                    MOSS.edge
                } else {
                    MOSS.base
                };
            }
            // A shingle missing here and there.
            if within != 3 && noise(shifted.div_euclid(5), course, 1142).is_multiple_of(23) {
                color = HOLLOW;
            }
            put(scene, x, y, color);
        }
        put(scene, from - 1, y, SHINGLE.edge);
        put(scene, to, y, SHINGLE.edge);
    }
    // The ridge, and the eaves' edge with the shadow under it.
    hline(
        scene,
        at(RIDGE).0,
        RIDGE - 1,
        at(RIDGE).1 - at(RIDGE).0,
        SHINGLE.edge,
    );
    hline(
        scene,
        at(RIDGE).0,
        RIDGE,
        at(RIDGE).1 - at(RIDGE).0,
        SHINGLE.light,
    );
    hline(scene, left, EAVES + 2, right - left, SHINGLE.edge);
    hline(
        scene,
        left + 2,
        EAVES + 3,
        right - left - 4,
        rgba(0x14241c, 110),
    );
    // The hole: dark inside, the rafters across it, and the broken edge of the shingles.
    for y in RIDGE..EAVES + 2 {
        let (from, to) = at(y);
        for x in from..to {
            if !fallen(x, y) {
                continue;
            }
            let edge = !fallen(x, y - 1) || !fallen(x - 1, y) || !fallen(x + 1, y);
            put(scene, x, y, if edge { SHINGLE.edge } else { HOLLOW });
        }
    }
    for &(top, bottom) in &[
        ((54, RIDGE + 3), (48, EAVES)),
        ((62, RIDGE + 3), (57, EAVES)),
        ((70, RIDGE + 3), (66, EAVES - 3)),
    ] {
        for (step, &(x, y)) in trace_points(top, bottom).iter().enumerate() {
            if !fallen(x, y) && !fallen(x + 1, y) {
                continue;
            }
            put(scene, x, y, SILVER_OAK.light);
            put(
                scene,
                x + 1,
                y,
                if step % 5 == 0 {
                    SILVER_OAK.edge
                } else {
                    SILVER_OAK.shadow
                },
            );
        }
    }
    // A purlin across the hole, broken, its end dropped.
    for (step, &(x, y)) in trace_points((46, 96), (58, 96))
        .iter()
        .chain(trace_points((60, 97), (70, 103)).iter())
        .enumerate()
    {
        if !fallen(x, y) {
            continue;
        }
        put(scene, x, y, SILVER_OAK.light);
        put(
            scene,
            x,
            y + 1,
            if step % 4 == 0 {
                SILVER_OAK.edge
            } else {
                SILVER_OAK.shadow
            },
        );
    }
    // Shingles slid down into a ragged lip over the eaves below the hole, the eaves sagging there.
    for x in 46..76 {
        let sag = 1 + (3.0 * (1.0 - ((x - 61) as f32 / 15.0).powi(2))).max(0.0) as i32;
        let lip = EAVES + sag + pick(x, 0, 1143, 2);
        put(scene, x, lip - 1, SHINGLE.light);
        put(scene, x, lip, SHINGLE.base);
        put(scene, x, lip + 1, SHINGLE.edge);
        if x % 4 == 1 {
            put(scene, x, lip + 2, SHINGLE.shadow);
        }
    }
}

/// Every pixel along a straight line.
fn trace_points(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let ((mut x, mut y), (x1, y1)) = (from, to);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
    let mut error = dx + dy;
    loop {
        points.push((x, y));
        if (x, y) == (x1, y1) {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
    points
}

/// The chimney at the hut's left end: dry stone, leaning a little, a stone gone from its top and
/// a tuft of grass growing there instead.
fn chimney(scene: &mut Canvas) {
    let (left, top, bottom) = (8, 62, 98);
    let lean = |y: i32| (bottom - y) / 12;
    drystone(
        scene,
        (left - 1, left + 15),
        |x| {
            let x = x - left;
            if (8..12).contains(&x) { top + 3 } else { top }
        },
        |_| bottom,
        |_| 3,
        1150,
    );
    // The lean: shift each row over by how far up it is.
    let mut leaned = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in top..bottom {
        for x in left - 1..left + 15 {
            leaned.set(x + lean(y), y, scene.get(x, y));
        }
    }
    for y in top..bottom {
        for x in left - 1..left + 16 {
            let pixel = leaned.get(x, y);
            if pixel.a > 0 {
                scene.set(x, y, pixel);
            }
        }
    }
    // Its left side catches the light; its right is in its own shadow.
    for y in top..bottom {
        put(scene, left - 1 + lean(y), y, DRYSTONE.edge);
        put(scene, left + 14 + lean(y), y, DRYSTONE.edge);
        put(scene, left + 13 + lean(y), y, rgba(0x14241c, 70));
    }
    hline(scene, left - 1 + lean(top), top - 1, 16, DRYSTONE.edge);
    for (dx, tall) in [(1, 3), (3, 4), (4, 2), (6, 3)] {
        vline(scene, left + dx + lean(top), top - tall, tall, GRASS.base);
        put(scene, left + dx + lean(top), top - tall, GRASS.light);
    }
}

/// Ivy climbing along `path`, its stem hidden under leaves in threes, dark and glossy.
fn ivy(scene: &mut Canvas, path: &[(i32, i32)], salt: u32) {
    for (index, pair) in path.windows(2).enumerate() {
        for (step, &(x, y)) in trace_points(pair[0], pair[1]).iter().enumerate() {
            put(scene, x, y, IVY.edge);
            if (step + index) % 3 != 0 {
                continue;
            }
            let side = if (step / 3 + index) % 2 == 0 { -2 } else { 2 };
            let (lx, ly) = (x + side, y - 1);
            put(scene, lx, ly, IVY.base);
            put(scene, lx - 1, ly, IVY.light);
            put(scene, lx + 1, ly, IVY.base);
            put(scene, lx, ly - 1, IVY.light);
            put(scene, lx, ly + 1, IVY.shadow);
            if chance(x, y, salt, 60) {
                put(scene, lx - 1, ly - 1, IVY.shine);
            }
        }
    }
}

/// The woodshed against the hut's right end: a lean-to of boards, its back still against the
/// wall with logs stacked under it, its front post broken and the rest of its roof come down.
fn woodshed(scene: &mut Canvas) {
    let back = HUT.1;
    // Inside, in the dark, the woodpile: log ends in rows.
    for y in 114..HUT_FOOT {
        for x in back..back + 24 {
            put(scene, x, y, mix(HOLLOW, LITTER.shadow, 0.3));
        }
    }
    // Log ends, each a ring of bark round pale sawn wood with its growth rings, in rows.
    const LOG: [&str; 5] = [".bBb.", "bwlwb", "Blrwb", "bwwsb", ".bbb."];
    for row in 0..5 {
        for col in 0..4 {
            let (left, top) = (back + 1 + col * 5 + (row % 2) * 2, HUT_FOOT - 6 - row * 5);
            if top < 117 || left + 5 > back + 24 {
                continue;
            }
            for (dy, line_of) in LOG.iter().enumerate() {
                for (dx, code) in line_of.bytes().enumerate() {
                    let color = match code {
                        b'b' => BARK.shadow,
                        b'B' => BARK.base,
                        b'w' => CHIPS.light,
                        b'l' => CHIPS.shine,
                        b'r' => CHIPS.shadow,
                        b's' => CHIPS.base,
                        _ => continue,
                    };
                    put(scene, left + dx as i32, top + dy as i32, color);
                }
            }
        }
    }
    // The roof boards: whole over the woodpile, then broken and come down to the ground.
    let roof = |x: i32| -> i32 {
        if x < back + 26 {
            110 + (x - back) * 9 / 26
        } else {
            119 + (x - back - 26) * 2
        }
    };
    for x in back - 2..back + 40 {
        let top = roof(x);
        if top > HUT_FOOT {
            break;
        }
        for dy in 0..3 {
            let color = match dy {
                0 => PLANKED.light,
                1 => PLANKED.base,
                _ => PLANKED.edge,
            };
            put(scene, x, top + dy, color);
        }
        if x % 7 == 0 {
            put(scene, x, top + 1, PLANKED.edge);
        }
    }
    // The back post, against the wall, and the stump of the front one.
    timber(scene, (back + 24, 120), (3, HUT_FOOT - 120), 1160);
    for (x, y) in [(back + 24, 119), (back + 25, 118), (back + 26, 120)] {
        put(scene, x, y, SILVER_OAK.light);
    }
}

// ---------------------------------------------------------------------------------------------
// The cart
// ---------------------------------------------------------------------------------------------

/// The broken cart at the front right: tipped on its nose where it broke down, its load spilled
/// out in front of it, one shaft snapped, the tailboard hanging from one chain, a spoke gone from
/// its wheel, and nettles growing up round it.
fn cart(scene: &mut Canvas) {
    // The bed's side, a tilted box from its nose on the ground to its tail in the air.
    let nose = (304, 194);
    let tail = (388, 154);
    let deep = 18;
    let bottom_at = |x: i32| nose.1 + (x - nose.0) * (tail.1 - nose.1) / (tail.0 - nose.0);
    // Its shadow on the ground under it.
    for x in nose.0..WIDTH {
        let reach = (x - nose.0) / 3;
        for y in nose.1 - 1..(nose.1 + 2 + reach / 4).min(HEIGHT) {
            put(scene, x, y, rgba(0x14241c, 60));
        }
    }
    // The shafts on the ground: one whole, one snapped and its end lying apart.
    for (from, to) in [
        ((306, 186), (262, 204)),
        ((310, 190), (290, 199)),
        ((284, 203), (270, 208)),
    ] {
        for (step, &(x, y)) in trace_points(from, to).iter().enumerate() {
            put(scene, x, y, PLANKED.light);
            put(scene, x, y + 1, PLANKED.base);
            put(scene, x, y + 2, PLANKED.edge);
            if step % 9 == 4 {
                put(scene, x, y + 1, IRON.base);
            }
        }
    }
    for (x, y) in [(290, 198), (291, 197), (289, 200)] {
        put(scene, x, y, CHIPS.shine);
    }
    // Inside the bed, seen over its side: dark, a little straw left in it.
    for x in nose.0 + 2..WIDTH {
        let top = bottom_at(x) - deep;
        for y in top - 4..top {
            let color = if y == top - 4 {
                CART_PAINT.edge
            } else if chance(x, y, 1171, 60) {
                STRAW.base
            } else {
                HOLLOW
            };
            put(scene, x, y, color);
        }
    }
    // The side: three boards along it, the paint faded and worn through to the wood, and iron
    // straps down it.
    for x in nose.0..WIDTH {
        let bottom = bottom_at(x);
        let top = bottom - deep;
        for y in top..=bottom {
            let into = y - top;
            let board = into / 6;
            let within = into % 6;
            let worn = patches(x, y, (6, 2), 1172 + board as u32) > 0.6;
            let ramp = if worn { PLANKED } else { CART_PAINT };
            let mut color = if y == bottom || within == 5 {
                ramp.edge
            } else if within == 0 {
                ramp.light
            } else if within == 4 {
                ramp.shadow
            } else {
                ramp.base
            };
            if x == nose.0 {
                color = ramp.edge;
            }
            if noise(x.div_euclid(4), y, 1173).is_multiple_of(9) {
                color = mix(color, MOSS.base, 0.5);
            }
            put(scene, x, y, color);
        }
        put(scene, x, top - 1, CART_PAINT.light);
    }
    for strap in [nose.0 + 12, nose.0 + 30, nose.0 + 70] {
        let bottom = bottom_at(strap);
        for y in bottom - deep..=bottom {
            put(scene, strap, y, IRON.base);
            put(scene, strap + 1, y, IRON.shadow);
        }
        for y in [bottom - deep + 2, bottom - 2] {
            put(scene, strap, y, IRON.shine);
        }
    }
    // The tailboard, hanging from one chain off the back.
    let back = (378, bottom_at(378));
    for (dx, dy) in [(0, 0), (1, 2), (1, 4), (2, 6)] {
        put(scene, back.0 + dx, back.1 + dy, IRON.light);
    }
    for y in 0..14 {
        let x = back.0 + 2 + y / 3;
        for across in 0..5 {
            let color = match across {
                0 => PLANKED.light,
                4 => PLANKED.edge,
                _ => CART_PAINT.base,
            };
            put(scene, x + across, back.1 + 6 + y, color);
        }
    }
    // The wheel, still on, in front of the bed.
    cart_wheel(scene, (348, 182), 20, 1174);
}

/// A cart wheel standing face on: an iron tyre round a rim of wooden felloes, spokes from a big
/// hub, lit from the upper left, rust on the tyre, and one spoke gone.
fn cart_wheel(scene: &mut Canvas, (cx, cy): (i32, i32), radius: i32, salt: u32) {
    let r = radius as f32;
    ellipse(
        scene,
        cx + 3,
        cy + radius,
        radius - 2,
        3,
        rgba(0x14241c, 80),
    );
    // Spokes from the hub out, all but the one that is gone.
    let spokes = 12;
    for spoke in 0..spokes {
        if spoke == 7 {
            continue;
        }
        let angle = spoke as f32 / spokes as f32 * TAU + 0.13;
        let (dx, dy) = (angle.cos(), angle.sin());
        let lit = dx + dy < 0.0;
        let reach = if spoke == 3 { r * 0.45 } else { r - 3.0 };
        let mut step = 4.0;
        while step < reach {
            let (x, y) = (
                cx + (dx * step).round() as i32,
                cy + (dy * step).round() as i32,
            );
            put(scene, x, y, if lit { PLANKED.light } else { PLANKED.base });
            put(scene, x + 1, y + 1, PLANKED.edge);
            step += 0.5;
        }
    }
    for y in cy - radius - 1..=cy + radius + 1 {
        for x in cx - radius - 1..=cx + radius + 1 {
            let (u, v) = ((x - cx) as f32 / r, (y - cy) as f32 / r);
            let out = (u * u + v * v).sqrt();
            let lit = -u - v;
            let color = if (0.93..=1.03).contains(&out) {
                // The iron tyre.
                if chance(x, y, salt, 50) {
                    rgb(0x8a4a2a)
                } else if lit > 0.5 {
                    IRON.light
                } else if lit > -0.6 {
                    IRON.base
                } else {
                    IRON.shadow
                }
            } else if (0.8..0.93).contains(&out) {
                // The felloes, a joint between each pair.
                let angle = v.atan2(u).rem_euclid(TAU);
                let joint = ((angle / TAU * 6.0).fract() < 0.04) as u8 == 1;
                if joint {
                    PLANKED.edge
                } else if lit > 0.2 {
                    PLANKED.light
                } else if lit > -0.6 {
                    PLANKED.base
                } else {
                    PLANKED.shadow
                }
            } else if out < 0.22 {
                // The hub, and the iron band round it.
                if out > 0.16 {
                    IRON.base
                } else if lit > 0.0 {
                    PLANKED.light
                } else {
                    PLANKED.shadow
                }
            } else {
                continue;
            };
            put(scene, x, y, color);
        }
    }
    put(scene, cx, cy, IRON.edge);
    put(scene, cx - 1, cy - 1, IRON.light);
    // The rim's inner edge in shadow.
    for step in 0..96 {
        let angle = step as f32 / 96.0 * TAU;
        let (x, y) = (
            cx + (angle.cos() * (r * 0.79)).round() as i32,
            cy + (angle.sin() * (r * 0.79)).round() as i32,
        );
        put(scene, x, y, PLANKED.edge);
    }
}

// ---------------------------------------------------------------------------------------------
// The milestone, the beds the heaps lie on, plants and light
// ---------------------------------------------------------------------------------------------

/// Where the milestone stands, its foot in the grass.
const MILESTONE_AT: (i32, i32) = (22, 176);
const MILESTONE_STONE: Ramp = Ramp::new(0x4e4a42, 0x6e685c, 0x8e8676, 0xaaa290, 0xc4bca8);

/// The old milestone: a squat post of pale stone, rounded on top, a number cut into its face,
/// moss over its crown and down its left side, lichen, and grass round its foot.
pub fn milestone(scene: &mut Canvas, (x, foot): (i32, i32)) {
    let (w, h) = (13, 21);
    let left = x - w / 2;
    let top = foot - h;
    ellipse(scene, x + 3, foot, w / 2 + 3, 2, SHADE);
    for py in top..foot {
        for px in left..left + w {
            let (u, from_top) = ((px - left) as f32 / (w - 1) as f32, py - top);
            // The rounded top.
            let round = if from_top < 5 {
                let r = (w as f32 / 2.0).powi(2);
                let (dx, dy) = (px as f32 + 0.5 - x as f32 - 0.5, (5 - from_top) as f32);
                dx * dx + dy * dy * 2.4 > r
            } else {
                false
            };
            if round {
                continue;
            }
            let edge = px == left
                || px == left + w - 1
                || py == foot - 1
                || from_top == 0
                || (from_top < 5 && {
                    let r = (w as f32 / 2.0).powi(2);
                    let (dx, dy) = (px as f32 + 0.5 - x as f32 - 0.5, (5 - from_top + 1) as f32);
                    dx * dx + dy * dy * 2.4 > r
                });
            let mut color = if edge {
                MILESTONE_STONE.edge
            } else if u < 0.25 {
                MILESTONE_STONE.light
            } else if u < 0.7 {
                MILESTONE_STONE.base
            } else {
                MILESTONE_STONE.shadow
            };
            if !edge && chance(px, py, 1201, 30) {
                color = mix(color, MILESTONE_STONE.edge, 0.3);
            }
            // Moss over its crown and down its left.
            if !edge
                && (from_top < 4 + pick(px, 0, 1202, 3)
                    || (u < 0.2 && patches(px, py, (2, 4), 1203) > 0.5))
            {
                color = if u < 0.4 { MOSS.light } else { MOSS.base };
            }
            put(scene, px, py, color);
        }
    }
    // The number cut into its face: II, each cut shadowed on its left and lit on its right.
    for cut in [x - 2, x + 1] {
        vline(scene, cut, top + 8, 7, MILESTONE_STONE.edge);
        vline(scene, cut + 1, top + 8, 7, MILESTONE_STONE.light);
    }
    hline(scene, x - 3, top + 7, 6, MILESTONE_STONE.edge);
    hline(scene, x - 3, top + 15, 6, MILESTONE_STONE.edge);
    // Lichen, and grass round its foot.
    put(scene, left + w - 4, top + 17, LICHEN.light);
    put(scene, left + w - 5, top + 17, LICHEN.base);
    for dx in [-1, 1, 3, w - 2, w, w + 1] {
        let tall = 2 + pick(dx, 0, 1204, 3);
        line(
            scene,
            (left + dx, foot),
            (left + dx + dx.signum(), foot - tall),
            GRASS.base,
        );
        put(scene, left + dx + dx.signum(), foot - tall, GRASS.light);
    }
}

/// The bed a heap lies on, of whatever it has shed: wood chips and bark under the woodshed's,
/// rubble and moss under the wall's, straw, spilled grain and a scrap of sacking under the
/// cart's.
fn bed(scene: &mut Canvas, site: &Site) {
    let (cx, cy) = (site.left + site.width / 2, site.ground);
    let (rx, ry) = (site.width / 2 + 10, 6);
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (u, v) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
            let out = u * u + v * v + (noise(x, y, 1210) % 100) as f32 / 250.0;
            if out > 1.0 {
                continue;
            }
            let thick = ((1.0 - out) * 220.0) as u32 + 30;
            if !chance(x, y, 1211 + site.left as u32, thick) {
                continue;
            }
            let color = match site.name {
                "the woodshed" => {
                    if chance(x, y, 1212, 90) {
                        tone(CHIPS, 2 + pick(x, y, 1213, 3))
                    } else {
                        tone(LITTER, 1 + pick(x, y, 1214, 2))
                    }
                }
                "the fallen wall" => {
                    if chance(x, y, 1215, 110) {
                        tone(DRYSTONE, 1 + pick(x, y, 1216, 3))
                    } else {
                        tone(MOSS, 1 + pick(x, y, 1217, 3))
                    }
                }
                _ => {
                    if chance(x, y, 1218, 140) {
                        tone(STRAW, 1 + pick(x, y, 1219, 4))
                    } else {
                        tone(EARTH, 2 + pick(x, y, 1220, 2))
                    }
                }
            };
            put(scene, x, y, color);
        }
    }
    // A few bigger bits lying about: fallen stones by the wall, a sacking scrap by the cart.
    match site.name {
        "the fallen wall" => {
            for index in 0..5 {
                let x = site.left - 6 + pick(index, 0, 1221, site.width + 12);
                let y = site.ground + 1 + pick(index, 1, 1221, 4);
                ellipse(scene, x, y, 3, 2, DRYSTONE.edge);
                ellipse(scene, x, y, 2, 1, DRYSTONE.base);
                put(scene, x - 1, y - 1, DRYSTONE.light);
            }
        }
        "the cart" => {
            for (dx, dy) in [(0, 0), (1, 0), (2, 1), (3, 1), (1, 1), (2, 2), (0, 1)] {
                put(
                    scene,
                    site.left - 8 + dx,
                    site.ground + 1 + dy,
                    rgb(0xa08654),
                );
            }
        }
        _ => {}
    }
}

/// Foxgloves by the hut, nettles round the cart, and grass and small flowers along the verges,
/// all kept off wherever anyone stands.
fn plants(scene: &mut Canvas) {
    let clear = |x: i32, y: i32| {
        let stands = SITES
            .iter()
            .map(|site| site.stand)
            .chain(super::PLACES_TO_STAND.iter().copied());
        let near_stand = stands
            .into_iter()
            .any(|(sx, sy)| (sx - x as f32).abs() < 14.0 && (sy - y as f32).abs() < 10.0);
        let near_heap = SITES.iter().any(|site| {
            x >= site.left - 6
                && x <= site.left + site.width + 6
                && y >= site.ground - 50
                && y <= site.ground + 6
        });
        let in_hut = x < HUT.1 + 44 && y < HUT_FOOT + 1;
        let by_milestone = (x - MILESTONE_AT.0).abs() < 10 && y < MILESTONE_AT.1 + 2;
        !near_stand && !near_heap && !in_hut && !by_milestone && on_track(x, y) < 0.5
    };
    // Foxgloves at the woods' edge beside the woodshed.
    for &(x, tall) in &[(150, 18), (156, 13), (144, 11)] {
        let foot = treeline(x) + 6;
        foxglove(scene, (x, foot), tall);
    }
    // Nettles round the cart's nose.
    for (index, x) in (296..336).step_by(5).enumerate() {
        let foot = 192 + pick(index as i32, 0, 1230, 6);
        if !clear(x, foot) {
            continue;
        }
        nettle(
            scene,
            (x, foot),
            6 + pick(index as i32, 1, 1230, 5),
            1231 + index as u32,
        );
    }
    // Grass tufts and small flowers along the verges.
    for index in 0..90 {
        let x = pick(index, 0, 1240, WIDTH);
        let y = treeline(x) + 4 + pick(index, 1, 1240, HEIGHT - treeline(x) - 4);
        if !clear(x, y) {
            continue;
        }
        let tall = 2 + (nearness(y) * 4.0) as i32 + pick(index, 2, 1240, 2);
        for blade in -1..=1 {
            let tip = (
                x + blade * (1 + pick(index, 3, 1240, 2)),
                y - tall + blade.abs(),
            );
            line(scene, (x + blade, y), tip, GRASS.base);
            put(scene, tip.0, tip.1, GRASS.light);
        }
        if index % 4 == 0 {
            let flower = if index % 8 == 0 {
                rgb(0xf4f2ea)
            } else {
                rgb(0xe07aa6)
            };
            put(scene, x + 2, y - tall, flower);
            put(scene, x + 2, y - tall + 1, GRASS.shadow);
        }
    }
}

/// A foxglove: a stalk hung with purple bells on one side, buds at its tip, leaves at its foot.
fn foxglove(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32) {
    for step in 0..tall {
        put(scene, x, foot - step, GRASS.shadow);
    }
    for bell in 0..tall / 2 - 1 {
        let y = foot - tall + 3 + bell * 2;
        put(scene, x + 1, y, FOXGLOVE.light);
        put(scene, x + 2, y, FOXGLOVE.base);
        put(scene, x + 1, y + 1, FOXGLOVE.shadow);
        put(scene, x + 2, y + 1, FOXGLOVE.edge);
        if bell % 2 == 0 {
            put(scene, x - 1, y + 1, FOXGLOVE.base);
        }
    }
    put(scene, x, foot - tall - 1, GRASS.light);
    for dx in [-3, -2, 2, 3] {
        put(scene, x + dx, foot - 1, GRASS.base);
        put(scene, x + dx / 2, foot - 2, GRASS.light);
    }
}

/// A nettle: an upright stem with pairs of toothed leaves, darker below.
fn nettle(scene: &mut Canvas, (x, foot): (i32, i32), tall: i32, salt: u32) {
    for step in 0..tall {
        put(scene, x, foot - step, NETTLE.shadow);
        if step % 3 == 1 {
            let level = if step > tall / 2 { 3 } else { 2 };
            for side in [-1, 1] {
                put(scene, x + side, foot - step, tone(NETTLE, level));
                put(
                    scene,
                    x + side * 2,
                    foot - step + 1,
                    tone(NETTLE, level - 1),
                );
            }
        }
    }
    put(scene, x, foot - tall, NETTLE.light);
    if chance(x, foot, salt, 128) {
        put(scene, x + 1, foot - tall + 1, NETTLE.shine);
    }
}

/// Shafts of sun slanting down the ride from the gap in the trees, and the warm pools where they
/// land on the track.
fn sunbeams(scene: &mut Canvas) {
    for &(top_x, half, end_y, salt) in &[(150.0, 7.0, 170, 1250), (176.0, 5.0, 150, 1251)] {
        for y in 30..end_y {
            let reach = (y - 30) as f32;
            let middle = top_x + reach * 0.45;
            let fade = 1.0 - reach / (end_y - 30) as f32 * 0.6;
            // A beam shows against the shade under the trees, not against the open sky and the
            // Hill at the end of the ride.
            let (open_left, open_right) = ride_edges(y);
            for x in (middle - half) as i32..=(middle + half) as i32 {
                if y < HORIZON && x > open_left && x < open_right {
                    continue;
                }
                let off = (x as f32 - middle).abs() / half;
                let alpha = ((1.0 - off) * 26.0 * fade) as u8;
                if alpha > 2 && !chance(x, y, salt, 40) {
                    put(scene, x, y, rgba(SUN, alpha));
                }
            }
        }
        let land = (top_x + (end_y - 30) as f32 * 0.45) as i32;
        ellipse(scene, land, end_y, half as i32 * 2, 3, rgba(SUN, 34));
        ellipse(scene, land, end_y, half as i32, 2, rgba(SUN, 30));
    }
    // Flecks of sun on the ground.
    for index in 0..40 {
        let x = pick(index, 0, 1252, WIDTH);
        let y = HORIZON + 10 + pick(index, 1, 1252, HEIGHT - HORIZON - 10);
        if y <= treeline(x) {
            continue;
        }
        let rx = 1 + pick(index, 2, 1252, 3);
        ellipse(scene, x, y, rx, 1, rgba(SUN, 36));
    }
}

// ---------------------------------------------------------------------------------------------
// In front of everyone, and after dark
// ---------------------------------------------------------------------------------------------

/// Bracken in the bottom corners and leaves hanging into the top ones, drawn over everyone: low
/// along the bottom, and nothing over the middle of it.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for &(x, angle, length) in &[
        (-2, -1.2, 18),
        (4, -1.0, 15),
        (10, -0.7, 12),
        (380, -2.0, 18),
        (374, -2.2, 14),
        (368, -2.5, 11),
    ] {
        stem_frond(&mut scene, (x, HEIGHT + 2), angle, length);
    }
    for (index, x) in (0..WIDTH).step_by(3).enumerate() {
        let corner = if x < 60 {
            1.0 - x as f32 / 60.0
        } else if x > WIDTH - 60 {
            (x - (WIDTH - 60)) as f32 / 60.0
        } else {
            continue;
        };
        let tall = (2.0 + corner * 7.0) as i32 + pick(index as i32, 0, 1260, 3);
        let lean = pick(index as i32, 1, 1260, 3) - 1;
        let color = if index % 2 == 0 {
            FERN.shadow
        } else {
            FERN.base
        };
        line(&mut scene, (x, HEIGHT), (x + lean, HEIGHT - tall), color);
        put(&mut scene, x + lean, HEIGHT - tall, FERN.light);
    }
    scene
}

/// Where the glow-worms shine after dark: along the foot of the wall and the verges.
const GLOW_WORMS: [(i32, i32); 7] = [
    (252, 112),
    (286, 124),
    (352, 140),
    (372, 150),
    (122, 132),
    (48, 160),
    (170, 128),
];

/// Glow-worms along the wall and the verges, and fireflies under the trees.
pub fn lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for &(x, y) in &GLOW_WORMS {
        crate::daylight::glow(&mut lights, (x, y), 5, rgb(0xb8f07a));
        put(&mut lights, x, y, rgb(0xeaffc0));
    }
    crate::daylight::fireflies(
        &mut lights,
        &[
            (10, 70, 120, 40, 9),
            (262, 70, 110, 36, 8),
            (150, 100, 90, 14, 4),
        ],
        1270,
    );
    lights
}

/// Fireflies under the trees on a treasure hunt.
pub fn hunt_lamplight() -> Canvas {
    let mut lights = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    crate::daylight::fireflies(
        &mut lights,
        &[(10, 60, 130, 50, 10), (250, 60, 120, 50, 9)],
        1271,
    );
    lights
}

/// Where the moon rises after dark: in the sky over the ride.
const MOON: (i32, i32) = (232, 16);

/// The sky after dark, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, HORIZON, Some(MOON))
}

// ---------------------------------------------------------------------------------------------
// Off the old track: forks, nooks and the dig
// ---------------------------------------------------------------------------------------------

/// Where the trees stand round a clearing off the old track, as the row their feet are on at
/// `x`: a little further off in the middle, nearer at the sides, with gaps where ways go in.
fn clearing_edge(x: i32) -> i32 {
    let off = (x - 192).abs() as f32 / 192.0;
    (112.0 + off * off * 18.0 + (patches(x, 0, (13, 1), 1301) - 0.5) * 4.0).round() as i32
}

/// Where the canopy over a clearing comes down to, at `x`: low at the sides, higher over the
/// middle, ragged.
fn canopy_foot(x: i32) -> i32 {
    let off = (x - 196).abs() as f32 / 196.0;
    (34.0 + off * off * 26.0 + (patches(x, 0, (11, 1), 1302) - 0.5) * 10.0) as i32
}

/// The woods all round a clearing: the air under the trees, trunks back and front rising into a
/// canopy that closes over everything, undergrowth along their feet but parted where each way
/// goes in, and the sun coming down through the leaves.
fn woods_round(scene: &mut Canvas, openings: &[f32], salt: u32) {
    for x in 0..WIDTH {
        for y in 0..clearing_edge(x) + 2 {
            let level = (y + pick(x, y, salt, 5) - 2).div_euclid(6) * 6;
            scene.set(x, y, air(level + 30));
        }
    }
    let opening = |x: i32, room: f32| openings.iter().any(|at| (x as f32 - at).abs() < room);
    for rank in 0..3 {
        let deep = 0.55 - rank as f32 * 0.22;
        for index in 0..16 {
            let i = index + rank * 30;
            let x = pick(i, 0, salt + 1 + rank as u32, WIDTH + 20) - 10;
            if rank == 2 && opening(x, 16.0) {
                continue;
            }
            let foot = clearing_edge(x) - (2 - rank) * 4 - pick(i, 1, salt, 3);
            let width = 2 + rank * 2 + pick(i, 2, salt, 3);
            trunk(scene, (x, width), foot, (deep, false), salt + 10 + i as u32);
        }
    }
    // The canopy: a mass of leaves over everything, its underside scalloped into clumps, lit
    // where the sun is on it.
    for x in 0..WIDTH {
        for y in 0..canopy_foot(x) {
            let color = if chance(x.div_euclid(2), y.div_euclid(2), salt + 3, 50) {
                OAK.base
            } else {
                OAK.shadow
            };
            scene.set(x, y, color);
        }
    }
    let mut x = -10;
    let mut index = 0;
    while x < WIDTH + 10 {
        let r = 9 + pick(index, 0, salt + 4, 7);
        let ramp = if index % 3 == 0 { BEECH } else { OAK };
        clump(
            scene,
            (x, canopy_foot(x) - r / 2),
            r,
            ramp,
            0.08,
            salt + 40 + index as u32 * 7,
        );
        x += r + 3;
        index += 1;
    }
    for (index, &(x, y, r)) in [(20, 0, 24), (90, -6, 22), (300, -6, 22), (370, 0, 24)]
        .iter()
        .enumerate()
    {
        clump(scene, (x, y), r, BEECH, 0.0, salt + 80 + index as u32 * 7);
    }
    // Undergrowth along the feet of the trees, parted where each way goes in.
    let mut x = -8;
    let mut index = 0;
    while x < WIDTH + 8 {
        let r = 6 + pick(index, 0, salt + 2, 6);
        if !opening(x, 20.0) {
            clump(
                scene,
                (x, clearing_edge(x) - r / 3),
                r,
                UNDERGROWTH,
                0.12,
                salt + 30 + index as u32 * 5,
            );
        }
        x += r * 3 / 2;
        index += 1;
    }
    // Sun slanting down through the leaves.
    for &(top_x, half, end_y) in &[(150.0, 6.0, 150), (236.0, 5.0, 140)] {
        for y in 20..end_y {
            let middle = top_x + (y - 20) as f32 * 0.4;
            let fade = 1.0 - (y - 20) as f32 / (end_y - 20) as f32 * 0.6;
            for px in (middle - half) as i32..=(middle + half) as i32 {
                let off = (px as f32 - middle).abs() / half;
                let alpha = ((1.0 - off) * 22.0 * fade) as u8;
                if alpha > 2 && !chance(px, y, salt + 5, 40) {
                    put(scene, px, y, rgba(SUN, alpha));
                }
            }
        }
    }
}

/// The floor of a clearing: earth under leaf litter, soft moss here and there, fallen leaves.
fn clearing_floor(scene: &mut Canvas, salt: u32) {
    for x in 0..WIDTH {
        for y in clearing_edge(x) + 1..HEIGHT {
            let soil = patches(x, y, (18, 6), salt) + (noise(x, y, salt + 1) % 100) as f32 / 600.0;
            let mut color = if soil < 0.35 {
                LITTER.shadow
            } else if soil < 0.75 {
                mix(LITTER.shadow, LITTER.base, 0.6)
            } else {
                LITTER.base
            };
            let mossy = patches(x, y, (24, 8), salt + 2);
            if mossy > 0.7 {
                let cushion = patches(x, y, (4, 3), salt + 3);
                let moss = if cushion > 0.7 { MOSS.light } else { MOSS.base };
                color = mix(color, moss, ((mossy - 0.7) * 4.0).min(0.6));
            }
            let far = (1.0 - ((y - 110) as f32 / 106.0).clamp(0.0, 1.0)).powi(3) * 0.35;
            scene.set(x, y, mix(color, air(118), far));
        }
    }
    for index in 0..160 {
        let x = pick(index, 0, salt + 4, WIDTH);
        let y = clearing_edge(x) + 3 + pick(index, 1, salt + 4, HEIGHT - clearing_edge(x) - 3);
        let long = 1 + (y - 110) / 40;
        let color = [LITTER.light, rgb(0x8e6434), rgb(0x7a4630), rgb(0x987a40)]
            [pick(index, 2, salt + 4, 4) as usize];
        hline(scene, x - long, y, long * 2 + 1, color);
        hline(scene, x - long + 1, y + 1, long * 2, rgba(0x1e140e, 90));
    }
}

/// A trodden way along `points`, wider nearer the eye and foreshortened as the ground is, so a
/// way going off sideways is a narrow band: bare earth worn pale along its middle, grass along
/// its edges, going into shadow where it goes into the trees.
fn path_along(scene: &mut Canvas, points: &[(f32, f32)], salt: u32) {
    let mut worn = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for pair in points.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = ((to.0 - from.0).abs().max((to.1 - from.1).abs()) * 2.0).ceil() as i32;
        for step in 0..=steps {
            let t = step as f32 / steps.max(1) as f32;
            let (cx, cy) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            let half = 3.0 + (cy - 110.0).max(0.0) * 0.2;
            let thick = (half * 0.3).max(1.2);
            for dy in -(thick as i32 + 1)..=thick as i32 + 1 {
                for dx in -(half as i32 + 1)..=half as i32 + 1 {
                    let (x, y) = (cx.round() as i32 + dx, cy.round() as i32 + dy);
                    let (u, v) = (dx as f32 / half, dy as f32 / thick);
                    let off = (u * u + v * v).sqrt() + (noise(x, y, salt) % 100) as f32 / 500.0;
                    if off > 1.1 {
                        continue;
                    }
                    let was = worn.get(x, y);
                    let mark = (255.0 * (1.1 - off).min(1.0)) as u8;
                    if mark > was.a {
                        worn.set(x, y, Rgba::new(0, 0, 0, mark));
                    }
                }
            }
        }
    }
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mark = worn.get(x, y).a;
            if mark == 0 {
                continue;
            }
            let depth = (1.0 - ((y - 112) as f32 / 40.0).clamp(0.0, 1.0)) * 0.5;
            let color = if mark < 60 {
                if chance(x, y, salt + 1, 140) {
                    GRASS.base
                } else {
                    continue;
                }
            } else if mark < 110 {
                mix(EARTH.base, GRASS.shadow, 0.3)
            } else if mark > 220 && patches(x, y, (5, 2), salt + 2) > 0.45 {
                mix(EARTH.light, LITTER.light, 0.45)
            } else {
                EARTH.light
            };
            put(scene, x, y, mix(color, air(112), depth));
        }
    }
}

/// The course of a stream across the way it crosses: out of the trees at the back, across that
/// way and on out of the picture, or round and back into the trees, never across another way.
/// The ways left and right mirror each other about the fork.
fn stream_course(dir: Dir) -> Vec<(f32, f32)> {
    const LEFT: [(f32, f32); 14] = [
        (138.0, 110.0),
        (132.0, 115.0),
        (127.0, 120.0),
        (117.0, 123.0),
        (106.0, 128.0),
        (99.0, 133.0),
        (88.0, 135.0),
        (76.0, 140.0),
        (67.0, 146.0),
        (53.0, 148.0),
        (40.0, 154.0),
        (27.0, 157.0),
        (13.0, 164.0),
        (-8.0, 169.0),
    ];
    match dir {
        Dir::Left => LEFT.to_vec(),
        Dir::Right => LEFT
            .iter()
            .map(|&(x, y)| (2.0 * PARTING.0 - x, y))
            .collect(),
        Dir::Ahead => vec![
            (142.0, 109.0),
            (149.0, 115.0),
            (158.0, 120.0),
            (169.0, 122.0),
            (180.0, 126.0),
            (191.0, 128.0),
            (202.0, 127.0),
            (212.0, 125.0),
            (222.0, 126.0),
            (232.0, 121.0),
            (241.0, 116.0),
            (250.0, 109.0),
        ],
    }
}

/// A stream crossing a way: water winding out of the undergrowth, under a dark cut of bank on
/// its far side and a lip of wet mud on its near one, catching the light in ripples down its
/// middle, reeds along it, and stepping stones where the way crosses.
fn stream_across(scene: &mut Canvas, dir: Dir) {
    let course = stream_course(dir);
    let half = |y: f32| 1.6 + (y - 110.0).max(0.0) * 0.07;
    // Where the water is, and how far across it each pixel lies: 1 at the far bank, 255 at the
    // near one.
    let mut bed = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for pair in course.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = ((to.0 - from.0).abs().max((to.1 - from.1).abs()) * 3.0).ceil() as i32;
        for step in 0..=steps {
            let t = step as f32 / steps.max(1) as f32;
            let (cx, cy) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            let reach = half(cy);
            let x = cx.round() as i32;
            for y in (cy - reach).floor() as i32..=(cy + reach).ceil() as i32 {
                let across = (y as f32 - cy) / reach;
                if across.abs() > 1.0 {
                    continue;
                }
                bed.set(x, y, Rgba::new(0, 0, 0, ((across + 1.0) * 127.0) as u8 + 1));
            }
        }
    }
    let wet = |x: i32, y: i32| bed.get(x, y).a > 0 && y > clearing_edge(x);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if !wet(x, y) {
                continue;
            }
            let across = bed.get(x, y).a as f32 / 127.0 - 1.0;
            let ripple = noise((x + y / 2).div_euclid(3), y, 1311) % 9;
            let color = if across < -0.45 {
                // The bank mirrored in the water under it.
                PUDDLE.shadow
            } else if ripple == 0 && across < 0.6 {
                PUDDLE.shine
            } else if ripple < 3 {
                PUDDLE.light
            } else if across > 0.55 {
                mix(PUDDLE.base, PUDDLE.light, 0.4)
            } else {
                PUDDLE.base
            };
            put(scene, x, y, color);
            // The far bank: a dark cut of earth, roots showing.
            if !wet(x, y - 1) && y - 1 > clearing_edge(x) {
                put(scene, x, y - 1, EARTH.shadow);
                put(
                    scene,
                    x,
                    y - 2,
                    if chance(x, y, 1312, 70) {
                        GRASS.shadow
                    } else {
                        EARTH.base
                    },
                );
            }
            // The near bank: a lip of wet mud.
            if !wet(x, y + 1) {
                put(scene, x, y + 1, EARTH.light);
                if chance(x, y, 1313, 90) {
                    put(scene, x, y + 2, EARTH.base);
                }
            }
        }
    }
    // Reeds along the banks, here and there, and thickest where the water goes into the trees.
    let path = dense(&way_path(dir));
    let water = dense(&course);
    let (_, at) = path
        .iter()
        .enumerate()
        .map(|(index, on)| {
            let near = water
                .iter()
                .map(|wet| crate::playground::distance(*wet, *on))
                .fold(f32::MAX, f32::min);
            (near, index)
        })
        .fold(
            (f32::MAX, 0),
            |best, next| if next.0 < best.0 { next } else { best },
        );
    let crossing = path[at];
    for (index, &(x, y)) in course.iter().enumerate() {
        let end = index == 0 || index == course.len() - 1;
        let near_crossing = crate::playground::distance((x, y), crossing) < 22.0;
        if near_crossing || !(0.0..WIDTH as f32).contains(&x) || (!end && index % 2 == 1) {
            continue;
        }
        let blades = if index == 0 || index == course.len() - 1 {
            5
        } else {
            3
        };
        for blade in 0..blades {
            let (bx, by) = (
                x as i32 - 3 + pick(index as i32, blade, 1314, 7),
                (y - half(y)) as i32 - 1 + pick(index as i32, blade + 9, 1314, 2),
            );
            let tall = 4 + pick(index as i32, blade + 20, 1314, 4) + (by - 110).max(0) / 18;
            let lean = pick(index as i32, blade + 30, 1314, 3) - 1;
            line(scene, (bx, by), (bx + lean, by - tall), NETTLE.base);
            put(scene, bx + lean, by - tall, NETTLE.light);
            put(scene, bx, by, NETTLE.shadow);
        }
    }
    // Stepping stones where the way crosses, along the way: one in the water and one on each
    // bank.
    let (back, front) = (
        path[at.saturating_sub(4)],
        path[(at + 4).min(path.len() - 1)],
    );
    let length = crate::playground::distance(back, front).max(0.01);
    let along = ((front.0 - back.0) / length, (front.1 - back.1) / length);
    let stride = (half(crossing.1) * 1.1 / along.1.abs().max(0.3)).max(4.0);
    for step in -1..=1 {
        let (x, y) = (
            (crossing.0 + along.0 * stride * step as f32).round() as i32,
            (crossing.1 + along.1 * stride * step as f32).round() as i32,
        );
        let size = if crossing.1 > 125.0 { 2 } else { 1 };
        ellipse(scene, x, y + 1, size + 1, size, PUDDLE.edge);
        ellipse(scene, x, y, size + 1, size, DRYSTONE.edge);
        ellipse(scene, x, y, size, (size - 1).max(1), DRYSTONE.base);
        put(scene, x - 1, y - 1, DRYSTONE.light);
        put(scene, x - size, y, DRYSTONE.light);
        put(scene, x + size, y, DRYSTONE.shadow);
    }
}

/// A fork off the old track: the way coming up to it from the front, parting into the ways the
/// map is about, a stream across one of them if the map says so. The landmarks stand on it as
/// props (see `landmarks`).
pub fn fork(fork: &Fork) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let openings: Vec<f32> = fork.ways.iter().map(|dir| way_end(*dir).0).collect();
    woods_round(&mut scene, &openings, 1320);
    clearing_floor(&mut scene, 1330);
    path_along(&mut scene, &[(192.0, 220.0), (194.0, 196.0), PARTING], 1340);
    for dir in &fork.ways {
        path_along(&mut scene, &way_path(*dir), 1341);
    }
    if let Some(way) = fork.stream {
        stream_across(&mut scene, fork.ways[way]);
    }
    let ways: Vec<Vec<(f32, f32)>> = fork
        .ways
        .iter()
        .map(|dir| way_path(*dir))
        .chain(std::iter::once(vec![
            (192.0, 220.0),
            (194.0, 196.0),
            PARTING,
        ]))
        .chain(fork.stream.map(|way| dense(&stream_course(fork.ways[way]))))
        .collect();
    let spots: Vec<(f32, f32)> = fork
        .landmarks
        .iter()
        .map(|placed| super::landmarks::spot_point(fork, placed.spot))
        .chain(std::iter::once(super::treasure::START))
        .collect();
    clearing_plants(&mut scene, &ways, &spots, 1342);
    clearing_litter(&mut scene, &ways, &spots, 1343);
    clearing_flecks(&mut scene);
    scene
}

/// The nook at the end of a wrong way: the way coming in from the front and petering out in a
/// clearing where somebody once camped, against a fallen tree, by a ring of fire stones gone
/// cold, and left a heap behind them (see `NOOK`).
pub fn nook() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    woods_round(&mut scene, &[], 1350);
    clearing_floor(&mut scene, 1360);
    let way = vec![(176.0, 220.0), (166.0, 200.0), (150.0, 186.0)];
    path_along(&mut scene, &way, 1370);
    fallen_tree(&mut scene);
    fire_ring(&mut scene, FIRE_RING);
    bed(&mut scene, &NOOK);
    let heap = (NOOK.left + NOOK.width / 2) as f32;
    let spots = [
        NOOK.stand,
        (heap, NOOK.ground as f32),
        (heap - 30.0, NOOK.ground as f32),
        (heap + 30.0, NOOK.ground as f32),
        (FIRE_RING.0 as f32, FIRE_RING.1 as f32),
        (FALLEN_TREE.0 as f32 + 4.0, FALLEN_TREE.2 as f32),
    ];
    clearing_plants(&mut scene, std::slice::from_ref(&way), &spots, 1371);
    clearing_litter(&mut scene, &[way], &spots, 1372);
    clearing_flecks(&mut scene);
    scene
}

/// Where the fallen tree behind the nook's heap lies: from its root plate to its broken end, and
/// the row its underside rests on.
const FALLEN_TREE: (i32, i32, i32) = (92, 306, 162);
/// Where the cold fire ring in the nook is.
const FIRE_RING: (i32, i32) = (304, 194);

/// A tree come down long ago behind the nook's heap: its trunk lying along the ground, round and
/// lit along its top, the bark furrowed and come away in places to grey wood, moss along its
/// back; its root plate standing up at one end with the earth still in it, and the other end
/// snapped off in splinters.
fn fallen_tree(scene: &mut Canvas) {
    let (from, to, rest) = FALLEN_TREE;
    let thick = 11;
    let underside = |x: i32| rest - (x - from) / 60;
    for x in from + 6..to {
        let bottom = underside(x);
        // The broken end is splintered: its length ragged.
        let ragged = if x > to - 6 {
            (to - x) * 2 + pick(x, 0, 1401, 4)
        } else {
            thick
        };
        for row in 0..thick.min(ragged) {
            let y = bottom - row;
            let up = row as f32 / (thick - 1) as f32;
            let mut color = if row == 0 || row == thick - 1 {
                BARK.edge
            } else if up > 0.72 {
                BARK.light
            } else if up > 0.42 {
                BARK.base
            } else if up > 0.18 {
                BARK.shadow
            } else {
                mix(BARK.shadow, BARK.edge, 0.5)
            };
            // Furrows running along it.
            if row > 0 && row < thick - 1 && noise(x.div_euclid(4), row, 1402).is_multiple_of(4) {
                color = mix(color, BARK.edge, 0.55);
            }
            // Bark come away to the grey wood under it.
            if row > 1 && row < thick - 2 && patches(x, row, (9, 4), 1403) > 0.74 {
                color = tone(SILVER_OAK, (up * 4.0) as i32 + 1);
            }
            // Moss along its back.
            if up > 0.6 && patches(x, row, (7, 3), 1404) > 0.5 - (up - 0.6) {
                color = if row >= thick - 2 {
                    MOSS.light
                } else {
                    MOSS.base
                };
                if row == thick - 1 {
                    color = MOSS.shadow;
                }
            }
            if x > to - 6 && row == ragged - 1 {
                color = CHIPS.light;
            }
            put(scene, x, y, color);
        }
        // Its shadow on the ground under it.
        put(scene, x, bottom + 1, rgba(0x14100c, 110));
        if x % 2 == 0 {
            put(scene, x, bottom + 2, rgba(0x14100c, 60));
        }
    }
    // The snapped end: pale splinters.
    for (dx, dy) in [(0, -3), (1, -5), (2, -4), (0, -7), (3, -6), (1, -8)] {
        let x = to - 4 + dx;
        let bottom = underside(x);
        put(scene, x, bottom + dy, CHIPS.shine);
        put(scene, x + 1, bottom + dy, CHIPS.base);
    }
    // The root plate, stood up on end with the earth still in it.
    let (cx, cy) = (from + 4, rest - 12);
    for y in cy - 15..=rest + 1 {
        for x in cx - 9..=cx + 8 {
            let (u, v) = ((x - cx) as f32 / 8.5, (y - cy) as f32 / 14.5);
            let edge = 1.0 + (noise(x, y, 1405) % 100) as f32 / 400.0;
            let out = u * u + v * v;
            if out > edge {
                continue;
            }
            let light = u + v * 0.6;
            let color = if out > edge - 0.18 {
                LITTER.edge
            } else if chance(x, y, 1406, 40) {
                tone(DRYSTONE, 2 + pick(x, y, 1407, 2))
            } else if chance(x, y, 1411, 30) {
                BARK.light
            } else if light < -0.5 {
                LITTER.light
            } else if light < 0.3 {
                LITTER.base
            } else {
                LITTER.shadow
            };
            put(scene, x, y, color);
        }
    }
    // Roots torn out of it, reaching every way.
    for (index, angle) in [-2.7f32, -2.0, -1.4, -0.7, 2.6, 0.4, 3.0]
        .iter()
        .enumerate()
    {
        let reach = 6 + pick(index as i32, 0, 1408, 6);
        let (dx, dy) = (angle.cos(), angle.sin() * 1.4);
        let (sx, sy) = (cx as f32 + dx * 6.0, cy as f32 + dy * 8.0);
        for step in 0..reach {
            let (x, y) = (
                (sx + dx * step as f32).round() as i32,
                (sy + dy * step as f32 + (step * step) as f32 * 0.04).round() as i32,
            );
            if y > rest + 1 {
                break;
            }
            put(
                scene,
                x,
                y,
                if step < reach / 2 {
                    BARK.base
                } else {
                    BARK.shadow
                },
            );
            put(scene, x, y + 1, BARK.edge);
        }
    }
    // A little grass grown up against it.
    for x in (from + 12..to - 4).step_by(9) {
        if (NOOK.left - 4..NOOK.left + NOOK.width + 4).contains(&x) {
            continue;
        }
        let bottom = underside(x) + 1;
        for blade in 0..2 {
            let tall = 1 + pick(x, blade, 1409, 3);
            line(
                scene,
                (x + blade, bottom),
                (x + blade - 1 + pick(x, blade + 5, 1409, 3), bottom - tall),
                GRASS.base,
            );
            put(scene, x + blade, bottom - tall, GRASS.light);
        }
    }
}

/// A ring of stones where somebody had a fire, long cold: ash and charred ends of sticks in it.
fn fire_ring(scene: &mut Canvas, (cx, cy): (i32, i32)) {
    ellipse(scene, cx, cy, 8, 3, rgb(0x3a3430));
    ellipse(scene, cx, cy, 6, 2, rgb(0x5e5852));
    for (index, (dx, dy)) in [(-3, 0), (2, -1), (0, 1), (4, 1), (-5, -1)]
        .into_iter()
        .enumerate()
    {
        put(
            scene,
            cx + dx,
            cy + dy,
            if index % 2 == 0 {
                rgb(0x8a847c)
            } else {
                rgb(0x24201c)
            },
        );
    }
    // Charred sticks, crossed.
    line(scene, (cx - 4, cy + 1), (cx + 3, cy - 1), rgb(0x1c1814));
    line(scene, (cx - 2, cy - 1), (cx + 5, cy + 1), rgb(0x2a2420));
    put(scene, cx + 5, cy + 1, CHIPS.shadow);
    // The stones, lit from the upper left, the ones at the back half hidden.
    for step in 0..9 {
        let angle = step as f32 / 9.0 * TAU + 0.3;
        let (x, y) = (
            cx + (angle.cos() * 10.0).round() as i32,
            cy + (angle.sin() * 4.0).round() as i32,
        );
        let size = 1 + pick(step, 0, 1410, 2);
        ellipse(scene, x, y + 1, size + 1, size, rgba(0x14100c, 90));
        ellipse(scene, x, y, size + 1, size, DRYSTONE.edge);
        ellipse(scene, x, y, size, (size - 1).max(1), DRYSTONE.base);
        put(scene, x - size + 1, y - 1, DRYSTONE.light);
        put(scene, x + size, y, DRYSTONE.shadow);
        if step % 3 == 0 {
            put(scene, x, y - 1, rgb(0x3a3430));
        }
    }
}

/// The dig: another stretch of the old track, its ruts running across the back of a clearing,
/// where the old milestone stands; the places that might be dug are props (see `landmarks`).
pub fn dig_site(dig: &Dig) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    woods_round(&mut scene, &[0.0, 384.0], 1380);
    clearing_floor(&mut scene, 1390);
    // The track across the back, two ruts with grass between.
    for x in 0..WIDTH {
        let middle = 132.0 + (x as f32 * 0.02).sin() * 2.0 - (x - 192) as f32 * 0.03;
        for dy in -7..=7_i32 {
            let y = (middle + dy as f32) as i32;
            let rut = (dy.abs() - 4).abs() <= 1;
            let color = if rut {
                if dy < 0 { RUT.base } else { RUT.light }
            } else if dy.abs() < 3 {
                if chance(x, y, 1391, 120) {
                    GRASS.light
                } else {
                    GRASS.base
                }
            } else if chance(x, y, 1392, 160) {
                GRASS.base
            } else {
                continue;
            };
            put(&mut scene, x, y, color);
        }
    }
    let way = vec![(192.0, 220.0), (190.0, 180.0), (186.0, 142.0)];
    path_along(&mut scene, &way, 1393);
    let spots: Vec<(f32, f32)> = dig
        .spots
        .iter()
        .map(|spot| spot.at)
        .chain([super::routes::MILESTONE, super::treasure::START])
        .collect();
    clearing_plants(&mut scene, &[way], &spots, 1394);
    clearing_flecks(&mut scene);
    scene
}

/// Ferns along the edge of a clearing, and grass and wood anemones over its floor, kept off the
/// ways and wherever anything stands.
fn clearing_plants(scene: &mut Canvas, ways: &[Vec<(f32, f32)>], spots: &[(f32, f32)], salt: u32) {
    let clear = |x: i32, y: i32| clear_of(ways, spots, (x, y));
    for (index, x) in (6..WIDTH).step_by(23).enumerate() {
        let foot = clearing_edge(x) + 4;
        if !clear(x, foot) {
            continue;
        }
        for frond in 0..4 {
            let angle =
                -2.2 + frond as f32 * 0.42 + pick(index as i32, frond, salt, 3) as f32 * 0.06;
            stem_frond(scene, (x + frond * 2 - 3, foot), angle, 9);
        }
    }
    for index in 0..70 {
        let x = pick(index, 0, salt + 1, WIDTH);
        let y = clearing_edge(x) + 8 + pick(index, 1, salt + 1, HEIGHT - clearing_edge(x) - 8);
        if !clear(x, y) {
            continue;
        }
        let tall = 2 + (y - 110) / 30 + pick(index, 2, salt + 1, 2);
        for blade in -1..=1 {
            let tip = (
                x + blade * (1 + pick(index, 3, salt + 1, 2)),
                y - tall + blade.abs(),
            );
            line(scene, (x + blade, y), tip, GRASS.base);
            put(scene, tip.0, tip.1, GRASS.light);
        }
        if index % 3 == 0 {
            put(scene, x + 2, y - tall, rgb(0xf4f2ea));
            put(scene, x + 3, y - tall, rgb(0xdcd8e8));
            put(scene, x + 2, y - tall + 1, GRASS.shadow);
        }
    }
}

/// Whether a point on a clearing's floor is clear of its ways and of wherever anything stands.
fn clear_of(ways: &[Vec<(f32, f32)>], spots: &[(f32, f32)], (x, y): (i32, i32)) -> bool {
    let point = (x as f32, y as f32);
    let room = 12.0 + (y - 110).max(0) as f32 * 0.2;
    let off_ways = ways.iter().all(|way| {
        way.iter()
            .all(|at| crate::playground::distance(*at, point) > room)
    });
    let off_spots = spots
        .iter()
        .all(|(sx, sy)| (sx - point.0).abs() > 16.0 || (sy - point.1).abs() > 14.0);
    off_ways && off_spots
}

/// What lies about the floor of a clearing off the track: fallen twigs, and pebbles lit from
/// the upper left, kept off the ways and wherever anything stands. Not at the dig, where a stone
/// might be taken for the one the map means.
fn clearing_litter(scene: &mut Canvas, ways: &[Vec<(f32, f32)>], spots: &[(f32, f32)], salt: u32) {
    for index in 0..18 {
        let x = 8 + pick(index, 0, salt, WIDTH - 16);
        let y = clearing_edge(x) + 10 + pick(index, 1, salt, HEIGHT - clearing_edge(x) - 14);
        if !clear_of(ways, spots, (x, y)) {
            continue;
        }
        if index % 2 == 0 {
            let long = 3 + pick(index, 2, salt, 4) + (y - 110) / 30;
            let rise = pick(index, 3, salt, 3) - 1;
            line(
                scene,
                (x, y + 1),
                (x + long, y + 1 + rise),
                rgba(0x14100c, 70),
            );
            line(scene, (x, y), (x + long, y + rise), BARK.base);
            put(scene, x, y, BARK.light);
            if long > 4 {
                put(scene, x + long / 2, y - 1 + rise / 2, BARK.base);
            }
        } else {
            let size = 1 + pick(index, 2, salt, 2);
            ellipse(scene, x + 1, y + 1, size + 1, size, rgba(0x14100c, 70));
            ellipse(scene, x, y, size + 1, size, DRYSTONE.edge);
            ellipse(scene, x, y, size, (size - 1).max(1), DRYSTONE.base);
            put(scene, x - size + 1, y - 1, DRYSTONE.light);
        }
    }
}

/// Flecks of sun on the floor of a clearing, from the gap overhead.
fn clearing_flecks(scene: &mut Canvas) {
    for index in 0..50 {
        let x = 120 + pick(index, 0, 1395, 160);
        let y = 120 + pick(index, 1, 1395, 90);
        let rx = 1 + pick(index, 2, 1395, 3);
        ellipse(scene, x, y, rx, 1, rgba(SUN, 34));
    }
}

/// Where the ways part at a fork, low in the middle of the picture.
pub const PARTING: (f32, f32) = (196.0, 168.0);

/// The bends a way follows from the fork, as it goes off into the trees.
fn bends(dir: Dir) -> [(f32, f32); 4] {
    match dir {
        Dir::Left => [PARTING, (152.0, 150.0), (100.0, 132.0), (40.0, 118.0)],
        Dir::Ahead => [PARTING, (196.0, 146.0), (196.0, 128.0), (196.0, 112.0)],
        Dir::Right => [PARTING, (240.0, 150.0), (292.0, 132.0), (350.0, 118.0)],
    }
}

/// Points a pixel apart along a line through `points`.
fn dense(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    for pair in points.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = crate::playground::distance(from, to).ceil().max(1.0) as i32;
        for step in 0..steps {
            let t = step as f32 / steps as f32;
            out.push((from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t));
        }
    }
    out.extend(points.last());
    out
}

/// Points every few pixels along a way, from the fork to where it goes out of sight.
pub fn way_path(dir: Dir) -> Vec<(f32, f32)> {
    let bends = bends(dir);
    let mut points = Vec::new();
    for pair in bends.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = ((to.0 - from.0).abs().max((to.1 - from.1).abs()) / 6.0).ceil() as i32;
        for step in 0..steps.max(1) {
            let t = step as f32 / steps.max(1) as f32;
            points.push((from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t));
        }
    }
    points.push(bends[3]);
    points
}

/// Where a way goes out of sight, where the party walks to on it.
pub fn way_end(dir: Dir) -> (f32, f32) {
    bends(dir)[3]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hilltop::{SPOTS, Standing};
    use crate::track::routes::Route;

    fn opaque(canvas: &Canvas) -> bool {
        canvas.pixels().iter().all(|pixel| pixel.a == 255)
    }

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(opaque(&backdrop(&Arrangement::new())));
    }

    #[test]
    fn every_view_off_the_track_fills_every_pixel() {
        assert!(opaque(&nook()), "the nook has holes in it");
        for map in 0..30 {
            let route = Route::of(map);
            for (index, at) in route.forks.iter().enumerate() {
                assert!(
                    opaque(&fork(at)),
                    "map {map}'s fork {index} has holes in it"
                );
            }
            assert!(
                opaque(&dig_site(&route.dig)),
                "map {map}'s dig has holes in it"
            );
        }
    }

    #[test]
    fn the_old_track_is_the_same_every_time() {
        assert_eq!(backdrop(&Arrangement::new()), backdrop(&Arrangement::new()));
        assert_eq!(foreground(), foreground());
        assert_eq!(lamplight(), lamplight());
        assert_eq!(nook(), nook());
        let route = Route::of(11);
        assert_eq!(fork(&route.forks[0]), fork(&route.forks[0]));
        assert_eq!(dig_site(&route.dig), dig_site(&route.dig));
    }

    #[test]
    fn the_foreground_keeps_low_and_off_where_anyone_stands() {
        let front = foreground();
        let (_, top, _, _) = front.alpha_bounds().expect("no bracken at all");
        assert!(top as i32 >= HEIGHT - 20, "the bracken reaches up to {top}");
        for y in 0..HEIGHT {
            for x in 70..WIDTH - 70 {
                assert_eq!(front.get(x, y).a, 0, "bracken in the middle at {x}, {y}");
            }
        }
        let stands = super::super::PLACES
            .iter()
            .map(|(_, at)| *at)
            .chain(SITES.iter().map(|site| site.stand));
        for (x, y) in stands {
            for dy in -30..=0 {
                for dx in -10..=10 {
                    let (px, py) = (x as i32 + dx, y as i32 + dy);
                    assert!(
                        front.get(px, py).a < 128,
                        "bracken over {x}, {y} at {px}, {py}"
                    );
                }
            }
        }
    }

    #[test]
    fn glow_worms_and_fireflies_shine_below_the_sky() {
        let night = night_sky(&backdrop(&Arrangement::new()));
        for lights in [lamplight(), hunt_lamplight()] {
            assert!(lights.alpha_bounds().is_some(), "nothing shines after dark");
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if lights.get(x, y).a > 0 {
                        assert_eq!(night.get(x, y).a, 0, "a light in the sky at {x}, {y}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_moon_rises_in_open_sky() {
        let night = night_sky(&backdrop(&Arrangement::new()));
        assert_eq!(night.get(MOON.0, MOON.1).a, 255, "the moon is hidden");
        for y in HORIZON..HEIGHT {
            for x in 0..WIDTH {
                assert_eq!(night.get(x, y).a, 0, "night sky on the ground at {x}, {y}");
            }
        }
    }

    #[test]
    fn the_old_tree_stands_on_the_hill_as_drawn() {
        assert!((VISTA.tree.1 - hill_crest(VISTA.tree.0)).abs() < 0.5);
    }

    #[test]
    fn whatever_stands_on_the_hilltop_shows_at_the_end_of_the_ride() {
        for standing in Standing::every() {
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
    fn a_stream_crosses_the_way_it_is_across_and_no_other() {
        let distance = |a: &[(f32, f32)], b: &[(f32, f32)]| {
            a.iter()
                .flat_map(|p| b.iter().map(move |q| crate::playground::distance(*p, *q)))
                .fold(f32::MAX, f32::min)
        };
        let coming = dense(&[(192.0, 220.0), (194.0, 196.0), PARTING]);
        for dir in [Dir::Left, Dir::Ahead, Dir::Right] {
            let water = dense(&stream_course(dir));
            assert!(
                distance(&water, &dense(&way_path(dir))) < 1.5,
                "{dir:?}'s stream misses it"
            );
            for other in [Dir::Left, Dir::Ahead, Dir::Right] {
                if other != dir {
                    let near = distance(&water, &dense(&way_path(other)));
                    assert!(near > 10.0, "{dir:?}'s stream runs {near} from {other:?}");
                }
            }
            assert!(
                distance(&water, &coming) > 10.0,
                "{dir:?}'s stream crosses the way in"
            );
        }
    }
}
