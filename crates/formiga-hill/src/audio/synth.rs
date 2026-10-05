//! The synthesiser every sound at the Hill is made with: oscillators, envelopes, noise, a few
//! simple filters and a small room to play in. Nothing is recorded or downloaded, so there is
//! nothing to license; and nothing here reads a clock or a random device, so a sound is made the
//! same way every time it is asked for, which is what lets the tests listen to it.

use std::f32::consts::{PI, TAU};
use std::sync::OnceLock;

/// The rate renders and tests are made at. A device plays at its own rate, and every sound is
/// made for whichever rate it is asked for.
pub const RATE: f32 = 44_100.0;

const TABLE_SIZE: usize = 4096;

/// One turn of a sine, and its first point again at the end, so a lookup never wraps.
fn table() -> &'static [f32] {
    static TABLE: OnceLock<Vec<f32>> = OnceLock::new();
    TABLE.get_or_init(|| {
        (0..=TABLE_SIZE)
            .map(|index| {
                (f64::from(index as u32) / TABLE_SIZE as f64 * std::f64::consts::TAU).sin() as f32
            })
            .collect()
    })
}

/// The sine of a phase measured in turns, from a table: cheap enough for dozens of voices at
/// once on the audio thread.
pub fn sine(turns: f32) -> f32 {
    let at = (turns - turns.floor()) * TABLE_SIZE as f32;
    let index = (at as usize).min(TABLE_SIZE - 1);
    let between = at - index as f32;
    let table = table();
    table[index] + (table[index + 1] - table[index]) * between
}

/// The frequency of a MIDI note: 69 is the A at 440 Hz, and each step is a semitone.
pub fn hz(note: f32) -> f32 {
    440.0 * 2f32.powf((note - 69.0) / 12.0)
}

/// A phase going round, in turns.
#[derive(Clone, Copy, Debug, Default)]
pub struct Phase(f32);

impl Phase {
    /// Starting partway round, so a crowd of oscillators does not all start together.
    pub fn at(turns: f32) -> Self {
        Self(turns - turns.floor())
    }

    /// Where it is, then on by one sample at `hz`.
    pub fn tick(&mut self, hz: f32, rate: f32) -> f32 {
        let now = self.0;
        self.0 += hz / rate;
        if self.0 >= 1.0 {
            self.0 -= self.0.floor();
        }
        now
    }
}

/// White noise from a seed: the same hiss every time for the same seed.
#[derive(Clone, Debug)]
pub struct Noise(u32);

impl Noise {
    pub fn new(seed: u32) -> Self {
        // Xorshift never leaves zero, so a seed is never allowed to put it there.
        let mixed = seed.wrapping_mul(0x9e37_79b9) ^ 0x6d2b_79f5;
        Self(if mixed == 0 { 1 } else { mixed })
    }

    /// The next sample, from -1 up to 1.
    pub fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / 8_388_608.0 - 1.0
    }

    /// A number from 0 up to 1, for choosing.
    pub fn unit(&mut self) -> f32 {
        (self.next() + 1.0) * 0.5
    }
}

/// A one-pole low-pass: the gentlest smoothing, for taking the edge off noise or a change.
#[derive(Clone, Debug)]
pub struct Smooth {
    coefficient: f32,
    held: f32,
}

impl Smooth {
    pub fn new(hz: f32, rate: f32) -> Self {
        Self {
            coefficient: 1.0 - (-TAU * hz / rate).exp(),
            held: 0.0,
        }
    }

    /// The same, already settled at `value`.
    pub fn starting(mut self, value: f32) -> Self {
        self.held = value;
        self
    }

    pub fn run(&mut self, input: f32) -> f32 {
        self.held += self.coefficient * (input - self.held);
        self.held
    }
}

/// Takes out any steady offset below hearing, so every sound settles back to silence at zero.
#[derive(Clone, Debug)]
pub struct DcBlock {
    pole: f32,
    last_in: f32,
    last_out: f32,
}

impl DcBlock {
    pub fn new(rate: f32) -> Self {
        Self {
            pole: (-TAU * 16.0 / rate).exp(),
            last_in: 0.0,
            last_out: 0.0,
        }
    }

    pub fn run(&mut self, input: f32) -> f32 {
        let out = input - self.last_in + self.pole * self.last_out;
        self.last_in = input;
        self.last_out = out;
        out
    }
}

/// What a state-variable filter gives at once.
#[derive(Clone, Copy, Debug)]
pub struct Bands {
    pub low: f32,
    /// Peaks at the input's own level, whatever the resonance.
    pub band: f32,
}

/// A state-variable filter, in the form (Zavalishin's) that stays stable however fast its
/// frequency is swept, which most of the effects do: a whistle's breath, a splash, a puff.
#[derive(Clone, Debug)]
pub struct Svf {
    rate: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1: f32,
    ic2: f32,
}

impl Svf {
    pub fn new(hz: f32, q: f32, rate: f32) -> Self {
        let mut filter = Self {
            rate,
            k: 1.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            ic1: 0.0,
            ic2: 0.0,
        };
        filter.tune(hz, q);
        filter
    }

    /// Moves the filter to `hz`, with resonance `q` (0.5 is gentle, 8 rings), keeping its state.
    pub fn tune(&mut self, hz: f32, q: f32) {
        let hz = hz.clamp(10.0, self.rate * 0.45);
        let g = (PI * hz / self.rate).tan();
        self.k = 1.0 / q.max(0.1);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    pub fn run(&mut self, input: f32) -> Bands {
        let v3 = input - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        Bands {
            low: v2,
            band: self.k * v1,
        }
    }
}

/// Smoothly from 0 to 1 as `x` goes from 0 to 1.
pub fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// A struck sound's loudness `t` seconds in: up over `attack`, then dying away, by a factor of e
/// every `decay` seconds.
pub fn strike(t: f32, attack: f32, decay: f32) -> f32 {
    if t < 0.0 {
        0.0
    } else if t < attack {
        t / attack
    } else {
        (-(t - attack) / decay).exp()
    }
}

/// A held sound's loudness `t` seconds in: up over `attack`, held until `held` seconds are up,
/// then down over `release`, smooth at each end.
pub fn swell(t: f32, attack: f32, held: f32, release: f32) -> f32 {
    if t < 0.0 {
        0.0
    } else if t < held {
        ease(t / attack.max(1e-4))
    } else {
        ease(held / attack.max(1e-4)) * (1.0 - ease((t - held) / release.max(1e-4)))
    }
}

/// A gentle limit on the loudest moments: untouched below 0.7, then bending smoothly towards a
/// ceiling a little under full scale, so nothing that reaches the speaker can clip.
pub fn soften(sample: f32) -> f32 {
    const KNEE: f32 = 0.7;
    const CEILING: f32 = 0.98;
    let size = sample.abs();
    if size <= KNEE {
        sample
    } else {
        let room = CEILING - KNEE;
        (KNEE + room * ((size - KNEE) / room).tanh()).copysign(sample)
    }
}

/// The shape of a room for music to sound in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    /// How long it rings, from 0 (a cupboard) towards 1 (a cathedral).
    pub size: f32,
    /// How much the walls soften the high notes in the ringing, from 0 to 1.
    pub damping: f32,
    /// How much of the ringing is heard, beside the sound itself.
    pub wet: f32,
}

/// Freeverb's eight combs, in samples at 44.1 kHz; the room uses the first six. Not multiples of
/// one another, so the ringing never builds into a note of its own.
const COMBS: [usize; 6] = [1116, 1188, 1277, 1356, 1422, 1491];
const PASSES: [usize; 3] = [556, 441, 341];

#[derive(Clone, Debug)]
struct Comb {
    line: Vec<f32>,
    at: usize,
    kept: f32,
}

#[derive(Clone, Debug)]
struct Pass {
    line: Vec<f32>,
    at: usize,
}

/// A small room, after Freeverb, in mono: a place's music sits in a room of its own shape.
#[derive(Clone, Debug)]
pub struct Room {
    shape: Shape,
    combs: Vec<Comb>,
    passes: Vec<Pass>,
}

impl Room {
    pub fn new(shape: Shape, rate: f32) -> Self {
        let scale = |samples: usize| ((samples as f32 * rate / RATE) as usize).max(1);
        Self {
            shape,
            combs: COMBS
                .iter()
                .map(|&samples| Comb {
                    line: vec![0.0; scale(samples)],
                    at: 0,
                    kept: 0.0,
                })
                .collect(),
            passes: PASSES
                .iter()
                .map(|&samples| Pass {
                    line: vec![0.0; scale(samples)],
                    at: 0,
                })
                .collect(),
        }
    }

    /// The sound and the room's ringing of it.
    pub fn run(&mut self, input: f32) -> f32 {
        let feedback = 0.7 + 0.28 * self.shape.size.clamp(0.0, 1.0);
        let damping = self.shape.damping.clamp(0.0, 1.0) * 0.4;
        let fed = input * 0.06;
        let mut ringing = 0.0;
        for comb in &mut self.combs {
            let out = comb.line[comb.at];
            comb.kept = out * (1.0 - damping) + comb.kept * damping;
            comb.line[comb.at] = fed + comb.kept * feedback;
            comb.at = (comb.at + 1) % comb.line.len();
            ringing += out;
        }
        for pass in &mut self.passes {
            let held = pass.line[pass.at];
            let out = held - ringing;
            pass.line[pass.at] = ringing + held * 0.5;
            pass.at = (pass.at + 1) % pass.line.len();
            ringing = out;
        }
        input + ringing * self.shape.wet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sine_table_is_a_sine() {
        for step in 0..1000 {
            let turns = step as f32 / 997.0 - 0.3;
            assert!(
                (sine(turns) - (turns * TAU).sin()).abs() < 1e-4,
                "at {turns}"
            );
        }
    }

    #[test]
    fn notes_are_tuned_to_concert_pitch() {
        assert!((hz(69.0) - 440.0).abs() < 1e-3);
        assert!((hz(81.0) - 880.0).abs() < 1e-2);
        assert!((hz(60.0) - 261.63).abs() < 1e-2);
    }

    #[test]
    fn noise_is_the_same_for_the_same_seed_and_stays_in_range() {
        let (mut a, mut b) = (Noise::new(7), Noise::new(7));
        let heard: Vec<f32> = (0..5000).map(|_| a.next()).collect();
        assert!(heard.iter().all(|sample| (-1.0..1.0).contains(sample)));
        assert!(
            heard
                .iter()
                .zip((0..5000).map(|_| b.next()))
                .all(|(x, y)| *x == y)
        );
        assert_ne!(Noise::new(7).next(), Noise::new(8).next());
        let mean = heard.iter().sum::<f32>() / heard.len() as f32;
        assert!(mean.abs() < 0.05, "noise leans {mean}");
        assert_ne!(Noise::new(0).next(), 0.0, "a zero seed still hisses");
    }

    #[test]
    fn a_band_pass_keeps_its_note_and_loses_the_rest() {
        let level = |hz: f32| {
            let mut filter = Svf::new(1000.0, 4.0, RATE);
            let mut phase = Phase::default();
            let mut loudest = 0.0f32;
            for sample in 0..8000 {
                let out = filter.run(sine(phase.tick(hz, RATE))).band;
                if sample > 4000 {
                    loudest = loudest.max(out.abs());
                }
            }
            loudest
        };
        assert!((level(1000.0) - 1.0).abs() < 0.05, "{}", level(1000.0));
        assert!(level(200.0) < 0.1);
        assert!(level(6000.0) < 0.1);
    }

    #[test]
    fn nothing_is_left_offset_once_blocked() {
        let mut block = DcBlock::new(RATE);
        let mut last = 1.0;
        for _ in 0..(RATE as usize) {
            last = block.run(0.5);
        }
        assert!(last.abs() < 1e-3, "{last}");
    }

    #[test]
    fn envelopes_rise_hold_and_fall_to_nothing() {
        assert_eq!(strike(-0.1, 0.01, 0.2), 0.0);
        assert!((strike(0.01, 0.01, 0.2) - 1.0).abs() < 1e-6);
        assert!(strike(2.0, 0.01, 0.2) < 1e-4);
        assert!((swell(0.5, 0.1, 1.0, 0.3) - 1.0).abs() < 1e-6);
        assert_eq!(swell(1.3, 0.1, 1.0, 0.3), 0.0);
        assert!(swell(1.15, 0.1, 1.0, 0.3) > 0.0);
    }

    #[test]
    fn softening_never_lets_a_sample_past_full_scale() {
        for step in -400..=400 {
            let sample = step as f32 / 40.0;
            let soft = soften(sample);
            assert!(soft.abs() < 1.0);
            assert_eq!(soft.signum(), sample.signum());
            if sample.abs() <= 0.7 {
                assert_eq!(soft, sample, "quiet sounds are left alone");
            }
        }
    }

    #[test]
    fn a_room_rings_on_after_the_sound_and_then_falls_quiet() {
        let shape = Shape {
            size: 0.6,
            damping: 0.4,
            wet: 0.5,
        };
        let mut room = Room::new(shape, RATE);
        room.run(1.0);
        let ringing: Vec<f32> = (0..(RATE as usize * 6)).map(|_| room.run(0.0)).collect();
        let early = ringing[..4410]
            .iter()
            .fold(0.0f32, |most, x| most.max(x.abs()));
        let late = ringing[ringing.len() - 4410..]
            .iter()
            .fold(0.0f32, |most, x| most.max(x.abs()));
        assert!(early > 1e-3, "the room does not ring");
        assert!(late < early * 1e-3, "the room rings on for ever");
    }
}
