//! Painting tools for Hill's scenery.
//!
//! Hill's areas are drawn at Formiga's pixel scale but with more care than Desktop's overlays can
//! afford: every material has a ramp of tones, light comes from the upper left, edges are outlined
//! in a darker shade of their own colour rather than in black (so the creatures, which are outlined
//! in near-black, always read first), and surfaces carry texture from a deterministic hash, so a
//! scene is the same pixel for pixel every time it is drawn.

use formiga_art::{Canvas, Rgba};

pub const fn rgb(hex: u32) -> Rgba {
    Rgba::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}

pub const fn rgba(hex: u32, alpha: u8) -> Rgba {
    Rgba::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, alpha)
}

/// One material's tones, darkest first. `edge` outlines it; `shine` is used a pixel at a time.
#[derive(Clone, Copy, Debug)]
pub struct Ramp {
    pub edge: Rgba,
    pub shadow: Rgba,
    pub base: Rgba,
    pub light: Rgba,
    pub shine: Rgba,
}

impl Ramp {
    pub const fn new(edge: u32, shadow: u32, base: u32, light: u32, shine: u32) -> Self {
        Self {
            edge: rgb(edge),
            shadow: rgb(shadow),
            base: rgb(base),
            light: rgb(light),
            shine: rgb(shine),
        }
    }
}

/// A well-mixed hash of a pixel and a salt: texture that never changes between runs.
pub fn noise(x: i32, y: i32, salt: u32) -> u32 {
    let mut hash = (x as u32).wrapping_mul(0x27d4_eb2d)
        ^ (y as u32).wrapping_mul(0x1656_67b1)
        ^ salt.wrapping_mul(0x9e37_79b9);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2c1b_3c6d);
    hash ^= hash >> 12;
    hash = hash.wrapping_mul(0x297a_2d39);
    hash ^ (hash >> 15)
}

/// True for about `per_256` pixels in every 256.
pub fn chance(x: i32, y: i32, salt: u32, per_256: u32) -> bool {
    noise(x, y, salt) & 0xff < per_256
}

/// Source-over: `top` drawn on `bottom` by its alpha. Onto an opaque pixel this is a plain mix;
/// onto a clear or half-clear one the colours are weighted by how much of each is there, so a
/// soft shadow or a glow drawn into an empty sprite keeps its own colour rather than going dark.
pub fn over(top: Rgba, bottom: Rgba) -> Rgba {
    match (top.a, bottom.a) {
        (255, _) | (_, 0) => top,
        (0, _) => bottom,
        (top_alpha, bottom_alpha) => {
            let top_alpha = u32::from(top_alpha);
            // How much of the bottom shows through, out of 255 * 255.
            let under = u32::from(bottom_alpha) * (255 - top_alpha);
            let total = top_alpha * 255 + under;
            let mix = |a: u8, b: u8| {
                ((u32::from(a) * top_alpha * 255 + u32::from(b) * under) / total) as u8
            };
            Rgba::new(
                mix(top.r, bottom.r),
                mix(top.g, bottom.g),
                mix(top.b, bottom.b),
                (total / 255).min(255) as u8,
            )
        }
    }
}

/// `a` moved `amount` of the way to `b`.
pub fn mix(a: Rgba, b: Rgba, amount: f32) -> Rgba {
    let channel = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * amount) as u8;
    Rgba::new(
        channel(a.r, b.r),
        channel(a.g, b.g),
        channel(a.b, b.b),
        channel(a.a, b.a),
    )
}

pub fn put(canvas: &mut Canvas, x: i32, y: i32, color: Rgba) {
    let blended = over(color, canvas.get(x, y));
    canvas.set(x, y, blended);
}

pub fn hline(canvas: &mut Canvas, x: i32, y: i32, width: i32, color: Rgba) {
    for dx in 0..width {
        put(canvas, x + dx, y, color);
    }
}

pub fn vline(canvas: &mut Canvas, x: i32, y: i32, height: i32, color: Rgba) {
    for dy in 0..height {
        put(canvas, x, y + dy, color);
    }
}

pub fn rect(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32, color: Rgba) {
    for dy in 0..height {
        hline(canvas, x, y + dy, width, color);
    }
}

/// A filled ellipse that blends, for glows, shade and smoke.
pub fn ellipse(canvas: &mut Canvas, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba) {
    if rx <= 0 || ry <= 0 {
        return;
    }
    let (rx2, ry2) = (i64::from(rx * rx), i64::from(ry * ry));
    for y in -ry..=ry {
        for x in -rx..=rx {
            if i64::from(x * x) * ry2 + i64::from(y * y) * rx2 <= rx2 * ry2 {
                put(canvas, cx + x, cy + y, color);
            }
        }
    }
}

/// A box lit from the upper left: highlight along the top and left, shade along the bottom and
/// right, outlined in the material's own edge tone.
pub fn bevel(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32, ramp: Ramp) {
    rect(canvas, x, y, width, height, ramp.edge);
    if width <= 2 || height <= 2 {
        return;
    }
    rect(canvas, x + 1, y + 1, width - 2, height - 2, ramp.base);
    hline(canvas, x + 1, y + 1, width - 2, ramp.light);
    vline(canvas, x + 1, y + 1, height - 2, ramp.light);
    hline(canvas, x + 1, y + height - 2, width - 2, ramp.shadow);
    vline(canvas, x + width - 2, y + 1, height - 2, ramp.shadow);
    put(canvas, x + 1, y + 1, ramp.shine);
}

/// Fills a polygon, asking `paint` for each pixel's colour. Pixel centres decide what is inside,
/// so two shapes sharing an edge neither overlap nor leave a seam.
pub fn polygon(
    canvas: &mut Canvas,
    points: &[(i32, i32)],
    mut paint: impl FnMut(i32, i32) -> Option<Rgba>,
) {
    let Some(top) = points.iter().map(|point| point.1).min() else {
        return;
    };
    let bottom = points.iter().map(|point| point.1).max().unwrap_or(top);
    let mut crossings = Vec::new();
    for y in top..bottom {
        let centre = y as f32 + 0.5;
        crossings.clear();
        for (index, &(x0, y0)) in points.iter().enumerate() {
            let (x1, y1) = points[(index + 1) % points.len()];
            let (fy0, fy1) = (y0 as f32, y1 as f32);
            if (fy0 <= centre && fy1 > centre) || (fy1 <= centre && fy0 > centre) {
                let t = (centre - fy0) / (fy1 - fy0);
                crossings.push(x0 as f32 + t * (x1 - x0) as f32);
            }
        }
        crossings.sort_by(f32::total_cmp);
        for span in crossings.chunks_exact(2) {
            let start = (span[0] - 0.5).ceil() as i32;
            let end = (span[1] - 0.5).ceil() as i32;
            for x in start..end {
                if let Some(color) = paint(x, y) {
                    put(canvas, x, y, color);
                }
            }
        }
    }
}

/// A one-pixel line that blends.
pub fn line(canvas: &mut Canvas, from: (i32, i32), to: (i32, i32), color: Rgba) {
    let ((mut x0, mut y0), (x1, y1)) = (from, to);
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let mut error = dx + dy;
    loop {
        put(canvas, x0, y0, color);
        if (x0, y0) == (x1, y1) {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x0 += sx;
        }
        if twice <= dx {
            error += dx;
            y0 += sy;
        }
    }
}

/// Every opaque pixel of `sprite` drawn over `canvas` with its top-left corner at `(x, y)`.
pub fn blit(canvas: &mut Canvas, sprite: &Canvas, x: i32, y: i32) {
    for sy in 0..sprite.height() as i32 {
        for sx in 0..sprite.width() as i32 {
            let pixel = sprite.get(sx, sy);
            if pixel.a > 0 {
                put(canvas, x + sx, y + sy, pixel);
            }
        }
    }
}

/// A number from the hash in `0..n`.
pub fn pick(index: i32, axis: i32, salt: u32, n: i32) -> i32 {
    (noise(index, axis, salt) % n.max(1) as u32) as i32
}

/// Smooth noise in 0..1 that changes over cells of `size`: patches rather than speckle.
pub fn patches(x: i32, y: i32, size: (i32, i32), salt: u32) -> f32 {
    let (cell_x, cell_y) = (x.div_euclid(size.0), y.div_euclid(size.1));
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let sx = smooth(x.rem_euclid(size.0) as f32 / size.0 as f32);
    let sy = smooth(y.rem_euclid(size.1) as f32 / size.1 as f32);
    let corner = |dx: i32, dy: i32| (noise(cell_x + dx, cell_y + dy, salt) % 1024) as f32 / 1023.0;
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * sx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// Noise that rolls smoothly between whole-numbered points, for soft patches rather than
/// speckle.
pub fn smooth_noise(x: f32, y: f32, salt: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy) = (ease(x - ix as f32), ease(y - iy as f32));
    let corner = |dx: i32, dy: i32| (noise(ix + dx, iy + dy, salt) % 1000) as f32 / 999.0;
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// Smooth noise in 0..1 along a loop `period` long, for things that go all the way round.
pub fn looped(t: f32, period: i32, salt: u32) -> f32 {
    let cell = t.floor() as i32;
    let f = t - cell as f32;
    let ease = f * f * (3.0 - 2.0 * f);
    let at = |c: i32| (noise(c.rem_euclid(period), 0, salt) % 1024) as f32 / 1023.0;
    at(cell) + (at(cell + 1) - at(cell)) * ease
}

/// A ramp's tone by level, 0 its edge and 4 its shine.
pub fn tone(ramp: Ramp, level: i32) -> Rgba {
    match level {
        ..=0 => ramp.edge,
        1 => ramp.shadow,
        2 => ramp.base,
        3 => ramp.light,
        _ => ramp.shine,
    }
}

/// A point along a smooth curve through `points`, `t` of the way from the first to the last.
pub fn along(points: &[(f32, f32)], t: f32) -> (f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_over_nothing_keeps_the_colour() {
        let shadow = rgba(0x2a4a2a, 70);
        assert_eq!(over(shadow, Rgba::new(0, 0, 0, 0)), shadow);
        let ground = rgb(0x7db36c);
        let mixed = over(shadow, ground);
        assert_eq!(mixed.a, 255);
        assert!(mixed.g < ground.g && mixed.g > 0x4a, "{mixed:?}");
        // Two half-clear layers build up rather than darkening.
        let twice = over(shadow, shadow);
        assert!(twice.a > shadow.a && twice.g == shadow.g, "{twice:?}");
    }

    #[test]
    fn noise_is_stable_and_spread() {
        assert_eq!(noise(3, 4, 5), noise(3, 4, 5));
        let hits = (0..64)
            .flat_map(|y| (0..64).map(move |x| (x, y)))
            .filter(|&(x, y)| chance(x, y, 9, 64))
            .count();
        assert!((800..1250).contains(&hits), "{hits} of 4096 for a quarter");
    }

    #[test]
    fn polygons_sharing_an_edge_tile_without_gaps_or_overlap() {
        let mut canvas = Canvas::new(16, 16);
        let red = rgba(0xff0000, 128);
        polygon(&mut canvas, &[(0, 0), (16, 0), (0, 16)], |_, _| Some(red));
        polygon(&mut canvas, &[(16, 0), (16, 16), (0, 16)], |_, _| Some(red));
        assert!(canvas.pixels().iter().all(|pixel| pixel.a == 128));
    }

    #[test]
    fn half_transparent_paint_blends() {
        let blended = over(rgba(0xffffff, 128), rgb(0x000000));
        assert!((120..=136).contains(&blended.r));
        assert_eq!(blended.a, 255);
    }
}
