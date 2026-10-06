//! Lettering for words on paper: what is said, what a button does, whose name is on a tag. It is
//! the sign painters' capitals from `font`, with small letters, figures and punctuation beside
//! them, set to each letter's own width, so a line of what someone says reads as easily as print
//! and is still made of pixels like everything else at the Hill.
//!
//! A line is drawn from its top: capitals stand [`ASCENT`] pixels tall on the baseline, and the
//! tails of g, j, p, q and y hang [`DESCENT`] below it. Lines of a paragraph are [`LINE`] apart.
//! Anything without a letter of its own is drawn as its plain letter where it has one (é as e),
//! and otherwise as a small empty box, so nothing a story says is ever silently dropped.

use std::collections::HashMap;
use std::sync::OnceLock;

/// How tall a capital is, from the top of a line to its baseline.
pub const ASCENT: i32 = 7;
/// How far a tail hangs below the baseline.
pub const DESCENT: i32 = 2;
/// From the top of one line to the top of the next.
pub const LINE: i32 = ASCENT + DESCENT + 2;
/// The space between words, and between two letters.
const SPACE: i32 = 3;
const GAP: i32 = 1;

/// One letter: its rows from the top of the line, each a bit per column from the left.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Glyph {
    width: i32,
    rows: Vec<u16>,
}

impl Glyph {
    fn from_art(rows: &[&str]) -> Self {
        let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as i32;
        let rows = rows
            .iter()
            .map(|row| {
                row.chars()
                    .enumerate()
                    .filter(|(_, pixel)| *pixel == '#')
                    .fold(0_u16, |bits, (column, _)| bits | (1 << column))
            })
            .collect();
        Self { width, rows }
    }

    /// A sign painter's capital, trimmed to the columns it uses.
    fn from_capital(bits: [u8; 7]) -> Self {
        let used = bits.iter().fold(0_u8, |all, row| all | row);
        let left = (0..5)
            .find(|column| used & (0b10000 >> column) != 0)
            .unwrap_or(0);
        let right = (0..5)
            .rev()
            .find(|column| used & (0b10000 >> column) != 0)
            .unwrap_or(0);
        let rows = bits
            .iter()
            .map(|row| {
                (left..=right)
                    .filter(|column| row & (0b10000 >> column) != 0)
                    .fold(0_u16, |out, column| out | (1 << (column - left)))
            })
            .collect();
        Self {
            width: right - left + 1,
            rows,
        }
    }

    fn lit(&self, column: i32, row: usize) -> bool {
        self.rows
            .get(row)
            .is_some_and(|bits| bits & (1 << column) != 0)
    }
}

/// The small letters and everything else that is not a sign painter's capital or figure, drawn
/// with `#` for ink. Rows start at the top of the line; small letters start two rows down.
const ART: &[(char, &[&str])] = &[
    ('a', &["", "", ".###.", "....#", ".####", "#...#", ".####"]),
    (
        'b',
        &[
            "#....", "#....", "####.", "#...#", "#...#", "#...#", "####.",
        ],
    ),
    ('c', &["", "", ".###", "#...", "#...", "#...", ".###"]),
    (
        'd',
        &[
            "....#", "....#", ".####", "#...#", "#...#", "#...#", ".####",
        ],
    ),
    ('e', &["", "", ".###.", "#...#", "#####", "#....", ".###."]),
    (
        'f',
        &["..##", ".#..", ".#..", "###.", ".#..", ".#..", ".#.."],
    ),
    (
        'g',
        &[
            "", "", ".####", "#...#", "#...#", "#...#", ".####", "....#", ".###.",
        ],
    ),
    (
        'h',
        &[
            "#....", "#....", "#.##.", "##..#", "#...#", "#...#", "#...#",
        ],
    ),
    ('i', &["#", "", "#", "#", "#", "#", "#"]),
    (
        'j',
        &["..#", "", "..#", "..#", "..#", "..#", "..#", "#.#", ".#."],
    ),
    (
        'k',
        &["#...", "#...", "#..#", "#.#.", "##..", "#.#.", "#..#"],
    ),
    ('l', &["#.", "#.", "#.", "#.", "#.", "#.", ".#"]),
    ('m', &["", "", "##.#.", "#.#.#", "#.#.#", "#.#.#", "#.#.#"]),
    ('n', &["", "", "#.##.", "##..#", "#...#", "#...#", "#...#"]),
    ('o', &["", "", ".###.", "#...#", "#...#", "#...#", ".###."]),
    (
        'p',
        &[
            "", "", "####.", "#...#", "#...#", "#...#", "####.", "#....", "#....",
        ],
    ),
    (
        'q',
        &[
            "", "", ".####", "#...#", "#...#", "#...#", ".####", "....#", "....#",
        ],
    ),
    ('r', &["", "", "#.##", "##..", "#...", "#...", "#..."]),
    ('s', &["", "", ".###", "#...", ".##.", "...#", "###."]),
    ('t', &["", ".#..", "###.", ".#..", ".#..", ".#..", "..##"]),
    ('u', &["", "", "#...#", "#...#", "#...#", "#..##", ".##.#"]),
    ('v', &["", "", "#...#", "#...#", "#...#", ".#.#.", "..#.."]),
    ('w', &["", "", "#...#", "#...#", "#.#.#", "#.#.#", ".#.#."]),
    ('x', &["", "", "#...#", ".#.#.", "..#..", ".#.#.", "#...#"]),
    (
        'y',
        &[
            "", "", "#...#", "#...#", "#...#", "#...#", ".####", "....#", ".###.",
        ],
    ),
    ('z', &["", "", "#####", "...#.", "..#..", ".#...", "#####"]),
    ('.', &["", "", "", "", "", "", "#"]),
    (',', &["", "", "", "", "", "", ".#", "#."]),
    ('!', &["#", "#", "#", "#", "#", "", "#"]),
    ('\'', &["#", "#"]),
    ('\u{2019}', &["##", ".#", "#."]),
    ('\u{2018}', &[".#", "#.", "##"]),
    ('"', &["#.#", "#.#"]),
    ('\u{201c}', &[".#..#", "#..#.", "##.##"]),
    ('\u{201d}', &["##.##", ".#..#", "#..#."]),
    (':', &["", "", "", "#", "", "", "#"]),
    (';', &["", "", "", ".#", "", "", ".#", "#."]),
    ('-', &["", "", "", "", "###"]),
    ('\u{2013}', &["", "", "", "", "####"]),
    ('\u{2014}', &["", "", "", "", "#######"]),
    ('_', &["", "", "", "", "", "", "", "", "#####"]),
    ('(', &[".#", "#.", "#.", "#.", "#.", "#.", "#.", ".#"]),
    (')', &["#.", ".#", ".#", ".#", ".#", ".#", ".#", "#."]),
    ('[', &["##", "#.", "#.", "#.", "#.", "#.", "#.", "##"]),
    (']', &["##", ".#", ".#", ".#", ".#", ".#", ".#", "##"]),
    (
        '/',
        &[
            "....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#....",
        ],
    ),
    ('+', &["", "", "", ".#.", "###", ".#.", ""]),
    ('=', &["", "", "", "###", "", "###"]),
    ('<', &["", "", "..#", ".#.", "#..", ".#.", "..#"]),
    ('>', &["", "", "#..", ".#.", "..#", ".#.", "#.."]),
    (
        '%',
        &[
            "##..#", "##.#.", "...#.", "..#..", ".#...", ".#.##", "#..##",
        ],
    ),
    (
        '#',
        &[
            ".#.#.", ".#.#.", "#####", ".#.#.", "#####", ".#.#.", ".#.#.",
        ],
    ),
    ('*', &["", "#.#", ".#.", "#.#"]),
    ('|', &["#", "#", "#", "#", "#", "#", "#", "#", "#"]),
    ('~', &["", "", "", ".#.#", "#.#."]),
    ('\u{2026}', &["", "", "", "", "", "", "#.#.#"]),
    ('\u{b7}', &["", "", "", "", "#"]),
    ('\u{2022}', &["", "", "", "##", "##"]),
    ('\u{d7}', &["", "", "", "#.#", ".#.", "#.#"]),
    ('\u{b0}', &[".#.", "#.#", ".#."]),
    (
        '\u{2605}',
        &["", "..#..", ".###.", "#####", ".###.", ".#.#."],
    ),
    ('\u{25b8}', &["", "", "#..", "##.", "###", "##.", "#.."]),
    ('\u{25c2}', &["", "", "..#", ".##", "###", ".##", "..#"]),
    ('\u{25be}', &["", "", "", "#####", ".###.", "..#.."]),
    ('\u{25b4}', &["", "", "", "..#..", ".###.", "#####"]),
];

/// The empty box drawn for anything there is no letter for.
const MISSING: &[&str] = &["", "###", "#.#", "#.#", "#.#", "#.#", "###"];

fn glyphs() -> &'static HashMap<char, Glyph> {
    static GLYPHS: OnceLock<HashMap<char, Glyph>> = OnceLock::new();
    GLYPHS.get_or_init(|| {
        let mut glyphs: HashMap<char, Glyph> = ('A'..='Z')
            .chain('0'..='9')
            .chain(['?', '&'])
            .filter_map(|letter| Some((letter, Glyph::from_capital(crate::font::glyph(letter)?))))
            .collect();
        for (letter, art) in ART {
            glyphs.insert(*letter, Glyph::from_art(art));
        }
        glyphs.insert('\u{fffd}', Glyph::from_art(MISSING));
        glyphs
    })
}

/// The letter `letter` is drawn as: itself, its plain letter, or the empty box.
fn glyph(letter: char) -> &'static Glyph {
    let glyphs = glyphs();
    glyphs
        .get(&letter)
        .or_else(|| glyphs.get(&plain(letter)))
        .unwrap_or(&glyphs[&'\u{fffd}'])
}

/// A letter without its accent, and a few look-alikes, so names and stories from anywhere read.
fn plain(letter: char) -> char {
    match letter {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => 'A',
        'ç' => 'c',
        'Ç' => 'C',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'È' | 'É' | 'Ê' | 'Ë' => 'E',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'Ì' | 'Í' | 'Î' | 'Ï' => 'I',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 'O',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
        'ý' | 'ÿ' => 'y',
        'Ý' => 'Y',
        '\u{2032}' | '`' | '\u{b4}' => '\'',
        '\u{2033}' => '"',
        '\u{2212}' | '\u{2010}' | '\u{2011}' => '-',
        '\u{2606}' => '\u{2605}',
        '\u{25b6}' | '\u{25ba}' => '\u{25b8}',
        '\u{25c0}' | '\u{25c4}' => '\u{25c2}',
        '\u{a0}' => ' ',
        other => other,
    }
}

/// How wide `text` is drawn on one line, in pixels.
pub fn width(text: &str) -> i32 {
    let letters = text.chars().map(|letter| {
        if letter.is_whitespace() {
            SPACE
        } else {
            glyph(letter).width
        }
    });
    let count = text.chars().count() as i32;
    letters.sum::<i32>() + GAP * (count - 1).max(0)
}

/// `text` broken into lines no wider than `most` pixels, at spaces where it can be and inside a
/// word only when one word is wider than a whole line. A line break in `text` is kept.
pub fn wrap(text: &str, most: i32) -> Vec<String> {
    let most = most.max(width("W"));
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ').filter(|word| !word.is_empty()) {
            let joined = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if width(&joined) <= most {
                line = joined;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            // A word too long for any line is broken where it has to be.
            for letter in word.chars() {
                let longer = format!("{line}{letter}");
                if !line.is_empty() && width(&longer) > most {
                    lines.push(std::mem::take(&mut line));
                    line.push(letter);
                } else {
                    line = longer;
                }
            }
        }
        lines.push(line);
    }
    lines
}

/// The widest of `lines`.
pub fn widest(lines: &[String]) -> i32 {
    lines.iter().map(|line| width(line)).max().unwrap_or(0)
}

/// Every run of ink in `text` drawn from (0, 0), the top of its line: (x, y, length), each a row
/// of pixels side by side. Runs rather than single pixels, so a page of words is a few hundred
/// rectangles to draw rather than thousands.
pub fn runs(text: &str) -> Vec<(i32, i32, i32)> {
    let mut runs = Vec::new();
    let mut left = 0;
    for (index, letter) in text.chars().enumerate() {
        if index > 0 {
            left += GAP;
        }
        if letter.is_whitespace() {
            left += SPACE;
            continue;
        }
        let glyph = glyph(letter);
        for row in 0..glyph.rows.len() {
            let mut column = 0;
            while column < glyph.width {
                if !glyph.lit(column, row) {
                    column += 1;
                    continue;
                }
                let start = column;
                while column < glyph.width && glyph.lit(column, row) {
                    column += 1;
                }
                runs.push((left + start, row as i32, column - start));
            }
        }
        left += glyph.width;
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;
    use formiga_art::{Canvas, Rgba};

    /// Draws `text` onto `canvas` with the top of its line at `(x, y)`, as the window paints it.
    fn draw(canvas: &mut Canvas, x: i32, y: i32, text: &str, color: Rgba) {
        for (left, top, length) in runs(text) {
            canvas.fill_rect(x + left, y + top, length, 1, color);
        }
    }

    #[test]
    fn every_letter_figure_and_mark_a_story_uses_has_a_glyph_of_its_own() {
        let alphabet = ('a'..='z').chain('A'..='Z').chain('0'..='9');
        let marks = ".,!?'\u{2019}\u{2018}\"\u{201c}\u{201d}:;-\u{2013}\u{2014}()/&\u{2026}\u{b7}\u{d7}\u{2605}\u{25b8}";
        for letter in alphabet.chain(marks.chars()) {
            assert!(glyphs().contains_key(&letter), "no glyph for {letter:?}");
        }
    }

    #[test]
    fn small_letters_sit_on_the_baseline_and_only_tails_hang_below_it() {
        for letter in 'a'..='z' {
            let glyph = glyph(letter);
            let bottom = glyph.rows.iter().rposition(|row| *row != 0).unwrap() as i32;
            let tailed = "gjpqy".contains(letter);
            let expected = if tailed {
                ASCENT + DESCENT - 1
            } else {
                ASCENT - 1
            };
            assert_eq!(bottom, expected, "{letter} ends on the wrong row");
            assert!(glyph.rows.len() as i32 <= ASCENT + DESCENT);
        }
    }

    #[test]
    fn an_accent_falls_back_to_its_plain_letter_and_anything_else_to_a_box() {
        assert_eq!(glyph('é'), glyph('e'));
        assert_eq!(glyph('Ö'), glyph('O'));
        assert_eq!(glyph('\u{4e2d}'), &Glyph::from_art(MISSING));
    }

    #[test]
    fn lettering_lands_where_it_is_measured() {
        let text = "Hello, Pip!";
        let mut canvas = Canvas::new(80, LINE as u32);
        draw(&mut canvas, 0, 0, text, Rgba::new(0, 0, 0, 255));
        let (left, top, right, bottom) = canvas.alpha_bounds().unwrap();
        assert_eq!((left, top), (0, 0));
        assert_eq!(right as i32 + 1, width(text));
        assert_eq!(bottom as i32 + 1, ASCENT + DESCENT, "the p hangs its tail");
    }

    #[test]
    fn words_wrap_at_spaces_and_no_line_is_too_wide() {
        let text = "The bun was gone, and only a trail of crumbs led away from the plate.";
        let lines = wrap(text, 90);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| width(line) <= 90), "{lines:?}");
        assert_eq!(lines.join(" "), text);
        let long = wrap("Supercalifragilistic", 30);
        assert!(long.len() > 1 && long.iter().all(|line| width(line) <= 30));
        assert_eq!(long.concat(), "Supercalifragilistic");
        assert_eq!(wrap("One\nTwo", 200), ["One", "Two"]);
    }
}
