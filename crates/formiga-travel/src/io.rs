//! Reading and writing travel files: bounded on the way in, atomic on the way out.

use crate::{
    ReceiptError, ReturnReceipt, SessionId, SnapshotError, TravelSnapshot, VersionError,
    check_versions,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// A full colony's snapshot is a few tens of kilobytes; anything near this is not one.
pub const MAX_SNAPSHOT_BYTES: u64 = 512 * 1024;
pub const MAX_RECEIPT_BYTES: u64 = 64 * 1024;

/// Where one trip's files live: both in the directory Desktop chose, both named for the trip,
/// so neither app ever needs to look anywhere else or list a directory to find them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TravelFiles {
    pub snapshot: PathBuf,
    pub receipt: PathBuf,
}

impl TravelFiles {
    pub fn new(dir: &Path, session_id: SessionId) -> Self {
        Self {
            snapshot: dir.join(format!("{session_id}.snapshot.json")),
            receipt: dir.join(format!("{session_id}.receipt.json")),
        }
    }

    /// The trip a snapshot at `snapshot_path` belongs to, read from the snapshot itself rather
    /// than from its file name.
    pub fn beside(snapshot_path: &Path, session_id: SessionId) -> Self {
        let dir = snapshot_path.parent().unwrap_or(Path::new("."));
        Self {
            snapshot: snapshot_path.to_path_buf(),
            receipt: Self::new(dir, session_id).receipt,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TravelIoError {
    #[error("could not read or write {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("{path} is larger than the {limit} bytes a travel file may be")]
    TooLarge { path: PathBuf, limit: u64 },
    #[error("{path} is not a readable travel file: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{path}: {source}")]
    Snapshot {
        path: PathBuf,
        source: SnapshotError,
    },
    #[error("{path}: {source}")]
    Receipt { path: PathBuf, source: ReceiptError },
}

/// Just enough of any travel file to tell whether the rest can be read at all, so a file from a
/// much newer app says so instead of failing on whatever field changed shape.
#[derive(Deserialize)]
struct Header {
    format_version: u32,
    minimum_reader_version: u32,
}

/// Reads, checks the version, parses, and validates a snapshot.
pub fn read_snapshot(path: &Path) -> Result<TravelSnapshot, TravelIoError> {
    let bytes = read_bounded(path, MAX_SNAPSHOT_BYTES)?;
    check_header(&bytes).map_err(|source| TravelIoError::Snapshot {
        path: path.to_owned(),
        source: source.into(),
    })?;
    let snapshot: TravelSnapshot = parse(path, &bytes)?;
    snapshot
        .validate()
        .map_err(|source| TravelIoError::Snapshot {
            path: path.to_owned(),
            source,
        })?;
    Ok(snapshot)
}

/// Validates and then writes a snapshot. Desktop's side of the trip.
pub fn write_snapshot(path: &Path, snapshot: &TravelSnapshot) -> Result<(), TravelIoError> {
    snapshot
        .validate()
        .map_err(|source| TravelIoError::Snapshot {
            path: path.to_owned(),
            source,
        })?;
    write_json_atomic(path, snapshot, MAX_SNAPSHOT_BYTES)
}

/// Reads a receipt and validates it against the snapshot it answers. Desktop's side.
pub fn read_receipt(
    path: &Path,
    snapshot: &TravelSnapshot,
) -> Result<ReturnReceipt, TravelIoError> {
    let bytes = read_bounded(path, MAX_RECEIPT_BYTES)?;
    check_header(&bytes).map_err(|source| TravelIoError::Receipt {
        path: path.to_owned(),
        source: source.into(),
    })?;
    let receipt: ReturnReceipt = parse(path, &bytes)?;
    receipt
        .validate(snapshot)
        .map_err(|source| TravelIoError::Receipt {
            path: path.to_owned(),
            source,
        })?;
    Ok(receipt)
}

/// Validates and then writes a receipt. Hill's side.
pub fn write_receipt(
    path: &Path,
    receipt: &ReturnReceipt,
    snapshot: &TravelSnapshot,
) -> Result<(), TravelIoError> {
    receipt
        .validate(snapshot)
        .map_err(|source| TravelIoError::Receipt {
            path: path.to_owned(),
            source,
        })?;
    write_json_atomic(path, receipt, MAX_RECEIPT_BYTES)
}

fn check_header(bytes: &[u8]) -> Result<(), VersionError> {
    // A file without a readable header fails as malformed JSON in `parse`, with its path.
    let Ok(header) = serde_json::from_slice::<Header>(bytes) else {
        return Ok(());
    };
    check_versions(header.format_version, header.minimum_reader_version)
}

fn parse<T: for<'de> Deserialize<'de>>(path: &Path, bytes: &[u8]) -> Result<T, TravelIoError> {
    serde_json::from_slice(bytes).map_err(|source| TravelIoError::Json {
        path: path.to_owned(),
        source,
    })
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, TravelIoError> {
    let io_error = |source| TravelIoError::Io {
        path: path.to_owned(),
        source,
    };
    let file = File::open(path).map_err(io_error)?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > limit {
        return Err(TravelIoError::TooLarge {
            path: path.to_owned(),
            limit,
        });
    }
    Ok(bytes)
}

/// Writes beside the destination and renames over it, so a reader sees the old file or the new
/// one and never half of either.
fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
    limit: u64,
) -> Result<(), TravelIoError> {
    let io_error = |source| TravelIoError::Io {
        path: path.to_owned(),
        source,
    };
    let bytes = serde_json::to_vec_pretty(value).map_err(|source| TravelIoError::Json {
        path: path.to_owned(),
        source,
    })?;
    if bytes.len() as u64 > limit {
        return Err(TravelIoError::TooLarge {
            path: path.to_owned(),
            limit,
        });
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let mut file = File::create(&temporary).map_err(io_error)?;
    file.write_all(&bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    fs::rename(&temporary, path).map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Completion, sample};

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "formiga-travel-{name}-{}",
            SessionId::random().unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_trip_writes_and_reads_both_files() {
        let dir = scratch_dir("trip");
        let snapshot = sample::snapshot();
        let files = TravelFiles::new(&dir, snapshot.session_id);

        write_snapshot(&files.snapshot, &snapshot).unwrap();
        let arrived = read_snapshot(&files.snapshot).unwrap();
        assert_eq!(arrived, snapshot);
        assert_eq!(
            TravelFiles::beside(&files.snapshot, arrived.session_id),
            files
        );

        let receipt = ReturnReceipt::new(
            arrived.session_id,
            "0.1.0",
            arrived.created_at_utc,
            Completion::Clean,
        );
        write_receipt(&files.receipt, &receipt, &arrived).unwrap();
        assert_eq!(read_receipt(&files.receipt, &snapshot).unwrap(), receipt);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_oversized_file_is_not_read() {
        let dir = scratch_dir("oversized");
        let path = dir.join("big.snapshot.json");
        fs::write(&path, vec![b' '; MAX_SNAPSHOT_BYTES as usize + 1]).unwrap();
        assert!(matches!(
            read_snapshot(&path),
            Err(TravelIoError::TooLarge { .. })
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_file_from_a_much_newer_desktop_says_so() {
        let dir = scratch_dir("newer");
        let path = dir.join("newer.snapshot.json");
        fs::write(
            &path,
            r#"{"format_version": 9, "minimum_reader_version": 7, "travelers": "changed shape"}"#,
        )
        .unwrap();
        let error = read_snapshot(&path).unwrap_err();
        assert!(
            matches!(
                error,
                TravelIoError::Snapshot {
                    source: SnapshotError::Version(VersionError::TooNew { needed: 7 }),
                    ..
                }
            ),
            "{error}"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn garbage_is_reported_with_its_path() {
        let dir = scratch_dir("garbage");
        let path = dir.join("garbage.snapshot.json");
        fs::write(&path, "not json").unwrap();
        let error = read_snapshot(&path).unwrap_err();
        assert!(matches!(error, TravelIoError::Json { .. }));
        assert!(error.to_string().contains("garbage.snapshot.json"));
        fs::remove_dir_all(dir).unwrap();
    }
}
