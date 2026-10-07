//! The clearing that isn't on any map: deep in the Woods and somehow also not. The sky has gone
//! violet and glows magenta low down behind where the Sovereign hangs, so the ring of trees
//! standing back around the clearing is all silhouette, rimmed in that light on whichever side
//! faces it. The floor is a desktop seen far too close: dark and glossy, ruled into a grid that
//! runs away to the trees, giving back the glow, with giant icons sunk into it like ghosts.
//!
//! It is the one place in Hill meant to look as if it doesn't belong, so it is lit from behind
//! rather than from the upper left. The trees are still shaded and outlined in darker tones of
//! their own, and the stage where everyone stands is left plain.

use super::{GROUND, SCENE_HEIGHT, SCENE_WIDTH};
use crate::paint::{Ramp, blit, chance, ellipse, line, mix, noise, pick, polygon, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;
use std::ops::Range;

const WIDTH: i32 = SCENE_WIDTH as i32;
const HEIGHT: i32 = SCENE_HEIGHT as i32;
/// Where the floor meets the foot of the far trees.
const HORIZON: i32 = 150;
/// The heart of the glow behind where the Sovereign hangs: everything is lit from here.
const GLOW: (f32, f32) = (300.0, 104.0);

/// The night, darkest first: blue-violet overhead, going to magenta towards the glow.
const NIGHT: [u32; 12] = [
    0x08061a, 0x0d0922, 0x130c2c, 0x1a1037, 0x231444, 0x2e1851, 0x3b1c5d, 0x4b2068, 0x5e2572,
    0x742c7c, 0x8e3586, 0xac428f,
];
/// The far side of the ring, faded into the night.
const FAR: Ramp = Ramp::new(0x22143a, 0x2a1846, 0x331d51, 0x3e245d, 0x4a2b69);
/// The ring a little nearer.
const BACK: Ramp = Ramp::new(0x140b20, 0x1a0f29, 0x221332, 0x2b183d, 0x361e49);
/// The nearest of the ring, darkest against the glow.
const MID: Ramp = Ramp::new(0x0d0716, 0x130a1e, 0x190e27, 0x221331, 0x2c193d);
/// Bark and leaves of the trees right at the edges of the picture.
const NEAR: Ramp = Ramp::new(0x0e0716, 0x190c25, 0x251333, 0x331b45, 0x442558);
/// Light from the glow caught along an edge: hot pink close to it, violet further off.
const RIM_NEAR: u32 = 0xff9ee8;
const RIM_FAR: u32 = 0x8e62d2;
/// The floor's own colour, before it gives anything back.
const FLOOR_FAR: u32 = 0x1c1130;
const FLOOR_NEAR: u32 = 0x0e0919;
/// The grid ruled across the floor.
const GRID: u32 = 0xa47ee8;
/// The ghostly icons: pale, cold, and see-through.
const GHOST: Ramp = Ramp::new(0x6a58b8, 0x8a78d6, 0xa898ec, 0xcabcf8, 0xf2ecff);
/// A folder keeps a ghost of its manila.
const MANILA: Ramp = Ramp::new(0xa88a58, 0xd4b070, 0xecd08c, 0xf8e4b0, 0xfff6dc);

#[derive(Clone, Copy)]
enum Kind {
    Round,
    Pine,
}

/// A tree of the ring: where its trunk stands, how high it reaches, the size of its crown.
#[derive(Clone, Copy)]
struct Tree {
    x: i32,
    base: i32,
    crown: i32,
    radius: i32,
    kind: Kind,
}

const fn tree(x: i32, base: i32, crown: i32, radius: i32, kind: Kind) -> Tree {
    Tree {
        x,
        base,
        crown,
        radius,
        kind,
    }
}

/// The ring a little way back: crowns crowding together, the odd pine standing up out of them,
/// and trunks showing beneath.
const BACK_RING: [Tree; 13] = [
    tree(6, 150, 106, 20, Kind::Round),
    tree(42, 150, 112, 18, Kind::Round),
    tree(68, 150, 100, 12, Kind::Pine),
    tree(98, 150, 110, 20, Kind::Round),
    tree(134, 150, 114, 17, Kind::Round),
    tree(160, 150, 102, 12, Kind::Pine),
    tree(190, 150, 108, 21, Kind::Round),
    tree(226, 150, 112, 18, Kind::Round),
    tree(252, 150, 98, 13, Kind::Pine),
    tree(284, 150, 108, 20, Kind::Round),
    tree(318, 151, 112, 18, Kind::Round),
    tree(346, 151, 96, 13, Kind::Pine),
    tree(376, 152, 106, 20, Kind::Round),
];
/// The nearer ring: bigger and darker, and coming closer round the sides, so the clearing reads
/// as a ring rather than a row. It parts behind the Sovereign so the glow shows round it.
const MID_RING: [Tree; 6] = [
    tree(-4, 172, 76, 30, Kind::Round),
    tree(42, 158, 96, 22, Kind::Round),
    tree(206, 151, 100, 19, Kind::Round),
    tree(244, 153, 86, 24, Kind::Round),
    tree(352, 162, 80, 26, Kind::Round),
    tree(394, 176, 66, 30, Kind::Round),
];

/// Everything behind the companions, filling every pixel.
pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    let far = far_ring();
    let back = ring(&BACK_RING, BACK, 0.5, 910);
    let brush = undergrowth();
    let mid = ring(&MID_RING, MID, 0.35, 930);
    lay(&mut scene, &far, 0..HORIZON, Light::far());
    lay(&mut scene, &back, 0..HORIZON, Light::back());
    lay(&mut scene, &brush, 0..HORIZON, Light::back());
    lay(&mut scene, &mid, 0..HORIZON, Light::mid());
    floor(&mut scene);
    // The feet of the nearer trees stand on the floor, in front of what it gives back.
    lay(&mut scene, &back, HORIZON..HEIGHT, Light::back());
    lay(&mut scene, &brush, HORIZON..HEIGHT, Light::back());
    lay(&mut scene, &mid, HORIZON..HEIGHT, Light::mid());
    icons(&mut scene);
    lay(&mut scene, &near_trees(), 0..HEIGHT, Light::near());
    motes(&mut scene, 0..36, 950);
    scene
}

/// What is nearer the eye than anyone: dark grass in the bottom corners, catching the glow, and a
/// few motes. Nothing in front of the stage.
pub fn foreground() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    let mut grass = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for (index, x) in (-4..20)
        .step_by(3)
        .chain((244..WIDTH + 4).step_by(4))
        .enumerate()
    {
        let i = index as i32;
        let tall = 8 + pick(i, 0, 970, 10) + if x < 20 { 6 } else { 0 };
        blade(&mut grass, x, HEIGHT + 1, tall, pick(i, 1, 971, 7) - 3);
    }
    lay(&mut scene, &grass, 0..HEIGHT, Light::mid());
    let (left, top, right, bottom) = GROUND;
    for index in 0..10 {
        let x = pick(index, 0, 975, WIDTH);
        let y = 40 + pick(index, 1, 976, 150);
        let on_stage = x as f32 >= left - 16.0
            && x as f32 <= right + 16.0
            && y as f32 >= top - 40.0
            && y as f32 <= bottom + 4.0;
        if !on_stage {
            mote(&mut scene, x, y, index % 3 == 0);
        }
    }
    scene
}

// ---------------------------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------------------------

/// The ordered-dither threshold for a pixel, in 0..1: gradients are laid down in bands of the
/// palette, crossing from one to the next in an even pattern, as pixel skies are.
fn dither(x: i32, y: i32) -> f32 {
    const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (f32::from(BAYER[y.rem_euclid(4) as usize][x.rem_euclid(4) as usize]) + 0.5) / 16.0
}

/// How strongly the glow reaches a point, from 1 at its heart to 0.
fn glow_at(x: f32, y: f32) -> f32 {
    let (dx, dy) = ((x - GLOW.0) / 175.0, (y - GLOW.1) / 120.0);
    (1.0 - (dx * dx + dy * dy).sqrt()).max(0.0).powf(1.6)
}

/// The light the glow casts on an edge at this point: its colour, and how strong it is.
fn rim_light(x: f32, y: f32) -> (Rgba, f32) {
    let distance = ((x - GLOW.0).powi(2) + (y - GLOW.1).powi(2)).sqrt();
    let reach = (1.0 - distance / 320.0).max(0.0);
    (
        mix(rgb(RIM_FAR), rgb(RIM_NEAR), reach * reach),
        0.3 + 0.7 * reach * reach,
    )
}

/// How a layer of trees takes the glow.
#[derive(Clone, Copy)]
struct Light {
    /// How strongly its edges catch it.
    rim: f32,
    /// How many pixels in from the edge the light reaches.
    width: i32,
    /// How far it is lost in the glowing air between it and the eye.
    haze: f32,
}

impl Light {
    const fn far() -> Self {
        Self {
            rim: 0.5,
            width: 1,
            haze: 0.55,
        }
    }

    const fn back() -> Self {
        Self {
            rim: 0.8,
            width: 2,
            haze: 0.22,
        }
    }

    const fn mid() -> Self {
        Self {
            rim: 1.0,
            width: 2,
            haze: 0.0,
        }
    }

    const fn near() -> Self {
        Self {
            rim: 1.3,
            width: 4,
            haze: 0.0,
        }
    }
}

/// Lays a painted layer over the scene, in the rows given, catching the glow along each edge that
/// faces it, brightest on the outermost pixel and fading inwards, and lost in the glowing air as
/// far as it stands back.
fn lay(scene: &mut Canvas, layer: &Canvas, rows: Range<i32>, light: Light) {
    let open = |x: i32, y: i32| {
        (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) && layer.get(x, y).a == 0
    };
    for y in rows {
        for x in 0..WIDTH {
            let mut pixel = layer.get(x, y);
            if pixel.a == 0 {
                continue;
            }
            if light.haze > 0.0 {
                let air = glow_at(x as f32, y as f32);
                pixel = mix(pixel, rgb(0x7a2c80), light.haze * air);
            }
            let (dx, dy) = (GLOW.0 - x as f32, GLOW.1 - y as f32);
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            let (ux, uy) = (dx / distance, dy / distance);
            let edge = (1..=light.width).find(|&step| {
                open(
                    x + (ux * step as f32).round() as i32,
                    y + (uy * step as f32).round() as i32,
                )
            });
            if let Some(step) = edge {
                let fall = 1.0 - (step - 1) as f32 / light.width as f32;
                let (rim, power) = rim_light(x as f32, y as f32);
                pixel = mix(pixel, rim, fall * fall * power * light.rim);
            }
            put(scene, x, y, pixel);
        }
    }
}

/// One glowing mote: a bright pixel with a soft cross of light round it.
fn mote(scene: &mut Canvas, x: i32, y: i32, warm: bool) {
    let color = if warm { 0xffc4f0 } else { 0xdcc8ff };
    put(scene, x, y, rgba(color, 220));
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        put(scene, x + dx, y + dy, rgba(color, 50));
    }
}

// ---------------------------------------------------------------------------------------------
// The sky
// ---------------------------------------------------------------------------------------------

/// The night, banded and dithered, with its stars, lines of force running out from the glow, and
/// a ring of light round where the Sovereign hangs.
fn sky(scene: &mut Canvas) {
    let top = (NIGHT.len() - 1) as f32;
    for y in 0..HEIGHT {
        let height = (y as f32 / HORIZON as f32).min(1.0);
        for x in 0..WIDTH {
            let level = 0.05 + 0.48 * height.powf(2.0) + 0.6 * glow_at(x as f32, y as f32);
            let index = (level.min(1.0) * top + dither(x, y)).floor().min(top) as usize;
            scene.set(x, y, rgb(NIGHT[index]));
        }
    }
    for index in 0..110 {
        let (x, y) = (
            pick(index, 0, 901, WIDTH),
            pick(index, 1, 902, HORIZON - 26),
        );
        if glow_at(x as f32, y as f32) > 0.12 {
            continue;
        }
        let alpha = 60 + pick(index, 2, 903, 160) as u8;
        put(scene, x, y, rgba(0xf0e8ff, alpha));
        if index % 17 == 0 {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                put(scene, x + dx, y + dy, rgba(0xe0d0ff, alpha / 3));
            }
        }
    }
    for ray in 0..44 {
        if ray % 4 == 1 {
            continue;
        }
        let angle = (ray as f32 + pick(ray, 0, 905, 60) as f32 / 100.0) / 44.0 * TAU;
        let (inner, outer) = (48.0, 120.0 + pick(ray, 1, 906, 110) as f32);
        let point = |reach: f32| {
            (
                (GLOW.0 + angle.cos() * reach) as i32,
                (GLOW.1 + angle.sin() * reach * 0.72) as i32,
            )
        };
        let alpha = 8 + pick(ray, 2, 907, 12) as u8;
        line(scene, point(inner), point(outer), rgba(0xc89aff, alpha));
    }
    for (rx, ry, alpha) in [(70, 58, 14), (50, 42, 16), (32, 28, 20)] {
        ellipse(
            scene,
            GLOW.0 as i32,
            GLOW.1 as i32,
            rx,
            ry,
            rgba(0xe878d8, alpha),
        );
    }
    // A thin, broken halo, as a boss's aura has.
    let steps = 360;
    for step in 0..steps {
        if chance(step / 6, 0, 908, 70) {
            continue;
        }
        let angle = step as f32 / steps as f32 * TAU;
        let (x, y) = (GLOW.0 + angle.cos() * 84.0, GLOW.1 + angle.sin() * 70.0);
        put(scene, x as i32, y as i32, rgba(0xf4b0f0, 46));
    }
}

// ---------------------------------------------------------------------------------------------
// The ring of trees
// ---------------------------------------------------------------------------------------------

/// The far side of the ring: a wall of crowns along the horizon, with pines standing up out of
/// it.
fn far_ring() -> Canvas {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    layer.fill_rect(0, 126, WIDTH, HORIZON - 126 + 2, FAR.shadow);
    for (index, x) in (-12..WIDTH + 12).step_by(10).enumerate() {
        let i = index as i32;
        let x = x + pick(i, 0, 920, 6);
        if index % 4 == 2 {
            pine(
                &mut layer,
                x,
                98 + pick(i, 1, 921, 12),
                HORIZON,
                10,
                FAR,
                922 + i as u32,
            );
        } else {
            let radius = 10 + pick(i, 2, 923, 6);
            crown(
                &mut layer,
                (x, 120 + pick(i, 3, 924, 8)),
                radius,
                FAR,
                925 + i as u32,
            );
        }
    }
    layer
}

/// A rank of the ring, trunks first and crowns over them.
fn ring(trees: &[Tree], ramp: Ramp, sky_light: f32, salt: u32) -> Canvas {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for (index, tree) in trees.iter().enumerate() {
        let salt = salt + index as u32 * 7;
        let width = 3 + tree.radius / 6;
        trunk(&mut layer, tree.x, tree.crown, tree.base, width, ramp, salt);
        match tree.kind {
            Kind::Round => crown(&mut layer, (tree.x, tree.crown), tree.radius, ramp, salt),
            Kind::Pine => pine(
                &mut layer,
                tree.x,
                tree.crown - tree.radius * 2,
                tree.base - 6,
                tree.radius,
                ramp,
                salt,
            ),
        }
    }
    // The faintest light from the sky on the tops of the crowns, from the upper left.
    if sky_light > 0.0 {
        let lit: Vec<(i32, i32)> = (0..HEIGHT)
            .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                layer.get(x, y).a > 0
                    && layer.get(x - 1, y - 1).a == 0
                    && layer.get(x, y - 1).a == 0
                    && y < HORIZON
            })
            .collect();
        for (x, y) in lit {
            let pixel = layer.get(x, y);
            layer.set(x, y, mix(pixel, ramp.shine, sky_light));
        }
    }
    layer
}

/// Low bushes crowding the foot of the ring, so the trees stand in a wood rather than a row.
fn undergrowth() -> Canvas {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    layer.fill_rect(0, 146, WIDTH, HORIZON - 146 + 1, BACK.edge);
    for (index, x) in (-8..WIDTH + 8).step_by(9).enumerate() {
        let i = index as i32;
        let radius = 5 + pick(i, 0, 940, 5);
        let y = 143 + pick(i, 1, 941, 5);
        crown(
            &mut layer,
            (x + pick(i, 2, 942, 5), y),
            radius,
            BACK,
            943 + i as u32,
        );
    }
    layer
}

/// The trees right at the edges of the picture, framing it: a trunk rising out of the top on
/// either side, and their leaves hanging into the corners.
fn near_trees() -> Canvas {
    let mut layer = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    trunk(&mut layer, 5, -4, HEIGHT + 2, 24, NEAR, 960);
    trunk(&mut layer, 380, -4, HEIGHT + 2, 26, NEAR, 961);
    for (at, radius, salt) in [
        ((4, 6), 28, 962),
        ((38, -6), 20, 963),
        ((66, -14), 13, 964),
        ((382, 10), 30, 965),
        ((346, -6), 20, 966),
        ((320, -16), 13, 967),
    ] {
        crown(&mut layer, at, radius, NEAR, salt);
    }
    layer
}

/// A trunk from `top` down to `base`, flaring into roots at the foot, its bark furrowed in broken
/// streaks and outlined in its own darkest tone.
fn trunk(layer: &mut Canvas, x: i32, top: i32, base: i32, width: i32, ramp: Ramp, salt: u32) {
    let tall = (base - top) as f32;
    for y in top.max(0)..=base {
        let flare = ((y - (base - 5)).max(0) * width) / 5;
        // A tall trunk is never quite straight.
        let bend = ((y - top) as f32 / tall * 2.6 + salt as f32).sin() * tall / 60.0;
        let left = x - width / 2 - flare / 2 + bend as i32;
        // Broad bark is knobbly at its edges.
        let (knob_left, knob_right) = if width >= 12 {
            (
                pick(y.div_euclid(4), 2, salt, 3) - 1,
                pick(y.div_euclid(3), 3, salt, 3) - 1,
            )
        } else {
            (0, 0)
        };
        let (from, to) = (left + knob_left, left + width + flare + knob_right);
        let span = (to - from).max(1);
        for px in from..to {
            let across = (px - from) as f32 / span as f32;
            let mut color = if px == from || px == to - 1 {
                ramp.edge
            } else if across < 0.3 {
                ramp.base
            } else if across < 0.7 {
                ramp.shadow
            } else {
                ramp.edge
            };
            if width >= 6 && px > from && px < to - 1 {
                // Furrows run up the bark in broken streaks, with ridges between them.
                let streak = noise(
                    px - from,
                    (y + pick(px - from, 1, salt, 9)).div_euclid(5),
                    salt,
                );
                if streak.is_multiple_of(5) {
                    color = ramp.edge;
                } else if streak % 7 == 1 && across < 0.6 {
                    color = ramp.light;
                } else if streak % 11 == 2 && across < 0.3 {
                    color = ramp.shine;
                }
            }
            layer.set(px, y, color);
        }
    }
}

/// A round crown of leaves: a main mass with smaller lobes about it, so no two are the same shape.
/// Each lobe is shaded as its own ball, lighter up and to the left, the lower ones standing in
/// front of the higher, with the tones broken into leafy clusters where they meet.
fn crown(layer: &mut Canvas, (cx, cy): (i32, i32), radius: i32, ramp: Ramp, salt: u32) {
    let count = 5 + pick(radius, 0, salt, 4);
    let mut lobes = vec![(cx as f32, cy as f32, radius as f32 * 0.85)];
    for lobe in 0..count {
        let angle = lobe as f32 / count as f32 * TAU + pick(lobe, 1, salt, 100) as f32 / 60.0;
        let reach = radius as f32 * (0.5 + pick(lobe, 3, salt, 30) as f32 / 100.0);
        let size = radius as f32 * (0.35 + pick(lobe, 2, salt, 25) as f32 / 100.0);
        lobes.push((
            cx as f32 + angle.cos() * reach * 1.15,
            cy as f32 + angle.sin() * reach * 0.7,
            size,
        ));
    }
    lobes.sort_by(|a, b| a.1.total_cmp(&b.1));
    let lobe_at = |x: i32, y: i32| {
        let wobble = (noise(x.div_euclid(2), y.div_euclid(2), salt) % 3) as f32 - 1.0;
        lobes.iter().rposition(|&(lx, ly, size)| {
            let (dx, dy) = (x as f32 - lx, (y as f32 - ly) / 0.82);
            (dx * dx + dy * dy).sqrt() < size + wobble * 0.8
        })
    };
    let reach = radius * 2;
    for y in cy - reach..=cy + reach {
        for x in cx - reach..=cx + reach {
            let Some(index) = lobe_at(x, y) else {
                continue;
            };
            let outside = |dx: i32, dy: i32| lobe_at(x + dx, y + dy).is_none();
            if outside(-1, 0) || outside(1, 0) || outside(0, -1) || outside(0, 1) {
                layer.set(x, y, ramp.edge);
                continue;
            }
            let (lx, ly, size) = lobes[index];
            let light = ((x as f32 - lx) + (y as f32 - ly) * 1.2) / (size * 1.5);
            let leafy = (noise(x.div_euclid(2), y, salt + 3) % 100) as f32 / 100.0 - 0.5;
            let tone = light + leafy * 0.35;
            // The top of a lobe standing in front of another catches a little light.
            let front = lobe_at(x, y - 1).is_some_and(|above| above < index);
            let color = if tone < -0.6 && leafy < -0.3 {
                ramp.shine
            } else if tone < -0.45 || front {
                ramp.light
            } else if tone < -0.05 {
                ramp.base
            } else if tone < 0.5 {
                ramp.shadow
            } else {
                ramp.edge
            };
            layer.set(x, y, color);
        }
    }
}

/// A pine: tiers of boughs from `top` to `bottom`, each wider than the one above and standing in
/// front of it, their undersides ragged with needles and their tips drooping.
fn pine(layer: &mut Canvas, x: i32, top: i32, bottom: i32, radius: i32, ramp: Ramp, salt: u32) {
    const TIERS: i32 = 4;
    let height = (bottom - top).max(TIERS);
    for tier in 0..TIERS {
        let tier_top = top + tier * height / TIERS * 4 / 5;
        let tier_bottom = top + (tier + 1) * height / TIERS;
        let span = (tier_bottom - tier_top).max(1);
        let width = radius as f32 * (0.4 + 0.6 * (tier + 1) as f32 / TIERS as f32);
        for y in tier_top..=tier_bottom {
            let down = (y - tier_top) as f32 / span as f32;
            let reach = (width * down.powf(0.85)) as i32 + pick(y, tier, salt, 2);
            for px in x - reach..=x + reach {
                let across = (px - x) as f32 / reach.max(1) as f32;
                // Needles hang ragged from the underside, longest at the tips.
                if y == tier_bottom && across.abs() < 0.8 && (px + y) % 2 == 0 {
                    continue;
                }
                let color = if px == x - reach || px == x + reach || y == tier_bottom {
                    ramp.edge
                } else if across < -0.35 && !noise(px, y, salt).is_multiple_of(3) {
                    if across < -0.75 {
                        ramp.light
                    } else {
                        ramp.base
                    }
                } else if across > 0.45 {
                    ramp.edge
                } else {
                    ramp.shadow
                };
                layer.set(px, y, color);
            }
        }
        // The tips of the boughs droop a pixel or two.
        let tip = (width as i32).max(1);
        for (side, droop) in [
            (-1, 1 + pick(tier, 3, salt, 2)),
            (1, 1 + pick(tier, 4, salt, 2)),
        ] {
            for step in 1..=droop {
                layer.set(x + side * tip, tier_bottom + step, ramp.edge);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The floor
// ---------------------------------------------------------------------------------------------

/// Where on the floor a pixel lies, in squares of the grid: across from the middle, and how far
/// back. The grid runs away to a point on the horizon.
fn on_floor(x: f32, y: f32) -> (f32, f32) {
    let below = (y - HORIZON as f32).max(0.3);
    ((x - 192.0) / (0.74 * below), 82.0 / below - 0.5)
}

/// The floor: dark and glossy, giving back the trees and the glow, ruled into a grid of great
/// squares like a desktop seen far too close.
fn floor(scene: &mut Canvas) {
    let span = (HEIGHT - HORIZON) as f32;
    for y in HORIZON..HEIGHT {
        let near = (y - HORIZON) as f32 / span;
        let base = mix(rgb(FLOOR_FAR), rgb(FLOOR_NEAR), near.powf(0.5));
        // A gloss gives back more at a glancing angle, so most just under the trees.
        let gloss = 0.08 + 0.5 * (1.0 - near).powf(3.0);
        let mirrored = 2 * HORIZON - 1 - y;
        for x in 0..WIDTH {
            let mut color = mix(base, scene.get(x, mirrored), gloss);
            // The glow's own reflection, a long column running down from under it.
            let wide = 14.0 + near * 34.0;
            let off = (x as f32 - GLOW.0) / wide;
            let column = (-off * off).exp() * (1.0 - near).powf(0.7) * 0.42;
            if column > 0.01 {
                let band = (column * 6.0 + dither(x, y)).floor() / 6.0;
                color = mix(color, rgb(0xd868c8), band);
            }
            let (across, back) = on_floor(x as f32 + 0.5, y as f32 + 0.5);
            if (across.floor() as i32 + back.floor() as i32) % 2 == 0 {
                color = mix(color, rgb(0x3a2458), 0.12);
            }
            scene.set(x, y, color);
        }
    }
    // The grid: a pixel is on a line where the square changes between it and the next.
    for y in HORIZON + 1..HEIGHT {
        let near = (y - HORIZON) as f32 / span;
        let fade = ((y - HORIZON) as f32 / 8.0).min(1.0);
        for x in 0..WIDTH {
            let (across, back) = on_floor(x as f32, y as f32);
            let (right, _) = on_floor(x as f32 + 1.0, y as f32);
            let (_, below) = on_floor(x as f32, y as f32 + 1.0);
            let ruled = back.floor() != below.floor() && back < 14.0;
            let column = across.floor() != right.floor();
            if !ruled && !column {
                continue;
            }
            let lit = glow_at(x as f32, (2 * HORIZON - y) as f32);
            let alpha = (18.0 + 46.0 * near + 120.0 * lit) * fade;
            let color = mix(rgb(GRID), rgb(0xffb8f0), (lit * 1.6).min(1.0));
            put(
                scene,
                x,
                y,
                Rgba {
                    a: alpha as u8,
                    ..color
                },
            );
        }
    }
    // Where floor meets trees, a hard line of sheen.
    for x in 0..WIDTH {
        let lit = glow_at(x as f32, HORIZON as f32 - 20.0);
        put(
            scene,
            x,
            HORIZON,
            rgba(0xe8a8f8, (50.0 + 140.0 * lit) as u8),
        );
    }
    // The odd glint on the gloss, kept off the stage.
    for y in HORIZON + 2..HEIGHT {
        for x in 0..WIDTH {
            let on_stage = (20..240).contains(&x) && y >= 156;
            if chance(x, y, 909, if on_stage { 1 } else { 3 }) {
                put(scene, x, y, rgba(0xe0d0ff, 70));
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The ghosts of icons
// ---------------------------------------------------------------------------------------------

/// Giant desktop icons standing half sunk in the floor like ghosts: a document out among the far
/// trees, a recycle bin further off, and a folder close by on the right, clear of the stage.
fn icons(scene: &mut Canvas) {
    ghost(scene, &document(), (40, 158), 152, 0.1, 0.42);
    ghost(scene, &bin(), (226, 154), 151, -0.05, 0.38);
    ghost(scene, &folder(), (330, 214), 201, -0.1, 0.34);
}

/// Draws an icon standing in the floor, leaning by `lean` radians about the middle of its foot,
/// sunk below the floor's surface at `surface`: see-through, its edges glowing, and what shows
/// above the surface given back faintly below it.
fn ghost(
    scene: &mut Canvas,
    icon: &Canvas,
    foot: (i32, i32),
    surface: i32,
    lean: f32,
    opacity: f32,
) {
    let (width, height) = (icon.width() as f32, icon.height() as f32);
    let (sin, cos) = lean.sin_cos();
    let reach = (width.max(height) * 1.2) as i32;
    // What shows above the surface, stood up and leaning, on its own layer.
    let mut standing = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in foot.1 - reach..=surface {
        for x in foot.0 - reach..foot.0 + reach {
            let (dx, dy) = ((x - foot.0) as f32 + 0.5, (y - foot.1) as f32 + 0.5);
            let (lx, ly) = (dx * cos + dy * sin, -dx * sin + dy * cos);
            let pixel = icon.get(
                (lx + width / 2.0).floor() as i32,
                (ly + height).floor() as i32,
            );
            standing.set(x, y, pixel);
        }
    }
    let shown = |x: i32, y: i32| standing.get(x, y).a > 0;
    let mut across = (i32::MAX, i32::MIN);
    for y in foot.1 - reach..=surface {
        for x in foot.0 - reach..foot.0 + reach {
            let pixel = standing.get(x, y);
            if pixel.a == 0 {
                // A faint halo just outside it.
                let near = shown(x - 1, y) || shown(x + 1, y) || shown(x, y - 1) || shown(x, y + 1);
                if near {
                    put(scene, x, y, rgba(0xe8dcff, (40.0 * opacity) as u8));
                }
                continue;
            }
            let edge = !shown(x - 1, y) || !shown(x + 1, y) || !shown(x, y - 1);
            let (color, alpha) = if edge {
                (mix(pixel, rgb(0xffffff), 0.35), (opacity * 1.8).min(0.95))
            } else {
                (pixel, opacity)
            };
            put(
                scene,
                x,
                y,
                Rgba {
                    a: (alpha * 255.0) as u8,
                    ..color
                },
            );
            if y == surface {
                across = (across.0.min(x), across.1.max(x));
            }
        }
    }
    // Under the surface, a reflection of what stands above it, broken and fading with depth.
    for depth in 1..14 {
        let fade = (1.0 - depth as f32 / 14.0) * 0.4 * opacity;
        if depth % 3 == 2 {
            continue;
        }
        for x in foot.0 - reach..foot.0 + reach {
            let pixel = standing.get(x, surface - depth);
            if pixel.a > 0 {
                put(
                    scene,
                    x,
                    surface + depth,
                    Rgba {
                        a: (fade * 255.0) as u8,
                        ..pixel
                    },
                );
            }
        }
    }
    // A bright ripple where it goes in.
    let (from, to) = across;
    if from <= to {
        for x in from - 3..=to + 3 {
            let alpha = if x < from || x > to { 50 } else { 140 };
            put(scene, x, surface, rgba(0xf4e8ff, alpha));
        }
        for x in (from - 6..=to + 6).filter(|x| x % 2 == 0) {
            put(scene, x, surface + 2, rgba(0xc8b0ff, 40));
        }
    }
}

/// A folder, as every desktop draws one: the back with its tab, the front a little lower.
fn folder() -> Canvas {
    let mut icon = Canvas::new(60, 46);
    let back = [(0, 1), (21, 1), (27, 8), (59, 8), (59, 45), (0, 45)];
    fill(&mut icon, &back, MANILA.shadow, MANILA.edge);
    for x in 1..21 {
        icon.set(x, 2, MANILA.base);
    }
    let front = [(0, 14), (59, 14), (59, 45), (0, 45)];
    fill(&mut icon, &front, MANILA.light, MANILA.edge);
    for x in 1..59 {
        icon.set(x, 15, MANILA.shine);
    }
    for y in 16..44 {
        icon.set(1, y, MANILA.shine);
        icon.set(58, y, MANILA.base);
    }
    for x in 2..58 {
        icon.set(x, 43, MANILA.base);
    }
    icon
}

/// A page of a document with its corner turned down and lines of writing on it.
fn document() -> Canvas {
    let mut icon = Canvas::new(26, 32);
    let page = [(0, 0), (18, 0), (25, 7), (25, 31), (0, 31)];
    fill(&mut icon, &page, GHOST.light, GHOST.edge);
    for y in 1..31 {
        icon.set(1, y, GHOST.shine);
    }
    let fold = [(18, 0), (18, 7), (25, 7)];
    fill(&mut icon, &fold, GHOST.shadow, GHOST.edge);
    for (row, length) in [(10, 16), (14, 20), (18, 18), (22, 20), (26, 12)] {
        for x in 4..4 + length {
            icon.set(x, row, GHOST.shadow);
        }
    }
    icon
}

/// A recycle bin: a lid, and a body ribbed from top to bottom, narrowing to its foot.
fn bin() -> Canvas {
    let mut icon = Canvas::new(16, 19);
    fill(
        &mut icon,
        &[(0, 1), (15, 1), (15, 4), (0, 4)],
        GHOST.light,
        GHOST.edge,
    );
    fill(
        &mut icon,
        &[(5, 0), (10, 0), (10, 1), (5, 1)],
        GHOST.light,
        GHOST.edge,
    );
    let body = [(1, 4), (14, 4), (12, 18), (3, 18)];
    fill(&mut icon, &body, GHOST.base, GHOST.edge);
    for x in [5, 8, 11] {
        for y in 6..17 {
            icon.set(x - (y - 6) / 12, y, GHOST.shadow);
        }
    }
    icon
}

/// Fills a shape on an icon and outlines it.
fn fill(icon: &mut Canvas, points: &[(i32, i32)], inside: Rgba, edge: Rgba) {
    let mut shape = Canvas::new(icon.width(), icon.height());
    polygon(&mut shape, points, |_, _| Some(inside));
    for (index, &from) in points.iter().enumerate() {
        line(&mut shape, from, points[(index + 1) % points.len()], edge);
    }
    blit(icon, &shape, 0, 0);
}

// ---------------------------------------------------------------------------------------------
// The air
// ---------------------------------------------------------------------------------------------

/// Motes hanging in the air, gathered towards the glow.
fn motes(scene: &mut Canvas, range: Range<i32>, salt: u32) {
    for index in range {
        let (x, y) = (
            pick(index, 0, salt, WIDTH),
            30 + pick(index, 1, salt + 1, 116),
        );
        if glow_at(x as f32, y as f32) < 0.03 && index % 3 != 0 {
            continue;
        }
        mote(scene, x, y, index % 3 == 0);
    }
}

/// A blade of grass rising from `(x, base)`, `tall` high, bending `bend` pixels over at the tip.
fn blade(layer: &mut Canvas, x: i32, base: i32, tall: i32, bend: i32) {
    for step in 0..tall {
        let along = step as f32 / tall as f32;
        let px = x + (bend as f32 * along * along) as i32;
        let y = base - step;
        let color = if along > 0.7 { NEAR.light } else { NEAR.shadow };
        layer.set(px, y, color);
        if along < 0.5 {
            layer.set(px + 1, y, NEAR.edge);
        }
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
    fn nothing_in_front_hides_the_stage() {
        let front = foreground();
        let (left, top, right, bottom) = GROUND;
        // Everyone standing on the stage, and as tall as anyone is.
        for y in top as i32 - 32..=bottom as i32 {
            for x in left as i32..=right as i32 {
                assert_eq!(front.get(x, y).a, 0, "something in front at {x}, {y}");
            }
        }
    }
}
