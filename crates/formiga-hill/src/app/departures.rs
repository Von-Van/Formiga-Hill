//! The station's two boards. The notice board by the door has whatever is new at the Hill for
//! this colony; the departures board under the canopy has every place to go, what is on there,
//! and who would like to go. Both are read from the snapshot and Hill's own memories, and neither
//! asks anything of anyone: a notice is news or an invitation, never a chore or a reminder of
//! time away.

use super::games::{reached, rings_rung};
use super::{Area, Card, HillApp, Visit, clock, listed, ordinal, paper};
use crate::character::Character;
use crate::hilltop::Standing;
use crate::station::Fixture;
use eframe::egui;
use formiga_core::TemperamentKind;

/// Something a companion would like to do at the Hill.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Outing {
    Picnic,
    Stories,
    HideAndSeek,
    SackRace,
    HighStriker,
    Rummaging,
    Fishing,
    BugCatching,
    Foraging,
    Hilltop,
}

impl Outing {
    const ALL: [Self; 10] = [
        Self::Picnic,
        Self::Stories,
        Self::HideAndSeek,
        Self::SackRace,
        Self::HighStriker,
        Self::Rummaging,
        Self::Fishing,
        Self::BugCatching,
        Self::Foraging,
        Self::Hilltop,
    ];

    /// Where it happens.
    fn area(self) -> Area {
        match self {
            Self::Picnic => Area::Green,
            Self::Stories => Area::Clubhouse,
            Self::HideAndSeek | Self::SackRace | Self::HighStriker => Area::Fairground,
            Self::Rummaging | Self::Fishing | Self::BugCatching | Self::Foraging => Area::Woods,
            Self::Hilltop => Area::Hilltop,
        }
    }

    /// As a notice puts it, after "would like to".
    fn wish(self) -> &'static str {
        match self {
            Self::Picnic => "lie about on the picnic blanket",
            Self::Stories => "hear a story by the Clubhouse fire",
            Self::HideAndSeek => "play hide-and-seek at the Fairground",
            Self::SackRace => "run in the sack race at the Fairground",
            Self::HighStriker => "have a go on the high striker",
            Self::Rummaging => "go rummaging in the Woods",
            Self::Fishing => "go fishing at the pool",
            Self::BugCatching => "go bug catching in the meadow",
            Self::Foraging => "go foraging along the hedgerow",
            Self::Hilltop => "sit up on the Hilltop",
        }
    }

    /// For the departures board, where a place offers more than one thing.
    fn short(self) -> Option<&'static str> {
        match self {
            Self::HideAndSeek => Some("hide-and-seek"),
            Self::SackRace => Some("sack race"),
            Self::HighStriker => Some("high striker"),
            Self::Rummaging => Some("rummaging"),
            Self::Fishing => Some("fishing"),
            Self::BugCatching => Some("bugs"),
            Self::Foraging => Some("foraging"),
            _ => None,
        }
    }

    /// How much a companion would like it, from its axes, and a little more from its kind.
    fn appeal(self, character: &Character) -> f32 {
        use TemperamentKind::*;
        let a = character.axes;
        let kind = |kinds: &[TemperamentKind]| {
            if kinds.contains(&character.kind) {
                0.25
            } else {
                0.0
            }
        };
        match self {
            Self::Picnic => {
                0.5 * (1.0 - a.energy)
                    + 0.3 * a.affection
                    + 0.2 * a.social
                    + kind(&[Lazybones, Sweetheart])
            }
            Self::Stories => {
                0.4 * a.curiosity + 0.3 * (1.0 - a.energy) + 0.3 * a.social + kind(&[Scholar])
            }
            Self::HideAndSeek => {
                0.5 * a.playfulness
                    + 0.3 * a.social
                    + 0.2 * a.impulsiveness
                    + kind(&[Troublemaker, Showoff])
            }
            Self::SackRace => 0.45 * a.energy + 0.3 * a.playfulness + 0.25 * a.boldness,
            Self::HighStriker => {
                0.5 * a.feistiness + 0.3 * a.boldness + 0.2 * a.energy + kind(&[Grump])
            }
            Self::Rummaging => 0.5 * a.curiosity + 0.5 * a.boldness + kind(&[Explorer]),
            Self::Fishing => {
                0.6 * (1.0 - a.impulsiveness) + 0.4 * (1.0 - a.social) + kind(&[Wallflower, Grump])
            }
            Self::BugCatching => {
                0.5 * a.energy + 0.3 * a.impulsiveness + 0.2 * a.playfulness + kind(&[Oddball])
            }
            Self::Foraging => {
                0.35 * a.affection
                    + 0.35 * (1.0 - a.impulsiveness)
                    + 0.3 * a.curiosity
                    + kind(&[Sweetheart])
            }
            Self::Hilltop => {
                0.4 * (1.0 - a.energy)
                    + 0.3 * a.curiosity
                    + 0.3 * (1.0 - a.social)
                    + kind(&[Guardian])
            }
        }
    }
}

/// What a companion would most like to do: the Hilltop only once something stands there.
pub(super) fn keen_on(character: &Character, hilltop_stands: bool) -> Outing {
    Outing::ALL
        .into_iter()
        .filter(|outing| hilltop_stands || *outing != Outing::Hilltop)
        .max_by(|a, b| a.appeal(character).total_cmp(&b.appeal(character)))
        .unwrap_or(Outing::Picnic)
}

/// Everything the two boards read, gathered once.
#[derive(Clone, Debug, Default)]
pub(super) struct Board {
    /// Each traveller, and what it would most like to do.
    pub wishes: Vec<(String, Outing)>,
    pub visits: u32,
    /// Stories on the Clubhouse board, and the titles of those not yet told.
    pub stories: usize,
    pub untold: Vec<String>,
    /// Finds waiting in the satchel, and pieces standing on the Hilltop.
    pub satchel: u32,
    pub standing: usize,
    /// What has grown on the Hilltop since the last visit, as each is called now.
    pub grown: Vec<String>,
    /// What there is enough in the satchel to build on the Hilltop.
    pub buildable: Vec<String>,
    /// The quickest hide-and-seek among those here: who, and in how long.
    pub record: Option<(String, f32)>,
    /// The quickest sack race among those here, and the highest swing at the high striker (from
    /// 0 to 1, the bell).
    pub race_record: Option<(String, f32)>,
    pub striker_record: Option<(String, f32)>,
    /// The most rings rung in one turn at hoopla among those here, and the most tug-of-war wins.
    pub hoopla_record: Option<(String, u32)>,
    pub tug_record: Option<(String, u32)>,
    /// Souvenirs that go home on the train to this Desktop.
    pub going_home: usize,
    /// Community packages that could not be read.
    pub problems: usize,
}

/// A note on the notice board.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Notice {
    pub heading: String,
    pub text: String,
}

/// A row on the departures board.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Departure {
    pub area: Area,
    pub to: &'static str,
    pub on: String,
    /// Who would like to go, with what for where a place has more than one thing.
    pub keen: Vec<String>,
}

fn notice(heading: &str, text: String) -> Notice {
    Notice {
        heading: heading.to_owned(),
        text,
    }
}

/// What has grown on the Hilltop since the last visit, as a line for the Hilltop's bar.
pub(super) fn grown_since(grown: &[(u8, Standing)]) -> Option<String> {
    let names = grown_names(grown);
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    (!names.is_empty()).then(|| format!("Grown since last time: {}.", listed(&names)))
}

/// What has grown, as each is called now, ready to go in the middle of a sentence.
fn grown_names(grown: &[(u8, Standing)]) -> Vec<String> {
    grown
        .iter()
        .map(|(_, standing)| standing.name().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect()
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The notice board's notes, most welcoming first.
pub(super) fn notices(board: &Board) -> Vec<Notice> {
    let mut notes = vec![if board.visits <= 1 {
        notice(
            "Welcome to Formiga Hill",
            "The train comes and goes whenever you like, and everything here keeps until you \
             are back."
                .to_owned(),
        )
    } else {
        notice(
            "Welcome back",
            format!(
                "The colony's {} visit. Everything is where it was left.",
                ordinal(board.visits as usize)
            ),
        )
    }];

    let mut wishes = Vec::new();
    for outing in Outing::ALL {
        let names: Vec<&str> = board
            .wishes
            .iter()
            .filter(|(_, wish)| *wish == outing)
            .map(|(name, _)| name.as_str())
            .collect();
        if !names.is_empty() {
            wishes.push(format!(
                "{} would like to {}.",
                listed(&names),
                outing.wish()
            ));
        }
    }
    if !wishes.is_empty() {
        notes.push(notice("Wishes", wishes.join("\n")));
    }

    let untold: Vec<&str> = board.untold.iter().map(String::as_str).collect();
    let pinned = match untold.as_slice() {
        [] => None,
        [one] => Some(format!("{one} is")),
        [one, two] => Some(format!("{one} and {two} are")),
        [one, two, rest @ ..] => Some(format!("{one}, {two} and {} more are", rest.len())),
    };
    if let Some(pinned) = pinned {
        notes.push(notice(
            "At the Clubhouse",
            format!("{pinned} pinned up by the fire, not yet told."),
        ));
    }
    if !board.grown.is_empty() {
        let names: Vec<&str> = board.grown.iter().map(String::as_str).collect();
        notes.push(notice(
            "On the Hilltop",
            format!("Grown since last time: {}.", listed(&names)),
        ));
    }
    if board.satchel > 0 {
        notes.push(notice(
            "From the Woods",
            format!(
                "{} in the satchel, waiting for a spot on the Hilltop.",
                plural(board.satchel as usize, "find", "finds")
            ),
        ));
    }
    if !board.buildable.is_empty() {
        let names: Vec<&str> = board.buildable.iter().map(String::as_str).collect();
        notes.push(notice(
            "Plans",
            format!(
                "There is enough in the satchel to build {} on the Hilltop.",
                listed(&names)
            ),
        ));
    }
    let mut records = Vec::new();
    if let Some((name, seconds)) = &board.record {
        records.push(format!(
            "{name} found everyone at hide-and-seek in {}, the quickest yet.",
            clock(*seconds)
        ));
    }
    if let Some((name, seconds)) = &board.race_record {
        records.push(format!(
            "{name} hopped the sack race in {}, the quickest yet.",
            clock(*seconds)
        ));
    }
    if let Some((name, height)) = &board.striker_record {
        records.push(if *height >= 1.0 {
            format!("{name} rang the bell on the high striker.")
        } else {
            format!(
                "{name} sent the high striker's puck up to {}, the highest yet.",
                reached(*height)
            )
        });
    }
    if let Some((name, rings)) = &board.hoopla_record {
        records.push(match rings {
            3.. => format!("{name} rang all three in one turn at hoopla."),
            2 => format!("{name} rang two in one turn at hoopla, the most yet."),
            _ => format!("{name} rang one in a turn at hoopla, the most yet."),
        });
    }
    if let Some((name, wins)) = &board.tug_record {
        records.push(match wins {
            1 => format!("{name} has won the tug-of-war."),
            2 => format!("{name} has won the tug-of-war twice."),
            _ => format!("{name} has won the tug-of-war {wins} times."),
        });
    }
    if !records.is_empty() {
        let heading = if records.len() == 1 {
            "Fairground record"
        } else {
            "Fairground records"
        };
        notes.push(notice(heading, records.join("\n")));
    }
    if board.going_home > 0 {
        notes.push(notice(
            "Souvenirs",
            format!(
                "The colony's {} will go home on the train too, to the Journal in Formiga \
                 Desktop.",
                plural(board.going_home, "souvenir", "souvenirs")
            ),
        ));
    }
    if board.problems > 0 {
        notes.push(notice(
            "Story packages",
            format!(
                "{} could not be read. The Clubhouse's package list says why.",
                plural(board.problems, "package", "packages")
            ),
        ));
    }
    notes
}

/// The departures board: every other place, what is on there, and who would like to go.
pub(super) fn departures(board: &Board) -> Vec<Departure> {
    let keen = |area: Area| -> Vec<String> {
        board
            .wishes
            .iter()
            .filter(|(_, wish)| wish.area() == area)
            .map(|(name, wish)| match wish.short() {
                Some(what) => format!("{name} ({what})"),
                None => name.clone(),
            })
            .collect()
    };
    let stories = match (board.stories, board.untold.len()) {
        (0, _) => "Free play by the fire".to_owned(),
        (all, 0) => format!(
            "{} by the fire, every one told",
            plural(all, "story", "stories")
        ),
        (all, untold) => format!(
            "{} by the fire, {untold} not yet told",
            plural(all, "story", "stories")
        ),
    };
    let mut fair =
        "Hide-and-seek, the sack race, the high striker, hoopla and the tug-of-war, for \
                    the colony to play"
            .to_owned();
    let records: Vec<String> = [
        board
            .record
            .as_ref()
            .map(|(name, seconds)| format!("hide-and-seek, {name} {}", clock(*seconds))),
        board
            .race_record
            .as_ref()
            .map(|(name, seconds)| format!("the sack race, {name} {}", clock(*seconds))),
        board
            .striker_record
            .as_ref()
            .map(|(name, height)| format!("the high striker, {name} ({})", reached(*height))),
        board
            .hoopla_record
            .as_ref()
            .map(|(name, rings)| format!("hoopla, {name} ({})", rings_rung(*rings))),
        board.tug_record.as_ref().map(|(name, wins)| {
            let wins = if *wins == 1 {
                "1 win".to_owned()
            } else {
                format!("{wins} wins")
            };
            format!("the tug-of-war, {name} ({wins})")
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !records.is_empty() {
        fair.push_str(&format!("\nRecords: {}", records.join("; ")));
    }
    let mut hilltop = if board.standing == 0 {
        "Bare for now, for whatever the Woods turns up".to_owned()
    } else {
        format!("{} standing", plural(board.standing, "piece", "pieces"))
    };
    if board.satchel > 0 {
        hilltop.push_str(&format!(", {} in the satchel", board.satchel));
    }
    [
        (
            Area::Green,
            "The Village Green",
            "Free play, a brush, and the dress-up box".to_owned(),
        ),
        (Area::Clubhouse, "The Clubhouse", stories),
        (Area::Fairground, "The Fairground", fair),
        (
            Area::Woods,
            "The Woods",
            "Rummaging in the glade, fishing at the pool, bugs in the meadow, foraging along the \
             hedgerow"
                .to_owned(),
        ),
        (Area::Hilltop, "The Hilltop", hilltop),
    ]
    .into_iter()
    .map(|(area, to, on)| Departure {
        area,
        to,
        on,
        keen: keen(area),
    })
    .collect()
}

impl HillApp {
    /// What the boards read, from the snapshot and Hill's memories of this colony.
    pub(super) fn board(&self) -> Board {
        let colony = self.memories.colony();
        let hilltop_stands = !colony.hilltop.is_empty();
        let cast = &self.arrival.cast;
        let wishes = cast
            .members
            .iter()
            .map(|member| {
                let character = Character::of(member);
                (member.name.clone(), keen_on(&character, hilltop_stands))
            })
            .collect();
        let mut stories = 0;
        let mut untold = Vec::new();
        for (package, story) in self.on_the_board() {
            stories += 1;
            if !colony
                .stories
                .contains(&format!("{}/{}", package.id, story.id))
            {
                untold.push(story.title.clone());
            }
        }
        let record = cast
            .members
            .iter()
            .filter_map(|member| {
                let best = colony.quickest_seekers.get(&member.id.to_string())?;
                Some((member.name.clone(), *best))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let going_home = match &self.arrival.visit {
            Visit::Trip(trip) => trip.souvenirs_going_home(&colony.souvenirs).len(),
            Visit::Rehearsal(_) => 0,
        };
        Board {
            wishes,
            visits: colony.visits,
            stories,
            untold,
            satchel: colony.satchel.values().sum::<u32>() + colony.lifted.len() as u32,
            standing: colony.hilltop.len(),
            grown: grown_names(&self.grown),
            buildable: self
                .ready_to_build()
                .iter()
                .map(|plan| plan.name.to_lowercase())
                .collect(),
            record,
            race_record: self.race_record(),
            striker_record: self.striker_record(),
            hoopla_record: self.hoopla_record(),
            tug_record: self.tug_record(),
            going_home,
            problems: self.library.problems.len(),
        }
    }

    /// Pins a note on the station's notice board for each notice.
    pub(super) fn pin_notices(&mut self) {
        let notes = notices(&self.board()).len();
        self.station.show_notices(notes);
    }

    /// Opens the board the person clicked on, once everyone is on the platform.
    pub(super) fn look_at(&mut self, fixture: Fixture) {
        match fixture {
            Fixture::Notices => self.card = Some(Card::Notices),
            Fixture::Departures => self.card = Some(Card::Departures),
            Fixture::Case => {}
        }
    }

    pub(super) fn notices_window(&mut self, ctx: &egui::Context) {
        if self.area != Area::Station {
            return;
        }
        let notes = notices(&self.board());
        self.show_card(ctx, Card::Notices, "Notices", 0.42, |_, ui| {
            for (index, note) in notes.iter().enumerate() {
                if index > 0 {
                    paper::rule(ui);
                }
                ui.add(paper::Words::new(&note.heading).color(paper::FOREST));
                paper::words(ui, &note.text);
            }
        });
    }

    pub(super) fn departures_window(&mut self, ctx: &egui::Context, now: f32) {
        if self.area != Area::Station {
            return;
        }
        let rows = departures(&self.board());
        let ready = self.station.is_settled() && self.leaving.is_none();
        let mut target = None;
        self.show_card(ctx, Card::Departures, "Departures", 0.5, |_, ui| {
            for (index, row) in rows.iter().enumerate() {
                if index > 0 {
                    paper::rule(ui);
                }
                ui.horizontal(|ui| {
                    if ui.add(paper::Button::new("Go").enabled(ready)).clicked() {
                        target = Some(row.area);
                    }
                    ui.vertical(|ui| {
                        ui.add(paper::Words::new(row.to).color(paper::FOREST));
                        paper::words(ui, &row.on);
                        if !row.keen.is_empty() {
                            paper::aside(ui, format!("Keen: {}", row.keen.join(", ")));
                        }
                    });
                });
            }
        });
        if let Some(area) = target {
            self.go_to(area, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;

    fn sample() -> Vec<Character> {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        cast.members.iter().map(Character::of).collect()
    }

    #[test]
    fn what_a_companion_would_like_comes_from_its_temperament_and_differs() {
        let wishes: std::collections::BTreeSet<Outing> =
            sample().iter().map(|c| keen_on(c, true)).collect();
        assert!(wishes.len() >= 2, "everyone in the sample wants {wishes:?}");
    }

    #[test]
    fn every_outing_is_someones_favourite() {
        let mut character = sample().remove(0);
        let mut seen = std::collections::BTreeSet::new();
        for kind in [
            TemperamentKind::Lazybones,
            TemperamentKind::Scholar,
            TemperamentKind::Troublemaker,
            TemperamentKind::Explorer,
            TemperamentKind::Wallflower,
            TemperamentKind::Oddball,
            TemperamentKind::Guardian,
            TemperamentKind::Sweetheart,
        ] {
            for level in [0.0, 0.5, 1.0] {
                character.kind = kind;
                character.axes.energy = level;
                character.axes.curiosity = 1.0 - level;
                character.axes.impulsiveness = level;
                character.axes.social = 0.5;
                seen.insert(keen_on(&character, true));
            }
        }
        // Lively, playful and bold, and patient enough not to be off bug catching: the sack race.
        // Feisty and bold, and a grump: the high striker.
        let mut racer = sample().remove(0);
        racer.kind = TemperamentKind::Guardian;
        racer.axes.energy = 1.0;
        racer.axes.playfulness = 1.0;
        racer.axes.boldness = 1.0;
        racer.axes.curiosity = 0.0;
        racer.axes.impulsiveness = 0.0;
        racer.axes.social = 0.5;
        seen.insert(keen_on(&racer, true));
        let mut striker = sample().remove(0);
        striker.kind = TemperamentKind::Grump;
        striker.axes.feistiness = 1.0;
        striker.axes.boldness = 1.0;
        striker.axes.curiosity = 0.0;
        seen.insert(keen_on(&striker, true));
        assert_eq!(seen.len(), Outing::ALL.len(), "only {seen:?}");
    }

    #[test]
    fn nobody_longs_for_a_bare_hilltop() {
        let mut character = sample().remove(0);
        character.kind = TemperamentKind::Guardian;
        character.axes.energy = 0.0;
        character.axes.social = 0.0;
        assert_eq!(keen_on(&character, true), Outing::Hilltop);
        assert_ne!(keen_on(&character, false), Outing::Hilltop);
    }

    #[test]
    fn a_first_visit_is_welcomed_and_a_later_one_counted() {
        let first = notices(&Board {
            visits: 1,
            ..Board::default()
        });
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].heading, "Welcome to Formiga Hill");
        let later = notices(&Board {
            visits: 22,
            ..Board::default()
        });
        assert!(later[0].text.contains("22nd"));
    }

    #[test]
    fn notices_tell_only_what_there_is_to_tell() {
        let board = Board {
            visits: 3,
            wishes: vec![
                ("Pip".into(), Outing::Fishing),
                ("Moss".into(), Outing::Fishing),
                ("Fern".into(), Outing::Stories),
            ],
            stories: 3,
            untold: vec!["The Last Bun".into()],
            satchel: 1,
            record: Some(("Fern".into(), 42.0)),
            going_home: 2,
            ..Board::default()
        };
        let notes = notices(&board);
        let headings: Vec<&str> = notes.iter().map(|n| n.heading.as_str()).collect();
        assert_eq!(
            headings,
            [
                "Welcome back",
                "Wishes",
                "At the Clubhouse",
                "From the Woods",
                "Fairground record",
                "Souvenirs"
            ]
        );
        assert_eq!(
            notes[1].text,
            "Fern would like to hear a story by the Clubhouse fire.\n\
             Pip and Moss would like to go fishing at the pool."
        );
        assert!(notes[2].text.starts_with("The Last Bun is pinned up"));
        assert!(notes[3].text.starts_with("1 find in the satchel"));
        assert!(notes[4].text.contains("0:42"));
    }

    #[test]
    fn what_grew_since_last_time_is_told_on_the_board_and_on_the_hilltop() {
        let grown = [
            (
                3,
                Standing::Planted {
                    planted: "bluebell_bulb".into(),
                    stage: 1,
                },
            ),
            (9, Standing::from("acorn_stash")),
        ];
        let line = grown_since(&grown).unwrap();
        assert_eq!(
            line,
            "Grown since last time: bluebell leaves and a young oak."
        );
        assert_eq!(grown_since(&[]), None, "nothing grew, so nothing is said");
        let notes = notices(&Board {
            visits: 2,
            grown: grown_names(&grown),
            ..Board::default()
        });
        assert_eq!(notes[1].heading, "On the Hilltop");
        assert_eq!(notes[1].text, line);
    }

    #[test]
    fn plans_there_is_enough_for_are_an_invitation_on_the_board() {
        let notes = notices(&Board {
            visits: 2,
            buildable: vec!["the grand cairn".into(), "a picnic table".into()],
            ..Board::default()
        });
        assert_eq!(notes[1].heading, "Plans");
        assert_eq!(
            notes[1].text,
            "There is enough in the satchel to build the grand cairn and a picnic table on the \
             Hilltop."
        );
        let quiet = notices(&Board {
            visits: 2,
            ..Board::default()
        });
        assert!(quiet.iter().all(|note| note.heading != "Plans"));
    }

    #[test]
    fn the_fairground_records_are_told_together_without_any_that_are_not_set() {
        let one = notices(&Board {
            race_record: Some(("Pip".into(), 14.0)),
            ..Board::default()
        });
        let record = one
            .iter()
            .find(|n| n.heading.starts_with("Fairground"))
            .unwrap();
        assert_eq!(record.heading, "Fairground record");
        assert_eq!(
            record.text,
            "Pip hopped the sack race in 0:14, the quickest yet."
        );
        let all = notices(&Board {
            record: Some(("Fern".into(), 42.0)),
            race_record: Some(("Pip".into(), 14.0)),
            striker_record: Some(("Moss".into(), 1.0)),
            hoopla_record: Some(("Fig".into(), 3)),
            tug_record: Some(("Tansy".into(), 4)),
            ..Board::default()
        });
        let records = all
            .iter()
            .find(|n| n.heading.starts_with("Fairground"))
            .unwrap();
        assert_eq!(records.heading, "Fairground records");
        assert_eq!(records.text.lines().count(), 5);
        assert!(records.text.contains("Moss rang the bell"));
        assert!(
            records
                .text
                .contains("Fig rang all three in one turn at hoopla.")
        );
        assert!(
            records
                .text
                .contains("Tansy has won the tug-of-war 4 times.")
        );
        let few = notices(&Board {
            hoopla_record: Some(("Fig".into(), 1)),
            tug_record: Some(("Tansy".into(), 1)),
            ..Board::default()
        });
        let records = few
            .iter()
            .find(|n| n.heading.starts_with("Fairground"))
            .unwrap();
        assert_eq!(
            records.text,
            "Fig rang one in a turn at hoopla, the most yet.\nTansy has won the tug-of-war."
        );
        let short = notices(&Board {
            striker_record: Some(("Moss".into(), 0.74)),
            ..Board::default()
        });
        assert!(short.iter().any(|n| n.text.contains("up to the 7th mark")));
        assert!(
            notices(&Board::default())
                .iter()
                .all(|n| !n.heading.starts_with("Fairground")),
            "no record, no notice"
        );
    }

    #[test]
    fn the_fairground_row_lists_its_games_and_their_records() {
        let fair = |board: &Board| {
            departures(board)
                .into_iter()
                .find(|row| row.area == Area::Fairground)
                .unwrap()
        };
        let bare = fair(&Board::default());
        for game in [
            "Hide-and-seek",
            "the sack race",
            "the high striker",
            "hoopla",
            "the tug-of-war",
        ] {
            assert!(bare.on.contains(game), "{}", bare.on);
        }
        assert!(!bare.on.contains("Records"));
        let row = fair(&Board {
            race_record: Some(("Pip".into(), 14.0)),
            striker_record: Some(("Moss".into(), 1.0)),
            wishes: vec![("Fern".into(), Outing::SackRace)],
            ..Board::default()
        });
        assert!(
            row.on
                .ends_with("Records: the sack race, Pip 0:14; the high striker, Moss (the bell)"),
            "{}",
            row.on
        );
        assert_eq!(row.keen, ["Fern (sack race)"]);
        let row = fair(&Board {
            hoopla_record: Some(("Fig".into(), 2)),
            tug_record: Some(("Tansy".into(), 1)),
            ..Board::default()
        });
        assert!(
            row.on
                .ends_with("Records: hoopla, Fig (2 rings); the tug-of-war, Tansy (1 win)"),
            "{}",
            row.on
        );
    }

    #[test]
    fn the_departures_board_lists_every_other_place_with_who_would_like_to_go() {
        let board = Board {
            wishes: vec![
                ("Pip".into(), Outing::Fishing),
                ("Fern".into(), Outing::Picnic),
            ],
            ..Board::default()
        };
        let rows = departures(&board);
        let areas: Vec<Area> = rows.iter().map(|row| row.area).collect();
        assert_eq!(
            areas,
            [
                Area::Green,
                Area::Clubhouse,
                Area::Fairground,
                Area::Woods,
                Area::Hilltop
            ]
        );
        assert_eq!(rows[0].keen, ["Fern"]);
        assert_eq!(rows[3].keen, ["Pip (fishing)"]);
        assert!(rows[4].on.starts_with("Bare"));
    }

    #[test]
    fn ordinals_read_as_spoken() {
        let read: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 102, 111].map(ordinal).into();
        assert_eq!(
            read,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "102nd", "111th"
            ]
        );
    }
}
