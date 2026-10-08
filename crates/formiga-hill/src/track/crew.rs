//! Who came along to the old track, and what each is like at a heap and with a map: read from
//! who each is, never written for anyone in particular.
//!
//! At a heap, someone strong lifts the beams and boulders on its own, and anyone else needs a
//! second pair of paws for them. A patient one eases things out so gently that nothing above
//! them comes crashing down; an impulsive one yanks, quick and cheap, and whatever is breakable
//! inside what it yanks gets jolted. Someone curious peeks under and into things before anything
//! is moved. A little one squeezes into the gap under a board and brings out what is there
//! without disturbing anything. One that looks food over looks into every sack, crate and tin,
//! and knows what is in each. With a map, a scholar reads the faded words, an explorer has a
//! hunch at every fork, someone suspicious doubts aloud before a wrong turn (and now and then a
//! right one), and someone curious spots a landmark far down a way.

use super::heap::How;
use crate::cast::Id;
use crate::character::Character;
use formiga_core::{Habit, TemperamentKind};

/// Strong enough to lift a beam or a boulder on its own.
pub fn strong(character: &Character) -> bool {
    (character.axes.energy + character.axes.feistiness) / 2.0 >= 0.6
}

/// Patient enough to ease something out without bringing the heap down.
pub fn patient(character: &Character) -> bool {
    character.axes.impulsiveness <= 0.35
}

/// Too impatient to do anything but yank.
pub fn impulsive(character: &Character) -> bool {
    character.axes.impulsiveness >= 0.65
}

/// Peeks under things, and spots what is far off.
pub fn curious(character: &Character) -> bool {
    character.kind == TemperamentKind::Explorer || character.axes.curiosity >= 0.6
}

/// Small enough to squeeze into a gap.
pub fn little(character: &Character) -> bool {
    character.parent.is_some()
}

/// Bold enough to climb for the high branches, or take the steep way.
pub fn bold(character: &Character) -> bool {
    character.axes.boldness >= 0.65
}

/// Looks things over: into every sack, crate and tin.
pub fn inspector(character: &Character) -> bool {
    character.habits.contains(&Habit::LooksFoodOver)
}

/// Doubts aloud.
pub fn suspicious(character: &Character) -> bool {
    character.axes.suspicion >= 0.65
}

/// Has a hunch which way.
pub fn explorer(character: &Character) -> bool {
    character.kind == TemperamentKind::Explorer
}

/// Reads the faded words on a map: a scholar, or anyone both curious and patient enough.
pub fn scholar(character: &Character) -> bool {
    character.kind == TemperamentKind::Scholar
        || (character.axes.curiosity >= 0.7 && character.axes.impulsiveness <= 0.35)
}

/// How a companion lifts things.
pub fn how(character: &Character) -> How {
    if patient(character) {
        How::Gentle
    } else if impulsive(character) {
        How::Yank
    } else {
        How::Plain
    }
}

/// Something someone can do at a heap, chosen before clicking what to do it to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    /// Lift something off, in its own way (see `how`).
    Lift(Id),
    /// Look under it and into it.
    Peek(Id),
    /// Squeeze into the gap under it.
    Squeeze(Id),
}

impl Hand {
    pub fn who(self) -> Id {
        match self {
            Self::Lift(who) | Self::Peek(who) | Self::Squeeze(who) => who,
        }
    }
}

/// Who came along, read once.
#[derive(Clone, Debug)]
pub struct Crew {
    pub party: Vec<Id>,
    pub characters: Vec<Character>,
    pub close_pair: bool,
}

impl Crew {
    pub fn new(party: Vec<Id>, characters: Vec<Character>, close_pair: bool) -> Self {
        let close_pair = close_pair && party.len() >= 2;
        Self {
            party,
            characters,
            close_pair,
        }
    }

    pub fn character(&self, id: Id) -> Option<&Character> {
        self.party
            .iter()
            .position(|member| *member == id)
            .and_then(|index| self.characters.get(index))
    }

    fn members(&self) -> impl Iterator<Item = (Id, &Character)> {
        self.party.iter().copied().zip(self.characters.iter())
    }

    /// The first in the party who is like this.
    pub fn first(&self, test: fn(&Character) -> bool) -> Option<Id> {
        self.members()
            .find(|(_, character)| test(character))
            .map(|(id, _)| id)
    }

    /// Everything the party can do at a heap: each one's way of lifting, then a peek for each
    /// curious one, and squeezing in for each little one.
    pub fn hands(&self) -> Vec<Hand> {
        let lifts = self.party.iter().map(|id| Hand::Lift(*id));
        let peeks = self
            .members()
            .filter(|(_, character)| curious(character))
            .map(|(id, _)| Hand::Peek(id));
        let squeezes = self
            .members()
            .filter(|(_, character)| little(character))
            .map(|(id, _)| Hand::Squeeze(id));
        lifts.chain(peeks).chain(squeezes).collect()
    }

    /// How `lifter` lifts something, heavy or not, and who lifts it: on its own, in its own way;
    /// or, for something heavy and a lifter not strong enough, together with the other, as
    /// steadily as the steadier of them (and a close pair steadily always). `None` if it is too
    /// heavy for whoever is there.
    pub fn lift(&self, lifter: Id, heavy: bool) -> Option<(How, Vec<Id>)> {
        let character = self.character(lifter)?;
        if !heavy || strong(character) {
            return Some((how(character), vec![lifter]));
        }
        let (other, helper) = self.members().find(|(id, _)| *id != lifter)?;
        let steady = self.close_pair || patient(character) || patient(helper);
        let how = if steady { How::Gentle } else { How::Plain };
        Some((how, vec![lifter, other]))
    }
}

/// What a companion is like at a heap, for the person choosing who to bring: the things it does
/// that others don't, from who it is.
pub fn scavenger(character: &Character) -> String {
    let mut ways: Vec<&str> = Vec::new();
    if little(character) {
        ways.push("small enough to squeeze into the gaps");
    }
    if strong(character) {
        ways.push("strong enough for beams and boulders");
    }
    match how(character) {
        How::Gentle => ways.push("eases things out without bringing the heap down"),
        How::Yank => ways.push("yanks things out, quick and careless"),
        How::Plain => {}
    }
    if curious(character) {
        ways.push("peeks under things first");
    }
    if inspector(character) {
        ways.push("looks into every sack, crate and tin");
    }
    if ways.is_empty() {
        ways.push("a steady pair of paws");
    }
    sentence(&ways)
}

/// How a companion reads a map, for the person choosing who to bring on a treasure hunt.
pub fn reader(character: &Character) -> String {
    let mut ways: Vec<&str> = Vec::new();
    if scholar(character) {
        ways.push("reads the faded words");
    }
    if explorer(character) {
        ways.push("has a hunch at every fork");
    }
    if curious(character) {
        ways.push("spots a landmark far off");
    }
    if suspicious(character) {
        ways.push("doubts aloud before a wrong turn, and sometimes a right one");
    }
    if ways.is_empty() {
        ways.push("happy to follow wherever the map goes");
    }
    sentence(&ways)
}

/// The first two of `ways`, as a sentence.
fn sentence(ways: &[&str]) -> String {
    let said = ways
        .iter()
        .take(2)
        .copied()
        .collect::<Vec<_>>()
        .join(", and ");
    let mut letters = said.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect::<String>() + ".")
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::{Cast, sample};
    use std::collections::BTreeSet;

    fn crew(cast: &Cast, names: &[&str], close_pair: bool) -> Crew {
        let members: Vec<_> = names
            .iter()
            .map(|name| {
                cast.members
                    .iter()
                    .find(|member| member.name == *name)
                    .unwrap_or_else(|| panic!("no {name} in the sample"))
            })
            .collect();
        Crew::new(
            members.iter().map(|member| member.id).collect(),
            members.iter().map(|member| Character::of(member)).collect(),
            close_pair,
        )
    }

    fn id(cast: &Cast, name: &str) -> Id {
        cast.members.iter().find(|m| m.name == name).unwrap().id
    }

    #[test]
    fn the_strong_lift_heavy_things_alone_and_anyone_else_needs_a_second_pair_of_paws() {
        let cast = sample();
        let biscuit = crew(&cast, &["Biscuit"], false);
        assert_eq!(
            biscuit.lift(id(&cast, "Biscuit"), true),
            Some((How::Yank, vec![id(&cast, "Biscuit")])),
            "strong, and impatient with it"
        );
        let mochi = crew(&cast, &["Mochi"], false);
        assert_eq!(
            mochi.lift(id(&cast, "Mochi"), true),
            None,
            "too heavy alone"
        );
        assert_eq!(
            mochi.lift(id(&cast, "Mochi"), false),
            Some((How::Gentle, vec![id(&cast, "Mochi")]))
        );
        let pair = crew(&cast, &["Fig", "Tansy"], false);
        assert_eq!(
            pair.lift(id(&cast, "Fig"), true),
            Some((How::Plain, vec![id(&cast, "Fig"), id(&cast, "Tansy")])),
            "two together manage it"
        );
        let close = crew(&cast, &["Fig", "Tansy"], true);
        assert_eq!(
            close.lift(id(&cast, "Fig"), true).map(|(how, _)| how),
            Some(How::Gentle),
            "and a close pair steadily"
        );
    }

    #[test]
    fn each_companion_brings_its_own_ways_to_a_heap() {
        let cast = sample();
        let hands = |names: &[&str]| crew(&cast, names, false).hands();
        assert_eq!(
            hands(&["Mochi", "Pip"]),
            vec![
                Hand::Lift(id(&cast, "Mochi")),
                Hand::Lift(id(&cast, "Pip")),
                Hand::Squeeze(id(&cast, "Pip")),
            ],
            "only the little one squeezes in"
        );
        assert!(hands(&["Tansy"]).contains(&Hand::Peek(id(&cast, "Tansy"))));
        assert!(
            !hands(&["Fig"])
                .iter()
                .any(|hand| matches!(hand, Hand::Peek(_)))
        );
        let ways: Vec<How> = ["Mochi", "Fig", "Biscuit"]
            .iter()
            .map(|name| {
                how(&Character::of(
                    cast.members.iter().find(|m| m.name == *name).unwrap(),
                ))
            })
            .collect();
        assert_eq!(ways, [How::Gentle, How::Plain, How::Yank]);
        assert_eq!(
            crew(&cast, &["Fig", "Biscuit"], false).first(inspector),
            Some(id(&cast, "Biscuit"))
        );
    }

    #[test]
    fn readers_and_scavengers_are_told_apart_from_who_they_are() {
        let cast = sample();
        let scavengers: BTreeSet<String> = cast
            .members
            .iter()
            .map(|member| scavenger(&Character::of(member)))
            .collect();
        assert!(scavengers.len() >= 4, "{scavengers:?}");
        let readers: BTreeSet<String> = cast
            .members
            .iter()
            .map(|member| reader(&Character::of(member)))
            .collect();
        assert!(readers.len() >= 2, "{readers:?}");
        let mut scholarly = Character::of(&cast.members[0]);
        scholarly.kind = TemperamentKind::Scholar;
        assert!(reader(&scholarly).starts_with("Reads the faded words"));
    }
}
