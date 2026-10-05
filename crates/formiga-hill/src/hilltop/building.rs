//! Building something on the Hilltop, the way the colony does it: everyone gathers round the spot
//! and lends a hand in its own way (see `Character::knack`), fetching things, hammering, holding
//! it steady, pointing out where everything goes or supervising from a comfortable seat; then
//! what they built appears in a puff of dust and they cheer it.
//!
//! With motion reduced it is a tableau rather than a scene: everyone is simply there round the
//! spot, each held in its work; then the piece is simply there, without the puff, and everyone is
//! held in a cheer.

use super::SPOTS;
use crate::actor::Step;
use crate::cast::Id;
use crate::character::Knack;
use crate::paint::{chance, put, rgba};
use crate::playground::Playground;
use formiga_art::Canvas;

/// How long, at most, the colony takes to gather round before the work begins in earnest, and how
/// long the work goes on after.
const GATHER_SECS: f32 = 5.0;
const WORK_SECS: f32 = 3.2;
/// With motion reduced, how long the tableau of everyone at work is held.
const HELD_SECS: f32 = 1.8;
/// How long everyone cheers what they built before getting on with their day.
const CHEER_SECS: f32 = 2.6;
/// How long a puff of dust takes to clear.
pub const PUFF_SECS: f32 = 0.9;

/// Where builders stand round a spot, from where the piece meets the ground: either side of it,
/// never in front, so what they build stays in view; nearest first, then further out.
const ROUND: [(f32, f32); 8] = [
    (-30.0, 1.0),
    (30.0, 1.0),
    (-46.0, 8.0),
    (46.0, 8.0),
    (-34.0, 15.0),
    (34.0, 15.0),
    (-52.0, -5.0),
    (52.0, -5.0),
];

/// A turn in the building, for whoever is watching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moment {
    /// What they built is there: draw it from now on.
    Appeared,
    /// Everyone has cheered it and gone back to free play.
    Over,
}

pub struct Building {
    pub spot: u8,
    builders: Vec<(Id, Knack)>,
    began: f32,
    working_since: Option<f32>,
    appeared: Option<f32>,
    over: bool,
    reduce_motion: bool,
}

impl Building {
    /// Calls the whole colony over to build on `spot`, each to its place round it and to its own
    /// kind of work there.
    pub fn begin(ground: &mut Playground, spot: u8, now: f32) -> Self {
        let (x, y) = SPOTS[usize::from(spot)];
        let reduce_motion = ground.reduce_motion();
        let mut builders = Vec::new();
        for (index, id) in ground.ids().into_iter().enumerate() {
            let Some(character) = ground.character(id).cloned() else {
                continue;
            };
            let knack = character.knack();
            let (dx, dy) = ROUND[index % ROUND.len()];
            let place = ground.beside_point((x, y + dy), dx);
            let mut steps = vec![Step::Walk { to: place }, Step::FaceX(x)];
            // A fetcher goes off to the edge of things and comes back with something; with
            // motion reduced, that would only be a flicker back and forth, so it simply works.
            if knack == Knack::Fetching && !reduce_motion {
                let fetch = ground.beside_point((x, y + dy), dx * 2.2);
                steps.push(Step::Walk { to: fetch });
                steps.push(Step::Beat(character.pick_up()));
                steps.push(Step::Walk { to: place });
                steps.push(Step::FaceX(x));
            }
            steps.extend(character.work(knack).into_iter().map(Step::Beat));
            ground.direct(id, steps, now);
            builders.push((id, knack));
        }
        ground.reserve(builders.iter().map(|(id, _)| *id).collect());
        Self {
            spot,
            builders,
            began: now,
            working_since: None,
            appeared: None,
            over: false,
            reduce_motion,
        }
    }

    /// Whether what is being built is there yet.
    pub fn showing(&self) -> bool {
        self.appeared.is_some()
    }

    /// How long the colony has been hard at work, from when everyone was gathered round until
    /// what they are building appears; nothing before or after.
    pub fn worked_for(&self, now: f32) -> Option<f32> {
        match (self.working_since, self.appeared) {
            (Some(since), None) => Some((now - since).max(0.0)),
            _ => None,
        }
    }

    /// Plays the building on to `now`, and says when it reaches a turn.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) -> Option<Moment> {
        if self.over {
            return None;
        }
        let Some(since) = self.working_since else {
            // Gathered once everyone but the fetchers, off on their errands, is in place.
            let gathered = self
                .builders
                .iter()
                .filter(|(_, knack)| *knack != Knack::Fetching)
                .all(|(id, _)| !ground.walking(*id));
            if gathered || now - self.began > GATHER_SECS {
                self.working_since = Some(now);
            }
            return None;
        };
        let Some(appeared) = self.appeared else {
            let work = if self.reduce_motion {
                HELD_SECS
            } else {
                WORK_SECS
            };
            if now - since < work {
                return None;
            }
            self.appeared = Some(now);
            let x = SPOTS[usize::from(self.spot)].0;
            for &(id, knack) in &self.builders {
                let Some(character) = ground.character(id).cloned() else {
                    continue;
                };
                let steps = std::iter::once(Step::FaceX(x))
                    .chain(character.admire(knack).into_iter().map(Step::Beat))
                    .collect();
                ground.direct(id, steps, now);
            }
            return Some(Moment::Appeared);
        };
        if now - appeared < CHEER_SECS {
            return None;
        }
        self.over = true;
        ground.release();
        Some(Moment::Over)
    }

    /// The puff what they built appears in, while it clears; with motion reduced there is none,
    /// and the piece is simply there.
    pub fn draw(&self, scene: &mut Canvas, now: f32) {
        if let Some(at) = self.appeared
            && !self.reduce_motion
        {
            puff(scene, self.spot, now - at);
        }
    }
}

/// A puff of dust on a spot, `age` seconds after it went up: soft cream clouds billowing out from
/// where the piece stands and thinning as they go, with a few bits of leaf and twig flung out.
pub fn puff(scene: &mut Canvas, spot: u8, age: f32) {
    let Some(&(x, y)) = SPOTS.get(usize::from(spot)) else {
        return;
    };
    let progress = (age / PUFF_SECS).clamp(0.0, 1.0);
    if progress >= 1.0 {
        return;
    }
    let fade = 1.0 - progress;
    let centre = (x, y - 14.0);
    let mut clouds = vec![(centre, 10.0 * (1.0 - progress * 0.7))];
    for index in 0..9 {
        let angle = index as f32 / 9.0 * std::f32::consts::TAU + 0.3;
        let reach = 8.0 + 18.0 * progress.sqrt();
        let at = (
            centre.0 + angle.cos() * reach * 1.4,
            centre.1 + angle.sin() * reach * 0.8,
        );
        clouds.push((at, 3.0 + 4.0 * fade));
    }
    for ((cx, cy), radius) in clouds {
        let reach = radius.ceil() as i32 + 1;
        for py in cy as i32 - reach..=cy as i32 + reach {
            for px in cx as i32 - reach..=cx as i32 + reach {
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                let d = (dx * dx + dy * dy).sqrt();
                if d > radius {
                    continue;
                }
                // Lit from the upper left, a shade darker round the lower right of each.
                let lower = dx + dy > radius * 0.6;
                let (color, alpha) = if lower {
                    (0xd8ccb0, 200.0)
                } else if dx + dy < -radius * 0.5 {
                    (0xfffaf0, 235.0)
                } else {
                    (0xf6eed8, 225.0)
                };
                // Thinning out from the edges in as it clears.
                if d > radius * (1.0 - 0.6 * progress) && chance(px, py, 520, 140) {
                    continue;
                }
                put(scene, px, py, rgba(color, (alpha * fade) as u8));
            }
        }
    }
    // Bits of leaf and twig flung out.
    for index in 0..6 {
        let angle = index as f32 / 6.0 * std::f32::consts::TAU;
        let reach = 10.0 + 30.0 * progress;
        let (px, py) = (
            centre.0 + angle.cos() * reach * 1.3,
            centre.1 + angle.sin() * reach * 0.7 - 6.0 * progress + 14.0 * progress * progress,
        );
        let color = if index % 2 == 0 { 0x6fa866 } else { 0x9a7048 };
        put(
            scene,
            px as i32,
            py as i32,
            rgba(color, (255.0 * fade) as u8),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::hilltop::{self, Arrangement};

    fn run(
        ground: &mut Playground,
        building: &mut Building,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<(f32, Moment)> {
        let mut moments = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            if let Some(moment) = building.tick(ground, now) {
                moments.push((now, moment));
            }
        }
        moments
    }

    #[test]
    fn the_colony_gathers_round_builds_cheers_and_goes_back_to_play() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut ground = hilltop::open(&cast, 0.0, &Arrangement::new());
        let mut building = Building::begin(&mut ground, 9, 12.0);
        assert!(!building.showing());
        let moments = run(&mut ground, &mut building, &cast, 12.0, 40.0);
        let kinds: Vec<Moment> = moments.iter().map(|(_, moment)| *moment).collect();
        assert_eq!(kinds, [Moment::Appeared, Moment::Over]);
        assert!(building.showing());
        assert_eq!(building.worked_for(moments[0].0), None, "the work is done");
        let (appeared, over) = (moments[0].0, moments[1].0);
        assert!(
            appeared - 12.0 < GATHER_SECS + WORK_SECS + 0.1,
            "a short scene"
        );
        assert!(over > appeared);
        // Back to free play: everyone wanders off on their own again.
        let (x, _) = SPOTS[9];
        let mut now = 40.0;
        while now < 80.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
        }
        let away = ground
            .ids()
            .into_iter()
            .filter(|id| {
                ground
                    .position(*id)
                    .is_some_and(|(px, _)| (px - x).abs() > 40.0)
            })
            .count();
        assert!(away > 0, "nobody went back to play");
    }

    #[test]
    fn the_colony_is_at_work_from_when_it_has_gathered_until_it_is_built() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut ground = hilltop::open(&cast, 0.0, &Arrangement::new());
        let mut building = Building::begin(&mut ground, 9, 12.0);
        assert_eq!(building.worked_for(12.0), None, "still gathering round");
        let mut worked = Vec::new();
        let mut now = 12.0;
        while now < 40.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            building.tick(&mut ground, now);
            worked.extend(building.worked_for(now));
        }
        assert!(worked.windows(2).all(|pair| pair[1] > pair[0]));
        let longest = worked.last().copied().unwrap_or_default();
        assert!((longest - WORK_SECS).abs() < 0.1, "worked for {longest}");
    }

    #[test]
    fn with_motion_reduced_it_is_a_tableau_and_a_cut() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let mut ground = hilltop::open(&cast, 0.0, &Arrangement::new());
        let mut building = Building::begin(&mut ground, 4, 0.0);
        // Everyone is simply there round the spot, and stays put while it is built.
        ground.tick(&cast, 0.05);
        let placed: Vec<_> = ground
            .ids()
            .into_iter()
            .map(|id| ground.position(id))
            .collect();
        let moments = run(&mut ground, &mut building, &cast, 0.05, 1.0);
        assert!(moments.is_empty());
        let still: Vec<_> = ground
            .ids()
            .into_iter()
            .map(|id| ground.position(id))
            .collect();
        assert_eq!(placed, still, "nobody moved");
        let moments = run(&mut ground, &mut building, &cast, 1.0, 10.0);
        assert_eq!(
            moments.first().map(|(_, moment)| *moment),
            Some(Moment::Appeared)
        );
        // No puff: the piece is simply there.
        let mut scene = Canvas::new(384, 216);
        building.draw(&mut scene, moments[0].0 + 0.1);
        assert!(scene.alpha_bounds().is_none());
    }

    #[test]
    fn a_puff_billows_out_and_clears() {
        let drawn = |age| {
            let mut scene = Canvas::new(384, 216);
            puff(&mut scene, 7, age);
            scene.pixels().iter().filter(|pixel| pixel.a > 0).count()
        };
        assert!(drawn(0.05) > 0);
        assert!(drawn(0.6) > 0);
        assert_eq!(drawn(PUFF_SECS + 0.01), 0, "cleared");
    }
}
