//! What only an expedition brings home: things from the places only an expedition reaches, a
//! catalogue of its own, so that nothing from afar turns up rummaging, foraging, on a line or in
//! a net, and nothing from those turns up afar. All of it is rare or exceptional, and each
//! becomes something like a landmark on the Hilltop. Their `kind` is only the sort of place in
//! the glade each draws the eye to once it stands there; none of them is for watching the sky.
//!
//! They come down the Far Falls (see `falls`), as everything else does, by who came along:
//! leaning only weights the odds, undiscovered things come down more often, and after a couple
//! of visits with nothing new the falls make sure, so whoever goes finds everything in the end.

use super::{DROUGHT, Find, Kind, Leaning, NOVELTY, Tier, Use, drawn_to};
use crate::character::Character;
use crate::dice::Dice;

use Kind::*;
use Leaning::*;
use Tier::*;

pub const AFAR: [Find; 5] = [
    Find {
        id: "carved_keystone",
        name: "A carved keystone",
        piece: "The old arch",
        kind: Dig,
        tier: Rare,
        leanings: &[Old, Wild],
        use_: Use::Look,
        blurb: "Wedge-shaped and carved with ivy leaves. The arch it held up is somewhere upstream.",
    },
    Find {
        id: "bronze_bell",
        name: "A little bronze bell",
        piece: "The bell post",
        kind: Reach,
        tier: Rare,
        leanings: &[Old, Odd],
        use_: Use::Play,
        blurb: "Green with age and still ringing true. Somebody once hung it where all could hear.",
    },
    Find {
        id: "falls_pearl",
        name: "A falls pearl",
        piece: "The pearl grotto",
        kind: Scoop,
        tier: Rare,
        leanings: &[Shiny, Tiny],
        use_: Use::Look,
        blurb: "Round and pink-white, rolled smooth by the falls. It shines even in the shade.",
    },
    Find {
        id: "stone_hare",
        name: "A carved stone hare",
        piece: "The hare stone",
        kind: Dig,
        tier: Exceptional,
        leanings: &[Old, Cosy],
        use_: Use::Rest,
        blurb: "Sitting up with its ears laid back, worn soft by the water. Everyone pats it for luck.",
    },
    Find {
        id: "rainbow_prism",
        name: "A rainbow crystal",
        piece: "The rainbow prism",
        kind: Scoop,
        tier: Exceptional,
        leanings: &[Shiny, Odd],
        use_: Use::Look,
        blurb: "Clear as the spray it came down in. Held up to the light, it throws a rainbow.",
    },
];

/// Whether a find is one only an expedition brings home.
pub fn is_from_afar(id: &str) -> bool {
    AFAR.iter().any(|find| find.id == id)
}

/// What the falls bring down on one visit: `count` things, each once, weighed by rarity, by who
/// came and by what is new. After a dry spell of visits, something new is certainly among them,
/// the least rare first.
pub fn bring_down(
    count: usize,
    party: &[&Character],
    found_before: impl Fn(&str) -> bool,
    drought: u32,
    dice: &mut Dice,
) -> Vec<&'static Find> {
    let weight = |find: &Find| {
        let novelty = if found_before(find.id) { 1.0 } else { NOVELTY };
        find.tier.weight() * drawn_to(find.leanings, party) * novelty
    };
    let mut coming: Vec<&'static Find> = Vec::new();
    let undiscovered: Vec<&'static Find> =
        AFAR.iter().filter(|find| !found_before(find.id)).collect();
    if drought >= DROUGHT
        && let Some(least) = undiscovered.iter().map(|find| find.tier).min()
    {
        let candidates: Vec<&'static Find> = undiscovered
            .into_iter()
            .filter(|find| find.tier == least)
            .collect();
        let weights: Vec<f32> = candidates.iter().map(|find| weight(find)).collect();
        if let Some(index) = dice.weighted(&weights) {
            coming.push(candidates[index]);
        }
    }
    while coming.len() < count.min(AFAR.len()) {
        let pool: Vec<&'static Find> = AFAR
            .iter()
            .filter(|find| !coming.iter().any(|already| already.id == find.id))
            .collect();
        let weights: Vec<f32> = pool.iter().map(|find| weight(find)).collect();
        let Some(index) = dice.weighted(&weights) else {
            break;
        };
        coming.push(pool[index]);
    }
    // The certain one needn't come first.
    if coming.len() > 1 && dice.chance(0.5) {
        coming.swap(0, 1);
    }
    coming
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::finds::{CATALOGUE, FORAGED, RELICS, find, is_foraged, is_relic, stock, surely};
    use std::collections::BTreeSet;

    fn characters() -> Vec<Character> {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        cast.members.iter().map(Character::of).collect()
    }

    #[test]
    fn the_catalogue_from_afar_is_rare_kept_apart_and_never_for_the_sky() {
        let all: BTreeSet<&str> = CATALOGUE
            .iter()
            .chain(FORAGED.iter())
            .chain(RELICS.iter())
            .chain(AFAR.iter())
            .map(|find| find.id)
            .collect();
        assert_eq!(
            all.len(),
            CATALOGUE.len() + FORAGED.len() + RELICS.len() + AFAR.len(),
            "ids are unique across every catalogue"
        );
        for far in &AFAR {
            assert!(
                far.tier >= Tier::Rare,
                "{} is too common to come so far",
                far.id
            );
            assert_eq!(find(far.id).map(|found| found.id), Some(far.id));
            assert!(is_from_afar(far.id) && !is_foraged(far.id) && !is_relic(far.id));
            assert!(!far.leanings.is_empty(), "{} leans nowhere", far.id);
            assert!(
                far.blurb.len() < 120,
                "{}'s blurb is too long to show",
                far.id
            );
            // A sky-gazing piece is part of the secret's trigger, and nothing from afar is one.
            assert_ne!(far.use_, Use::Gaze, "{} would gaze at the sky", far.id);
        }
        for other in CATALOGUE.iter().chain(FORAGED.iter()).chain(RELICS.iter()) {
            assert!(
                !is_from_afar(other.id),
                "{} is counted as from afar",
                other.id
            );
        }
        let tiers: BTreeSet<Tier> = AFAR.iter().map(|find| find.tier).collect();
        assert_eq!(tiers, BTreeSet::from([Tier::Rare, Tier::Exceptional]));
    }

    #[test]
    fn nothing_from_afar_turns_up_anywhere_else_and_the_falls_bring_only_it() {
        let characters = characters();
        let party: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(21);
        const SPOTS: [Kind; 12] = [
            Dig, Dig, Dig, Reach, Reach, Reach, Scoop, Scoop, Scoop, Shake, Shake, Shake,
        ];
        for _ in 0..300 {
            let stocked = stock(&SPOTS, &party, |_| false, 3, &[0.2; 4], &mut dice);
            for found in stocked.iter().flatten() {
                assert!(!is_from_afar(found.id), "{} turned up rummaging", found.id);
            }
            for kind in Kind::ALL {
                let surely = surely(kind, &stocked, &party, |_| false, &mut dice);
                assert!(surely.is_some_and(|found| !is_from_afar(found.id)));
            }
            for coming in bring_down(3, &party, |_| false, 3, &mut dice) {
                assert!(is_from_afar(coming.id), "{} came down the falls", coming.id);
            }
        }
    }

    #[test]
    fn each_thing_comes_down_once_a_visit_and_who_comes_changes_what() {
        let characters = characters();
        let tally = |character: &Character| {
            let mut dice = Dice::new(5);
            let mut seen = std::collections::BTreeMap::new();
            for _ in 0..400 {
                let coming = bring_down(2, &[character], |_| true, 0, &mut dice);
                let ids: BTreeSet<&str> = coming.iter().map(|find| find.id).collect();
                assert_eq!(ids.len(), coming.len(), "something came down twice");
                assert_eq!(coming.len(), 2);
                for id in ids {
                    *seen.entry(id).or_insert(0) += 1;
                }
            }
            seen
        };
        let tallies: Vec<_> = characters.iter().map(tally).collect();
        assert!(tallies.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn undiscovered_things_come_down_more_often_and_after_a_dry_spell_something_new_is_certain() {
        let characters = characters();
        let party = [&characters[0]];
        let count = |found: &dyn Fn(&str) -> bool, drought: u32, id: &str| {
            let mut dice = Dice::new(17);
            (0..400)
                .filter(|_| {
                    bring_down(2, &party, found, drought, &mut dice)
                        .iter()
                        .any(|find| find.id == id)
                })
                .count()
        };
        let all = |_: &str| true;
        let but_the_hare = |id: &str| id != "stone_hare";
        assert!(count(&but_the_hare, 0, "stone_hare") > count(&all, 0, "stone_hare"));
        let but_the_prism = |id: &str| id != "rainbow_prism";
        let wet = count(&but_the_prism, 0, "rainbow_prism");
        assert!(wet < 400, "the prism always comes down anyway");
        assert_eq!(count(&but_the_prism, DROUGHT, "rainbow_prism"), 400);
    }

    /// The completeness guarantee: with only ever one companion, keep going to the falls and
    /// everything from afar is found in the end, whoever that companion is.
    #[test]
    fn anyone_on_their_own_finds_everything_from_afar_in_the_end() {
        for character in characters() {
            let mut dice = Dice::new(character.axes.curiosity.to_bits() as u64);
            let mut found: BTreeSet<&str> = BTreeSet::new();
            let mut drought = 0;
            let mut visits = 0;
            while found.len() < AFAR.len() {
                visits += 1;
                assert!(
                    visits <= 40,
                    "{:?} after {visits} visits found only {found:?}",
                    character.kind
                );
                // A middling visit: one of what comes down is caught.
                let coming = bring_down(
                    2,
                    &[&character],
                    |id| found.contains(id),
                    drought,
                    &mut dice,
                );
                let caught = coming.first().map(|find| find.id);
                let new = caught.is_some_and(|id| found.insert(id));
                drought = if new { 0 } else { drought + 1 };
            }
        }
    }
}
