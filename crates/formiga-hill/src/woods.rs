//! The Woods: where the person goes rummaging with a companion or two, and brings home finds for
//! the Hilltop. Unlike the Fairground, the person plays here: chooses where to search, reads the
//! signs, and catches the moment. Who came along changes how it goes and what turns up.

pub mod rummage;
mod scenery;

use crate::cast::{Cast, Id};
use crate::finds::Kind;
use crate::playground::{Layout, Patch, Playground};

/// A place in the glade worth searching.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub name: &'static str,
    pub kind: Kind,
    /// Where whatever is hidden there shows itself: the mouth of the hollow, the water's surface.
    pub sign: (f32, f32),
    /// Where the companion stands to search it.
    pub stand: (f32, f32),
}

/// The glade's spots: three of each kind. The scenery is painted around these.
pub const SPOTS: [Spot; 12] = [
    Spot {
        name: "the shallows",
        kind: Kind::Scoop,
        sign: (92.0, 122.0),
        stand: (92.0, 138.0),
    },
    Spot {
        name: "the pool",
        kind: Kind::Scoop,
        sign: (212.0, 120.0),
        stand: (212.0, 138.0),
    },
    Spot {
        name: "the reed bed",
        kind: Kind::Scoop,
        sign: (300.0, 120.0),
        stand: (292.0, 138.0),
    },
    Spot {
        name: "the old oak's hollow",
        kind: Kind::Reach,
        sign: (354.0, 146.0),
        stand: (328.0, 172.0),
    },
    Spot {
        name: "the hollow log",
        kind: Kind::Reach,
        sign: (126.0, 166.0),
        stand: (108.0, 174.0),
    },
    Spot {
        name: "the fox hole",
        kind: Kind::Reach,
        sign: (40.0, 144.0),
        stand: (58.0, 156.0),
    },
    Spot {
        name: "the mossy bank",
        kind: Kind::Dig,
        sign: (252.0, 152.0),
        stand: (252.0, 162.0),
    },
    Spot {
        name: "the mushroom ring",
        kind: Kind::Dig,
        sign: (204.0, 186.0),
        stand: (204.0, 190.0),
    },
    Spot {
        name: "the root tangle",
        kind: Kind::Dig,
        sign: (316.0, 190.0),
        stand: (302.0, 198.0),
    },
    Spot {
        name: "the ferns",
        kind: Kind::Shake,
        sign: (26.0, 186.0),
        stand: (46.0, 200.0),
    },
    Spot {
        name: "the bramble",
        kind: Kind::Shake,
        sign: (262.0, 190.0),
        stand: (262.0, 202.0),
    },
    Spot {
        name: "the hazel",
        kind: Kind::Shake,
        sign: (152.0, 120.0),
        stand: (152.0, 142.0),
    },
];

/// Where anyone can walk: the near bank of the stream down to the front, less the log and the oak.
pub const GROUND: Patch = (12.0, 134.0, 372.0, 206.0);
/// The fallen log and the old oak's trunk, which nobody walks through.
pub const LOG: Patch = (116.0, 156.0, 186.0, 178.0);
pub const OAK: Patch = (336.0, 0.0, 384.0, 176.0);

fn walkable(x: f32, y: f32) -> bool {
    let inside = |(l, t, r, b): Patch| x >= l && x <= r && y >= t && y <= b;
    inside(GROUND) && !inside(LOG) && !inside(OAK)
}

/// The glade's named spots, for free play and for stories one day.
const PLACES: [(&str, (f32, f32)); 3] = [
    ("centre", (190.0, 160.0)),
    ("front", (190.0, 200.0)),
    ("left", (60.0, 186.0)),
];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        // In along the path from the green, at the front left.
        entrance: (-24.0, 192.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// The glade, with the companions who came walking in.
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
    fn every_spot_can_be_reached_and_each_kind_has_three() {
        for spot in SPOTS {
            assert!(
                walkable(spot.stand.0, spot.stand.1),
                "nobody can stand at {}",
                spot.name
            );
        }
        for kind in Kind::ALL {
            assert_eq!(SPOTS.iter().filter(|spot| spot.kind == kind).count(), 3);
        }
    }
}
