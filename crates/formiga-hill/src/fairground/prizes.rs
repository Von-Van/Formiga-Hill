//! The hoopla stall's prizes: a paper windmill, a balloon and a pennant, each standing on its
//! painted block along the counter until someone rings it, and then carried about by whoever won
//! it for the rest of the visit. Like the dress-up box's pieces they are only for the visit: none
//! of them goes home, and none changes who anyone is.
//!
//! Drawn as the Fairground is: every material shaded with a ramp, lit from the upper left and
//! outlined in a darker shade of its own colour, never black, so the companion carrying one reads
//! first. The windmill turns and the pennant flutters; with motion reduced, both hold still.

use crate::paint::{Ramp, line, put};
use formiga_art::{Canvas, Rgba};

/// A prize to be won at hoopla.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Prize {
    Windmill,
    Balloon,
    Pennant,
}

impl Prize {
    /// The prizes in the order they stand along the counter, nearest the line first.
    pub const ALL: [Self; 3] = [Self::Windmill, Self::Balloon, Self::Pennant];

    /// As the person is told it, in the middle of a sentence.
    pub fn name(self) -> &'static str {
        match self {
            Self::Windmill => "the paper windmill",
            Self::Balloon => "the balloon",
            Self::Pennant => "the pennant",
        }
    }
}

/// Red and gold paper for the windmill's sails, a sky-blue balloon, a green pennant, hickory for
/// the sticks, and brass for the windmill's pin.
const SAIL_RED: Ramp = Ramp::new(0x7a1a28, 0xa82a38, 0xd8434a, 0xf06e68, 0xffa898);
const SAIL_GOLD: Ramp = Ramp::new(0x8a5a1a, 0xc08a2a, 0xf0c040, 0xf8dc70, 0xfff4b0);
const BALLOON: Ramp = Ramp::new(0x1c3c78, 0x2a5aa8, 0x3e7ed2, 0x6ea6ec, 0xc8e4ff);
const PENNANT: Ramp = Ramp::new(0x1e5a2e, 0x2a7e3e, 0x3ea456, 0x6cc47a, 0xb4ecb4);
const STICK: Ramp = Ramp::new(0x5e4228, 0x8a6640, 0xae8656, 0xc8a26e, 0xe0c08e);
const PIN: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
/// The balloon's string.
const STRING: Rgba = crate::paint::rgb(0xb4ae9e);

/// How tall the windmill's and the pennant's sticks are, standing in their blocks, and how long the
/// balloon's string is, tied to its block: low enough that they stand against the bare boards
/// under the stall's shelf.
const STICK_TALL: i32 = 7;
const STRING_LONG: i32 = 9;
/// How far above a companion's crown a prize it carries reaches, at most.
pub const CARRIED_ABOVE: i32 = 18;

/// Draws `prize` as it stands in its block on the counter, its foot at `(x, foot)`: the windmill
/// and the pennant on their sticks, the balloon tied to the block and bobbing a little above it.
pub fn standing(canvas: &mut Canvas, prize: Prize, (x, foot): (i32, i32), now: f32, still: bool) {
    match prize {
        Prize::Balloon => {
            let knot = (x + 1, foot - STRING_LONG + bob(now, 1.7, still));
            string(canvas, (x, foot), knot, now, still);
            balloon(canvas, knot);
        }
        Prize::Windmill | Prize::Pennant => {
            let top = (x, foot - STICK_TALL);
            stick(canvas, (x, foot), top);
            head(canvas, prize, top, true, now, still);
        }
    }
}

/// Draws `prize` carried by a companion facing right if `facing_right`, whose head's top middle
/// is at `crown` and whose throat is `throat` rows below it: the windmill and the pennant held up
/// in front of it on their sticks, the balloon tied on and floating up behind it.
pub fn carried(
    canvas: &mut Canvas,
    prize: Prize,
    crown: (i32, i32),
    throat: i32,
    facing_right: bool,
    now: f32,
    still: bool,
) {
    let ahead = if facing_right { 1 } else { -1 };
    match prize {
        Prize::Balloon => {
            let knot = (crown.0 - ahead * 7, crown.1 - 10 + bob(now, 1.5, still));
            string(
                canvas,
                (crown.0 - ahead * 3, crown.1 + throat + 2),
                knot,
                now,
                still,
            );
            balloon(canvas, knot);
        }
        Prize::Windmill | Prize::Pennant => {
            let held = (crown.0 + ahead * 7, crown.1 + throat + 3);
            let top = (crown.0 + ahead * 9, crown.1 - 8);
            stick(canvas, held, top);
            head(canvas, prize, top, facing_right, now, still);
        }
    }
}

/// How far a balloon bobs, up and down a pixel: not at all with motion reduced.
fn bob(now: f32, speed: f32, still: bool) -> i32 {
    if still {
        0
    } else {
        ((now * speed).sin() * 1.2).round() as i32
    }
}

/// A thin hickory stick from `foot` up to `top`, lit down its left.
fn stick(canvas: &mut Canvas, foot: (i32, i32), top: (i32, i32)) {
    line(canvas, foot, top, STICK.base);
    line(
        canvas,
        (foot.0 - 1, foot.1),
        (top.0 - 1, top.1 + 1),
        STICK.light,
    );
    put(canvas, top.0, top.1, STICK.edge);
}

/// The balloon's string, from where it is tied up to the balloon's knot, with a kink in it that
/// drifts (and holds still with motion reduced).
fn string(canvas: &mut Canvas, tied: (i32, i32), knot: (i32, i32), now: f32, still: bool) {
    let rows = (tied.1 - knot.1).max(1);
    let drift = if still { 0.0 } else { now * 2.3 };
    for step in 0..rows {
        let t = step as f32 / rows as f32;
        let sway = ((t * 5.0 + drift).sin() * 0.8 * t * (1.0 - t) * 4.0).round() as i32;
        let x = tied.0 + ((knot.0 - tied.0) as f32 * t).round() as i32 + sway;
        put(canvas, x, tied.1 - step, STRING);
    }
}

/// A balloon whose knot is at `(x, y)`: round, shaded from a shine at its upper left, outlined in
/// its own darkest blue.
fn balloon(canvas: &mut Canvas, (x, y): (i32, i32)) {
    stamp(
        canvas,
        (x - 2, y - 6),
        &[".###.", "#*lo#", "#loo#", "#oos#", ".#s#.", "..#.."],
        BALLOON,
    );
}

/// A windmill's sails about `at`, turning (held still with motion reduced), or a pennant flying
/// from `at` away from the way its carrier faces, fluttering.
fn head(
    canvas: &mut Canvas,
    prize: Prize,
    at: (i32, i32),
    facing_right: bool,
    now: f32,
    still: bool,
) {
    match prize {
        Prize::Windmill => {
            let turn = if still { 0.0 } else { now * 12.0 };
            sails(canvas, at, turn);
        }
        Prize::Pennant => {
            let flutter = !still && (now * 4.0) as i32 % 2 == 1;
            let rows: [&str; 4] = if flutter {
                ["####..", "#lloo#", "#oss#.", "##...."]
            } else {
                ["#####.", "#llos#", "#os##.", "##...."]
            };
            let away = if facing_right { -1 } else { 1 };
            for (dy, row) in rows.iter().enumerate() {
                for (dx, letter) in row.bytes().enumerate() {
                    let color = match letter {
                        b'#' => PENNANT.edge,
                        b's' => PENNANT.shadow,
                        b'o' => PENNANT.base,
                        b'l' => PENNANT.light,
                        _ => continue,
                    };
                    put(canvas, at.0 + away * dx as i32, at.1 + dy as i32, color);
                }
            }
        }
        Prize::Balloon => balloon(canvas, at),
    }
}

/// A paper windmill's four sails about its pin at `at`, red and gold by turns, each folded in to
/// the pin: lit towards the upper left, in shade towards the lower right, and the tips of the
/// sails in their own darkest tone. Turned a quarter, a windmill looks the same with its colours
/// swapped, so turning is the two drawn by turns (and held still with motion reduced).
fn sails(canvas: &mut Canvas, at: (i32, i32), turn: f32) {
    const SAILS: [&str; 7] = [
        "rrrr..g", ".rrr.gg", "..rrggg", "gggoggg", "gggrr..", "gg.rrr.", "g..rrrr",
    ];
    let turned = (turn / std::f32::consts::FRAC_PI_2) as i32 % 2 == 1;
    for (dy, row) in SAILS.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            let (x, y) = (dx as i32, dy as i32);
            let red = match letter {
                b'r' => !turned,
                b'g' => turned,
                b'o' => {
                    put(canvas, at.0 - 3 + x, at.1 - 3 + y, PIN.light);
                    continue;
                }
                _ => continue,
            };
            let ramp = if red { SAIL_RED } else { SAIL_GOLD };
            let tip = (x == 0 || x == 6) && (y == 0 || y == 6);
            let color = if tip {
                ramp.edge
            } else if x + y <= 4 {
                ramp.light
            } else if x + y >= 9 {
                ramp.shadow
            } else {
                ramp.base
            };
            put(canvas, at.0 - 3 + x, at.1 - 3 + y, color);
        }
    }
}

/// Paints rows of letters with their top-left at `(x, y)`: `#`, `s`, `o`, `l` and `*` are
/// `ramp`'s edge, shadow, base, light and shine; anything else is left clear.
fn stamp(canvas: &mut Canvas, (x, y): (i32, i32), rows: &[&str], ramp: Ramp) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            let color = match letter {
                b'#' => ramp.edge,
                b's' => ramp.shadow,
                b'o' => ramp.base,
                b'l' => ramp.light,
                b'*' => ramp.shine,
                _ => continue,
            };
            put(canvas, x + dx as i32, y + dy as i32, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drawn(paint: impl Fn(&mut Canvas)) -> Canvas {
        let mut canvas = Canvas::new(40, 40);
        paint(&mut canvas);
        canvas
    }

    #[test]
    fn every_prize_looks_like_itself_and_is_never_outlined_in_black() {
        let mut seen: Vec<Canvas> = Vec::new();
        for prize in Prize::ALL {
            let canvas = drawn(|canvas| standing(canvas, prize, (20, 34), 0.0, true));
            let pixels: Vec<_> = canvas.pixels().iter().filter(|p| p.a > 0).collect();
            assert!(pixels.len() >= 20, "{prize:?} is barely there");
            assert!(
                pixels
                    .iter()
                    .all(|p| u32::from(p.r) + u32::from(p.g) + u32::from(p.b) > 90),
                "{prize:?} is outlined in black"
            );
            assert!(!seen.contains(&canvas), "{prize:?} looks like another");
            seen.push(canvas);
        }
    }

    #[test]
    fn the_windmill_turns_and_the_pennant_flutters_unless_motion_is_reduced() {
        for prize in [Prize::Windmill, Prize::Pennant] {
            let at = |now: f32, still: bool| {
                drawn(|canvas| carried(canvas, prize, (20, 20), 8, true, now, still))
            };
            let moving: Vec<Canvas> = (0..8).map(|step| at(step as f32 * 0.07, false)).collect();
            assert!(
                moving.windows(2).any(|pair| pair[0] != pair[1]),
                "{prize:?} is stuck"
            );
            let held: Vec<Canvas> = (0..8).map(|step| at(step as f32 * 0.07, true)).collect();
            assert!(
                held.windows(2).all(|pair| pair[0] == pair[1]),
                "{prize:?} moves"
            );
        }
    }

    #[test]
    fn a_carried_prize_is_held_on_the_side_its_carrier_faces_and_rises_over_its_head() {
        for prize in [Prize::Windmill, Prize::Pennant] {
            let right = drawn(|canvas| carried(canvas, prize, (20, 20), 8, true, 0.0, true));
            let left = drawn(|canvas| carried(canvas, prize, (20, 20), 8, false, 0.0, true));
            let (l, top, r, _) = right.alpha_bounds().unwrap();
            assert!(top < 20, "{prize:?} is held below its carrier's head");
            assert!((l + r) / 2 > 20, "{prize:?} is held behind it");
            let (l, _, r, _) = left.alpha_bounds().unwrap();
            assert!((l + r) / 2 < 20);
        }
        let balloon = drawn(|canvas| carried(canvas, Prize::Balloon, (20, 20), 8, true, 0.0, true));
        let (left, top, right, _) = balloon.alpha_bounds().unwrap();
        assert!((left + right) / 2 < 20, "the balloon floats up behind it");
        assert!(top < 20, "the balloon floats over its head");
        for prize in Prize::ALL {
            let canvas = drawn(|canvas| carried(canvas, prize, (20, 24), 8, true, 0.0, true));
            let (_, top, _, _) = canvas.alpha_bounds().unwrap();
            assert!(
                top as i32 >= 24 - CARRIED_ABOVE,
                "{prize:?} reaches higher than a name tag allows for"
            );
        }
    }
}
