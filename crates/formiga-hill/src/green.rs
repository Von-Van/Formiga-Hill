//! The Village Green: the first place the colony goes from the station, for free play. (Stories
//! are staged indoors, in the Clubhouse.) The playground engine does the playing; this is the
//! ground it is played on.

mod scenery;

use crate::cast::Cast;
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Playground};
use scenery::{BLANKET, SHADE, WALK_BOTTOM, WALK_LEFT, WALK_RIGHT, WALK_TOP, walkable};

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};

/// The green's named spots, for anyone sent somewhere in particular.
const SPOTS: [(&str, (f32, f32)); 10] = [
    ("blanket", (266.0, 166.0)),
    ("well", (232.0, 102.0)),
    ("oak", (108.0, 128.0)),
    ("swing", (124.0, 120.0)),
    ("chest", (336.0, 104.0)),
    ("left", (60.0, 176.0)),
    ("right", (330.0, 180.0)),
    ("front", (192.0, 196.0)),
    ("back", (176.0, 104.0)),
    ("centre", (192.0, 150.0)),
];

pub fn layout() -> Layout {
    let (back, front, (back_left, back_right), _) = BLANKET;
    Layout {
        walkable,
        ground: (WALK_LEFT, WALK_TOP, WALK_RIGHT, WALK_BOTTOM),
        spots: &SPOTS,
        seats: Some((
            back_left as f32 + 4.0,
            back as f32 + 8.0,
            back_right as f32 - 4.0,
            front as f32 - 2.0,
        )),
        shade: Some(SHADE),
        // In along the gravel path from the lane at the bottom corner.
        entrance: (396.0, 206.0),
        entrance_step: (18.0, 4.0),
    }
}

/// Shows what stands on the Hilltop, far off through the gap in the trees.
pub fn show_hilltop(ground: &mut Playground, hilltop: &Arrangement) {
    ground.set_backdrop(scenery::backdrop(hilltop));
}

/// The green, with the colony walking in.
pub fn open(cast: &Cast, now: f32) -> Playground {
    Playground::new(
        cast,
        now,
        layout(),
        scenery::backdrop(&Arrangement::new()),
        scenery::foreground(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::blit;

    #[test]
    fn every_named_spot_is_somewhere_to_stand() {
        for (name, (x, y)) in SPOTS {
            assert!(walkable(x, y), "{name} is somewhere nobody can stand");
        }
    }

    #[test]
    fn composing_puts_the_colony_on_the_green() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut green = open(&cast, 0.0);
        let mut now = 0.0;
        while now < 15.0 {
            now += 1.0 / 30.0;
            green.tick(&cast, now);
        }
        let scene = green.compose(now);
        let mut empty = scenery::backdrop(&Arrangement::new());
        blit(&mut empty, &scenery::foreground(), 0, 0);
        assert_ne!(scene, empty);
    }
}
