//! The old track, deeper in the Woods: an overgrown cart track where things were left behind long
//! ago. A fifth Woods activity, and a sixth: where rummaging is reading signs and catching a
//! moment, fishing reading bites, bug catching creeping and swinging, and foraging reading
//! ripeness, scavenging is reading a heap and lifting things in the right order; and a torn map
//! turned up in a heap leads off along the track on a treasure hunt.
//!
//! The place is the track itself, running away between the trees to a gap where the Hill shows
//! far off, with a woodcutter's tumbledown hut on the left and its woodshed fallen in, a dry-stone
//! wall fallen in along the right, a broken cart that never got any further, and a mossy
//! milestone. Three heaps lie where things came down: the woodshed's, the wall's, and the cart's
//! spilled load. The scenery is painted round where they lie, and the heaps are drawn as they
//! stand on each outing (see `heap`).

pub mod crew;
pub mod heap;
mod landmarks;
pub mod routes;
pub mod scavenging;
mod scenery;
pub mod stuff;
pub mod treasure;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::dice::Dice;
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Patch, Playground, Prop};
use heap::{Heap, Recipe, Stuff};

/// Somewhere a heap lies.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub name: &'static str,
    /// The heap's left end and the row it stands on, and how wide it can spread.
    pub left: i32,
    pub ground: i32,
    pub width: i32,
    /// Where whoever works it stands, beside it.
    pub stand: (f32, f32),
    pub recipe: Recipe,
}

use Stuff::*;

/// The old track's three heaps, back to front. The scenery is painted round these.
pub const SITES: [Site; 3] = [
    Site {
        name: "the fallen wall",
        left: 268,
        ground: 140,
        width: 76,
        stand: (254.0, 146.0),
        recipe: Recipe {
            ground: (&[Stone, Stone, Boulder, Stone, Crate], (3, 3)),
            across: (&[Plank, Beam, Plank], (1, 2)),
            top: (&[Stone, Stone, Tin, Sack, Stone], (2, 4)),
        },
    },
    Site {
        name: "the woodshed",
        left: 66,
        ground: 156,
        width: 78,
        stand: (160.0, 162.0),
        recipe: Recipe {
            ground: (&[Crate, Stone, Sack, Boulder, Crate], (2, 3)),
            across: (&[Plank, Plank, Beam], (1, 2)),
            top: (&[Tin, Plank, Sack, Stone, Tin, Crate], (2, 4)),
        },
    },
    Site {
        name: "the cart",
        left: 224,
        ground: 198,
        width: 78,
        stand: (208.0, 203.0),
        recipe: Recipe {
            ground: (&[Sack, Crate, Sack, Crate, Stone], (2, 3)),
            across: (&[Plank, Plank, Beam], (1, 2)),
            top: (&[Tin, Sack, Tin, Crate, Sack], (2, 4)),
        },
    },
];

/// The heap at the end of a wrong turn, in a nook off the track.
pub const NOOK: Site = Site {
    name: "the heap",
    left: 154,
    ground: 178,
    width: 76,
    stand: (140.0, 184.0),
    recipe: Recipe {
        ground: (&[Crate, Stone, Sack, Stone, Crate], (3, 3)),
        across: (&[Plank, Beam, Plank], (1, 2)),
        top: (&[Tin, Stone, Sack, Tin], (2, 3)),
    },
};

/// Where anyone can walk: the track and its verges, from the hut to the cart.
pub const GROUND: Patch = (12.0, 138.0, 372.0, 210.0);

/// Whether someone can stand here: on the ground, clear of the heaps, the hut, the cart and the
/// wall.
pub fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = GROUND;
    if x < left || x > right || y < top || y > bottom {
        return false;
    }
    let in_heap = SITES.iter().any(|site| {
        x >= (site.left - 2) as f32
            && x <= (site.left + site.width + 2) as f32
            && y >= (site.ground - 10) as f32
            && y <= (site.ground + 3) as f32
    });
    let in_hut = x < 66.0 && y < 156.0;
    let in_cart = x > 304.0 && y > 158.0;
    let behind_wall = x > 344.0 && y < 152.0;
    !(in_heap || in_hut || in_cart || behind_wall)
}

/// The track's named spots, for free play between outings.
const PLACES: [(&str, (f32, f32)); 3] = [
    ("centre", (196.0, 172.0)),
    ("front", (176.0, 202.0)),
    ("left", (56.0, 194.0)),
];

/// Where anyone might stand between outings, kept clear in the scenery.
const PLACES_TO_STAND: [(f32, f32); 3] = [PLACES[0].1, PLACES[1].1, PLACES[2].1];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Up the track from the glade, from the front left.
        entrance: (-24.0, 200.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// The old track, with the companions who came walking up it, and the Hill far off through the
/// gap in the trees. After dark, glow-worms along the wall, fireflies under the trees, and the
/// stars and the moon over the gap.
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

/// Shows what stands on the Hilltop now, through the gap at the end of the track.
pub fn show_hilltop(ground: &mut Playground, hilltop: &Arrangement) {
    ground.set_backdrop(scenery::backdrop(hilltop));
}

/// Where anyone can walk on a treasure hunt: the ground at a fork, in a nook, or at the dig.
pub const HUNT_GROUND: Patch = (8.0, 112.0, 376.0, 210.0);

fn hunt_walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = HUNT_GROUND;
    x >= left && x <= right && y >= top && y <= bottom
}

const HUNT_PLACES: [(&str, (f32, f32)); 1] = [("centre", treasure::START)];

pub fn hunt_layout() -> Layout {
    Layout {
        walkable: hunt_walkable,
        ground: HUNT_GROUND,
        spots: &HUNT_PLACES,
        seats: None,
        shade: None,
        // Up the way from the front.
        entrance: (192.0, 236.0),
        entrance_step: (14.0, 4.0),
    }
}

/// The ground a treasure hunt is played on, with the companions who came: the hunt shows each
/// fork, nook and the dig on it in turn. After dark, fireflies under the trees.
pub fn hunt_ground(cast: &Cast, party: &[Id], now: f32) -> Playground {
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        hunt_layout(),
        scenery::nook(),
        scenery::foreground(),
        Vec::new(),
    );
    ground.set_nightlights(Nightlights {
        lamps: scenery::hunt_lamplight(),
        sky: formiga_art::Canvas::new(1, 1),
        indoors: false,
    });
    ground
}

/// Whether a torn map turns up somewhere else in the Woods than the old track: rolled up in a
/// hollow with whatever a rummage found there, about one rummage in twelve, the same each time
/// for the same colony and outing.
pub fn stray_map(outings: u32, colony: &str) -> bool {
    let hash = colony.bytes().fold(
        0x7a3c_u64 ^ u64::from(outings).wrapping_mul(0x9e37_79b9),
        |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3),
    );
    Dice::new(hash).chance(1.0 / 12.0)
}

/// Every landmark a map can name, near and far off, and every kind of thing a heap is made of,
/// on one sheet for review.
pub fn landmarks_sheet() -> formiga_art::Canvas {
    landmarks::sheet()
}

/// The heaps as they lie with nobody scavenging, for the place between outings.
pub fn at_rest() -> Vec<Prop> {
    let mut dice = Dice::new(0x01d_7ac);
    SITES
        .iter()
        .map(|site| {
            let heap = Heap::build(&site.recipe, site.width, &mut dice);
            stuff::heap_prop(&heap, site, &stuff::Motion::default(), 0.0, false)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::character::Character;
    use crate::finds::{self, CATALOGUE, FORAGED, RELICS, SCAVENGED, Use};
    use heap::{Hoard, Place, Stocking};
    use std::collections::BTreeSet;

    /// The completeness guarantee: with only ever one companion, keep going and everything the
    /// old track gives is found in the end, whoever that companion is, getting only at what its
    /// own paws can (nothing under a beam or a boulder unless it is strong, or small enough to
    /// squeeze under), and following every map it finds.
    #[test]
    fn anyone_on_their_own_finds_everything_on_the_old_track_in_the_end() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for member in &cast.members {
            let character = Character::of(member);
            let mut dice = Dice::new(member.id);
            let mut found: BTreeSet<&str> = BTreeSet::new();
            let (mut drought, mut chest_drought, mut maps) = (0, 0, 0usize);
            let mut outings = 0;
            while found.len() < SCAVENGED.len() {
                outings += 1;
                assert!(
                    outings <= 60,
                    "{} after {outings} outings had found only {found:?}",
                    member.name
                );
                if maps > 0 {
                    // A careful reader gets to the chest.
                    maps -= 1;
                    let chest = treasure::fill_chest(
                        &[&character],
                        &|id| found.contains(id),
                        chest_drought,
                        &mut dice,
                    );
                    let mut new = false;
                    for find in chest {
                        new |= found.insert(find.id);
                    }
                    chest_drought = if new { 0 } else { chest_drought + 1 };
                    continue;
                }
                let mut heaps: Vec<Heap> = SITES
                    .iter()
                    .map(|site| Heap::build(&site.recipe, site.width, &mut dice))
                    .collect();
                let stocking = Stocking {
                    party: &[&character],
                    found_before: &|id| found.contains(id),
                    drought,
                    map: scavenging::map_chance(maps),
                    inspector: crew::inspector(&character),
                };
                heap::stock(&mut heaps, &stocking, &mut dice);
                // A middling scavenge: a basketful of what it can get at, the new things first.
                let mut picked: Vec<&str> = Vec::new();
                for heap in &heaps {
                    for hidden in &heap.hidden {
                        let reachable = match hidden.place {
                            Place::Under(at) if heap.items[at].stuff.heavy() => {
                                crew::strong(&character)
                                    || (crew::little(&character) && heap.gap(at))
                            }
                            _ => true,
                        };
                        match hidden.what {
                            _ if !reachable => {}
                            Hoard::Map => maps += 1,
                            Hoard::Find(find) => picked.push(find.id),
                        }
                    }
                }
                picked.sort_by_key(|id| found.contains(id));
                let mut new = false;
                for id in picked.into_iter().take(scavenging::BASKET) {
                    new |= found.insert(id);
                }
                drought = if new { 0 } else { drought + 1 };
            }
        }
    }

    #[test]
    fn nothing_from_the_old_track_turns_up_anywhere_else_and_nothing_else_turns_up_here() {
        let all: Vec<&finds::Find> = CATALOGUE
            .iter()
            .chain(&FORAGED)
            .chain(&SCAVENGED)
            .chain(&RELICS)
            .collect();
        let ids: BTreeSet<&str> = all.iter().map(|find| find.id).collect();
        assert_eq!(ids.len(), all.len(), "an id is used twice");
        for find in &SCAVENGED {
            assert!(finds::is_scavenged(find.id));
            assert_eq!(finds::find(find.id).map(|f| f.id), Some(find.id));
            assert!(!finds::is_foraged(find.id) && !finds::is_relic(find.id));
            assert!(crate::hedgerow::Plant::of(find.id).is_none());
            assert!(find.blurb.len() < 120, "{}'s blurb is too long", find.id);
            assert!(!find.leanings.is_empty(), "{} leans nowhere", find.id);
            // A sky-gazing piece is part of the secret's trigger, and nothing here is one.
            assert_ne!(find.use_, Use::Gaze, "{} would gaze at the sky", find.id);
        }
        let tiers: BTreeSet<finds::Tier> = SCAVENGED.iter().map(|find| find.tier).collect();
        assert_eq!(tiers.len(), 4, "something of every rarity");
        // Rummaging never turns up anything from here.
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let characters: Vec<Character> = cast.members.iter().map(Character::of).collect();
        let party: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(19);
        let kinds = [
            finds::Kind::Dig,
            finds::Kind::Reach,
            finds::Kind::Scoop,
            finds::Kind::Shake,
        ];
        for _ in 0..300 {
            let stocked = finds::stock(&kinds, &party, |_| false, 3, &[0.2; 4], &mut dice);
            for find in stocked.iter().flatten() {
                assert!(
                    !finds::is_scavenged(find.id),
                    "{} turned up rummaging",
                    find.id
                );
            }
        }
        // And heaps and chests hold only what is from here.
        for _ in 0..200 {
            let mut heaps: Vec<Heap> = SITES
                .iter()
                .map(|site| Heap::build(&site.recipe, site.width, &mut dice))
                .collect();
            let stocking = Stocking {
                party: &party,
                found_before: &|_| false,
                drought: 3,
                map: 0.3,
                inspector: false,
            };
            heap::stock(&mut heaps, &stocking, &mut dice);
            for hidden in heaps.iter().flat_map(|heap| &heap.hidden) {
                if let Hoard::Find(find) = hidden.what {
                    assert!(finds::is_scavenged(find.id), "{} in a heap", find.id);
                }
            }
            for find in treasure::fill_chest(&party, &|_| false, 0, &mut dice) {
                assert!(finds::is_scavenged(find.id), "{} in a chest", find.id);
            }
        }
    }

    #[test]
    fn everyone_can_stand_beside_every_heap_and_never_in_one() {
        for site in SITES.iter().chain([&NOOK]) {
            if site.name != NOOK.name {
                assert!(
                    walkable(site.stand.0, site.stand.1),
                    "nobody can stand at {}",
                    site.name
                );
                assert!(
                    !walkable((site.left + site.width / 2) as f32, site.ground as f32),
                    "{} can be walked through",
                    site.name
                );
            }
            // Beside it, not in front of it, so nobody stands over what is being lifted.
            let middle = (site.left + site.width / 2) as f32;
            assert!(
                (site.stand.0 - middle).abs() > (site.width / 2) as f32 - 4.0,
                "{} is worked from in front",
                site.name
            );
        }
        for (_, (x, y)) in PLACES {
            assert!(walkable(x, y));
        }
    }
}
