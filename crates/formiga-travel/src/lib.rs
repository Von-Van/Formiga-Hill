//! The travel contract between Formiga Desktop and Formiga Hill.
//!
//! Desktop owns the colony. When the colony takes the train, Desktop writes a [`TravelSnapshot`]:
//! a read-only projection of who is travelling, narrower than its save and versioned on its own.
//! Hill reads it, owns everything that happens at the Hill, and on the way home writes a
//! [`ReturnReceipt`] holding only requests, which Desktop is free to accept or ignore.
//!
//! Neither app reads the other's save. Everything that crosses goes through the types here, and
//! both sides validate it on the way in. `docs/TRAVEL.md` describes the format and the handoff.

mod export;
mod io;
mod receipt;
pub mod sample;
mod session;
mod snapshot;
mod text;

pub use export::{ExportOptions, export_snapshot};
pub use io::{
    MAX_RECEIPT_BYTES, MAX_SNAPSHOT_BYTES, TravelFiles, TravelIoError, read_receipt, read_snapshot,
    write_receipt, write_snapshot,
};
pub use receipt::{
    Completion, MAX_SOUVENIRS, Outing, ReceiptError, ReturnReceipt, Souvenir, SouvenirError,
};
pub use session::SessionId;
pub use snapshot::{
    AccessoryInk, Bond, ColonyTag, Level, MAX_BONDS, MAX_CAPABILITIES, MAX_TRAITS, MAX_TRAVELERS,
    Presentation, SnapshotError, Theme, TravelSnapshot, Traveler, WornAccessory, capability,
};

/// The newest snapshot and receipt format this build writes and can read.
pub const FORMAT_VERSION: u32 = 1;

/// Why a travel file's version header cannot be read by this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VersionError {
    #[error(
        "format version {format_version} with minimum reader version {minimum_reader_version} \
         is not a valid pair"
    )]
    Malformed {
        format_version: u32,
        minimum_reader_version: u32,
    },
    #[error(
        "this file needs a reader for format {needed} or later; this build reads up to {FORMAT_VERSION}"
    )]
    TooNew { needed: u32 },
}

/// A writer may add to a format without breaking older readers, and says so by leaving the
/// minimum reader version where it was. Only a change an older reader would misread raises it.
pub(crate) fn check_versions(
    format_version: u32,
    minimum_reader_version: u32,
) -> Result<(), VersionError> {
    if format_version == 0 || minimum_reader_version == 0 || minimum_reader_version > format_version
    {
        return Err(VersionError::Malformed {
            format_version,
            minimum_reader_version,
        });
    }
    if minimum_reader_version > FORMAT_VERSION {
        return Err(VersionError::TooNew {
            needed: minimum_reader_version,
        });
    }
    Ok(())
}
