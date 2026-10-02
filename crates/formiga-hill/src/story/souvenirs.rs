//! Hill's own souvenirs: the only rewards a story or a game can give. A package names one of
//! these by id and Hill's catalogue says what it is; a package can never invent a reward, and
//! nothing here reaches Desktop unless Desktop one day lists it among what it accepts.

/// Who can give a souvenir.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Giver {
    Story,
    /// Won at one of Hill's games, never handed out by a story.
    Game,
}

/// Every souvenir: its id, what it is called, and who gives it.
const CATALOGUE: [(&str, &str, Giver); 6] = [
    (
        "picnic_ribbon",
        "A gingham ribbon from the first picnic",
        Giver::Story,
    ),
    ("pressed_daisy", "A daisy pressed flat", Giver::Story),
    (
        "well_penny",
        "A penny from the bottom of the well",
        Giver::Story,
    ),
    ("oak_acorn", "An acorn from the old oak", Giver::Story),
    (
        "swing_feather",
        "A feather caught on the swing",
        Giver::Story,
    ),
    (FAIR_TICKET, "A ticket from the Fairground", Giver::Game),
];

/// Given the first time everyone is found at hide-and-seek.
pub const FAIR_TICKET: &str = "fair_ticket";

/// Whether a story may give this souvenir.
pub fn a_story_can_give(id: &str) -> bool {
    CATALOGUE
        .iter()
        .any(|(known, _, giver)| *known == id && *giver == Giver::Story)
}

/// Every souvenir, in the order the display case keeps them.
pub fn ids() -> Vec<&'static str> {
    CATALOGUE.iter().map(|(id, _, _)| *id).collect()
}

/// The souvenirs a story may give.
pub fn for_stories() -> Vec<&'static str> {
    CATALOGUE
        .iter()
        .filter(|(_, _, giver)| *giver == Giver::Story)
        .map(|(id, _, _)| *id)
        .collect()
}

/// What a souvenir is called, for showing.
pub fn name(id: &str) -> Option<&'static str> {
    CATALOGUE
        .iter()
        .find(|(known, _, _)| *known == id)
        .map(|(_, name, _)| *name)
}
