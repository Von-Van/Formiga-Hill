//! The train: a little tank engine and two coaches, built to the colony's size, so a traveller
//! fits at a window and the train cuts anyone standing behind it clearly at the middle.
//!
//! The bodies are painted once into a sprite. The wheels, the rods, the steam and whoever is
//! looking out of the windows are drawn every frame, so the wheels turn as far as the train has
//! rolled and the passengers are the travellers themselves.

use crate::paint::{Ramp, bevel, ellipse, hline, line, mix, put, rect, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};

/// From the back of the last coach to the front buffers.
pub const TRAIN_LENGTH: i32 = 202;
const TRAIN_HEIGHT: i32 = 64;
/// The scene row the sprite's top row sits on: the wheels then meet the near rail.
pub const TRAIN_TOP: i32 = 136;
/// Seats with a window.
pub const WINDOWS: usize = 6;

const COACH_LENGTH: i32 = 66;
const COUPLING: i32 = 4;
const LOCO_X: i32 = 2 * (COACH_LENGTH + COUPLING);

// The coach, top to bottom.
const ROOF_TOP: i32 = 8;
const BODY_TOP: i32 = 14;
const BELT: i32 = 34;
const BODY_BOTTOM: i32 = 54;
const AXLE: i32 = 59;
const BUFFERS: i32 = 46;

const WINDOW_SIZE: (i32, i32) = (14, 13);
const WINDOW_TOP: i32 = 18;
const WINDOW_XS: [i32; 3] = [8, 26, 44];

const LOCO: Ramp = Ramp::new(0x163829, 0x21503b, 0x2d6b4e, 0x418a66, 0x62ab84);
const SOOT: Ramp = Ramp::new(0x111114, 0x1d1e23, 0x2a2c32, 0x40434b, 0x5d616b);
const BRASS: Ramp = Ramp::new(0x6b4a24, 0x9a7434, 0xc9a14e, 0xe4c06c, 0xf6e3a2);
const BUFFER_RED: Ramp = Ramp::new(0x5a1a1c, 0x8c2a2a, 0xb33a33, 0xd4564a, 0xea806c);
const CREAM: Ramp = Ramp::new(0x8a7a63, 0xd8c9a8, 0xf0e4c6, 0xfaf3e1, 0xffffff);
const MAROON: Ramp = Ramp::new(0x3a1418, 0x5c2026, 0x7a2c33, 0x963c42, 0xb4585a);
const ROOF: Ramp = Ramp::new(0x4a4950, 0x6d6c73, 0x8a8990, 0xa9a8ae, 0xcfced3);
const STEEL: Ramp = Ramp::new(0x3f4249, 0x5d626b, 0x80858e, 0xb2b7bf, 0xe3e7ec);
const LINING: Rgba = rgb(0xe4c06c);
const INTERIOR: Rgba = rgb(0x3a3440);

/// A wheel of the train, relative to the sprite.
#[derive(Clone, Copy)]
struct Wheel {
    x: i32,
    y: i32,
    radius: i32,
}

const LOCO_WHEELS: [Wheel; 3] = [
    Wheel {
        x: LOCO_X + 14,
        y: AXLE - 4,
        radius: 7,
    },
    Wheel {
        x: LOCO_X + 29,
        y: AXLE - 4,
        radius: 7,
    },
    Wheel {
        x: LOCO_X + 44,
        y: AXLE - 4,
        radius: 7,
    },
];

pub struct Train {
    body: Canvas,
}

/// Someone looking out of a window: their current frame, and where their face is in it.
pub struct Passenger<'a> {
    pub window: usize,
    pub frame: &'a Canvas,
    /// The centre of the face, in `frame`'s pixels, as `formiga-art` placed it.
    pub face: (i32, i32),
}

impl Train {
    pub fn new() -> Self {
        let mut body = Canvas::new(TRAIN_LENGTH as u32, TRAIN_HEIGHT as u32);
        coach(&mut body, 0);
        coupling(&mut body, COACH_LENGTH);
        coach(&mut body, COACH_LENGTH + COUPLING);
        coupling(&mut body, LOCO_X - COUPLING);
        locomotive(&mut body, LOCO_X);
        Self { body }
    }

    /// Where the chimney's mouth is: across from the train's left end, and down the scene.
    pub fn chimney(&self) -> (i32, i32) {
        (LOCO_X + 54, TRAIN_TOP + 1)
    }

    /// The train with its left end at `x`, its wheels turned for `rolled` pixels of travel.
    pub fn draw(&self, scene: &mut Canvas, x: i32, rolled: f32, passengers: &[Passenger<'_>]) {
        // Shade on the ballast under the train.
        rect(
            scene,
            x + 2,
            TRAIN_TOP + BODY_BOTTOM + 2,
            TRAIN_LENGTH - 4,
            7,
            rgba(0x1e1a1e, 70),
        );

        crate::paint::blit(scene, &self.body, x, TRAIN_TOP);

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
        }
        for window in 0..WINDOWS {
            let (wx, wy) = window_origin(window);
            glass(scene, x + wx, TRAIN_TOP + wy);
        }

        // Wheels: a rotation is the wheel's circumference of travel.
        for coach_x in [0, COACH_LENGTH + COUPLING] {
            for wheel_x in [8, 16, 50, 58] {
                let wheel = Wheel {
                    x: coach_x + wheel_x,
                    y: AXLE,
                    radius: 3,
                };
                draw_wheel(scene, x, wheel, rolled, 2, STEEL);
            }
        }
        for wheel in LOCO_WHEELS {
            draw_wheel(scene, x, wheel, rolled, 6, LOCO);
        }
        rods(scene, x, rolled);
    }
}

/// Steam from the chimney, trailing back as far as the train's `speed` (pixels a second) carries
/// it. Held as a still wisp when motion is reduced.
pub fn steam(scene: &mut Canvas, chimney: (i32, i32), now: f32, speed: f32, reduce_motion: bool) {
    const PUFFS: usize = 6;
    for index in 0..PUFFS {
        let offset = index as f32 / PUFFS as f32;
        let age = if reduce_motion {
            offset * 0.6 + 0.1
        } else {
            (now * 1.3 + offset).fract()
        };
        let trail = 4.0 + speed.max(0.0) * 0.4;
        let x = chimney.0 as f32 - age * trail;
        let y = chimney.1 as f32 - 2.0 - age * 16.0;
        let radius = (1.5 + age * 4.0).round() as i32;
        let fade = ((1.0 - age) * 200.0) as u8;
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        ellipse(
            scene,
            cx + 1,
            cy + 1,
            radius,
            radius,
            rgba(0xbdb8bc, fade / 2),
        );
        ellipse(scene, cx, cy, radius, radius, rgba(0xf7f5f2, fade));
    }
}

fn window_origin(window: usize) -> (i32, i32) {
    let coach = (window / 3) as i32;
    (
        coach * (COACH_LENGTH + COUPLING) + WINDOW_XS[window % 3],
        WINDOW_TOP,
    )
}

/// The glass over a window: a faint sheen and a glint, so a face reads as behind it.
fn glass(scene: &mut Canvas, x: i32, y: i32) {
    let (width, height) = WINDOW_SIZE;
    rect(scene, x, y, width, height, rgba(0xa6c8d2, 30));
    hline(scene, x, y, width, rgba(0x2e2a33, 110));
    for (dx, dy) in [(1, 4), (2, 3), (3, 2), (4, 1)] {
        put(scene, x + width - 6 + dx, y + dy, rgba(0xffffff, 150));
    }
    put(scene, x + 1, y + height - 2, rgba(0xffffff, 60));
}

fn draw_wheel(scene: &mut Canvas, x: i32, wheel: Wheel, rolled: f32, spokes: usize, paint: Ramp) {
    let (cx, cy, radius) = (x + wheel.x, TRAIN_TOP + wheel.y, wheel.radius);
    let angle = rolled / radius as f32;
    ellipse(scene, cx, cy, radius, radius, SOOT.edge);
    ellipse(scene, cx, cy, radius - 1, radius - 1, paint.base);
    if radius > 3 {
        ellipse(scene, cx, cy, radius - 2, radius - 2, paint.shadow);
    }
    for spoke in 0..spokes {
        let a = angle + spoke as f32 * std::f32::consts::TAU / spokes as f32;
        let reach = (radius - 1) as f32;
        let tip = (
            cx + (a.cos() * reach).round() as i32,
            cy + (a.sin() * reach).round() as i32,
        );
        line(scene, (cx, cy), tip, paint.light);
    }
    // The steel tyre catches the light along its top.
    for dx in -radius / 2..=radius / 2 {
        put(scene, cx + dx, cy - radius, STEEL.light);
    }
    put(scene, cx, cy, BRASS.light);
}

/// The coupling rod joining the driving wheels, and the connecting rod from the cylinder.
fn rods(scene: &mut Canvas, x: i32, rolled: f32) {
    let radius = LOCO_WHEELS[0].radius;
    let angle = rolled / radius as f32;
    let crank = (radius - 3) as f32;
    let pin = |wheel: Wheel| {
        (
            x + wheel.x + (angle.cos() * crank).round() as i32,
            TRAIN_TOP + wheel.y + (angle.sin() * crank).round() as i32,
        )
    };
    let (first, middle, last) = (
        pin(LOCO_WHEELS[0]),
        pin(LOCO_WHEELS[1]),
        pin(LOCO_WHEELS[2]),
    );
    line(
        scene,
        (first.0, first.1 + 1),
        (last.0, last.1 + 1),
        STEEL.shadow,
    );
    line(scene, first, last, STEEL.light);
    let crosshead = (x + LOCO_X + 55, TRAIN_TOP + BUFFERS + 4);
    line(scene, crosshead, middle, STEEL.base);
    for point in [first, middle, last] {
        put(scene, point.0, point.1, BRASS.shine);
    }
}

fn coupling(body: &mut Canvas, x: i32) {
    rect(body, x - 1, BUFFERS + 2, COUPLING + 2, 2, SOOT.base);
    put(body, x + 1, BUFFERS + 2, SOOT.light);
    for side in [x - 1, x + COUPLING - 1] {
        bevel(body, side, BUFFERS, 2, 5, SOOT);
    }
}

fn coach(body: &mut Canvas, x: i32) {
    let length = COACH_LENGTH;
    // A curved roof with vents along it.
    hline(body, x + 3, ROOF_TOP, length - 6, ROOF.edge);
    hline(body, x + 1, ROOF_TOP + 1, length - 2, ROOF.shine);
    hline(body, x, ROOF_TOP + 2, length, ROOF.light);
    for row in 3..5 {
        hline(body, x, ROOF_TOP + row, length, ROOF.base);
    }
    hline(body, x, ROOF_TOP + 5, length, ROOF.shadow);
    for vent in [x + 15, x + 33, x + 51] {
        bevel(body, vent - 2, ROOF_TOP - 2, 5, 3, ROOF);
    }

    // Cream above the waist, maroon below, lined in gold.
    rect(body, x, BODY_TOP, length, BELT - BODY_TOP, CREAM.base);
    hline(body, x, BODY_TOP, length, CREAM.shadow);
    hline(body, x, BODY_TOP + 1, length, CREAM.light);
    rect(body, x, BELT, length, BODY_BOTTOM - BELT, MAROON.base);
    hline(body, x, BELT, length, MAROON.edge);
    hline(body, x, BELT + 1, length, MAROON.light);
    hline(body, x + 2, BELT + 3, length - 4, LINING);
    hline(body, x + 2, BODY_BOTTOM - 3, length - 4, LINING);
    hline(body, x, BODY_BOTTOM - 1, length, MAROON.shadow);
    for panel in [x + 23, x + 41] {
        vline(body, panel, BELT + 5, BODY_BOTTOM - BELT - 9, MAROON.shadow);
        vline(
            body,
            panel + 1,
            BELT + 5,
            BODY_BOTTOM - BELT - 9,
            MAROON.light,
        );
    }
    vline(body, x, BODY_TOP, BODY_BOTTOM - BODY_TOP, CREAM.edge);
    vline(
        body,
        x + length - 1,
        BODY_TOP,
        BODY_BOTTOM - BODY_TOP,
        MAROON.edge,
    );

    // Doors at both ends, each with a droplight and a brass handle.
    for door in [x + 1, x + length - 7] {
        vline(
            body,
            door,
            BODY_TOP + 1,
            BODY_BOTTOM - BODY_TOP - 2,
            MAROON.edge,
        );
        vline(
            body,
            door + 6,
            BODY_TOP + 1,
            BODY_BOTTOM - BODY_TOP - 2,
            MAROON.edge,
        );
        rect(body, door + 2, WINDOW_TOP, 3, 9, INTERIOR);
        put(body, door + 2, WINDOW_TOP, rgba(0xa6c8d2, 200));
        put(body, door + 4, BELT + 6, BRASS.light);
        put(body, door + 4, BELT + 5, BRASS.shine);
    }

    for column in WINDOW_XS {
        let (wx, wy, (width, height)) = (x + column, WINDOW_TOP, WINDOW_SIZE);
        // A frame, lit along its top, around a softly lit compartment.
        rect(body, wx - 1, wy - 1, width + 2, height + 2, CREAM.edge);
        for row in 0..height {
            let t = row as f32 / height as f32;
            hline(body, wx, wy + row, width, mix(INTERIOR, rgb(0x5b4c4a), t));
        }
        hline(body, wx - 1, wy - 2, width + 2, CREAM.shine);
        hline(body, wx - 1, wy + height + 1, width + 2, CREAM.shadow);
    }

    // Underframe, truss rods and the two bogies.
    rect(body, x + 2, BODY_BOTTOM, length - 4, 3, SOOT.base);
    hline(body, x + 2, BODY_BOTTOM, length - 4, SOOT.light);
    line(
        body,
        (x + 22, BODY_BOTTOM + 2),
        (x + 28, BODY_BOTTOM + 5),
        SOOT.light,
    );
    line(
        body,
        (x + 28, BODY_BOTTOM + 5),
        (x + 38, BODY_BOTTOM + 5),
        SOOT.light,
    );
    line(
        body,
        (x + 38, BODY_BOTTOM + 5),
        (x + 44, BODY_BOTTOM + 2),
        SOOT.light,
    );
    for bogie in [x + 12, x + 54] {
        rect(body, bogie - 7, AXLE - 3, 14, 3, SOOT.shadow);
        hline(body, bogie - 7, AXLE - 3, 14, SOOT.light);
        bevel(body, bogie - 2, AXLE - 3, 4, 4, SOOT);
    }
    // Buffers at each end.
    for buffer in [x - 1, x + length - 1] {
        bevel(body, buffer, BUFFERS, 2, 4, STEEL);
    }
}

fn locomotive(body: &mut Canvas, x: i32) {
    let plate = BODY_BOTTOM - 4;
    // The cab, at the back, with a curved roof and a round-cornered window.
    rect(body, x - 1, 4, 24, 2, SOOT.base);
    hline(body, x, 3, 22, SOOT.light);
    hline(body, x - 1, 6, 24, SOOT.edge);
    panel(body, x, 7, 21, plate - 7);
    rect(body, x + 5, 12, 11, 11, CREAM.edge);
    rect(body, x + 6, 13, 9, 9, INTERIOR);
    put(body, x + 6, 13, CREAM.edge);
    put(body, x + 14, 13, CREAM.edge);
    rect(body, x + 6, 13, 9, 3, rgba(0xa6c8d2, 110));
    put(body, x + 13, 14, rgba(0xffffff, 200));
    put(body, x + 12, 15, rgba(0xffffff, 140));
    // A brass number plate on the cab side.
    bevel(body, x + 5, 31, 11, 6, BRASS);
    hline(body, x + 7, 33, 7, BRASS.shadow);

    // The boiler: a cylinder, lit along its top, banded in brass.
    let (boiler_left, boiler_right, boiler_top, boiler_bottom) = (x + 21, x + 52, 16, 42);
    for y in boiler_top..boiler_bottom {
        let t = (y - boiler_top) as f32 / (boiler_bottom - boiler_top) as f32;
        let color = match t {
            t if t < 0.06 => LOCO.edge,
            t if t < 0.14 => LOCO.shine,
            t if t < 0.34 => LOCO.light,
            t if t < 0.72 => LOCO.base,
            t if t < 0.93 => LOCO.shadow,
            _ => LOCO.edge,
        };
        hline(body, boiler_left, y, boiler_right - boiler_left, color);
    }
    for band in [x + 29, x + 42] {
        vline(
            body,
            band,
            boiler_top + 1,
            boiler_bottom - boiler_top - 2,
            BRASS.base,
        );
        put(body, band, boiler_top + 2, BRASS.shine);
    }
    hline(
        body,
        boiler_left + 1,
        boiler_top + 6,
        boiler_right - boiler_left - 2,
        BRASS.shadow,
    );

    // Side tanks over the lower half of the boiler, lined out in cream.
    panel(body, x + 21, 27, 27, plate - 27);
    for y in [30, plate - 4] {
        hline(body, x + 24, y, 21, CREAM.base);
    }
    for column in [x + 24, x + 44] {
        vline(body, column, 30, plate - 33, CREAM.base);
    }
    // The colony's crest: a hill under a sun, in a ring.
    let (crest_x, crest_y) = (x + 34, 36);
    ellipse(body, crest_x, crest_y, 4, 4, CREAM.base);
    ellipse(body, crest_x, crest_y, 3, 3, LOCO.shadow);
    ellipse(body, crest_x, crest_y + 3, 3, 2, LOCO.light);
    put(body, crest_x + 1, crest_y - 2, BRASS.shine);
    put(body, crest_x + 1, crest_y - 1, BRASS.light);

    // Dome and safety valve.
    ellipse(body, x + 36, 15, 4, 4, BRASS.edge);
    ellipse(body, x + 36, 15, 3, 3, BRASS.base);
    rect(body, x + 32, 15, 9, 2, BRASS.base);
    put(body, x + 35, 13, BRASS.shine);
    put(body, x + 34, 14, BRASS.light);
    bevel(body, x + 24, 11, 3, 6, BRASS);

    // Smokebox and chimney, in black.
    let (smoke_top, smoke_bottom) = (15, 45);
    for y in smoke_top..smoke_bottom {
        let t = (y - smoke_top) as f32 / (smoke_bottom - smoke_top) as f32;
        let color = if t < 0.08 {
            SOOT.light
        } else if t < 0.4 {
            SOOT.base
        } else if t < 0.9 {
            SOOT.shadow
        } else {
            SOOT.edge
        };
        hline(body, x + 50, y, 8, color);
    }
    vline(body, x + 57, smoke_top, smoke_bottom - smoke_top, SOOT.edge);
    for rivet in (smoke_top + 3..smoke_bottom - 2).step_by(4) {
        put(body, x + 51, rivet, SOOT.light);
    }
    rect(body, x + 51, 4, 6, 11, SOOT.base);
    vline(body, x + 51, 4, 11, SOOT.light);
    vline(body, x + 56, 4, 11, SOOT.edge);
    rect(body, x + 50, 1, 8, 3, SOOT.base);
    hline(body, x + 50, 1, 8, SOOT.light);
    put(body, x + 51, 2, BRASS.light);
    // A lamp on the front.
    bevel(body, x + 56, BUFFERS - 9, 4, 5, CREAM);
    put(body, x + 57, BUFFERS - 8, rgb(0xfff0bd));

    // Running plate, buffer beam, buffers and cylinders.
    rect(body, x - 1, plate, 61, 2, SOOT.base);
    hline(body, x - 1, plate, 61, SOOT.light);
    hline(body, x, plate + 2, 58, BUFFER_RED.base);
    bevel(body, x + 57, BUFFERS - 2, 4, 9, BUFFER_RED);
    bevel(body, x + 59, BUFFERS, 3, 4, STEEL);
    bevel(body, x - 2, BUFFERS, 3, 4, STEEL);
    bevel(body, x + 49, plate + 1, 10, 7, SOOT);
    hline(body, x + 50, plate + 2, 8, SOOT.light);
    put(body, x + 57, plate + 4, STEEL.light);
}

/// A painted panel of the engine's green, lit on its top and left edges.
fn panel(body: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    rect(body, x, y, width, height, LOCO.base);
    hline(body, x, y, width, LOCO.light);
    vline(body, x, y, height, LOCO.light);
    hline(body, x, y + height - 1, width, LOCO.shadow);
    vline(body, x + width - 1, y, height, LOCO.edge);
    put(body, x + 1, y + 1, LOCO.shine);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_wheel_meets_the_rail() {
        let coach_bottom = TRAIN_TOP + AXLE + 3;
        let loco_bottoms: Vec<_> = LOCO_WHEELS
            .iter()
            .map(|wheel| TRAIN_TOP + wheel.y + wheel.radius)
            .collect();
        for bottom in loco_bottoms.into_iter().chain([coach_bottom]) {
            assert!(
                (197..=199).contains(&bottom),
                "a wheel bottoms out at {bottom}"
            );
        }
    }

    #[test]
    fn the_train_cuts_anyone_behind_it_at_the_middle_not_the_feet() {
        // Standing travellers are about 34 pixels tall; the coach roof should cross them well up
        // their bodies, so nobody looks as though they are standing on it.
        let roof = TRAIN_TOP + ROOF_TOP;
        assert!(super::super::STAND_Y - roof >= 12, "the roof is at {roof}");
    }

    #[test]
    fn every_window_is_on_the_train() {
        for window in 0..WINDOWS {
            let (x, y) = window_origin(window);
            assert!(x >= 0 && x + WINDOW_SIZE.0 <= TRAIN_LENGTH);
            assert!(y >= 0 && y + WINDOW_SIZE.1 <= TRAIN_HEIGHT);
        }
    }

    #[test]
    fn turning_wheels_change_the_picture() {
        let train = Train::new();
        let draw = |rolled| {
            let mut scene = Canvas::new(400, 216);
            train.draw(&mut scene, 10, rolled, &[]);
            scene
        };
        assert_ne!(draw(0.0), draw(2.0));
    }
}
