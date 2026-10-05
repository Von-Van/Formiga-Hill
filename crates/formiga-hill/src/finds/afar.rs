//! What only an expedition brings home: things from the places only an expedition reaches, a
//! catalogue of its own, so that nothing from afar turns up rummaging, foraging, on a line or in
//! a net, and nothing from those turns up afar. All of it is rare or exceptional, and each
//! becomes something like a landmark on the Hilltop. Their `kind` is only the sort of place in
//! the glade each draws the eye to once it stands there; none of them is for watching the sky.
//!
//! They come down the Far Falls (see `falls`), as everything else does, by who came along:
//! leaning only weights the odds, undiscovered things come down more often, and after a couple
//! of visits with nothing new the falls make sure, so whoever goes finds everything in the end.

use super::{Find, Kind, Leaning, Tier, Use};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::{CATALOGUE, FORAGED, RELICS, find, is_foraged, is_relic};
    use std::collections::BTreeSet;

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
}
