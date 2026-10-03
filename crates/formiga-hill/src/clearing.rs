//! The clearing that isn't on any map: where, very rarely, something out of place is waiting in
//! the Woods. The one thing here is the Cursor Sovereign (see `sovereign`), played on a small
//! stage (see `stage`) that could one day carry other secret set pieces.

pub mod boss;
mod scenery;
pub mod sovereign;
pub mod stage;

use crate::cast::{Cast, Id};
use crate::font::{GLYPH_HEIGHT, draw_text, text_width};
use crate::paint::{blit, mix, rect, rgb, rgba};
use crate::playground::{Layout, Patch, Playground};
use formiga_art::{Canvas, Rgba};
use sovereign::{Sovereign, State};

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};

/// Where anyone can stand: the near half of the clearing, facing the Sovereign.
const GROUND: Patch = (24.0, 160.0, 236.0, 206.0);

fn walkable(x: f32, y: f32) -> bool {
    let (l, t, r, b) = GROUND;
    x >= l && x <= r && y >= t && y <= b
}

const PLACES: [(&str, (f32, f32)); 1] = [("centre", (130.0, 186.0))];

fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        entrance: (-24.0, 194.0),
        entrance_step: (-18.0, 4.0),
    }
}

/// The clearing, with the companions who found it walking in.
pub fn open(cast: &Cast, party: &[Id], now: f32) -> Playground {
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        layout(),
        scenery::backdrop(),
        scenery::foreground(),
        Vec::new(),
    );
    for (id, mark) in party.iter().zip(sovereign::MARKS) {
        ground.direct(
            *id,
            vec![
                crate::actor::Step::Walk { to: mark },
                crate::actor::Step::FaceX(300.0),
            ],
            now,
        );
    }
    ground
}

/// The whole picture: the clearing and everyone in it, the Sovereign, its name and health, any
/// close-up, and the shaking.
pub fn compose(ground: &mut Playground, sovereign: &Sovereign, now: f32) -> Canvas {
    let reduce_motion = ground.reduce_motion();
    let mut scene = ground.compose(now);
    let stage = &sovereign.stage;
    let (step, since) = stage.boss;
    boss::draw(&mut scene, step, now - since, now, reduce_motion);
    if sovereign.state() != State::Over {
        title(&mut scene, stage.health, stage.bar);
    }
    if let Some((who, name, since)) = &stage.cutin {
        let pictures: Vec<Canvas> = who
            .iter()
            .filter_map(|id| ground.picture(*id, now))
            .collect();
        cutin(&mut scene, &pictures, name, now - since, reduce_motion);
    }
    if now < stage.shake_until && !reduce_motion {
        let (dx, dy) = (
            ((now * 61.0).sin() * 2.5) as i32,
            ((now * 47.0).cos() * 1.5) as i32,
        );
        let mut shaken = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        shaken.fill_rect(0, 0, SCENE_WIDTH as i32, SCENE_HEIGHT as i32, rgb(0x0c0a14));
        blit(&mut shaken, &scene, dx, dy);
        scene = shaken;
    }
    scene
}

/// Lettering `scale` times the size of the sign font, with a drop shadow.
fn big_text(scene: &mut Canvas, x: i32, y: i32, text: &str, scale: i32, color: Rgba, shadow: Rgba) {
    let width = text_width(text).max(1);
    let mut small = Canvas::new(width as u32 + 1, GLYPH_HEIGHT as u32 + 1);
    draw_text(&mut small, 1, 1, text, shadow);
    draw_text(&mut small, 0, 0, text, color);
    for sy in 0..small.height() as i32 {
        for sx in 0..small.width() as i32 {
            let pixel = small.get(sx, sy);
            if pixel.a > 0 {
                rect(scene, x + sx * scale, y + sy * scale, scale, scale, pixel);
            }
        }
    }
}

/// The Sovereign's name across the top, and its health beneath: the yellow trailing behind the
/// red as a blow lands.
fn title(scene: &mut Canvas, health: f32, bar: f32) {
    const NAME: &str = "THE CURSOR SOVEREIGN";
    let width = SCENE_WIDTH as i32;
    let name_width = text_width(NAME) * 2;
    big_text(
        scene,
        (width - name_width) / 2,
        6,
        NAME,
        2,
        rgb(0xf6eed8),
        rgb(0x3a1440),
    );
    let (left, top, long) = (60, 24, width - 120);
    rect(scene, left - 2, top - 2, long + 4, 9, rgba(0x0c0a14, 220));
    scene.fill_rect(left, top, long, 5, rgb(0x2a1830));
    scene.fill_rect(left, top, (long as f32 * bar) as i32, 5, rgb(0xf5d25e));
    scene.fill_rect(left, top, (long as f32 * health) as i32, 5, rgb(0xd03a48));
    scene.fill_rect(left, top, (long as f32 * health) as i32, 1, rgb(0xf07080));
}

/// A close-up across the screen: a dark band of speed lines, the faces of whoever is acting
/// twice their size sliding in from the left, and the attack's name.
fn cutin(scene: &mut Canvas, pictures: &[Canvas], name: &str, t: f32, reduce_motion: bool) {
    let (top, height) = (62, 70);
    let width = SCENE_WIDTH as i32;
    let slide = if reduce_motion {
        1.0
    } else {
        (t / 0.25).min(1.0)
    };
    for y in top..top + height {
        let edge = ((y - top) as f32 / height as f32 - 0.5).abs() * 2.0;
        let tone = mix(rgb(0x3a1e68), rgb(0x120a24), edge);
        scene.fill_rect(0, y, width, 1, tone);
    }
    for streak in 0..26 {
        let y = top + 4 + ((streak * 37) % (height - 8));
        let length = 30 + (streak * 53) % 90;
        let x = if reduce_motion {
            (streak * 71) % width
        } else {
            (((streak * 71) as f32 + t * 600.0) as i32) % (width + length) - length
        };
        rect(scene, x, y, length, 1, rgba(0xc8b0ff, 70));
    }
    scene.fill_rect(0, top, width, 2, rgb(0xf5d25e));
    scene.fill_rect(0, top + height - 2, width, 2, rgb(0xf5d25e));
    for (index, picture) in pictures.iter().enumerate() {
        let target = 10 + index as i32 * 46;
        let x = (target as f32 * slide - 120.0 * (1.0 - slide)) as i32;
        let Some((left, picture_top, right, bottom)) = picture.alpha_bounds() else {
            continue;
        };
        for sy in picture_top as i32..=bottom as i32 {
            for sx in left as i32..=right as i32 {
                let pixel = picture.get(sx, sy);
                if pixel.a > 0 {
                    let (px, py) = (
                        x + (sx - left as i32) * 2,
                        top + 4 + (sy - picture_top as i32) * 2,
                    );
                    if py < top + height - 2 {
                        rect(scene, px, py, 2, 2, pixel);
                    }
                }
            }
        }
    }
    let scale = if text_width(name) * 2 > width - 150 {
        1
    } else {
        2
    };
    let name_width = text_width(name) * scale;
    let x = width - name_width - 10 + ((1.0 - slide) * 140.0) as i32;
    big_text(
        scene,
        x,
        top + height / 2 - 7,
        name,
        scale,
        rgb(0xf5d25e),
        rgb(0x1a0a20),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_reduced_motion_the_clearing_holds_still() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let party: Vec<Id> = cast.ids().take(2).collect();
        let mut ground = open(&cast, &party, 0.0);
        let mut sovereign = Sovereign::new(&mut ground, &cast, &party, false, 0.0);
        let mut now = 0.0;
        // Into the opening, where the screen shakes and the Sovereign would bob.
        while now < 0.5 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            sovereign.tick(&mut ground, &cast, now);
        }
        assert!(
            now < sovereign.stage.shake_until,
            "the opening shake has passed"
        );
        let first = compose(&mut ground, &sovereign, now);
        let later = compose(&mut ground, &sovereign, now + 0.07);
        assert_eq!(first, later, "something moved");
    }
}
