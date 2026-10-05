//! The finds picked at the hedgerow: their icons and their Hilltop pieces. Fruit and flowers
//! mostly, in their ripe colours, and what each becomes: a pie cooling on a stool, a strawberry
//! bed, a fairy ring, a rack of drying herbs, a table set with elderflower cordial, a golden
//! toadstool to sit on, a honeysuckle bower and a fountain shaped like a clover; and the things
//! for planting grown into what they will be, a hazel, a crab apple, a wild rose and a cushion of
//! thyme.

#[cfg(test)]
use super::ICON;
use super::Piece;
use super::brush::*;
use crate::paint::{Ramp, chance, hline, mix, noise, put, rgb, rgba, vline};
use formiga_art::{Canvas, Rgba};

const LEAF: Ramp = Ramp::new(0x22401f, 0x335a2a, 0x4a7834, 0x649444, 0x88b45c);
const FRESH: Ramp = Ramp::new(0x23422a, 0x335e34, 0x4a7e40, 0x66a050, 0x8ec068);
const HAZEL_LEAF: Ramp = Ramp::new(0x29441f, 0x3a5c2a, 0x4f7834, 0x6a9442, 0x8cb058);
const ROSE_LEAF: Ramp = Ramp::new(0x203a20, 0x30542c, 0x427038, 0x5a8a46, 0x7aa65a);
const THYME: Ramp = Ramp::new(0x24381e, 0x34502a, 0x486a36, 0x5e8444, 0x7aa05a);
const GARLIC: Ramp = Ramp::new(0x1a3a20, 0x28542c, 0x3a7038, 0x528c46, 0x72a85a);
const CLOVER: Ramp = Ramp::new(0x1e4a22, 0x2e6a2e, 0x429040, 0x5eb050, 0xa0e080);
const BERRY: Ramp = Ramp::new(0x0e0818, 0x22143a, 0x36204e, 0x54386e, 0xe4dcff);
const STRAWBERRY: Ramp = Ramp::new(0x6a0e14, 0xa81c22, 0xd8302c, 0xf05a44, 0xffd0c0);
const NUT: Ramp = Ramp::new(0x3a1e0e, 0x6a3a18, 0x965424, 0xb87436, 0xf4d8b0);
const HUSK: Ramp = Ramp::new(0x5a4220, 0x82602c, 0xa8823e, 0xc4a258, 0xdcc488);
const BUTTON: Ramp = Ramp::new(0x8a8478, 0xc4bcae, 0xe4ded2, 0xf4f0e8, 0xffffff);
const GILLS: Ramp = Ramp::new(0x6a3a3a, 0x9a5a56, 0xc48478, 0xd8a094, 0xecc4b8);
const STAR: Ramp = Ramp::new(0x9aa0a4, 0xd4d8d8, 0xf4f4f0, 0xfcfcf8, 0xffffff);
const ELDER: Ramp = Ramp::new(0x8a8460, 0xc8c098, 0xece4c0, 0xf8f4dc, 0xffffff);
const APPLE: Ramp = Ramp::new(0x7a5212, 0xb88a1e, 0xe8c034, 0xf8dc60, 0xfffcd8);
const BLUSH: Ramp = Ramp::new(0x6a1414, 0xa02420, 0xd03c2c, 0xe86440, 0xf89070);
const HIP: Ramp = Ramp::new(0x5a0a10, 0x9a121a, 0xd42020, 0xf04a34, 0xffe0d0);
const ROSE: Ramp = Ramp::new(0xa0506a, 0xd27a92, 0xec9eb2, 0xf8c4d0, 0xfde8ee);
const BLOSSOM: Ramp = Ramp::new(0xb07a8a, 0xe0a8b8, 0xf4cad4, 0xfce4ea, 0xffffff);
const THYME_FLOWER: Ramp = Ramp::new(0x5a1e5a, 0x8a3088, 0xb84ab0, 0xd874cc, 0xf4b8ec);
const GOLD: Ramp = Ramp::new(0x7a4a08, 0xc07a10, 0xf0a818, 0xffc840, 0xfff4b0);
const SUCKLE: Ramp = Ramp::new(0x9a7a3a, 0xd8b85a, 0xf4dc84, 0xfcecb0, 0xfffcf0);
const SUCKLE_PINK: Ramp = Ramp::new(0x7a3a4a, 0xb05a6a, 0xd88094, 0xeca8b8, 0xfcd8e0);
const WOOD: Ramp = Ramp::new(0x4e3626, 0x6c4e36, 0x8c6a4a, 0xa88660, 0xbea078);
const RUSTIC: Ramp = Ramp::new(0x3a2a20, 0x543e2e, 0x6e543e, 0x8a6e52, 0xa68a6a);
const BARK: Ramp = Ramp::new(0x2a201c, 0x40322a, 0x58463a, 0x70604e, 0x8a7a64);
const CRUST: Ramp = Ramp::new(0x7a4a1a, 0xb07428, 0xd8a048, 0xecc070, 0xfae0a8);
const DISH: Ramp = Ramp::new(0x7a7064, 0xb8ae9e, 0xdcd4c4, 0xeee8dc, 0xfcfaf4);
const GLASS: Ramp = Ramp::new(0x7a9aa0, 0xa8c4c8, 0xc8dee0, 0xe4f2f2, 0xffffff);
const CORDIAL: Ramp = Ramp::new(0xa08a30, 0xd0b850, 0xe8d474, 0xf4e8a0, 0xfffad8);
const STONE: Ramp = Ramp::new(0x605a5d, 0x857d7c, 0xa69d94, 0xc2baae, 0xdcd5c8);
const WATER: Ramp = Ramp::new(0x2a5a78, 0x3e7a9a, 0x5c9cba, 0x8cc0d8, 0xd8f0f8);
const STRAW: Ramp = Ramp::new(0x8a7438, 0xb89a4e, 0xd6ba68, 0xe8d08a, 0xf6e6b4);
const SOIL: Ramp = Ramp::new(0x2e2018, 0x46301f, 0x5c402a, 0x74543a, 0x8c6a4c);
const STALK: Rgba = rgb(0x4a5a2a);
const TWIG: Rgba = rgb(0x5a3a2a);
const SEED: Rgba = rgb(0xf0d060);
const STRING: Rgba = rgb(0xc8b48a);
const CATKIN: Ramp = Ramp::new(0x8a7a2a, 0xb8a03a, 0xd8c050, 0xece070, 0xfff4a8);
const BEE: Rgba = rgb(0x2a2018);
const BEE_GOLD: Rgba = rgb(0xf0c040);

/// The icon for one of these finds, nine pixels square, or `None` if it isn't drawn yet.
pub fn icon(id: &str) -> Option<Canvas> {
    let leaf = [
        ('g', LEAF.base),
        ('G', LEAF.light),
        ('d', LEAF.edge),
        ('k', STALK),
    ];
    Some(match id {
        // One blackberry, glossy and black, under its sepals.
        "blackberries" => icon_from(
            [
                "....k....",
                "...gGg...",
                "..#o#o#..",
                ".#*lol*#.",
                ".#lolos#.",
                ".#olos##.",
                "..#oso#..",
                "...###...",
                ".........",
            ],
            BERRY,
            &leaf,
        ),
        // A strawberry with its calyx, and the runner reaching off to a little plantlet.
        "strawberry_runner" => icon_from(
            [
                "..dgGgd..",
                "..#*l#...",
                ".#lyoy#..",
                ".#yoly#..",
                "..#oy#...",
                "...#s#...",
                "....#.kk.",
                "...kkk..G",
                ".......gd",
            ],
            STRAWBERRY,
            &[
                ('g', LEAF.base),
                ('G', LEAF.light),
                ('d', LEAF.edge),
                ('k', STALK),
                ('y', SEED),
            ],
        ),
        // A hazelnut in its frilled husk.
        "hazelnut" => icon_from(
            [
                "..E.E.E..",
                ".ELHLOE..",
                ".EOLOSE..",
                "..#*lo#..",
                ".#lolos#.",
                ".#olos##.",
                "..#oss#..",
                "...###...",
                ".........",
            ],
            NUT,
            &[
                ('E', HUSK.edge),
                ('O', HUSK.base),
                ('L', HUSK.light),
                ('H', HUSK.shine),
                ('S', HUSK.shadow),
            ],
        ),
        // A field mushroom, open, its pink gills under it, and a button beside it.
        "field_mushrooms" => icon_from(
            [
                ".........",
                "..#####..",
                ".#*lllo#.",
                "#llloooo#",
                ".PSPSPSP.",
                "...#lo#..",
                "...#lo#..",
                "...#ls#..",
                "..g###g..",
            ],
            BUTTON,
            &[('P', GILLS.base), ('S', GILLS.shadow), ('g', LEAF.light)],
        ),
        // A ball of white stars on its stem, a broad leaf beside it.
        "wild_garlic" => icon_from(
            [
                ".*.*.....",
                "*o*l*....",
                ".*o*o*...",
                "..*.k....",
                "....k.d..",
                "...dkdGd.",
                "..dGkGgd.",
                ".dGgkgd..",
                "..ddkd...",
            ],
            STAR,
            &[
                ('g', GARLIC.base),
                ('G', GARLIC.light),
                ('d', GARLIC.edge),
                ('k', GARLIC.shadow),
            ],
        ),
        // A head of elderflower, cream with gold eyes, on its rays of stalk.
        "elderflower" => icon_from(
            [
                ".........",
                "..*l*l*..",
                ".lylolyl.",
                "*loyolol*",
                ".sosososo",
                "..k.k.k..",
                "...kkk...",
                "....k....",
                "....k....",
            ],
            ELDER,
            &[('k', STALK), ('y', rgb(0xe8c84a))],
        ),
        // A crab apple, gold with a red blush, and a leaf on its stalk.
        "crab_apple_pip" => icon_from(
            [
                "....w.gG.",
                "....wgd..",
                "..##o##..",
                ".#*llol#.",
                "#rRllooo#",
                "#rrlooos#",
                "#rlooos##",
                ".#osss#..",
                "..####...",
            ],
            APPLE,
            &[
                ('g', LEAF.base),
                ('G', LEAF.light),
                ('d', LEAF.edge),
                ('w', TWIG),
                ('r', BLUSH.base),
                ('R', BLUSH.light),
            ],
        ),
        // A rose hip, glossy scarlet, its dried sepals at its foot.
        "rose_hip_seeds" => icon_from(
            [
                "....w....",
                "....w....",
                "...#o#...",
                "..#*lo#..",
                "..#llo#..",
                "..#loo#..",
                "..#oos#..",
                "...#s#...",
                "..c.c.c..",
            ],
            HIP,
            &[('w', TWIG), ('c', rgb(0x2a1a14))],
        ),
        // A sprig of thyme in flower, a root on it.
        "thyme_cutting" => icon_from(
            [
                "..o*o....",
                ".olslo...",
                "..oko....",
                ".g.k.g...",
                "..gkG....",
                ".G.k.g...",
                "..gkg....",
                "...r.r...",
                "..r..r.r.",
            ],
            THYME_FLOWER,
            &[
                ('g', THYME.base),
                ('G', THYME.light),
                ('k', THYME.shadow),
                ('r', rgb(0x8a6a4a)),
            ],
        ),
        // A golden chanterelle, a trumpet with a wavy rim.
        "golden_chanterelle" => icon_from(
            [
                ".........",
                ".#.##.#..",
                "#*l*llo#.",
                "#llloooo#",
                ".#slloss#",
                "..#llos#.",
                "...#lo#..",
                "...#lo#..",
                "....ls...",
            ],
            GOLD,
            &[],
        ),
        // A whorl of honeysuckle trumpets, cream from pink throats, on its leaves.
        "honeysuckle" => icon_from(
            [
                "*l.....l*",
                ".lo...ol.",
                "..op.po..",
                "*l.ppp.l*",
                ".lopppol.",
                "...ppp...",
                "..dGkGd..",
                "...dkd...",
                "....k....",
            ],
            SUCKLE,
            &[
                ('p', SUCKLE_PINK.base),
                ('G', LEAF.light),
                ('d', LEAF.edge),
                ('k', STALK),
            ],
        ),
        // A four-leaf clover, every leaf with its pale chevron.
        "four_leaf_clover" => icon_from(
            [
                ".##...##.",
                "#*l#.#lo#",
                "#lcl#lco#",
                ".#ll#os#.",
                "...#k#...",
                ".#lo#os#.",
                "#lco#sco#",
                "#lo#.#so#",
                ".##.k.##.",
            ],
            CLOVER,
            &[('c', rgb(0xd8f4c8)), ('k', STALK)],
        ),
        _ => return None,
    })
}

/// The Hilltop piece for one of these finds, or `None` if it isn't drawn yet.
pub fn piece(id: &str) -> Option<Piece> {
    Some(match id {
        "blackberries" => pie_on_a_stool(),
        "strawberry_runner" => strawberry_bed(),
        "hazelnut" => hazel_sapling(),
        "field_mushrooms" => fairy_ring(),
        "wild_garlic" => herb_rack(),
        "elderflower" => cordial_table(),
        "crab_apple_pip" => crab_apple_sapling(),
        "rose_hip_seeds" => wild_rose(),
        "thyme_cutting" => thyme_cushion(),
        "golden_chanterelle" => golden_toadstool(),
        "honeysuckle" => honeysuckle_bower(),
        "four_leaf_clover" => clover_fountain(),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------------------------

/// A clump of leaves in a bush: its middle and its size.
type Clump = ((i32, i32), (i32, i32));

/// A bush's worth of leaves: rounded clumps lit from the upper left, parted by shade, outlined in
/// the leaves' own darkest green, with leaves catching the light and tips poking out.
fn foliage(s: &mut Canvas, clumps: &[Clump], ramp: Ramp, salt: u32) {
    let mut layer = Canvas::new(s.width(), s.height());
    for (index, &(centre, size)) in clumps.iter().enumerate() {
        model(&mut layer, ramp, ROUND, 40, salt + index as u32, |x, y| {
            in_ellipse(x, y, centre, size)
        });
    }
    let leafy = |x: i32, y: i32| layer.get(x, y).a > 0;
    let mut bush = layer.clone();
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !leafy(x, y) {
                let beside = leafy(x + 1, y) || leafy(x - 1, y) || leafy(x, y + 1);
                if beside && !leafy(x, y - 1) && chance(x, y, salt + 50, 60) {
                    bush.set(x, y, ramp.edge);
                }
                continue;
            }
            let inner = leafy(x - 1, y) && leafy(x + 1, y) && leafy(x, y - 1) && leafy(x, y + 1);
            let color = layer.get(x, y);
            if inner && color == ramp.edge {
                bush.set(x, y, ramp.shadow);
            } else if inner && color != ramp.shadow && chance(x, y, salt + 51, 34) {
                bush.set(x, y, ramp.light);
                if leafy(x + 1, y + 1) {
                    bush.set(x + 1, y + 1, ramp.base);
                }
            }
        }
    }
    blit_over(s, &bush);
}

/// Everything opaque in `top` laid over `s`.
fn blit_over(s: &mut Canvas, top: &Canvas) {
    for y in 0..top.height() as i32 {
        for x in 0..top.width() as i32 {
            let pixel = top.get(x, y);
            if pixel.a > 0 {
                put(s, x, y, pixel);
            }
        }
    }
}

/// A five-petalled flower, pale outside, with its middle.
fn bloom(s: &mut Canvas, (x, y): (i32, i32), ramp: Ramp, middle: Rgba) {
    put(s, x, y - 1, ramp.shine);
    put(s, x - 1, y, ramp.light);
    put(s, x + 1, y, ramp.base);
    put(s, x - 1, y + 1, ramp.base);
    put(s, x + 1, y + 1, ramp.shadow);
    put(s, x, y, middle);
}

/// A plank lit from the upper left, edged, its grain in streaks.
fn plank(s: &mut Canvas, (x, y): (i32, i32), (width, height): (i32, i32), ramp: Ramp, salt: u32) {
    for py in y..y + height {
        for px in x..x + width {
            let (col, row) = (px - x, py - y);
            let mut color = if row == height - 1 || col == width - 1 || col == 0 {
                ramp.edge
            } else if row == 0 {
                ramp.light
            } else {
                ramp.base
            };
            if row > 0 && row < height - 1 && noise(px.div_euclid(3), py, salt).is_multiple_of(5) {
                color = ramp.shadow;
            }
            put(s, px, py, color);
        }
    }
}

/// A round post from `top` down to `foot`, lit from the left.
fn post(s: &mut Canvas, x: i32, (top, foot): (i32, i32), width: i32, ramp: Ramp, salt: u32) {
    for y in top..foot {
        for dx in 0..width {
            let across = dx as f32 / (width - 1).max(1) as f32;
            let mut color = if dx == 0 || dx == width - 1 {
                ramp.edge
            } else {
                cylinder(ramp, across)
            };
            if dx > 0 && dx < width - 1 && noise(x + dx, y.div_euclid(3), salt).is_multiple_of(6) {
                color = mix(color, ramp.edge, 0.5);
            }
            put(s, x + dx, y, color);
        }
    }
    hline(s, x, top, width, ramp.edge);
}

// ---------------------------------------------------------------------------------------------
// The pieces
// ---------------------------------------------------------------------------------------------

/// A blackberry pie cooling on a three-legged stool: a golden lattice crust in a cream dish with
/// purple juice bubbling up through it and dripping down the side, a sprig of berries beside it.
fn pie_on_a_stool() -> Piece {
    let mut s = Canvas::new(26, 26);
    let ground = 24;
    shadow(&mut s, 13, ground, 10, 2);
    // Legs: two at the front splayed out, one behind.
    for &(from, to) in &[
        ((12, 17), (13, 23)),
        ((6, 17), (3, 24)),
        ((19, 17), (22, 24)),
    ] {
        for (step, &(x, y)) in trace(&[from, to]).iter().enumerate() {
            let lit = step % 4 != 3;
            put(&mut s, x, y, if lit { WOOD.light } else { WOOD.base });
            put(&mut s, x + 1, y, WOOD.shadow);
            put(&mut s, x + 2, y, WOOD.edge);
        }
    }
    hline(&mut s, 5, 21, 16, WOOD.shadow);
    // The seat, a round of wood seen a little from above, its edge showing.
    for y in 13..19 {
        for x in 2..24 {
            let (u, v) = (
                (x as f32 + 0.5 - 13.0) / 11.0,
                (y as f32 + 0.5 - 15.0) / 3.2,
            );
            if u * u + v * v > 1.0 {
                continue;
            }
            let edge = u * u + v * v > 0.78;
            let color = if edge {
                WOOD.edge
            } else if v < -0.2 && u < 0.3 {
                WOOD.light
            } else if (x + y * 3) % 7 == 0 {
                WOOD.shadow
            } else {
                WOOD.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 4, 18, 18, WOOD.edge);
    hline(&mut s, 5, 17, 16, WOOD.shadow);
    // The dish, and the pie domed in it.
    for y in 8..15 {
        for x in 5..21 {
            let (u, v) = ((x as f32 + 0.5 - 13.0) / 7.6, (y as f32 + 0.5 - 12.0) / 2.6);
            if u * u + v * v > 1.0 {
                continue;
            }
            put(&mut s, x, y, if v > 0.3 { DISH.shadow } else { DISH.light });
        }
    }
    hline(&mut s, 6, 14, 14, DISH.edge);
    lump(&mut s, 13.0, 10.4, 6.6, 3.2, CRUST, 301);
    // The lattice: strips of crust crossing, the filling showing between them.
    for y in 8..13 {
        for x in 7..20 {
            if s.get(x, y) == CRUST.edge {
                continue;
            }
            let strip = (x + y).rem_euclid(4) < 2 || (x - y).rem_euclid(4) == 0;
            if !strip && s.get(x, y).a > 0 {
                let deep = if (x + y).rem_euclid(3) == 0 {
                    BERRY.light
                } else {
                    BERRY.base
                };
                put(&mut s, x, y, deep);
            }
        }
    }
    put(&mut s, 10, 8, CRUST.shine);
    put(&mut s, 11, 8, CRUST.shine);
    // Juice bubbling over and running down the dish.
    for (x, y) in [(18, 12), (18, 13), (19, 14), (8, 13)] {
        put(&mut s, x, y, BERRY.base);
    }
    put(&mut s, 19, 15, BERRY.light);
    // A sprig of blackberries on the seat beside it.
    for &(x, y) in &[(19, 15), (21, 14)] {
        stamp(
            &mut s,
            (x, y),
            &[".#.", "#*o", ".os"],
            &letters(BERRY, b"#sol*"),
            false,
        );
    }
    put(&mut s, 22, 13, LEAF.light);
    put(&mut s, 23, 13, LEAF.base);
    // Steam curling off it.
    for (x, y, alpha) in [
        (12, 6, 120),
        (13, 5, 90),
        (12, 4, 70),
        (13, 3, 50),
        (15, 6, 90),
        (16, 5, 60),
    ] {
        put(&mut s, x, y, rgba(0xffffff, alpha));
    }
    Piece {
        sprite: s,
        anchor: (13, ground),
    }
}

/// A bed of wild strawberries edged with straw: plants with leaves in threes, white flowers and
/// red berries lying on the straw, and a runner reaching out to root a new plant.
fn strawberry_bed() -> Piece {
    let mut s = Canvas::new(34, 18);
    let ground = 15;
    shadow(&mut s, 17, ground, 15, 2);
    // The bed: dark soil mounded, straw laid over it.
    for y in 10..ground + 1 {
        for x in 2..32 {
            let (u, v) = (
                (x as f32 + 0.5 - 17.0) / 15.0,
                (y as f32 + 0.5 - 13.0) / 3.4,
            );
            if u * u + v * v > 1.0 {
                continue;
            }
            let straw = (x * 3 + y * 5).rem_euclid(7) < 3;
            let color = if straw {
                if v < 0.0 { STRAW.light } else { STRAW.base }
            } else if v < -0.3 {
                SOIL.light
            } else {
                SOIL.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 4, ground, 26, SOIL.edge);
    // The plants.
    for &(x, y) in &[(7, 9), (14, 7), (21, 8), (27, 10), (11, 11), (24, 11)] {
        strawberry_plant(&mut s, (x, y));
    }
    // Berries on the straw, and white flowers.
    for &(x, y) in &[(9, 12), (17, 11), (25, 13), (13, 13), (29, 12)] {
        stamp(
            &mut s,
            (x, y),
            &["#*o#", "#yoy", ".#s."],
            &[
                (b'#', STRAWBERRY.edge),
                (b'o', STRAWBERRY.base),
                (b'*', STRAWBERRY.shine),
                (b'y', SEED),
                (b's', STRAWBERRY.shadow),
            ],
            false,
        );
    }
    for &(x, y) in &[(12, 6), (19, 5), (26, 7)] {
        bloom(&mut s, (x, y), STAR, rgb(0xf0d050));
    }
    // A runner arching off to the right, rooting a new little plant.
    for &(x, y) in &trace(&[(28, 9), (31, 7), (33, 10)]) {
        put(&mut s, x, y, rgb(0xa04a3a));
    }
    put(&mut s, 32, 9, LEAF.light);
    put(&mut s, 33, 8, LEAF.base);
    Piece {
        sprite: s,
        anchor: (17, ground),
    }
}

/// One strawberry plant: three leaflets, toothed, lit from the upper left.
fn strawberry_plant(s: &mut Canvas, (x, y): (i32, i32)) {
    vline(s, x, y, 3, LEAF.shadow);
    for (dx, dy) in [(-3, -1), (0, -3), (3, -1)] {
        let (lx, ly) = (x + dx - 1, y + dy);
        stamp(
            s,
            (lx, ly - 1),
            &["#.#", "lGo", "ooS", "###"],
            &[
                (b'#', LEAF.edge),
                (b'l', LEAF.light),
                (b'G', LEAF.shine),
                (b'o', LEAF.base),
                (b'S', LEAF.shadow),
            ],
            false,
        );
    }
}

/// A young hazel: three slim stems from its foot, round leaves in loose clumps, catkins hanging
/// and a cluster of nuts in their husks.
fn hazel_sapling() -> Piece {
    let mut s = Canvas::new(28, 44);
    let ground = 42;
    shadow(&mut s, 14, ground, 9, 2);
    for path in [
        &[(13, 41), (12, 30), (8, 16), (7, 8)][..],
        &[(14, 41), (15, 28), (14, 12), (15, 4)],
        &[(15, 41), (18, 30), (21, 18), (22, 10)],
    ] {
        bough(&mut s, path, RUSTIC);
    }
    foliage(
        &mut s,
        &[
            ((7, 10), (5, 5)),
            ((21, 12), (5, 5)),
            ((14, 6), (6, 5)),
            ((10, 19), (5, 4)),
            ((19, 21), (5, 4)),
            ((14, 14), (5, 4)),
        ],
        HAZEL_LEAF,
        311,
    );
    // Round leaves catching the sun at the edges.
    for &(x, y) in &[(2, 9), (25, 11), (14, 0), (5, 18), (23, 20)] {
        stamp(
            &mut s,
            (x, y),
            &[".##.", "#*lo", "#los", ".##."],
            &letters(HAZEL_LEAF, b"#sol*"),
            false,
        );
    }
    // Catkins hanging, pale gold.
    for &(x, y) in &[(4, 15), (24, 17), (11, 24), (18, 25)] {
        for dy in 0..4 {
            put(
                &mut s,
                x,
                y + dy,
                if dy % 2 == 0 {
                    CATKIN.light
                } else {
                    CATKIN.base
                },
            );
        }
        put(&mut s, x + 1, y + 1, CATKIN.shadow);
    }
    // A cluster of nuts in their frilled husks.
    for &(x, y) in &[(15, 26), (18, 27)] {
        stamp(
            &mut s,
            (x, y),
            &["EOE", "#*o", "#os", ".#."],
            &[
                (b'E', HUSK.edge),
                (b'O', HUSK.light),
                (b'#', NUT.edge),
                (b'*', NUT.shine),
                (b'o', NUT.base),
                (b's', NUT.shadow),
            ],
            false,
        );
    }
    tuft(&mut s, 14, ground, 5, 312);
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// A fairy ring: a ring of lusher grass in the turf with white field mushrooms all round it, the
/// ones at the back smaller, a button or two coming up between.
fn fairy_ring() -> Piece {
    let mut s = Canvas::new(46, 20);
    let (cx, cy) = (23.0, 12.0);
    // The ring of greener grass.
    for y in 4..20 {
        for x in 0..46 {
            let (u, v) = ((x as f32 + 0.5 - cx) / 20.0, (y as f32 + 0.5 - cy) / 6.0);
            let reach = (u * u + v * v).sqrt();
            if (reach - 1.0).abs() < 0.18 {
                let lit = if v < 0.0 { FRESH.light } else { FRESH.base };
                put(
                    &mut s,
                    x,
                    y,
                    if chance(x, y, 321, 60) {
                        FRESH.shine
                    } else {
                        lit
                    },
                );
            } else if (reach - 1.0).abs() < 0.3 && chance(x, y, 322, 120) {
                put(&mut s, x, y, FRESH.shadow);
            }
        }
    }
    // Mushrooms round the ring, back to front.
    let mut places: Vec<(f32, f32, f32)> = (0..9)
        .map(|index| {
            let angle = index as f32 / 9.0 * std::f32::consts::TAU + 0.3;
            let (x, y) = (cx + angle.cos() * 19.0, cy + angle.sin() * 5.5);
            let near = (angle.sin() + 1.0) / 2.0;
            (x, y, near)
        })
        .collect();
    places.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (index, &(x, y, near)) in places.iter().enumerate() {
        let big = near > 0.45 || index % 3 == 0;
        mushroom(&mut s, (x.round() as i32, y.round() as i32), big);
    }
    Piece {
        sprite: s,
        anchor: (23, 17),
    }
}

/// A field mushroom with its foot at `foot`: open with pink gills under, or a smaller button.
fn mushroom(s: &mut Canvas, (x, foot): (i32, i32), big: bool) {
    let inks = [
        (b'#', BUTTON.edge),
        (b's', BUTTON.shadow),
        (b'o', BUTTON.base),
        (b'l', BUTTON.light),
        (b'*', BUTTON.shine),
        (b'P', GILLS.base),
        (b'S', GILLS.shadow),
    ];
    if big {
        stamp(
            s,
            (x - 3, foot - 6),
            &[
                ".###.", "#*llo#", "#looos#", ".PSPSP.", "..#l#..", "..#ls#..",
            ],
            &inks,
            false,
        );
    } else {
        stamp(
            s,
            (x - 2, foot - 3),
            &[".##.", "#*lo", "#los", ".#s#"],
            &inks,
            false,
        );
    }
}

/// A rack for drying herbs: two posts and a crossbar, bunches of wild garlic hung upside down by
/// their stalks, tied with string, their white flowers drooping, and a basket at its foot.
fn herb_rack() -> Piece {
    let mut s = Canvas::new(28, 32);
    let ground = 30;
    shadow(&mut s, 14, ground, 12, 2);
    post(&mut s, 2, (4, ground), 3, RUSTIC, 331);
    post(&mut s, 23, (4, ground), 3, RUSTIC, 332);
    plank(&mut s, (1, 4), (26, 3), RUSTIC, 333);
    // The bunches, hung by their stalks.
    for (index, &x) in [7, 13, 19].iter().enumerate() {
        vline(&mut s, x, 7, 3, STRING);
        put(&mut s, x - 1, 9, STRING);
        put(&mut s, x + 1, 9, STRING);
        // Stalks down to leaves, the flowers hanging lowest.
        for dx in -2..=2 {
            let length = 6 + (dx + index as i32).rem_euclid(3);
            for dy in 0..length {
                let color = if dy < 2 {
                    GARLIC.shadow
                } else if dx < 0 {
                    GARLIC.light
                } else {
                    GARLIC.base
                };
                put(&mut s, x + dx / 2 + dx.signum() * (dy / 4), 10 + dy, color);
            }
        }
        for (dx, dy) in [(-2, 17), (0, 18), (2, 17), (-1, 19), (1, 19)] {
            let fx = x + dx;
            put(&mut s, fx, dy, STAR.light);
            put(&mut s, fx, dy + 1, STAR.shadow);
        }
        put(&mut s, x, 18, STAR.shine);
    }
    // A basket at its foot, half full.
    for y in 22..ground {
        for x in 6..22 {
            let (u, v) = ((x as f32 + 0.5 - 14.0) / 8.0, (y as f32 + 0.5 - 22.0) / 8.0);
            if v < 0.0 || u * u + v * v > 1.0 {
                continue;
            }
            let weave = (x + y).rem_euclid(3) == 0;
            let color = if u * u + v * v > 0.8 {
                STRAW.edge
            } else if weave {
                STRAW.shadow
            } else if u < -0.2 {
                STRAW.light
            } else {
                STRAW.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 6, 22, 16, STRAW.light);
    hline(&mut s, 7, 21, 14, STRAW.edge);
    for &(x, y) in &[(9, 20), (12, 19), (16, 20), (18, 21)] {
        put(&mut s, x, y, GARLIC.light);
        put(&mut s, x + 1, y, GARLIC.base);
        put(&mut s, x, y - 1, STAR.light);
    }
    tuft(&mut s, 3, ground, 2, 334);
    tuft(&mut s, 24, ground, 2, 335);
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// A little round table on a stump, set with a glass jug of elderflower cordial, pale gold with
/// flower heads floating in it, two cups, and a head of elderflower laid by.
fn cordial_table() -> Piece {
    let mut s = Canvas::new(28, 26);
    let ground = 24;
    shadow(&mut s, 14, ground, 11, 2);
    // The stump it stands on.
    for y in 15..ground {
        let flare = (y - (ground - 3)).max(0);
        for x in 10 - flare..18 + flare {
            let across = (x - (10 - flare)) as f32 / (8 + flare * 2) as f32;
            let color = if x == 10 - flare || x == 17 + flare {
                BARK.edge
            } else {
                cylinder(BARK, across)
            };
            put(&mut s, x, y, color);
        }
    }
    // The top, seen a little from above.
    for y in 11..17_i32 {
        for x in 1..27_i32 {
            let (u, v) = (
                (x as f32 + 0.5 - 14.0) / 12.6,
                (y as f32 + 0.5 - 13.5) / 3.0,
            );
            if u * u + v * v > 1.0 {
                continue;
            }
            let color = if u * u + v * v > 0.8 {
                WOOD.edge
            } else if v < -0.1 && u < 0.2 {
                WOOD.light
            } else if (x * 2 + y).rem_euclid(9) == 0 {
                WOOD.shadow
            } else {
                WOOD.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 3, 16, 22, WOOD.edge);
    hline(&mut s, 4, 15, 20, WOOD.shadow);
    // The jug: glass, the cordial in it, a handle and a lip.
    for y in 3..14 {
        let half = if y < 5 {
            2
        } else if y < 7 {
            3
        } else {
            4
        };
        for x in 11 - half..=11 + half {
            let rim = x == 11 - half || x == 11 + half || y == 13;
            let color = if rim {
                GLASS.edge
            } else if y < 6 {
                GLASS.light
            } else if x < 10 {
                CORDIAL.light
            } else if x > 12 {
                CORDIAL.shadow
            } else {
                CORDIAL.base
            };
            put(&mut s, x, y, color);
        }
    }
    hline(&mut s, 9, 3, 5, GLASS.edge);
    put(&mut s, 13, 2, GLASS.edge);
    for y in 6..11 {
        put(&mut s, 16, y, GLASS.edge);
    }
    put(&mut s, 15, 6, GLASS.edge);
    put(&mut s, 15, 10, GLASS.edge);
    vline(&mut s, 8, 7, 4, GLASS.shine);
    // Flower heads floating in it.
    for (x, y) in [(10, 6), (12, 7), (11, 9)] {
        put(&mut s, x, y, ELDER.shine);
    }
    // Two cups, poured.
    for &x in &[3, 19] {
        stamp(
            &mut s,
            (x, 10),
            &["#lll#", "#*oo#", "#los#", ".###."],
            &[
                (b'#', GLASS.edge),
                (b'l', CORDIAL.light),
                (b'*', GLASS.shine),
                (b'o', CORDIAL.base),
                (b's', CORDIAL.shadow),
            ],
            false,
        );
    }
    // A head of elderflower laid by.
    stamp(
        &mut s,
        (19, 14),
        &[".*l*l*.", "lylolyl", ".s.k.s."],
        &[
            (b'*', ELDER.shine),
            (b'l', ELDER.light),
            (b'y', rgb(0xe8c84a)),
            (b'o', ELDER.base),
            (b's', ELDER.shadow),
            (b'k', STALK),
        ],
        false,
    );
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// A young crab apple, staked and tied: a slim trunk, a crown of fresh leaves with pink-white
/// blossom and a few small apples, gold and blushing.
fn crab_apple_sapling() -> Piece {
    let mut s = Canvas::new(30, 46);
    let ground = 44;
    shadow(&mut s, 15, ground, 9, 2);
    // The stake beside it, and the trunk.
    post(&mut s, 18, (20, ground), 2, WOOD, 341);
    for path in [
        &[(14, 43), (14, 30), (13, 20)][..],
        &[(14, 28), (8, 20), (5, 14)],
        &[(14, 26), (20, 18), (24, 14)],
        &[(13, 22), (12, 12), (14, 6)],
    ] {
        bough(&mut s, path, BARK);
    }
    // The tie round trunk and stake.
    hline(&mut s, 14, 31, 6, STRING);
    foliage(
        &mut s,
        &[
            ((6, 13), (6, 5)),
            ((23, 13), (6, 5)),
            ((14, 7), (8, 6)),
            ((14, 17), (8, 5)),
            ((9, 20), (4, 3)),
            ((20, 20), (4, 3)),
        ],
        FRESH,
        342,
    );
    for &(x, y) in &[
        (4, 10),
        (12, 3),
        (19, 6),
        (25, 11),
        (9, 16),
        (17, 15),
        (22, 19),
        (6, 19),
    ] {
        bloom(&mut s, (x, y), BLOSSOM, rgb(0xf0d060));
    }
    for &(x, y) in &[(10, 21), (19, 22), (14, 21), (4, 16)] {
        stamp(
            &mut s,
            (x, y),
            &[".w.", "#rl", "#oo", ".#."],
            &[
                (b'w', rgb(0x5a3a2a)),
                (b'#', APPLE.edge),
                (b'r', BLUSH.light),
                (b'l', APPLE.light),
                (b'o', APPLE.base),
            ],
            false,
        );
    }
    tuft(&mut s, 15, ground, 5, 343);
    Piece {
        sprite: s,
        anchor: (15, ground),
    }
}

/// A wild rose: a rounded bush of small dark leaves with canes arching out of it, pale pink roses
/// with gold middles all over it, and scarlet hips among them.
fn wild_rose() -> Piece {
    let mut s = Canvas::new(32, 30);
    let ground = 28;
    shadow(&mut s, 16, ground, 13, 2);
    foliage(
        &mut s,
        &[
            ((9, 17), (7, 6)),
            ((23, 17), (7, 6)),
            ((16, 11), (8, 6)),
            ((16, 21), (11, 6)),
        ],
        ROSE_LEAF,
        351,
    );
    let cane = Ramp::new(0x2a3a1c, 0x3e5228, 0x566e34, 0x6e8a42, 0x8aa858);
    for path in [
        &[(10, 25), (5, 14), (2, 8), (4, 4)][..],
        &[(22, 25), (27, 15), (30, 9), (28, 5)],
    ] {
        bough(&mut s, path, cane);
    }
    for &(x, y) in &[
        (4, 6),
        (27, 7),
        (10, 9),
        (19, 7),
        (6, 16),
        (14, 14),
        (24, 15),
        (18, 21),
        (9, 21),
        (27, 19),
    ] {
        bloom(&mut s, (x, y), ROSE, rgb(0xf0c840));
    }
    for &(x, y) in &[(12, 18), (21, 12), (3, 12), (25, 22), (15, 24)] {
        stamp(
            &mut s,
            (x, y),
            &["#*", "#o", ".#"],
            &letters(HIP, b"#sol*"),
            false,
        );
    }
    tuft(&mut s, 7, ground, 3, 352);
    tuft(&mut s, 25, ground, 3, 353);
    Piece {
        sprite: s,
        anchor: (16, ground),
    }
}

/// A cushion of wild thyme, low and soft enough to sit on, covered in purple flowers with a bee
/// at work, and a flat warm stone beside it.
fn thyme_cushion() -> Piece {
    let mut s = Canvas::new(30, 16);
    let ground = 13;
    shadow(&mut s, 15, ground, 14, 2);
    // The flat stone, at the right.
    lump(&mut s, 24.0, 11.0, 5.0, 2.4, STONE, 361);
    // The cushion: a low mound of tiny leaves.
    model(&mut s, THYME, ROUND, 70, 362, |x, y| {
        in_ellipse(x, y, (12, 9), (11, 5))
    });
    // Flowers all over its top.
    for index in 0..34 {
        let x = 2 + (noise(index, 0, 363) % 21) as i32;
        let y = 5 + (noise(index, 1, 363) % 6) as i32;
        let (u, v) = ((x - 12) as f32 / 11.0, (y - 9) as f32 / 5.0);
        if u * u + v * v > 0.9 || v > 0.4 {
            continue;
        }
        let color = if u + v < -0.5 {
            THYME_FLOWER.shine
        } else if u + v < 0.0 {
            THYME_FLOWER.light
        } else {
            THYME_FLOWER.base
        };
        put(&mut s, x, y, color);
        put(&mut s, x + 1, y, THYME_FLOWER.shadow);
    }
    // A bumblebee.
    stamp(
        &mut s,
        (15, 2),
        &[".ww.", "bgbg", ".bb."],
        &[(b'w', rgba(0xe8f4f8, 200)), (b'b', BEE), (b'g', BEE_GOLD)],
        false,
    );
    Piece {
        sprite: s,
        anchor: (14, ground),
    }
}

/// The golden toadstool: a great chanterelle big enough to sit on, its cap a wide golden cup
/// with a wavy rim, the ridges under it running down into a short stout stalk, and smaller ones
/// coming up round its foot.
fn golden_toadstool() -> Piece {
    let mut s = Canvas::new(32, 28);
    let ground = 26;
    shadow(&mut s, 16, ground, 11, 2);
    // The underside of the cap narrowing into the stalk, and the stalk down to the grass.
    model(&mut s, GOLD, UPRIGHT, 24, 371, |x, y| {
        let half = if y < 15 {
            12.0 - (y - 9) as f32 * 1.3
        } else {
            4.4 + (y - 15).max(0) as f32 * 0.08 + if y >= ground - 2 { 1.0 } else { 0.0 }
        };
        (9..ground).contains(&y) && (x as f32 + 0.5 - 16.0).abs() <= half
    });
    // Its ridges, running down from the rim into the stalk.
    for x in [6, 9, 12, 15, 18, 21, 24] {
        let points = trace(&[(x, 10), (16 + (x - 16) / 3, 16)]);
        for &(px, py) in &points {
            if s.get(px, py).a > 0 {
                put(&mut s, px, py, GOLD.shadow);
            }
        }
    }
    // The cap's top, a wide shallow cup seen a little from above, its rim wavy.
    for y in 2..12 {
        for x in 1..31 {
            let (u, v) = ((x as f32 + 0.5 - 16.0) / 14.6, (y as f32 + 0.5 - 7.0) / 4.0);
            let wave = ((x as f32) * 1.1).sin() * 0.1;
            let reach = u * u + v * v;
            if reach > 1.0 + wave {
                continue;
            }
            let rim = reach > 0.72 + wave;
            let hollow = u * u + (v + 0.15).powi(2) * 1.4 < 0.42;
            let color = if rim {
                if v < 0.1 && u < 0.4 {
                    GOLD.shine
                } else if v < 0.3 {
                    GOLD.light
                } else {
                    GOLD.edge
                }
            } else if hollow {
                if u - v < -0.3 { GOLD.shadow } else { GOLD.base }
            } else {
                GOLD.light
            };
            put(&mut s, x, y, color);
        }
    }
    // Little ones round its foot.
    for &(x, y) in &[(4, 23), (24, 24), (27, 21)] {
        stamp(
            &mut s,
            (x, y),
            &["#*l#", ".lo.", ".#s."],
            &letters(GOLD, b"#sol*"),
            false,
        );
    }
    tuft(&mut s, 16, ground, 6, 372);
    Piece {
        sprite: s,
        anchor: (16, ground),
    }
}

/// A honeysuckle bower: two rustic posts with a branch bent over between them, honeysuckle
/// twining up and over it with its leaves in pairs and its flowers out, cream and pink, and a
/// little bench under it to sit on in the evening.
fn honeysuckle_bower() -> Piece {
    let mut s = Canvas::new(40, 48);
    let ground = 46;
    shadow(&mut s, 20, ground, 18, 2);
    post(&mut s, 3, (14, ground), 3, RUSTIC, 381);
    post(&mut s, 34, (14, ground), 3, RUSTIC, 382);
    // The arch over the top, a bent branch.
    let arch: Vec<(i32, i32)> = (0..=36)
        .map(|step| {
            let t = step as f32 / 36.0;
            let x = 4.0 + t * 31.0;
            let y = 15.0 - (t * std::f32::consts::PI).sin() * 12.0;
            (x.round() as i32, y.round() as i32)
        })
        .collect();
    bough(&mut s, &arch, RUSTIC);
    // The bench under it.
    plank(&mut s, (9, 37), (22, 3), WOOD, 383);
    for x in [11, 27] {
        vline(&mut s, x, 40, ground - 40, WOOD.shadow);
        vline(&mut s, x + 1, 40, ground - 40, WOOD.edge);
    }
    // Honeysuckle twining up both posts and along the arch.
    let twine = |s: &mut Canvas, points: &[(i32, i32)], salt: u32| {
        for (index, &(x, y)) in points.iter().enumerate() {
            let wobble = ((index as f32) * 0.8 + salt as f32).sin().round() as i32;
            put(s, x + wobble, y, rgb(0x76603e));
            if index % 4 == 0 {
                stamp(
                    s,
                    (x + wobble - 3, y - 1),
                    &["Gg...gG", ".dg.gd."],
                    &[(b'G', FRESH.light), (b'g', FRESH.base), (b'd', FRESH.edge)],
                    false,
                );
            }
        }
    };
    let left: Vec<(i32, i32)> = (14..ground - 1).map(|y| (4, y)).collect();
    let right: Vec<(i32, i32)> = (14..ground - 1).map(|y| (35, y)).collect();
    twine(&mut s, &left, 3);
    twine(&mut s, &right, 7);
    twine(&mut s, &arch, 11);
    // Its flowers, here and there along it.
    let flowers = [
        (6, 10),
        (12, 5),
        (20, 3),
        (28, 5),
        (33, 10),
        (3, 22),
        (36, 26),
        (5, 32),
        (34, 18),
    ];
    for &(x, y) in &flowers {
        stamp(
            &mut s,
            (x - 2, y - 2),
            &["*...*", ".lpl.", "*.p.l", ".lpl."],
            &[
                (b'*', SUCKLE.shine),
                (b'l', SUCKLE.light),
                (b'p', SUCKLE_PINK.base),
            ],
            false,
        );
    }
    tuft(&mut s, 4, ground, 3, 384);
    tuft(&mut s, 35, ground, 3, 385);
    Piece {
        sprite: s,
        anchor: (20, ground),
    }
}

/// The clover fountain: a stone basin shaped like a four-leaf clover, seen a little from above,
/// on a short pedestal with a clover carved on its front; a little bowl on a column in the
/// middle, water spilling from it into the basin, and the water catching the light.
fn clover_fountain() -> Piece {
    let mut s = Canvas::new(44, 42);
    let ground = 40;
    shadow(&mut s, 22, ground, 19, 2);
    // The pedestal under the basin.
    for y in 28..ground {
        let flare = (y - (ground - 3)).max(0);
        for x in 14 - flare..30 + flare {
            let across = (x - (14 - flare)) as f32 / (16 + flare * 2) as f32;
            let color = if x == 14 - flare || x == 29 + flare || y == ground - 1 {
                STONE.edge
            } else {
                cylinder(STONE, across)
            };
            put(&mut s, x, y, color);
        }
    }
    // A clover carved on its front.
    stamp(
        &mut s,
        (19, 31),
        &[".#.#.", "#o#o#", ".#k#.", "#o#o#", ".#.#."],
        &[
            (b'#', STONE.edge),
            (b'o', CLOVER.light),
            (b'k', STONE.shadow),
        ],
        false,
    );
    // The basin: four round lobes, a clover from above, with a rim and water in it.
    let lobes = [(14.0, 20.0), (30.0, 20.0), (22.0, 16.0), (22.0, 25.0)];
    let in_basin = |x: i32, y: i32, grow: f32| {
        lobes.iter().any(|&(cx, cy)| {
            let (u, v) = (
                (x as f32 + 0.5 - cx) / (9.0 + grow),
                (y as f32 + 0.5 - cy) / (5.0 + grow * 0.6),
            );
            u * u + v * v <= 1.0
        })
    };
    for y in 8..34 {
        for x in 1..43 {
            if !in_basin(x, y, 0.0) {
                continue;
            }
            let water = in_basin(x, y, -2.0);
            let color = if water {
                let ripple = ((x + y * 2) as f32 * 0.7).sin() > 0.6;
                if ripple {
                    WATER.light
                } else if y < 19 {
                    WATER.base
                } else {
                    WATER.shadow
                }
            } else if in_basin(x, y + 1, 0.0) && !in_basin(x, y - 1, 0.0) {
                STONE.shine
            } else if y > 22 {
                STONE.shadow
            } else {
                STONE.light
            };
            put(&mut s, x, y, color);
        }
    }
    // The basin's outer side, below its rim.
    for x in 1..43 {
        let mut bottom = None;
        for y in (8..34).rev() {
            if in_basin(x, y, 0.0) {
                bottom = Some(y);
                break;
            }
        }
        if let Some(y) = bottom {
            put(&mut s, x, y + 1, STONE.base);
            put(&mut s, x, y + 2, STONE.shadow);
            put(&mut s, x, y + 3, STONE.edge);
        }
    }
    // The column and the little bowl in the middle, and the water spilling.
    post(&mut s, 20, (10, 22), 4, STONE, 391);
    for y in 7..10 {
        let half = 6 - (y - 7);
        hline(
            &mut s,
            22 - half,
            y,
            half * 2,
            if y == 7 { STONE.light } else { STONE.base },
        );
    }
    hline(&mut s, 16, 6, 12, STONE.edge);
    hline(&mut s, 17, 7, 10, WATER.light);
    for (x, top) in [(15, 8), (28, 8)] {
        for y in top..top + 9 {
            put(&mut s, x, y, rgba(0xc8eaf8, 180));
        }
    }
    // The jet, and sparkles.
    vline(&mut s, 22, 1, 6, rgba(0xe8f6fc, 200));
    put(&mut s, 21, 2, rgba(0xffffff, 160));
    put(&mut s, 23, 3, rgba(0xffffff, 160));
    for &(x, y) in &[(10, 18), (33, 21), (22, 27), (26, 15)] {
        put(&mut s, x, y, WATER.shine);
    }
    // Moss and flowers about its foot.
    tuft(&mut s, 12, ground, 3, 392);
    tuft(&mut s, 32, ground, 3, 393);
    flowers(&mut s, 34, ground - 1, 2, 2, 394);
    for y in 28..ground {
        for x in 14..30 {
            if s.get(x, y).a == 255 && chance(x, y, 395, 12) {
                put(&mut s, x, y, rgb(0xc8c070));
            }
        }
    }
    Piece {
        sprite: s,
        anchor: (22, ground),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::FORAGED;
    use crate::finds::art::PIECE_MAX;

    #[test]
    fn every_find_from_the_hedgerow_is_drawn_and_fits_any_spot() {
        for find in &FORAGED {
            let icon = icon(find.id).unwrap_or_else(|| panic!("{} has no icon", find.id));
            assert_eq!((icon.width(), icon.height()), (ICON, ICON));
            let piece = piece(find.id).unwrap_or_else(|| panic!("{} has no piece", find.id));
            let (width, height) = (piece.sprite.width(), piece.sprite.height());
            assert!(
                width <= PIECE_MAX.0 && height <= PIECE_MAX.1,
                "{} is {width}x{height}",
                find.id
            );
            // It stands on its anchor: something drawn on or just above the ground there.
            let (x, y) = piece.anchor;
            let footing = (y - 3..=y).any(|row| piece.sprite.get(x, row).a > 0);
            assert!(footing, "{} doesn't stand on its anchor", find.id);
        }
    }
}
