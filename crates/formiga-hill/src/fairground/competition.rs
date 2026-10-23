//! Competitions: the Fairground's own games played one after another, for points, by whoever the
//! format says may take part. A competition is only an order of games, how many rounds of each,
//! who is eligible, the points for each place and how it is won: the games are played exactly as
//! they always are, by the colony as it is, and the person watches. Nothing in a competition
//! names a companion, or says how anyone plays, so the same format runs with any colony.
//!
//! A competition is content, written in a package as a story is (docs/PACKAGES.md), so it can be
//! made in the Fairground, shared as a file and taken in, inspected and removed like any other.
//!
//! ```toml
//! [competition]
//! id = "summer-games"
//! title = "title"                  # a line in the localisation file
//! points = [3, 2, 1]               # for 1st, 2nd and 3rd; nobody else scores
//! win = "most-points"              # or "first-to:10"
//!
//! [[events]]
//! game = "sack-race"
//! rounds = 2
//! players = "everyone"             # or "keenest:4", or "leaders:3"
//! ```

use super::{Game, Games};
use crate::cast::{Cast, Id};
use crate::character::Character;
use crate::playground::Playground;
use crate::story::lines::Lines;
use serde::Deserialize;

/// The most events a competition holds.
pub const MOST_EVENTS: usize = 8;
/// The most rounds of one event.
pub const MOST_ROUNDS: u8 = 3;
/// The most places that score.
pub const MOST_PLACES: usize = 6;
/// The most points one place is worth.
pub const MOST_POINTS: u32 = 10;
/// The highest score a "first to" competition can be played to.
pub const MOST_TARGET: u32 = 99;
/// The most entrants a "keenest" or "leaders" event can name.
pub const MOST_ENTRANTS: u8 = 12;
/// The longest title a competition can have.
pub const MOST_TITLE_CHARS: usize = 40;
/// How long between one game and the next, for the person to see how it stands.
const BETWEEN_SECS: f32 = 4.0;
/// How long before the first game, for the person to see what is coming.
const OPENING_SECS: f32 = 2.0;

/// Who may play an event, said without naming anyone, so it means something in any colony.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entrants {
    /// Everyone who came.
    Everyone,
    /// The keenest few for this game, by their own temperaments.
    Keenest(u8),
    /// The few with the most points so far: a final. Before anyone has scored, the keenest.
    Leaders(u8),
}

impl Entrants {
    fn parse(text: &str) -> Result<Self, String> {
        let count = |rest: &str| -> Result<u8, String> {
            rest.parse::<u8>()
                .ok()
                .filter(|count| (1..=MOST_ENTRANTS).contains(count))
                .ok_or_else(|| format!("\"{text}\": the count must be from 1 to {MOST_ENTRANTS}"))
        };
        match text.split_once(':') {
            None if text == "everyone" => Ok(Self::Everyone),
            Some(("keenest", rest)) => Ok(Self::Keenest(count(rest)?)),
            Some(("leaders", rest)) => Ok(Self::Leaders(count(rest)?)),
            _ => Err(format!(
                "players \"{text}\" must be \"everyone\", \"keenest:N\" or \"leaders:N\""
            )),
        }
    }

    pub fn key(self) -> String {
        match self {
            Self::Everyone => "everyone".to_owned(),
            Self::Keenest(count) => format!("keenest:{count}"),
            Self::Leaders(count) => format!("leaders:{count}"),
        }
    }

    /// As the person reads it: "everyone", "the 4 keenest", "the top 3 so far".
    pub fn said(self) -> String {
        match self {
            Self::Everyone => "everyone".to_owned(),
            Self::Keenest(1) => "the keenest".to_owned(),
            Self::Keenest(count) => format!("the {count} keenest"),
            Self::Leaders(1) => "whoever leads so far".to_owned(),
            Self::Leaders(count) => format!("the top {count} so far"),
        }
    }
}

/// How a competition is won.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Win {
    /// The most points once every event is played.
    MostPoints,
    /// The first to reach this many points; if nobody does, the most points at the end.
    FirstTo(u32),
}

impl Win {
    fn parse(text: &str) -> Result<Self, String> {
        match text.split_once(':') {
            None if text == "most-points" => Ok(Self::MostPoints),
            Some(("first-to", rest)) => rest
                .parse::<u32>()
                .ok()
                .filter(|target| (1..=MOST_TARGET).contains(target))
                .map(Self::FirstTo)
                .ok_or_else(|| format!("\"{text}\": the points must be from 1 to {MOST_TARGET}")),
            _ => Err(format!(
                "win \"{text}\" must be \"most-points\" or \"first-to:N\""
            )),
        }
    }

    pub fn key(self) -> String {
        match self {
            Self::MostPoints => "most-points".to_owned(),
            Self::FirstTo(target) => format!("first-to:{target}"),
        }
    }

    /// As the person reads it.
    pub fn said(self) -> String {
        match self {
            Self::MostPoints => "the most points after every event wins".to_owned(),
            Self::FirstTo(target) => format!("the first to {target} points wins"),
        }
    }
}

/// One of a competition's games, played for some rounds by some of the colony.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub game: Game,
    pub rounds: u8,
    pub players: Entrants,
}

/// A competition as written: its games in order, and how it is scored and won.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Format {
    pub id: String,
    /// The title, in the author's own words.
    pub title: String,
    pub events: Vec<Event>,
    /// The points for each place, first place first.
    pub points: Vec<u32>,
    pub win: Win,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    competition: RawHeader,
    #[serde(default)]
    events: Vec<RawEvent>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHeader {
    id: String,
    title: String,
    points: Vec<u32>,
    win: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvent {
    game: String,
    #[serde(default = "one")]
    rounds: u8,
    #[serde(default = "everyone")]
    players: String,
}

fn one() -> u8 {
    1
}

fn everyone() -> String {
    "everyone".to_owned()
}

/// Whether a content file is a competition, rather than a story: it has a `[competition]` table.
pub fn is_competition(text: &str) -> bool {
    toml::from_str::<toml::Table>(text).is_ok_and(|table| table.contains_key("competition"))
}

/// Reads a competition from its content file and its package's words, checking everything in it.
/// Unknown fields are refused: a competition is small, and a field Hill does not know is more
/// likely a mistake than something from a later Hill.
pub fn parse(text: &str, lines: &Lines) -> Result<Format, String> {
    let raw: RawFile = toml::from_str(text).map_err(|error| error.to_string())?;
    let header = raw.competition;
    if !is_id(&header.id) {
        return Err("competition.id must be short, lowercase, letters, digits and '-'".into());
    }
    let title = lines.get(&header.title).ok_or_else(|| {
        format!(
            "competition.title \"{}\" is not in the localisation file",
            header.title
        )
    })?;
    let title = title.text.trim().to_owned();
    if !is_title(&title) {
        return Err(format!(
            "competition.title must be one line of up to {MOST_TITLE_CHARS} characters, naming nobody"
        ));
    }
    if header.points.is_empty() || header.points.len() > MOST_PLACES {
        return Err(format!(
            "competition.points must give from 1 to {MOST_PLACES} places their points"
        ));
    }
    if header.points.iter().any(|points| *points > MOST_POINTS) {
        return Err(format!(
            "competition.points: a place is worth at most {MOST_POINTS}"
        ));
    }
    if header.points.iter().all(|points| *points == 0) {
        return Err("competition.points: some place must be worth a point".into());
    }
    let win = Win::parse(&header.win).map_err(|problem| format!("competition.win: {problem}"))?;
    if raw.events.is_empty() || raw.events.len() > MOST_EVENTS {
        return Err(format!(
            "a competition has from 1 to {MOST_EVENTS} [[events]]"
        ));
    }
    let events = raw
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let at = |problem: String| format!("events[{}]: {problem}", index + 1);
            let game = Game::from_key(&event.game).ok_or_else(|| {
                at(format!(
                    "game \"{}\" is not one of the Fairground's: {}",
                    event.game,
                    Game::ALL.map(Game::key).join(", ")
                ))
            })?;
            if !(1..=MOST_ROUNDS).contains(&event.rounds) {
                return Err(at(format!("rounds must be from 1 to {MOST_ROUNDS}")));
            }
            let players = Entrants::parse(&event.players).map_err(at)?;
            Ok(Event {
                game,
                rounds: event.rounds,
                players,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Format {
        id: header.id,
        title,
        events,
        points: header.points,
        win,
    })
}

fn is_id(text: &str) -> bool {
    (1..=48).contains(&text.len())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// One plain line, short, and naming nobody: no `{role}` in it, since nobody plays a role.
pub fn is_title(text: &str) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= MOST_TITLE_CHARS
        && !text.chars().any(|c| c.is_control() || c == '{' || c == '}')
}

impl Format {
    /// A new competition to start from: one of each game, everyone playing, 3, 2 and 1 points.
    pub fn starter() -> Self {
        Self {
            id: "my-competition".to_owned(),
            title: "My competition".to_owned(),
            events: vec![
                Event {
                    game: Game::SackRace,
                    rounds: 1,
                    players: Entrants::Everyone,
                },
                Event {
                    game: Game::Hoopla,
                    rounds: 1,
                    players: Entrants::Everyone,
                },
            ],
            points: vec![3, 2, 1],
            win: Win::MostPoints,
        }
    }

    /// How many games will be played, if nobody wins early.
    pub fn games(&self) -> usize {
        self.events.iter().map(|event| event.rounds as usize).sum()
    }

    /// The content file this competition is written as, with its title as the line `title`.
    pub fn content(&self) -> String {
        let points: Vec<String> = self.points.iter().map(u32::to_string).collect();
        let mut text = format!(
            "[competition]\nid = \"{}\"\ntitle = \"title\"\npoints = [{}]\nwin = \"{}\"\n",
            self.id,
            points.join(", "),
            self.win.key()
        );
        for event in &self.events {
            text.push_str(&format!(
                "\n[[events]]\ngame = \"{}\"\nrounds = {}\nplayers = \"{}\"\n",
                event.game.key(),
                event.rounds,
                event.players.key()
            ));
        }
        text
    }

    /// The localisation file for the content file: just its title.
    pub fn words(&self) -> String {
        let title: toml::Value = self.title.trim().into();
        format!("title = {title}\n")
    }

    /// Each problem that would stop the competition loading, as the person would put it right,
    /// for the editor to say before anything is saved.
    pub fn problems(&self) -> Vec<String> {
        let lines = match Lines::parse(&self.words()) {
            Ok(lines) => lines,
            Err(problem) => return vec![problem],
        };
        match parse(&self.content(), &lines) {
            Ok(_) => Vec::new(),
            Err(problem) => vec![problem],
        }
    }

    /// The whole competition in a few lines, as the person would tell it: for the editor's
    /// preview, and for looking over a competition from a file before taking it in.
    pub fn told(&self) -> Vec<String> {
        let mut told: Vec<String> = self
            .events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let rounds = match event.rounds {
                    1 => String::new(),
                    rounds => format!(", {rounds} rounds"),
                };
                format!(
                    "{}. {}{rounds}, for {}.",
                    index + 1,
                    event.game.name(),
                    event.players.said()
                )
            })
            .collect();
        told.push(format!(
            "Points: {}; {}.",
            places_said(&self.points),
            self.win.said()
        ));
        told
    }
}

/// "3 for 1st, 2 for 2nd, 1 for 3rd".
fn places_said(points: &[u32]) -> String {
    points
        .iter()
        .enumerate()
        .map(|(index, points)| format!("{points} for {}", ordinal(index + 1)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// How keen a companion is on a game, from its own temperament: who "the keenest" are.
pub fn keenness(game: Game, character: &Character) -> f32 {
    let a = character.axes;
    match game {
        Game::HideAndSeek => a.playfulness + a.boldness,
        Game::SackRace => a.energy + a.playfulness,
        Game::HighStriker => super::high_striker::keenness(character),
        Game::Hoopla => super::hoopla::keenness(character),
        Game::TugOfWar => a.feistiness + a.social,
    }
}

/// The fewest who can play `game`.
pub fn fewest(game: Game) -> usize {
    match game.players() {
        super::Players::Some { least } => least,
        super::Players::It | super::Players::Sides => 2,
    }
}

/// Something in a competition for the person to be told.
#[derive(Clone, Debug, PartialEq)]
pub enum Happening {
    /// The next game is about to start, among these.
    Next {
        game: Game,
        round: u8,
        rounds: u8,
        players: Vec<Id>,
    },
    /// Too few could play a game, so it was passed over.
    Skipped { game: Game },
    /// A game is over, and these scored, in places: each place's companions and their points.
    Scored {
        game: Game,
        places: Vec<(Vec<Id>, u32)>,
    },
    /// It is over: these won, on this many points; nobody, if nobody scored.
    Won { winners: Vec<Id>, points: u32 },
    /// The person stopped it.
    CalledOff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    /// Waiting, before the next game starts.
    Between {
        until: f32,
    },
    /// A game is on.
    Playing,
    Over,
}

/// A competition being run at the Fairground.
pub struct Contest {
    format: Format,
    /// Which event is next or on, and which round of it, from 1.
    event: usize,
    round: u8,
    stage: Stage,
    /// Everyone who has played, with their points, in the order they first played.
    scores: Vec<(Id, u32)>,
    happenings: Vec<Happening>,
}

impl Contest {
    pub fn new(format: Format, now: f32) -> Self {
        Self {
            format,
            event: 0,
            round: 1,
            stage: Stage::Between {
                until: now + OPENING_SECS,
            },
            scores: Vec::new(),
            happenings: Vec::new(),
        }
    }

    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn is_over(&self) -> bool {
        self.stage == Stage::Over
    }

    /// The event under way or next, and the round of it: (event from 1, round from 1).
    pub fn at(&self) -> (usize, u8) {
        (self.event + 1, self.round)
    }

    /// The event under way or next, if any is left.
    pub fn event(&self) -> Option<Event> {
        self.format.events.get(self.event).copied()
    }

    /// Everyone who has played so far, most points first; ties in the order they first played.
    pub fn standings(&self) -> Vec<(Id, u32)> {
        let mut standings = self.scores.clone();
        standings.sort_by_key(|(_, points)| std::cmp::Reverse(*points));
        standings
    }

    /// What has happened since last asked.
    pub fn take_happenings(&mut self) -> Vec<Happening> {
        std::mem::take(&mut self.happenings)
    }

    /// Stops the competition where it is, with nobody winning.
    pub fn call_off(&mut self) {
        if self.stage != Stage::Over {
            self.stage = Stage::Over;
            self.happenings.push(Happening::CalledOff);
        }
    }

    /// Who may play `event` now, from those at the Fairground.
    fn entrants(&self, event: Event, ground: &Playground) -> Vec<Id> {
        let mut here = ground.ids();
        let keen = |id: Id| {
            ground
                .character(id)
                .map_or(0.0, |character| keenness(event.game, character))
        };
        let points = |id: Id| {
            self.scores
                .iter()
                .find(|(scored, _)| *scored == id)
                .map_or(0, |(_, points)| *points)
        };
        let count = match event.players {
            Entrants::Everyone => return here,
            Entrants::Keenest(count) => {
                here.sort_by(|a, b| keen(*b).total_cmp(&keen(*a)).then(a.cmp(b)));
                count
            }
            Entrants::Leaders(count) => {
                here.sort_by(|a, b| {
                    points(*b)
                        .cmp(&points(*a))
                        .then(keen(*b).total_cmp(&keen(*a)))
                        .then(a.cmp(b))
                });
                count
            }
        };
        here.truncate(count as usize);
        here
    }

    /// Runs the competition on: starts each game when it is time, and scores it when it is over.
    pub fn tick(&mut self, games: &mut Games, ground: &mut Playground, cast: &Cast, now: f32) {
        match self.stage {
            Stage::Over => {}
            Stage::Between { until } if now >= until => {
                let Some(event) = self.event() else {
                    self.finish();
                    return;
                };
                if games.playing().is_some() {
                    return;
                }
                let players = self.entrants(event, ground);
                if players.len() < fewest(event.game) {
                    self.happenings
                        .push(Happening::Skipped { game: event.game });
                    self.next_event(now);
                    return;
                }
                games.start_among(event.game, ground, cast, &players, now);
                if games.playing() != Some(event.game) {
                    self.happenings
                        .push(Happening::Skipped { game: event.game });
                    self.next_event(now);
                    return;
                }
                for id in &players {
                    if !self.scores.iter().any(|(scored, _)| scored == id) {
                        self.scores.push((*id, 0));
                    }
                }
                self.happenings.push(Happening::Next {
                    game: event.game,
                    round: self.round,
                    rounds: event.rounds,
                    players,
                });
                self.stage = Stage::Playing;
            }
            Stage::Between { .. } => {}
            Stage::Playing if games.playing().is_none() => {
                let Some(event) = self.event() else {
                    self.finish();
                    return;
                };
                let places = scored(&places(games, event.game), &self.format.points);
                for (ids, points) in &places {
                    for id in ids {
                        match self.scores.iter_mut().find(|(scored, _)| scored == id) {
                            Some((_, total)) => *total += points,
                            None => self.scores.push((*id, *points)),
                        }
                    }
                }
                self.happenings.push(Happening::Scored {
                    game: event.game,
                    places,
                });
                let reached = match self.format.win {
                    Win::FirstTo(target) => self.scores.iter().any(|(_, points)| *points >= target),
                    Win::MostPoints => false,
                };
                if reached {
                    self.finish();
                } else if self.round < event.rounds {
                    self.round += 1;
                    self.stage = Stage::Between {
                        until: now + BETWEEN_SECS,
                    };
                } else {
                    self.next_event(now);
                }
            }
            Stage::Playing => {}
        }
    }

    fn next_event(&mut self, now: f32) {
        self.event += 1;
        self.round = 1;
        if self.event >= self.format.events.len() {
            self.finish();
        } else {
            self.stage = Stage::Between {
                until: now + BETWEEN_SECS,
            };
        }
    }

    fn finish(&mut self) {
        if self.stage == Stage::Over {
            return;
        }
        self.stage = Stage::Over;
        let top = self
            .scores
            .iter()
            .map(|(_, points)| *points)
            .max()
            .unwrap_or(0);
        let winners = if top == 0 {
            Vec::new()
        } else {
            self.standings()
                .into_iter()
                .filter(|(_, points)| *points == top)
                .map(|(id, _)| id)
                .collect()
        };
        self.happenings.push(Happening::Won {
            winners,
            points: top,
        });
    }
}

/// How `game` finished, in places, best first: each place's companions, more than one where
/// they tied. Only those who did something worth a place are placed.
pub fn places(games: &Games, game: Game) -> Vec<Vec<Id>> {
    match game {
        // The best hidden is the last found.
        Game::HideAndSeek => games
            .hide_and_seek
            .found_order()
            .iter()
            .rev()
            .map(|id| vec![*id])
            .collect(),
        Game::SackRace => games
            .sack_race
            .results()
            .iter()
            .map(|(id, _)| vec![*id])
            .collect(),
        // The same mark on the striker is the same place.
        Game::HighStriker => grouped(
            games
                .high_striker
                .ranking()
                .into_iter()
                .map(|(id, height)| (id, super::high_striker::mark(height))),
        ),
        // A ring or more to be placed at all.
        Game::Hoopla => grouped(
            games
                .hoopla
                .tally()
                .into_iter()
                .filter(|(_, rings)| *rings > 0),
        ),
        Game::TugOfWar => {
            let (left, right) = games.tug_of_war.sides();
            match games.tug_of_war.winners() {
                Some(super::tug_of_war::Side::Left) => vec![left, right],
                Some(super::tug_of_war::Side::Right) => vec![right, left],
                None => Vec::new(),
            }
        }
    }
}

/// Places from a ranking already in order, best first, with those level together.
fn grouped<T: PartialEq>(ranking: impl Iterator<Item = (Id, T)>) -> Vec<Vec<Id>> {
    let mut places: Vec<(T, Vec<Id>)> = Vec::new();
    for (id, score) in ranking {
        match places.last_mut() {
            Some((last, ids)) if *last == score => ids.push(id),
            _ => places.push((score, vec![id])),
        }
    }
    places.into_iter().map(|(_, ids)| ids).collect()
}

/// Each place with its points: those tied share a place, and the next place after them is as
/// far down as there were of them, as in any sports day. Places worth nothing are left out.
pub fn scored(places: &[Vec<Id>], points: &[u32]) -> Vec<(Vec<Id>, u32)> {
    let mut above = 0;
    let mut scored = Vec::new();
    for ids in places {
        let worth = points.get(above).copied().unwrap_or(0);
        if worth > 0 && !ids.is_empty() {
            scored.push((ids.clone(), worth));
        }
        above += ids.len();
    }
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fairground::{open, testing::colony_of};

    fn lines(title: &str) -> Lines {
        Lines::parse(&format!("title = {}", toml::Value::from(title))).unwrap()
    }

    const WRITTEN: &str = r#"
        [competition]
        id = "summer-games"
        title = "title"
        points = [3, 2, 1]
        win = "first-to:8"

        [[events]]
        game = "sack-race"
        rounds = 2

        [[events]]
        game = "hoopla"
        players = "keenest:3"

        [[events]]
        game = "tug-of-war"
        players = "leaders:4"
    "#;

    #[test]
    fn a_written_competition_reads_as_its_author_meant() {
        let format = parse(WRITTEN, &lines("The Summer Games")).unwrap();
        assert_eq!(format.title, "The Summer Games");
        assert_eq!(format.win, Win::FirstTo(8));
        assert_eq!(format.games(), 4);
        assert_eq!(
            format.events[1],
            Event {
                game: Game::Hoopla,
                rounds: 1,
                players: Entrants::Keenest(3)
            }
        );
        assert_eq!(format.events[2].players, Entrants::Leaders(4));
        // Written back out, it reads the same.
        let again = parse(&format.content(), &Lines::parse(&format.words()).unwrap()).unwrap();
        assert_eq!(again, format);
        assert!(format.problems().is_empty());
    }

    #[test]
    fn anything_out_of_bounds_or_unknown_is_refused_with_the_reason() {
        let refused = |from: &str, to: &str| {
            let text = WRITTEN.replace(from, to);
            parse(&text, &lines("Games")).unwrap_err()
        };
        assert!(refused("sack-race", "dodgems").contains("not one of the Fairground's"));
        assert!(refused("rounds = 2", "rounds = 9").contains("rounds"));
        assert!(refused("keenest:3", "keenest:0").contains("count"));
        assert!(refused("keenest:3", "named:Fig").contains("players"));
        assert!(refused("[3, 2, 1]", "[3, 2, 1, 1, 1, 1, 1]").contains("places"));
        assert!(refused("[3, 2, 1]", "[50]").contains("at most"));
        assert!(refused("[3, 2, 1]", "[0, 0]").contains("worth a point"));
        assert!(refused("first-to:8", "first-to:1000").contains("points"));
        assert!(refused("first-to:8", "fastest").contains("win"));
        // A field Hill does not know, such as one trying to change who someone is, is refused.
        assert!(refused("rounds = 2", "rounds = 2\ntemperament = \"grump\"").contains("unknown"));
        assert!(refused("win = ", "strength = 9\nwin = ").contains("unknown"));
        let too_many = format!("{WRITTEN}{}", "\n[[events]]\ngame = \"hoopla\"\n".repeat(6));
        assert!(
            parse(&too_many, &lines("Games"))
                .unwrap_err()
                .contains("events")
        );
        assert!(parse(WRITTEN, &lines("Games for {winner}")).is_err());
        assert!(parse(WRITTEN, &lines(&"Long ".repeat(20))).is_err());
        assert!(parse(WRITTEN, &Lines::default()).is_err(), "no title line");
    }

    #[test]
    fn ties_share_a_place_and_push_the_next_one_down() {
        let places = vec![vec![1, 2], vec![3], vec![4]];
        assert_eq!(
            scored(&places, &[3, 2, 1]),
            vec![(vec![1, 2], 3), (vec![3], 1)]
        );
        assert_eq!(scored(&places, &[5]), vec![(vec![1, 2], 5)]);
        assert_eq!(
            grouped([(1, 7), (2, 7), (3, 4)].into_iter()),
            vec![vec![1, 2], vec![3]]
        );
    }

    #[test]
    fn the_preview_tells_the_whole_competition_plainly() {
        let format = parse(WRITTEN, &lines("The Summer Games")).unwrap();
        assert_eq!(
            format.told(),
            [
                "1. The sack race, 2 rounds, for everyone.",
                "2. Hoopla, for the 3 keenest.",
                "3. The tug-of-war, for the top 4 so far.",
                "Points: 3 for 1st, 2 for 2nd, 1 for 3rd; the first to 8 points wins.",
            ]
        );
    }

    /// Plays `contest` through at the Fairground with `cast`, as the window would.
    fn run(contest: &mut Contest, cast: &Cast) -> Vec<Happening> {
        let (mut ground, mut games) = open(cast, 0.0);
        let mut now = 0.0;
        let mut happened = Vec::new();
        while !contest.is_over() && now < 3000.0 {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            games.tick(&mut ground, now);
            contest.tick(&mut games, &mut ground, cast, now);
            happened.extend(contest.take_happenings());
        }
        assert!(contest.is_over(), "the competition never ended");
        happened
    }

    #[test]
    fn a_competition_plays_every_game_in_turn_and_crowns_the_most_points() {
        let cast = colony_of(6);
        let format = Format {
            id: "all".into(),
            title: "All of them".into(),
            events: Game::ALL
                .into_iter()
                .map(|game| Event {
                    game,
                    rounds: 1,
                    players: Entrants::Everyone,
                })
                .collect(),
            points: vec![3, 2, 1],
            win: Win::MostPoints,
        };
        let mut contest = Contest::new(format, 0.0);
        let happened = run(&mut contest, &cast);
        let played: Vec<Game> = happened
            .iter()
            .filter_map(|happening| match happening {
                Happening::Scored { game, .. } => Some(*game),
                _ => None,
            })
            .collect();
        assert_eq!(played, Game::ALL);
        // The points add up to what each scored, and the winners have the most.
        let mut totals: std::collections::HashMap<Id, u32> = Default::default();
        for happening in &happened {
            if let Happening::Scored { places, .. } = happening {
                for (ids, points) in places {
                    for id in ids {
                        *totals.entry(*id).or_default() += points;
                    }
                }
            }
        }
        let Some(Happening::Won { winners, points }) = happened.last() else {
            panic!("no winner: {happened:?}");
        };
        assert!(!winners.is_empty());
        assert_eq!(*points, totals.values().copied().max().unwrap());
        for winner in winners {
            assert_eq!(totals[winner], *points);
        }
        for (id, points) in contest.standings() {
            assert_eq!(totals.get(&id).copied().unwrap_or(0), points);
        }
    }

    #[test]
    fn only_the_eligible_play_and_a_final_is_for_the_leaders() {
        let cast = colony_of(6);
        let format = Format {
            id: "final".into(),
            title: "Heats and a final".into(),
            events: vec![
                Event {
                    game: Game::HighStriker,
                    rounds: 1,
                    players: Entrants::Keenest(4),
                },
                Event {
                    game: Game::SackRace,
                    rounds: 1,
                    players: Entrants::Leaders(2),
                },
            ],
            points: vec![3, 2, 1],
            win: Win::MostPoints,
        };
        let mut contest = Contest::new(format, 0.0);
        let happened = run(&mut contest, &cast);
        let mut nexts = happened.iter().filter_map(|happening| match happening {
            Happening::Next { players, .. } => Some(players.clone()),
            _ => None,
        });
        let heat = nexts.next().unwrap();
        assert_eq!(heat.len(), 4, "the 4 keenest have a go at the striker");
        let final_players = nexts.next().unwrap();
        assert_eq!(final_players.len(), 2);
        let scores_after_heat: Vec<(Id, u32)> = happened
            .iter()
            .find_map(|happening| match happening {
                Happening::Scored { places, .. } => Some(
                    places
                        .iter()
                        .flat_map(|(ids, points)| ids.iter().map(move |id| (*id, *points)))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap();
        let best = scores_after_heat.iter().map(|(_, p)| *p).max().unwrap();
        assert!(
            final_players
                .iter()
                .any(|id| scores_after_heat.contains(&(*id, best))),
            "the final is for whoever leads"
        );
    }

    #[test]
    fn first_to_ends_as_soon_as_someone_gets_there() {
        let cast = colony_of(4);
        let format = Format {
            id: "quick".into(),
            title: "Quick".into(),
            events: vec![Event {
                game: Game::SackRace,
                rounds: 3,
                players: Entrants::Everyone,
            }],
            points: vec![5],
            win: Win::FirstTo(5),
        };
        let mut contest = Contest::new(format, 0.0);
        let happened = run(&mut contest, &cast);
        let games = happened
            .iter()
            .filter(|happening| matches!(happening, Happening::Scored { .. }))
            .count();
        assert_eq!(games, 1, "the first race's winner reached 5");
        assert!(
            matches!(happened.last(), Some(Happening::Won { points: 5, winners }) if winners.len() == 1)
        );
    }

    #[test]
    fn a_game_too_few_can_play_is_passed_over_and_calling_off_crowns_nobody() {
        let cast = colony_of(1);
        let format = Format {
            id: "alone".into(),
            title: "Alone".into(),
            events: vec![
                Event {
                    game: Game::TugOfWar,
                    rounds: 1,
                    players: Entrants::Everyone,
                },
                Event {
                    game: Game::HighStriker,
                    rounds: 1,
                    players: Entrants::Everyone,
                },
            ],
            points: vec![1],
            win: Win::MostPoints,
        };
        let mut contest = Contest::new(format.clone(), 0.0);
        let happened = run(&mut contest, &cast);
        assert_eq!(
            happened.first(),
            Some(&Happening::Skipped {
                game: Game::TugOfWar
            })
        );
        assert!(
            matches!(happened.last(), Some(Happening::Won { winners, .. }) if winners.len() == 1)
        );

        let mut contest = Contest::new(format, 0.0);
        contest.call_off();
        contest.call_off();
        assert_eq!(contest.take_happenings(), vec![Happening::CalledOff]);
        assert!(contest.is_over());
    }
}
