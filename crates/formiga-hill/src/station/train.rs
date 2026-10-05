//! The train: a little tank engine and two coaches, built to the colony's size, so a traveller
//! fits at a window and the train cuts anyone standing behind it clearly at the middle.
//!
//! The train itself is `formiga-art`'s, the same one Formiga Desktop sends off across the desktop,
//! so the train that leaves is the one that arrives. Hill bakes its frames once, by day and lit
//! after dark, and picks one by how far the train has rolled, so the wheels turn as far as it has
//! gone. Hill adds only whoever is looking out of the windows: the travellers themselves, behind
//! the glass.

use crate::paint::{hline, put, rect, rgba};
use formiga_art::{
    Canvas, RUNNING_FRAMES, TRAIN_FRAMES, TRAIN_GROUND, TRAIN_WIDTH, TrainLook, TrainRenderer,
};

/// From the back of the last coach to the front buffers.
pub const TRAIN_LENGTH: i32 = TRAIN_WIDTH as i32;
/// The scene row the train's own top row (its roof vents) sits on: the wheels then meet the
/// near rail.
pub const TRAIN_TOP: i32 = 136;
/// Seats with a window.
pub const WINDOWS: usize = 6;

const COACH_LENGTH: i32 = 66;
const COUPLING: i32 = 4;
/// Where the train's own top row is in `formiga-art`'s frames, below the room left for steam.
const HEADROOM: i32 = TRAIN_GROUND as i32 - (AXLE + 3);
const AXLE: i32 = 59;
const WINDOW_SIZE: (i32, i32) = (14, 13);
const WINDOW_TOP: i32 = 18;
const WINDOW_XS: [i32; 3] = [8, 26, 44];
/// The driving wheels' radius: a full turn of them is this much travel, times τ.
pub const DRIVING_WHEEL: f32 = 7.0;
/// How often the standing train puffs, from one of its two standing frames to the other.
const PUFFS_PER_SECOND: f32 = 1.3;

pub struct Train {
    /// Every frame, by day and lit, in `formiga-art`'s order: running, then standing.
    day: Vec<Canvas>,
    lit: Vec<Canvas>,
    reduce_motion: bool,
}

/// Someone looking out of a window: their current frame, and where their face is in it.
pub struct Passenger<'a> {
    pub window: usize,
    pub frame: &'a Canvas,
    /// The centre of the face, in `frame`'s pixels, as `formiga-art` placed it.
    pub face: (i32, i32),
}

/// How the train is moving, which decides the frame it is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Going {
    /// Rolling, this far so far in pixels, for how far round the wheels are.
    Rolling(f32),
    /// Stood at the platform, puffing.
    Standing,
}

impl Train {
    pub fn new(reduce_motion: bool) -> Self {
        let bake = |lit| {
            (0..TRAIN_FRAMES)
                .map(|frame| TrainRenderer::render(&TrainLook { lit }, frame))
                .collect()
        };
        Self {
            day: bake(false),
            lit: bake(true),
            reduce_motion,
        }
    }

    /// The train with its left end at `x`, rolling or standing, its compartments lit after dark,
    /// and the passengers at their windows. With reduced motion a standing train holds one puff.
    pub fn draw(
        &self,
        scene: &mut Canvas,
        x: i32,
        going: Going,
        lit: bool,
        now: f32,
        passengers: &[Passenger<'_>],
    ) {
        let frame = match going {
            Going::Rolling(rolled) => {
                let turn = std::f32::consts::TAU * DRIVING_WHEEL;
                let eighths = (rolled.max(0.0) / turn * f32::from(RUNNING_FRAMES)) as usize;
                eighths % usize::from(RUNNING_FRAMES)
            }
            Going::Standing if self.reduce_motion => usize::from(RUNNING_FRAMES),
            Going::Standing => usize::from(RUNNING_FRAMES) + (now * PUFFS_PER_SECOND) as usize % 2,
        };
        let frames = if lit { &self.lit } else { &self.day };
        crate::paint::blit(scene, &frames[frame], x, TRAIN_TOP - HEADROOM);

        for passenger in passengers {
            let (wx, wy) = window_origin(passenger.window);
            let (left, top) = (x + wx, TRAIN_TOP + wy);
            let (width, height) = WINDOW_SIZE;
            // The face in the middle of the window; whatever does not fit is out of sight.
            let frame_x = left + width / 2 - passenger.face.0;
            let frame_y = top + height / 2 - passenger.face.1;
            let size = formiga_art::FRAME_SIZE as i32;
            for fy in 0..size {
                for fx in 0..size {
                    let (sx, sy) = (frame_x + fx, frame_y + fy);
                    if sx < left || sy < top || sx >= left + width || sy >= top + height {
                        continue;
                    }
                    let pixel = passenger.frame.get(fx, fy);
                    if pixel.a > 0 {
                        put(scene, sx, sy, pixel);
                    }
                }
            }
            // The glass again, over whoever is behind it.
            glass(scene, left, top);
        }
    }
}

fn window_origin(window: usize) -> (i32, i32) {
    let coach = (window / 3) as i32;
    (
        coach * (COACH_LENGTH + COUPLING) + WINDOW_XS[window % 3],
        WINDOW_TOP,
    )
}

/// The glass over a window, as `formiga-art` lays it: a faint sheen and a glint, so a face reads
/// as behind it.
fn glass(scene: &mut Canvas, x: i32, y: i32) {
    let (width, height) = WINDOW_SIZE;
    rect(scene, x, y, width, height, rgba(0xa6c8d2, 30));
    hline(scene, x, y, width, rgba(0x2e2a33, 110));
    for (dx, dy) in [(1, 4), (2, 3), (3, 2), (4, 1)] {
        put(scene, x + width - 6 + dx, y + dy, rgba(0xffffff, 150));
    }
    put(scene, x + 1, y + height - 2, rgba(0xffffff, 60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheels_meet_the_rail() {
        let wheels_bottom = TRAIN_TOP - HEADROOM + TRAIN_GROUND as i32;
        assert!(
            (197..=199).contains(&wheels_bottom),
            "the wheels bottom out at {wheels_bottom}"
        );
    }

    #[test]
    fn the_train_cuts_anyone_behind_it_at_the_middle_not_the_feet() {
        // Standing travellers are about 34 pixels tall; the coach roof should cross them well up
        // their bodies, so nobody looks as though they are standing on it.
        // The coach roof's top, below the vents, in the train's own rows.
        const ROOF_TOP: i32 = 8;
        let roof = TRAIN_TOP + ROOF_TOP;
        assert!(super::super::STAND_Y - roof >= 12, "the roof is at {roof}");
    }

    #[test]
    fn every_window_is_on_the_train() {
        for window in 0..WINDOWS {
            let (x, y) = window_origin(window);
            assert!(x >= 0 && x + WINDOW_SIZE.0 <= TRAIN_LENGTH);
            assert!(y >= 0 && y + WINDOW_SIZE.1 <= AXLE);
        }
    }

    #[test]
    fn turning_wheels_change_the_picture_and_a_held_puff_does_not() {
        let draw = |going, now, reduce_motion| {
            let mut scene = Canvas::new(400, 216);
            Train::new(reduce_motion).draw(&mut scene, 10, going, false, now, &[]);
            scene
        };
        assert_ne!(
            draw(Going::Rolling(0.0), 0.0, false),
            draw(Going::Rolling(8.0), 0.0, false)
        );
        assert_eq!(
            draw(Going::Standing, 0.0, true),
            draw(Going::Standing, 5.3, true)
        );
    }

    #[test]
    fn after_dark_the_compartments_are_lit() {
        let train = Train::new(true);
        let draw = |lit| {
            let mut scene = Canvas::new(400, 216);
            train.draw(&mut scene, 10, Going::Standing, lit, 0.0, &[]);
            scene
        };
        assert_ne!(draw(false), draw(true));
    }
}
