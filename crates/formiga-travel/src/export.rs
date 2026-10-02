//! Desktop's side of the trip: projecting a save into a snapshot.
//!
//! This lives in the shared crate rather than in Desktop so the projection is written and tested
//! once, beside the format it produces. Until Desktop calls it, Hill's `--from-save` development
//! mode does, on a copy of a colony it only ever reads.

use crate::snapshot::{
    AccessoryInk, Bond, ColonyTag, MAX_TRAITS, MAX_TRAVELERS, Presentation, TravelSnapshot,
    Traveler, WornAccessory, capability,
};
use crate::{FORMAT_VERSION, SessionId};
use formiga_art::{AccessoryArt, Palette, palette_for};
use formiga_core::{MAX_HABITS, SaveFile, default_creature_name, validate_creature_name};
use time::OffsetDateTime;

pub struct ExportOptions {
    pub session_id: SessionId,
    pub created_at_utc: OffsetDateTime,
    /// The exporting Desktop's version string, such as `0.66.1`.
    pub desktop_version: String,
}

/// The whole colony, ready to board. Takes everyone: choosing who travels is a later decision
/// (see the open questions in `docs/DESIGN.md`), and the format already allows a partial train.
pub fn export_snapshot(save: &SaveFile, options: ExportOptions) -> TravelSnapshot {
    let palettes: Vec<Palette> = save
        .creatures
        .iter()
        .map(|creature| palette_for(&creature.appearance))
        .collect();

    let mut creatures: Vec<_> = save.creatures.iter().collect();
    creatures.sort_by_key(|creature| creature.colony_order);
    let travelers: Vec<Traveler> = creatures
        .into_iter()
        .take(MAX_TRAVELERS)
        .map(|creature| Traveler {
            id: creature.id,
            name: validate_creature_name(&creature.name).unwrap_or_else(|_| {
                default_creature_name(save.colony_seed, creature.generation, &[])
            }),
            role: creature.role,
            generation: creature.generation,
            born_at_utc: creature.born_at_utc,
            appearance: creature.appearance.clone(),
            temperament: creature.temperament(),
            traits: creature.traits().into_iter().take(MAX_TRAITS).collect(),
            habits: creature
                .memory
                .habits
                .iter()
                .copied()
                .take(MAX_HABITS)
                .collect(),
            accessory: creature.accessory.map(|item| WornAccessory {
                item,
                ink: AccessoryInk::from(
                    AccessoryArt::resolve(item, save.colony_seed, &palettes).ink,
                ),
            }),
        })
        .collect();

    let traveling = |id| travelers.iter().any(|traveler| traveler.id == id);
    let mut bonds: Vec<Bond> = save
        .relationships
        .iter()
        .filter(|relationship| traveling(relationship.a) && traveling(relationship.b))
        .filter_map(Bond::from_relationship)
        .collect();
    bonds.sort_by_key(|bond| (bond.a, bond.b));
    bonds.dedup_by_key(|bond| (bond.a, bond.b));

    TravelSnapshot {
        format_version: FORMAT_VERSION,
        minimum_reader_version: 1,
        session_id: options.session_id,
        created_at_utc: options.created_at_utc,
        desktop_version: options.desktop_version,
        colony: ColonyTag::for_seed(&save.colony_seed),
        travelers,
        bonds,
        presentation: Presentation {
            reduce_motion: save.settings.reduce_motion,
            theme: save.companion.appearance.theme.into(),
            text_scale_percent: save.companion.appearance.text_scale,
        },
        capabilities: capability::ALL
            .iter()
            .map(|name| name.to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample;

    #[test]
    fn every_companion_boards_as_itself() {
        let save = sample::save();
        let snapshot = sample::snapshot();
        assert_eq!(snapshot.travelers.len(), save.creatures.len());
        for traveler in &snapshot.travelers {
            let creature = save
                .creatures
                .iter()
                .find(|creature| creature.id == traveler.id)
                .expect("every traveller is a colony member");
            assert_eq!(traveler.name, creature.name);
            assert_eq!(traveler.appearance, creature.appearance);
            assert_eq!(traveler.temperament, creature.temperament());
            assert_eq!(traveler.traits, creature.traits());
            assert_eq!(traveler.habits, creature.memory.habits);
            assert_eq!(traveler.accessory.map(|worn| worn.item), creature.accessory);
        }
    }

    #[test]
    fn worn_accessories_keep_desktops_inks() {
        let save = sample::save();
        let snapshot = sample::snapshot();
        let palettes: Vec<_> = save
            .creatures
            .iter()
            .map(|creature| palette_for(&creature.appearance))
            .collect();
        let worn: Vec<_> = snapshot
            .travelers
            .iter()
            .filter_map(|traveler| traveler.accessory)
            .collect();
        assert!(!worn.is_empty(), "the sample dresses someone up");
        for accessory in worn {
            let desktop = AccessoryArt::resolve(accessory.item, save.colony_seed, &palettes);
            assert_eq!(accessory.art(), desktop);
        }
    }

    #[test]
    fn nothing_machine_specific_crosses() {
        let json = serde_json::to_string(&sample::snapshot()).unwrap();
        for field in [
            "colony_seed",
            "behavior_seed",
            "monitor",
            "display_key",
            "position",
            "journal",
            "habitat",
            "settings",
            "routines",
        ] {
            assert!(!json.contains(field), "{field} leaked into the snapshot");
        }
    }

    #[test]
    fn bonds_cover_each_pair_of_travellers_once() {
        let snapshot = sample::snapshot();
        let n = snapshot.travelers.len();
        assert_eq!(snapshot.bonds.len(), n * (n - 1) / 2);
    }
}
