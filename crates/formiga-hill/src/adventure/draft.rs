//! An adventure as the Woods' map editor holds it while someone makes it: places put down on
//! the map, paths joined between them, and plain words for whatever is said along the way.
//! Nothing here is new to adventures: a draft becomes an ordinary package, in exactly the format
//! `docs/PACKAGES.md` describes, and is loaded by the same loader as any other before it is
//! tried, kept in the Woods or shared.
//!
//! A draft can be unfinished, or wrong, at any point: a place with no path to it, no ending yet.
//! It says what is wrong in plain words, and can be put down and picked up again as it is.

use super::{Adventure, BOTTOM, CLOSEST, LEFT, MOST_PATHS, MOST_PLACES, RIGHT, TOP};
use crate::expedition::map::{Map, Opener, Path, Place, Site, Way};
use crate::story::{self, Files, Package};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use toml::{Table, Value};

/// Where a made adventure's package ids begin, and how the adventure inside is known.
pub const MADE_PREFIX: &str = "made.hill.adventure-";
const ADVENTURE_ID: &str = "adventure";
const ADVENTURE_FILE: &str = "content/adventure.toml";
const LINES_FILE: &str = "localization/en.toml";
pub const MAX_TITLE_CHARS: usize = 40;
pub const MAX_NAME_CHARS: usize = 32;
pub const MAX_LINE_CHARS: usize = 200;
/// What is said at an ending nobody has written words for.
const PLAIN_ENDING: &str = "And that is the end of the adventure.";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    pub title: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub author: String,
    /// Made once when the draft is begun, so every version kept or shared replaces the last.
    pub package_id: String,
    /// Counts each time the adventure is kept in the Woods or shared.
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub places: Vec<DraftPlace>,
    #[serde(default)]
    pub paths: Vec<DraftPath>,
    /// Where everyone comes in.
    #[serde(default)]
    pub start: usize,
    /// The next number for a new place; places keep theirs, so moving one never breaks a path.
    #[serde(default)]
    pub next_place: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftPlace {
    pub number: u32,
    /// A `Site` key: what kind of place it is.
    pub site: String,
    pub at: (f32, f32),
    /// What it is called; blank for what its kind is called.
    #[serde(default)]
    pub name: String,
    /// What is said on arriving, if anything.
    #[serde(default)]
    pub arrive: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub ending: bool,
    /// What is said on getting there, if it is an ending.
    #[serde(default)]
    pub ending_words: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftPath {
    /// The places it joins, by number.
    pub ends: (u32, u32),
    /// "open", or the key of whoever opens it.
    #[serde(default = "open")]
    pub way: String,
    /// What a way only some company takes is called.
    #[serde(default)]
    pub name: String,
}

fn open() -> String {
    "open".to_owned()
}

impl DraftPlace {
    pub fn site(&self) -> Site {
        Site::from_key(&self.site).unwrap_or(Site::Glade)
    }

    /// What it is called, as the map letters it.
    pub fn called(&self) -> String {
        if self.name.trim().is_empty() {
            self.site().name().to_owned()
        } else {
            self.name.trim().to_owned()
        }
    }
}

impl DraftPath {
    pub fn opener(&self) -> Option<Opener> {
        super::opener_from_key(&self.way)
    }
}

/// Whether a spot is somewhere a place can stand.
pub fn on_the_map((x, y): (f32, f32)) -> bool {
    (LEFT..=RIGHT).contains(&x) && (TOP..=BOTTOM).contains(&y)
}

/// The nearest spot to `at` a place can stand.
pub fn onto_the_map((x, y): (f32, f32)) -> (f32, f32) {
    (x.clamp(LEFT, RIGHT).round(), y.clamp(TOP, BOTTOM).round())
}

impl Draft {
    /// A new adventure, with the way in from the lane already on the map: `key` makes its
    /// package id, which never changes after.
    pub fn new(key: &str, author: &str) -> Self {
        Self {
            title: String::new(),
            about: String::new(),
            author: author.to_owned(),
            package_id: format!("{MADE_PREFIX}{}", crate::sharing::slug(key, 24)),
            version: 0,
            places: vec![DraftPlace {
                number: 1,
                site: Site::Edge.key().to_owned(),
                at: (36.0, 186.0),
                name: String::new(),
                arrive: String::new(),
                hidden: false,
                ending: false,
                ending_words: String::new(),
            }],
            paths: Vec::new(),
            start: 0,
            next_place: 2,
        }
    }

    /// For review: an adventure half made, as someone might leave one, with a thing or two
    /// still to do.
    pub fn sample() -> Self {
        let mut draft = Self::new("sample", "Someone at the Hill");
        draft.title = "The Bell in the Fog".into();
        draft.about = "Somewhere in the Woods a bell rings, but only on foggy days.".into();
        let places = [
            (Site::Hedgerow, (104.0, 196.0)),
            (Site::Glade, (160.0, 146.0)),
            (Site::Signpost, (236.0, 110.0)),
            (Site::Pool, (300.0, 176.0)),
            (Site::Log, (96.0, 98.0)),
        ];
        for (site, at) in places {
            let _ = draft.add_place(site, at);
        }
        // The way in, the hedgerow, the glade, the signpost, the pool and the log, joined up but
        // for the pool, which only the old way reaches yet.
        for (a, b) in [(0, 1), (1, 2), (0, 2), (2, 3), (2, 5), (5, 3), (3, 4)] {
            let _ = draft.join(a, b);
        }
        draft.places[2].name = "the bell oak".into();
        draft.places[2].arrive = "A bell rings, somewhere further in.".into();
        draft.places[4].ending = true;
        draft.places[4].hidden = true;
        draft.places[4].name = "the fog pool".into();
        draft.places[4].ending_words =
            "The bell hangs over the pool, and {leader} rings it.".into();
        draft.paths[6].way = "reader".into();
        draft.paths[6].name = "the bell-ringers' way".into();
        draft
    }

    /// The draft kept as text, for `sharing::Drafts`.
    pub fn to_text(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// A draft read back from its text, if it is one.
    pub fn from_text(text: &str) -> Option<Self> {
        let draft: Self = serde_json::from_str(text).ok()?;
        let fits = draft.places.len() <= MOST_PLACES
            && draft.paths.len() <= MOST_PATHS
            && draft.start < draft.places.len().max(1)
            && draft.package_id.starts_with(MADE_PREFIX);
        fits.then_some(draft)
    }

    /// Where the place with this number stands among the places, if it is still on the map.
    pub fn index(&self, number: u32) -> Option<usize> {
        self.places.iter().position(|place| place.number == number)
    }

    /// The place standing under a point, if any, nearest first.
    pub fn place_at(&self, (x, y): (f32, f32)) -> Option<usize> {
        self.places
            .iter()
            .enumerate()
            .map(|(index, place)| {
                let (px, py) = place.at;
                let near = crate::playground::distance((x, y), (px, py - 10.0))
                    .min(crate::playground::distance((x, y), (px, py)));
                (index, near)
            })
            .filter(|(_, near)| *near <= 16.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The path drawn under a point, if any.
    pub fn path_at(&self, at: (f32, f32)) -> Option<usize> {
        let map = self.map();
        (0..self.paths.len())
            .map(|index| {
                let points = map.walk(index, map.paths[index].ends.0);
                let near = points
                    .windows(2)
                    .map(|pair| from_segment(at, pair[0], pair[1]))
                    .fold(f32::MAX, f32::min);
                (index, near)
            })
            .filter(|(_, near)| *near <= 4.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// Why a place can't stand at `at`, if it can't: off the map, or too close to another.
    fn room_at(&self, at: (f32, f32), but: Option<usize>) -> Result<(), String> {
        if !on_the_map(at) {
            return Err("That's too near the edge of the map.".into());
        }
        if self.places.iter().enumerate().any(|(index, place)| {
            Some(index) != but && crate::playground::distance(place.at, at) < CLOSEST
        }) {
            return Err("That's too close to another place.".into());
        }
        Ok(())
    }

    /// Puts a new place on the map: its index.
    pub fn add_place(&mut self, site: Site, at: (f32, f32)) -> Result<usize, String> {
        if self.places.len() >= MOST_PLACES {
            return Err(format!("An adventure has at most {MOST_PLACES} places."));
        }
        let at = onto_the_map(at);
        self.room_at(at, None)?;
        self.places.push(DraftPlace {
            number: self.next_place,
            site: site.key().to_owned(),
            at,
            name: String::new(),
            arrive: String::new(),
            hidden: false,
            ending: false,
            ending_words: String::new(),
        });
        self.next_place += 1;
        Ok(self.places.len() - 1)
    }

    /// Moves a place, if it has room there; says whether it moved.
    pub fn move_place(&mut self, index: usize, at: (f32, f32)) -> bool {
        let at = onto_the_map(at);
        if index >= self.places.len() || self.room_at(at, Some(index)).is_err() {
            return false;
        }
        self.places[index].at = at;
        true
    }

    /// Takes a place off the map, with its paths. The way in stays.
    pub fn remove_place(&mut self, index: usize) -> bool {
        if index >= self.places.len() || index == self.start {
            return false;
        }
        let number = self.places[index].number;
        self.places.remove(index);
        self.paths
            .retain(|path| path.ends.0 != number && path.ends.1 != number);
        if self.start > index {
            self.start -= 1;
        }
        true
    }

    /// Joins two places with an open path, if they aren't already: its index.
    pub fn join(&mut self, a: usize, b: usize) -> Result<usize, String> {
        if a == b || a >= self.places.len() || b >= self.places.len() {
            return Err("A path goes from one place to another.".into());
        }
        if self.paths.len() >= MOST_PATHS {
            return Err(format!("An adventure has at most {MOST_PATHS} paths."));
        }
        let (na, nb) = (self.places[a].number, self.places[b].number);
        let joined = |path: &DraftPath| {
            path.way == "open" && (path.ends == (na, nb) || path.ends == (nb, na))
        };
        if self.paths.iter().any(joined) {
            return Err("There's a path between those two already.".into());
        }
        self.paths.push(DraftPath {
            ends: (na, nb),
            way: open(),
            name: String::new(),
        });
        Ok(self.paths.len() - 1)
    }

    pub fn remove_path(&mut self, index: usize) {
        if index < self.paths.len() {
            self.paths.remove(index);
        }
    }

    /// Makes a place where everyone comes in.
    pub fn set_start(&mut self, index: usize) {
        if index < self.places.len() {
            self.start = index;
            self.places[index].hidden = false;
        }
    }

    /// The draft's map as it stands, right or wrong, for drawing: every place in sight.
    pub fn map(&self) -> Map {
        let places = self
            .places
            .iter()
            .map(|place| {
                let site = place.site();
                Place {
                    id: Cow::Owned(format!("p{}", place.number)),
                    name: Cow::Owned(place.called()),
                    site,
                    stop: site.stop(),
                    at: place.at,
                    depth: 0,
                    hidden: place.hidden,
                    ending: place.ending,
                }
            })
            .collect();
        let paths = self
            .paths
            .iter()
            .filter_map(|path| {
                let a = self.index(path.ends.0)?;
                let b = self.index(path.ends.1)?;
                let way = match path.opener() {
                    None => Way::Open,
                    Some(opener) => Way::Shortcut(Cow::Owned(path.called()), opener),
                };
                let mut length_points = vec![self.places[a].at, self.places[b].at];
                length_points.dedup();
                Some(Path {
                    ends: (a, b),
                    length: super::drawn_length(&length_points, path.opener().is_some()),
                    way,
                    bends: Cow::Owned(Vec::new()),
                })
            })
            .collect();
        Map {
            title: Some(if self.title.trim().is_empty() {
                "A new adventure".to_owned()
            } else {
                self.title.trim().to_owned()
            }),
            places,
            paths,
            start: self.start.min(self.places.len().saturating_sub(1)),
        }
    }

    /// What is wrong with the draft, in plain words, the most pressing first; nothing, once it
    /// is ready to try, keep or share.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.title.trim().is_empty() {
            problems.push("Give the adventure a name.".to_owned());
        }
        if self.author.trim().is_empty() {
            problems.push("Say who made it.".to_owned());
        }
        if self.places.len() < 2 {
            problems.push("Put some places on the map.".to_owned());
        }
        if !self.places.iter().any(|place| place.ending) {
            problems.push("Choose where it ends: make a place an ending.".to_owned());
        }
        if self.places.len() >= 2 {
            let map = self.map();
            let reached = map.reachable(|path| path.way == Way::Open);
            for (index, place) in self.places.iter().enumerate() {
                if !reached.contains(&index) {
                    problems.push(format!(
                        "Nobody can get to {} yet: join it with an open path.",
                        place.called()
                    ));
                }
            }
        }
        for path in &self.paths {
            if path.opener() == Some(Opener::Reader) {
                let signpost = [path.ends.0, path.ends.1].iter().any(|number| {
                    self.index(*number)
                        .is_some_and(|index| self.places[index].site() == Site::Signpost)
                });
                if !signpost {
                    problems.push("An old way is read off a signpost: start it at one.".to_owned());
                }
            }
        }
        if problems.is_empty()
            && let Err(problem) = self.package()
        {
            problems.push(plain(&problem.problem));
        }
        problems
    }

    /// The draft as a package's files, in the documented format.
    pub fn files(&self) -> Files {
        let mut lines = Table::new();
        let mut put = |key: String, text: &str| {
            lines.insert(key.clone(), Value::String(one_line(text)));
            key
        };
        let mut header = Table::new();
        header.insert("id".into(), ADVENTURE_ID.into());
        header.insert("title".into(), put("title".into(), &self.title).into());
        if !self.about.trim().is_empty() {
            header.insert("about".into(), put("about".into(), &self.about).into());
        }
        header.insert(
            "start".into(),
            format!("p{}", self.places.get(self.start).map_or(1, |p| p.number)).into(),
        );
        let mut places = Vec::new();
        for place in &self.places {
            let id = format!("p{}", place.number);
            let mut table = Table::new();
            table.insert("id".into(), id.clone().into());
            table.insert("site".into(), place.site().key().into());
            table.insert(
                "at".into(),
                Value::Array(vec![
                    Value::Integer(place.at.0.round() as i64),
                    Value::Integer(place.at.1.round() as i64),
                ]),
            );
            if !place.name.trim().is_empty() {
                table.insert("name".into(), put(format!("{id}_name"), &place.name).into());
            }
            if !place.arrive.trim().is_empty() {
                table.insert(
                    "arrive".into(),
                    put(format!("{id}_arrive"), &place.arrive).into(),
                );
            }
            if place.hidden {
                table.insert("hidden".into(), true.into());
            }
            if place.ending {
                let words = if place.ending_words.trim().is_empty() {
                    PLAIN_ENDING
                } else {
                    &place.ending_words
                };
                table.insert("ending".into(), put(format!("{id}_ending"), words).into());
            }
            places.push(Value::Table(table));
        }
        let mut paths = Vec::new();
        for (number, path) in self.paths.iter().enumerate() {
            let mut table = Table::new();
            table.insert(
                "between".into(),
                Value::Array(vec![
                    format!("p{}", path.ends.0).into(),
                    format!("p{}", path.ends.1).into(),
                ]),
            );
            if path.opener().is_some() {
                table.insert("way".into(), path.way.clone().into());
                table.insert(
                    "name".into(),
                    put(format!("way{}_name", number + 1), &path.called()).into(),
                );
            }
            paths.push(Value::Table(table));
        }
        let mut adventure = Table::new();
        adventure.insert("adventure".into(), Value::Table(header));
        adventure.insert("places".into(), Value::Array(places));
        adventure.insert("paths".into(), Value::Array(paths));

        let mut manifest = Table::new();
        manifest.insert("package_id".into(), self.package_id.clone().into());
        manifest.insert("title".into(), one_line(&self.title).into());
        manifest.insert("author".into(), one_line(&self.author).into());
        manifest.insert("version".into(), format!("1.0.{}", self.version).into());
        manifest.insert(
            "hill_api".into(),
            Value::Integer(i64::from(story::HILL_API)),
        );
        manifest.insert(
            "content_types".into(),
            Value::Array(vec!["adventure".into()]),
        );
        manifest.insert(
            "entry_points".into(),
            Value::Array(vec![ADVENTURE_FILE.into()]),
        );
        manifest.insert("default_locale".into(), "en".into());

        let text = |table: &Table| toml::to_string(table).unwrap_or_default().into_bytes();
        Files::from([
            ("manifest.toml".to_owned(), text(&manifest)),
            (ADVENTURE_FILE.to_owned(), text(&adventure)),
            (LINES_FILE.to_owned(), text(&lines)),
        ])
    }

    /// The draft loaded as any package is: what Hill would make of it.
    pub fn package(&self) -> Result<Package, story::PackageError> {
        story::load(&self.files(), &self.package_id)
    }

    /// The adventure itself, once it loads.
    pub fn adventure(&self) -> Result<Adventure, String> {
        let package = self.package().map_err(|problem| plain(&problem.problem))?;
        package
            .adventures
            .into_iter()
            .next()
            .ok_or_else(|| "There's no adventure in it.".to_owned())
    }
}

impl DraftPath {
    /// What a way only some company takes is called: blank for the plain word.
    pub fn called(&self) -> String {
        if self.name.trim().is_empty() {
            "a shortcut".to_owned()
        } else {
            self.name.trim().to_owned()
        }
    }
}

/// Words kept to one line, as a package's lines must be.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A loader's problem, for someone who never wrote the file it names.
fn plain(problem: &str) -> String {
    let mut words = problem.to_owned();
    if let Some(first) = words.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    if !words.ends_with('.') {
        words.push('.');
    }
    words
}

/// How far a point is from a stretch of line.
fn from_segment(at: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((at.0 - a.0) * dx + (at.1 - a.1) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    crate::playground::distance(at, (a.0 + dx * t, a.1 + dy * t))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A little adventure, made as someone would in the editor.
    fn made() -> Draft {
        let mut draft = Draft::new("abc", "Someone");
        draft.title = "Down to the Pond".into();
        let glade = draft.add_place(Site::Glade, (150.0, 150.0)).unwrap();
        let pool = draft.add_place(Site::Pool, (290.0, 170.0)).unwrap();
        draft.join(0, glade).unwrap();
        draft.join(glade, pool).unwrap();
        draft.places[pool].ending = true;
        draft.places[pool].ending_words = "{leader} found the pond!".into();
        draft.places[pool].hidden = true;
        draft
    }

    #[test]
    fn a_draft_becomes_an_ordinary_package_and_loads() {
        let draft = made();
        assert!(draft.problems().is_empty(), "{:?}", draft.problems());
        let package = draft.package().unwrap();
        assert_eq!(package.id, "made.hill.adventure-abc");
        assert_eq!(package.author, "Someone");
        let adventure = &package.adventures[0];
        assert_eq!(adventure.title, "Down to the Pond");
        assert_eq!(adventure.map.places.len(), 3);
        assert!(adventure.map.places[2].hidden && adventure.map.places[2].ending);
        // And it passes the same checks as anything shared.
        assert!(crate::sharing::check(&draft.files(), "test").is_ok());
    }

    #[test]
    fn what_is_wrong_is_said_plainly_until_it_is_right() {
        let mut draft = Draft::new("abc", "");
        let problems = draft.problems();
        assert!(problems.iter().any(|p| p.contains("name")));
        assert!(problems.iter().any(|p| p.contains("who made it")));
        assert!(problems.iter().any(|p| p.contains("where it ends")));
        draft.title = "Lost".into();
        draft.author = "Me".into();
        let pool = draft.add_place(Site::Pool, (290.0, 170.0)).unwrap();
        draft.places[pool].ending = true;
        assert!(
            draft.problems()[0].contains("Nobody can get to the pool"),
            "{:?}",
            draft.problems()
        );
        draft.join(0, pool).unwrap();
        assert!(draft.problems().is_empty(), "{:?}", draft.problems());
    }

    #[test]
    fn places_keep_their_room_and_the_way_in_stays() {
        let mut draft = made();
        assert!(
            draft.add_place(Site::Log, (152.0, 152.0)).is_err(),
            "too close"
        );
        assert!(
            draft.add_place(Site::Log, (150.0, 2.0)).is_ok(),
            "pulled onto the map"
        );
        assert!(!draft.move_place(1, (290.0, 168.0)), "onto another place");
        assert!(draft.move_place(1, (160.0, 120.0)));
        assert!(!draft.remove_place(0), "the way in stays");
        let paths = draft.paths.len();
        assert!(draft.remove_place(1));
        assert!(draft.paths.len() < paths, "its paths go with it");
        assert!(draft.join(0, 1).is_ok());
        assert!(draft.join(1, 0).is_err(), "joined already");
    }

    #[test]
    fn a_shortcut_only_some_take_needs_an_open_way_too() {
        let mut draft = Draft::new("abc", "Me");
        draft.title = "Short".into();
        let pool = draft.add_place(Site::Pool, (290.0, 170.0)).unwrap();
        draft.places[pool].ending = true;
        let path = draft.join(0, pool).unwrap();
        draft.paths[path].way = "explorer".into();
        assert!(draft.problems()[0].contains("Nobody can get to"));
        draft.join(0, pool).unwrap();
        assert!(draft.problems().is_empty(), "{:?}", draft.problems());
        let adventure = draft.adventure().unwrap();
        assert!(matches!(
            adventure.map.paths[0].way,
            Way::Shortcut(_, Opener::Explorer)
        ));
    }

    #[test]
    fn a_draft_is_kept_as_text_and_read_back() {
        let draft = made();
        assert_eq!(Draft::from_text(&draft.to_text()), Some(draft));
        assert_eq!(Draft::from_text("{\"title\": 3}"), None);
    }

    #[test]
    fn words_with_a_stray_brace_or_a_stranger_named_are_turned_away() {
        let mut draft = made();
        draft.places[2].ending_words = "{finder} found it".into();
        assert!(
            draft.problems()[0].contains("leader"),
            "{:?}",
            draft.problems()
        );
        draft.places[2].ending_words = "a { stray".into();
        assert!(!draft.problems().is_empty());
    }

    #[test]
    fn whatever_is_under_the_pointer_is_found() {
        let draft = made();
        assert_eq!(draft.place_at((150.0, 145.0)), Some(1));
        assert_eq!(draft.place_at((10.0, 10.0)), None);
        let middle = (
            (draft.places[1].at.0 + draft.places[2].at.0) / 2.0,
            (draft.places[1].at.1 + draft.places[2].at.1) / 2.0,
        );
        assert_eq!(draft.path_at(middle), Some(1));
    }

    /// The whole way round: made, kept in the Woods, shared as one file, looked over and let in
    /// at another Hill, played to its end there, and taken out again.
    #[test]
    fn an_adventure_made_here_is_kept_shared_let_in_elsewhere_and_finished() {
        let root = std::env::temp_dir().join(format!("hill-made-adventure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (here, there) = (root.join("here"), root.join("there"));
        let draft = made();
        let files = draft.files();
        crate::sharing::install(&here, &files).unwrap();
        let library = crate::story::Library::load(Some(&here), &[]);
        assert!(library.problems.is_empty(), "{:?}", library.problems);
        assert!(
            library
                .adventures()
                .any(|(package, adventure)| package.id == draft.package_id
                    && adventure.title == "Down to the Pond")
        );

        let shared = root.join("pond.formiga-hill-share");
        crate::sharing::export(&files, &shared).unwrap();
        let looked =
            crate::sharing::inspect(crate::sharing::read(&shared).unwrap(), "pond").unwrap();
        assert_eq!(looked.summary.content_types, ["adventure"]);
        assert_eq!(looked.package.adventures.len(), 1);
        crate::sharing::install(&there, &looked.files).unwrap();
        let elsewhere = crate::story::Library::load(Some(&there), &[]);
        let (_, adventure) = elsewhere
            .adventures()
            .find(|(package, _)| package.id == draft.package_id)
            .unwrap();

        let cast = crate::cast::Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let outset = crate::expedition::Outset {
            map: adventure.map.clone(),
            party: vec![cast.members[0].id],
            influence: crate::woods::Influence::default(),
            seed: 1,
        };
        let mut trip = crate::expedition::Expedition::new(&cast, outset, Default::default(), 0.0);
        let mut now = 0.0;
        for place in super::super::shortest_way(&adventure.map)
            .into_iter()
            .skip(1)
        {
            while trip.phase() != crate::expedition::Phase::Choosing {
                now += 1.0 / 30.0;
                trip.tick(&cast, Default::default(), now);
            }
            assert!(trip.go(place, now));
            while matches!(trip.phase(), crate::expedition::Phase::Walking { .. }) {
                now += 1.0 / 30.0;
                trip.tick(&cast, Default::default(), now);
            }
        }
        assert!(trip.finished(), "the pond was never reached");

        crate::sharing::remove(&there, &draft.package_id).unwrap();
        assert_eq!(
            crate::story::Library::load(Some(&there), &[])
                .adventures()
                .count(),
            crate::story::Library::load(None, &[]).adventures().count()
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
