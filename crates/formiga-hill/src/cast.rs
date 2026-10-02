//! The travellers as Hill knows them: each one ready to draw, and the facts about who it is and
//! how it gets on with the others, read once from the snapshot Desktop sent.

use formiga_art::AccessoryArt;
use formiga_core::{AppearanceGenome, Creature};
use formiga_travel::{Band, TravelError, TravelRole, TravelSnapshot, Traveler, TravelerId};

/// A traveller's id, as the rest of Hill passes it about.
pub type Id = u64;

pub struct Member {
    pub id: Id,
    pub name: String,
    /// Desktop's stand-in for the companion: its look, temperament, pace, habits and what it
    /// wears, for the art crate and anything else that reads a whole creature.
    pub creature: Creature,
    pub dress: Option<AccessoryArt>,
    pub traveler: Traveler,
}

impl Member {
    pub fn genome(&self) -> &AppearanceGenome {
        &self.creature.appearance
    }

    pub fn parent(&self) -> Option<Id> {
        match self.traveler.role {
            TravelRole::Adult => None,
            TravelRole::Mini { parent_id } => Some(parent_id.0),
        }
    }

    pub fn kind(&self) -> formiga_core::TemperamentKind {
        self.traveler.character.temperament.into()
    }

    pub fn axes(&self) -> formiga_core::Axes {
        self.traveler.character.axes.into()
    }

    pub fn has_habit(&self, habit: formiga_core::Habit) -> bool {
        self.traveler
            .habits
            .iter()
            .any(|own| formiga_core::Habit::from(*own) == habit)
    }

    /// Whether Desktop's notebook names this trait for it, whatever the capitals.
    pub fn has_trait(&self, label: &str) -> bool {
        self.traveler
            .character
            .traits
            .iter()
            .any(|own| own.eq_ignore_ascii_case(label))
    }
}

/// How one pair gets on, under Hill's names for Desktop's bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bond {
    /// Desktop's affinity: how fond they are of each other.
    pub warmth: Band,
    pub familiarity: Band,
    pub playfulness: Band,
    /// Desktop's avoidance: how much they get under each other's feet.
    pub friction: Band,
}

pub struct Cast {
    pub snapshot: TravelSnapshot,
    pub members: Vec<Member>,
}

impl Cast {
    /// Everyone on the train, ready for the stage. Fails only if Desktop sent a look this build
    /// cannot draw, which is a reason to refuse the trip rather than show someone wrong.
    pub fn new(snapshot: TravelSnapshot) -> Result<Self, TravelError> {
        let members = snapshot
            .travelers
            .iter()
            .map(|traveler| {
                Ok(Member {
                    id: traveler.id.0,
                    name: traveler.name.clone(),
                    creature: traveler.to_creature()?,
                    dress: traveler.accessory.map(|accessory| accessory.to_art()),
                    traveler: traveler.clone(),
                })
            })
            .collect::<Result<_, TravelError>>()?;
        Ok(Self { snapshot, members })
    }

    pub fn reduce_motion(&self) -> bool {
        self.snapshot.presentation.reduce_motion
    }

    pub fn member(&self, id: Id) -> Option<&Member> {
        self.members.iter().find(|member| member.id == id)
    }

    pub fn ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.members.iter().map(|member| member.id)
    }

    /// Its closest friend among `candidates`: the warmest bond not outweighed by friction.
    pub fn closest_friend(&self, id: Id, candidates: impl IntoIterator<Item = Id>) -> Option<Id> {
        candidates
            .into_iter()
            .filter(|&other| other != id)
            .filter_map(|other| self.bond(id, other).map(|bond| (other, bond)))
            .filter(|(_, bond)| bond.warmth >= Band::Medium && bond.friction < bond.warmth)
            .max_by_key(|(other, bond)| (bond.warmth, bond.familiarity, std::cmp::Reverse(*other)))
            .map(|(other, _)| other)
    }

    /// Someone among `candidates` it plays with.
    pub fn playmate(&self, id: Id, candidates: impl IntoIterator<Item = Id>) -> Option<Id> {
        candidates
            .into_iter()
            .filter(|&other| other != id)
            .filter_map(|other| self.bond(id, other).map(|bond| (other, bond)))
            .filter(|(_, bond)| bond.playfulness >= Band::Medium && bond.friction <= bond.warmth)
            .max_by_key(|(other, bond)| (bond.playfulness, bond.warmth, std::cmp::Reverse(*other)))
            .map(|(other, _)| other)
    }

    /// Whether a pair rubs each other up the wrong way enough to keep apart.
    pub fn at_odds(&self, a: Id, b: Id) -> bool {
        self.bond(a, b)
            .is_some_and(|bond| bond.friction >= Band::Medium && bond.friction > bond.warmth)
    }

    pub fn bond(&self, a: Id, b: Id) -> Option<Bond> {
        let (low, high) = if a < b { (a, b) } else { (b, a) };
        self.snapshot
            .relationships
            .iter()
            .find(|pair| pair.a == TravelerId(low) && pair.b == TravelerId(high))
            .map(|pair| Bond {
                warmth: pair.affinity,
                familiarity: pair.familiarity,
                playfulness: pair.playfulness,
                friction: pair.avoidance,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everyone_in_the_sample_can_be_drawn() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        assert_eq!(cast.members.len(), cast.snapshot.travelers.len());
        for member in &cast.members {
            assert_eq!(member.creature.id, member.id);
            assert_eq!(member.name, member.creature.name);
        }
    }

    #[test]
    fn bonds_read_the_same_whichever_way_round() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let ids: Vec<_> = cast.ids().collect();
        let (a, b) = (ids[0], ids[1]);
        assert!(cast.bond(a, b).is_some());
        assert_eq!(cast.bond(a, b), cast.bond(b, a));
        assert_eq!(cast.bond(a, a), None);
    }

    #[test]
    fn minis_know_their_parents() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for member in &cast.members {
            if let Some(parent) = member.parent() {
                assert!(
                    cast.member(parent).is_some(),
                    "{} came without its adult",
                    member.name
                );
            }
        }
    }
}
