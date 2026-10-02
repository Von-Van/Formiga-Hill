//! The Hill window: the station and the green, the colony in them, and the way home.

use crate::cast::{Cast, Id};
use crate::character::Offer;
use crate::green::Green;
use crate::memories::Memories;
use crate::station::{Journey, SCENE_HEIGHT, SCENE_WIDTH, STAND_Y, Station};
use crate::story::{Director, Library, souvenirs};
use crate::trip::Trip;
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Theme;
use std::time::{Duration, Instant};

/// Who has come, and how.
pub struct Arrival {
    pub cast: Cast,
    pub visit: Visit,
}

pub enum Visit {
    /// A real trip: Desktop is waiting for the colony to come home.
    Trip(Trip),
    /// A development visit, from the sample colony or a colony file read for testing. Nobody is
    /// waiting for it, so it writes nothing.
    Rehearsal(String),
}

/// How often Hill looks for Desktop calling the colony home.
const RECALL_CHECK_SECS: f32 = 1.0;
/// How long moving between areas takes, as a pixel dissolve.
const DISSOLVE_SECS: f32 = 0.5;

/// Where the person at the Hill is looking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Area {
    Station,
    Green,
}

pub struct HillApp {
    arrival: Arrival,
    station: Station,
    /// Made the first time anyone goes there, and kept for the rest of the visit.
    green: Option<Green>,
    area: Area,
    /// What the person is holding out on the green.
    tool: Offer,
    /// The last picture of the area just left, dissolving into the new one.
    leaving: Option<(Canvas, f32)>,
    texture: Option<egui::TextureHandle>,
    shown_frames: Vec<usize>,
    started: Instant,
    /// The visit is over, one way or another, and nothing more is written.
    departed: bool,
    last_recall_check: f32,
    library: Library,
    /// The story being played on the green, if any, and the package it came from.
    story: Option<(String, Director)>,
    memories: Memories,
    /// A short note in the bottom bar, and when it was posted.
    notice: Option<(String, f32)>,
}

/// How long a notice stays in the bottom bar.
const NOTICE_SECS: f32 = 8.0;

const INK: egui::Color32 = egui::Color32::from_rgb(0x4a, 0x36, 0x26);
const PAPER: egui::Color32 = egui::Color32::from_rgb(0xf6, 0xee, 0xd8);
const LETTERBOX: egui::Color32 = egui::Color32::from_rgb(0x2f, 0x3b, 0x2c);
const TAG: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0xf6, 0xee, 0xd8, 0xe0);

impl HillApp {
    pub fn new(cc: &eframe::CreationContext<'_>, arrival: Arrival, library: Library) -> Self {
        let presentation = arrival.cast.snapshot.presentation;
        cc.egui_ctx.set_theme(match presentation.theme {
            Theme::System => egui::ThemePreference::System,
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
        });
        cc.egui_ctx
            .set_zoom_factor(f32::from(presentation.text_scale_percent.clamp(100, 150)) / 100.0);
        // Every visit begins with the train pulling in.
        let station = Station::new(&arrival.cast, Journey::Arriving { since: 0.0 });
        let mut memories = Memories::open(
            Memories::folder().as_deref(),
            &arrival.cast.snapshot.colony_id,
        );
        memories.arrived();
        for problem in &library.problems {
            eprintln!("formiga-hill: a package was not loaded: {problem}");
        }
        Self {
            arrival,
            station,
            green: None,
            area: Area::Station,
            tool: Offer::Pet,
            leaving: None,
            texture: None,
            shown_frames: Vec::new(),
            started: Instant::now(),
            departed: false,
            last_recall_check: 0.0,
            library,
            story: None,
            memories,
            notice: None,
        }
    }

    fn start_story(&mut self, package: &str, story: &str, now: f32) {
        let Some(green) = &mut self.green else {
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
                green.reserve(director.players());
                self.story = Some((package.to_owned(), director));
            }
            Err(problem) => self.notice = Some((problem, now)),
        }
    }

    /// Plays the story on, and settles it once it ends.
    fn run_story(&mut self, now: f32) {
        let (Some(green), Some((package, director))) = (&mut self.green, &mut self.story) else {
            return;
        };
        director.run(green, &self.arrival.cast, now);
        green.set_speaker(
            director
                .shown()
                .and_then(|shown| shown.speaker.as_ref().map(|(id, _)| *id)),
        );
        if director.finished() {
            green.release();
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
            self.story = None;
        }
    }

    fn leave_story(&mut self) {
        if let Some(green) = &mut self.green {
            green.release();
        }
        self.story = None;
    }

    /// The way home. Writes the receipt once, however the visit ends.
    fn depart(&mut self) {
        if std::mem::replace(&mut self.departed, true) {
            return;
        }
        if let Visit::Trip(trip) = &self.arrival.visit
            && let Err(error) = trip.come_home()
        {
            // Desktop treats a trip with no receipt as one that came home unchanged.
            eprintln!("formiga-hill: could not write the receipt: {error}");
        }
    }

    /// Whether Desktop has taken the colony home already, checked now and then.
    fn recalled(&mut self, now: f32) -> bool {
        let Visit::Trip(trip) = &self.arrival.visit else {
            return false;
        };
        if now - self.last_recall_check < RECALL_CHECK_SECS {
            return false;
        }
        self.last_recall_check = now;
        trip.recalled()
    }

    /// Seconds since the window opened: the clock the whole visit runs on.
    fn now(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    fn go_to(&mut self, area: Area, now: f32) {
        if area == self.area {
            return;
        }
        if !self.arrival.cast.reduce_motion() {
            self.leaving = Some((self.compose(now), now));
        }
        if area == Area::Green && self.green.is_none() {
            self.green = Some(Green::new(&self.arrival.cast, now));
        }
        self.area = area;
        self.shown_frames.clear();
    }

    fn compose(&mut self, now: f32) -> Canvas {
        match (self.area, &mut self.green) {
            (Area::Green, Some(green)) => green.compose(now),
            _ => self.station.compose(now),
        }
    }

    fn refresh_scene(&mut self, ctx: &egui::Context, now: f32) -> egui::TextureId {
        // The green is always on the move; the station only when its key says so.
        let frames = match self.area {
            Area::Station if self.leaving.is_none() => self.station.frame_key(now),
            _ => Vec::new(),
        };
        if self.texture.is_none() || frames.is_empty() || frames != self.shown_frames {
            let mut canvas = self.compose(now);
            if let Some((before, since)) = &self.leaving {
                let progress = (now - since) / DISSOLVE_SECS;
                if progress >= 1.0 {
                    self.leaving = None;
                } else {
                    dissolve(&mut canvas, before, progress);
                }
            }
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [SCENE_WIDTH as usize, SCENE_HEIGHT as usize],
                &canvas.rgba_bytes(),
            );
            match &mut self.texture {
                Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
                None => {
                    self.texture =
                        Some(ctx.load_texture("scene", image, egui::TextureOptions::NEAREST));
                }
            }
            self.shown_frames = frames;
        }
        self.texture
            .as_ref()
            .map_or(egui::TextureId::default(), |texture| texture.id())
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        ui.add_space(6.0);
        ui.horizontal(|ui| match self.area {
            Area::Station => {
                ui.label(arrivals_line(&self.arrival));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let leaving = matches!(self.station.journey(), Journey::Leaving { .. });
                    let home = egui::Button::new("Take the train home");
                    if ui.add_enabled(!leaving, home).clicked() {
                        self.station.skip_arrival();
                        self.station.set_off_home(now);
                    }
                    let settled = self.station.is_settled();
                    let green = egui::Button::new("Walk to the Village Green");
                    if ui.add_enabled(settled, green).clicked() {
                        self.go_to(Area::Green, now);
                    }
                });
            }
            Area::Green if self.story.is_some() => self.story_panel(ui),
            Area::Green => {
                ui.label("Hold out:");
                for (offer, label, key) in TOOLS {
                    let chosen = self.tool == offer;
                    let response = ui.selectable_label(chosen, format!("{label}  {key}"));
                    if response.clicked() {
                        self.tool = offer;
                    }
                }
                ui.separator();
                self.stories_menu(ui, now);
                if let Some((notice, _)) = &self.notice {
                    ui.label(egui::RichText::new(notice).italics());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Back to the station").clicked() {
                        self.go_to(Area::Station, now);
                    }
                });
            }
        });
        ui.add_space(6.0);
    }

    fn stories_menu(&mut self, ui: &mut egui::Ui, now: f32) {
        let travellers = self.arrival.cast.members.len();
        let mut start = None;
        ui.menu_button("Stories", |ui| {
            for (package, story) in self.library.stories() {
                let finished = self
                    .memories
                    .colony()
                    .stories
                    .contains(&format!("{}/{}", package.id, story.id));
                let label = if finished {
                    format!("{}  \u{2713}", story.title)
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
                if button.clicked() {
                    start = Some((package.id.clone(), story.id.clone()));
                    ui.close();
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
        });
        if let Some((package, story)) = start {
            self.start_story(&package, &story, now);
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

const TOOLS: [(Offer, &str, &str); 3] = [
    (Offer::Pet, "A pat", "1"),
    (Offer::Snack, "A snack", "2"),
    (Offer::Toy, "A toy", "3"),
];

impl eframe::App for HillApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = self.now();
        self.station.update(now);
        if !self.departed && self.recalled(now) {
            // Desktop has the colony already: close, and write nothing.
            self.departed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.station.gone_home(now) && !self.departed {
            self.depart();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.input(|input| {
            let onwards =
                input.key_pressed(egui::Key::Space) || input.key_pressed(egui::Key::Enter);
            match &mut self.story {
                Some((_, director)) => {
                    if onwards && director.shown().is_some() {
                        director.read_on();
                    }
                    let numbers = [
                        egui::Key::Num1,
                        egui::Key::Num2,
                        egui::Key::Num3,
                        egui::Key::Num4,
                    ];
                    for (index, key) in numbers.into_iter().enumerate() {
                        if input.key_pressed(key) && index < director.choices().len() {
                            director.choose(index);
                        }
                    }
                }
                None => {
                    if onwards || input.key_pressed(egui::Key::Escape) {
                        self.station.skip_arrival();
                    }
                    for (key, (offer, _, _)) in [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3]
                        .into_iter()
                        .zip(TOOLS)
                    {
                        if input.key_pressed(key) {
                            self.tool = offer;
                        }
                    }
                }
            }
        });
        if self
            .notice
            .as_ref()
            .is_some_and(|(_, since)| now - since > NOTICE_SECS)
        {
            self.notice = None;
        }
        if let (Area::Green, Some(green)) = (self.area, &mut self.green) {
            green.tick(&self.arrival.cast, now);
        }
        self.run_story(now);
        let texture = self.refresh_scene(&ctx, now);

        egui::Panel::bottom("platform").show(ui, |ui| self.bottom_bar(ui, now));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(LETTERBOX))
            .show(ui, |ui| {
                let available = ui.available_rect_before_wrap();
                let rect = scene_rect(available, ctx.pixels_per_point());
                let response = ui.allocate_rect(rect, egui::Sense::click());
                let painter = ui.painter_at(rect);
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                painter.image(texture, rect, uv, egui::Color32::WHITE);

                let point = rect.width() / SCENE_WIDTH as f32;
                let to_screen = |x: f32, y: f32| rect.min + egui::vec2(x, y) * point;
                let pointer = response.hover_pos().map(|pos| {
                    let scene = (pos - rect.min) / point;
                    (scene.x, scene.y)
                });
                let font = egui::FontId::proportional((5.5 * point).clamp(10.0, 20.0));
                let tag = |text: &str, centre: egui::Pos2| {
                    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), INK);
                    let label = egui::Rect::from_center_size(
                        centre,
                        galley.size() + egui::vec2(point * 4.0, point * 1.0),
                    );
                    painter.rect_filled(label, point * 2.0, TAG);
                    painter.galley(label.center() - galley.size() / 2.0, galley, INK);
                };

                let hovered = match (self.area, &mut self.green) {
                    (Area::Green, Some(green)) => {
                        green.set_pointer(pointer);
                        let hovered = pointer.and_then(|(x, y)| green.actor_at(x, y, now));
                        if let Some((_, director)) = &mut self.story {
                            // During a story, a click on the scene reads on.
                            if response.clicked() && director.shown().is_some() {
                                director.read_on();
                            }
                        } else if let Some(id) = hovered {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            if response.clicked() {
                                green.offer(id, self.tool, now);
                            }
                        }
                        if let Some(id) = hovered {
                            // Only the one under the pointer wears a name on the green.
                            if let (Some((x, y)), Some(member)) =
                                (green.head(id, now), self.arrival.cast.member(id))
                            {
                                tag(&member.name, to_screen(x, y - 6.0));
                            }
                        }
                        hovered
                    }
                    _ => {
                        if response.clicked() {
                            // A click on the station skips the arrival.
                            self.station.skip_arrival();
                        }
                        // Name tags on little cream labels, so they read over boards and stone
                        // alike, once everyone is standing still to wear them.
                        if self.station.is_settled() && self.leaving.is_none() {
                            for traveler in self.station.travelers() {
                                let (left, _, right, _) = traveler.bounds;
                                let centre = to_screen(
                                    (left + right + 1) as f32 / 2.0,
                                    STAND_Y as f32 + 11.0,
                                );
                                tag(&traveler.name, centre);
                            }
                        }
                        pointer.and_then(|(x, y)| self.station.traveler_at(x, y).map(|t| t.id))
                    }
                };
                if let Some(id) = hovered {
                    let lines = describe(&self.arrival.cast, id);
                    response.on_hover_ui_at_pointer(|ui| {
                        for (index, line) in lines.iter().enumerate() {
                            if index == 0 {
                                ui.strong(line);
                            } else {
                                ui.label(line);
                            }
                        }
                    });
                }
            });

        if self.area == Area::Green || self.leaving.is_some() || self.station.in_motion(now) {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if matches!(self.arrival.visit, Visit::Trip(_)) {
            // Keep looking for a recall even when nothing on screen moves.
            ctx.request_repaint_after(Duration::from_secs_f32(RECALL_CHECK_SECS));
        } else if !self.arrival.cast.reduce_motion() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self) {
        self.depart();
    }
}

/// Mixes `before` into `scene` in an ordered dither, `progress` of the way to `scene`: a pixel
/// art dissolve, with no colour in it that neither picture has.
fn dissolve(scene: &mut Canvas, before: &Canvas, progress: f32) {
    const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let shown = (progress.clamp(0.0, 1.0) * 16.0) as u8;
    for y in 0..scene.height() as i32 {
        for x in 0..scene.width() as i32 {
            if BAYER[(y % 4) as usize][(x % 4) as usize] >= shown {
                scene.set(x, y, before.get(x, y));
            }
        }
    }
}

/// The largest rectangle the scene fits in at a whole number of screen pixels per scene pixel,
/// so the art stays crisp, centred in `available`.
fn scene_rect(available: egui::Rect, pixels_per_point: f32) -> egui::Rect {
    let fit = (available.width() * pixels_per_point / SCENE_WIDTH as f32)
        .min(available.height() * pixels_per_point / SCENE_HEIGHT as f32);
    let pixels = if fit >= 1.0 {
        fit.floor()
    } else {
        fit.max(0.1)
    };
    let size = egui::vec2(SCENE_WIDTH as f32, SCENE_HEIGHT as f32) * pixels / pixels_per_point;
    egui::Rect::from_center_size(available.center(), size)
}

fn arrivals_line(arrival: &Arrival) -> String {
    let names: Vec<&str> = arrival
        .cast
        .members
        .iter()
        .map(|member| member.name.as_str())
        .collect();
    let names = match names.as_slice() {
        [] => String::new(),
        [only] => (*only).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    match &arrival.visit {
        Visit::Trip(_) => format!("{names} came on the train."),
        Visit::Rehearsal(label) => format!("{names} ({label})"),
    }
}

/// What the station knows about one traveller, for its tooltip: everything here came from the
/// snapshot, in Desktop's own words where Desktop has them.
fn describe(cast: &Cast, id: Id) -> Vec<String> {
    let Some(member) = cast.member(id) else {
        return Vec::new();
    };
    let character = &member.traveler.character;
    let mut lines = vec![member.name.clone(), character.phrase.clone()];
    if !character.traits.is_empty() {
        lines.push(character.traits.join(", "));
    }
    if let Some(parent) = member.parent() {
        lines.push(match cast.member(parent) {
            Some(parent) => format!("{}'s little one", parent.name),
            None => "A little one".to_owned(),
        });
    }
    for habit in &member.traveler.habits {
        lines.push(formiga_core::Habit::from(*habit).label().to_owned());
    }
    if let Some(accessory) = member.traveler.accessory {
        lines.push(format!(
            "Wearing a {}",
            accessory.to_accessory().label().to_lowercase()
        ));
    }
    if let Some(friend) = cast
        .closest_friend(id, cast.ids())
        .and_then(|id| cast.member(id))
    {
        lines.push(format!("Closest to {}", friend.name));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_scales_by_whole_pixels_and_fits() {
        let available = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1300.0, 700.0));
        let rect = scene_rect(available, 2.0);
        assert_eq!(rect.width() * 2.0 % SCENE_WIDTH as f32, 0.0);
        assert!(available.contains_rect(rect));
    }

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    #[test]
    fn a_tooltip_reads_like_desktops_notebook() {
        let cast = sample();
        for member in &cast.members {
            let lines = describe(&cast, member.id);
            assert_eq!(lines[0], member.name);
            assert_eq!(lines[1], member.traveler.character.phrase);
            if member.traveler.accessory.is_some() {
                assert!(lines.iter().any(|line| line.starts_with("Wearing a ")));
            }
            if member.parent().is_some() {
                assert!(lines.iter().any(|line| line.ends_with("little one")));
            }
        }
        let friendly = cast.members.iter().any(|member| {
            describe(&cast, member.id)
                .iter()
                .any(|line| line.starts_with("Closest to "))
        });
        assert!(friendly, "someone in the sample has a close friend");
    }

    #[test]
    fn a_dissolve_runs_from_one_picture_to_the_other() {
        let mut from = Canvas::new(8, 8);
        from.fill_rect(0, 0, 8, 8, formiga_art::Rgba::new(255, 0, 0, 255));
        let to = Canvas::new(8, 8);
        let at = |progress| {
            let mut scene = to.clone();
            dissolve(&mut scene, &from, progress);
            scene
        };
        assert_eq!(at(0.0), from);
        assert_eq!(at(1.0), to);
        let halfway = at(0.5)
            .pixels()
            .iter()
            .filter(|pixel| pixel.r == 255)
            .count();
        assert_eq!(halfway, 32);
    }

    #[test]
    fn the_arrivals_line_lists_everyone() {
        let arrival = Arrival {
            cast: sample(),
            visit: Visit::Rehearsal("sample colony".to_owned()),
        };
        let line = arrivals_line(&arrival);
        for member in &arrival.cast.members {
            assert!(line.contains(&member.name));
        }
        assert!(line.contains(" and "));
    }
}
