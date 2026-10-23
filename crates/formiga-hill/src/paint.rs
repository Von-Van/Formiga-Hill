//! Painting tools for Hill's scenery.
//!
//! Hill's areas are drawn at Formiga's pixel scale but with more care than Desktop's overlays can
//! afford: every material has a ramp of tones, light comes from the upper left, edges are outlined
//! in a darker shade of their own colour rather than in black (so the creatures, which are outlined
//! in near-black, always read first), and surfaces carry texture from a deterministic hash, so a
//! scene is the same pixel for pixel every time it is drawn.
//!
//! Most of the tools are Desktop's own, shared by the companion apps and drawing exactly as Hill's
//! did. Hill keeps its own `ellipse`, which draws nothing for a zero radius, and its own `polygon`,
//! which takes whole-pixel corners and leaves out a pixel whose centre lies on a right-hand edge;
//! its `mix` cuts each channel down to a whole number, as Desktop's `mix_truncated` does.

use formiga_art::{Canvas, Rgba};

pub use formiga_art::paint::{
    Ramp, bevel, blit, chance, hline, line, mix_truncated as mix, noise, over, put, rect, rgb,
    rgba, vline,
};

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

/// Just the part of a picture with anything drawn in it, and where its top-left corner was: a
/// layer the size of the scene with a little painted on it, cut down so laying it over a scene
/// looks only at what was painted. A layer with nothing on it comes back as one clear pixel.
pub fn trimmed(layer: &Canvas) -> (Canvas, (i32, i32)) {
    let Some((left, top, right, bottom)) = layer.alpha_bounds() else {
        return (Canvas::new(1, 1), (0, 0));
    };
    let (left, top) = (left as i32, top as i32);
    let mut sprite = Canvas::new(
        (right as i32 - left + 1) as u32,
        (bottom as i32 - top + 1) as u32,
    );
    for y in 0..sprite.height() as i32 {
        for x in 0..sprite.width() as i32 {
            sprite.set(x, y, layer.get(left + x, top + y));
        }
    }
    (sprite, (left, top))
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
