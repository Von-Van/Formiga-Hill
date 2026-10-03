//! The brushwork every find is drawn with: shading rounded and flat shapes from the upper left,
//! rows of letters stamped in colours, stems and boughs, tufts, flowers and soft shadows.

use super::ICON;
use crate::materials::BLOSSOMS;
use crate::paint::{Ramp, chance, mix, noise, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};

/// The moss on stones and the green of stems and tufts, as the brushwork draws them.
const BRUSH_MOSS: Ramp = Ramp::new(0x34512a, 0x4a6e34, 0x638c40, 0x80aa52, 0xa4c872);
const BRUSH_STEM: Ramp = Ramp::new(0x2f5a2e, 0x3f7a3a, 0x58964a, 0x76b25c, 0x9ccc7e);

/// How a rounded thing turns to the light from the upper left: how much its tone follows where
/// a pixel sits across it, and how much where it sits down it.
pub(super) type Light = (f32, f32);

pub(super) const ROUND: Light = (0.55, 0.45);

pub(super) const UPRIGHT: Light = (0.85, 0.2);

pub(super) const LYING: Light = (0.15, 0.85);

/// Shades the solid that `inside` describes as something rounded, lit from the upper left: how
/// far across and down it a pixel sits stands in for the way the surface there faces. Noise
/// dithers the bands so they don't step, `grain` in every 256 pixels are a shade darker (and
/// half as many a shade lighter) for texture, and the outermost pixels take the edge tone.
pub(super) fn model(
    s: &mut Canvas,
    ramp: Ramp,
    light: Light,
    grain: u32,
    salt: u32,
    inside: impl Fn(i32, i32) -> bool,
) {
    let tones = [ramp.edge, ramp.shadow, ramp.base, ramp.light, ramp.shine];
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !inside(x, y) {
                continue;
            }
            let reach = |dx: i32, dy: i32| {
                let mut n = 0;
                while n < 64 && inside(x + dx * (n + 1), y + dy * (n + 1)) {
                    n += 1;
                }
                n
            };
            let (left, right, up, down) = (reach(-1, 0), reach(1, 0), reach(0, -1), reach(0, 1));
            if left.min(right).min(up).min(down) == 0 {
                put(s, x, y, ramp.edge);
                continue;
            }
            let across = (left - right) as f32 / (left + right) as f32;
            let along = (up - down) as f32 / (up + down) as f32;
            let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
            let lit = -light.0 * across - light.1 * along + 0.3 * jitter;
            let mut level: usize = if lit > 0.75 {
                4
            } else if lit > 0.3 {
                3
            } else if lit > -0.3 {
                2
            } else {
                1
            };
            if chance(x, y, salt.wrapping_add(1), grain) {
                level = (level - 1).max(1);
            } else if chance(x, y, salt.wrapping_add(2), grain / 2) {
                level = (level + 1).min(3);
            }
            put(s, x, y, tones[level]);
        }
    }
}

/// A twig or branch along `path`, two pixels thick: lit along its upper side, in its own edge
/// tone along its lower.
pub(super) fn bough(s: &mut Canvas, path: &[(i32, i32)], ramp: Ramp) {
    let points = trace(path);
    for &(x, y) in &points {
        put(s, x + 1, y, ramp.edge);
        put(s, x, y + 1, ramp.edge);
        put(s, x + 1, y + 1, ramp.edge);
    }
    for &(x, y) in &points {
        let lit = if chance(x, y, 121, 90) {
            ramp.light
        } else {
            ramp.base
        };
        put(s, x, y, lit);
    }
}

/// Every pixel along a path of straight steps.
pub(super) fn trace(path: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut points: Vec<(i32, i32)> = Vec::new();
    for pair in path.windows(2) {
        let ((mut x, mut y), (x1, y1)) = (pair[0], pair[1]);
        let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
        let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
        let mut error = dx + dy;
        loop {
            if points.last() != Some(&(x, y)) {
                points.push((x, y));
            }
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
    }
    points
}

/// Grass grown up round a piece: a few blades, lit at the tips.
pub(super) const TUFT: [&str; 3] = ["l.l.l", ".glg.", ".dgd."];

pub(super) fn tuft_inks() -> Vec<(u8, Rgba)> {
    vec![
        (b'l', rgb(0xa6d48c)),
        (b'g', rgb(0x5e9050)),
        (b'd', rgb(0x467440)),
    ]
}

/// A ramp's tones named by five letters, darkest first, for `stamp`. A `.` names nothing.
pub(super) fn letters(ramp: Ramp, names: &[u8; 5]) -> [(u8, Rgba); 5] {
    [
        (names[0], ramp.edge),
        (names[1], ramp.shadow),
        (names[2], ramp.base),
        (names[3], ramp.light),
        (names[4], ramp.shine),
    ]
}

/// Whether `(x, y)` is inside the ellipse, measured a little generously so that small ones come
/// out round rather than with a lone pixel sticking out at each end.
pub(super) fn in_ellipse(x: i32, y: i32, centre: (i32, i32), size: (i32, i32)) -> bool {
    let (rx, ry) = (size.0 as f32 + 0.4, size.1 as f32 + 0.4);
    let (dx, dy) = ((x - centre.0) as f32 / rx, (y - centre.1) as f32 / ry);
    size.0 > 0 && size.1 > 0 && dx * dx + dy * dy <= 1.0
}

/// The soft shade every piece casts on the summit grass.
pub(super) const SHADOW: Rgba = rgba(0x2a4a2a, 70);

/// Paints rows of letters with their top-left at `at`, each letter a colour from `inks`; a
/// letter not in `inks` is left clear. `flip` mirrors it left to right.
pub(super) fn stamp(
    s: &mut Canvas,
    at: (i32, i32),
    rows: &[&str],
    inks: &[(u8, Rgba)],
    flip: bool,
) {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as i32;
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            if letter == b'.' {
                continue;
            }
            let Some(&(_, color)) = inks.iter().find(|(name, _)| *name == letter) else {
                continue;
            };
            let dx = if flip {
                width - 1 - dx as i32
            } else {
                dx as i32
            };
            put(s, at.0 + dx, at.1 + dy as i32, color);
        }
    }
}

/// An icon painted from nine rows of codes, as `paint_rows` reads them.
pub(super) fn icon_from(rows: [&str; 9], ramp: Ramp, extras: &[(char, Rgba)]) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    for row in rows {
        debug_assert_eq!(row.chars().count(), ICON as usize, "{row:?}");
    }
    paint_rows(&mut canvas, 0, 0, &rows, ramp, extras);
    canvas
}

/// Paints rows of codes with their top-left at `(x, y)`: `#`, `s`, `o`, `l` and `*` for the main
/// material's edge, shadow, base, light and shine, `extras` for anything else, `.` for clear.
pub(super) fn paint_rows(
    s: &mut Canvas,
    x: i32,
    y: i32,
    rows: &[&str],
    ramp: Ramp,
    extras: &[(char, Rgba)],
) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, code) in row.chars().enumerate() {
            let color = match code {
                '#' => ramp.edge,
                's' => ramp.shadow,
                'o' => ramp.base,
                'l' => ramp.light,
                '*' => ramp.shine,
                other => match extras.iter().find(|(c, _)| *c == other) {
                    Some(&(_, color)) => color,
                    None => continue,
                },
            };
            put(s, x + dx as i32, y + dy as i32, color);
        }
    }
}

/// Freshly turned earth, where something has been planted.
pub(super) const TILTH: Ramp = Ramp::new(0x3e2a1c, 0x5a3c26, 0x765232, 0x926c46, 0xb08c62);

/// A mound of freshly turned earth where something has just been planted: `rx` pixels either side
/// of `cx` and `ry` high above the ground row, lit from the upper left, with crumbs of soil
/// catching the light and its soft shadow on the grass. A low one (`ry` of 1) is the bare earth
/// still showing round the foot of something that has grown a while.
pub(super) fn mound(s: &mut Canvas, cx: i32, ground: i32, (rx, ry): (i32, i32), salt: u32) {
    shadow(s, cx + 1, ground, rx + 1, 2);
    let inside = |x: i32, y: i32| {
        let dx = (x - cx) as f32 / (rx as f32 + 0.4);
        let dy = (ground - y) as f32 / (ry as f32 + 0.6);
        y <= ground && dx * dx + dy * dy <= 1.0
    };
    model(s, TILTH, ROUND, 50, salt, inside);
    for y in ground - ry..ground {
        for x in cx - rx..=cx + rx {
            let within = inside(x, y) && inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1);
            if !within {
                continue;
            }
            if chance(x, y, salt + 3, 30) && x <= cx {
                put(s, x, y, TILTH.shine);
            } else if chance(x, y, salt + 4, 34) {
                put(s, x, y, TILTH.edge);
            }
        }
    }
}

/// A pointed shoot or upright leaf from `base` up `height` pixels, its tip `lean` pixels over:
/// two pixels wide and lit on its left, its own darkest shade down its right, narrowing to a
/// single pixel at the tip.
pub(super) fn spike(s: &mut Canvas, base: (i32, i32), height: i32, lean: i32, ramp: Ramp) {
    for step in 0..height {
        let t = step as f32 / (height - 1).max(1) as f32;
        let x = base.0 + (lean as f32 * t * t).round() as i32;
        let y = base.1 - step;
        let left = height - 1 - step;
        put(s, x, y, if left == 0 { ramp.base } else { ramp.light });
        if left >= 1 {
            put(s, x + 1, y, if left >= 3 { ramp.base } else { ramp.edge });
        }
        if left >= 3 {
            put(s, x + 2, y, ramp.edge);
        }
    }
}

/// The soft shade under a piece, where it stands on the grass. Drawn first, onto clear canvas.
pub(super) fn shadow(s: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32) {
    for y in -ry..=ry {
        for x in -rx..=rx {
            if (x * x * ry * ry + y * y * rx * rx) <= rx * rx * ry * ry {
                s.set(cx + x, cy + y, SHADOW);
            }
        }
    }
}

/// A tone from `ramp` for a point `(u, v)` on a rounded surface, each from -1 to 1 across it,
/// lit from the upper left; `grain` jitters it so the surface isn't smooth.
pub(super) fn lit(ramp: Ramp, u: f32, v: f32, grain: u32) -> Rgba {
    let depth = (1.0 - u * u - v * v).max(0.0).sqrt();
    let light = -0.55 * u - 0.62 * v + 0.56 * depth;
    let light = light + ((grain % 64) as f32 / 64.0 - 0.5) * 0.18;
    if light > 0.9 {
        ramp.shine
    } else if light > 0.58 {
        ramp.light
    } else if light > 0.2 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// A rounded lump lit from the upper left and outlined in its own edge tone: a pebble, a cone, a
/// clump of leaves.
pub(super) fn lump(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, ramp: Ramp, salt: u32) {
    tilted_lump(s, (cx, cy), (rx, ry), 0.0, ramp, salt);
}

/// A lump turned `tilt` radians clockwise, still lit from the upper left: a stone set down a
/// little askew, a cone lying on the slant.
pub(super) fn tilted_lump(
    s: &mut Canvas,
    (cx, cy): (f32, f32),
    (rx, ry): (f32, f32),
    tilt: f32,
    ramp: Ramp,
    salt: u32,
) {
    let (sin, cos) = tilt.sin_cos();
    // Where a pixel falls across the lump, as it lies and as the light sees it.
    let at = |x: i32, y: i32| {
        let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
        let (u, v) = ((dx * cos + dy * sin) / rx, (dy * cos - dx * sin) / ry);
        ((u, v), (u * cos - v * sin, u * sin + v * cos))
    };
    let inside = |x: i32, y: i32| {
        let ((u, v), _) = at(x, y);
        u * u + v * v <= 1.0
    };
    let reach = rx.max(ry).ceil() as i32 + 1;
    let (mx, my) = (cx.round() as i32, cy.round() as i32);
    for y in my - reach..=my + reach {
        for x in mx - reach..=mx + reach {
            if !inside(x, y) {
                continue;
            }
            let edge =
                !(inside(x - 1, y) && inside(x + 1, y) && inside(x, y - 1) && inside(x, y + 1));
            let (_, (u, v)) = at(x, y);
            let color = if edge {
                ramp.edge
            } else {
                lit(ramp, u, v, noise(x, y, salt))
            };
            put(s, x, y, color);
        }
    }
}

/// Darkens whatever is already painted within an ellipse: where one thing rests on another.
pub(super) fn tuck(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgba) {
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            if u * u + v * v <= 1.0 && s.get(x, y).a == 255 {
                put(s, x, y, color);
            }
        }
    }
}

/// Moss in a patch over whatever is already painted there.
pub(super) fn moss(s: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, salt: u32) {
    for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
        for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
            let (u, v) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
            let d = u * u + v * v;
            let thick = (230.0 * (1.0 - d)) as u32 + 40;
            if d > 1.0 || s.get(x, y).a < 255 || !chance(x, y, salt, thick) {
                continue;
            }
            let color = if v < -0.2 && chance(x, y, salt + 1, 140) {
                BRUSH_MOSS.light
            } else if u + v > 0.6 {
                BRUSH_MOSS.shadow
            } else {
                BRUSH_MOSS.base
            };
            put(s, x, y, color);
        }
    }
}

/// A tone across a cylinder lit from the left, `across` from 0 at its left to 1 at its right.
pub(super) fn cylinder(ramp: Ramp, across: f32) -> Rgba {
    if across < 0.12 {
        ramp.light
    } else if across < 0.3 {
        ramp.shine
    } else if across < 0.45 {
        ramp.light
    } else if across < 0.78 {
        ramp.base
    } else {
        ramp.shadow
    }
}

/// Pixels along a curve from `from`, pulled towards `via`, to `to`, one step at a time.
pub(super) fn curve(from: (f32, f32), via: (f32, f32), to: (f32, f32)) -> Vec<(i32, i32)> {
    let span = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let steps = ((span(from, via) + span(via, to)) * 2.0).ceil().max(1.0) as usize;
    let mut points: Vec<(i32, i32)> = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * from.0 + 2.0 * u * t * via.0 + t * t * to.0;
        let y = u * u * from.1 + 2.0 * u * t * via.1 + t * t * to.1;
        let point = (x.round() as i32, y.round() as i32);
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

/// A strappy leaf or grass blade from `from` to `to`, two pixels wide for most of its length,
/// lit along its upper side.
pub(super) fn blade(s: &mut Canvas, from: (f32, f32), via: (f32, f32), to: (f32, f32), ramp: Ramp) {
    let points = curve(from, via, to);
    let count = points.len();
    for (index, &(x, y)) in points.iter().enumerate() {
        let t = index as f32 / count as f32;
        if t < 0.75 {
            put(s, x, y + 1, ramp.edge);
        }
        put(s, x, y, if t < 0.4 { ramp.base } else { ramp.light });
    }
    if let Some(&(x, y)) = points.last() {
        put(s, x, y, ramp.shadow);
    }
}

/// A tuft of grass blades about `(x, y)` on the ground, `spread` pixels either side.
pub(super) fn tuft(s: &mut Canvas, x: i32, y: i32, spread: i32, salt: u32) {
    for dx in -spread..=spread {
        if !chance(x + dx, y, salt, 170) {
            continue;
        }
        let tall = 1 + (noise(x + dx, y, salt + 1) % 3) as i32 + (spread - dx.abs()) / 2;
        let lean = (dx.signum() * (noise(dx, y, salt + 2) % 2) as i32).clamp(-1, 1);
        for step in 0..tall {
            let color = if step == tall - 1 {
                BRUSH_STEM.light
            } else if step == 0 {
                BRUSH_STEM.shadow
            } else {
                BRUSH_STEM.base
            };
            let px = x + dx + if step == tall - 1 { lean } else { 0 };
            put(s, px, y - step, color);
        }
    }
}

/// A few flowers on short stems around `(x, y)`.
pub(super) fn flowers(s: &mut Canvas, x: i32, y: i32, spread: i32, count: u32, salt: u32) {
    for index in 0..count {
        let fx = x - spread + (noise(index as i32, x, salt) % (spread * 2 + 1) as u32) as i32;
        let fy = y - (noise(x, index as i32, salt + 1) % 3) as i32;
        let blossom = BLOSSOMS[(noise(fx, fy, salt + 2) % 5) as usize];
        put(s, fx, fy + 1, BRUSH_STEM.shadow);
        put(s, fx, fy, blossom);
        put(s, fx - 1, fy, mix(blossom, rgb(0xffffff), 0.35));
        put(s, fx + 1, fy, mix(blossom, rgb(0x000000), 0.15));
        put(s, fx, fy - 1, mix(blossom, rgb(0xffffff), 0.2));
    }
}
