//! The Green's dress-up box: Hill's own costume pieces, worn over a companion for the visit only.
//! A costume never touches who a companion is. Desktop's own accessories stay as they are, the
//! costume is drawn over them, and nothing about it goes home: the pieces live in the box at the
//! Hill. Each companion takes to being dressed up in its own way.

pub mod art;

use crate::character::{Beat, Character, Cue};
use formiga_art::ExpressionKind;
use formiga_core::{Gesture, TemperamentKind};

/// Where on a companion a piece is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// On top of its head, between its ears: `art::anchor` is the point that sits on the crown.
    Crown,
    /// Round its neck, under its face: `art::anchor` is the point that sits at the throat.
    Neck,
}

#[derive(Debug)]
pub struct Piece {
    pub id: &'static str,
    pub name: &'static str,
    pub slot: Slot,
}

pub const PIECES: [Piece; 8] = [
    Piece {
        id: "paper_crown",
        name: "A paper crown",
        slot: Slot::Crown,
    },
    Piece {
        id: "straw_hat",
        name: "A straw sun hat",
        slot: Slot::Crown,
    },
    Piece {
        id: "top_hat",
        name: "A tall top hat",
        slot: Slot::Crown,
    },
    Piece {
        id: "daisy_chain",
        name: "A daisy chain",
        slot: Slot::Crown,
    },
    Piece {
        id: "party_hat",
        name: "A party hat",
        slot: Slot::Crown,
    },
    Piece {
        id: "wizard_hat",
        name: "A wizard's hat",
        slot: Slot::Crown,
    },
    Piece {
        id: "bow_tie",
        name: "A spotted bow tie",
        slot: Slot::Neck,
    },
    Piece {
        id: "knitted_scarf",
        name: "A knitted scarf",
        slot: Slot::Neck,
    },
];

pub fn piece(id: &str) -> Option<&'static Piece> {
    PIECES.iter().find(|piece| piece.id == id)
}

/// How a companion takes to being dressed up: its own way, from its temperament.
pub fn dressed(character: &Character) -> Vec<Beat> {
    let with = |mut beat: Beat, cue: Cue| {
        beat.cue = Some(cue);
        beat
    };
    match character.kind {
        TemperamentKind::Showoff => vec![
            with(
                Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.6),
                Cue::Sparkle,
            ),
            Beat::new(Gesture::Bop, ExpressionKind::Joy, 1.0),
        ],
        TemperamentKind::Grump => vec![with(
            Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.8),
            Cue::Huff,
        )],
        TemperamentKind::Wallflower => vec![
            Beat::new(Gesture::Peek, ExpressionKind::Worried, 1.4),
            with(
                Beat::new(Gesture::Cover, ExpressionKind::Worried, 0.8),
                Cue::Heart,
            ),
        ],
        TemperamentKind::Sweetheart => vec![with(
            Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.2),
            Cue::Heart,
        )],
        TemperamentKind::Troublemaker => vec![
            Beat::new(Gesture::Bop, ExpressionKind::Smug, 1.4),
            with(
                Beat::new(Gesture::Strut, ExpressionKind::Smug, 0.8),
                Cue::Note,
            ),
        ],
        TemperamentKind::Lazybones => vec![Beat::new(Gesture::Yawn, ExpressionKind::Sleepy, 1.6)],
        TemperamentKind::Scholar => vec![Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.4)],
        TemperamentKind::Explorer => vec![
            Beat::new(Gesture::Balance, ExpressionKind::Curious, 1.0),
            with(
                Beat::new(Gesture::Cheer, ExpressionKind::Joy, 0.8),
                Cue::Exclaim,
            ),
        ],
        TemperamentKind::Guardian => {
            vec![Beat::new(Gesture::Stomp, ExpressionKind::Determined, 1.2)]
        }
        TemperamentKind::Oddball => vec![with(
            Beat::new(Gesture::Swoon, ExpressionKind::Joy, 1.6),
            Cue::Sparkle,
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_piece_has_its_own_id_and_a_picture() {
        let ids: HashSet<&str> = PIECES.iter().map(|piece| piece.id).collect();
        assert_eq!(ids.len(), PIECES.len());
        for piece in &PIECES {
            let sprite = art::sprite(piece.id);
            assert!(
                sprite.alpha_bounds().is_some(),
                "{} draws nothing",
                piece.id
            );
            let (x, y) = art::anchor(piece.id);
            assert!(
                x >= 0 && y >= 0 && x < sprite.width() as i32 && y < sprite.height() as i32,
                "{}'s anchor is off its picture",
                piece.id
            );
        }
    }

    #[test]
    fn everyone_takes_to_a_costume_somehow() {
        let cast = crate::cast::Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for member in &cast.members {
            assert!(!dressed(&Character::of(member)).is_empty());
        }
    }
}
