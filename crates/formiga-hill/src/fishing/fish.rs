//! The fish in the pool: Hill's own catalogue of them, and which are about on a given day.
//!
//! As with finds, each kind leans towards some sorts of companion without ruling anyone out, kinds
//! not yet caught are about more often, and after a couple of trips without a new kind, one is
//! certainly in the pool: whoever comes along, every fish can be caught in the end.

use super::Haunt;
use crate::character::Character;
use crate::dice::Dice;
use crate::finds::{Leaning, NOVELTY, Tier, drawn_to};

pub struct Fish {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub tier: Tier,
    pub leanings: &'static [Leaning],
    /// Where in the pool it keeps.
    pub haunt: Haunt,
    /// How long it grows, in centimetres, smallest to largest.
    pub length: (f32, f32),
    /// How fast it cruises, in pixels a second.
    pub speed: f32,
    /// How close a splash can land before it bolts, in pixels; nothing for the bold.
    pub wary: f32,
    /// How many times it nibbles before it bites, fewest to most.
    pub nibbles: (u8, u8),
    /// How long the bite lasts: the moment to strike.
    pub bite: f32,
    /// How hard it pulls once hooked, from 0 to 1.
    pub strength: f32,
    /// Only about as the light goes.
    pub dusk: bool,
}

use Haunt::*;
use Leaning::*;
use Tier::*;

pub const CATALOGUE: [Fish; 8] = [
    Fish {
        id: "minnow",
        name: "A minnow",
        blurb: "Quick as a thought and about as big. They come in whole flickering clouds.",
        tier: Common,
        leanings: &[Tiny],
        haunt: Shallows,
        length: (3.0, 7.0),
        speed: 34.0,
        wary: 0.0,
        nibbles: (0, 1),
        bite: 0.6,
        strength: 0.1,
        dusk: false,
    },
    Fish {
        id: "roach",
        name: "A roach",
        blurb: "Silver, with red fins, and in no hurry about anything.",
        tier: Common,
        leanings: &[Cosy],
        haunt: Lilies,
        length: (10.0, 25.0),
        speed: 18.0,
        wary: 10.0,
        nibbles: (1, 2),
        bite: 0.55,
        strength: 0.3,
        dusk: false,
    },
    Fish {
        id: "perch",
        name: "A perch",
        blurb: "Striped like a tiger and spiny along the back. Always nibbles twice.",
        tier: Common,
        leanings: &[Wild],
        haunt: Reeds,
        length: (12.0, 32.0),
        speed: 20.0,
        wary: 8.0,
        nibbles: (2, 2),
        bite: 0.5,
        strength: 0.35,
        dusk: false,
    },
    Fish {
        id: "trout",
        name: "A brown trout",
        blurb: "Speckled, and wary of any splash. Takes the bait at once or not at all.",
        tier: Uncommon,
        leanings: &[Wild, Shiny],
        haunt: Falls,
        length: (20.0, 48.0),
        speed: 28.0,
        wary: 30.0,
        nibbles: (0, 0),
        bite: 0.38,
        strength: 0.55,
        dusk: false,
    },
    Fish {
        id: "carp",
        name: "A carp",
        blurb: "Big and bronze and patient. It mouths the bait for ages before it decides.",
        tier: Uncommon,
        leanings: &[Cosy, Old],
        haunt: Lilies,
        length: (30.0, 72.0),
        speed: 10.0,
        wary: 14.0,
        nibbles: (2, 4),
        bite: 0.7,
        strength: 0.75,
        dusk: false,
    },
    Fish {
        id: "pike",
        name: "A pike",
        blurb: "Waits in the reeds without moving a fin, then comes like an arrow.",
        tier: Rare,
        leanings: &[Shiny, Wild],
        haunt: Reeds,
        length: (45.0, 95.0),
        speed: 16.0,
        wary: 0.0,
        nibbles: (0, 0),
        bite: 0.34,
        strength: 0.85,
        dusk: false,
    },
    Fish {
        id: "golden_tench",
        name: "A golden tench",
        blurb: "Like an ordinary tench that somebody gilded. Said to bring luck.",
        tier: Rare,
        leanings: &[Shiny, Odd],
        haunt: Deep,
        length: (25.0, 52.0),
        speed: 12.0,
        wary: 18.0,
        nibbles: (1, 3),
        bite: 0.5,
        strength: 0.6,
        dusk: false,
    },
    Fish {
        id: "old_one",
        name: "The Old One",
        blurb: "Nobody knows how old. Only comes up from the deep as the light goes.",
        tier: Exceptional,
        leanings: &[Old, Odd],
        haunt: Deep,
        length: (95.0, 125.0),
        speed: 8.0,
        wary: 22.0,
        nibbles: (3, 3),
        bite: 0.45,
        strength: 1.0,
        dusk: true,
    },
];

pub fn fish(id: &str) -> Option<&'static Fish> {
    CATALOGUE.iter().find(|fish| fish.id == id)
}

/// How many fish are about on a trip.
pub const ABOUT: usize = 8;
/// After this many trips in a row without a new kind of fish, a new kind is certainly about.
pub const DROUGHT: u32 = 2;

/// Which fish are about in the pool on one trip. `caught_before` says whether the colony has
/// caught a kind already; `drought` is how many trips in a row have caught nothing new.
pub fn about(
    party: &[&Character],
    caught_before: impl Fn(&str) -> bool,
    drought: u32,
    dice: &mut Dice,
) -> Vec<&'static Fish> {
    let weight = |fish: &Fish| {
        let novelty = if caught_before(fish.id) { 1.0 } else { NOVELTY };
        fish.tier.weight() * drawn_to(fish.leanings, party) * novelty
    };
    let total: f32 = CATALOGUE.iter().map(weight).sum();
    let mut about = Vec::with_capacity(ABOUT + 1);
    for _ in 0..ABOUT {
        let mut roll = dice.range(0.0, total);
        let mut chosen = &CATALOGUE[0];
        for fish in &CATALOGUE {
            roll -= weight(fish);
            if roll <= 0.0 {
                chosen = fish;
                break;
            }
        }
        about.push(chosen);
    }
    // A long run of nothing new: a new kind is certainly about, the least rare first.
    let uncaught: Vec<&'static Fish> = CATALOGUE
        .iter()
        .filter(|fish| !caught_before(fish.id))
        .collect();
    if drought >= DROUGHT
        && let Some(least) = uncaught.iter().map(|fish| fish.tier).min()
        && !about
            .iter()
            .any(|fish| !caught_before(fish.id) && fish.tier == least)
    {
        let candidates: Vec<&'static Fish> = uncaught
            .into_iter()
            .filter(|fish| fish.tier == least)
            .collect();
        let pick = (dice.unit() * candidates.len() as f32) as usize % candidates.len();
        about.push(candidates[pick]);
    }
    about
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use std::collections::BTreeSet;

    fn characters() -> Vec<Character> {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        cast.members.iter().map(Character::of).collect()
    }

    #[test]
    fn the_catalogue_is_well_formed() {
        let ids: BTreeSet<&str> = CATALOGUE.iter().map(|fish| fish.id).collect();
        assert_eq!(ids.len(), CATALOGUE.len());
        for haunt in Haunt::ALL {
            assert!(
                CATALOGUE.iter().any(|fish| fish.haunt == haunt),
                "nothing lives in {}",
                haunt.name()
            );
        }
        for fish in &CATALOGUE {
            assert!(fish.length.0 <= fish.length.1 && fish.nibbles.0 <= fish.nibbles.1);
            assert!((0.0..=1.0).contains(&fish.strength) && fish.bite > 0.2);
        }
    }

    #[test]
    fn anyone_on_their_own_catches_every_kind_in_the_end() {
        for character in characters() {
            let mut dice = Dice::new(character.axes.playfulness.to_bits() as u64);
            let mut caught: BTreeSet<&str> = BTreeSet::new();
            let mut drought = 0;
            let mut trips = 0;
            while caught.len() < CATALOGUE.len() {
                trips += 1;
                assert!(
                    trips <= 80,
                    "{:?} after {trips} trips caught only {caught:?}",
                    character.kind
                );
                let pool = about(&[&character], |id| caught.contains(id), drought, &mut dice);
                // A middling trip: three fish landed, a new kind if there's one about.
                let mut new = false;
                let landed: Vec<&str> = pool
                    .iter()
                    .filter(|fish| !caught.contains(fish.id))
                    .take(1)
                    .chain(pool.iter().take(2))
                    .map(|fish| fish.id)
                    .collect();
                for id in landed {
                    new |= caught.insert(id);
                }
                drought = if new { 0 } else { drought + 1 };
            }
        }
    }
}
