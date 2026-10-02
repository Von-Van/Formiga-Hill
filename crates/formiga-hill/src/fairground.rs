//! The Fairground, down the lane from the green: free play among the stalls, and games the colony
//! plays among themselves while the person watches. (Games the person plays, for finds to take
//! home, belong to the Woods.) The first is hide-and-seek.

mod hide_and_seek;
mod scenery;

use crate::cast::Cast;
use crate::playground::{Layout, Patch, Playground};
use scenery::GROUND;
use std::sync::LazyLock;

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
pub use hide_and_seek::{Event, HideAndSeek, Phase};

const SPOTS: [(&str, (f32, f32)); 4] = [
    ("centre", (200.0, 152.0)),
    ("front", (192.0, 198.0)),
    ("left", (56.0, 176.0)),
    ("right", (350.0, 190.0)),
];

/// The ground just behind each prop, where whoever stands is hidden: free play keeps out of it,
/// so nobody vanishes unless they mean to.
static FOOTPRINTS: LazyLock<Vec<Patch>> = LazyLock::new(|| {
    scenery::props()
        .0
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
        seats: None,
        shade: None,
        // Up the lane from the green, in at the front right.
        entrance: (400.0, 196.0),
        entrance_step: (18.0, 5.0),
    }
}

/// The fairground with the colony walking in, and the game ready to play.
pub fn open(cast: &Cast, now: f32) -> (Playground, HideAndSeek) {
    let (props, places) = scenery::props();
    let ground = Playground::new(
        cast,
        now,
        layout(),
        scenery::backdrop(),
        scenery::foreground(),
        props,
    );
    (ground, HideAndSeek::new(places))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spots_and_the_way_in_are_clear_of_the_props() {
        for (name, (x, y)) in SPOTS {
            assert!(walkable(x, y), "{name} is behind something");
        }
        let (props, places) = scenery::props();
        for (prop, place) in props.iter().zip(&places) {
            let (x, y) = place.behind;
            assert!(
                !walkable(x, y),
                "free play would wander behind {}",
                place.name
            );
            assert!(prop.base < GROUND.3);
        }
    }

    #[test]
    fn free_play_never_hides_anyone() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, _) = open(&cast, 0.0);
        // Walking past behind something is fine; stopping there is not.
        let mut hidden_for = std::collections::HashMap::new();
        let mut now = 0.0;
        while now < 90.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            for id in ground.ids() {
                let (x, y) = ground.position(id).unwrap();
                let hidden = FOOTPRINTS
                    .iter()
                    .any(|&(l, t, r, b)| x > l + 6.0 && x < r - 6.0 && y > t + 4.0 && y < b - 3.0);
                let seconds = hidden_for.entry(id).or_insert(0.0);
                *seconds = if hidden { *seconds + 1.0 / 30.0 } else { 0.0 };
                assert!(*seconds < 3.0, "{id} stopped behind a prop at {now}s");
            }
        }
    }
}
