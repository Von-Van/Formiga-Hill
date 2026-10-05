//! How the things in a heap look: weathered planks, adzed beams, mossy stones and boulders,
//! hessian sacks, slatted crates and old tins, each shaded with its own ramp, lit from the upper
//! left and outlined in a darker shade of itself, with its grain, weave or rust from the hash so
//! every one is its own. And a heap drawn whole as it stands, with whatever is lying out in the
//! open on it, ready to stand among the companions.

use super::Site;
use super::heap::{Heap, Hoard, Item, Place, Stuff};
use crate::finds::art::icon;
use crate::paint::{Ramp, blit, chance, ellipse, hline, mix, noise, put, rgb, rgba, vline};
use crate::playground::Prop;
use formiga_art::{Canvas, Rgba};

const OLD_WOOD: Ramp = Ramp::new(0x3e3229, 0x5e4c3c, 0x7c6650, 0x9a8268, 0xb49c80);
const BEAM: Ramp = Ramp::new(0x2c221c, 0x45362a, 0x5e4a38, 0x786048, 0x917a5e);
const ROCK: Ramp = Ramp::new(0x343e3c, 0x4c5752, 0x66706a, 0x828a80, 0x9fa596);
const MOSS: Ramp = Ramp::new(0x243c24, 0x35552e, 0x4a6f39, 0x638b45, 0x82a656);
const LICHEN: Ramp = Ramp::new(0x5a5428, 0x7a7234, 0x9a9042, 0xb8ac56, 0xd0c470);
const HESSIAN: Ramp = Ramp::new(0x5a4628, 0x7e663e, 0xa08654, 0xbca36c, 0xd2bc88);
const PATCHED: Ramp = Ramp::new(0x3e3a4a, 0x56506a, 0x6e6a86, 0x8a86a0, 0xa6a2b8);
const CRATE: Ramp = Ramp::new(0x4a3624, 0x6a5034, 0x8c6c46, 0xa8885a, 0xc2a274);
const TIN: Ramp = Ramp::new(0x3c3c40, 0x5e6066, 0x868a90, 0xaeb2b6, 0xdcdee0);
const RUST: Ramp = Ramp::new(0x4a2418, 0x6e3820, 0x8a4a2a, 0xa8643a, 0xc4844e);
const LABEL: [Ramp; 3] = [
    Ramp::new(0x5a1e1e, 0x7e2c28, 0xa03a32, 0xbc5a48, 0xd47e66),
    Ramp::new(0x1e3456, 0x2a4874, 0x3a5e92, 0x5478ac, 0x7c9cc4),
    Ramp::new(0x2e4a26, 0x3e6232, 0x527c40, 0x6c9652, 0x8eb06a),
];
const STRING: Rgba = rgb(0xc8b48a);
const IRON: Rgba = rgb(0x2e3436);
const HOLLOW: Rgba = rgb(0x1c140e);
const SHADE: Rgba = rgba(0x14241c, 80);
const DUST: Ramp = Ramp::new(0x6a5c4a, 0x8a7a64, 0xa8987e, 0xc4b69a, 0xdcd0b6);
const PARCHMENT: Ramp = Ramp::new(0x7a5e38, 0xa8865a, 0xd2b484, 0xe6d0a4, 0xf6e8c8);
const INK: Rgba = rgb(0x6a3a28);

/// How a heap is moving: things being lifted off it, things coming down, and dust.
#[derive(Clone, Debug, Default)]
pub struct Motion {
    /// Things lifted off, as they were, and since when.
    pub lifting: Vec<(Item, f32)>,
    /// Things coming down: which, from how high, and since when.
    pub falling: Vec<(usize, i32, f32)>,
    /// Puffs of dust: where across the heap, how high, and since when.
    pub puffs: Vec<(i32, i32, f32)>,
}

/// How long something takes to be lifted clear, to come down, and how long dust hangs.
pub const LIFT_SECS: f32 = 0.5;
pub const FALL_SECS: f32 = 0.28;
pub const PUFF_SECS: f32 = 0.7;
/// How far round a heap its drawing reaches: for things lifted aside and dust.
const MARGIN: i32 = 12;
const TALL: i32 = 72;

impl Motion {
    /// Forgets whatever has finished moving.
    pub fn settle(&mut self, now: f32) {
        self.lifting.retain(|(_, since)| now - since < LIFT_SECS);
        self.falling.retain(|(_, _, since)| now - since < FALL_SECS);
        self.puffs.retain(|(_, _, since)| now - since < PUFF_SECS);
    }
}

/// Where something in a heap is drawn in the scene, as its left and top.
pub fn scene_rect(site: &Site, item: &Item) -> (i32, i32, i32, i32) {
    (
        site.left + item.x,
        site.ground - item.y - item.h,
        item.w,
        item.h,
    )
}

/// The heap as it stands now, with everything moving as it is at `now`, as a prop standing on the
/// heap's own row. With reduced motion, something lifted is simply gone and something coming down
/// is already down, and the dust is held still a moment.
pub fn heap_prop(heap: &Heap, site: &Site, motion: &Motion, now: f32, reduce_motion: bool) -> Prop {
    let width = heap.width + MARGIN * 2;
    let mut sprite = Canvas::new(width as u32, TALL as u32);
    let floor = TALL - 1;
    let origin = |x: i32, y: i32, h: i32| (MARGIN + x, floor - y - h + 1);
    // The soft shade the heap casts on the ground.
    let (left, right) = heap.span();
    if right > left {
        let middle = MARGIN + (left + right) / 2;
        ellipse(
            &mut sprite,
            middle + 2,
            floor,
            (right - left) / 2 + 3,
            2,
            SHADE,
        );
    }
    for (index, item) in heap.items.iter().enumerate() {
        if item.lifted {
            continue;
        }
        let mut y = item.y;
        if !reduce_motion
            && let Some(&(_, from, since)) = motion.falling.iter().find(|(at, _, _)| *at == index)
        {
            let t = ((now - since) / FALL_SECS).clamp(0.0, 1.0);
            y = (from as f32 + (item.y - from) as f32 * t * t).round() as i32;
        }
        let (x, top) = origin(item.x, y, item.h);
        paint(&mut sprite, item, (x, top));
    }
    // Whatever is out in the open, lying where it fell.
    for hidden in &heap.hidden {
        if hidden.place != Place::Open {
            continue;
        }
        let picture = match hidden.what {
            Hoard::Find(find) => icon(find.id),
            Hoard::Map => map_icon(),
        };
        let (x, top) = origin(hidden.at - 4, hidden.lies, 8);
        blit(&mut sprite, &picture, x, top);
    }
    if !reduce_motion {
        for (item, since) in &motion.lifting {
            let t = ((now - since) / LIFT_SECS).clamp(0.0, 1.0);
            if t >= 1.0 {
                continue;
            }
            let rise = (t * 14.0) as i32;
            let aside = (t * 6.0) as i32
                * if item.x + item.w / 2 < heap.width / 2 {
                    -1
                } else {
                    1
                };
            let mut ghost = Canvas::new(item.w as u32 + 2, item.h as u32 + 2);
            paint(&mut ghost, item, (1, 1));
            let fade = (255.0 * (1.0 - t * t)) as u8;
            for pixel_y in 0..ghost.height() as i32 {
                for pixel_x in 0..ghost.width() as i32 {
                    let pixel = ghost.get(pixel_x, pixel_y);
                    if pixel.a > 0 {
                        ghost.set(pixel_x, pixel_y, Rgba::new(pixel.r, pixel.g, pixel.b, fade));
                    }
                }
            }
            let (x, top) = origin(item.x + aside, item.y + rise, item.h);
            blit(&mut sprite, &ghost, x - 1, top - 1);
        }
    }
    for &(at, high, since) in &motion.puffs {
        let t = if reduce_motion {
            0.4
        } else {
            ((now - since) / PUFF_SECS).clamp(0.0, 1.0)
        };
        let (x, top) = origin(at, high, 0);
        dust(&mut sprite, (x, top), t);
    }
    Prop::new(
        sprite,
        (site.left - MARGIN, site.ground - floor),
        site.ground as f32,
    )
}

/// A puff of dust, `t` of the way through hanging in the air.
fn dust(sprite: &mut Canvas, (x, y): (i32, i32), t: f32) {
    let spread = 3.0 + t * 7.0;
    let alpha = (170.0 * (1.0 - t)) as u8;
    for index in 0..9 {
        let angle = index as f32 / 9.0 * std::f32::consts::PI;
        let (dx, dy) = (
            (angle.cos() * spread).round() as i32,
            -(angle.sin() * spread * 0.5).round() as i32 - (t * 3.0) as i32,
        );
        let ramp = if index % 3 == 0 {
            DUST.light
        } else {
            DUST.base
        };
        ellipse(
            sprite,
            x + dx,
            y + dy,
            1 + (t * 2.0) as i32,
            1,
            Rgba::new(ramp.r, ramp.g, ramp.b, alpha),
        );
    }
}

/// Paints one thing with its top-left at `(x, y)`.
pub fn paint(canvas: &mut Canvas, item: &Item, (x, y): (i32, i32)) {
    let (w, h, salt) = (item.w, item.h, item.salt);
    match item.stuff {
        Stuff::Plank => plank(canvas, (x, y), (w, h), OLD_WOOD, salt),
        Stuff::Beam => beam(canvas, (x, y), (w, h), salt),
        Stuff::Stone => stone(canvas, (x, y), (w, h), salt, false),
        Stuff::Boulder => stone(canvas, (x, y), (w, h), salt, true),
        Stuff::Sack => sack(canvas, (x, y), (w, h), salt),
        Stuff::Crate => crate_box(canvas, (x, y), (w, h), salt),
        Stuff::Tin => tin(canvas, (x, y), (w, h), salt),
    }
}

/// A weathered board seen edge-on and a little from above: its top lit, grain running along it in
/// streaks, its sawn ends showing, a nail or two rusting in it, and moss where it has lain.
fn plank(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), ramp: Ramp, salt: u32) {
    // One end broken off short, at a slant.
    let broken = noise(0, 0, salt).is_multiple_of(3);
    for py in 0..h {
        for px in 0..w {
            if broken && px >= w - 2 && py < (px - (w - 3)) {
                continue;
            }
            let end = px == 0 || px == w - 1;
            let mut color = if py == h - 1 || end {
                ramp.edge
            } else if py == 0 {
                ramp.light
            } else {
                ramp.base
            };
            if !end
                && py > 0
                && py < h - 1
                && noise((x + px).div_euclid(4), py, salt).is_multiple_of(4)
            {
                color = ramp.shadow;
            }
            if py == 0 && !end && chance(px, py, salt + 1, 26) {
                color = ramp.shine;
            }
            put(canvas, x + px, y + py, color);
        }
    }
    // End grain: a paler end with a dark check in it.
    put(canvas, x + 1, y + h / 2, mix(ramp.light, ramp.base, 0.5));
    for nail in 0..1 + (salt % 2) as i32 {
        let nx = x + 3 + (noise(nail, 1, salt) % (w - 6).max(1) as u32) as i32;
        put(canvas, nx, y + h / 2, IRON);
        put(canvas, nx + 1, y + h / 2, RUST.base);
    }
    moss_along(canvas, (x + 2, y), w - 4, salt + 2);
}

/// A heavy squared timber: darker, adze marks across its face, an iron bolt near each end.
fn beam(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32) {
    plank(canvas, (x, y), (w, h), BEAM, salt);
    for mark in (4..w - 4).step_by(5) {
        let mx = x + mark + (noise(mark, 2, salt) % 2) as i32;
        put(canvas, mx, y + 1, BEAM.shadow);
        put(canvas, mx + 1, y + 2, BEAM.shadow);
        put(canvas, mx - 1, y + 1, BEAM.light);
    }
    for bx in [x + 3, x + w - 4] {
        put(canvas, bx, y + 2, rgb(0x5a5e60));
        put(canvas, bx, y + 1, rgb(0x8a8e90));
        put(canvas, bx + 1, y + 2, IRON);
    }
}

/// A stone, or a boulder: rounded, a little squared, lit from the upper left, moss on its crown
/// and a rosette of lichen on its face; a boulder with a crack across it.
fn stone(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32, big: bool) {
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0 + 0.5);
    let (rx, ry) = (w as f32 / 2.0, h as f32 / 2.0 + 0.5);
    let inside = |px: i32, py: i32| {
        let (u, v) = ((px as f32 + 0.5 - cx) / rx, (py as f32 + 0.5 - cy) / ry);
        u.abs().powf(2.4) + v.abs().powf(2.4) <= 1.0 && py < h
    };
    for py in 0..h {
        for px in 0..w {
            if !inside(px, py) {
                continue;
            }
            let edge =
                !inside(px - 1, py) || !inside(px + 1, py) || !inside(px, py - 1) || py == h - 1;
            let (u, v) = ((px as f32 + 0.5 - cx) / rx, (py as f32 + 0.5 - cy) / ry);
            let lit = -0.6 * u - 0.7 * v
                + ((noise(x + px, y + py, salt) % 100) as f32 / 100.0 - 0.5) * 0.3;
            let mut color = if edge {
                ROCK.edge
            } else if lit > 0.55 {
                ROCK.light
            } else if lit > -0.15 {
                ROCK.base
            } else {
                ROCK.shadow
            };
            if !edge && lit > 0.85 {
                color = ROCK.shine;
            }
            // Moss over the crown.
            let crown = v < -0.35 + (noise(px, 0, salt + 1) % 3) as f32 * 0.12;
            if crown && !(edge && v > -0.6) && chance(px, py, salt + 2, if big { 230 } else { 170 })
            {
                color = if u < -0.2 {
                    MOSS.light
                } else if v < -0.7 {
                    MOSS.base
                } else {
                    MOSS.shadow
                };
            }
            put(canvas, x + px, y + py, color);
        }
    }
    if w > 9 {
        let (lx, ly) = (x + w / 2 + 1, y + h / 2 + 1);
        put(canvas, lx, ly, LICHEN.base);
        put(canvas, lx - 1, ly, LICHEN.light);
        put(canvas, lx, ly + 1, LICHEN.shadow);
    }
    if big {
        let mut cx = x + w / 3;
        for py in 2..h - 2 {
            put(canvas, cx, y + py, ROCK.edge);
            if noise(py, 3, salt).is_multiple_of(2) {
                cx += 1;
            }
        }
    }
}

/// A hessian sack slumped on its side: rounded and full, its neck tied off with string, the weave
/// showing, and a patch sewn on.
fn sack(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32) {
    let neck_left = noise(0, 1, salt).is_multiple_of(2);
    let body = w - 3;
    let bx = if neck_left { x + 3 } else { x };
    let (cx, cy) = (body as f32 / 2.0, h as f32 / 2.0 + 0.8);
    let inside = |px: i32, py: i32| {
        let (u, v) = (
            (px as f32 + 0.5 - cx) / (body as f32 / 2.0),
            (py as f32 + 0.5 - cy) / (h as f32 / 2.0 + 0.6),
        );
        // Flattened where it sits.
        u * u + v * v * if v > 0.0 { 0.5 } else { 1.0 } <= 1.0 && py < h
    };
    for py in 0..h {
        for px in 0..body {
            if !inside(px, py) {
                continue;
            }
            let edge =
                !inside(px - 1, py) || !inside(px + 1, py) || !inside(px, py - 1) || py == h - 1;
            let (u, v) = (
                (px as f32 + 0.5 - cx) / (body as f32 / 2.0),
                (py as f32 + 0.5 - cy) / (h as f32 / 2.0),
            );
            let lit = -0.55 * u - 0.75 * v;
            let mut color = if edge {
                HESSIAN.edge
            } else if lit > 0.5 {
                HESSIAN.light
            } else if lit > -0.2 {
                HESSIAN.base
            } else {
                HESSIAN.shadow
            };
            // The weave.
            if !edge && (px + py) % 2 == 0 && noise(px, py, salt).is_multiple_of(3) {
                color = mix(color, HESSIAN.edge, 0.3);
            }
            // A crease where it sags.
            if !edge && px == body / 2 + (py / 3) && py > 2 {
                color = HESSIAN.shadow;
            }
            put(canvas, bx + px, y + py, color);
        }
    }
    // A patch, sewn on with big stitches.
    let (px, py) = (bx + body / 2 - 1, y + h / 2 - 1);
    for (dx, dy) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
        put(
            canvas,
            px + dx,
            py + dy,
            if dy == 0 { PATCHED.light } else { PATCHED.base },
        );
    }
    put(canvas, px + 3, py, STRING);
    put(canvas, px - 1, py + 1, STRING);
    // The tied neck, and the tuft of hessian beyond the string.
    let (nx, step) = if neck_left {
        (x + 2, -1)
    } else {
        (x + body - 1, 1)
    };
    let ny = y + h / 2 - 1;
    put(canvas, nx, ny, HESSIAN.base);
    put(canvas, nx, ny + 1, HESSIAN.shadow);
    put(canvas, nx + step, ny, STRING);
    put(canvas, nx + step, ny + 1, mix(STRING, HESSIAN.edge, 0.4));
    put(canvas, nx + step * 2, ny - 1, HESSIAN.light);
    put(canvas, nx + step * 2, ny, HESSIAN.base);
    put(canvas, nx + step * 2, ny + 1, HESSIAN.edge);
}

/// A slatted crate: corner posts, boards across with dark gaps between, one board split and the
/// dark showing through, and a faded stencilled mark.
fn crate_box(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32) {
    let broken = 1 + (noise(1, 2, salt) % ((h - 3) / 4).max(1) as u32) as i32;
    for py in 0..h {
        for px in 0..w {
            let post = px <= 1 || px >= w - 2;
            let row = (py - 1).rem_euclid(4);
            let gap = !post && row == 3 && py > 0 && py < h - 1;
            let slat = (py - 1).div_euclid(4);
            let mut color = if py == h - 1 || px == 0 || px == w - 1 || py == 0 {
                CRATE.edge
            } else if gap {
                HOLLOW
            } else if post {
                if px == 1 { CRATE.base } else { CRATE.shadow }
            } else if row == 0 {
                CRATE.light
            } else {
                CRATE.base
            };
            if !post && !gap && slat == broken && px > w / 2 && py > 0 && py < h - 1 {
                color = if row == 0 { CRATE.shadow } else { HOLLOW };
            }
            if color == CRATE.base && noise((x + px).div_euclid(3), py, salt).is_multiple_of(5) {
                color = CRATE.shadow;
            }
            put(canvas, x + px, y + py, color);
        }
    }
    // The top edge lit along its length, a nail in each post.
    hline(canvas, x + 1, y, w - 2, CRATE.light);
    put(canvas, x + 1, y, CRATE.shine);
    for py in [y + 2, y + h - 3] {
        put(canvas, x + 1, py, IRON);
        put(canvas, x + w - 2, py, IRON);
    }
    // A stencilled mark, faded: a ring, or a cross.
    let (mx, my) = (x + w / 2 - 2, y + h / 2 - 1);
    let faded = mix(LABEL[0].base, CRATE.base, 0.55);
    if noise(2, 2, salt).is_multiple_of(2) {
        for (dx, dy) in [(1, 0), (2, 0), (0, 1), (3, 1), (1, 2), (2, 2)] {
            put(canvas, mx + dx, my + dy, faded);
        }
    } else {
        for d in 0..3 {
            put(canvas, mx + d, my + d, faded);
            put(canvas, mx + 2 - d, my + d, faded);
        }
    }
}

/// An old tin: round, lit across from the left like a cylinder, its lid's rim, a faded label band,
/// and rust coming through.
fn tin(canvas: &mut Canvas, (x, y): (i32, i32), (w, h): (i32, i32), salt: u32) {
    let label = LABEL[(salt % LABEL.len() as u32) as usize];
    let band = (h / 2, h / 2 + 1);
    for py in 0..h {
        for px in 0..w {
            let across = px as f32 / (w - 1).max(1) as f32;
            let edge = px == 0 || px == w - 1 || py == h - 1;
            let ramp = if py >= band.0 && py <= band.1 {
                label
            } else {
                TIN
            };
            let mut color = if edge {
                ramp.edge
            } else if py == 0 {
                TIN.shine
            } else if across < 0.3 {
                ramp.light
            } else if across < 0.7 {
                ramp.base
            } else {
                ramp.shadow
            };
            if !edge && py > 0 && chance(x + px, y + py, salt, 50) {
                color = if chance(px, py, salt + 1, 128) {
                    RUST.base
                } else {
                    RUST.light
                };
            }
            put(canvas, x + px, y + py, color);
        }
    }
    // The lid, a little wider than the tin, with its lip lit.
    hline(canvas, x, y, w, TIN.light);
    put(canvas, x - 1, y + 1, TIN.edge);
    put(canvas, x + w, y + 1, TIN.edge);
    hline(canvas, x, y + 1, w, TIN.shadow);
}

/// Moss in a ragged line along the top of something, here and there.
fn moss_along(canvas: &mut Canvas, (x, y): (i32, i32), long: i32, salt: u32) {
    if !noise(5, 5, salt).is_multiple_of(2) {
        return;
    }
    let start = (noise(1, 1, salt) % long.max(1) as u32) as i32;
    let run = 3 + (noise(2, 1, salt) % 5) as i32;
    for px in start..(start + run).min(long) {
        put(
            canvas,
            x + px,
            y,
            if px % 3 == 0 { MOSS.light } else { MOSS.base },
        );
        if chance(px, y, salt, 120) {
            put(canvas, x + px, y - 1, MOSS.light);
        }
    }
}

/// A torn map, rolled and tied, nine pixels square: for the basket and for one lying in a heap.
pub fn map_icon() -> Canvas {
    let mut canvas = Canvas::new(9, 9);
    // The roll, lying across, its torn end curling.
    for py in 2..7 {
        for px in 1..8 {
            let edge = py == 2 || py == 6 || px == 1 || px == 7;
            let color = if edge {
                PARCHMENT.edge
            } else if py == 3 {
                PARCHMENT.light
            } else if py == 5 {
                PARCHMENT.shadow
            } else {
                PARCHMENT.base
            };
            put(&mut canvas, px, py, color);
        }
    }
    put(&mut canvas, 2, 3, PARCHMENT.shine);
    // The rolled end, and the string round it.
    vline(&mut canvas, 6, 3, 3, PARCHMENT.shadow);
    put(&mut canvas, 7, 1, PARCHMENT.light);
    put(&mut canvas, 8, 2, PARCHMENT.edge);
    vline(&mut canvas, 4, 2, 5, STRING);
    put(&mut canvas, 4, 7, STRING);
    // A dotted way, and an X.
    put(&mut canvas, 2, 4, INK);
    put(&mut canvas, 3, 5, INK);
    put(&mut canvas, 5, 4, INK);
    put(&mut canvas, 1, 7, PARCHMENT.edge);
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::Dice;
    use crate::track::SITES;

    #[test]
    fn every_kind_of_thing_is_drawn_inside_its_own_size() {
        for stuff in [
            Stuff::Plank,
            Stuff::Beam,
            Stuff::Stone,
            Stuff::Boulder,
            Stuff::Sack,
            Stuff::Crate,
            Stuff::Tin,
        ] {
            for salt in 0..20 {
                let item = Item {
                    stuff,
                    x: 0,
                    y: 0,
                    w: 14,
                    h: if matches!(stuff, Stuff::Plank) { 3 } else { 9 },
                    rests_on: Vec::new(),
                    lifted: false,
                    peeked: false,
                    salt,
                };
                let mut canvas = Canvas::new(30, 30);
                paint(&mut canvas, &item, (8, 8));
                let (left, top, right, bottom) = canvas.alpha_bounds().expect("nothing drawn");
                assert!(left >= 7 && right <= 8 + 14, "{stuff:?} spills sideways");
                assert!(
                    top >= 7 && bottom < 8 + item.h as u32,
                    "{stuff:?} spills up or down"
                );
            }
        }
    }

    #[test]
    fn a_heap_stands_on_its_own_row_and_with_reduced_motion_nothing_is_caught_midway() {
        let site = &SITES[1];
        let mut dice = Dice::new(3);
        let heap = Heap::build(&site.recipe, site.width, &mut dice);
        let prop = heap_prop(&heap, site, &Motion::default(), 0.0, false);
        assert_eq!(prop.base, site.ground as f32);
        let (_, _, _, bottom) = prop.bounds();
        assert!(bottom <= site.ground + 2, "the heap sinks below its row");
        // Something coming down from high up, drawn halfway with motion and already down without.
        let mut falling = Motion::default();
        falling.falling.push((0, heap.items[0].y + 20, 0.0));
        let moving = heap_prop(&heap, site, &falling, FALL_SECS * 0.5, false);
        let still = heap_prop(&heap, site, &falling, FALL_SECS * 0.5, true);
        assert_ne!(moving.sprite, still.sprite);
        assert_eq!(
            still.sprite, prop.sprite,
            "with reduced motion it is already down"
        );
    }
}
