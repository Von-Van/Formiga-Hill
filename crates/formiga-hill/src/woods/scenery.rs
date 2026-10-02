//! The glade, painted. (A plain stand-in until the scenery is drawn.)

use super::SPOTS;
use crate::paint::{ellipse, mix, rgb};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::Canvas;

pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in 0..SCENE_HEIGHT as i32 {
        let t = y as f32 / SCENE_HEIGHT as f32;
        let color = if y < 112 {
            mix(rgb(0x2c4a34), rgb(0x4a6e48), t)
        } else if y < 130 {
            rgb(0x4a7f8a)
        } else {
            mix(rgb(0x5a7a3e), rgb(0x46622f), t)
        };
        scene.fill_rect(0, y, SCENE_WIDTH as i32, 1, color);
    }
    for spot in SPOTS {
        ellipse(
            &mut scene,
            spot.sign.0 as i32,
            spot.sign.1 as i32,
            6,
            3,
            rgb(0x3a2a22),
        );
    }
    scene
}

pub fn foreground() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}
