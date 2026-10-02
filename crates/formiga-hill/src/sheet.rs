//! Review sheets: pictures for checking by eye what tests can only count. Numbers say a reaction
//! happened; only a sheet says whether a pose reads.

use crate::actor::{Actor, Step};
use crate::cast::Cast;
use crate::character::Offer;
use crate::cues::draw_cue;
use crate::dice::Dice;
use crate::paint::{hline, mix, rect, rgb};
use formiga_art::Canvas;

const CELL: (i32, i32) = (64, 60);
/// Each column: what was offered, and how far into the answer the picture is taken.
const MOMENTS: [(Offer, f32); 8] = [
    (Offer::Pet, 0.25),
    (Offer::Pet, 1.0),
    (Offer::Pet, 1.9),
    (Offer::Snack, 0.6),
    (Offer::Snack, 2.4),
    (Offer::Toy, 0.4),
    (Offer::Toy, 1.8),
    (Offer::Toy, 3.6),
];

/// Every traveller, one row each, answering a pat, a snack and a toy, a few moments into each.
pub fn reactions(cast: &Cast) -> Canvas {
    let (width, height) = (
        CELL.0 * MOMENTS.len() as i32,
        CELL.1 * cast.members.len() as i32,
    );
    let mut sheet = Canvas::new(width as u32, height as u32);
    for (row, member) in cast.members.iter().enumerate() {
        for (column, &(offer, at)) in MOMENTS.iter().enumerate() {
            let (left, top) = (column as i32 * CELL.0, row as i32 * CELL.1);
            let lawn = if (row + column) % 2 == 0 {
                rgb(0x8cc178)
            } else {
                rgb(0x7db36c)
            };
            rect(&mut sheet, left, top, CELL.0, CELL.1, lawn);
            hline(
                &mut sheet,
                left,
                top + CELL.1 - 1,
                CELL.0,
                mix(lawn, rgb(0x000000), 0.2),
            );

            let feet = ((left + CELL.0 / 2) as f32, (top + CELL.1 - 8) as f32);
            let mut actor = Actor::new(member, feet, true, cast.reduce_motion());
            let mut dice = Dice::new(member.id);
            let beats = actor.character.react(offer, 0.0, 0, &mut dice);
            actor.begin(0.0, beats.into_iter().map(Step::Beat));
            let mut now = 0.0;
            while now < at {
                now += 1.0 / 30.0;
                actor.advance(now, 1.0 / 30.0, &[]);
            }
            actor.draw_shadow(&mut sheet, now);
            actor.draw(&mut sheet, now);
            if let Some(cue) = actor.current_beat().and_then(|beat| beat.cue) {
                let (l, t, r, _) = actor.bounds(now);
                draw_cue(
                    &mut sheet,
                    cue,
                    (l + r) / 2,
                    t - 2,
                    actor.step_elapsed(now),
                    false,
                );
            }
        }
    }
    sheet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sheet_has_a_cell_for_everyone_and_every_moment() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let sheet = reactions(&cast);
        assert_eq!(sheet.width() as i32, CELL.0 * MOMENTS.len() as i32);
        assert_eq!(sheet.height() as i32, CELL.1 * cast.members.len() as i32);
    }
}
