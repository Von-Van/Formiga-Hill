//! Little signs over a creature's head: a heart, a start of surprise, a note of a game, a huff,
//! a sparkle, a doze. Pixel sprites, so they belong to the scene rather than to the window.

use crate::character::Cue;
use crate::paint::{ellipse, put, rgba};
use formiga_art::{Canvas, Rgba};

const HEART: [&str; 6] = [
    ".##.##.", "#oo#oo#", "#ooooo#", ".#ooo#.", "..#o#..", "...#...",
];
const EXCLAIM: [&str; 8] = ["###", "#o#", "#o#", "#o#", "#o#", "###", "#o#", "###"];
const NOTE: [&str; 7] = [
    "..###", "..#o#", "..#.#", "..#..", "###..", "#o#..", "###..",
];
const SPARKLE: [&str; 5] = ["..#..", "..o..", "#o*o#", "..o..", "..#.."];
const DOZE: [&str; 5] = ["###", "..#", ".#.", "#..", "###"];
const SPEECH: [&str; 7] = [
    ".#######.",
    "#ooooooo#",
    "#o*o*o*o#",
    "#ooooooo#",
    ".###.###.",
    "....##...",
    "....#....",
];

/// Draws `cue` with its bottom centre at `(x, y)`, `age` seconds after it began.
pub fn draw_cue(scene: &mut Canvas, cue: Cue, x: i32, y: i32, age: f32, reduce_motion: bool) {
    // Signs drift up a little and fade towards the end of a second and a half; held still when
    // motion is reduced, so the sign is there without anything moving.
    let rise = if reduce_motion {
        0
    } else {
        (age * 7.0).min(10.0) as i32
    };
    // A doze and a speech bubble last as long as what they belong to; the rest are a moment's.
    let lasting = matches!(cue, Cue::Sleep | Cue::Speech);
    let fade = if reduce_motion || lasting {
        255
    } else {
        (255.0 * (1.0 - ((age - 1.1) / 0.6).clamp(0.0, 1.0))) as u8
    };
    if fade == 0 {
        return;
    }
    match cue {
        Cue::Heart => {
            sprite(
                scene,
                &HEART,
                x - 3,
                y - 6 - rise,
                [0x8c2a3a, 0xf26b8a, 0xffd3df],
                fade,
            );
            if age > 0.35 || reduce_motion {
                let late = if reduce_motion {
                    0
                } else {
                    ((age - 0.35) * 7.0).min(10.0) as i32
                };
                sprite(
                    scene,
                    &HEART,
                    x + 4,
                    y - 3 - late,
                    [0x8c2a3a, 0xf26b8a, 0xffd3df],
                    fade / 2 + fade / 4,
                );
            }
        }
        Cue::Exclaim => {
            let hop = if reduce_motion || age > 0.2 { 0 } else { 2 };
            sprite(
                scene,
                &EXCLAIM,
                x - 1,
                y - 8 - hop,
                [0x6b4a24, 0xf5d25e, 0xffffff],
                fade,
            );
        }
        Cue::Note => {
            let sway = if reduce_motion {
                0
            } else {
                ((age * 5.0).sin() * 2.0) as i32
            };
            sprite(
                scene,
                &NOTE,
                x - 2 + sway,
                y - 7 - rise,
                [0x3b2a4a, 0x9b7fd0, 0xffffff],
                fade,
            );
        }
        Cue::Sparkle => {
            let twinkle = reduce_motion || (age * 6.0) as i32 % 2 == 0;
            let colors = if twinkle {
                [0xc9a14e, 0xf6e3a2, 0xffffff]
            } else {
                [0xe4c06c, 0xffffff, 0xffffff]
            };
            sprite(scene, &SPARKLE, x - 8, y - 5 - rise / 2, colors, fade);
            sprite(
                scene,
                &SPARKLE,
                x + 4,
                y - 9 - rise,
                colors,
                fade / 2 + fade / 3,
            );
        }
        Cue::Huff => {
            // A puff blown out to one side.
            let drift = if reduce_motion {
                2
            } else {
                (age * 9.0).min(8.0) as i32
            };
            let alpha = (fade as u32 * 3 / 4) as u8;
            ellipse(scene, x + 4 + drift, y - 2, 3, 2, rgba(0xd8d4d8, alpha));
            ellipse(scene, x + 7 + drift, y - 4, 2, 2, rgba(0xeeeaee, alpha));
        }
        Cue::Speech => {
            // Held while the line is on show; the dots light one at a time, then all three.
            let lit = if reduce_motion {
                3
            } else {
                (age * 3.0) as usize % 4
            };
            let middle: String = SPEECH[2]
                .chars()
                .enumerate()
                .map(|(index, cell)| match cell {
                    '*' if lit < 3 && index != 2 + lit * 2 => 'o',
                    other => other,
                })
                .collect();
            let rows = [
                SPEECH[0], SPEECH[1], &middle, SPEECH[3], SPEECH[4], SPEECH[5], SPEECH[6],
            ];
            sprite(
                scene,
                &rows,
                x - 4,
                y - 7,
                [0x6b5a48, 0xfbf6ee, 0x6b5a48],
                255,
            );
        }
        Cue::Sleep => {
            // Two z's, the second higher and fainter, coming round again and again.
            let cycle = if reduce_motion {
                0.5
            } else {
                (age * 0.6).fract()
            };
            let up = (cycle * 8.0) as i32;
            let alpha = (255.0 * (1.0 - cycle * 0.8)) as u8;
            sprite(
                scene,
                &DOZE,
                x + 2,
                y - 5 - up,
                [0x3b4a6a, 0xffffff, 0xffffff],
                alpha,
            );
            sprite(
                scene,
                &DOZE,
                x + 7,
                y - 11 - up,
                [0x3b4a6a, 0xffffff, 0xffffff],
                alpha / 2,
            );
        }
    }
}

/// A sprite drawn from rows of `#` (outline), `o` (fill) and `*` (highlight).
fn sprite(scene: &mut Canvas, rows: &[&str], x: i32, y: i32, colors: [u32; 3], alpha: u8) {
    let color = |hex: u32| -> Rgba { rgba(hex, alpha) };
    for (dy, row) in rows.iter().enumerate() {
        for (dx, cell) in row.bytes().enumerate() {
            let paint = match cell {
                b'#' => color(colors[0]),
                b'o' => color(colors[1]),
                b'*' => color(colors[2]),
                _ => continue,
            };
            put(scene, x + dx as i32, y + dy as i32, paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cue_draws_something_and_then_fades() {
        for cue in [
            Cue::Heart,
            Cue::Exclaim,
            Cue::Note,
            Cue::Huff,
            Cue::Sparkle,
            Cue::Sleep,
        ] {
            let mut fresh = Canvas::new(40, 40);
            draw_cue(&mut fresh, cue, 20, 30, 0.2, false);
            assert!(fresh.alpha_bounds().is_some(), "{cue:?} drew nothing");
            if cue != Cue::Sleep {
                // Every passing sign fades; see below for the lasting ones.
                let mut gone = Canvas::new(40, 40);
                draw_cue(&mut gone, cue, 20, 30, 5.0, false);
                assert!(gone.alpha_bounds().is_none(), "{cue:?} never faded");
            }
        }
    }

    #[test]
    fn a_doze_and_a_speech_bubble_last_as_long_as_their_moment() {
        for cue in [Cue::Sleep, Cue::Speech] {
            let mut late = Canvas::new(40, 40);
            draw_cue(&mut late, cue, 20, 30, 30.0, false);
            assert!(late.alpha_bounds().is_some(), "{cue:?} faded away");
        }
    }

    #[test]
    fn with_reduced_motion_a_cue_holds_still() {
        let draw = |age| {
            let mut canvas = Canvas::new(40, 40);
            draw_cue(&mut canvas, Cue::Note, 20, 30, age, true);
            canvas
        };
        assert_eq!(draw(0.1), draw(1.0));
    }
}
