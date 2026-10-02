//! Snapshots as each format version first wrote them. Fixtures are frozen once committed: if one
//! stops reading, something upstream, usually a `formiga-core` bump that reshaped the appearance
//! genome or temperament, has broken every Hill already out there, and the fix is a format
//! version bump rather than a new fixture.

use formiga_travel::{read_snapshot, sample, write_snapshot};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn the_first_v1_snapshot_still_reads() {
    let snapshot = read_snapshot(&fixtures().join("snapshot-v1.json"))
        .expect("a v1 snapshot must always read");
    assert_eq!(snapshot.format_version, 1);
    assert_eq!(snapshot.travelers.len(), 4);
    assert_eq!(snapshot.bonds.len(), 6);
    assert!(snapshot.travelers.iter().any(|t| t.accessory.is_some()));
    assert!(snapshot.travelers.iter().any(|t| !t.habits.is_empty()));
    assert!(snapshot.travelers.iter().all(|t| t.traits.len() == 3));
}

/// Today's export may add to what v1 wrote, but never drop or rename a field a v1 reader needs.
#[test]
fn todays_export_still_has_everything_v1_wrote() {
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures().join("snapshot-v1.json")).unwrap(),
    )
    .unwrap();
    let today = serde_json::to_value(sample::snapshot()).unwrap();
    let missing: Vec<_> = key_paths(&fixture)
        .difference(&key_paths(&today))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "today's export no longer writes {missing:?}, which a v1 reader needs; \
         raise minimum_reader_version (and FORMAT_VERSION) instead"
    );
}

/// Writes today's sample export as a new fixture, once, when a format version is born:
/// `FORMIGA_TRAVEL_NEW_FIXTURE=snapshot-v2.json cargo test -p formiga-travel --test compat`
#[test]
fn write_a_new_fixture_when_asked() {
    let Ok(name) = std::env::var("FORMIGA_TRAVEL_NEW_FIXTURE") else {
        return;
    };
    let path = fixtures().join(name);
    assert!(!path.exists(), "{} is frozen", path.display());
    write_snapshot(&path, &sample::snapshot()).unwrap();
}

/// Every object key in a JSON tree, by path, with array positions folded together.
fn key_paths(value: &Value) -> BTreeSet<String> {
    fn walk(value: &Value, prefix: &str, paths: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let path = format!("{prefix}.{key}");
                    paths.insert(path.clone());
                    walk(child, &path, paths);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item, &format!("{prefix}[]"), paths);
                }
            }
            _ => {}
        }
    }
    let mut paths = BTreeSet::new();
    walk(value, "", &mut paths);
    paths
}
