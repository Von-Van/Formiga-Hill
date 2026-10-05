//! Plans: what the colony can build on the Hilltop out of several finds, or make by putting finds
//! together into something new. The finds' own blurbs hint at some of them ("With a few more like
//! it, a cairn"). Commoner finds make homely things; the rarest make landmarks.
//!
//! A plan is thought of once the colony has found any one of the finds it takes, never before, so
//! the list grows with the journal. Building takes the finds from the satchel and stands the
//! result on one spot, like any piece; taking it apart gives every find back. Nothing is used up.

use super::{Tier, Use, find};
use std::collections::BTreeMap;

/// Something that can be built on the Hilltop.
#[derive(Debug)]
pub struct Plan {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    /// The finds it takes, and how many of each.
    pub needs: &'static [(&'static str, u32)],
    /// How the colony enjoys it once it stands.
    pub use_: Use,
}

pub const PLANS: [Plan; 12] = [
    Plan {
        id: "grand_cairn",
        name: "The grand cairn",
        needs: &[("smooth_pebble", 3)],
        use_: Use::Look,
        blurb: "Three smooth pebbles and every flat stone near them, stacked as tall as anyone \
                dared: a waymark for the Hill.",
    },
    Plan {
        id: "flower_bed",
        name: "A flower bed",
        needs: &[("bluebell_bulb", 2), ("dandelion_clock", 2)],
        use_: Use::Rest,
        blurb: "Bluebells at the back, dandelions at the front, and a little woven edge to keep \
                them in.",
    },
    Plan {
        id: "picnic_table",
        name: "A picnic table",
        needs: &[
            ("skimming_stone", 2),
            ("smooth_pebble", 2),
            ("wild_berries", 1),
        ],
        use_: Use::Sit,
        blurb: "Flat stones for a table and two seats, pebbles to stand them on, and berries out \
                for whoever sits first.",
    },
    Plan {
        id: "bird_table",
        name: "A bird table",
        needs: &[("old_nest", 1), ("pinecone", 1), ("wild_berries", 1)],
        use_: Use::Look,
        blurb: "A roof on a post with the nest tucked under it, berries on the tray and a \
                pinecone feeder. Somebody always comes.",
    },
    Plan {
        id: "snug_den",
        name: "A snug den",
        needs: &[("old_nest", 1), ("pinecone", 3)],
        use_: Use::Rest,
        blurb: "Pinecones heaped over a frame of twigs and lined with the nest's moss: room \
                inside for a nap.",
    },
    Plan {
        id: "lantern_tree",
        name: "The lantern tree",
        needs: &[("bottle_message", 1), ("lost_lantern", 2)],
        use_: Use::Rest,
        blurb: "The bottle tree with two lanterns hung among its bottles, and glass beads \
                strung between, to light the summit.",
    },
    Plan {
        id: "wishing_well",
        name: "A wishing well",
        needs: &[("smooth_pebble", 2), ("sun_coin", 1)],
        use_: Use::Play,
        blurb: "Pebbles for its wall, a roof over it, and the sun-stamped coin at the bottom \
                for the first wish.",
    },
    Plan {
        id: "great_telescope",
        name: "The great telescope",
        needs: &[("brass_lens", 2)],
        use_: Use::Gaze,
        blurb: "Two lenses in one long brass tube on a stone pier that turns: the stars come \
                right up close.",
    },
    Plan {
        id: "bandstand",
        name: "The bandstand",
        needs: &[("music_box", 1), ("lost_lantern", 1), ("river_glass", 2)],
        use_: Use::Play,
        blurb: "The music box playing under a striped roof, glass chimes along the eaves, and \
                a lantern for evening tunes.",
    },
    Plan {
        id: "burrow_house",
        name: "The burrow house",
        needs: &[("tiny_door", 1), ("pinecone", 2), ("smooth_pebble", 1)],
        use_: Use::Look,
        blurb: "The tiny door in a hill of its own, with a pinecone porch, a window in the roof \
                and smoke from the chimney.",
    },
    Plan {
        id: "woodshed",
        name: "The woodshed",
        needs: &[
            ("woodcutters_lantern", 1),
            ("carved_sign", 1),
            ("pinecone", 2),
        ],
        use_: Use::Rest,
        blurb: "Logs stacked under a little roof, the carved sign over it and the lantern hung \
                by it: somewhere dry to doze.",
    },
    Plan {
        id: "tea_party",
        name: "A tea party",
        needs: &[
            ("china_teacup", 2),
            ("copper_kettle", 1),
            ("wild_berries", 1),
        ],
        use_: Use::Sit,
        blurb: "A cloth on a stump, the kettle on, the good china out and berries in a bowl. \
                Everyone is invited.",
    },
];

pub fn plan(id: &str) -> Option<&'static Plan> {
    PLANS.iter().find(|plan| plan.id == id)
}

impl Plan {
    /// As rare as the rarest find it takes.
    pub fn tier(&self) -> Tier {
        self.needs
            .iter()
            .filter_map(|(id, _)| find(id))
            .map(|find| find.tier)
            .max()
            .unwrap_or(Tier::Common)
    }

    /// Whether the colony has thought of it: it has found at least one of the finds it takes.
    pub fn thought_of(&self, found: impl Fn(&str) -> bool) -> bool {
        self.needs.iter().any(|(id, _)| found(id))
    }

    /// What it is still short of in the satchel: each find, and how many more.
    pub fn short(&self, satchel: &BTreeMap<String, u32>) -> Vec<(&'static str, u32)> {
        self.needs
            .iter()
            .filter_map(|&(id, count)| {
                let have = satchel.get(id).copied().unwrap_or(0);
                (have < count).then(|| (id, count - have))
            })
            .collect()
    }

    /// Whether everything it takes is in the satchel.
    pub fn ready(&self, satchel: &BTreeMap<String, u32>) -> bool {
        self.short(satchel).is_empty()
    }

    /// Every find it takes, one entry for each of them.
    pub fn finds(&self) -> Vec<&'static str> {
        self.needs
            .iter()
            .flat_map(|&(id, count)| std::iter::repeat_n(id, count as usize))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::{CATALOGUE, SCAVENGED, is_relic};
    use std::collections::BTreeSet;

    #[test]
    fn every_plan_takes_finds_the_woods_turns_up_and_more_than_one() {
        let ids: BTreeSet<&str> = PLANS.iter().map(|plan| plan.id).collect();
        assert_eq!(ids.len(), PLANS.len(), "ids are unique");
        for plan in &PLANS {
            assert!(!plan.needs.is_empty(), "{} takes nothing", plan.id);
            for (id, count) in plan.needs {
                assert!(
                    CATALOGUE
                        .iter()
                        .chain(SCAVENGED.iter())
                        .any(|find| find.id == *id)
                        && !is_relic(id),
                    "{} takes {id}, which the Woods never turns up",
                    plan.id
                );
                assert!(*count >= 1);
            }
            assert!(plan.finds().len() >= 2, "{} is only one find", plan.id);
            assert!(
                plan.blurb.len() < 130,
                "{}'s blurb is too long to show",
                plan.id
            );
            assert!(
                find(plan.id).is_none(),
                "{} has the same id as a find",
                plan.id
            );
        }
    }

    #[test]
    fn there_is_something_to_build_from_every_rarity() {
        let tiers: BTreeSet<Tier> = PLANS.iter().map(Plan::tier).collect();
        assert_eq!(tiers.len(), 4, "only {tiers:?}");
    }

    #[test]
    fn a_plan_is_thought_of_with_its_first_find_and_says_what_it_is_short_of() {
        let cairn = plan("grand_cairn").unwrap();
        assert!(!cairn.thought_of(|_| false));
        assert!(cairn.thought_of(|id| id == "smooth_pebble"));
        let mut satchel = BTreeMap::from([("smooth_pebble".to_owned(), 2)]);
        assert_eq!(cairn.short(&satchel), [("smooth_pebble", 1)]);
        assert!(!cairn.ready(&satchel));
        satchel.insert("smooth_pebble".to_owned(), 3);
        assert!(cairn.ready(&satchel));
    }
}
