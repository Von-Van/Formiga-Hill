//! The train's comings and goings, as a timeline: where the train is and where each traveller is
//! at any moment, and what is heard of it. Pure, so the whole sequence can be checked without
//! drawing a pixel or playing a sound.
//!
//! Arriving, the train pulls in with everyone at a window, stops, lets them hop down one at a
//! time, and steams away. Leaving is the same in reverse. With reduced motion there is no
//! movement at all: a held view of the train at the platform with everyone aboard, then a cut.
//!
//! It whistles as it comes and again as it goes, chuffs once for each turn of its driving wheels,
//! so the chuffs slow as it pulls in and quicken as it pulls out, and lets off steam as it stops.

use super::SCENE_WIDTH;
use super::train::{DRIVING_WHEEL, TRAIN_LENGTH, WINDOWS};

const PULL_IN: f32 = 3.2;
const SETTLE: f32 = 0.7;
const HOP: f32 = 0.42;
const HOP_GAP: f32 = 0.32;
const LINGER: f32 = 0.7;
const PULL_OUT: f32 = 3.0;
/// How long the held view lasts when motion is reduced.
const HOLD: f32 = 1.6;
/// How high a hop goes, in scene pixels.
const HOP_HEIGHT: f32 = 12.0;

/// Where the train stops: centred on the platform.
pub const STOP_X: f32 = (SCENE_WIDTH as f32 - TRAIN_LENGTH as f32) / 2.0;
const START_X: f32 = -(TRAIN_LENGTH as f32) - 8.0;
const END_X: f32 = SCENE_WIDTH as f32 + 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Journey {
    /// The colony is arriving; `since` is when the train set off, in seconds of the visit.
    Arriving { since: f32 },
    /// Everyone is on the platform and the line is clear.
    Here,
    /// The colony is going home.
    Leaving { since: f32 },
}

/// Something the train is heard doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Heard {
    /// Its whistle, as it pulls in and again as it pulls out.
    Whistle,
    /// A puff of steam from the engine, as loud as the engine is near: 1 at the middle of the
    /// platform, softer towards the ends and beyond them.
    Chuff { near: f32 },
    /// Steam let off as it comes to a stop.
    Brakes,
}

/// When the whistle goes, as the train comes into hearing; when it lets off steam, a little
/// before it stops; and when it whistles again, just before it pulls away.
const WHISTLE_IN: f32 = 0.15;
const BRAKES_BEFORE_STOP: f32 = 0.9;
const WHISTLE_BEFORE_OFF: f32 = 0.35;

/// Where one traveller is at a moment of the journey.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// On the platform, `lift` pixels off the boards mid-hop.
    Platform { lift: f32 },
    /// Aboard, at this window.
    Window(usize),
    /// Aboard, but with no window of its own: only a train of more than six has these.
    Unseen,
    /// Gone home.
    Away,
}

/// Everything that moves, at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct Stage {
    /// The train's left end, if it is in sight.
    pub train_x: Option<f32>,
    /// How far the train has rolled since it set off, for turning its wheels.
    pub rolled: f32,
    pub places: Vec<Place>,
}

impl Journey {
    /// Whether the journey has run its course by `now`.
    pub fn finished(self, now: f32, travelers: usize, reduce_motion: bool) -> bool {
        match self {
            Self::Here => true,
            Self::Arriving { since } | Self::Leaving { since } => {
                now - since >= duration(travelers, reduce_motion)
            }
        }
    }

    /// What is heard of the train after `from` and up to `now`, in order: everything once,
    /// however the moments in between are sampled.
    pub fn heard(self, from: f32, now: f32, travelers: usize, reduce_motion: bool) -> Vec<Heard> {
        let (Self::Arriving { since } | Self::Leaving { since }) = self else {
            return Vec::new();
        };
        let (from, to) = (from - since, now - since);
        let during = |t: f32| from < t && t <= to;
        if reduce_motion {
            // A held view of the train at the platform: nothing rolls, but it whistles.
            return if during(WHISTLE_IN) {
                vec![Heard::Whistle]
            } else {
                Vec::new()
            };
        }
        let off = PULL_IN + SETTLE + hops(travelers) + LINGER;
        let mut heard = Vec::new();
        if during(WHISTLE_IN) {
            heard.push(Heard::Whistle);
        }
        if during(PULL_IN - BRAKES_BEFORE_STOP) {
            heard.push(Heard::Brakes);
        }
        if during(off - WHISTLE_BEFORE_OFF) {
            heard.push(Heard::Whistle);
        }
        // A chuff each time the driving wheels come round: one is plenty for a moment however
        // many turns it spans, which only a long pause between frames could.
        let turn = std::f32::consts::TAU * DRIVING_WHEEL;
        let (before, after) = (rolled(from, off), rolled(to, off));
        if (after / turn).floor() > (before / turn).floor() && to > 0.0 {
            heard.push(Heard::Chuff {
                near: nearness(START_X + after),
            });
        }
        heard
    }

    pub fn stage(self, now: f32, travelers: usize, reduce_motion: bool) -> Stage {
        match self {
            Self::Here => Stage {
                train_x: None,
                rolled: 0.0,
                places: vec![Place::Platform { lift: 0.0 }; travelers],
            },
            Self::Arriving { since } => {
                timeline(now - since, travelers, reduce_motion, Direction::Off)
            }
            Self::Leaving { since } => {
                timeline(now - since, travelers, reduce_motion, Direction::On)
            }
        }
    }
}

/// The window each traveller takes: spread along the train however many are travelling, and
/// none for anyone beyond the sixth.
pub fn window_for(index: usize, travelers: usize) -> Option<usize> {
    if travelers <= WINDOWS {
        Some(index * WINDOWS / travelers.max(1))
    } else {
        (index < WINDOWS).then_some(index)
    }
}

fn duration(travelers: usize, reduce_motion: bool) -> f32 {
    if reduce_motion {
        HOLD
    } else {
        PULL_IN + SETTLE + hops(travelers) + LINGER + PULL_OUT
    }
}

fn hops(travelers: usize) -> f32 {
    travelers.saturating_sub(1) as f32 * HOP_GAP + HOP
}

/// How far the train has rolled `t` seconds into a journey on which it pulls out at `off`: easing
/// in to the stop, standing, then gathering speed as it goes.
fn rolled(t: f32, off: f32) -> f32 {
    let pull_in = STOP_X - START_X;
    if t <= 0.0 {
        0.0
    } else if t < PULL_IN {
        let u = t / PULL_IN;
        pull_in * (1.0 - (1.0 - u).powi(3))
    } else if t < off {
        pull_in
    } else {
        let u = ((t - off) / PULL_OUT).min(1.0);
        pull_in + (END_X - STOP_X) * u * u
    }
}

/// How near the engine sounds with the train's back end at `x`: loudest when it is at the middle
/// of the platform, never quite silent, as a train is heard well before and after it is seen.
fn nearness(x: f32) -> f32 {
    let engine = x + TRAIN_LENGTH as f32 - 24.0;
    let middle = SCENE_WIDTH as f32 / 2.0;
    (1.0 - (engine - middle).abs() / SCENE_WIDTH as f32).clamp(0.25, 1.0)
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    /// Getting off the train.
    Off,
    /// Getting on it.
    On,
}

fn timeline(t: f32, travelers: usize, reduce_motion: bool, direction: Direction) -> Stage {
    let aboard = |index| window_for(index, travelers).map_or(Place::Unseen, Place::Window);
    let ashore = Place::Platform { lift: 0.0 };
    let after = match direction {
        Direction::Off => ashore,
        Direction::On => Place::Away,
    };

    if reduce_motion {
        // A held view of everyone aboard, then a cut to how things end.
        if t < HOLD {
            return Stage {
                train_x: Some(STOP_X),
                rolled: 0.0,
                places: (0..travelers).map(aboard).collect(),
            };
        }
        return Stage {
            train_x: None,
            rolled: 0.0,
            places: vec![after; travelers],
        };
    }

    let off = PULL_IN + SETTLE + hops(travelers) + LINGER;
    if t < PULL_IN {
        let travelled = rolled(t, off);
        let place = |index| match direction {
            Direction::Off => aboard(index),
            Direction::On => ashore,
        };
        return Stage {
            train_x: Some(START_X + travelled),
            rolled: travelled,
            places: (0..travelers).map(place).collect(),
        };
    }

    let hopping = t - PULL_IN - SETTLE;
    let places = (0..travelers)
        .map(|index| {
            let start = index as f32 * HOP_GAP;
            let (before, after_hop) = match direction {
                Direction::Off => (aboard(index), ashore),
                Direction::On => (ashore, aboard(index)),
            };
            if hopping < start {
                before
            } else if hopping < start + HOP {
                let u = (hopping - start) / HOP;
                Place::Platform {
                    lift: (u * std::f32::consts::PI).sin() * HOP_HEIGHT,
                }
            } else {
                after_hop
            }
        })
        .collect::<Vec<_>>();

    if t < off {
        return Stage {
            train_x: Some(STOP_X),
            rolled: rolled(t, off),
            places,
        };
    }
    if t < off + PULL_OUT {
        let travelled = rolled(t, off);
        return Stage {
            train_x: Some(START_X + travelled),
            rolled: travelled,
            places,
        };
    }
    Stage {
        train_x: None,
        rolled: 0.0,
        places: vec![after; travelers],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERYONE: usize = 4;

    fn at(journey: Journey, now: f32) -> Stage {
        journey.stage(now, EVERYONE, false)
    }

    #[test]
    fn arriving_everyone_starts_aboard_and_ends_on_the_platform() {
        let journey = Journey::Arriving { since: 0.0 };
        let start = at(journey, 0.0);
        assert!(
            start.train_x.unwrap() < 0.0,
            "the train starts out of sight"
        );
        assert!(
            start
                .places
                .iter()
                .all(|place| matches!(place, Place::Window(_)))
        );

        let end = duration(EVERYONE, false);
        let settled = at(journey, end + 0.1);
        assert_eq!(settled, at(Journey::Here, 0.0));
        assert!(journey.finished(end, EVERYONE, false));
        assert!(!journey.finished(end - 0.1, EVERYONE, false));
    }

    #[test]
    fn the_train_stops_at_the_platform_while_everyone_gets_off() {
        let journey = Journey::Arriving { since: 0.0 };
        for t in [PULL_IN + 0.01, PULL_IN + SETTLE + hops(EVERYONE) / 2.0] {
            assert_eq!(at(journey, t).train_x, Some(STOP_X));
        }
        let stopped = at(journey, PULL_IN + SETTLE + hops(EVERYONE) + 0.01);
        assert!(
            stopped
                .places
                .iter()
                .all(|place| *place == Place::Platform { lift: 0.0 })
        );
    }

    #[test]
    fn the_train_never_runs_backwards() {
        for journey in [
            Journey::Arriving { since: 0.0 },
            Journey::Leaving { since: 0.0 },
        ] {
            let mut last = f32::MIN;
            let mut t = 0.0;
            while let Some(x) = at(journey, t).train_x {
                assert!(x >= last, "the train went back at {t}");
                last = x;
                t += 0.05;
            }
            assert!(last > STOP_X, "the train left the platform");
        }
    }

    #[test]
    fn travellers_hop_down_one_at_a_time_in_order() {
        let journey = Journey::Arriving { since: 0.0 };
        let first_hop = PULL_IN + SETTLE + HOP / 2.0;
        let stage = at(journey, first_hop);
        assert!(matches!(stage.places[0], Place::Platform { lift } if lift > HOP_HEIGHT * 0.9));
        assert!(
            stage.places[1..]
                .iter()
                .all(|place| matches!(place, Place::Window(_)))
        );
    }

    #[test]
    fn leaving_everyone_boards_and_goes() {
        let journey = Journey::Leaving { since: 10.0 };
        let start = at(journey, 10.0);
        assert!(
            start
                .places
                .iter()
                .all(|place| *place == Place::Platform { lift: 0.0 })
        );
        let departing = at(
            journey,
            10.0 + PULL_IN + SETTLE + hops(EVERYONE) + LINGER + 0.5,
        );
        assert!(
            departing
                .places
                .iter()
                .all(|place| matches!(place, Place::Window(_)))
        );
        let gone = at(journey, 10.0 + duration(EVERYONE, false) + 0.01);
        assert_eq!(gone.train_x, None);
        assert!(gone.places.iter().all(|place| *place == Place::Away));
    }

    #[test]
    fn reduced_motion_cuts_instead_of_moving() {
        let journey = Journey::Arriving { since: 0.0 };
        let held = journey.stage(0.5, EVERYONE, true);
        assert_eq!(held, journey.stage(1.2, EVERYONE, true));
        assert_eq!(held.train_x, Some(STOP_X));
        assert_eq!(journey.stage(HOLD, EVERYONE, true), at(Journey::Here, 0.0));
    }

    /// Everything heard over a whole journey, listened to every `step` seconds, and when.
    fn listen(journey: Journey, step: f32, reduce_motion: bool) -> Vec<(f32, Heard)> {
        let mut heard = Vec::new();
        let mut t = 0.0;
        while t < duration(EVERYONE, false) + 1.0 {
            for sound in journey.heard(t, t + step, EVERYONE, reduce_motion) {
                heard.push((t + step, sound));
            }
            t += step;
        }
        heard
    }

    #[test]
    fn the_train_whistles_coming_and_going_chuffs_as_it_rolls_and_brakes_to_a_stop() {
        for journey in [
            Journey::Arriving { since: 0.0 },
            Journey::Leaving { since: 0.0 },
        ] {
            let heard = listen(journey, 1.0 / 60.0, false);
            let whistles: Vec<f32> = heard
                .iter()
                .filter(|(_, sound)| *sound == Heard::Whistle)
                .map(|(t, _)| *t)
                .collect();
            assert_eq!(whistles.len(), 2, "{heard:?}");
            assert!(whistles[0] < PULL_IN && whistles[1] > PULL_IN + SETTLE);
            let brakes: Vec<f32> = heard
                .iter()
                .filter(|(_, sound)| *sound == Heard::Brakes)
                .map(|(t, _)| *t)
                .collect();
            assert_eq!(brakes.len(), 1);
            assert!(brakes[0] < PULL_IN, "the brakes go on before it stops");
            let chuffs = heard
                .iter()
                .filter(|(_, sound)| matches!(sound, Heard::Chuff { .. }))
                .count();
            assert!(chuffs >= 10, "only {chuffs} chuffs");
        }
        assert!(Journey::Here.heard(0.0, 100.0, EVERYONE, false).is_empty());
    }

    #[test]
    fn the_chuffs_slow_as_the_train_pulls_in_and_quicken_as_it_pulls_out() {
        let heard = listen(Journey::Arriving { since: 0.0 }, 1.0 / 240.0, false);
        let chuffs: Vec<f32> = heard
            .iter()
            .filter(|(_, sound)| matches!(sound, Heard::Chuff { .. }))
            .map(|(t, _)| *t)
            .collect();
        let (coming, going): (Vec<f32>, Vec<f32>) = chuffs.iter().partition(|t| **t < PULL_IN);
        let gaps = |times: &[f32]| -> Vec<f32> {
            times.windows(2).map(|pair| pair[1] - pair[0]).collect()
        };
        assert!(gaps(&coming).windows(2).all(|pair| pair[1] > pair[0]));
        assert!(gaps(&going).windows(2).all(|pair| pair[1] < pair[0]));
        // Nothing rolls while it stands at the platform.
        assert!(going[0] > PULL_IN + SETTLE + hops(EVERYONE) + LINGER);
    }

    #[test]
    fn everything_is_heard_once_however_the_frames_fall() {
        let journey = Journey::Leaving { since: 0.0 };
        let count = |step: f32| {
            let heard = listen(journey, step, false);
            let chuffs = heard
                .iter()
                .filter(|(_, sound)| matches!(sound, Heard::Chuff { .. }))
                .count();
            (heard.len() - chuffs, chuffs)
        };
        assert_eq!(count(1.0 / 30.0), count(1.0 / 144.0));
    }

    #[test]
    fn with_motion_reduced_the_train_stands_and_only_whistles() {
        let heard = listen(Journey::Arriving { since: 0.0 }, 1.0 / 60.0, true);
        let sounds: Vec<Heard> = heard.into_iter().map(|(_, sound)| sound).collect();
        assert_eq!(sounds, [Heard::Whistle]);
    }

    #[test]
    fn the_engine_is_heard_loudest_at_the_platform() {
        assert!(nearness(STOP_X) > nearness(START_X));
        assert!(nearness(STOP_X) > nearness(END_X));
        assert!(nearness(END_X + 500.0) > 0.0);
    }

    #[test]
    fn every_traveller_gets_a_window_of_their_own() {
        for travelers in 1..=WINDOWS {
            let windows: Vec<_> = (0..travelers)
                .map(|i| window_for(i, travelers).unwrap())
                .collect();
            let mut unique = windows.clone();
            unique.dedup();
            assert_eq!(unique, windows, "{travelers} travellers share a window");
            assert!(windows.iter().all(|window| *window < WINDOWS));
        }
    }

    #[test]
    fn a_crowd_fills_the_windows_and_the_rest_ride_unseen() {
        let windows: Vec<_> = (0..12).map(|i| window_for(i, 12)).collect();
        assert!(windows[..WINDOWS].iter().all(Option::is_some));
        assert!(windows[WINDOWS..].iter().all(Option::is_none));
    }
}
