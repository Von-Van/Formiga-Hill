//! How each costume piece looks, facing right. A placeholder until each is drawn: a simple shape
//! in the piece's own colours, so placement can be checked on every companion.

use crate::paint::{ellipse, rect, rgb};
use formiga_art::Canvas;

/// A piece facing right; mirrored for a companion facing left.
pub fn sprite(id: &str) -> Canvas {
    match super::piece(id).map(|piece| piece.slot) {
        Some(super::Slot::Neck) => {
            let mut canvas = Canvas::new(10, 4);
            rect(&mut canvas, 0, 0, 10, 4, rgb(0xc04a40));
            canvas
        }
        _ => {
            let mut canvas = Canvas::new(10, 8);
            ellipse(&mut canvas, 5, 5, 4, 3, rgb(0xf5d25e));
            canvas
        }
    }
}

/// The point on the piece that sits on the companion: on the crown for a hat, at the throat for
/// a neck piece.
pub fn anchor(id: &str) -> (i32, i32) {
    let sprite = sprite(id);
    match super::piece(id).map(|piece| piece.slot) {
        Some(super::Slot::Neck) => (sprite.width() as i32 / 2, 1),
        _ => (sprite.width() as i32 / 2, sprite.height() as i32 - 1),
    }
}

/// The piece's icon for the dress-up box, nine pixels square.
pub fn icon(id: &str) -> Canvas {
    let mut canvas = Canvas::new(9, 9);
    let sprite = sprite(id);
    crate::paint::blit(
        &mut canvas,
        &sprite,
        (9 - sprite.width() as i32) / 2,
        (9 - sprite.height() as i32) / 2,
    );
    canvas
}
