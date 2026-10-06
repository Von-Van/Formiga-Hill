//! The Fairground's tray: which game to watch and who plays it, the game under way, and what the
//! person is told as it goes; and what Hill remembers of each game once it is seen through.

use super::{HillApp, clock, paper, tools};
use crate::audio::Cue;
use crate::cast::{Cast, Id};
use crate::fairground::tug_of_war::{self, Side};
use crate::fairground::{self, Event, Game, Phase, Players, high_striker, hoopla, sack_race};
use crate::story::souvenirs;
use eframe::egui;

/// A name, or "Someone" for anyone the cast doesn't know.
fn name(cast: &Cast, id: Option<Id>) -> String {
    id.and_then(|id| cast.member(id))
        .map_or("Someone".to_owned(), |member| member.name.clone())
}

/// "Pip", "Pip and Moss", "Pip, Moss and Fern".
fn listed(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// "1st", "2nd", "3rd".
fn place(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// How high the puck went, as the striker's marks put it: "the 7th mark", or "the bell".
pub(super) fn reached(height: f32) -> String {
    match high_striker::mark(height) {
        10 => "the bell".to_owned(),
        mark => format!("the {} mark", place(mark as usize)),
    }
}

/// A race's finish order as the tray shows it: "1st Tansy 0:11 · 2nd Button 0:12".
fn finish_order(cast: &Cast, results: &[(Id, f32)]) -> String {
    results
        .iter()
        .enumerate()
        .map(|(index, (id, took))| {
            format!(
                "{} {} {}",
                place(index + 1),
                name(cast, Some(*id)),
                clock(*took)
            )
        })
        .collect::<Vec<_>>()
        .join("  \u{b7}  ")
}

/// The striker's ranking as the tray shows it: "1st Fig, the bell · 2nd Biscuit, the 8th mark".
fn standings(cast: &Cast, ranking: &[(Id, f32)]) -> String {
    ranking
        .iter()
        .enumerate()
        .map(|(index, (id, height))| {
            format!(
                "{} {}, {}",
                place(index + 1),
                name(cast, Some(*id)),
                reached(*height)
            )
        })
        .collect::<Vec<_>>()
        .join("  \u{b7}  ")
}

/// Hoopla's tally as the tray shows it: "1st Fig, 3 rings · 2nd Biscuit, 1 ring".
fn rung(cast: &Cast, tally: &[(Id, u32)]) -> String {
    tally
        .iter()
        .enumerate()
        .map(|(index, (id, rings))| {
            format!(
                "{} {}, {}",
                place(index + 1),
                name(cast, Some(*id)),
                rings_rung(*rings)
            )
        })
        .collect::<Vec<_>>()
        .join("  \u{b7}  ")
}

/// "1 ring", "3 rings", "no rings".
pub(super) fn rings_rung(rings: u32) -> String {
    match rings {
        0 => "no rings".to_owned(),
        1 => "1 ring".to_owned(),
        _ => format!("{rings} rings"),
    }
}

/// The two sides of a tug-of-war as the tray says them: "Fig, Moss and Pip against Button, Tansy
/// and Biscuit".
fn against(cast: &Cast, (left, right): &(Vec<Id>, Vec<Id>)) -> String {
    let names = |ids: &[Id]| -> String {
        let names: Vec<String> = ids.iter().map(|id| name(cast, Some(*id))).collect();
        listed(&names)
    };
    format!("{} against {}", names(left), names(right))
}

/// A side of the rope as the tray calls it, by whoever anchors it at the back: "Fig's side".
fn side_of(cast: &Cast, sides: &(Vec<Id>, Vec<Id>), side: Side) -> String {
    let ids = match side {
        Side::Left => &sides.0,
        Side::Right => &sides.1,
    };
    format!("{}'s side", name(cast, ids.last().copied()))
}

/// Something to tell the person: a line as it is, or a game seen through, to be remembered first.
enum Told {
    Line(String),
    AllFound {
        it: Option<Id>,
        took: f32,
    },
    AllHome {
        results: Vec<(Id, f32)>,
        took: f32,
    },
    AllDone {
        ranking: Vec<(Id, f32)>,
    },
    AllThrown {
        tally: Vec<(Id, u32)>,
    },
    Pulled {
        winners: Vec<Id>,
        losers: Vec<Id>,
        took: f32,
    },
}

/// What has happened in the games since last asked, in the person's words; and how it sounded,
/// each cue with how loud, into `heard`.
fn happenings(
    games: &mut fairground::Games,
    cast: &Cast,
    heard: &mut Vec<(Cue, f32)>,
) -> Vec<Told> {
    let who = |id: Id| name(cast, Some(id));
    let mut told = Vec::new();
    let it = games.hide_and_seek.seeker();
    for event in games.hide_and_seek.take_events() {
        match event {
            Event::Coming => heard.push((Cue::ReadyOrNot, 1.0)),
            Event::Found { .. } => heard.push((Cue::Found, 1.0)),
            _ => {}
        }
        let place_name = |place| games.hide_and_seek.place_name(place);
        told.push(match event {
            Event::Dozed => Told::Line(format!("{} has nodded off mid-count.", name(cast, it))),
            Event::Coming => Told::Line(format!(
                "\u{201c}Ready or not, here I come!\u{201d} calls {}.",
                name(cast, it)
            )),
            Event::Noticed { place } => Told::Line(format!(
                "{} heard something by {}.",
                name(cast, it),
                place_name(place)
            )),
            Event::Nobody { place } => Told::Line(format!("Nobody behind {}.", place_name(place))),
            Event::Found { hider, place } => Told::Line(format!(
                "{} found {} behind {}!",
                name(cast, it),
                who(hider),
                place_name(place)
            )),
            Event::AllFound { took } => Told::AllFound { it, took },
        });
    }
    for event in games.sack_race.take_events() {
        use sack_race::Event;
        if event == Event::Go {
            heard.push((Cue::StartWhistle, 1.0));
        }
        told.push(match event {
            Event::Steady => Told::Line("\u{201c}Ready, steady\u{2026}\u{201d}".to_owned()),
            Event::FalseStart { racer } => Told::Line(format!(
                "False start! {} was off before \u{201c}go\u{201d}. Everyone back to the line.",
                who(racer)
            )),
            Event::Go => Told::Line("\u{201c}Go!\u{201d}".to_owned()),
            Event::Lead { racer } => Told::Line(format!("{} takes the lead!", who(racer))),
            Event::Tumble { racer } => Told::Line(format!("{} takes a tumble!", who(racer))),
            Event::Breather { racer } => {
                Told::Line(format!("{} sits down for a breather.", who(racer)))
            }
            Event::Waving { racer } => {
                Told::Line(format!("{} stops to wave to the crowd\u{2026}", who(racer)))
            }
            Event::HelpedUp { helper, racer } => {
                Told::Line(format!("{} stops to help {} up.", who(helper), who(racer)))
            }
            Event::Glare { racer, rival } => Told::Line(format!(
                "{} and {} glare at each other as they pass.",
                who(racer),
                who(rival)
            )),
            Event::Finished {
                racer,
                place: 1,
                took,
            } => Told::Line(format!("{} wins in {}!", who(racer), clock(took))),
            Event::Finished { .. } => continue,
            Event::AllHome { took } => Told::AllHome {
                results: games.sack_race.results().to_vec(),
                took,
            },
        });
    }
    for event in games.high_striker.take_events() {
        use high_striker::Event;
        match event {
            Event::Swung { .. } => heard.push((Cue::Strike, 1.0)),
            // A half-hearted tap is heard as one.
            Event::Tap { .. } => heard.push((Cue::Strike, 0.35)),
            Event::Rang { .. } => heard.push((Cue::Bell, 1.0)),
            _ => {}
        }
        told.push(match event {
            Event::Up {
                player,
                helper: Some(helper),
            } => Told::Line(format!(
                "{} steps up to the striker, with {} to help hold the mallet.",
                who(player),
                who(helper)
            )),
            Event::Up { player, .. } => {
                Told::Line(format!("{} steps up to the striker.", who(player)))
            }
            Event::Flourish { player } => {
                Told::Line(format!("{} does a flourish first.", who(player)))
            }
            Event::Squint { player } => {
                Told::Line(format!("{} squints up at the marks.", who(player)))
            }
            Event::Tap { player } => {
                Told::Line(format!("{} gives it a half-hearted tap.", who(player)))
            }
            Event::Swung { height, .. } if height >= 1.0 => continue,
            Event::Swung {
                player,
                height,
                early,
                ..
            } => Told::Line(if early {
                format!(
                    "{} swings too soon: up to {}.",
                    who(player),
                    reached(height)
                )
            } else {
                format!("{} sends the puck up to {}.", who(player), reached(height))
            }),
            Event::Rang {
                player,
                helper: Some(helper),
            } => Told::Line(format!(
                "{} and {} ring the bell together!",
                who(player),
                who(helper)
            )),
            Event::Rang { player, .. } => Told::Line(format!("{} rings the bell!", who(player))),
            Event::AllDone => Told::AllDone {
                ranking: games.high_striker.ranking(),
            },
        });
    }
    for event in games.hoopla.take_events() {
        use hoopla::{Event, Landing};
        if let Event::Landed { landing, .. } = event {
            match landing {
                Landing::Rung { .. } => heard.extend([(Cue::Clink, 1.0), (Cue::Prize, 1.0)]),
                Landing::Peg => heard.push((Cue::Clink, 1.0)),
                // Into the sawdust, without a sound.
                Landing::Short => {}
            }
        }
        told.push(match event {
            Event::Up { player, near: true } => Told::Line(format!(
                "{} takes its rings, and gets to throw from the nearer line.",
                who(player)
            )),
            Event::Up { player, .. } => Told::Line(format!(
                "{} takes its three rings to the line.",
                who(player)
            )),
            Event::Measures { player } => Told::Line(format!(
                "{} steadies itself and measures each throw.",
                who(player)
            )),
            Event::BehindTheBack { player } => Told::Line(format!(
                "{} turns its back to throw one over its shoulder\u{2026}",
                who(player)
            )),
            Event::Landed {
                player,
                landing: Landing::Rung { prize },
                behind_back: true,
                ..
            } => Told::Line(format!(
                "Behind its back, and {} rings {}! Spectacular!",
                who(player),
                prize.name()
            )),
            Event::Landed {
                player,
                landing: Landing::Rung { prize },
                ..
            } => Told::Line(format!("{} rings {}!", who(player), prize.name())),
            Event::Landed {
                player,
                landing: Landing::Peg,
                ..
            } => Told::Line(format!("Clink! {}'s ring bounces off a peg.", who(player))),
            Event::Landed { player, .. } => {
                Told::Line(format!("{}'s ring falls short.", who(player)))
            }
            Event::Gave { from, to, prize } => Told::Line(format!(
                "{} gives {} to {}.",
                who(from),
                prize.name(),
                who(to)
            )),
            Event::TurnOver { rings: 0, .. } => continue,
            Event::TurnOver { player, rings } => Told::Line(match rings {
                1 => format!("{} rang one of three.", who(player)),
                2 => format!("{} rang two of three.", who(player)),
                _ => format!("{} rang all three!", who(player)),
            }),
            Event::AllDone => Told::AllThrown {
                tally: games.hoopla.tally(),
            },
        });
    }
    let sides = games.tug_of_war.sides();
    for event in games.tug_of_war.take_events() {
        use tug_of_war::Event;
        if let Event::Won { .. } = event {
            heard.push((Cue::Cheer, 1.0));
        }
        told.push(match event {
            Event::Sides { chosen: true } => {
                Told::Line(format!("{}, as you picked them.", against(cast, &sides)))
            }
            Event::Sides { .. } => Told::Line(format!(
                "{}: the colony has sorted itself.",
                against(cast, &sides)
            )),
            Event::InStep { a, b } => {
                Told::Line(format!("{} and {} pull in rhythm.", who(a), who(b)))
            }
            Event::OutOfStep { a, b } => {
                Told::Line(format!("{} and {} can't keep in step.", who(a), who(b)))
            }
            Event::Beside { parent, little } => Told::Line(format!(
                "{} pulls all the harder beside {}.",
                who(parent),
                who(little)
            )),
            Event::Pull => Told::Line("\u{201c}Pull!\u{201d}".to_owned()),
            Event::Slack { puller } => {
                Told::Line(format!("{} lets the rope go slack\u{2026}", who(puller)))
            }
            Event::Waves { puller } => {
                Told::Line(format!("{} stops to wave to the crowd!", who(puller)))
            }
            Event::Cheers { puller } => Told::Line(format!("{} cheers its side on!", who(puller))),
            Event::Won { side, took } => {
                let (winners, losers) = match side {
                    Side::Left => (sides.0.clone(), sides.1.clone()),
                    Side::Right => (sides.1.clone(), sides.0.clone()),
                };
                Told::Pulled {
                    winners,
                    losers,
                    took,
                }
            }
            Event::AllDone => continue,
        });
    }
    told
}

impl HillApp {
    /// Tells the person what is happening in the game, and remembers how it went once it is over.
    pub(super) fn fairground_events(&mut self, now: f32) {
        let mut heard = Vec::new();
        let told = match &mut self.fairground {
            Some((_, games)) => happenings(games, &self.arrival.cast, &mut heard),
            None => return,
        };
        for (cue, loudness) in heard {
            self.sound.play_softly(cue, loudness);
        }
        let mut last = None;
        for told in told {
            let cast = &self.arrival.cast;
            let who = |id: Id| name(cast, Some(id));
            let line = match told {
                Told::Line(line) => line,
                Told::AllFound { it, took } => {
                    let first = !self.kept(Game::HideAndSeek.souvenir());
                    let quickest = it.is_some_and(|it| self.memories.found_everyone(it, took));
                    let mut line = format!("{} found everyone in {}", name(cast, it), clock(took));
                    if quickest {
                        line.push_str(", the quickest yet");
                    }
                    line.push('!');
                    self.kept_line(&mut line, first, Game::HideAndSeek.souvenir());
                    line
                }
                Told::AllHome { results, took } => {
                    let winner = results.first().map(|(id, _)| *id);
                    let first = !self.kept(Game::SackRace.souvenir());
                    let quicker: Vec<String> =
                        self.memories.raced(&results).into_iter().map(who).collect();
                    let mut line = format!(
                        "Everyone's home! {} won in {}.",
                        name(cast, winner),
                        clock(took)
                    );
                    if !quicker.is_empty() {
                        line.push_str(&format!(" Quickest yet for {}.", listed(&quicker)));
                    }
                    self.kept_line(&mut line, first, Game::SackRace.souvenir());
                    line
                }
                Told::AllDone { ranking } => {
                    let first = !self.kept(Game::HighStriker.souvenir());
                    let higher: Vec<String> = self
                        .memories
                        .struck(&ranking)
                        .into_iter()
                        .map(who)
                        .collect();
                    let mut line = match ranking.first() {
                        Some((top, height)) => format!(
                            "Everyone's had a go. {} went highest: {}!",
                            who(*top),
                            reached(*height)
                        ),
                        None => "Everyone's had a go.".to_owned(),
                    };
                    if !higher.is_empty() {
                        line.push_str(&format!(" Highest yet for {}.", listed(&higher)));
                    }
                    self.kept_line(&mut line, first, Game::HighStriker.souvenir());
                    line
                }
                Told::AllThrown { tally } => {
                    let first = !self.kept(Game::Hoopla.souvenir());
                    let more: Vec<String> =
                        self.memories.hooped(&tally).into_iter().map(who).collect();
                    let mut line = match tally.first() {
                        Some((top, rings)) if *rings > 0 => format!(
                            "Everyone's had their rings. {} rang the most: {}!",
                            who(*top),
                            rings_rung(*rings)
                        ),
                        _ => "Everyone's had their rings, and the prizes are all still there."
                            .to_owned(),
                    };
                    if !more.is_empty() {
                        line.push_str(&format!(" The most yet for {}.", listed(&more)));
                    }
                    self.kept_line(&mut line, first, Game::Hoopla.souvenir());
                    line
                }
                Told::Pulled {
                    winners,
                    losers,
                    took,
                } => {
                    let first = !self.kept(Game::TugOfWar.souvenir());
                    self.memories.tugged(&winners);
                    let names = |ids: &[Id]| {
                        let names: Vec<String> = ids.iter().map(|id| who(*id)).collect();
                        listed(&names)
                    };
                    let mut line = format!(
                        "Over the line! {} win in {}, and into the hay go {}.",
                        names(&winners),
                        clock(took),
                        names(&losers)
                    );
                    self.kept_line(&mut line, first, Game::TugOfWar.souvenir());
                    line
                }
            };
            last = Some(line);
        }
        if let Some(line) = last {
            self.station
                .show_keepsakes(&self.memories.colony().souvenirs);
            self.notice = Some((line, now));
        }
    }

    /// Whether the colony has kept this souvenir already.
    fn kept(&self, souvenir: &str) -> bool {
        self.memories
            .colony()
            .souvenirs
            .iter()
            .any(|kept| kept == souvenir)
    }

    /// Adds what was kept for the display case, if it is the first time.
    fn kept_line(&self, line: &mut String, first: bool, souvenir: &str) {
        if first
            && self.kept(souvenir)
            && let Some(kept) = souvenirs::name(souvenir)
        {
            line.push_str(&format!(" Kept: {}.", kept.to_lowercase()));
        }
    }

    /// The game under way, or the offers, a game to choose and who plays it when nobody is
    /// playing.
    pub(super) fn fairground_tray(&mut self, ui: &mut egui::Ui, now: f32) {
        let race_record = self.race_record();
        let striker_record = self.striker_record();
        let hoopla_record = self.hoopla_record();
        let tug_record = self.tug_record();
        let Some((ground, games)) = &mut self.fairground else {
            return;
        };
        let cast = &self.arrival.cast;
        let mut start = false;
        let mut call_off = false;
        match games.playing() {
            None => {
                tools(ui, &mut self.tool);
                let current = self.game;
                let (_, picked) = paper::menu(
                    ui,
                    paper::Button::new(format!("{} \u{25b4}", current.name())),
                    false,
                    |ui| {
                        let mut picked = None;
                        for game in Game::ALL {
                            let button = paper::Button::new(game.name()).selected(game == current);
                            if ui.add(button).clicked() {
                                picked = Some(game);
                            }
                        }
                        picked
                    },
                );
                if let Some(Some(game)) = picked {
                    self.game = game;
                }
                let travellers = cast.members.len();
                let enough = self.picks.enough(self.game, travellers);
                let why = match self.game.players() {
                    _ if enough => "",
                    _ if travellers < 2 => "It takes two or more travellers to play this.",
                    Players::Sides => "Put someone on each side, or nobody for them to sort.",
                    _ => "Pick two or more to race, or nobody for everyone.",
                };
                let watch = ui.add(paper::Button::new("Watch").enabled(enough));
                start = paper::explain(ui, watch, why).clicked();
                let game = self.game;
                match game.players() {
                    Players::It => {
                        let turn = games
                            .hide_and_seek
                            .next_it()
                            .map_or("Whoever's keenest".to_owned(), |id| {
                                format!("{}'s turn", name(cast, Some(id)))
                            });
                        let mut it = self.picks.of(game).first().copied();
                        let chosen = it.map_or(turn.clone(), |id| name(cast, Some(id)));
                        let (_, pick) = paper::menu(
                            ui,
                            paper::Button::new(format!("It: {chosen} \u{25b4}")),
                            false,
                            |ui| {
                                let mut pick = None;
                                if ui
                                    .add(paper::Button::new(&turn).selected(it.is_none()))
                                    .clicked()
                                {
                                    pick = Some(None);
                                }
                                for member in &cast.members {
                                    let button = paper::Button::new(&member.name)
                                        .selected(it == Some(member.id));
                                    if ui.add(button).clicked() {
                                        pick = Some(Some(member.id));
                                    }
                                }
                                pick
                            },
                        );
                        if let Some(Some(new)) = pick {
                            it = new;
                        }
                        if it != self.picks.of(game).first().copied() {
                            match it {
                                Some(id) => self.picks.toggle(game, id),
                                None => self.picks.clear(game),
                            }
                        }
                        if let Some(best) = it.or(games.hide_and_seek.next_it()).and_then(|id| {
                            self.memories.colony().quickest_seekers.get(&id.to_string())
                        }) {
                            paper::slip(ui, format!("Quickest: {}", clock(*best)));
                        }
                    }
                    Players::Some { .. } => {
                        let picked = self.picks.of(game).to_vec();
                        let who = if picked.is_empty() {
                            match game {
                                Game::HighStriker => "Everyone, keenest first".to_owned(),
                                _ => "Everyone".to_owned(),
                            }
                        } else if picked.len() <= 2 {
                            let names: Vec<String> =
                                picked.iter().map(|id| name(cast, Some(*id))).collect();
                            listed(&names)
                        } else {
                            format!("{} of them", picked.len())
                        };
                        let label = match game {
                            Game::SackRace => "Racing",
                            _ => "Having a go",
                        };
                        let picks = &mut self.picks;
                        paper::menu(
                            ui,
                            paper::Button::new(format!("{label}: {who} \u{25b4}")),
                            true,
                            |ui| {
                                let everyone =
                                    paper::Button::new("Everyone").selected(picked.is_empty());
                                if ui.add(everyone).clicked() {
                                    picks.clear(game);
                                }
                                for member in &cast.members {
                                    let on = picked.contains(&member.id);
                                    if ui
                                        .add(paper::Button::new(&member.name).selected(on))
                                        .clicked()
                                    {
                                        picks.toggle(game, member.id);
                                    }
                                }
                            },
                        );
                        let record = match game {
                            Game::SackRace => race_record
                                .map(|(who, seconds)| format!("Record: {who} {}", clock(seconds))),
                            Game::Hoopla => hoopla_record.map(|(who, rings)| {
                                format!("Most rings: {who}, {}", rings_rung(rings))
                            }),
                            _ => striker_record.map(|(who, height)| {
                                format!("Highest: {who}, {}", reached(height))
                            }),
                        };
                        if let Some(record) = record {
                            paper::slip(ui, record);
                        }
                        let last = match game {
                            Game::SackRace => finish_order(cast, games.sack_race.results()),
                            Game::Hoopla => rung(cast, &games.hoopla.tally()),
                            _ => standings(cast, &games.high_striker.ranking()),
                        };
                        if !last.is_empty() {
                            let slip = paper::slip(ui, "Last time");
                            paper::explain(ui, slip, &last);
                        }
                    }
                    Players::Sides => {
                        let (left, right) = self.picks.sides();
                        let chosen = if left.is_empty() && right.is_empty() {
                            "the colony sorts itself".to_owned()
                        } else {
                            format!("{} against {}", left.len(), right.len())
                        };
                        let sorted = {
                            let everyone: Vec<(Id, crate::character::Character)> = cast
                                .members
                                .iter()
                                .map(|m| (m.id, crate::character::Character::of(m)))
                                .collect();
                            tug_of_war::sort(cast, &everyone)
                        };
                        let picks = &mut self.picks;
                        let (sides, _) = paper::menu(
                            ui,
                            paper::Button::new(format!("Sides: {chosen} \u{25b4}")),
                            true,
                            |ui| {
                                let sorting = left.is_empty() && right.is_empty();
                                let sort = ui.add(
                                    paper::Button::new("Let the colony sort itself")
                                        .selected(sorting),
                                );
                                if paper::explain(ui, sort, &against(cast, &sorted)).clicked() {
                                    picks.unsort();
                                }
                                egui::Grid::new("sides-grid").show(ui, |ui| {
                                    for member in &cast.members {
                                        paper::words(ui, &member.name);
                                        let now_on = picks.side(member.id);
                                        for (label, side) in [
                                            ("Left", Some(Side::Left)),
                                            ("Right", Some(Side::Right)),
                                            ("Out", None),
                                        ] {
                                            let button =
                                                paper::Button::new(label).selected(now_on == side);
                                            if ui.add(button).clicked() {
                                                picks.put(member.id, side);
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                            },
                        );
                        let about = if left.is_empty() && right.is_empty() {
                            against(cast, &sorted)
                        } else {
                            against(cast, &(left.clone(), right.clone()))
                        };
                        paper::explain(ui, sides, &about);
                        if let Some((who, wins)) = &tug_record {
                            paper::slip(ui, format!("Most wins: {who}, {wins}"));
                        }
                        if let Some(winners) = games.tug_of_war.winners() {
                            let sides = games.tug_of_war.sides();
                            let slip = paper::slip(ui, "Last time");
                            paper::explain(
                                ui,
                                slip,
                                &format!(
                                    "{} won: {}",
                                    side_of(cast, &sides, winners),
                                    against(cast, &sides)
                                ),
                            );
                        }
                    }
                }
            }
            Some(Game::TugOfWar) => {
                let tug = &games.tug_of_war;
                let sides = tug.sides();
                let status = match tug.phase() {
                    tug_of_war::Phase::TakingUp { .. } => {
                        format!("{}, taking up the rope\u{2026}", against(cast, &sides))
                    }
                    tug_of_war::Phase::Strain { .. } => "Take the strain\u{2026}".to_owned(),
                    tug_of_war::Phase::Pulling { .. } => {
                        let ribbon = tug.ribbon();
                        let way = if ribbon.abs() < 2.0 {
                            "level".to_owned()
                        } else {
                            let side = if ribbon > 0.0 {
                                Side::Right
                            } else {
                                Side::Left
                            };
                            format!("going {}'s way", side_of(cast, &sides, side))
                        };
                        format!(
                            "The tug-of-war  \u{b7}  {}  \u{b7}  {way}",
                            clock(tug.pulling_for(now))
                        )
                    }
                    tug_of_war::Phase::Tumbling { winners, .. }
                    | tug_of_war::Phase::Over { winners, .. } => {
                        format!("{} won!", side_of(cast, &sides, winners))
                    }
                    tug_of_war::Phase::Ready => String::new(),
                };
                paper::slip(ui, status);
                if matches!(
                    tug.phase(),
                    tug_of_war::Phase::TakingUp { .. }
                        | tug_of_war::Phase::Strain { .. }
                        | tug_of_war::Phase::Pulling { .. }
                ) {
                    call_off = paper::button(ui, "Call it off").clicked();
                }
            }
            Some(Game::HideAndSeek) => match games.hide_and_seek.phase() {
                Phase::Counting { .. } => {
                    paper::slip(
                        ui,
                        format!(
                            "{} is counting. Everyone's hiding!",
                            name(cast, games.hide_and_seek.seeker())
                        ),
                    );
                    call_off = paper::button(ui, "Call it off").clicked();
                }
                Phase::Seeking { .. } => {
                    let (found, of) = games.hide_and_seek.tally();
                    paper::slip(
                        ui,
                        format!(
                            "{} is looking  \u{b7}  found {found} of {of}  \u{b7}  {}",
                            name(cast, games.hide_and_seek.seeker()),
                            clock(games.hide_and_seek.searching_for(now))
                        ),
                    );
                    call_off = paper::button(ui, "Call it off").clicked();
                }
                Phase::Ready | Phase::Over { .. } => {}
            },
            Some(Game::SackRace) => {
                let race = &games.sack_race;
                let status = match race.phase() {
                    sack_race::Phase::LiningUp { .. } => {
                        "Into the sacks, and up to the line\u{2026}".to_owned()
                    }
                    sack_race::Phase::Steady { .. } => "Ready, steady\u{2026}".to_owned(),
                    sack_race::Phase::Racing { .. } => {
                        let mut status =
                            format!("The sack race  \u{b7}  {}", clock(race.racing_for(now)));
                        if race.results().is_empty() {
                            if let Some(leader) = race.leader() {
                                status.push_str(&format!(
                                    "  \u{b7}  {} in front",
                                    name(cast, Some(leader))
                                ));
                            }
                        } else {
                            status.push_str(&format!(
                                "  \u{b7}  {}",
                                finish_order(cast, race.results())
                            ));
                        }
                        status
                    }
                    sack_race::Phase::Over { .. } | sack_race::Phase::Ready => {
                        finish_order(cast, race.results())
                    }
                };
                paper::slip(ui, status);
                if !matches!(race.phase(), sack_race::Phase::Over { .. }) {
                    call_off = paper::button(ui, "Call it off").clicked();
                }
            }
            Some(Game::HighStriker) => {
                let striker = &games.high_striker;
                match striker.phase() {
                    high_striker::Phase::Playing { .. } => {
                        let (done, of) = striker.tally();
                        let mut status = match striker.player() {
                            Some((player, Some(helper))) => format!(
                                "{} is having a go, with {}",
                                name(cast, Some(player)),
                                name(cast, Some(helper))
                            ),
                            Some((player, None)) => {
                                format!("{} is having a go", name(cast, Some(player)))
                            }
                            None => String::new(),
                        };
                        status.push_str(&format!("  \u{b7}  {} of {of}", done + 1));
                        if let Some((top, height)) = striker.ranking().first() {
                            status.push_str(&format!(
                                "  \u{b7}  highest so far: {}, {}",
                                name(cast, Some(*top)),
                                reached(*height)
                            ));
                        }
                        paper::slip(ui, status);
                        call_off = paper::button(ui, "Call it off").clicked();
                    }
                    _ => {
                        paper::slip(ui, standings(cast, &striker.ranking()));
                    }
                }
            }
            Some(Game::Hoopla) => {
                let stall = &games.hoopla;
                match stall.phase() {
                    hoopla::Phase::Playing { .. } => {
                        let (done, of) = stall.progress();
                        let mut status = stall.player().map_or(String::new(), |player| {
                            format!("{} is throwing", name(cast, Some(player)))
                        });
                        status.push_str(&format!("  \u{b7}  {} of {of}", done + 1));
                        if let Some((top, rings)) =
                            stall.tally().first().filter(|(_, rings)| *rings > 0)
                        {
                            status.push_str(&format!(
                                "  \u{b7}  most so far: {}, {}",
                                name(cast, Some(*top)),
                                rings_rung(*rings)
                            ));
                        }
                        paper::slip(ui, status);
                        call_off = paper::button(ui, "Call it off").clicked();
                    }
                    _ => {
                        paper::slip(ui, rung(cast, &stall.tally()));
                    }
                }
            }
        }
        if start {
            if self.game == Game::TugOfWar {
                games.start_tug(ground, cast, self.picks.sides(), now);
            } else {
                let picked = self.picks.of(self.game).to_vec();
                games.start(self.game, ground, cast, &picked, now);
            }
            self.picks.used(self.game);
        }
        if call_off {
            games.stop(ground, now);
        }
    }

    /// The quickest sack race among those here: who, and in how long.
    pub(super) fn race_record(&self) -> Option<(String, f32)> {
        let colony = self.memories.colony();
        self.arrival
            .cast
            .members
            .iter()
            .filter_map(|member| {
                let best = colony.quickest_racers.get(&member.id.to_string())?;
                Some((member.name.clone(), *best))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The most rings rung in one turn at hoopla among those here: who, and how many.
    pub(super) fn hoopla_record(&self) -> Option<(String, u32)> {
        let colony = self.memories.colony();
        self.arrival
            .cast
            .members
            .iter()
            .filter_map(|member| {
                let best = colony.most_rings.get(&member.id.to_string())?;
                Some((member.name.clone(), *best))
            })
            .filter(|(_, rings)| *rings > 0)
            .max_by_key(|(_, rings)| *rings)
    }

    /// The most tug-of-war wins among those here: who, and how many.
    pub(super) fn tug_record(&self) -> Option<(String, u32)> {
        let colony = self.memories.colony();
        self.arrival
            .cast
            .members
            .iter()
            .filter_map(|member| {
                let wins = colony.tug_wins.get(&member.id.to_string())?;
                Some((member.name.clone(), *wins))
            })
            .max_by_key(|(_, wins)| *wins)
    }

    /// The highest swing at the striker among those here: who, and how high.
    pub(super) fn striker_record(&self) -> Option<(String, f32)> {
        let colony = self.memories.colony();
        self.arrival
            .cast
            .members
            .iter()
            .filter_map(|member| {
                let best = colony.highest_strikes.get(&member.id.to_string())?;
                Some((member.name.clone(), *best))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_race_finish_and_the_strikers_marks_read_as_spoken() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (a, b) = (cast.members[0].id, cast.members[1].id);
        let order = finish_order(&cast, &[(a, 11.4), (b, 72.0)]);
        assert_eq!(
            order,
            format!(
                "1st {} 0:11  \u{b7}  2nd {} 1:12",
                cast.members[0].name, cast.members[1].name
            )
        );
        assert_eq!(reached(1.0), "the bell");
        assert_eq!(reached(0.84), "the 8th mark");
        assert_eq!(reached(0.11), "the 1st mark");
        assert_eq!(reached(0.02), "the 1st mark");
        assert_eq!(
            listed(&["Pip".into(), "Moss".into(), "Fern".into()]),
            "Pip, Moss and Fern"
        );
    }
}
