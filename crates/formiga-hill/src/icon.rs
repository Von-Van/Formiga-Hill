//! Hill's own icon: the Hill, with its old tree on the crest, under a summer sky. It is drawn once
//! as pixel art, 32 pixels square, and every size is made from that drawing by whole pixels, so
//! each stays crisp. The same picture is the window's icon, the macOS bundle's `.icns` and the
//! Windows installer's `.ico`; the packaging scripts ask the binary for them, so there is one
//! source.

use crate::materials::{CLOUD, SKY_LOW, SKY_TOP};
use crate::paint::{Ramp, chance, ellipse, mix, put, rgb};
use formiga_art::{Canvas, Rgba};

/// How big the drawing is, in pixels each way.
pub const SIZE: u32 = 32;

const TURF: Ramp = Ramp::new(0x3e6a3c, 0x5d9150, 0x78ad62, 0x92c473, 0xb2da8c);
const CROWN: Ramp = Ramp::new(0x24452b, 0x335e3b, 0x447a4a, 0x5f9a5b, 0x86bd73);
const BARK: Rgba = rgb(0x6a4a32);
const PATH: Rgba = rgb(0xe6d3a4);

/// The icon as drawn: a rounded square of sky with the Hill rising out of the bottom of it.
pub fn draw() -> Canvas {
    let size = SIZE as i32;
    let mut icon = Canvas::new(SIZE, SIZE);
    let inside = |x: i32, y: i32| {
        // A rounded square, its corners cut on a radius of six.
        let corner = |a: i32| if a < 6 { 6 - a } else if a > size - 7 { a - (size - 7) } else { 0 };
        let (cx, cy) = (corner(x), corner(y));
        cx * cx + cy * cy <= 36
    };
    // The sky, banded towards the horizon.
    for y in 0..size {
        let band = mix(SKY_TOP, SKY_LOW, (y as f32 / 22.0).min(1.0) * 0.85);
        for x in 0..size {
            if inside(x, y) {
                icon.set(x, y, band);
            }
        }
    }
    // A fair-weather cloud.
    for (x, y, rx) in [(23, 7, 3), (26, 6, 3), (24, 5, 2)] {
        ellipse(&mut icon, x, y, rx, 2, CLOUD);
    }
    // The Hill: a broad dome, lit from the upper left, shaded down its right.
    let crest = |x: i32| -> i32 {
        let t = (x as f32 - 14.0) / 18.0;
        (13.0 + 14.0 * t * t).round() as i32
    };
    for x in 0..size {
        for y in crest(x)..size {
            if !inside(x, y) {
                continue;
            }
            let from_crest = y - crest(x);
            let lean = (x - 14) as f32 / 18.0;
            let mut color = if from_crest == 0 {
                TURF.shine
            } else if lean < -0.2 {
                TURF.light
            } else if lean > 0.45 {
                TURF.shadow
            } else {
                TURF.base
            };
            if y > 26 {
                color = mix(color, TURF.shadow, 0.4);
            }
            if chance(x, y, 31, 34) {
                color = mix(color, TURF.light, 0.5);
            }
            icon.set(x, y, color);
        }
    }
    // Its outline against the sky, in the turf's own darkest green.
    for x in 0..size {
        let y = crest(x) - 1;
        if inside(x, y) {
            put(&mut icon, x, y, mix(TURF.edge, SKY_LOW, 0.35));
        }
    }
    // The path winding up to the tree.
    for (x, y) in [
        (17, 30),
        (18, 29),
        (18, 28),
        (17, 27),
        (16, 26),
        (16, 25),
        (17, 24),
        (18, 23),
        (18, 22),
        (17, 21),
        (16, 20),
        (15, 19),
        (14, 18),
        (14, 17),
    ] {
        if inside(x, y) {
            icon.set(x, y, PATH);
        }
    }
    // The old tree on the crest: a short trunk and a crown of leaf clumps, lit from the left.
    for y in 9..14 {
        icon.set(13, y, BARK);
        icon.set(14, y, mix(BARK, rgb(0x2a1c14), 0.4));
    }
    icon.set(12, 13, BARK);
    icon.set(15, 13, mix(BARK, rgb(0x2a1c14), 0.4));
    // A broad oak's crown, wider than tall, as the Hilltop's own.
    for (cx, cy, rx, ry) in [(13, 6, 6, 4), (8, 8, 3, 3), (18, 8, 3, 3), (12, 4, 4, 3)] {
        ellipse(&mut icon, cx, cy, rx, ry, CROWN.edge);
    }
    for (cx, cy, rx, ry) in [(13, 6, 5, 3), (8, 8, 2, 2), (18, 8, 2, 2), (12, 4, 3, 2)] {
        ellipse(&mut icon, cx, cy, rx, ry, CROWN.base);
    }
    for (x, y) in [(9, 5), (10, 4), (11, 3), (8, 7), (12, 5), (13, 3), (10, 6)] {
        put(&mut icon, x, y, CROWN.light);
    }
    for (x, y) in [(17, 9), (18, 9), (16, 8), (19, 8), (15, 9), (13, 9)] {
        put(&mut icon, x, y, CROWN.shadow);
    }
    put(&mut icon, 10, 3, CROWN.shine);
    // A soft rim, so the square reads on a light dock or a dark one.
    for y in 0..size {
        for x in 0..size {
            if inside(x, y)
                && [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .any(|(dx, dy)| !inside(x + dx, y + dy))
            {
                let pixel = icon.get(x, y);
                icon.set(x, y, mix(pixel, rgb(0x2c3e33), 0.45));
            }
        }
    }
    icon
}

/// The icon at `size` pixels each way: scaled up by whole pixels, or down by averaging.
pub fn at(size: u32) -> Canvas {
    let drawn = draw();
    let mut out = Canvas::new(size, size);
    if size >= SIZE {
        let scale = size / SIZE;
        let offset = ((size - SIZE * scale) / 2) as i32;
        for y in 0..(SIZE * scale) as i32 {
            for x in 0..(SIZE * scale) as i32 {
                let pixel = drawn.get(x / scale as i32, y / scale as i32);
                out.set(x + offset, y + offset, pixel);
            }
        }
    } else {
        let block = SIZE / size;
        for y in 0..size as i32 {
            for x in 0..size as i32 {
                let mut sum = [0u32; 4];
                for dy in 0..block as i32 {
                    for dx in 0..block as i32 {
                        let p = drawn.get(x * block as i32 + dx, y * block as i32 + dy);
                        let a = u32::from(p.a);
                        sum[0] += u32::from(p.r) * a;
                        sum[1] += u32::from(p.g) * a;
                        sum[2] += u32::from(p.b) * a;
                        sum[3] += a;
                    }
                }
                let n = block * block;
                if sum[3] > 0 {
                    out.set(
                        x,
                        y,
                        Rgba::new(
                            (sum[0] / sum[3]) as u8,
                            (sum[1] / sum[3]) as u8,
                            (sum[2] / sum[3]) as u8,
                            (sum[3] / n) as u8,
                        ),
                    );
                }
            }
        }
    }
    out
}

/// A picture as PNG bytes.
pub fn png(canvas: &Canvas) -> Vec<u8> {
    let (width, height) = (canvas.width(), canvas.height());
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let pixel = canvas.get(x, y);
            data.extend([pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    // Writing into memory cannot fail but for a mismatch in size, which this never makes.
    if let Ok(mut writer) = encoder.write_header() {
        let _ = writer.write_image_data(&data);
    }
    bytes
}

/// A macOS `.icns`: each size as a PNG, under the type codes macOS reads PNGs from.
pub fn icns() -> Vec<u8> {
    // Each size at 1x and 2x, as an `.iconset` makes them.
    let entries: [(&[u8; 4], u32); 10] = [
        (b"icp4", 16),
        (b"ic11", 32),
        (b"icp5", 32),
        (b"ic12", 64),
        (b"ic07", 128),
        (b"ic13", 256),
        (b"ic08", 256),
        (b"ic14", 512),
        (b"ic09", 512),
        (b"ic10", 1024),
    ];
    let mut body = Vec::new();
    for (code, size) in entries {
        let data = png(&at(size));
        body.extend_from_slice(code);
        body.extend_from_slice(&(data.len() as u32 + 8).to_be_bytes());
        body.extend_from_slice(&data);
    }
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&(body.len() as u32 + 8).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

/// A Windows `.ico`: each size as a PNG, which every Windows since Vista reads.
pub fn ico() -> Vec<u8> {
    // Only sizes made from the drawing by whole pixels; Windows scales between them.
    let sizes = [16u32, 32, 64, 128, 256];
    let images: Vec<Vec<u8>> = sizes.iter().map(|size| png(&at(*size))).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (size, image) in sizes.iter().zip(&images) {
        // A side of 256 is written as 0.
        let side = if *size >= 256 { 0 } else { *size as u8 };
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    for image in images {
        out.extend_from_slice(&image);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_is_the_same_picture_made_crisp() {
        let drawn = draw();
        let big = at(256);
        for y in 0..SIZE as i32 {
            for x in 0..SIZE as i32 {
                assert_eq!(big.get(x * 8 + 3, y * 8 + 5), drawn.get(x, y));
            }
        }
        // The corners are clear, so the rounded square reads as one.
        assert_eq!(drawn.get(0, 0).a, 0);
        assert_eq!(at(16).get(0, 0).a, 0);
    }

    #[test]
    fn the_icon_files_say_what_they_hold() {
        let icns = icns();
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
            icns.len()
        );
        let ico = ico();
        assert_eq!(&ico[..4], &[0, 0, 1, 0]);
        let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
        let last = 6 + 16 * (count - 1);
        let size = u32::from_le_bytes(ico[last + 8..last + 12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(ico[last + 12..last + 16].try_into().unwrap()) as usize;
        assert_eq!(offset + size, ico.len(), "the last image runs to the end");
        assert_eq!(&ico[offset + 1..offset + 4], b"PNG");
    }
}
