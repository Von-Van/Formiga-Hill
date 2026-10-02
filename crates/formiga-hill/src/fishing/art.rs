//! How each fish looks when it is landed and held up, and its icon for the journal. (Plain
//! stand-ins until they are drawn.)

use super::fish::fish;
use crate::paint::{ellipse, rgb};
use formiga_art::Canvas;

/// Icons are this many pixels square, as finds' are.
pub const ICON: u32 = 9;

/// How long a fish of `length` centimetres is drawn, in pixels, when it is held up.
pub fn drawn_length(length: f32) -> u32 {
    (8.0 + length * 0.36).round() as u32
}

/// A fish side on, facing right, sized for the biggest of its kind; scaled down for smaller ones
/// by `held`.
pub fn catch(id: &str) -> Canvas {
    let length = fish(id).map_or(20.0, |fish| fish.length.1);
    let width = drawn_length(length);
    let height = (width / 3).max(5);
    let mut canvas = Canvas::new(width + 2, height + 2);
    ellipse(
        &mut canvas,
        (width / 2) as i32,
        (height / 2 + 1) as i32,
        (width / 2) as i32,
        (height / 2) as i32,
        rgb(0x5a6a72),
    );
    canvas
}

pub fn icon(id: &str) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    if fish(id).is_some() {
        ellipse(&mut canvas, 4, 4, 4, 2, rgb(0x5a6a72));
    }
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::fish::CATALOGUE;

    #[test]
    fn every_fish_has_a_catch_and_an_icon() {
        for fish in &CATALOGUE {
            let icon = icon(fish.id);
            assert_eq!((icon.width(), icon.height()), (ICON, ICON));
            assert!(
                icon.pixels().iter().filter(|pixel| pixel.a > 0).count() >= 12,
                "{}",
                fish.id
            );
            let catch = catch(fish.id);
            assert!(
                catch.width() >= drawn_length(fish.length.1),
                "{} is drawn too short",
                fish.id
            );
            assert!(
                catch.width() <= 64 && catch.height() <= 32,
                "{} is too big to hold up",
                fish.id
            );
        }
    }
}
