//! The Woods' map editor in the window: putting places down on a map, joining them with paths,
//! words for whatever is said along the way, then trying it, keeping it in the Woods or sharing
//! it. The adventure itself is `crate::adventure::draft`'s; keeping and sharing are
//! `crate::sharing`'s, the same for anything made at the Hill.
//!
//! The person works on the map itself: a place from the tray is put down with a click, a place
//! is picked up and moved by dragging it, two places are joined by clicking one then the other,
//! and whatever is clicked is shown on the card to the right, to name it and say what it is.

use super::{Card, HillApp, paper};
use crate::adventure::draft::{self, Draft, MAX_LINE_CHARS, MAX_NAME_CHARS, MAX_TITLE_CHARS};
use crate::adventure::{self, opener_key, opener_words};
use crate::expedition::map::{Map, Opener, Site, Way};
use crate::paint::{ellipse, put, rgb, rgba};
use crate::sharing::{self, Drafts};
use crate::story::Library;
use eframe::egui;
use formiga_art::Canvas;

/// The kind of thing `Drafts` keeps for the map editor.
const DRAFTS: &str = "adventures";

/// An adventure being made.
pub struct Making {
    pub draft: Draft,
    /// The draft's name among the kept drafts.
    pub name: String,
    pub tool: Tool,
    pub selected: Selected,
    /// A place being moved, as it is dragged.
    pub dragging: Option<usize>,
    /// The map as last drawn, and what it was drawn from.
    picture: Option<(Map, Canvas)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Clicking picks out a place or a path.
    Pick,
    /// Clicking puts a place of this kind down.
    Put(Site),
    /// Clicking one place then another joins them; the first, once clicked.
    Join(Option<usize>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Nothing,
    Place(usize),
    Path(usize),
}

/// Each kind of place, as the tray offers it: what it is, and what is done there.
fn site_label(site: Site) -> &'static str {
    match site {
        Site::Edge => "A way in",
        Site::Hedgerow => "Hedgerow: forage",
        Site::Meadow => "Meadow: catch a bug",
        Site::Glade => "Glade: rummage",
        Site::Log => "Fallen log: a rest",
        Site::Signpost => "Signpost: a fork",
        Site::Pool => "Pool: fish",
        Site::OldTrack => "Old track: scavenge",
        Site::Falls => "Falls: wade in",
    }
}

/// Who can take a path, as the card offers it.
fn way_label(way: Option<Opener>) -> &'static str {
    match way {
        None => "Anyone",
        Some(Opener::Explorer) => "An explorer",
        Some(Opener::LittleOne) => "A little one",
        Some(Opener::ClosePair) => "Close friends",
        Some(Opener::Bold) => "Someone bold",
        Some(Opener::Reader) => "A reader of signs",
    }
}

impl HillApp {
    fn drafts(&self) -> Drafts {
        Drafts::open(crate::memories::Memories::folder().as_deref(), DRAFTS)
    }

    /// The adventures being made, by name and title, the latest worked on first.
    fn adventure_drafts(&self) -> Vec<(String, String)> {
        let drafts = self.drafts();
        drafts
            .list()
            .into_iter()
            .filter_map(|name| {
                let draft = Draft::from_text(&drafts.load(&name)?)?;
                let title = if draft.title.trim().is_empty() {
                    "An adventure with no name yet".to_owned()
                } else {
                    draft.title.clone()
                };
                Some((name, title))
            })
            .collect()
    }

    /// The Woods' tray's way into the map editor: a new adventure, or one being made.
    pub(super) fn making_choice(&mut self, ui: &mut egui::Ui) {
        let begun = self.adventure_drafts();
        let mut open = None;
        let new = ui.add(paper::Button::new("Make an adventure"));
        if paper::explain(ui, new, "Lay out an adventure of your own on a map").clicked() {
            self.begin_adventure();
        }
        if !begun.is_empty() {
            let (_, picked) = paper::menu(ui, paper::Button::new("Yours \u{25b4}"), false, |ui| {
                let mut picked = None;
                for (name, title) in &begun {
                    if paper::button(ui, title).clicked() {
                        picked = Some(name.clone());
                    }
                }
                picked
            });
            open = picked.flatten();
        }
        if let Some(name) = open {
            self.open_adventure_draft(&name);
        }
    }

    /// Begins a new adventure, with the way in from the lane on the map already.
    fn begin_adventure(&mut self) {
        let key = format!(
            "{}{}",
            self.arrival.cast.snapshot.colony_id,
            self.started.elapsed().as_nanos()
        );
        let stamp: u64 = key
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        let name = format!("adventure-{:08x}", stamp as u32);
        let mut draft = Draft::new(&name, "");
        if let Some(last) = self
            .adventure_drafts()
            .first()
            .and_then(|(name, _)| self.drafts().load(name))
            .and_then(|text| Draft::from_text(&text))
        {
            // Whoever made the last one most likely makes this one too.
            draft.author = last.author;
        }
        self.woods.making = Some(Making {
            draft,
            name,
            tool: Tool::Put(Site::Glade),
            selected: Selected::Nothing,
            dragging: None,
            picture: None,
        });
        self.keep_draft();
        self.card = Some(Card::Making);
    }

    fn open_adventure_draft(&mut self, name: &str) {
        let Some(draft) = self.drafts().load(name).and_then(|text| Draft::from_text(&text)) else {
            return;
        };
        self.woods.making = Some(Making {
            draft,
            name: name.to_owned(),
            tool: Tool::Pick,
            selected: Selected::Nothing,
            dragging: None,
            picture: None,
        });
        self.card = Some(Card::Making);
    }

    /// Keeps the draft as it stands, so nothing made is lost.
    fn keep_draft(&self) {
        if let Some(making) = &self.woods.making
            && let Err(error) = self.drafts().save(&making.name, &making.draft.to_text())
        {
            eprintln!("formiga-hill: could not keep the adventure being made: {error}");
        }
    }

    /// The map being made, with whatever is picked out marked on it.
    pub(super) fn compose_making(&mut self) -> Option<Canvas> {
        if self.woods.expedition.is_some() {
            return None;
        }
        let pointer = self.pointer;
        let making = self.woods.making.as_mut()?;
        let map = making.draft.map();
        let stale = making.picture.as_ref().is_none_or(|(drawn, _)| *drawn != map);
        if stale {
            let picture = crate::expedition::sketch(&map);
            making.picture = Some((map.clone(), picture));
        }
        let mut scene = making.picture.as_ref()?.1.clone();
        // Hidden places, as the mist will hide them: a veil over each.
        for place in map.places.iter().filter(|place| place.hidden) {
            veil(&mut scene, place.at);
        }
        if let Selected::Path(index) = making.selected
            && index < map.paths.len()
        {
            crate::expedition::draw_way(&mut scene, &map.walk(index, map.paths[index].ends.0), true, true);
        }
        let ring = |scene: &mut Canvas, (x, y): (f32, f32), color| {
            for step in 0..48 {
                let angle = step as f32 / 48.0 * std::f32::consts::TAU;
                put(
                    scene,
                    (x + angle.cos() * 15.0) as i32,
                    (y - 10.0 + angle.sin() * 13.0) as i32,
                    color,
                );
            }
        };
        if let Selected::Place(index) = making.selected
            && let Some(place) = map.places.get(index)
        {
            ring(&mut scene, place.at, rgb(0xf5d25e));
        }
        if let Tool::Join(Some(from)) = making.tool
            && let Some(place) = map.places.get(from)
        {
            ring(&mut scene, place.at, rgb(0xf5d25e));
            if let Some(to) = pointer {
                crate::expedition::draw_way(&mut scene, &[place.at, to], true, true);
            }
        }
        if let (Tool::Put(_), Some((x, y))) = (making.tool, pointer) {
            let room = draft::on_the_map((x, y));
            let color = if room {
                rgba(0x3a6a2a, 160)
            } else {
                rgba(0xd05a40, 160)
            };
            ellipse(&mut scene, x as i32, y as i32, 6, 3, color);
        }
        Some(scene)
    }

    /// A click on the map being made.
    pub(super) fn making_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some(at) = pointer else {
            return;
        };
        let Some(making) = &mut self.woods.making else {
            return;
        };
        let place = making.draft.place_at(at);
        let mut said = None;
        let mut changed = false;
        match making.tool {
            Tool::Put(site) => match place {
                Some(index) => making.selected = Selected::Place(index),
                None => match making.draft.add_place(site, at) {
                    Ok(index) => {
                        making.selected = Selected::Place(index);
                        // One place at a time: then join it up.
                        making.tool = Tool::Join(Some(index));
                        said = Some(
                            "Now click another place to join this one to it, or click the map \
                             to put the next place."
                                .to_owned(),
                        );
                        changed = true;
                    }
                    Err(problem) => said = Some(problem),
                },
            },
            Tool::Join(None) => {
                if let Some(index) = place {
                    making.tool = Tool::Join(Some(index));
                    making.selected = Selected::Place(index);
                }
            }
            Tool::Join(Some(from)) => match place {
                Some(to) if to != from => {
                    match making.draft.join(from, to) {
                        Ok(path) => {
                            making.selected = Selected::Path(path);
                            changed = true;
                        }
                        Err(problem) => said = Some(problem),
                    }
                    making.tool = Tool::Join(Some(to));
                }
                Some(_) => making.tool = Tool::Join(None),
                None => {
                    making.tool = Tool::Pick;
                    making.selected = Selected::Nothing;
                }
            },
            Tool::Pick => {
                making.selected = match (place, making.draft.path_at(at)) {
                    (Some(index), _) => Selected::Place(index),
                    (None, Some(path)) => Selected::Path(path),
                    (None, None) => Selected::Nothing,
                };
            }
        }
        if let Some(line) = said {
            self.notice = Some((line, now));
        }
        if changed {
            self.keep_draft();
        }
        if !matches!(
            self.woods.making.as_ref().map(|making| making.selected),
            Some(Selected::Nothing) | None
        ) {
            self.card = Some(Card::Making);
        }
    }

    /// A place picked up and moved by dragging it.
    pub(super) fn making_drag(&mut self, pointer: Option<(f32, f32)>, started: bool, stopped: bool) {
        let Some(making) = &mut self.woods.making else {
            return;
        };
        if started {
            making.dragging = pointer.and_then(|at| making.draft.place_at(at));
            if let Some(index) = making.dragging {
                making.selected = Selected::Place(index);
            }
        }
        if let (Some(index), Some(at)) = (making.dragging, pointer) {
            making.draft.move_place(index, (at.0, at.1 + 6.0));
        }
        if stopped && making.dragging.take().is_some() {
            self.keep_draft();
        }
    }

    /// What is under the pointer on the map being made.
    pub(super) fn making_hover(&self, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
        let making = self.woods.making.as_ref()?;
        if self.woods.expedition.is_some() {
            return None;
        }
        let at = pointer?;
        let label = match (making.tool, making.draft.place_at(at)) {
            (Tool::Put(site), None) => format!("Put {} here", site.name()),
            (Tool::Join(Some(from)), Some(to)) if from != to => format!(
                "Join {} to {}",
                making.draft.places[from].called(),
                making.draft.places[to].called()
            ),
            (_, Some(index)) => {
                let place = &making.draft.places[index];
                let mut label = place.called();
                if index == making.draft.start {
                    label.push_str(" \u{b7} the way in");
                }
                if place.hidden {
                    label.push_str(" \u{b7} hidden");
                }
                if place.ending {
                    label.push_str(" \u{b7} an ending");
                }
                label
            }
            _ => return None,
        };
        Some((label, (at.0, at.1 - 18.0)))
    }

    /// The tray while an adventure is being made: what clicking the map does, and the way out.
    pub(super) fn making_tray(&mut self, ui: &mut egui::Ui, now: f32) {
        let Some(making) = &mut self.woods.making else {
            return;
        };
        let putting = match making.tool {
            Tool::Put(site) => Some(site),
            _ => None,
        };
        let label = putting.map_or("Add a place \u{25b4}", |site| site_label(site));
        let (_, picked) = paper::menu(
            ui,
            paper::Button::new(label).selected(putting.is_some()),
            false,
            |ui| {
                let mut picked = None;
                for site in Site::ALL {
                    if ui
                        .add(paper::Button::new(site_label(site)).selected(putting == Some(site)))
                        .clicked()
                    {
                        picked = Some(site);
                    }
                }
                picked
            },
        );
        if let Some(Some(site)) = picked {
            making.tool = Tool::Put(site);
        }
        let joining = matches!(making.tool, Tool::Join(_));
        if ui
            .add(paper::Button::new("Join places").selected(joining))
            .clicked()
        {
            making.tool = if joining { Tool::Pick } else { Tool::Join(None) };
        }
        let picking = making.tool == Tool::Pick;
        if ui.add(paper::Button::new("Pick").selected(picking)).clicked() {
            making.tool = Tool::Pick;
        }
        let hint = match making.tool {
            Tool::Put(_) => "Click the map to put it there. Drag a place to move it.",
            Tool::Join(None) => "Click a place, then another, to join them with a path.",
            Tool::Join(Some(_)) => "Click another place to join them, or the map to stop.",
            Tool::Pick => "Click a place or a path to change it. Drag a place to move it.",
        };
        paper::hint(ui, hint);
        let card = self.showing(Card::Making);
        if ui
            .add(paper::Button::new("Details").selected(card))
            .clicked()
        {
            self.toggle(Card::Making);
        }
        if paper::button(ui, "Done").clicked() {
            self.keep_draft();
            self.woods.making = None;
            if self.showing(Card::Making) {
                self.card = None;
            }
            self.notice = Some((
                "Put away for now. It's under \u{201c}Yours\u{201d} whenever you want it.".into(),
                now,
            ));
        }
    }

    /// The card of whatever is being made: its name and words, whatever is picked out on the
    /// map, what is still to do, and trying, keeping and sharing it.
    pub(super) fn making_window(&mut self, ctx: &egui::Context, now: f32) {
        if self.area != super::Area::Woods
            || self.woods.making.is_none()
            || self.woods.expedition.is_some()
        {
            return;
        }
        let mut changed = false;
        let mut act = None;
        self.show_card(ctx, Card::Making, "Making an adventure", 0.46, |app, ui| {
            let Some(making) = &mut app.woods.making else {
                return;
            };
            let draft = &mut making.draft;
            changed |= field(ui, "title", &mut draft.title, "What is it called?", MAX_TITLE_CHARS);
            changed |= field(
                ui,
                "about",
                &mut draft.about,
                "A line about it, for choosing it",
                MAX_LINE_CHARS,
            );
            changed |= field(ui, "author", &mut draft.author, "Made by", MAX_TITLE_CHARS);
            match making.selected {
                Selected::Place(index) if index < draft.places.len() => {
                    changed |= place_card(ui, draft, index, &mut making.selected);
                }
                Selected::Path(index) if index < draft.paths.len() => {
                    changed |= path_card(ui, draft, index, &mut making.selected);
                }
                _ => {
                    paper::heading(ui, "On the map");
                    paper::aside(
                        ui,
                        "Click a place or a path on the map to name it and say what happens there.",
                    );
                }
            }
            paper::rule(ui);
            let problems = draft.problems();
            if problems.is_empty() {
                paper::words(ui, "Ready to try, keep or share.");
            } else {
                paper::heading(ui, "Still to do");
                for problem in problems.iter().take(4) {
                    paper::aside(ui, problem);
                }
            }
            let ready = problems.is_empty();
            ui.horizontal_wrapped(|ui| {
                let party = !app.woods.party.is_empty();
                let try_it = ui.add(paper::Button::new("Try it").enabled(ready && party));
                if paper::explain(ui, try_it, "Play it now with whoever is chosen; nothing is kept")
                    .clicked()
                {
                    act = Some(Act::Try);
                }
                let keep = ui.add(paper::Button::new("Keep it in the Woods").enabled(ready));
                if paper::explain(ui, keep, "Add it to the Woods' adventures, to play any time")
                    .clicked()
                {
                    act = Some(Act::Keep);
                }
                let share = ui.add(paper::Button::new("Share\u{2026}").enabled(ready));
                if paper::explain(ui, share, "Save it as one file to pass on").clicked() {
                    act = Some(Act::Share);
                }
            });
            if paper::button(ui, "Throw this one away").clicked() {
                act = Some(Act::ThrowAway);
            }
        });
        if changed {
            self.keep_draft();
        }
        match act {
            Some(Act::Try) => self.try_making(now),
            Some(Act::Keep) => self.keep_making(now),
            Some(Act::Share) => self.share_making(now),
            Some(Act::ThrowAway) => {
                if let Some(making) = self.woods.making.take() {
                    let _ = self.drafts().remove(&making.name);
                }
                self.card = None;
                self.notice = Some((
                    "Thrown away. Anything kept in the Woods stays there.".into(),
                    now,
                ));
            }
            None => {}
        }
    }

    fn try_making(&mut self, now: f32) {
        let Some(making) = &self.woods.making else {
            return;
        };
        match making.draft.adventure() {
            Ok(adventure) => {
                self.card = None;
                self.try_adventure(adventure, now);
            }
            Err(problem) => self.notice = Some((problem, now)),
        }
    }

    /// Keeps the adventure in the Woods, as a package in the packages folder, replacing the
    /// version kept before.
    fn keep_making(&mut self, now: f32) {
        let Some(folder) = self.packages_folder.clone() else {
            self.notice = Some(("There's nowhere on this computer to keep it.".into(), now));
            return;
        };
        let Some(making) = &mut self.woods.making else {
            return;
        };
        making.draft.version += 1;
        let files = making.draft.files();
        let title = making.draft.title.clone();
        let key = (making.draft.package_id.clone(), adventure_id(&making.draft));
        self.keep_draft();
        match sharing::install(&folder, &files) {
            Ok(installed) => {
                self.reload_packages();
                self.woods.adventure = Some(key);
                let line = if installed.replaced.is_some() {
                    format!("{title} is kept in the Woods, in place of the last version.")
                } else {
                    format!("{title} is kept in the Woods: choose it under Adventure.")
                };
                self.notice = Some((line, now));
            }
            Err(problem) => self.notice = Some((format!("It couldn't be kept: {}", problem.problem), now)),
        }
    }

    /// Saves the adventure as one share file, wherever the person likes.
    fn share_making(&mut self, now: f32) {
        let Some(making) = &mut self.woods.making else {
            return;
        };
        let file = format!(
            "{}.{}",
            sharing::slug(&making.draft.title, 40),
            sharing::EXTENSION
        );
        let Some(path) = rfd::FileDialog::new()
            .set_title("Share an adventure")
            .set_file_name(&file)
            .add_filter("Formiga Hill adventure", &[sharing::EXTENSION])
            .save_file()
        else {
            return;
        };
        making.draft.version += 1;
        let files = making.draft.files();
        self.keep_draft();
        let line = match sharing::export(&files, &path) {
            Ok(()) => format!(
                "Saved to share: {}. Whoever opens it at their Hill sees what's in it first.",
                path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned())
            ),
            Err(problem) => format!("It couldn't be saved: {}", problem.problem),
        };
        self.notice = Some((line, now));
    }

    /// Reads every package again, after one has been kept or taken out.
    fn reload_packages(&mut self) {
        self.library = Library::load(self.packages_folder.as_deref(), &[]);
        for problem in &self.library.problems {
            eprintln!("formiga-hill: a package was not loaded: {problem}");
        }
        self.woods.plan = None;
    }
}

#[derive(Clone, Copy)]
enum Act {
    Try,
    Keep,
    Share,
    ThrowAway,
}

/// The id an adventure made in the editor goes by inside its package.
fn adventure_id(draft: &Draft) -> String {
    draft
        .adventure()
        .map_or_else(|_| "adventure".to_owned(), |adventure| adventure.id)
}

/// A place picked out on the map: what kind it is, its name and words, and what it is to the
/// adventure. Says whether anything changed.
fn place_card(ui: &mut egui::Ui, draft: &mut Draft, index: usize, selected: &mut Selected) -> bool {
    let mut changed = false;
    let start = draft.start == index;
    let place = &mut draft.places[index];
    paper::heading(ui, place.called());
    let kind = place.site();
    let (_, picked) = paper::menu(
        ui,
        paper::Button::new(format!("{} \u{25b4}", site_label(kind))),
        false,
        |ui| {
            let mut picked = None;
            for site in Site::ALL {
                if ui
                    .add(paper::Button::new(site_label(site)).selected(site == kind))
                    .clicked()
                {
                    picked = Some(site);
                }
            }
            picked
        },
    );
    if let Some(Some(site)) = picked
        && site != kind
    {
        place.site = site.key().to_owned();
        changed = true;
    }
    changed |= field(
        ui,
        ("name", place.number),
        &mut place.name,
        kind.name(),
        MAX_NAME_CHARS,
    );
    changed |= field(
        ui,
        ("arrive", place.number),
        &mut place.arrive,
        "Something said on getting here (optional)",
        MAX_LINE_CHARS,
    );
    ui.horizontal_wrapped(|ui| {
        if !start {
            let hidden = ui.add(paper::Button::new("Hidden in the mist").selected(place.hidden));
            if paper::explain(ui, hidden, "Out of sight on the map until somebody gets there")
                .clicked()
            {
                place.hidden = !place.hidden;
                changed = true;
            }
        }
        let ending = ui.add(paper::Button::new("An ending").selected(place.ending));
        if paper::explain(ui, ending, "Getting here finishes the adventure").clicked() {
            place.ending = !place.ending;
            changed = true;
        }
    });
    if place.ending {
        changed |= field(
            ui,
            ("ending", place.number),
            &mut place.ending_words,
            "What is said at the end; {leader} names whoever leads",
            MAX_LINE_CHARS,
        );
    }
    ui.horizontal_wrapped(|ui| {
        if start {
            paper::faint(ui, "Everyone comes in here.");
        } else {
            if paper::button(ui, "Everyone comes in here").clicked() {
                draft.set_start(index);
                changed = true;
            }
            if paper::button(ui, "Take it off the map").clicked() && draft.remove_place(index) {
                *selected = Selected::Nothing;
                changed = true;
            }
        }
    });
    changed
}

/// A path picked out on the map: who can take it, what it is called, and taking it away.
fn path_card(ui: &mut egui::Ui, draft: &mut Draft, index: usize, selected: &mut Selected) -> bool {
    let mut changed = false;
    let ends = draft.paths[index].ends;
    let name = |number: u32| {
        draft
            .places
            .iter()
            .find(|place| place.number == number)
            .map_or_else(String::new, draft::DraftPlace::called)
    };
    paper::heading(ui, format!("{} to {}", name(ends.0), name(ends.1)));
    let path = &mut draft.paths[index];
    let way = path.opener();
    let (_, picked) = paper::menu(
        ui,
        paper::Button::new(format!("{} can take it \u{25b4}", way_label(way))),
        false,
        |ui| {
            let mut picked = None;
            for option in std::iter::once(None).chain(Opener::ALL.into_iter().map(Some)) {
                let button = ui.add(paper::Button::new(way_label(option)).selected(option == way));
                let about = option.map_or("Anyone on their own", opener_words);
                if paper::explain(ui, button, about).clicked() {
                    picked = Some(option);
                }
            }
            picked
        },
    );
    if let Some(Some(option)) = picked
        && option != way
    {
        path.way = option.map_or("open", opener_key).to_owned();
        changed = true;
    }
    if way.is_some() {
        paper::aside(
            ui,
            "A shortcut: quicker for whoever can take it. Every place needs an open way too.",
        );
        changed |= field(
            ui,
            ("way", index),
            &mut path.name,
            "What is it called? Like \u{201c}the deer track\u{201d}",
            MAX_NAME_CHARS,
        );
    }
    if paper::button(ui, "Take the path away").clicked() {
        draft.remove_path(index);
        *selected = Selected::Nothing;
        changed = true;
    }
    let _ = Way::Open;
    let _ = adventure::LEADER;
    changed
}

/// A line to write in, on a card. Says whether its words changed.
fn field(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    text: &mut String,
    hint: &str,
    most: usize,
) -> bool {
    let before = text.clone();
    let edit = egui::TextEdit::singleline(text)
        .id(ui.make_persistent_id(id))
        .hint_text(hint)
        .char_limit(most)
        .desired_width(ui.available_width());
    ui.add(edit);
    *text != before
}

/// A veil of mist over a hidden place, lighter than the mist it will be in, so whoever is
/// making it can still see it.
fn veil(scene: &mut Canvas, (x, y): (f32, f32)) {
    let (cx, cy) = (x as i32, y as i32 - 12);
    for py in cy - 18..cy + 18 {
        for px in cx - 24..cx + 24 {
            let (u, v) = ((px - cx) as f32 / 24.0, (py - cy) as f32 / 18.0);
            let r = u * u + v * v;
            if r < 1.0 {
                put(scene, px, py, rgba(0xf4f0e4, ((1.0 - r) * 120.0) as u8));
            }
        }
    }
}
