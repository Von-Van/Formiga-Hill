//! The relics: things won rather than found. For now, the arrow the Cursor Sovereign left behind,
//! which stands on the Hilltop as the fallen cursor: the desktop's own pointer, outlined in true
//! black like nothing else at the Hill, driven into the turf at a slant, its crown in the grass.

use super::Piece;
use crate::paint::{Ramp, line, polygon, put, rect, rgb, rgba};
use formiga_art::{Canvas, Rgba};

const INK: Rgba = rgb(0x0c0a10);
const WHITE: Ramp = Ramp::new(0x6a6280, 0xb4acc8, 0xe6e0f0, 0xf6f2fc, 0xffffff);
const GOLD: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xd4a842, 0xecc864, 0xfaeaa8);

pub fn icon(id: &str) -> Option<Canvas> {
    if id != "sovereign_arrow" {
        return None;
    }
    const ROWS: [&str; 9] = [
        "#........",
        "##.......",
        "#o#......",
        "#oo#.....",
        "#ooo#....",
        "#oooo#...",
        "#oo###...",
        "#o#o#....",
        "##.##....",
    ];
    let mut canvas = Canvas::new(super::ICON, super::ICON);
    for (y, row) in ROWS.iter().enumerate() {
        for (x, cell) in row.bytes().enumerate() {
            match cell {
                b'#' => canvas.set(x as i32 + 2, y as i32, INK),
                b'o' => canvas.set(
                    x as i32 + 2,
                    y as i32,
                    if y < 4 { WHITE.shine } else { WHITE.base },
                ),
                _ => {}
            }
        }
    }
    canvas.set(8, 0, GOLD.light);
    canvas.set(7, 1, GOLD.base);
    Some(canvas)
}

pub fn piece(id: &str) -> Option<Piece> {
    if id != "sovereign_arrow" {
        return None;
    }
    let mut s = Canvas::new(44, 56);
    let ground = 51;
    // Its shadow, and the turf heaved up where it struck.
    for (dx, half) in [(0, 18), (2, 16)] {
        for x in -half..=half {
            put(&mut s, 22 + x + dx, ground, rgba(0x2a4a2a, 70));
        }
    }
    for x in 8i32..30 {
        let lift = 2 - (x - 19).abs() / 5;
        if lift > 0 {
            for dy in 0..lift {
                put(&mut s, x, ground - 1 - dy, rgb(0x5a4030));
            }
            put(&mut s, x, ground - 1 - lift, rgb(0x6f9a4a));
        }
    }
    // The arrow, tip buried, leaning back: the classic pointer, scaled up and tipped over.
    let outline = [
        (18.0, 50.0),
        (10.0, 14.0),
        (17.0, 19.0),
        (14.0, 5.0),
        (19.0, 3.0),
        (22.0, 17.0),
        (31.0, 16.0),
    ];
    let points: Vec<(i32, i32)> = outline
        .iter()
        .map(|(x, y)| (*x as i32, *y as i32))
        .collect();
    polygon(&mut s, &points, |x, y| {
        let lit = 1.0 - y as f32 / 56.0 - (x as f32 - 10.0) / 40.0;
        Some(if lit > 0.65 {
            WHITE.shine
        } else if lit > 0.45 {
            WHITE.light
        } else if lit > 0.25 {
            WHITE.base
        } else {
            WHITE.shadow
        })
    });
    for (index, from) in points.iter().enumerate() {
        let to = points[(index + 1) % points.len()];
        line(&mut s, *from, to, INK);
        line(&mut s, (from.0 + 1, from.1), (to.0 + 1, to.1), INK);
    }
    // A gleam down its edge.
    for y in (16..44).step_by(2) {
        put(&mut s, 12 + (44 - y) / 8, y, rgba(0xffffff, 200));
    }
    // The crown, fallen in the grass at its foot.
    let (cx, cy) = (32, ground - 4);
    for (dx, height) in [(0, 3), (2, 2), (4, 4), (6, 2), (8, 3)] {
        for row in 0..height {
            put(&mut s, cx + dx, cy - row, GOLD.light);
        }
    }
    rect(&mut s, cx, cy, 10, 2, GOLD.base);
    for dx in 0..10 {
        put(&mut s, cx + dx, cy + 2, GOLD.edge);
    }
    put(&mut s, cx + 4, cy + 1, rgb(0xd02840));
    Some(Piece {
        sprite: s,
        anchor: (20, ground),
    })
}
