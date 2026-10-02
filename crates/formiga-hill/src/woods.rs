//! The Woods: where the person goes rummaging with a companion or two, and brings home finds for
//! the Hilltop. Unlike the Fairground, the person plays here: chooses where to search, reads the
//! signs, and catches the moment. Who came along changes how it goes and what turns up.

pub mod rummage;
mod scenery;

use crate::cast::{Cast, Id};
use crate::finds::{self, Kind};
use crate::hilltop::Arrangement;
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

/// Who can open up an extra spot: a place the glade only shows to certain company.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opener {
    /// An explorer, or anyone very curious, spots a badger's sett.
    Explorer,
    /// Only a little one fits into the crevice in the oak's roots.
    LittleOne,
    /// It takes two close friends to heave the mossy boulder over.
    ClosePair,
}

/// A spot only some company opens up. It is an extra chance, never the only way to anything:
/// whatever turns up there turns up at the ordinary spots too.
#[derive(Clone, Copy, Debug)]
pub struct Extra {
    pub spot: Spot,
    pub opener: Opener,
}

pub const EXTRAS: [Extra; 3] = [
    Extra {
        spot: Spot {
            name: "the badger sett",
            kind: Kind::Dig,
            sign: (352.0, 194.0),
            stand: (338.0, 204.0),
        },
        opener: Opener::Explorer,
    },
    Extra {
        spot: Spot {
            name: "the crevice in the roots",
            kind: Kind::Reach,
            sign: (336.0, 180.0),
            stand: (324.0, 188.0),
        },
        opener: Opener::LittleOne,
    },
    Extra {
        spot: Spot {
            name: "the mossy boulder",
            kind: Kind::Dig,
            sign: (82.0, 188.0),
            stand: (82.0, 200.0),
        },
        opener: Opener::ClosePair,
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

/// How what stands on the Hilltop changes the Woods. Nothing is unlocked by it and nothing is
/// shut out without it: it lends a little light, brings the rarest signs out sooner, and draws
/// the eye to the kinds of places the colony has been finding things in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Influence {
    /// Extra light each outing starts with.
    pub light: f32,
    /// How much sooner, in light, the rarest signs start to show.
    pub earlier: f32,
    /// How much likelier a spot of each kind is to hold something.
    pub richer: [f32; 4],
    /// What to tell the person about it as they set off.
    pub notes: Vec<&'static str>,
}

/// The most any of it can add up to, so the Hilltop helps without taking over.
const MOST_LIGHT: f32 = 20.0;
const MOST_EARLIER: f32 = 15.0;
const MOST_RICHER: f32 = 0.2;

pub fn influence(arrangement: &Arrangement) -> Influence {
    let mut influence = Influence::default();
    let note = |text: &'static str, notes: &mut Vec<&'static str>| {
        if !notes.contains(&text) {
            notes.push(text);
        }
    };
    for id in arrangement.values() {
        let Some(find) = finds::find(id) else {
            continue;
        };
        // Each piece draws the eye to the sort of place it came from.
        let richer = &mut influence.richer[find.kind.index()];
        *richer = (*richer + 0.04).min(MOST_RICHER);
        match find.id {
            "lost_lantern" => {
                influence.light += 10.0;
                note(
                    "The lantern post on the Hilltop lights the way home: a little more light.",
                    &mut influence.notes,
                );
            }
            "sun_coin" => {
                influence.light += 6.0;
                note(
                    "With the sundial to go by, nobody loses track of the time: a little more light.",
                    &mut influence.notes,
                );
            }
            "fallen_star" => {
                influence.light += 6.0;
                influence.earlier += 8.0;
                note(
                    "The star stone glows on the Hilltop: rare things show themselves sooner.",
                    &mut influence.notes,
                );
            }
            "brass_lens" => {
                influence.earlier += 10.0;
                note(
                    "From the telescope you can see where to look: rare things show themselves sooner.",
                    &mut influence.notes,
                );
            }
            _ => {}
        }
    }
    influence.light = influence.light.min(MOST_LIGHT);
    influence.earlier = influence.earlier.min(MOST_EARLIER);
    influence
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
    fn the_hilltop_helps_the_woods_but_only_so_far() {
        assert_eq!(influence(&Arrangement::new()), Influence::default());
        let lit = influence(&Arrangement::from([
            (0, "lost_lantern".to_owned()),
            (1, "lost_lantern".to_owned()),
            (2, "lost_lantern".to_owned()),
            (3, "brass_lens".to_owned()),
            (4, "fallen_star".to_owned()),
        ]));
        assert_eq!(lit.light, MOST_LIGHT);
        assert_eq!(lit.earlier, MOST_EARLIER);
        assert_eq!(
            lit.notes.len(),
            3,
            "each kind of help is told once: {:?}",
            lit.notes
        );
        let planted: Arrangement = (0..18)
            .map(|spot| (spot, "wild_berries".to_owned()))
            .collect();
        let richer = influence(&planted).richer;
        assert_eq!(richer[Kind::Shake.index()], MOST_RICHER);
        assert_eq!(richer[Kind::Dig.index()], 0.0);
    }

    #[test]
    fn every_spot_can_be_reached_and_each_kind_has_three() {
        for spot in SPOTS.iter().chain(EXTRAS.iter().map(|extra| &extra.spot)) {
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
