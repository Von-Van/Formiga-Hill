//! The Hilltop: the colony's own place at the top of the Hill, empty at first. Whatever the Woods
//! turns up can stand on any of its spots, wherever the person likes, and the colony plays among
//! it all: sitting on the stones, looking through the telescope, napping by the berry bush.

mod scenery;

use crate::cast::Cast;
use crate::finds::{self, art};
use crate::playground::{Attraction, Layout, Patch, Playground, Prop};
use std::collections::BTreeMap;

/// What stands where: a find's id for each spot that has one.
pub type Arrangement = BTreeMap<u8, String>;

/// Where pieces can stand, as the middle of where each meets the ground: three rows across the
/// summit, back to front. Any spot takes any piece; they are spaced so the widest piece fits.
pub const SPOTS: [(f32, f32); 18] = [
    (108.0, 122.0),
    (158.0, 122.0),
    (208.0, 122.0),
    (258.0, 122.0),
    (308.0, 122.0),
    (358.0, 122.0),
    (34.0, 162.0),
    (90.0, 162.0),
    (146.0, 162.0),
    (202.0, 162.0),
    (258.0, 162.0),
    (314.0, 162.0),
    (64.0, 204.0),
    (122.0, 204.0),
    (180.0, 204.0),
    (238.0, 204.0),
    (296.0, 204.0),
    (352.0, 204.0),
];

/// The summit's grass, and the old tree's trunk at the back on the left.
pub const GROUND: Patch = (10.0, 112.0, 374.0, 208.0);
pub const TREE: Patch = (36.0, 96.0, 70.0, 116.0);
/// How much room each spot keeps clear around where a piece meets the ground.
const FOOTPRINT: (f32, f32, f32) = (16.0, 8.0, 3.0);

/// Whether someone can stand here: on the grass, clear of the tree and of every spot, taken or
/// not, so nobody wanders into a piece.
fn walkable(x: f32, y: f32) -> bool {
    let inside = |(l, t, r, b): Patch| x >= l && x <= r && y >= t && y <= b;
    let (half, behind, before) = FOOTPRINT;
    inside(GROUND)
        && !inside(TREE)
        && !SPOTS
            .iter()
            .any(|(sx, sy)| inside((sx - half, sy - behind, sx + half, sy + before)))
}

const PLACES: [(&str, (f32, f32)); 3] = [
    ("centre", (200.0, 182.0)),
    ("front", (210.0, 206.0)),
    ("left", (40.0, 190.0)),
];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Up the path from the station, onto the summit at the front left.
        entrance: (-24.0, 196.0),
        entrance_step: (-18.0, 5.0),
    }
}

/// The pieces standing on the summit, as props drawn among the colony.
pub fn props(arrangement: &Arrangement) -> Vec<Prop> {
    arrangement
        .iter()
        .filter_map(|(spot, id)| {
            let (x, y) = *SPOTS.get(usize::from(*spot))?;
            finds::find(id)?;
            let piece = art::piece(id);
            let at = (x as i32 - piece.anchor.0, y as i32 - piece.anchor.1);
            Some(Prop::new(piece.sprite, at, y))
        })
        .collect()
}

/// What the colony can go and enjoy: each piece, from in front of it.
pub fn attractions(arrangement: &Arrangement) -> Vec<Attraction> {
    arrangement
        .iter()
        .filter_map(|(spot, id)| {
            let (x, y) = *SPOTS.get(usize::from(*spot))?;
            let find = finds::find(id)?;
            // Beside it on whichever side has more room, a little in front.
            let side = if x < 192.0 { 22.0 } else { -22.0 };
            Some(Attraction {
                stand: (x + side, y + 6.0),
                facing_x: x,
                use_: find.use_,
            })
        })
        .collect()
}

/// The summit with the colony walking up onto it, and whatever has been placed.
pub fn open(cast: &Cast, now: f32, arrangement: &Arrangement) -> Playground {
    let mut ground = Playground::new(
        cast,
        now,
        layout(),
        scenery::backdrop(),
        scenery::foreground(),
        props(arrangement),
    );
    ground.set_attractions(attractions(arrangement));
    ground
}

/// The spot nearest a point in the scene, if it is close enough to mean that one.
pub fn spot_at(x: f32, y: f32) -> Option<u8> {
    SPOTS
        .iter()
        .enumerate()
        .map(|(index, (sx, sy))| {
            (
                index,
                ((x - sx) / 24.0).powi(2) + ((y - (sy - 14.0)) / 20.0).powi(2),
            )
        })
        .filter(|(_, reach)| *reach <= 1.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::CATALOGUE;

    #[test]
    fn spots_leave_room_for_the_widest_piece_and_for_walking_between() {
        let half = art::PIECE_MAX.0 as f32 / 2.0;
        for (index, (x, y)) in SPOTS.iter().enumerate() {
            assert!(
                *x - half >= 0.0 && *x + half <= 384.0,
                "spot {index} is off the edge"
            );
            for (other, (ox, oy)) in SPOTS.iter().enumerate().skip(index + 1) {
                if oy == y {
                    assert!(
                        (ox - x).abs() >= art::PIECE_MAX.0 as f32,
                        "{index} and {other} overlap"
                    );
                }
            }
        }
        assert!(walkable(200.0, 182.0) && walkable(40.0, 190.0));
    }

    #[test]
    fn any_spot_takes_any_piece() {
        for (index, find) in CATALOGUE.iter().enumerate() {
            for spot in 0..SPOTS.len() as u8 {
                let arrangement = Arrangement::from([(spot, find.id.to_owned())]);
                let props = props(&arrangement);
                assert_eq!(props.len(), 1, "{} at {spot}", find.id);
                let (left, top, right, bottom) = props[0].bounds();
                assert!(
                    left >= -4 && right < 388 && top >= -8 && bottom < 216,
                    "{} spills off spot {spot} ({index})",
                    find.id
                );
            }
        }
    }

    #[test]
    fn clicking_near_a_spot_finds_it() {
        for (index, (x, y)) in SPOTS.iter().enumerate() {
            assert_eq!(spot_at(*x, *y - 10.0), Some(index as u8));
        }
        assert_eq!(spot_at(5.0, 5.0), None);
    }
}
