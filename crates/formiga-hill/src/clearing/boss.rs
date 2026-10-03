//! The Cursor Sovereign as it is drawn: an enormous mouse pointer, crowned, with a glare, looming
//! over the clearing. It is the one thing in Hill outlined in true black: it is the desktop's own
//! pointer, come to the Hill, and it should look like it doesn't belong.

use crate::cast::Id;
use crate::paint::{Ramp, line, mix, polygon, put, rect, rgb, rgba};
use crate::playground::Patch;
use formiga_art::{Canvas, Rgba};
use std::f32::consts::TAU;

/// Where the pointer's tip rests while it hovers.
pub const HOME: (f32, f32) = (262.0, 46.0);
/// How many scene pixels one unit of the pointer's outline is.
const SCALE: f32 = 6.0;

/// The pointer's outline, in units, tip first, as the classic arrow is drawn.
const ARROW: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 16.0),
    (4.0, 12.6),
    (6.8, 18.6),
    (9.4, 17.4),
    (6.6, 11.6),
    (11.6, 11.6),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// Hovering, bobbing, menacing.
    Hover,
    /// Pressing down where something stands.
    Click { at: (f32, f32) },
    /// Hauling one of them across the stage, and dropping it.
    Drag {
        who: Id,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// Dragging a selection out around them all.
    Select { area: Patch },
    /// Knocked back by a blow.
    Reel,
    /// Breaking into pieces.
    Shatter,
    /// What is left of it, a very ordinary little pointer, scurrying off.
    Flee,
    /// Gone.
    Gone,
}

impl Move {
    /// How long the move takes before the stage carries on.
    pub fn secs(self) -> f32 {
        match self {
            Self::Hover | Self::Gone => 0.0,
            Self::Click { .. } => 1.2,
            Self::Drag { .. } => 2.6,
            Self::Select { .. } => 2.4,
            Self::Reel => 0.8,
            Self::Shatter => 1.8,
            Self::Flee => 2.4,
        }
    }
}

/// Where the tip is, `t` seconds into a move.
pub fn tip(step: Move, t: f32) -> (f32, f32) {
    let ease = |a: (f32, f32), b: (f32, f32), u: f32| {
        let u = u.clamp(0.0, 1.0);
        let u = u * u * (3.0 - 2.0 * u);
        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
    };
    match step {
        Move::Hover | Move::Shatter | Move::Gone => HOME,
        Move::Click { at } => {
            // Swoop over, press, come back.
            let over = (at.0, at.1 - 34.0);
            if t < 0.45 {
                ease(HOME, over, t / 0.45)
            } else if t < 0.7 {
                (over.0, over.1 + 6.0)
            } else {
                ease(over, HOME, (t - 0.7) / 0.5)
            }
        }
        Move::Drag { from, to, .. } => {
            let grab = (from.0, from.1 - 30.0);
            let drop = (to.0, to.1 - 30.0);
            if t < 0.5 {
                ease(HOME, grab, t / 0.5)
            } else if t < 1.9 {
                // Hauled across, with a wobble.
                let along = ease(grab, drop, (t - 0.5) / 1.4);
                (along.0, along.1 - ((t - 0.5) * 7.0).sin().abs() * 10.0)
            } else {
                ease(drop, HOME, (t - 1.9) / 0.7)
            }
        }
        Move::Select { area } => {
            let (left, top, _, _) = area;
            ease(HOME, (left - 4.0, top - 4.0), t / 0.4)
        }
        Move::Reel => {
            // Knocked back and up, settling home again.
            let left = (1.0 - t / 0.8).max(0.0);
            (HOME.0 + 10.0 * left, HOME.1 - 4.0 * left)
        }
        Move::Flee => ease((HOME.0 + 20.0, 150.0), (420.0, 190.0), t / 2.4),
    }
}

/// Draws the Sovereign as it is `t` seconds into its move. `now` keeps it bobbing.
pub fn draw(scene: &mut Canvas, step: Move, t: f32, now: f32, reduce_motion: bool) {
    match step {
        Move::Gone => {}
        Move::Shatter => shatter(scene, t, reduce_motion),
        Move::Flee => {
            let at = if reduce_motion {
                (330.0, 180.0)
            } else {
                tip(step, t)
            };
            little_pointer(scene, at, now, reduce_motion);
        }
        Move::Select { area } => {
            selection(scene, area, t, now, reduce_motion);
            pointer(scene, tip(step, t), false, now, reduce_motion);
        }
        _ => {
            let mut at = tip(step, t);
            if !reduce_motion && step == Move::Hover {
                at.1 += (now * 1.6).sin() * 3.0;
            }
            if reduce_motion && !matches!(step, Move::Hover | Move::Reel) {
                // A cut, not a swoop: it is simply there.
                at = match step {
                    Move::Click { at } => (at.0, at.1 - 30.0),
                    Move::Drag { to, .. } => (to.0, to.1 - 30.0),
                    _ => at,
                };
            }
            let hit = step == Move::Reel;
            if let Move::Click { at: target } = step
                && (0.45..1.0).contains(&t)
            {
                shockwave(
                    scene,
                    (target.0, target.1 - 22.0),
                    (t - 0.45) / 0.55,
                    reduce_motion,
                );
            }
            pointer(scene, at, hit, now, reduce_motion);
        }
    }
}

const INK: Rgba = rgb(0x0c0a10);
const WHITE: Ramp = Ramp::new(0x6a6280, 0xb4acc8, 0xe6e0f0, 0xf6f2fc, 0xffffff);
const CROWN: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xd4a842, 0xecc864, 0xfaeaa8);

/// The great pointer, its tip at `at`: a violet aura, the white arrow outlined in black, a glare,
/// a crown. Flushed red when struck.
fn pointer(scene: &mut Canvas, at: (f32, f32), hit: bool, now: f32, reduce_motion: bool) {
    let points: Vec<(i32, i32)> = ARROW
        .iter()
        .map(|(x, y)| ((at.0 + x * SCALE) as i32, (at.1 + y * SCALE) as i32))
        .collect();
    // The aura, breathing.
    let breath = if reduce_motion {
        0.5
    } else {
        (now * 2.0).sin() * 0.5 + 0.5
    };
    for ring in (1..=6).rev() {
        let alpha = (14.0 + 10.0 * breath) as u8 / ring as u8 + 4;
        let grown: Vec<(i32, i32)> = ARROW
            .iter()
            .map(|(x, y)| {
                let (cx, cy) = (4.5, 11.0);
                let grow = 1.0 + ring as f32 * 0.07;
                (
                    (at.0 + (cx + (x - cx) * grow) * SCALE) as i32,
                    (at.1 + (cy + (y - cy) * grow) * SCALE) as i32,
                )
            })
            .collect();
        polygon(scene, &grown, |_, _| Some(rgba(0x9a5ad8, alpha)));
    }
    let bottom = at.1 + 18.6 * SCALE;
    polygon(scene, &points, |x, y| {
        let down = (y as f32 - at.1) / (bottom - at.1);
        let across = (x as f32 - at.0) / (11.6 * SCALE);
        let lit = 1.0 - down * 0.7 - across * 0.4;
        let tone = if lit > 0.8 {
            WHITE.shine
        } else if lit > 0.55 {
            WHITE.light
        } else if lit > 0.3 {
            WHITE.base
        } else {
            WHITE.shadow
        };
        Some(if hit {
            mix(tone, rgb(0xe04040), 0.45)
        } else {
            tone
        })
    });
    // A gleam down the left edge, as anime has it.
    for step in 4..60 {
        let y = at.1 as i32 + step;
        if (step / 9) % 2 == 0 {
            put(scene, at.0 as i32 + 3, y, rgba(0xffffff, 220));
        }
    }
    // The outline, thick and black.
    for (index, from) in points.iter().enumerate() {
        let to = points[(index + 1) % points.len()];
        for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
            line(
                scene,
                (from.0 + dx, from.1 + dy),
                (to.0 + dx, to.1 + dy),
                INK,
            );
        }
    }
    glare(scene, at, hit);
    crown(scene, (at.0 + 2.0, at.1 - 2.0));
}

/// Two narrowed eyes set in the arrow, glaring down at the clearing.
fn glare(scene: &mut Canvas, at: (f32, f32), hit: bool) {
    let eye = |scene: &mut Canvas, x: i32, y: i32, slant: i32| {
        rect(scene, x, y, 6, 4, INK);
        rect(
            scene,
            x + 1,
            y + 1,
            4,
            2,
            if hit { rgb(0xffe080) } else { rgb(0xe02838) },
        );
        put(scene, x + 2, y + 1, rgb(0xffffff));
        // The brow, scowling.
        line(scene, (x - 1, y - 2 + slant), (x + 6, y - 2 - slant), INK);
        line(scene, (x - 1, y - 3 + slant), (x + 6, y - 3 - slant), INK);
    };
    let (x, y) = (at.0 as i32, at.1 as i32);
    eye(scene, x + 6, y + 40, -1);
    eye(scene, x + 18, y + 48, 1);
}

/// A small gold crown, slightly askew on the tip.
fn crown(scene: &mut Canvas, at: (f32, f32)) {
    let (x, y) = (at.0 as i32 - 6, at.1 as i32 - 9);
    for (dx, height) in [(0, 6), (3, 4), (6, 7), (9, 4), (12, 6)] {
        for row in 0..height {
            let tone = if dx < 6 { CROWN.light } else { CROWN.base };
            put(scene, x + dx, y + 7 - row, tone);
            put(scene, x + dx + 1, y + 7 - row, CROWN.shadow);
        }
    }
    rect(scene, x, y + 7, 14, 3, CROWN.base);
    for dx in 0..14 {
        put(scene, x + dx, y + 10, CROWN.edge);
    }
    put(scene, x + 6, y + 8, rgb(0xd02840));
    put(scene, x + 2, y + 8, rgb(0x3a6ad0));
    put(scene, x + 10, y + 8, rgb(0x3a6ad0));
    line(scene, (x - 1, y + 10), (x - 1, y + 2), CROWN.edge);
}

/// The ring a click sends out, `progress` of the way spread.
fn shockwave(scene: &mut Canvas, at: (f32, f32), progress: f32, reduce_motion: bool) {
    let rings: &[f32] = if reduce_motion {
        &[0.6]
    } else {
        &[progress, progress * 0.6]
    };
    for (index, spread) in rings.iter().enumerate() {
        let radius = 6.0 + spread * 34.0;
        let alpha = ((1.0 - spread) * 230.0) as u8;
        for step in 0..72 {
            let angle = step as f32 / 72.0 * TAU;
            let x = at.0 + angle.cos() * radius;
            let y = at.1 + angle.sin() * radius * 0.45;
            let color = if index == 0 {
                rgba(0xfff4c0, alpha)
            } else {
                rgba(0xb48af0, alpha)
            };
            put(scene, x as i32, y as i32, color);
        }
    }
}

/// The selection being dragged out: marching ants round a pale wash.
fn selection(scene: &mut Canvas, area: Patch, t: f32, now: f32, reduce_motion: bool) {
    let grow = if reduce_motion {
        1.0
    } else {
        ((t - 0.3) / 0.9).clamp(0.0, 1.0)
    };
    if grow <= 0.0 {
        return;
    }
    let (left, top, right, bottom) = area;
    let (width, height) = (
        ((right - left) * grow) as i32,
        ((bottom - top) * grow) as i32,
    );
    let (x, y) = (left as i32, top as i32);
    rect(scene, x, y, width, height, rgba(0x6aa0f0, 50));
    let march = if reduce_motion {
        0
    } else {
        (now * 12.0) as i32
    };
    let ant = |i: i32| {
        if ((i + march) / 3) % 2 == 0 {
            INK
        } else {
            rgb(0xffffff)
        }
    };
    for dx in 0..width {
        put(scene, x + dx, y, ant(dx));
        put(scene, x + dx, y + height - 1, ant(dx + 1));
    }
    for dy in 0..height {
        put(scene, x, y + dy, ant(dy + 1));
        put(scene, x + width - 1, y + dy, ant(dy));
    }
}

/// The pointer breaking apart: shards flying out from where it hung, fading.
fn shatter(scene: &mut Canvas, t: f32, reduce_motion: bool) {
    let progress = if reduce_motion {
        0.5
    } else {
        (t / 1.8).clamp(0.0, 1.0)
    };
    let centre = (HOME.0 + 30.0, HOME.1 + 56.0);
    for shard in 0..18 {
        let angle = shard as f32 / 18.0 * TAU + 0.3;
        let distance = progress * (40.0 + (shard % 5) as f32 * 14.0);
        let (x, y) = (
            centre.0 + angle.cos() * distance,
            centre.1 + angle.sin() * distance + progress * progress * 30.0,
        );
        let alpha = ((1.0 - progress) * 255.0) as u8;
        let size = 2 + (shard % 3);
        let tone = if shard % 4 == 0 {
            rgba(0x0c0a10, alpha)
        } else {
            rgba(0xf6f2fc, alpha)
        };
        rect(scene, x as i32, y as i32, size, size, tone);
    }
    if progress < 0.3 {
        let alpha = ((0.3 - progress) / 0.3 * 200.0) as u8;
        rect(
            scene,
            0,
            0,
            scene.width() as i32,
            scene.height() as i32,
            rgba(0xffffff, alpha),
        );
    }
}

/// What is left: a very ordinary little pointer, hurrying off on tiny legs.
fn little_pointer(scene: &mut Canvas, at: (f32, f32), now: f32, reduce_motion: bool) {
    const SMALL: [&str; 11] = [
        "#.......", "##......", "#o#.....", "#oo#....", "#ooo#...", "#oooo#..", "#ooooo#.",
        "#oo####.", "#o#o#...", "##.#o#..", "#...##..",
    ];
    let (x, y) = (at.0 as i32, at.1 as i32 - 14);
    for (dy, row) in SMALL.iter().enumerate() {
        for (dx, cell) in row.bytes().enumerate() {
            let color = match cell {
                b'#' => INK,
                b'o' => rgb(0xffffff),
                _ => continue,
            };
            put(scene, x + dx as i32, y + dy as i32, color);
        }
    }
    let stride = if reduce_motion {
        0
    } else {
        ((now * 16.0).sin() * 1.5) as i32
    };
    line(scene, (x + 2, y + 11), (x + 1 + stride, y + 14), INK);
    line(scene, (x + 4, y + 11), (x + 5 - stride, y + 14), INK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_move_comes_back_home_or_goes() {
        for step in [
            Move::Click { at: (120.0, 186.0) },
            Move::Drag {
                who: 1,
                from: (90.0, 186.0),
                to: (170.0, 190.0),
            },
            Move::Reel,
        ] {
            let end = tip(step, step.secs());
            assert!(
                (end.0 - HOME.0).abs() < 1.0 && (end.1 - HOME.1).abs() < 1.0,
                "{step:?} ends at {end:?}"
            );
        }
        let fled = tip(Move::Flee, Move::Flee.secs());
        assert!(fled.0 > 384.0, "it scurries off the edge");
    }

    #[test]
    fn the_sovereign_is_drawn_in_every_move() {
        for step in [
            Move::Hover,
            Move::Click { at: (120.0, 186.0) },
            Move::Select {
                area: (40.0, 150.0, 200.0, 204.0),
            },
            Move::Reel,
            Move::Shatter,
            Move::Flee,
        ] {
            let mut scene = Canvas::new(384, 216);
            draw(&mut scene, step, step.secs() * 0.5, 1.0, false);
            assert!(scene.alpha_bounds().is_some(), "{step:?} drew nothing");
        }
        let mut gone = Canvas::new(384, 216);
        draw(&mut gone, Move::Gone, 0.0, 1.0, false);
        assert!(gone.alpha_bounds().is_none());
    }
}
