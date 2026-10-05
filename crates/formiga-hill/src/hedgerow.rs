//! The hedgerow at the Woods' edge, where the person goes foraging with a companion or two. A
//! fourth Woods activity: where rummaging is reading signs and catching a moment, fishing knowing
//! where each fish keeps, and bug catching stalking, foraging is reading ripeness. Everything on
//! the hedge and the bank ripens and goes over through the outing on its own rhythm, and the
//! person learns each one's look, works out a way round that arrives at each thing as it turns
//! ripe, and chooses what is worth a place in the basket.
//!
//! The place is a lane under a bank, the hedge along the top of the bank with a field gate in it
//! (and the Hill through the gate), a crab-apple tree standing up out of the hedge, and the edge
//! of the Woods on the right with mushrooms and wild garlic under the trees. Clover grows on the
//! near verge. Everything that can be picked grows at a slot in a patch; the scenery is painted
//! round the slots, and nothing that can be picked is painted into it.

pub mod foraging;
pub mod produce;
mod scenery;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::finds::{self, Find};
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Patch as Ground, Playground, Prop};

/// The plants of the hedgerow, each with the one find it gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Plant {
    Bramble,
    Strawberry,
    Hazel,
    Mushroom,
    WildGarlic,
    Elder,
    CrabApple,
    DogRose,
    Thyme,
    Chanterelle,
    Honeysuckle,
    Clover,
}

impl Plant {
    pub const ALL: [Self; 12] = [
        Self::Bramble,
        Self::Strawberry,
        Self::Hazel,
        Self::Mushroom,
        Self::WildGarlic,
        Self::Elder,
        Self::CrabApple,
        Self::DogRose,
        Self::Thyme,
        Self::Chanterelle,
        Self::Honeysuckle,
        Self::Clover,
    ];

    /// The find picked off it.
    pub fn find(self) -> &'static Find {
        let id = match self {
            Self::Bramble => "blackberries",
            Self::Strawberry => "strawberry_runner",
            Self::Hazel => "hazelnut",
            Self::Mushroom => "field_mushrooms",
            Self::WildGarlic => "wild_garlic",
            Self::Elder => "elderflower",
            Self::CrabApple => "crab_apple_pip",
            Self::DogRose => "rose_hip_seeds",
            Self::Thyme => "thyme_cutting",
            Self::Chanterelle => "golden_chanterelle",
            Self::Honeysuckle => "honeysuckle",
            Self::Clover => "four_leaf_clover",
        };
        finds::find(id).expect("every plant gives a foraged find")
    }

    /// The plant a foraged find grows on.
    pub fn of(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|plant| plant.find().id == id)
    }

    /// What the person calls what grows on it, as it hangs there.
    pub fn produce(self) -> &'static str {
        match self {
            Self::Bramble => "blackberries",
            Self::Strawberry => "wild strawberries",
            Self::Hazel => "hazelnuts",
            Self::Mushroom => "field mushrooms",
            Self::WildGarlic => "wild garlic",
            Self::Elder => "elderflower",
            Self::CrabApple => "crab apples",
            Self::DogRose => "rose hips",
            Self::Thyme => "wild thyme",
            Self::Chanterelle => "a chanterelle",
            Self::Honeysuckle => "honeysuckle",
            Self::Clover => "clover",
        }
    }

    /// When in the outing its produce turns ripe, as shares of the light gone, earliest to
    /// latest; and how long it stays ripe, as a share. Each plant keeps its own rhythm, which is
    /// a thing to learn: strawberries and garlic early, nuts and apples in the middle, hips late,
    /// and the rare things later and more briefly still. The four-leaf clover only shows itself
    /// in the last of the light.
    pub fn rhythm(self) -> ((f32, f32), f32) {
        match self {
            Self::Bramble => ((0.04, 0.72), 0.2),
            Self::Strawberry => ((0.02, 0.46), 0.18),
            Self::Hazel => ((0.2, 0.66), 0.2),
            // Mushrooms come up and go over quickly.
            Self::Mushroom => ((0.04, 0.66), 0.13),
            Self::WildGarlic => ((0.0, 0.42), 0.22),
            Self::Elder => ((0.08, 0.5), 0.11),
            Self::CrabApple => ((0.3, 0.7), 0.11),
            Self::DogRose => ((0.36, 0.74), 0.11),
            Self::Thyme => ((0.14, 0.6), 0.11),
            Self::Chanterelle => ((0.46, 0.72), 0.06),
            Self::Honeysuckle => ((0.56, 0.78), 0.06),
            Self::Clover => ((0.8, 0.9), 0.04),
        }
    }
}

/// How a slot is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// From the lane, by anyone.
    Within,
    /// Low and tucked in, under the leaves or deep in the bramble: only a little one gets a paw
    /// in.
    Tucked,
    /// Up in the high branches: someone bold climbs, or a close pair bends the branch down.
    High,
}

/// Somewhere one thing grows.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub plant: Plant,
    /// Where it hangs or stands.
    pub at: (f32, f32),
    pub reach: Reach,
    /// Under the leaves: not seen until someone looks closely, or someone curious spots it.
    pub hidden: bool,
}

/// A place along the hedgerow worth going to, and what grows there.
#[derive(Clone, Copy, Debug)]
pub struct Patch {
    pub name: &'static str,
    /// Where whoever picks there stands.
    pub stand: (f32, f32),
    /// The row whatever grows there stands on, for drawing everyone in front of it or behind.
    pub base: f32,
    /// Where something that falls comes to rest.
    pub floor: f32,
    pub slots: &'static [Slot],
}

const fn within(plant: Plant, x: f32, y: f32) -> Slot {
    Slot {
        plant,
        at: (x, y),
        reach: Reach::Within,
        hidden: false,
    }
}

const fn hidden(plant: Plant, x: f32, y: f32) -> Slot {
    Slot {
        plant,
        at: (x, y),
        reach: Reach::Within,
        hidden: true,
    }
}

const fn tucked(plant: Plant, x: f32, y: f32) -> Slot {
    Slot {
        plant,
        at: (x, y),
        reach: Reach::Tucked,
        hidden: false,
    }
}

const fn high(plant: Plant, x: f32, y: f32) -> Slot {
    Slot {
        plant,
        at: (x, y),
        reach: Reach::High,
        hidden: false,
    }
}

use Plant::*;

/// The hedgerow's patches, left to right along the lane, then the clover on the near verge. The
/// scenery is painted round these.
pub const PATCHES: [Patch; 9] = [
    Patch {
        name: "the bramble",
        stand: (46.0, 154.0),
        base: 142.0,
        floor: 143.0,
        slots: &[
            within(Bramble, 14.0, 98.0),
            within(Bramble, 32.0, 92.0),
            within(Bramble, 52.0, 104.0),
            within(Bramble, 22.0, 118.0),
            within(Bramble, 44.0, 124.0),
            tucked(Bramble, 30.0, 136.0),
            tucked(Bramble, 60.0, 138.0),
        ],
    },
    Patch {
        name: "the dog rose",
        stand: (88.0, 150.0),
        base: 112.0,
        floor: 116.0,
        slots: &[
            within(DogRose, 74.0, 96.0),
            within(DogRose, 88.0, 86.0),
            within(DogRose, 100.0, 100.0),
            within(DogRose, 84.0, 106.0),
            within(Honeysuckle, 96.0, 74.0),
            within(Honeysuckle, 106.0, 86.0),
            high(DogRose, 80.0, 64.0),
        ],
    },
    Patch {
        name: "the strawberry bank",
        stand: (124.0, 156.0),
        base: 143.0,
        floor: 143.0,
        slots: &[
            within(Strawberry, 106.0, 128.0),
            within(Strawberry, 118.0, 134.0),
            within(Strawberry, 132.0, 126.0),
            within(Strawberry, 144.0, 136.0),
            hidden(Strawberry, 126.0, 140.0),
            tucked(Strawberry, 112.0, 141.0),
            tucked(Strawberry, 150.0, 142.0),
        ],
    },
    Patch {
        name: "the elder",
        stand: (142.0, 150.0),
        base: 112.0,
        floor: 116.0,
        slots: &[
            within(Elder, 126.0, 86.0),
            within(Elder, 142.0, 78.0),
            within(Elder, 156.0, 94.0),
            high(Elder, 132.0, 58.0),
            high(Elder, 148.0, 50.0),
        ],
    },
    Patch {
        name: "the thyme",
        stand: (216.0, 152.0),
        base: 136.0,
        floor: 136.0,
        slots: &[
            within(Thyme, 206.0, 124.0),
            within(Thyme, 220.0, 128.0),
            within(Thyme, 232.0, 122.0),
        ],
    },
    Patch {
        name: "the hazel",
        stand: (246.0, 150.0),
        base: 112.0,
        floor: 116.0,
        slots: &[
            within(Hazel, 234.0, 90.0),
            within(Hazel, 248.0, 84.0),
            within(Hazel, 258.0, 98.0),
            hidden(Hazel, 242.0, 106.0),
            high(Hazel, 242.0, 60.0),
        ],
    },
    Patch {
        name: "the crab apple",
        stand: (286.0, 152.0),
        base: 112.0,
        floor: 118.0,
        slots: &[
            within(CrabApple, 262.0, 83.0),
            within(CrabApple, 282.0, 89.0),
            within(CrabApple, 304.0, 85.0),
            high(CrabApple, 276.0, 40.0),
            high(CrabApple, 292.0, 30.0),
            high(CrabApple, 306.0, 46.0),
        ],
    },
    Patch {
        name: "under the trees",
        stand: (326.0, 168.0),
        base: 150.0,
        floor: 150.0,
        slots: &[
            within(WildGarlic, 330.0, 132.0),
            within(WildGarlic, 348.0, 128.0),
            within(WildGarlic, 366.0, 136.0),
            within(Mushroom, 334.0, 182.0),
            within(Mushroom, 348.0, 176.0),
            within(Mushroom, 362.0, 184.0),
            within(Mushroom, 346.0, 191.0),
            hidden(Chanterelle, 376.0, 158.0),
        ],
    },
    Patch {
        name: "the clover",
        stand: (186.0, 196.0),
        base: 202.0,
        floor: 202.0,
        slots: &[hidden(Clover, 200.0, 200.0)],
    },
];

/// Every slot along the hedgerow, with the patch it is in.
pub fn slots() -> impl Iterator<Item = (usize, &'static Slot)> {
    PATCHES
        .iter()
        .enumerate()
        .flat_map(|(patch, place)| place.slots.iter().map(move |slot| (patch, slot)))
}

/// Where anyone can walk: the lane, the foot of the bank and the near verge, and the floor under
/// the trees on the right.
pub const GROUND: Ground = (14.0, 146.0, 372.0, 206.0);

pub fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = GROUND;
    // The bramble spills down onto the lane at the far left, and the mushroom ring is kept to.
    let in_bramble = x < 34.0 && y < 152.0;
    let in_ring = ((x - 348.0) / 22.0).powi(2) + ((y - 183.0) / 10.0).powi(2) < 1.0;
    x >= left && x <= right && y >= top && y <= bottom && !in_bramble && !in_ring
}

/// The lane's named spots, for free play between forays.
const PLACES: [(&str, (f32, f32)); 3] = [
    ("centre", (190.0, 168.0)),
    ("front", (190.0, 196.0)),
    ("left", (70.0, 176.0)),
];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Down the lane from the Woods, from the left.
        entrance: (-24.0, 172.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// Shows what stands on the Hilltop now, through the gate.
pub fn show_hilltop(ground: &mut Playground, hilltop: &Arrangement) {
    ground.set_backdrop(scenery::backdrop(hilltop));
}

/// The hedgerow, with the companions who came walking down the lane. After dark, glow-worms on
/// the bank, fireflies under the trees, the lantern on the gate lit, and the stars and the moon
/// over the hedge.
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

/// The hedgerow as it looks with nobody foraging: everything growing at some stage of ripeness,
/// for the place between forays. Nothing in it can be picked.
pub fn growing() -> Vec<Prop> {
    use produce::Stage;
    const STAGES: [Stage; 5] = [
        Stage::Unripe,
        Stage::Turning,
        Stage::Ripe,
        Stage::Unripe,
        Stage::Ripe,
    ];
    slots()
        .enumerate()
        .filter(|(_, (_, slot))| slot.reach == Reach::Within && !slot.hidden)
        .filter(|(_, (_, slot))| slot.plant.find().tier <= finds::Tier::Uncommon)
        .map(|(index, (patch, slot))| {
            let stage = STAGES[index % STAGES.len()];
            produce::prop(slot, PATCHES[patch].base, stage, 0.0, true)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playground::distance;

    #[test]
    fn every_patch_can_be_stood_at_and_reached() {
        for patch in &PATCHES {
            assert!(
                walkable(patch.stand.0, patch.stand.1),
                "nobody can stand at {}",
                patch.name
            );
            for slot in patch.slots {
                let reach = distance(slot.at, patch.stand);
                assert!(
                    reach < 130.0,
                    "{:?} at {:?} is too far from {}",
                    slot.plant,
                    slot.at,
                    patch.name
                );
            }
        }
    }

    #[test]
    fn every_plant_grows_somewhere_anyone_can_reach() {
        for plant in Plant::ALL {
            assert!(
                slots().any(|(_, slot)| slot.plant == plant && slot.reach == Reach::Within),
                "{plant:?} only grows where some company can reach"
            );
            assert_eq!(Plant::of(plant.find().id), Some(plant));
        }
        for find in &finds::FORAGED {
            assert!(Plant::of(find.id).is_some(), "{} grows nowhere", find.id);
        }
    }

    #[test]
    fn the_rarer_a_plant_the_later_and_briefer_it_is_ripe() {
        for plant in Plant::ALL {
            let ((early, late), window) = plant.rhythm();
            assert!(early <= late && late + window <= 1.0, "{plant:?}");
            let tier = plant.find().tier;
            if tier >= finds::Tier::Rare {
                assert!(early >= 0.45, "{plant:?} is ripe too early");
                assert!(window <= 0.06, "{plant:?} is ripe too long");
            }
        }
        let ((clover, _), _) = Clover.rhythm();
        assert!(
            clover >= 0.8,
            "the clover only shows in the last of the light"
        );
    }

    #[test]
    fn patches_are_told_apart_by_where_they_grow() {
        // Every slot is nearer its own patch's other slots than any other patch's stand, so a
        // click on one goes to the right place.
        for (index, patch) in PATCHES.iter().enumerate() {
            for (other, there) in PATCHES.iter().enumerate() {
                if other != index {
                    assert!(
                        distance(patch.stand, there.stand) >= 14.0,
                        "{} and {} stand on top of each other",
                        patch.name,
                        there.name
                    );
                }
            }
        }
    }
}
