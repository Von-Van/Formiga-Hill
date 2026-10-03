//! Hill's own pictures of the souvenirs Formiga Desktop doesn't draw yet. Desktop draws the ones
//! it knows (`formiga_art::draw_souvenir`), the same in both apps; these are drawn the same way,
//! seven pixels square in each souvenir's own fixed colours, rows of letters naming four inks
//! (`#`, `o`, `*` and `x`, `.` left clear), so that Desktop can take them over row for row. Once
//! Desktop knows a souvenir, its picture here comes out: a test says so.

use formiga_art::{Canvas, Rgba, SOUVENIR_ICON};

/// Draws the souvenir `id` with its top-left at `(x, y)`, if Hill has a picture of its own for
/// it. Says whether it did.
pub fn draw(canvas: &mut Canvas, id: &str, x: i32, y: i32) -> bool {
    let Some((rows, inks)) = picture(id) else {
        return false;
    };
    for (dy, row) in rows.iter().enumerate() {
        for (dx, code) in row.bytes().enumerate() {
            let ink = match code {
                b'#' => inks[0],
                b'o' => inks[1],
                b'*' => inks[2],
                b'x' => inks[3],
                _ => continue,
            };
            canvas.set(x + dx as i32, y + dy as i32, rgb(ink));
        }
    }
    true
}

const fn rgb(hex: u32) -> Rgba {
    Rgba::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255)
}

/// One souvenir's rows, and its four inks.
fn picture(id: &str) -> Option<([&'static str; SOUVENIR_ICON as usize], [u32; 4])> {
    Some(match id {
        // A rosette of pleated blue ribbon round a gilt button, its two tails hanging below.
        super::RACE_ROSETTE => (
            [
                ".#o#o#.", "#o*xxo#", "ooxxxoo", "#oxxxo#", ".#o#o#.", ".#o.o#.", ".#...#.",
            ],
            [0x1e2c58, 0x5a7ec4, 0xfff2bc, 0xe4c06c],
        ),
        // A little brass bell, lit from the left, its clapper showing under its rim.
        super::STRIKER_BELL => (
            [
                "...#...", "..#*#..", ".#*ox#.", ".#*ox#.", "#*ooox#", "#######", "...x...",
            ],
            [0x6b4a24, 0xd8b058, 0xfff0b8, 0x9a7434],
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_art::draw_souvenir;
    use formiga_core::Souvenir;

    /// The souvenirs Hill has pictures of.
    fn ids() -> [&'static str; 2] {
        [super::super::RACE_ROSETTE, super::super::STRIKER_BELL]
    }

    fn pixels_of(id: &str) -> Canvas {
        let mut canvas = Canvas::new(SOUVENIR_ICON, SOUVENIR_ICON);
        match Souvenir::from_id(id) {
            Some(souvenir) => draw_souvenir(&mut canvas, souvenir, 0, 0),
            None => assert!(draw(&mut canvas, id, 0, 0), "nobody draws {id}"),
        }
        canvas
    }

    #[test]
    fn every_souvenir_is_drawn_by_desktop_or_by_hills_own_pictures() {
        let mut seen: Vec<Canvas> = Vec::new();
        for id in super::super::ids() {
            let canvas = pixels_of(id);
            let drawn = canvas.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(drawn >= 12, "{id} is barely there");
            assert!(
                canvas
                    .pixels()
                    .iter()
                    .all(|pixel| pixel.a == 0 || pixel.a == 255),
                "{id} has half-clear pixels"
            );
            assert!(!seen.contains(&canvas), "{id} looks like another");
            seen.push(canvas);
        }
    }

    /// Desktop draws whatever it knows, the same in both apps: a picture here is only for a
    /// souvenir Desktop doesn't know yet, and comes out once it does.
    #[test]
    fn hills_pictures_are_only_for_souvenirs_desktop_does_not_know() {
        for id in ids() {
            assert_eq!(
                Souvenir::from_id(id),
                None,
                "Desktop draws {id} now: take Hill's picture of it out"
            );
            assert!(super::super::name(id).is_some(), "{id} is in no catalogue");
        }
        let mut canvas = Canvas::new(SOUVENIR_ICON, SOUVENIR_ICON);
        assert!(!draw(&mut canvas, "picnic_ribbon", 0, 0));
        assert!(!draw(&mut canvas, "no_such_thing", 0, 0));
    }
}
