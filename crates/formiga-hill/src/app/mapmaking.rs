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
use crate::adventure::{opener_key, opener_words};
use crate::expedition::map::{Map, Opener, Site};
use crate::paint::{ellipse, put, rgb, rgba};
use crate::sharing::{self, Drafts};
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
    /// A place being moved, by number, as it is dragged.
    pub dragging: Option<u32>,
    /// The map as last drawn, and what it was drawn from.
    picture: Option<(Map, Canvas)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Clicking picks out a place or a path.
    Pick,
    /// Clicking puts a place of this kind down.
    Put(Site),
    /// Clicking one place then another joins them; the first, by number, once clicked.
    Join(Option<u32>),
}

/// Whatever is picked out on the map. A place goes by its number, which it keeps however the
/// places around it come and go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Nothing,
    Place(u32),
    Path(usize),
}

impl Making {
    /// The number of the place standing at this index.
    fn number(&self, index: usize) -> u32 {
        self.draft.places[index].number
    }

    /// Lets go of anything no longer on the map: a place taken off, or a path taken away.
    fn tidy(&mut self) {
        let gone = |number: u32| self.draft.index(number).is_none();
        if let Tool::Join(Some(from)) = self.tool
            && gone(from)
        {
            self.tool = Tool::Join(None);
        }
        let selected_gone = match self.selected {
            Selected::Place(number) => gone(number),
            Selected::Path(index) => index >= self.draft.paths.len(),
            Selected::Nothing => false,
        };
        if selected_gone {
            self.selected = Selected::Nothing;
        }
        if self.dragging.is_some_and(gone) {
            self.dragging = None;
        }
    }

    /// Takes up the draft as it was, or as it will be again, from Undo or Redo, keeping its
    /// version as it is. Whatever was picked out, joined from or dragged is let go of at once: it
    /// may not be on the map now, or a path may be another by the same index.
    fn step_to(&mut self, draft: Draft) {
        let version = self.draft.version;
        self.draft = draft;
        self.draft.version = version;
        self.selected = Selected::Nothing;
        self.dragging = None;
        if matches!(self.tool, Tool::Join(Some(_))) {
            self.tool = Tool::Join(None);
        }
    }

    /// A click on the map: anything to say about it, and whether the draft changed.
    fn click(&mut self, at: (f32, f32)) -> (Option<String>, bool) {
        self.tidy();
        let place = self.draft.place_at(at);
        let mut said = None;
        let mut changed = false;
        match self.tool {
            Tool::Put(site) => match place {
                Some(index) => self.selected = Selected::Place(self.number(index)),
                None => match self.draft.add_place(site, at) {
                    Ok(index) => {
                        let number = self.number(index);
                        self.selected = Selected::Place(number);
                        // One place at a time: then join it up.
                        self.tool = Tool::Join(Some(number));
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
            Tool::Join(from) => match (from.and_then(|from| self.draft.index(from)), place) {
                (Some(from), Some(to)) if to != from => {
                    match self.draft.join(from, to) {
                        Ok(path) => {
                            self.selected = Selected::Path(path);
                            changed = true;
                        }
                        Err(problem) => said = Some(problem),
                    }
                    self.tool = Tool::Join(Some(self.number(to)));
                }
                (Some(_), Some(_)) => self.tool = Tool::Join(None),
                (None, Some(index)) => {
                    let number = self.number(index);
                    self.tool = Tool::Join(Some(number));
                    self.selected = Selected::Place(number);
                }
                (Some(_), None) => {
                    self.tool = Tool::Pick;
                    self.selected = Selected::Nothing;
                }
                (None, None) => {}
            },
            Tool::Pick => {
                self.selected = match (place, self.draft.path_at(at)) {
                    (Some(index), _) => Selected::Place(self.number(index)),
                    (None, Some(path)) => Selected::Path(path),
                    (None, None) => Selected::Nothing,
                };
            }
        }
        (said, changed)
    }

    /// What is under the pointer, said over it.
    fn hover(&self, at: (f32, f32)) -> Option<String> {
        let draft = &self.draft;
        let from = match self.tool {
            Tool::Join(Some(from)) => draft.index(from),
            _ => None,
        };
        let label = match (self.tool, from, draft.place_at(at)) {
            (Tool::Put(site), _, None) => format!("Put {} here", site.name()),
            (_, Some(from), Some(to)) if from != to => format!(
                "Join {} to {}",
                draft.places[from].called(),
                draft.places[to].called()
            ),
            (_, _, Some(index)) => {
                let place = &draft.places[index];
                let mut label = place.called();
                if index == draft.start {
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
        Some(label)
    }
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
        Drafts::open(self.data.as_deref(), DRAFTS)
    }

    /// The adventures being made, by name and title, the latest worked on first: read from the
    /// drafts kept the first time they are asked for, and again only after one is kept or thrown
    /// away, not every time the tray is drawn.
    fn adventure_drafts(&mut self) -> &[(String, String)] {
        if self.woods.begun.is_none() {
            let drafts = self.drafts();
            let begun = drafts
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
                .collect();
            self.woods.begun = Some(begun);
        }
        self.woods.begun.as_deref().unwrap_or_default()
    }

    /// The Woods' tray's way into the map editor: a new adventure, or one being made.
    pub(super) fn making_choice(&mut self, ui: &mut egui::Ui) {
        let begun = self.adventure_drafts().to_vec();
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
    pub(super) fn begin_adventure(&mut self) {
        let key = format!(
            "{}{}",
            self.arrival.cast.snapshot.colony_id,
            self.started.elapsed().as_nanos()
        );
        let stamp: u64 = key.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        });
        let name = format!("adventure-{:08x}", stamp as u32);
        let mut draft = Draft::new(&name, "");
        if let Some(last) = self
            .adventure_drafts()
            .first()
            .map(|(name, _)| name.clone())
            .and_then(|name| self.drafts().load(&name))
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
        let Some(draft) = self
            .drafts()
            .load(name)
            .and_then(|text| Draft::from_text(&text))
        else {
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

    /// For review: an adventure half made, never kept, with the `chapter`th place picked out, or
    /// past the places, a path.
    pub(super) fn open_sample_making(&mut self, chapter: Option<usize>) {
        let draft = Draft::sample();
        let places = draft.places.len();
        let selected = match chapter {
            Some(n) if n >= 1 && n <= places => Selected::Place(draft.places[n - 1].number),
            Some(n) if n > places && n - places <= draft.paths.len() => {
                Selected::Path(n - places - 1)
            }
            _ => Selected::Nothing,
        };
        self.woods.activity = super::rummaging::Activity::Adventure;
        self.woods.making = Some(Making {
            draft,
            // Never kept: no draft has this name.
            name: String::new(),
            tool: Tool::Pick,
            selected,
            dragging: None,
            picture: None,
        });
    }

    /// Keeps the draft as it stands, so nothing made is lost.
    fn keep_draft(&mut self) {
        let Some(making) = &self.woods.making else {
            return;
        };
        if making.name.is_empty() {
            return;
        }
        if let Err(error) = self.drafts().save(&making.name, &making.draft.to_text()) {
            eprintln!("formiga-hill: could not keep the adventure being made: {error}");
        }
        // Its title, and which was worked on last, may have changed.
        self.woods.begun = None;
    }

    /// The map being made, with whatever is picked out marked on it.
    pub(super) fn compose_making(&mut self) -> Option<Canvas> {
        if self.woods.expedition.is_some() {
            return None;
        }
        let pointer = self.pointer;
        let making = self.woods.making.as_mut()?;
        let map = making.draft.map();
        let stale = making
            .picture
            .as_ref()
            .is_none_or(|(drawn, _)| *drawn != map);
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
            crate::expedition::draw_way(
                &mut scene,
                &map.walk(index, map.paths[index].ends.0),
                true,
                true,
            );
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
        // The map's places stand in the same order as the draft's.
        if let Selected::Place(number) = making.selected
            && let Some(place) = making.draft.index(number).and_then(|i| map.places.get(i))
        {
            ring(&mut scene, place.at, rgb(0xf5d25e));
        }
        if let Tool::Join(Some(from)) = making.tool
            && let Some(place) = making.draft.index(from).and_then(|i| map.places.get(i))
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
        let (said, changed) = making.click(at);
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
    pub(super) fn making_drag(
        &mut self,
        pointer: Option<(f32, f32)>,
        started: bool,
        stopped: bool,
    ) {
        let Some(making) = &mut self.woods.making else {
            return;
        };
        if started {
            making.dragging = pointer
                .and_then(|at| making.draft.place_at(at))
                .map(|index| making.number(index));
            if let Some(number) = making.dragging {
                making.selected = Selected::Place(number);
            }
        }
        if let (Some(index), Some(at)) =
            (making.dragging.and_then(|n| making.draft.index(n)), pointer)
        {
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
        let label = making.hover(at)?;
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
            making.tool = if joining {
                Tool::Pick
            } else {
                Tool::Join(None)
            };
        }
        let picking = making.tool == Tool::Pick;
        if ui
            .add(paper::Button::new("Pick").selected(picking))
            .clicked()
        {
            making.tool = Tool::Pick;
        }
        // Undo and Redo, over the map and every word on it. Not over its version, which counts
        // what has been kept or shared: it stays as it is, so no version is ever given out twice.
        let history = &mut self.woods.making_history;
        let version = std::mem::take(&mut making.draft.version);
        history.feed(ui.ctx(), &making.name, &making.draft);
        let back = history.buttons(ui, &making.draft);
        making.draft.version = version;
        let undone = back.is_some_and(|back| {
            making.step_to(back);
            true
        });
        let hint = match making.tool {
            Tool::Put(_) => "Click the map to put it there. Drag a place to move it.",
            Tool::Join(None) => "Click a place, then another, to join them with a path.",
            Tool::Join(Some(_)) => "Click another place to join them, or the map to stop.",
            Tool::Pick => "Click a place or a path to change it. Drag a place to move it.",
        };
        paper::hint(ui, hint);
        if undone {
            self.keep_draft();
        }
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
        self.show_card(ctx, Card::Making, "Making an adventure", 0.4, |app, ui| {
            let Some(making) = &mut app.woods.making else {
                return;
            };
            let draft = &mut making.draft;
            // Whatever is picked out on the map, or else the adventure itself.
            let place = match making.selected {
                Selected::Place(number) => draft.index(number),
                _ => None,
            };
            let path = match making.selected {
                Selected::Path(index) if index < draft.paths.len() => Some(index),
                _ => None,
            };
            match (place, path) {
                (Some(index), _) => {
                    changed |= place_card(ui, draft, index, &mut making.selected);
                }
                (None, Some(index)) => {
                    changed |= path_card(ui, draft, index, &mut making.selected);
                }
                (None, None) => {
                    paper::faint(ui, "Called");
                    changed |= field(
                        ui,
                        "title",
                        &mut draft.title,
                        "What is it called?",
                        MAX_TITLE_CHARS,
                    );
                    paper::faint(ui, "About it");
                    changed |= field(
                        ui,
                        "about",
                        &mut draft.about,
                        "A line about it, for choosing it",
                        MAX_LINE_CHARS,
                    );
                    paper::faint(ui, "Made by");
                    changed |= field(
                        ui,
                        "author",
                        &mut draft.author,
                        "Your name",
                        MAX_TITLE_CHARS,
                    );
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
                if paper::explain(
                    ui,
                    try_it,
                    "Play it now with whoever is chosen; nothing is kept",
                )
                .clicked()
                {
                    act = Some(Act::Try);
                }
                let keep = ui.add(paper::Button::new("Keep it in the Woods").enabled(ready));
                if paper::explain(
                    ui,
                    keep,
                    "Add it to the Woods' adventures, to play any time",
                )
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
        if let Some(making) = &mut self.woods.making {
            making.tidy();
        }
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
                    self.woods.begun = None;
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
                self.reload_library();
                self.woods.plan = None;
                self.woods.adventure = Some(key);
                let line = if installed.replaced.is_some() {
                    format!("{title} is kept in the Woods, in place of the last version.")
                } else {
                    format!("{title} is kept in the Woods: choose it under Adventure.")
                };
                self.notice = Some((line, now));
            }
            Err(problem) => {
                self.notice = Some((format!("It couldn't be kept: {}", problem.problem), now))
            }
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
                path.file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
            ),
            Err(problem) => format!("It couldn't be saved: {}", problem.problem),
        };
        self.notice = Some((line, now));
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
    if paper::back(ui, &place.called()) {
        *selected = Selected::Nothing;
    }
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
    paper::faint(ui, "Called");
    changed |= field(
        ui,
        ("name", place.number),
        &mut place.name,
        kind.name(),
        MAX_NAME_CHARS,
    );
    paper::faint(ui, "Said on getting here");
    changed |= field(
        ui,
        ("arrive", place.number),
        &mut place.arrive,
        "Nothing, unless you write something",
        MAX_LINE_CHARS,
    );
    ui.horizontal_wrapped(|ui| {
        if !start {
            let hidden = ui.add(paper::Button::new("Hidden in the mist").selected(place.hidden));
            if paper::explain(
                ui,
                hidden,
                "Out of sight on the map until somebody gets there",
            )
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
        paper::faint(ui, "Said at the end");
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
    if paper::back(ui, &format!("{} to {}", name(ends.0), name(ends.1))) {
        *selected = Selected::Nothing;
    }
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
    changed
}

/// A line to write in, on a card. Says whether its words changed.
fn field(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    text: &mut String,
    hint: &str,
    most: usize,
) -> bool {
    paper::field(ui, id, text, hint, most, None).changed()
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A new adventure in the editor, with a glade and a pool put down after the way in, joining
    /// from the pool as the editor does after putting it.
    fn making() -> Making {
        let mut making = Making {
            draft: Draft::new("abc", "Someone"),
            name: String::new(),
            tool: Tool::Put(Site::Glade),
            selected: Selected::Nothing,
            dragging: None,
            picture: None,
        };
        making.click((150.0, 150.0));
        making.tool = Tool::Put(Site::Pool);
        making.click((290.0, 170.0));
        assert_eq!(making.draft.places.len(), 3);
        making
    }

    fn at(making: &Making, index: usize) -> (f32, f32) {
        making.draft.places[index].at
    }

    #[test]
    fn taking_off_the_place_just_put_lets_go_of_it() {
        let mut making = making();
        let pool = making.draft.places[2].number;
        assert_eq!(making.tool, Tool::Join(Some(pool)));
        assert_eq!(making.selected, Selected::Place(pool));
        assert!(making.draft.remove_place(2));
        // Pointing at what is left, before anything else happens, names it plainly.
        let glade = at(&making, 1);
        assert_eq!(making.hover(glade).as_deref(), Some("the glade"));
        making.tidy();
        assert_eq!(making.tool, Tool::Join(None));
        assert_eq!(making.selected, Selected::Nothing);
    }

    #[test]
    fn undoing_the_place_just_put_lets_go_of_it() {
        let mut making = making();
        making.draft.version = 3;
        let pool = making.draft.places[2].number;
        let mut before = making.draft.clone();
        assert!(before.remove_place(2));
        before.version = 0;
        making.dragging = Some(pool);
        making.step_to(before);
        assert_eq!(making.draft.places.len(), 2);
        assert_eq!(making.draft.version, 3, "the version stays as it was");
        assert_eq!(making.tool, Tool::Join(None));
        assert_eq!(making.selected, Selected::Nothing);
        assert_eq!(making.dragging, None);
        // Pointing at what is left, before anything else happens, names it plainly.
        let glade = at(&making, 1);
        assert_eq!(making.hover(glade).as_deref(), Some("the glade"));
    }

    #[test]
    fn taking_off_a_place_never_joins_from_whichever_moved_into_its_place() {
        let mut making = making();
        // Joining from the glade, which is then taken off the map: the pool moves up into its
        // place among the places.
        let glade = at(&making, 1);
        making.tool = Tool::Join(None);
        making.click(glade);
        assert!(making.draft.remove_place(1));
        let way_in = at(&making, 0);
        let said = making.hover(way_in).unwrap();
        assert!(!said.starts_with("Join"), "{said}");
        let paths = making.draft.paths.len();
        making.click(way_in);
        assert_eq!(making.draft.paths.len(), paths, "no path to the pool");
        assert_eq!(making.tool, Tool::Join(Some(making.draft.places[0].number)));
    }
}
