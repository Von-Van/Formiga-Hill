//! How each find looks: a small icon for the basket, the satchel and the journal, and the piece it
//! becomes on the Hilltop. Drawn to the same standard as the areas: shaded with ramps, lit from the
//! upper left, outlined in a darker shade of their own colour, never black.

mod brush;
mod built;
mod earth;
mod hedgerow;
mod hollow;
mod relic;
mod undergrowth;
mod water;

use super::{Kind, find};
use crate::paint::{Ramp, bevel, ellipse, rgba};
use formiga_art::{Canvas, Rgba};

/// Icons are this many pixels square.
pub const ICON: u32 = 9;
/// No piece is bigger than this, so any Hilltop spot can take any piece.
pub const PIECE_MAX: (u32, u32) = (48, 56);

/// A piece as it stands on the Hilltop: its sprite, and the point in it that stands on the spot,
/// the middle of where it meets the ground.
pub struct Piece {
    pub sprite: Canvas,
    pub anchor: (i32, i32),
}

/// The icon for a find, or an empty one for an id Hill doesn't know.
pub fn icon(id: &str) -> Canvas {
    let Some(find) = find(id) else {
        return Canvas::new(ICON, ICON);
    };
    let drawn = match find.kind {
        _ if super::is_relic(id) => relic::icon(id),
        _ if super::is_foraged(id) => hedgerow::icon(id),
        Kind::Dig => earth::icon(id),
        Kind::Reach => hollow::icon(id),
        Kind::Scoop => water::icon(id),
        Kind::Shake => undergrowth::icon(id),
    };
    drawn.unwrap_or_else(|| placeholder_icon(find.kind))
}

/// The piece a find becomes on the Hilltop.
pub fn piece(id: &str) -> Piece {
    let Some(find) = find(id) else {
        return placeholder_piece(Kind::Dig);
    };
    let drawn = match find.kind {
        _ if super::is_relic(id) => relic::piece(id),
        _ if super::is_foraged(id) => hedgerow::piece(id),
        Kind::Dig => earth::piece(id),
        Kind::Reach => hollow::piece(id),
        Kind::Scoop => water::piece(id),
        Kind::Shake => undergrowth::piece(id),
    };
    let piece = drawn.unwrap_or_else(|| placeholder_piece(find.kind));
    debug_assert!(
        piece.sprite.width() <= PIECE_MAX.0 && piece.sprite.height() <= PIECE_MAX.1,
        "{id} is too big for a spot"
    );
    piece
}

/// A find planted on the Hilltop as it stands at `stage` of its growing (see `finds::growing`):
/// a mound of turned earth with a sprout at first, and then each time a little more like its full
/// piece. Anything without its stages drawn stands as its full piece.
pub fn stage(id: &str, stage: u8) -> Piece {
    let Some(find) = find(id) else {
        return placeholder_piece(Kind::Dig);
    };
    let drawn = match find.kind {
        _ if super::is_foraged(id) => hedgerow::stage(id, stage),
        Kind::Dig => earth::stage(id, stage),
        Kind::Reach => hollow::stage(id, stage),
        Kind::Shake => undergrowth::stage(id, stage),
        _ => None,
    };
    let piece = drawn.unwrap_or_else(|| self::piece(id));
    debug_assert!(
        piece.sprite.width() <= PIECE_MAX.0 && piece.sprite.height() <= PIECE_MAX.1,
        "{id} at stage {stage} is too big for a spot"
    );
    piece
}

/// What a plan builds (see `finds::plans`), standing on its one spot.
pub fn built(plan: &str) -> Piece {
    let piece = built::piece(plan).unwrap_or_else(|| placeholder_piece(Kind::Reach));
    debug_assert!(
        piece.sprite.width() <= PIECE_MAX.0 && piece.sprite.height() <= PIECE_MAX.1,
        "{plan} is too big for a spot"
    );
    piece
}

/// What shines from something built after dark: each light's colour, and where in the piece.
pub fn built_lights(plan: &str) -> Vec<(Rgba, (i32, i32))> {
    built::lights(plan)
}

fn tint(kind: Kind) -> Ramp {
    match kind {
        Kind::Dig => Ramp::new(0x4a3326, 0x6e4c36, 0x8f6747, 0xad855e, 0xc9a57c),
        Kind::Reach => Ramp::new(0x3a2a22, 0x5a4232, 0x7a5a3a, 0x977552, 0xb4936e),
        Kind::Scoop => Ramp::new(0x22404a, 0x33606a, 0x4a8290, 0x6aa2ae, 0x9cc8cf),
        Kind::Shake => Ramp::new(0x2c5233, 0x3d6e45, 0x518a55, 0x6fa866, 0x93c67e),
    }
}

/// Stands in for an icon not drawn yet.
fn placeholder_icon(kind: Kind) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    let ramp = tint(kind);
    ellipse(&mut canvas, 4, 4, 3, 3, ramp.edge);
    ellipse(&mut canvas, 4, 4, 2, 2, ramp.base);
    canvas.set(3, 3, ramp.shine);
    canvas
}

/// Stands in for a piece not drawn yet: a plain block on a shadow.
fn placeholder_piece(kind: Kind) -> Piece {
    let mut sprite = Canvas::new(20, 18);
    ellipse(&mut sprite, 10, 16, 9, 2, rgba(0x203020, 70));
    bevel(&mut sprite, 3, 2, 14, 14, tint(kind));
    Piece {
        sprite,
        anchor: (10, 16),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::CATALOGUE;

    /// How far a piece rises above where it stands, in pixels.
    fn rises(piece: &Piece) -> i32 {
        let (_, top, _, _) = piece.sprite.alpha_bounds().expect("nothing drawn");
        piece.anchor.1 - top as i32
    }

    #[test]
    fn everything_built_is_drawn_bigger_than_its_finds_and_fits_any_spot() {
        for plan in &crate::finds::plans::PLANS {
            let drawn = built::piece(plan.id).unwrap_or_else(|| panic!("{} is not drawn", plan.id));
            let (width, height) = (drawn.sprite.width(), drawn.sprite.height());
            assert!(
                width <= PIECE_MAX.0 && height <= PIECE_MAX.1,
                "{} is {width}x{height}",
                plan.id
            );
            let (x, y) = drawn.anchor;
            assert!(x >= 0 && y >= 0 && x < width as i32 && y < height as i32);
            // How much of a spot it takes up: its drawing's extent, across and up.
            let extent = |piece: &Piece| {
                let (left, top, right, bottom) =
                    piece.sprite.alpha_bounds().expect("nothing drawn");
                (right - left + 1) * (bottom - top + 1)
            };
            for (id, _) in plan.needs {
                assert!(
                    extent(&drawn) > extent(&piece(id)),
                    "{} is no grander than the {id} it takes",
                    plan.id
                );
            }
        }
    }

    #[test]
    fn everything_planted_comes_up_a_little_more_each_stage_and_fits_any_spot() {
        for growth in crate::finds::growing::GROWING {
            let full = piece(growth.id);
            let mut heights = Vec::new();
            for at in 0..growth.stages.len() as u8 {
                let young = stage(growth.id, at);
                let (width, height) = (young.sprite.width(), young.sprite.height());
                assert!(
                    width <= PIECE_MAX.0 && height <= PIECE_MAX.1,
                    "{} at {at} is {width}x{height}",
                    growth.id
                );
                let (x, y) = young.anchor;
                assert!(x >= 0 && y >= 0 && x < width as i32 && y < height as i32);
                assert_ne!(
                    young.sprite, full.sprite,
                    "{} at {at} is not drawn",
                    growth.id
                );
                heights.push(rises(&young));
            }
            heights.push(rises(&full));
            // A last stage may flower or fruit at its full height, but nothing comes up shorter
            // than it was, and it starts out well short of grown.
            let grown = heights[heights.len() - 1];
            assert!(
                heights.windows(2).all(|pair| pair[0] <= pair[1]) && heights[0] + 4 <= grown,
                "{} does not come up stage by stage: {heights:?}",
                growth.id
            );
        }
    }

    #[test]
    fn every_find_has_an_icon_and_a_piece_that_fits_any_spot() {
        let all = CATALOGUE.iter().chain(crate::finds::FORAGED.iter());
        for find in all.chain(crate::finds::RELICS.iter()) {
            let icon = icon(find.id);
            assert_eq!((icon.width(), icon.height()), (ICON, ICON), "{}", find.id);
            let drawn = icon.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(drawn >= 12, "{}'s icon is barely there", find.id);

            let piece = piece(find.id);
            let (width, height) = (piece.sprite.width(), piece.sprite.height());
            assert!(
                width <= PIECE_MAX.0 && height <= PIECE_MAX.1,
                "{} is {width}x{height}",
                find.id
            );
            let (x, y) = piece.anchor;
            assert!(
                x >= 0 && y >= 0 && x < width as i32 && y < height as i32,
                "{}'s anchor is off it",
                find.id
            );
            assert!(
                piece.sprite.alpha_bounds().is_some(),
                "{} is invisible",
                find.id
            );
        }
    }
}
