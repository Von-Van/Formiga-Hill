//! The Hill's little band: each instrument a handful of sine partials with an envelope, and a
//! breath of noise where one wants it. Cheap enough that a dozen notes ring at once on the audio
//! thread, and each note is a function of nothing but its own time since it was struck, so a
//! note sounds the same on every pass of a loop.

use crate::audio::synth::{Noise, Phase, Smooth, ease, sine, strike, swell};

/// Who plays a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instrument {
    /// A nylon string, plucked: the station's and the green's guitar, and the Hilltop's harp.
    Pluck,
    /// A music box's tine, for the Fairground's waltz.
    MusicBox,
    /// A felt-soft electric piano, for the Clubhouse fire.
    Keys,
    /// A soft round chime.
    Chime,
    /// A kalimba's tine, for the Woods.
    Kalimba,
    /// A held chord voice, breathing slowly.
    Pad,
    /// The same, with air in it, for the Woods.
    AiryPad,
    /// A wooden flute.
    Flute,
    /// A mellow horn.
    Horn,
    /// A double bass, plucked.
    Bass,
    /// Bowed strings, trembling.
    Strings,
    /// A kettle drum.
    Timpani,
    /// A log on the fire, popping.
    Crackle,
}

/// The most partials any instrument uses.
const MOST: usize = 12;

#[derive(Clone, Copy, Debug, Default)]
struct Partial {
    phase: Phase,
    /// Its frequency over the note's.
    ratio: f32,
    /// How loud it is now, and what that is multiplied by each sample.
    level: f32,
    decay: f32,
}

/// One note sounding.
#[derive(Clone, Debug)]
pub struct Sounding {
    instrument: Instrument,
    rate: f32,
    hz: f32,
    /// How long the note is held, in seconds, and how loud it is.
    held: f32,
    loud: f32,
    /// Seconds since it was struck, and when it will have died away.
    t: f32,
    end: f32,
    partials: [Partial; MOST],
    count: usize,
    noise: Noise,
    air: Smooth,
    vibrato: Phase,
}

impl Sounding {
    /// A note of `instrument` at `hz`, held `held` seconds, `loud` from 0 to 1. `seed` gives it
    /// the same breath every time.
    pub fn new(
        instrument: Instrument,
        hz: f32,
        held: f32,
        loud: f32,
        seed: u32,
        rate: f32,
    ) -> Self {
        let mut note = Self {
            instrument,
            rate,
            hz,
            held,
            loud,
            t: 0.0,
            end: 0.0,
            partials: [Partial::default(); MOST],
            count: 0,
            noise: Noise::new(seed),
            air: Smooth::new(1200.0, rate),
            vibrato: Phase::at((seed % 97) as f32 / 97.0),
        };
        // How long a struck note rings: low notes longer than high ones.
        let ring = |at_220: f32| at_220 * (220.0 / hz).powf(0.35);
        match instrument {
            Instrument::Pluck => {
                let ring = ring(1.4).clamp(0.5, 2.6);
                for (n, amp) in [1.0, 0.55, 0.3, 0.17, 0.09, 0.05].into_iter().enumerate() {
                    let n = n as f32 + 1.0;
                    note.partial(
                        n * (1.0 + 0.0004 * n * n),
                        amp,
                        ring / (1.0 + 0.8 * (n - 1.0)),
                    );
                }
                note.end = held.min(ring * 4.0) + 0.3;
            }
            Instrument::MusicBox => {
                let ring = ring(1.1).clamp(0.4, 1.6);
                note.partial(1.0, 1.0, ring);
                note.partial(2.0, 0.16, ring * 0.4);
                note.partial(3.0, 0.05, ring * 0.2);
                note.partial(5.4, 0.22, 0.03);
                note.partial(8.93, 0.08, 0.015);
                note.end = ring * 5.0;
            }
            Instrument::Keys => {
                let ring = ring(2.0).clamp(0.8, 3.0);
                note.partial(1.0, 1.0, ring);
                note.partial(2.0, 0.28, ring * 0.45);
                note.partial(3.0, 0.09, ring * 0.25);
                note.partial(4.0, 0.035, ring * 0.15);
                note.partial(7.03, 0.04, 0.1);
                note.end = held.min(ring * 4.0) + 0.4;
            }
            Instrument::Chime => {
                let ring = ring(2.0).clamp(0.6, 2.4);
                note.partial(1.0, 1.0, ring);
                note.partial(2.0, 0.2, ring * 0.45);
                note.partial(3.0, 0.06, ring * 0.25);
                note.partial(4.2, 0.04, 0.08);
                note.end = ring * 5.0;
            }
            Instrument::Kalimba => {
                let ring = ring(1.2).clamp(0.4, 1.6);
                note.partial(1.0, 1.0, ring);
                note.partial(2.0, 0.08, ring * 0.3);
                note.partial(5.95, 0.2, 0.04);
                note.end = ring * 5.0;
            }
            Instrument::Pad | Instrument::AiryPad => {
                for (detune, phase) in [(0.996, 0.0), (1.0, 0.33), (1.004, 0.71)] {
                    for (n, amp) in [(1.0, 0.34), (2.0, 0.075), (3.0, 0.022)] {
                        note.partial(n * detune, amp, f32::INFINITY);
                        note.partials[note.count - 1].phase = Phase::at(phase * n);
                    }
                }
                note.end = held + 1.6;
            }
            Instrument::Flute => {
                note.partial(1.0, 1.0, f32::INFINITY);
                note.partial(2.0, 0.15, f32::INFINITY);
                note.partial(3.0, 0.045, f32::INFINITY);
                note.end = held + 0.2;
            }
            Instrument::Horn => {
                for (n, amp) in [1.0, 0.42, 0.2, 0.09, 0.04].into_iter().enumerate() {
                    note.partial(n as f32 + 1.0, amp, f32::INFINITY);
                }
                note.end = held + 0.35;
            }
            Instrument::Bass => {
                let ring = ring(1.6).clamp(0.6, 2.4);
                note.partial(1.0, 1.0, ring);
                note.partial(2.0, 0.32, ring * 0.45);
                note.partial(3.0, 0.1, ring * 0.25);
                note.end = held.min(ring * 4.0) + 0.15;
            }
            Instrument::Strings => {
                for detune in [0.997, 1.003] {
                    for n in 1..=6 {
                        let n = n as f32;
                        note.partial(n * detune, 0.5 / n, f32::INFINITY);
                    }
                }
                note.end = held + 0.6;
            }
            Instrument::Timpani => {
                note.partial(1.0, 1.0, 0.9);
                note.partial(1.5, 0.45, 0.5);
                note.partial(1.98, 0.28, 0.32);
                note.partial(2.44, 0.16, 0.2);
                note.end = 4.5;
            }
            Instrument::Crackle => note.end = 0.03,
        }
        note
    }

    fn partial(&mut self, ratio: f32, amp: f32, decay_secs: f32) {
        if self.count < MOST {
            self.partials[self.count] = Partial {
                phase: Phase::default(),
                ratio,
                level: amp,
                decay: (-1.0 / (decay_secs * self.rate)).exp(),
            };
            self.count += 1;
        }
    }

    /// Whether it has died away.
    pub fn done(&self) -> bool {
        self.t >= self.end
    }

    /// The next sample.
    pub fn next(&mut self) -> f32 {
        let t = self.t;
        self.t += 1.0 / self.rate;
        let held = self.held;
        // Pitch: a breath of vibrato in what is blown or bowed, a drum's slap settling.
        let pitch = match self.instrument {
            Instrument::Flute | Instrument::Horn | Instrument::Strings => {
                let depth = match self.instrument {
                    Instrument::Flute => 0.005,
                    Instrument::Horn => 0.0025,
                    _ => 0.0035,
                };
                let hz = if self.instrument == Instrument::Strings {
                    5.5
                } else {
                    5.0
                };
                1.0 + depth * ease((t - 0.2) / 0.3) * sine(self.vibrato.tick(hz, self.rate))
            }
            Instrument::Pad | Instrument::AiryPad => {
                1.0 + 0.0012 * sine(self.vibrato.tick(0.23, self.rate))
            }
            Instrument::Timpani => 1.0 + 0.1 * (-t / 0.04).exp(),
            _ => 1.0,
        };
        // The horn opens up as it is blown: its upper partials come in after the note.
        let bright = if self.instrument == Instrument::Horn {
            0.35 + 0.65 * ease(t / 0.18)
        } else {
            1.0
        };
        let mut tone = 0.0;
        for (index, partial) in self.partials[..self.count].iter_mut().enumerate() {
            let turn = partial
                .phase
                .tick(self.hz * partial.ratio * pitch, self.rate);
            let level = if index > 0 {
                partial.level * bright
            } else {
                partial.level
            };
            tone += level * sine(turn);
            partial.level *= partial.decay;
        }
        let shape = match self.instrument {
            Instrument::Pluck | Instrument::Keys | Instrument::Bass => {
                let attack = if self.instrument == Instrument::Bass {
                    0.008
                } else {
                    0.003
                };
                let release = if self.instrument == Instrument::Keys {
                    0.3
                } else {
                    0.12
                };
                ease(t / attack) * release_after(t, held, release)
            }
            Instrument::MusicBox | Instrument::Chime | Instrument::Kalimba => ease(t / 0.0015),
            Instrument::Pad | Instrument::AiryPad => swell(t, 0.9, held, 1.5),
            Instrument::Flute => swell(t, 0.07, held, 0.16),
            Instrument::Horn => swell(t, 0.12, held, 0.3),
            // Bowed fast, trembling: the drama of it.
            Instrument::Strings => swell(t, 0.3, held, 0.55) * (0.78 + 0.22 * sine(t * 6.5)),
            Instrument::Timpani => ease(t / 0.002),
            Instrument::Crackle => 0.0,
        };
        let mut out = tone * shape;
        // Breath and texture.
        match self.instrument {
            Instrument::Flute => {
                let breath = self.noise.next();
                let air = breath - self.air.run(breath);
                out += air * (0.035 * shape + 0.12 * strike(t, 0.004, 0.03));
            }
            Instrument::AiryPad => {
                let air = self.air.run(self.noise.next());
                out += air * 0.3 * shape;
            }
            Instrument::Pluck => {
                // The fingertip on the string.
                let touch = self.noise.next();
                out += (touch - self.air.run(touch)) * 0.25 * strike(t, 0.0005, 0.006);
            }
            Instrument::Timpani => {
                out += self.air.run(self.noise.next()) * 1.2 * strike(t, 0.001, 0.025);
            }
            Instrument::Crackle => {
                let pop = self.noise.next();
                out = (pop - self.air.run(pop)) * strike(t, 0.0004, 0.004);
            }
            _ => {}
        }
        // A struck note that rings past its end is let go of smoothly.
        let tail = ((self.end - t) / 0.05).clamp(0.0, 1.0);
        out * self.loud * tail
    }
}

/// Full until `held`, then dying away by a factor of e every `release` seconds.
fn release_after(t: f32, held: f32, release: f32) -> f32 {
    if t < held {
        1.0
    } else {
        (-(t - held) / release).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synth::RATE;

    const EVERY: [Instrument; 13] = [
        Instrument::Pluck,
        Instrument::MusicBox,
        Instrument::Keys,
        Instrument::Chime,
        Instrument::Kalimba,
        Instrument::Pad,
        Instrument::AiryPad,
        Instrument::Flute,
        Instrument::Horn,
        Instrument::Bass,
        Instrument::Strings,
        Instrument::Timpani,
        Instrument::Crackle,
    ];

    #[test]
    fn every_instrument_sounds_and_dies_away_by_its_end() {
        for instrument in EVERY {
            for hz in [55.0, 220.0, 880.0] {
                let mut note = Sounding::new(instrument, hz, 1.0, 1.0, 3, RATE);
                let mut loudest = 0.0f32;
                let mut last = 0.0f32;
                let mut count = 0;
                while !note.done() {
                    last = note.next();
                    loudest = loudest.max(last.abs());
                    count += 1;
                    assert!(count < (RATE * 12.0) as usize, "{instrument:?} never ends");
                }
                assert!(loudest > 0.05, "{instrument:?} at {hz} is silent");
                assert!(
                    loudest < 2.5,
                    "{instrument:?} at {hz} is far too loud: {loudest}"
                );
                assert!(last.abs() < 0.01, "{instrument:?} at {hz} stops dead");
            }
        }
    }
}
