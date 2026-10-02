//! The finds in the earth: the finds dug up (Kind::Dig): their icons and their Hilltop pieces.

use super::Piece;
use formiga_art::Canvas;

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(_id: &str) -> Option<Canvas> {
    None
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(_id: &str) -> Option<Piece> {
    None
}
