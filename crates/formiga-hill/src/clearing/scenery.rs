//! The clearing that isn't on any map: deep in the Woods and somehow also not, under a sky gone
//! violet, the trees standing back in a ring, the ground underfoot faintly ruled like a desktop.

use super::{SCENE_HEIGHT, SCENE_WIDTH};
use crate::paint::{chance, ellipse, line, mix, noise, put, rgb, rgba};
use formiga_art::Canvas;

pub fn backdrop() -> Canvas {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    // The sky, deep violet going to a low magenta glow behind where the Sovereign hangs.
    for y in 0..height {
        let t = (y as f32 / 150.0).min(1.0);
        let band = (t * 6.0).floor() / 6.0;
        scene.fill_rect(0, y, width, 1, mix(rgb(0x0e0a1e), rgb(0x4a1e5a), band));
    }
    for index in 0..60 {
        let x = (noise(index, 0, 901) % width as u32) as i32;
        let y = (noise(index, 1, 902) % 110) as i32;
        put(
            &mut scene,
            x,
            y,
            rgba(0xf0e8ff, 90 + (noise(index, 2, 903) % 150) as u8),
        );
    }
    // Lines of force running out from where it hangs.
    let centre = (300.0, 104.0);
    for ray in 0..40 {
        let angle = ray as f32 / 40.0 * std::f32::consts::TAU;
        let (inner, outer) = (40.0, 260.0);
        let from = (
            centre.0 + angle.cos() * inner,
            centre.1 + angle.sin() * inner * 0.7,
        );
        let to = (
            centre.0 + angle.cos() * outer,
            centre.1 + angle.sin() * outer * 0.7,
        );
        if ray % 3 != 0 {
            line(
                &mut scene,
                (from.0 as i32, from.1 as i32),
                (to.0 as i32, to.1 as i32),
                rgba(0x9a6ad8, 26),
            );
        }
    }
    ellipse(
        &mut scene,
        centre.0 as i32,
        centre.1 as i32,
        70,
        50,
        rgba(0xc070e0, 24),
    );
    // The trees, standing back in a ring, dark against it.
    for index in 0..18 {
        let x = index * 24 - 6 + (noise(index, 0, 904) % 10) as i32;
        let top = 70 + (noise(index, 1, 905) % 40) as i32;
        let trunk = rgb(0x140e1c);
        scene.fill_rect(x + 6, top, 5, 160 - top, trunk);
        ellipse(&mut scene, x + 8, top, 14 + (index % 3), 18, rgb(0x1a1226));
        ellipse(&mut scene, x + 4, top + 6, 10, 12, rgb(0x160f20));
    }
    // The ground: dark, glossy, ruled into squares like a desktop seen too close.
    for y in 150..height {
        let near = (y - 150) as f32 / (height - 150) as f32;
        scene.fill_rect(0, y, width, 1, mix(rgb(0x1a1228), rgb(0x2a1e3a), near));
    }
    for row in 0..8 {
        let y = 150 + row * row + row * 4;
        if y < height {
            line(
                &mut scene,
                (0, y),
                (width, y),
                rgba(0xb48af0, 34 + row as u8 * 4),
            );
        }
    }
    for column in -12..=12 {
        let x_far = 192 + column * 14;
        let x_near = 192 + column * 44;
        line(
            &mut scene,
            (x_far, 150),
            (x_near, height),
            rgba(0xb48af0, 30),
        );
    }
    for y in 150..height {
        for x in 0..width {
            if chance(x, y, 906, 3) {
                put(&mut scene, x, y, rgba(0xe0d0ff, 60));
            }
        }
    }
    scene
}

pub fn foreground() -> Canvas {
    Canvas::new(SCENE_WIDTH, SCENE_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_fills_every_pixel() {
        assert!(backdrop().pixels().iter().all(|pixel| pixel.a == 255));
    }
}
