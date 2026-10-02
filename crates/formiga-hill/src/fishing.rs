//! The pool in the Woods, where the person goes fishing with a companion or two. A second Woods
//! activity: where rummaging is reading signs and catching a moment, fishing is knowing where each
//! fish lives, telling a nibble from a bite, and reeling one in without snapping the line.

pub mod angling;
pub mod art;
pub mod fish;
mod scenery;

use crate::cast::{Cast, Id};
use crate::playground::{Layout, Patch, Playground};

/// Where the one fishing sits on the near bank, and where a second stands ready with the net.
pub const SEAT: (f32, f32) = (112.0, 186.0);
pub const NET: (f32, f32) = (148.0, 190.0);

/// The open water, as an ellipse: centre and radii. Fish keep within it.
pub const WATER: (f32, f32, f32, f32) = (200.0, 112.0, 176.0, 50.0);

/// The parts of the pool different fish keep to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Haunt {
    Shallows,
    Lilies,
    Reeds,
    Falls,
    Deep,
}

impl Haunt {
    pub const ALL: [Self; 5] = [
        Self::Shallows,
        Self::Lilies,
        Self::Reeds,
        Self::Falls,
        Self::Deep,
    ];

    /// What the person calls it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Shallows => "the shallows",
            Self::Lilies => "the lily pads",
            Self::Reeds => "the reeds",
            Self::Falls => "under the falls",
            Self::Deep => "the deep water",
        }
    }

    /// Its middle and how far it reaches. The scenery is painted around these.
    pub fn area(self) -> ((f32, f32), f32) {
        match self {
            Self::Shallows => ((64.0, 140.0), 26.0),
            Self::Lilies => ((128.0, 98.0), 30.0),
            Self::Reeds => ((330.0, 112.0), 28.0),
            Self::Falls => ((204.0, 74.0), 24.0),
            Self::Deep => ((236.0, 124.0), 32.0),
        }
    }
}

/// Whether a point is on the open water, a little in from its edge.
pub fn in_water(x: f32, y: f32) -> bool {
    let (cx, cy, rx, ry) = WATER;
    ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 0.92
}

/// The near bank, where anyone can stand.
pub const BANK: Patch = (16.0, 168.0, 368.0, 206.0);

fn walkable(x: f32, y: f32) -> bool {
    let (l, t, r, b) = BANK;
    x >= l && x <= r && y >= t && y <= b
}

const PLACES: [(&str, (f32, f32)); 2] = [("centre", (200.0, 192.0)), ("left", (60.0, 194.0))];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: BANK,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Along the path round the pool, from the left.
        entrance: (-24.0, 196.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// The pool, with the companions who came walking up to the bank.
pub fn open(cast: &Cast, party: &[Id], now: f32) -> Playground {
    Playground::with_members(
        cast,
        party,
        now,
        layout(),
        scenery::backdrop(),
        scenery::foreground(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_haunt_is_on_the_water_and_the_bank_is_dry() {
        for haunt in Haunt::ALL {
            let ((x, y), _) = haunt.area();
            assert!(in_water(x, y), "{} is on dry land", haunt.name());
        }
        assert!(walkable(SEAT.0, SEAT.1) && walkable(NET.0, NET.1));
        assert!(!in_water(SEAT.0, SEAT.1));
    }
}
