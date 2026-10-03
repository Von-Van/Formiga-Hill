//! The Clubhouse, at the edge of the green: a snug room with a fire, where stories are staged.
//! Between stories it is somewhere else to spend time, played in like any playground: someone
//! warms by the hearth, someone looks along the books or out at the Hill. The notice board shows
//! a card for each story, starred once the colony has finished it.

mod scenery;

use crate::cast::Cast;
use crate::daylight::{Daylight, Nightlights};
use crate::finds::Use;
use crate::hilltop::Arrangement;
use crate::playground::{Attraction, Layout, Patch, Playground};
use formiga_art::Canvas;
use scenery::{BOARD, FLICKERS, GROUND, HEARTHSIDE, SEATS};
use std::sync::LazyLock;

/// The Clubhouse's named spots, as a story's `walk … to = "…"` names them.
const SPOTS: [(&str, (f32, f32)); 13] = [
    ("rug", (156.0, 146.0)),
    ("hearth", (142.0, 110.0)),
    ("armchair", (102.0, 150.0)),
    ("bookshelf", (46.0, 108.0)),
    ("window", (244.0, 108.0)),
    ("board", (312.0, 108.0)),
    ("table", (306.0, 186.0)),
    ("chest", (76.0, 196.0)),
    ("centre", (192.0, 164.0)),
    ("left", (34.0, 150.0)),
    ("right", (348.0, 156.0)),
    ("front", (192.0, 198.0)),
    ("back", (194.0, 106.0)),
];

/// How many times a second the fire changes.
const FLICKER_RATE: f32 = 6.0;

/// The floor just behind each piece of furniture, where whoever stands is hidden: free play
/// keeps out of it.
static FOOTPRINTS: LazyLock<Vec<Patch>> = LazyLock::new(|| {
    scenery::props()
        .iter()
        .map(|prop| {
            let (left, _, right, _) = prop.bounds();
            (
                left as f32 - 6.0,
                prop.base - 16.0,
                right as f32 + 6.0,
                prop.base + 3.0,
            )
        })
        .collect()
});

fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = GROUND;
    let inside = |(l, t, r, b): Patch| x >= l && x <= r && y >= t && y <= b;
    inside((left, top, right, bottom)) && !FOOTPRINTS.iter().any(|patch| inside(*patch))
}

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &SPOTS,
        seats: Some(SEATS),
        shade: Some(HEARTHSIDE),
        // In through the door in the near wall, off to the right.
        entrance: (400.0, 160.0),
        entrance_step: (18.0, 6.0),
    }
}

/// What is worth going over to between stories.
fn attractions() -> Vec<Attraction> {
    vec![
        Attraction {
            stand: (60.0, 110.0),
            facing_x: 40.0,
            use_: Use::Look,
        },
        Attraction {
            stand: (226.0, 108.0),
            facing_x: 244.0,
            use_: Use::Gaze,
        },
        Attraction {
            stand: (122.0, 108.0),
            facing_x: 142.0,
            use_: Use::Rest,
        },
        Attraction {
            stand: (296.0, 108.0),
            facing_x: 312.0,
            use_: Use::Look,
        },
        Attraction {
            stand: (78.0, 196.0),
            facing_x: 44.0,
            use_: Use::Play,
        },
    ]
}

/// Whether a point in the scene is on the notice board.
pub fn on_board(x: f32, y: f32) -> bool {
    let (left, top, wide, tall) = BOARD;
    (left as f32..(left + wide) as f32).contains(&x)
        && (top as f32..(top + tall) as f32).contains(&y)
}

/// Where to hang a label for the notice board.
pub fn board_label() -> (f32, f32) {
    let (left, top, wide, _) = BOARD;
    ((left + wide / 2) as f32, top as f32 - 6.0)
}

pub struct Clubhouse {
    ground: Playground,
    hilltop: Arrangement,
    pinned: Vec<bool>,
    /// The room once for each flicker of the fire, and which is on show.
    flickers: Vec<Canvas>,
    shown: usize,
    /// The hour the window was last painted for.
    outside: Daylight,
}

impl Clubhouse {
    /// The room with the colony coming in, the Hilltop through the window, and a card on the
    /// board for each story, `true` where it has been finished.
    pub fn open(cast: &Cast, now: f32, hilltop: &Arrangement, pinned: Vec<bool>) -> Self {
        let outside = Daylight::default();
        let flickers = paint(hilltop, &pinned, outside);
        let mut ground = Playground::new(
            cast,
            now,
            layout(),
            flickers[0].clone(),
            scenery::foreground(),
            scenery::props(),
        );
        ground.set_attractions(attractions());
        ground.set_nightlights(Nightlights {
            lamps: scenery::lamplight(),
            sky: Canvas::new(1, 1),
            indoors: true,
        });
        Self {
            ground,
            hilltop: hilltop.clone(),
            pinned,
            flickers,
            shown: 0,
            outside,
        }
    }

    /// The hour: the room dims only to lamplight, and the window shows the Hill outside as it is
    /// now, repainted whenever the light out there has noticeably changed.
    pub fn set_daylight(&mut self, daylight: Daylight) {
        self.ground.set_daylight(daylight);
        let look = |day: Daylight| {
            let [r, g, b] = day.ambient();
            [r, g, b, day.stars(), day.lamps()].map(|value| (value * 40.0).round() as i32)
        };
        if look(daylight) != look(self.outside) {
            self.outside = daylight;
            self.repaint();
        }
    }

    pub fn ground(&mut self) -> &mut Playground {
        &mut self.ground
    }

    /// Shows what stands on the Hilltop now, through the window.
    pub fn show_hilltop(&mut self, hilltop: &Arrangement) {
        if *hilltop != self.hilltop {
            self.hilltop = hilltop.clone();
            self.repaint();
        }
    }

    /// Pins up the stories again, as when one is finished.
    pub fn pin_up(&mut self, pinned: Vec<bool>) {
        if pinned != self.pinned {
            self.pinned = pinned;
            self.repaint();
        }
    }

    fn repaint(&mut self) {
        self.flickers = paint(&self.hilltop, &self.pinned, self.outside);
        self.ground.set_backdrop(self.flickers[self.shown].clone());
    }

    /// Everyone carries on, and the fire flickers; with motion reduced it burns steady.
    pub fn tick(&mut self, cast: &Cast, now: f32) {
        self.ground.tick(cast, now);
        let flicker = if self.ground.reduce_motion() {
            0
        } else {
            (now * FLICKER_RATE) as usize % FLICKERS
        };
        if flicker != self.shown {
            self.shown = flicker;
            self.ground.set_backdrop(self.flickers[flicker].clone());
        }
    }

    pub fn compose(&mut self, now: f32) -> Canvas {
        self.ground.compose(now)
    }
}

fn paint(hilltop: &Arrangement, pinned: &[bool], outside: Daylight) -> Vec<Canvas> {
    let room = scenery::room(hilltop, pinned, outside);
    (0..FLICKERS)
        .map(|flicker| scenery::with_fire(&room, flicker))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::script::PLACES;

    #[test]
    fn every_place_a_story_can_name_is_in_the_clubhouse() {
        for place in PLACES {
            assert!(
                SPOTS.iter().any(|(name, _)| *name == place),
                "no {place} in the clubhouse"
            );
        }
        for (name, (x, y)) in SPOTS {
            assert!(walkable(x, y), "{name} is somewhere nobody can stand");
        }
    }

    #[test]
    fn whatever_is_worth_going_over_to_can_be_reached() {
        for attraction in attractions() {
            let (x, y) = attraction.stand;
            assert!(walkable(x, y), "nobody can stand at {x}, {y}");
        }
        let (left, top, right, bottom) = SEATS;
        for (x, y) in [(left, top), (right, bottom), (left, bottom), (right, top)] {
            assert!(
                walkable(x, y),
                "the rug runs under the furniture at {x}, {y}"
            );
        }
    }

    #[test]
    fn the_fire_flickers_unless_motion_is_reduced() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut room = Clubhouse::open(&cast, 0.0, &Arrangement::new(), vec![false, true]);
        let mut seen = Vec::new();
        let mut now = 0.0;
        while now < 2.0 {
            now += 1.0 / 30.0;
            room.tick(&cast, now);
            if !seen.contains(&room.shown) {
                seen.push(room.shown);
            }
        }
        assert_eq!(seen.len(), FLICKERS);
        assert_ne!(room.flickers[0], room.flickers[1]);
    }

    #[test]
    fn a_finished_story_gets_a_star_on_its_card() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut room = Clubhouse::open(&cast, 0.0, &Arrangement::new(), vec![false]);
        let before = room.flickers[0].clone();
        room.pin_up(vec![true]);
        assert_ne!(before, room.flickers[0]);
    }
}
