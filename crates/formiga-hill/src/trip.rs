//! Hill's side of a trip, as Desktop's `formiga-travel` lays it out.
//!
//! Desktop starts Hill with `--formiga-travel <session directory>`. Hill reads the snapshot there,
//! answers it once with an acknowledgement, keeps an eye out for a recall while the visit runs,
//! and on the way home writes one receipt. Desktop never depends on any of it: whatever goes
//! wrong, the colony goes home as it left.

use crate::cast::Cast;
use anyhow::{Context, Result, bail};
use formiga_travel::{
    ACK_FILE, AckRefusal, Acknowledgement, Capability, RECALL_FILE, RECEIPT_FILE, ReturnEffect,
    ReturnReceipt, SNAPSHOT_FILE, SessionId, SnapshotSeal, TRAVEL_FORMAT_VERSION, TravelError,
    TravelSnapshot, decode, limits, read_bounded, sha256_hex, write_document,
};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

pub const HILL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A trip in progress: where its files are, and the seal every answer must carry.
pub struct Trip {
    dir: PathBuf,
    seal: SnapshotSeal,
    arrived_at_utc: OffsetDateTime,
    /// Whether this Desktop records visits; Hill asks only for what is offered.
    records_visits: bool,
    /// The souvenirs this Desktop can keep, if it offers to keep any.
    accepts_souvenirs: Vec<String>,
}

/// Reads the snapshot Desktop left in `dir` and answers it: accepted, with the colony ready for
/// the stage, or refused, with the reason written for Desktop and returned as an error. `busy`
/// when another Hill is already hosting a colony, which refuses it whatever it holds.
pub fn arrive(dir: &Path, busy: bool) -> Result<(Trip, Cast)> {
    // The trip is named by its directory, and nothing else about the path is trusted.
    let session = dir
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(SessionId::parse)
        .with_context(|| format!("{} is not a trip's directory", dir.display()))?;
    let bytes = read_bounded(&dir.join(SNAPSHOT_FILE), limits::MAX_SNAPSHOT_BYTES)
        .context("could not read the colony's snapshot")?;
    let arrived_at_utc = OffsetDateTime::now_utc();

    let refuse = |refusal: AckRefusal, why: String| -> Result<(Trip, Cast)> {
        let seal = SnapshotSeal {
            session_id: session.clone(),
            snapshot_sha256: sha256_hex(&bytes),
            created_at_utc: arrived_at_utc,
        };
        write_document(
            &dir.join(ACK_FILE),
            &Acknowledgement::refused(&seal, HILL_VERSION, refusal),
        )?;
        bail!("Formiga Hill could not take the colony: {why}")
    };

    if busy {
        return refuse(
            AckRefusal::Busy,
            "another Hill is already hosting a colony".into(),
        );
    }
    let snapshot = match decode::<TravelSnapshot>(&bytes) {
        Ok(snapshot) if snapshot.session_id == session => snapshot,
        Ok(_) => {
            return refuse(
                AckRefusal::Invalid,
                "the snapshot is for another trip".into(),
            );
        }
        Err(TravelError::UnsupportedVersion { needs, .. }) => {
            return refuse(
                AckRefusal::UnsupportedVersion {
                    reads: TRAVEL_FORMAT_VERSION,
                },
                format!(
                    "it needs travel version {needs}, and this Hill reads {TRAVEL_FORMAT_VERSION}"
                ),
            );
        }
        Err(error) => return refuse(AckRefusal::Invalid, error.to_string()),
    };
    let seal = SnapshotSeal::of(&snapshot, &bytes);
    let records_visits = snapshot.offers(Capability::VisitRecord);
    let accepts_souvenirs: Vec<String> = snapshot
        .accepts_souvenirs
        .iter()
        .filter(|id| snapshot.accepts_souvenir(id))
        .cloned()
        .collect();
    let cast = match Cast::new(snapshot) {
        Ok(cast) => cast,
        Err(error) => return refuse(AckRefusal::Invalid, error.to_string()),
    };
    write_document(
        &dir.join(ACK_FILE),
        &Acknowledgement::accepted(&seal, HILL_VERSION),
    )?;
    let trip = Trip {
        dir: dir.to_owned(),
        seal,
        arrived_at_utc,
        records_visits,
        accepts_souvenirs,
    };
    Ok((trip, cast))
}

impl Trip {
    /// Desktop has taken the colony home without waiting, by leaving a recall or by sweeping the
    /// trip away: either way the visit ends, with no receipt.
    pub fn recalled(&self) -> bool {
        self.dir.join(RECALL_FILE).exists() || !self.dir.join(SNAPSHOT_FILE).exists()
    }

    /// The souvenirs the colony has `kept` that this Desktop keeps, in the order of Hill's
    /// catalogue: those the receipt takes home.
    pub fn souvenirs_going_home(&self, kept: &[String]) -> Vec<&'static str> {
        crate::story::souvenirs::ids()
            .into_iter()
            .filter(|id| {
                kept.iter().any(|own| own == id) && self.accepts_souvenirs.iter().any(|ok| ok == id)
            })
            .collect()
    }

    /// The receipt for the way home. Written once, and only ever with things Desktop offered to
    /// take: the visit, and every souvenir the colony has `kept` at the Hill that this Desktop
    /// can keep. Every one kept, not just this visit's, so one whose trip ended without a
    /// receipt still comes home next time; Desktop keeps each once and ignores a repeat.
    pub fn come_home(&self, kept: &[String]) -> Result<()> {
        let left_at_utc = OffsetDateTime::now_utc();
        let mut effects = Vec::new();
        if self.records_visits {
            effects.push(ReturnEffect::Visit {
                arrived_at_utc: self.arrived_at_utc,
                left_at_utc,
            });
        }
        for id in self.souvenirs_going_home(kept) {
            if effects.len() >= limits::MAX_EFFECTS {
                break;
            }
            effects.push(ReturnEffect::Souvenir { id: id.to_owned() });
        }
        write_document(
            &self.dir.join(RECEIPT_FILE),
            &ReturnReceipt::new(&self.seal, left_at_utc, HILL_VERSION, effects),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_travel::read_document;

    /// A trip directory as Desktop would leave it, in a scratch folder of its own.
    fn left_by_desktop(snapshot: &TravelSnapshot) -> PathBuf {
        let scratch = std::env::temp_dir().join(format!(
            "formiga-hill-trip-{}",
            SessionId::generate().unwrap()
        ));
        let dir = scratch.join(snapshot.session_id.as_str());
        std::fs::create_dir_all(&dir).unwrap();
        write_document(&dir.join(SNAPSHOT_FILE), snapshot).unwrap();
        dir
    }

    #[test]
    fn a_good_snapshot_is_accepted_and_answered_with_a_receipt() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        let (trip, cast) = arrive(&dir, false).unwrap();
        assert_eq!(cast.members.len(), snapshot.travelers.len());

        let bytes = std::fs::read(dir.join(SNAPSHOT_FILE)).unwrap();
        let seal = SnapshotSeal::of(&snapshot, &bytes);
        let ack: Acknowledgement = read_document(&dir.join(ACK_FILE)).unwrap();
        assert!(ack.accepted && ack.answers(&seal));

        assert!(!trip.recalled());
        trip.come_home(&[]).unwrap();
        let receipt: ReturnReceipt = read_document(&dir.join(RECEIPT_FILE)).unwrap();
        assert!(receipt.answers(&seal));
        assert!(matches!(receipt.effects[..], [ReturnEffect::Visit { .. }]));
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    /// The souvenirs a receipt brings home: every one kept that this Desktop keeps, whichever
    /// trip earned it.
    fn souvenirs_home(offered: bool, accepts: &[&str], kept: &[&str]) -> Vec<String> {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot
            .capabilities
            .retain(|c| *c != Capability::Souvenirs);
        if offered {
            snapshot.capabilities.push(Capability::Souvenirs);
        }
        snapshot.accepts_souvenirs = accepts.iter().map(|id| (*id).to_owned()).collect();
        let dir = left_by_desktop(&snapshot);
        let (trip, _) = arrive(&dir, false).unwrap();
        let kept: Vec<String> = kept.iter().map(|id| (*id).to_owned()).collect();
        trip.come_home(&kept).unwrap();
        let receipt: ReturnReceipt = read_document(&dir.join(RECEIPT_FILE)).unwrap();
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
        receipt
            .effects
            .into_iter()
            .filter_map(|effect| match effect {
                ReturnEffect::Souvenir { id } => Some(id),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn every_souvenir_kept_comes_home_when_desktop_keeps_it() {
        let all: Vec<&str> = crate::story::souvenirs::ids();
        assert_eq!(
            souvenirs_home(true, &all, &["fair_ticket", "picnic_ribbon"]),
            vec!["picnic_ribbon", "fair_ticket"],
            "in the catalogue's order"
        );
    }

    #[test]
    fn a_souvenir_desktop_does_not_list_stays_at_the_hill() {
        assert_eq!(
            souvenirs_home(true, &["picnic_ribbon"], &["picnic_ribbon", "chest_marble"]),
            vec!["picnic_ribbon"]
        );
    }

    #[test]
    fn a_desktop_that_does_not_offer_souvenirs_gets_none() {
        // As from a Desktop before 0.66.6, whatever it lists.
        assert!(souvenirs_home(false, &["picnic_ribbon"], &["picnic_ribbon"]).is_empty());
        assert!(souvenirs_home(true, &[], &["picnic_ribbon"]).is_empty());
    }

    /// Desktop draws and names every souvenir it lists from its own copy, so each must be one of
    /// Hill's. Hill may have more: Desktop lists a new one in a later release.
    #[test]
    fn every_souvenir_desktop_knows_is_one_of_hills() {
        let ours = crate::story::souvenirs::ids();
        for souvenir in formiga_core::Souvenir::ALL {
            assert!(
                ours.contains(&souvenir.id()),
                "{} is not Hill's",
                souvenir.id()
            );
        }
    }

    #[test]
    fn a_snapshot_for_another_trip_is_refused() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        let elsewhere = dir.with_file_name(SessionId::generate().unwrap().as_str());
        std::fs::rename(&dir, &elsewhere).unwrap();
        assert!(arrive(&elsewhere, false).is_err());
        let ack: Acknowledgement = read_document(&elsewhere.join(ACK_FILE)).unwrap();
        assert_eq!(ack.refusal, Some(AckRefusal::Invalid));
        std::fs::remove_dir_all(elsewhere.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_snapshot_that_does_not_check_out_is_refused() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        std::fs::write(
            dir.join(SNAPSHOT_FILE),
            b"{\"format\": \"formiga.travel.snapshot\"",
        )
        .unwrap();
        assert!(arrive(&dir, false).is_err());
        let ack: Acknowledgement = read_document(&dir.join(ACK_FILE)).unwrap();
        assert!(!ack.accepted);
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_trip_while_another_colony_is_visiting_is_refused_as_busy() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        assert!(arrive(&dir, true).is_err());
        let ack: Acknowledgement = read_document(&dir.join(ACK_FILE)).unwrap();
        assert!(!ack.accepted);
        assert_eq!(ack.refusal, Some(AckRefusal::Busy));
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_recall_is_noticed() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        let (trip, _) = arrive(&dir, false).unwrap();
        std::fs::write(dir.join(RECALL_FILE), b"{}").unwrap();
        assert!(trip.recalled());
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_trip_swept_away_counts_as_a_recall() {
        let snapshot = formiga_travel::sample::snapshot();
        let dir = left_by_desktop(&snapshot);
        let (trip, _) = arrive(&dir, false).unwrap();
        assert!(!trip.recalled());
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
        assert!(trip.recalled());
    }
}
