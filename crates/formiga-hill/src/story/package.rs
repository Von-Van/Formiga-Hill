//! Content packages: a folder named `<something>.formiga-hill` holding a manifest, story files
//! and localisation tables, all plain TOML.
//!
//! A package is read whole into memory before any of it is parsed, by a reader that refuses
//! links, nested folders past a few levels, unexpected kinds of file, and anything too large. So
//! nothing a package says can make Hill open a file, follow a path, or run anything. Official
//! stories are packages too, built into Hill and loaded by the same code.

use super::lines::Lines;
use super::script::{AREAS, Story, parse_story};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// The newest content API this Hill hosts.
pub const HILL_API: u32 = 1;

pub mod limits {
    pub const MAX_FILES: usize = 64;
    pub const MAX_FILE_BYTES: u64 = 128 * 1024;
    pub const MAX_PACKAGE_BYTES: u64 = 1024 * 1024;
    pub const MAX_DEPTH: usize = 3;
    pub const MAX_ENTRY_POINTS: usize = 16;
    pub const MAX_ID_CHARS: usize = 64;
    pub const MAX_LABEL_CHARS: usize = 60;
}

/// A package's files by their path inside it, with `/` between folders.
pub type Files = BTreeMap<String, Vec<u8>>;

/// What is wrong with a package, said so its author can put it right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageError {
    /// The package's id, or its folder's name if the manifest never got that far.
    pub package: String,
    pub file: Option<String>,
    pub problem: String,
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{} ({file}): {}", self.package, self.problem),
            None => write!(f, "{}: {}", self.package, self.problem),
        }
    }
}

impl std::error::Error for PackageError {}

/// A package that checked out.
#[derive(Clone, Debug)]
pub struct Package {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    pub stories: Vec<Story>,
}

/// Reads a package folder into memory, refusing anything a text package should not contain.
pub fn read_folder(root: &Path) -> Result<Files, PackageError> {
    let label = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "a package".to_owned());
    let refuse = |file: Option<String>, problem: String| PackageError {
        package: label.clone(),
        file,
        problem,
    };
    let metadata = fs::symlink_metadata(root).map_err(|error| refuse(None, error.to_string()))?;
    if !metadata.is_dir() {
        return Err(refuse(None, "a package is a folder".into()));
    }
    let mut files = Files::new();
    let mut total = 0;
    walk(root, "", 0, &mut files, &mut total).map_err(|(file, problem)| refuse(file, problem))?;
    Ok(files)
}

type WalkError = (Option<String>, String);

fn walk(
    dir: &Path,
    prefix: &str,
    depth: usize,
    files: &mut Files,
    total: &mut u64,
) -> Result<(), WalkError> {
    if depth > limits::MAX_DEPTH {
        return Err((Some(prefix.to_owned()), "folders nest too deep".into()));
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|error| (Some(prefix.to_owned()), error.to_string()))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err((
                Some(prefix.to_owned()),
                "a file name that is not plain text".into(),
            ));
        };
        // Hidden files are the operating system's, not the author's: .DS_Store and the like.
        if name.starts_with('.') {
            continue;
        }
        let path = format!("{prefix}{name}");
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err((
                Some(path),
                "file names may use only letters, digits, '.', '-' and '_'".into(),
            ));
        }
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| (Some(path.clone()), error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err((Some(path), "links are not allowed in a package".into()));
        }
        if metadata.is_dir() {
            walk(&entry.path(), &format!("{path}/"), depth + 1, files, total)?;
            continue;
        }
        if !metadata.is_file() {
            return Err((Some(path), "only ordinary files belong in a package".into()));
        }
        let extension = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !matches!(extension, "toml" | "md" | "txt") {
            return Err((
                Some(path),
                "packages for this Hill carry only .toml, .md and .txt files".into(),
            ));
        }
        if metadata.len() > limits::MAX_FILE_BYTES {
            return Err((
                Some(path),
                format!("larger than {} KiB", limits::MAX_FILE_BYTES / 1024),
            ));
        }
        *total += metadata.len();
        if *total > limits::MAX_PACKAGE_BYTES || files.len() >= limits::MAX_FILES {
            return Err((None, "the package is larger than Hill takes".into()));
        }
        let bytes =
            fs::read(entry.path()).map_err(|error| (Some(path.clone()), error.to_string()))?;
        files.insert(path, bytes);
    }
    Ok(())
}

#[derive(Deserialize)]
struct RawManifest {
    package_id: String,
    title: String,
    author: String,
    version: String,
    hill_api: u32,
    #[serde(default)]
    content_types: Vec<String>,
    entry_points: Vec<String>,
    #[serde(default = "english")]
    default_locale: String,
    #[serde(default)]
    requirements: RawRequirements,
    #[serde(default)]
    permissions: Vec<String>,
}

fn english() -> String {
    "en".to_owned()
}

#[derive(Default, Deserialize)]
struct RawRequirements {
    #[serde(default)]
    min_cast: Option<usize>,
    #[serde(default)]
    areas: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

/// Checks a package's files and parses everything in it. Unknown fields are allowed, so a
/// package written for a later Hill can still load here; anything it requires that this Hill
/// lacks fails closed, with the reason.
pub fn load(files: &Files, label: &str) -> Result<Package, PackageError> {
    let mut package = label.to_owned();
    let refuse = |package: &str, file: Option<&str>, problem: String| PackageError {
        package: package.to_owned(),
        file: file.map(str::to_owned),
        problem,
    };

    let manifest_text = text(files, "manifest.toml")
        .map_err(|problem| refuse(&package, Some("manifest.toml"), problem))?;
    let manifest: RawManifest = toml::from_str(manifest_text)
        .map_err(|error| refuse(&package, Some("manifest.toml"), error.to_string()))?;
    if !is_package_id(&manifest.package_id) {
        return Err(refuse(
            &package,
            Some("manifest.toml"),
            "package_id must be like \"com.example.my-story\": lowercase, with at least one dot"
                .into(),
        ));
    }
    package = manifest.package_id.clone();
    let in_manifest = |problem: String| refuse(&package, Some("manifest.toml"), problem);

    for (field, value) in [("title", &manifest.title), ("author", &manifest.author)] {
        if !is_label(value, limits::MAX_LABEL_CHARS) {
            return Err(in_manifest(format!(
                "{field} must be one short line of text"
            )));
        }
    }
    if !is_label(&manifest.version, 16) {
        return Err(in_manifest(
            "version must be short, such as \"1.0.0\"".into(),
        ));
    }
    if manifest.hill_api == 0 || manifest.hill_api > HILL_API {
        return Err(in_manifest(format!(
            "it is written for Hill content API {}, and this Hill hosts up to {HILL_API}",
            manifest.hill_api
        )));
    }
    if !manifest.permissions.is_empty() {
        return Err(in_manifest(
            "it asks for permissions, and Hill grants packages none".into(),
        ));
    }
    if let Some(capability) = manifest.requirements.capabilities.first() {
        return Err(in_manifest(format!(
            "it needs \"{capability}\", which this Hill does not have"
        )));
    }
    for content in &manifest.content_types {
        if content != "story" {
            return Err(in_manifest(format!(
                "this Hill cannot host \"{content}\" content yet"
            )));
        }
    }
    for area in &manifest.requirements.areas {
        if !AREAS.contains(&area.as_str()) {
            return Err(in_manifest(format!("there is no area called \"{area}\"")));
        }
    }
    if manifest.entry_points.is_empty() || manifest.entry_points.len() > limits::MAX_ENTRY_POINTS {
        return Err(in_manifest(format!(
            "entry_points must list between 1 and {} story files",
            limits::MAX_ENTRY_POINTS
        )));
    }

    let locale_file = format!("localization/{}.toml", manifest.default_locale);
    if !is_locale(&manifest.default_locale) {
        return Err(in_manifest(
            "default_locale must be a language code such as \"en\"".into(),
        ));
    }
    let lines_text = text(files, &locale_file)
        .map_err(|problem| refuse(&package, Some(&locale_file), problem))?;
    let lines = Lines::parse(lines_text)
        .map_err(|problem| refuse(&package, Some(&locale_file), problem))?;

    let min_cast = manifest.requirements.min_cast.unwrap_or(1).max(1);
    let mut stories = Vec::new();
    for entry in &manifest.entry_points {
        if !entry.starts_with("content/") || !entry.ends_with(".toml") {
            return Err(in_manifest(format!(
                "entry point \"{entry}\" must be a .toml file in content/"
            )));
        }
        let story_text =
            text(files, entry).map_err(|problem| refuse(&package, Some(entry), problem))?;
        let story = parse_story(story_text, &lines, min_cast)
            .map_err(|problem| refuse(&package, Some(entry), problem))?;
        if !manifest.requirements.areas.is_empty()
            && !manifest.requirements.areas.contains(&story.area.to_owned())
        {
            return Err(refuse(
                &package,
                Some(entry),
                format!(
                    "it is staged on \"{}\", which the manifest does not list",
                    story.area
                ),
            ));
        }
        if stories.iter().any(|other: &Story| other.id == story.id) {
            return Err(refuse(
                &package,
                Some(entry),
                format!("two stories are called \"{}\"", story.id),
            ));
        }
        stories.push(story);
    }

    Ok(Package {
        id: manifest.package_id,
        title: manifest.title.trim().to_owned(),
        author: manifest.author.trim().to_owned(),
        version: manifest.version.trim().to_owned(),
        stories,
    })
}

fn text<'a>(files: &'a Files, path: &str) -> Result<&'a str, String> {
    let bytes = files
        .get(path)
        .ok_or_else(|| "this file is missing".to_owned())?;
    std::str::from_utf8(bytes).map_err(|_| "this file is not UTF-8 text".to_owned())
}

fn is_package_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    id.len() <= limits::MAX_ID_CHARS
        && id.contains('.')
        && bytes
            .next()
            .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
        && !id.ends_with('.')
}

fn is_locale(code: &str) -> bool {
    (2..=8).contains(&code.len())
        && code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
}

/// One line of display text: not blank, not too long, no control characters.
pub(crate) fn is_label(text: &str, max_chars: usize) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= max_chars
        && !text.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> Files {
        let mut files = Files::new();
        files.insert(
            "manifest.toml".into(),
            br#"
                package_id = "org.example.tiny"
                title = "A tiny story"
                author = "Someone"
                version = "1.0.0"
                hill_api = 1
                content_types = ["story"]
                entry_points = ["content/tiny.toml"]
                future_field = "is ignored"
            "#
            .to_vec(),
        );
        files.insert(
            "content/tiny.toml".into(),
            br#"
                [story]
                id = "tiny"
                title = "title"
                area = "clubhouse"
                start = "only"

                [roles.anyone]
                select = ["any"]

                [[scenes]]
                id = "only"

                [[scenes.beats]]
                say = "anyone"
                line = "hello"
            "#
            .to_vec(),
        );
        files.insert(
            "localization/en.toml".into(),
            br#"
                title = "A tiny story"
                hello = "Hello from {anyone}."
            "#
            .to_vec(),
        );
        files
    }

    fn problem(files: &Files) -> String {
        load(files, "test").unwrap_err().problem
    }

    #[test]
    fn a_minimal_package_loads() {
        let package = load(&minimal(), "tiny.formiga-hill").unwrap();
        assert_eq!(package.id, "org.example.tiny");
        assert_eq!(package.stories.len(), 1);
    }

    #[test]
    fn a_package_for_a_newer_hill_fails_closed() {
        let mut files = minimal();
        let manifest = String::from_utf8(files["manifest.toml"].clone()).unwrap();
        files.insert(
            "manifest.toml".into(),
            manifest
                .replace("hill_api = 1", "hill_api = 2")
                .into_bytes(),
        );
        assert!(problem(&files).contains("content API 2"));
    }

    #[test]
    fn permissions_and_unknown_capabilities_are_refused() {
        for (from, to) in [
            ("hill_api = 1", "hill_api = 1\npermissions = [\"network\"]"),
            (
                "hill_api = 1",
                "hill_api = 1\n[requirements]\ncapabilities = [\"photos\"]",
            ),
            (
                "content_types = [\"story\"]",
                "content_types = [\"activity\"]",
            ),
        ] {
            let mut files = minimal();
            let manifest = String::from_utf8(files["manifest.toml"].clone()).unwrap();
            files.insert(
                "manifest.toml".into(),
                manifest.replacen(from, to, 1).into_bytes(),
            );
            assert!(load(&files, "test").is_err(), "{to} was accepted");
        }
    }

    #[test]
    fn entry_points_cannot_reach_outside_content() {
        let mut files = minimal();
        let manifest = String::from_utf8(files["manifest.toml"].clone()).unwrap();
        files.insert(
            "manifest.toml".into(),
            manifest
                .replace("content/tiny.toml", "../../colony.json")
                .into_bytes(),
        );
        assert!(problem(&files).contains("content/"));
    }

    #[test]
    fn a_missing_line_is_named() {
        let mut files = minimal();
        files.insert(
            "localization/en.toml".into(),
            b"title = \"A tiny story\"".to_vec(),
        );
        let error = load(&files, "test").unwrap_err();
        assert_eq!(error.file.as_deref(), Some("content/tiny.toml"));
        assert!(error.problem.contains("hello"), "{}", error.problem);
    }

    #[test]
    fn package_ids_are_namespaced_and_plain() {
        assert!(is_package_id("com.formiga.hill.first-picnic"));
        assert!(!is_package_id("nodots"));
        assert!(!is_package_id("Com.Example"));
        assert!(!is_package_id("../escape.x"));
    }

    #[test]
    fn folders_with_links_or_strange_files_are_refused() {
        let root = std::env::temp_dir().join(format!("hill-package-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(root.join("manifest.toml"), "x = 1").unwrap();
        assert!(read_folder(&root).is_ok());
        fs::write(root.join("content/run.sh"), "echo hi").unwrap();
        assert!(read_folder(&root).unwrap_err().problem.contains(".toml"));
        fs::remove_file(root.join("content/run.sh")).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/hosts", root.join("content/hosts.toml")).unwrap();
            assert!(read_folder(&root).unwrap_err().problem.contains("links"));
        }
        fs::remove_dir_all(&root).unwrap();
    }
}
