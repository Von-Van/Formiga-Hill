//! The pool, painted. (A plain stand-in until the scenery is drawn.)

use super::{Haunt, WATER};
use crate::paint::{ellipse, mix, rgb, rgba};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::Canvas;

pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in 0..SCENE_HEIGHT as i32 {
        let t = y as f32 / SCENE_HEIGHT as f32;
        scene.fill_rect(
            0,
            y,
            SCENE_WIDTH as i32,
            1,
            mix(rgb(0x2c4a34), rgb(0x5a7a3e), t),
        );
    }
    let (cx, cy, rx, ry) = WATER;
    ellipse(
        &mut scene,
        cx as i32,
        cy as i32,
        rx as i32 + 6,
        ry as i32 + 6,
        rgb(0x4a7f8a),
    );
    for haunt in Haunt::ALL {
        let ((x, y), r) = haunt.area();
        ellipse(
            &mut scene,
            x as i32,
            y as i32,
            r as i32,
            (r / 2.5) as i32,
            rgba(0xffffff, 20),
        );
    }
    scene
}

pub fn foreground() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}
