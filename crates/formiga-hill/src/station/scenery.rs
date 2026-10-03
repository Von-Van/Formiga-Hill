//! The station's scenery, painted back to front: sky, the Hill, the garden behind the fence, the
//! platform and the line, then the station house, the canopy, and the things standing about.
//!
//! Everything here is painted once into the backdrop, except the chimney smoke, which drifts.

use crate::font::{GLYPH_HEIGHT, draw_text_shadowed, text_width};
use crate::hilltop::{Arrangement, Tint, Vista, skyline};
use crate::kit::{Courses, bush, flower_box, plaster, ridge_tiles, roof, stonework, timber};
use crate::materials::*;
use crate::paint::{
    Ramp, bevel, blit, chance, ellipse, hline, line, mix, noise, polygon, put, rect, rgb, rgba,
    vline,
};
use formiga_art::{Canvas, Rgba};

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
const CHIMNEY_TOP: (i32, i32) = (109, 52);

/// The fixed part of the scene.
/// The display case of kept souvenirs, set into the station house's left wall: its left, top,
/// right and bottom edges, for knowing when the pointer is over it.
pub const CASE: (i32, i32, i32, i32) = (15, 101, 51, 134);

/// The Hill as the station sees it: the old tree at the left of its broad summit, which is wide
/// enough for everything on the Hilltop to stand along it to the tree's right, drawn a little
/// larger than the distance would make them so that each can be made out.
const VISTA: Vista = Vista {
    tree: (292.0, 56.0),
    crest: hill_crest,
    spread: 5.0,
    shrink: 4,
    tint: Tint::Haze(HAZE, 0.15),
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
    planter(&mut scene, 138);
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

const WIDTH: i32 = SCENE_WIDTH as i32;

/// The Hill's turf, its old tree and the path up it, in the Hilltop's own tones, so the Hill
/// seen from the station is the same place, only further off.
const TURF: Ramp = Ramp::new(0x55814c, 0x72a569, 0x86bb7c, 0x9ccb8b, 0xbadca4);
const BARK: Ramp = Ramp::new(0x3e2c22, 0x5a4230, 0x7a5a3a, 0x93714f, 0xae8d68);
const CROWN: Ramp = Ramp::new(0x2b4f31, 0x3f7143, 0x5d9a5a, 0x78b46a, 0x9dcf85);
const EARTH: Ramp = Ramp::new(0x8e7553, 0xb39a6f, 0xcdb688, 0xdcc89a, 0xece0bb);
const CLOUDS: Ramp = Ramp::new(0xc9d7e2, 0xdfe7ee, 0xf4f4f1, 0xfdfbf5, 0xffffff);
const HEDGE: Ramp = Ramp::new(0x2f5733, 0x3f6e40, 0x4f8250, 0x67995e, 0x85b277);
/// The colour of the air a long way off, which everything fades towards with distance.
const HAZE: Rgba = rgb(0xcfe1dc);
const SHADE: Rgba = rgba(0x2c4a2e, 60);

fn sky(scene: &mut Canvas) {
    const BANDS: i32 = 8;
    // Where the last band begins; it runs on down behind the hills to the horizon.
    const LOW: i32 = 96;
    // In bands, as pixel skies are, each dithered a row into the next.
    let band = |y: i32| (y * (BANDS - 1) / LOW).min(BANDS - 1);
    for y in 0..HORIZON {
        for x in 0..WIDTH {
            let mut index = band(y);
            if band(y + 1) != index && (x + y) % 2 == 0 {
                index += 1;
            }
            let color = mix(SKY_TOP, SKY_LOW, index as f32 / (BANDS - 1) as f32);
            scene.set(x, y, color);
        }
    }
    // A big fair-weather cloud over the line, smaller ones drifting, clear of the chimney's smoke
    // and of the old tree, and long thin ones low down where the far hills begin.
    cloud(
        scene,
        (206, 32),
        &[
            (-26, 3, 7),
            (-13, -3, 10),
            (2, -7, 12),
            (16, -2, 9),
            (27, 3, 6),
        ],
    );
    cloud(scene, (44, 24), &[(-10, 0, 6), (0, -4, 8), (10, -1, 6)]);
    cloud(scene, (352, 14), &[(-7, 0, 4), (1, -3, 6), (9, 0, 4)]);
    for (x, y, long) in [(0, 64, 30), (228, 72, 36), (130, 78, 26)] {
        for dx in 0..long {
            let fade = (dx.min(long - dx) * 40).min(150) as u8;
            put(scene, x + dx, y, Rgba { a: fade, ..CLOUD });
            if (4..long - 6).contains(&dx) {
                put(scene, x + dx + 3, y + 1, rgba(0xdfe7ee, fade / 2));
            }
        }
    }
}

/// A heaped cloud from round puffs `(dx, dy, radius)`: flat underneath, sunlit on top and to the
/// left, cool grey below, and edged only along its underside, which is the only hard edge a cloud
/// has. The Hilltop's clouds, so the two skies are one sky.
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
                // Lit towards the upper left of each puff, shaded towards the flat base.
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
                if layer.get(x, y).a == 0 || brightness(color) > brightness(layer.get(x, y)) {
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

/// Noise that rolls smoothly between whole-numbered points, for soft patches rather than
/// speckle.
fn smooth_noise(x: f32, y: f32, salt: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy) = (ease(x - ix as f32), ease(y - iy as f32));
    let corner = |dx: i32, dy: i32| (noise(ix + dx, iy + dy, salt) % 1000) as f32 / 999.0;
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// A point along a smooth curve through `points`, `t` of the way from the first to the last.
fn along(points: &[(f32, f32)], t: f32) -> (f32, f32) {
    let spans = (points.len() - 1) as f32;
    let at = (t * spans).clamp(0.0, spans - 0.001);
    let index = at.floor() as usize;
    let local = at - index as f32;
    let get = |i: isize| points[i.clamp(0, points.len() as isize - 1) as usize];
    let (p0, p1, p2, p3) = (
        get(index as isize - 1),
        get(index as isize),
        get(index as isize + 1),
        get(index as isize + 2),
    );
    // Catmull-Rom, so the curve passes through every point.
    let blend = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (c - a) * local
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * local * local
            + (3.0 * b - a - 3.0 * c + d) * local * local * local)
    };
    (blend(p0.0, p1.0, p2.0, p3.0), blend(p0.1, p1.1, p2.1, p3.1))
}

// ---------------------------------------------------------------------------------------------
// The country beyond, and the Hill
// ---------------------------------------------------------------------------------------------

fn hills(scene: &mut Canvas) {
    for range in RANGES {
        country(scene, &range);
    }
    hill(scene);
    hill_path(scene);
    old_tree(scene);
    meadow(scene);
}

/// One range of the country beyond the line, seen across the valley: how high it stands and how
/// its crest rolls, the colour of its grass, and how much the air between pales it.
struct Range {
    base: f32,
    rise: f32,
    phase: f32,
    grass: Rgba,
    far: f32,
    /// How big its fields look, across and down, in pixels: they grow as the land comes nearer.
    fields: (f32, f32),
    /// Where woods stand on its slope.
    woods: &'static [(i32, i32)],
    salt: u32,
}

/// Three ranges, furthest first: blue downs along the horizon, a patchwork of fields and
/// hedgerows below them, and the nearest rise, with its hedgerow trees and a wood.
const RANGES: [Range; 3] = [
    Range {
        base: 99.0,
        rise: 9.0,
        phase: 0.6,
        grass: rgb(0x8fb3ac),
        far: 0.55,
        fields: (0.0, 0.0),
        woods: &[],
        salt: 860,
    },
    Range {
        base: 110.0,
        rise: 5.0,
        phase: 2.3,
        grass: FAR_HILL,
        far: 0.3,
        fields: (16.0, 3.0),
        woods: &[(6, 104), (186, 107)],
        salt: 870,
    },
    Range {
        base: 121.0,
        rise: 5.0,
        phase: 4.1,
        grass: rgb(0x86bb72),
        far: 0.24,
        fields: (26.0, 4.5),
        woods: &[(176, 118)],
        salt: 880,
    },
];

impl Range {
    fn crest(&self, x: i32) -> i32 {
        let x = x as f32;
        let phase = self.phase;
        (self.base
            - self.rise
                * (1.0
                    + 0.55 * (x / 47.0 + phase).sin()
                    + 0.3 * (x / 23.0 + phase * 2.0).sin()
                    + 0.12 * (x / 9.0 + phase).sin()))
        .round() as i32
    }

    fn fade(&self, color: Rgba) -> Rgba {
        mix(color, HAZE, self.far)
    }

    /// Which field a point below the crest falls in: rows that follow the lie of the land, their
    /// hedges wandering and some fields deeper than others, each row cut into fields of its own
    /// widths by hedges running aslant, so they never line up into a grid.
    fn field(&self, x: i32, y: i32) -> (i32, i32) {
        let (across, down) = self.fields;
        let wander = (smooth_noise(x as f32 / 24.0, 0.5, self.salt) - 0.5) * down * 1.8;
        let below = (y - self.crest(x)) as f32 + wander;
        let mut row = (below / down).floor() as i32;
        if noise(row, 2, self.salt).is_multiple_of(3) {
            row -= 1;
        }
        let width = across * (0.6 + (noise(row, 0, self.salt) % 80) as f32 / 100.0);
        let offset = (noise(row, 1, self.salt) % 100) as f32 / 100.0;
        let aslant = (noise(row, 3, self.salt) % 100) as f32 / 50.0 - 1.0;
        let column = ((x as f32 + below * aslant * 1.5) / width + offset).floor() as i32;
        (row, column)
    }
}

/// What a field grows, as two tones: pasture mostly, with hay, ripe wheat and plough among it.
fn crop(kind: u32) -> (Rgba, Rgba) {
    match kind % 14 {
        0..=4 => (rgb(0x8cc27a), rgb(0x7fb66f)),
        5..=7 => (rgb(0xa3cf8a), rgb(0x94c47e)),
        8 | 9 => (rgb(0x74a862), rgb(0x689b58)),
        10 => (rgb(0xdcc57c), rgb(0xc8b06a)),
        11 => (rgb(0xe8d898), rgb(0xd8c47e)),
        12 => (rgb(0xb39272), rgb(0x977657)),
        _ => (rgb(0x95ba66), rgb(0x7c9e56)),
    }
}

/// A range of the country: lit on the slopes that face left, edged along its crest in a darker
/// shade of its own green, and, where it is near enough, cut into fields by hedgerows, with
/// trees standing in them and a wood on its slope.
fn country(scene: &mut Canvas, range: &Range) {
    let edge = mix(range.grass, rgb(0x3d5e58), 0.35);
    let lit = mix(range.grass, rgb(0xf4f6ee), 0.3);
    let fielded = range.fields.0 > 0.0;
    let mut trees = Vec::new();
    for x in 0..WIDTH {
        let crest = range.crest(x);
        let facing = range.crest(x + 2) - range.crest(x - 2);
        for y in crest..=HORIZON {
            let mut color = if fielded && y > crest + 1 {
                let here = range.field(x, y);
                let (a, b) = crop(noise(here.0, here.1, range.salt + 1));
                let hedge = range.field(x, y - 1) != here
                    || (range.field(x - 1, y) != here && y > crest + 2);
                if hedge {
                    if range.fields.0 > 20.0 && chance(x, y, range.salt + 2, 26) {
                        trees.push((x, y));
                    }
                    HEDGE.shadow
                } else if range.field(x - 1, y - 1) != here {
                    // The hedge's shadow, falling to the lower right.
                    mix(a, HEDGE.shadow, 0.3)
                } else if noise(x, y, range.salt + 3).is_multiple_of(9) {
                    b
                } else {
                    a
                }
            } else if smooth_noise(x as f32 / 7.0, y as f32 / 3.0, range.salt + 6) > 0.72 {
                // Woods too far off to make out, darkening the far slopes in patches.
                mix(range.grass, edge, 0.45)
            } else {
                range.grass
            };
            // Slopes facing left catch the light; those facing right are in shade.
            if y == crest || (y == crest + 1 && range.crest(x - 1) > crest + 1) {
                color = edge;
            } else if facing > 0 && y < crest + 3 {
                color = mix(color, lit, 0.6);
            } else if facing < 0 && y < crest + 3 {
                color = mix(color, edge, 0.35);
            }
            // Deeper into the valley, the land is in the shade of the nearer ranges.
            let depth = ((y - crest) as f32 / 14.0).min(1.0);
            color = mix(color, HEDGE.edge, depth * 0.12);
            scene.set(x, y, range.fade(color));
        }
    }
    for (x, y) in trees {
        hedgerow_tree(scene, x, y, range.far);
    }
    for &at in range.woods {
        wood(scene, at, range);
    }
}

/// A tree in a hedgerow: a dark round head lit on its upper left, paled by `far`.
fn hedgerow_tree(scene: &mut Canvas, x: i32, y: i32, far: f32) {
    let fade = |color| mix(color, HAZE, far);
    ellipse(scene, x + 1, y, 2, 2, fade(HEDGE.edge));
    ellipse(scene, x, y - 1, 2, 1, fade(HEDGE.base));
    put(scene, x - 1, y - 2, fade(HEDGE.light));
    put(scene, x + 1, y + 1, fade(HEDGE.shadow));
}

/// A wood on a range's slope: tree heads crowded into a low mound, the nearer in front of the
/// further, each lit on its upper left.
fn wood(scene: &mut Canvas, (cx, cy): (i32, i32), range: &Range) {
    let size = if range.fields.0 > 20.0 { 2 } else { 1 };
    let span = 10 * size + 4;
    let mut heads: Vec<(i32, i32)> = (0..span * 2)
        .map(|index| {
            let x = cx - span + (noise(index, 0, range.salt + 4) % (span as u32 * 2)) as i32;
            let hump = (span - (x - cx).abs()) * 3 / span;
            let y = cy - (noise(index, 1, range.salt + 5) % (hump as u32 + 2)) as i32;
            (x, y)
        })
        .collect();
    heads.sort_by_key(|&(x, y)| (y, x));
    for (x, y) in heads {
        ellipse(scene, x + 1, y + 1, size + 1, size, range.fade(HEDGE.edge));
        ellipse(scene, x, y, size, size, range.fade(HEDGE.shadow));
        put(scene, x - 1, y - size + 1, range.fade(HEDGE.base));
        if size > 1 {
            put(scene, x - 1, y - 1, range.fade(HEDGE.light));
        }
    }
}

/// The middle of the Hill, where its summit is highest.
const HILL_MIDDLE: f32 = 326.0;
/// Where the summit stands, and how far it rounds down across its breadth.
const HILL_TOP: f32 = 53.0;
const HILL_DOME: f32 = 5.0;
/// Half the summit's breadth, before it falls away into the slopes, and how far the slopes run.
const SUMMIT_HALF: f32 = 36.0;
const SLOPE_RUN: f32 = 72.0;
/// How high the summit stands above the horizon.
const HILL_RISE: f32 = HORIZON as f32 - HILL_TOP;

/// The Hill's skyline at a point across: a broad summit, gently domed, rounding over into long
/// slopes that flare out at the foot as real hills do.
fn hill_crest(x: f32) -> f32 {
    let off = (x - HILL_MIDDLE).abs();
    let dome = HILL_DOME * (1.0 - (-(off / SUMMIT_HALF).powi(2)).exp());
    let slope = ((off - SUMMIT_HALF).max(0.0) / SLOPE_RUN).powi(2);
    HILL_TOP + dome + (HILL_RISE - HILL_DOME) * (1.0 - (-slope).exp())
}

/// How much the air pales the Hill at a row: a little at the foot, more up at the summit, which
/// is further off.
fn hill_haze(y: i32) -> f32 {
    let up = ((HORIZON - y) as f32 / HILL_RISE).clamp(0.0, 1.0);
    0.02 + up * 0.12
}

/// How far up the Hill's face a point is, 0 at the foot and 1 on the skyline above it. Lines of
/// the same height arch over the Hill as its skyline does, as they would round any dome seen
/// from below it.
fn hill_up(x: i32, y: i32) -> f32 {
    let top = hill_crest(x as f32);
    ((HORIZON - y) as f32 / (HORIZON as f32 - top).max(1.0)).clamp(0.0, 1.0)
}

/// How lit the Hill's turf is at a point, 0 to 1: brightest on the slopes turned towards the sun
/// on the left and up on the summit, rounding into shade on the right and down towards the
/// foot, with broad soft patches where the grass grows differently and a cloud's shadow lying
/// across it, so the face reads as a great curve of land rather than a flat green.
fn hill_light(x: i32, y: i32) -> f32 {
    let across = ((x as f32 - HILL_MIDDLE) / 100.0).clamp(-1.2, 1.0);
    let up = hill_up(x, y);
    let patches = smooth_noise(x as f32 / 38.0, y as f32 / 13.0, 891) * 0.65
        + smooth_noise(x as f32 / 14.0, y as f32 / 6.0, 892) * 0.35;
    // The shadow of a cloud passing over, lying across the right of the face.
    let (cx, cy) = ((x as f32 - 334.0) / 44.0, (y as f32 - 108.0) / 13.0);
    let cloud = if cx * cx + cy * cy <= 1.0 { 0.28 } else { 0.0 };
    0.5 - 0.68 * across + 0.32 * (up - 0.5) + (patches - 0.5) * 0.85 - cloud
}

/// The turf's tone at a step of its ramp, from -1 (deep in the grass) to 4 (sunlit).
fn turf(step: i32) -> Rgba {
    match step {
        ..=-1 => mix(TURF.edge, TURF.shadow, 0.45),
        0 => TURF.shadow,
        1 => mix(TURF.shadow, TURF.base, 0.5),
        2 => TURF.base,
        3 => mix(TURF.base, TURF.light, 0.5),
        _ => TURF.light,
    }
}

/// Which step of the turf's ramp a pixel of the Hill takes: its light, its edges broken a pixel
/// at a time.
fn turf_step(x: i32, y: i32) -> i32 {
    let scatter = (noise(x, y, 893) % 100) as f32 / 100.0 - 0.5;
    (hill_light(x, y) * 4.0 + scatter * 0.55)
        .round()
        .clamp(0.0, 4.0) as i32
}

/// A colour on the Hill at a row, paled by the air.
fn on_hill(color: Rgba, y: i32) -> Rgba {
    mix(color, HAZE, hill_haze(y))
}

/// Gorse on the Hill's slopes: where each patch grows, and how many bushes are in it.
const GORSE: [(i32, i32, i32); 7] = [
    (212, 118, 5),
    (264, 112, 2),
    (324, 101, 4),
    (343, 82, 3),
    (374, 106, 5),
    (312, 124, 3),
    (306, 70, 2),
];

/// The Hill, close by on the right: a great dome of turf, lit from the left, its skyline
/// outlined and grass standing up along it, sheep-tracks running round its slopes, gorse here
/// and there, and a hedge along its foot.
fn hill(scene: &mut Canvas) {
    /// How many sheep-tracks run round the Hill, from the foot to the summit.
    const TRACKS: i32 = 13;
    for x in 0..WIDTH {
        let top = hill_crest(x as f32).round() as i32;
        if top >= HORIZON {
            continue;
        }
        for y in top..HORIZON {
            let mut step = turf_step(x, y);
            // Grass a little longer than the turf, in short darker blades, closer together as
            // the slope comes nearer.
            if chance(x, y, 897, (8.0 + (1.0 - hill_up(x, y)) * 30.0) as u32) {
                step -= 1;
            }
            scene.set(x, y, on_hill(turf(step), y));
        }
        // Sheep-tracks, terraced round the slopes along lines of the same height, so they arch
        // over the Hill as its skyline does, wider apart as they come nearer: each a shade
        // darker, its lip below catching the light, broken where the turf has grown over.
        let wobble = smooth_noise(x as f32 / 11.0, 0.0, 894) * 0.6;
        for track in 1..TRACKS {
            let up = (track as f32 / TRACKS as f32).powf(0.8);
            let y = (HORIZON as f32 - (up * (HORIZON - top) as f32) + wobble).round() as i32;
            if up > 0.85 || y <= top + 2 || chance(x.div_euclid(5), track, 895, 100) {
                continue;
            }
            scene.set(x, y, on_hill(turf(turf_step(x, y) - 1), y));
            if chance(x, y, 896, 150) {
                scene.set(x, y + 1, on_hill(turf(turf_step(x, y + 1) + 1), y));
            }
        }
        // The skyline, outlined in the turf's own shade, catching the light just under it where
        // it faces the sun, with grass standing up along it.
        scene.set(x, top, on_hill(TURF.shadow, top));
        if x < HILL_MIDDLE as i32 + 24 {
            put(scene, x, top + 1, on_hill(TURF.light, top));
        }
        if chance(x, 0, 898, 70) {
            let tall = 1 + (noise(x, 1, 899) % 2) as i32;
            vline(scene, x, top - tall, tall, on_hill(TURF.shadow, top));
        }
    }
    for (index, &(cx, cy, bushes)) in GORSE.iter().enumerate() {
        // Back to front, so the nearer bushes of a patch stand in front of the further.
        let mut patch: Vec<(i32, i32, i32)> = (0..bushes)
            .map(|bush| {
                let salt = (index * 10) as i32 + bush;
                let x = cx + bush * 4 - bushes * 2 + (noise(salt, 0, 900) % 3) as i32;
                let y = cy + (noise(salt, 1, 901) % 4) as i32 - 2 + (bush % 2) * 2;
                (x, y, salt)
            })
            .collect();
        patch.sort_by_key(|&(x, y, _)| (y, x));
        for (x, y, salt) in patch {
            gorse(scene, x, y, salt);
        }
    }
    // Daisies and buttercups on the lower slopes, in little scatters.
    for index in 0..14 {
        let x = 200 + (noise(index, 0, 909) % 184) as i32;
        let y = HORIZON - 6 - (noise(index, 1, 910) % 22) as i32;
        for floret in 0..3 {
            let (fx, fy) = (
                x + (noise(index, floret, 911) % 7) as i32 - 3,
                y + (noise(floret, index, 912) % 3) as i32 - 1,
            );
            if fy <= hill_crest(fx as f32) as i32 + 2 || path_near(fx, fy, 2.0) {
                continue;
            }
            let blossom = if noise(index, floret, 913).is_multiple_of(3) {
                BLOSSOMS[2]
            } else {
                BLOSSOMS[3]
            };
            put(scene, fx, fy, on_hill(blossom, fy));
        }
    }
    // A hedge along the foot of the Hill, with trees standing in it.
    for x in 0..WIDTH {
        let top = hill_crest(x as f32).round() as i32;
        if top > HORIZON - 4 {
            continue;
        }
        let rise = (noise(x.div_euclid(2), 0, 902) % 2) as i32;
        for y in HORIZON - 3 - rise..HORIZON {
            let color = if y == HORIZON - 3 - rise {
                HEDGE.base
            } else if chance(x, y, 903, 60) {
                HEDGE.light
            } else {
                HEDGE.shadow
            };
            scene.set(x, y, on_hill(color, y));
        }
        if chance(x, 1, 904, 14) && !path_near(x, HORIZON - 4, 6.0) {
            hedgerow_tree(scene, x, HORIZON - 5, hill_haze(HORIZON - 4));
        }
    }
}

/// A gorse bush on the Hill: a dark, prickly mound lit on its upper left, its shade falling
/// downhill to the right, flowering yellow, bigger the nearer it grows.
fn gorse(scene: &mut Canvas, x: i32, y: i32, salt: i32) {
    if path_near(x, y, 5.0) {
        return;
    }
    let (rx, ry) = if y > 104 {
        (3, 3)
    } else if y > 80 {
        (3, 2)
    } else {
        (2, 2)
    };
    // A round bush sitting on the ground: its middle is up off the turf.
    let middle = y - ry + 1;
    let inside = |dx: i32, dy: i32| {
        middle + dy <= y && (dx * dx * ry * ry + dy * dy * rx * rx) <= rx * rx * ry * ry
    };
    ellipse(scene, x + 2, y, rx, 1, SHADE);
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            if !inside(dx, dy) {
                continue;
            }
            let rim = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(ox, oy)| !inside(dx + ox, dy + oy));
            let lit = -(dx as f32) / rx as f32 - dy as f32 / ry as f32;
            let (px, py) = (x + dx, middle + dy);
            let color = if rim {
                HEDGE.edge
            } else if lit > 0.7 && !chance(px, py, 906, 50) {
                HEDGE.light
            } else if lit > -0.2 && !chance(px, py, 907, 60) {
                HEDGE.base
            } else {
                HEDGE.shadow
            };
            put(scene, px, py, on_hill(color, y));
        }
    }
    // Yellow flowers among the prickles.
    for flower in 0..rx - 1 {
        let dx = (noise(salt, flower, 905) % (rx as u32 * 2 - 1)) as i32 - rx + 1;
        let dy = (noise(flower, salt, 908) % ry as u32) as i32 - ry / 2;
        if inside(dx, dy) && inside(dx, dy - 1) && inside(dx, dy + 1) {
            put(scene, x + dx, middle + dy, on_hill(rgb(0xe8c860), y));
        }
    }
}

/// The way up the Hill, from the garden gate to the old tree on top: switching back and forth
/// across the slope, the turns closer together as it climbs.
const HILL_PATH: [(f32, f32); 15] = [
    (250.0, 132.0),
    (254.0, 126.0),
    (276.0, 121.0),
    (299.0, 116.0),
    (295.0, 110.0),
    (272.0, 105.0),
    (255.0, 100.0),
    (259.0, 94.0),
    (280.0, 89.0),
    (297.0, 84.0),
    (295.0, 79.0),
    (282.0, 74.0),
    (280.0, 68.0),
    (287.0, 62.0),
    (292.0, 57.0),
];

/// Points along the path up the Hill, a pixel or so apart, with how wide it is there.
fn path_points() -> Vec<(f32, f32, f32)> {
    let steps = 700;
    (0..=steps)
        .map(|step| {
            let (x, y) = along(&HILL_PATH, step as f32 / steps as f32);
            let up = ((HORIZON as f32 - y) / HILL_RISE).clamp(0.0, 1.0);
            (x, y, 3.2 - up * 2.2)
        })
        .collect()
}

/// Whether a point is within `room` of the path.
fn path_near(x: i32, y: i32, room: f32) -> bool {
    HILL_PATH.windows(2).any(|pair| {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let t =
            (((x as f32 - x0) * dx + (y as f32 - y0) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (px, py) = (x0 + dx * t - x as f32, y0 + dy * t - y as f32);
        px * px + py * py <= room * room
    })
}

/// The path up the Hill: worn earth, pale where it is trodden, its upper edge in the shade of
/// the turf above it and its lower edge a bank catching the light; narrower the further it goes.
fn hill_path(scene: &mut Canvas) {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for (x, y, width) in path_points() {
        let half = width / 2.0;
        let (left, right) = ((x - half).round() as i32, (x + half).round() as i32);
        let row = y.round() as i32;
        for px in left..right.max(left + 1) {
            layer.set(px, row, EARTH.light);
        }
    }
    for y in 0..HORIZON {
        for x in 0..WIDTH {
            if layer.get(x, y).a == 0 {
                continue;
            }
            let color = if layer.get(x, y - 1).a == 0 {
                EARTH.shadow
            } else if chance(x, y, 903, 40) {
                EARTH.base
            } else if chance(x, y, 904, 20) {
                EARTH.shine
            } else {
                EARTH.light
            };
            scene.set(x, y, on_hill(color, y));
            if layer.get(x, y + 1).a == 0 {
                put(scene, x, y + 1, on_hill(turf(turf_step(x, y + 1) + 1), y));
            }
        }
    }
}

/// The Hilltop's old tree, as the Hilltop draws it: the masses of its crown `(x, y, radius)`, and
/// where its trunk meets the ground there. The station draws the same tree from these, smaller.
const HILLTOP_CROWN: [(i32, i32, i32); 8] = [
    (50, 30, 27),
    (26, 20, 16),
    (74, 22, 15),
    (8, 36, 13),
    (22, 50, 18),
    (80, 46, 15),
    (66, 54, 9),
    (6, 56, 9),
];
const HILLTOP_FOOT: (i32, i32) = (53, 112);
/// How much smaller the old tree looks from the station than from under it.
const TREE_SCALE: f32 = 0.36;

/// The crown's masses as the station sees them, `(x, y, radius)` in the scene.
fn tree_masses() -> [(i32, i32, i32); 8] {
    let (fx, fy) = (VISTA.tree.0.round() as i32, VISTA.tree.1.round() as i32);
    HILLTOP_CROWN.map(|(x, y, radius)| {
        (
            fx + ((x - HILLTOP_FOOT.0) as f32 * TREE_SCALE).round() as i32,
            fy + ((y - HILLTOP_FOOT.1) as f32 * TREE_SCALE).round() as i32,
            (radius as f32 * TREE_SCALE).round() as i32,
        )
    })
}

/// Whether a point is within the crown, `inset` pixels in from its edge.
fn in_tree_crown(masses: &[(i32, i32, i32)], x: i32, y: i32, inset: i32) -> bool {
    masses.iter().any(|&(cx, cy, radius)| {
        let reach = radius - inset;
        reach > 0 && (x - cx).pow(2) + (y - cy).pow(2) <= reach * reach
    })
}

/// The old tree on the summit, seen from the station: its crooked trunk, its crown in clumps of
/// leaves lit from the upper left, its shade on the turf, all a little paled by the distance.
fn old_tree(scene: &mut Canvas) {
    let (foot_x, foot_y) = (VISTA.tree.0.round() as i32, VISTA.tree.1.round() as i32);
    let masses = tree_masses();
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    // The trunk: old and a little crooked, flaring into roots at its foot, furrowed, lit on the
    // left, in shade under the crown.
    for y in foot_y - 22..=foot_y {
        let up = (foot_y - y) as f32;
        let centre = foot_x as f32 - up * 0.06 + (up / 4.5).sin() * 0.9;
        let half = 2.2 + (4.0 - up).max(0.0).powi(2) * 0.18;
        let (left, right) = (
            (centre - half).round() as i32,
            (centre + half).round() as i32,
        );
        for x in left..=right {
            let mut color = if x == left || x == right {
                BARK.edge
            } else if x == left + 1 {
                BARK.light
            } else if x == right - 1 {
                BARK.shadow
            } else {
                BARK.base
            };
            if x > left + 1 && x < right - 1 && noise(x, y.div_euclid(3), 914).is_multiple_of(3) {
                color = mix(color, BARK.edge, 0.5);
            }
            if up > 9.0 {
                color = mix(color, BARK.edge, ((up - 9.0) / 8.0).min(0.6));
            }
            layer.set(x, y, color);
        }
    }
    // The crown, as clumps back to front, then the limbs among the lowest of them.
    let mut crown = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let (left, right) = (foot_x - 30, foot_x + 30);
    let (top, bottom) = (foot_y - 45, foot_y);
    for y in top..bottom {
        for x in left..right {
            if in_tree_crown(&masses, x, y, 1) {
                crown.set(x, y, CROWN.edge);
            }
        }
    }
    let mut clumps = Vec::new();
    for gy in (top..bottom).step_by(3) {
        for gx in (left..right).step_by(4) {
            let x = gx + (noise(gx, gy, 905) % 3) as i32 - 1 + if gy % 6 == 0 { 2 } else { 0 };
            let y = gy + (noise(gx, gy, 906) % 3) as i32 - 1;
            if in_tree_crown(&masses, x, y, 1) {
                let radius = if in_tree_crown(&masses, x, y, 4) {
                    4
                } else {
                    3
                };
                clumps.push((x, y, radius));
            }
        }
    }
    clumps.sort_by_key(|&(x, y, _)| (y, x));
    for &(x, y, radius) in &clumps {
        clump(
            &mut crown,
            (x, y, radius),
            clump_tones(&masses, x, y, foot_x, foot_y),
        );
    }
    for (from, to) in [
        ((foot_x - 1, foot_y - 12), (foot_x - 7, foot_y - 18)),
        ((foot_x + 1, foot_y - 12), (foot_x + 7, foot_y - 17)),
    ] {
        line(&mut crown, from, to, mix(BARK.shadow, BARK.edge, 0.3));
        line(
            &mut crown,
            (from.0, from.1 + 1),
            (to.0, to.1 + 1),
            BARK.edge,
        );
    }
    // One outline round the whole crown, in its own darkest green.
    let mut edges = Vec::new();
    for y in top - 2..bottom + 2 {
        for x in left - 2..right + 2 {
            if crown.get(x, y).a == 0 {
                continue;
            }
            let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(dx, dy)| crown.get(x + dx, y + dy).a == 0);
            if open && !is_bark(crown.get(x, y)) {
                edges.push((x, y));
            }
        }
    }
    for (x, y) in edges {
        crown.set(x, y, CROWN.edge);
    }
    blit(&mut layer, &crown, 0, 0);
    // Its shade on the turf, falling away to the right.
    for y in foot_y - 1..foot_y + 3 {
        for x in foot_x - 6..foot_x + 16 {
            let (dx, dy) = (
                (x - foot_x - 4) as f32 / 11.0,
                (y - foot_y - 1) as f32 / 2.0,
            );
            if dx * dx + dy * dy <= 1.0 {
                put(scene, x, y, SHADE);
            }
        }
    }
    for y in top - 2..=foot_y + 2 {
        for x in left - 2..right + 2 {
            let pixel = layer.get(x, y);
            if pixel.a > 0 {
                put(scene, x, y, mix(pixel, HAZE, hill_haze(y) * 0.8));
            }
        }
    }
}

/// Whether a colour is one of the limbs' bark tones, which keep their own outline.
fn is_bark(color: Rgba) -> bool {
    color.r > color.g
}

/// The tones for a clump of leaves at a point: lit by where it sits on its own mass, and on the
/// tree as a whole.
fn clump_tones(
    masses: &[(i32, i32, i32)],
    x: i32,
    y: i32,
    foot_x: i32,
    foot_y: i32,
) -> ([Rgba; 3], Option<Rgba>) {
    let (mx, my, radius) = masses
        .iter()
        .copied()
        .filter(|&(cx, cy, r)| (x - cx).pow(2) + (y - cy).pow(2) <= r * r)
        .min_by_key(|&(_, _, r)| r)
        .unwrap_or(masses[0]);
    let local = 0.5 - ((x - mx) + (y - my)) as f32 / (3.0 * radius.max(1) as f32);
    let (across, up) = ((x - foot_x + 18) as f32 / 36.0, (foot_y - y) as f32 / 36.0);
    let whole = 1.0 - across * 0.45 - (1.0 - up) * 0.55;
    let lit = local * 0.45 + whole * 0.55;
    if lit > 0.62 {
        ([CROWN.shadow, CROWN.base, CROWN.light], Some(CROWN.shine))
    } else if lit > 0.44 {
        ([CROWN.shadow, CROWN.base, CROWN.light], None)
    } else if lit > 0.28 {
        ([CROWN.edge, CROWN.shadow, CROWN.base], None)
    } else {
        (
            [CROWN.edge, mix(CROWN.edge, CROWN.shadow, 0.5), CROWN.shadow],
            None,
        )
    }
}

/// A clump of leaves: a dark underside, its body, and a lit crescent on its upper left.
fn clump(
    layer: &mut Canvas,
    (cx, cy, radius): (i32, i32, i32),
    ([dark, mid, bright], shine): ([Rgba; 3], Option<Rgba>),
) {
    ellipse(layer, cx + 1, cy + 1, radius, radius - 1, dark);
    ellipse(layer, cx, cy, radius - 1, (radius - 2).max(1), mid);
    for y in cy - radius..=cy + radius {
        for x in cx - radius..=cx + radius {
            let (dx, dy) = (x - cx, y - cy);
            let (rx, ry) = (radius - 1, (radius - 2).max(1));
            if dx * dx * ry * ry + dy * dy * rx * rx > rx * rx * ry * ry {
                continue;
            }
            // Ragged, leafy edges to the light, not a smooth arc.
            let ragged = (noise(x, y, 907) % 2) as i32;
            if dx + dy < -radius / 2 - ragged + 1 {
                layer.set(x, y, bright);
            } else if dx + dy > radius / 2 + ragged && noise(x, y, 908).is_multiple_of(3) {
                layer.set(x, y, dark);
            }
        }
    }
    if let Some(shine) = shine {
        layer.set(cx - radius / 2, cy - radius / 2, shine);
    }
}

/// The meadow behind the garden, between the hedge at the foot of the Hill and the fence, with
/// the path crossing it to the gate.
fn meadow(scene: &mut Canvas) {
    for y in HORIZON..PLATFORM_BACK {
        for x in 0..WIDTH {
            let patch = smooth_noise(x as f32 / 14.0, y as f32 / 4.0, 909);
            let mut color = if patch > 0.62 {
                mix(GRASS, GRASS_LIGHT, 0.5)
            } else if patch < 0.3 {
                mix(GRASS, GRASS_DARK, 0.4)
            } else {
                GRASS
            };
            if chance(x, y, 910, 22) {
                color = GRASS_DARK;
            } else if chance(x, y, 911, 10) {
                color = GRASS_LIGHT;
            }
            scene.set(x, y, color);
        }
        // The hedge's shade along the back.
        if y < HORIZON + 2 {
            hline(
                scene,
                0,
                y,
                WIDTH,
                rgba(0x2c4a2e, 50 - (y - HORIZON) as u8 * 20),
            );
        }
    }
    // The path across it to the gate.
    for y in HORIZON..PLATFORM_BACK {
        let x = 250 + (y - HORIZON) * 2 / (PLATFORM_BACK - HORIZON);
        put(scene, x - 1, y, EARTH.shadow);
        hline(scene, x, y, 2, PATH);
        put(scene, x + 2, y, EARTH.base);
    }
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
/// The ticket window's top-left corner: shared with the clerk's lamp that lights it after dark.
const TICKET_WINDOW: (i32, i32) = (106, 110);
const HOUSE_RIGHT: i32 = 134;
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
    dormer(scene, 72);

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
    for post in [HOUSE_LEFT, 52, 100, HOUSE_RIGHT - 4] {
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

    flower_box(scene, 16, 136, 34);
    door(scene, 60, 106);
    timetable(scene, 84, 109);
    ticket_window(scene, TICKET_WINDOW.0, TICKET_WINDOW.1);

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
    let (left, top, width, bottom): (i32, i32, i32, i32) = (103, 58, 12, 80);
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
/// How many places each of the display case's two shelves has.
const PER_SHELF: usize = 4;

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
    // Two shelves, four places on each, in the catalogue's order.
    let shelves = [inner_top + 8, inner_top + 20];
    for shelf in shelves {
        hline(scene, inner_left, shelf, inner_width, PLANK.light);
        hline(scene, inner_left, shelf + 1, inner_width, PLANK.shadow);
    }
    let pitch = crate::keepsake_art::ICON + 1;
    for (index, id) in crate::story::souvenirs::ids().into_iter().enumerate() {
        let shelf = shelves[index / PER_SHELF];
        let x = inner_left + (index % PER_SHELF) as i32 * pitch;
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
    // Between the second and third places, so it never cuts across a souvenir.
    let seam = inner_left + 2 * pitch - 1;
    vline(scene, seam, glass_top + 1, sill - glass_top - 1, IRON.edge);
    for (dx, dy) in [(0, 2), (1, 1), (2, 0)] {
        put(
            scene,
            right - 6 + dx,
            inner_top + 1 + dy,
            rgba(0xffffff, 120),
        );
    }
    for (dx, dy) in [(0, 2), (1, 1), (2, 0)] {
        put(scene, seam + 4 + dx, inner_top + 9 + dy, rgba(0xffffff, 90));
    }
    for handle in [seam - 2, seam + 1] {
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
    // Centred over the canopy.
    let (width, top, height) = (86, 82, 15);
    let left = (CANOPY_LEFT + CANOPY_RIGHT - width) / 2;
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

// ---------------------------------------------------------------------------------------------
// After dark
// ---------------------------------------------------------------------------------------------

/// Where the moon rides, between the chimney and the clouds.
const MOON: (i32, i32) = (132, 20);

/// What shines after dark: the platform lamp and the pool of light it throws on the boards, the
/// clerk's lamp in the ticket office, and the display case, lit for the night.
pub fn lamplight() -> Canvas {
    let mut lamps = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let x = 356;
    crate::daylight::glow(&mut lamps, (x + 1, 94), 40, rgb(0xffd77a));
    ellipse(
        &mut lamps,
        x + 1,
        PLATFORM_BACK + 6,
        30,
        7,
        rgba(0xffd77a, 46),
    );
    let (lx, ly) = (x - 4, 86);
    for (pane, glow) in [(lx + 1, GLOW[1]), (lx + 6, GLOW[0])] {
        rect(&mut lamps, pane, ly + 1, 4, 10, glow);
        vline(&mut lamps, pane, ly + 1, 10, GLOW[2]);
    }
    let (tx, ty) = TICKET_WINDOW;
    rect(&mut lamps, tx + 2, ty + 2, 18, 13, rgba(0xffc860, 140));
    crate::daylight::glow(&mut lamps, (tx + 11, ty + 8), 20, rgba(0xffc860, 200));
    let (left, top, right, bottom) = CASE;
    rect(
        &mut lamps,
        left + 2,
        top + 2,
        right - left - 4,
        bottom - top - 4,
        rgba(0xffe2a0, 96),
    );
    lamps
}

/// The sky after dark, deepened, with its stars and the moon.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, HORIZON, Some(MOON))
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
    fn every_souvenir_has_a_place_of_its_own_in_the_case() {
        let ids: Vec<String> = crate::story::souvenirs::ids()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let all = backdrop(&ids, &Arrangement::new());
        for id in &ids {
            let others: Vec<String> = ids.iter().filter(|kept| *kept != id).cloned().collect();
            assert_ne!(
                all,
                backdrop(&others, &Arrangement::new()),
                "{id} doesn't show"
            );
        }
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
