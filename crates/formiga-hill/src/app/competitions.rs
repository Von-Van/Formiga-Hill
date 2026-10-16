//! The Fairground's competitions in the window: the card of them, Hill's own and any made here or
//! taken in from a file, to run, change, copy, pass on or remove; the editor for making one, with
//! its preview against this colony; and the tray while one is run, with how it stands.
//!
//! A competition made here is saved as a package in the packages folder, as any shared one is,
//! so one made here and one from a friend are kept, set aside and removed the same way. A file
//! someone gave is looked over on the same "Shared with you" card as a shared story or adventure
//! (`super::shared`), which shows its competitions as `describe` does.

use super::{Card, HillApp, listed, paper};
use crate::cast::{Cast, Id};
use crate::fairground::Game;
use crate::fairground::competition::{
    self, Entrants, Event, Format, Happening, MOST_ENTRANTS, MOST_EVENTS, MOST_PLACES, MOST_POINTS,
    MOST_ROUNDS, MOST_TARGET, MOST_TITLE_CHARS, Win,
};
use crate::sharing;
use crate::story::{Files, Origin, Package};
use eframe::egui;

/// The longest "by" line a competition made here can have.
const MOST_AUTHOR_CHARS: usize = 40;
/// Who a competition made here is by, until the person says.
const MADE_HERE: &str = "Made at the Hill";
/// What a competition made here has its package id begin with: never Hill's own.
const MADE_PREFIX: &str = "made.at.hill.";

/// What the competitions card is showing, and what happened last.
#[derive(Default)]
pub(super) struct Competitions {
    editing: Option<Editing>,
    /// The package id the person has asked to remove, waiting for them to say so again.
    removing: Option<String>,
    /// What came of the last thing done on the card, to say at its top.
    said: Option<String>,
    /// How the last competition here ended: its title and the standings.
    last: Option<(String, Vec<(Id, u32)>)>,
}

/// A competition being made or changed.
struct Editing {
    format: Format,
    author: String,
    /// The package it will be saved over, if it is one made here being changed, with its version.
    over: Option<(String, String)>,
}

/// What the person did on the card, done once it is drawn.
enum Act {
    Run(Format),
    Edit(Editing),
    Export(Files, String),
    Open,
    AskRemove(String),
    Remove(String),
    Close,
    Save,
}

/// The files of a package holding just `format`, by `author`, with this id and version.
fn package_files(format: &Format, author: &str, id: &str, version: &str) -> Files {
    let quoted = |text: &str| toml::Value::from(text.trim()).to_string();
    let entry = format!("content/{}.toml", format.id);
    let manifest = format!(
        "package_id = \"{id}\"\ntitle = {}\nauthor = {}\nversion = \"{version}\"\nhill_api = {}\n\
         content_types = [\"competition\"]\nentry_points = [\"{entry}\"]\ndefault_locale = \"en\"\n",
        quoted(&format.title),
        quoted(author),
        crate::story::HILL_API,
    );
    Files::from([
        ("manifest.toml".to_owned(), manifest.into_bytes()),
        (entry, format.content().into_bytes()),
        (
            "localization/en.toml".to_owned(),
            format.words().into_bytes(),
        ),
    ])
}

/// A new package id for a competition made here, from its title and the moment, so two made
/// with the same title, here or by a friend, never take each other's place.
fn new_package_id(title: &str) -> String {
    let moment = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_micros());
    format!(
        "{MADE_PREFIX}{}-{:05x}",
        sharing::slug(title, 40),
        moment & 0xf_ffff
    )
}

/// The version after `version`: whole numbers counting up, from 1.
fn next_version(version: Option<&str>) -> String {
    version
        .and_then(|version| version.parse::<u32>().ok())
        .map_or(1, |n| n + 1)
        .to_string()
}

/// One short line saying who a competition is by.
fn is_by(text: &str) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= MOST_AUTHOR_CHARS
        && !text.chars().any(char::is_control)
}

/// Whether a package holds only competitions, so it can be changed or removed from here.
fn only_competitions(package: &Package) -> bool {
    package.stories.is_empty() && !package.competitions.is_empty()
}

/// Whether a competition can be changed and saved in its package's place: the package is in the
/// packages folder and holds it alone, so saving just it leaves nothing of the package out.
fn changeable(origin: Option<Origin>, package: &Package) -> bool {
    origin == Some(Origin::Folder) && only_competitions(package) && package.competitions.len() == 1
}

/// A name, or "someone" for anyone not here.
fn name(cast: &Cast, id: Id) -> String {
    cast.member(id)
        .map_or("someone".to_owned(), |member| member.name.clone())
}

/// The standings as the tray says them: "Fig 9 · Moss 6 · Pip 4".
pub(super) fn standings_said(cast: &Cast, standings: &[(Id, u32)]) -> String {
    standings
        .iter()
        .map(|(id, points)| format!("{} {points}", name(cast, *id)))
        .collect::<Vec<_>>()
        .join("  \u{b7}  ")
}

/// Who plays an event, as said before it starts: "everyone", or their names.
fn players_said(cast: &Cast, players: &[Id]) -> String {
    if players.len() == cast.members.len() {
        return "everyone".to_owned();
    }
    let names: Vec<String> = players.iter().map(|id| name(cast, *id)).collect();
    listed(&names)
}

/// "points" or "point".
fn points(count: u32) -> &'static str {
    if count == 1 { "point" } else { "points" }
}

/// What the person is told when something happens in the competition called `title`.
pub(super) fn told(cast: &Cast, title: &str, happening: &Happening) -> String {
    match happening {
        Happening::Next {
            game,
            round,
            rounds,
            players,
        } => {
            let round = if *rounds > 1 {
                format!(", round {round} of {rounds}")
            } else {
                String::new()
            };
            format!(
                "{title}: {}{round}, for {}.",
                game.name(),
                players_said(cast, players)
            )
        }
        Happening::Skipped { game } => format!(
            "{title}: too few here to play {}, so on to the next.",
            lower(game.name())
        ),
        Happening::Scored { places, .. } if places.is_empty() => {
            format!("{title}: nobody scored that time.")
        }
        Happening::Scored { places, .. } => {
            let scored: Vec<String> = places
                .iter()
                .map(|(ids, worth)| {
                    let names: Vec<String> = ids.iter().map(|id| name(cast, *id)).collect();
                    format!("{} {worth}", listed(&names))
                })
                .collect();
            format!("Points: {}.", scored.join(", "))
        }
        Happening::Won { winners, .. } if winners.is_empty() => {
            format!("{title} is over, and nobody scored a point.")
        }
        Happening::Won {
            winners,
            points: on,
        } => {
            let names: Vec<String> = winners.iter().map(|id| name(cast, *id)).collect();
            if winners.len() == 1 {
                format!(
                    "{} wins {title}, with {on} {}!",
                    listed(&names),
                    points(*on)
                )
            } else {
                format!(
                    "{} share {title}, on {on} {} each!",
                    listed(&names),
                    points(*on)
                )
            }
        }
        Happening::CalledOff => format!("{title} is called off."),
    }
}

/// A game's name as it reads mid-sentence: "the sack race", "hoopla".
fn lower(name: &str) -> String {
    super::lower(name)
}

impl HillApp {
    /// Tells the person what is happening in the competition, and keeps how it ended.
    pub(super) fn contest_events(&mut self) -> Option<String> {
        let (_, games) = self.fairground.as_mut()?;
        let contest = games.contest.as_mut()?;
        let title = contest.format().title.clone();
        let happened = contest.take_happenings();
        let over = contest.is_over();
        let standings = contest.standings();
        if over {
            games.contest = None;
            self.competitions.last = Some((title.clone(), standings));
        }
        let cast = &self.arrival.cast;
        let lines: Vec<String> = happened
            .iter()
            .map(|happening| told(cast, &title, happening))
            .collect();
        (!lines.is_empty()).then(|| lines.join(" "))
    }

    /// The tray while a competition is run and no game of it is on: what is next, how it
    /// stands, and the way to stop it.
    pub(super) fn contest_tray(&mut self, ui: &mut egui::Ui) {
        let Some((ground, games)) = &mut self.fairground else {
            return;
        };
        let cast = &self.arrival.cast;
        let Some(contest) = &mut games.contest else {
            return;
        };
        let (at, round) = contest.at();
        let of = contest.format().events.len();
        let next = contest.event().map_or(String::new(), |event| {
            let round = if event.rounds > 1 {
                format!(", round {round} of {}", event.rounds)
            } else {
                String::new()
            };
            format!(
                "  \u{b7}  next: {}{round} ({at} of {of})",
                lower(event.game.name())
            )
        });
        paper::slip(ui, format!("{}{next}", contest.format().title));
        let standings = contest.standings();
        if !standings.is_empty() {
            let slip = paper::slip(
                ui,
                standings_said(cast, &standings[..standings.len().min(3)]),
            );
            paper::explain(ui, slip, &standings_said(cast, &standings));
        }
        if paper::button(ui, "Call it off").clicked() {
            contest.call_off();
            let _ = ground;
        }
    }

    /// A slip naming the competition a game is part of, while it is on.
    pub(super) fn contest_slip(&self, ui: &mut egui::Ui) {
        let Some((_, games)) = &self.fairground else {
            return;
        };
        let Some(contest) = games.contest.as_ref().filter(|contest| !contest.is_over()) else {
            return;
        };
        let (at, round) = contest.at();
        let rounds = contest.event().map_or(1, |event| event.rounds);
        let round = if rounds > 1 {
            format!(", round {round} of {rounds}")
        } else {
            String::new()
        };
        let slip = paper::slip(
            ui,
            format!(
                "{}  \u{b7}  {at} of {}{round}",
                contest.format().title,
                contest.format().events.len()
            ),
        );
        let standings = contest.standings();
        paper::explain(ui, slip, &standings_said(&self.arrival.cast, &standings));
    }

    /// The button for the competitions card, and how the last one ended.
    pub(super) fn competitions_button(&mut self, ui: &mut egui::Ui) {
        let open = self.showing(Card::Competitions);
        let button = ui.add(paper::Button::new("Competitions").selected(open));
        if paper::explain(
            ui,
            button,
            "Run a competition of the Fairground's games, or make your own.",
        )
        .clicked()
        {
            self.toggle(Card::Competitions);
        }
        if let Some((title, standings)) = &self.competitions.last {
            let slip = paper::slip(ui, "Last competition");
            paper::explain(
                ui,
                slip,
                &format!("{title}: {}", standings_said(&self.arrival.cast, standings)),
            );
        }
    }

    /// Whether a competition can be run now, and if not, why not.
    fn can_run(&self) -> Result<(), &'static str> {
        if self.arrival.cast.members.len() < 2 {
            return Err("It takes two or more travellers for a competition.");
        }
        match &self.fairground {
            Some((_, games)) if games.playing().is_some() || games.contest_on() => {
                Err("A game is on: wait for it to end, or call it off.")
            }
            Some(_) => Ok(()),
            None => Err("Go to the Fairground to run it."),
        }
    }

    /// The competitions card: the list, or the editor.
    pub(super) fn competitions_window(&mut self, ctx: &egui::Context, now: f32) {
        let title = if self.competitions.editing.is_some() {
            "Make a competition"
        } else {
            "Competitions"
        };
        let acted = self.show_card(ctx, Card::Competitions, title, 0.56, |app, ui| {
            if let Some(said) = &app.competitions.said {
                paper::hint(ui, said);
            }
            if app.competitions.editing.is_some() {
                app.editor(ui)
            } else {
                app.list(ui)
            }
        });
        if let Some(Some(act)) = acted {
            self.act(act, now);
        }
    }

    /// Every competition there is to run.
    fn list(&mut self, ui: &mut egui::Ui) -> Option<Act> {
        let mut act = None;
        paper::aside(
            ui,
            "A competition is a run of the Fairground's games, for points. The colony plays \
             each game as it always does; you watch.",
        );
        let can_run = self.can_run();
        let removing = self.competitions.removing.clone();
        for (package, format) in self.library.competitions() {
            if self.set_aside.contains(&package.id) {
                continue;
            }
            let origin = self.library.origin(&package.id);
            paper::rule(ui);
            let heading = paper::name(ui, &format.title);
            paper::explain(ui, heading, &format.told().join("\n"));
            let by = match origin {
                Some(Origin::Official) => "Hill's own".to_owned(),
                _ => format!("by {}", package.author),
            };
            paper::aside(
                ui,
                format!(
                    "{by}  \u{b7}  {} {}  \u{b7}  {}",
                    format.games(),
                    if format.games() == 1 { "game" } else { "games" },
                    format.win.said()
                ),
            );
            ui.horizontal_wrapped(|ui| {
                let run = ui.add(paper::Button::new("Run it").enabled(can_run.is_ok()));
                let why = can_run.err().unwrap_or("The colony plays it; you watch.");
                if paper::explain(ui, run, why).clicked() {
                    act = Some(Act::Run(format.clone()));
                }
                let changeable = changeable(origin, package);
                let label = if changeable { "Change" } else { "Copy" };
                let change = paper::button(ui, label);
                let about = if changeable {
                    "Change it, and save it in its place."
                } else {
                    "Make your own from it, to change as you like."
                };
                if paper::explain(ui, change, about).clicked() {
                    act = Some(Act::Edit(Editing {
                        format: format.clone(),
                        author: if changeable {
                            package.author.clone()
                        } else {
                            MADE_HERE.to_owned()
                        },
                        over: changeable.then(|| (package.id.clone(), package.version.clone())),
                    }));
                }
                if origin != Some(Origin::Official) {
                    let pass = paper::button(ui, "Pass it on");
                    if paper::explain(ui, pass, "Save it as a file to give to someone.").clicked() {
                        let files =
                            package_files(format, &package.author, &package.id, &package.version);
                        let files = if only_competitions(package) && package.competitions.len() == 1
                        {
                            files
                        } else {
                            // A package with more in it is passed on whole, from the folder.
                            Files::new()
                        };
                        act = Some(Act::Export(files, package.id.clone()));
                    }
                }
                if origin == Some(Origin::Folder) && only_competitions(package) {
                    if removing.as_deref() == Some(package.id.as_str()) {
                        if paper::button(ui, "Remove it for good").clicked() {
                            act = Some(Act::Remove(package.id.clone()));
                        }
                        if paper::button(ui, "Keep it").clicked() {
                            act = Some(Act::AskRemove(String::new()));
                        }
                    } else {
                        let remove = paper::button(ui, "Remove");
                        if paper::explain(ui, remove, "Take it out of Hill.").clicked() {
                            act = Some(Act::AskRemove(package.id.clone()));
                        }
                    }
                }
            });
        }
        paper::rule(ui);
        ui.horizontal_wrapped(|ui| {
            let new = paper::button(ui, "Make a new one");
            if paper::explain(ui, new, "Choose the games, who plays, and how it is won.").clicked()
            {
                act = Some(Act::Edit(Editing {
                    format: Format::starter(),
                    author: MADE_HERE.to_owned(),
                    over: None,
                }));
            }
            let open = paper::button(ui, "Open a shared one\u{2026}");
            if paper::explain(
                ui,
                open,
                "Look at a competition someone gave you, before adding it.",
            )
            .clicked()
            {
                act = Some(Act::Open);
            }
        });
        act
    }

    /// The editor: a competition's title, games, who plays, points and how it is won, and a
    /// preview of how it would go with this colony.
    fn editor(&mut self, ui: &mut egui::Ui) -> Option<Act> {
        let mut act = None;
        let can_run = self.can_run();
        let cast = &self.arrival.cast;
        let can_keep = self.packages_folder.is_some();
        let editing = self.competitions.editing.as_mut()?;
        let format = &mut editing.format;
        format.id = sharing::slug(&format.title, 40);

        // Trying and saving come first, so they are in reach however long it grows.
        let mut problems = format.problems();
        if !is_by(&editing.author) {
            problems.push("Say who it is by, in a few words.".to_owned());
        }
        for problem in &problems {
            paper::faint(ui, format!("To put right: {problem}"));
        }
        let ready = problems.is_empty();
        ui.horizontal_wrapped(|ui| {
            let try_it = ui.add(paper::Button::new("Try it now").enabled(ready && can_run.is_ok()));
            let why = if ready {
                can_run
                    .err()
                    .unwrap_or("Run it with the colony, without saving it.")
            } else {
                "Put it right first."
            };
            if paper::explain(ui, try_it, why).clicked() {
                act = Some(Act::Run(format.clone()));
            }
            let save = ui.add(paper::Button::new("Save").enabled(ready && can_keep));
            let why = if !can_keep {
                "There is nowhere to keep competitions on this computer."
            } else if editing.over.is_some() {
                "Save it in place of the one it was."
            } else {
                "Keep it with the Fairground's competitions."
            };
            if paper::explain(ui, save, why).clicked() {
                act = Some(Act::Save);
            }
            let back = paper::button(ui, "Back");
            if paper::explain(ui, back, "Leave it without saving.").clicked() {
                act = Some(Act::Close);
            }
        });
        paper::rule(ui);

        paper::heading(ui, "Called");
        ui.add(
            egui::TextEdit::singleline(&mut format.title)
                .char_limit(MOST_TITLE_CHARS)
                .desired_width(ui.available_width().min(320.0)),
        );
        paper::heading(ui, "By");
        ui.add(
            egui::TextEdit::singleline(&mut editing.author)
                .char_limit(MOST_AUTHOR_CHARS)
                .desired_width(ui.available_width().min(320.0)),
        );

        paper::heading(ui, "The games, in order");
        let mut moved = None;
        let mut dropped = None;
        let count = format.events.len();
        for (index, event) in format.events.iter_mut().enumerate() {
            ui.horizontal_wrapped(|ui| {
                paper::words(ui, format!("{}.", index + 1));
                let (_, picked) = paper::menu(
                    ui,
                    paper::Button::new(format!("{} \u{25be}", event.game.name())),
                    false,
                    |ui| {
                        let mut picked = None;
                        for game in Game::ALL {
                            let button =
                                paper::Button::new(game.name()).selected(game == event.game);
                            if ui.add(button).clicked() {
                                picked = Some(game);
                            }
                        }
                        picked
                    },
                );
                if let Some(Some(game)) = picked {
                    event.game = game;
                }
                entrants(ui, &mut event.players);
                stepper(
                    ui,
                    &mut event.rounds,
                    1,
                    MOST_ROUNDS,
                    |rounds| match rounds {
                        1 => "1 round".to_owned(),
                        n => format!("{n} rounds"),
                    },
                );
                let up = ui.add(paper::Button::new("\u{25b4}").enabled(index > 0));
                if paper::explain(ui, up, "Earlier").clicked() {
                    moved = Some((index, index - 1));
                }
                let down = ui.add(paper::Button::new("\u{25be}").enabled(index + 1 < count));
                if paper::explain(ui, down, "Later").clicked() {
                    moved = Some((index, index + 1));
                }
                let drop = ui.add(paper::Button::new("\u{d7}").enabled(count > 1));
                if paper::explain(ui, drop, "Take this game out").clicked() {
                    dropped = Some(index);
                }
            });
        }
        if let Some((from, to)) = moved {
            format.events.swap(from, to);
        }
        if let Some(index) = dropped {
            format.events.remove(index);
        }
        let add = ui.add(paper::Button::new("Add a game").enabled(count < MOST_EVENTS));
        if add.clicked() {
            let game = Game::ALL[format.events.len() % Game::ALL.len()];
            format.events.push(Event {
                game,
                rounds: 1,
                players: Entrants::Everyone,
            });
        }

        paper::heading(ui, "Points for each place");
        ui.horizontal_wrapped(|ui| {
            for (index, worth) in format.points.iter_mut().enumerate() {
                let place = super::ordinal(index + 1);
                stepper(ui, worth, 0, MOST_POINTS, |worth| {
                    format!("{place}: {worth}")
                });
            }
        });
        ui.horizontal_wrapped(|ui| {
            let places = format.points.len();
            if ui
                .add(paper::Button::new("Another place").enabled(places < MOST_PLACES))
                .clicked()
            {
                let last = format.points.last().copied().unwrap_or(1);
                format.points.push(last.saturating_sub(1));
            }
            if ui
                .add(paper::Button::new("One place fewer").enabled(places > 1))
                .clicked()
            {
                format.points.pop();
            }
        });

        paper::heading(ui, "Who wins");
        ui.horizontal_wrapped(|ui| {
            let most = ui.add(
                paper::Button::new("The most points at the end")
                    .selected(format.win == Win::MostPoints),
            );
            if most.clicked() {
                format.win = Win::MostPoints;
            }
            let target = match format.win {
                Win::FirstTo(target) => Some(target),
                Win::MostPoints => None,
            };
            let first = ui.add(paper::Button::new("The first to").selected(target.is_some()));
            if first.clicked() && target.is_none() {
                format.win = Win::FirstTo(10);
            }
            if let Some(mut target) = target {
                stepper(ui, &mut target, 1, MOST_TARGET, |target| {
                    format!("{target} {}", points(target))
                });
                format.win = Win::FirstTo(target);
            }
        });

        paper::heading(ui, "How it goes");
        for line in format.told() {
            paper::words(ui, line);
        }
        for line in preview(cast, format) {
            paper::aside(ui, line);
        }
        act
    }

    /// Does what the person asked on the card.
    fn act(&mut self, act: Act, now: f32) {
        let state = &mut self.competitions;
        state.said = None;
        match act {
            Act::Run(format) => {
                if self.can_run().is_ok()
                    && let Some((_, games)) = &mut self.fairground
                {
                    games.run(format, now);
                    self.card = None;
                }
            }
            Act::Edit(editing) => {
                state.removing = None;
                state.editing = Some(editing);
            }
            Act::Close => state.editing = None,
            Act::AskRemove(id) => state.removing = (!id.is_empty()).then_some(id),
            Act::Remove(id) => {
                state.removing = None;
                let Some(folder) = &self.packages_folder else {
                    return;
                };
                state.said = Some(match sharing::remove(folder, &id) {
                    Ok(()) => {
                        let title = take_out(&mut self.library, &id);
                        format!("Removed {title}.")
                    }
                    Err(problem) => format!("It couldn't be removed: {}.", problem.problem),
                });
            }
            Act::Save => {
                let Some(editing) = state.editing.take() else {
                    return;
                };
                let Some(folder) = &self.packages_folder else {
                    return;
                };
                let (id, version) = match &editing.over {
                    Some((id, version)) => (id.clone(), next_version(Some(version))),
                    None => (new_package_id(&editing.format.title), next_version(None)),
                };
                let files = package_files(&editing.format, &editing.author, &id, &version);
                let title = editing.format.title.trim().to_owned();
                match put_in(&mut self.library, folder, &files) {
                    Ok(_) => state.said = Some(format!("Saved {title}.")),
                    Err(problem) => {
                        state.said = Some(format!("It couldn't be saved: {problem}."));
                        state.editing = Some(editing);
                    }
                }
            }
            Act::Export(files, id) => {
                let files = if files.is_empty() {
                    let in_folder = self
                        .packages_folder
                        .as_deref()
                        .and_then(|folder| sharing::installed(folder, &id))
                        .and_then(|path| crate::story::read_folder(&path).ok());
                    match in_folder {
                        Some(files) => files,
                        None => {
                            state.said = Some("It couldn't be read to pass on.".to_owned());
                            return;
                        }
                    }
                } else {
                    files
                };
                let name = format!(
                    "{}.{}",
                    id.strip_prefix(MADE_PREFIX).unwrap_or(&id),
                    sharing::EXTENSION
                );
                let Some(path) = rfd::FileDialog::new()
                    .set_title("Pass on a competition")
                    .set_file_name(&name)
                    .add_filter("Formiga Hill file", &[sharing::EXTENSION])
                    .save_file()
                else {
                    return;
                };
                state.said = Some(match sharing::export(&files, &path) {
                    Ok(()) => format!(
                        "Saved to {}: give it to anyone with Formiga Hill.",
                        path.display()
                    ),
                    Err(problem) => format!("It couldn't be saved: {}.", problem.problem),
                });
            }
            // Looked over where anything shared is, before it is added.
            Act::Open => self.choose_shared_file(now),
        }
    }

    /// Sets the competitions card up for a review picture: the list, or the editor with a new
    /// competition in it.
    pub(super) fn snap_competitions(&mut self, editing: bool) {
        if editing {
            self.competitions.editing = Some(Editing {
                format: Format::starter(),
                author: MADE_HERE.to_owned(),
                over: None,
            });
        }
    }
}

/// One competition in a shared package, as the "Shared with you" card shows it: its games in
/// order, who plays each and how it is won.
pub(super) fn describe(ui: &mut egui::Ui, format: &Format) {
    paper::heading(ui, format!("At the Fairground: {}", format.title));
    for line in format.told() {
        paper::words(ui, line);
    }
    paper::aside(
        ui,
        "It only orders the Fairground's own games and scores them: it can't change anyone, or \
         how they play.",
    );
}

/// Lets a package into the packages `folder`, and into the library in place of any with its id.
/// Returns the version it replaced, if any.
fn put_in(
    library: &mut crate::story::Library,
    folder: &std::path::Path,
    files: &Files,
) -> Result<Option<String>, String> {
    let installed = sharing::install(folder, files).map_err(|problem| problem.problem)?;
    let package = sharing::inspect(files.clone(), "the package")
        .map_err(|problem| problem.problem)?
        .package;
    take_out(library, &package.id);
    library.packages.push(package);
    library.origins.push(Origin::Folder);
    Ok(installed.replaced)
}

/// Takes the package with this id out of the library, and says what it was called.
fn take_out(library: &mut crate::story::Library, id: &str) -> String {
    let Some(index) = library.packages.iter().position(|package| package.id == id) else {
        return "it".to_owned();
    };
    library.origins.remove(index);
    let package = library.packages.remove(index);
    package
        .competitions
        .first()
        .map_or(package.title, |format| format.title.clone())
}

/// A count with buttons either side to make it less or more, between `least` and `most`.
fn stepper<T>(ui: &mut egui::Ui, value: &mut T, least: T, most: T, said: impl Fn(T) -> String)
where
    T: Copy + PartialOrd + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + From<u8>,
{
    let one = T::from(1);
    if ui
        .add(paper::Button::new("\u{2212}").enabled(*value > least))
        .clicked()
    {
        *value = *value - one;
    }
    paper::words(ui, said(*value));
    if ui
        .add(paper::Button::new("+").enabled(*value < most))
        .clicked()
    {
        *value = *value + one;
    }
}

/// The choice of who plays an event.
fn entrants(ui: &mut egui::Ui, players: &mut Entrants) {
    let label = match *players {
        Entrants::Everyone => "Everyone".to_owned(),
        other => capitalised(&other.said()),
    };
    paper::menu(
        ui,
        paper::Button::new(format!("{label} \u{25be}")),
        true,
        |ui| {
            if ui
                .add(paper::Button::new("Everyone").selected(*players == Entrants::Everyone))
                .clicked()
            {
                *players = Entrants::Everyone;
            }
            let keenest = matches!(players, Entrants::Keenest(_));
            let button = ui.add(paper::Button::new("The keenest few").selected(keenest));
            if paper::explain(
                ui,
                button,
                "Whoever is keenest on this game, by temperament.",
            )
            .clicked()
                && !keenest
            {
                *players = Entrants::Keenest(3);
            }
            let leaders = matches!(players, Entrants::Leaders(_));
            let button = ui.add(paper::Button::new("Whoever leads so far").selected(leaders));
            if paper::explain(ui, button, "A final: the few with the most points so far.").clicked()
                && !leaders
            {
                *players = Entrants::Leaders(2);
            }
            match players {
                Entrants::Keenest(count) | Entrants::Leaders(count) => {
                    ui.horizontal(|ui| {
                        stepper(ui, count, 1, MOST_ENTRANTS, |count| {
                            format!("{count} of them")
                        });
                    });
                }
                Entrants::Everyone => {}
            }
        },
    );
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or(String::new(), |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// How the competition would go with this colony, as far as can be said before it is played:
/// who would play each game for the keenest, and any game too few here could play.
fn preview(cast: &Cast, format: &Format) -> Vec<String> {
    let mut lines = Vec::new();
    let here = cast.members.len();
    for (index, event) in format.events.iter().enumerate() {
        let can = match event.players {
            Entrants::Everyone => here,
            Entrants::Keenest(count) | Entrants::Leaders(count) => here.min(count as usize),
        };
        if can < competition::fewest(event.game) {
            lines.push(format!(
                "{}. With {here} here, too few can play {}: it would be passed over.",
                index + 1,
                lower(event.game.name())
            ));
            continue;
        }
        if let Entrants::Keenest(count) = event.players {
            let mut keen: Vec<(Id, f32)> = cast
                .members
                .iter()
                .map(|member| {
                    let character = crate::character::Character::of(member);
                    (member.id, competition::keenness(event.game, &character))
                })
                .collect();
            keen.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let names: Vec<String> = keen
                .iter()
                .take(count as usize)
                .map(|(id, _)| name(cast, *id))
                .collect();
            lines.push(format!(
                "{}. Here, that would be {}.",
                index + 1,
                listed(&names)
            ));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_competition_made_here_is_a_package_that_loads_as_it_was_made() {
        let mut format = Format::starter();
        format.title = "Pip's \"Big\" Day".to_owned();
        format.id = sharing::slug(&format.title, 40);
        format.win = Win::FirstTo(6);
        let id = new_package_id(&format.title);
        assert!(id.starts_with("made.at.hill.pip-s-big-day-"));
        let files = package_files(&format, "Jake's colony", &id, "1");
        let inspection = sharing::inspect(files, "made here").unwrap();
        assert_eq!(inspection.summary.author, "Jake's colony");
        assert_eq!(inspection.summary.content_types, ["competition"]);
        assert_eq!(inspection.package.competitions, [format]);
        assert!(inspection.package.stories.is_empty());
    }

    #[test]
    fn a_competition_passes_on_as_a_file_and_comes_back_the_same() {
        let format = Format::starter();
        let files = package_files(&format, MADE_HERE, &new_package_id("x"), "3");
        let text = sharing::encode(&files, "out").unwrap();
        let back = sharing::decode(text.as_bytes(), "in").unwrap();
        assert_eq!(back, files);
        let package = sharing::inspect(back, "in").unwrap().package;
        assert_eq!(package.competitions, [format]);
        assert_eq!(package.version, "3");
    }

    #[test]
    fn saving_counts_versions_up_and_installing_replaces_by_id() {
        assert_eq!(next_version(None), "1");
        assert_eq!(next_version(Some("1")), "2");
        assert_eq!(next_version(Some("1.0.0")), "1");
        let folder = std::env::temp_dir().join(format!("hill-competitions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let mut library = crate::story::Library::load(None, &[]);
        let before = library.packages.len();
        let mut format = Format::starter();
        let id = new_package_id(&format.title);
        let first = package_files(&format, MADE_HERE, &id, "1");
        assert_eq!(put_in(&mut library, &folder, &first), Ok(None));
        format.title = "Changed".to_owned();
        let second = package_files(&format, MADE_HERE, &id, "2");
        assert_eq!(put_in(&mut library, &folder, &second), Ok(Some("1".into())));
        assert_eq!(library.packages.len(), before + 1);
        assert_eq!(library.origin(&id), Some(Origin::Folder));
        assert_eq!(take_out(&mut library, &id), "Changed");
        assert_eq!(library.packages.len(), before);
        // Hill's own can't be taken over by one made to look like it.
        let mut official = first.clone();
        let manifest = String::from_utf8(official["manifest.toml"].clone())
            .unwrap()
            .replace(&id, "com.formiga.hill.fairground-games");
        official.insert("manifest.toml".into(), manifest.into_bytes());
        assert!(put_in(&mut library, &folder, &official).is_err());
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// A package of `competitions` copies of the starter competition, and nothing else.
    fn package_of(competitions: usize) -> Package {
        Package {
            id: new_package_id("x"),
            title: "Games".to_owned(),
            author: MADE_HERE.to_owned(),
            version: "1".to_owned(),
            stories: Vec::new(),
            adventures: Vec::new(),
            competitions: vec![Format::starter(); competitions],
        }
    }

    #[test]
    fn only_a_folder_package_of_one_competition_is_changed_in_its_place() {
        let one = package_of(1);
        assert!(changeable(Some(Origin::Folder), &one));
        assert!(!changeable(Some(Origin::Official), &one));
        // Saving one of two in the package's place would lose the other: it is copied instead.
        assert!(!changeable(Some(Origin::Folder), &package_of(2)));
    }

    #[test]
    fn what_happens_in_a_competition_reads_as_said() {
        let cast = crate::fairground::testing::colony_of(3);
        let ids: Vec<Id> = cast.ids().collect();
        let names: Vec<String> = cast.members.iter().map(|m| m.name.clone()).collect();
        let next = Happening::Next {
            game: Game::SackRace,
            round: 2,
            rounds: 3,
            players: ids.clone(),
        };
        assert_eq!(
            told(&cast, "Games", &next),
            "Games: The sack race, round 2 of 3, for everyone."
        );
        let scored = Happening::Scored {
            game: Game::HighStriker,
            places: vec![(vec![ids[0], ids[1]], 3), (vec![ids[2]], 1)],
        };
        assert_eq!(
            told(&cast, "Games", &scored),
            format!("Points: {} and {} 3, {} 1.", names[0], names[1], names[2])
        );
        let won = Happening::Won {
            winners: vec![ids[2]],
            points: 1,
        };
        assert_eq!(
            told(&cast, "Games", &won),
            format!("{} wins Games, with 1 point!", names[2])
        );
        let shared = Happening::Won {
            winners: vec![ids[0], ids[1]],
            points: 7,
        };
        assert!(told(&cast, "Games", &shared).contains("share Games, on 7 points each!"));
    }

    #[test]
    fn the_preview_names_the_keenest_here_and_warns_of_games_too_few_can_play() {
        let cast = crate::fairground::testing::colony_of(1);
        let mut format = Format::starter();
        format.events[1].players = Entrants::Keenest(2);
        let lines = preview(&cast, &format);
        assert!(
            lines[0].contains("too few can play the sack race"),
            "{lines:?}"
        );
        assert!(lines[1].contains(&cast.members[0].name), "{lines:?}");
    }
}
