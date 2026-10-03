//! The Clubhouse in the window: the notice board of stories, a story played out by the fire, and
//! the words of it in the bottom bar.

use super::{HillApp, INK, PAPER, tools};
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
                &self.memories.colony().hilltop,
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
                self.board = false;
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

    pub(super) fn clubhouse_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        if self.story.is_some() {
            self.story_panel(ui);
            return;
        }
        tools(ui, &mut self.tool);
        ui.separator();
        if ui
            .selectable_label(self.board, "The notice board")
            .on_hover_text("The stories pinned up in the Clubhouse")
            .clicked()
        {
            self.board = !self.board;
        }
        if let Some((notice, _)) = &self.notice {
            ui.label(egui::RichText::new(notice).italics());
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            self.go_menu(ui, now);
        });
    }

    /// The notice board, read close up: every story, and a way to start one.
    pub(super) fn board_window(&mut self, ctx: &egui::Context, now: f32) {
        if !self.board || self.area != super::Area::Clubhouse || self.story.is_some() {
            return;
        }
        let travellers = self.arrival.cast.members.len();
        let mut start = None;
        let mut open = true;
        let mut open_shelf = false;
        egui::Window::new("The notice board")
            .open(&mut open)
            .default_width(300.0)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Stories to play out by the fire. \u{2605} once finished.")
                        .italics(),
                );
                ui.add_space(4.0);
                for (package, story) in self.on_the_board() {
                    let finished = self
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
                    let button = ui
                        .add_enabled(fits, egui::Button::new(label))
                        .on_disabled_hover_text(format!(
                            "Needs at least {} travellers",
                            story.min_cast
                        ));
                    // A community story says whose it is.
                    let button = if self.library.origin(&package.id) == Some(Origin::Official) {
                        button
                    } else {
                        button.on_hover_text(format!("By {}", package.author))
                    };
                    if button.clicked() {
                        start = Some((package.id.clone(), story.id.clone()));
                    }
                }
                let kept = &self.memories.colony().souvenirs;
                if !kept.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new("Kept").strong());
                    for id in kept {
                        if let Some(name) = souvenirs::name(id) {
                            ui.label(name);
                        }
                    }
                }
                ui.separator();
                if ui.small_button("Story packages\u{2026}").clicked() {
                    open_shelf = true;
                }
            });
        if open_shelf {
            self.shelf = true;
        }
        if !open {
            self.board = false;
        }
        if let Some((package, story)) = start {
            self.start_story(&package, &story, now);
        }
    }

    /// Every package Hill found, where it came from, and a way to set any but Hill's own aside;
    /// then any that would not load, and why, and where community packages go.
    pub(super) fn shelf_window(&mut self, ctx: &egui::Context) {
        if !self.shelf {
            return;
        }
        let mut open = true;
        let mut changes = Vec::new();
        egui::Window::new("Story packages")
            .open(&mut open)
            .default_width(340.0)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for (package, origin) in
                            self.library.packages.iter().zip(&self.library.origins)
                        {
                            let stories = package.stories.len();
                            let about = format!(
                                "by {} \u{b7} version {} \u{b7} {stories} {}",
                                package.author,
                                package.version,
                                if stories == 1 { "story" } else { "stories" }
                            );
                            ui.horizontal(|ui| {
                                if *origin == Origin::Official {
                                    ui.label(egui::RichText::new(&package.title).strong());
                                    ui.label(egui::RichText::new("Hill's own").small().weak());
                                } else {
                                    let mut on = !self.set_aside.contains(&package.id);
                                    if ui
                                        .checkbox(
                                            &mut on,
                                            egui::RichText::new(&package.title).strong(),
                                        )
                                        .on_hover_text(if on {
                                            "On the notice board. Untick to set it aside."
                                        } else {
                                            "Set aside: its stories are off the notice board."
                                        })
                                        .changed()
                                    {
                                        changes.push((package.id.clone(), !on));
                                    }
                                    if *origin == Origin::Named {
                                        ui.label(
                                            egui::RichText::new("named to try").small().weak(),
                                        );
                                    }
                                }
                            });
                            ui.label(egui::RichText::new(about).small());
                            ui.add_space(4.0);
                        }
                        if !self.library.problems.is_empty() {
                            ui.separator();
                            ui.label(egui::RichText::new("Would not load").strong());
                            for problem in &self.library.problems {
                                ui.label(egui::RichText::new(problem.to_string()).small());
                            }
                        }
                    });
                ui.separator();
                match &self.packages_folder {
                    Some(folder) => {
                        ui.label(
                            egui::RichText::new(
                                "To add a story, put its package folder here and come back to \
                                 the Hill:",
                            )
                            .small(),
                        );
                        ui.add(egui::Label::new(
                            egui::RichText::new(folder.display().to_string())
                                .small()
                                .monospace(),
                        ));
                    }
                    None => {
                        ui.label(
                            egui::RichText::new(
                                "There is nowhere to keep packages on this computer.",
                            )
                            .small(),
                        );
                    }
                }
            });
        if !open {
            self.shelf = false;
        }
        for (id, aside) in changes {
            self.set_aside.set(&id, aside);
        }
        let pinned = self.pinned();
        if let Some(room) = &mut self.clubhouse {
            room.pin_up(pinned);
        }
    }

    /// The story's words: who is speaking and what they say, or the choice to make.
    fn story_panel(&mut self, ui: &mut egui::Ui) {
        let Some((_, director)) = &mut self.story else {
            return;
        };
        let mut leave = false;
        egui::Frame::new()
            .fill(PAPER)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(director.title())
                            .italics()
                            .color(INK)
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        leave = ui.small_button("Leave the story").clicked();
                    });
                });
                if let Some(shown) = director.shown().cloned() {
                    if let Some((_, name)) = &shown.speaker {
                        ui.label(egui::RichText::new(name).strong().color(INK));
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(&shown.text).color(INK).size(16.0));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Next  \u{25b8}").clicked() {
                            director.read_on();
                        }
                    });
                } else if !director.choices().is_empty() {
                    let mut chosen = None;
                    ui.horizontal_wrapped(|ui| {
                        for (index, choice) in director.choices().into_iter().enumerate() {
                            if ui.button(format!("{}  {choice}", index + 1)).clicked() {
                                chosen = Some(index);
                            }
                        }
                    });
                    if let Some(index) = chosen {
                        director.choose(index);
                    }
                } else {
                    ui.label(egui::RichText::new("\u{2026}").color(INK));
                }
            });
        if leave {
            self.leave_story();
        }
    }
}
