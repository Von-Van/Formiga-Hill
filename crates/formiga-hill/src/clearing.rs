//! The clearing that isn't on any map: where, very rarely, something out of place is waiting in
//! the Woods. The one thing here is the Cursor Sovereign (see `sovereign`), played on a small
//! stage (see `stage`) that could one day carry other secret set pieces.

pub mod boss;
mod scenery;
pub mod sovereign;
pub mod stage;

use crate::cast::{Cast, Id};
use crate::font::{GLYPH_HEIGHT, draw_text, text_width};
use crate::paint::{blit, mix, put, rect, rgb, rgba};
use crate::playground::{Layout, Patch, Playground};
use formiga_art::{Canvas, Rgba};
use sovereign::{Sovereign, State};

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};

/// Where anyone can stand: the near half of the clearing, facing the Sovereign.
const GROUND: Patch = (24.0, 160.0, 236.0, 206.0);

fn walkable(x: f32, y: f32) -> bool {
    let (l, t, r, b) = GROUND;
    x >= l && x <= r && y >= t && y <= b
}

const PLACES: [(&str, (f32, f32)); 1] = [("centre", (130.0, 186.0))];

fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &PLACES,
        seats: None,
        shade: None,
        entrance: (-24.0, 194.0),
        entrance_step: (-18.0, 4.0),
    }
}

/// The clearing, with the companions who found it walking in.
pub fn open(cast: &Cast, party: &[Id], now: f32) -> Playground {
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        layout(),
        scenery::backdrop(),
        scenery::foreground(),
        Vec::new(),
    );
    for (id, mark) in party.iter().zip(sovereign::MARKS) {
        ground.direct(
            *id,
            vec![
                crate::actor::Step::Walk { to: mark },
                crate::actor::Step::FaceX(300.0),
            ],
            now,
        );
    }
    ground
}

/// The whole picture: the clearing and everyone in it, the Sovereign, its name and health, any
/// close-up, and the shaking.
pub fn compose(ground: &mut Playground, sovereign: &Sovereign, now: f32) -> Canvas {
    let reduce_motion = ground.reduce_motion();
    let mut scene = ground.compose(now);
    let stage = &sovereign.stage;
    let (step, since) = stage.boss;
    boss::draw(&mut scene, step, now - since, now, reduce_motion);
    if sovereign.state() != State::Over {
        title(&mut scene, stage.health, stage.bar);
    }
    if let Some((who, name, since)) = &stage.cutin {
        let pictures: Vec<Canvas> = who
            .iter()
            .filter_map(|id| ground.picture(*id, now))
            .collect();
        cutin(&mut scene, &pictures, name, now - since, reduce_motion);
    }
    if now < stage.shake_until && !reduce_motion {
        let (dx, dy) = (
            ((now * 61.0).sin() * 2.5) as i32,
            ((now * 47.0).cos() * 1.5) as i32,
        );
        let mut shaken = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        shaken.fill_rect(0, 0, SCENE_WIDTH as i32, SCENE_HEIGHT as i32, rgb(0x0c0a14));
        blit(&mut shaken, &scene, dx, dy);
        scene = shaken;
    }
    scene
}

/// Lettering `scale` times the size of the sign font, with a drop shadow.
fn big_text(scene: &mut Canvas, x: i32, y: i32, text: &str, scale: i32, color: Rgba, shadow: Rgba) {
    let width = text_width(text).max(1);
    let mut small = Canvas::new(width as u32 + 1, GLYPH_HEIGHT as u32 + 1);
    draw_text(&mut small, 1, 1, text, shadow);
    draw_text(&mut small, 0, 0, text, color);
    for sy in 0..small.height() as i32 {
        for sx in 0..small.width() as i32 {
            let pixel = small.get(sx, sy);
            if pixel.a > 0 {
                rect(scene, x + sx * scale, y + sy * scale, scale, scale, pixel);
            }
        }
    }
}

/// The Sovereign's name across the top, and its health beneath: the yellow trailing behind the
/// red as a blow lands.
fn title(scene: &mut Canvas, health: f32, bar: f32) {
    const NAME: &str = "THE CURSOR SOVEREIGN";
    let width = SCENE_WIDTH as i32;
    let name_width = text_width(NAME) * 2;
    big_text(
        scene,
        (width - name_width) / 2,
        6,
        NAME,
        2,
        rgb(0xf6eed8),
        rgb(0x3a1440),
    );
    let (left, top, long) = (60, 24, width - 120);
    rect(scene, left - 2, top - 2, long + 4, 9, rgba(0x0c0a14, 220));
    scene.fill_rect(left, top, long, 5, rgb(0x2a1830));
    scene.fill_rect(left, top, (long as f32 * bar) as i32, 5, rgb(0xf5d25e));
    scene.fill_rect(left, top, (long as f32 * health) as i32, 5, rgb(0xd03a48));
    scene.fill_rect(left, top, (long as f32 * health) as i32, 1, rgb(0xf07080));
}

/// How tall the close-up's band is, and where its top edge sits when fully open.
const CUTIN_TOP: i32 = 62;
const CUTIN_HEIGHT: i32 = 70;
/// The gold that edges the band and fills the attack's name.
const GOLD: u32 = 0xf5d25e;

/// A close-up across the screen, as the attack is called: a band that snaps open with a flash,
/// speed lines streaming through it, whoever is acting framed head and shoulders and sliding in
/// side by side from the left, and the attack's name slammed in from the right, flaring as it
/// lands. The name sits on the band beside whoever is acting if there is room; when there isn't,
/// they take the whole band and the name breaks out over its top edge. With reduced motion it is
/// simply there, still.
fn cutin(scene: &mut Canvas, pictures: &[Canvas], name: &str, t: f32, reduce_motion: bool) {
    let width = SCENE_WIDTH as i32;
    let after = |delay: f32, secs: f32| {
        if reduce_motion {
            1.0
        } else {
            ease_out((t - delay) / secs)
        }
    };
    let mut band = Canvas::new(SCENE_WIDTH, CUTIN_HEIGHT as u32);
    // Dark at the edges and lighter through the middle, with a burst of light behind the faces:
    // rings of it, building up towards their middle.
    const BURST: [(i32, i32); 4] = [(130, 40), (104, 33), (78, 26), (52, 19)];
    let middle = (60, CUTIN_HEIGHT / 2);
    for y in 0..CUTIN_HEIGHT {
        let edge = (y as f32 / CUTIN_HEIGHT as f32 - 0.5).abs() * 2.0;
        let row = mix(rgb(0x3a1e68), rgb(0x120a24), edge);
        let lit: [Rgba; 5] = std::array::from_fn(|rings| {
            mix(row, rgb(0x7a46c0), 1.0 - 0.914_f32.powi(rings as i32))
        });
        band.fill_rect(0, y, width, 1, row);
        let dy = (y - middle.1) as f32;
        for (ring, &(rx, ry)) in BURST.iter().enumerate() {
            let across = 1.0 - (dy / ry as f32).powi(2);
            if across >= 0.0 {
                let half = (rx as f32 * across.sqrt()) as i32;
                band.fill_rect(middle.0 - half, y, half * 2 + 1, 1, lit[ring + 1]);
            }
        }
    }
    for streak in 0..30 {
        let y = 4 + (streak * 37) % (CUTIN_HEIGHT - 8);
        let length = 30 + (streak * 53) % 90;
        let x = if reduce_motion {
            (streak * 71) % width
        } else {
            (((streak * 71) as f32 + t * 600.0) as i32) % (width + length) - length
        };
        let (thick, alpha) = if streak % 7 == 0 { (2, 90) } else { (1, 60) };
        rect(&mut band, x, y, length, thick, rgba(0xc8b0ff, alpha));
    }
    let (scale, lines) = fit_name(name, width * 5 / 8);
    let lettered = |line: &&str| text_width(line) * scale;
    let block = lines.iter().map(lettered).max().unwrap_or(0);
    let line_height = GLYPH_HEIGHT * scale + 4;
    let block_height = line_height * lines.len() as i32 - 4;
    let group = framing(pictures);
    let beside = width - block - 26;
    let crowded = group_width(&group, 0) > beside;
    portraits(
        &mut band,
        &group,
        if crowded { width - 8 } else { beside },
        t,
        reduce_motion,
    );
    // The band snaps open from its middle, flashing as it does.
    let open = after(0.0, 0.12);
    let half = (CUTIN_HEIGHT as f32 / 2.0 * open).round() as i32;
    let middle = CUTIN_TOP + CUTIN_HEIGHT / 2;
    let flash = if reduce_motion {
        0.0
    } else {
        1.0 - (t / 0.18).clamp(0.0, 1.0)
    };
    for y in middle - half..middle + half {
        for x in 0..width {
            let mut pixel = band.get(x, y - CUTIN_TOP);
            if flash > 0.0 {
                pixel = mix(pixel, rgb(0xfff8ff), flash * flash);
            }
            scene.set(x, y, pixel);
        }
    }
    for (y, shade) in [
        (middle - half - 1, 0xfff0a8),
        (middle - half, GOLD),
        (middle - half + 1, 0xb08a2a),
        (middle + half - 2, 0xb08a2a),
        (middle + half - 1, GOLD),
        (middle + half, 0xfff0a8),
    ] {
        scene.fill_rect(0, y, width, 1, rgb(shade));
    }
    // The name, on a dark plate slanting in from the right.
    let shift = ((1.0 - after(0.1, 0.25)) * (block + 60) as f32) as i32;
    let top = if crowded {
        // Mostly above the band, over the tops of heads rather than faces, and clear of the
        // stage and the Sovereign's name.
        CUTIN_TOP - block_height + 8
    } else {
        CUTIN_TOP + (CUTIN_HEIGHT - block_height) / 2
    };
    let (plate_top, plate_height) = (top - 5, block_height + 10);
    let plate_left = width - block - 24 + shift;
    for row in 0..plate_height {
        let slant = (plate_height - row) / 2;
        let y = plate_top + row;
        let from = plate_left + slant;
        rect(scene, from, y, width - from, 1, rgba(0x0a0414, 170));
        put(scene, plate_left + slant, y, rgba(0xe070d0, 220));
        put(scene, plate_left + slant + 1, y, rgba(0xe070d0, 90));
    }
    let flare = if reduce_motion || t < 0.3 {
        0.0
    } else {
        1.0 - ((t - 0.3) / 0.25).clamp(0.0, 1.0)
    };
    for (index, line) in lines.iter().enumerate() {
        let x = width - 10 - lettered(line) + shift;
        lettering(
            scene,
            (x, top + index as i32 * line_height),
            line,
            scale,
            flare,
        );
    }
}

/// `u` eased so it arrives fast and settles, clamped to 0 to 1.
fn ease_out(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    1.0 - (1.0 - u).powi(3)
}

/// The attack's name, at the largest size that fits `room`: on one line if it can be, or broken
/// at the space nearest its middle onto two.
fn fit_name(name: &str, room: i32) -> (i32, Vec<&str>) {
    if text_width(name) * 2 <= room {
        return (2, vec![name]);
    }
    let split = name
        .match_indices(' ')
        .map(|(at, _)| (name[..at].trim_end(), name[at..].trim_start()))
        .min_by_key(|(first, second)| text_width(first).max(text_width(second)));
    match split {
        Some((first, second)) if text_width(first).max(text_width(second)) * 2 <= room => {
            (2, vec![first, second])
        }
        _ => (1, vec![name]),
    }
}

/// The part of a picture that shows: left, top, right and bottom, inclusive.
type Bounds = (i32, i32, i32, i32);

/// Each picture in a close-up with the part of it that is drawn, and how much it is enlarged:
/// one or two are framed closer than a crowd.
struct Group<'a> {
    framed: Vec<(&'a Canvas, Bounds)>,
    scale: i32,
}

fn framing(pictures: &[Canvas]) -> Group<'_> {
    let framed: Vec<(&Canvas, Bounds)> = pictures
        .iter()
        .filter_map(|picture| {
            let (left, top, right, bottom) = picture.alpha_bounds()?;
            Some((
                picture,
                (left as i32, top as i32, right as i32, bottom as i32),
            ))
        })
        .collect();
    let scale = if framed.len() <= 2 { 4 } else { 3 };
    Group { framed, scale }
}

/// How wide a group stands side by side, each overlapping the next by `overlap` beyond the
/// slight overlap they always have.
fn group_width(group: &Group, overlap: i32) -> i32 {
    let widths = portrait_widths(group);
    let slight = widths.iter().min().copied().unwrap_or(0) / 6;
    let overlaps = (widths.len() as i32 - 1).max(0) * (slight + overlap);
    widths.iter().sum::<i32>() - overlaps + 8
}

fn portrait_widths(group: &Group) -> Vec<i32> {
    group
        .framed
        .iter()
        .map(|(_, (left, _, right, _))| (right - left + 1) * group.scale)
        .collect()
}

/// Whoever is acting, each framed on head and shoulders: the top of their picture enlarged so the
/// face fills the band, side by side and a little overlapping, the first in front. They keep
/// within `room` from the left, overlapping more if they must.
fn portraits(band: &mut Canvas, group: &Group, room: i32, t: f32, reduce_motion: bool) {
    let widths = portrait_widths(group);
    let count = widths.len() as i32;
    let extra = if count > 1 {
        ((group_width(group, 0) - room) / (count - 1)).max(0)
    } else {
        0
    };
    let slight = widths.iter().min().copied().unwrap_or(0) / 6;
    let mut places = Vec::new();
    let mut x = 8;
    for width in &widths {
        places.push(x);
        x += width - slight - extra;
    }
    let scale = group.scale;
    for (index, ((picture, (left, top, right, bottom)), place)) in
        group.framed.iter().zip(&places).enumerate().rev()
    {
        let slide = if reduce_motion {
            1.0
        } else {
            ease_out((t - 0.04 - index as f32 * 0.07) / 0.28)
        };
        let width = (right - left + 1) * scale;
        let x = (*place as f32 * slide - (width + 16) as f32 * (1.0 - slide)) as i32;
        // Each a little lower than the one before, so they stand as a group rather than a row.
        let y = 4 + (index as i32 % 2) * 3;
        for sy in *top..=*bottom {
            for sx in *left..=*right {
                let pixel = picture.get(sx, sy);
                if pixel.a == 0 {
                    continue;
                }
                let (px, py) = (x + (sx - left) * scale, y + (sy - top) * scale);
                if py >= CUTIN_HEIGHT {
                    continue;
                }
                // Their shadow on the band, then themselves.
                rect(band, px + 4, py + 3, scale, scale, rgba(0x0a0414, 90));
                if pixel.a == 255 {
                    band.fill_rect(px, py, scale, scale, pixel);
                } else {
                    rect(band, px, py, scale, scale, pixel);
                }
            }
        }
    }
}

/// The attack's name in great gold capitals: lit pale at the top and burning orange at the foot,
/// outlined dark, with a hard shadow. `flare` turns it white as it lands.
fn lettering(scene: &mut Canvas, (x, y): (i32, i32), text: &str, scale: i32, flare: f32) {
    let mut glyphs = Canvas::new(text_width(text).max(1) as u32, GLYPH_HEIGHT as u32);
    draw_text(&mut glyphs, 0, 0, text, rgb(0xffffff));
    // The letters enlarged, with a pixel's margin all round for the outline.
    let (width, height) = (glyphs.width() as i32 * scale + 2, GLYPH_HEIGHT * scale + 2);
    let lit: Vec<bool> = (0..height)
        .flat_map(|py| (0..width).map(move |px| (px, py)))
        .map(|(px, py)| px > 0 && py > 0 && glyphs.get((px - 1) / scale, (py - 1) / scale).a > 0)
        .collect();
    let mut outline = vec![false; lit.len()];
    for py in 1..height - 1 {
        for px in 1..width - 1 {
            if lit[(py * width + px) as usize] {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        outline[((py + dy) * width + px + dx) as usize] = true;
                    }
                }
            }
        }
    }
    let (x, y) = (x - 1, y - 1);
    for (dx, dy, color) in [(3, 3, rgba(0x2a0c3a, 230)), (0, 0, rgb(0x14061c))] {
        for (index, _) in outline.iter().enumerate().filter(|(_, near)| **near) {
            let (px, py) = (index as i32 % width, index as i32 / width);
            put(scene, x + px + dx, y + py + dy, color);
        }
    }
    for (index, _) in lit.iter().enumerate().filter(|(_, lit)| **lit) {
        let (px, py) = (index as i32 % width, index as i32 / width);
        let tone = match (py - 1) / scale {
            0 | 1 => rgb(0xfff2b8),
            2 | 3 => rgb(GOLD),
            4 | 5 => rgb(0xf0a838),
            _ => rgb(0xd8742a),
        };
        put(scene, x + px, y + py, mix(tone, rgb(0xffffff), flare));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_reduced_motion_the_clearing_holds_still() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let party: Vec<Id> = cast.ids().take(2).collect();
        let mut ground = open(&cast, &party, 0.0);
        let mut sovereign = Sovereign::new(&mut ground, &cast, &party, false, 0.0);
        let mut now = 0.0;
        // Into the opening, where the screen shakes and the Sovereign would bob.
        while now < 0.5 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            sovereign.tick(&mut ground, &cast, now);
        }
        assert!(
            now < sovereign.stage.shake_until,
            "the opening shake has passed"
        );
        let first = compose(&mut ground, &sovereign, now);
        let later = compose(&mut ground, &sovereign, now + 0.07);
        assert_eq!(first, later, "something moved");
    }

    #[test]
    fn a_long_attack_name_breaks_where_its_halves_balance() {
        assert_eq!(
            fit_name("POCKET FULL OF CHAOS", 240),
            (2, vec!["POCKET FULL OF CHAOS"])
        );
        assert_eq!(
            fit_name("HESITANT... ULTIMATE... PEEK", 240),
            (2, vec!["HESITANT...", "ULTIMATE... PEEK"])
        );
    }

    /// Stand-ins for whoever is acting: a body of one colour each.
    fn figures(colors: &[u32]) -> Vec<Canvas> {
        colors
            .iter()
            .map(|&color| {
                let mut picture = Canvas::new(48, 48);
                picture.fill_rect(12, 14, 24, 30, rgb(color));
                picture
            })
            .collect()
    }

    #[test]
    fn everyone_in_a_crowded_close_up_shows_a_face() {
        let colors = [0x10e010, 0xe01010, 0x1010e0, 0xe0e010];
        let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
        let name = "BONDS BEYOND THE DESKTOP: EVERYONE";
        cutin(&mut scene, &figures(&colors), name, 1.0, false);
        for color in colors {
            let showing = (CUTIN_TOP..CUTIN_TOP + 40)
                .flat_map(|y| (0..SCENE_WIDTH as i32).map(move |x| (x, y)))
                .filter(|&(x, y)| scene.get(x, y) == rgb(color))
                .count();
            assert!(showing > 600, "{color:06x} shows only {showing} pixels");
        }
    }

    #[test]
    fn with_reduced_motion_a_close_up_holds_still() {
        let pictures = figures(&[0x80c0a0, 0xc080a0]);
        let close_up = |t: f32| {
            let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
            cutin(&mut scene, &pictures, "UNBREAKABLE BULWARK", t, true);
            scene
        };
        assert_eq!(close_up(0.05), close_up(1.2), "something moved");
    }
}
