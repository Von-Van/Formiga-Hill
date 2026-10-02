//! The train's comings and goings, as a timeline: where the train is and where each traveller is
//! at any moment. Pure, so the whole sequence can be checked without drawing a pixel.
//!
//! Arriving, the train pulls in with everyone at a window, stops, lets them hop down one at a
//! time, and steams away. Leaving is the same in reverse. With reduced motion there is no
//! movement at all: a held view of the train at the platform with everyone aboard, then a cut.

use super::SCENE_WIDTH;
use super::train::{TRAIN_LENGTH, WINDOWS};

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

/// Where one traveller is at a moment of the journey.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    /// On the platform, `lift` pixels off the boards mid-hop.
    Platform { lift: f32 },
    /// Aboard, at this window.
    Window(usize),
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

/// The window each traveller takes: spread along the train however many are travelling.
pub fn window_for(index: usize, travelers: usize) -> usize {
    index * WINDOWS / travelers.max(1)
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

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    /// Getting off the train.
    Off,
    /// Getting on it.
    On,
}

fn timeline(t: f32, travelers: usize, reduce_motion: bool, direction: Direction) -> Stage {
    let aboard = |index| Place::Window(window_for(index, travelers));
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

    let pull_in_distance = STOP_X - START_X;
    if t < PULL_IN {
        // Easing in to the stop.
        let u = t / PULL_IN;
        let travelled = pull_in_distance * (1.0 - (1.0 - u).powi(3));
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

    let leaving = hopping - hops(travelers) - LINGER;
    if leaving < 0.0 {
        return Stage {
            train_x: Some(STOP_X),
            rolled: pull_in_distance,
            places,
        };
    }
    if leaving < PULL_OUT {
        // Gathering speed as it goes.
        let u = leaving / PULL_OUT;
        let travelled = (END_X - STOP_X) * u * u;
        return Stage {
            train_x: Some(STOP_X + travelled),
            rolled: pull_in_distance + travelled,
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

    #[test]
    fn every_traveller_gets_a_window_of_their_own() {
        for travelers in 1..=WINDOWS {
            let windows: Vec<_> = (0..travelers).map(|i| window_for(i, travelers)).collect();
            let mut unique = windows.clone();
            unique.dedup();
            assert_eq!(unique, windows, "{travelers} travellers share a window");
            assert!(windows.iter().all(|window| *window < WINDOWS));
        }
    }
}
