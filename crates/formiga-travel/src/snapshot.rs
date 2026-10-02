//! What Desktop hands Hill when the colony boards the train.

use crate::text::{is_display_text, is_identifier};
use crate::{SessionId, VersionError, check_versions};
use formiga_core::{
    Accessory, AppearanceGenome, CreatureId, CreatureRelationship, CreatureRole, Habit,
    MAX_COLONY_CREATURES, MAX_HABITS, MAX_RELATIONSHIPS, Temperament, Trait,
    validate_creature_name,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use time::OffsetDateTime;

/// A whole colony at most: Hill never receives more companions than Desktop can hold.
pub const MAX_TRAVELERS: usize = MAX_COLONY_CREATURES;
/// One bond per pair of travellers.
pub const MAX_BONDS: usize = MAX_RELATIONSHIPS;
/// The traits a companion's profile names.
pub const MAX_TRAITS: usize = 3;
pub const MAX_CAPABILITIES: usize = 64;
const MAX_CAPABILITY_LEN: usize = 64;
const MAX_VERSION_CHARS: usize = 64;
const MAX_COLONY_ID_LEN: usize = 64;

/// The parts of the contract a snapshot fills in. Hill content asks for a capability rather than
/// a version number, so a snapshot from an older Desktop that lacks one still opens, and only the
/// content that needs it steps aside. Names are never reused for a different meaning.
pub mod capability {
    /// `Traveler::appearance` holds Desktop's full appearance genome.
    pub const APPEARANCE: &str = "appearance";
    /// `Traveler::temperament` and `Traveler::traits` are filled in.
    pub const TEMPERAMENT: &str = "temperament";
    /// `Traveler::habits` lists the habits each companion has learned.
    pub const HABITS: &str = "habits";
    /// `Traveler::accessory` carries what each companion is wearing, inked as Desktop draws it.
    pub const ACCESSORIES: &str = "accessories";
    /// `Traveler::role` names each mini's parent.
    pub const FAMILY: &str = "family";
    /// `TravelSnapshot::bonds` describes the pairs, in bands.
    pub const BONDS: &str = "bonds";

    /// Everything this build's exporter fills in.
    pub const ALL: [&str; 6] = [APPEARANCE, TEMPERAMENT, HABITS, ACCESSORIES, FAMILY, BONDS];
}

/// Who is travelling, as Desktop knows them at the moment the train leaves. Read-only for Hill:
/// nothing in it is ever written back.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TravelSnapshot {
    pub format_version: u32,
    pub minimum_reader_version: u32,
    pub session_id: SessionId,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at_utc: OffsetDateTime,
    /// The Desktop release that wrote it, for diagnostics and for nothing else.
    pub desktop_version: String,
    pub colony: ColonyTag,
    pub travelers: Vec<Traveler>,
    #[serde(default)]
    pub bonds: Vec<Bond>,
    #[serde(default)]
    pub presentation: Presentation,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Which colony is visiting, without saying anything else about it: a one-way tag from the
/// colony's seed, so Hill can keep one save per colony and never learns the seed itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyTag {
    pub public_id: String,
}

impl ColonyTag {
    pub fn for_seed(colony_seed: &[u8; 32]) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"formiga-travel/colony-tag/v1");
        hash.update(colony_seed);
        let digest = hash.finalize();
        Self {
            public_id: digest[..8]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        }
    }
}

/// One companion on the train: the same individual Desktop shows, so Hill can draw and cast it
/// as itself rather than as a lookalike.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Traveler {
    /// Desktop's own ID for it, stable for as long as it lives in the colony.
    pub id: CreatureId,
    pub name: String,
    /// Adult, or a mini with its parent's ID. The parent may not be on the train.
    #[serde(default)]
    pub role: CreatureRole,
    pub generation: u8,
    #[serde(with = "time::serde::rfc3339")]
    pub born_at_utc: OffsetDateTime,
    /// Everything `formiga-art` needs to draw it exactly as Desktop does.
    pub appearance: AppearanceGenome,
    pub temperament: Temperament,
    /// The traits its profile names, as Desktop resolved them. A trait this build does not know
    /// is dropped on reading: a newer Desktop may name more, and none of them is identity.
    #[serde(default, with = "trait_names")]
    pub traits: Vec<Trait>,
    /// The little habits it has picked up, in the order it picked them up.
    #[serde(default)]
    pub habits: Vec<Habit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessory: Option<WornAccessory>,
}

impl Traveler {
    pub fn parent_id(&self) -> Option<CreatureId> {
        self.role.parent_id()
    }

    pub fn is_mini(&self) -> bool {
        !self.role.is_adult()
    }
}

/// What a companion is wearing, with the inks Desktop drew it in. The inks are chosen against
/// the colony's own seed and coats, which Hill does not receive, so they travel resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WornAccessory {
    pub item: Accessory,
    pub ink: AccessoryInk,
}

/// The five inks an accessory is drawn in, as RGBA.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessoryInk {
    pub outline: [u8; 4],
    pub deep: [u8; 4],
    pub body: [u8; 4],
    pub light: [u8; 4],
    pub accent: [u8; 4],
}

impl From<formiga_art::TrinketInk> for AccessoryInk {
    fn from(ink: formiga_art::TrinketInk) -> Self {
        let rgba = |c: formiga_art::Rgba| [c.r, c.g, c.b, c.a];
        Self {
            outline: rgba(ink.outline),
            deep: rgba(ink.deep),
            body: rgba(ink.body),
            light: rgba(ink.light),
            accent: rgba(ink.accent),
        }
    }
}

impl From<AccessoryInk> for formiga_art::TrinketInk {
    fn from(ink: AccessoryInk) -> Self {
        let rgba = |[r, g, b, a]: [u8; 4]| formiga_art::Rgba::new(r, g, b, a);
        Self {
            outline: rgba(ink.outline),
            deep: rgba(ink.deep),
            body: rgba(ink.body),
            light: rgba(ink.light),
            accent: rgba(ink.accent),
        }
    }
}

impl WornAccessory {
    /// Ready for `formiga-art` to draw onto a body frame.
    pub fn art(self) -> formiga_art::AccessoryArt {
        formiga_art::AccessoryArt {
            accessory: self.item,
            ink: self.ink.into(),
        }
    }
}

/// How one pair of travellers get on, in coarse bands rather than Desktop's raw scores: enough
/// for casting and staging, and free to stay the same while Desktop retunes its own scales.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bond {
    /// The lower of the two IDs.
    pub a: CreatureId,
    /// The higher of the two IDs.
    pub b: CreatureId,
    /// How fond they are of each other: Desktop's affinity.
    pub warmth: Level,
    /// How used to each other they are.
    pub familiarity: Level,
    /// How much they play together, roughly or not.
    pub playfulness: Level,
    /// How much they get under each other's feet: Desktop's avoidance.
    pub friction: Level,
}

impl Bond {
    pub fn from_relationship(relationship: &CreatureRelationship) -> Option<Self> {
        let (a, b) = formiga_core::canonical_creature_pair(relationship.a, relationship.b)?;
        Some(Self {
            a,
            b,
            warmth: Level::from_score(relationship.affinity),
            familiarity: Level::from_score(relationship.familiarity),
            playfulness: Level::from_score(relationship.playfulness),
            friction: Level::from_score(relationship.avoidance),
        })
    }

    pub fn contains(&self, id: CreatureId) -> bool {
        self.a == id || self.b == id
    }

    pub fn other(&self, id: CreatureId) -> Option<CreatureId> {
        if self.a == id {
            Some(self.b)
        } else if self.b == id {
            Some(self.a)
        } else {
            None
        }
    }
}

/// A band on one of a bond's scales.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    #[default]
    None,
    Low,
    Medium,
    High,
}

impl Level {
    /// Bands over Desktop's 0–255 scores. The edges sit near the thresholds Desktop's own
    /// behaviour reads at (48, 64, 96 and 128), so a band says much what Desktop would.
    pub const fn from_score(score: u8) -> Self {
        match score {
            0..=15 => Self::None,
            16..=63 => Self::Low,
            64..=127 => Self::Medium,
            _ => Self::High,
        }
    }
}

/// The person's presentation preferences that should feel the same in both apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Presentation {
    pub reduce_motion: bool,
    pub theme: Theme,
    /// Percent of the base text size, 100 to 150.
    pub text_scale_percent: u8,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            reduce_motion: false,
            theme: Theme::System,
            text_scale_percent: 100,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl From<formiga_core::ThemeChoice> for Theme {
    fn from(choice: formiga_core::ThemeChoice) -> Self {
        match choice {
            formiga_core::ThemeChoice::System => Self::System,
            formiga_core::ThemeChoice::Light => Self::Light,
            formiga_core::ThemeChoice::Dark => Self::Dark,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SnapshotError {
    #[error(transparent)]
    Version(#[from] VersionError),
    #[error("the snapshot's {0} is missing or malformed")]
    BadField(&'static str),
    #[error("nobody is on the train")]
    NoTravelers,
    #[error("{0} travellers is more than a colony holds ({MAX_TRAVELERS})")]
    TooManyTravelers(usize),
    #[error("traveller {0} appears more than once")]
    DuplicateTraveler(CreatureId),
    #[error("traveller {0} has a name Desktop would not allow")]
    BadName(CreatureId),
    #[error("traveller {0} is listed as its own parent")]
    OwnParent(CreatureId),
    #[error("traveller {0} lists more traits or habits than a companion can have")]
    TooManyQualities(CreatureId),
    #[error("{0} bonds is more than a colony has pairs ({MAX_BONDS})")]
    TooManyBonds(usize),
    #[error("the bond between {a} and {b} is not between two different travellers in order")]
    BadBond { a: CreatureId, b: CreatureId },
    #[error("the bond between {a} and {b} is listed twice")]
    DuplicateBond { a: CreatureId, b: CreatureId },
    #[error("the snapshot lists more than {MAX_CAPABILITIES} capabilities, or a malformed one")]
    BadCapabilities,
}

impl TravelSnapshot {
    /// Everything a reader checks before trusting a snapshot. Unknown fields have already been
    /// ignored by then, which is what lets a newer Desktop add to the format without breaking an
    /// older Hill; a change an older Hill would misread raises `minimum_reader_version` instead.
    pub fn validate(&self) -> Result<(), SnapshotError> {
        check_versions(self.format_version, self.minimum_reader_version)?;
        if !is_display_text(&self.desktop_version, MAX_VERSION_CHARS) {
            return Err(SnapshotError::BadField("desktop_version"));
        }
        if !is_identifier(&self.colony.public_id, MAX_COLONY_ID_LEN) {
            return Err(SnapshotError::BadField("colony.public_id"));
        }

        if self.travelers.is_empty() {
            return Err(SnapshotError::NoTravelers);
        }
        if self.travelers.len() > MAX_TRAVELERS {
            return Err(SnapshotError::TooManyTravelers(self.travelers.len()));
        }
        let mut ids = BTreeSet::new();
        for traveler in &self.travelers {
            if !ids.insert(traveler.id) {
                return Err(SnapshotError::DuplicateTraveler(traveler.id));
            }
            if validate_creature_name(&traveler.name).as_deref() != Ok(traveler.name.as_str()) {
                return Err(SnapshotError::BadName(traveler.id));
            }
            if traveler.parent_id() == Some(traveler.id) {
                return Err(SnapshotError::OwnParent(traveler.id));
            }
            if traveler.traits.len() > MAX_TRAITS || traveler.habits.len() > MAX_HABITS {
                return Err(SnapshotError::TooManyQualities(traveler.id));
            }
        }

        if self.bonds.len() > MAX_BONDS {
            return Err(SnapshotError::TooManyBonds(self.bonds.len()));
        }
        let mut pairs = BTreeSet::new();
        for bond in &self.bonds {
            let (a, b) = (bond.a, bond.b);
            if a >= b || !ids.contains(&a) || !ids.contains(&b) {
                return Err(SnapshotError::BadBond { a, b });
            }
            if !pairs.insert((a, b)) {
                return Err(SnapshotError::DuplicateBond { a, b });
            }
        }

        if self.capabilities.len() > MAX_CAPABILITIES
            || !self
                .capabilities
                .iter()
                .all(|name| is_identifier(name, MAX_CAPABILITY_LEN))
        {
            return Err(SnapshotError::BadCapabilities);
        }
        Ok(())
    }

    pub fn traveler(&self, id: CreatureId) -> Option<&Traveler> {
        self.travelers.iter().find(|traveler| traveler.id == id)
    }

    pub fn bond(&self, a: CreatureId, b: CreatureId) -> Option<&Bond> {
        let (a, b) = formiga_core::canonical_creature_pair(a, b)?;
        self.bonds.iter().find(|bond| bond.a == a && bond.b == b)
    }

    /// The traveller's parent, if it is a mini and its parent came along.
    pub fn parent_of(&self, traveler: &Traveler) -> Option<&Traveler> {
        self.traveler(traveler.parent_id()?)
    }

    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability == name)
    }
}

/// `Trait` carries no serde derive in `formiga-core`, so a trait travels as its variant name,
/// which Desktop keeps stable for the same reason it keeps its generation append-only.
mod trait_names {
    use formiga_core::Trait;
    use serde::{Deserialize, Deserializer, Serializer};

    fn name(value: Trait) -> String {
        format!("{value:?}")
    }

    pub fn serialize<S: Serializer>(traits: &[Trait], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(traits.iter().map(|value| name(*value)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Trait>, D::Error> {
        let names = Vec::<String>::deserialize(deserializer)?;
        Ok(names
            .iter()
            .filter_map(|wanted| Trait::ALL.into_iter().find(|known| name(*known) == *wanted))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample;

    #[test]
    fn the_sample_snapshot_is_valid_and_complete() {
        let snapshot = sample::snapshot();
        snapshot.validate().expect("the sample is a valid snapshot");
        assert_eq!(snapshot.travelers.len(), 4);
        for name in capability::ALL {
            assert!(snapshot.has_capability(name), "missing {name}");
        }
    }

    #[test]
    fn a_snapshot_round_trips_through_json_unchanged() {
        let snapshot = sample::snapshot();
        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let back: TravelSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back, snapshot);
    }

    #[test]
    fn unknown_fields_from_a_newer_desktop_are_ignored() {
        let mut value = serde_json::to_value(sample::snapshot()).unwrap();
        value["format_version"] = 3.into();
        value["weather"] = "drizzle".into();
        value["travelers"][0]["favourite_song"] = "humming".into();
        value["travelers"][0]["traits"]
            .as_array_mut()
            .unwrap()
            .push("SomeTraitFromTheFuture".into());
        let snapshot: TravelSnapshot = serde_json::from_value(value).unwrap();
        snapshot
            .validate()
            .expect("a compatible newer snapshot still reads");
        assert_eq!(snapshot.travelers[0].traits.len(), 3);
    }

    #[test]
    fn a_snapshot_that_needs_a_newer_reader_is_refused() {
        let mut snapshot = sample::snapshot();
        snapshot.format_version = 2;
        snapshot.minimum_reader_version = 2;
        assert_eq!(
            snapshot.validate(),
            Err(SnapshotError::Version(VersionError::TooNew { needed: 2 }))
        );
    }

    #[test]
    fn structural_problems_are_refused() {
        let base = sample::snapshot();
        let first = base.travelers[0].id;

        let mut empty = base.clone();
        empty.travelers.clear();
        empty.bonds.clear();
        assert_eq!(empty.validate(), Err(SnapshotError::NoTravelers));

        let mut doubled = base.clone();
        doubled.travelers.push(doubled.travelers[0].clone());
        assert_eq!(
            doubled.validate(),
            Err(SnapshotError::DuplicateTraveler(first))
        );

        let mut crowded = base.clone();
        while crowded.travelers.len() <= MAX_TRAVELERS {
            let mut extra = crowded.travelers[0].clone();
            extra.id = crowded.travelers.iter().map(|t| t.id).max().unwrap() + 1;
            crowded.travelers.push(extra);
        }
        assert_eq!(
            crowded.validate(),
            Err(SnapshotError::TooManyTravelers(MAX_TRAVELERS + 1))
        );

        let mut renamed = base.clone();
        renamed.travelers[0].name = "Line\nbreak".into();
        assert_eq!(renamed.validate(), Err(SnapshotError::BadName(first)));

        let mut own_parent = base.clone();
        own_parent.travelers[0].role = CreatureRole::Mini { parent_id: first };
        assert_eq!(own_parent.validate(), Err(SnapshotError::OwnParent(first)));

        let mut stranger = base.clone();
        stranger.bonds[0].b = CreatureId::MAX;
        assert!(matches!(
            stranger.validate(),
            Err(SnapshotError::BadBond { .. })
        ));

        let mut reversed = base.clone();
        let bond = &mut reversed.bonds[0];
        (bond.a, bond.b) = (bond.b, bond.a);
        assert!(matches!(
            reversed.validate(),
            Err(SnapshotError::BadBond { .. })
        ));

        let mut twice = base.clone();
        twice.bonds.push(twice.bonds[0]);
        assert!(matches!(
            twice.validate(),
            Err(SnapshotError::DuplicateBond { .. })
        ));

        let mut odd_capability = base;
        odd_capability.capabilities.push("../../escape".into());
        assert_eq!(
            odd_capability.validate(),
            Err(SnapshotError::BadCapabilities)
        );
    }

    #[test]
    fn colony_tags_are_stable_and_reveal_nothing_of_the_seed() {
        let tag = ColonyTag::for_seed(&[7; 32]);
        assert_eq!(tag, ColonyTag::for_seed(&[7; 32]));
        assert_ne!(tag, ColonyTag::for_seed(&[8; 32]));
        assert_eq!(tag.public_id.len(), 16);
        assert!(!tag.public_id.contains("0707"));
    }

    #[test]
    fn bands_follow_the_scores() {
        assert_eq!(Level::from_score(0), Level::None);
        assert_eq!(Level::from_score(16), Level::Low);
        assert_eq!(Level::from_score(64), Level::Medium);
        assert_eq!(Level::from_score(128), Level::High);
        assert_eq!(Level::from_score(255), Level::High);
    }

    #[test]
    fn accessory_inks_survive_the_trip() {
        let ink = AccessoryInk {
            outline: [1, 2, 3, 255],
            deep: [4, 5, 6, 255],
            body: [7, 8, 9, 255],
            light: [10, 11, 12, 255],
            accent: [13, 14, 15, 128],
        };
        assert_eq!(AccessoryInk::from(formiga_art::TrinketInk::from(ink)), ink);
    }
}
