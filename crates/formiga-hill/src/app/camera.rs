//! The camera and the album, in the window. With the camera out, a viewfinder follows the
//! pointer over the scene (the scroll wheel makes it bigger or smaller) and a click takes a
//! photo of what is in it, into the colony's album. The album shows them all, and saves a copy
//! of any wherever the person chooses. Photos are Hill's: nothing goes to Desktop.

use super::{Card, HillApp, SCENE_HEIGHT, SCENE_WIDTH, paper, plural};
use crate::audio::Cue;
use crate::photos::{self, Album};
use eframe::egui;
use formiga_art::Canvas;
use time::OffsetDateTime;

/// The viewfinder's shape, wide like the scene, and how small and big it can be made.
const ASPECT: f32 = 16.0 / 9.0;
const SMALLEST: f32 = 64.0;
/// How long the flash shows after a photo is taken.
const FLASH_SECS: f32 = 0.25;
/// How much bigger a saved copy is than the photo, each way.
const COPY_SCALE: u32 = 4;

/// The camera's state through the visit.
pub struct Camera {
    pub out: bool,
    /// How wide the viewfinder is, in scene pixels.
    pub width: f32,
    album: Option<Album>,
    /// The photo being looked at in the album, if one is.
    viewing: Option<usize>,
    /// When the last photo was taken, for the flash.
    flashed: Option<f32>,
    /// Pictures for the album, drawn once each.
    thumbnails: Vec<(String, egui::TextureHandle)>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            out: false,
            width: 192.0,
            album: None,
            viewing: None,
            flashed: None,
            thumbnails: Vec::new(),
        }
    }
}

/// The viewfinder at `pointer` (its middle), kept within the scene: left, top, width, height.
pub fn viewfinder(pointer: (f32, f32), width: f32) -> (i32, i32, i32, i32) {
    let (scene_w, scene_h) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    // Whole pixels first, so the frame never reaches past the scene's edge.
    let width = (width.clamp(SMALLEST, SCENE_WIDTH as f32).round() as i32).min(scene_w);
    let height = ((width as f32 / ASPECT).round() as i32).min(scene_h);
    let left = (pointer.0.round() as i32 - width / 2).clamp(0, scene_w - width);
    let top = (pointer.1.round() as i32 - height / 2).clamp(0, scene_h - height);
    (left, top, width, height)
}

/// What is inside the viewfinder.
pub fn crop(scene: &Canvas, (left, top, width, height): (i32, i32, i32, i32)) -> Canvas {
    let mut photo = Canvas::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            photo.set(x, y, scene.get(left + x, top + y));
        }
    }
    photo
}

impl HillApp {
    fn album(&mut self) -> &mut Album {
        let colony = self.arrival.cast.snapshot.colony_id.clone();
        self.camera.album.get_or_insert_with(|| {
            Album::open(crate::memories::Memories::folder().as_deref(), &colony)
        })
    }

    /// The album and camera buttons, beside the sound wherever the camera can go.
    pub(super) fn camera_buttons(&mut self, ui: &mut egui::Ui) {
        let album = ui.add(
            paper::Button::new("")
                .picture(paper::picture(paper::pictures::ALBUM), 1)
                .selected(self.showing(Card::Album)),
        );
        paper::name_it(ui, &album, "Album");
        if album.clicked() {
            self.toggle(Card::Album);
            self.camera.viewing = None;
        }
        let camera = ui.add(
            paper::Button::new("")
                .picture(paper::picture(paper::pictures::CAMERA), 1)
                .selected(self.camera.out),
        );
        paper::name_it(ui, &camera, "Camera  C");
        if camera.clicked() {
            self.camera.out = !self.camera.out;
        }
    }

    /// The scroll wheel sizes the viewfinder; a click takes the photo.
    pub(super) fn use_camera(&mut self, clicked: bool, scroll: f32, now: f32) {
        if !self.camera.out {
            return;
        }
        self.camera.width = (self.camera.width - scroll * 0.4).clamp(SMALLEST, SCENE_WIDTH as f32);
        let (Some(pointer), true) = (self.pointer, clicked) else {
            return;
        };
        let Some(scene) = self.last_scene.as_ref() else {
            return;
        };
        let photo = crop(scene, viewfinder(pointer, self.camera.width));
        let line = match self.album().keep(photo, OffsetDateTime::now_utc()) {
            Ok(()) => {
                self.camera.flashed = Some(now);
                self.sound.play(Cue::Shutter);
                "Snap! It's in the album.".to_owned()
            }
            Err(error) if self.album().is_full() => {
                format!("The album is full ({error}). Take some out to make room.")
            }
            Err(error) => format!("The photo couldn't be kept: {error}."),
        };
        self.notice = Some((line, now));
    }

    /// The viewfinder over the scene, and the flash after a photo.
    pub(super) fn draw_camera(
        &self,
        painter: &egui::Painter,
        to_screen: impl Fn(f32, f32) -> egui::Pos2,
        rect: egui::Rect,
        now: f32,
    ) {
        if let Some(since) = self.camera.flashed {
            let t = (now - since) / FLASH_SECS;
            // With reduced motion, a single white frame rather than a fade.
            let alpha = if self.arrival.cast.reduce_motion() {
                if t < 0.5 { 120 } else { 0 }
            } else {
                ((1.0 - t).max(0.0) * 200.0) as u8
            };
            if alpha > 0 {
                painter.rect_filled(rect, 0.0, egui::Color32::from_white_alpha(alpha));
            }
        }
        if !self.camera.out {
            return;
        }
        let Some(pointer) = self.pointer else {
            return;
        };
        let (left, top, width, height) = viewfinder(pointer, self.camera.width);
        let frame = egui::Rect::from_min_max(
            to_screen(left as f32, top as f32),
            to_screen((left + width) as f32, (top + height) as f32),
        );
        // Dim what is outside the frame, and mark its corners.
        let shade = egui::Color32::from_black_alpha(90);
        for outside in [
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, frame.min.y)),
            egui::Rect::from_min_max(egui::pos2(rect.min.x, frame.max.y), rect.max),
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x, frame.min.y),
                egui::pos2(frame.min.x, frame.max.y),
            ),
            egui::Rect::from_min_max(
                egui::pos2(frame.max.x, frame.min.y),
                egui::pos2(rect.max.x, frame.max.y),
            ),
        ] {
            painter.rect_filled(outside, 0.0, shade);
        }
        let corner = frame.width().min(frame.height()) * 0.15;
        let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0xf6, 0xee, 0xd8));
        for (at, dx, dy) in [
            (frame.left_top(), 1.0, 1.0),
            (frame.right_top(), -1.0, 1.0),
            (frame.left_bottom(), 1.0, -1.0),
            (frame.right_bottom(), -1.0, -1.0),
        ] {
            painter.line_segment([at, at + egui::vec2(corner * dx, 0.0)], stroke);
            painter.line_segment([at, at + egui::vec2(0.0, corner * dy)], stroke);
        }
    }

    /// The album: every photo, and one looked at closely, with a copy to save.
    pub(super) fn album_window(&mut self, ctx: &egui::Context, now: f32) {
        if !self.showing(Card::Album) {
            return;
        }
        self.album();
        let names: Vec<String> = self
            .camera
            .album
            .as_ref()
            .map(|album| {
                album
                    .photos
                    .iter()
                    .map(|photo| photo.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        // Pictures for any photo not drawn yet; none kept for photos gone from the album.
        self.camera
            .thumbnails
            .retain(|(name, _)| names.contains(name));
        if let Some(album) = &self.camera.album {
            for photo in &album.photos {
                if !self
                    .camera
                    .thumbnails
                    .iter()
                    .any(|(name, _)| *name == photo.name)
                {
                    let texture = ctx.load_texture(
                        format!("photo-{}", photo.name),
                        egui::ColorImage::from_rgba_unmultiplied(
                            [
                                photo.picture.width() as usize,
                                photo.picture.height() as usize,
                            ],
                            &photo.picture.rgba_bytes(),
                        ),
                        egui::TextureOptions::NEAREST,
                    );
                    self.camera.thumbnails.push((photo.name.clone(), texture));
                }
            }
        }
        let offset = match self.clock {
            crate::daylight::Clock::Local(offset) => offset,
            crate::daylight::Clock::Held(_) => time::UtcOffset::UTC,
        };
        let mut chosen = None;
        let mut save = None;
        let mut remove = None;
        let mut back = false;
        self.show_card(ctx, Card::Album, "Album", 0.5, |app, ui| {
            let Some(album) = &app.camera.album else {
                return;
            };
            if album.photos.is_empty() {
                paper::aside(
                    ui,
                    "No photos yet. Take the camera out (C), frame something, and click.",
                );
                return;
            }
            let texture = |name: &str| {
                app.camera
                    .thumbnails
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, texture)| texture)
            };
            let grain = paper::Grain::of(ui.ctx());
            match app
                .camera
                .viewing
                .filter(|index| *index < album.photos.len())
            {
                Some(index) => {
                    let photo = &album.photos[index];
                    if let Some(texture) = texture(&photo.name) {
                        // At whole pixels, as big as the card allows.
                        let width = photo.picture.width() as f32;
                        let height = photo.picture.height() as f32;
                        let ppp = ui.ctx().pixels_per_point();
                        let fit = (ui.available_width() * ppp / width).floor().max(1.0) / ppp;
                        framed(ui, texture, egui::vec2(width, height) * fit, grain, false);
                    }
                    paper::aside(ui, when(photo.taken, offset));
                    ui.horizontal_wrapped(|ui| {
                        back = paper::button(ui, "\u{25c2} All photos").clicked();
                        if paper::button(ui, "Save a copy\u{2026}").clicked() {
                            save = Some(index);
                        }
                        if paper::button(ui, "Take out of the album").clicked() {
                            remove = Some(index);
                        }
                    });
                }
                None => {
                    paper::aside(
                        ui,
                        format!(
                            "{} photo{} \u{b7} click one to look closer",
                            album.photos.len(),
                            plural(album.photos.len())
                        ),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for (index, photo) in album.photos.iter().enumerate().rev() {
                            let Some(texture) = texture(&photo.name) else {
                                continue;
                            };
                            let height = grain.len(40);
                            let width = height * photo.picture.width() as f32
                                / photo.picture.height().max(1) as f32;
                            let shown = framed(ui, texture, egui::vec2(width, height), grain, true);
                            if paper::explain(ui, shown, &when(photo.taken, offset)).clicked() {
                                chosen = Some(index);
                            }
                        }
                    });
                }
            }
        });
        if back {
            self.camera.viewing = None;
        }
        if chosen.is_some() {
            self.camera.viewing = chosen;
        }
        if let Some(index) = save {
            self.save_copy(index, now);
        }
        if let Some(index) = remove {
            let line = match self.album().remove(index) {
                Ok(()) => "Taken out of the album.".to_owned(),
                Err(error) => format!("The photo couldn't be taken out: {error}."),
            };
            self.camera.viewing = None;
            self.notice = Some((line, now));
        }
    }

    /// Asks where to save a copy of a photo, and saves it there, bigger by whole pixels.
    fn save_copy(&mut self, index: usize, now: f32) {
        let Some(photo) = self.album().photos.get(index) else {
            return;
        };
        let picture = photo.picture.clone();
        let name = format!("Formiga Hill {}.png", photo.name);
        let Some(path) = rfd::FileDialog::new()
            .set_title("Save a copy of the photo")
            .set_file_name(&name)
            .add_filter("PNG image", &["png"])
            .save_file()
        else {
            return;
        };
        let line = match photos::save_copy(&picture, &path, COPY_SCALE) {
            Ok(()) => format!("Saved a copy to {}.", path.display()),
            Err(error) => format!("The copy couldn't be saved: {error}."),
        };
        self.notice = Some((line, now));
    }
}

/// A photo in a plum frame, as a print is mounted; one to click lights up when pointed at.
fn framed(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    size: egui::Vec2,
    grain: paper::Grain,
    clickable: bool,
) -> egui::Response {
    let border = grain.len(2);
    let sense = if clickable {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size + egui::vec2(border, border) * 2.0, sense);
    let lit = clickable && response.hovered();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, if lit { paper::CHOSEN } else { paper::OUTLINE });
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    painter.image(texture.id(), rect.shrink(border), uv, egui::Color32::WHITE);
    if lit {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// When a photo was taken, in the person's own time.
fn when(taken: OffsetDateTime, offset: time::UtcOffset) -> String {
    let local = taken.to_offset(offset);
    local
        .format(time::macros::format_description!(
            "[day padding:none] [month repr:long] [year], [hour]:[minute]"
        ))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_viewfinder_stays_within_the_scene_and_keeps_its_shape() {
        for pointer in [(0.0, 0.0), (384.0, 216.0), (192.0, 108.0), (-50.0, 500.0)] {
            for width in [10.0, 120.0, 500.0] {
                let (left, top, w, h) = viewfinder(pointer, width);
                assert!(left >= 0 && top >= 0);
                assert!(left + w <= SCENE_WIDTH as i32 && top + h <= SCENE_HEIGHT as i32);
                assert!(w >= SMALLEST as i32);
                assert!(((w as f32 / h as f32) - ASPECT).abs() < 0.1);
            }
        }
    }

    #[test]
    fn a_photo_is_exactly_what_was_in_the_frame() {
        let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        scene.fill_rect(100, 50, 10, 10, crate::paint::rgb(0xd0574a));
        let frame = viewfinder((105.0, 55.0), 64.0);
        let photo = crop(&scene, frame);
        assert_eq!(photo.width() as i32, frame.2);
        assert_eq!(
            photo.get(100 - frame.0, 50 - frame.1),
            crate::paint::rgb(0xd0574a)
        );
    }
}
