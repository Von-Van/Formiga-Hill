//! Hill's paper: everything the window puts over the scene, drawn the way Formiga Desktop draws
//! its speech bubbles and its creature menu. Cream paper, the shared dark plum outline, corners
//! stepped a pixel at a time, and words in Hill's own lettering (`lettering`), so a button, a card
//! of notices or what someone says by the fire reads as part of the same world as the creatures
//! rather than a window laid on top of it.
//!
//! Paper has a pixel of its own, half a scene pixel (more with larger text), on whole screen
//! pixels so it stays crisp: fine enough for a sentence to fit over a creature's head, coarse
//! enough to read as pixel art beside it. Every piece here measures itself in those pixels and
//! leaves egui only the arranging and the clicking.

use crate::lettering::{self, ASCENT, DESCENT, LINE};
use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Ui, Vec2, pos2, vec2};
use formiga_art::Canvas;

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Desktop's shared dark plum: every outline, and the ink words are written in.
pub const OUTLINE: Color32 = rgb(66, 53, 72);
pub const INK: Color32 = OUTLINE;
/// Desktop's soft cream paper.
pub const PAPER: Color32 = rgb(255, 246, 224);
/// A quiet warm grey: the lip under a button, rules and empty bars.
pub const SHADE: Color32 = rgb(226, 211, 186);
/// Desktop's hovered cell: darker than the paper as well as warmer, so it survives greyscale.
pub const HOVER: Color32 = rgb(250, 226, 180);
pub const CHOSEN: Color32 = rgb(244, 206, 140);
pub const AMBER: Color32 = rgb(236, 172, 68);
pub const AMBER_DEEP: Color32 = rgb(190, 128, 44);
/// Headings, as the notebook sets its printed labels in forest green.
pub const FOREST: Color32 = rgb(62, 112, 96);
/// Words that explain rather than say: hints, dates, what something takes.
pub const FADED: Color32 = rgb(118, 104, 124);
/// What cannot be used yet, and what is not known yet.
pub const MUTED: Color32 = rgb(160, 148, 164);
/// A shadow under paper laid over the scene, so it lifts off whatever is behind it.
const SHADOW: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 70);
/// Night paper, for the one who speaks from the dark.
pub const NIGHT: Color32 = rgb(26, 16, 38);
pub const GOLD: Color32 = rgb(245, 210, 94);

/// How big a paper pixel is: a whole number of screen pixels, about half a scene pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    /// Points per paper pixel.
    pub point: f32,
}

impl Grain {
    /// The grain for a scene drawn in `scene` (points), `scene_width` scene pixels across, at
    /// `pixels_per_point`, with text made `zoom` times larger than usual.
    pub fn for_scene(scene: Rect, scene_width: u32, pixels_per_point: f32, zoom: f32) -> Self {
        let scene_pixel = scene.width() * pixels_per_point / scene_width as f32;
        let paper_pixel = (scene_pixel * 0.5 * zoom.max(1.0)).round().max(1.0);
        Self {
            point: paper_pixel / pixels_per_point,
        }
    }

    fn id() -> egui::Id {
        egui::Id::new("hill-paper-grain")
    }

    /// Keeps this grain for every piece of paper drawn this frame.
    pub fn keep(self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(Self::id(), self));
    }

    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(Self::id()))
            .unwrap_or(Self { point: 1.5 })
    }

    /// `pixels` paper pixels, in points.
    pub fn len(self, pixels: i32) -> f32 {
        pixels as f32 * self.point
    }

    pub fn size(self, width: i32, height: i32) -> Vec2 {
        vec2(self.len(width), self.len(height))
    }

    /// How many whole paper pixels fit in `points`.
    pub fn fit(self, points: f32) -> i32 {
        (points / self.point).floor() as i32
    }
}

/// Rounds a position onto the screen's own pixels, so paper drawn from it stays crisp.
pub fn snap(ctx: &egui::Context, at: Pos2) -> Pos2 {
    let ppp = ctx.pixels_per_point();
    pos2((at.x * ppp).round() / ppp, (at.y * ppp).round() / ppp)
}

/// Paper and lettering gathered into one mesh, drawn in paper pixels from an origin.
pub struct Ink {
    mesh: egui::Mesh,
    origin: Pos2,
    grain: Grain,
}

impl Ink {
    pub fn new(ctx: &egui::Context, origin: Pos2, grain: Grain) -> Self {
        Self {
            mesh: egui::Mesh::default(),
            origin: snap(ctx, origin),
            grain,
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color32) {
        if width <= 0 || height <= 0 || color.a() == 0 {
            return;
        }
        let min = self.origin + self.grain.size(x, y);
        self.mesh.add_colored_rect(
            Rect::from_min_size(min, self.grain.size(width, height)),
            color,
        );
    }

    /// One line of lettering with its top at `(x, y)`.
    pub fn text(&mut self, x: i32, y: i32, text: &str, color: Color32) {
        for (left, top, length) in lettering::runs(text) {
            self.rect(x + left, y + top, length, 1, color);
        }
    }

    /// A small picture, each of its pixels `scale` paper pixels.
    pub fn picture(&mut self, x: i32, y: i32, picture: &Canvas, scale: i32) {
        for row in 0..picture.height() as i32 {
            for column in 0..picture.width() as i32 {
                let pixel = picture.get(column, row);
                if pixel.a > 0 {
                    let color = Color32::from_rgba_unmultiplied(pixel.r, pixel.g, pixel.b, pixel.a);
                    self.rect(x + column * scale, y + row * scale, scale, scale, color);
                }
            }
        }
    }

    /// A box with stepped corners: `fill` inside a one-pixel `outline`. `corner` is how many
    /// steps each corner takes, one for a button or a tag, two for a card or a speech box.
    pub fn frame(
        &mut self,
        (x, y, w, h): (i32, i32, i32, i32),
        fill: Color32,
        outline: Color32,
        corner: i32,
    ) {
        if corner >= 2 {
            self.rect(x + 2, y, w - 4, 1, outline);
            self.rect(x + 2, y + h - 1, w - 4, 1, outline);
            self.rect(x, y + 2, 1, h - 4, outline);
            self.rect(x + w - 1, y + 2, 1, h - 4, outline);
            for (cx, cy) in [
                (x + 1, y + 1),
                (x + w - 2, y + 1),
                (x + 1, y + h - 2),
                (x + w - 2, y + h - 2),
            ] {
                self.rect(cx, cy, 1, 1, outline);
            }
            self.rect(x + 2, y + 1, w - 4, 1, fill);
            self.rect(x + 1, y + 2, w - 2, h - 4, fill);
            self.rect(x + 2, y + h - 2, w - 4, 1, fill);
        } else {
            self.rect(x + 1, y, w - 2, 1, outline);
            self.rect(x + 1, y + h - 1, w - 2, 1, outline);
            self.rect(x, y + 1, 1, h - 2, outline);
            self.rect(x + w - 1, y + 1, 1, h - 2, outline);
            self.rect(x + 1, y + 1, w - 2, h - 2, fill);
        }
    }

    /// The same box's shadow, a pixel down and to the right.
    pub fn shadow(&mut self, (x, y, w, h): (i32, i32, i32, i32), corner: i32) {
        self.frame((x + 1, y + 1, w, h), SHADOW, SHADOW, corner);
    }

    pub fn paint(self, painter: &egui::Painter) {
        if !self.mesh.is_empty() {
            painter.add(egui::Shape::mesh(self.mesh));
        }
    }

    /// Paints in place of a shape reserved earlier, so paper can sit behind what was laid on it.
    pub fn paint_at(self, painter: &egui::Painter, slot: egui::layers::ShapeIdx) {
        painter.set(slot, egui::Shape::mesh(self.mesh));
    }
}

/// How tall a line of words stands, from the top of its capitals to the bottom of its tails.
pub const WORDS_HEIGHT: i32 = ASCENT + DESCENT;
/// How tall a paragraph of `lines` lines is.
pub fn paragraph_height(lines: usize) -> i32 {
    if lines == 0 {
        0
    } else {
        lines as i32 * LINE - (LINE - WORDS_HEIGHT)
    }
}

// ---------------------------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------------------------

/// A button's height: outline, three pixels of paper, capitals, three more, a lip, and outline.
/// Room enough for Desktop's own menu pictures, so a row of buttons with and without them is
/// level.
pub const BUTTON_HEIGHT: i32 = 1 + 3 + ASCENT + 3 + 1 + 1;
const BUTTON_PAD: i32 = 4;

/// A paper button with its words in capitals, as Desktop labels its menu.
pub struct Button {
    text: String,
    selected: bool,
    enabled: bool,
    picture: Option<(Canvas, i32)>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selected: false,
            enabled: true,
            picture: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// A small picture before the words, each of its pixels `scale` paper pixels.
    pub fn picture(mut self, picture: Canvas, scale: i32) -> Self {
        self.picture = Some((picture, scale));
        self
    }
}

impl egui::Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let grain = Grain::of(ui.ctx());
        let label = self.text.to_uppercase();
        let words = lettering::width(&label);
        let (picture_w, picture_h) = self.picture.as_ref().map_or((0, 0), |(picture, scale)| {
            (
                picture.width() as i32 * scale,
                picture.height() as i32 * scale,
            )
        });
        let gap = if picture_w > 0 && words > 0 { 3 } else { 0 };
        let width = 2 + BUTTON_PAD * 2 + picture_w + gap + words;
        let height = BUTTON_HEIGHT.max(picture_h + 4);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(grain.size(width, height), sense);
        if ui.is_rect_visible(rect) {
            let hovered = self.enabled && response.hovered();
            let pressed = self.enabled && response.is_pointer_button_down_on();
            let fill = match (self.enabled, self.selected, hovered) {
                (false, _, _) => PAPER,
                (true, true, _) => CHOSEN,
                (true, false, true) => HOVER,
                _ => PAPER,
            };
            let ink_color = if self.enabled { INK } else { MUTED };
            let mut ink = Ink::new(ui.ctx(), rect.min, grain);
            let sunk = i32::from(pressed || self.selected);
            ink.shadow((0, 0, width, height), 1);
            ink.frame((0, 0, width, height), fill, OUTLINE, 1);
            if sunk == 0 {
                // The lip along the bottom, as a key has, gone when it is pressed or chosen.
                ink.rect(
                    1,
                    height - 2,
                    width - 2,
                    1,
                    if fill == PAPER { SHADE } else { AMBER },
                );
            }
            let mut x = 1 + BUTTON_PAD;
            if let Some((picture, scale)) = &self.picture {
                let top = (height - picture_h) / 2 + sunk - 1;
                if self.enabled {
                    ink.picture(x, top, picture, *scale);
                } else {
                    ink.picture(x, top, &faded(picture), *scale);
                }
                x += picture_w + gap;
            }
            ink.text(x, (height - ASCENT) / 2 - 1 + sunk, &label, ink_color);
            ink.paint(ui.painter());
        }
        if self.enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response
    }
}

/// A button, for short.
pub fn button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Button::new(text))
}

/// A button that is pressed in while it is the choice, like `selectable_value`.
pub fn choice<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, value: T, text: &str) -> Response {
    let response = ui.add(Button::new(text).selected(*current == value));
    if response.clicked() {
        *current = value;
    }
    response
}

/// A picture with its colours washed out, for something that cannot be used just now.
fn faded(picture: &Canvas) -> Canvas {
    let mut out = picture.clone();
    for y in 0..out.height() as i32 {
        for x in 0..out.width() as i32 {
            let mut pixel = out.get(x, y);
            if pixel.a > 0 {
                let grey =
                    ((u16::from(pixel.r) + u16::from(pixel.g) + u16::from(pixel.b)) / 3) as u8;
                let mix = |c: u8| ((u16::from(c) + u16::from(grey) * 2 + 255 * 2) / 5) as u8;
                pixel.r = mix(pixel.r);
                pixel.g = mix(pixel.g);
                pixel.b = mix(pixel.b);
                out.set(x, y, pixel);
            }
        }
    }
    out
}

/// What a little picture button is called, in a tab under or over it while it is pointed at.
pub fn name_it(ui: &Ui, response: &Response, name: &str) {
    if !response.hovered() {
        return;
    }
    let grain = Grain::of(ui.ctx());
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("hill-paper-names"),
    ));
    let label = name.to_uppercase();
    let width = lettering::width(&label) + 6;
    let height = ASCENT + 4;
    let rect = response.rect;
    let room_above = rect.top() - painter.clip_rect().top() > grain.len(height + 4);
    let top = if room_above {
        rect.top() - grain.len(height + 2)
    } else {
        rect.bottom() + grain.len(2)
    };
    let left = (rect.center().x - grain.len(width) / 2.0).clamp(
        painter.clip_rect().left(),
        painter.clip_rect().right() - grain.len(width),
    );
    let mut ink = Ink::new(ui.ctx(), pos2(left, top), grain);
    ink.frame((0, 0, width, height), PAPER, OUTLINE, 1);
    ink.text(3, 2, &label, INK);
    ink.paint(&painter);
}

// ---------------------------------------------------------------------------------------------
// Words
// ---------------------------------------------------------------------------------------------

/// Words in Hill's lettering, wrapped to the room there is.
pub struct Words {
    text: String,
    color: Color32,
    caps: bool,
    slip: bool,
    most: Option<i32>,
}

impl Words {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: INK,
            caps: false,
            slip: false,
            most: None,
        }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// In capitals, as a printed label.
    pub fn caps(mut self) -> Self {
        self.caps = true;
        self
    }

    /// On a slip of paper of their own, to read over the scene.
    pub fn slip(mut self) -> Self {
        self.slip = true;
        self
    }

    /// No wider than this many paper pixels, however much room there is.
    pub fn most(mut self, pixels: i32) -> Self {
        self.most = Some(pixels);
        self
    }
}

impl egui::Widget for Words {
    fn ui(self, ui: &mut Ui) -> Response {
        let grain = Grain::of(ui.ctx());
        let text = if self.caps {
            self.text.to_uppercase()
        } else {
            self.text
        };
        let pad = if self.slip { 4 } else { 0 };
        // In a wrapping row the words may start a new line rather than squeeze in beside others.
        let room = if ui.layout().main_wrap() {
            ui.max_rect().width()
        } else {
            ui.available_width()
        };
        let room = grain.fit(room) - pad * 2;
        let most = self.most.map_or(room, |most| most.min(room)).max(24);
        let lines = lettering::wrap(&text, most);
        let width = lettering::widest(&lines) + pad * 2;
        // A one-line slip is as tall as a button, to sit level beside one.
        let height = paragraph_height(lines.len()) + if self.slip { 7 } else { 0 };
        let (rect, response) = ui.allocate_exact_size(grain.size(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let mut ink = Ink::new(ui.ctx(), rect.min, grain);
            let top = if self.slip {
                ink.shadow((0, 0, width, height), 1);
                ink.frame((0, 0, width, height), PAPER, OUTLINE, 1);
                3
            } else {
                0
            };
            for (index, line) in lines.iter().enumerate() {
                ink.text(pad, top + index as i32 * LINE, line, self.color);
            }
            ink.paint(ui.painter());
        }
        response
    }
}

/// Words as a heading: printed capitals in forest green, with a little room above.
pub fn heading(ui: &mut Ui, text: impl Into<String>) -> Response {
    let grain = Grain::of(ui.ctx());
    ui.add_space(grain.len(4));
    ui.add(Words::new(text).caps().color(FOREST))
}

/// Plain words.
pub fn words(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text))
}

/// The name of something, in a list of things: in capitals, as a catalogue sets them.
pub fn name(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text).caps())
}

/// Words that explain: a hint, a date, what something takes.
pub fn aside(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text).color(FADED))
}

/// Words about what is not known yet.
pub fn faint(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text).color(MUTED))
}

/// Words on their own slip of paper, to read over the scene.
pub fn slip(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text).slip())
}

/// A slip that says what the person can do now, a shade stronger than one that only tells.
pub fn hint(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Words::new(text).slip().color(FOREST))
}

/// A thin rule between parts of a card.
pub fn rule(ui: &mut Ui) {
    let grain = Grain::of(ui.ctx());
    let width = grain.fit(ui.available_width());
    let (rect, _) = ui.allocate_exact_size(grain.size(width, 5), Sense::hover());
    let mut ink = Ink::new(ui.ctx(), rect.min, grain);
    // Dashed, as the notebook rules its pages.
    let mut x = 0;
    while x < width {
        ink.rect(x, 2, 3.min(width - x), 1, SHADE);
        x += 5;
    }
    ink.paint(ui.painter());
}

/// While `response` is pointed at, a note beside it saying more about it.
pub fn explain(ui: &Ui, response: Response, text: &str) -> Response {
    if response.hovered()
        && !text.is_empty()
        && let Some(pointer) = response.hover_pos()
    {
        let bounds = ui.ctx().content_rect();
        note(ui.ctx(), bounds, pointer, &[], text);
    }
    response
}

/// A note at the pointer: an optional heading in capitals, then words, kept inside `bounds`.
pub fn note(ctx: &egui::Context, bounds: Rect, pointer: Pos2, heading: &[&str], text: &str) {
    let grain = Grain::of(ctx);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("hill-paper-note"),
    ));
    let most = 150;
    let mut lines: Vec<(String, Color32)> = heading
        .iter()
        .map(|line| (line.to_uppercase(), FOREST))
        .collect();
    for paragraph in text.split('\n').filter(|paragraph| !paragraph.is_empty()) {
        for line in lettering::wrap(paragraph, most) {
            lines.push((line, INK));
        }
    }
    if lines.is_empty() {
        return;
    }
    let width = lines
        .iter()
        .map(|(line, _)| lettering::width(line))
        .max()
        .unwrap_or(0)
        + 10;
    let height = paragraph_height(lines.len()) + 8;
    let size = grain.size(width, height);
    let mut at = pointer + vec2(grain.len(8), grain.len(8));
    if at.x + size.x > bounds.right() {
        at.x = pointer.x - size.x - grain.len(4);
    }
    if at.y + size.y > bounds.bottom() {
        at.y = pointer.y - size.y - grain.len(4);
    }
    at.x = at.x.max(bounds.left());
    at.y = at.y.max(bounds.top());
    let mut ink = Ink::new(ctx, at, grain);
    ink.shadow((0, 0, width, height), 2);
    ink.frame((0, 0, width, height), PAPER, OUTLINE, 2);
    for (index, (line, color)) in lines.iter().enumerate() {
        ink.text(5, 4 + index as i32 * LINE, line, *color);
    }
    ink.paint(&painter);
}

// ---------------------------------------------------------------------------------------------
// Meters and levels
// ---------------------------------------------------------------------------------------------

/// How much of something is left, as a slip with a bar on it: the light, the day.
pub fn meter(ui: &mut Ui, fraction: f32, label: &str) -> Response {
    let grain = Grain::of(ui.ctx());
    let label = label.to_uppercase();
    let bar = 32;
    let words = lettering::width(&label);
    let width = 4 + words + 4 + bar + 4;
    let height = ASCENT + 6;
    let (rect, response) = ui.allocate_exact_size(grain.size(width, height), Sense::hover());
    if ui.is_rect_visible(rect) {
        let mut ink = Ink::new(ui.ctx(), rect.min, grain);
        ink.shadow((0, 0, width, height), 1);
        ink.frame((0, 0, width, height), PAPER, OUTLINE, 1);
        ink.text(4, 2, &label, INK);
        let left = 4 + words + 4;
        ink.frame((left, 2, bar, ASCENT + 2), SHADE, OUTLINE, 1);
        let filled = ((bar - 2) as f32 * fraction.clamp(0.0, 1.0)).round() as i32;
        ink.rect(left + 1, 3, filled, ASCENT, AMBER);
        ink.rect(left + 1, 3, filled, 1, CHOSEN);
        ink.paint(ui.painter());
    }
    response
}

/// A level from 0 to 1 to drag, with its name and how far up it is: the sound's sliders.
pub fn level(ui: &mut Ui, label: &str, value: &mut f32, enabled: bool) -> Response {
    let grain = Grain::of(ui.ctx());
    let label = label.to_uppercase();
    let name_width = 48;
    let track = 80;
    let width = name_width + track + 34;
    let height = ASCENT + 6;
    let sense = if enabled {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let (rect, mut response) = ui.allocate_exact_size(grain.size(width, height), sense);
    let track_left = rect.left() + grain.len(name_width + 3);
    let track_width = grain.len(track - 6);
    if enabled
        && (response.dragged() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let new = ((pointer.x - track_left) / track_width).clamp(0.0, 1.0);
        let new = (new * 100.0).round() / 100.0;
        if new != *value {
            *value = new;
            response.mark_changed();
        }
    }
    if ui.is_rect_visible(rect) {
        let mut ink = Ink::new(ui.ctx(), rect.min, grain);
        let color = if enabled { INK } else { MUTED };
        ink.text(0, 3, &label, color);
        let left = name_width;
        ink.frame((left, 4, track, 5), SHADE, OUTLINE, 1);
        let filled = ((track - 6) as f32 * value.clamp(0.0, 1.0)).round() as i32;
        if enabled {
            ink.rect(left + 1, 5, filled + 2, 3, AMBER);
        }
        // The knob: a little paper block on the track.
        let knob = left + 1 + filled;
        let knob_fill = if response.hovered() || response.dragged() {
            HOVER
        } else {
            PAPER
        };
        ink.frame((knob, 1, 5, 11), knob_fill, OUTLINE, 1);
        ink.text(
            left + track + 4,
            3,
            &format!("{}%", (*value * 100.0).round() as u32),
            color,
        );
        ink.paint(ui.painter());
    }
    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

// ---------------------------------------------------------------------------------------------
// Backing, cards, menus and trays
// ---------------------------------------------------------------------------------------------

/// Paper behind whatever `add` lays out, with `margin` paper pixels of it all round.
pub fn backed<R>(
    ui: &mut Ui,
    margin: i32,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let grain = Grain::of(ui.ctx());
    let slot = ui.painter().add(egui::Shape::Noop);
    let inner = egui::Frame::NONE
        .inner_margin(egui::Margin::same(grain.len(margin).round() as i8))
        .show(ui, add);
    let rect = inner.response.rect;
    let width = (rect.width() / grain.point).ceil() as i32;
    let height = (rect.height() / grain.point).ceil() as i32;
    let mut ink = Ink::new(ui.ctx(), rect.min, grain);
    ink.shadow((0, 0, width, height), 2);
    ink.frame((0, 0, width, height), PAPER, OUTLINE, 2);
    ink.paint_at(ui.painter(), slot);
    egui::InnerResponse::new(inner.inner, inner.response)
}

/// Lays out paper widgets with the gaps paper wants between them.
pub fn spaced(ui: &mut Ui) {
    let grain = Grain::of(ui.ctx());
    ui.spacing_mut().item_spacing = grain.size(3, 3);
}

/// A card over the scene, at its right, in place of a window: a title in capitals, a close
/// button, and whatever `add` puts on it, scrolling if there is more than fits. `open` is
/// cleared when the person closes it.
pub fn card<R>(
    ctx: &egui::Context,
    scene: Rect,
    id: &str,
    title: &str,
    width: f32,
    open: &mut bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if !*open {
        return None;
    }
    let grain = Grain::of(ctx);
    let margin = grain.len(6);
    let width = (scene.width() * width).min(scene.width() - margin * 2.0);
    let mut close = false;
    let shown = egui::Area::new(egui::Id::new(("hill-card", id)))
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::RIGHT_TOP)
        .fixed_pos(snap(ctx, scene.right_top() + vec2(-margin, margin)))
        .constrain_to(scene)
        .show(ctx, |ui| {
            spaced(ui);
            backed(ui, 6, |ui| {
                ui.set_width(width - grain.len(12));
                ui.horizontal(|ui| {
                    ui.add(Words::new(title).caps().color(FOREST));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let shut = ui.add(Button::new("\u{d7}"));
                        name_it(ui, &shut, "close");
                        close = shut.clicked();
                    });
                });
                rule(ui);
                let room = scene.height() - margin * 2.0 - grain.len(12 + BUTTON_HEIGHT + 8);
                egui::ScrollArea::vertical()
                    .max_height(room.max(grain.len(40)))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        spaced(ui);
                        add(ui)
                    })
                    .inner
            })
            .inner
        });
    if close {
        *open = false;
    }
    Some(shown.inner)
}

/// A button that opens a little card of choices over it, as a menu would.
pub fn menu<R>(
    ui: &mut Ui,
    button: Button,
    stays_open: bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> (Response, Option<R>) {
    let response = ui.add(button);
    let close = if stays_open {
        egui::PopupCloseBehavior::CloseOnClickOutside
    } else {
        egui::PopupCloseBehavior::CloseOnClick
    };
    let shown = egui::Popup::from_toggle_button_response(&response)
        .frame(egui::Frame::NONE)
        .close_behavior(close)
        .align(egui::RectAlign::TOP_START)
        .align_alternatives(&[egui::RectAlign::BOTTOM_START, egui::RectAlign::TOP_END])
        .gap(Grain::of(ui.ctx()).len(2))
        .show(|ui| {
            spaced(ui);
            backed(ui, 4, |ui| {
                spaced(ui);
                add(ui)
            })
            .inner
        });
    (response, shown.map(|shown| shown.inner))
}

/// A row of controls laid on the scene's bottom edge, from `corner` (left or right), wrapping
/// onto another row above if they run out of room. Returns where it is, for laying out round it.
pub fn tray<R>(
    ctx: &egui::Context,
    scene: Rect,
    id: &str,
    right: bool,
    most: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    let grain = Grain::of(ctx);
    let margin = grain.len(5);
    let (pivot, at) = if right {
        (
            egui::Align2::RIGHT_BOTTOM,
            scene.right_bottom() + vec2(-margin, -margin),
        )
    } else {
        (
            egui::Align2::LEFT_BOTTOM,
            scene.left_bottom() + vec2(margin, -margin),
        )
    };
    egui::Area::new(egui::Id::new(("hill-tray", id)))
        .order(egui::Order::Middle)
        .pivot(pivot)
        .fixed_pos(snap(ctx, at))
        .constrain_to(scene)
        .show(ctx, |ui| {
            spaced(ui);
            ui.set_max_width(most.max(grain.len(60)));
            // Rows line up along their tops: aligned to the bottom, egui sizes each row by all
            // the room above it, and the tray grows into the scene.
            let layout = if right {
                egui::Layout::right_to_left(egui::Align::Min).with_main_wrap(true)
            } else {
                egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true)
            };
            ui.with_layout(layout, add).inner
        })
}

// ---------------------------------------------------------------------------------------------
// Over the scene
// ---------------------------------------------------------------------------------------------

/// A name on a tab, centred on `centre`, as Desktop labels a menu's cell.
pub fn tag(ctx: &egui::Context, painter: &egui::Painter, text: &str, centre: Pos2) {
    let grain = Grain::of(ctx);
    let label = text.to_uppercase();
    let width = lettering::width(&label) + 6;
    let height = ASCENT + 4;
    let at = centre - grain.size(width, height) / 2.0;
    let mut ink = Ink::new(ctx, at, grain);
    ink.shadow((0, 0, width, height), 1);
    ink.frame((0, 0, width, height), PAPER, OUTLINE, 1);
    ink.text(3, 2, &label, INK);
    ink.paint(painter);
}

/// Who is speaking, which decides how a speech box looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    /// Someone in the scene, with a tail to them.
    Speaker,
    /// The story itself, in a caption at the top.
    Narrator,
    /// The one from the dark.
    Night,
}

/// The widest a line of speech runs, in paper pixels.
const SPEECH_WIDTH: i32 = 168;

/// What someone says, in a box over `head` (the top of their head, on screen) with a tail down
/// to it, their name on a tab at its corner, kept inside `scene`; or, for the narrator or with no
/// one to point at, a caption across the top. `more` shows the mark that there is more to read.
/// Returns where the box went.
#[allow(clippy::too_many_arguments)]
pub fn speech(
    ctx: &egui::Context,
    painter: &egui::Painter,
    scene: Rect,
    head: Option<Pos2>,
    name: Option<&str>,
    text: &str,
    voice: Voice,
    more: bool,
) -> Rect {
    let grain = Grain::of(ctx);
    let room = grain.fit(scene.width()) - 16;
    let lines = lettering::wrap(text, SPEECH_WIDTH.min(room));
    let named = name.filter(|name| !name.is_empty());
    let label = named.map(str::to_uppercase);
    let label_width = label.as_deref().map_or(0, lettering::width);
    let mark = if more { 6 } else { 0 };
    let width = (lettering::widest(&lines) + mark).max(label_width + 4) + 12;
    let height = paragraph_height(lines.len()) + 10 + i32::from(named.is_some()) * 2;
    let size = grain.size(width, height);
    let margin = grain.len(4);
    let tail = 4;
    let head = head.filter(|_| voice != Voice::Narrator);
    let tab_room = if named.is_some() { grain.len(7) } else { 0.0 };
    let (mut left, mut top) = match head {
        Some(head) => (head.x - size.x / 2.0, head.y - size.y - grain.len(tail + 2)),
        None => (
            scene.center().x - size.x / 2.0,
            scene.top() + margin + tab_room,
        ),
    };
    left = left.clamp(scene.left() + margin, scene.right() - margin - size.x);
    top = top.clamp(
        scene.top() + margin + tab_room,
        scene.bottom() - margin - size.y,
    );
    let (fill, outline, color, accent) = match voice {
        Voice::Speaker => (PAPER, OUTLINE, INK, AMBER_DEEP),
        Voice::Narrator => (rgb(248, 236, 210), OUTLINE, FADED, AMBER_DEEP),
        Voice::Night => (NIGHT, GOLD, PAPER, GOLD),
    };
    let mut ink = Ink::new(ctx, pos2(left, top), grain);
    let origin = ink.origin;
    ink.shadow((0, 0, width, height), 2);
    ink.frame((0, 0, width, height), fill, outline, 2);
    // The tail, from the bottom edge down towards the head, if the head is below the box.
    if let Some(head) = head {
        let x = (((head.x - origin.x) / grain.point).round() as i32).clamp(6, width - 7);
        let below = head.y > origin.y + size.y;
        if below {
            ink.rect(x - 1, height - 1, 3, 1, fill);
            ink.rect(x - 2, height - 1, 1, 1, outline);
            ink.rect(x + 2, height - 1, 1, 1, outline);
            ink.rect(x - 1, height, 1, 1, outline);
            ink.rect(x, height, 1, 1, fill);
            ink.rect(x + 1, height, 1, 1, outline);
            ink.rect(x, height + 1, 1, 1, outline);
        }
    }
    let words_top = 5 + i32::from(named.is_some()) * 2;
    for (index, line) in lines.iter().enumerate() {
        ink.text(6, words_top + index as i32 * LINE, line, color);
    }
    if more {
        let last = lines.last().map_or(0, |line| lettering::width(line));
        let y = words_top + (lines.len() as i32 - 1) * LINE;
        ink.text(6 + last + 3, y, "\u{25b8}", accent);
    }
    if let Some(label) = &label {
        // The name on a tab over the box's top-left corner.
        let tab = (5, -6, label_width + 6, ASCENT + 4);
        ink.frame(tab, fill, outline, 1);
        ink.text(
            8,
            -4,
            label,
            if voice == Voice::Night { GOLD } else { FOREST },
        );
    }
    ink.paint(painter);
    Rect::from_min_size(pos2(left, top), size)
}

// ---------------------------------------------------------------------------------------------
// Pictures for buttons
// ---------------------------------------------------------------------------------------------

/// A small picture drawn with letters: `#` outline, `p` paper, `s` shade, and a few colours.
pub fn picture(art: &[&str]) -> Canvas {
    let width = art.iter().map(|row| row.len()).max().unwrap_or(0) as u32;
    let mut canvas = Canvas::new(width, art.len() as u32);
    for (y, row) in art.iter().enumerate() {
        for (x, pixel) in row.chars().enumerate() {
            let color = match pixel {
                '#' => OUTLINE,
                'p' => PAPER,
                's' => SHADE,
                'a' => AMBER,
                'd' => AMBER_DEEP,
                'r' => rgb(214, 92, 108),
                'R' => rgb(168, 62, 82),
                'g' => rgb(104, 156, 96),
                'G' => FOREST,
                't' => rgb(86, 170, 174),
                'T' => rgb(174, 226, 222),
                'b' => rgb(168, 104, 58),
                'B' => rgb(120, 72, 40),
                _ => continue,
            };
            let [r, g, b, a] = color.to_srgba_unmultiplied();
            canvas.set(x as i32, y as i32, formiga_art::Rgba::new(r, g, b, a));
        }
    }
    canvas
}

/// The pictures on Hill's own buttons.
pub mod pictures {
    pub const CAMERA: &[&str] = &[
        "...###.....",
        ".#########.",
        "#sss###sss#",
        "#ss#TTt#ss#",
        "#ss#Ttt#ss#",
        "#ss#ttt#ss#",
        "#sss###sss#",
        ".#########.",
    ];
    pub const ALBUM: &[&str] = &[
        "###########",
        "#ppppppppp#",
        "#pTTTTTaTp#",
        "#pTTTTTTTp#",
        "#pTTgTTTTp#",
        "#pgggggggp#",
        "#ppppppppp#",
        "###########",
    ];
    pub const SOUND: &[&str] = &[
        "....#....#.",
        "...#p#.#..#",
        "####p#..#.#",
        "#pppp#..#.#",
        "####p#..#.#",
        "...#p#.#..#",
        "....#....#.",
    ];
    pub const QUIET: &[&str] = &[
        "....#......",
        "...#p#.....",
        "####p#.#.#.",
        "#pppp#..#..",
        "####p#.#.#.",
        "...#p#.....",
        "....#......",
    ];
    pub const JOURNAL: &[&str] = &[
        ".########.",
        "#gggggggG#",
        "#gG####GG#",
        "#gG#pp#GG#",
        "#gG####GG#",
        "#gGGGGGGG#",
        "#gGGGGGGG#",
        "#########.",
        "#ppppppps#",
        ".####rr##.",
    ];
    pub const SIGNPOST: &[&str] = &[
        "....##....",
        "########..",
        "#aaaaaaa#.",
        "#aaaaaaaa#",
        "#ddddddd#.",
        "########..",
        "....##....",
        "...####...",
    ];
    pub const PAT: &[&str] = &[
        ".##...##.",
        "#rr#.#rr#",
        "#rrr#rrR#",
        "#rrrrrrR#",
        ".#rrrrR#.",
        "..#rrR#..",
        "...#R#...",
        "....#....",
    ];
    pub const BRUSH: &[&str] = &[
        ".#.#.#.#..",
        ".#.#.#.#..",
        "#########.",
        "#bbbbbbbb#",
        ".#BBBBBB##",
        "......##B#",
        ".......#B#",
        ".......###",
    ];
}

/// Desktop's own picture for something its menu offers, cut from its UI atlas, so a snack held
/// out at the Hill looks like a snack offered on the desktop.
pub fn desktop_picture(icon: formiga_art::MenuIcon) -> Canvas {
    use std::sync::OnceLock;
    static ATLAS: OnceLock<Canvas> = OnceLock::new();
    let atlas = ATLAS.get_or_init(formiga_art::UiAtlasRenderer::render);
    let sprite = formiga_art::UiAtlasRenderer::menu_icon(icon, false);
    let mut cell = Canvas::new(sprite.width, sprite.height);
    for y in 0..sprite.height as i32 {
        for x in 0..sprite.width as i32 {
            cell.set(x, y, atlas.get(sprite.x as i32 + x, sprite.y as i32 + y));
        }
    }
    // Only the drawn part of its cell, so it sits in a button as Hill's own pictures do.
    let Some((left, top, right, bottom)) = cell.alpha_bounds() else {
        return cell;
    };
    let mut out = Canvas::new(right - left + 1, bottom - top + 1);
    for y in 0..out.height() as i32 {
        for x in 0..out.width() as i32 {
            out.set(x, y, cell.get(left as i32 + x, top as i32 + y));
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// The rest of egui, in paper
// ---------------------------------------------------------------------------------------------

/// Dresses what egui still draws itself (scroll bars, the odd tooltip) in the same paper, in
/// both light and dark: Desktop's bubbles are the same pixels on any desktop, and so is this.
pub fn dress(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    let stroke = |color| egui::Stroke::new(1.0, color);
    visuals.override_text_color = Some(INK);
    visuals.weak_text_color = Some(FADED);
    visuals.window_fill = PAPER;
    visuals.panel_fill = PAPER;
    visuals.window_stroke = stroke(OUTLINE);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    let shadow = egui::Shadow {
        offset: [2, 2],
        blur: 0,
        spread: 0,
        color: SHADOW,
    };
    visuals.window_shadow = shadow;
    visuals.popup_shadow = shadow;
    visuals.extreme_bg_color = SHADE;
    visuals.faint_bg_color = rgb(250, 238, 214);
    visuals.selection.bg_fill = CHOSEN;
    visuals.selection.stroke = stroke(OUTLINE);
    for (widget, fill) in [
        (&mut visuals.widgets.noninteractive, PAPER),
        (&mut visuals.widgets.inactive, SHADE),
        (&mut visuals.widgets.hovered, HOVER),
        (&mut visuals.widgets.active, CHOSEN),
        (&mut visuals.widgets.open, CHOSEN),
    ] {
        widget.bg_fill = fill;
        widget.weak_bg_fill = fill;
        widget.bg_stroke = stroke(OUTLINE);
        widget.fg_stroke = stroke(INK);
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.expansion = 0.0;
    }
    visuals.widgets.noninteractive.bg_stroke = stroke(SHADE);
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.set_visuals_of(theme, visuals.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_stays_on_whole_screen_pixels_at_about_half_a_scene_pixel() {
        for (ppp, scene_pixels, zoom, expected) in [
            (1.0, 3.0, 1.0, 2.0),
            (2.0, 3.0, 1.0, 2.0),
            (2.0, 5.0, 1.0, 3.0),
            (1.0, 1.0, 1.0, 1.0),
            (2.0, 4.0, 1.5, 3.0),
        ] {
            let scene = Rect::from_min_size(Pos2::ZERO, vec2(384.0 * scene_pixels / ppp, 216.0));
            let grain = Grain::for_scene(scene, 384, ppp, zoom);
            assert_eq!(
                grain.point * ppp,
                expected,
                "at {ppp}x, {scene_pixels} a pixel"
            );
        }
    }

    #[test]
    fn a_paragraph_is_as_tall_as_its_lines_without_a_gap_under_the_last() {
        assert_eq!(paragraph_height(0), 0);
        assert_eq!(paragraph_height(1), ASCENT + DESCENT);
        assert_eq!(paragraph_height(3), 2 * LINE + ASCENT + DESCENT);
    }

    #[test]
    fn every_picture_is_small_enough_for_a_button() {
        for art in [
            pictures::CAMERA,
            pictures::ALBUM,
            pictures::SOUND,
            pictures::QUIET,
            pictures::JOURNAL,
            pictures::SIGNPOST,
            pictures::PAT,
            pictures::BRUSH,
        ] {
            let picture = picture(art);
            assert!(picture.width() <= 12 && picture.height() as i32 + 4 <= BUTTON_HEIGHT);
            assert!(picture.alpha_bounds().is_some());
        }
    }

    #[test]
    fn desktops_snack_and_toy_come_from_its_own_menu() {
        for icon in [formiga_art::MenuIcon::Snack, formiga_art::MenuIcon::Toy] {
            let picture = desktop_picture(icon);
            assert!(picture.alpha_bounds().is_some(), "{icon:?} is empty");
            assert!(
                picture.height() as i32 + 4 <= BUTTON_HEIGHT,
                "{icon:?} makes its button taller than the rest"
            );
        }
    }
}
