//! The station: where the train comes in, and the first thing the colony sees of the Hill.
//!
//! The scene is composed into one small pixel canvas at Formiga's own scale, and the window
//! scales it up by whole pixels. Every traveller is drawn by `formiga-art` from the appearance
//! the snapshot carried, wearing what it wore on Desktop, so it is the same individual rather
//! than a lookalike. The scenery is placeholder art standing in for the real station.

use formiga_art::{AnimationSpec, Canvas, CreatureRenderer, FRAME_SIZE, Rgba};
use formiga_core::ActionKind;
use formiga_travel::{TravelSnapshot, Traveler};

pub const SCENE_WIDTH: u32 = 384;
pub const SCENE_HEIGHT: u32 = 216;
/// The back edge of the platform boards.
pub const PLATFORM_Y: i32 = 158;
/// Where feet meet the boards: a little in from the back edge, so everyone stands on them.
pub const STAND_Y: i32 = PLATFORM_Y + 5;
/// Where the station sign hangs, for the window to letter.
pub const SIGN_CENTER: (i32, i32) = (192, 34);
const STAND_LEFT: i32 = 92;
const STAND_RIGHT: i32 = 330;

pub struct Station {
    backdrop: Canvas,
    travelers: Vec<StationTraveler>,
}

pub struct StationTraveler {
    pub id: formiga_core::CreatureId,
    pub name: String,
    /// The top-left corner of its frame in the scene.
    pub origin: (i32, i32),
    /// The opaque part of its resting frame, in scene coordinates, inclusive.
    pub bounds: (i32, i32, i32, i32),
    frames: Vec<Canvas>,
    spec: AnimationSpec,
    /// So a platform of companions does not breathe in step.
    phase: f32,
}

impl StationTraveler {
    /// The frame it is showing `elapsed` seconds into the visit.
    pub fn frame_at(&self, elapsed: f32) -> usize {
        if self.frames.len() == 1 {
            return 0;
        }
        usize::from(self.spec.frame_at(elapsed + self.phase)) % self.frames.len()
    }
}

impl Station {
    pub fn new(snapshot: &TravelSnapshot) -> Self {
        let reduce_motion = snapshot.presentation.reduce_motion;
        let count = snapshot.travelers.len() as i32;
        let travelers = snapshot
            .travelers
            .iter()
            .enumerate()
            .map(|(index, traveler)| {
                let index = index as i32;
                let center_x = if count == 1 {
                    (STAND_LEFT + STAND_RIGHT) / 2
                } else {
                    STAND_LEFT + (STAND_RIGHT - STAND_LEFT) * index / (count - 1)
                };
                // Gathered round: the left half looks right and the right half looks left.
                let facing_right = index * 2 < count;
                StationTraveler::new(traveler, center_x, facing_right, reduce_motion)
            })
            .collect();
        Self {
            backdrop: backdrop(),
            travelers,
        }
    }

    pub fn travelers(&self) -> &[StationTraveler] {
        &self.travelers
    }

    /// Which frame each traveller is on: the scene only needs composing again when this changes.
    pub fn frame_key(&self, elapsed: f32) -> Vec<usize> {
        self.travelers
            .iter()
            .map(|traveler| traveler.frame_at(elapsed))
            .collect()
    }

    pub fn compose(&self, elapsed: f32) -> Canvas {
        let mut scene = self.backdrop.clone();
        for traveler in &self.travelers {
            let (x, y) = traveler.origin;
            let foot_y = traveler.bounds.3;
            let (left, right) = (traveler.bounds.0, traveler.bounds.2);
            shadow(&mut scene, (left + right) / 2, foot_y, (right - left) / 2);
            blit(
                &mut scene,
                &traveler.frames[traveler.frame_at(elapsed)],
                x,
                y,
            );
        }
        scene
    }

    /// The traveller drawn at a point in scene coordinates, front-most first.
    pub fn traveler_at(&self, x: f32, y: f32) -> Option<&StationTraveler> {
        self.travelers.iter().rev().find(|traveler| {
            let (left, top, right, bottom) = traveler.bounds;
            x >= left as f32
                && x <= right as f32 + 1.0
                && y >= top as f32
                && y <= bottom as f32 + 1.0
        })
    }
}

impl StationTraveler {
    fn new(traveler: &Traveler, center_x: i32, facing_right: bool, reduce_motion: bool) -> Self {
        let spec = AnimationSpec::for_action(ActionKind::Idle);
        let dress = traveler.accessory.map(|worn| worn.art());
        let frame_count = if reduce_motion { 1 } else { spec.frames };
        let frames: Vec<Canvas> = (0..frame_count)
            .map(|frame| {
                CreatureRenderer::render_dressed_frame(
                    &traveler.appearance,
                    dress,
                    ActionKind::Idle,
                    frame,
                    facing_right,
                )
            })
            .collect();

        // Stand it on the boards by the lowest opaque row of its resting frame.
        let size = FRAME_SIZE as i32;
        let (min_x, min_y, max_x, max_y) = frames[0]
            .alpha_bounds()
            .map(|(a, b, c, d)| (a as i32, b as i32, c as i32, d as i32))
            .unwrap_or((0, 0, size - 1, size - 1));
        let origin = (center_x - size / 2, STAND_Y - max_y);
        Self {
            id: traveler.id,
            name: traveler.name.clone(),
            origin,
            bounds: (
                origin.0 + min_x,
                origin.1 + min_y,
                origin.0 + max_x,
                origin.1 + max_y,
            ),
            frames,
            spec,
            phase: (traveler.id % 997) as f32 / 997.0 * 2.0,
        }
    }
}

/// Source-over, pixel by pixel. Creature frames are hard-edged pixel art, so this only ever
/// blends the odd soft shadow or highlight.
fn blit(scene: &mut Canvas, sprite: &Canvas, x: i32, y: i32) {
    for sy in 0..sprite.height() as i32 {
        for sx in 0..sprite.width() as i32 {
            let source = sprite.get(sx, sy);
            if source.a == 0 {
                continue;
            }
            let (dx, dy) = (x + sx, y + sy);
            scene.set(dx, dy, over(source, scene.get(dx, dy)));
        }
    }
}

fn over(top: Rgba, bottom: Rgba) -> Rgba {
    if top.a == 255 {
        return top;
    }
    let alpha = u32::from(top.a);
    let mix = |a: u8, b: u8| ((u32::from(a) * alpha + u32::from(b) * (255 - alpha)) / 255) as u8;
    Rgba::new(
        mix(top.r, bottom.r),
        mix(top.g, bottom.g),
        mix(top.b, bottom.b),
        top.a.max(bottom.a),
    )
}

fn shadow(scene: &mut Canvas, center_x: i32, foot_y: i32, half_width: i32) {
    let shade = Rgba::new(0x5b, 0x40, 0x2c, 70);
    let radius = (half_width - 2).max(4);
    for y in -1..=1 {
        for x in -radius..=radius {
            if x * x * 4 + y * y * radius * radius <= radius * radius * 4 {
                let (px, py) = (center_x + x, foot_y + 1 + y);
                scene.set(px, py, over(shade, scene.get(px, py)));
            }
        }
    }
}

const SKY_TOP: Rgba = Rgba::new(0xb9, 0xdd, 0xec, 255);
const SKY_LOW: Rgba = Rgba::new(0xf6, 0xe8, 0xcf, 255);
const CLOUD: Rgba = Rgba::new(0xfd, 0xfb, 0xf5, 255);
const FAR_HILL: Rgba = Rgba::new(0xa9, 0xcf, 0xa4, 255);
const HILL: Rgba = Rgba::new(0x86, 0xbb, 0x7c, 255);
const HILL_SHADE: Rgba = Rgba::new(0x72, 0xa5, 0x69, 255);
const PATH: Rgba = Rgba::new(0xe6, 0xd3, 0xa4, 255);
const GRASS: Rgba = Rgba::new(0x7d, 0xb3, 0x6c, 255);
const GRASS_DARK: Rgba = Rgba::new(0x63, 0x98, 0x58, 255);
const TRUNK: Rgba = Rgba::new(0x7a, 0x5a, 0x3a, 255);
const LEAVES: Rgba = Rgba::new(0x5d, 0x9a, 0x5a, 255);
const BOARDS: Rgba = Rgba::new(0xd0, 0xa8, 0x72, 255);
const BOARD_LINE: Rgba = Rgba::new(0xae, 0x86, 0x55, 255);
const PLATFORM_EDGE: Rgba = Rgba::new(0x8a, 0x66, 0x42, 255);
const STONE: Rgba = Rgba::new(0xb7, 0xae, 0xa0, 255);
const STONE_DARK: Rgba = Rgba::new(0x96, 0x8d, 0x80, 255);
const SLEEPER: Rgba = Rgba::new(0x6e, 0x55, 0x40, 255);
const RAIL: Rgba = Rgba::new(0x8c, 0x8f, 0x96, 255);
const RAIL_LIGHT: Rgba = Rgba::new(0xc9, 0xcc, 0xd2, 255);
const WALL: Rgba = Rgba::new(0xf1, 0xe4, 0xc6, 255);
const ROOF: Rgba = Rgba::new(0xc7, 0x5d, 0x4c, 255);
const ROOF_DARK: Rgba = Rgba::new(0xa2, 0x47, 0x3a, 255);
const DOOR: Rgba = Rgba::new(0x8f, 0x6a, 0x45, 255);
const WINDOW: Rgba = Rgba::new(0xf7, 0xdd, 0x86, 255);
const SIGN_BOARD: Rgba = Rgba::new(0x5d, 0x46, 0x32, 255);
const SIGN_FACE: Rgba = Rgba::new(0xf2, 0xe6, 0xc8, 255);
const POST: Rgba = Rgba::new(0x4f, 0x47, 0x45, 255);
const LAMP: Rgba = Rgba::new(0xff, 0xe7, 0x9a, 255);

/// The fixed part of the scene, painted once.
fn backdrop() -> Canvas {
    let (width, height) = (SCENE_WIDTH as i32, SCENE_HEIGHT as i32);
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);

    // Sky, in bands rather than a smooth ramp, as pixel skies are.
    let horizon = 132;
    for y in 0..horizon {
        let t = (y * 6 / horizon) as f32 / 5.0;
        let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
        let band = Rgba::new(
            mix(SKY_TOP.r, SKY_LOW.r),
            mix(SKY_TOP.g, SKY_LOW.g),
            mix(SKY_TOP.b, SKY_LOW.b),
            255,
        );
        scene.fill_rect(0, y, width, 1, band);
    }
    for (x, y, size) in [(58, 30, 9), (300, 22, 11), (236, 56, 7)] {
        scene.fill_ellipse(x, y, size * 2, size / 2 + 2, CLOUD);
        scene.fill_ellipse(x - size / 2, y - 3, size, size / 2 + 1, CLOUD);
        scene.fill_ellipse(x + size / 2, y - 4, size, size / 2 + 2, CLOUD);
    }

    // Far hills, then the Hill itself, with its path winding up to a tree at the top.
    scene.fill_ellipse(70, 150, 120, 42, FAR_HILL);
    scene.fill_ellipse(200, 156, 140, 34, FAR_HILL);
    scene.fill_ellipse(312, 160, 112, 92, HILL);
    scene.fill_ellipse(340, 168, 70, 70, HILL_SHADE);
    let path = [
        (250, 134),
        (292, 122),
        (276, 106),
        (314, 92),
        (300, 80),
        (306, 72),
    ];
    for pair in path.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        scene.line(x0, y0, x1, y1, 2, PATH);
    }
    scene.fill_rect(306, 62, 3, 10, TRUNK);
    scene.fill_circle(307, 58, 8, LEAVES);
    scene.fill_circle(301, 61, 5, LEAVES);
    scene.fill_circle(313, 61, 5, LEAVES);

    // Grass in front of everything.
    scene.fill_rect(0, horizon, width, height - horizon, GRASS);
    for x in (0..width).step_by(7) {
        scene.set(x, horizon + 3 + (x % 5), GRASS_DARK);
        scene.set(x + 3, horizon + 9 + (x % 3), GRASS_DARK);
    }

    // The station house, to the left, behind the platform.
    scene.fill_rect(20, 110, 62, 48, WALL);
    scene.fill_rect(42, 128, 16, 30, DOOR);
    scene.fill_rect(26, 122, 11, 10, WINDOW);
    scene.fill_rect(64, 122, 11, 10, WINDOW);
    for row in 0..14 {
        let inset = 14 - row;
        let color = if row % 4 == 3 { ROOF_DARK } else { ROOF };
        scene.fill_rect(12 + inset, 96 + row, 78 - inset * 2, 1, color);
    }

    // The sign, on two posts over the platform.
    let (sign_x, sign_y) = SIGN_CENTER;
    scene.fill_rect(sign_x - 46, sign_y + 8, 3, PLATFORM_Y - sign_y - 8, POST);
    scene.fill_rect(sign_x + 43, sign_y + 8, 3, PLATFORM_Y - sign_y - 8, POST);
    scene.fill_rect(sign_x - 54, sign_y - 11, 108, 22, SIGN_BOARD);
    scene.fill_rect(sign_x - 52, sign_y - 9, 104, 18, SIGN_FACE);

    // A lamp at the far end.
    scene.fill_rect(352, 112, 3, PLATFORM_Y - 112, POST);
    scene.fill_rect(348, 106, 11, 8, POST);
    scene.fill_rect(350, 108, 7, 5, LAMP);

    // The platform: boards on a stone edge.
    scene.fill_rect(8, PLATFORM_Y, width - 16, 10, BOARDS);
    for x in (14..width - 8).step_by(18) {
        scene.fill_rect(x, PLATFORM_Y + 1, 1, 9, BOARD_LINE);
    }
    scene.fill_rect(8, PLATFORM_Y + 10, width - 16, 2, PLATFORM_EDGE);
    scene.fill_rect(8, PLATFORM_Y + 12, width - 16, 10, STONE);
    for x in (8..width - 8).step_by(16) {
        scene.fill_rect(x, PLATFORM_Y + 12, 1, 10, STONE_DARK);
    }
    scene.fill_rect(8, PLATFORM_Y + 21, width - 16, 1, STONE_DARK);

    // The line the train comes in on.
    let track = PLATFORM_Y + 32;
    for x in (0..width).step_by(10) {
        scene.fill_rect(x, track - 2, 6, 9, SLEEPER);
    }
    scene.fill_rect(0, track, width, 2, RAIL);
    scene.fill_rect(0, track, width, 1, RAIL_LIGHT);
    scene.fill_rect(0, track + 5, width, 2, RAIL);
    scene.fill_rect(0, track + 5, width, 1, RAIL_LIGHT);

    scene
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_traveller_is_drawn_standing_on_the_platform() {
        let snapshot = formiga_travel::sample::snapshot();
        let station = Station::new(&snapshot);
        assert_eq!(station.travelers().len(), snapshot.travelers.len());
        for traveler in station.travelers() {
            assert_eq!(
                traveler.bounds.3, STAND_Y,
                "{} is not on the boards",
                traveler.name
            );
            assert!(traveler.bounds.0 >= 0 && traveler.bounds.2 < SCENE_WIDTH as i32);
        }
    }

    #[test]
    fn travellers_do_not_overlap_on_a_full_platform() {
        let mut snapshot = formiga_travel::sample::snapshot();
        while snapshot.travelers.len() < formiga_travel::MAX_TRAVELERS {
            let mut extra = snapshot.travelers[0].clone();
            extra.id += snapshot.travelers.len() as u64;
            snapshot.travelers.push(extra);
        }
        let station = Station::new(&snapshot);
        for pair in station.travelers().windows(2) {
            let gap = pair[1].origin.0 - pair[0].origin.0;
            assert!(gap >= 40, "only {gap} pixels between neighbours");
        }
    }

    #[test]
    fn composing_draws_the_travellers_over_the_scenery() {
        let station = Station::new(&formiga_travel::sample::snapshot());
        let empty = backdrop();
        let scene = station.compose(0.0);
        assert_eq!((scene.width(), scene.height()), (SCENE_WIDTH, SCENE_HEIGHT));
        for traveler in station.travelers() {
            let (left, top, right, bottom) = traveler.bounds;
            let changed = (top..=bottom)
                .flat_map(|y| (left..=right).map(move |x| (x, y)))
                .filter(|&(x, y)| scene.get(x, y) != empty.get(x, y))
                .count();
            assert!(changed > 100, "{} barely shows", traveler.name);
        }
    }

    #[test]
    fn reduced_motion_holds_one_resting_frame() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let station = Station::new(&snapshot);
        assert_eq!(station.frame_key(0.0), station.frame_key(7.3));
    }

    #[test]
    fn hit_testing_finds_whoever_is_under_the_pointer() {
        let station = Station::new(&formiga_travel::sample::snapshot());
        let traveler = &station.travelers()[1];
        let (left, top, right, bottom) = traveler.bounds;
        let middle = ((left + right) as f32 / 2.0, (top + bottom) as f32 / 2.0);
        assert_eq!(
            station.traveler_at(middle.0, middle.1).map(|t| t.id),
            Some(traveler.id)
        );
        assert!(station.traveler_at(2.0, 2.0).is_none());
    }
}
