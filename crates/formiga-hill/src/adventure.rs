//! Woods adventures: short outings someone has laid out on a map of their own, made from the
//! places and paths the Woods already has, and walked as an expedition is.
//!
//! An adventure is a package's content, in the same folder of plain TOML as a story (see
//! `docs/PACKAGES.md`): where it starts, its places, the paths between them, and where it ends.
//! Each place is one of the Woods' own kinds (`Site`), so whatever is done there is that place's
//! own activity, played as it always is: the finds, the fish and the bugs come by Hill's rules,
//! and an adventure can give nothing else. Its words, a line on arriving somewhere and one for
//! each ending, are the author's, kept in the package's localisation file and checked before
//! they are shown.
//!
//! An adventure has to be fair to whoever plays it: every place can be reached by anyone on
//! their own along the open paths, a way only some company takes is a shortcut and never the
//! only way, and at least one ending is near enough to reach in a day. One that is not is
//! refused, with the reason, and never stops Hill opening.

pub mod draft;

use crate::expedition::map::{Map, Opener, Path, Place, Site, Way};
use crate::story::lines::{Line, Lines};
use formiga_core::TemperamentKind;
use serde::Deserialize;
use std::borrow::Cow;

/// The most places, paths and bends an adventure may have: a short outing, not a world.
pub const MOST_PLACES: usize = 16;
pub const MOST_PATHS: usize = 32;
pub const MOST_BENDS: usize = 8;
/// The longest a path may take, of the day's light.
pub const LONGEST: f32 = 30.0;
/// How much of the day's light the map's pixels take: as far as the Woods' own paths.
pub const PIXELS_PER_LIGHT: f32 = 12.0;
/// How much shorter a way only some company takes is, when its length is left to the drawing.
pub const SHORTCUT: f32 = 0.7;
/// Where on the map places may stand: inside the parchment, clear of its edges, with room above
/// each for its picture.
pub const LEFT: f32 = 16.0;
pub const RIGHT: f32 = 368.0;
pub const TOP: f32 = 48.0;
pub const BOTTOM: f32 = 206.0;
/// How close two places may stand: their pictures need the room.
pub const CLOSEST: f32 = 22.0;
/// The one name an adventure's lines may use: whoever leads the party.
pub const LEADER: &str = "leader";

/// An adventure, checked and ready to walk.
#[derive(Clone, Debug, PartialEq)]
pub struct Adventure {
    pub id: String,
    pub title: String,
    /// A line or two about it, for choosing it.
    pub about: String,
    /// Its map, with its title on it.
    pub map: Map,
    /// What is told on arriving at each place, if anything.
    pub arrivals: Vec<Option<Line>>,
    /// What is told on reaching each ending.
    pub endings: Vec<Option<Line>>,
}

/// A line as told about whoever leads: their name in place of `{leader}`, and in the words
/// their temperament would use, if the author wrote a version for it.
pub fn told(line: &Line, leader: &str, kind: Option<TemperamentKind>) -> String {
    line.for_kind(kind).replace("{leader}", leader)
}

#[derive(Deserialize)]
struct RawFile {
    adventure: RawHeader,
    #[serde(default)]
    places: Vec<RawPlace>,
    #[serde(default)]
    paths: Vec<RawPath>,
}

#[derive(Deserialize)]
struct RawHeader {
    id: String,
    title: String,
    #[serde(default)]
    about: Option<String>,
    start: String,
}

#[derive(Deserialize)]
struct RawPlace {
    id: String,
    site: String,
    at: (f32, f32),
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arrive: Option<String>,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    ending: Option<String>,
}

#[derive(Deserialize)]
struct RawPath {
    between: (String, String),
    #[serde(default)]
    length: Option<f32>,
    #[serde(default = "open_way")]
    way: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    bends: Vec<(f32, f32)>,
}

fn open_way() -> String {
    "open".to_owned()
}

/// Whether a content file is an adventure, rather than a story: it says so at its top.
pub fn is_adventure(text: &str) -> bool {
    toml::from_str::<toml::Table>(text).is_ok_and(|table| table.contains_key("adventure"))
}

/// What a way only some company takes is called in an adventure's file.
pub fn opener_key(opener: Opener) -> &'static str {
    match opener {
        Opener::Explorer => "explorer",
        Opener::LittleOne => "little_one",
        Opener::ClosePair => "close_pair",
        Opener::Bold => "bold",
        Opener::Reader => "reader",
    }
}

pub fn opener_from_key(key: &str) -> Option<Opener> {
    Opener::ALL
        .into_iter()
        .find(|opener| opener_key(*opener) == key)
}

/// Who takes a way, in a few words, for whoever is making one.
pub fn opener_words(opener: Opener) -> &'static str {
    match opener {
        Opener::Explorer => "an explorer, or anyone very curious",
        Opener::LittleOne => "a little one",
        Opener::ClosePair => "two close friends together",
        Opener::Bold => "someone bold",
        Opener::Reader => "someone who reads old signs, once a signpost is read",
    }
}

/// How much of the day a path drawn through `points` takes, if its author leaves it to the map.
pub fn drawn_length(points: &[(f32, f32)], shortcut: bool) -> f32 {
    let pixels: f32 = points
        .windows(2)
        .map(|pair| crate::playground::distance(pair[0], pair[1]))
        .sum();
    let length = (pixels / PIXELS_PER_LIGHT * if shortcut { SHORTCUT } else { 1.0 }).round();
    length.clamp(1.0, LONGEST)
}

fn is_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

/// A line the adventure uses, which must be there, and name nobody but whoever leads.
fn line(lines: &Lines, key: &str, what: &str) -> Result<Line, String> {
    let found = lines
        .get(key)
        .ok_or_else(|| format!("{what}: there is no line \"{key}\" in the localisation file"))?;
    if let Some(name) = found.placeholders().into_iter().find(|name| name != LEADER) {
        return Err(format!(
            "{what}: \"{key}\" names {{{name}}}; an adventure's lines can only name {{{LEADER}}}"
        ));
    }
    Ok(found.clone())
}

/// A name, from a line: one short line of plain words, nobody named in it.
fn name(lines: &Lines, key: &str, what: &str) -> Result<String, String> {
    let found = line(lines, key, what)?;
    if !found.placeholders().is_empty()
        || !found.variants.is_empty()
        || found.text.chars().count() > 40
    {
        return Err(format!(
            "{what}: \"{key}\" is a name, so it must be short, plain and name nobody"
        ));
    }
    Ok(found.text)
}

/// What an adventure is about, from a line: shown when choosing it, before anyone leads, so
/// plain words that name nobody and are the same for every temperament.
fn summary(lines: &Lines, key: &str, what: &str) -> Result<String, String> {
    let found = line(lines, key, what)?;
    if !found.placeholders().is_empty() || !found.variants.is_empty() {
        return Err(format!(
            "{what}: \"{key}\" is shown before anyone leads, so it must be plain and name nobody"
        ));
    }
    Ok(found.text)
}

/// Reads an adventure from its file and the package's lines, and checks that it is fair to play.
pub fn parse(text: &str, lines: &Lines) -> Result<Adventure, String> {
    let raw: RawFile = toml::from_str(text).map_err(|error| error.to_string())?;
    let header = raw.adventure;
    if !is_id(&header.id) {
        return Err("adventure.id must be short: lowercase letters, digits, '-' and '_'".into());
    }
    let title = name(lines, &header.title, "adventure.title")?;
    let about = match &header.about {
        Some(key) => summary(lines, key, "adventure.about")?,
        None => String::new(),
    };
    if raw.places.len() < 2 || raw.places.len() > MOST_PLACES {
        return Err(format!(
            "an adventure has between 2 and {MOST_PLACES} places"
        ));
    }
    if raw.paths.is_empty() || raw.paths.len() > MOST_PATHS {
        return Err(format!("an adventure has between 1 and {MOST_PATHS} paths"));
    }

    let mut places: Vec<Place> = Vec::new();
    let mut arrivals = Vec::new();
    let mut endings = Vec::new();
    for raw in &raw.places {
        let what = format!("place \"{}\"", raw.id);
        if !is_id(&raw.id) {
            return Err(format!(
                "{what} needs an id of lowercase letters, digits, '-' and '_'"
            ));
        }
        if places.iter().any(|place| place.id == raw.id) {
            return Err(format!("two places are called \"{}\"", raw.id));
        }
        let site = Site::from_key(&raw.site).ok_or_else(|| {
            let sites: Vec<&str> = Site::ALL.iter().map(|site| site.key()).collect();
            format!(
                "{what}: \"{}\" is not a kind of place ({})",
                raw.site,
                sites.join(", ")
            )
        })?;
        let (x, y) = raw.at;
        if !(LEFT..=RIGHT).contains(&x) || !(TOP..=BOTTOM).contains(&y) {
            return Err(format!(
                "{what} is off the map: at must be between [{LEFT}, {TOP}] and [{RIGHT}, {BOTTOM}]"
            ));
        }
        if let Some(near) = places
            .iter()
            .find(|place| crate::playground::distance(place.at, raw.at) < CLOSEST)
        {
            return Err(format!(
                "{what} stands too close to \"{}\" for both to be drawn",
                near.id
            ));
        }
        let name = match &raw.name {
            Some(key) => name(lines, key, &what)?,
            None => site.name().to_owned(),
        };
        arrivals.push(
            raw.arrive
                .as_deref()
                .map(|key| line(lines, key, &what))
                .transpose()?,
        );
        endings.push(
            raw.ending
                .as_deref()
                .map(|key| line(lines, key, &what))
                .transpose()?,
        );
        places.push(Place {
            id: Cow::Owned(raw.id.clone()),
            name: Cow::Owned(name),
            site,
            stop: site.stop(),
            at: raw.at,
            depth: 0,
            hidden: raw.hidden,
            ending: raw.ending.is_some(),
        });
    }
    let index = |id: &str, what: &str| {
        places
            .iter()
            .position(|place| place.id == id)
            .ok_or_else(|| format!("{what}: there is no place \"{id}\""))
    };
    let start = index(&header.start, "adventure.start")?;
    if places[start].hidden {
        return Err("adventure.start can't be hidden: it is where everyone comes in".into());
    }
    if !places.iter().any(|place| place.ending) {
        return Err("an adventure needs somewhere it ends: give a place an ending".into());
    }

    let mut paths: Vec<Path> = Vec::new();
    for (number, raw) in raw.paths.iter().enumerate() {
        let what = format!(
            "path {} ({} to {})",
            number + 1,
            raw.between.0,
            raw.between.1
        );
        let a = index(&raw.between.0, &what)?;
        let b = index(&raw.between.1, &what)?;
        if a == b {
            return Err(format!("{what} goes nowhere"));
        }
        if raw.bends.len() > MOST_BENDS {
            return Err(format!("{what} has more than {MOST_BENDS} bends"));
        }
        for &(x, y) in &raw.bends {
            if !(8.0..=376.0).contains(&x) || !(20.0..=212.0).contains(&y) {
                return Err(format!("{what} bends off the map"));
            }
        }
        let opener = match raw.way.as_str() {
            "open" => None,
            key => Some(opener_from_key(key).ok_or_else(|| {
                let ways: Vec<&str> = Opener::ALL.iter().map(|o| opener_key(*o)).collect();
                format!(
                    "{what}: way \"{key}\" is not open, or one of {}",
                    ways.join(", ")
                )
            })?),
        };
        if opener == Some(Opener::Reader)
            && places[a].site != Site::Signpost
            && places[b].site != Site::Signpost
        {
            return Err(format!(
                "{what}: the old way is read off a signpost, so one end must be a signpost"
            ));
        }
        let way = match opener {
            None => Way::Open,
            Some(opener) => {
                let called = match &raw.name {
                    Some(key) => name(lines, key, &what)?,
                    None => "a shortcut".to_owned(),
                };
                Way::Shortcut(Cow::Owned(called), opener)
            }
        };
        if paths
            .iter()
            .any(|path| path.way.opener() == opener && path.from(a) == Some(b))
        {
            return Err(format!("{what} is there twice"));
        }
        let mut points = vec![places[a].at];
        points.extend_from_slice(&raw.bends);
        points.push(places[b].at);
        let length = match raw.length {
            Some(length) if (1.0..=LONGEST).contains(&length) => length,
            Some(_) => {
                return Err(format!("{what}: length must be between 1 and {LONGEST}"));
            }
            None => drawn_length(&points, opener.is_some()),
        };
        paths.push(Path {
            ends: (a, b),
            length,
            way,
            bends: Cow::Owned(raw.bends.clone()),
        });
    }

    let mut map = Map {
        title: Some(title.clone()),
        places,
        paths,
        start,
    };
    check_fair(&map)?;
    let steps = map.steps_in();
    for (place, steps) in map.places.iter_mut().zip(steps) {
        place.depth = steps;
    }
    Ok(Adventure {
        id: header.id,
        title,
        about,
        map,
        arrivals,
        endings,
    })
}

/// Whether a map is fair to anyone who plays it: everywhere reachable along the open paths, and
/// an ending near enough to reach in a day, at the slowest pace.
pub fn check_fair(map: &Map) -> Result<(), String> {
    let open = |path: &Path| path.way == Way::Open;
    let reached = map.reachable(open);
    if let Some(stranded) = (0..map.places.len()).find(|place| !reached.contains(place)) {
        return Err(format!(
            "nobody on their own can get to \"{}\": every place needs an open path to it",
            map.places[stranded].id
        ));
    }
    let nearest = map.nearest(open);
    let slowest = 1.0 / 0.8;
    let day = crate::expedition::LIGHT;
    let near_enough = (0..map.places.len())
        .filter(|place| map.places[*place].ending)
        .any(|place| nearest[place].is_some_and(|cost| cost * slowest <= day * 0.8));
    if !near_enough {
        return Err(
            "every ending is too far to reach in a day: shorten the paths to one of them".into(),
        );
    }
    Ok(())
}

/// The shortest way anyone could go, along the open paths, from the start to the nearest
/// ending: every place on it, the start first.
pub fn shortest_way(map: &Map) -> Vec<usize> {
    let open = |path: &Path| path.way == Way::Open;
    let nearest = map.nearest(open);
    let Some(mut here) = (0..map.places.len())
        .filter(|place| map.places[*place].ending)
        .filter(|place| nearest[*place].is_some())
        .min_by(|a, b| nearest[*a].unwrap().total_cmp(&nearest[*b].unwrap()))
    else {
        return vec![map.start];
    };
    let mut way = vec![here];
    while here != map.start {
        let cost = nearest[here].unwrap_or(0.0);
        let Some(back) = map.paths.iter().filter(|path| open(path)).find_map(|path| {
            let there = path.from(here)?;
            let through = nearest[there]? + path.length;
            ((through - cost).abs() < 0.01).then_some(there)
        }) else {
            break;
        };
        way.push(back);
        here = back;
    }
    way.reverse();
    way
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const LINES: &str = r#"
        title = "The Lantern Walk"
        about = "Follow the lantern light to the old pool."
        [lane]
        arrive = "The lane is quiet tonight."
        [pool]
        name = "the lantern pool"
        ending = { text = "{leader} sees the lanterns on the water.", grump = "Lanterns. On water. Fine." }
        [ferns]
        name = "the fern tunnel"
    "#;

    pub const FILE: &str = r#"
        [adventure]
        id = "lantern-walk"
        title = "title"
        about = "about"
        start = "lane"

        [[places]]
        id = "lane"
        site = "edge"
        at = [40, 186]
        arrive = "lane.arrive"

        [[places]]
        id = "glade"
        site = "glade"
        at = [150, 150]

        [[places]]
        id = "post"
        site = "signpost"
        at = [230, 110]

        [[places]]
        id = "pool"
        site = "pool"
        at = [300, 170]
        name = "pool.name"
        hidden = true
        ending = "pool.ending"

        [[paths]]
        between = ["lane", "glade"]
        bends = [[90, 170]]

        [[paths]]
        between = ["glade", "post"]

        [[paths]]
        between = ["post", "pool"]
        length = 9

        [[paths]]
        between = ["lane", "pool"]
        way = "explorer"
        name = "ferns.name"
    "#;

    fn lines() -> Lines {
        Lines::parse(LINES).unwrap()
    }

    fn problem(file: &str) -> String {
        parse(file, &lines()).unwrap_err()
    }

    #[test]
    fn an_adventure_reads_as_a_map_of_the_woods_own_places() {
        let adventure = parse(FILE, &lines()).unwrap();
        assert_eq!(adventure.title, "The Lantern Walk");
        let map = &adventure.map;
        assert_eq!(map.title.as_deref(), Some("The Lantern Walk"));
        assert_eq!(map.places.len(), 4);
        assert_eq!(map.places[3].name, "the lantern pool");
        assert_eq!(
            map.places[1].name, "the glade",
            "unnamed, a place is called by its kind"
        );
        assert!(map.places[3].hidden && map.places[3].ending);
        let endings: Vec<usize> = (0..map.places.len())
            .filter(|index| map.places[*index].ending)
            .collect();
        assert_eq!(endings, [3]);
        assert_eq!(map.paths[2].length, 9.0);
        assert!(
            map.paths[0].length >= 1.0,
            "left to the drawing, a length is measured"
        );
        assert_eq!(
            map.paths[3].way,
            Way::Shortcut("the fern tunnel".into(), Opener::Explorer)
        );
        assert_eq!(
            map.places[3].depth, 1,
            "the shortcut makes the pool one path in"
        );
        let ending = adventure.endings[3].as_ref().unwrap();
        assert_eq!(
            told(ending, "Pip", None),
            "Pip sees the lanterns on the water."
        );
        assert_eq!(
            told(ending, "Fig", Some(TemperamentKind::Grump)),
            "Lanterns. On water. Fine."
        );
    }

    #[test]
    fn a_place_only_a_shortcut_reaches_is_refused() {
        let file = FILE.replace(
            "[[paths]]\n        between = [\"post\", \"pool\"]\n        length = 9\n",
            "",
        );
        assert!(problem(&file).contains("nobody on their own can get to \"pool\""));
    }

    #[test]
    fn an_adventure_without_an_ending_or_a_way_in_is_refused() {
        let unending = FILE.replace("ending = \"pool.ending\"", "");
        assert!(problem(&unending).contains("somewhere it ends"));
        let nowhere = FILE.replace("start = \"lane\"", "start = \"nowhere\"");
        assert!(problem(&nowhere).contains("no place \"nowhere\""));
        let hidden = FILE.replace("start = \"lane\"", "start = \"pool\"");
        assert!(problem(&hidden).contains("can't be hidden"));
    }

    #[test]
    fn an_ending_too_far_for_a_day_is_refused() {
        // A long way in, one place after another, every path as long as a path can be.
        let chain = |count: usize| {
            let mut file =
                String::from("[adventure]\nid = \"far\"\ntitle = \"title\"\nstart = \"p0\"\n");
            for place in 0..count {
                let ending = if place + 1 == count {
                    "ending = \"pool.ending\"\n"
                } else {
                    ""
                };
                file.push_str(&format!(
                    "[[places]]\nid = \"p{place}\"\nsite = \"glade\"\nat = [{}, 120]\n{ending}",
                    20 + place * 40
                ));
            }
            for place in 1..count {
                file.push_str(&format!(
                    "[[paths]]\nbetween = [\"p{}\", \"p{place}\"]\nlength = 30\n",
                    place - 1
                ));
            }
            file
        };
        assert!(parse(&chain(4), &lines()).is_ok());
        assert!(problem(&chain(7)).contains("too far"));
    }

    #[test]
    fn what_an_adventure_says_is_checked_like_any_package_line() {
        let missing = FILE.replace("name = \"pool.name\"", "name = \"pool.called\"");
        assert!(problem(&missing).contains("no line \"pool.called\""));
        let naming = Lines::parse(&LINES.replace("{leader} sees", "{finder} sees")).unwrap();
        assert!(
            parse(FILE, &naming)
                .unwrap_err()
                .contains("only name {leader}")
        );
    }

    #[test]
    fn what_an_adventure_is_about_names_nobody_and_reads_the_same_for_everyone() {
        for about in [
            "about = \"{leader} follows the lanterns.\"",
            "about = { text = \"Follow the lanterns.\", grump = \"Lanterns, then.\" }",
        ] {
            let lines = Lines::parse(&LINES.replace(
                "about = \"Follow the lantern light to the old pool.\"",
                about,
            ))
            .unwrap();
            let problem = parse(FILE, &lines).unwrap_err();
            assert!(
                problem.contains("before anyone leads"),
                "{about}: {problem}"
            );
        }
    }

    #[test]
    fn places_and_paths_stay_on_the_map_and_within_bounds() {
        assert!(
            problem(&FILE.replace("at = [150, 150]", "at = [150, 10]")).contains("off the map")
        );
        assert!(problem(&FILE.replace("at = [150, 150]", "at = [45, 180]")).contains("too close"));
        assert!(
            problem(&FILE.replace("site = \"glade\"", "site = \"castle\""))
                .contains("not a kind of place")
        );
        assert!(
            problem(&FILE.replace("way = \"explorer\"", "way = \"teleport\"")).contains("not open")
        );
        assert!(
            problem(&FILE.replace("way = \"explorer\"", "way = \"reader\"")).contains("signpost")
        );
        assert!(problem(&FILE.replace("length = 9", "length = 99")).contains("length"));
    }

    /// Plays every one of Hill's own adventures to an ending with each of Desktop's sample
    /// colony on their own, stopping wherever there is something to do on the way, so none of
    /// them is out of reach of anyone, or out of a day's light.
    #[test]
    fn every_official_adventure_can_be_finished_by_anyone_on_their_own() {
        use crate::expedition::{Expedition, Known, Outset, Phase};
        let cast = crate::cast::Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let library = crate::story::Library::load(None, &[]);
        assert!(library.problems.is_empty(), "{:?}", library.problems);
        let adventures: Vec<&Adventure> = library.adventures().map(|(_, a)| a).collect();
        assert!(adventures.len() >= 3);
        for adventure in adventures {
            for member in &cast.members {
                let outset = Outset {
                    map: adventure.map.clone(),
                    party: vec![member.id],
                    influence: crate::woods::Influence::default(),
                    seed: 11,
                };
                let mut trip = Expedition::new(&cast, outset, Known::default(), 0.0);
                let way = shortest_way(&adventure.map);
                let mut next = 1;
                let mut now = 0.0;
                while !trip.finished() && now < 900.0 {
                    now += 1.0 / 30.0;
                    let input = trip
                        .leg_mut()
                        .map(|leg| leg.steady(now))
                        .unwrap_or_default();
                    trip.tick(&cast, input, now);
                    match trip.phase() {
                        Phase::Choosing if trip.can_stop() => {
                            trip.stop(&cast, now);
                        }
                        Phase::Choosing if next < way.len() => {
                            assert!(trip.go(way[next], now), "{} is stuck", adventure.title);
                            next += 1;
                        }
                        Phase::MakingRoom => {
                            trip.leave_behind(0, now);
                        }
                        _ => {}
                    }
                }
                assert!(
                    trip.finished(),
                    "{} on their own never finished {}",
                    member.name,
                    adventure.title
                );
            }
        }
    }
}
