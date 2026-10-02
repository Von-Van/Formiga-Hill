//! The Hill window: the station, the colony standing on it, and the way home.

use crate::station::{SCENE_HEIGHT, SCENE_WIDTH, SIGN_CENTER, STAND_Y, Station};
use eframe::egui;
use formiga_core::CreatureId;
use formiga_travel::{
    Completion, Outing, ReturnReceipt, Theme, TravelSnapshot, Traveler, write_receipt,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

/// How the colony got here.
pub enum Arrival {
    /// A real trip: Desktop is waiting for a receipt at `receipt`.
    Trip {
        snapshot: TravelSnapshot,
        receipt: PathBuf,
    },
    /// A development visit, from the sample colony or a colony file read for testing. Nobody is
    /// waiting for it to come home, so it writes nothing.
    Visit {
        snapshot: TravelSnapshot,
        label: String,
    },
}

impl Arrival {
    pub fn snapshot(&self) -> &TravelSnapshot {
        match self {
            Self::Trip { snapshot, .. } | Self::Visit { snapshot, .. } => snapshot,
        }
    }
}

pub struct HillApp {
    arrival: Arrival,
    station: Station,
    texture: Option<egui::TextureHandle>,
    shown_frames: Vec<usize>,
    started: Instant,
    departed: bool,
}

const INK: egui::Color32 = egui::Color32::from_rgb(0x4a, 0x36, 0x26);
const LETTERBOX: egui::Color32 = egui::Color32::from_rgb(0x2f, 0x3b, 0x2c);

impl HillApp {
    pub fn new(cc: &eframe::CreationContext<'_>, arrival: Arrival) -> Self {
        let presentation = arrival.snapshot().presentation;
        cc.egui_ctx.set_theme(match presentation.theme {
            Theme::System => egui::ThemePreference::System,
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
        });
        cc.egui_ctx
            .set_zoom_factor(f32::from(presentation.text_scale_percent.clamp(100, 150)) / 100.0);
        let station = Station::new(arrival.snapshot());
        Self {
            arrival,
            station,
            texture: None,
            shown_frames: Vec::new(),
            started: Instant::now(),
            departed: false,
        }
    }

    /// The way home. Writes the receipt once, however the visit ends.
    fn depart(&mut self) {
        if std::mem::replace(&mut self.departed, true) {
            return;
        }
        let Arrival::Trip { snapshot, receipt } = &self.arrival else {
            return;
        };
        let mut answer = ReturnReceipt::new(
            snapshot.session_id,
            env!("CARGO_PKG_VERSION"),
            OffsetDateTime::now_utc(),
            Completion::Clean,
        );
        answer.outing = Some(Outing {
            package_id: None,
            title: "Spent time at Formiga Hill".to_owned(),
        });
        if let Err(error) = write_receipt(receipt, &answer, snapshot) {
            // Desktop treats a trip with no receipt as one that came home with nothing.
            eprintln!("formiga-hill: could not write the return receipt: {error}");
        }
    }

    fn refresh_scene(&mut self, ctx: &egui::Context) -> egui::TextureId {
        let elapsed = self.started.elapsed().as_secs_f32();
        let frames = self.station.frame_key(elapsed);
        if self.texture.is_none() || frames != self.shown_frames {
            let canvas = self.station.compose(elapsed);
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [SCENE_WIDTH as usize, SCENE_HEIGHT as usize],
                &canvas.rgba_bytes(),
            );
            match &mut self.texture {
                Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
                None => {
                    self.texture =
                        Some(ctx.load_texture("station", image, egui::TextureOptions::NEAREST));
                }
            }
            self.shown_frames = frames;
        }
        self.texture
            .as_ref()
            .map_or(egui::TextureId::default(), |texture| texture.id())
    }
}

impl eframe::App for HillApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let texture = self.refresh_scene(&ctx);

        egui::Panel::bottom("platform").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(arrivals_line(&self.arrival));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let leave = match self.arrival {
                        Arrival::Trip { .. } => "Take the train home",
                        Arrival::Visit { .. } => "Close",
                    };
                    if ui.button(leave).clicked() {
                        self.depart();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(LETTERBOX))
            .show(ui, |ui| {
                let available = ui.available_rect_before_wrap();
                let rect = scene_rect(available, ctx.pixels_per_point());
                let response = ui.allocate_rect(rect, egui::Sense::hover());
                let painter = ui.painter_at(rect);
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                painter.image(texture, rect, uv, egui::Color32::WHITE);

                let point = rect.width() / SCENE_WIDTH as f32;
                let to_screen = |x: f32, y: f32| rect.min + egui::vec2(x, y) * point;
                painter.text(
                    to_screen(SIGN_CENTER.0 as f32, SIGN_CENTER.1 as f32 + 0.5),
                    egui::Align2::CENTER_CENTER,
                    "Formiga Hill",
                    egui::FontId::proportional((11.0 * point).max(10.0)),
                    INK,
                );
                for traveler in self.station.travelers() {
                    let (left, _, right, _) = traveler.bounds;
                    painter.text(
                        to_screen((left + right + 1) as f32 / 2.0, STAND_Y as f32 + 9.0),
                        egui::Align2::CENTER_TOP,
                        &traveler.name,
                        egui::FontId::proportional((6.5 * point).clamp(10.0, 22.0)),
                        INK,
                    );
                }

                let hovered = response.hover_pos().and_then(|pos| {
                    let scene = (pos - rect.min) / point;
                    self.station.traveler_at(scene.x, scene.y).map(|t| t.id)
                });
                if let Some(id) = hovered {
                    let lines = describe(self.arrival.snapshot(), id);
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

        if !self.arrival.snapshot().presentation.reduce_motion {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self) {
        self.depart();
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
        .snapshot()
        .travelers
        .iter()
        .map(|traveler| traveler.name.as_str())
        .collect();
    let names = match names.as_slice() {
        [] => String::new(),
        [only] => (*only).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    match arrival {
        Arrival::Trip { .. } => format!("{names} came on the train."),
        Arrival::Visit { label, .. } => format!("{names} ({label})"),
    }
}

/// What the station knows about one traveller, for its tooltip: everything here came from the
/// snapshot, so it reads the same as Desktop's notebook would.
fn describe(snapshot: &TravelSnapshot, id: CreatureId) -> Vec<String> {
    let Some(traveler) = snapshot.traveler(id) else {
        return Vec::new();
    };
    let mut lines = vec![
        traveler.name.clone(),
        traveler.temperament.kind.label().to_owned(),
    ];
    if !traveler.traits.is_empty() {
        let traits: Vec<_> = traveler.traits.iter().map(|t| t.label()).collect();
        lines.push(traits.join(", "));
    }
    if let Some(parent) = traveler.parent_id() {
        lines.push(match snapshot.traveler(parent) {
            Some(parent) => format!("{}'s little one", parent.name),
            None => "A little one".to_owned(),
        });
    }
    for habit in &traveler.habits {
        lines.push(format!("Habit: {}", habit.label()));
    }
    if let Some(worn) = traveler.accessory {
        lines.push(format!("Wearing: {}", worn.item.label()));
    }
    if let Some(friend) = closest_friend(snapshot, traveler) {
        lines.push(format!("Closest to {}", friend.name));
    }
    lines
}

fn closest_friend<'a>(snapshot: &'a TravelSnapshot, traveler: &Traveler) -> Option<&'a Traveler> {
    snapshot
        .bonds
        .iter()
        .filter(|bond| bond.contains(traveler.id))
        .filter(|bond| bond.warmth >= formiga_travel::Level::Medium && bond.friction < bond.warmth)
        .max_by_key(|bond| (bond.warmth, bond.familiarity))
        .and_then(|bond| snapshot.traveler(bond.other(traveler.id)?))
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

    #[test]
    fn a_tooltip_names_temperament_and_closest_friend() {
        let snapshot = formiga_travel::sample::snapshot();
        let first = &snapshot.travelers[0];
        let lines = describe(&snapshot, first.id);
        assert_eq!(lines[0], first.name);
        assert_eq!(lines[1], first.temperament.kind.label());
        assert!(lines.iter().any(|line| line.starts_with("Closest to ")));
        assert!(lines.iter().any(|line| line.starts_with("Wearing: ")));
    }

    #[test]
    fn the_arrivals_line_lists_everyone() {
        let arrival = Arrival::Visit {
            snapshot: formiga_travel::sample::snapshot(),
            label: "sample colony".to_owned(),
        };
        let line = arrivals_line(&arrival);
        for traveler in &arrival.snapshot().travelers {
            assert!(line.contains(&traveler.name));
        }
        assert!(line.contains(" and "));
    }
}
