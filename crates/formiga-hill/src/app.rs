//! The Hill window: the station and the green, the colony in them, and the way home.

use crate::cast::{Cast, Id};
use crate::character::Offer;
use crate::green::Green;
use crate::station::{Journey, SCENE_HEIGHT, SCENE_WIDTH, STAND_Y, Station};
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
}

const INK: egui::Color32 = egui::Color32::from_rgb(0x4a, 0x36, 0x26);
const LETTERBOX: egui::Color32 = egui::Color32::from_rgb(0x2f, 0x3b, 0x2c);
const TAG: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0xf6, 0xee, 0xd8, 0xe0);

impl HillApp {
    pub fn new(cc: &eframe::CreationContext<'_>, arrival: Arrival) -> Self {
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
        }
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
            Area::Green => {
                ui.label("Hold out:");
                for (offer, label, key) in TOOLS {
                    let chosen = self.tool == offer;
                    let response = ui.selectable_label(chosen, format!("{label}  {key}"));
                    if response.clicked() {
                        self.tool = offer;
                    }
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
            if input.key_pressed(egui::Key::Space) || input.key_pressed(egui::Key::Escape) {
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
        });
        if let (Area::Green, Some(green)) = (self.area, &mut self.green) {
            green.tick(&self.arrival.cast, now);
        }
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
                        if let Some(id) = hovered {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            if response.clicked() {
                                green.offer(id, self.tool, now);
                            }
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
