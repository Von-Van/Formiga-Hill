//! What Hill remembers of each colony it has hosted: the stories finished, the souvenirs kept, how
//! often it has visited. Hill's own record, kept in Hill's own data folder and keyed by the one-way
//! colony id Desktop sends, so Hill never learns more about a colony than the trip told it.
//!
//! Nothing here is a debt. A colony that stays away for a year comes back to everything it left.

use crate::hilltop::{Arrangement, Standing};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The book's format. Version 2 can keep something planted or built on the Hilltop, and growing
/// things lifted into the satchel; a version 1 book reads as it was, everything on its Hilltop
/// fully grown. A Hill older than the book sets it aside rather than losing what it cannot read.
const VERSION: u32 = 2;
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
    /// The quickest each traveller has hopped the sack race, in seconds, by its Desktop id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quickest_racers: BTreeMap<String, f32>,
    /// The highest each traveller has sent the high striker's puck, from 0 to 1 (the bell), by
    /// its Desktop id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub highest_strikes: BTreeMap<String, f32>,
    /// Every find the colony has brought home from the Woods, by id: its journal.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub finds: BTreeMap<String, FindRecord>,
    /// Finds brought home and not yet placed on the Hilltop, and how many of each.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub satchel: BTreeMap<String, u32>,
    /// What stands on each of the Hilltop's spots.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hilltop: Arrangement,
    /// Growing things lifted from the Hilltop into the satchel, each as far as it had grown, to
    /// be planted again just as they are.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lifted: Vec<Standing>,
    /// Outings to the Woods, all told and by each traveller's Desktop id.
    #[serde(default)]
    pub outings: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outings_by: BTreeMap<String, u32>,
    /// Outings in a row that turned up nothing new: after a couple, the Woods makes sure.
    #[serde(default)]
    pub drought: u32,
    /// Every kind of fish the colony has caught at the pool, by id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fish: BTreeMap<String, FishRecord>,
    #[serde(default)]
    pub fishing_trips: u32,
    /// Trips in a row that caught no new kind of fish.
    #[serde(default)]
    pub fish_drought: u32,
    /// How many times the colony has seen off the Cursor Sovereign.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub sovereign_bested: u32,
    /// Every kind of bug the colony has caught in the meadow, by id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bugs: BTreeMap<String, BugRecord>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bug_hunts: u32,
    /// Hunts in a row that caught no new kind of bug.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bug_drought: u32,
}

fn is_zero(count: &u32) -> bool {
    *count == 0
}

/// One kind of fish in the journal.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FishRecord {
    /// How many have been landed, and let go again.
    pub count: u32,
    /// The longest landed, in centimetres.
    pub longest: f32,
    /// Who was fishing when the first was landed, by Desktop id.
    pub first_by: String,
}

/// One kind of bug in the journal.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BugRecord {
    /// How many have been caught, and let go again.
    pub count: u32,
    /// The biggest caught, in millimetres across.
    pub biggest: f32,
    /// Who had the net when the first was caught, by Desktop id.
    pub first_by: String,
}

/// What a fishing trip or a bug hunt brought: kinds caught for the first time, kinds caught
/// longer or bigger than ever before, and finds that came home and are new to the journal.
#[derive(Debug, Default, PartialEq)]
pub struct Haul {
    pub new_kinds: Vec<&'static str>,
    pub longest_yet: Vec<&'static str>,
    pub new_finds: Vec<&'static str>,
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

    /// Remembers the train arriving: another visit, and everything planted on the Hilltop grows a
    /// stage. Says what has grown, as it is now, spot by spot.
    pub fn arrived(&mut self) -> Vec<(u8, Standing)> {
        let colony = self.colony_mut();
        colony.visits += 1;
        let mut grown = Vec::new();
        for (spot, standing) in &mut colony.hilltop {
            if standing.grow() {
                grown.push((*spot, standing.clone()));
            }
        }
        self.keep();
        grown
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

    /// Remembers a sack race seen through: each racer's time, by its Desktop id, and the rosette
    /// the first time. Says who beat a time of its own from an earlier race.
    pub fn raced(&mut self, times: &[(u64, f32)]) -> Vec<u64> {
        let colony = self.colony_mut();
        let mut quicker = Vec::new();
        for (id, seconds) in times {
            let best = colony
                .quickest_racers
                .entry(id.to_string())
                .or_insert(f32::MAX);
            if *seconds < *best {
                if *best < f32::MAX {
                    quicker.push(*id);
                }
                *best = *seconds;
            }
        }
        keep_souvenir(colony, crate::story::souvenirs::RACE_ROSETTE);
        self.keep();
        quicker
    }

    /// Remembers a game at the high striker seen through: how high each player sent the puck, by
    /// its Desktop id, and the bell the first time. Says who beat a swing of its own from an
    /// earlier game.
    pub fn struck(&mut self, heights: &[(u64, f32)]) -> Vec<u64> {
        let colony = self.colony_mut();
        let mut higher = Vec::new();
        for (id, height) in heights {
            match colony.highest_strikes.get_mut(&id.to_string()) {
                Some(best) if *height > *best => {
                    *best = *height;
                    higher.push(*id);
                }
                Some(_) => {}
                None => {
                    colony.highest_strikes.insert(id.to_string(), *height);
                }
            }
        }
        keep_souvenir(colony, crate::story::souvenirs::STRIKER_BELL);
        self.keep();
        higher
    }

    /// Remembers an outing to the Woods: who went, and what came home in the basket, which goes
    /// into the journal and the satchel. Says which finds were new.
    pub fn back_from_the_woods(&mut self, party: &[u64], basket: &[&str]) -> Vec<&'static str> {
        let colony = self.colony_mut();
        colony.outings += 1;
        for id in party {
            *colony.outings_by.entry(id.to_string()).or_default() += 1;
        }
        let new = keep_finds(colony, party, basket);
        colony.drought = if new.is_empty() {
            colony.drought + 1
        } else {
            0
        };
        self.keep();
        new
    }

    /// Remembers a fishing trip: the fish landed and their lengths go into the journal (they were
    /// let go), and anything that snagged into the journal and the satchel.
    pub fn back_from_fishing(
        &mut self,
        party: &[u64],
        creel: &[(&str, f32)],
        snagged: &[&str],
    ) -> Haul {
        let colony = self.colony_mut();
        colony.fishing_trips += 1;
        let angler = party.first().map(u64::to_string).unwrap_or_default();
        let mut haul = Haul::default();
        for (id, length) in creel {
            let Some(fish) = crate::fishing::fish::fish(id) else {
                continue;
            };
            let record = colony.fish.entry(fish.id.to_owned()).or_default();
            if record.count == 0 {
                record.first_by.clone_from(&angler);
                haul.new_kinds.push(fish.id);
            } else if *length > record.longest && !haul.longest_yet.contains(&fish.id) {
                haul.longest_yet.push(fish.id);
            }
            record.count += 1;
            record.longest = record.longest.max(*length);
        }
        colony.fish_drought = if haul.new_kinds.is_empty() {
            colony.fish_drought + 1
        } else {
            0
        };
        haul.new_finds = keep_finds(colony, party, snagged);
        self.keep();
        haul
    }

    /// Remembers a bug hunt: the bugs caught and their sizes go into the journal (they were let
    /// go), and anything else the net brought up into the journal and the satchel.
    pub fn back_from_the_meadow(
        &mut self,
        party: &[u64],
        jar: &[(&str, f32)],
        netted: &[&str],
    ) -> Haul {
        let colony = self.colony_mut();
        colony.bug_hunts += 1;
        let netter = party.first().map(u64::to_string).unwrap_or_default();
        let mut haul = Haul::default();
        for (id, size) in jar {
            let Some(bug) = crate::meadow::bugs::bug(id) else {
                continue;
            };
            let record = colony.bugs.entry(bug.id.to_owned()).or_default();
            if record.count == 0 {
                record.first_by.clone_from(&netter);
                haul.new_kinds.push(bug.id);
            } else if *size > record.biggest && !haul.longest_yet.contains(&bug.id) {
                haul.longest_yet.push(bug.id);
            }
            record.count += 1;
            record.biggest = record.biggest.max(*size);
        }
        colony.bug_drought = if haul.new_kinds.is_empty() {
            colony.bug_drought + 1
        } else {
            0
        };
        haul.new_finds = keep_finds(colony, party, netted);
        self.keep();
        haul
    }

    /// Remembers the Cursor Sovereign seen off by `party`. The first time, its arrow comes home for
    /// the Hilltop; after that it is only a story worth telling again. Says whether it was the
    /// first time.
    pub fn bested_the_sovereign(&mut self, party: &[u64]) -> bool {
        let colony = self.colony_mut();
        colony.sovereign_bested += 1;
        let first = colony.sovereign_bested == 1;
        if first {
            keep_finds(colony, party, &["sovereign_arrow"]);
        }
        self.keep();
        first
    }

    /// Stands a find from the satchel on a Hilltop spot, planted if it is one that grows. Whatever
    /// stood there goes back into the satchel. Says whether it was placed.
    pub fn place(&mut self, spot: u8, id: &str) -> bool {
        if usize::from(spot) >= crate::hilltop::SPOTS.len() {
            return false;
        }
        let colony = self.colony_mut();
        if !take_from_satchel(colony, id, 1) {
            return false;
        }
        if let Some(old) = colony.hilltop.insert(spot, Standing::from_satchel(id)) {
            put_back(colony, old);
        }
        self.keep();
        true
    }

    /// Plants something lifted into the satchel again, on a Hilltop spot, as far as it had grown.
    /// Whatever stood there goes back into the satchel. Says whether it was planted.
    pub fn replant(&mut self, spot: u8, lifted: &Standing) -> bool {
        if usize::from(spot) >= crate::hilltop::SPOTS.len() {
            return false;
        }
        let colony = self.colony_mut();
        let Some(index) = colony.lifted.iter().position(|kept| kept == lifted) else {
            return false;
        };
        let standing = colony.lifted.remove(index);
        if let Some(old) = colony.hilltop.insert(spot, standing) {
            put_back(colony, old);
        }
        self.keep();
        true
    }

    /// Builds a plan on a Hilltop spot from finds in the satchel, taking exactly what it needs.
    /// Whatever stood there goes back into the satchel. Says whether it was built.
    pub fn build(&mut self, spot: u8, plan: &str) -> bool {
        let Some(plan) = crate::finds::plans::plan(plan) else {
            return false;
        };
        if usize::from(spot) >= crate::hilltop::SPOTS.len() {
            return false;
        }
        let colony = self.colony_mut();
        if !plan.ready(&colony.satchel) {
            return false;
        }
        for &(id, count) in plan.needs {
            take_from_satchel(colony, id, count);
        }
        if let Some(old) = colony.hilltop.insert(spot, Standing::built(plan.id)) {
            put_back(colony, old);
        }
        self.keep();
        true
    }

    /// Takes apart something built on a spot, every find it took going back into the satchel.
    /// Says what it was, if there was something built there.
    pub fn take_apart(&mut self, spot: u8) -> Option<Standing> {
        let colony = self.colony_mut();
        let standing = colony.hilltop.get(&spot)?;
        standing.comes_apart()?;
        let standing = colony.hilltop.remove(&spot)?;
        put_back(colony, standing.clone());
        self.keep();
        Some(standing)
    }

    /// Takes whatever stands on a spot back into the satchel: a find among the finds, anything
    /// growing lifted whole, anything built taken apart into its finds.
    pub fn pick_up(&mut self, spot: u8) -> Option<Standing> {
        let colony = self.colony_mut();
        let standing = colony.hilltop.remove(&spot)?;
        put_back(colony, standing.clone());
        self.keep();
        Some(standing)
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

/// Takes `count` of a find out of the satchel, if there are that many there. Says whether it did.
fn take_from_satchel(colony: &mut ColonyMemories, id: &str, count: u32) -> bool {
    let Some(there) = colony.satchel.get_mut(id).filter(|there| **there >= count) else {
        return false;
    };
    *there -= count;
    if *there == 0 {
        colony.satchel.remove(id);
    }
    true
}

/// Puts something taken off the Hilltop back into the satchel: a find that never grows among the
/// finds, anything planted lifted whole, so nothing is ever less grown for having been moved, and
/// anything built taken apart, every find it took given back.
fn put_back(colony: &mut ColonyMemories, standing: Standing) {
    if let Some(finds) = standing.comes_apart() {
        for &(id, count) in finds {
            *colony.satchel.entry(id.to_owned()).or_default() += count;
        }
    } else if standing.back_among_finds() {
        *colony.satchel.entry(standing.id().to_owned()).or_default() += 1;
    } else {
        colony.lifted.push(standing);
    }
}

/// Keeps a souvenir in the display case, once.
fn keep_souvenir(colony: &mut ColonyMemories, souvenir: &str) {
    if !colony.souvenirs.iter().any(|kept| kept == souvenir) {
        colony.souvenirs.push(souvenir.to_owned());
    }
}

/// Puts finds into the journal and the satchel. Says which were new.
fn keep_finds(colony: &mut ColonyMemories, party: &[u64], finds: &[&str]) -> Vec<&'static str> {
    let leader = party.first().map(u64::to_string).unwrap_or_default();
    let mut new = Vec::new();
    for id in finds {
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
    new
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
    fn the_sack_race_remembers_each_racers_quickest_and_gives_one_rosette() {
        let mut memories = Memories::open(None, "c");
        assert!(
            memories.raced(&[(7, 14.0), (9, 18.5)]).is_empty(),
            "a first race is nobody's quickest yet: there was nothing to beat"
        );
        assert_eq!(memories.raced(&[(9, 16.0), (7, 15.0)]), vec![9]);
        assert_eq!(memories.colony().quickest_racers["7"], 14.0);
        assert_eq!(memories.colony().quickest_racers["9"], 16.0);
        memories.raced(&[(11, 30.0)]);
        assert_eq!(memories.colony().quickest_racers["11"], 30.0);
        assert_eq!(memories.colony().souvenirs, vec!["race_rosette".to_owned()]);
    }

    #[test]
    fn the_high_striker_remembers_each_best_swing_and_gives_one_bell() {
        let mut memories = Memories::open(None, "c");
        assert!(memories.struck(&[(7, 0.6), (9, 1.0)]).is_empty());
        assert_eq!(memories.struck(&[(7, 0.8), (9, 0.4)]), vec![7]);
        assert_eq!(memories.colony().highest_strikes["7"], 0.8);
        assert_eq!(memories.colony().highest_strikes["9"], 1.0);
        memories.found_everyone(7, 30.0);
        memories.struck(&[(7, 0.2)]);
        assert_eq!(
            memories.colony().souvenirs,
            vec!["striker_bell".to_owned(), "fair_ticket".to_owned()]
        );
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
    fn fish_are_remembered_and_let_go_and_snags_come_home() {
        let mut memories = Memories::open(None, "c");
        let haul = memories.back_from_fishing(
            &[7, 9],
            &[("perch", 21.0), ("perch", 25.5), ("carp", 40.0)],
            &["geode"],
        );
        assert_eq!(haul.new_kinds, vec!["perch", "carp"]);
        assert_eq!(
            haul.longest_yet,
            vec!["perch"],
            "the second perch beat the first"
        );
        assert_eq!(haul.new_finds, vec!["geode"]);
        let colony = memories.colony();
        assert_eq!(colony.fish["perch"].count, 2);
        assert_eq!(colony.fish["perch"].longest, 25.5);
        assert_eq!(colony.fish["carp"].first_by, "7");
        assert_eq!(
            colony.satchel["geode"], 1,
            "a snag comes home for the Hilltop"
        );
        assert_eq!(colony.fish_drought, 0);
        assert_eq!(colony.outings, 0, "fishing is not a rummage");
        let haul = memories.back_from_fishing(&[9], &[("perch", 20.0)], &[]);
        assert!(haul.new_kinds.is_empty() && haul.longest_yet.is_empty());
        assert_eq!(memories.colony().fish_drought, 1);
        assert_eq!(memories.colony().fishing_trips, 2);
    }

    #[test]
    fn bugs_are_remembered_and_let_go_and_whatever_else_was_netted_comes_home() {
        let mut memories = Memories::open(None, "c");
        let haul = memories.back_from_the_meadow(
            &[7, 9],
            &[("ladybird", 6.0), ("ladybird", 7.5), ("moon_moth", 110.0)],
            &["jay_feather"],
        );
        assert_eq!(haul.new_kinds, vec!["ladybird", "moon_moth"]);
        assert_eq!(haul.longest_yet, vec!["ladybird"]);
        assert_eq!(haul.new_finds, vec!["jay_feather"]);
        let colony = memories.colony();
        assert_eq!(colony.bugs["ladybird"].count, 2);
        assert_eq!(colony.bugs["ladybird"].biggest, 7.5);
        assert_eq!(colony.bugs["moon_moth"].first_by, "7");
        assert_eq!(colony.satchel["jay_feather"], 1);
        assert_eq!(colony.bug_drought, 0);
        let haul = memories.back_from_the_meadow(&[9], &[("ladybird", 5.0)], &[]);
        assert!(haul.new_kinds.is_empty() && haul.longest_yet.is_empty());
        assert_eq!(memories.colony().bug_drought, 1);
        assert_eq!(memories.colony().bug_hunts, 2);
    }

    #[test]
    fn the_sovereign_leaves_its_arrow_only_once() {
        let mut memories = Memories::open(None, "c");
        assert!(memories.bested_the_sovereign(&[7, 9]));
        assert_eq!(memories.colony().satchel["sovereign_arrow"], 1);
        assert_eq!(memories.colony().finds["sovereign_arrow"].first_by, "7");
        assert!(!memories.bested_the_sovereign(&[9]));
        assert_eq!(
            memories.colony().satchel["sovereign_arrow"],
            1,
            "one arrow, ever"
        );
        assert_eq!(memories.colony().sovereign_bested, 2);
    }

    #[test]
    fn the_hilltop_takes_from_the_satchel_and_gives_back() {
        let mut memories = Memories::open(None, "c");
        memories.back_from_the_woods(&[7], &["pinecone", "geode"]);
        assert!(!memories.place(3, "brass_lens"), "not in the satchel");
        assert!(memories.place(3, "pinecone"));
        assert!(!memories.colony().satchel.contains_key("pinecone"));
        assert!(memories.place(3, "geode"), "a spot takes anything");
        assert_eq!(memories.colony().hilltop[&3], Standing::from("geode"));
        assert_eq!(
            memories.colony().satchel["pinecone"],
            1,
            "what stood there went back"
        );
        memories.move_piece(3, 10);
        assert_eq!(
            memories.colony().hilltop.get(&10),
            Some(&Standing::from("geode"))
        );
        assert_eq!(memories.pick_up(10), Some(Standing::from("geode")));
        assert!(memories.colony().hilltop.is_empty());
        assert_eq!(memories.colony().satchel["geode"], 1);
        assert!(memories.colony().lifted.is_empty());
        assert!(!memories.place(99, "geode"), "no such spot");
    }

    #[test]
    fn something_planted_grows_a_stage_with_each_visit_and_never_shrinks() {
        let mut memories = Memories::open(None, "c");
        memories.back_from_the_woods(&[7], &["bluebell_bulb", "pinecone"]);
        assert!(memories.place(4, "bluebell_bulb"));
        assert!(memories.place(5, "pinecone"));
        assert_eq!(memories.colony().hilltop[&4].stage(), Some(0));
        let stages = crate::finds::growing::growth("bluebell_bulb")
            .unwrap()
            .stages
            .len();
        let mut was = 0;
        for visit in 1..=stages {
            let grown = memories.arrived();
            assert_eq!(grown.len(), 1, "only the bulb grows, visit {visit}");
            assert_eq!(grown[0].0, 4);
            // Fully grown counts as further on than any stage.
            let now = memories.colony().hilltop[&4].stage().unwrap_or(u8::MAX);
            assert!(now > was, "it went from {was} to {now}");
            was = now;
        }
        assert_eq!(
            memories.colony().hilltop[&4],
            Standing::from("bluebell_bulb"),
            "after its last stage, a full clump of bluebells"
        );
        for _ in 0..5 {
            assert!(memories.arrived().is_empty(), "grown is grown");
        }
        assert_eq!(
            memories.colony().hilltop[&4],
            Standing::from("bluebell_bulb")
        );
        assert_eq!(memories.colony().hilltop[&5], Standing::from("pinecone"));
    }

    #[test]
    fn growth_goes_with_what_was_planted_moved_or_lifted() {
        let mut memories = Memories::open(None, "c");
        memories.back_from_the_woods(&[7], &["acorn_stash", "acorn_stash"]);
        assert!(memories.place(2, "acorn_stash"));
        memories.arrived();
        memories.move_piece(2, 12);
        assert_eq!(memories.colony().hilltop[&12].stage(), Some(1), "moved");
        let lifted = memories.pick_up(12).unwrap();
        assert_eq!(lifted.stage(), Some(1));
        assert_eq!(
            memories.colony().satchel["acorn_stash"],
            1,
            "the seedling is lifted, not turned back into an acorn"
        );
        assert_eq!(memories.colony().lifted, vec![lifted.clone()]);
        memories.arrived();
        assert_eq!(
            memories.colony().lifted[0].stage(),
            Some(1),
            "nothing grows in the satchel, and nothing shrinks there"
        );
        assert!(memories.replant(7, &lifted));
        assert_eq!(memories.colony().hilltop[&7].stage(), Some(1));
        assert!(memories.colony().lifted.is_empty());
        assert!(!memories.replant(8, &lifted), "it was only lifted once");
        // Planting the other acorn on top lifts the seedling again, still as grown.
        assert!(memories.place(7, "acorn_stash"));
        assert_eq!(memories.colony().hilltop[&7].stage(), Some(0));
        assert_eq!(memories.colony().lifted, [lifted]);
    }

    #[test]
    fn building_takes_exactly_its_finds_and_taking_it_apart_gives_them_back() {
        let mut memories = Memories::open(None, "c");
        memories.back_from_the_woods(
            &[7],
            &["smooth_pebble", "smooth_pebble", "geode", "brass_lens"],
        );
        assert!(
            !memories.build(4, "grand_cairn"),
            "two pebbles are not enough"
        );
        assert!(!memories.build(4, "no_such_plan"));
        memories.back_from_the_woods(&[7], &["smooth_pebble", "smooth_pebble"]);
        let before = memories.colony().satchel.clone();
        assert!(memories.place(4, "geode"));
        assert!(memories.build(4, "grand_cairn"), "built over the geode");
        let colony = memories.colony();
        assert_eq!(colony.hilltop[&4], Standing::built("grand_cairn"));
        assert_eq!(colony.satchel["smooth_pebble"], 1, "three of four taken");
        assert_eq!(colony.satchel["geode"], 1, "what stood there went back");
        assert_eq!(colony.satchel["brass_lens"], 1, "nothing else touched");
        memories.move_piece(4, 9);
        assert_eq!(memories.take_apart(9), Some(Standing::built("grand_cairn")));
        assert!(memories.colony().hilltop.is_empty());
        assert_eq!(memories.colony().satchel, before, "every find given back");
        assert!(memories.colony().lifted.is_empty());
        assert_eq!(memories.take_apart(9), None, "nothing left to take apart");
        // Putting it back in the satchel takes it apart too, and a find is not taken apart.
        assert!(memories.build(1, "grand_cairn"));
        assert!(memories.place(2, "geode"));
        assert_eq!(memories.take_apart(2), None);
        memories.pick_up(1);
        assert_eq!(memories.colony().satchel["smooth_pebble"], 4);
    }

    #[test]
    fn an_old_book_reads_with_what_stood_on_its_hilltop_fully_grown() {
        let dir = scratch("old-book");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(FILE),
            r#"{
                "version": 1,
                "colonies": {
                    "c": {
                        "visits": 4,
                        "satchel": {"bluebell_bulb": 2},
                        "hilltop": {"3": "bluebell_bulb", "9": "acorn_stash", "12": "geode"},
                        "finds": {"geode": {"count": 1, "first_by": "7"}}
                    }
                }
            }"#,
        )
        .unwrap();
        let mut memories = Memories::open(Some(&dir), "c");
        let colony = memories.colony();
        assert_eq!(colony.visits, 4);
        assert_eq!(colony.hilltop.len(), 3);
        assert!(
            colony.hilltop.values().all(|standing| !standing.growing()),
            "nothing standing before anything grew is any less than it was"
        );
        assert_eq!(colony.hilltop[&3], Standing::from("bluebell_bulb"));
        assert_eq!(colony.satchel["bluebell_bulb"], 2);
        assert!(memories.arrived().is_empty(), "nothing to grow");
        let again = Memories::open(Some(&dir), "c");
        assert_eq!(again.colony().hilltop[&9], Standing::from("acorn_stash"));
        assert_eq!(again.colony().visits, 5);
        fs::remove_dir_all(dir).unwrap();
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
