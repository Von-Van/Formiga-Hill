//! What Hill hands back when the colony comes home.

use crate::text::{is_display_text, is_identifier};
use crate::{SessionId, TravelSnapshot, VersionError, check_versions};
use formiga_core::CreatureId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Souvenirs one trip can bring home.
pub const MAX_SOUVENIRS: usize = 4;
const MAX_TITLE_CHARS: usize = 48;
const MAX_ID_LEN: usize = 96;
const MAX_VERSION_CHARS: usize = 64;

/// The colony's way home. Everything in it is a request: Desktop decides what to accept, and
/// nothing here can change who a companion is, who it knows, or how Desktop is set up. A trip
/// with no receipt at all, because Hill closed or crashed before writing one, ends exactly like
/// one that came back with nothing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReturnReceipt {
    pub format_version: u32,
    pub minimum_reader_version: u32,
    pub session_id: SessionId,
    /// The Hill release that wrote it, for diagnostics and for nothing else.
    pub hill_version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub returned_at_utc: OffsetDateTime,
    pub completion: Completion,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub souvenirs: Vec<Souvenir>,
    /// One line for Desktop's journal, if Desktop wants it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outing: Option<Outing>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completion {
    /// The colony took the train home.
    Clean,
    /// Hill stopped some other way but still managed to say so.
    Unclean,
}

/// A request for a keepsake or cosmetic, named by the content package that awarded it. Desktop
/// looks the item up in its own catalogue and ignores anything it does not recognise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Souvenir {
    pub package_id: String,
    pub item_id: String,
    pub title: String,
    /// The traveller it was given to, if it was given to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub for_creature: Option<CreatureId>,
}

/// "Came back from Formiga Hill", and from what.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outing {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReceiptError {
    #[error(transparent)]
    Version(#[from] VersionError),
    #[error("the receipt is for trip {found}, not trip {expected}")]
    WrongSession {
        expected: SessionId,
        found: SessionId,
    },
    #[error("the receipt's {0} is missing or malformed")]
    BadField(&'static str),
    #[error("a trip brings home at most {MAX_SOUVENIRS} souvenirs; this one lists {0}")]
    TooManySouvenirs(usize),
    #[error("souvenir {index}: {problem}")]
    BadSouvenir {
        index: usize,
        problem: SouvenirError,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SouvenirError {
    #[error("its package or item ID is malformed")]
    BadId,
    #[error("its title is blank, too long, or has control characters")]
    BadTitle,
    #[error("it names a companion that was not on the train")]
    UnknownCreature,
}

impl ReturnReceipt {
    pub fn new(
        session_id: SessionId,
        hill_version: impl Into<String>,
        returned_at_utc: OffsetDateTime,
        completion: Completion,
    ) -> Self {
        Self {
            format_version: crate::FORMAT_VERSION,
            minimum_reader_version: 1,
            session_id,
            hill_version: hill_version.into(),
            returned_at_utc,
            completion,
            souvenirs: Vec::new(),
            outing: None,
        }
    }

    /// Checks the receipt against the snapshot it answers. Desktop runs this before reading
    /// anything else in it; Hill runs it before writing one.
    pub fn validate(&self, snapshot: &TravelSnapshot) -> Result<(), ReceiptError> {
        check_versions(self.format_version, self.minimum_reader_version)?;
        if self.session_id != snapshot.session_id {
            return Err(ReceiptError::WrongSession {
                expected: snapshot.session_id,
                found: self.session_id,
            });
        }
        if !is_display_text(&self.hill_version, MAX_VERSION_CHARS) {
            return Err(ReceiptError::BadField("hill_version"));
        }
        if self.souvenirs.len() > MAX_SOUVENIRS {
            return Err(ReceiptError::TooManySouvenirs(self.souvenirs.len()));
        }
        for (index, souvenir) in self.souvenirs.iter().enumerate() {
            souvenir
                .check(snapshot)
                .map_err(|problem| ReceiptError::BadSouvenir { index, problem })?;
        }
        if let Some(outing) = &self.outing {
            let package_ok = outing
                .package_id
                .as_deref()
                .is_none_or(|id| is_identifier(id, MAX_ID_LEN));
            if !package_ok || !is_display_text(&outing.title, MAX_TITLE_CHARS) {
                return Err(ReceiptError::BadField("outing"));
            }
        }
        Ok(())
    }
}

impl Souvenir {
    fn check(&self, snapshot: &TravelSnapshot) -> Result<(), SouvenirError> {
        if !is_identifier(&self.package_id, MAX_ID_LEN) || !is_identifier(&self.item_id, MAX_ID_LEN)
        {
            return Err(SouvenirError::BadId);
        }
        if !is_display_text(&self.title, MAX_TITLE_CHARS) {
            return Err(SouvenirError::BadTitle);
        }
        if self
            .for_creature
            .is_some_and(|id| snapshot.traveler(id).is_none())
        {
            return Err(SouvenirError::UnknownCreature);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample;

    fn receipt_for(snapshot: &TravelSnapshot) -> ReturnReceipt {
        let mut receipt = ReturnReceipt::new(
            snapshot.session_id,
            "0.1.0",
            snapshot.created_at_utc,
            Completion::Clean,
        );
        receipt.souvenirs.push(Souvenir {
            package_id: "com.formiga.hill.station".into(),
            item_id: "platform-ticket".into(),
            title: "A platform ticket".into(),
            for_creature: Some(snapshot.travelers[0].id),
        });
        receipt.outing = Some(Outing {
            package_id: None,
            title: "Went to Formiga Hill".into(),
        });
        receipt
    }

    #[test]
    fn a_receipt_validates_against_its_own_trip() {
        let snapshot = sample::snapshot();
        let receipt = receipt_for(&snapshot);
        receipt.validate(&snapshot).expect("a well-formed receipt");
        let json = serde_json::to_string(&receipt).unwrap();
        assert_eq!(
            serde_json::from_str::<ReturnReceipt>(&json).unwrap(),
            receipt
        );
    }

    #[test]
    fn a_receipt_cannot_settle_another_trip() {
        let snapshot = sample::snapshot();
        let mut receipt = receipt_for(&snapshot);
        receipt.session_id = SessionId::from_bytes([9; 16]);
        assert!(matches!(
            receipt.validate(&snapshot),
            Err(ReceiptError::WrongSession { .. })
        ));
    }

    #[test]
    fn souvenirs_are_bounded_and_well_formed() {
        let snapshot = sample::snapshot();

        let mut greedy = receipt_for(&snapshot);
        let souvenir = greedy.souvenirs[0].clone();
        greedy.souvenirs = vec![souvenir; MAX_SOUVENIRS + 1];
        assert_eq!(
            greedy.validate(&snapshot),
            Err(ReceiptError::TooManySouvenirs(MAX_SOUVENIRS + 1))
        );

        let mut pathlike = receipt_for(&snapshot);
        pathlike.souvenirs[0].item_id = "../../colony.json".into();
        assert_eq!(
            pathlike.validate(&snapshot),
            Err(ReceiptError::BadSouvenir {
                index: 0,
                problem: SouvenirError::BadId
            })
        );

        let mut stranger = receipt_for(&snapshot);
        stranger.souvenirs[0].for_creature = Some(CreatureId::MAX);
        assert_eq!(
            stranger.validate(&snapshot),
            Err(ReceiptError::BadSouvenir {
                index: 0,
                problem: SouvenirError::UnknownCreature
            })
        );

        let mut shouting = receipt_for(&snapshot);
        shouting.outing.as_mut().unwrap().title = "x".repeat(MAX_TITLE_CHARS + 1);
        assert_eq!(
            shouting.validate(&snapshot),
            Err(ReceiptError::BadField("outing"))
        );
    }
}
