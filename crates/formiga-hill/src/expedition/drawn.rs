//! An adventure's map, painted from its places alone: the same parchment, inks and little trees
//! as the map of the Woods, with each place's picture where its author put it, the paths inked
//! between them, the lane in from the Hill to where it starts, and its title on the scroll.
//!
//! Every kind of place (`Site`) has a picture of its own here, drawn about where the party
//! stands at it, so a map anyone makes looks as composed as the Woods' own: the woods fill in
//! round whatever is placed, and nothing is drawn over where anyone stands.

use super::map::{Map, Site};
use super::scenery::{
    self, HEDGE, HEIGHT, LANE, ROCK, SHORTCUT_INK, Shown, TREE, WIDTH, WOOD, bush, from_line,
    on_paper, pine, traced, wash,
};
use crate::font::text_width;
use crate::paint::{ellipse, hline, mix, noise, patches, pick, put, rgb, rgba, vline};
use crate::playground::distance;
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use formiga_art::Canvas;
use std::f32::consts::TAU;

/// The scroll the title is lettered on, top left, at its widest, and the compass, bottom right.
const SCROLL: (i32, i32, i32) = (8, 7, 18);
const MOST_SCROLL: i32 = 180;
const COMPASS: (i32, i32) = (360, 196);
/// How far round a place its picture reaches, for the trees to leave room.
const ROOM: f32 = 24.0;

/// Where on an adventure's map each picture is drawn round a place: the pond lies a little up
/// and right of where the party fishes, and the falls come down behind where they wade.
pub fn pond(at: (f32, f32)) -> (i32, i32, i32, i32) {
    (at.0 as i32 + 10, at.1 as i32 - 16, 18, 10)
}

fn falls_top(at: (f32, f32)) -> (i32, i32) {
    (at.0 as i32 + 8, at.1 as i32 - 46)
}

fn plunge(at: (f32, f32)) -> (i32, i32, i32, i32) {
    (at.0 as i32 + 8, at.1 as i32 - 12, 10, 4)
}

/// The meadow's open ground about a place: its middle and reach.
fn field(at: (f32, f32)) -> (f32, f32, f32, f32) {
    (at.0, at.1 - 10.0, 30.0, 17.0)
}

/// The adventure's map, filling every pixel.
pub fn map(map: &Map, shown: &Shown) -> Canvas {
    let mut scene = Canvas::new(SCENE_WIDTH, SCENE_HEIGHT);
    scenery::table(&mut scene);
    scenery::parchment(&mut scene);
    let start = map.places[map.start].at;
    lane(&mut scene, start);
    for (index, place) in map.places.iter().enumerate() {
        if !shown.places[index] {
            continue;
        }
        match place.site {
            Site::Meadow => meadow_wash(&mut scene, place.at),
            Site::Pool => {
                let (cx, cy, _, _) = pond(place.at);
                water_ground(&mut scene, (cx as f32, cy as f32));
            }
            _ => {}
        }
    }
    let title = title(map);
    trees(&mut scene, map, shown, scroll_width(&title));
    for (index, path) in map.paths.iter().enumerate() {
        if shown.paths[index] {
            scenery::ink_path(&mut scene, &map.places, path, shown, false);
        }
    }
    for (index, place) in map.places.iter().enumerate() {
        if !shown.places[index] {
            continue;
        }
        let at = place.at;
        match place.site {
            Site::Edge => stile(&mut scene, at),
            Site::Hedgerow => hedgerow(&mut scene, at, index as u32),
            Site::Meadow => meadow(&mut scene, at),
            Site::Glade => scenery::glade(&mut scene, at),
            Site::Log => scenery::log(&mut scene, at),
            Site::Signpost => scenery::signpost(&mut scene, at),
            Site::Pool => scenery::pool(&mut scene, pond(at)),
            Site::OldTrack => scenery::old_track(&mut scene, at),
            Site::Falls => {
                crag(&mut scene, at);
                scenery::falls_at(&mut scene, falls_top(at), plunge(at));
            }
        }
        if place.ending {
            pennant(&mut scene, at);
        }
    }
    for (index, place) in map.places.iter().enumerate() {
        if !shown.places[index] {
            scenery::mist(&mut scene, place.at);
        }
    }
    for (index, place) in map.places.iter().enumerate() {
        // The way in needs no name, unless its author gave it one: the lane says what it is.
        let unnamed_way_in = place.site == Site::Edge && place.name == Site::Edge.name();
        if shown.places[index] && !unnamed_way_in {
            let text = lettering(&place.name);
            // Kept on the parchment, however near its edge the place is.
            let half = text.chars().count() as i32 * 2;
            let x = (place.at.0 as i32).clamp(6 + half, WIDTH - 6 - half);
            scenery::label(&mut scene, &text, (x, place.at.1 as i32 + 6));
        }
    }
    let (x, y, height) = SCROLL;
    scenery::scroll(&mut scene, (x, y, scroll_width(&title), height), &title);
    scenery::compass(&mut scene, COMPASS);
    scenery::folds(&mut scene);
    scene
}

/// The title as lettered on the scroll: in capitals, shortened to fit.
fn title(map: &Map) -> String {
    let mut title: String = map
        .title
        .as_deref()
        .unwrap_or("")
        .chars()
        .map(|letter| letter.to_ascii_uppercase())
        .collect();
    while text_width(&title) > MOST_SCROLL - 12 && title.pop().is_some() {}
    title.trim_end().to_owned()
}

fn scroll_width(title: &str) -> i32 {
    (text_width(title) + 14).max(60)
}

/// A place's name as the map letters it: in capitals, without "the", and short.
pub fn lettering(name: &str) -> String {
    let name = name.trim();
    let bare = name
        .strip_prefix("the ")
        .or_else(|| name.strip_prefix("The "))
        .unwrap_or(name);
    bare.chars()
        .filter(|letter| !matches!(letter, '\'' | '\u{2019}'))
        .take(18)
        .map(|letter| letter.to_ascii_uppercase())
        .collect()
}

/// The lane in from the Hill: from the left margin to where the adventure starts, winding.
fn lane(scene: &mut Canvas, (to_x, to_y): (f32, f32)) {
    let from = (0.0, to_y + 6.0);
    let points = [
        from,
        ((to_x * 0.5).max(4.0), to_y + 9.0),
        (to_x, to_y + 2.0),
    ];
    for (x, y) in traced(&points) {
        let wobble = (patches(x as i32, 0, (13, 1), 2230) * 2.0) as i32;
        let (x, top) = (x as i32, y as i32 - 3 + wobble);
        for py in top..top + 6 {
            if !on_paper(x, py) {
                continue;
            }
            let color = if py == top {
                LANE.shadow
            } else if py == top + 5 {
                LANE.edge
            } else if crate::paint::chance(x, py, 2231, 30) {
                LANE.light
            } else {
                LANE.base
            };
            scene.set(x, py, color);
        }
    }
}

/// The meadow's open ground in a pale wash, its edge ragged.
fn meadow_wash(scene: &mut Canvas, at: (f32, f32)) {
    let (cx, cy, rx, ry) = field(at);
    for y in (cy - ry - 4.0) as i32..=(cy + ry + 4.0) as i32 {
        for x in (cx - rx - 4.0) as i32..=(cx + rx + 4.0) as i32 {
            let (u, v) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
            let r = u * u + v * v + (patches(x, y, (5, 5), 2220) - 0.5) * 0.4;
            if r < 1.0 && on_paper(x, y) {
                wash(scene, x, y, scenery::FIELD_WASH, 0.55);
            }
        }
    }
}

/// Reeds and wet ground round a pond, before the trees.
fn water_ground(scene: &mut Canvas, (cx, cy): (f32, f32)) {
    for y in cy as i32 - 16..cy as i32 + 16 {
        for x in cx as i32 - 26..cx as i32 + 26 {
            let (u, v) = ((x as f32 - cx) / 26.0, (y as f32 - cy) / 15.0);
            if u * u + v * v < 1.0 && on_paper(x, y) {
                wash(scene, x, y, rgb(0xa8c4a8), 0.3);
            }
        }
    }
}

/// The woods: little trees everywhere the places, the paths, the title and the compass leave
/// room, broad-leaved near the lane and darker pines deeper in.
fn trees(scene: &mut Canvas, map: &Map, shown: &Shown, scroll: i32) {
    let deep_at = |(x, y): (i32, i32)| {
        let start = map.places[map.start].at;
        (distance((x as f32, y as f32), start) / 320.0).clamp(0.0, 1.0)
    };
    let mut planted: Vec<(i32, i32)> = Vec::new();
    for gy in (20..HEIGHT - 8).step_by(9) {
        for gx in (4..WIDTH - 4).step_by(10) {
            let row = gy / 9;
            let x = gx + pick(gx, gy, 2240, 7) - 3 + if row % 2 == 0 { 5 } else { 0 };
            let y = gy + pick(gx, gy, 2241, 5) - 2;
            let at = (x as f32, y as f32);
            if !on_paper(x - 6, y - 8) || !on_paper(x + 6, y + 2) {
                continue;
            }
            let near_place = map.places.iter().any(|place| {
                let reach = match place.site {
                    Site::OldTrack | Site::Falls => ROOM + 14.0,
                    Site::Meadow | Site::Pool => ROOM + 8.0,
                    _ => ROOM,
                };
                distance(at, (place.at.0 + 2.0, place.at.1 - 12.0)) < reach
            });
            let near_path = map.paths.iter().enumerate().any(|(index, path)| {
                shown.paths[index] && from_line(at, &path.walk_from(&map.places, path.ends.0)) < 7.0
            });
            let start = map.places[map.start].at;
            let lane = from_line(
                at,
                &[
                    (0.0, start.1 + 6.0),
                    ((start.0 * 0.5).max(4.0), start.1 + 9.0),
                    start,
                ],
            ) < 8.0;
            let (sx, sy, sh) = SCROLL;
            let titled = x > sx - 6 && x < sx + scroll + 6 && y > sy - 4 && y < sy + sh + 10;
            let compass = distance(at, (COMPASS.0 as f32, COMPASS.1 as f32)) < 22.0;
            if near_place || near_path || lane || titled || compass {
                continue;
            }
            planted.push((x, y));
        }
    }
    planted.retain(|(x, y)| patches(*x, *y, (30, 22), 2244) > 0.16);
    for &(x, y) in &planted {
        let wood = mix(rgb(0xb8c48e), rgb(0x8a9a6a), deep_at((x, y)));
        for dy in -9..=3 {
            for dx in -6..=6 {
                if dx * dx + dy * dy * 2 <= 40 && on_paper(x + dx, y + dy) {
                    let pixel = scene.get(x + dx, y + dy);
                    scene.set(x + dx, y + dy, mix(pixel, wood, 0.08));
                }
            }
        }
    }
    planted.sort_by_key(|(_, y)| *y);
    for (index, (x, y)) in planted.into_iter().enumerate() {
        let i = index as i32;
        let deep = deep_at((x, y));
        match pick(i, 3, 2242, 12) {
            0 => bush(scene, (x, y - 2), 2, HEDGE, 2245 + index as u32),
            _ if pick(i, 0, 2242, 10) as f32 / 10.0 < deep - 0.3 => {
                pine(scene, (x, y), 6 + pick(i, 1, 2242, 5));
            }
            _ => {
                let radius = if pick(i, 4, 2242, 9) == 0 {
                    5
                } else {
                    3 + pick(i, 2, 2242, 2)
                };
                vline(scene, x, y - 1, 3, WOOD.shadow);
                put(scene, x - 1, y + 1, WOOD.edge);
                put(scene, x + 1, y + 1, rgba(0x2a1c10, 60));
                bush(
                    scene,
                    (x, y - radius - 2),
                    radius,
                    TREE,
                    2243 + index as u32,
                );
            }
        }
    }
}

/// The way in, where the lane comes into the Woods: a stile, and the first bushes either side.
fn stile(scene: &mut Canvas, (x, y): (f32, f32)) {
    let (x, y) = (x as i32, y as i32);
    for (dx, salt) in [(-14, 2250), (14, 2251)] {
        bush(scene, (x + dx, y - 9), 4, HEDGE, salt);
    }
    for px in [x - 7, x + 7] {
        vline(scene, px, y - 12, 8, WOOD.edge);
        vline(scene, px - 1, y - 12, 8, WOOD.light);
    }
    hline(scene, x - 7, y - 9, 15, WOOD.base);
    hline(scene, x - 7, y - 8, 15, WOOD.edge);
}

/// A run of hedge behind the place, a field gate in it, and blackberries along it.
fn hedgerow(scene: &mut Canvas, (x, y): (f32, f32), salt: u32) {
    let (x, y) = (x as i32, y as i32);
    let foot = y - 10;
    let mut px = x - 26;
    let mut index = 0;
    while px < x + 14 {
        let radius = 3 + pick(index, 0, 2260 + salt, 2);
        bush(
            scene,
            (px, foot - radius + 1),
            radius,
            HEDGE,
            2261 + salt * 7 + index as u32,
        );
        if pick(index, 1, 2260 + salt, 3) == 0 {
            put(scene, px + 1, foot - radius, rgb(0x6a2a48));
        }
        px += radius + 2;
        index += 1;
    }
    let (gx, top) = (x + 14, foot - 9);
    for post in [gx, gx + 12] {
        vline(scene, post, top, 10, WOOD.edge);
        vline(scene, post + 1, top, 10, WOOD.light);
    }
    for row in [top + 2, top + 5, top + 8] {
        hline(scene, gx + 1, row, 11, WOOD.base);
    }
    crate::paint::line(scene, (gx + 1, top + 8), (gx + 11, top + 2), WOOD.shadow);
    for (dx, dy) in [(-10, -4), (-6, -6), (-14, -5)] {
        put(scene, x + dx, y + dy - 12, rgb(0x3a1a3a));
        put(scene, x + dx + 1, y + dy - 12, rgb(0x6a2a5a));
    }
}

/// The meadow: grass and flowers in its wash, an old stump, and a butterfly.
fn meadow(scene: &mut Canvas, at: (f32, f32)) {
    let (cx, cy, rx, ry) = field(at);
    for index in 0..50 {
        let angle = pick(index, 0, 2270, 360) as f32 / 360.0 * TAU;
        let reach = (pick(index, 1, 2270, 100) as f32 / 100.0).sqrt();
        let (x, y) = (
            (cx + angle.cos() * rx * reach * 0.9) as i32,
            (cy + angle.sin() * ry * reach * 0.85) as i32,
        );
        // Not where the party stands.
        if (x - at.0 as i32).abs() < 6 && (y - at.1 as i32).abs() < 10 {
            continue;
        }
        if index % 3 == 0 {
            let flower = [0xe0605a, 0xfbf6ee, 0xf5d25e, 0xb79be0][index as usize % 4];
            put(scene, x, y, rgb(flower));
            put(scene, x, y + 1, TREE.shadow);
        } else {
            put(scene, x, y, TREE.light);
            put(scene, x + 1, y - 1, TREE.base);
        }
    }
    let (x, y) = (at.0 as i32 + 14, at.1 as i32 - 14);
    ellipse(scene, x, y, 4, 2, WOOD.edge);
    ellipse(scene, x, y - 1, 3, 1, WOOD.light);
    for (dx, dy, color) in [
        (-18, -10, 0xf5d25e),
        (-17, -11, 0xf5d25e),
        (-16, -10, 0x5a3c28),
    ] {
        put(scene, x + dx, y + dy, rgb(color));
    }
}

/// Rock behind the falls, hatched, lit from the upper left.
fn crag(scene: &mut Canvas, at: (f32, f32)) {
    let (fx, top) = falls_top(at);
    for y in top - 16..top + 34 {
        for x in fx - 30..fx + 30 {
            let (u, v) = ((x - fx) as f32 / 30.0, (y - top - 8) as f32 / 26.0);
            let r =
                u * u + v * v + (noise(x.div_euclid(3), y.div_euclid(3), 2280) % 10) as f32 / 60.0;
            if r > 1.0 || !on_paper(x, y) {
                continue;
            }
            let hatch = (x + y).rem_euclid(5) == 0 && crate::paint::chance(x, y, 2281, 180);
            let color = if r > 0.88 {
                ROCK.edge
            } else if hatch {
                ROCK.shadow
            } else if u + v < -0.3 {
                ROCK.light
            } else {
                ROCK.base
            };
            wash(scene, x, y, color, 0.8);
        }
    }
}

/// A little red pennant by an ending, for the person to make for.
fn pennant(scene: &mut Canvas, (x, y): (f32, f32)) {
    let (x, y) = (x as i32 - 14, y as i32 - 8);
    vline(scene, x, y - 12, 13, WOOD.edge);
    put(scene, x - 1, y - 12, WOOD.light);
    for row in 0..5 {
        let reach = 5 - (row - 2i32).abs() * 2;
        for dx in 1..=reach.max(1) {
            let color = if row == 0 || dx == reach.max(1) {
                SHORTCUT_INK
            } else {
                rgb(0xd0503a)
            };
            put(scene, x + dx, y - 12 + row, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_lettered_without_the_and_kept_short() {
        assert_eq!(lettering("the old mill pond"), "OLD MILL POND");
        assert_eq!(lettering("Bramble Corner"), "BRAMBLE CORNER");
        assert_eq!(lettering("the carters' track"), "CARTERS TRACK");
        assert_eq!(
            lettering("a very long name for a little place"),
            "A VERY LONG NAME F"
        );
    }
}
