//! Hill's own souvenirs: the only rewards a story or a game can give. A package names one of
//! these by id and Hill's catalogue says what it is; a package can never invent a reward, and
//! nothing here reaches Desktop unless Desktop one day lists it among what it accepts.

pub mod art;

/// Who can give a souvenir.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Giver {
    Story,
    /// Won at one of Hill's games, never handed out by a story.
    Game,
}

/// Every souvenir: its id, what it is called, and who gives it.
const CATALOGUE: [(&str, &str, Giver); 9] = [
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
    ("chest_marble", "A marble from the toy chest", Giver::Story),
    (FAIR_TICKET, "A ticket from the Fairground", Giver::Game),
    (RACE_ROSETTE, "A rosette from the sack race", Giver::Game),
    (
        STRIKER_BELL,
        "A little brass bell from the high striker",
        Giver::Game,
    ),
];

/// Given the first time everyone is found at hide-and-seek.
pub const FAIR_TICKET: &str = "fair_ticket";
/// Given the first time a sack race is seen through.
pub const RACE_ROSETTE: &str = "race_rosette";
/// Given the first time everyone has a go at the high striker.
pub const STRIKER_BELL: &str = "striker_bell";

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Packages are written against this list and colonies keep what they were given, so nothing
    /// a story could once give ever leaves it.
    #[test]
    fn every_souvenir_a_story_has_been_able_to_give_still_can() {
        for id in [
            "picnic_ribbon",
            "pressed_daisy",
            "well_penny",
            "oak_acorn",
            "swing_feather",
            "chest_marble",
        ] {
            assert!(a_story_can_give(id), "{id}");
            assert!(name(id).is_some(), "{id}");
        }
        assert!(!a_story_can_give(FAIR_TICKET));
    }

    #[test]
    fn the_fairgrounds_souvenirs_are_won_at_its_games_and_never_given_by_a_story() {
        for id in [FAIR_TICKET, RACE_ROSETTE, STRIKER_BELL] {
            assert!(name(id).is_some(), "{id}");
            assert!(!a_story_can_give(id), "{id}");
            assert!(!for_stories().contains(&id), "{id}");
        }
        for id in ids() {
            assert!(
                id.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{id}"
            );
        }
    }
}
