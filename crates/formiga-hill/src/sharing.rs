//! Sharing what people make at the Hill: one plain file to pass on, a careful look at one before
//! it is let in, and a tidy way to keep, replace and remove what was.
//!
//! Whatever is shared is a content package, the same folder of TOML that `docs/PACKAGES.md`
//! describes, whatever kind of content it carries. A share file is that folder in one text file:
//!
//! ```toml
//! formiga_hill_share = 1
//!
//! [files]
//! "manifest.toml" = "package_id = \"org.example.lost-kite\"\n…"
//! "content/lost-kite.toml" = "…"
//! "localization/en.toml" = "…"
//! ```
//!
//! Nothing in one is trusted. A share file is read only up to a size, only as text, and its
//! files are held to exactly the limits a package folder is; then the package is loaded by the
//! same strict loader as any other, before anything is written. What is let in goes into the
//! packages folder as an ordinary package, so setting it aside, and removing it, work the same
//! for a package made at the Hill, one shared by someone else, and one put there by hand.
//!
//! Each editor keeps its unfinished work in its own `Drafts`, in whatever form suits it.

use crate::story::{self, Files, Package, PackageError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What a share file's name ends with, after a dot.
pub const EXTENSION: &str = "formiga-hill-share";
/// The share file format this Hill writes, and the newest it reads.
pub const FORMAT: u32 = 1;
/// The largest share file Hill reads: a whole package, written out as quoted text.
pub const MAX_SHARE_BYTES: u64 = 2 * story::limits::MAX_PACKAGE_BYTES;
/// Package ids kept for Hill's own content, which nothing shared may take.
pub const RESERVED: &str = "com.formiga.";

/// The few words at the top of a share file, for anyone who opens one to see what it is.
const PREAMBLE: &str = "# Something made for Formiga Hill. Open it from Formiga Hill, which shows \
                        what is inside before adding it.\n";

#[derive(Serialize, Deserialize)]
struct RawShare {
    formiga_hill_share: u32,
    files: BTreeMap<String, String>,
}

fn problem(label: &str, file: Option<&str>, problem: impl Into<String>) -> PackageError {
    PackageError {
        package: label.to_owned(),
        file: file.map(str::to_owned),
        problem: problem.into(),
    }
}

/// Checks a package's files by the rules a package folder is read by.
pub fn check(files: &Files, label: &str) -> Result<(), PackageError> {
    story::check_files(files, label)
}

/// A package's files as the text of a share file.
pub fn encode(files: &Files, label: &str) -> Result<String, PackageError> {
    check(files, label)?;
    let mut text = BTreeMap::new();
    for (path, bytes) in files {
        let words = String::from_utf8(bytes.clone())
            .map_err(|_| problem(label, Some(path), "this file is not UTF-8 text"))?;
        text.insert(path.clone(), words);
    }
    let raw = RawShare {
        formiga_hill_share: FORMAT,
        files: text,
    };
    let body = toml::to_string(&raw).map_err(|error| problem(label, None, error.to_string()))?;
    Ok(format!("{PREAMBLE}{body}"))
}

/// Writes a package's files to a share file at `to`.
pub fn export(files: &Files, to: &Path) -> Result<(), PackageError> {
    let label = name_of(to);
    let text = encode(files, &label)?;
    write_atomically(to, text.as_bytes()).map_err(|error| problem(&label, None, error.to_string()))
}

/// The files in a share file's text, checked by the rules a package folder is read by.
pub fn decode(bytes: &[u8], label: &str) -> Result<Files, PackageError> {
    if bytes.len() as u64 > MAX_SHARE_BYTES {
        return Err(problem(label, None, "it is larger than Hill takes"));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| problem(label, None, "it is not a Formiga Hill share file"))?;
    let raw: RawShare = toml::from_str(text)
        .map_err(|_| problem(label, None, "it is not a Formiga Hill share file"))?;
    if raw.formiga_hill_share == 0 || raw.formiga_hill_share > FORMAT {
        return Err(problem(
            label,
            None,
            "it was made by a newer Formiga Hill: update Hill to open it",
        ));
    }
    let files: Files = raw
        .files
        .into_iter()
        .map(|(path, text)| (path, text.into_bytes()))
        .collect();
    check(&files, label)?;
    Ok(files)
}

/// Reads a share file, refusing one too large before reading any of it.
pub fn read(path: &Path) -> Result<Files, PackageError> {
    let label = name_of(path);
    let metadata = fs::metadata(path).map_err(|error| problem(&label, None, error.to_string()))?;
    if !metadata.is_file() {
        return Err(problem(&label, None, "it is not a Formiga Hill share file"));
    }
    if metadata.len() > MAX_SHARE_BYTES {
        return Err(problem(&label, None, "it is larger than Hill takes"));
    }
    let bytes = fs::read(path).map_err(|error| problem(&label, None, error.to_string()))?;
    decode(&bytes, &label)
}

/// What a package says about itself, for showing before it is let in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    /// The kinds of content it carries, such as `story`.
    pub content_types: Vec<String>,
    pub files: usize,
    pub bytes: u64,
}

/// A package looked over, and found to load.
#[derive(Clone, Debug)]
pub struct Inspection {
    pub files: Files,
    pub summary: Summary,
    pub package: Package,
}

#[derive(Deserialize)]
struct RawManifest {
    package_id: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    content_types: Vec<String>,
}

/// Loads a package's files as Hill would, and says what it is. Nothing is written.
pub fn inspect(files: Files, label: &str) -> Result<Inspection, PackageError> {
    check(&files, label)?;
    let package = story::load(&files, label)?;
    let manifest = files
        .get("manifest.toml")
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|text| toml::from_str::<RawManifest>(text).ok());
    let summary = Summary {
        id: package.id.clone(),
        title: package.title.clone(),
        author: package.author.clone(),
        version: package.version.clone(),
        content_types: manifest.map(|m| m.content_types).unwrap_or_default(),
        files: files.len(),
        bytes: files.values().map(|bytes| bytes.len() as u64).sum(),
    };
    Ok(Inspection {
        files,
        summary,
        package,
    })
}

/// A package let into the packages folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub path: PathBuf,
    /// The version of the package it took the place of, if there was one with its id.
    pub replaced: Option<String>,
}

/// Lets a package into the packages `folder`, once it has been checked and loads: as
/// `<id>.formiga-hill`, in place of any package already there with its id. It is written beside
/// the folder's packages first and only then put in place, so a package is never half there.
pub fn install(folder: &Path, files: &Files) -> Result<Installed, PackageError> {
    let inspection = inspect(files.clone(), "the package")?;
    let id = inspection.summary.id;
    if id.starts_with(RESERVED) {
        return Err(problem(
            &id,
            Some("manifest.toml"),
            "ids beginning \"com.formiga.\" are kept for Hill's own",
        ));
    }
    let failed = |error: std::io::Error| problem(&id, None, error.to_string());
    fs::create_dir_all(folder).map_err(failed)?;
    let target = folder.join(format!("{id}{}", story::shelf::SUFFIX));
    let existing = installed(folder, &id);
    if existing.as_deref() != Some(target.as_path()) && fs::symlink_metadata(&target).is_ok() {
        return Err(problem(
            &id,
            None,
            format!(
                "something else is already called {}",
                target.file_name().unwrap_or_default().to_string_lossy()
            ),
        ));
    }
    let replaced = existing.as_deref().map(version_in);
    let process = std::process::id();
    // Hidden, so the packages folder never shows either half-way.
    let incoming = folder.join(format!(".incoming-{id}-{process}"));
    let outgoing = folder.join(format!(".outgoing-{id}-{process}"));
    let _ = fs::remove_dir_all(&incoming);
    let written = (|| -> std::io::Result<()> {
        for (path, bytes) in files {
            let path = incoming.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
        if let Some(existing) = &existing {
            let _ = fs::remove_dir_all(&outgoing);
            fs::rename(existing, &outgoing)?;
        }
        if let Err(error) = fs::rename(&incoming, &target) {
            // Put back what was there.
            if let Some(existing) = &existing {
                let _ = fs::rename(&outgoing, existing);
            }
            return Err(error);
        }
        Ok(())
    })();
    if let Err(error) = written {
        let _ = fs::remove_dir_all(&incoming);
        return Err(failed(error));
    }
    let _ = fs::remove_dir_all(&outgoing);
    Ok(Installed {
        path: target,
        replaced,
    })
}

/// The folder in the packages `folder` holding the package with this id, if one does.
pub fn installed(folder: &Path, id: &str) -> Option<PathBuf> {
    let (found, _) = story::shelf::found_in(folder);
    found.into_iter().find(|path| {
        manifest_in(path)
            .and_then(|text| toml::from_str::<RawManifest>(&text).ok())
            .is_some_and(|manifest| manifest.package_id == id)
    })
}

/// Takes the package with this id out of the packages `folder`, for good.
pub fn remove(folder: &Path, id: &str) -> Result<(), PackageError> {
    let path = installed(folder, id)
        .ok_or_else(|| problem(id, None, "it is not in the packages folder"))?;
    // `installed` only finds folders, never links, but look again just before removing.
    let metadata =
        fs::symlink_metadata(&path).map_err(|error| problem(id, None, error.to_string()))?;
    if !metadata.is_dir() {
        return Err(problem(id, None, "a package is a folder"));
    }
    fs::remove_dir_all(&path).map_err(|error| problem(id, None, error.to_string()))
}

fn manifest_in(package: &Path) -> Option<String> {
    let path = package.join("manifest.toml");
    let metadata = fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > story::limits::MAX_FILE_BYTES {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn version_in(package: &Path) -> String {
    manifest_in(package)
        .and_then(|text| toml::from_str::<RawManifest>(&text).ok())
        .map(|manifest| manifest.version)
        .unwrap_or_default()
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// Writes a file whole or not at all: to a file beside it first, then put in its place.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(folder) = path.parent()
        && !folder.as_os_str().is_empty()
    {
        fs::create_dir_all(folder)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path)
}

/// Unfinished work, kept by an editor in its own folder of Hill's data, one file each, in
/// whatever form the editor writes. Nothing here is ever shared or loaded as content.
#[derive(Clone, Debug, Default)]
pub struct Drafts {
    folder: Option<PathBuf>,
}

/// The most drafts an editor lists, and the largest it reads.
pub const MOST_DRAFTS: usize = 200;
pub const MAX_DRAFT_BYTES: u64 = 512 * 1024;
/// What a draft's file name ends with.
const DRAFT_SUFFIX: &str = ".toml";

impl Drafts {
    /// The drafts of one `kind` of thing, such as `stories`, kept in Hill's `data` folder;
    /// nowhere, if there is none, so nothing is kept.
    pub fn open(data: Option<&Path>, kind: &str) -> Self {
        Self {
            folder: data.map(|data| data.join("drafts").join(kind)),
        }
    }

    /// Whether drafts can be kept on this computer at all.
    pub fn can_keep(&self) -> bool {
        self.folder.is_some()
    }

    /// Every draft by name, the latest worked on first.
    pub fn list(&self) -> Vec<String> {
        let Some(folder) = &self.folder else {
            return Vec::new();
        };
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        let mut found: Vec<(std::time::SystemTime, String)> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry
                    .file_name()
                    .to_str()?
                    .strip_suffix(DRAFT_SUFFIX)?
                    .to_owned();
                let metadata = fs::symlink_metadata(entry.path()).ok()?;
                (is_draft_name(&name) && metadata.is_file() && metadata.len() <= MAX_DRAFT_BYTES)
                    .then(|| (metadata.modified().unwrap_or(std::time::UNIX_EPOCH), name))
            })
            .collect();
        found.sort_by(|a, b| b.cmp(a));
        found.truncate(MOST_DRAFTS);
        found.into_iter().map(|(_, name)| name).collect()
    }

    pub fn load(&self, name: &str) -> Option<String> {
        let path = self.path(name)?;
        let metadata = fs::symlink_metadata(&path).ok()?;
        if !metadata.is_file() || metadata.len() > MAX_DRAFT_BYTES {
            return None;
        }
        fs::read_to_string(path).ok()
    }

    pub fn save(&self, name: &str, text: &str) -> std::io::Result<()> {
        let path = self.path(name).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "there is nowhere to keep drafts",
            )
        })?;
        if text.len() as u64 > MAX_DRAFT_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the draft is too long to keep",
            ));
        }
        write_atomically(&path, text.as_bytes())
    }

    pub fn remove(&self, name: &str) -> std::io::Result<()> {
        match self.path(name) {
            Some(path) => fs::remove_file(path),
            None => Ok(()),
        }
    }

    fn path(&self, name: &str) -> Option<PathBuf> {
        let folder = self.folder.as_ref()?;
        is_draft_name(name).then(|| folder.join(format!("{name}{DRAFT_SUFFIX}")))
    }
}

/// A draft's name is a plain file name of its own making, never a path.
fn is_draft_name(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// A plain name made from someone's words, for a draft, a package id or a file: lowercase
/// letters and digits joined by `-`, never empty.
pub fn slug(words: &str, most: usize) -> String {
    let mut slug = String::new();
    for c in words.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.len() >= most {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_owned();
    if slug.is_empty() {
        "untitled".to_owned()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn official(name: &str) -> Files {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("content")
            .join(name);
        story::read_folder(&root).unwrap()
    }

    /// A community copy of an official package, under an id of its own.
    fn community() -> Files {
        let mut files = official("first-picnic.formiga-hill");
        let manifest = String::from_utf8(files["manifest.toml"].clone()).unwrap();
        files.insert(
            "manifest.toml".into(),
            manifest
                .replace("com.formiga.hill.first-picnic", "org.example.picnic")
                .into_bytes(),
        );
        files
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-sharing-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_package_comes_back_from_its_share_file_exactly() {
        let files = community();
        let text = encode(&files, "picnic").unwrap();
        assert!(text.starts_with("# Something made for Formiga Hill"));
        assert_eq!(decode(text.as_bytes(), "picnic").unwrap(), files);
        let inspection = inspect(files, "picnic").unwrap();
        assert_eq!(inspection.summary.id, "org.example.picnic");
        assert_eq!(inspection.summary.content_types, ["story"]);
        assert_eq!(inspection.package.stories.len(), 1);
    }

    #[test]
    fn a_share_file_that_is_not_one_or_is_too_new_is_refused_in_plain_words() {
        for bytes in [&b"\xff\xfe"[..], b"hello = 1", b"[files]\n\"a.toml\" = 3"] {
            let error = decode(bytes, "x").unwrap_err();
            assert!(
                error.problem.contains("not a Formiga Hill share file"),
                "{error}"
            );
        }
        let newer = "formiga_hill_share = 2\n[files]\n";
        assert!(
            decode(newer.as_bytes(), "x")
                .unwrap_err()
                .problem
                .contains("newer")
        );
        let huge = vec![b'#'; MAX_SHARE_BYTES as usize + 1];
        assert!(decode(&huge, "x").unwrap_err().problem.contains("larger"));
    }

    #[test]
    fn files_in_a_share_file_are_held_to_a_package_folders_rules() {
        for path in [
            "../escape.toml",
            "/etc/hosts.toml",
            "content/.hidden.toml",
            "content//double.toml",
            "content/run.sh",
            "a/b/c/d/e.toml",
            "content/sp ace.toml",
        ] {
            let mut files = community();
            files.insert(path.into(), b"x = 1".to_vec());
            let text = format!(
                "formiga_hill_share = 1\n[files]\n{}",
                files
                    .iter()
                    .map(|(p, b)| format!(
                        "{} = {}\n",
                        toml::Value::String(p.clone()),
                        toml::Value::String(String::from_utf8(b.clone()).unwrap())
                    ))
                    .collect::<String>()
            );
            assert!(decode(text.as_bytes(), "x").is_err(), "{path} was let in");
        }
        let mut files = community();
        files.insert(
            "content/big.txt".into(),
            vec![b'a'; story::limits::MAX_FILE_BYTES as usize + 1],
        );
        assert!(check(&files, "x").is_err());
    }

    #[test]
    fn a_package_is_installed_replaced_and_removed_by_its_id() {
        let folder = scratch("install");
        let installed_at = install(&folder, &community()).unwrap();
        assert_eq!(installed_at.replaced, None);
        assert!(
            installed_at
                .path
                .ends_with("org.example.picnic.formiga-hill")
        );
        assert_eq!(
            installed(&folder, "org.example.picnic").as_deref(),
            Some(installed_at.path.as_path())
        );
        // Nothing half-made is left about.
        let names: Vec<_> = fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["org.example.picnic.formiga-hill"]);

        // A newer version takes its place, wherever the old one was kept.
        fs::rename(&installed_at.path, folder.join("by-hand.formiga-hill")).unwrap();
        let mut newer = community();
        let manifest = String::from_utf8(newer["manifest.toml"].clone()).unwrap();
        newer.insert(
            "manifest.toml".into(),
            manifest
                .replace("version = \"1.0.0\"", "version = \"1.1.0\"")
                .into_bytes(),
        );
        let again = install(&folder, &newer).unwrap();
        assert_eq!(again.replaced.as_deref(), Some("1.0.0"));
        assert!(!folder.join("by-hand.formiga-hill").exists());
        let library = story::Library::load(Some(&folder), &[]);
        assert!(library.problems.is_empty(), "{:?}", library.problems);
        assert!(library.packages.iter().any(|p| p.version == "1.1.0"));

        remove(&folder, "org.example.picnic").unwrap();
        assert!(installed(&folder, "org.example.picnic").is_none());
        assert!(remove(&folder, "org.example.picnic").is_err());
    }

    #[test]
    fn a_package_that_will_not_load_or_takes_hills_own_id_is_never_written() {
        let folder = scratch("refuse");
        let mut broken = community();
        broken.insert("localization/en.toml".into(), b"title = \"x\"".to_vec());
        assert!(install(&folder, &broken).is_err());
        assert!(install(&folder, &official("first-picnic.formiga-hill")).is_err());
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 0);
    }

    #[test]
    fn drafts_are_kept_by_plain_names_only() {
        let data = scratch("drafts");
        let drafts = Drafts::open(Some(&data), "stories");
        assert!(drafts.list().is_empty());
        drafts
            .save("lost-kite", "title = \"The Lost Kite\"")
            .unwrap();
        drafts.save("second", "x").unwrap();
        assert_eq!(drafts.list().len(), 2);
        assert_eq!(
            drafts.load("lost-kite").as_deref(),
            Some("title = \"The Lost Kite\"")
        );
        assert!(drafts.save("../escape", "x").is_err());
        assert!(drafts.load("../escape").is_none());
        drafts.remove("second").unwrap();
        assert_eq!(drafts.list(), ["lost-kite"]);
        assert!(!Drafts::open(None, "stories").can_keep());
    }

    #[test]
    fn a_slug_is_plain_and_never_empty() {
        assert_eq!(slug("The Lost Kite!", 40), "the-lost-kite");
        assert_eq!(slug("  ¿¡ ", 40), "untitled");
        assert_eq!(slug("Ünïcode name 2", 40), "n-code-name-2");
        assert!(slug(&"long ".repeat(40), 20).len() <= 20);
    }
}
