//! The meadow at the Woods' edge, where the person goes bug catching with a companion or two. A
//! third Woods activity: where rummaging is reading signs and catching a moment, and fishing is
//! knowing where each fish keeps and when it bites, bug catching is stalking. Creep up on
//! something without startling it, learn its rhythm, and swing the net at the one moment it
//! will not see coming.

// Being built: the catching itself is still to come.
#![allow(dead_code)]

pub mod art;
pub mod bugs;
mod scenery;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Patch, Playground};

/// Where anyone may stand: the meadow grass, short of the woods along the back and the edges.
pub const GROUND: Patch = (20.0, 104.0, 364.0, 206.0);

/// The parts of the meadow different bugs keep to, each with the perches the scenery puts a
/// flower head, a grass tip, a leaf or a reed top at, for a bug to settle on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Haunt {
    /// The wildflower patch, left of middle.
    Flowers,
    /// The long grass, front right.
    Grass,
    /// The bramble thicket on the left edge.
    Bramble,
    /// The fallen log across the back.
    Log,
    /// The old stump, back right of the log.
    Stump,
    /// The reeds round the boggy pond, right.
    Reeds,
}

impl Haunt {
    pub const ALL: [Self; 6] = [
        Self::Flowers,
        Self::Grass,
        Self::Bramble,
        Self::Log,
        Self::Stump,
        Self::Reeds,
    ];

    /// What the person calls it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Flowers => "the wildflowers",
            Self::Grass => "the long grass",
            Self::Bramble => "the brambles",
            Self::Log => "the fallen log",
            Self::Stump => "the old stump",
            Self::Reeds => "the reeds by the pond",
        }
    }

    /// Its extent: left, top, right, bottom. The scenery is painted to fill it.
    pub fn area(self) -> Patch {
        match self {
            Self::Flowers => (70.0, 110.0, 170.0, 150.0),
            Self::Grass => (230.0, 162.0, 350.0, 204.0),
            Self::Bramble => (6.0, 68.0, 68.0, 124.0),
            Self::Log => (146.0, 88.0, 254.0, 108.0),
            Self::Stump => (266.0, 78.0, 302.0, 108.0),
            Self::Reeds => (292.0, 96.0, 374.0, 152.0),
        }
    }

    /// Where a bug can settle: on a flower head, a grass tip, a bramble leaf, the top of the log
    /// or the stump, a reed's head. Over the pond, where a dragonfly hovers.
    pub fn perches(self) -> &'static [(f32, f32)] {
        match self {
            Self::Flowers => &[
                (78.0, 118.0),
                (92.0, 126.0),
                (104.0, 116.0),
                (118.0, 130.0),
                (130.0, 120.0),
                (144.0, 128.0),
                (156.0, 118.0),
                (100.0, 140.0),
                (136.0, 142.0),
                (162.0, 138.0),
            ],
            Self::Grass => &[
                (238.0, 170.0),
                (256.0, 182.0),
                (270.0, 168.0),
                (288.0, 190.0),
                (302.0, 176.0),
                (318.0, 186.0),
                (334.0, 172.0),
                (346.0, 194.0),
            ],
            Self::Bramble => &[
                (18.0, 84.0),
                (32.0, 76.0),
                (46.0, 90.0),
                (58.0, 80.0),
                (24.0, 104.0),
                (52.0, 110.0),
                (38.0, 118.0),
            ],
            Self::Log => &[
                (160.0, 95.0),
                (178.0, 93.0),
                (198.0, 94.0),
                (218.0, 93.0),
                (238.0, 96.0),
            ],
            Self::Stump => &[(278.0, 84.0), (290.0, 83.0), (284.0, 96.0)],
            Self::Reeds => &[
                (300.0, 104.0),
                (312.0, 100.0),
                (326.0, 106.0),
                (346.0, 102.0),
                (362.0, 108.0),
                (314.0, 128.0),
                (336.0, 134.0),
                (356.0, 126.0),
            ],
        }
    }
}

/// The pond in the reeds: centre and radii. Nobody wades in.
pub const POND: (f32, f32, f32, f32) = (332.0, 134.0, 40.0, 16.0);

/// Ground nobody stands on: the bramble, the log, the stump.
const SOLID: [Patch; 3] = [
    (0.0, 0.0, 70.0, 126.0),
    (146.0, 100.0, 254.0, 110.0),
    (264.0, 98.0, 304.0, 110.0),
];

pub fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = GROUND;
    let inside = |(l, t, r, b): Patch| x >= l && x <= r && y >= t && y <= b;
    let (cx, cy, rx, ry) = POND;
    let in_pond = ((x - cx) / (rx + 4.0)).powi(2) + ((y - cy) / (ry + 4.0)).powi(2) <= 1.0;
    inside((left, top, right, bottom)) && !in_pond && !SOLID.iter().any(|patch| inside(*patch))
}

const PLACES: [(&str, (f32, f32)); 2] = [("centre", (196.0, 160.0)), ("left", (90.0, 180.0))];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Out of the Woods along the path, from the left.
        entrance: (-24.0, 186.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// The meadow, with the companions who came walking out of the trees.
pub fn open(cast: &Cast, party: &[Id], now: f32, hilltop: &Arrangement) -> Playground {
    let backdrop = scenery::backdrop(hilltop);
    let nightlights = Nightlights {
        lamps: scenery::lamplight(),
        sky: scenery::night_sky(&backdrop),
        indoors: false,
    };
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        layout(),
        backdrop,
        scenery::foreground(),
        Vec::new(),
    );
    ground.set_nightlights(nightlights);
    ground
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_perch_lies_in_its_haunt() {
        for haunt in Haunt::ALL {
            let (left, top, right, bottom) = haunt.area();
            for &(x, y) in haunt.perches() {
                assert!(
                    (left..=right).contains(&x) && (top..=bottom).contains(&y),
                    "a perch at {x}, {y} is outside {}",
                    haunt.name()
                );
            }
        }
    }

    #[test]
    fn somewhere_to_stand_near_every_perch() {
        for haunt in Haunt::ALL {
            for &(x, y) in haunt.perches() {
                let near = (0..60)
                    .any(|dy| (-40..=40).any(|dx| walkable(x + dx as f32, y + 8.0 + dy as f32)));
                assert!(near, "nowhere to stand within reach of {x}, {y}");
            }
        }
    }
}
