//! The story builder, on a card in the Clubhouse: a story written a step at a time, from plain
//! choices and plain words, tried out with whoever is visiting, kept as a draft, put on the
//! notice board, and shared as one file. What it writes is an ordinary story package
//! (`story::draft`), loaded by the same loader as every other before it is played anywhere.

use super::{Area, Card, HillApp, paper};
use crate::sharing::{self, Drafts};
use crate::story::draft::{
    self, Choice, Draft, DraftRole, EVERYONE, FEELINGS, KIND_NAMES, KINDS_OF_STEP, MAX_LINE_CHARS,
    MAX_NAME_CHARS, MAX_TITLE_CHARS, MOST_ROLES, MOST_SCENES, MOST_STEPS, PICKS, PLACE_NAMES,
    POSES, Problem, RELATIONS, Spot, StepAction, Then,
};
use crate::story::lines::KINDS;
use crate::story::souvenirs;
use eframe::egui;

/// Which page of the builder is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    /// The stories being written.
    #[default]
    Drafts,
    /// One story: its title, who is in it, and its scenes.
    Story,
    /// One scene of it, by number.
    Scene(u32),
}

/// A story being tried out, and where to go back to once it is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Trying {
    /// A draft, from the builder.
    Draft,
    /// A shared story, before it is added.
    Shared,
}

/// The builder's state: the drafts kept on this computer, the one open, and where in it.
#[derive(Default)]
pub(super) struct Writing {
    drafts: Drafts,
    /// Each draft kept, by name, with its title and a few words about it, read when the list is
    /// shown rather than every frame.
    listed: Option<Vec<(String, String, String)>>,
    open: Option<Draft>,
    page: Page,
    /// A role being renamed, and the name so far: it is renamed everywhere once it is finished.
    naming: Option<(usize, String)>,
    /// Changed since it was last kept.
    changed: bool,
    /// Asked to throw the open draft away, waiting to be sure.
    throwing_away: bool,
    /// The open draft as it was, to step back to.
    history: super::undo::History<Draft>,
}

impl Writing {
    pub(super) fn new(data: Option<&std::path::Path>) -> Self {
        Self {
            drafts: Drafts::open(data, "stories"),
            ..Self::default()
        }
    }

    /// The page open, so each keeps how far it was scrolled.
    pub(super) fn page_name(&self) -> String {
        format!("{:?}", self.page)
    }

    /// For review: a new story open, at its `scene`th scene if one is given.
    pub(super) fn open_sample(&mut self, scene: Option<usize>) {
        let mut draft = Draft::begin("sample");
        draft.title = "The Lost Kite".into();
        draft.author = "A friend of the Hill".into();
        let second = draft.add_scene("Up the bookshelf");
        if let Some(first) = draft.scenes.first_mut() {
            first.steps.push(draft::Step::new(StepAction::Choose {
                options: vec![
                    Choice {
                        text: "Climb up for the kite".into(),
                        goto: second,
                    },
                    Choice {
                        text: "Leave it where it is".into(),
                        goto: second,
                    },
                ],
            }));
        }
        self.page = match scene.and_then(|n| draft.scenes.get(n.saturating_sub(1))) {
            Some(scene) => Page::Scene(scene.number),
            None => Page::Story,
        };
        self.open = Some(draft);
    }

    fn name_of(draft: &Draft) -> String {
        draft
            .package_id
            .rsplit('.')
            .next()
            .unwrap_or("untitled")
            .to_owned()
    }

    /// Keeps the open draft, if it has changed. Says why not, if it could not be kept.
    fn keep(&mut self) -> Option<String> {
        let draft = self.open.as_ref()?;
        if !self.changed {
            return None;
        }
        match self.drafts.save(&Self::name_of(draft), &draft.to_text()) {
            Ok(()) => {
                self.changed = false;
                self.listed = None;
                None
            }
            Err(error) => Some(format!("The draft couldn't be kept: {error}.")),
        }
    }

    fn list(&mut self) -> &[(String, String, String)] {
        let drafts = &self.drafts;
        self.listed.get_or_insert_with(|| {
            drafts
                .list()
                .into_iter()
                .filter_map(|name| {
                    let draft = Draft::from_text(&drafts.load(&name)?)?;
                    let scenes = draft.scenes.len();
                    let about = format!(
                        "{} {} \u{b7} {} {}{}",
                        scenes,
                        if scenes == 1 { "scene" } else { "scenes" },
                        draft.roles.len(),
                        if draft.roles.len() == 1 {
                            "role"
                        } else {
                            "roles"
                        },
                        if draft.version > 0 {
                            format!(" \u{b7} shared or on the board, version {}", draft.version)
                        } else {
                            String::new()
                        }
                    );
                    Some((name, draft.title, about))
                })
                .collect()
        })
    }
}

/// A fresh key for a new draft's package id: from the time and this Hill, so two drafts begun
/// anywhere never share one.
fn fresh_key() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let mixed = format!("{nanos}-{}", std::process::id())
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{mixed:012x}")
}

/// What the builder asks to be done once the card is drawn, outside it.
enum Ask {
    Try,
    Share,
    PutOnBoard,
}

impl HillApp {
    /// The builder's card, while it is open in the Clubhouse.
    pub(super) fn writing_window(&mut self, ctx: &egui::Context, now: f32) {
        if self.area != Area::Clubhouse || self.story.is_some() {
            return;
        }
        if !self.showing(Card::Writing) {
            // Whatever was open is kept as soon as the card is put away.
            if let Some(problem) = self.writing.keep() {
                self.notice = Some((problem, now));
            }
            return;
        }
        let title = match self.writing.page {
            Page::Drafts => "Write a story",
            _ => "Writing a story",
        };
        let ask = self
            .show_card(ctx, Card::Writing, title, 0.66, |app, ui| {
                app.writing_page(ui)
            })
            .flatten();
        match ask {
            Some(Ask::Try) => self.try_draft(now),
            Some(Ask::Share) => self.share_draft(now),
            Some(Ask::PutOnBoard) => self.put_draft_on_board(now),
            None => {}
        }
    }

    fn writing_page(&mut self, ui: &mut egui::Ui) -> Option<Ask> {
        if self.writing.page != Page::Drafts {
            self.writing_undo(ui);
        }
        match self.writing.page {
            Page::Drafts => {
                self.drafts_page(ui);
                None
            }
            Page::Story => self.story_page(ui),
            Page::Scene(number) => {
                self.scene_page(ui, number);
                None
            }
        }
    }

    /// Undo and Redo over the open story, on every page of it.
    fn writing_undo(&mut self, ui: &mut egui::Ui) {
        let writing = &mut self.writing;
        let Some(draft) = writing.open.as_mut() else {
            return;
        };
        let name = draft.package_id.clone();
        // Not over its version, which counts what has been shared or put on the board: it stays
        // as it is, so no version is ever given out twice.
        let version = std::mem::take(&mut draft.version);
        writing.history.feed(ui.ctx(), &name, draft);
        let history = &mut writing.history;
        let back = ui.horizontal(|ui| history.buttons(ui, draft)).inner;
        draft.version = version;
        if let Some(back) = back {
            *draft = back;
            draft.version = version;
            writing.changed = true;
            writing.naming = None;
            writing.throwing_away = false;
        }
    }

    /// The contents: what the builder is for, a new story, and every draft kept.
    fn drafts_page(&mut self, ui: &mut egui::Ui) {
        let writing = &mut self.writing;
        paper::aside(
            ui,
            "Write a story for the colony to play out by the fire. Try it as you go, put it on \
             the notice board, or share it with a friend who has Formiga Hill.",
        );
        if paper::button(ui, "Begin a new story").clicked() {
            writing.open = Some(Draft::begin(&fresh_key()));
            writing.page = Page::Story;
            writing.changed = true;
            return;
        }
        if !writing.drafts.can_keep() {
            paper::faint(
                ui,
                "There is nowhere to keep drafts on this computer, so a story is lost when Hill \
                 closes unless it is put on the board or shared.",
            );
        }
        let listed = writing.list().to_vec();
        if listed.is_empty() {
            return;
        }
        paper::heading(ui, "Your stories");
        for (name, title, about) in listed {
            if ui.add(paper::Entry::new(title, about)).clicked()
                && let Some(draft) = writing
                    .drafts
                    .load(&name)
                    .and_then(|text| Draft::from_text(&text))
            {
                writing.open = Some(draft);
                writing.page = Page::Story;
                writing.changed = false;
                writing.throwing_away = false;
            }
        }
    }

    /// One story: its title and author, who is in it, its scenes, what is still wrong with it,
    /// and what to do with it.
    fn story_page(&mut self, ui: &mut egui::Ui) -> Option<Ask> {
        let writing = &mut self.writing;
        let Some(draft) = writing.open.as_mut() else {
            writing.page = Page::Drafts;
            return None;
        };
        if paper::back(ui, "Your stories") {
            writing.page = Page::Drafts;
            let problem = writing.keep();
            writing.open = None;
            writing.throwing_away = false;
            if let Some(problem) = problem {
                paper::aside(ui, problem);
            }
            return None;
        }
        let mut changed = false;
        paper::heading(ui, "Title");
        changed |= paper::field(
            ui,
            "title",
            &mut draft.title,
            "What is the story called?",
            MAX_TITLE_CHARS,
            None,
        )
        .changed();
        paper::heading(ui, "Written by");
        changed |= paper::field(
            ui,
            "author",
            &mut draft.author,
            "Your name, to show beside the title",
            MAX_TITLE_CHARS,
            None,
        )
        .changed();

        paper::heading(ui, "Who is in it");
        paper::aside(
            ui,
            "Each role is played by whoever in the visiting colony fits it best. Write a role's \
             name in braces in a line, such as {host}, and it is said with the name of whoever \
             plays it.",
        );
        changed |= cast_rows(ui, draft, &mut writing.naming);

        paper::heading(ui, "Scenes");
        paper::aside(
            ui,
            "Played from the first, one after another, unless a choice or a scene says otherwise.",
        );
        let mut open_scene = None;
        for (index, scene) in draft.scenes.iter().enumerate() {
            let steps = scene.steps.len();
            let ends_on_choice = scene
                .steps
                .last()
                .is_some_and(|step| step.kind() == "choose");
            let then = if ends_on_choice {
                "then a choice".to_owned()
            } else {
                match scene.then {
                    Then::Next if index + 1 == draft.scenes.len() => "then the end".to_owned(),
                    Then::Next => "then the next scene".to_owned(),
                    Then::End => "then the end".to_owned(),
                    Then::Scene(to) => format!("then {}", draft.scene_name(to)),
                }
            };
            let about = format!(
                "{steps} {}, {then}",
                if steps == 1 { "step" } else { "steps" }
            );
            if ui
                .add(paper::Entry::new(
                    draft::scene_label(index, &scene.name),
                    about,
                ))
                .clicked()
            {
                open_scene = Some(scene.number);
            }
        }
        if draft.scenes.len() < MOST_SCENES && paper::button(ui, "Add a scene").clicked() {
            let number = draft.add_scene(&format!("Scene {}", draft.scenes.len() + 1));
            open_scene = Some(number);
            changed = true;
        }

        let problems = draft.problems();
        if !problems.is_empty() {
            paper::heading(ui, "Not ready yet");
            for problem in problems.iter().take(8) {
                ui.add(paper::Words::new(where_it_is(draft, problem)).color(paper::AMBER_DEEP));
            }
            if problems.len() > 8 {
                paper::faint(ui, format!("and {} more", problems.len() - 8));
            }
        }

        paper::rule(ui);
        let ready = problems.is_empty();
        let mut ask = None;
        ui.horizontal_wrapped(|ui| {
            let try_it = ui.add(paper::Button::new("Try it").enabled(ready));
            if paper::explain(
                ui,
                try_it,
                "Play it now with whoever is visiting. Nothing is kept: it's only a try.",
            )
            .clicked()
            {
                ask = Some(Ask::Try);
            }
            let board = ui.add(paper::Button::new("Put on the board").enabled(ready));
            if paper::explain(
                ui,
                board,
                "Pin it to the notice board, for every colony that visits. Put it there again \
                 after changing it.",
            )
            .clicked()
            {
                ask = Some(Ask::PutOnBoard);
            }
            let share = ui.add(paper::Button::new("Share\u{2026}").enabled(ready));
            if paper::explain(
                ui,
                share,
                "Save it as one file to give a friend. They open it from the Clubhouse's Story \
                 packages, and see what is in it before adding it.",
            )
            .clicked()
            {
                ask = Some(Ask::Share);
            }
        });
        if !ready {
            paper::faint(
                ui,
                "Once nothing is left to put right, it can be tried and shared.",
            );
        }
        if writing.drafts.can_keep() {
            paper::faint(ui, "Kept as you go, as a draft on this computer.");
        }
        let mut thrown = false;
        if writing.throwing_away {
            ui.horizontal_wrapped(|ui| {
                paper::words(ui, "Throw this draft away for good?");
                thrown = paper::button(ui, "Throw it away").clicked();
                if paper::button(ui, "Keep it").clicked() {
                    writing.throwing_away = false;
                }
            });
        } else if paper::button(ui, "Throw it away\u{2026}").clicked() {
            writing.throwing_away = true;
        }
        if thrown {
            let name = Writing::name_of(draft);
            let _ = writing.drafts.remove(&name);
            writing.open = None;
            writing.page = Page::Drafts;
            writing.listed = None;
            writing.changed = false;
            writing.throwing_away = false;
            return None;
        }

        if changed {
            writing.changed = true;
        }
        if let Some(number) = open_scene {
            writing.page = Page::Scene(number);
        }
        // Kept as it is written, a moment at a time rather than every keystroke: whenever the
        // page changes or a field is finished.
        if ask.is_some() || open_scene.is_some() || ui.ctx().memory(|m| m.focused().is_none()) {
            let _ = writing.keep();
        }
        ask
    }

    /// One scene: its name, its steps, and where the story goes after it.
    fn scene_page(&mut self, ui: &mut egui::Ui, number: u32) {
        let writing = &mut self.writing;
        let Some(draft) = writing.open.as_mut() else {
            writing.page = Page::Drafts;
            return;
        };
        let Some(index) = draft.scene_index(number) else {
            writing.page = Page::Story;
            return;
        };
        let title = draft.title.clone();
        if paper::back(ui, &title) {
            writing.page = Page::Story;
            let _ = writing.keep();
            return;
        }
        let problems = draft.problems();
        let roles: Vec<String> = draft.roles.iter().map(|role| role.name.clone()).collect();
        let scenes: Vec<(u32, String)> = draft
            .scenes
            .iter()
            .enumerate()
            .map(|(i, scene)| (scene.number, draft::scene_label(i, &scene.name)))
            .collect();
        let scene_count = draft.scenes.len();
        let mut changed = false;
        let mut moved: Option<(usize, isize)> = None;
        let mut removed = None;
        let mut scene_moved: Option<isize> = None;
        let mut scene_removed = false;

        paper::heading(ui, format!("Scene {}", index + 1));
        let scene = &mut draft.scenes[index];
        changed |= paper::field(
            ui,
            ("scene-name", number),
            &mut scene.name,
            "What happens in this scene?",
            MAX_TITLE_CHARS,
            None,
        )
        .changed();
        for problem in problems.iter().filter(|p| p.spot == Spot::Scene(index)) {
            ui.add(paper::Words::new(&problem.words).color(paper::AMBER_DEEP));
        }

        paper::heading(ui, "What happens");
        if scene.steps.is_empty() {
            paper::faint(ui, "Nothing yet. Add a step below.");
        }
        let steps = scene.steps.len();
        for (b, step) in scene.steps.iter_mut().enumerate() {
            ui.push_id(("step", number, b), |ui| {
                paper::rule(ui);
                ui.horizontal_wrapped(|ui| {
                    paper::name(ui, format!("{}.", b + 1));
                    changed |= step_editor(ui, &mut step.action, &roles, &scenes);
                });
                for problem in problems.iter().filter(|p| p.spot == Spot::Step(index, b)) {
                    ui.add(paper::Words::new(&problem.words).color(paper::AMBER_DEEP));
                }
                ui.horizontal_wrapped(|ui| {
                    if b > 0 && step.kind() != "choose" {
                        let together = ui.add(
                            paper::Button::new("At the same time as the one before")
                                .selected(step.together),
                        );
                        if paper::explain(
                            ui,
                            together,
                            "Starts with the step before rather than after it: two can walk at \
                             once, or someone can move while a line is read.",
                        )
                        .clicked()
                        {
                            step.together = !step.together;
                            changed = true;
                        }
                    }
                    if b > 0 {
                        let up = ui.add(paper::Button::new("\u{25b4}"));
                        paper::name_it(ui, &up, "Earlier");
                        if up.clicked() {
                            moved = Some((b, -1));
                        }
                    }
                    if b + 1 < steps {
                        let down = ui.add(paper::Button::new("\u{25be}"));
                        paper::name_it(ui, &down, "Later");
                        if down.clicked() {
                            moved = Some((b, 1));
                        }
                    }
                    let gone = ui.add(paper::Button::new("\u{d7}"));
                    paper::name_it(ui, &gone, "Take this step out");
                    if gone.clicked() {
                        removed = Some(b);
                    }
                });
            });
        }
        paper::rule(ui);
        let has_choice = scene.steps.iter().any(|step| step.kind() == "choose");
        let mut added = None;
        if steps < MOST_STEPS {
            paper::menu(ui, paper::Button::new("Add a step"), false, |ui| {
                columns(ui, 200, |ui| {
                    for (kind, words) in KINDS_OF_STEP {
                        if kind == "choose" && has_choice {
                            continue;
                        }
                        if paper::button(ui, words).clicked() {
                            added = Some(kind);
                        }
                    }
                });
            });
        }

        if !has_choice {
            paper::heading(ui, "Afterwards");
            let then_words = match scene.then {
                Then::Next if index + 1 == scene_count => "The end of the story".to_owned(),
                Then::Next => "On to the next scene".to_owned(),
                Then::End => "The end of the story".to_owned(),
                Then::Scene(to) => format!(
                    "On to {}",
                    scenes
                        .iter()
                        .find(|(n, _)| *n == to)
                        .map_or("a scene that has gone", |(_, name)| name.as_str())
                ),
            };
            paper::menu(ui, paper::Button::new(then_words), false, |ui| {
                columns(ui, 220, |ui| {
                    if index + 1 < scene_count
                        && paper::button(ui, "On to the next scene").clicked()
                    {
                        scene.then = Then::Next;
                        changed = true;
                    }
                    if paper::button(ui, "The end of the story").clicked() {
                        scene.then = Then::End;
                        changed = true;
                    }
                    for (to, name) in &scenes {
                        if *to != number && paper::button(ui, format!("On to {name}")).clicked() {
                            scene.then = Then::Scene(*to);
                            changed = true;
                        }
                    }
                });
            });
        } else {
            paper::faint(
                ui,
                "The scene ends on the choice: each option says where it goes.",
            );
        }

        paper::rule(ui);
        ui.horizontal_wrapped(|ui| {
            if index > 0 && paper::button(ui, "Move it earlier").clicked() {
                scene_moved = Some(-1);
            }
            if index + 1 < scene_count && paper::button(ui, "Move it later").clicked() {
                scene_moved = Some(1);
            }
            if scene_count > 1 && paper::button(ui, "Take this scene out").clicked() {
                scene_removed = true;
            }
        });

        if let Some(kind) = added {
            let step = draft.new_step(kind, index);
            let scene = &mut draft.scenes[index];
            // A choice ends its scene, so anything else goes in before one.
            let at = if scene.steps.last().is_some_and(|s| s.kind() == "choose") {
                scene.steps.len() - 1
            } else {
                scene.steps.len()
            };
            scene.steps.insert(at, step);
            changed = true;
        }
        let scene = &mut draft.scenes[index];
        if let Some((b, by)) = moved {
            let to = b.saturating_add_signed(by).min(scene.steps.len() - 1);
            scene.steps.swap(b, to);
            changed = true;
        }
        if let Some(b) = removed {
            scene.steps.remove(b);
            changed = true;
        }
        if let Some(by) = scene_moved {
            let to = index.saturating_add_signed(by).min(scene_count - 1);
            draft.scenes.swap(index, to);
            changed = true;
        }
        if scene_removed {
            draft.remove_scene(index);
            writing.page = Page::Story;
            changed = true;
        }
        if changed {
            writing.changed = true;
        }
        if ui.ctx().memory(|m| m.focused().is_none()) {
            let _ = writing.keep();
        }
    }

    fn try_draft(&mut self, now: f32) {
        let Some(draft) = &self.writing.open else {
            return;
        };
        match draft.package() {
            Ok(package) => {
                let _ = self.writing.keep();
                if let Some(story) = package.stories.into_iter().next() {
                    self.try_story(&package.id, story, Trying::Draft, now);
                }
            }
            Err(problems) => self.notice = Some((not_ready(&problems), now)),
        }
    }

    /// A new version of the open draft, ready to go somewhere: its version counted up and kept.
    fn next_version(&mut self, now: f32) -> Option<(String, crate::story::Files)> {
        let draft = self.writing.open.as_mut()?;
        if let Err(problems) = draft.package() {
            self.notice = Some((not_ready(&problems), now));
            return None;
        }
        draft.version += 1;
        let out = (draft.title.trim().to_owned(), draft.files());
        self.writing.changed = true;
        if let Some(problem) = self.writing.keep() {
            self.notice = Some((problem, now));
        }
        Some(out)
    }

    fn share_draft(&mut self, now: f32) {
        let Some((title, files)) = self.next_version(now) else {
            return;
        };
        let name = format!(
            "{}.{}",
            title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], ""),
            sharing::EXTENSION
        );
        let Some(path) = rfd::FileDialog::new()
            .set_title("Share a story")
            .set_file_name(&name)
            .add_filter("Formiga Hill story", &[sharing::EXTENSION])
            .save_file()
        else {
            return;
        };
        let line = match sharing::export(&files, &path) {
            Ok(()) => format!(
                "Saved \u{201c}{title}\u{201d} to share. A friend opens it from the Clubhouse's \
                 Story packages."
            ),
            Err(error) => format!("The story couldn't be saved: {}.", error.problem),
        };
        self.notice = Some((line, now));
    }

    fn put_draft_on_board(&mut self, now: f32) {
        let Some(folder) = self.packages_folder.clone() else {
            self.notice = Some((
                "There is nowhere to keep stories on this computer.".into(),
                now,
            ));
            return;
        };
        let Some((title, files)) = self.next_version(now) else {
            return;
        };
        let line = match sharing::install(&folder, &files) {
            Ok(installed) => {
                self.reload_library();
                if installed.replaced.is_some() {
                    format!("The notice board has the new \u{201c}{title}\u{201d}.")
                } else {
                    format!("\u{201c}{title}\u{201d} is on the notice board.")
                }
            }
            Err(error) => format!("It couldn't be put on the board: {}.", error.problem),
        };
        self.notice = Some((line, now));
    }
}

/// The few words saying a story isn't ready to try or share yet.
fn not_ready(problems: &[Problem]) -> String {
    match problems.first() {
        Some(problem) => format!("Not ready yet: {}", problem.words),
        None => "Not ready yet.".into(),
    }
}

/// A problem, with where it is, for the list on the story's page.
fn where_it_is(draft: &Draft, problem: &Problem) -> String {
    match problem.spot {
        Spot::Story => problem.words.clone(),
        Spot::Role(index) => format!(
            "{}: {}",
            draft
                .roles
                .get(index)
                .map_or("A role", |role| role.name.as_str()),
            problem.words
        ),
        Spot::Scene(index) => format!("Scene {}: {}", index + 1, problem.words),
        Spot::Step(scene, step) => {
            format!("Scene {}, step {}: {}", scene + 1, step + 1, problem.words)
        }
    }
}

/// Buttons laid out in rows no wider than `most` paper pixels, for a long list of choices.
fn columns(ui: &mut egui::Ui, most: i32, add: impl FnOnce(&mut egui::Ui)) {
    let grain = paper::Grain::of(ui.ctx());
    ui.set_max_width(grain.len(most));
    ui.horizontal_wrapped(add);
}

/// A button showing the choice made, which opens the choices; true when another is chosen.
fn choose_from<'a>(
    ui: &mut egui::Ui,
    value: &mut String,
    shown: &str,
    choices: impl IntoIterator<Item = (String, &'a str)>,
) -> bool {
    let mut chosen = None;
    paper::menu(ui, paper::Button::new(shown), false, |ui| {
        columns(ui, 220, |ui| {
            for (key, words) in choices {
                let button = ui.add(paper::Button::new(words).selected(*value == key));
                if button.clicked() {
                    chosen = Some(key);
                }
            }
        });
    });
    match chosen {
        Some(key) if key != *value => {
            *value = key;
            true
        }
        _ => false,
    }
}

/// Who a step is about: one of the roles, or everyone if `everyone`.
fn choose_who(ui: &mut egui::Ui, who: &mut String, roles: &[String], everyone: bool) -> bool {
    let shown = draft::who_name(who).to_owned();
    let choices = roles
        .iter()
        .map(|role| (role.clone(), role.as_str()))
        .chain(everyone.then(|| (EVERYONE.to_owned(), "everyone")));
    choose_from(ui, who, &shown, choices)
}

/// One of the roles, but not `not`.
fn choose_other(ui: &mut egui::Ui, value: &mut String, roles: &[String], not: &str) -> bool {
    let shown = value.clone();
    let choices = roles
        .iter()
        .filter(|role| role.as_str() != not)
        .map(|role| (role.clone(), role.as_str()));
    choose_from(ui, value, &shown, choices)
}

/// The cast: each role's name, how it is cast, whether it must be, and its place in the order.
fn cast_rows(ui: &mut egui::Ui, draft: &mut Draft, naming: &mut Option<(usize, String)>) -> bool {
    let mut changed = false;
    let mut moved: Option<(usize, isize)> = None;
    let mut removed = None;
    let mut renamed: Option<(usize, String)> = None;
    let count = draft.roles.len();
    for index in 0..count {
        let earlier: Vec<String> = draft.roles[..index]
            .iter()
            .map(|role| role.name.clone())
            .collect();
        let role = &mut draft.roles[index];
        ui.push_id(("role", index), |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut name = match naming {
                    Some((at, name)) if *at == index => name.clone(),
                    _ => role.name.clone(),
                };
                let field = paper::field(ui, "name", &mut name, "a name", MAX_NAME_CHARS, Some(64));
                if field.changed() {
                    *naming = Some((index, draft::tidy_role_name(&name)));
                }
                if field.lost_focus()
                    && let Some((at, name)) = naming.take()
                    && at == index
                {
                    renamed = Some((index, name));
                }
                paper::words(ui, "is played by");
                let shown = draft::pick_name(&role.pick);
                let relations = earlier.iter().flat_map(|other| {
                    RELATIONS.iter().map(move |(verb, words)| {
                        (format!("{verb}:{other}"), words.replace("{}", other))
                    })
                });
                let choices: Vec<(String, String)> = PICKS
                    .iter()
                    .map(|(pick, words)| ((*pick).to_owned(), (*words).to_owned()))
                    .chain(relations)
                    .collect();
                changed |= choose_from(
                    ui,
                    &mut role.pick,
                    &shown,
                    choices
                        .iter()
                        .map(|(pick, words)| (pick.clone(), words.as_str())),
                );
            });
            ui.horizontal_wrapped(|ui| {
                let always = ui.add(
                    paper::Button::new(if role.optional {
                        "Only if someone fits"
                    } else {
                        "Always in it"
                    })
                    .selected(!role.optional),
                );
                if paper::explain(
                    ui,
                    always,
                    if role.optional {
                        "Left out if nobody fits, and everything about it skipped. Click to have \
                         someone play it always."
                    } else {
                        "If nobody fits, anyone left plays it. Click to leave it out instead when \
                         nobody fits."
                    },
                )
                .clicked()
                {
                    role.optional = !role.optional;
                    changed = true;
                }
                if index > 0 {
                    let up = ui.add(paper::Button::new("\u{25b4}"));
                    paper::name_it(ui, &up, "Cast earlier");
                    if up.clicked() {
                        moved = Some((index, -1));
                    }
                }
                if index + 1 < count {
                    let down = ui.add(paper::Button::new("\u{25be}"));
                    paper::name_it(ui, &down, "Cast later");
                    if down.clicked() {
                        moved = Some((index, 1));
                    }
                }
                if count > 1 {
                    let gone = ui.add(paper::Button::new("\u{d7}"));
                    paper::name_it(ui, &gone, "Take this role out");
                    if gone.clicked() {
                        removed = Some(index);
                    }
                }
            });
        });
    }
    if count < MOST_ROLES && paper::button(ui, "Add someone").clicked() {
        let name = draft.free_role_name();
        draft.roles.push(DraftRole {
            name,
            pick: "any".into(),
            optional: false,
        });
        changed = true;
    }
    if let Some((index, name)) = renamed {
        // A new name is taken only if it is a good one nobody else has.
        if draft::is_role_name(&name)
            && !draft
                .roles
                .iter()
                .enumerate()
                .any(|(other, role)| other != index && role.name == name)
        {
            draft.rename_role(index, &name);
            changed = true;
        }
    }
    if let Some((index, by)) = moved {
        let to = index.saturating_add_signed(by).min(count - 1);
        draft.roles.swap(index, to);
        changed = true;
    }
    if let Some(index) = removed {
        draft.roles.remove(index);
        changed = true;
    }
    changed
}

/// What a step does, laid out as a sentence with its choices and words in it.
fn step_editor(
    ui: &mut egui::Ui,
    action: &mut StepAction,
    roles: &[String],
    scenes: &[(u32, String)],
) -> bool {
    let mut changed = false;
    let line = |ui: &mut egui::Ui, id: &str, text: &mut String, hint: &str| {
        ui.end_row();
        paper::field(ui, id, text, hint, MAX_LINE_CHARS, None).changed()
    };
    match action {
        StepAction::Narrate { text } => {
            paper::words(ui, "The storyteller says:");
            changed |= line(ui, "text", text, "What the storyteller says");
        }
        StepAction::Say {
            who,
            text,
            variants,
        } => {
            changed |= choose_who(ui, who, roles, false);
            paper::words(ui, "says:");
            changed |= line(ui, "text", text, "What they say");
            let mut gone = None;
            for (kind, words) in variants.iter_mut() {
                ui.end_row();
                paper::aside(
                    ui,
                    format!("Said by {}:", draft::kind_name(kind).to_lowercase()),
                );
                let x = ui.add(paper::Button::new("\u{d7}"));
                paper::name_it(ui, &x, "No different version");
                if x.clicked() {
                    gone = Some(kind.clone());
                }
                changed |= line(ui, kind, words, "How they say it");
            }
            if let Some(kind) = gone {
                variants.remove(&kind);
                changed = true;
            }
            if variants.len() < KINDS.len() {
                ui.end_row();
                let mut added = None;
                let (menu, _) = paper::menu(
                    ui,
                    paper::Button::new("Said differently by\u{2026}"),
                    false,
                    |ui| {
                        columns(ui, 200, |ui| {
                            for (kind, name) in KINDS.iter().zip(KIND_NAMES) {
                                if !variants.contains_key(*kind)
                                    && paper::button(ui, name).clicked()
                                {
                                    added = Some(*kind);
                                }
                            }
                        });
                    },
                );
                paper::explain(
                    ui,
                    menu,
                    "Write how a speaker of one temperament would say it instead: a grump \
                     grumbles, a sweetheart is kind. Everyone else says the line above.",
                );
                if let Some(kind) = added {
                    variants.insert(kind.to_owned(), text.clone());
                    changed = true;
                }
            }
        }
        StepAction::Walk { who, to } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "goes to");
            let shown = draft::place_name(to);
            let choices = PLACE_NAMES
                .iter()
                .map(|(place, words)| ((*place).to_owned(), (*words).to_owned()))
                .chain(
                    roles
                        .iter()
                        .filter(|role| role.as_str() != who.as_str())
                        .map(|role| (format!("beside:{role}"), format!("beside {role}"))),
                )
                .collect::<Vec<_>>();
            changed |= choose_from(
                ui,
                to,
                &shown,
                choices
                    .iter()
                    .map(|(key, words)| (key.clone(), words.as_str())),
            );
        }
        StepAction::Face { who, to } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "turns to");
            changed |= choose_other(ui, to, roles, who);
        }
        StepAction::React { who, feeling } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "shows");
            let shown = draft::feeling_name(feeling).to_owned();
            changed |= choose_from(
                ui,
                feeling,
                &shown,
                FEELINGS
                    .iter()
                    .map(|(key, words)| ((*key).to_owned(), *words)),
            );
        }
        StepAction::Pose { who, pose } => {
            changed |= choose_who(ui, who, roles, true);
            let shown = draft::pose_name(pose).to_owned();
            changed |= choose_from(
                ui,
                pose,
                &shown,
                POSES.iter().map(|(key, words)| ((*key).to_owned(), *words)),
            );
        }
        StepAction::Eat { who } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "has a snack");
        }
        StepAction::Nap { who } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "takes a nap");
        }
        StepAction::Celebrate { who } => {
            changed |= choose_who(ui, who, roles, true);
            paper::words(ui, "celebrates");
        }
        StepAction::Play { who, with } => {
            changed |= choose_who(ui, who, roles, false);
            paper::words(ui, "plays with");
            changed |= choose_other(ui, with, roles, who);
        }
        StepAction::Wait { seconds } => {
            paper::words(ui, "A pause of");
            if paper::button(ui, "\u{2212}").clicked() && *seconds > 0.5 {
                *seconds = (*seconds - 0.5).max(0.5);
                changed = true;
            }
            paper::words(ui, format!("{seconds:.1} seconds"));
            if paper::button(ui, "+").clicked() && *seconds < 30.0 {
                *seconds = (*seconds + 0.5).min(30.0);
                changed = true;
            }
        }
        StepAction::Choose { options } => {
            paper::words(ui, "Whoever is watching chooses:");
            let mut gone = None;
            let count = options.len();
            for (index, option) in options.iter_mut().enumerate() {
                ui.end_row();
                ui.push_id(("option", index), |ui| {
                    changed |= paper::field(
                        ui,
                        "text",
                        &mut option.text,
                        "What the choice says",
                        MAX_LINE_CHARS,
                        Some(150),
                    )
                    .changed();
                    paper::words(ui, "leads to");
                    let mut goto = option.goto.to_string();
                    let shown = scenes
                        .iter()
                        .find(|(n, _)| *n == option.goto)
                        .map_or("a scene", |(_, name)| name.as_str())
                        .to_owned();
                    if choose_from(
                        ui,
                        &mut goto,
                        &shown,
                        scenes
                            .iter()
                            .map(|(n, name)| (n.to_string(), name.as_str())),
                    ) && let Ok(number) = goto.parse()
                    {
                        option.goto = number;
                        changed = true;
                    }
                    if count > 1 {
                        let x = ui.add(paper::Button::new("\u{d7}"));
                        paper::name_it(ui, &x, "Take this option out");
                        if x.clicked() {
                            gone = Some(index);
                        }
                    }
                });
            }
            if let Some(index) = gone {
                options.remove(index);
                changed = true;
            }
            if options.len() < crate::story::script::MAX_CHOICES {
                ui.end_row();
                if paper::button(ui, "Add an option").clicked() {
                    let goto = options
                        .last()
                        .map_or(scenes.first().map_or(0, |(n, _)| *n), |o| o.goto);
                    options.push(Choice {
                        text: String::new(),
                        goto,
                    });
                    changed = true;
                }
            }
        }
        StepAction::Souvenir { id } => {
            paper::words(ui, "Everyone keeps");
            let shown = souvenirs::name(id).unwrap_or("a souvenir").to_lowercase();
            let names: Vec<(String, String)> = souvenirs::for_stories()
                .into_iter()
                .map(|id| {
                    (
                        id.to_owned(),
                        souvenirs::name(id).unwrap_or(id).to_lowercase(),
                    )
                })
                .collect();
            changed |= choose_from(
                ui,
                id,
                &shown,
                names.iter().map(|(id, name)| (id.clone(), name.as_str())),
            );
        }
    }
    changed
}
