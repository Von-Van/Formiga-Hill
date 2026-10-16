//! What someone else has shared: a shared file opened (or dropped on the window), looked over on
//! a card before anything is kept, a story tried out if the person likes, then added or turned
//! away. Anything added can be removed again from the Clubhouse's Story packages card. The
//! checking, keeping and removing are `crate::sharing`'s, the same for any kind of content Hill
//! hosts, and so is this card: it opens wherever the person is, from any place's own button
//! (`choose_shared_file`), and says what kinds of content a package holds; it shows a story's
//! own details, and offers to try one, because stories are what the Clubhouse stages.

use super::writing::Trying;
use super::{Area, Card, HillApp, paper};
use crate::sharing::{self, Inspection};
use crate::story::script::Action;
use crate::story::{Origin, Story, souvenirs};
use eframe::egui;
use std::path::Path;

/// A shared file opened, and what was found in it.
pub(super) enum Importing {
    /// It loads: what it is, and the version of it already kept, if there is one.
    Looked(Box<Inspection>, Option<String>),
    /// It can't be added, and why, in plain words.
    Refused(String, String),
}

impl HillApp {
    /// Asks which shared file to open, and looks it over.
    pub(super) fn choose_shared_file(&mut self, now: f32) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Open something shared")
            .add_filter("Shared from Formiga Hill", &[sharing::EXTENSION])
            .pick_file()
        else {
            return;
        };
        self.open_shared_file(&path, now);
    }

    /// Looks a shared file over, and shows what is in it on a card over wherever the person is.
    /// Nothing is kept until the person says so.
    pub(super) fn open_shared_file(&mut self, path: &Path, now: f32) {
        let label = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let looked = sharing::read(path).and_then(|files| sharing::inspect(files, &label));
        self.importing = Some(match looked {
            Ok(inspection) => {
                let id = &inspection.summary.id;
                if id.starts_with(sharing::RESERVED)
                    || self.library.origin(id) == Some(Origin::Official)
                {
                    Importing::Refused(
                        label,
                        "it claims to be one of Hill's own, which are already here".into(),
                    )
                } else {
                    // Whatever loads holds only content this Hill knows how to host.
                    let have = self
                        .library
                        .packages
                        .iter()
                        .find(|package| package.id == *id)
                        .map(|package| package.version.clone());
                    Importing::Looked(Box::new(inspection), have)
                }
            }
            Err(error) => Importing::Refused(label, error.problem),
        });
        if self.story.is_some() {
            self.leave_story();
        }
        let _ = now;
        self.card = Some(Card::Import);
    }

    /// Whatever shared files were dropped on the window, looked over: the first that is one.
    pub(super) fn take_dropped(&mut self, ctx: &egui::Context, now: f32) {
        let dropped: Vec<std::path::PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_owned())
                .collect()
        });
        if let Some(path) = dropped.iter().find(|path| {
            path.extension()
                .is_some_and(|extension| extension == sharing::EXTENSION)
        }) {
            self.open_shared_file(path, now);
        } else if !dropped.is_empty() {
            self.notice = Some((
                format!(
                    "Hill opens what is shared as .{} files.",
                    sharing::EXTENSION
                ),
                now,
            ));
        }
    }

    /// The shared package, looked over: whose it is, what is in it, what it can give, and what it
    /// can't do; then a way to try a story, add it, or turn it away.
    pub(super) fn import_window(&mut self, ctx: &egui::Context, now: f32) {
        if self.story.is_some() {
            return;
        }
        if !self.showing(Card::Import) {
            return;
        }
        let mut add = false;
        let mut try_it = false;
        let mut done = false;
        self.show_card(ctx, Card::Import, "Shared with you", 0.5, |app, ui| {
            match &app.importing {
                None => done = true,
                Some(Importing::Refused(file, problem)) => {
                    paper::name(ui, file);
                    paper::words(ui, "This file can't be added.");
                    paper::aside(ui, capitalised(problem));
                    paper::rule(ui);
                    done = paper::button(ui, "Close").clicked();
                }
                Some(Importing::Looked(inspection, have)) => {
                    let summary = &inspection.summary;
                    paper::name(ui, &summary.title);
                    paper::aside(
                        ui,
                        format!("by {} \u{b7} version {}", summary.author, summary.version),
                    );
                    if let Some(have) = have {
                        paper::words(
                            ui,
                            format!(
                                "You have version {have} of it already: adding this one takes its place."
                            ),
                        );
                    }
                    let others: Vec<&str> = summary
                        .content_types
                        .iter()
                        .map(String::as_str)
                        .filter(|kind| !matches!(*kind, "story" | "adventure"))
                        .collect();
                    if !others.is_empty() {
                        paper::words(ui, format!("It holds: {}.", others.join(", ")));
                    }
                    let stories = &inspection.package.stories;
                    for story in stories {
                        // A story called what its package is needs no second heading.
                        let named = stories.len() > 1 || story.title != summary.title;
                        describe(ui, story, named);
                    }
                    for adventure in &inspection.package.adventures {
                        super::adventures::describe(ui, adventure);
                    }
                    paper::rule(ui);
                    paper::faint(
                        ui,
                        "Something shared can only be played at the Hill. It can't run anything, \
                         look at your files, or change your colony.",
                    );
                    ui.horizontal_wrapped(|ui| {
                        if !stories.is_empty() {
                            let first = ui.add(paper::Button::new("Try it first"));
                            try_it = paper::explain(
                                ui,
                                first,
                                "Play the story now in the Clubhouse, with whoever is visiting, \
                                 without adding it. Nothing is kept.",
                            )
                            .clicked();
                        }
                        add = paper::button(ui, "Add it").clicked();
                        done = paper::button(ui, "Don't add it").clicked();
                    });
                }
            }
        });
        if try_it
            && let Some(Importing::Looked(inspection, _)) = &self.importing
            && let Some(story) = inspection.package.stories.first().cloned()
        {
            let id = inspection.summary.id.clone();
            self.try_story(&id, story, Trying::Shared, now);
        }
        if add {
            self.add_shared(now);
        }
        if done {
            self.importing = None;
            self.card = (self.area == Area::Clubhouse).then_some(Card::Shelf);
        }
    }

    fn add_shared(&mut self, now: f32) {
        let Some(Importing::Looked(inspection, _)) = self.importing.take() else {
            return;
        };
        let Some(folder) = self.packages_folder.clone() else {
            self.notice = Some(("There is nowhere to keep it on this computer.".into(), now));
            return;
        };
        let title = inspection.summary.title.clone();
        let stories = !inspection.package.stories.is_empty();
        let here = self.area == Area::Clubhouse;
        match sharing::install(&folder, &inspection.files) {
            Ok(_) => {
                // Added by choice, so it is in play straight away.
                self.set_aside.set(&inspection.summary.id, false);
                self.reload_library();
                self.notice = Some((
                    if stories {
                        format!("\u{201c}{title}\u{201d} is on the Clubhouse's notice board.")
                    } else {
                        format!("\u{201c}{title}\u{201d} is added.")
                    },
                    now,
                ));
                self.card = (here && stories).then_some(Card::Board);
            }
            Err(error) => {
                self.notice = Some((format!("It couldn't be added: {}.", error.problem), now));
                self.card = here.then_some(Card::Shelf);
            }
        }
    }

    /// Takes a package out of the packages folder for good, once the person is sure.
    pub(super) fn remove_package(&mut self, id: &str, now: f32) {
        let Some(folder) = self.packages_folder.clone() else {
            return;
        };
        let title = self
            .library
            .packages
            .iter()
            .find(|package| package.id == id)
            .map_or_else(|| id.to_owned(), |package| package.title.clone());
        let line = match sharing::remove(&folder, id) {
            Ok(()) => {
                self.set_aside.set(id, false);
                self.reload_library();
                format!("\u{201c}{title}\u{201d} is taken out.")
            }
            Err(error) => format!("It couldn't be taken out: {}.", error.problem),
        };
        self.notice = Some((line, now));
    }
}

/// One story in a shared package, as the card shows it: who it needs, how long it is, what it
/// gives, and how it begins.
fn describe(ui: &mut egui::Ui, story: &Story, named: bool) {
    if named {
        paper::heading(ui, &story.title);
    }
    let lines = story
        .scenes
        .iter()
        .flat_map(|scene| &scene.beats)
        .filter(|beat| matches!(beat.action, Action::Say { .. } | Action::Narrate { .. }))
        .count();
    paper::aside(
        ui,
        format!(
            "For {} or more travellers: {} {}, {} {} and {} {} said.",
            story.min_cast,
            story.roles.len(),
            if story.roles.len() == 1 {
                "role"
            } else {
                "roles"
            },
            story.scenes.len(),
            if story.scenes.len() == 1 {
                "scene"
            } else {
                "scenes"
            },
            lines,
            if lines == 1 { "line" } else { "lines" },
        ),
    );
    let given: Vec<&str> = story
        .scenes
        .iter()
        .flat_map(|scene| &scene.beats)
        .filter_map(|beat| match &beat.action {
            Action::Souvenir { id } => souvenirs::name(id),
            _ => None,
        })
        .collect();
    if !given.is_empty() {
        paper::aside(
            ui,
            format!("It can give: {}.", given.join(", ").to_lowercase()),
        );
    }
    // How it begins, its names left as the roles they are.
    let opening: Vec<String> = story.scenes[story.start]
        .beats
        .iter()
        .filter_map(|beat| match &beat.action {
            Action::Say { line, .. } | Action::Narrate { line, .. } => story.lines.get(line),
            _ => None,
        })
        .take(2)
        .map(|line| {
            crate::story::lines::fill(&line.text, |role| Some(format!("the {role}")))
                .unwrap_or_else(|| line.text.clone())
        })
        .collect();
    for line in opening {
        ui.add(paper::Words::new(format!("\u{201c}{line}\u{201d}")).color(paper::FADED));
    }
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect::<String>() + ".",
        None => String::new(),
    }
}
