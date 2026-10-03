//! The packages folder, where the person puts community stories for Hill to find, and which of
//! them the person has set aside for now.
//!
//! Hill only ever reads the folder: each package in it is a folder of its own, read and checked
//! by the same strict loader as Hill's own stories, so nothing a package contains can do more
//! than one of them can. A package that will not load is reported and left out, and never stops
//! Hill opening. Setting a package aside takes its stories off the notice board without touching
//! it; nothing about it is kept per colony.

use super::package::PackageError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What a package folder's name ends with.
pub const SUFFIX: &str = ".formiga-hill";
/// The most packages the folder is read for: past this, the rest are left out and said so.
pub const MOST: usize = 64;
const SETTINGS: &str = "packages.json";
/// The settings file is a short list of ids; anything bigger is not Hill's.
const MAX_SETTINGS_BYTES: u64 = 64 * 1024;

/// Where the packages folder is, in Hill's own data folder.
pub fn folder(data: &Path) -> PathBuf {
    data.join("packages")
}

/// Every package folder in `folder`, in name order, and anything wrong with what is there. A
/// folder that does not exist yet holds nothing, and is not a problem.
pub fn found_in(folder: &Path) -> (Vec<PathBuf>, Vec<PackageError>) {
    let mut problems = Vec::new();
    let Ok(entries) = fs::read_dir(folder) else {
        return (Vec::new(), problems);
    };
    let mut packages: Vec<PathBuf> = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        // Links are refused, as they are inside a package: a package is a folder, and only that.
        let is_folder = fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_dir());
        if !name.ends_with(SUFFIX) {
            if is_folder {
                problems.push(PackageError {
                    package: name,
                    file: None,
                    problem: format!("a package folder's name ends with {SUFFIX}"),
                });
            }
            continue;
        }
        if !is_folder {
            problems.push(PackageError {
                package: name,
                file: None,
                problem: "a package is a folder".into(),
            });
            continue;
        }
        packages.push(path);
    }
    packages.sort();
    if packages.len() > MOST {
        for left_out in packages.drain(MOST..) {
            problems.push(PackageError {
                package: left_out
                    .file_name()
                    .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                file: None,
                problem: format!("only the first {MOST} packages are read"),
            });
        }
    }
    (packages, problems)
}

/// The packages the person has set aside, by id, kept beside Hill's other data.
#[derive(Debug, Default)]
pub struct SetAside {
    ids: BTreeSet<String>,
    path: Option<PathBuf>,
}

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    #[serde(default)]
    set_aside: BTreeSet<String>,
}

impl SetAside {
    /// What was set aside last time, kept in `data`; nothing, if there is no data folder.
    pub fn open(data: Option<&Path>) -> Self {
        let path = data.map(|data| data.join(SETTINGS));
        let ids = path
            .as_deref()
            .and_then(|path| {
                let meta = fs::metadata(path).ok()?;
                if meta.len() > MAX_SETTINGS_BYTES {
                    return None;
                }
                serde_json::from_slice::<Settings>(&fs::read(path).ok()?).ok()
            })
            .map(|settings| settings.set_aside)
            .unwrap_or_default();
        Self { ids, path }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }

    /// Sets a package aside, or takes it back up, and remembers that.
    pub fn set(&mut self, id: &str, aside: bool) {
        let changed = if aside {
            self.ids.insert(id.to_owned())
        } else {
            self.ids.remove(id)
        };
        if changed {
            self.keep();
        }
    }

    fn keep(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let settings = Settings {
            set_aside: self.ids.clone(),
        };
        let write = || -> std::io::Result<()> {
            if let Some(folder) = path.parent() {
                fs::create_dir_all(folder)?;
            }
            let mut temporary = path.as_os_str().to_owned();
            temporary.push(".tmp");
            let temporary = PathBuf::from(temporary);
            let mut file = fs::File::create(&temporary)?;
            file.write_all(&serde_json::to_vec_pretty(&settings)?)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, path)
        };
        if let Err(error) = write() {
            eprintln!("formiga-hill: could not keep which packages are set aside: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-shelf-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn only_package_folders_are_picked_up_and_anything_else_is_said_so() {
        let dir = scratch("found");
        fs::create_dir(dir.join("b.formiga-hill")).unwrap();
        fs::create_dir(dir.join("a.formiga-hill")).unwrap();
        fs::create_dir(dir.join("not-a-package")).unwrap();
        fs::write(dir.join("c.formiga-hill"), b"a file, not a folder").unwrap();
        fs::write(dir.join("notes.txt"), b"ignored").unwrap();
        fs::create_dir(dir.join(".hidden.formiga-hill")).unwrap();
        let (found, problems) = found_in(&dir);
        let names: Vec<_> = found
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.formiga-hill", "b.formiga-hill"]);
        let mut said: Vec<_> = problems.iter().map(|p| p.package.as_str()).collect();
        said.sort_unstable();
        assert_eq!(said, ["c.formiga-hill", "not-a-package"]);
    }

    #[test]
    fn a_missing_folder_holds_nothing_and_is_no_trouble() {
        let (found, problems) = found_in(Path::new("/no/such/folder/for/hill"));
        assert!(found.is_empty() && problems.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_in_the_folder_is_refused() {
        let dir = scratch("link");
        let target = scratch("link-target");
        std::os::unix::fs::symlink(&target, dir.join("sneaky.formiga-hill")).unwrap();
        let (found, problems) = found_in(&dir);
        assert!(found.is_empty());
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn what_is_set_aside_is_remembered() {
        let dir = scratch("aside");
        let mut aside = SetAside::open(Some(&dir));
        assert!(!aside.contains("org.example.kite"));
        aside.set("org.example.kite", true);
        let reopened = SetAside::open(Some(&dir));
        assert!(reopened.contains("org.example.kite"));
        let mut reopened = reopened;
        reopened.set("org.example.kite", false);
        assert!(!SetAside::open(Some(&dir)).contains("org.example.kite"));
    }
}
