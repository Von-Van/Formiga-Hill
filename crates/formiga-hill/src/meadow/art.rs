//! How each bug looks in the meadow, held up in the net, and as an icon in the journal. A
//! placeholder until each is drawn: a blob in the bug's own colours.

use super::bugs::bug;
use crate::paint::{ellipse, rgb};
use formiga_art::Canvas;

/// Icons are this many pixels square, as finds' and fish's are.
pub const ICON: u32 = 9;

/// What a bug is doing, which decides how it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    /// In the air, wings beating.
    Flying,
    /// Settled on a perch. For butterflies and moths, frame 0 has the wings open and frame 1
    /// shut; for a hopper, frame 1 is mid-hop.
    Settled,
    /// Walking along a perch, legs going.
    Crawling,
}

/// How many frames a bug has for a pose; the mechanics cycle through them.
pub fn frames(id: &str, pose: Pose) -> usize {
    let _ = (id, pose);
    2
}

/// A bug as it is seen in the meadow, facing right, at `frame` of its pose.
pub fn sprite(id: &str, pose: Pose, frame: usize) -> Canvas {
    let _ = (pose, frame);
    let size = bug(id).map_or(6, |bug| (bug.size.1 / 8.0).clamp(3.0, 14.0) as i32);
    let mut canvas = Canvas::new((size + 2) as u32, (size + 2) as u32);
    ellipse(
        &mut canvas,
        size / 2 + 1,
        size / 2 + 1,
        size / 2,
        size / 3,
        rgb(0x3a3550),
    );
    canvas
}

/// Where in a sprite the bug itself is: the point that sits on a perch, or follows its path.
pub fn anchor(id: &str, pose: Pose) -> (i32, i32) {
    let sprite = sprite(id, pose, 0);
    (sprite.width() as i32 / 2, sprite.height() as i32 - 1)
}

/// The bug close up, as it is held up in the net to be looked at before it is let go.
pub fn close_up(id: &str) -> Canvas {
    let mut canvas = Canvas::new(24, 24);
    let _ = id;
    ellipse(&mut canvas, 12, 12, 9, 6, rgb(0x3a3550));
    canvas
}

/// The bug's icon for the journal.
pub fn icon(id: &str) -> Canvas {
    let mut canvas = Canvas::new(ICON, ICON);
    let _ = id;
    ellipse(&mut canvas, 4, 4, 3, 2, rgb(0x3a3550));
    canvas
}
