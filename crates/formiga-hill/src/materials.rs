//! The Hill's palette: every material as a ramp of tones, and the colours of sky, grass, glass
//! and flowers. Shadows lean cool and purple and highlights warm, so nothing goes muddy in shade.

use crate::paint::{Ramp, rgb};
use formiga_art::Rgba;

pub const PLASTER: Ramp = Ramp::new(0xa48d73, 0xd8c6a4, 0xece0c2, 0xf6eed8, 0xfffaf0);
pub const TIMBER: Ramp = Ramp::new(0x34241f, 0x523629, 0x6c4a37, 0x876045, 0xa57b58);
pub const TILE: Ramp = Ramp::new(0x612a2b, 0x8c3d39, 0xb05445, 0xc96c53, 0xe08c69);
pub const STONE: Ramp = Ramp::new(0x605a5d, 0x857d7c, 0xa69d94, 0xc2baae, 0xdcd5c8);
pub const BRICK: Ramp = Ramp::new(0x5a2a28, 0x7c3a33, 0x98503f, 0xb0674e, 0xc98463);
pub const PLANK: Ramp = Ramp::new(0x6f4f37, 0x9b734d, 0xbb9061, 0xd0a874, 0xe2c08c);
pub const IRON: Ramp = Ramp::new(0x1c2a28, 0x2b423c, 0x3b5a50, 0x52796a, 0x70998a);
pub const TRIM: Ramp = Ramp::new(0x9c958a, 0xcdc6b8, 0xe9e3d6, 0xf6f2ea, 0xffffff);
pub const LEAF: Ramp = Ramp::new(0x2c5233, 0x3d6e45, 0x518a55, 0x6fa866, 0x93c67e);
pub const SLEEPER: Ramp = Ramp::new(0x3a2a22, 0x553e30, 0x6a4f3d, 0x80634d, 0x957a61);
pub const RAIL: Ramp = Ramp::new(0x3f4249, 0x5d626b, 0x80858e, 0xb2b7bf, 0xe3e7ec);
pub const LEATHER: Ramp = Ramp::new(0x3e2419, 0x5e3624, 0x7b4a31, 0x96603f, 0xb27b52);
pub const DOOR: Ramp = Ramp::new(0x1e322d, 0x2f4a42, 0x3f6155, 0x557c6c, 0x6f9884);

pub const SKY_TOP: Rgba = rgb(0xb9ddec);
pub const SKY_LOW: Rgba = rgb(0xf6e8cf);
pub const CLOUD: Rgba = rgb(0xfdfbf5);
pub const FAR_HILL: Rgba = rgb(0xa9cfa4);
pub const HILL: Rgba = rgb(0x86bb7c);
pub const HILL_SHADE: Rgba = rgb(0x72a569);
pub const PATH: Rgba = rgb(0xe6d3a4);
pub const GRASS: Rgba = rgb(0x7db36c);
pub const GRASS_DARK: Rgba = rgb(0x639858);
pub const GRASS_LIGHT: Rgba = rgb(0x97c784);
pub const TRUNK: Rgba = rgb(0x7a5a3a);
pub const LEAVES: Rgba = rgb(0x5d9a5a);
pub const GLASS: [Rgba; 4] = [rgb(0x4f6f80), rgb(0x7499a8), rgb(0xa6c8d2), rgb(0xe9f6f7)];
pub const GLOW: [Rgba; 3] = [rgb(0xf0b45a), rgb(0xffd77a), rgb(0xfff0bd)];
pub const ENAMEL: Rgba = rgb(0x2f5d50);
pub const ENAMEL_DEEP: Rgba = rgb(0x1f4238);
pub const CREAM: Rgba = rgb(0xf3e9cf);
pub const BLOSSOMS: [Rgba; 5] = [
    rgb(0xe0605a),
    rgb(0xf19bb0),
    rgb(0xf5d25e),
    rgb(0xfbf6ee),
    rgb(0xb79be0),
];
pub const BALLAST: [Rgba; 5] = [
    rgb(0x6b625d),
    rgb(0x857b74),
    rgb(0x9c928a),
    rgb(0xb3aaa0),
    rgb(0x95806c),
];
