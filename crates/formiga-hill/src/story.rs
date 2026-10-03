//! Stories: short authored scenes staged in the Clubhouse with the colony as cast, loaded from
//! declarative content packages. Hill's own stories are packages in exactly the format a
//! community author writes, built into the app and loaded by the same code; see
//! `docs/PACKAGES.md` for the format.

mod casting;
mod director;
pub mod lines;
mod package;
pub(crate) mod script;
pub mod shelf;
pub mod souvenirs;

pub use director::Director;
pub use package::{Files, Package, PackageError, load, read_folder};
pub use script::Story;

use std::path::{Path, PathBuf};

/// Hill's own packages, built in. Each is a folder under `content/` loaded through the same reader
/// and checks as any other package; the test below keeps this list and the folder in step.
macro_rules! built_in {
    ($folder:literal, [$($file:literal),* $(,)?]) => {
        ($folder, vec![$((
            $file,
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/content/", $folder, "/", $file)) as &[u8],
        )),*])
    };
}

/// A built-in package: its folder's name, and each file's path and contents.
type BuiltIn = (&'static str, Vec<(&'static str, &'static [u8])>);

fn official() -> Vec<BuiltIn> {
    vec![
        built_in!(
            "first-picnic.formiga-hill",
            [
                "manifest.toml",
                "content/first-picnic.toml",
                "localization/en.toml",
            ]
        ),
        built_in!(
            "book-with-no-ending.formiga-hill",
            [
                "manifest.toml",
                "content/book-with-no-ending.toml",
                "localization/en.toml",
            ]
        ),
    ]
}

/// Where a package came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Hill's own, built in.
    Official,
    /// Found in the packages folder.
    Folder,
    /// Named on the command line, by an author trying it out.
    Named,
}

/// Every story Hill can stage, and anything that would not load, with why.
#[derive(Default)]
pub struct Library {
    pub packages: Vec<Package>,
    /// Where each package came from, in the same order.
    pub origins: Vec<Origin>,
    pub problems: Vec<PackageError>,
}

impl Library {
    /// Hill's own packages, then whatever is in the packages `folder`, then any package folders
    /// named for testing or authoring.
    pub fn load(folder: Option<&Path>, extra: &[PathBuf]) -> Self {
        let mut library = Self::default();
        for (name, files) in official() {
            let files: Files = files
                .into_iter()
                .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
                .collect();
            library.add(load(&files, name), Origin::Official);
        }
        if let Some(folder) = folder {
            let (found, problems) = shelf::found_in(folder);
            library.problems.extend(problems);
            for path in found {
                library.read(&path, Origin::Folder);
            }
        }
        for path in extra {
            library.read(path, Origin::Named);
        }
        library
    }

    fn read(&mut self, path: &Path, origin: Origin) {
        let label = path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        self.add(
            read_folder(path).and_then(|files| load(&files, &label)),
            origin,
        );
    }

    fn add(&mut self, package: Result<Package, PackageError>, origin: Origin) {
        match package {
            Ok(package) if self.packages.iter().any(|known| known.id == package.id) => {
                self.problems.push(PackageError {
                    package: package.id,
                    file: None,
                    problem: "another package already has this id".into(),
                });
            }
            Ok(package) => {
                self.packages.push(package);
                self.origins.push(origin);
            }
            Err(problem) => self.problems.push(problem),
        }
    }

    /// Where a package came from.
    pub fn origin(&self, id: &str) -> Option<Origin> {
        let index = self.packages.iter().position(|package| package.id == id)?;
        self.origins.get(index).copied()
    }

    /// Every story, with the package it came from.
    pub fn stories(&self) -> impl Iterator<Item = (&Package, &Story)> {
        self.packages
            .iter()
            .flat_map(|package| package.stories.iter().map(move |story| (package, story)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;

    #[test]
    fn every_official_package_loads() {
        let library = Library::load(None, &[]);
        assert!(library.problems.is_empty(), "{:?}", library.problems);
        assert!(library.stories().count() >= 1);
    }

    #[test]
    fn the_built_in_list_matches_the_folders_on_disk() {
        let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content");
        for (folder, files) in official() {
            let on_disk = read_folder(&content.join(folder)).unwrap();
            let built_in: Vec<&str> = files.iter().map(|(path, _)| *path).collect();
            let found: Vec<&str> = on_disk
                .keys()
                .map(String::as_str)
                .filter(|p| p.ends_with(".toml"))
                .collect();
            let mut built_in_sorted = built_in.clone();
            built_in_sorted.sort();
            assert_eq!(
                built_in_sorted, found,
                "{folder} has files that are not built in"
            );
        }
    }

    #[test]
    fn packages_in_the_folder_load_beside_hills_own_and_clash_with_nothing() {
        let folder = std::env::temp_dir().join(format!("hill-library-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content");
        // A copy of an official package is turned away for its id, and says so; a broken one
        // is reported; neither stops the rest loading.
        let copy = folder.join("again.formiga-hill");
        for (file, bytes) in read_folder(&content.join("first-picnic.formiga-hill")).unwrap() {
            let path = copy.join(&file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        let broken = folder.join("broken.formiga-hill");
        std::fs::create_dir_all(&broken).unwrap();
        std::fs::write(broken.join("manifest.toml"), b"package_id = ").unwrap();
        let library = Library::load(Some(&folder), &[]);
        let official = Library::load(None, &[]);
        assert_eq!(library.packages.len(), official.packages.len());
        assert_eq!(library.problems.len(), 2, "{:?}", library.problems);
        assert!(
            library
                .origins
                .iter()
                .all(|origin| *origin == Origin::Official)
        );
    }

    /// Plays every official story to its end with Desktop's sample colony, choosing each option
    /// in turn, so no branch can strand the cast or leave a line unfilled.
    #[test]
    fn every_official_story_plays_to_the_end_down_every_first_choice() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let library = Library::load(None, &[]);
        for (_, story) in library.stories() {
            for option in 0..4 {
                let mut room =
                    crate::clubhouse::Clubhouse::open(&cast, 0.0, &Default::default(), Vec::new());
                let mut director = Director::new(story.clone(), &cast, 1).unwrap();
                room.ground().reserve(director.players());
                let mut now = 0.0;
                let mut lines = 0;
                while !director.finished() && now < 600.0 {
                    now += 1.0 / 30.0;
                    room.tick(&cast, now);
                    director.run(room.ground(), &cast, now);
                    if director.shown().is_some() {
                        assert!(!director.shown().unwrap().text.contains('{'));
                        lines += 1;
                        director.read_on();
                    }
                    let choices = director.choices().len();
                    if choices > 0 {
                        director.choose(option.min(choices - 1));
                    }
                }
                assert!(director.finished(), "\"{}\" never ended", story.title);
                assert!(lines > 2, "\"{}\" said almost nothing", story.title);
            }
        }
    }
}
