//! The Clubhouse in the window: the notice board of stories, and a story played out by the fire,
//! its words in a box over whoever says them.

use super::{Area, Card, HillApp, paper, tools};
use crate::clubhouse::Clubhouse;
use crate::story::{Director, Origin, Package, Story, souvenirs};
use eframe::egui;

impl HillApp {
    /// A card on the board for each story, starred where the colony has finished it.
    fn pinned(&self) -> Vec<bool> {
        let finished = &self.memories.colony().stories;
        self.on_the_board()
            .map(|(package, story)| finished.contains(&format!("{}/{}", package.id, story.id)))
            .collect()
    }

    /// The stories pinned up: every one, but for those in packages the person has set aside.
    pub(super) fn on_the_board(&self) -> impl Iterator<Item = (&Package, &Story)> {
        self.library
            .stories()
            .filter(|(package, _)| !self.set_aside.contains(&package.id))
    }

    /// Readies the room the first time anyone goes in.
    pub(super) fn open_clubhouse(&mut self, now: f32) {
        if self.clubhouse.is_none() {
            self.clubhouse = Some(Clubhouse::open(
                &self.arrival.cast,
                now,
                &self.hilltop_standing(),
                self.pinned(),
            ));
        }
    }

    pub(super) fn start_story(&mut self, package: &str, story: &str, now: f32) {
        let Some(room) = &mut self.clubhouse else {
            return;
        };
        let Some((_, chosen)) = self
            .library
            .stories()
            .find(|(p, s)| p.id == package && s.id == story)
        else {
            return;
        };
        // The same colony always casts a story the same way.
        let seed = self
            .arrival
            .cast
            .snapshot
            .colony_id
            .bytes()
            .chain(story.bytes())
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        match Director::new(chosen.clone(), &self.arrival.cast, seed) {
            Ok(director) => {
                room.ground().reserve(director.players());
                self.story = Some((package.to_owned(), director));
                self.card = None;
            }
            Err(problem) => self.notice = Some((problem, now)),
        }
    }

    /// Plays the story on, and settles it once it ends.
    pub(super) fn run_story(&mut self, now: f32) {
        let (Some(room), Some((package, director))) = (&mut self.clubhouse, &mut self.story) else {
            return;
        };
        let ground = room.ground();
        director.run(ground, &self.arrival.cast, now);
        ground.set_speaker(
            director
                .shown()
                .and_then(|shown| shown.speaker.as_ref().map(|(id, _)| *id)),
        );
        if director.finished() {
            ground.release();
            let kept: Vec<&str> = director
                .souvenirs()
                .iter()
                .filter_map(|id| souvenirs::name(id))
                .collect();
            self.notice = Some((
                if kept.is_empty() {
                    format!("The end of \u{201c}{}\u{201d}.", director.title())
                } else {
                    format!(
                        "The end of \u{201c}{}\u{201d}. Kept: {}.",
                        director.title(),
                        kept.join(", ").to_lowercase()
                    )
                },
                now,
            ));
            let finished = format!("{package}/{}", director.story_id());
            self.memories.finished(finished, director.souvenirs());
            self.station
                .show_keepsakes(&self.memories.colony().souvenirs);
            self.story = None;
            let pinned = self.pinned();
            if let Some(room) = &mut self.clubhouse {
                room.pin_up(pinned);
            }
        }
    }

    pub(super) fn leave_story(&mut self) {
        if let Some(room) = &mut self.clubhouse {
            room.ground().release();
        }
        self.story = None;
    }

    pub(super) fn clubhouse_tray(&mut self, ui: &mut egui::Ui, _now: f32) {
        if self.story.is_some() {
            self.story_tray(ui);
            return;
        }
        tools(ui, &mut self.tool);
        let board =
            ui.add(paper::Button::new("The notice board").selected(self.showing(Card::Board)));
        if paper::explain(ui, board, "The stories pinned up in the Clubhouse").clicked() {
            self.toggle(Card::Board);
        }
    }

    /// The notice board, read close up: every story, and a way to start one.
    pub(super) fn board_window(&mut self, ctx: &egui::Context, now: f32) {
        if self.area != Area::Clubhouse || self.story.is_some() {
            return;
        }
        let travellers = self.arrival.cast.members.len();
        let mut start = None;
        let mut open_shelf = false;
        self.show_card(ctx, Card::Board, "The notice board", 0.42, |app, ui| {
            paper::aside(
                ui,
                "Stories to play out by the fire. \u{2605} once finished.",
            );
            for (package, story) in app.on_the_board() {
                let finished = app
                    .memories
                    .colony()
                    .stories
                    .contains(&format!("{}/{}", package.id, story.id));
                let label = if finished {
                    format!("{}  \u{2605}", story.title)
                } else {
                    story.title.clone()
                };
                let fits = travellers >= story.min_cast;
                let button = ui.add(paper::Button::new(label).enabled(fits));
                // A community story says whose it is; one too big for the visit says so.
                let about = if !fits {
                    format!("Needs at least {} travellers", story.min_cast)
                } else if app.library.origin(&package.id) == Some(Origin::Official) {
                    String::new()
                } else {
                    format!("By {}", package.author)
                };
                if paper::explain(ui, button, &about).clicked() {
                    start = Some((package.id.clone(), story.id.clone()));
                }
            }
            let kept = &app.memories.colony().souvenirs;
            if !kept.is_empty() {
                paper::heading(ui, "Kept");
                for id in kept {
                    if let Some(name) = souvenirs::name(id) {
                        paper::words(ui, name);
                    }
                }
            }
            paper::rule(ui);
            if paper::button(ui, "Story packages\u{2026}").clicked() {
                open_shelf = true;
            }
        });
        if open_shelf {
            self.card = Some(Card::Shelf);
        }
        if let Some((package, story)) = start {
            self.start_story(&package, &story, now);
        }
    }

    /// Every package Hill found, where it came from, and a way to set any but Hill's own aside;
    /// then any that would not load, and why, and where community packages go.
    pub(super) fn shelf_window(&mut self, ctx: &egui::Context) {
        let mut changes = Vec::new();
        self.show_card(ctx, Card::Shelf, "Story packages", 0.5, |app, ui| {
            for (package, origin) in app.library.packages.iter().zip(&app.library.origins) {
                let stories = package.stories.len();
                let about = format!(
                    "by {} \u{b7} version {} \u{b7} {stories} {}",
                    package.author,
                    package.version,
                    if stories == 1 { "story" } else { "stories" }
                );
                ui.horizontal_wrapped(|ui| {
                    if *origin == Origin::Official {
                        paper::words(ui, &package.title);
                        paper::faint(ui, "Hill's own");
                    } else {
                        let on = !app.set_aside.contains(&package.id);
                        let toggle = ui.add(
                            paper::Button::new(if on { "On the board" } else { "Set aside" })
                                .selected(on),
                        );
                        let toggle = paper::explain(
                            ui,
                            toggle,
                            if on {
                                "On the notice board. Click to set it aside."
                            } else {
                                "Set aside: its stories are off the notice board."
                            },
                        );
                        if toggle.clicked() {
                            changes.push((package.id.clone(), on));
                        }
                        paper::words(ui, &package.title);
                        if *origin == Origin::Named {
                            paper::faint(ui, "named to try");
                        }
                    }
                });
                paper::aside(ui, about);
            }
            if !app.library.problems.is_empty() {
                paper::heading(ui, "Would not load");
                for problem in &app.library.problems {
                    paper::aside(ui, problem.to_string());
                }
            }
            paper::rule(ui);
            match &app.packages_folder {
                Some(folder) => {
                    paper::aside(
                        ui,
                        "To add a story, put its package folder here and come back to the Hill:",
                    );
                    paper::words(ui, folder.display().to_string());
                }
                None => {
                    paper::aside(ui, "There is nowhere to keep packages on this computer.");
                }
            }
        });
        for (id, aside) in changes {
            self.set_aside.set(&id, aside);
        }
        let pinned = self.pinned();
        if let Some(room) = &mut self.clubhouse {
            room.pin_up(pinned);
        }
    }

    /// The story's controls on the tray: its title, reading on or the choice to make, and the way
    /// out. What is said is over the speaker's head, in `story_speech`.
    fn story_tray(&mut self, ui: &mut egui::Ui) {
        let Some((_, director)) = &mut self.story else {
            return;
        };
        ui.add(
            paper::Words::new(format!("\u{201c}{}\u{201d}", director.title()))
                .slip()
                .color(paper::FADED)
                .most(160),
        );
        if director.shown().is_some() {
            let next = ui.add(paper::Button::new("Next \u{25b8}"));
            if paper::explain(ui, next, "Or click the scene, or press Space").clicked() {
                director.read_on();
            }
        } else if !director.choices().is_empty() {
            let mut chosen = None;
            paper::hint(ui, "What happens next?");
            for (index, choice) in director.choices().into_iter().enumerate() {
                if paper::button(ui, format!("{}  {choice}", index + 1)).clicked() {
                    chosen = Some(index);
                }
            }
            if let Some(index) = chosen {
                director.choose(index);
            }
        }
        if paper::button(ui, "Leave the story").clicked() {
            self.leave_story();
        }
    }

    /// What is being said in the story, in a box over the speaker; the story's own telling in a
    /// caption at the top.
    pub(super) fn story_speech(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        scene: egui::Rect,
        now: f32,
    ) {
        let (Some(room), Some((_, director))) = (&mut self.clubhouse, &self.story) else {
            return;
        };
        let Some(shown) = director.shown() else {
            return;
        };
        let point = scene.width() / super::SCENE_WIDTH as f32;
        let (head, name) = match &shown.speaker {
            Some((id, name)) => (
                room.ground()
                    .head(*id, now)
                    .map(|(x, y)| scene.min + egui::vec2(x, y - 2.0) * point),
                Some(name.as_str()),
            ),
            None => (None, None),
        };
        let voice = if shown.speaker.is_some() {
            paper::Voice::Speaker
        } else {
            paper::Voice::Narrator
        };
        paper::speech(ctx, painter, scene, head, name, &shown.text, voice, true);
    }
}
