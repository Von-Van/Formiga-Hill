//! The meadow at the Woods' edge. A placeholder until it is painted: sky, the treeline, grass,
//! and each haunt blocked in where it goes.

use super::{GROUND, Haunt, POND};
use crate::hilltop::Arrangement;
use crate::materials::*;
use crate::paint::{ellipse, rect, rgb};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::Canvas;

pub fn backdrop(hilltop: &Arrangement) -> Canvas {
    let _ = hilltop;
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut scene);
    rect(&mut scene, 0, 30, SCENE_WIDTH as i32, 50, LEAF.shadow);
    rect(
        &mut scene,
        0,
        80,
        SCENE_WIDTH as i32,
        SCENE_HEIGHT as i32 - 80,
        GRASS,
    );
    let _ = GROUND;
    for haunt in Haunt::ALL {
        let (left, top, right, bottom) = haunt.area();
        let color = match haunt {
            Haunt::Flowers => BLOSSOMS[1],
            Haunt::Grass => GRASS_DARK,
            Haunt::Bramble => LEAF.edge,
            Haunt::Log => TIMBER.base,
            Haunt::Stump => TIMBER.shadow,
            Haunt::Reeds => LEAF.base,
        };
        rect(
            &mut scene,
            left as i32,
            top as i32,
            (right - left) as i32,
            (bottom - top) as i32,
            color,
        );
    }
    let (cx, cy, rx, ry) = POND;
    ellipse(
        &mut scene, cx as i32, cy as i32, rx as i32, ry as i32, GLASS[1],
    );
    scene
}

/// The sky on its own, clouds and all.
pub fn sky(scene: &mut Canvas) {
    rect(scene, 0, 0, SCENE_WIDTH as i32, 40, SKY_TOP);
}

/// Long grass drawn over everyone along the bottom edge.
pub fn foreground() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}

/// What glows of its own after dark.
pub fn lamplight() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}

/// The sky after dark.
pub fn night_sky(painted: &Canvas) -> Canvas {
    let mut only = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    sky(&mut only);
    crate::daylight::night_sky(painted, &only, 80, Some((60, 14)))
}

#[allow(dead_code)]
const PLACEHOLDER: formiga_art::Rgba = rgb(0xff00ff);
