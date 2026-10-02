//! How each find looks: a small icon for the basket, the satchel and the journal, and the piece it
//! becomes on the Hilltop. Drawn to the same standard as the areas: shaded with ramps, lit from the
//! upper left, outlined in a darker shade of their own colour, never black.

mod brush;
mod earth;
mod hollow;
mod undergrowth;
mod water;

use super::{Kind, find};
use crate::paint::{Ramp, bevel, ellipse, rgba};
use formiga_art::Canvas;

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

    #[test]
    fn every_find_has_an_icon_and_a_piece_that_fits_any_spot() {
        for find in &CATALOGUE {
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
