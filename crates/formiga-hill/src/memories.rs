//! What Hill remembers of each colony it has hosted: the stories finished, the souvenirs kept, how
//! often it has visited. Hill's own record, kept in Hill's own data folder and keyed by the one-way
//! colony id Desktop sends, so Hill never learns more about a colony than the trip told it.
//!
//! Nothing here is a debt. A colony that stays away for a year comes back to everything it left.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const VERSION: u32 = 1;
const MAX_BYTES: u64 = 1024 * 1024;
const FILE: &str = "memories.json";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ColonyMemories {
    /// Stories finished, as `<package id>/<story id>`.
    #[serde(default)]
    pub stories: BTreeSet<String>,
    /// Souvenirs kept, in the order they were found.
    #[serde(default)]
    pub souvenirs: Vec<String>,
    #[serde(default)]
    pub visits: u32,
    /// The quickest each traveller has found everyone at hide-and-seek, in seconds, by its
    /// Desktop id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quickest_seekers: BTreeMap<String, f32>,
    /// Every find the colony has brought home from the Woods, by id: its journal.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub finds: BTreeMap<String, FindRecord>,
    /// Finds brought home and not yet placed on the Hilltop, and how many of each.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub satchel: BTreeMap<String, u32>,
    /// What stands on each of the Hilltop's spots.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hilltop: BTreeMap<u8, String>,
    /// Outings to the Woods, all told and by each traveller's Desktop id.
    #[serde(default)]
    pub outings: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outings_by: BTreeMap<String, u32>,
    /// Outings in a row that turned up nothing new: after a couple, the Woods makes sure.
    #[serde(default)]
    pub drought: u32,
}

/// One find in the journal.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindRecord {
    /// How many have been brought home.
    pub count: u32,
    /// Who led the outing that first found one, by Desktop id.
    pub first_by: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Book {
    version: u32,
    #[serde(default)]
    colonies: BTreeMap<String, ColonyMemories>,
}

/// The memories of one colony, and where to keep them.
pub struct Memories {
    path: Option<PathBuf>,
    colony: String,
    book: Book,
}

impl Memories {
    /// Hill's data folder: `FORMIGA_HILL_DATA_DIR` if set (for development, as Desktop's
    /// `FORMIGA_DATA_DIR` is), otherwise the platform's usual place.
    pub fn folder() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("FORMIGA_HILL_DATA_DIR").filter(|dir| !dir.is_empty()) {
            return Some(PathBuf::from(dir));
        }
        directories::ProjectDirs::from("com", "Formiga", "Formiga Hill")
            .map(|dirs| dirs.data_dir().to_owned())
    }

    /// Opens the memories kept in `folder` for `colony`. A missing or unreadable file starts a
    /// fresh book; an unreadable one is set aside first, never overwritten.
    pub fn open(folder: Option<&Path>, colony: &str) -> Self {
        let path = folder.map(|folder| folder.join(FILE));
        let book = path.as_deref().map(read).unwrap_or_default();
        Self {
            path,
            colony: colony.to_owned(),
            book,
        }
    }

    pub fn colony(&self) -> &ColonyMemories {
        static EMPTY: std::sync::OnceLock<ColonyMemories> = std::sync::OnceLock::new();
        self.book
            .colonies
            .get(&self.colony)
            .unwrap_or_else(|| EMPTY.get_or_init(ColonyMemories::default))
    }

    fn colony_mut(&mut self) -> &mut ColonyMemories {
        self.book.colonies.entry(self.colony.clone()).or_default()
    }

    pub fn arrived(&mut self) {
        self.colony_mut().visits += 1;
        self.keep();
    }

    /// Remembers a story finished and whatever souvenirs it gave.
    pub fn finished(&mut self, story: String, souvenirs: &[String]) {
        let colony = self.colony_mut();
        colony.stories.insert(story);
        for souvenir in souvenirs {
            if !colony.souvenirs.contains(souvenir) {
                colony.souvenirs.push(souvenir.clone());
            }
        }
        self.keep();
    }

    /// Remembers a game of hide-and-seek in which `seeker` found everyone in `seconds`, and gives
    /// the Fairground's ticket the first time a game is seen through. Says whether it was that
    /// seeker's quickest yet.
    pub fn found_everyone(&mut self, seeker: u64, seconds: f32) -> bool {
        let colony = self.colony_mut();
        let best = colony
            .quickest_seekers
            .entry(seeker.to_string())
            .or_insert(f32::MAX);
        let quickest = seconds < *best;
        if quickest {
            *best = seconds;
        }
        let ticket = crate::story::souvenirs::FAIR_TICKET;
        if !colony.souvenirs.iter().any(|kept| kept == ticket) {
            colony.souvenirs.push(ticket.to_owned());
        }
        self.keep();
        quickest
    }

    /// Remembers an outing to the Woods: who went, and what came home in the basket, which goes
    /// into the journal and the satchel. Says which finds were new.
    pub fn back_from_the_woods(&mut self, party: &[u64], basket: &[&str]) -> Vec<&'static str> {
        let colony = self.colony_mut();
        colony.outings += 1;
        for id in party {
            *colony.outings_by.entry(id.to_string()).or_default() += 1;
        }
        let leader = party.first().map(u64::to_string).unwrap_or_default();
        let mut new = Vec::new();
        for id in basket {
            let Some(find) = crate::finds::find(id) else {
                continue;
            };
            let record = colony.finds.entry(find.id.to_owned()).or_default();
            if record.count == 0 {
                record.first_by.clone_from(&leader);
                new.push(find.id);
            }
            record.count += 1;
            *colony.satchel.entry(find.id.to_owned()).or_default() += 1;
        }
        colony.drought = if new.is_empty() {
            colony.drought + 1
        } else {
            0
        };
        self.keep();
        new
    }

    /// Stands a find from the satchel on a Hilltop spot. Whatever stood there goes back into the
    /// satchel. Says whether it was placed.
    pub fn place(&mut self, spot: u8, id: &str) -> bool {
        if usize::from(spot) >= crate::hilltop::SPOTS.len() {
            return false;
        }
        let colony = self.colony_mut();
        let Some(count) = colony.satchel.get_mut(id).filter(|count| **count > 0) else {
            return false;
        };
        *count -= 1;
        if *count == 0 {
            colony.satchel.remove(id);
        }
        if let Some(old) = colony.hilltop.insert(spot, id.to_owned()) {
            *colony.satchel.entry(old).or_default() += 1;
        }
        self.keep();
        true
    }

    /// Takes whatever stands on a spot back into the satchel.
    pub fn pick_up(&mut self, spot: u8) -> Option<String> {
        let colony = self.colony_mut();
        let id = colony.hilltop.remove(&spot)?;
        *colony.satchel.entry(id.clone()).or_default() += 1;
        self.keep();
        Some(id)
    }

    /// Moves what stands on one spot to another, swapping with whatever is there.
    pub fn move_piece(&mut self, from: u8, to: u8) {
        if usize::from(to) >= crate::hilltop::SPOTS.len() || from == to {
            return;
        }
        let colony = self.colony_mut();
        let Some(moving) = colony.hilltop.remove(&from) else {
            return;
        };
        if let Some(there) = colony.hilltop.insert(to, moving) {
            colony.hilltop.insert(from, there);
        }
        self.keep();
    }

    /// Writes the book to disk beside itself and renames it into place. A failure is reported
    /// and otherwise ignored: the visit goes on, it is only not remembered.
    fn keep(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let book = Book {
            version: VERSION,
            colonies: self.book.colonies.clone(),
        };
        if let Err(error) = write(path, &book) {
            eprintln!("formiga-hill: could not keep the colony's memories: {error}");
        }
    }
}

fn read(path: &Path) -> Book {
    let Ok(metadata) = fs::metadata(path) else {
        return Book::default();
    };
    let parsed = (metadata.len() <= MAX_BYTES)
        .then(|| fs::read(path).ok())
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<Book>(&bytes).ok())
        .filter(|book| book.version <= VERSION);
    parsed.unwrap_or_else(|| {
        // Keep what was there for whoever wants to look at it, and start again.
        let mut aside = path.as_os_str().to_owned();
        aside.push(".unreadable");
        let _ = fs::rename(path, PathBuf::from(aside));
        Book::default()
    })
}

fn write(path: &Path, book: &Book) -> std::io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let mut file = fs::File::create(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(book)?)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-memories-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn memories_survive_a_new_visit_and_stay_with_their_colony() {
        let dir = scratch("survive");
        let mut first = Memories::open(Some(&dir), "aaaaaaaaaaaaaaaa");
        first.arrived();
        first.finished("pkg/picnic".into(), &["picnic_ribbon".into()]);

        let again = Memories::open(Some(&dir), "aaaaaaaaaaaaaaaa");
        assert_eq!(again.colony().visits, 1);
        assert!(again.colony().stories.contains("pkg/picnic"));
        assert_eq!(again.colony().souvenirs, vec!["picnic_ribbon".to_owned()]);

        let other = Memories::open(Some(&dir), "bbbbbbbbbbbbbbbb");
        assert_eq!(other.colony(), &ColonyMemories::default());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_souvenir_is_kept_once() {
        let mut memories = Memories::open(None, "c");
        memories.finished("p/s".into(), &["oak_acorn".into()]);
        memories.finished("p/s".into(), &["oak_acorn".into()]);
        assert_eq!(memories.colony().souvenirs.len(), 1);
    }

    #[test]
    fn hide_and_seek_remembers_each_seekers_quickest_and_gives_one_ticket() {
        let mut memories = Memories::open(None, "c");
        assert!(memories.found_everyone(7, 40.0));
        assert!(!memories.found_everyone(7, 55.0));
        assert!(
            memories.found_everyone(9, 61.0),
            "each seeker has its own best"
        );
        assert!(memories.found_everyone(7, 21.5));
        assert_eq!(memories.colony().quickest_seekers["7"], 21.5);
        assert_eq!(memories.colony().quickest_seekers["9"], 61.0);
        assert_eq!(memories.colony().souvenirs, vec!["fair_ticket".to_owned()]);
    }

    #[test]
    fn finds_come_home_into_the_journal_and_the_satchel() {
        let mut memories = Memories::open(None, "c");
        let new =
            memories.back_from_the_woods(&[7, 9], &["pinecone", "pinecone", "geode", "nonsense"]);
        assert_eq!(new, vec!["pinecone", "geode"]);
        let colony = memories.colony();
        assert_eq!(colony.finds["pinecone"].count, 2);
        assert_eq!(colony.finds["geode"].first_by, "7");
        assert_eq!(colony.satchel["pinecone"], 2);
        assert_eq!(colony.outings_by["9"], 1);
        assert_eq!(colony.drought, 0);
        memories.back_from_the_woods(&[9], &["pinecone"]);
        assert_eq!(memories.colony().drought, 1, "nothing new that time");
        assert_eq!(memories.colony().finds["pinecone"].first_by, "7");
    }

    #[test]
    fn the_hilltop_takes_from_the_satchel_and_gives_back() {
        let mut memories = Memories::open(None, "c");
        memories.back_from_the_woods(&[7], &["pinecone", "geode"]);
        assert!(!memories.place(3, "brass_lens"), "not in the satchel");
        assert!(memories.place(3, "pinecone"));
        assert!(!memories.colony().satchel.contains_key("pinecone"));
        assert!(memories.place(3, "geode"), "a spot takes anything");
        assert_eq!(memories.colony().hilltop[&3], "geode");
        assert_eq!(
            memories.colony().satchel["pinecone"],
            1,
            "what stood there went back"
        );
        memories.move_piece(3, 10);
        assert_eq!(
            memories.colony().hilltop.get(&10).map(String::as_str),
            Some("geode")
        );
        assert_eq!(memories.pick_up(10).as_deref(), Some("geode"));
        assert!(memories.colony().hilltop.is_empty());
        assert!(!memories.place(99, "geode"), "no such spot");
    }

    #[test]
    fn an_unreadable_book_is_set_aside_not_lost() {
        let dir = scratch("unreadable");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE), b"not json at all").unwrap();
        let memories = Memories::open(Some(&dir), "c");
        assert_eq!(memories.colony().visits, 0);
        assert!(dir.join("memories.json.unreadable").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
