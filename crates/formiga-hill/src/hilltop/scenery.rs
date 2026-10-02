//! The summit, painted. (A plain stand-in until the scenery is drawn.)

use super::SPOTS;
use crate::paint::{ellipse, mix, rgb};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::Canvas;

pub fn backdrop() -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    for y in 0..SCENE_HEIGHT as i32 {
        let t = y as f32 / SCENE_HEIGHT as f32;
        let color = if y < 100 {
            mix(rgb(0xb9ddec), rgb(0xf6e8cf), t * 2.0)
        } else {
            mix(rgb(0x86bb7c), rgb(0x639858), t)
        };
        scene.fill_rect(0, y, SCENE_WIDTH as i32, 1, color);
    }
    for (x, y) in SPOTS {
        ellipse(&mut scene, x as i32, y as i32, 14, 3, rgb(0x97c784));
    }
    scene.fill_rect(44, 40, 20, 72, rgb(0x7a5a3a));
    scene.fill_circle(54, 40, 30, rgb(0x5d9a5a));
    scene
}

pub fn foreground() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}
