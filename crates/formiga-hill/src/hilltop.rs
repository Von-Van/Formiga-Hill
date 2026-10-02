//! The Hilltop: the colony's own place at the top of the Hill, empty at first. Whatever the Woods
//! turns up can stand on any of its spots, wherever the person likes, and the colony plays among
//! it all: sitting on the stones, looking through the telescope, napping by the berry bush.

mod scenery;

use crate::cast::Cast;
use crate::finds::{self, art};
use crate::paint::{blit, mix};
use crate::playground::{Attraction, Layout, Patch, Playground, Prop};
use formiga_art::{Canvas, Rgba};
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

/// How the Hilltop looks from somewhere below it: where the old tree stands on that view's
/// skyline, how far off it is, and what the distance does to colours.
pub struct Vista {
    /// Where the tree meets the crest, in that view.
    pub tree: (f32, f32),
    /// The crest's height at a point across that view.
    pub crest: fn(f32) -> f32,
    /// How many of the summit's pixels across make one of the view's.
    pub spread: f32,
    /// How many of a piece's pixels make one of the view's, each way.
    pub shrink: u32,
    pub tint: Tint,
}

pub enum Tint {
    /// Colours faded towards the distance's own, by this much.
    Haze(Rgba, f32),
    /// Dark against a bright sky.
    Silhouette(Rgba),
}

/// Where the summit's tree stands, across the summit.
const TREE_X: f32 = (TREE.0 + TREE.2) / 2.0;

/// Draws what stands on the Hilltop, small and far off, on another view's skyline, back row
/// first: so the Hill seen from below grows with what the colony has found.
pub fn skyline(scene: &mut Canvas, arrangement: &Arrangement, vista: &Vista) {
    let mut placed: Vec<(usize, &String)> = arrangement
        .iter()
        .map(|(spot, id)| (usize::from(*spot), id))
        .filter(|(spot, _)| *spot < SPOTS.len())
        .collect();
    placed.sort_by(|a, b| SPOTS[a.0].1.total_cmp(&SPOTS[b.0].1));
    for (spot, id) in placed {
        if finds::find(id).is_none() {
            continue;
        }
        let (x, y) = SPOTS[spot];
        let piece = art::piece(id);
        let small = shrink(&piece.sprite, vista.shrink, &vista.tint);
        let across = vista.tree.0 + (x - TREE_X) / vista.spread;
        // Nearer rows sit a little lower down the face of the Hill.
        let down = (y - SPOTS[0].1) / 40.0 * 1.5;
        let base = (vista.crest)(across) + 1.0 + down;
        let anchor = (
            piece.anchor.0 / vista.shrink as i32,
            piece.anchor.1 / vista.shrink as i32,
        );
        blit(
            scene,
            &small,
            across.round() as i32 - anchor.0,
            base.round() as i32 - anchor.1,
        );
    }
}

/// A sprite made smaller by `factor` each way: each block that is mostly there becomes one pixel
/// of its average colour, tinted for the distance.
fn shrink(sprite: &Canvas, factor: u32, tint: &Tint) -> Canvas {
    let factor = factor.max(1);
    let (width, height) = (
        sprite.width().div_ceil(factor),
        sprite.height().div_ceil(factor),
    );
    let mut small = Canvas::new(width, height);
    let block = (factor * factor) as usize;
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let mut sum = [0u32; 3];
            let mut opaque = 0;
            for dy in 0..factor as i32 {
                for dx in 0..factor as i32 {
                    let pixel = sprite.get(x * factor as i32 + dx, y * factor as i32 + dy);
                    if pixel.a >= 128 {
                        opaque += 1;
                        sum[0] += u32::from(pixel.r);
                        sum[1] += u32::from(pixel.g);
                        sum[2] += u32::from(pixel.b);
                    }
                }
            }
            if opaque * 3 < block {
                continue;
            }
            let average = Rgba::new(
                (sum[0] / opaque as u32) as u8,
                (sum[1] / opaque as u32) as u8,
                (sum[2] / opaque as u32) as u8,
                255,
            );
            let color = match tint {
                Tint::Haze(distance, amount) => mix(average, *distance, *amount),
                Tint::Silhouette(dark) => *dark,
            };
            small.set(x, y, color);
        }
    }
    small
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
    fn the_hilltop_shows_on_another_skyline_and_only_there() {
        let vista = Vista {
            tree: (200.0, 60.0),
            crest: |x| 60.0 + (x - 200.0).abs() * 0.1,
            spread: 6.0,
            shrink: 5,
            tint: Tint::Silhouette(Rgba::new(40, 30, 50, 255)),
        };
        let mut empty = Canvas::new(384, 216);
        skyline(&mut empty, &Arrangement::new(), &vista);
        assert!(empty.alpha_bounds().is_none());
        let mut grown = Canvas::new(384, 216);
        let arrangement =
            Arrangement::from([(0, "weathervane".to_owned()), (17, "geode".to_owned())]);
        skyline(&mut grown, &arrangement, &vista);
        let (left, top, right, bottom) = grown.alpha_bounds().expect("nothing showed");
        assert!(
            left >= 180 && right <= 280,
            "spread too wide: {left}..{right}"
        );
        assert!(
            top >= 40 && bottom <= 76,
            "not on the crest: {top}..{bottom}"
        );
    }

    #[test]
    fn clicking_near_a_spot_finds_it() {
        for (index, (x, y)) in SPOTS.iter().enumerate() {
            assert_eq!(spot_at(*x, *y - 10.0), Some(index as u8));
        }
        assert_eq!(spot_at(5.0, 5.0), None);
    }
}
