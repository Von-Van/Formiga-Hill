//! The Fairground's bar: which game to watch and who plays it, the game under way, and what the
//! person is told as it goes; and what Hill remembers of each game once it is seen through.

use super::{HillApp, clock, tools};
use crate::cast::{Cast, Id};
use crate::fairground::{self, Event, Game, Phase, Players, high_striker, sack_race};
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

/// A race's finish order as the bar shows it: "1st Tansy 0:11 · 2nd Button 0:12".
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

/// The striker's ranking as the bar shows it: "1st Fig, the bell · 2nd Biscuit, the 8th mark".
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

/// Something to tell the person: a line as it is, or a game seen through, to be remembered first.
enum Told {
    Line(String),
    AllFound { it: Option<Id>, took: f32 },
    AllHome { results: Vec<(Id, f32)>, took: f32 },
    AllDone { ranking: Vec<(Id, f32)> },
}

/// What has happened in the games since last asked, in the person's words.
fn happenings(games: &mut fairground::Games, cast: &Cast) -> Vec<Told> {
    let who = |id: Id| name(cast, Some(id));
    let mut told = Vec::new();
    let it = games.hide_and_seek.seeker();
    for event in games.hide_and_seek.take_events() {
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
    told
}

impl HillApp {
    /// Tells the person what is happening in the game, and remembers how it went once it is over.
    pub(super) fn fairground_events(&mut self, now: f32) {
        let told = match &mut self.fairground {
            Some((_, games)) => happenings(games, &self.arrival.cast),
            None => return,
        };
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
    pub(super) fn fairground_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let race_record = self.race_record();
        let striker_record = self.striker_record();
        let Some((ground, games)) = &mut self.fairground else {
            return;
        };
        let cast = &self.arrival.cast;
        let mut start = false;
        let mut call_off = false;
        match games.playing() {
            None => {
                tools(ui, &mut self.tool);
                ui.separator();
                egui::ComboBox::from_id_salt("game")
                    .selected_text(self.game.name())
                    .show_ui(ui, |ui| {
                        for game in Game::ALL {
                            ui.selectable_value(&mut self.game, game, game.name());
                        }
                    });
                let enough = self.picks.enough(self.game);
                let watch = ui.add_enabled(enough, egui::Button::new("Watch"));
                start = watch
                    .on_disabled_hover_text("Pick two or more to race, or nobody for everyone.")
                    .clicked();
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
                        egui::ComboBox::from_id_salt("it")
                            .selected_text(format!("It: {chosen}"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut it, None, turn);
                                for member in &cast.members {
                                    ui.selectable_value(&mut it, Some(member.id), &member.name);
                                }
                            });
                        if it != self.picks.of(game).first().copied() {
                            match it {
                                Some(id) => self.picks.toggle(game, id),
                                None => self.picks.clear(game),
                            }
                        }
                        if let Some(best) = it.or(games.hide_and_seek.next_it()).and_then(|id| {
                            self.memories.colony().quickest_seekers.get(&id.to_string())
                        }) {
                            ui.label(format!("Quickest: {}", clock(*best)));
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
                        egui::ComboBox::from_id_salt("players")
                            .selected_text(format!("{label}: {who}"))
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(picked.is_empty(), "Everyone").clicked() {
                                    self.picks.clear(game);
                                }
                                for member in &cast.members {
                                    let mut on = picked.contains(&member.id);
                                    if ui.checkbox(&mut on, &member.name).changed() {
                                        self.picks.toggle(game, member.id);
                                    }
                                }
                            });
                        let record = match game {
                            Game::SackRace => race_record
                                .map(|(who, seconds)| format!("Record: {who} {}", clock(seconds))),
                            _ => striker_record.map(|(who, height)| {
                                format!("Highest: {who}, {}", reached(height))
                            }),
                        };
                        if let Some(record) = record {
                            ui.label(record);
                        }
                        let last = match game {
                            Game::SackRace => finish_order(cast, games.sack_race.results()),
                            _ => standings(cast, &games.high_striker.ranking()),
                        };
                        if !last.is_empty() {
                            ui.label("Last time").on_hover_text(last);
                        }
                    }
                }
            }
            Some(Game::HideAndSeek) => match games.hide_and_seek.phase() {
                Phase::Counting { .. } => {
                    ui.label(format!(
                        "{} is counting. Everyone's hiding!",
                        name(cast, games.hide_and_seek.seeker())
                    ));
                    call_off = ui.button("Call it off").clicked();
                }
                Phase::Seeking { .. } => {
                    let (found, of) = games.hide_and_seek.tally();
                    ui.label(format!(
                        "{} is looking  \u{b7}  found {found} of {of}  \u{b7}  {}",
                        name(cast, games.hide_and_seek.seeker()),
                        clock(games.hide_and_seek.searching_for(now))
                    ));
                    call_off = ui.button("Call it off").clicked();
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
                ui.label(status);
                if !matches!(race.phase(), sack_race::Phase::Over { .. }) {
                    call_off = ui.button("Call it off").clicked();
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
                        ui.label(status);
                        call_off = ui.button("Call it off").clicked();
                    }
                    _ => {
                        ui.label(standings(cast, &striker.ranking()));
                    }
                }
            }
        }
        if start {
            let picked = self.picks.of(self.game).to_vec();
            games.start(self.game, ground, cast, &picked, now);
            self.picks.used(self.game);
        }
        if call_off {
            games.stop(ground, now);
        }
        if let Some((notice, _)) = &self.notice {
            ui.label(egui::RichText::new(notice).italics());
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            self.go_menu(ui, now);
        });
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
