//! The finds that grow once they are planted on the Hilltop, and what each is called on the way.
//!
//! A table keyed by find, rather than something every find carries: whatever turns up in the
//! Woods and grows can join it with a row here and its stages drawn beside its piece (see
//! `finds::art::stage`). Something planted grows one stage with each visit, the train's arrival,
//! and after its last stage it is its full piece, the find's own. Growing only ever goes forward:
//! nothing wilts while the colony is away, however long that is.

/// One find that grows.
#[derive(Debug)]
pub struct Growth {
    /// The find planted.
    pub id: &'static str,
    /// What it is called at each stage before it is grown, youngest first: just planted, then
    /// one more for each visit since. After the last of these it is its full piece.
    pub stages: &'static [&'static str],
}

pub const GROWING: &[Growth] = &[
    Growth {
        id: "bluebell_bulb",
        stages: &["A bluebell shoot", "Bluebell leaves", "Bluebells in bud"],
    },
    Growth {
        id: "pot_shard",
        stages: &["A mended pot, just sown", "A pot of herb seedlings"],
    },
    Growth {
        id: "acorn_stash",
        stages: &["A sprouting acorn", "An oak seedling", "An oak sapling"],
    },
    Growth {
        id: "sycamore_key",
        stages: &[
            "A sprouting sycamore key",
            "A sycamore seedling",
            "A sycamore in leaf",
        ],
    },
    Growth {
        id: "wild_berries",
        stages: &[
            "A berry sprout",
            "A little berry bush",
            "A berry bush in flower",
        ],
    },
    Growth {
        id: "dandelion_clock",
        stages: &[
            "A dandelion seedling",
            "Dandelion leaves",
            "Dandelions in bud",
        ],
    },
    Growth {
        id: "strange_seed",
        stages: &[
            "A strange sprout",
            "A strange curling shoot",
            "A strange little tree",
        ],
    },
    Growth {
        id: "silk_cocoon",
        stages: &[
            "A cocoon on a sprig",
            "A little bush with a cocoon",
            "A butterfly bush in bud",
        ],
    },
    // The hedgerow's seeds and cuttings.
    Growth {
        id: "strawberry_runner",
        stages: &[
            "A strawberry plantlet",
            "Spreading strawberry plants",
            "Strawberries in flower",
        ],
    },
    Growth {
        id: "hazelnut",
        stages: &[
            "A sprouting hazelnut",
            "A hazel seedling",
            "A hazel hung with catkins",
        ],
    },
    Growth {
        id: "crab_apple_pip",
        stages: &[
            "A sprouting crab-apple pip",
            "A crab-apple seedling",
            "A crab apple in blossom",
        ],
    },
    Growth {
        id: "rose_hip_seeds",
        stages: &[
            "Wild rose seedlings",
            "A little rose bush",
            "A wild rose in bud",
        ],
    },
    Growth {
        id: "thyme_cutting",
        stages: &[
            "A rooted thyme cutting",
            "A tuft of thyme",
            "A thyme cushion in bud",
        ],
    },
];

/// How a find grows, if it is one that does.
pub fn growth(id: &str) -> Option<&'static Growth> {
    GROWING.iter().find(|growth| growth.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_that_grows_is_a_find_and_grows_through_a_few_stages() {
        for (index, growth) in GROWING.iter().enumerate() {
            assert!(
                crate::finds::find(growth.id).is_some_and(|find| !crate::finds::is_relic(find.id)),
                "{} is not something the Woods turns up",
                growth.id
            );
            assert!(
                (2..=4).contains(&growth.stages.len()),
                "{} grows in {} stages",
                growth.id,
                growth.stages.len()
            );
            assert!(
                GROWING[index + 1..]
                    .iter()
                    .all(|other| other.id != growth.id),
                "{} grows twice",
                growth.id
            );
        }
    }

    #[test]
    fn the_hedgerows_seeds_and_cuttings_grow_once_planted_and_are_tended_till_grown() {
        use crate::finds::Use;
        use crate::hilltop::Standing;
        // Everything the hedgerow gives for planting, as `finds::FORAGED` says.
        for id in [
            "strawberry_runner",
            "hazelnut",
            "crab_apple_pip",
            "rose_hip_seeds",
            "thyme_cutting",
        ] {
            assert!(crate::finds::is_foraged(id), "{id} is not picked");
            let growth = growth(id).unwrap_or_else(|| panic!("{id} never grows"));
            let mut planted = Standing::from_satchel(id);
            for (stage, name) in growth.stages.iter().enumerate() {
                assert_eq!(planted.use_(), Some(Use::Tend), "{id} at {stage}");
                assert_eq!(planted.name(), *name);
                assert!(planted.grow());
            }
            assert_eq!(planted, Standing::from(id), "{id} grows into its piece");
            assert_ne!(planted.use_(), Some(Use::Tend), "{id} is grown");
        }
    }
}
