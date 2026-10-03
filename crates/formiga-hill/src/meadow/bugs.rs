//! The bugs of the meadow: Hill's own catalogue of them, and which are out on a given outing.
//!
//! As with finds and fish, each kind leans towards some sorts of companion without ruling anyone
//! out, kinds not yet caught are out more often, and after a couple of outings without a new
//! kind, one is certainly out: whoever comes along, every bug can be caught in the end.

use super::Haunt;
use crate::character::Character;
use crate::dice::Dice;
use crate::finds::{Leaning, NOVELTY, Tier, drawn_to};

/// How a kind gets about, which is what the person learns to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// Flutters from perch to perch on a wandering path and settles a while, wings opening and
    /// closing. Caught settled, wings shut.
    Flutter,
    /// Buzzes straight from flower to flower, never staying long. Caught the moment it lands.
    Buzz,
    /// Crawls slowly from perch to perch, and flies off only when startled. Easy to reach, easy
    /// to frighten.
    Crawl,
    /// Sits still in the grass and chirps, then hops in an arc to another tip: caught after it
    /// lands, before it chirps again.
    Hop,
    /// Darts between perches over the water and hangs in the air at each: caught while it hovers.
    Dart,
    /// Drifts low over the grass, lighting up and going dark: only to be caught while it is lit.
    Blink,
}

#[derive(Debug)]
pub struct Bug {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub tier: Tier,
    pub leanings: &'static [Leaning],
    /// Where in the meadow it keeps.
    pub haunt: Haunt,
    pub way: Way,
    /// How far off it notices someone moving, in pixels.
    pub notice: f32,
    /// How fast its nerve goes when someone moves that close, per second, at a walk.
    pub skittish: f32,
    /// How long it stays put at a perch, shortest to longest, in seconds.
    pub settles: (f32, f32),
    /// How fast it goes from perch to perch, in pixels a second.
    pub speed: f32,
    /// How big it grows, in millimetres across, smallest to largest.
    pub size: (f32, f32),
    /// Only out as the light goes.
    pub dusk: bool,
}

use Haunt::*;
use Leaning::*;
use Tier::*;
use Way::*;

pub const CATALOGUE: [Bug; 11] = [
    Bug {
        id: "cabbage_white",
        name: "A cabbage white",
        blurb: "White as washing on a line, with a dab of black at each wingtip.",
        tier: Common,
        leanings: &[Growing],
        haunt: Flowers,
        way: Flutter,
        notice: 48.0,
        skittish: 1.2,
        settles: (2.0, 5.0),
        speed: 18.0,
        size: (45.0, 60.0),
        dusk: false,
    },
    Bug {
        id: "ladybird",
        name: "A ladybird",
        blurb: "Seven spots, or six, or nine. Nobody has ever counted the same one twice.",
        tier: Common,
        leanings: &[Tiny, Cosy],
        haunt: Bramble,
        way: Crawl,
        notice: 30.0,
        skittish: 0.9,
        settles: (3.0, 7.0),
        speed: 4.0,
        size: (5.0, 8.0),
        dusk: false,
    },
    Bug {
        id: "grasshopper",
        name: "A grasshopper",
        blurb: "Sings by rubbing its legs together, and stops the moment anyone listens.",
        tier: Common,
        leanings: &[Wild],
        haunt: Grass,
        way: Hop,
        notice: 44.0,
        skittish: 1.5,
        settles: (2.0, 4.0),
        speed: 40.0,
        size: (15.0, 25.0),
        dusk: false,
    },
    Bug {
        id: "bumblebee",
        name: "A bumblebee",
        blurb: "Too round to fly, by all accounts. Nobody has told the bumblebee.",
        tier: Common,
        leanings: &[Growing, Cosy],
        haunt: Flowers,
        way: Buzz,
        notice: 34.0,
        skittish: 0.9,
        settles: (1.0, 2.2),
        speed: 26.0,
        size: (12.0, 22.0),
        dusk: false,
    },
    Bug {
        id: "red_admiral",
        name: "A red admiral",
        blurb: "Velvet black with a red stripe, basking on the warm log as if it owns it.",
        tier: Uncommon,
        leanings: &[Shiny],
        haunt: Log,
        way: Flutter,
        notice: 58.0,
        skittish: 1.65,
        settles: (3.0, 6.0),
        speed: 24.0,
        size: (55.0, 65.0),
        dusk: false,
    },
    Bug {
        id: "damselfly",
        name: "A damselfly",
        blurb: "A needle of bright blue that folds its wings along its back to rest.",
        tier: Uncommon,
        leanings: &[Shiny, Wild],
        haunt: Reeds,
        way: Dart,
        notice: 46.0,
        skittish: 1.35,
        settles: (1.5, 3.0),
        speed: 30.0,
        size: (30.0, 40.0),
        dusk: false,
    },
    Bug {
        id: "rose_chafer",
        name: "A rose chafer",
        blurb: "A beetle polished green and gold, asleep in a flower more often than not.",
        tier: Uncommon,
        leanings: &[Shiny, Growing],
        haunt: Flowers,
        way: Crawl,
        notice: 28.0,
        skittish: 0.75,
        settles: (3.0, 8.0),
        speed: 3.0,
        size: (14.0, 20.0),
        dusk: false,
    },
    Bug {
        id: "firefly",
        name: "A firefly",
        blurb: "A small brown beetle by day. Nobody would look twice, until the light goes.",
        tier: Uncommon,
        leanings: &[Odd, Cosy],
        haunt: Grass,
        way: Blink,
        notice: 36.0,
        skittish: 0.9,
        settles: (1.0, 2.0),
        speed: 10.0,
        size: (10.0, 15.0),
        dusk: true,
    },
    Bug {
        id: "stag_beetle",
        name: "A stag beetle",
        blurb: "Antlers like a stag's, and in no hurry. It has been coming to this stump for years.",
        tier: Rare,
        leanings: &[Old, Wild],
        haunt: Stump,
        way: Crawl,
        notice: 32.0,
        skittish: 0.75,
        settles: (4.0, 9.0),
        speed: 3.0,
        size: (35.0, 75.0),
        dusk: true,
    },
    Bug {
        id: "emperor_dragonfly",
        name: "An emperor dragonfly",
        blurb: "Blue and green and fast as a thrown stone, and it sees everything coming.",
        tier: Rare,
        leanings: &[Wild, Shiny],
        haunt: Reeds,
        way: Dart,
        notice: 72.0,
        skittish: 2.4,
        settles: (0.8, 1.6),
        speed: 70.0,
        size: (70.0, 80.0),
        dusk: false,
    },
    Bug {
        id: "moon_moth",
        name: "A moon moth",
        blurb: "Pale green, with long trailing tails. It only comes out of the brambles at dusk.",
        tier: Exceptional,
        leanings: &[Odd],
        haunt: Bramble,
        way: Flutter,
        notice: 56.0,
        skittish: 1.8,
        settles: (2.0, 4.0),
        speed: 14.0,
        size: (100.0, 120.0),
        dusk: true,
    },
];

/// How many bugs are out at once on an outing, before any made certain.
const OUT: usize = 6;
/// After this many outings with nothing new, a new kind is certainly out.
const DROUGHT: u32 = 2;

pub fn bug(id: &str) -> Option<&'static Bug> {
    CATALOGUE.iter().find(|bug| bug.id == id)
}

/// The bugs out on this outing, by their leanings towards the party, new kinds more often, and a
/// new kind certainly among them after a dry spell.
pub fn out(
    party: &[&Character],
    caught_before: impl Fn(&str) -> bool,
    drought: u32,
    dice: &mut Dice,
) -> Vec<&'static Bug> {
    let weight = |bug: &Bug| {
        let novelty = if caught_before(bug.id) { 1.0 } else { NOVELTY };
        bug.tier.weight() * drawn_to(bug.leanings, party) * novelty
    };
    let total: f32 = CATALOGUE.iter().map(weight).sum();
    let mut out = Vec::with_capacity(OUT + 1);
    for _ in 0..OUT {
        let mut roll = dice.range(0.0, total);
        let mut chosen = &CATALOGUE[0];
        for bug in &CATALOGUE {
            roll -= weight(bug);
            if roll <= 0.0 {
                chosen = bug;
                break;
            }
        }
        out.push(chosen);
    }
    let uncaught: Vec<&'static Bug> = CATALOGUE
        .iter()
        .filter(|bug| !caught_before(bug.id))
        .collect();
    if drought >= DROUGHT
        && let Some(least) = uncaught.iter().map(|bug| bug.tier).min()
        && !out
            .iter()
            .any(|bug| !caught_before(bug.id) && bug.tier == least)
    {
        let candidates: Vec<&'static Bug> = uncaught
            .into_iter()
            .filter(|bug| bug.tier == least)
            .collect();
        let pick = (dice.unit() * candidates.len() as f32) as usize % candidates.len();
        out.push(candidates[pick]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use std::collections::HashSet;

    #[test]
    fn every_bug_has_a_haunt_with_somewhere_to_settle() {
        for bug in &CATALOGUE {
            assert!(
                !bug.haunt.perches().is_empty(),
                "{} has nowhere to go",
                bug.id
            );
            assert!(bug.settles.0 > 0.0 && bug.settles.0 <= bug.settles.1);
            assert!(bug.size.0 > 0.0 && bug.size.0 <= bug.size.1);
        }
        let ids: HashSet<&str> = CATALOGUE.iter().map(|bug| bug.id).collect();
        assert_eq!(ids.len(), CATALOGUE.len());
    }

    /// The completeness guarantee: with only ever one companion, keep going and every kind is
    /// caught, whoever that companion is.
    #[test]
    fn any_one_companion_meets_every_bug_in_the_end() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for member in &cast.members {
            let character = Character::of(member);
            let mut caught: HashSet<&str> = HashSet::new();
            let mut drought = 0;
            let mut dice = Dice::new(member.id);
            for _ in 0..200 {
                let out = out(&[&character], |id| caught.contains(id), drought, &mut dice);
                // Only what is out can be caught, and only the first of each counts.
                let before = caught.len();
                if let Some(bug) = out.iter().find(|bug| !caught.contains(bug.id)) {
                    caught.insert(bug.id);
                }
                drought = if caught.len() > before {
                    0
                } else {
                    drought + 1
                };
                if caught.len() == CATALOGUE.len() {
                    break;
                }
            }
            assert_eq!(
                caught.len(),
                CATALOGUE.len(),
                "{} never met every bug",
                member.name
            );
        }
    }
}
