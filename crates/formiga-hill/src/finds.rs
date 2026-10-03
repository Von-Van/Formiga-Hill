//! Everything the Woods can turn up, and what each becomes on the Hilltop.
//!
//! Finds are Hill's own: a catalogue, not a drop table a package can add to (yet). Each belongs to
//! one kind of searching spot, has a rarity, and leans towards certain sorts of companion: a bold
//! show-off turns up shiny things more often, a scholar old things, a little one small things.
//! Leaning only weights the odds and never rules anything out, so whoever comes along, everything
//! can turn up. Undiscovered finds come up more often than ones already found, and after a couple
//! of outings without anything new, the Woods makes sure something new is out there somewhere: a
//! colony that only ever brings the same companion still finds everything in the end.

pub mod art;

use crate::character::Character;
use crate::dice::Dice;
use formiga_core::TemperamentKind;

/// The four ways of searching a spot in the Woods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// Soil: a bank, a ring of mushrooms, a tangle of roots.
    Dig,
    /// Hollows: a tree, a log, a burrow.
    Reach,
    /// Water: the shallows, a pool, the reeds.
    Scoop,
    /// Undergrowth: ferns, brambles, a low bush.
    Shake,
}

impl Kind {
    pub const ALL: [Self; 4] = [Self::Dig, Self::Reach, Self::Scoop, Self::Shake];

    /// Its place in `ALL`, for anything kept per kind.
    pub fn index(self) -> usize {
        match self {
            Self::Dig => 0,
            Self::Reach => 1,
            Self::Scoop => 2,
            Self::Shake => 3,
        }
    }

    /// What a companion does there, for the person to read: "dig", "reach in".
    pub fn verb(self) -> &'static str {
        match self {
            Self::Dig => "dig",
            Self::Reach => "reach in",
            Self::Scoop => "scoop",
            Self::Shake => "shake",
        }
    }

    /// The knack for it, as a companion might be good at it.
    pub fn knack(self) -> &'static str {
        match self {
            Self::Dig => "digging",
            Self::Reach => "reaching into things",
            Self::Scoop => "scooping",
            Self::Shake => "shaking things out",
        }
    }

    /// Where finds of this kind come from, for a journal entry not yet found.
    pub fn whereabouts(self) -> &'static str {
        match self {
            Self::Dig => "in the earth",
            Self::Reach => "in a hollow",
            Self::Scoop => "in the water",
            Self::Shake => "in the undergrowth",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tier {
    Common,
    Uncommon,
    Rare,
    Exceptional,
}

impl Tier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Exceptional => "exceptional",
        }
    }

    /// How often it turns up, all else being equal.
    pub fn weight(self) -> f32 {
        match self {
            Self::Common => 10.0,
            Self::Uncommon => 4.0,
            Self::Rare => 1.4,
            Self::Exceptional => 0.45,
        }
    }
}

/// What sort of companion a find leans towards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaning {
    Shiny,
    Growing,
    Old,
    Wild,
    Tiny,
    Odd,
    Cosy,
}

impl Leaning {
    /// How strongly a companion is drawn to this sort of thing, from 0 to 1, read off who it is.
    pub fn pull(self, character: &Character) -> f32 {
        let a = character.axes;
        let is = |kinds: &[TemperamentKind]| f32::from(u8::from(kinds.contains(&character.kind)));
        let little = f32::from(u8::from(character.parent.is_some()));
        use TemperamentKind as T;
        let pull = match self {
            Self::Shiny => {
                0.5 * a.boldness + 0.3 * a.playfulness + 0.4 * is(&[T::Showoff, T::Troublemaker])
            }
            Self::Growing => {
                0.5 * (1.0 - a.feistiness) + 0.2 * a.affection + 0.4 * is(&[T::Sweetheart])
            }
            Self::Old => {
                0.5 * a.curiosity
                    + 0.2 * (1.0 - a.impulsiveness)
                    + 0.5 * is(&[T::Scholar, T::Guardian])
            }
            Self::Wild => 0.4 * a.energy + 0.3 * a.curiosity + 0.4 * is(&[T::Explorer]),
            Self::Tiny => 0.7 * little + 0.3 * (1.0 - a.boldness) + 0.3 * is(&[T::Wallflower]),
            Self::Odd => 0.3 * a.playfulness + 0.3 * a.curiosity + 0.6 * is(&[T::Oddball]),
            Self::Cosy => {
                0.4 * (1.0 - a.energy) + 0.2 * a.affection + 0.5 * is(&[T::Lazybones, T::Guardian])
            }
        };
        pull.clamp(0.0, 1.0)
    }
}

/// How the colony enjoys a piece once it stands on the Hilltop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Goes over and has a good look.
    Look,
    /// Sits on it or beside it.
    Sit,
    /// Gazes up from it: at the sky, or the stars.
    Gaze,
    /// Plays with it, or to it.
    Play,
    /// Settles beside it for a rest.
    Rest,
}

#[derive(Debug)]
pub struct Find {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    /// What it becomes on the Hilltop.
    pub piece: &'static str,
    pub kind: Kind,
    pub tier: Tier,
    pub leanings: &'static [Leaning],
    pub use_: Use,
}

use Kind::*;
use Leaning::*;
use Tier::*;

pub const CATALOGUE: [Find; 28] = [
    // In the earth.
    Find {
        id: "smooth_pebble",
        name: "A smooth pebble",
        piece: "A pebble cairn",
        kind: Dig,
        tier: Common,
        leanings: &[Cosy],
        use_: Use::Look,
        blurb: "Warm from the earth and perfectly round. With a few more like it, a cairn.",
    },
    Find {
        id: "bluebell_bulb",
        name: "A bluebell bulb",
        piece: "A clump of bluebells",
        kind: Dig,
        tier: Common,
        leanings: &[Growing],
        use_: Use::Rest,
        blurb: "Papery and plump, and already pushing out a green tip.",
    },
    Find {
        id: "old_horseshoe",
        name: "An old horseshoe",
        piece: "A lucky horseshoe post",
        kind: Dig,
        tier: Common,
        leanings: &[Old, Shiny],
        use_: Use::Look,
        blurb: "Worn thin at the heel by somebody's long-ago walks.",
    },
    Find {
        id: "brass_key",
        name: "A brass key",
        piece: "A garden gate to nowhere",
        kind: Dig,
        tier: Uncommon,
        leanings: &[Old, Odd],
        use_: Use::Play,
        blurb: "It fits no lock anyone knows of, so it might as well have a gate of its own.",
    },
    Find {
        id: "pot_shard",
        name: "A painted pot shard",
        piece: "A mended pot of herbs",
        kind: Dig,
        tier: Uncommon,
        leanings: &[Old, Growing],
        use_: Use::Rest,
        blurb: "Blue flowers on cream. The rest of the pot was under it.",
    },
    Find {
        id: "sun_coin",
        name: "A sun-stamped coin",
        piece: "A sundial",
        kind: Dig,
        tier: Rare,
        leanings: &[Shiny, Old],
        use_: Use::Look,
        blurb: "Stamped with a sun and a line of numbers that turn out to be hours.",
    },
    Find {
        id: "tiny_door",
        name: "A tiny door",
        piece: "A little door in the hill",
        kind: Dig,
        tier: Exceptional,
        leanings: &[Odd, Tiny],
        use_: Use::Look,
        blurb: "A real door, hinges and all, just the right size for somebody very small.",
    },
    // In a hollow.
    Find {
        id: "pinecone",
        name: "A pinecone",
        piece: "A pinecone pile",
        kind: Reach,
        tier: Common,
        leanings: &[Wild],
        use_: Use::Play,
        blurb: "Closed up tight; it opens when the weather is fine.",
    },
    Find {
        id: "acorn_stash",
        name: "A squirrel's acorn stash",
        piece: "A young oak",
        kind: Reach,
        tier: Common,
        leanings: &[Cosy, Growing],
        use_: Use::Rest,
        blurb: "Somebody's winter larder. One of them has already sprouted.",
    },
    Find {
        id: "jay_feather",
        name: "A jay feather",
        piece: "A feather windvane",
        kind: Reach,
        tier: Common,
        leanings: &[Wild, Shiny],
        use_: Use::Look,
        blurb: "Barred in the brightest blue in the Woods.",
    },
    Find {
        id: "old_nest",
        name: "An empty nest",
        piece: "A nest box on a pole",
        kind: Reach,
        tier: Uncommon,
        leanings: &[Cosy, Tiny],
        use_: Use::Look,
        blurb: "Woven tight and lined with moss, and nobody home.",
    },
    Find {
        id: "lost_lantern",
        name: "A lost lantern",
        piece: "A lantern post",
        kind: Reach,
        tier: Uncommon,
        leanings: &[Old, Cosy],
        use_: Use::Rest,
        blurb: "There is still a stub of candle in it.",
    },
    Find {
        id: "brass_lens",
        name: "A brass lens",
        piece: "A little telescope",
        kind: Reach,
        tier: Rare,
        leanings: &[Old, Shiny],
        use_: Use::Gaze,
        blurb: "Heavy, clear, and set in a brass ring. Things look much closer through it.",
    },
    Find {
        id: "music_box",
        name: "A music box",
        piece: "A music box on a stump",
        kind: Reach,
        tier: Exceptional,
        leanings: &[Odd, Shiny],
        use_: Use::Play,
        blurb: "Wind it and it plays a tune nobody can quite remember learning.",
    },
    // In the water.
    Find {
        id: "skimming_stone",
        name: "A skimming stone",
        piece: "A ring of flat stones",
        kind: Scoop,
        tier: Common,
        leanings: &[Wild],
        use_: Use::Sit,
        blurb: "Flat as a biscuit. Too good to throw.",
    },
    Find {
        id: "river_glass",
        name: "A piece of river glass",
        piece: "A glass wind chime",
        kind: Scoop,
        tier: Common,
        leanings: &[Shiny, Tiny],
        use_: Use::Look,
        blurb: "Green, frosted and smooth, worn soft by the stream.",
    },
    Find {
        id: "mussel_shell",
        name: "A mussel shell",
        piece: "A birdbath",
        kind: Scoop,
        tier: Common,
        leanings: &[Cosy],
        use_: Use::Look,
        blurb: "Pearly inside, like the sky just before it rains.",
    },
    Find {
        id: "bottle_message",
        name: "A message in a bottle",
        piece: "A bottle tree",
        kind: Scoop,
        tier: Uncommon,
        leanings: &[Odd, Old],
        use_: Use::Look,
        blurb: "The message has run, all but the word \u{201c}hello\u{201d}.",
    },
    Find {
        id: "wooden_duck",
        name: "A wooden duck",
        piece: "A little pond",
        kind: Scoop,
        tier: Uncommon,
        leanings: &[Cosy, Wild],
        use_: Use::Rest,
        blurb: "A painted decoy, still bobbing. It wants water of its own.",
    },
    Find {
        id: "geode",
        name: "A geode",
        piece: "A geode on a plinth",
        kind: Scoop,
        tier: Rare,
        leanings: &[Shiny, Wild],
        use_: Use::Look,
        blurb: "A plain grey stone, cracked open on a cave of purple crystal.",
    },
    Find {
        id: "fallen_star",
        name: "A fallen star",
        piece: "The star stone",
        kind: Scoop,
        tier: Exceptional,
        leanings: &[Odd, Shiny],
        use_: Use::Gaze,
        blurb: "It was glowing under the water. It is still glowing now.",
    },
    // In the undergrowth.
    Find {
        id: "sycamore_key",
        name: "A sycamore key",
        piece: "A sycamore sapling",
        kind: Shake,
        tier: Common,
        leanings: &[Growing, Wild],
        use_: Use::Rest,
        blurb: "Drop it and it spins all the way down like a tiny helicopter.",
    },
    Find {
        id: "wild_berries",
        name: "A sprig of wild berries",
        piece: "A berry bush",
        kind: Shake,
        tier: Common,
        leanings: &[Growing, Cosy],
        use_: Use::Rest,
        blurb: "Most of them were eaten on the way home. Enough were left to plant.",
    },
    Find {
        id: "dandelion_clock",
        name: "A dandelion clock",
        piece: "A patch of dandelions",
        kind: Shake,
        tier: Common,
        leanings: &[Tiny, Growing],
        use_: Use::Play,
        blurb: "Carried home very carefully, and only blown a little.",
    },
    Find {
        id: "strange_seed",
        name: "A strange seed",
        piece: "A strange sapling",
        kind: Shake,
        tier: Uncommon,
        leanings: &[Odd, Growing],
        use_: Use::Look,
        blurb: "Striped, and faintly warm. Nobody knows what it will grow into.",
    },
    Find {
        id: "tangled_kite",
        name: "A tangled kite",
        piece: "A kite on a post",
        kind: Shake,
        tier: Uncommon,
        leanings: &[Wild, Shiny],
        use_: Use::Play,
        blurb: "Caught in the brambles by its tail. It only needs a breeze.",
    },
    Find {
        id: "silk_cocoon",
        name: "A silk cocoon",
        piece: "A butterfly bush",
        kind: Shake,
        tier: Rare,
        leanings: &[Tiny, Growing],
        use_: Use::Look,
        blurb: "Light as nothing, and something inside is nearly ready.",
    },
    Find {
        id: "weathervane",
        name: "An old weathervane",
        piece: "The fox weathervane",
        kind: Shake,
        tier: Exceptional,
        leanings: &[Old, Wild],
        use_: Use::Gaze,
        blurb: "A copper fox, green with age, blown into the bushes by some long-ago storm.",
    },
];

/// Things the Woods never turns up, won some other way: kept secret, out of the journal's hints,
/// until they are found.
pub const RELICS: [Find; 1] = [Find {
    id: "sovereign_arrow",
    name: "The Sovereign's arrow",
    piece: "The fallen cursor",
    kind: Reach,
    tier: Exceptional,
    leanings: &[Odd],
    use_: Use::Gaze,
    blurb: "A great white arrow, still faintly warm. Yes, that really happened.",
}];

pub fn find(id: &str) -> Option<&'static Find> {
    CATALOGUE
        .iter()
        .chain(RELICS.iter())
        .find(|find| find.id == id)
}

/// Whether a find is one of the secret relics.
pub fn is_relic(id: &str) -> bool {
    RELICS.iter().any(|relic| relic.id == id)
}

/// How likely `find` is to turn up for this party, before rarity.
fn affinity(find: &Find, party: &[&Character]) -> f32 {
    drawn_to(find.leanings, party)
}

/// How much a party is drawn to something that leans as `leanings` do: the keenest of them
/// counts. Never below one: leaning only adds, so nothing is out of anyone's reach.
pub fn drawn_to(leanings: &[Leaning], party: &[&Character]) -> f32 {
    let best = party
        .iter()
        .map(|character| {
            leanings
                .iter()
                .map(|leaning| leaning.pull(character))
                .sum::<f32>()
                / leanings.len().max(1) as f32
        })
        .fold(0.0, f32::max);
    1.0 + 1.6 * best
}

/// How strongly something undiscovered is favoured over what has been found before.
pub const NOVELTY: f32 = 2.5;
/// After this many outings in a row with nothing new found, something new is certain.
pub const DROUGHT: u32 = 2;
/// How many of the spots hold something on an outing, roughly.
const STOCKED: f32 = 0.65;

/// What is hidden where on one outing: a find, or nothing, for each spot of the given kinds.
/// `found_before` says whether the colony already has a find in its journal; `drought` is how
/// many outings in a row have turned up nothing new; `richer` adds to the chance that a spot of
/// each kind holds something, as what stands on the Hilltop draws the eye to it.
pub fn stock(
    spots: &[Kind],
    party: &[&Character],
    found_before: impl Fn(&str) -> bool,
    drought: u32,
    richer: &[f32; 4],
    dice: &mut Dice,
) -> Vec<Option<&'static Find>> {
    let mut stocked: Vec<Option<&'static Find>> = Vec::with_capacity(spots.len());
    for kind in spots {
        if !dice.chance((STOCKED + richer[kind.index()]).min(0.95)) {
            stocked.push(None);
            continue;
        }
        let find = choose(*kind, &stocked, party, &found_before, dice);
        stocked.push(find);
    }
    // A long run of nothing new: something new is certainly out there, the least rare first.
    let undiscovered: Vec<&'static Find> = CATALOGUE
        .iter()
        .filter(|find| !found_before(find.id))
        .filter(|find| spots.contains(&find.kind))
        .filter(|find| !stocked.iter().flatten().any(|s| s.id == find.id))
        .collect();
    if drought >= DROUGHT
        && let Some(least) = undiscovered.iter().map(|find| find.tier).min()
    {
        let candidates: Vec<&'static Find> = undiscovered
            .into_iter()
            .filter(|find| find.tier == least)
            .collect();
        if let Some(new) = pick(&candidates, |find| affinity(find, party), dice) {
            let places: Vec<usize> = (0..spots.len()).filter(|i| spots[*i] == new.kind).collect();
            let place = places[(dice.unit() * places.len() as f32) as usize % places.len()];
            stocked[place] = Some(new);
        }
    }
    stocked
}

/// What is in a spot that is never empty, such as one only some company opens: chosen as `stock`
/// chooses, without the chance of nothing. `stocked` is what the outing holds already, so the
/// rarer things still turn up only once.
pub fn surely(
    kind: Kind,
    stocked: &[Option<&'static Find>],
    party: &[&Character],
    found_before: impl Fn(&str) -> bool,
    dice: &mut Dice,
) -> Option<&'static Find> {
    choose(kind, stocked, party, &found_before, dice)
}

/// One find for a spot of `kind`, weighed by rarity, by who came and by what is new.
fn choose(
    kind: Kind,
    stocked: &[Option<&'static Find>],
    party: &[&Character],
    found_before: &impl Fn(&str) -> bool,
    dice: &mut Dice,
) -> Option<&'static Find> {
    // Rarer things only once per outing.
    let pool: Vec<&'static Find> = CATALOGUE
        .iter()
        .filter(|find| find.kind == kind)
        .filter(|find| find.tier == Common || !stocked.iter().flatten().any(|s| s.id == find.id))
        .collect();
    let weight = |find: &Find| {
        let novelty = if found_before(find.id) { 1.0 } else { NOVELTY };
        find.tier.weight() * affinity(find, party) * novelty
    };
    pick(&pool, weight, dice)
}

fn pick(
    pool: &[&'static Find],
    weight: impl Fn(&Find) -> f32,
    dice: &mut Dice,
) -> Option<&'static Find> {
    let total: f32 = pool.iter().map(|find| weight(find)).sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = dice.range(0.0, total);
    for find in pool {
        roll -= weight(find);
        if roll <= 0.0 {
            return Some(find);
        }
    }
    pool.last().copied()
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

    /// Twelve spots, three of each kind, as the Woods has them.
    const SPOTS: [Kind; 12] = [
        Dig, Dig, Dig, Reach, Reach, Reach, Scoop, Scoop, Scoop, Shake, Shake, Shake,
    ];

    #[test]
    fn the_catalogue_is_well_formed() {
        let ids: BTreeSet<&str> = CATALOGUE.iter().map(|find| find.id).collect();
        assert_eq!(ids.len(), CATALOGUE.len(), "ids are unique");
        for kind in Kind::ALL {
            let tiers: BTreeSet<Tier> = CATALOGUE
                .iter()
                .filter(|find| find.kind == kind)
                .map(|find| find.tier)
                .collect();
            assert_eq!(tiers.len(), 4, "{kind:?} has something of every rarity");
        }
        for find in &CATALOGUE {
            assert!(!find.leanings.is_empty(), "{} leans nowhere", find.id);
            assert!(
                find.blurb.len() < 120,
                "{}'s blurb is too long to show",
                find.id
            );
        }
    }

    #[test]
    fn who_comes_along_changes_what_turns_up() {
        let characters = characters();
        let tally = |character: &Character| {
            let mut dice = Dice::new(7);
            let mut seen = std::collections::BTreeMap::new();
            for _ in 0..400 {
                for find in stock(&SPOTS, &[character], |_| true, 0, &[0.0; 4], &mut dice)
                    .into_iter()
                    .flatten()
                {
                    *seen.entry(find.id).or_insert(0) += 1;
                }
            }
            seen
        };
        let tallies: Vec<_> = characters.iter().map(tally).collect();
        assert!(tallies.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn anyone_on_their_own_finds_everything_in_the_end() {
        for character in characters() {
            let mut dice = Dice::new(character.axes.curiosity.to_bits() as u64);
            let mut found: BTreeSet<&str> = BTreeSet::new();
            let mut drought = 0;
            let mut outings = 0;
            while found.len() < CATALOGUE.len() {
                outings += 1;
                assert!(
                    outings <= 120,
                    "{:?} after {outings} outings found only {}",
                    character.kind,
                    found.len()
                );
                let stocked = stock(
                    &SPOTS,
                    &[&character],
                    |id| found.contains(id),
                    drought,
                    &[0.0; 4],
                    &mut dice,
                );
                // A middling outing: the light lasts for about five good searches.
                let mut new = false;
                let mut picked = 0;
                for find in stocked.iter().flatten() {
                    // Something new is always worth going for; otherwise whatever is near.
                    if picked < 5 && (!found.contains(find.id) || dice.chance(0.5)) {
                        picked += 1;
                        new |= found.insert(find.id);
                    }
                }
                drought = if new { 0 } else { drought + 1 };
            }
        }
    }

    #[test]
    fn leaning_never_rules_anything_out() {
        for character in characters() {
            for find in &CATALOGUE {
                assert!(affinity(find, &[&character]) >= 1.0);
            }
        }
    }
}
