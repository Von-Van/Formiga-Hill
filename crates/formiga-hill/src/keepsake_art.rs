//! Little pictures of Hill's souvenirs, seven pixels square, for wherever they are shown.

use crate::paint::{put, rgb};
use formiga_art::{Canvas, Rgba};

pub const ICON: i32 = 7;

/// Draws a souvenir with its top-left corner at `(x, y)`. Unknown ids draw nothing.
pub fn draw_souvenir(scene: &mut Canvas, id: &str, x: i32, y: i32) {
    let (rows, colors): ([&str; 7], [u32; 4]) = match id {
        // A bow of red gingham.
        "picnic_ribbon" => (
            [
                ".......", "##...##", "#o#.#o#", "#oo#oo#", "#o#.#o#", "##.#.##", "..#.#..",
            ],
            [0x6e2a2a, 0xd0574a, 0xf3e9cf, 0xffffff],
        ),
        // A daisy, pressed flat.
        "pressed_daisy" => (
            [
                "...o...", ".o.o.o.", "..ooo..", "oo*#*oo", "..ooo..", ".o.o.o.", "...o...",
            ],
            [0xe0a82a, 0xfbf6ee, 0xf5d25e, 0xffffff],
        ),
        // A copper penny, catching the light.
        "well_penny" => (
            [
                ".......", "..###..", ".#ooo#.", ".#o*o#.", ".#ooo#.", "..###..", ".......",
            ],
            [0x7a3e1e, 0xc0743a, 0xf2b07a, 0xffffff],
        ),
        // An acorn in its cap.
        "oak_acorn" => (
            [
                "...#...", "..###..", ".#####.", ".ooooo.", ".o*ooo.", "..ooo..", "...o...",
            ],
            [0x5a4232, 0x9a6a38, 0xd2a060, 0xffffff],
        ),
        // A soft grey feather.
        "swing_feather" => (
            [
                ".....##", "....#o#", "...#o*.", "..#oo..", ".#oo...", "#o.....", "#......",
            ],
            [0x7a8590, 0xd8dde2, 0xffffff, 0xffffff],
        ),
        // A glass marble with an amber twist through it.
        "chest_marble" => (
            [
                "..###..", ".#*oo#.", "#*oxxo#", "#oxooo#", "#ooxxo#", ".#ooo#.", "..###..",
            ],
            [0x2c5a86, 0x5c9bd2, 0xe6f4ff, 0xf0b44c],
        ),
        // A ticket stub, torn along its perforations.
        "fair_ticket" => (
            [
                ".......", "#######", "#o*o*o#", "#ooooo#", "#o*o*o#", "#######", "x.x.x.x",
            ],
            [0x8a2a30, 0xf3d8a0, 0xd0584c, 0xc9b080],
        ),
        _ => return,
    };
    let ink = |code: u8| -> Option<Rgba> {
        Some(rgb(match code {
            b'#' => colors[0],
            b'o' => colors[1],
            b'*' => colors[2],
            b'x' => colors[3],
            _ => return None,
        }))
    };
    for (dy, row) in rows.iter().enumerate() {
        for (dx, code) in row.bytes().enumerate() {
            if let Some(color) = ink(code) {
                put(scene, x + dx as i32, y + dy as i32, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::souvenirs;

    #[test]
    fn every_souvenir_has_a_picture() {
        for id in souvenirs::ids() {
            let mut canvas = Canvas::new(ICON as u32, ICON as u32);
            draw_souvenir(&mut canvas, id, 0, 0);
            let drawn = canvas.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(drawn >= 12, "{id} is barely there");
        }
    }
}
