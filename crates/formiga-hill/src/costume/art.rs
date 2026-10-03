//! How each costume piece looks, facing right and seen a little from the front and above, as
//! everything on the Green is. Drawn at the companions' own scale, as the areas are drawn: every
//! material shaded with a ramp, lit from the upper left, outlined in a darker shade of its own
//! colour (never black, so the companion's own near-black outline reads first), and straw, felt
//! and wool textured from `paint::noise`.
//!
//! Each piece is painted from rows of letters, one a pixel: a material's five tones are named by
//! five letters, darkest (its outline) first, and `.` is clear.

use crate::paint::{Ramp, chance, mix, put};
use formiga_art::{Canvas, Rgba};

/// Icons are this many pixels square.
pub const ICON: u32 = 9;

/// A piece facing right; mirrored for a companion facing left.
pub fn sprite(id: &str) -> Canvas {
    let Some(look) = look(id) else {
        return Canvas::new(1, 1);
    };
    let mut canvas = paint(look.sprite, look.inks);
    if let Some((ramps, salt)) = look.texture {
        texture(&mut canvas, ramps, salt);
    }
    canvas
}

/// The point on the piece that sits on the companion: on the crown for a hat, at the throat for
/// a neck piece.
pub fn anchor(id: &str) -> (i32, i32) {
    look(id).map_or((0, 0), |look| look.anchor)
}

/// The piece's icon for the dress-up box, nine pixels square.
pub fn icon(id: &str) -> Canvas {
    match look(id) {
        Some(look) => paint(&look.icon, look.inks),
        None => Canvas::new(ICON, ICON),
    }
}

/// Everything about how one piece is drawn.
struct Look {
    sprite: &'static [&'static str],
    icon: [&'static str; ICON as usize],
    /// Which letters name which material's tones: edge, shadow, base, light and shine.
    inks: &'static [(&'static str, Ramp)],
    anchor: (i32, i32),
    /// The materials with a grain to them, and the salt for it.
    texture: Option<(&'static [Ramp], u32)>,
}

fn look(id: &str) -> Option<Look> {
    Some(match id {
        "paper_crown" => PAPER_CROWN,
        "straw_hat" => STRAW_HAT,
        "top_hat" => TOP_HAT,
        "daisy_chain" => DAISY_CHAIN,
        "party_hat" => PARTY_HAT,
        "wizard_hat" => WIZARD_HAT,
        "bow_tie" => BOW_TIE,
        "knitted_scarf" => KNITTED_SCARF,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------------------------

/// Gold card, as a cracker's crown is cut from.
const GOLD: Ramp = Ramp::new(0x8a5a22, 0xc08a30, 0xe6b84c, 0xf6d878, 0xfff2bc);
/// A glass jewel, red.
const GEM: Ramp = Ramp::new(0x6a1424, 0x9a2034, 0xd8404a, 0xf07070, 0xffd0d0);
const STRAW: Ramp = Ramp::new(0x8c6a2c, 0xc29a4c, 0xdcbc6e, 0xeed492, 0xfaecc0);
/// A sun hat's ribbon.
const ROSE: Ramp = Ramp::new(0x8a3448, 0xc0546a, 0xe07a8a, 0xf2a2ae, 0xfcd4da);
/// The top hat's felt: plum so dark it reads as black, but its own colour all the same.
const FELT: Ramp = Ramp::new(0x3b2347, 0x4a3360, 0x5d4677, 0x7a6396, 0xa48fbe);
/// The top hat's band.
const WINE: Ramp = Ramp::new(0x6a1e2c, 0x9a2e3c, 0xc04654, 0xd86a72, 0xf0a0a0);
/// The party hat's card, and the stripes round it.
const CHERRY: Ramp = Ramp::new(0x8a2a48, 0xc0405e, 0xe8607a, 0xf4889a, 0xfcc0c8);
const LEMON: Ramp = Ramp::new(0xa07020, 0xe0b040, 0xf6d86a, 0xfff0a8, 0xfffbe0);
/// A pompom of tissue, white, its shade cool.
const TISSUE: Ramp = Ramp::new(0x9890a8, 0xc8c4d4, 0xeeeaf2, 0xfaf8fc, 0xffffff);
/// The wizard's hat, and the stars on it.
const STARRY: Ramp = Ramp::new(0x1f2a5e, 0x2c3d86, 0x3e56ab, 0x5b78c9, 0x8ba6e2);
const STAR: Ramp = Ramp::new(0x9a6a18, 0xd8a838, 0xf6d860, 0xfff0a0, 0xfffbe0);
/// The daisy chain: stems, petals and their yellow eyes.
const STEM: Ramp = Ramp::new(0x2c5a2c, 0x3f7a3a, 0x58964a, 0x76b25c, 0x9ccc7e);
const PETAL: Ramp = Ramp::new(0xa8a8b8, 0xd4d6e0, 0xf6f4ee, 0xfffdf8, 0xffffff);
const EYE: Ramp = Ramp::new(0xa0601c, 0xd89030, 0xf5c94e, 0xffe080, 0xfff4c0);
/// The bow tie's silk, and its spots.
const SILK: Ramp = Ramp::new(0x7a1e26, 0xa8303a, 0xd04a4e, 0xe87a72, 0xf8b0a0);
const SPOT: Ramp = Ramp::new(0xb8a8a0, 0xd8ccc4, 0xf6efe6, 0xfffaf4, 0xffffff);
/// The scarf's two wools.
const TEAL: Ramp = Ramp::new(0x1e5a5a, 0x2c7a74, 0x3f9a8e, 0x62b8a8, 0x96d6c4);
const MUSTARD: Ramp = Ramp::new(0x8a5a1a, 0xc08a2a, 0xe0ac3c, 0xf0c862, 0xfae09a);

/// The letters most pieces name their main material's tones by.
const MAIN: &str = "#sol*";

// ---------------------------------------------------------------------------------------------
// The pieces
// ---------------------------------------------------------------------------------------------

/// A gold paper crown: three points, and a red glass jewel on the band.
const PAPER_CROWN: Look = Look {
    sprite: &[
        ".#...#...#.",
        "#*#.#l#.#o#",
        "#lo#los#os#",
        "#*llloooss#",
        "#loowrross#",
        "#loorqqoss#",
        "#oooooosss#",
        ".#########.",
    ],
    icon: [
        ".........",
        ".#..#..#.",
        "#*##l##o#",
        "#l#los#s#",
        "#lloooss#",
        "#llwrrss#",
        "#lorqqss#",
        "#looooss#",
        ".#######.",
    ],
    inks: &[(MAIN, GOLD), ("..qrw", GEM)],
    anchor: (5, 6),
    texture: None,
};

/// A wide straw sun hat: a low crown with a rose ribbon round it, the brim's back showing either
/// side and its front coming down over the brow.
const STRAW_HAT: Look = Look {
    sprite: &[
        ".....#####.....",
        "....#*llos#....",
        "....#looss#....",
        "..##kprrrqk##..",
        ".#*lsssssssos#.",
        "#looooooooooos#",
        "#looooooooooss#",
        ".#lllooooosss#.",
        "...#########...",
    ],
    icon: [
        ".........",
        ".........",
        "...###...",
        "..#*lo#..",
        ".#kprqk#.",
        "#lssssos#",
        "#looooos#",
        ".#oooss#.",
        "..#####..",
    ],
    inks: &[(MAIN, STRAW), ("kqrp.", ROSE)],
    anchor: (7, 6),
    texture: Some((&[STRAW], 11)),
};

/// A tall top hat in dark plum felt, with a wine-red band and a brim curled up at the sides.
const TOP_HAT: Look = Look {
    sprite: &[
        "...#####...",
        "..#*llll#..",
        "..#sssss#..",
        "..#lloos#..",
        "..#lloos#..",
        "..#looos#..",
        "..#prrrq#..",
        "#.#rrrqq#.#",
        "#l#######s#",
        "#lloooooss#",
        ".#########.",
    ],
    icon: [
        "...###...",
        "..#*ll#..",
        "..#los#..",
        "..#los#..",
        "..#prq#..",
        "#.#rrq#.#",
        "#l#####s#",
        "#lloooss#",
        ".#######.",
    ],
    inks: &[(MAIN, FELT), (".qrp.", WINE)],
    anchor: (5, 9),
    texture: Some((&[FELT], 23)),
};

/// A ring of daisies on their stems: the two at the back in the shade of the head, three across
/// the front.
const DAISY_CHAIN: Look = Look {
    sprite: &[
        "...v.....v...",
        "..vYVgggvYV..",
        ".g.V.....V.g.",
        "g..w.....w..g",
        ".gwyvhwhwyvg.",
        "..hvhwyvhvh..",
        "......v......",
    ],
    icon: [
        ".........",
        "..v...v..",
        ".vYVgvYV.",
        "g.V...V.g",
        "g.w...w.g",
        ".wyvwwyv.",
        ".gvwyvvg.",
        "....v....",
        ".........",
    ],
    inks: &[("Gghl.", STEM), ("Vvw.*", PETAL), (".Yy..", EYE)],
    anchor: (6, 3),
    texture: None,
};

/// A cone of cherry card striped in lemon, with a tissue pompom on its tip.
const PARTY_HAT: Look = Look {
    sprite: &[
        "...PPP...",
        "..PQqqP..",
        "..PqqpP..",
        "...PpP...",
        "...#o#...",
        "...#o#...",
        "..#lzy#..",
        "..#Zzs#..",
        ".#Zzoos#.",
        ".#Zoooy#.",
        "#Zooozys#",
        "#loozzss#",
        ".#######.",
    ],
    icon: [
        "...PPP...",
        "...QqP...",
        "...#o#...",
        "...#z#...",
        "..#lzy#..",
        "..#Zos#..",
        ".#Zoozy#.",
        ".#lzzos#.",
        "..#####..",
    ],
    inks: &[("#sol.", CHERRY), (".yzZ.", LEMON), ("Ppq.Q", TISSUE)],
    anchor: (4, 11),
    texture: None,
};

/// A wizard's hat, starry blue, its tip too long to stand and bent over behind.
const WIZARD_HAT: Look = Look {
    sprite: &[
        "....###......",
        "..#*llo#.....",
        ".#l#.#ls#....",
        ".#...#ls#....",
        "....#los#....",
        "....#loXs#...",
        "....#loos#...",
        "...#llxos#...",
        "...#lxXxss#..",
        "...#llxoss#..",
        ".###llooss##.",
        "#ll########s#",
        "#lloooooosss#",
        ".###########.",
    ],
    icon: [
        "..####...",
        ".#*llo#..",
        "#.#.#s#..",
        "....#o#..",
        "...#lXs#.",
        "...#loos#",
        "#.#lloss#",
        "#l######s",
        ".#######.",
    ],
    inks: &[(MAIN, STARRY), ("..x.X", STAR)],
    anchor: (6, 12),
    texture: None,
};

/// A red silk bow tie with cream spots, its knot in the middle.
const BOW_TIE: Look = Look {
    sprite: &[
        "##.......##",
        "#l##...##o#",
        "#lwo###wos#",
        "#loo#l#oss#",
        "#oow###ows#",
        "#o##...##s#",
        "##.......##",
    ],
    icon: [
        ".........",
        ".........",
        "##.....##",
        "#lw###ws#",
        "#lo#l#os#",
        "#ow###ss#",
        "##.....##",
        ".........",
        ".........",
    ],
    inks: &[(MAIN, SILK), ("..w..", SPOT)],
    anchor: (5, 2),
    texture: None,
};

/// A knitted scarf striped teal and mustard, wound once round, one end thrown over the front and
/// hanging down, fringed.
const KNITTED_SCARF: Look = Look {
    sprite: &[
        ".NN######NN#.",
        "#uu#los#omms#",
        "#mm#los#snns#",
        ".NNNumnN#NN#.",
        "...NumnN.....",
        "..#los#......",
        "..#los#......",
        "..#####......",
        "..u.m.u......",
    ],
    icon: [
        ".........",
        ".N####NN.",
        "Nu#lo#mmN",
        "Nm#ls#nnN",
        ".NNunNNN.",
        "..NunN...",
        ".#lo#....",
        ".####....",
        ".u.u.....",
    ],
    inks: &[("#sol.", TEAL), ("Nnmu.", MUSTARD)],
    anchor: (6, 1),
    texture: Some((&[TEAL, MUSTARD], 37)),
};

// ---------------------------------------------------------------------------------------------
// Painting
// ---------------------------------------------------------------------------------------------

/// Rows of letters painted into a canvas their own size.
fn paint(rows: &[&str], inks: &[(&str, Ramp)]) -> Canvas {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut canvas = Canvas::new(width.max(1) as u32, rows.len().max(1) as u32);
    for (y, row) in rows.iter().enumerate() {
        for (x, letter) in row.bytes().enumerate() {
            if let Some(color) = ink(letter, inks) {
                put(&mut canvas, x as i32, y as i32, color);
            }
        }
    }
    canvas
}

/// The colour a letter names, if any: its place among a material's letters is the tone.
fn ink(letter: u8, inks: &[(&str, Ramp)]) -> Option<Rgba> {
    if letter == b'.' {
        return None;
    }
    inks.iter().find_map(|(letters, ramp)| {
        let tone = letters.bytes().position(|name| name == letter)?;
        Some([ramp.edge, ramp.shadow, ramp.base, ramp.light, ramp.shine][tone])
    })
}

/// Grain over a material's middle tones, from the hash: strands of straw, the nap of felt,
/// stitches of wool. Outlines and highlights are left as they are, so the shape still reads.
fn texture(canvas: &mut Canvas, ramps: &[Ramp], salt: u32) {
    for y in 0..canvas.height() as i32 {
        for x in 0..canvas.width() as i32 {
            let pixel = canvas.get(x, y);
            for ramp in ramps {
                let grain = if pixel == ramp.base {
                    if chance(x, y, salt, 52) {
                        mix(ramp.base, ramp.light, 0.6)
                    } else if chance(x, y, salt + 1, 44) {
                        mix(ramp.base, ramp.shadow, 0.6)
                    } else {
                        continue;
                    }
                } else if pixel == ramp.shadow && chance(x, y, salt + 2, 40) {
                    mix(ramp.shadow, ramp.edge, 0.35)
                } else {
                    continue;
                };
                canvas.set(x, y, grain);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::costume::{PIECES, Slot};

    #[test]
    fn every_piece_draws_something_round_its_anchor() {
        for piece in &PIECES {
            let sprite = sprite(piece.id);
            let drawn = sprite.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(drawn >= 30, "{} is barely there", piece.id);
            let (x, y) = anchor(piece.id);
            assert!(
                x >= 0 && y >= 0 && x < sprite.width() as i32 && y < sprite.height() as i32,
                "{}'s anchor is off its picture",
                piece.id
            );
        }
    }

    #[test]
    fn every_piece_is_drawn_to_scale() {
        for piece in &PIECES {
            let sprite = sprite(piece.id);
            let (width, height) = (sprite.width(), sprite.height());
            let (most_wide, most_tall) = match piece.slot {
                // A companion's head is twelve to sixteen pixels across.
                Slot::Crown => (16, 16),
                Slot::Neck => (14, 13),
            };
            assert!(
                width <= most_wide && height <= most_tall,
                "{} is {width}x{height}",
                piece.id
            );
            assert!(width >= 7, "{} is too narrow to make out", piece.id);
        }
    }

    #[test]
    fn a_hat_sits_on_the_head_rather_than_floating_over_it() {
        for piece in PIECES.iter().filter(|piece| piece.slot == Slot::Crown) {
            let sprite = sprite(piece.id);
            let (_, y) = anchor(piece.id);
            let bottom = sprite.alpha_bounds().expect("drawn").3 as i32;
            assert!(
                (1..=4).contains(&(bottom - y)),
                "{} reaches {} below its crown",
                piece.id,
                bottom - y
            );
        }
    }

    #[test]
    fn icons_are_nine_pixels_square_and_drawn() {
        for piece in &PIECES {
            let icon = icon(piece.id);
            assert_eq!((icon.width(), icon.height()), (ICON, ICON), "{}", piece.id);
            let drawn = icon.pixels().iter().filter(|pixel| pixel.a > 0).count();
            assert!(drawn >= 20, "{}'s icon is barely there", piece.id);
        }
    }

    #[test]
    fn every_letter_names_a_colour() {
        for piece in &PIECES {
            let look = look(piece.id).expect("every piece has a look");
            for rows in [look.sprite, &look.icon[..]] {
                let width = rows[0].len();
                for row in rows {
                    assert_eq!(row.len(), width, "{}: {row:?} is ragged", piece.id);
                    for letter in row.bytes().filter(|&letter| letter != b'.') {
                        assert!(
                            ink(letter, look.inks).is_some(),
                            "{}: {:?} names no colour",
                            piece.id,
                            letter as char
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn nothing_is_outlined_in_black() {
        for piece in &PIECES {
            for pixel in sprite(piece.id).pixels() {
                let darkest = pixel.r.max(pixel.g).max(pixel.b);
                assert!(
                    pixel.a == 0 || darkest >= 0x40,
                    "{} has a near-black pixel {pixel:?}",
                    piece.id
                );
            }
        }
    }

    #[test]
    fn a_piece_hill_does_not_know_draws_nothing() {
        assert!(sprite("ermine_cloak").alpha_bounds().is_none());
        assert!(icon("ermine_cloak").alpha_bounds().is_none());
    }
}
