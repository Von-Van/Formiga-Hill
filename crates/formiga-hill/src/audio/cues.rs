//! Every effect at the Hill, as a cue: what happened, and the sound it makes. Each sound is a
//! placeholder made in code, from the synthesiser's parts, at the moment it is played, until real
//! audio replaces it.
//!
//! A cue played again and again (a chuff, a hop, a hammer tap) comes out a little different each
//! time, so a run of them sounds like the thing itself and not a recording; but the `n`th time is
//! always the same, so every sound can be listened to in a test.

use super::synth::{DcBlock, Noise, Phase, Smooth, Svf, ease, hz as note_hz, sine, strike, swell};
use crate::dice::Dice;

/// Something that makes a sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cue {
    // The train, as `station::Journey::heard` has it.
    /// Its whistle, as it pulls in and as it pulls out.
    Whistle,
    /// A puff of steam from the engine, one for each turn of its driving wheels.
    Chuff,
    /// The brakes letting off steam as it comes to a stop.
    Brakes,

    // The green, and anywhere a companion is held something out.
    /// A pat.
    Pat,
    /// A snack, crunched.
    Snack,
    /// A squeaky toy.
    Toy,
    /// A stroke of the brush.
    Brush,
    /// Something from the dress-up box put on.
    Costume,

    // The camera, and a story.
    /// The shutter.
    Shutter,
    /// Reading on, and a page turned.
    Page,
    /// A choice made.
    Choice,

    // The Fairground.
    /// Hide-and-seek: "Ready or not, here I come!"
    ReadyOrNot,
    /// Hide-and-seek: a hider found.
    Found,
    /// The sack race: the whistle for "go".
    StartWhistle,
    /// The sack race: a hop landing.
    Hop,
    /// The high striker: the mallet coming down, and the puck going up.
    Strike,
    /// The high striker's bell.
    Bell,
    /// Hoopla: a ring clinking off a peg, or over a block.
    Clink,
    /// Hoopla: a prize won.
    Prize,
    /// The tug-of-war: everyone heaving on the rope.
    Heave,
    /// The tug-of-war: the winners' cheer.
    Cheer,

    // The Hilltop.
    /// Something stood on a spot.
    Thunk,
    /// A hammer tap, while the colony builds.
    Hammer,
    /// The puff of dust what it built appears in, or comes apart in.
    Puff,
    /// Something has grown since the last visit.
    Grown,

    /// A quiet click, for the window's own controls, used sparingly.
    Click,

    // The Woods: made here, and played by the Woods' own windows.
    /// Rummaging at a spot in the glade.
    Rummage,
    /// Fishing: the line cast out over the pool.
    Cast,
    /// Bug catching: a bug's wings.
    Flutter,
    /// Foraging: a paw into the hedgerow.
    Forage,
    /// Something turned up.
    Find,
    /// A fish breaking the water, or something going into it.
    Splash,
    /// The bug net swung.
    Swish,
    /// Something ripe picked.
    Pick,
    /// Treasure dug up.
    Treasure,

    // The clearing.
    /// The moment it is reached.
    Sting,
    /// The moment it is over.
    Bested,
}

impl Cue {
    pub const ALL: [Cue; 37] = [
        Cue::Whistle,
        Cue::Chuff,
        Cue::Brakes,
        Cue::Pat,
        Cue::Snack,
        Cue::Toy,
        Cue::Brush,
        Cue::Costume,
        Cue::Shutter,
        Cue::Page,
        Cue::Choice,
        Cue::ReadyOrNot,
        Cue::Found,
        Cue::StartWhistle,
        Cue::Hop,
        Cue::Strike,
        Cue::Bell,
        Cue::Clink,
        Cue::Prize,
        Cue::Heave,
        Cue::Cheer,
        Cue::Thunk,
        Cue::Hammer,
        Cue::Puff,
        Cue::Grown,
        Cue::Click,
        Cue::Rummage,
        Cue::Cast,
        Cue::Flutter,
        Cue::Forage,
        Cue::Find,
        Cue::Splash,
        Cue::Swish,
        Cue::Pick,
        Cue::Treasure,
        Cue::Sting,
        Cue::Bested,
    ];

    /// Its name, for a file of it: "start-whistle".
    pub fn name(self) -> &'static str {
        match self {
            Cue::Whistle => "whistle",
            Cue::Chuff => "chuff",
            Cue::Brakes => "brakes",
            Cue::Pat => "pat",
            Cue::Snack => "snack",
            Cue::Toy => "toy",
            Cue::Brush => "brush",
            Cue::Costume => "costume",
            Cue::Shutter => "shutter",
            Cue::Page => "page",
            Cue::Choice => "choice",
            Cue::ReadyOrNot => "ready-or-not",
            Cue::Found => "found",
            Cue::StartWhistle => "start-whistle",
            Cue::Hop => "hop",
            Cue::Strike => "strike",
            Cue::Bell => "bell",
            Cue::Clink => "clink",
            Cue::Prize => "prize",
            Cue::Heave => "heave",
            Cue::Cheer => "cheer",
            Cue::Thunk => "thunk",
            Cue::Hammer => "hammer",
            Cue::Puff => "puff",
            Cue::Grown => "grown",
            Cue::Click => "click",
            Cue::Rummage => "rummage",
            Cue::Cast => "cast",
            Cue::Flutter => "flutter",
            Cue::Forage => "forage",
            Cue::Find => "find",
            Cue::Splash => "splash",
            Cue::Swish => "swish",
            Cue::Pick => "pick",
            Cue::Treasure => "treasure",
            Cue::Sting => "secret-sting",
            Cue::Bested => "secret-bested",
        }
    }

    /// Its place in `ALL`.
    pub fn index(self) -> usize {
        Cue::ALL
            .iter()
            .position(|cue| *cue == self)
            .unwrap_or_default()
    }

    /// The least time between two of it, so a crowd of them is heard as a patter and not a roar.
    pub fn spacing(self) -> f32 {
        match self {
            Cue::Hop | Cue::Brush => 0.07,
            Cue::Chuff | Cue::Hammer | Cue::Clink => 0.04,
            Cue::Click => 0.05,
            _ => 0.0,
        }
    }

    /// The sound, the `variation`th time it is played, made at `rate` samples a second.
    pub fn voice(self, variation: u32, rate: f32) -> Voice {
        let mut dice = Dice::new(
            u64::from(variation).wrapping_mul(0x9e37_79b9) ^ ((self.index() as u64 + 1) << 40),
        );
        let seed = dice.next_u64() as u32;
        match self {
            Cue::Whistle => whistle(rate, seed),
            Cue::Chuff => chuff(rate, seed, dice.range(0.9, 1.12)),
            Cue::Brakes => brakes(rate, seed),
            Cue::Pat => pat(rate, seed, dice.range(0.92, 1.08)),
            Cue::Snack => snack(rate, seed),
            Cue::Toy => toy(rate, dice.range(0.95, 1.05)),
            Cue::Brush => brush(rate, seed, dice.range(0.9, 1.1)),
            Cue::Costume => costume(rate, seed),
            Cue::Shutter => shutter(rate, seed),
            Cue::Page => page(rate, seed),
            Cue::Choice => choice(rate),
            Cue::ReadyOrNot => ready_or_not(rate, seed),
            Cue::Found => found(rate),
            Cue::StartWhistle => start_whistle(rate, seed),
            Cue::Hop => hop(rate, seed, dice.range(0.85, 1.15)),
            Cue::Strike => strike_sound(rate, seed),
            Cue::Bell => bell(rate, seed),
            Cue::Clink => clink(rate, seed, dice.range(0.94, 1.06)),
            Cue::Prize => prize(rate),
            Cue::Heave => heave(rate, seed, dice.range(0.9, 1.1)),
            Cue::Cheer => cheer(rate, seed),
            Cue::Thunk => thunk(rate, seed, dice.range(0.92, 1.08)),
            Cue::Hammer => hammer(rate, seed, dice.range(0.9, 1.1)),
            Cue::Puff => puff(rate, seed),
            Cue::Grown => grown(rate),
            Cue::Click => click(rate, seed),
            Cue::Rummage => rummage(rate, seed),
            Cue::Cast => cast(rate, seed),
            Cue::Flutter => flutter(rate, seed),
            Cue::Forage => forage(rate, seed),
            Cue::Find => find(rate),
            Cue::Splash => splash(rate, seed),
            Cue::Swish => swish(rate, seed, dice.range(0.92, 1.08)),
            Cue::Pick => pick(rate, seed, dice.range(0.92, 1.08)),
            Cue::Treasure => treasure(rate, seed),
            Cue::Sting => sting(rate, seed),
            Cue::Bested => bested(rate),
        }
    }
}

/// How long the last part of every sound takes to fade to nothing, so none ends on a click.
const FADE_SECS: f32 = 0.006;

/// One sound playing: made a sample at a time until it is over.
pub struct Voice {
    rate: f32,
    made: usize,
    length: usize,
    loudness: f32,
    dc: DcBlock,
    make: Box<dyn FnMut(f32) -> f32 + Send>,
}

impl Voice {
    fn new(rate: f32, seconds: f32, make: impl FnMut(f32) -> f32 + Send + 'static) -> Self {
        Self {
            rate,
            made: 0,
            length: (seconds * rate) as usize,
            loudness: 1.0,
            dc: DcBlock::new(rate),
            make: Box::new(make),
        }
    }

    /// The same sound, `by` as loud: a train further off, a tap made softly.
    pub fn louder(mut self, by: f32) -> Self {
        self.loudness *= by.max(0.0);
        self
    }
}

impl Iterator for Voice {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.made >= self.length {
            return None;
        }
        let t = self.made as f32 / self.rate;
        let left = (self.length - self.made) as f32 / self.rate;
        self.made += 1;
        let fade = (left / FADE_SECS).min(1.0);
        Some(self.dc.run((self.make)(t)) * fade * self.loudness)
    }
}

/// Partials of a struck thing: how far above the note each is, how loud, and how many seconds it
/// takes to die away by a factor of e.
type Partials = &'static [(f32, f32, f32)];

/// A soft chime: nearly a pure tone, with a bell's faint overtones.
const CHIME: Partials = &[(1.0, 1.0, 0.9), (2.0, 0.22, 0.45), (3.0, 0.08, 0.25)];
/// A bright bell, its overtones out of tune with the note as a bell's are.
const BELL: Partials = &[
    (1.0, 1.0, 1.5),
    (2.0, 0.5, 0.9),
    (2.76, 0.35, 0.6),
    (4.07, 0.2, 0.32),
    (5.4, 0.12, 0.18),
];
/// A marimba's wooden bar.
const BAR: Partials = &[(1.0, 1.0, 0.32), (3.93, 0.12, 0.05), (9.2, 0.04, 0.02)];
/// A hollow wooden block, tapped.
const BLOCK: Partials = &[(1.0, 1.0, 0.035), (2.71, 0.35, 0.018), (4.4, 0.1, 0.01)];
/// A thin ring of wood or glass.
const RING: Partials = &[
    (1.0, 1.0, 0.13),
    (1.58, 0.6, 0.09),
    (2.33, 0.4, 0.06),
    (3.1, 0.2, 0.04),
];

/// Notes struck one after another, each ringing on with the same partials.
struct Struck {
    partials: Partials,
    /// When each note is struck, its pitch, and how loud.
    notes: Vec<(f32, f32, f32)>,
    phases: Vec<Phase>,
}

impl Struck {
    fn new(partials: Partials, notes: &[(f32, f32, f32)]) -> Self {
        Self {
            partials,
            notes: notes.to_vec(),
            phases: vec![Phase::default(); notes.len() * partials.len()],
        }
    }

    fn run(&mut self, t: f32, rate: f32) -> f32 {
        let mut out = 0.0;
        for (index, &(at, hz, loud)) in self.notes.iter().enumerate() {
            let since = t - at;
            if since < 0.0 {
                continue;
            }
            for (part, &(ratio, amp, decay)) in self.partials.iter().enumerate() {
                let phase = &mut self.phases[index * self.partials.len() + part];
                let turn = phase.tick(hz * ratio, rate);
                out += sine(turn) * amp * loud * strike(since, 0.0015, decay);
            }
        }
        out
    }
}

/// Noise through a band-pass, swept wherever each sound wants it.
struct Hiss {
    noise: Noise,
    filter: Svf,
}

impl Hiss {
    fn new(seed: u32, rate: f32) -> Self {
        Self {
            noise: Noise::new(seed),
            filter: Svf::new(1000.0, 1.0, rate),
        }
    }

    fn band(&mut self, hz: f32, q: f32) -> f32 {
        self.filter.tune(hz, q);
        self.filter.run(self.noise.next()).band
    }

    fn low(&mut self, hz: f32) -> f32 {
        self.filter.tune(hz, 0.7);
        self.filter.run(self.noise.next()).low
    }
}

/// A thump: a low sine dropping in pitch, like something soft landing.
struct Thump {
    phase: Phase,
}

impl Thump {
    fn new() -> Self {
        Self {
            phase: Phase::default(),
        }
    }

    fn run(&mut self, t: f32, from: f32, to: f32, rate: f32) -> f32 {
        let hz = to + (from - to) * (-t / 0.03).exp();
        sine(self.phase.tick(hz, rate))
    }
}

/// A sung or blown line: a tone gliding between the notes of a tune, with each note's own swell.
struct Line {
    /// Each note: when it starts, its pitch, and how long it lasts.
    notes: Vec<(f32, f32, f32)>,
    glide: Smooth,
    phase: Phase,
    vibrato: Phase,
}

impl Line {
    fn new(notes: &[(f32, f32, f32)], rate: f32) -> Self {
        // Starting on the first note, not sliding up to it from nothing.
        let first = notes.first().map_or(440.0, |&(_, hz, _)| hz);
        Self {
            notes: notes.to_vec(),
            glide: Smooth::new(30.0, rate).starting(first),
            phase: Phase::default(),
            vibrato: Phase::default(),
        }
    }

    /// The tone `t` seconds in, with `harmonics` above it, and its loudness.
    fn run(&mut self, t: f32, harmonics: &[f32], rate: f32) -> (f32, f32) {
        let now = self
            .notes
            .iter()
            .rev()
            .find(|(at, _, _)| t >= *at)
            .or(self.notes.first())
            .copied()
            .unwrap_or((0.0, 440.0, 0.0));
        let (at, target, length) = now;
        let loud = swell(t - at, 0.03, length, 0.05);
        let wobble = 1.0 + 0.006 * ease((t - at - 0.12) / 0.2) * sine(self.vibrato.tick(5.5, rate));
        let hz = self.glide.run(target) * wobble;
        let turn = self.phase.tick(hz, rate);
        let mut out = sine(turn);
        for (index, amp) in harmonics.iter().enumerate() {
            out += amp * sine(turn * (index as f32 + 2.0));
        }
        (out, loud)
    }
}

fn whistle(rate: f32, seed: u32) -> Voice {
    // A steam whistle of three pipes, a major chord: a short toot, then a long one.
    let pipes = [587.3, 740.0, 880.0];
    let mut phases = [Phase::at(0.0), Phase::at(0.3), Phase::at(0.6)];
    let mut breath = Hiss::new(seed, rate);
    let mut wobble = Phase::default();
    Voice::new(rate, 1.85, move |t| {
        let blast = swell(t, 0.05, 0.3, 0.07) + swell(t - 0.44, 0.07, 0.9, 0.38);
        let since = if t < 0.44 { t } else { t - 0.44 };
        // Each blast comes up to pitch as the steam builds.
        let bend =
            (1.0 - 0.04 * (-since / 0.07).exp()) * (1.0 + 0.003 * sine(wobble.tick(5.0, rate)));
        let mut tone = 0.0;
        for ((phase, hz), amp) in phases.iter_mut().zip(pipes).zip([0.5, 0.36, 0.3]) {
            let turn = phase.tick(hz * bend, rate);
            tone += amp * (sine(turn) + 0.12 * sine(turn * 2.0));
        }
        let air = breath.band(1300.0, 1.2) * 0.35;
        (tone + air) * blast * 0.34
    })
}

fn chuff(rate: f32, seed: u32, pitch: f32) -> Voice {
    let mut steam = Hiss::new(seed, rate);
    let mut breath = Hiss::new(seed ^ 0x55, rate);
    Voice::new(rate, 0.3, move |t| {
        // A puff of steam, darkening as it opens out, and a softer breath trailing after.
        let puff = steam.band(pitch * (450.0 + 1400.0 * (-t / 0.03).exp()), 0.8);
        let after = breath.band(1700.0 * pitch, 0.7);
        puff * strike(t, 0.004, 0.06) * 0.78 + after * strike(t, 0.01, 0.1) * 0.1
    })
}

fn brakes(rate: f32, seed: u32) -> Voice {
    // Steam let off as the train stops: a long "psssh", falling and softening.
    let mut steam = Hiss::new(seed, rate);
    let mut clank = Thump::new();
    Voice::new(rate, 1.5, move |t| {
        let hiss = steam.band(3400.0 - 1600.0 * ease(t / 1.2), 0.9) * swell(t, 0.07, 0.75, 0.6);
        let clunk = clank.run(t, 160.0, 90.0, rate) * strike(t, 0.003, 0.06);
        hiss * 0.34 + clunk * 0.28
    })
}

fn pat(rate: f32, seed: u32, pitch: f32) -> Voice {
    // Two soft pats on fur.
    let mut fur = Hiss::new(seed, rate);
    let mut body = Thump::new();
    Voice::new(rate, 0.36, move |t| {
        let pats = strike(t, 0.004, 0.035) + 0.8 * strike(t - 0.15, 0.004, 0.035);
        let since = if t < 0.15 { t } else { t - 0.15 };
        let thump = body.run(since, 210.0 * pitch, 130.0 * pitch, rate);
        (fur.low(1100.0) * 1.3 + thump * 0.5) * pats * 0.6
    })
}

fn snack(rate: f32, seed: u32) -> Voice {
    // Crunch, crunch, crunch, and a happy little "mm".
    let mut crunch = Hiss::new(seed, rate);
    let mut grain = Noise::new(seed ^ 0xa5);
    let mut hum = Phase::default();
    let bites = [0.0, 0.12, 0.25];
    Voice::new(rate, 0.62, move |t| {
        let bite: f32 = bites.iter().map(|at| strike(t - at, 0.002, 0.03)).sum();
        let crackle = 0.4 + 0.6 * grain.unit();
        let crunching = crunch.band(2600.0, 0.9) * crackle * bite * 0.8;
        let mm = swell(t - 0.38, 0.04, 0.12, 0.08);
        let hz = 520.0 + 140.0 * ease((t - 0.38) / 0.15);
        crunching + sine(hum.tick(hz, rate)) * mm * 0.16
    })
}

fn toy(rate: f32, pitch: f32) -> Voice {
    // A squeaky toy, squeezed twice.
    let mut phase = Phase::default();
    let mut soften = Smooth::new(3500.0, rate);
    Voice::new(rate, 0.5, move |t| {
        let (since, height) = if t < 0.22 { (t, 1.0) } else { (t - 0.22, 0.92) };
        let rise = ease(since / 0.06) - 0.35 * ease((since - 0.07) / 0.1);
        let hz = (880.0 + 620.0 * rise) * pitch * height;
        let turn = phase.tick(hz, rate);
        let tone = sine(turn) + 0.32 * sine(turn * 2.0) + 0.14 * sine(turn * 3.0);
        let loud = swell(since, 0.015, 0.13, 0.05);
        soften.run(tone) * loud * 0.24
    })
}

fn brush(rate: f32, seed: u32, pitch: f32) -> Voice {
    // Bristles drawn through fur.
    let mut bristles = Hiss::new(seed, rate);
    let mut grain = Noise::new(seed ^ 0x3c);
    Voice::new(rate, 0.26, move |t| {
        let sweep = (2500.0 + 1500.0 * ease(t / 0.2)) * pitch;
        let texture = 0.6 + 0.4 * grain.unit();
        bristles.band(sweep, 0.8) * texture * swell(t, 0.05, 0.12, 0.09) * 0.36
    })
}

fn costume(rate: f32, seed: u32) -> Voice {
    // A rustle of fabric, and a little sparkle as it goes on.
    let mut cloth = Hiss::new(seed, rate);
    let mut flutter = Phase::default();
    let mut sparkle = Struck::new(CHIME, &[(0.2, 1318.5, 0.5), (0.31, 1975.5, 0.45)]);
    Voice::new(rate, 1.05, move |t| {
        let rustle = cloth.band(1900.0, 0.6)
            * (0.5 + 0.5 * sine(flutter.tick(17.0, rate)).abs())
            * swell(t, 0.04, 0.2, 0.12);
        rustle * 0.45 + sparkle.run(t, rate) * 0.3
    })
}

fn shutter(rate: f32, seed: u32) -> Voice {
    // Click-clack.
    let mut spring = Hiss::new(seed, rate);
    let mut ping = Phase::default();
    Voice::new(rate, 0.16, move |t| {
        let clicks = strike(t, 0.0005, 0.007) + 1.3 * strike(t - 0.075, 0.0005, 0.009);
        let ring = sine(ping.tick(if t < 0.075 { 1900.0 } else { 1500.0 }, rate))
            * (strike(t, 0.001, 0.02) + strike(t - 0.075, 0.001, 0.025));
        spring.band(2400.0, 2.5) * clicks * 0.7 + ring * 0.1
    })
}

fn page(rate: f32, seed: u32) -> Voice {
    // A page turned over: a swish of paper rising, and the soft flap of it settling.
    let mut paper = Hiss::new(seed, rate);
    let mut flap = Hiss::new(seed ^ 0x77, rate);
    let mut grain = Noise::new(seed ^ 0x19);
    Voice::new(rate, 0.5, move |t| {
        let sweep = 800.0 + 2200.0 * ease(t / 0.3);
        let swish = paper.band(sweep, 0.8) * (0.7 + 0.3 * grain.unit()) * swell(t, 0.12, 0.2, 0.15);
        let settle = flap.low(700.0) * strike(t - 0.3, 0.004, 0.04);
        swish * 0.38 + settle * 0.6
    })
}

fn choice(rate: f32) -> Voice {
    // Two notes on a wooden bar, rising: decided.
    let mut bar = Struck::new(BAR, &[(0.0, 784.0, 0.8), (0.09, 1046.5, 1.0)]);
    Voice::new(rate, 0.6, move |t| bar.run(t, rate) * 0.26)
}

fn ready_or_not(rate: f32, seed: u32) -> Voice {
    // A sing-song call, "rea-dy or NOT!", on an ocarina.
    let tune = [
        (0.0, note_hz(79.0), 0.16),
        (0.19, note_hz(76.0), 0.15),
        (0.37, note_hz(79.0), 0.14),
        (0.55, note_hz(84.0), 0.5),
    ];
    let mut line = Line::new(&tune, rate);
    let mut breath = Hiss::new(seed, rate);
    Voice::new(rate, 1.2, move |t| {
        let (tone, loud) = line.run(t, &[0.06, 0.03], rate);
        (tone + breath.band(2200.0, 1.0) * 0.08) * loud * 0.21
    })
}

fn found(rate: f32) -> Voice {
    // "Aha!": a bright rise, and a sparkle.
    let mut rise = Struck::new(CHIME, &[(0.0, 1046.5, 0.8), (0.1, 1568.0, 1.0)]);
    let mut sparkle = Struck::new(
        CHIME,
        &[
            (0.16, 3136.0, 0.25),
            (0.22, 3951.0, 0.2),
            (0.29, 4186.0, 0.18),
        ],
    );
    Voice::new(rate, 0.9, move |t| {
        (rise.run(t, rate) + sparkle.run(t, rate)) * 0.25
    })
}

fn start_whistle(rate: f32, seed: u32) -> Voice {
    // A pea whistle: "fweeet!", the pea rattling in it.
    let mut tone = Phase::default();
    let mut pea = Phase::default();
    let mut breath = Hiss::new(seed, rate);
    Voice::new(rate, 0.72, move |t| {
        let rattle = sine(pea.tick(30.0, rate));
        let hz = 2250.0 * (1.0 + 0.012 * rattle) * (1.0 - 0.03 * (-t / 0.04).exp());
        let turn = tone.tick(hz, rate);
        let pipe = (sine(turn) + 0.1 * sine(turn * 2.0)) * (0.78 + 0.22 * rattle);
        let air = breath.band(2300.0, 2.0) * 0.12;
        (pipe + air) * swell(t, 0.015, 0.5, 0.12) * 0.18
    })
}

fn hop(rate: f32, seed: u32, pitch: f32) -> Voice {
    // A sack landing on the sawdust: "fwump".
    let mut sack = Hiss::new(seed, rate);
    let mut body = Thump::new();
    Voice::new(rate, 0.2, move |t| {
        let thump = body.run(t, 190.0 * pitch, 115.0 * pitch, rate) * strike(t, 0.003, 0.045);
        let cloth = sack.low(1300.0 * pitch) * strike(t, 0.002, 0.03);
        (thump * 0.5 + cloth * 1.1) * 0.45
    })
}

fn strike_sound(rate: f32, seed: u32) -> Voice {
    // The mallet coming down on the pad, and the puck zipping up the track.
    let mut wood = Hiss::new(seed, rate);
    let mut body = Thump::new();
    let mut puck = Phase::default();
    let mut ratchet = Phase::default();
    Voice::new(rate, 0.62, move |t| {
        let thock = body.run(t, 170.0, 90.0, rate) * strike(t, 0.002, 0.07) * 0.7
            + wood.band(1400.0, 2.0) * strike(t, 0.0008, 0.014) * 0.6;
        let climb = t - 0.03;
        let hz = 330.0 * (1100.0f32 / 330.0).powf(ease(climb / 0.4));
        let turn = puck.tick(hz, rate);
        let zip = (sine(turn) + 0.11 * sine(turn * 3.0))
            * (0.7 + 0.3 * sine(ratchet.tick(38.0, rate)))
            * swell(climb, 0.03, 0.32, 0.12);
        (thock + zip * 0.13) * 0.7
    })
}

fn bell(rate: f32, seed: u32) -> Voice {
    // The fairground bell: "DING!"
    let mut bell = Struck::new(BELL, &[(0.0, 1180.0, 1.0)]);
    let mut hit = Hiss::new(seed, rate);
    Voice::new(rate, 2.4, move |t| {
        bell.run(t, rate) * 0.3 + hit.band(5000.0, 1.5) * strike(t, 0.0005, 0.006) * 0.3
    })
}

fn clink(rate: f32, seed: u32, pitch: f32) -> Voice {
    // A ring clinking on a peg.
    let mut ring = Struck::new(RING, &[(0.0, 2350.0 * pitch, 1.0)]);
    let mut tick = Hiss::new(seed, rate);
    Voice::new(rate, 0.32, move |t| {
        ring.run(t, rate) * 0.18 + tick.band(6000.0, 1.2) * strike(t, 0.0004, 0.004) * 0.24
    })
}

fn prize(rate: f32) -> Voice {
    // A prize won: a little fanfare of bells, up to the top note.
    let mut bells = Struck::new(
        CHIME,
        &[
            (0.04, 1046.5, 0.7),
            (0.13, 1318.5, 0.75),
            (0.22, 1568.0, 0.8),
            (0.33, 2093.0, 1.0),
        ],
    );
    Voice::new(rate, 1.3, move |t| bells.run(t, rate) * 0.21)
}

fn heave(rate: f32, seed: u32, pitch: f32) -> Voice {
    // The rope creaking as everyone leans back on it.
    let mut fibre = Noise::new(seed);
    let mut catch = Phase::default();
    let mut low = Svf::new(650.0 * pitch, 5.0, rate);
    let mut high = Svf::new(1750.0 * pitch, 6.0, rate);
    Voice::new(rate, 0.6, move |t| {
        // A creak is a run of tiny catches of the rope's fibres, faster as the strain grows.
        let hz = (55.0 + 45.0 * ease(t / 0.45)) * pitch;
        let before = catch.tick(hz, rate);
        let caught = if before + hz / rate >= 1.0 {
            0.5 + 0.5 * fibre.unit()
        } else {
            0.0
        };
        let creak = low.run(caught).band + 0.6 * high.run(caught).band;
        creak * swell(t, 0.06, 0.38, 0.15) * 8.0
    })
}

fn cheer(rate: f32, seed: u32) -> Voice {
    // A little crowd: "yaaay!", each voice rising, and clapping.
    let mut dice = Dice::new(u64::from(seed));
    let voices: Vec<(f32, f32, f32)> = (0..5)
        .map(|_| {
            (
                dice.range(0.0, 0.12),
                dice.range(380.0, 720.0),
                dice.range(0.75, 1.05),
            )
        })
        .collect();
    let mut phases = vec![Phase::default(); voices.len()];
    let mut vibratos: Vec<Phase> = (0..voices.len())
        .map(|index| Phase::at(index as f32 * 0.37))
        .collect();
    let mut claps = Hiss::new(seed ^ 0x11, rate);
    let mut chance = Noise::new(seed ^ 0x99);
    let mut clap = 0.0f32;
    let mut mouth = Svf::new(1100.0, 0.8, rate);
    Voice::new(rate, 1.9, move |t| {
        let mut crowd = 0.0;
        for (index, &(at, base, held)) in voices.iter().enumerate() {
            let since = t - at;
            if since < 0.0 {
                continue;
            }
            let wobble = 1.0 + 0.02 * sine(vibratos[index].tick(6.0, rate)) * ease(since / 0.4);
            let hz = base * (1.0 + 0.28 * ease(since / 0.25)) * wobble;
            let turn = phases[index].tick(hz, rate);
            let tone = sine(turn) + 0.45 * sine(turn * 2.0) + 0.25 * sine(turn * 3.0);
            crowd += tone * swell(since, 0.07, held, 0.4);
        }
        let voiced = mouth.run(crowd).band * 0.5 + crowd * 0.08;
        // Claps, now and then, thickest in the middle of the cheer.
        let busy = swell(t, 0.15, 1.2, 0.5);
        if chance.unit() < 0.0016 * busy {
            clap = 1.0;
        }
        clap *= 0.997;
        let clapping = claps.band(1500.0, 1.3) * clap * clap;
        (voiced * 0.32 + clapping * 0.4) * 0.42
    })
}

fn thunk(rate: f32, seed: u32, pitch: f32) -> Voice {
    // Something stood on the grass of the Hilltop.
    let mut body = Thump::new();
    let mut earth = Hiss::new(seed, rate);
    let mut grass = Hiss::new(seed ^ 0x44, rate);
    Voice::new(rate, 0.3, move |t| {
        let thump = body.run(t, 200.0 * pitch, 120.0 * pitch, rate) * strike(t, 0.002, 0.06);
        let soil = earth.low(1000.0) * strike(t, 0.001, 0.035);
        let blades = grass.band(3200.0, 0.8) * strike(t, 0.004, 0.05);
        (thump * 0.6 + soil * 0.8 + blades * 0.1) * 0.7
    })
}

fn hammer(rate: f32, seed: u32, pitch: f32) -> Voice {
    // A light hammer tap on wood.
    let mut block = Struck::new(BLOCK, &[(0.0, 900.0 * pitch, 1.0)]);
    let mut tick = Hiss::new(seed, rate);
    Voice::new(rate, 0.16, move |t| {
        block.run(t, rate) * 0.24 + tick.band(3200.0, 1.2) * strike(t, 0.0004, 0.005) * 0.22
    })
}

fn puff(rate: f32, seed: u32) -> Voice {
    // "Poof!": a puff of dust, and a sparkle in it.
    let mut dust = Hiss::new(seed, rate);
    let mut body = Thump::new();
    let mut sparkle = Struck::new(
        CHIME,
        &[
            (0.24, 2637.0, 0.3),
            (0.32, 3136.0, 0.25),
            (0.42, 3520.0, 0.22),
        ],
    );
    Voice::new(rate, 1.0, move |t| {
        let cloud = dust.low(2600.0 - 2300.0 * ease(t / 0.5)) * swell(t, 0.015, 0.08, 0.5);
        let whump = body.run(t, 110.0, 60.0, rate) * strike(t, 0.006, 0.1);
        (cloud * 0.8 + whump * 0.3 + sparkle.run(t, rate) * 0.18) * 0.8
    })
}

fn grown(rate: f32) -> Voice {
    // Something has grown: a gentle chime, rising.
    let mut chime = Struck::new(
        CHIME,
        &[
            (0.0, 784.0, 0.8),
            (0.15, 1046.5, 0.75),
            (0.3, 1174.7, 0.7),
            (0.45, 1568.0, 0.8),
        ],
    );
    Voice::new(rate, 1.9, move |t| chime.run(t, rate) * 0.26)
}

fn click(rate: f32, seed: u32) -> Voice {
    // A quiet click.
    let mut tick = Hiss::new(seed, rate);
    let mut ping = Phase::default();
    Voice::new(rate, 0.05, move |t| {
        tick.band(2600.0, 2.0) * strike(t, 0.0003, 0.004) * 0.35
            + sine(ping.tick(1600.0, rate)) * strike(t, 0.0005, 0.008) * 0.08
    })
}

fn rummage(rate: f32, seed: u32) -> Voice {
    // Leaves turned over.
    let mut leaves = Hiss::new(seed, rate);
    let mut crinkle = Noise::new(seed ^ 0x5a);
    let mut held = 0.0f32;
    let mut until = 0.0f32;
    Voice::new(rate, 0.62, move |t| {
        // The rustle catches and lets go in little crinkles.
        if t >= until {
            held = crinkle.unit().powi(2);
            until = t + 0.012 + 0.03 * crinkle.unit();
        }
        leaves.band(2600.0, 0.7) * (0.25 + held) * swell(t, 0.05, 0.38, 0.2) * 0.55
    })
}

fn cast(rate: f32, seed: u32) -> Voice {
    // The line whipped out over the pool, and the float plopping down on the water.
    let mut air = Hiss::new(seed, rate);
    let mut water = Hiss::new(seed ^ 0x21, rate);
    let mut plop = Phase::default();
    Voice::new(rate, 0.95, move |t| {
        let whip = air.band(600.0 + 2200.0 * ease(t / 0.22), 1.1) * swell(t, 0.06, 0.16, 0.1);
        let down = t - 0.55;
        let hz = 260.0 + 420.0 * (-down.max(0.0) / 0.02).exp();
        let drop = sine(plop.tick(hz, rate)) * strike(down, 0.002, 0.05);
        let splashlet = water.band(1800.0, 1.0) * strike(down, 0.002, 0.04);
        whip * 0.4 + drop * 0.35 + splashlet * 0.3
    })
}

fn flutter(rate: f32, seed: u32) -> Voice {
    // A bug's wings, whirring up and away.
    let mut wings = Hiss::new(seed, rate);
    let mut beat = Phase::default();
    Voice::new(rate, 0.5, move |t| {
        let beating = sine(beat.tick(24.0 + 10.0 * t, rate)).abs();
        wings.band(700.0 + 500.0 * ease(t / 0.4), 1.4) * beating * swell(t, 0.04, 0.3, 0.15) * 0.6
    })
}

fn forage(rate: f32, seed: u32) -> Voice {
    // A paw into the hedgerow: a rustle and the snap of a twig or two.
    let mut leaves = Hiss::new(seed, rate);
    let mut twigs = Hiss::new(seed ^ 0x66, rate);
    Voice::new(rate, 0.46, move |t| {
        let rustle = leaves.band(2100.0, 0.7) * swell(t, 0.04, 0.25, 0.12);
        let snaps = strike(t - 0.1, 0.0004, 0.006) + 0.7 * strike(t - 0.24, 0.0004, 0.006);
        rustle * 0.4 + twigs.band(4200.0, 1.5) * snaps * 0.6
    })
}

fn find(rate: f32) -> Voice {
    // Something turned up: a run of bells, rising.
    let mut bells = Struck::new(
        CHIME,
        &[
            (0.0, 1046.5, 0.7),
            (0.07, 1318.5, 0.75),
            (0.14, 1568.0, 0.8),
            (0.21, 2093.0, 0.9),
        ],
    );
    let mut shimmer = Struck::new(CHIME, &[(0.26, 4186.0, 0.15), (0.31, 5274.0, 0.12)]);
    Voice::new(rate, 1.1, move |t| {
        (bells.run(t, rate) + shimmer.run(t, rate)) * 0.2
    })
}

fn splash(rate: f32, seed: u32) -> Voice {
    // Water broken: a splash, and droplets falling back.
    let mut water = Hiss::new(seed, rate);
    let mut dice = Dice::new(u64::from(seed));
    let drops: Vec<f32> = (0..4).map(|_| dice.range(0.1, 0.4)).collect();
    let mut drip = Phase::default();
    Voice::new(rate, 0.7, move |t| {
        let sweep = 3000.0 - 2200.0 * ease(t / 0.25);
        let splashing = water.band(sweep, 0.8) * strike(t, 0.003, 0.11);
        let mut droplets = 0.0;
        for at in &drops {
            let since = t - at;
            if (0.0..0.03).contains(&since) {
                let hz = 1100.0 + 1300.0 * since / 0.03;
                droplets += sine(drip.tick(hz, rate)) * strike(since, 0.001, 0.008);
            }
        }
        splashing * 0.6 + droplets * 0.14
    })
}

fn swish(rate: f32, seed: u32, pitch: f32) -> Voice {
    // The net swung through the air.
    let mut air = Hiss::new(seed, rate);
    Voice::new(rate, 0.34, move |t| {
        let up = ease(t / 0.12);
        let down = ease((t - 0.12) / 0.18);
        let hz = (500.0 + 1700.0 * up - 1300.0 * down) * pitch;
        air.band(hz, 1.0) * swell(t, 0.08, 0.12, 0.12) * 0.55
    })
}

fn pick(rate: f32, seed: u32, pitch: f32) -> Voice {
    // Something ripe coming away from its stalk: a snap and a pop.
    let mut snap = Hiss::new(seed, rate);
    let mut pop = Phase::default();
    Voice::new(rate, 0.14, move |t| {
        let hz = (520.0 + 300.0 * (-t / 0.01).exp()) * pitch;
        snap.band(3600.0, 1.5) * strike(t, 0.0004, 0.006) * 0.55
            + sine(pop.tick(hz, rate)) * strike(t, 0.001, 0.03) * 0.3
    })
}

fn treasure(rate: f32, seed: u32) -> Voice {
    // Two scrapes of digging, and then something that gleams.
    let mut earth = Hiss::new(seed, rate);
    let mut grit = Noise::new(seed ^ 0x42);
    let mut gleam = Struck::new(
        BELL,
        &[
            (0.62, 1046.5, 0.6),
            (0.68, 1318.5, 0.6),
            (0.74, 1568.0, 0.65),
            (0.84, 2093.0, 0.8),
        ],
    );
    Voice::new(rate, 2.1, move |t| {
        let scrapes = swell(t, 0.04, 0.16, 0.06) + swell(t - 0.28, 0.04, 0.16, 0.06);
        let digging = earth.band(950.0, 0.8) * (0.4 + 0.6 * grit.unit()) * scrapes;
        digging * 0.5 + gleam.run(t, rate) * 0.16
    })
}

fn sting(rate: f32, seed: u32) -> Voice {
    // A boom from below, and a minor chord swelling up out of it: something is here.
    let mut body = Thump::new();
    let mut dust = Hiss::new(seed, rate);
    let chord = [50.0, 53.0, 57.0, 62.0].map(note_hz);
    let mut phases = [Phase::default(); 4];
    let mut tremble = Phase::default();
    Voice::new(rate, 2.6, move |t| {
        let boom = body.run(t, 70.0, 44.0, rate) * strike(t, 0.004, 0.55) * 0.6
            + dust.low(220.0) * strike(t, 0.002, 0.07) * 0.6;
        let shiver = 1.0 + 0.15 * sine(tremble.tick(6.5, rate));
        let mut brass = 0.0;
        for (phase, hz) in phases.iter_mut().zip(chord) {
            let turn = phase.tick(hz, rate);
            for harmonic in 1..=6 {
                brass += sine(turn * harmonic as f32) / harmonic as f32;
            }
        }
        boom + brass * swell(t - 0.1, 0.35, 1.3, 1.0) * shiver * 0.07
    })
}

fn bested(rate: f32) -> Voice {
    // It is over: a bright fanfare climbing to a held major chord.
    let fanfare = [(0.0, 74.0), (0.12, 78.0), (0.24, 81.0), (0.38, 86.0)];
    let mut phases = [Phase::default(); 4];
    let mut glint = Struck::new(BELL, &[(0.38, 2349.3, 0.5)]);
    Voice::new(rate, 2.3, move |t| {
        let mut horns = 0.0;
        for ((at, note), phase) in fanfare.iter().zip(phases.iter_mut()) {
            let since = t - at;
            if since < 0.0 {
                continue;
            }
            let turn = phase.tick(note_hz(*note), rate);
            let bright = 0.5 + 0.5 * (-since / 0.3).exp();
            let mut tone = 0.0;
            for harmonic in 1..=5 {
                tone += sine(turn * harmonic as f32) * bright.powi(harmonic - 1) / harmonic as f32;
            }
            horns += tone * swell(since, 0.03, 1.5 - at, 0.5);
        }
        horns * 0.12 + glint.run(t, rate) * 0.12
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synth::RATE;

    fn heard(cue: Cue, variation: u32) -> Vec<f32> {
        cue.voice(variation, RATE).collect()
    }

    #[test]
    fn every_cue_has_a_sound_of_its_own() {
        let mut names = std::collections::BTreeSet::new();
        for cue in Cue::ALL {
            let samples = heard(cue, 0);
            assert!(!samples.is_empty(), "{cue:?} is silent");
            assert!(
                samples.len() < (RATE * 3.0) as usize,
                "{cue:?} goes on too long"
            );
            let loudest = samples.iter().fold(0.0f32, |most, x| most.max(x.abs()));
            assert!(loudest > 0.04, "{cue:?} can barely be heard: {loudest}");
            assert!(names.insert(cue.name()), "{cue:?} shares a name");
            assert_eq!(Cue::ALL[cue.index()], cue);
        }
    }

    #[test]
    fn every_cue_stays_within_full_scale_and_starts_and_ends_in_silence() {
        for cue in Cue::ALL {
            for variation in 0..4 {
                let samples = heard(cue, variation);
                for sample in &samples {
                    assert!((-1.0..=1.0).contains(sample), "{cue:?} reached {sample}");
                }
                let (first, last) = (samples[0], samples[samples.len() - 1]);
                assert!(first.abs() < 0.05, "{cue:?} starts with a click: {first}");
                assert!(last.abs() < 0.01, "{cue:?} ends with a click: {last}");
            }
        }
    }

    #[test]
    fn a_cue_sounds_the_same_every_time_it_is_made() {
        for cue in Cue::ALL {
            assert_eq!(heard(cue, 3), heard(cue, 3), "{cue:?} changed");
        }
        // A run of chuffs is a little different each time, not one recording again and again.
        assert_ne!(heard(Cue::Chuff, 0), heard(Cue::Chuff, 1));
    }

    #[test]
    fn a_softer_cue_is_the_same_sound_quieter() {
        let full = heard(Cue::Chuff, 2);
        let soft: Vec<f32> = Cue::Chuff.voice(2, RATE).louder(0.5).collect();
        for (loud, quiet) in full.iter().zip(&soft) {
            assert!((loud * 0.5 - quiet).abs() < 1e-6);
        }
    }
}
