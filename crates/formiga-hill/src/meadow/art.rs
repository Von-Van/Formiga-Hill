//! How each bug looks: in the meadow, at every frame of flying, settling and crawling; close up,
//! held up in the net to be looked at; and as an icon in the journal.
//!
//! In the meadow a bug is a handful of pixels across, so each frame is placed pixel by pixel, as
//! the minnow is: a strong silhouette in the bug's own colours, outlined in a darker shade of
//! them, lit from the upper left, with the one or two marks that tell it apart at a glance (the
//! admiral's red bands, the cabbage white's black tips, the stag beetle's antlers). Every bug
//! faces right, and the mechanics mirror it. The meadow is seen from a little above, so a
//! butterfly basking with its wings open shows its far pair above its body and its near pair
//! below; with its wings shut they stand up over its body, edge on, showing their undersides.
//!
//! Close up, a bug is big enough to be built like a specimen in a case: butterflies and moths
//! with their wings spread, beetles and dragonflies from above, the grasshopper from the side
//! for the sake of its legs. Bodies are shaded as rounded things and wings as flat ones, both
//! from the upper left and textured with noise, and markings are placed on a wing by where they
//! fall on it rather than at fixed pixels.

use super::bugs::bug;
use crate::paint::{Ramp, chance, line, noise, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};

/// Icons are this many pixels square, as finds' and fish's are.
pub const ICON: u32 = 9;

/// What a bug is doing, which decides how it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    /// In the air, wings beating; for the grasshopper, mid-hop with its legs out.
    Flying,
    /// Settled on a perch. For butterflies and moths, frame 0 has the wings open and frame 1
    /// shut; for the grasshopper, frame 1 is chirping, a hind leg raised.
    Settled,
    /// Walking along a perch, legs going. A bug that never walks stays as it settled.
    Crawling,
}

/// How many frames a bug has for a pose; the mechanics cycle through them.
pub fn frames(id: &str, pose: Pose) -> usize {
    look(id).map_or(1, |look| look.frames(pose).len())
}

/// A bug as it is seen in the meadow, facing right, at `frame` of its pose. Every frame of
/// every pose of a bug is the same size, so nothing jumps as it moves.
pub fn sprite(id: &str, pose: Pose, frame: usize) -> Canvas {
    let Some(look) = look(id) else {
        return plain_sprite();
    };
    let frames = look.frames(pose);
    let mut canvas = Canvas::new(look.size.0, look.size.1);
    stamp(&mut canvas, (0, 0), frames[frame % frames.len()], look.inks);
    canvas
}

/// Where in a sprite the bug itself is: the middle of its body in flight, which follows its
/// path, and where its feet meet the perch when it is settled or crawling.
pub fn anchor(id: &str, pose: Pose) -> (i32, i32) {
    match (look(id), pose) {
        (Some(look), Pose::Flying) => look.middle,
        (Some(look), _) => look.feet,
        (None, Pose::Flying) => (3, 2),
        (None, _) => (3, 4),
    }
}

/// The bug close up, as it is held up in the net to be looked at before it is let go.
pub fn close_up(id: &str) -> Canvas {
    if bug(id).is_none() {
        return plain_close_up();
    }
    match id {
        "cabbage_white" => cabbage_white(),
        "ladybird" => ladybird(),
        "grasshopper" => grasshopper(),
        "bumblebee" => bumblebee(),
        "red_admiral" => red_admiral(),
        "damselfly" => damselfly(),
        "rose_chafer" => rose_chafer(),
        "firefly" => firefly(),
        "stag_beetle" => stag_beetle(),
        "emperor_dragonfly" => emperor_dragonfly(),
        "moon_moth" => moon_moth(),
        _ => plain_close_up(),
    }
}

/// The bug's icon for the journal, nine pixels square.
pub fn icon(id: &str) -> Canvas {
    let Some((rows, inks)) = icon_rows(id) else {
        return plain_icon();
    };
    let mut canvas = Canvas::new(ICON, ICON);
    stamp(&mut canvas, (0, 0), rows, inks);
    canvas
}

// ---------------------------------------------------------------------------------------------
// In the meadow
// ---------------------------------------------------------------------------------------------

/// Rows of letters, one frame of a sprite; each letter is a colour from its bug's inks.
type Grid = &'static [&'static str];

/// The colour each letter of a grid stands for.
type Inks = &'static [(u8, Rgba)];

/// A bug's sprites in the meadow: every frame of each pose, all one size, and where it is held.
struct Look {
    size: (u32, u32),
    flying: &'static [Grid],
    settled: &'static [Grid],
    /// Empty for a bug that never walks: it stays as it settled.
    crawling: &'static [Grid],
    /// The middle of its body, which follows its path through the air.
    middle: (i32, i32),
    /// Where its feet meet the perch.
    feet: (i32, i32),
    inks: Inks,
}

impl Look {
    fn frames(&self, pose: Pose) -> &'static [Grid] {
        match pose {
            Pose::Flying => self.flying,
            Pose::Settled => self.settled,
            Pose::Crawling if self.crawling.is_empty() => &self.settled[..1],
            Pose::Crawling => self.crawling,
        }
    }
}

fn look(id: &str) -> Option<&'static Look> {
    bug(id)?;
    Some(match id {
        "cabbage_white" => &CABBAGE_WHITE,
        "ladybird" => &LADYBIRD,
        "grasshopper" => &GRASSHOPPER,
        "bumblebee" => &BUMBLEBEE,
        "red_admiral" => &RED_ADMIRAL,
        "damselfly" => &DAMSELFLY,
        "rose_chafer" => &ROSE_CHAFER,
        "firefly" => &FIREFLY_LOOK,
        "stag_beetle" => &STAG_BEETLE,
        "emperor_dragonfly" => &EMPEROR,
        "moon_moth" => &MOON_MOTH,
        _ => return None,
    })
}

/// Paints rows of letters with their top-left at `at`, each letter a colour from `inks`; a
/// letter not in `inks` is left clear.
fn stamp(s: &mut Canvas, at: (i32, i32), rows: &[&str], inks: &[(u8, Rgba)]) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, letter) in row.bytes().enumerate() {
            if let Some(&(_, color)) = inks.iter().find(|(name, _)| *name == letter) {
                put(s, at.0 + dx as i32, at.1 + dy as i32, color);
            }
        }
    }
}

/// A ramp's tones under five letters, darkest first.
const fn tones(ramp: Ramp, names: [u8; 5]) -> [(u8, Rgba); 5] {
    [
        (names[0], ramp.edge),
        (names[1], ramp.shadow),
        (names[2], ramp.base),
        (names[3], ramp.light),
        (names[4], ramp.shine),
    ]
}

/// Five letters' inks and any number more, as one list.
const fn inks<const N: usize, const M: usize>(
    first: [(u8, Rgba); 5],
    rest: [(u8, Rgba); N],
) -> [(u8, Rgba); M] {
    let mut all = [(0, Rgba::new(0, 0, 0, 0)); M];
    let mut index = 0;
    while index < 5 {
        all[index] = first[index];
        index += 1;
    }
    while index < M {
        all[index] = rest[index - 5];
        index += 1;
    }
    all
}

// The colours. Each material is a ramp, darkest first; a bug's black is a dark shade of its own
// warm or cool colour rather than black, so the creatures, outlined in near-black, read first.

const WHITE_WING: Ramp = Ramp::new(0x8c889a, 0xc8c5d0, 0xe9e7e1, 0xf7f5ef, 0xffffff);
const WING_TIP: Ramp = Ramp::new(0x262229, 0x302b34, 0x3e3843, 0x564f5c, 0x726a78);
const WHITE_UNDER: Ramp = Ramp::new(0x8a8a58, 0xc4c486, 0xe0dea6, 0xf0eec6, 0xfcfce6);
const DUSKY: Ramp = Ramp::new(0x252229, 0x34303a, 0x48444f, 0x68646e, 0x9c98a2);

const LADY_RED: Ramp = Ramp::new(0x6e1616, 0xa62420, 0xd8382a, 0xf06a4a, 0xffd2bc);
const PITCH: Ramp = Ramp::new(0x1f1a1f, 0x2b262c, 0x39333a, 0x534b55, 0x7c727e);
const CHEEK: Rgba = rgb(0xf2eee4);

const HOPPER: Ramp = Ramp::new(0x33561b, 0x5c8c26, 0x8aba34, 0xb2da58, 0xdcf29a);
const STRAW: Ramp = Ramp::new(0x56461f, 0x7a6432, 0x9a8048, 0xbea062, 0xdcc48c);
const HOPPER_EYE: Rgba = rgb(0x5a3418);

const FUR: Ramp = Ramp::new(0x1e1a1c, 0x2a2427, 0x3a3236, 0x564b50, 0x7a6c72);
const POLLEN: Ramp = Ramp::new(0x8a6010, 0xc8941e, 0xf2c232, 0xffe27a, 0xfff6c8);
const BUFF: Ramp = Ramp::new(0x9a9286, 0xcec6b8, 0xeee6d8, 0xfaf6ec, 0xffffff);

const VELVET: Ramp = Ramp::new(0x1e1519, 0x2c2026, 0x3c2c33, 0x58444c, 0x786068);
const ADMIRAL_RED: Ramp = Ramp::new(0x7a1c14, 0xb02a1c, 0xe0442c, 0xf47a52, 0xffbc9c);
const CHALK: Rgba = rgb(0xf6f2e8);
const BARK_UNDER: Ramp = Ramp::new(0x3c2e28, 0x5a4a40, 0x7a6656, 0x9a8676, 0xbcaa9a);
const ADMIRAL_BLUE: Rgba = rgb(0x6c8ed4);

const AZURE: Ramp = Ramp::new(0x16467a, 0x2468a8, 0x3a92d6, 0x6cbaf2, 0xc2e8ff);
const RING: Rgba = rgb(0x1c2438);

const EMPEROR_BLUE: Ramp = Ramp::new(0x173f74, 0x2262a6, 0x3a8ad4, 0x6cb4f0, 0xc4e8ff);
const APPLE: Ramp = Ramp::new(0x2a5a1c, 0x46862a, 0x6eb43a, 0x98d45a, 0xd2f2a0);
const DORSAL: Rgba = rgb(0x1c2642);
const COMPOUND: Ramp = Ramp::new(0x1a4a64, 0x246e8c, 0x3a9ab4, 0x6cc4d8, 0xd0f4fa);
const STIGMA: Rgba = rgb(0x8a6a34);

const CHAFER: Ramp = Ramp::new(0x173f33, 0x23704a, 0x45a03c, 0x98cc48, 0xf6e88a);
const CHAFER_LEG: Ramp = Ramp::new(0x22301e, 0x34482a, 0x4c6436, 0x6c8446, 0x98a862);
const FLECK: Rgba = rgb(0xeef2e2);

const FIREFLY: Ramp = Ramp::new(0x3a2416, 0x5a3a22, 0x7a5434, 0x9c7048, 0xc49a6c);
const SHIELD: Ramp = Ramp::new(0x8a4a30, 0xc07450, 0xe6a070, 0xf6c494, 0xfff0d8);
const LANTERN: Ramp = Ramp::new(0x8a9a3e, 0xc0d064, 0xe2f08c, 0xf4fcba, 0xffffe6);

const STAG: Ramp = Ramp::new(0x2e1610, 0x52261a, 0x7a3a24, 0x9c5434, 0xd08a5a);
const STAG_BLACK: Ramp = Ramp::new(0x170f11, 0x271c1f, 0x372a2d, 0x514145, 0x7e6a6e);
const ANTLER: Ramp = Ramp::new(0x401b10, 0x6c3018, 0x964824, 0xba6834, 0xe29c62);

const LUNA: Ramp = Ramp::new(0x4c7a58, 0x84b894, 0xaedcb4, 0xd0f2d4, 0xf2fff2);
const MAROON: Ramp = Ramp::new(0x4a1e36, 0x6a2a4a, 0x8a3a5e, 0xae5a7c, 0xd08aa4);
const EYESPOT: Ramp = Ramp::new(0x8a6a1a, 0xc09a30, 0xe8c84a, 0xf6e27a, 0xfff6c0);
const DOWN: Ramp = Ramp::new(0xa8a298, 0xd2ccc2, 0xeeeae2, 0xfaf8f2, 0xffffff);
const FEATHER: Ramp = Ramp::new(0x6a5424, 0x96783a, 0xbc9a50, 0xd8b870, 0xf0dca0);

/// Clear wings, and the blur of beating ones: pale, and mostly see-through.
const GLASS: Rgba = rgba(0xe8f4fa, 130);
const VEIN: Rgba = rgba(0x7c98aa, 210);
const BLUR: Rgba = rgba(0xeef6fa, 105);
const BLUR_EDGE: Rgba = rgba(0xc6d8e2, 150);

const BLURS: [(u8, Rgba); 2] = [(b'F', BLUR_EDGE), (b'f', BLUR)];

// The cabbage white, seen from a little above: its far wings over its body, its near wings
// under it, black at each forewing's tip.

static CABBAGE_WHITE: Look = Look {
    size: (11, 9),
    flying: &[&WHITE_UP, &WHITE_OPEN, &WHITE_DOWN, &WHITE_OPEN],
    settled: &[&WHITE_OPEN, &WHITE_SHUT],
    crawling: &[],
    middle: (5, 4),
    feet: (6, 5),
    inks: &WHITE_INKS,
};

const WHITE_INKS: [(u8, Rgba); 17] = inks(
    tones(WHITE_WING, *b"#sol*"),
    [
        (b'K', WING_TIP.edge),
        (b'k', WING_TIP.base),
        (b'Y', WHITE_UNDER.edge),
        (b'v', WHITE_UNDER.shadow),
        (b'y', WHITE_UNDER.base),
        (b'u', WHITE_UNDER.light),
        (b'B', DUSKY.edge),
        (b'b', DUSKY.base),
        (b'h', DUSKY.light),
        (b'a', DUSKY.shadow),
        (b'c', CHEEK),
        (b'g', DUSKY.edge),
    ],
);

const WHITE_OPEN: [&str; 9] = [
    "........KK.",
    "..##...#kK.",
    ".#lo#.#lo#c",
    ".#os##os#a.",
    ".Bbbbbhbb..",
    "..#o#.#ok#.",
    ".......kK..",
    "...........",
    "...........",
];

const WHITE_SHUT: [&str; 9] = [
    ".......YY..",
    ".....YuuY..",
    "....YuykY.c",
    "...YyyvvYa.",
    ".Bbbbbhbb..",
    ".....g.g...",
    "...........",
    "...........",
    "...........",
];

const WHITE_UP: [&str; 9] = [
    "......KK...",
    ".....#kK...",
    ".....#lo#.c",
    "....#loo#a.",
    ".Bbbbbhbb..",
    "...........",
    "...........",
    "...........",
    "...........",
];

const WHITE_DOWN: [&str; 9] = [
    "...........",
    "...........",
    "..........c",
    ".........a.",
    ".Bbbbbhbb..",
    "....#oss#..",
    ".....#os#..",
    ".....#kK...",
    "......KK...",
];

// The ladybird: a red dome with its spots, a black head with white cheeks.

static LADYBIRD: Look = Look {
    size: (6, 6),
    flying: &[&LADYBIRD_FLY_UP, &LADYBIRD_FLY_BACK],
    settled: &[&LADYBIRD_STILL],
    crawling: &[&LADYBIRD_STEP, &LADYBIRD_STILL],
    middle: (2, 3),
    feet: (3, 5),
    inks: &LADYBIRD_INKS,
};

const LADYBIRD_INKS: [(u8, Rgba); 12] = inks(
    tones(LADY_RED, *b"Esol*"),
    [
        (b'k', PITCH.base),
        (b'h', PITCH.base),
        (b'H', PITCH.edge),
        (b'w', CHEEK),
        (b'g', PITCH.edge),
        BLURS[0],
        BLURS[1],
    ],
);

const LADYBIRD_STILL: [&str; 6] = ["......", ".sss..", "sl*kE.", "skoohw", ".EEEhH", ".g.g.g"];

const LADYBIRD_STEP: [&str; 6] = ["......", ".sss..", "sl*kE.", "skoohw", ".EEEhH", "g.g.g."];

const LADYBIRD_FLY_UP: [&str; 6] = [".fFf..", "fsssf.", "sl*kE.", "skoohw", ".EEEhH", "..g.g."];

const LADYBIRD_FLY_BACK: [&str; 6] = ["......", "Fsss..", "fl*kE.", "skoohw", ".EEEhH", "..g.g."];

// The grasshopper: long and green, wings folded brown along its back, and the big hind leg
// that makes it, knee up above its back.

static GRASSHOPPER: Look = Look {
    size: (12, 7),
    flying: &[&HOPPER_LEAP, &HOPPER_SAIL],
    settled: &[&HOPPER_STILL, &HOPPER_CHIRP],
    crawling: &[],
    middle: (6, 4),
    feet: (6, 6),
    inks: &HOPPER_INKS,
};

const HOPPER_INKS: [(u8, Rgba); 13] = inks(
    tones(HOPPER, *b"Gsol*"),
    [
        (b'B', STRAW.edge),
        (b'b', STRAW.base),
        (b'c', STRAW.light),
        (b'e', HOPPER_EYE),
        (b'a', HOPPER.shadow),
        (b'k', HOPPER.edge),
        BLURS[0],
        BLURS[1],
    ],
);

const HOPPER_STILL: [&str; 7] = [
    "............",
    "..........a.",
    "..GG.....a..",
    ".GllGBBGGG..",
    ".GbllGbllleG",
    "G.ssslGsooG.",
    "k....k.k.k..",
];

const HOPPER_CHIRP: [&str; 7] = [
    "............",
    ".GG.......a.",
    "GllG.....a..",
    "GsllGBBGGG..",
    "GBbllGbllleG",
    "..ssslGsooG.",
    ".....k.k.k..",
];

const HOPPER_LEAP: [&str; 7] = [
    "............",
    "..........a.",
    ".........a..",
    "..BBBBBGGG..",
    ".BbccbbllleG",
    ".GlllsssooGk",
    "k...........",
];

const HOPPER_SAIL: [&str; 7] = [
    "............",
    "...FFFF...a.",
    "..FffffF.a..",
    "..BBBBBGGG..",
    ".BbccbbllleG",
    ".GlllsssooGk",
    "k...........",
];

// The bumblebee: round and furred, black with a yellow collar and band and a white tail.

static BUMBLEBEE: Look = Look {
    size: (9, 8),
    flying: &[&BEE_WINGS_UP, &BEE_WINGS_BACK],
    settled: &[&BEE_STILL],
    crawling: &[],
    middle: (4, 4),
    feet: (5, 7),
    inks: &BEE_INKS,
};

const BEE_INKS: [(u8, Rgba); 15] = inks(
    tones(FUR, *b"Kgkj+"),
    [
        (b'Y', POLLEN.edge),
        (b'y', POLLEN.base),
        (b'u', POLLEN.light),
        (b'W', BUFF.edge),
        (b'w', BUFF.base),
        (b'x', BUFF.light),
        (b'z', GLASS),
        (b'Z', VEIN),
        BLURS[0],
        BLURS[1],
    ],
);

const BEE_STILL: [&str; 8] = [
    ".........",
    "..ZzzZ...",
    "..KYKKYK.",
    ".WkujjukK",
    "WxkykkykK",
    "WwkykkYK.",
    ".WKYKKK..",
    "...g.g.g.",
];

const BEE_WINGS_UP: [&str; 8] = [
    "..fFFf...",
    "...FFf...",
    "..KYKKYK.",
    ".WkujjukK",
    "WxkykkykK",
    "WwkykkYK.",
    ".WKYKKK..",
    "..g..g...",
];

const BEE_WINGS_BACK: [&str; 8] = [
    ".........",
    "fFFFf....",
    "..KYKKYK.",
    ".WkujjukK",
    "WxkykkykK",
    "WwkykkYK.",
    ".WKYKKK..",
    "..g..g...",
];

// The red admiral: velvet black, a red band across each forewing and red along the hindwings'
// edge, white spots at the tips. Shut, the mottled undersides show, the red band peeping out.

static RED_ADMIRAL: Look = Look {
    size: (13, 11),
    flying: &[&ADMIRAL_UP, &ADMIRAL_OPEN, &ADMIRAL_DOWN, &ADMIRAL_OPEN],
    settled: &[&ADMIRAL_OPEN, &ADMIRAL_SHUT],
    crawling: &[],
    middle: (6, 5),
    feet: (7, 6),
    inks: &ADMIRAL_INKS,
};

const ADMIRAL_INKS: [(u8, Rgba); 19] = inks(
    tones(VELVET, *b"#vkj+"),
    [
        (b'R', ADMIRAL_RED.edge),
        (b'r', ADMIRAL_RED.base),
        (b'q', ADMIRAL_RED.light),
        (b'w', CHALK),
        (b'B', VELVET.edge),
        (b'b', VELVET.light),
        (b'h', VELVET.shine),
        (b'a', VELVET.light),
        (b'c', CHALK),
        (b'o', BARK_UNDER.shadow),
        (b'M', BARK_UNDER.edge),
        (b'm', BARK_UNDER.base),
        (b'n', BARK_UNDER.light),
        (b'u', ADMIRAL_BLUE),
    ],
);

const ADMIRAL_OPEN: [&str; 11] = [
    ".........##..",
    "..RR....#wk#.",
    ".RrrR..Rrrr#.",
    ".Rkjk##kjk#..",
    "..#kkk#kk#...",
    ".BbbbbbhhbBac",
    "..#jkk#jk#...",
    ".RrrR.Rrrr#..",
    "..RR...#w#...",
    "........#....",
    ".............",
];

const ADMIRAL_SHUT: [&str; 11] = [
    ".......##....",
    "......#wk#...",
    ".....MMrq#...",
    "....MnmMr#..c",
    "....MmomM#.a.",
    ".BbbbbbhhbB..",
    "......M.M....",
    ".............",
    ".............",
    ".............",
    ".............",
];

const ADMIRAL_UP: [&str; 11] = [
    ".......##....",
    "......#wk#...",
    ".....Rrrk#...",
    "....RqRrr#..c",
    "....#kjkv#.a.",
    ".BbbbbbhhbB..",
    ".............",
    ".............",
    ".............",
    ".............",
    ".............",
];

const ADMIRAL_DOWN: [&str; 11] = [
    ".............",
    ".............",
    ".............",
    "............c",
    "...........a.",
    ".BbbbbbhhbB..",
    "....#kjkv#...",
    "....RqRrr#...",
    ".....Rrrk#...",
    "......#wk#...",
    ".......##....",
];

// The damselfly: a needle of blue, ringed in black, its wings folded along its back.

static DAMSELFLY: Look = Look {
    size: (13, 7),
    flying: &[&DAMSEL_BLUR_UP, &DAMSEL_BLUR_SPREAD],
    settled: &[&DAMSEL_STILL],
    crawling: &[],
    middle: (7, 4),
    feet: (10, 5),
    inks: &DAMSEL_INKS,
};

const DAMSEL_INKS: [(u8, Rgba); 12] = inks(
    tones(AZURE, *b"Udbl*"),
    [
        (b'r', RING),
        (b'w', GLASS),
        (b'W', VEIN),
        (b'p', STIGMA),
        (b'k', RING),
        BLURS[0],
        BLURS[1],
    ],
);

const DAMSEL_STILL: [&str; 7] = [
    ".............",
    ".............",
    "..WpwwwwwW...",
    ".........rlr.",
    "Ubrbbrbbrbb*b",
    ".........k.k.",
    ".............",
];

const DAMSEL_BLUR_UP: [&str; 7] = [
    ".....fFFf....",
    "....fFFFFf...",
    ".....fFFf....",
    ".........rlr.",
    "Ubrbbrbbrbb*b",
    ".............",
    ".............",
];

const DAMSEL_BLUR_SPREAD: [&str; 7] = [
    ".............",
    ".............",
    "...fFFFFFf...",
    ".........rlr.",
    "Ubrbbrbbrbb*b",
    "...fFFFFFf...",
    ".............",
];

// The rose chafer: a squarish beetle, metallic green going gold in the light, flecked white.

static ROSE_CHAFER: Look = Look {
    size: (7, 6),
    flying: &[&CHAFER_FLY_UP, &CHAFER_FLY_BACK],
    settled: &[&CHAFER_STILL],
    crawling: &[&CHAFER_STEP, &CHAFER_STILL],
    middle: (3, 3),
    feet: (3, 5),
    inks: &CHAFER_INKS,
};

const CHAFER_INKS: [(u8, Rgba); 9] = inks(
    tones(CHAFER, *b"Gsol*"),
    [(b'w', FLECK), (b'g', CHAFER_LEG.base), BLURS[0], BLURS[1]],
);

const CHAFER_STILL: [&str; 6] = [
    ".......", ".GGGG..", "Gl**lG.", "GowossG", ".GsssGG", ".g.g.g.",
];

const CHAFER_STEP: [&str; 6] = [
    ".......", ".GGGG..", "Gl**lG.", "GowossG", ".GsssGG", "g.g.g..",
];

const CHAFER_FLY_UP: [&str; 6] = [
    ".fFFf..", "FGGGGF.", "Gl**lG.", "GowossG", ".GsssGG", "..g.g..",
];

const CHAFER_FLY_BACK: [&str; 6] = [
    ".......", "fFGGGG.", "FFl**lG", "GowossG", ".GsssGG", "..g.g..",
];

// The firefly: a small brown beetle, its shield pale over its head, and the pale yellow-green
// end that lights up. The light itself is the mechanics' to draw.

static FIREFLY_LOOK: Look = Look {
    size: (7, 5),
    flying: &[&FIREFLY_FLY_UP, &FIREFLY_FLY_BACK],
    settled: &[&FIREFLY_STILL],
    crawling: &[&FIREFLY_STEP, &FIREFLY_STILL],
    middle: (2, 2),
    feet: (2, 4),
    inks: &FIREFLY_INKS,
};

const FIREFLY_INKS: [(u8, Rgba); 14] = inks(
    tones(FIREFLY, *b"Bbol*"),
    [
        (b'T', LANTERN.shadow),
        (b'P', SHIELD.base),
        (b'q', SHIELD.shadow),
        (b't', LANTERN.base),
        (b'u', LANTERN.light),
        (b'g', FIREFLY.edge),
        (b'a', FIREFLY.shadow),
        BLURS[0],
        BLURS[1],
    ],
);

const FIREFLY_STILL: [&str; 5] = ["......a", ".BBBqa.", "TlloPq.", "utbbq..", ".g.g.g."];

const FIREFLY_STEP: [&str; 5] = ["......a", ".BBBqa.", "TlloPq.", "utbbq..", "g.g.g.."];

const FIREFLY_FLY_UP: [&str; 5] = [".fFf..a", "fBBBqa.", "TlloPq.", "utbbq..", "..g.g.."];

const FIREFLY_FLY_BACK: [&str; 5] = ["......a", "FfBBqa.", "fFloPq.", "utbbq..", "..g.g.."];

// The stag beetle: chestnut wing cases, a black head and shield, and its antlers.

static STAG_BEETLE: Look = Look {
    size: (13, 8),
    flying: &[&STAG_FLY_UP, &STAG_FLY_BACK],
    settled: &[&STAG_STILL],
    crawling: &[&STAG_STEP, &STAG_STILL],
    middle: (6, 4),
    feet: (6, 7),
    inks: &STAG_INKS,
};

const STAG_INKS: [(u8, Rgba); 16] = inks(
    tones(STAG, *b"Ccol*"),
    [
        (b'K', STAG_BLACK.edge),
        (b'k', STAG_BLACK.base),
        (b'j', STAG_BLACK.light),
        (b'M', ANTLER.edge),
        (b'm', ANTLER.base),
        (b'n', ANTLER.light),
        (b'g', STAG_BLACK.shadow),
        (b'a', STAG_BLACK.light),
        BLURS[0],
        BLURS[1],
        (b'z', GLASS),
    ],
);

const STAG_STILL: [&str; 8] = [
    "............M",
    "...........Mn",
    "..CCCCC...Mm.",
    ".Cl*loCKK.Mn.",
    ".ColooKjkKm..",
    ".CcoocKkkkKM.",
    "..CCCCKKKK...",
    "..g..g..g....",
];

const STAG_STEP: [&str; 8] = [
    "............M",
    "...........Mn",
    "..CCCCC...Mm.",
    ".Cl*loCKK.Mn.",
    ".ColooKjkKm..",
    ".CcoocKkkkKM.",
    "..CCCCKKKK...",
    ".g..g..g.....",
];

const STAG_FLY_UP: [&str; 8] = [
    "..fFFf......M",
    ".CCCFFf....Mn",
    "Cl*lCzz...Mm.",
    ".CooCCCKK.Mn.",
    ".ColooKjkKm..",
    ".CcoocKkkkKM.",
    "..CCCCKKKK...",
    "...g..g......",
];

const STAG_FLY_BACK: [&str; 8] = [
    "............M",
    "FFf........Mn",
    "fFCCC.....Mm.",
    "Cl*loCzKK.Mn.",
    ".ColooKjkKm..",
    ".CcoocKkkkKM.",
    "..CCCCKKKK...",
    "...g..g......",
];

// The emperor dragonfly: a long blue body with a dark line down its back, an apple-green
// thorax, great blue-green eyes, and four clear wings held out.

static EMPEROR: Look = Look {
    size: (17, 9),
    flying: &[&EMPEROR_BLUR_UP, &EMPEROR_BLUR_SPREAD],
    settled: &[&EMPEROR_STILL],
    crawling: &[],
    middle: (9, 4),
    feet: (12, 6),
    inks: &EMPEROR_INKS,
};

const EMPEROR_INKS: [(u8, Rgba); 19] = inks(
    tones(EMPEROR_BLUE, *b"Uubl*"),
    [
        (b'd', DORSAL),
        (b'G', APPLE.edge),
        (b'g', APPLE.base),
        (b'h', APPLE.light),
        (b'E', COMPOUND.edge),
        (b'e', COMPOUND.base),
        (b'i', COMPOUND.light),
        (b'w', GLASS),
        (b'W', VEIN),
        (b'p', STIGMA),
        (b'k', DORSAL),
        BLURS[0],
        BLURS[1],
        (b'x', COMPOUND.shine),
    ],
);

const EMPEROR_STILL: [&str; 9] = [
    "...WWWWWWWW......",
    "..WpwwwwwwwwW....",
    "....WWWWwwwwGGEE.",
    ".........dGhhGxiE",
    "Ubdbbdbbdll*ggeeE",
    ".UUUUUUUUuUGGGEE.",
    "...WWWWwwwwk.k...",
    "..WpwwwwwwwW.....",
    "...WWWWWWW.......",
];

const EMPEROR_BLUR_UP: [&str; 9] = [
    ".....fFFFFf......",
    "....fFFFFFFFf....",
    "......fFFFFfGGEE.",
    ".........dGhhGxiE",
    "Ubdbbdbbdll*ggeeE",
    ".UUUUUUUUuUGGGEE.",
    ".................",
    ".................",
    ".................",
];

const EMPEROR_BLUR_SPREAD: [&str; 9] = [
    ".................",
    ".................",
    "...fFFFFFFFfGGEE.",
    ".........dGhhGxiE",
    "Ubdbbdbbdll*ggeeE",
    ".UUUUUUUUuUGGGEE.",
    "...fFFFFFFFf.....",
    ".................",
    ".................",
];

// The moon moth: pale green, the forewings edged in maroon, an eyespot on every wing, a white
// furred body, feathered antennae, and the long tails trailing from its hindwings.

static MOON_MOTH: Look = Look {
    size: (16, 13),
    flying: &[&MOON_UP, &MOON_OPEN, &MOON_DOWN, &MOON_OPEN],
    settled: &[&MOON_OPEN, &MOON_SHUT],
    crawling: &[],
    middle: (7, 6),
    feet: (8, 7),
    inks: &MOON_INKS,
};

const MOON_INKS: [(u8, Rgba); 14] = inks(
    tones(LUNA, *b"#sol*"),
    [
        (b'P', MAROON.edge),
        (b'p', MAROON.base),
        (b'e', EYESPOT.base),
        (b'E', MAROON.shadow),
        (b'W', DOWN.edge),
        (b'w', DOWN.base),
        (b'x', DOWN.light),
        (b'f', FEATHER.base),
        (b'F', FEATHER.edge),
    ],
);

const MOON_OPEN: [&str; 13] = [
    "...........#P...",
    "#.........#lP...",
    ".o.###...#loP...",
    "..o#lo#.#leoP...",
    "..#oeo##looP....",
    "...#ss#ossP.....",
    "...WwwwwwxWfF...",
    "...#lo#looP.....",
    "..o#es#.#esP....",
    ".o.###....#P....",
    "#...............",
    "................",
    "................",
];

const MOON_SHUT: [&str; 13] = [
    "................",
    ".........#P.....",
    "........#lP.....",
    "......#looP.....",
    "....#loeooP.....",
    "...#oosssp......",
    "..oWwwwwwxWfF...",
    ".o.....W.W......",
    ".#..............",
    "................",
    "................",
    "................",
    "................",
];

const MOON_UP: [&str; 13] = [
    "................",
    ".........#P.....",
    "........#lP.....",
    "......#leoP.....",
    "....#looooP.....",
    ".o#oeosssp......",
    "#..WwwwwwxWfF...",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const MOON_DOWN: [&str; 13] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "#..WwwwwwxWfF...",
    ".o#oeosssp......",
    "....#looooP.....",
    "......#leoP.....",
    "........#lP.....",
    ".........#P.....",
    "................",
];

// ---------------------------------------------------------------------------------------------
// In the journal
// ---------------------------------------------------------------------------------------------

/// A bug's icon rows and the inks they are painted in: from above, wings spread, but for the
/// grasshopper, which is its hind leg or nothing, side on.
fn icon_rows(id: &str) -> Option<(Grid, Inks)> {
    bug(id)?;
    Some(match id {
        "cabbage_white" => (&WHITE_ICON, &WHITE_INKS),
        "ladybird" => (&LADYBIRD_ICON, &LADYBIRD_INKS),
        "grasshopper" => (&HOPPER_ICON, &HOPPER_INKS),
        "bumblebee" => (&BEE_ICON, &BEE_INKS),
        "red_admiral" => (&ADMIRAL_ICON, &ADMIRAL_INKS),
        "damselfly" => (&DAMSEL_ICON, &DAMSEL_ICON_INKS),
        "rose_chafer" => (&CHAFER_ICON, &CHAFER_INKS),
        "firefly" => (&FIREFLY_ICON, &FIREFLY_ICON_INKS),
        "stag_beetle" => (&STAG_ICON, &STAG_INKS),
        "emperor_dragonfly" => (&EMPEROR_ICON, &EMPEROR_ICON_INKS),
        "moon_moth" => (&MOON_ICON, &MOON_INKS),
        _ => return None,
    })
}

/// Clear wings in an icon, a little more solid than in the meadow so they show at a glance.
const ICON_GLASS: Rgba = rgba(0xdcecf4, 200);
const ICON_VEIN: Rgba = rgb(0x86a2b2);

const WHITE_ICON: [&str; 9] = [
    "KK.a.a.KK",
    "KlloblooK",
    "#lkoblks#",
    ".#osbos#.",
    "..#sbs#..",
    ".#loblo#.",
    "#losboos#",
    ".###b###.",
    ".........",
];

const LADYBIRD_ICON: [&str; 9] = [
    ".........",
    "...hhh...",
    "..whhhw..",
    ".EllkooE.",
    "Elk*sokoE",
    "EloosoosE",
    "EokosoksE",
    ".EossssE.",
    "..EEEEE..",
];

const HOPPER_ICON: [&str; 9] = [
    ".........",
    ".........",
    "........a",
    ".GG....a.",
    "GllGBGGG.",
    "GbllGlleG",
    "GssslGooG",
    "k...k.kk.",
    ".........",
];

const BEE_ICON: [&str; 9] = [
    "..g...g..",
    "...KkK...",
    "zZKYuYKZz",
    "zzKkjkKzz",
    ".ZKkkkKZ.",
    "..YyuyY..",
    "..KkjkK..",
    "..WwxwW..",
    "...WWW...",
];

const ADMIRAL_ICON: [&str; 9] = [
    ".###b###.",
    "#wkrbrkw#",
    "#krkbkrk#",
    "#rkkbkkr#",
    ".#kkbkk#.",
    "RrjkbkjrR",
    "RrkkbkkrR",
    ".RrkbkrR.",
    "..RRbRR..",
];

const DAMSEL_ICON: [&str; 9] = [
    "...lrl...",
    "pWWWbWWWp",
    ".wwWbWww.",
    "pWWWbWWWp",
    "....b....",
    "....r....",
    "....b....",
    "....r....",
    "....l....",
];

const DAMSEL_ICON_INKS: [(u8, Rgba); 7] = [
    (b'b', AZURE.base),
    (b'l', AZURE.light),
    (b'r', RING),
    (b'p', STIGMA),
    (b'W', ICON_VEIN),
    (b'w', ICON_GLASS),
    (b'U', AZURE.edge),
];

const CHAFER_ICON: [&str; 9] = [
    "..g...g..",
    "...GoG...",
    "..GllsG..",
    ".Gl*sooG.",
    "Gl*lsoooG",
    "GloosowsG",
    "GwoosoosG",
    ".GowsosG.",
    "..GGGGG..",
];

const FIREFLY_ICON: [&str; 9] = [
    ".a.....a.",
    "..aqqqa..",
    "..qPkPq..",
    "..PlBoP..",
    "..PlBoP..",
    "..PlBbP..",
    "..qbBbq..",
    "...tut...",
    "...TtT...",
];

const FIREFLY_ICON_INKS: [(u8, Rgba); 11] = [
    (b'a', FIREFLY.shadow),
    (b'B', FIREFLY.edge),
    (b'b', FIREFLY.shadow),
    (b'o', FIREFLY.base),
    (b'l', FIREFLY.light),
    (b'P', SHIELD.light),
    (b'q', SHIELD.shadow),
    (b'k', PINK.shadow),
    (b't', LANTERN.base),
    (b'u', LANTERN.light),
    (b'T', LANTERN.shadow),
];

const STAG_ICON: [&str; 9] = [
    ".n.....n.",
    ".mM...Mm.",
    "..mM.Mm..",
    "..KjkkK..",
    ".KjkkkkK.",
    ".CllclcC.",
    ".ClococC.",
    ".CoocccC.",
    "..CCCCC..",
];

const EMPEROR_ICON: [&str; 9] = [
    "...eie...",
    "pWWWgWWWp",
    ".wwWhWww.",
    "pWWWgWWWp",
    ".wwwbwww.",
    "....b....",
    "....l....",
    "....b....",
    "....d....",
];

const EMPEROR_ICON_INKS: [(u8, Rgba); 11] = [
    (b'b', EMPEROR_BLUE.base),
    (b'l', EMPEROR_BLUE.light),
    (b'd', DORSAL),
    (b'g', APPLE.base),
    (b'h', APPLE.light),
    (b'e', COMPOUND.base),
    (b'i', COMPOUND.light),
    (b'p', STIGMA),
    (b'W', ICON_VEIN),
    (b'w', ICON_GLASS),
    (b'U', EMPEROR_BLUE.edge),
];

const MOON_ICON: [&str; 9] = [
    "..F...F..",
    "PppPwPppP",
    "#lelwlel#",
    ".#oowoo#.",
    ".#lowol#.",
    "#oeowoeo#",
    ".#o#w#o#.",
    ".s.....s.",
    "#.......#",
];

// Anything Hill doesn't know: a plain little beetle in grey-brown, so nothing is ever missing.

const PLAIN: Ramp = Ramp::new(0x3e3536, 0x5a4e4c, 0x786a64, 0x968880, 0xb8aca2);

fn plain_sprite() -> Canvas {
    let mut s = Canvas::new(6, 5);
    stamp(
        &mut s,
        (0, 0),
        &["......", ".####.", "#lol##", "#osss#", ".####."],
        &tones(PLAIN, *b"#sol*"),
    );
    s
}

fn plain_close_up() -> Canvas {
    let mut s = Canvas::new(24, 24);
    model(&mut s, PLAIN, 0, 1, |x, y| {
        oval(x, y, (12.0, 12.0), (8.4, 9.4))
    });
    s
}

fn plain_icon() -> Canvas {
    let mut s = Canvas::new(ICON, ICON);
    model(&mut s, PLAIN, 0, 1, |x, y| {
        oval(x, y, (4.5, 4.5), (3.4, 3.4))
    });
    s
}

// ---------------------------------------------------------------------------------------------
// Close up
// ---------------------------------------------------------------------------------------------

// The extra colours the close-ups need, as ramps like the rest.

const DUSTED: Ramp = Ramp::new(0x7c7888, 0xaaa7b4, 0xcac8d0, 0xdedce4, 0xf0eff4);
const CHALK_SPOT: Ramp = Ramp::new(0x9a948c, 0xd6d0c6, 0xf4f0e6, 0xfcfaf4, 0xffffff);
const AZURE_SPOT: Ramp = Ramp::new(0x2c3c6a, 0x4a62a0, 0x6c8ed4, 0x9ab4ea, 0xd4e2fa);
const WINDOW: Ramp = Ramp::new(0x8a9a80, 0xc8d4b8, 0xe8f0dc, 0xf6faee, 0xffffff);
const TAIL_TIP: Ramp = Ramp::new(0x7e7036, 0xb4a65a, 0xd4d08a, 0xe6e4ac, 0xf8f6d8);
const PINK: Ramp = Ramp::new(0x7a2a30, 0xa8424a, 0xd06a6a, 0xe8948c, 0xffd0c4);
const HOPPER_EYES: Ramp = Ramp::new(0x3a2010, 0x5a3418, 0x7a4a24, 0x9c6a3a, 0xf0dcc0);
const TIBIA: Ramp = Ramp::new(0x5a2a1a, 0x8a4028, 0xb05a34, 0xcc7a4c, 0xecac80);
const PATCH: Ramp = Ramp::new(0x9c968c, 0xd0cac0, 0xf2eee4, 0xfaf8f2, 0xffffff);

/// Clear wing close up: veins round the edge and through it, a faint net of cross-veins
/// between, and the membrane itself only a breath of colour.
const RIM_VEIN: Rgba = rgba(0x5a7486, 230);
const LONG_VEIN: Rgba = rgba(0x6c8698, 170);
const CROSS_VEIN: Rgba = rgba(0x88a2b2, 95);
const MEMBRANE: Rgba = rgba(0xe4f0f6, 85);
const SMOKY: Rgba = rgba(0xcfc6bc, 105);

/// The cabbage white, wings spread: white, dusted grey towards the body, black at each
/// forewing's tip and two black spots on each.
fn cabbage_white() -> Canvas {
    let (mut s, axis) = (Canvas::new(37, 27), 18.5);
    pair(
        &mut s,
        axis,
        &[
            (1.0, 10.0),
            (6.0, 10.4),
            (11.5, 12.0),
            (14.2, 15.0),
            (13.6, 19.0),
            (10.6, 22.6),
            (6.6, 25.2),
            (3.0, 24.2),
            (1.4, 20.0),
            (1.0, 15.0),
        ],
        &[
            [(1.5, 11.5), (13.0, 15.5)],
            [(1.5, 12.5), (11.0, 21.5)],
            [(1.5, 13.5), (6.0, 24.0)],
        ],
        11,
        &|dx, _, _| if dx < 3.2 { DUSTED } else { WHITE_WING },
    );
    pair(
        &mut s,
        axis,
        &[
            (1.0, 7.0),
            (5.0, 3.5),
            (10.0, 1.5),
            (15.5, 0.8),
            (17.6, 2.4),
            (17.0, 6.0),
            (14.6, 10.5),
            (11.0, 13.0),
            (5.0, 12.6),
            (1.4, 11.0),
        ],
        &[
            [(1.5, 8.0), (15.0, 2.0)],
            [(1.5, 9.5), (16.5, 5.5)],
            [(2.0, 10.5), (13.5, 11.5)],
        ],
        12,
        &|dx, y, _| {
            let tip = (dx - 17.5).hypot((y - 1.0) * 1.3) < 6.2;
            let spot = (dx - 10.5).hypot(y - 7.0) < 1.5 || (dx - 9.0).hypot(y - 10.4) < 1.2;
            if tip || spot {
                WING_TIP
            } else if dx < 3.2 {
                DUSTED
            } else {
                WHITE_WING
            }
        },
    );
    model(&mut s, DUSKY, 40, 13, |x, y| {
        oval(x, y, (axis, 16.5), (1.6, 6.4))
    });
    model(&mut s, DUSTED, 60, 14, |x, y| {
        oval(x, y, (axis, 9.0), (2.4 + fuzz(x, y, 14), 2.6))
    });
    model(&mut s, DUSKY, 0, 15, |x, y| {
        oval(x, y, (axis, 5.6), (1.6, 1.4))
    });
    feelers(
        &mut s,
        axis,
        &[(0.6, 4.6), (2.5, 2.4), (5.0, 0.5)],
        DUSKY.shadow,
        CHEEK,
    );
    s
}

/// The red admiral, wings spread: velvet black, a red band across each forewing, red along
/// the hindwings' edge with black dots in it, white spots towards the tips, and a dab of blue
/// where the hindwings meet.
fn red_admiral() -> Canvas {
    let (mut s, axis) = (Canvas::new(39, 29), 19.5);
    pair(
        &mut s,
        axis,
        &[
            (1.0, 10.0),
            (7.0, 11.0),
            (12.5, 12.5),
            (15.5, 15.5),
            (15.6, 19.5),
            (13.2, 23.5),
            (9.2, 26.4),
            (5.2, 27.4),
            (2.4, 24.0),
            (1.0, 17.0),
        ],
        &[
            [(1.5, 11.5), (14.5, 16.0)],
            [(1.5, 12.5), (12.0, 23.0)],
            [(1.5, 13.5), (6.0, 26.0)],
        ],
        21,
        &|dx, y, depth| {
            let dots = [(13.4, 20.6), (11.0, 23.6), (8.0, 25.4)];
            if (dx - 3.6).hypot(y - 25.0) < 1.3 {
                AZURE_SPOT
            } else if depth <= 3 && (dx - 1.5).hypot(y - 12.0) > 9.5 {
                if dots.iter().any(|&(x0, y0)| (dx - x0).hypot(y - y0) < 0.8) {
                    VELVET
                } else {
                    ADMIRAL_RED
                }
            } else {
                VELVET
            }
        },
    );
    pair(
        &mut s,
        axis,
        &[
            (1.0, 7.0),
            (6.0, 3.0),
            (12.0, 1.0),
            (16.6, 0.5),
            (18.6, 2.0),
            (18.0, 4.6),
            (16.4, 7.6),
            (16.2, 10.4),
            (14.2, 13.6),
            (6.0, 12.6),
            (1.4, 11.0),
        ],
        &[
            [(1.5, 8.0), (14.0, 1.6)],
            [(1.5, 9.5), (16.5, 8.5)],
            [(2.0, 10.5), (12.0, 12.6)],
        ],
        22,
        &|dx, y, _| {
            let spots = [
                (15.8, 3.0, 1.1),
                (17.4, 4.6, 0.8),
                (14.2, 4.8, 0.8),
                (16.2, 6.4, 0.8),
            ];
            let bar = ((dx - 13.4) / 1.6).hypot((y - 1.6) / 0.9) < 1.0;
            if bar || spots.iter().any(|&(x0, y0, r)| (dx - x0).hypot(y - y0) < r) {
                CHALK_SPOT
            } else if to_segment((dx, y), (8.0, 1.6), (12.6, 13.4)) < 1.7 {
                ADMIRAL_RED
            } else {
                VELVET
            }
        },
    );
    model(&mut s, VELVET, 30, 23, |x, y| {
        oval(x, y, (axis, 16.5), (1.7, 6.6))
    });
    model(&mut s, VELVET, 50, 24, |x, y| {
        oval(x, y, (axis, 9.0), (2.4 + fuzz(x, y, 24), 2.6))
    });
    model(&mut s, VELVET, 0, 25, |x, y| {
        oval(x, y, (axis, 5.6), (1.6, 1.4))
    });
    feelers(
        &mut s,
        axis,
        &[(0.6, 4.6), (2.6, 2.2), (5.4, 0.4)],
        VELVET.light,
        CHALK,
    );
    s
}

/// The moon moth, wings spread: pale green, the forewings edged along the front in maroon, an
/// eyespot on every wing, the long tails trailing below; a white furred body with a maroon
/// collar, and feathered antennae.
fn moon_moth() -> Canvas {
    let (mut s, axis) = (Canvas::new(41, 34), 20.5);
    let eyespot = |dx: f32, y: f32, at: (f32, f32), size: f32| {
        let r = (dx - at.0).hypot((y - at.1) * 0.85) / size;
        if r < 0.38 {
            Some(WINDOW)
        } else if r < 0.66 {
            Some(MAROON)
        } else if r < 1.0 {
            Some(EYESPOT)
        } else {
            None
        }
    };
    pair(
        &mut s,
        axis,
        &[
            (1.5, 10.0),
            (7.0, 11.0),
            (13.0, 12.6),
            (15.6, 15.6),
            (14.6, 19.2),
            (11.6, 21.2),
            (10.6, 24.6),
            (11.2, 28.6),
            (12.8, 32.6),
            (10.6, 33.6),
            (8.4, 31.0),
            (6.8, 26.6),
            (4.8, 22.6),
            (2.4, 20.0),
            (1.4, 15.0),
        ],
        &[
            [(1.5, 11.5), (14.0, 16.0)],
            [(1.5, 12.5), (12.0, 20.5)],
            [(2.5, 13.5), (10.0, 30.5)],
        ],
        31,
        &|dx, y, _| {
            eyespot(dx, y, (9.0, 16.2), 2.3).unwrap_or(if y > 28.5 {
                TAIL_TIP
            } else if dx < 2.6 {
                DOWN
            } else {
                LUNA
            })
        },
    );
    pair(
        &mut s,
        axis,
        &[
            (1.5, 6.0),
            (6.0, 3.5),
            (12.0, 1.4),
            (17.6, 0.4),
            (19.6, 1.0),
            (18.6, 3.6),
            (15.6, 7.6),
            (13.2, 11.6),
            (7.0, 12.6),
            (2.0, 11.0),
        ],
        &[
            [(2.0, 8.0), (17.0, 2.4)],
            [(2.0, 9.4), (15.0, 8.0)],
            [(2.0, 10.6), (12.0, 11.8)],
        ],
        32,
        &|dx, y, _| {
            eyespot(dx, y, (10.6, 6.8), 2.5).unwrap_or(
                if to_segment((dx, y), (1.5, 6.0), (17.6, 0.4)) < 1.5 {
                    MAROON
                } else if dx < 2.6 {
                    DOWN
                } else {
                    LUNA
                },
            )
        },
    );
    model(&mut s, DOWN, 50, 33, |x, y| {
        oval(x, y, (axis, 16.0), (2.4 + fuzz(x, y, 33), 5.8))
    });
    shade(
        &mut s,
        60,
        34,
        |x, y| oval(x, y, (axis, 8.2), (3.0 + fuzz(x, y, 34), 3.2)),
        |_, y| if y < 6 { MAROON } else { DOWN },
    );
    model(&mut s, DOWN, 0, 35, |x, y| {
        oval(x, y, (axis, 4.4), (1.6, 1.3))
    });
    for side in [-1.0, 1.0] {
        feather(&mut s, (axis + side * 0.8, 3.6), (axis + side * 5.6, 0.4));
    }
    s
}

/// The ladybird from above: a red dome with its seven spots, the shield over its head black
/// with white patches at the corners, its head tucked under, legs just showing.
fn ladybird() -> Canvas {
    let (mut s, axis) = (Canvas::new(25, 23), 12.5);
    for path in [
        [(4.0, 8.0), (8.0, 6.4), (9.6, 7.6)],
        [(5.0, 12.0), (10.0, 12.0), (11.0, 14.2)],
        [(5.0, 16.0), (9.0, 19.2), (9.8, 20.8)],
    ] {
        limbs(&mut s, axis, &path, PITCH.shadow);
    }
    limbs(&mut s, axis, &[(1.4, 2.4), (2.8, 0.6)], PITCH.shadow);
    model(&mut s, LADY_RED, 0, 41, |x, y| {
        oval(x, y, (axis, 13.6), (9.4, 8.6)) && y >= 6
    });
    let spots = [
        (0.0, 7.6, 1.9),
        (4.0, 10.6, 1.6),
        (6.6, 14.6, 2.0),
        (3.6, 18.4, 1.6),
    ];
    let mut painted = s.clone();
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            let under = s.get(x, y);
            let dx = (x as f32 + 0.5 - axis).abs();
            let in_spot = spots
                .iter()
                .any(|&(x0, y0, r)| (dx - x0).hypot(y as f32 + 0.5 - y0) < r);
            if !in_spot || under.a == 0 || under == LADY_RED.edge {
                continue;
            }
            let lit = under == LADY_RED.shine || under == LADY_RED.light;
            painted.set(x, y, if lit { PITCH.light } else { PITCH.base });
        }
    }
    s = painted;
    for y in 7..22 {
        if s.get(12, y) != LADY_RED.edge && s.get(12, y).a > 0 {
            put(&mut s, 12, y, PITCH.shadow);
        }
    }
    model(&mut s, PITCH, 0, 42, |x, y| {
        oval(x, y, (axis, 3.4), (2.6, 1.6))
    });
    for x in [10, 14] {
        put(&mut s, x, 3, CHEEK);
    }
    shade(
        &mut s,
        0,
        43,
        |x, y| oval(x, y, (axis, 6.4), (5.6, 2.2)),
        |x, y| {
            let dx = (x as f32 + 0.5 - axis).abs();
            if dx > 2.8 && y <= 6 { PATCH } else { PITCH }
        },
    );
    s
}

/// The firefly from above: a shield over its head, pale yellow at the rim and pink inside a
/// dark spot; long brown wing cases, pale along their edges; and the pale yellow-green end
/// that lights up, showing past them. The light itself is the mechanics' to draw.
fn firefly() -> Canvas {
    let (mut s, axis) = (Canvas::new(23, 30), 11.5);
    for path in [
        [(2.6, 10.0), (5.6, 9.0), (7.4, 10.6)],
        [(2.6, 13.0), (6.4, 14.4), (7.6, 16.6)],
        [(2.6, 16.6), (5.8, 19.4), (6.6, 22.0)],
    ] {
        limbs(&mut s, axis, &path, FIREFLY.shadow);
    }
    limbs(
        &mut s,
        axis,
        &[(1.4, 3.6), (3.6, 1.6), (6.4, 0.2)],
        FIREFLY.shadow,
    );
    shade(
        &mut s,
        0,
        51,
        |x, y| oval(x, y, (axis, 25.6), (3.3, 3.0)),
        |_, y| if y == 25 { TAIL_SEAM } else { LANTERN },
    );
    shade(
        &mut s,
        30,
        52,
        |x, y| {
            let (dx, fy) = ((x as f32 + 0.5 - axis).abs(), y as f32 + 0.5);
            fy > 8.6 && ((fy < 21.0 && dx < 5.4) || (dx / 5.4).hypot((fy - 21.0) / 3.6) <= 1.0)
        },
        |x, _| {
            if (x as f32 + 0.5 - axis).abs() > 4.0 {
                SHIELD
            } else {
                FIREFLY
            }
        },
    );
    for y in 10..24 {
        put(&mut s, 11, y, FIREFLY.edge);
    }
    shade(
        &mut s,
        0,
        53,
        |x, y| oval(x, y, (axis, 6.2), (5.2, 3.8)),
        |x, y| {
            let (dx, dy) = (x as f32 + 0.5 - axis, y as f32 + 0.5 - 6.8);
            if (dx / 1.3).hypot(dy / 1.9) < 1.0 {
                FIREFLY
            } else if (dx / 3.2).hypot(dy / 2.6) < 1.0 {
                PINK
            } else {
                SHIELD
            }
        },
    );
    s
}

/// The seam between the lantern's two segments.
const TAIL_SEAM: Ramp = Ramp::new(0x8a9a3e, 0xa8b850, 0xc0d064, 0xd4e276, 0xe2f08c);

/// The rose chafer from above: broad and squarish, metallic green going gold where the light
/// catches it and blue-green in shade, with fine white flecks across the wing cases.
fn rose_chafer() -> Canvas {
    let (mut s, axis) = (Canvas::new(27, 28), 13.5);
    for path in [
        [(3.0, 7.6), (7.0, 6.0), (9.0, 3.6)],
        [(4.6, 13.0), (9.6, 14.0), (11.2, 17.4)],
        [(5.0, 19.0), (9.0, 22.6), (10.4, 26.4)],
    ] {
        thick_limbs(&mut s, axis, &path, CHAFER_LEG);
    }
    limbs(&mut s, axis, &[(1.2, 2.2), (3.4, 0.8)], CHAFER_LEG.shadow);
    for side in [-1.0, 1.0] {
        let x = mirror(axis, axis + side * 3.6);
        put(&mut s, x, 0, CHAFER_LEG.edge);
        put(&mut s, x + side as i32, 0, CHAFER_LEG.shadow);
    }
    model(&mut s, CHAFER, 70, 61, |x, y| {
        let (dx, fy) = ((x as f32 + 0.5 - axis).abs(), y as f32 + 0.5);
        fy > 9.2
            && ((fy < 20.0 && dx < 7.2 - (fy - 9.2) * 0.04)
                || (dx / 6.8).hypot((fy - 20.0) / 5.6) <= 1.0)
    });
    let flecks: [(i32, i32); 14] = [
        (-5, 16),
        (-4, 16),
        (-3, 19),
        (-5, 21),
        (-4, 22),
        (-2, 23),
        (-6, 18),
        (4, 15),
        (5, 18),
        (3, 20),
        (4, 20),
        (5, 22),
        (2, 23),
        (6, 17),
    ];
    for (dx, y) in flecks {
        put(&mut s, 13 + dx, y, FLECK);
    }
    for y in 10..25 {
        put(&mut s, 13, y, CHAFER.edge);
    }
    for (x, y) in [(12, 10), (13, 10), (14, 10), (13, 11)] {
        put(&mut s, x, y, CHAFER.light);
    }
    model(&mut s, CHAFER, 30, 62, |x, y| {
        let (dx, fy) = ((x as f32 + 0.5 - axis).abs(), y as f32 + 0.5);
        (4.4..10.2).contains(&fy) && dx < 3.4 + (fy - 4.4) * 0.62
    });
    model(&mut s, CHAFER, 0, 63, |x, y| {
        oval(x, y, (axis, 3.2), (2.1, 1.7))
    });
    s
}

/// The stag beetle from above: chestnut wing cases, a broad black head and shield, and the
/// antlers, curving in to forked tips with a tooth halfway along.
fn stag_beetle() -> Canvas {
    let (mut s, axis) = (Canvas::new(31, 34), 15.5);
    for path in [
        [(4.6, 15.6), (9.0, 13.4), (11.4, 10.0), (12.6, 8.2)],
        [(5.0, 19.0), (10.4, 20.0), (12.4, 24.0), (13.4, 25.6)],
        [(5.0, 23.0), (9.6, 27.0), (11.4, 31.4), (12.2, 33.0)],
    ] {
        thick_limbs(&mut s, axis, &path, STAG_BLACK);
    }
    limbs(
        &mut s,
        axis,
        &[(4.4, 12.6), (7.6, 10.6), (9.6, 11.6)],
        STAG_BLACK.light,
    );
    model(&mut s, STAG, 40, 71, |x, y| {
        oval(x, y, (axis, 25.4), (6.4, 7.8))
    });
    for y in 19..33 {
        if s.get(15, y).a > 0 {
            put(&mut s, 15, y, STAG.edge);
        }
    }
    model(&mut s, STAG_BLACK, 0, 72, |x, y| {
        oval(x, y, (axis, 17.2), (6.0, 2.8))
    });
    model(&mut s, STAG_BLACK, 20, 73, |x, y| {
        oval(x, y, (axis, 13.0), (5.6, 2.6))
    });
    let path = [
        (2.0, 11.4),
        (4.4, 9.6),
        (6.2, 7.0),
        (6.6, 4.2),
        (5.6, 1.8),
        (4.0, 0.6),
    ];
    model(&mut s, ANTLER, 0, 74, |x, y| {
        let (dx, fy) = ((x as f32 + 0.5 - axis).abs(), y as f32 + 0.5);
        let along = path.windows(2).enumerate().any(|(index, pair)| {
            let t = index as f32 / (path.len() - 1) as f32;
            to_segment((dx, fy), pair[0], pair[1]) < 1.5 - 0.8 * t
        });
        let tooth = to_segment((dx, fy), (5.6, 6.0), (3.4, 5.4)) < 0.8;
        let fork = to_segment((dx, fy), (6.2, 3.0), (7.4, 1.6)) < 0.6;
        along || tooth || fork
    });
    s
}

/// The bumblebee from above: black fur with a yellow collar and a yellow band, a white tail,
/// smoky wings out to the sides and full pollen baskets on its hind legs.
fn bumblebee() -> Canvas {
    let (mut s, axis) = (Canvas::new(31, 27), 15.5);
    for path in [
        [(3.0, 7.6), (6.0, 6.0), (7.0, 4.4)],
        [(3.6, 11.0), (7.4, 13.2), (8.2, 15.6)],
        [(3.6, 13.4), (8.4, 17.2), (9.2, 21.0)],
    ] {
        thick_limbs(&mut s, axis, &path, FUR);
    }
    for side in [-1.0, 1.0] {
        let x = mirror(axis, axis + side * 8.6);
        model(&mut s, POLLEN_BASKET, 0, 81, |px, py| {
            oval(px, py, (x as f32 + 0.5, 18.2), (1.3, 1.6))
        });
    }
    glass_pair(
        &mut s,
        axis,
        &[
            (2.6, 8.2),
            (8.0, 6.6),
            (14.0, 7.6),
            (15.4, 9.4),
            (13.0, 11.0),
            (6.0, 11.0),
            (2.6, 10.2),
        ],
        &[[(3.0, 9.0), (13.0, 8.4)], [(5.0, 10.0), (12.0, 10.2)]],
        82,
        SMOKY,
    );
    glass_pair(
        &mut s,
        axis,
        &[
            (3.0, 11.0),
            (9.0, 11.6),
            (11.2, 13.4),
            (9.0, 14.8),
            (4.0, 13.6),
        ],
        &[[(3.5, 12.0), (10.0, 13.2)]],
        83,
        SMOKY,
    );
    shade(
        &mut s,
        70,
        84,
        |x, y| {
            oval(
                x,
                y,
                (axis, 18.6),
                (5.8 + fuzz(x, y, 84), 6.4 + fuzz(x, y, 85)),
            )
        },
        |_, y| match y {
            ..14 => FUR,
            14..17 => POLLEN,
            17..21 => FUR,
            _ => BUFF,
        },
    );
    shade(
        &mut s,
        70,
        86,
        |x, y| {
            oval(
                x,
                y,
                (axis, 9.6),
                (4.6 + fuzz(x, y, 86), 4.0 + fuzz(x, y, 87)),
            )
        },
        |_, y| if y < 8 { POLLEN } else { FUR },
    );
    model(&mut s, FUR, 30, 88, |x, y| {
        oval(x, y, (axis, 4.4), (2.5, 1.8))
    });
    limbs(
        &mut s,
        axis,
        &[(1.0, 3.0), (1.8, 1.0), (4.0, 0.4)],
        FUR.light,
    );
    s
}

/// A bumblebee's pollen, packed on its hind legs.
const POLLEN_BASKET: Ramp = Ramp::new(0x8a4a10, 0xc06a18, 0xe68e2a, 0xf6b050, 0xffdc98);

/// The grasshopper side on, facing right, for the sake of its great hind leg: the long thigh
/// patterned in chevrons, the knee dark, the shin red-brown and spined.
fn grasshopper() -> Canvas {
    let mut s = Canvas::new(39, 22);
    model(&mut s, BELLY, 20, 91, |x, y| {
        oval(x, y, (12.5, 13.6), (10.6, 3.2))
    });
    for x in [6, 9, 12, 15, 18] {
        for y in 14..17 {
            if s.get(x, y).a > 0 && s.get(x, y) != BELLY.edge {
                put(&mut s, x, y, BELLY.shadow);
            }
        }
    }
    wing(
        &mut s,
        &[
            (22.0, 8.0),
            (12.0, 8.4),
            (4.0, 9.8),
            (1.6, 11.4),
            (4.0, 12.4),
            (13.0, 12.0),
            (21.5, 11.6),
        ],
        &[[(21.0, 9.6), (3.0, 11.0)], [(20.0, 10.6), (6.0, 11.6)]],
        92,
        &|_, _, _| STRAW,
    );
    for (x, y) in [(9, 10), (13, 10), (16, 9)] {
        put(&mut s, x, y, STRAW.edge);
    }
    for path in [
        [(25, 14), (26, 18), (28, 19)],
        [(22, 14), (21, 18), (19, 19)],
    ] {
        thick_path(&mut s, &path, HOPPER);
    }
    model(&mut s, HOPPER, 10, 93, |x, y| {
        oval(x, y, (24.5, 11.6), (3.6, 3.8))
    });
    for x in 22..28 {
        put(&mut s, x, 10, HOPPER.shadow);
    }
    model(&mut s, HOPPER, 10, 94, |x, y| {
        oval(x, y, (30.0, 11.6), (3.2, 4.2))
    });
    model(&mut s, HOPPER_EYES, 0, 95, |x, y| {
        oval(x, y, (30.6, 9.6), (1.6, 2.1))
    });
    put(&mut s, 30, 8, HOPPER_EYES.shine);
    for (x, y) in [(32, 14), (32, 15), (31, 15)] {
        put(&mut s, x, y, HOPPER.edge);
    }
    let mut antenna = Vec::new();
    for t in 0..=10 {
        let t = t as f32 / 10.0;
        antenna.push((
            (31.0 + 6.0 * t) as i32,
            (7.0 - 6.0 * t + 1.5 * t * (1.0 - t)) as i32,
        ));
    }
    for pair in antenna.windows(2) {
        line(&mut s, pair[0], pair[1], HOPPER.shadow);
    }
    // The hind leg at rest: the shin folded up under the thigh, spined along its back, its
    // foot on the ground beneath the hip; then the thigh over it, swelling from the knee to the
    // hip and barred across.
    thick_path(&mut s, &[(8, 8), (13, 12), (18, 16)], TIBIA);
    for (x, y) in [(10, 11), (13, 13), (16, 16)] {
        put(&mut s, x, y, TIBIA.edge);
    }
    line(&mut s, (18, 17), (19, 19), TIBIA.shadow);
    line(&mut s, (19, 19), (22, 19), TIBIA.shadow);
    let (hip, knee) = ((19.6, 13.0), (7.6, 5.8));
    shade(
        &mut s,
        0,
        96,
        |x, y| {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let t = along(p, knee, hip);
            to_segment(p, knee, hip) < 1.1 + 1.9 * t.powf(0.7)
        },
        |x, y| {
            let t = along((x as f32 + 0.5, y as f32 + 0.5), knee, hip);
            if (0.2..0.85).contains(&t) && (t * 8.0).fract() < 0.2 {
                CHEVRON
            } else {
                THIGH
            }
        },
    );
    model(&mut s, HOPPER_EYES, 0, 97, |x, y| {
        oval(x, y, (8.0, 6.0), (1.3, 1.3))
    });
    s
}

/// A grasshopper's belly, in shade under its wings; its thigh, yellower and in the light; and
/// the bars across the thigh.
const BELLY: Ramp = Ramp::new(0x2c4c18, 0x46702a, 0x64962e, 0x84b440, 0xaed46c);
const THIGH: Ramp = Ramp::new(0x3a5e1c, 0x84b03a, 0xb2d856, 0xd2ec82, 0xf0fac0);
const CHEVRON: Ramp = Ramp::new(0x33561b, 0x4e7a22, 0x6a9a2a, 0x8cb63e, 0xb4d86a);

/// The damselfly from above: a needle of bright blue ringed in black, eyes set wide either side
/// of its head, and four narrow clear wings, each with a dark cell near the tip.
fn damselfly() -> Canvas {
    let (mut s, axis) = (Canvas::new(32, 32), 16.0);
    glass_pair(
        &mut s,
        axis,
        &[
            (1.5, 6.2),
            (6.0, 5.6),
            (11.0, 4.4),
            (15.0, 4.2),
            (16.4, 5.4),
            (15.2, 7.0),
            (11.0, 7.6),
            (6.0, 7.2),
            (1.5, 7.4),
        ],
        &[],
        101,
        MEMBRANE,
    );
    glass_pair(
        &mut s,
        axis,
        &[
            (1.5, 8.4),
            (6.0, 8.6),
            (11.0, 8.8),
            (15.0, 9.4),
            (16.0, 10.8),
            (14.6, 12.0),
            (10.0, 11.6),
            (6.0, 10.2),
            (1.5, 9.6),
        ],
        &[],
        102,
        MEMBRANE,
    );
    for side in [-1.0, 1.0] {
        for (dx, y) in [(13.6, 5), (14.6, 5), (13.4, 10), (14.4, 10)] {
            put(&mut s, mirror(axis, axis + side * dx), y, STIGMA);
        }
    }
    // The body in ten segments: the first two blue with a black mark on the second, five more
    // each with a black ring at its end, two all blue, and a black tip.
    for y in 10..32 {
        let ring = match y {
            13 | 31 => true,
            14..29 => (y - 14) % 3 == 2,
            _ => false,
        };
        let (left, right) = if ring {
            (RING, RING)
        } else {
            (AZURE.light, AZURE.base)
        };
        put(&mut s, 15, y, left);
        put(&mut s, 16, y, right);
    }
    shade(
        &mut s,
        0,
        103,
        |x, y| oval(x, y, (axis, 7.4), (2.6, 3.0)),
        |x, _| {
            let dx = (x as f32 + 0.5 - axis).abs();
            if (1.0..1.6).contains(&dx) {
                BLACK_STRIPE
            } else {
                AZURE
            }
        },
    );
    model(&mut s, BLACK_STRIPE, 0, 104, |x, y| {
        oval(x, y, (axis, 3.4), (2.6, 1.2))
    });
    for side in [-1.0, 1.0] {
        let centre = (axis + side * 2.8, 3.2);
        shade(
            &mut s,
            0,
            105,
            |x, y| oval(x, y, centre, (1.5, 1.5)),
            |_, y| if y < 3 { BLACK_STRIPE } else { AZURE },
        );
    }
    s
}

/// The damselfly's black.
const BLACK_STRIPE: Ramp = Ramp::new(0x10141e, 0x1c2438, 0x283450, 0x3a4a6c, 0x5a6e94);

/// The emperor dragonfly from above: great blue-green eyes meeting on top of its head, an
/// apple-green thorax, a long blue body with a black line down it, and four wide clear wings
/// netted with veins, each with its brown cell near the tip.
fn emperor_dragonfly() -> Canvas {
    let (mut s, axis) = (Canvas::new(41, 34), 20.5);
    glass_pair(
        &mut s,
        axis,
        &[
            (2.0, 5.6),
            (8.0, 4.0),
            (14.0, 3.4),
            (18.6, 4.0),
            (19.6, 5.4),
            (18.0, 6.8),
            (12.0, 7.6),
            (6.0, 8.0),
            (2.0, 8.0),
        ],
        &[[(2.0, 6.4), (18.0, 4.8)], [(2.6, 7.4), (16.0, 6.6)]],
        111,
        MEMBRANE,
    );
    glass_pair(
        &mut s,
        axis,
        &[
            (2.0, 8.6),
            (7.0, 8.8),
            (13.0, 9.0),
            (18.0, 10.0),
            (19.0, 11.3),
            (17.0, 12.6),
            (11.0, 13.0),
            (6.0, 14.6),
            (3.0, 14.0),
            (2.0, 11.6),
        ],
        &[
            [(2.0, 9.4), (17.6, 10.6)],
            [(2.6, 11.0), (16.0, 12.0)],
            [(3.0, 12.6), (9.0, 13.4)],
        ],
        112,
        MEMBRANE,
    );
    for side in [-1.0, 1.0] {
        for (dx, y) in [(16.0, 4), (17.0, 4), (15.8, 10), (16.8, 10)] {
            put(&mut s, mirror(axis, axis + side * dx), y, STIGMA);
        }
    }
    // The body: swollen where it joins the thorax, pinched, then long and even to a tip with
    // two little claspers; sky blue, the black line down its back, darker at every joint.
    for y in 11..34 {
        let half: i32 = match y {
            11..14 | 16..31 => 2,
            33 => 0,
            _ => 1,
        };
        let joint = y > 15 && (y - 16) % 3 == 2;
        for x in 20 - half..=20 + half {
            let outer = half > 0 && (x - 20).abs() == half;
            let color = if x == 20 {
                DORSAL
            } else if y == 33 || (outer && joint) {
                EMPEROR_BLUE.edge
            } else if outer || joint {
                EMPEROR_BLUE.shadow
            } else if x < 20 {
                EMPEROR_BLUE.light
            } else {
                EMPEROR_BLUE.base
            };
            put(&mut s, x, y, color);
        }
    }
    for x in [19, 21] {
        put(&mut s, x, 33, DORSAL);
    }
    shade(
        &mut s,
        20,
        114,
        |x, y| oval(x, y, (axis, 7.6), (3.3, 3.5)),
        |x, y| {
            let dx = (x as f32 + 0.5 - axis).abs();
            if (1.6..2.2).contains(&dx) && y > 6 {
                THORAX_SEAM
            } else {
                APPLE
            }
        },
    );
    model(&mut s, APPLE, 0, 115, |x, y| {
        oval(x, y, (axis, 4.6), (1.6, 0.9))
    });
    for side in [-1.0, 1.0] {
        model(&mut s, COMPOUND, 0, 116, |x, y| {
            oval(x, y, (axis + side * 1.7, 2.4), (2.3, 2.1))
        });
    }
    s
}

/// The dark seams down an emperor's thorax.
const THORAX_SEAM: Ramp = Ramp::new(0x1c3416, 0x284a1e, 0x34602a, 0x4a7a34, 0x6a9a48);

// ---------------------------------------------------------------------------------------------
// Brushwork for the close-ups
// ---------------------------------------------------------------------------------------------

/// A point on a close-up, in pixels, and a straight line between two.
type Point = (f32, f32);
type Segment = [Point; 2];

/// Whether a pixel's middle is inside the ellipse about `centre`.
fn oval(x: i32, y: i32, centre: (f32, f32), size: (f32, f32)) -> bool {
    let (dx, dy) = (
        (x as f32 + 0.5 - centre.0) / size.0,
        (y as f32 + 0.5 - centre.1) / size.1,
    );
    dx * dx + dy * dy <= 1.0
}

/// Up to most of a pixel more for a furred edge, so a body's outline is ragged.
fn fuzz(x: i32, y: i32, salt: u32) -> f32 {
    (noise(x, y, salt) & 0xff) as f32 / 255.0 * 0.9 - 0.3
}

/// How far `p` is from the segment `a` to `b`.
fn to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let t = along(p, a, b);
    (p.0 - a.0 - t * (b.0 - a.0)).hypot(p.1 - a.1 - t * (b.1 - a.1))
}

/// How far along the segment `a` to `b` the point nearest `p` is, from 0 to 1.
fn along(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dy * dy;
    if length == 0.0 {
        return 0.0;
    }
    (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length).clamp(0.0, 1.0)
}

/// Whether `(x, y)` is inside the polygon `points`, by the even-odd rule.
fn within(points: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut inside = false;
    for (index, &(x0, y0)) in points.iter().enumerate() {
        let (x1, y1) = points[(index + 1) % points.len()];
        if (y0 <= y) != (y1 <= y) && x < x0 + (y - y0) / (y1 - y0) * (x1 - x0) {
            inside = !inside;
        }
    }
    inside
}

/// The pixel across the middle of a specimen from `x`: a specimen is set out either side of
/// the line `axis`, and its two halves are mirror images, pixel for pixel.
fn mirror(axis: f32, x: f32) -> i32 {
    if x >= axis {
        x.floor() as i32
    } else {
        (2.0 * axis) as i32 - 1 - (2.0 * axis - x).floor() as i32
    }
}

/// How deep each pixel of a shape lies: 0 outside it, 1 on its rim, and one more for every
/// pixel further in, up to 8.
struct Depth {
    width: i32,
    height: i32,
    depth: Vec<u8>,
}

impl Depth {
    fn of(width: u32, height: u32, inside: impl Fn(i32, i32) -> bool) -> Self {
        let (w, h) = (width as i32, height as i32);
        let mut depth: Vec<u8> = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| if inside(x, y) { 8 } else { 0 })
            .collect();
        for level in 1..8 {
            let before = depth.clone();
            let at = |x: i32, y: i32| {
                if x < 0 || y < 0 || x >= w || y >= h {
                    0
                } else {
                    before[(y * w + x) as usize]
                }
            };
            for y in 0..h {
                for x in 0..w {
                    let index = (y * w + x) as usize;
                    let edge = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                        .iter()
                        .any(|&(nx, ny)| at(nx, ny) < level);
                    if before[index] > level && edge {
                        depth[index] = level;
                    }
                }
            }
        }
        Self {
            width: w,
            height: h,
            depth,
        }
    }

    fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            0
        } else {
            self.depth[(y * self.width + x) as usize]
        }
    }
}

/// Every pixel along the segments `lines`, each from one point to the next.
fn traced(lines: &[[(f32, f32); 2]]) -> Vec<(i32, i32)> {
    let mut pixels = Vec::new();
    for &[from, to] in lines {
        let (mut x, mut y) = (from.0.floor() as i32, from.1.floor() as i32);
        let (x1, y1) = (to.0.floor() as i32, to.1.floor() as i32);
        let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
        let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
        let mut error = dx + dy;
        loop {
            pixels.push((x, y));
            if (x, y) == (x1, y1) {
                break;
            }
            let twice = 2 * error;
            if twice >= dy {
                error += dy;
                x += sx;
            }
            if twice <= dx {
                error += dx;
                y += sy;
            }
        }
    }
    pixels
}

/// A flat wing filling `outline`, each pixel's material from `pattern` (given where it is and
/// how deep in the wing it lies). Its rim takes each material's edge; just inside, the side
/// towards the upper left catches the light and the side away from it is in shade; veins run
/// a shade darker; and noise scatters lighter and darker scales over the rest.
fn wing(
    s: &mut Canvas,
    outline: &[(f32, f32)],
    veins: &[[(f32, f32); 2]],
    salt: u32,
    pattern: &dyn Fn(f32, f32, u8) -> Ramp,
) {
    let depth = Depth::of(s.width(), s.height(), |x, y| {
        within(outline, x as f32 + 0.5, y as f32 + 0.5)
    });
    let veined = traced(veins);
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            let d = depth.at(x, y);
            if d == 0 {
                continue;
            }
            let ramp = pattern(x as f32 + 0.5, y as f32 + 0.5, d);
            let color = if d == 1 {
                ramp.edge
            } else if veined.contains(&(x, y)) {
                ramp.shadow
            } else if d == 2 && (depth.at(x - 1, y) == 1 || depth.at(x, y - 1) == 1) {
                ramp.light
            } else if d == 2 && (depth.at(x + 1, y) == 1 || depth.at(x, y + 1) == 1) {
                ramp.shadow
            } else if chance(x, y, salt, 28) {
                ramp.light
            } else if chance(x, y, salt.wrapping_add(1), 16) {
                ramp.shadow
            } else {
                ramp.base
            };
            put(s, x, y, color);
        }
    }
}

/// A pair of wings either side of `axis`, `outline` and `veins` given for the right-hand one
/// as distances out from the axis and down the canvas. `pattern` is given the same, so the
/// markings match; the light, which comes from the upper left, does not.
fn pair(
    s: &mut Canvas,
    axis: f32,
    outline: &[(f32, f32)],
    veins: &[[(f32, f32); 2]],
    salt: u32,
    pattern: &dyn Fn(f32, f32, u8) -> Ramp,
) {
    for side in [1.0, -1.0] {
        let (points, lines) = sided(axis, side, outline, veins);
        wing(s, &points, &lines, salt, &|x, y, depth| {
            pattern((x - axis).abs(), y, depth)
        });
    }
}

/// A wing's outline and veins put on one side of `axis`.
fn sided(
    axis: f32,
    side: f32,
    outline: &[(f32, f32)],
    veins: &[[(f32, f32); 2]],
) -> (Vec<Point>, Vec<Segment>) {
    let points = outline
        .iter()
        .map(|&(dx, y)| (axis + side * dx, y))
        .collect();
    let lines = veins
        .iter()
        .map(|&[a, b]| [(axis + side * a.0, a.1), (axis + side * b.0, b.1)])
        .collect();
    (points, lines)
}

/// A pair of clear wings either side of `axis`: veined all round and along, netted faintly
/// between, the membrane only `tint`.
fn glass_pair(
    s: &mut Canvas,
    axis: f32,
    outline: &[(f32, f32)],
    veins: &[[(f32, f32); 2]],
    salt: u32,
    tint: Rgba,
) {
    for side in [1.0, -1.0] {
        let (points, lines) = sided(axis, side, outline, veins);
        let depth = Depth::of(s.width(), s.height(), |x, y| {
            within(&points, x as f32 + 0.5, y as f32 + 0.5)
        });
        let veined = traced(&lines);
        for y in 0..s.height() as i32 {
            for x in 0..s.width() as i32 {
                let color = match depth.at(x, y) {
                    0 => continue,
                    1 => RIM_VEIN,
                    _ if veined.contains(&(x, y)) => LONG_VEIN,
                    _ if chance(x, y, salt, 60) => CROSS_VEIN,
                    _ => tint,
                };
                put(s, x, y, color);
            }
        }
    }
}

/// Shades the solid that `inside` describes as something rounded, lit from the upper left, in
/// `ramp`: see `shade`.
fn model(s: &mut Canvas, ramp: Ramp, grain: u32, salt: u32, inside: impl Fn(i32, i32) -> bool) {
    shade(s, grain, salt, inside, |_, _| ramp);
}

/// Shades the solid that `inside` describes as something rounded, lit from the upper left,
/// each pixel in the ramp `ramp` gives it: how far across and down the solid a pixel sits
/// stands in for the way the surface there faces. Noise dithers the bands so they don't step,
/// `grain` in every 256 pixels are a shade darker for texture, and the outermost pixels take
/// the edge tone.
fn shade(
    s: &mut Canvas,
    grain: u32,
    salt: u32,
    inside: impl Fn(i32, i32) -> bool,
    ramp: impl Fn(i32, i32) -> Ramp,
) {
    for y in 0..s.height() as i32 {
        for x in 0..s.width() as i32 {
            if !inside(x, y) {
                continue;
            }
            let ramp = ramp(x, y);
            let reach = |dx: i32, dy: i32| {
                let mut n = 0;
                while n < 64 && inside(x + dx * (n + 1), y + dy * (n + 1)) {
                    n += 1;
                }
                n
            };
            let (left, right, up, down) = (reach(-1, 0), reach(1, 0), reach(0, -1), reach(0, 1));
            if left.min(right).min(up).min(down) == 0 {
                put(s, x, y, ramp.edge);
                continue;
            }
            let across = (left - right) as f32 / (left + right) as f32;
            let along = (up - down) as f32 / (up + down) as f32;
            let jitter = (noise(x, y, salt) & 0xff) as f32 / 255.0 - 0.5;
            let lit = -0.55 * across - 0.45 * along + 0.3 * jitter;
            let mut tone = if lit > 0.75 {
                ramp.shine
            } else if lit > 0.3 {
                ramp.light
            } else if lit > -0.3 {
                ramp.base
            } else {
                ramp.shadow
            };
            if chance(x, y, salt.wrapping_add(1), grain) {
                tone = if tone == ramp.shine || tone == ramp.light {
                    ramp.base
                } else {
                    ramp.shadow
                };
            }
            put(s, x, y, tone);
        }
    }
}

/// Legs or feelers along `path`, given for the right-hand one as distances out from `axis` and
/// down the canvas, and their mirror image on the left: a pixel wide in `color`.
fn limbs(s: &mut Canvas, axis: f32, path: &[(f32, f32)], color: Rgba) {
    for side in [1.0, -1.0] {
        for pair in path.windows(2) {
            let ends = pair
                .iter()
                .map(|&(dx, y)| (mirror(axis, axis + side * dx), y as i32));
            let ends: Vec<(i32, i32)> = ends.collect();
            line(s, ends[0], ends[1], color);
        }
    }
}

/// Legs along `path` and its mirror image, two pixels thick: lit along the top, the ramp's
/// edge beneath.
fn thick_limbs(s: &mut Canvas, axis: f32, path: &[(f32, f32)], ramp: Ramp) {
    for side in [1.0, -1.0] {
        let points: Vec<(i32, i32)> = path
            .iter()
            .map(|&(dx, y)| (mirror(axis, axis + side * dx), y as i32))
            .collect();
        thick_path(s, &points, ramp);
    }
}

/// A limb along `path`, two pixels thick: lit along the top, the ramp's edge beneath.
fn thick_path(s: &mut Canvas, path: &[(i32, i32)], ramp: Ramp) {
    let lines: Vec<[(f32, f32); 2]> = path
        .windows(2)
        .map(|pair| {
            [
                (pair[0].0 as f32, pair[0].1 as f32),
                (pair[1].0 as f32, pair[1].1 as f32),
            ]
        })
        .collect();
    let pixels = traced(&lines);
    for &(x, y) in &pixels {
        put(s, x, y + 1, ramp.edge);
    }
    for &(x, y) in &pixels {
        put(s, x, y, ramp.base);
    }
}

/// Clubbed feelers along `path` and its mirror image, tipped in `tip`.
fn feelers(s: &mut Canvas, axis: f32, path: &[(f32, f32)], color: Rgba, tip: Rgba) {
    limbs(s, axis, path, color);
    if let Some(&(dx, y)) = path.last() {
        for side in [1.0, -1.0] {
            let x = mirror(axis, axis + side * dx);
            put(s, x, y as i32, tip);
            put(s, x - side as i32, y as i32, color);
        }
    }
}

/// A moth's feathered antenna from `from` to `to`: a shaft with barbs either side of it that
/// grow longer towards the middle.
fn feather(s: &mut Canvas, from: (f32, f32), to: (f32, f32)) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.hypot(dy);
    let (nx, ny) = (-dy / length, dx / length);
    let steps = (length * 2.0) as i32;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let (x, y) = (from.0 + dx * t, from.1 + dy * t);
        let reach = 1.6 * (std::f32::consts::PI * t).sin();
        for side in [-1.0, 1.0] {
            let (bx, by) = (x + nx * reach * side, y + ny * reach * side);
            put(s, bx.floor() as i32, by.floor() as i32, FEATHER.light);
        }
    }
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let (x, y) = (from.0 + dx * t, from.1 + dy * t);
        put(s, x.floor() as i32, y.floor() as i32, FEATHER.shadow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meadow::bugs::CATALOGUE;

    const POSES: [Pose; 3] = [Pose::Flying, Pose::Settled, Pose::Crawling];

    fn drawn(canvas: &Canvas) -> usize {
        canvas.pixels().iter().filter(|pixel| pixel.a > 0).count()
    }

    #[test]
    fn every_bug_is_drawn_in_every_pose_and_frame() {
        for bug in &CATALOGUE {
            for pose in POSES {
                let count = frames(bug.id, pose);
                assert!(count >= 1, "{} has no {pose:?} frames", bug.id);
                for frame in 0..count {
                    let picture = sprite(bug.id, pose, frame);
                    assert!(
                        drawn(&picture) >= 8,
                        "{} is barely there {pose:?} at frame {frame}",
                        bug.id
                    );
                }
            }
            assert!(drawn(&close_up(bug.id)) >= 200, "{} close up", bug.id);
        }
    }

    #[test]
    fn a_bug_is_one_size_in_every_frame() {
        // Steady across poses as well as frames, so a bug settling or taking off doesn't jump.
        for bug in &CATALOGUE {
            let first = sprite(bug.id, Pose::Settled, 0);
            for pose in POSES {
                for frame in 0..frames(bug.id, pose) {
                    let picture = sprite(bug.id, pose, frame);
                    assert_eq!(
                        (picture.width(), picture.height()),
                        (first.width(), first.height()),
                        "{} changes size {pose:?} at frame {frame}",
                        bug.id
                    );
                }
            }
        }
    }

    #[test]
    fn every_row_of_a_frame_is_as_wide_as_the_sprite() {
        for bug in &CATALOGUE {
            let look = look(bug.id).expect("every bug in the catalogue is drawn");
            for pose in POSES {
                for grid in look.frames(pose) {
                    assert_eq!(grid.len() as u32, look.size.1, "{} {pose:?}", bug.id);
                    for row in grid.iter() {
                        assert_eq!(row.len() as u32, look.size.0, "{} {row:?}", bug.id);
                        for letter in row.bytes() {
                            assert!(
                                letter == b'.' || look.inks.iter().any(|(name, _)| *name == letter),
                                "{} has no ink for {:?}",
                                bug.id,
                                letter as char
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn no_letter_names_two_inks() {
        for bug in &CATALOGUE {
            let look = look(bug.id).expect("every bug in the catalogue is drawn");
            let (_, icon_inks) = icon_rows(bug.id).expect("every bug has an icon");
            for inks in [look.inks, icon_inks] {
                for (index, (name, _)) in inks.iter().enumerate() {
                    assert!(
                        inks[index + 1..].iter().all(|(other, _)| other != name),
                        "{} has two inks for {:?}",
                        bug.id,
                        *name as char
                    );
                }
            }
        }
    }

    #[test]
    fn every_icon_row_is_icon_wide_and_inked() {
        for bug in &CATALOGUE {
            let (rows, inks) = icon_rows(bug.id).expect("every bug has an icon");
            for row in rows.iter() {
                assert_eq!(row.len() as u32, ICON, "{} {row:?}", bug.id);
                for letter in row.bytes() {
                    assert!(
                        letter == b'.' || inks.iter().any(|(name, _)| *name == letter),
                        "{}'s icon has no ink for {:?}",
                        bug.id,
                        letter as char
                    );
                }
            }
        }
    }

    #[test]
    fn the_anchor_lies_on_the_bug() {
        for bug in &CATALOGUE {
            for pose in POSES {
                let (x, y) = anchor(bug.id, pose);
                for frame in 0..frames(bug.id, pose) {
                    let picture = sprite(bug.id, pose, frame);
                    assert!(
                        x >= 0
                            && y >= 0
                            && x < picture.width() as i32
                            && y < picture.height() as i32,
                        "{}'s {pose:?} anchor is off it",
                        bug.id
                    );
                    let near = (-1..=1)
                        .flat_map(|dy| (-1..=1).map(move |dx| (x + dx, y + dy)))
                        .any(|(nx, ny)| picture.get(nx, ny).a > 0);
                    assert!(
                        near,
                        "{}'s {pose:?} anchor is in the air at frame {frame}",
                        bug.id
                    );
                }
            }
        }
    }

    #[test]
    fn icons_are_icon_square_and_filled() {
        for bug in &CATALOGUE {
            let icon = icon(bug.id);
            assert_eq!((icon.width(), icon.height()), (ICON, ICON), "{}", bug.id);
            assert!(drawn(&icon) >= 12, "{}'s icon is barely there", bug.id);
        }
    }

    #[test]
    fn a_bug_hill_does_not_know_is_still_drawn() {
        for pose in POSES {
            assert_eq!(frames("unicorn_beetle", pose), 1);
            assert!(drawn(&sprite("unicorn_beetle", pose, 0)) > 0);
        }
        assert!(drawn(&close_up("unicorn_beetle")) > 0);
        assert_eq!(icon("unicorn_beetle").width(), ICON);
    }
}
