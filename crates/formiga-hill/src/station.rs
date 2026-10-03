//! The station: where the train comes in, and the first thing the colony sees of the Hill.
//!
//! The scene is composed into one small pixel canvas at Formiga's own scale, and the window
//! scales it up by whole pixels. Every traveller is drawn by `formiga-art` from the appearance
//! the snapshot carried, wearing what it wore on Desktop, so it is the same individual rather
//! than a lookalike. The scenery around them is in `scenery`, the train in `train`, and the
//! timeline of its comings and goings in `journey`.

mod journey;
mod scenery;
mod train;

use crate::cast::{Cast, Id, Member};
use crate::daylight::{Daylight, Nightlights};
use crate::hilltop::Arrangement;
use crate::paint::{blit, ellipse, rgba};
use formiga_art::{AnimationSpec, Canvas, CreatureRenderer, FRAME_SIZE};
use formiga_core::ActionKind;
use journey::Place;
use train::{Passenger, Train};

pub use journey::Journey;
pub use scenery::STAND_Y;

pub const SCENE_WIDTH: u32 = 384;
pub const SCENE_HEIGHT: u32 = 216;
const STAND_LEFT: i32 = 92;
const STAND_RIGHT: i32 = 330;

pub struct Station {
    backdrop: Canvas,
    /// What the backdrop shows: the display case's souvenirs, and the Hilltop on the skyline.
    keepsakes: Vec<String>,
    hilltop: Arrangement,
    train: Train,
    travelers: Vec<StationTraveler>,
    reduce_motion: bool,
    journey: Journey,
    /// What shines after dark, and the hour's light.
    nightlights: Nightlights,
    daylight: Daylight,
}

/// How often the scene is drawn while the train is in motion.
const JOURNEY_FRAMES_PER_SECOND: f32 = 30.0;

pub struct StationTraveler {
    pub id: Id,
    pub name: String,
    /// The top-left corner of its frame in the scene.
    pub origin: (i32, i32),
    /// The opaque part of its resting frame, in scene coordinates, inclusive.
    pub bounds: (i32, i32, i32, i32),
    frames: Vec<Canvas>,
    /// The centre of its face in its frames, for framing it in a carriage window.
    face: (i32, i32),
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
    pub fn new(cast: &Cast, journey: Journey, keepsakes: &[String]) -> Self {
        let reduce_motion = cast.reduce_motion();
        let count = cast.members.len() as i32;
        let mut travelers: Vec<StationTraveler> = cast
            .members
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
                // A crowd larger than a colony of today stands in two staggered rows.
                let stand_y = if count > 6 && index % 2 == 1 {
                    STAND_Y - 6
                } else {
                    STAND_Y
                };
                StationTraveler::new(traveler, center_x, stand_y, facing_right, reduce_motion)
            })
            .collect();
        // Those further back are drawn first.
        travelers.sort_by_key(|traveler| traveler.bounds.3);
        let backdrop = scenery::backdrop(keepsakes, &Arrangement::new());
        Self {
            nightlights: nightlights(&backdrop),
            daylight: Daylight::default(),
            backdrop,
            keepsakes: keepsakes.to_vec(),
            hilltop: Arrangement::new(),
            train: Train::new(),
            travelers,
            reduce_motion,
            journey,
        }
    }

    /// Puts the colony's kept souvenirs in the display case.
    pub fn show_keepsakes(&mut self, keepsakes: &[String]) {
        keepsakes.clone_into(&mut self.keepsakes);
        self.repaint();
    }

    /// Shows what stands on the Hilltop, up on the skyline.
    pub fn show_hilltop(&mut self, hilltop: &Arrangement) {
        hilltop.clone_into(&mut self.hilltop);
        self.repaint();
    }

    fn repaint(&mut self) {
        self.backdrop = scenery::backdrop(&self.keepsakes, &self.hilltop);
        self.nightlights = nightlights(&self.backdrop);
    }

    /// The hour's light over the station.
    pub fn set_daylight(&mut self, daylight: Daylight) {
        self.daylight = daylight;
    }

    /// Whether a point in the scene is on the display case.
    pub fn on_display_case(&self, x: f32, y: f32) -> bool {
        let (left, top, right, bottom) = scenery::CASE;
        (left as f32..right as f32).contains(&x) && (top as f32..bottom as f32).contains(&y)
    }

    pub fn travelers(&self) -> &[StationTraveler] {
        &self.travelers
    }

    pub fn journey(&self) -> Journey {
        self.journey
    }

    /// Everyone is on the platform and the line is clear.
    pub fn is_settled(&self) -> bool {
        self.journey == Journey::Here
    }

    /// Settles an arrival that has run its course.
    pub fn update(&mut self, now: f32) {
        if matches!(self.journey, Journey::Arriving { .. })
            && self
                .journey
                .finished(now, self.travelers.len(), self.reduce_motion)
        {
            self.journey = Journey::Here;
        }
    }

    /// Cuts an arrival short: everyone is simply on the platform.
    pub fn skip_arrival(&mut self) {
        if matches!(self.journey, Journey::Arriving { .. }) {
            self.journey = Journey::Here;
        }
    }

    /// Calls the train to take everyone home, unless it is already on its way.
    pub fn set_off_home(&mut self, now: f32) {
        if !matches!(self.journey, Journey::Leaving { .. }) {
            self.journey = Journey::Leaving { since: now };
        }
    }

    /// The train has left with everyone aboard.
    pub fn gone_home(&self, now: f32) -> bool {
        matches!(self.journey, Journey::Leaving { .. })
            && self
                .journey
                .finished(now, self.travelers.len(), self.reduce_motion)
    }

    /// Whether the train is coming or going, which wants a faster redraw.
    pub fn in_motion(&self, now: f32) -> bool {
        !self.is_settled() && !self.gone_home(now)
    }

    /// What is showing at `now`: the scene only needs composing again when this changes.
    pub fn frame_key(&self, now: f32) -> Vec<usize> {
        let ticks = |per_second: f32| {
            if self.reduce_motion {
                0
            } else {
                (now * per_second) as usize
            }
        };
        let journey = if self.is_settled() {
            0
        } else {
            1 + ticks(JOURNEY_FRAMES_PER_SECOND)
        };
        self.travelers
            .iter()
            .map(|traveler| traveler.frame_at(now))
            .chain([ticks(scenery::SMOKE_STEPS_PER_SECOND), journey])
            .collect()
    }

    pub fn compose(&self, now: f32) -> Canvas {
        let mut scene = self.backdrop.clone();
        let smoke_time =
            (now * scenery::SMOKE_STEPS_PER_SECOND).floor() / scenery::SMOKE_STEPS_PER_SECOND;
        scenery::smoke(&mut scene, smoke_time, self.reduce_motion);

        let count = self.travelers.len();
        let stage = self.journey.stage(now, count, self.reduce_motion);
        let mut passengers = Vec::new();
        for (traveler, place) in self.travelers.iter().zip(&stage.places) {
            let frame = &traveler.frames[traveler.frame_at(now)];
            match *place {
                Place::Platform { lift } => {
                    let (left, right) = (traveler.bounds.0, traveler.bounds.2);
                    let half_width = (right - left) / 2 - (lift / 3.0) as i32;
                    shadow(
                        &mut scene,
                        (left + right) / 2,
                        traveler.bounds.3,
                        half_width,
                    );
                    let (x, y) = traveler.origin;
                    blit(&mut scene, frame, x, y - lift.round() as i32);
                }
                Place::Window(window) => passengers.push(Passenger {
                    window,
                    frame,
                    face: traveler.face,
                }),
                Place::Away | Place::Unseen => {}
            }
        }

        if let Some(train_x) = stage.train_x {
            let x = train_x.round() as i32;
            self.train.draw(&mut scene, x, stage.rolled, &passengers);
            // Steam trails behind as fast as the train is going.
            let ahead = self.journey.stage(now + 0.1, count, self.reduce_motion);
            let speed = ahead.train_x.map_or(0.0, |ahead| (ahead - train_x) * 10.0);
            let (chimney_x, chimney_y) = self.train.chimney();
            train::steam(
                &mut scene,
                (x + chimney_x, chimney_y),
                now,
                speed,
                self.reduce_motion,
            );
        }
        self.nightlights
            .light(&mut scene, &self.backdrop, self.daylight);
        scene
    }

    /// The traveller drawn at a point in scene coordinates, front-most first. Only once everyone
    /// is settled on the platform.
    pub fn traveler_at(&self, x: f32, y: f32) -> Option<&StationTraveler> {
        if !self.is_settled() {
            return None;
        }
        self.travelers.iter().rev().find(|traveler| {
            let (left, top, right, bottom) = traveler.bounds;
            x >= left as f32
                && x <= right as f32 + 1.0
                && y >= top as f32
                && y <= bottom as f32 + 1.0
        })
    }
}

/// What shines at the station after dark, and its night sky.
fn nightlights(backdrop: &Canvas) -> Nightlights {
    Nightlights {
        lamps: scenery::lamplight(),
        sky: scenery::night_sky(backdrop),
        indoors: false,
    }
}

impl StationTraveler {
    fn new(
        traveler: &Member,
        center_x: i32,
        stand_y: i32,
        facing_right: bool,
        reduce_motion: bool,
    ) -> Self {
        let spec = AnimationSpec::for_action(ActionKind::Idle);
        let dress = traveler.dress;
        let frame_count = if reduce_motion { 1 } else { spec.frames };
        let frames: Vec<Canvas> = (0..frame_count)
            .map(|frame| {
                CreatureRenderer::render_dressed_frame(
                    traveler.genome(),
                    dress,
                    ActionKind::Idle,
                    frame,
                    facing_right,
                )
            })
            .collect();

        // Where its face is: the renderer draws frames facing right and mirrors the rest.
        let size = FRAME_SIZE as i32;
        let anchor = CreatureRenderer::render_dressed_body_frame(
            traveler.genome(),
            dress,
            ActionKind::Idle,
            0,
            reduce_motion,
        )
        .face_anchor;
        let face = if facing_right {
            (anchor.x, anchor.y)
        } else {
            (size - anchor.x, anchor.y)
        };

        // Stand it on the boards by the lowest opaque row of its resting frame.
        let (min_x, min_y, max_x, max_y) = frames[0]
            .alpha_bounds()
            .map(|(a, b, c, d)| (a as i32, b as i32, c as i32, d as i32))
            .unwrap_or((0, 0, size - 1, size - 1));
        let origin = (center_x - size / 2, stand_y - max_y);
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
            face,
            spec,
            phase: (traveler.id % 997) as f32 / 997.0 * 2.0,
        }
    }
}

/// A soft oval of shade under a traveller's feet.
fn shadow(scene: &mut Canvas, center_x: i32, foot_y: i32, half_width: i32) {
    let radius = (half_width - 2).max(4);
    ellipse(scene, center_x, foot_y + 1, radius, 1, rgba(0x4a3024, 64));
    ellipse(
        scene,
        center_x,
        foot_y + 1,
        radius - 3,
        1,
        rgba(0x4a3024, 40),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_travel::{TravelSnapshot, TravelerId};

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// The sample, with copies of its first traveller added until there are `count`.
    fn crowd(count: usize) -> Cast {
        let mut snapshot: TravelSnapshot = formiga_travel::sample::snapshot();
        snapshot.travelers.truncate(count);
        let first = snapshot.travelers[0].clone();
        while snapshot.travelers.len() < count {
            let mut extra = first.clone();
            extra.id = TravelerId(first.id.0 + snapshot.travelers.len() as u64);
            extra.role = formiga_travel::TravelRole::Adult;
            snapshot.travelers.push(extra);
        }
        Cast::new(snapshot).unwrap()
    }

    #[test]
    fn every_traveller_is_drawn_standing_on_the_platform() {
        let cast = sample();
        let station = Station::new(&cast, Journey::Here, &[]);
        assert_eq!(station.travelers().len(), cast.members.len());
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
        let station = Station::new(&crowd(6), Journey::Here, &[]);
        for pair in station.travelers().windows(2) {
            let gap = pair[1].origin.0 - pair[0].origin.0;
            assert!(gap >= 40, "only {gap} pixels between neighbours");
        }
    }

    #[test]
    fn composing_draws_the_travellers_over_the_scenery() {
        let station = Station::new(&sample(), Journey::Here, &[]);
        let mut empty = scenery::backdrop(&[], &Arrangement::new());
        scenery::smoke(&mut empty, 0.0, false);
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
        let station = Station::new(&Cast::new(snapshot).unwrap(), Journey::Here, &[]);
        assert_eq!(station.frame_key(0.0), station.frame_key(7.3));
    }

    #[test]
    fn an_arrival_brings_the_train_in_and_settles_on_its_own() {
        let cast = sample();
        let mut station = Station::new(&cast, Journey::Arriving { since: 0.0 }, &[]);
        assert!(station.in_motion(1.0));
        assert!(
            station.traveler_at(100.0, 150.0).is_none(),
            "no tooltips mid-arrival"
        );
        let settled = Station::new(&cast, Journey::Here, &[]);
        assert_ne!(
            station.compose(3.5),
            settled.compose(3.5),
            "the train is at the platform"
        );
        station.update(60.0);
        assert!(station.is_settled());
        assert!(!station.in_motion(60.0));
    }

    #[test]
    fn going_home_ends_with_everyone_gone() {
        let mut station = Station::new(&sample(), Journey::Here, &[]);
        station.set_off_home(5.0);
        assert!(!station.gone_home(6.0));
        station.set_off_home(7.0);
        assert!(
            station.gone_home(60.0),
            "calling again does not restart the train"
        );
        let empty = {
            let mut scene = scenery::backdrop(&[], &Arrangement::new());
            scenery::smoke(&mut scene, 60.0, false);
            scene
        };
        assert_eq!(
            station.compose(60.0),
            empty,
            "the platform is empty once they have gone"
        );
    }

    #[test]
    fn skipping_an_arrival_puts_everyone_on_the_platform() {
        let mut station = Station::new(&sample(), Journey::Arriving { since: 0.0 }, &[]);
        station.skip_arrival();
        assert!(station.is_settled());
    }

    #[test]
    fn hit_testing_finds_whoever_is_under_the_pointer() {
        let station = Station::new(&sample(), Journey::Here, &[]);
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
