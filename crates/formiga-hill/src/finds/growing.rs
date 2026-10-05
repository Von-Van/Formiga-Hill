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
}
