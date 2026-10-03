//! The Hill window: the station, the green, the Clubhouse, the Fairground, the Woods and the
//! Hilltop, the colony in them, and the way home.

mod arranging;
mod bug_hunt;
mod camera;
mod departures;
mod encounter;
mod finery;
mod fishing_trip;
mod games;
mod plans;
mod rummaging;
mod storytelling;

use crate::cast::{Cast, Id};
use crate::character::Offer;
use crate::clubhouse::{self, Clubhouse};
use crate::daylight::{Clock, Daylight};
use crate::fairground::{self, Game, Games, Picks};
use crate::memories::Memories;
use crate::playground::{Playground, Trust};
use crate::station::{Fixture, Journey, SCENE_HEIGHT, SCENE_WIDTH, STAND_Y, Station};
use crate::story::shelf::SetAside;
use crate::story::{Director, Library, souvenirs};
use crate::trip::Trip;
use arranging::Placing;
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
/// How often Hill looks at the clock for the light of the hour.
const HOUR_CHECK_SECS: f32 = 10.0;
/// How long moving between areas takes, as a pixel dissolve.
const DISSOLVE_SECS: f32 = 0.5;

/// Where the person at the Hill is looking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Area {
    Station,
    Green,
    Clubhouse,
    Fairground,
    Woods,
    Hilltop,
    /// The clearing that isn't on any map. Not on the "Go to" menu.
    Clearing,
}

/// Every area, as the "Go to" menu lists them.
const AREAS: [(Area, &str); 6] = [
    (Area::Station, "The station"),
    (Area::Green, "The Village Green"),
    (Area::Clubhouse, "The Clubhouse"),
    (Area::Fairground, "The Fairground"),
    (Area::Woods, "The Woods"),
    (Area::Hilltop, "The Hilltop"),
];

pub struct HillApp {
    arrival: Arrival,
    station: Station,
    /// Made the first time anyone goes there, and kept for the rest of the visit.
    green: Option<Playground>,
    /// The same, for the room where stories are staged.
    clubhouse: Option<Clubhouse>,
    /// Whether the notice board of stories is open, and the list of packages behind it.
    board: bool,
    shelf: bool,
    /// Whether the station's notices, or its departures board, are open.
    notices_open: bool,
    departures_open: bool,
    /// The packages the person has set aside, and where community packages go.
    set_aside: SetAside,
    packages_folder: Option<std::path::PathBuf>,
    /// The same, and the games played there.
    fairground: Option<(Playground, Games)>,
    /// The game chosen to watch next, and who the person has picked to play each game.
    game: Game,
    picks: Picks,
    /// Who is going to the Woods, and the outing under way.
    woods: rummaging::Woods,
    /// The summit, made the first time anyone goes up.
    hilltop: Option<Playground>,
    /// The clearing and what is waiting there, while it lasts.
    clearing: Option<(Playground, crate::clearing::sovereign::Sovereign)>,
    /// What the person is about to stand somewhere on the Hilltop.
    placing: Option<Placing>,
    /// The plans, and anything the colony is building on the Hilltop.
    crafting: plans::Crafting,
    /// Where each piece on the Hilltop is drawn, for pointing at them.
    piece_bounds: Vec<(u8, (i32, i32, i32, i32))>,
    /// Whether the journal of finds is open.
    journal: bool,
    /// Where the pointer is over the scene, in scene pixels.
    pointer: Option<(f32, f32)>,
    /// When the last frame was drawn, for anything that moves by how long a key is held.
    last_frame: f32,
    area: Area,
    /// What the person is holding out on the green.
    tool: Offer,
    /// What everyone has on from the dress-up box, and who is groomed till they shine: for the
    /// visit only.
    costumes: std::collections::HashMap<Id, &'static str>,
    groomed: std::collections::HashSet<Id>,
    /// Whether the dress-up box is open, what has been picked out of it, and its pictures.
    dress_up: bool,
    picked: Option<finery::Pick>,
    costume_icons: Vec<egui::TextureHandle>,
    /// Where the brush last was, over whom, and who the pointer is over this frame.
    stroke: Option<(Id, (f32, f32))>,
    hovered: Option<Id>,
    /// The camera and the album, and the scene as last composed, for photos.
    camera: camera::Camera,
    last_scene: Option<Canvas>,
    /// How each traveller has warmed to the person, wherever they have met this visit.
    trust: Trust,
    /// The last picture of the area just left, dissolving into the new one.
    leaving: Option<(Canvas, f32)>,
    texture: Option<egui::TextureHandle>,
    shown_frames: Vec<usize>,
    started: Instant,
    /// The visit is over, one way or another, and nothing more is written.
    departed: bool,
    last_recall_check: f32,
    library: Library,
    /// Where the hours come from, the light of the one it is, and when Hill last looked.
    clock: Clock,
    daylight: Daylight,
    last_hour_check: Option<f32>,
    /// The story being played in the Clubhouse, if any, and the package it came from.
    story: Option<(String, Director)>,
    memories: Memories,
    /// What has grown on the Hilltop since the last visit, as it is now, spot by spot.
    grown: Vec<(u8, crate::hilltop::Standing)>,
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
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        arrival: Arrival,
        library: Library,
        clock: Clock,
    ) -> Self {
        let presentation = arrival.cast.snapshot.presentation;
        cc.egui_ctx.set_theme(match presentation.theme {
            Theme::System => egui::ThemePreference::System,
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
        });
        cc.egui_ctx
            .set_zoom_factor(f32::from(presentation.text_scale_percent.clamp(100, 150)) / 100.0);
        // Every visit begins with the train pulling in.
        let mut memories = Memories::open(
            Memories::folder().as_deref(),
            &arrival.cast.snapshot.colony_id,
        );
        let grown = memories.arrived();
        let mut station = Station::new(
            &arrival.cast,
            Journey::Arriving { since: 0.0 },
            &memories.colony().souvenirs,
        );
        station.show_hilltop(&memories.colony().hilltop);
        for problem in &library.problems {
            eprintln!("formiga-hill: a package was not loaded: {problem}");
        }
        let mut app = Self {
            arrival,
            station,
            green: None,
            clubhouse: None,
            board: false,
            shelf: false,
            notices_open: false,
            departures_open: false,
            set_aside: SetAside::open(Memories::folder().as_deref()),
            packages_folder: Memories::folder().map(|data| crate::story::shelf::folder(&data)),
            fairground: None,
            game: Game::HideAndSeek,
            picks: Picks::default(),
            woods: rummaging::Woods::default(),
            hilltop: None,
            clearing: None,
            placing: None,
            crafting: plans::Crafting::default(),
            piece_bounds: Vec::new(),
            journal: false,
            pointer: None,
            last_frame: 0.0,
            area: Area::Station,
            tool: Offer::Pet,
            costumes: std::collections::HashMap::new(),
            groomed: std::collections::HashSet::new(),
            dress_up: false,
            picked: None,
            costume_icons: Vec::new(),
            stroke: None,
            hovered: None,
            camera: camera::Camera::default(),
            last_scene: None,
            trust: Trust::default(),
            leaving: None,
            texture: None,
            shown_frames: Vec::new(),
            started: Instant::now(),
            departed: false,
            last_recall_check: 0.0,
            library,
            clock,
            daylight: clock.daylight(),
            last_hour_check: None,
            story: None,
            memories,
            grown,
            notice: None,
        };
        app.pin_notices();
        app
    }

    /// The way home. Writes the receipt once, however the visit ends.
    fn depart(&mut self) {
        if std::mem::replace(&mut self.departed, true) {
            return;
        }
        if let Visit::Trip(trip) = &self.arrival.visit
            && let Err(error) = trip.come_home(&self.memories.colony().souvenirs)
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

    /// Looks at the clock now and then, and lights everywhere for the hour it is: everywhere
    /// the colony has been, so a place gone back to is already in the right light.
    fn keep_hours(&mut self, now: f32) {
        if self
            .last_hour_check
            .is_none_or(|since| now - since >= HOUR_CHECK_SECS)
        {
            self.daylight = self.clock.daylight();
            self.last_hour_check = Some(now);
        }
        let daylight = self.daylight;
        self.station.set_daylight(daylight);
        if let Some(room) = &mut self.clubhouse {
            room.set_daylight(daylight);
        }
        let grounds = [
            self.green.as_mut(),
            self.fairground.as_mut().map(|(ground, _)| ground),
            self.hilltop.as_mut(),
            self.woods.empty.as_mut(),
            self.woods.pool.as_mut(),
            self.woods.meadow.as_mut(),
        ];
        for ground in grounds.into_iter().flatten() {
            ground.set_daylight(daylight);
        }
        if let Some((ground, rummage)) = &mut self.woods.outing {
            ground.set_daylight(daylight);
            rummage.set_hour_dark(daylight.darkness());
        }
        if let Some((ground, trip)) = &mut self.woods.fishing {
            ground.set_daylight(daylight);
            trip.set_hour_dark(daylight.darkness());
        }
        if let Some((ground, hunt)) = &mut self.woods.hunt {
            ground.set_daylight(daylight);
            hunt.set_hour_dark(daylight.darkness());
        }
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
            let mut green = crate::green::open(&self.arrival.cast, now);
            crate::green::show_hilltop(&mut green, &self.memories.colony().hilltop);
            self.green = Some(green);
        }
        if area == Area::Clubhouse {
            self.open_clubhouse(now);
        }
        if area == Area::Fairground && self.fairground.is_none() {
            let (mut ground, game) = fairground::open(&self.arrival.cast, now);
            fairground::show_hilltop(&mut ground, &self.memories.colony().hilltop);
            self.fairground = Some((ground, game));
        }
        if area == Area::Woods {
            self.open_woods(now);
        }
        if area == Area::Hilltop {
            self.open_hilltop(now);
        }
        // Leaving the Woods brings the basket home; leaving the Hilltop puts down what was held.
        if self.woods.outing.is_some() {
            self.finish_outing(now);
        }
        if self.woods.fishing.is_some() {
            self.finish_fishing(now);
        }
        if self.woods.hunt.is_some() {
            self.finish_bug_hunt(now);
        }
        self.placing = None;
        self.notices_open = false;
        self.departures_open = false;
        if area == Area::Station {
            self.pin_notices();
        }
        // Walking away calls a game off.
        if let Some((ground, games)) = &mut self.fairground
            && games.playing().is_some()
        {
            games.stop(ground, now);
        }
        self.area = area;
        self.shown_frames.clear();
    }

    fn compose(&mut self, now: f32) -> Canvas {
        match (self.area, &mut self.green, &mut self.fairground) {
            (Area::Green, Some(green), _) => green.compose(now),
            (Area::Clubhouse, _, _) => self
                .clubhouse
                .as_mut()
                .map_or_else(|| self.station.compose(now), |room| room.compose(now)),
            (Area::Fairground, _, Some((ground, _))) => ground.compose(now),
            (Area::Woods, _, _) => self.compose_woods(now),
            (Area::Hilltop, _, _) => self.compose_hilltop(now),
            (Area::Clearing, _, _) => self.compose_clearing(now),
            _ => self.station.compose(now),
        }
    }

    /// Everywhere else the colony can go.
    fn go_menu(&mut self, ui: &mut egui::Ui, now: f32) {
        let mut target = None;
        ui.menu_button("Go to\u{2026}", |ui| {
            for (area, label) in AREAS {
                if area != self.area && ui.button(label).clicked() {
                    target = Some(area);
                    ui.close();
                }
            }
        });
        if let Some(area) = target {
            self.go_to(area, now);
        }
        self.camera_buttons(ui);
    }

    fn refresh_scene(&mut self, ctx: &egui::Context, now: f32) -> egui::TextureId {
        // The green is always on the move; the station only when its key says so, or the hour.
        let frames = match self.area {
            Area::Station if self.leaving.is_none() => {
                let mut key = self.station.frame_key(now);
                key.push((self.daylight.hour() * 60.0) as usize);
                key
            }
            _ => Vec::new(),
        };
        if self.texture.is_none() || frames.is_empty() || frames != self.shown_frames {
            let mut canvas = self.compose(now);
            self.last_scene = Some(canvas.clone());
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
                    ui.add_enabled_ui(settled, |ui| self.go_menu(ui, now));
                });
            }
            Area::Green => {
                tools(ui, &mut self.tool);
                ui.separator();
                if ui
                    .selectable_label(self.dress_up, "The dress-up box")
                    .clicked()
                {
                    self.dress_up = !self.dress_up;
                    self.picked = None;
                }
                if let Some((notice, _)) = &self.notice {
                    ui.label(egui::RichText::new(notice).italics());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.go_menu(ui, now);
                });
            }
            Area::Clubhouse => self.clubhouse_bar(ui, now),
            Area::Fairground => self.fairground_bar(ui, now),
            Area::Woods => {
                self.woods_bar(ui, now);
                if let Some((notice, _)) = &self.notice {
                    ui.label(egui::RichText::new(notice).italics());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.go_menu(ui, now);
                });
            }
            Area::Clearing => self.clearing_bar(ui, now),
            Area::Hilltop => {
                self.hilltop_bar(ui);
                if let Some((notice, _)) = &self.notice {
                    ui.label(egui::RichText::new(notice).italics());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.go_menu(ui, now);
                });
            }
        });
        ui.add_space(6.0);
    }
}

const TOOLS: [(Offer, &str, &str); 4] = [
    (Offer::Pet, "A pat", "1"),
    (Offer::Snack, "A snack", "2"),
    (Offer::Toy, "A toy", "3"),
    (Offer::Brush, "A brush", "4"),
];

/// What the person can hold out, to choose from.
fn tools(ui: &mut egui::Ui, tool: &mut Offer) {
    ui.label("Hold out:");
    for (offer, label, key) in TOOLS {
        let response = ui.selectable_label(*tool == offer, format!("{label}  {key}"));
        if response.clicked() {
            *tool = offer;
        }
    }
}

/// Seconds as a game clock: 0:07, 1:32.
fn clock(seconds: f32) -> String {
    let whole = seconds.max(0.0).round() as u32;
    format!("{}:{:02}", whole / 60, whole % 60)
}

impl eframe::App for HillApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = self.now();
        self.keep_hours(now);
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
            // The camera comes out, and goes away, wherever the person is, a story included.
            if input.key_pressed(egui::Key::C) {
                self.camera.out = !self.camera.out;
            }
            if input.key_pressed(egui::Key::Escape) {
                self.camera.out = false;
            }
            match &mut self.story {
                Some((_, director)) if self.area == Area::Clubhouse => {
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
                _ => {
                    if onwards || input.key_pressed(egui::Key::Escape) {
                        self.station.skip_arrival();
                    }
                    if self.area == Area::Woods && input.key_pressed(egui::Key::Space) {
                        self.woods_strike(now);
                    }
                    if self.area == Area::Clearing {
                        if onwards {
                            self.clearing_read_on();
                        }
                        let numbers = [
                            egui::Key::Num1,
                            egui::Key::Num2,
                            egui::Key::Num3,
                            egui::Key::Num4,
                        ];
                        for (index, key) in numbers.into_iter().enumerate() {
                            if input.key_pressed(key) {
                                self.clearing_choose(index);
                            }
                        }
                    }
                    if input.key_pressed(egui::Key::Escape) {
                        self.placing = None;
                    }
                    if input.key_pressed(egui::Key::Escape)
                        && self.area == Area::Fairground
                        && let Some((ground, games)) = &mut self.fairground
                        && games.playing().is_some()
                    {
                        games.stop(ground, now);
                    }
                    for (key, (offer, _, _)) in [
                        egui::Key::Num1,
                        egui::Key::Num2,
                        egui::Key::Num3,
                        egui::Key::Num4,
                    ]
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
        if let (Area::Clubhouse, Some(room)) = (self.area, &mut self.clubhouse) {
            room.tick(&self.arrival.cast, now);
        }
        if let (Area::Fairground, Some((ground, game))) = (self.area, &mut self.fairground) {
            ground.tick(&self.arrival.cast, now);
            game.tick(ground, now);
        }
        self.fairground_events(now);
        if self.area == Area::Woods {
            // With reduced motion, the catching marker turns while the pointer or Space is held.
            let holding =
                ctx.input(|input| input.pointer.primary_down() || input.key_down(egui::Key::Space));
            // Creeping up on a bug is the pointer held down, or an arrow key or W; Space swings.
            self.woods.creeping = ctx.input(|input| {
                input.pointer.primary_down()
                    || input.key_down(egui::Key::ArrowUp)
                    || input.key_down(egui::Key::W)
            });
            let dt = (now - self.last_frame).clamp(0.0, 0.1);
            self.tick_woods(now, holding, dt);
        }
        if self.area == Area::Hilltop {
            self.tick_hilltop(now);
        }
        if self.area == Area::Clearing {
            self.tick_clearing(now);
        }
        self.last_frame = now;
        self.run_story(now);
        let texture = self.refresh_scene(&ctx, now);

        egui::Panel::bottom("platform").show(ui, |ui| self.bottom_bar(ui, now));

        // Whoever the person clicks with something picked out of the dress-up box, and whether
        // a click took a photo.
        let mut dress_on = None;
        let mut camera_click = false;
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

                let mut fixture = None;
                let mut on_box = false;
                self.pointer = pointer;
                // With the camera out, the scene is only for framing photos.
                camera_click = self.camera.out && response.clicked();
                let hovered = if self.camera.out {
                    None
                } else {
                    match (self.area, &mut self.green, &mut self.fairground) {
                        (Area::Woods, _, _) => {
                            let mut hovered = None;
                            if let Some((ground, _)) = &mut self.woods.outing {
                                ground.set_pointer(pointer);
                                hovered = pointer.and_then(|(x, y)| ground.actor_at(x, y, now));
                            }
                            let spot = self.woods_hover(pointer);
                            if let Some((label, (x, y))) = &spot {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                tag(label, to_screen(*x, *y));
                            }
                            if response.clicked() {
                                self.woods_click(pointer, now);
                            }
                            hovered.filter(|_| spot.is_none())
                        }
                        (Area::Clearing, _, _) => {
                            // A click on the scene reads on.
                            if response.clicked() {
                                self.clearing_read_on();
                            }
                            None
                        }
                        (Area::Hilltop, _, _) => {
                            let mut hovered = None;
                            if let Some(ground) = &mut self.hilltop {
                                ground.set_pointer(pointer);
                                hovered = pointer.and_then(|(x, y)| ground.actor_at(x, y, now));
                            }
                            let piece = self.hilltop_hover(pointer);
                            if let Some((label, (x, y))) = &piece {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                tag(label, to_screen(*x, *y));
                            } else if hovered.is_some() {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if response.clicked()
                                && !self.hilltop_click(pointer)
                                && self.placing.is_none()
                                && let (Some(id), Some(ground)) = (hovered, &mut self.hilltop)
                            {
                                ground.offer(id, self.tool, &mut self.trust, now);
                            }
                            hovered.filter(|_| piece.is_none())
                        }
                        (Area::Fairground, _, Some((ground, games))) => {
                            ground.set_pointer(pointer);
                            let hovered = pointer.and_then(|(x, y)| ground.actor_at(x, y, now));
                            let playing = games.playing().is_some();
                            // During a game the person only watches; otherwise, as on the green.
                            if let (false, Some(id)) = (playing, hovered) {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                if response.clicked() {
                                    ground.offer(id, self.tool, &mut self.trust, now);
                                }
                            }
                            // "It", whoever is in front, or whoever is having a go wears its name
                            // all game, so the watcher can follow it about.
                            let tags = games.tags();
                            let tagged = hovered
                                .into_iter()
                                .filter(|id| tags.iter().all(|(tagged, _)| tagged != id))
                                .map(|id| (id, None))
                                .chain(tags.iter().map(|(id, role)| (*id, Some(*role))));
                            for (id, role) in tagged {
                                if let (Some((x, y)), Some(member)) =
                                    (ground.head(id, now), self.arrival.cast.member(id))
                                {
                                    let label = match role {
                                        Some(role) => format!("{} \u{b7} {role}", member.name),
                                        None => member.name.clone(),
                                    };
                                    tag(&label, to_screen(x, y - 6.0));
                                }
                            }
                            hovered
                        }
                        (Area::Clubhouse, _, _) => {
                            let Some(room) = &mut self.clubhouse else {
                                return;
                            };
                            let ground = room.ground();
                            ground.set_pointer(pointer);
                            let hovered = pointer.and_then(|(x, y)| ground.actor_at(x, y, now));
                            let on_board = self.story.is_none()
                                && hovered.is_none()
                                && pointer.is_some_and(|(x, y)| clubhouse::on_board(x, y));
                            if let Some((_, director)) = &mut self.story {
                                // During a story, a click on the scene reads on.
                                if response.clicked() && director.shown().is_some() {
                                    director.read_on();
                                }
                            } else if on_board {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                let (x, y) = clubhouse::board_label();
                                tag("The notice board", to_screen(x, y));
                                if response.clicked() {
                                    self.board = !self.board;
                                }
                            } else if let Some(id) = hovered {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                if response.clicked() {
                                    ground.offer(id, self.tool, &mut self.trust, now);
                                }
                            }
                            if let Some(id) = hovered
                                && let (Some((x, y)), Some(member)) =
                                    (ground.head(id, now), self.arrival.cast.member(id))
                            {
                                tag(&member.name, to_screen(x, y - 6.0));
                            }
                            hovered
                        }
                        (Area::Green, Some(green), _) => {
                            green.set_pointer(pointer);
                            let hovered = pointer.and_then(|(x, y)| green.actor_at(x, y, now));
                            if let Some(id) = hovered {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                if response.clicked() {
                                    if self.picked.is_some() {
                                        dress_on = Some(id);
                                    } else {
                                        green.offer(id, self.tool, &mut self.trust, now);
                                    }
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
                            // The dress-up box opens with a click, as well as from the bar.
                            let (left, top, right, bottom) = crate::green::DRESS_UP;
                            on_box = hovered.is_none()
                                && pointer.is_some_and(|(x, y)| {
                                    (left as f32..right as f32).contains(&x)
                                        && (top as f32..bottom as f32).contains(&y)
                                });
                            if on_box {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                if response.clicked() {
                                    self.dress_up = !self.dress_up;
                                    self.picked = None;
                                }
                            }
                            hovered
                        }
                        _ => {
                            let settled = self.area == Area::Station
                                && self.station.is_settled()
                                && self.leaving.is_none();
                            fixture = pointer
                                .filter(|_| settled)
                                .and_then(|(x, y)| self.station.fixture_at(x, y));
                            let over_someone = pointer
                                .is_some_and(|(x, y)| self.station.traveler_at(x, y).is_some());
                            if response.clicked() {
                                // A click on the station skips the arrival; once everyone is
                                // down, a click on a board brings it close.
                                self.station.skip_arrival();
                                if let Some(fixture) = fixture.filter(|_| !over_someone) {
                                    self.look_at(fixture);
                                }
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
                    }
                };
                self.hovered = hovered;
                if on_box {
                    let open = self.dress_up;
                    response.on_hover_ui_at_pointer(|ui| {
                        ui.strong("The dress-up box");
                        ui.label(if open {
                            "Click to close it."
                        } else {
                            "Click to open it and dress someone up."
                        });
                    });
                } else if fixture == Some(Fixture::Notices) && hovered.is_none() {
                    let notes = departures::notices(&self.board()).len();
                    response.on_hover_ui_at_pointer(|ui| {
                        ui.strong("The notice board");
                        ui.label(format!(
                            "{notes} pinned up. Click to read {}.",
                            if notes == 1 { "it" } else { "them" }
                        ));
                    });
                } else if fixture == Some(Fixture::Departures) && hovered.is_none() {
                    response.on_hover_ui_at_pointer(|ui| {
                        ui.strong("Departures");
                        ui.label("Where to go, and who would like to. Click to look closer.");
                    });
                } else if fixture == Some(Fixture::Case) && hovered.is_none() {
                    let kept: Vec<&str> = self
                        .memories
                        .colony()
                        .souvenirs
                        .iter()
                        .filter_map(|id| souvenirs::name(id))
                        .collect();
                    response.on_hover_ui_at_pointer(|ui| {
                        ui.strong("The display case");
                        if kept.is_empty() {
                            ui.label(
                                "Empty for now. Souvenirs from stories and games are kept here.",
                            );
                        }
                        for name in kept {
                            ui.label(name);
                        }
                    });
                } else if let Some(id) = hovered {
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
                self.draw_camera(&painter, to_screen, rect, now);
            });

        if let Some(id) = dress_on {
            self.dress(id, now);
        }
        let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
        self.use_camera(camera_click, scroll, now);
        let held = ctx.input(|input| input.pointer.primary_down()) && !self.camera.out;
        self.groom(held, now);
        self.dress_everyone();
        self.dress_up_window(&ctx);
        self.album_window(&ctx, now);
        self.journal_window(&ctx);
        self.plans_window(&ctx);
        self.board_window(&ctx, now);
        self.shelf_window(&ctx);
        self.notices_window(&ctx);
        self.departures_window(&ctx, now);

        if self.area != Area::Station || self.leaving.is_some() || self.station.in_motion(now) {
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
