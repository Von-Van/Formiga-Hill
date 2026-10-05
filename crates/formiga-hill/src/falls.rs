//! The Far Falls, deep in the Woods, which only an expedition reaches: a tall fall of water out
//! of the crag under an old stone arch, into a plunge pool. Things wash down from wherever the
//! water has been, and the party wades at the pool's edge to catch them as the eddy brings them
//! round (see `wading`). What comes down here comes down nowhere else (see `finds::afar`).

mod scenery;
pub mod wading;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::playground::{Layout, Patch, Playground};

/// Where the water comes over the lip, high up under the arch, and where it meets the pool.
pub const LIP: (f32, f32) = (192.0, 40.0);
pub const FOOT: (f32, f32) = (192.0, 124.0);

/// The eddy the pool turns in: its middle and its reach across and back. Whatever comes down is
/// carried round it, coming past the wading stones at the front.
pub const EDDY: (f32, f32, f32, f32) = (192.0, 140.0, 74.0, 16.0);

/// Where whoever wades in first stands, on the wading stone at the pool's edge, and where a
/// second stands beside it, and anyone else watches from the shingle.
pub const WADE: (f32, f32) = (192.0, 196.0);
pub const BESIDE: (f32, f32) = (162.0, 200.0);
pub const WATCH: (f32, f32) = (238.0, 203.0);

/// Where anyone can stand: the stones and shingle along the front of the pool.
pub const SHORE: Patch = (20.0, 184.0, 364.0, 208.0);

fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = SHORE;
    x >= left && x <= right && y >= top && y <= bottom
}

const PLACES: [(&str, (f32, f32)); 2] = [("centre", (192.0, 192.0)), ("left", (70.0, 194.0))];

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: SHORE,
        spots: &PLACES,
        seats: None,
        shade: None,
        // Down the last of the path, out of the trees at the left.
        entrance: (-24.0, 194.0),
        entrance_step: (-20.0, 4.0),
    }
}

/// The falls, with the party come down the path to the pool. After dark, glow-worms in the
/// ferns, and the moon over the arch.
pub fn open(cast: &Cast, party: &[Id], now: f32) -> Playground {
    let backdrop = scenery::backdrop();
    let nightlights = Nightlights {
        lamps: scenery::lamplight(),
        sky: scenery::night_sky(&backdrop),
        indoors: false,
    };
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        layout(),
        backdrop,
        scenery::foreground(),
        Vec::new(),
    );
    ground.set_nightlights(nightlights);
    ground
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wading_stones_are_on_the_shore_and_the_eddy_comes_past_them() {
        for at in [WADE, BESIDE, WATCH] {
            assert!(walkable(at.0, at.1), "nobody can stand at {at:?}");
        }
        let (cx, cy, rx, ry) = EDDY;
        // The front of the eddy, where things come past, is out from the wading stone, and high
        // enough to be seen over the heads of whoever wades there.
        assert!((cx - WADE.0).abs() < 1.0);
        assert!(WADE.1 - (cy + ry) < 45.0 && WADE.1 - (cy + ry) > 34.0);
        assert!(rx > ry && FOOT.1 < cy && LIP.1 < FOOT.1);
    }
}
