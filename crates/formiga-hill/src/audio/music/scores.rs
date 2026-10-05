//! The placeholder pieces, written out note by note: a pastoral air for the station and the green,
//! a fireside piece for the Clubhouse, a music-box waltz for the Fairground, something sparse and
//! airy for the Woods, and something open for the Hilltop; each slower, softer and sparser by
//! night, when the lamps are lit. Pitches are MIDI notes (60 is middle C) and times are beats.
//!
//! Each is written to be heard for a long while without wearing: gentle in register, quiet in its
//! accents, and long enough that its loop is not quickly learnt.

use super::{Instrument, Music, Note, Score, Track};
use crate::audio::synth::Shape;
use crate::dice::Dice;

/// Writes the score of a piece.
pub fn write(music: Music) -> Score {
    match music.track {
        Track::Pastoral => pastoral(music.night),
        Track::Fireside => fireside(music.night),
        Track::Waltz => waltz(music.night),
        Track::Woods => woods(music.night),
        Track::Hilltop => hilltop(music.night),
        Track::Secret => secret(),
    }
}

/// A score being written: notes put down part by part.
struct Writer {
    beats: f32,
    notes: Vec<Note>,
    dice: Dice,
}

impl Writer {
    fn new(beats: f32, seed: u64) -> Self {
        Self {
            beats,
            notes: Vec::new(),
            dice: Dice::new(seed),
        }
    }

    /// A note exactly where it is written, inside the loop.
    fn note(&mut self, instrument: Instrument, at: f32, held: f32, pitch: f32, loud: f32) {
        self.notes.push(Note {
            at: at.clamp(0.0, self.beats - 0.01),
            held,
            pitch,
            loud,
            instrument,
        });
    }

    /// A note as a player would play it: a hair early or late, a touch louder or softer.
    fn played(&mut self, instrument: Instrument, at: f32, held: f32, pitch: f32, loud: f32) {
        let at = at + self.dice.range(-0.012, 0.012);
        let loud = loud * self.dice.range(0.88, 1.06);
        self.note(instrument, at, held, pitch, loud);
    }

    fn chord(&mut self, instrument: Instrument, at: f32, held: f32, pitches: &[f32], loud: f32) {
        for &pitch in pitches {
            self.note(instrument, at, held, pitch, loud);
        }
    }

    fn score(self, bpm: f32, room: Shape, level: f32) -> Score {
        Score {
            bpm,
            beats: self.beats,
            notes: self.notes,
            room,
            level,
        }
    }
}

/// The station and the green: a guitar picking through a pastoral round in G, a soft pad under
/// it, and a wooden flute that joins in for the second half of each phrase.
fn pastoral(night: bool) -> Score {
    // Each bar's bass note and chord.
    const BARS: [(f32, [f32; 3]); 16] = [
        (43.0, [55.0, 59.0, 62.0]), // G
        (42.0, [54.0, 57.0, 62.0]), // D over F sharp
        (40.0, [55.0, 59.0, 64.0]), // E minor
        (48.0, [55.0, 60.0, 64.0]), // C
        (43.0, [55.0, 59.0, 62.0]), // G
        (45.0, [57.0, 60.0, 64.0]), // A minor
        (50.0, [54.0, 57.0, 62.0]), // D
        (50.0, [54.0, 57.0, 60.0]), // D seventh
        (43.0, [55.0, 59.0, 62.0]), // G
        (47.0, [54.0, 59.0, 62.0]), // B minor
        (48.0, [55.0, 60.0, 64.0]), // C
        (43.0, [55.0, 59.0, 62.0]), // G
        (40.0, [55.0, 59.0, 64.0]), // E minor
        (45.0, [57.0, 60.0, 64.0]), // A minor
        (48.0, [55.0, 60.0, 64.0]), // C
        (50.0, [54.0, 57.0, 62.0]), // D, and round again to G
    ];
    // The flute: bar, beat, beats held, pitch.
    const TUNE: [(f32, f32, f32, f32); 23] = [
        (4.0, 0.0, 1.5, 74.0),
        (4.0, 1.5, 0.5, 76.0),
        (4.0, 2.0, 2.0, 74.0),
        (5.0, 0.0, 1.5, 72.0),
        (5.0, 1.5, 0.5, 71.0),
        (5.0, 2.0, 2.0, 69.0),
        (6.0, 0.0, 1.0, 69.0),
        (6.0, 1.0, 1.0, 71.0),
        (6.0, 2.0, 2.0, 74.0),
        (7.0, 0.0, 2.0, 72.0),
        (7.0, 2.0, 1.0, 69.0),
        (7.0, 3.0, 1.0, 66.0),
        (12.0, 0.0, 1.5, 71.0),
        (12.0, 1.5, 0.5, 74.0),
        (12.0, 2.0, 2.0, 76.0),
        (13.0, 0.0, 1.5, 76.0),
        (13.0, 1.5, 0.5, 74.0),
        (13.0, 2.0, 1.0, 72.0),
        (13.0, 3.0, 1.0, 69.0),
        (14.0, 0.0, 2.0, 67.0),
        (14.0, 2.0, 1.0, 69.0),
        (14.0, 3.0, 1.0, 72.0),
        (15.0, 0.0, 3.0, 74.0),
    ];
    // A high chime on the first beat of the bars the flute leaves be.
    const SPARKLE: [(f32, f32); 4] = [(8.0, 83.0), (9.0, 86.0), (10.0, 88.0), (11.0, 86.0)];
    let mut score = Writer::new(64.0, 0x9a57);
    for (bar, &(bass, chord)) in BARS.iter().enumerate() {
        let at = bar as f32 * 4.0;
        score.chord(
            Instrument::Pad,
            at,
            4.0,
            &chord,
            if night { 0.2 } else { 0.24 },
        );
        let [a, b, c] = chord;
        if night {
            for (step, pitch) in [a, c, b, c].into_iter().enumerate() {
                let accent = if step == 0 { 0.36 } else { 0.24 };
                score.played(Instrument::Pluck, at + step as f32, 2.0, pitch, accent);
            }
            score.played(Instrument::Bass, at, 3.5, bass, 0.42);
        } else {
            let pattern = [a, c, b, c, a + 12.0, c, b, c];
            let accents = [1.0, 0.6, 0.75, 0.6, 0.85, 0.6, 0.75, 0.6];
            for (step, (pitch, accent)) in pattern.into_iter().zip(accents).enumerate() {
                score.played(
                    Instrument::Pluck,
                    at + step as f32 * 0.5,
                    1.5,
                    pitch,
                    accent * 0.4,
                );
            }
            score.played(Instrument::Bass, at, 1.8, bass, 0.5);
            score.played(Instrument::Bass, at + 2.0, 1.8, bass + 7.0, 0.32);
        }
    }
    if night {
        // Only the first note of each of the flute's phrases, on a soft chime.
        for &(bar, beat, _, pitch) in TUNE
            .iter()
            .filter(|(_, beat, _, _)| *beat == 0.0)
            .step_by(2)
        {
            score.note(Instrument::Chime, bar * 4.0 + beat, 2.0, pitch + 12.0, 0.16);
        }
    } else {
        for (bar, beat, held, pitch) in TUNE {
            score.played(
                Instrument::Flute,
                bar * 4.0 + beat,
                held * 0.95,
                pitch,
                0.42,
            );
        }
        for (bar, pitch) in SPARKLE {
            score.note(Instrument::Chime, bar * 4.0, 2.0, pitch, 0.2);
        }
    }
    let room = if night {
        Shape {
            size: 0.72,
            damping: 0.6,
            wet: 0.38,
        }
    } else {
        Shape {
            size: 0.55,
            damping: 0.5,
            wet: 0.28,
        }
    };
    score.score(
        if night { 72.0 } else { 88.0 },
        room,
        if night { 0.26 } else { 0.28 },
    )
}

/// The Clubhouse: an electric piano by the fire in F, unhurried, the bass walking slowly under it,
/// and the fire popping now and then.
fn fireside(night: bool) -> Score {
    const BARS: [(f32, [f32; 3]); 16] = [
        (41.0, [57.0, 60.0, 64.0]), // F major seventh
        (45.0, [55.0, 60.0, 64.0]), // A minor seventh
        (38.0, [57.0, 60.0, 65.0]), // D minor seventh
        (46.0, [57.0, 62.0, 65.0]), // B flat major seventh
        (43.0, [58.0, 62.0, 65.0]), // G minor seventh
        (48.0, [58.0, 60.0, 65.0]), // C seventh, suspended
        (45.0, [57.0, 60.0, 64.0]), // F over A
        (48.0, [55.0, 58.0, 64.0]), // C seventh
        (41.0, [57.0, 60.0, 64.0]), // F major seventh
        (38.0, [57.0, 60.0, 65.0]), // D minor seventh
        (43.0, [58.0, 62.0, 65.0]), // G minor seventh
        (48.0, [55.0, 58.0, 64.0]), // C seventh
        (45.0, [55.0, 60.0, 64.0]), // A minor seventh
        (38.0, [57.0, 60.0, 65.0]), // D minor seventh
        (43.0, [58.0, 62.0, 65.0]), // G minor seventh
        (48.0, [58.0, 60.0, 64.0]), // C seventh, and home to F
    ];
    // The tune, high on the piano in the second half: bar, beat, beats held, pitch.
    const TUNE: [(f32, f32, f32, f32); 16] = [
        (8.0, 0.0, 2.0, 72.0),
        (8.0, 2.0, 2.0, 76.0),
        (9.0, 0.0, 3.0, 77.0),
        (9.0, 3.0, 1.0, 76.0),
        (10.0, 0.0, 2.0, 74.0),
        (10.0, 2.0, 2.0, 70.0),
        (11.0, 0.0, 4.0, 72.0),
        (12.0, 0.0, 2.0, 76.0),
        (12.0, 2.0, 2.0, 79.0),
        (13.0, 0.0, 3.0, 77.0),
        (13.0, 3.0, 1.0, 74.0),
        (14.0, 0.0, 2.0, 74.0),
        (14.0, 2.0, 1.0, 72.0),
        (14.0, 3.0, 1.0, 70.0),
        (15.0, 0.0, 2.0, 72.0),
        (15.0, 2.0, 2.0, 70.0),
    ];
    let mut score = Writer::new(64.0, 0xf12e);
    for (bar, &(bass, chord)) in BARS.iter().enumerate() {
        let at = bar as f32 * 4.0;
        // The chord rolled gently from the bottom up.
        for (step, pitch) in chord.into_iter().enumerate() {
            let loud = if night { 0.26 } else { 0.32 };
            score.played(Instrument::Keys, at + step as f32 * 0.07, 3.6, pitch, loud);
        }
        if night {
            score.played(Instrument::Bass, at, 3.6, bass, 0.46);
        } else {
            score.played(Instrument::Keys, at + 2.5, 1.4, chord[1], 0.17);
            score.played(Instrument::Keys, at + 2.5, 1.4, chord[2], 0.17);
            score.played(Instrument::Bass, at, 1.9, bass, 0.5);
            score.played(Instrument::Bass, at + 2.0, 1.9, bass + 7.0, 0.36);
        }
    }
    if !night {
        for (bar, beat, held, pitch) in TUNE {
            score.played(Instrument::Keys, bar * 4.0 + beat, held, pitch, 0.3);
        }
    }
    // The fire, popping where it likes: the same pops every time round.
    let mut fire = Dice::new(0xf1e);
    let mut beat = 0.0;
    while beat < 64.0 {
        score.note(Instrument::Crackle, beat, 0.1, 60.0, fire.range(0.05, 0.16));
        beat += fire.range(0.3, 1.6);
    }
    let room = Shape {
        size: 0.35,
        damping: 0.65,
        wet: if night { 0.26 } else { 0.22 },
    };
    score.score(
        if night { 54.0 } else { 64.0 },
        room,
        if night { 0.32 } else { 0.35 },
    )
}

/// The Fairground: a waltz on a music box, oom-pah-pah, with a tune that goes up and comes back.
fn waltz(night: bool) -> Score {
    // Each bar's chord: C, the dominant seventh (G7) or F.
    const C: (f32, [f32; 2]) = (60.0, [64.0, 67.0]);
    const G7: (f32, [f32; 2]) = (55.0, [65.0, 71.0]);
    const F: (f32, [f32; 2]) = (53.0, [65.0, 69.0]);
    const BARS: [(f32, [f32; 2]); 32] = [
        C, C, G7, G7, G7, G7, C, C, // up the hill
        C, C, F, F, C, G7, C, C, // and down again
        F, F, C, C, G7, G7, C, C, // round the carousel
        C, C, G7, G7, F, G7, C, C, // and home
    ];
    // The tune: bar, beat, beats held, pitch.
    const TUNE: [(f32, f32, f32, f32); 50] = [
        (0.0, 0.0, 2.0, 76.0),
        (0.0, 2.0, 1.0, 79.0),
        (1.0, 0.0, 3.0, 84.0),
        (2.0, 0.0, 2.0, 83.0),
        (2.0, 2.0, 1.0, 81.0),
        (3.0, 0.0, 3.0, 79.0),
        (4.0, 0.0, 2.0, 77.0),
        (4.0, 2.0, 1.0, 79.0),
        (5.0, 0.0, 2.0, 74.0),
        (5.0, 2.0, 1.0, 77.0),
        (6.0, 0.0, 2.0, 76.0),
        (6.0, 2.0, 1.0, 74.0),
        (7.0, 0.0, 3.0, 72.0),
        (8.0, 0.0, 2.0, 76.0),
        (8.0, 2.0, 1.0, 79.0),
        (9.0, 0.0, 2.0, 84.0),
        (9.0, 2.0, 1.0, 88.0),
        (10.0, 0.0, 2.0, 86.0),
        (10.0, 2.0, 1.0, 84.0),
        (11.0, 0.0, 3.0, 81.0),
        (12.0, 0.0, 2.0, 79.0),
        (12.0, 2.0, 1.0, 76.0),
        (13.0, 0.0, 2.0, 77.0),
        (13.0, 2.0, 1.0, 74.0),
        (14.0, 0.0, 3.0, 72.0),
        (15.0, 2.0, 1.0, 67.0),
        (16.0, 0.0, 1.0, 81.0),
        (16.0, 1.0, 1.0, 84.0),
        (16.0, 2.0, 1.0, 81.0),
        (17.0, 0.0, 3.0, 77.0),
        (18.0, 0.0, 1.0, 79.0),
        (18.0, 1.0, 1.0, 84.0),
        (18.0, 2.0, 1.0, 79.0),
        (19.0, 0.0, 3.0, 76.0),
        (20.0, 0.0, 1.0, 77.0),
        (20.0, 1.0, 1.0, 79.0),
        (20.0, 2.0, 1.0, 83.0),
        (21.0, 0.0, 2.0, 86.0),
        (21.0, 2.0, 1.0, 83.0),
        (22.0, 0.0, 3.0, 84.0),
        (23.0, 2.0, 1.0, 79.0),
        (24.0, 0.0, 2.0, 76.0),
        (24.0, 2.0, 1.0, 79.0),
        (25.0, 0.0, 3.0, 84.0),
        (26.0, 0.0, 2.0, 83.0),
        (26.0, 2.0, 1.0, 81.0),
        (27.0, 0.0, 3.0, 79.0),
        (28.0, 0.0, 2.0, 81.0),
        (28.0, 2.0, 1.0, 77.0),
        (29.0, 0.0, 2.0, 74.0),
    ];
    const ENDING: [(f32, f32, f32, f32); 2] = [(29.0, 2.0, 1.0, 71.0), (30.0, 0.0, 3.0, 72.0)];
    let mut score = Writer::new(96.0, 0x3a17);
    for (bar, &(oom, pah)) in BARS.iter().enumerate() {
        let at = bar as f32 * 3.0;
        score.played(Instrument::MusicBox, at, 1.0, oom, 0.3);
        let pahs: &[f32] = if night { &[1.0] } else { &[1.0, 2.0] };
        for &beat in pahs {
            for pitch in pah {
                score.played(Instrument::MusicBox, at + beat, 1.0, pitch, 0.13);
            }
        }
    }
    for (bar, beat, held, pitch) in TUNE.into_iter().chain(ENDING) {
        let loud = if night { 0.45 } else { 0.55 };
        score.played(Instrument::MusicBox, bar * 3.0 + beat, held, pitch, loud);
    }
    let room = Shape {
        size: if night { 0.62 } else { 0.5 },
        damping: 0.3,
        wet: if night { 0.32 } else { 0.24 },
    };
    score.score(
        if night { 112.0 } else { 144.0 },
        room,
        if night { 0.25 } else { 0.31 },
    )
}

/// The Woods: wide, slow chords with air in them, a kalimba picking out notes of the pentatonic
/// here and there like light through leaves, and now and then a bird.
fn woods(night: bool) -> Score {
    // Each four bars' bass note and chord.
    const CHORDS: [(f32, [f32; 4]); 6] = [
        (40.0, [52.0, 59.0, 62.0, 66.0]), // E minor ninth
        (36.0, [48.0, 55.0, 59.0, 64.0]), // C major seventh
        (45.0, [52.0, 57.0, 60.0, 67.0]), // A minor seventh
        (38.0, [50.0, 55.0, 59.0, 66.0]), // G major seventh, over D
        (40.0, [52.0, 59.0, 62.0, 67.0]), // E minor seventh
        (38.0, [50.0, 57.0, 62.0, 64.0]), // D, suspended
    ];
    const SCALE: [f32; 11] = [
        64.0, 67.0, 69.0, 71.0, 74.0, 76.0, 79.0, 81.0, 83.0, 86.0, 88.0,
    ];
    let mut score = Writer::new(96.0, 0x700d);
    for (index, &(bass, chord)) in CHORDS.iter().enumerate() {
        let at = index as f32 * 16.0;
        score.chord(
            Instrument::AiryPad,
            at,
            16.0,
            &chord,
            if night { 0.15 } else { 0.18 },
        );
        score.played(Instrument::Bass, at, 6.0, bass, 0.34);
        if !night {
            score.played(Instrument::Bass, at + 8.0, 6.0, bass, 0.24);
        }
    }
    // The kalimba wanders the scale a step or two at a time, never in a hurry.
    let mut picking = Dice::new(0x6a11);
    let mut place = 5usize;
    let chance = if night { 0.16 } else { 0.32 };
    for beat in 0..96 {
        if !picking.chance(chance) {
            continue;
        }
        let step = picking.range(-2.6, 2.6).round() as isize;
        place = (place as isize + step).clamp(0, SCALE.len() as isize - 1) as usize;
        let offbeat = if picking.chance(0.3) { 0.5 } else { 0.0 };
        let loud = picking.range(0.16, 0.3);
        score.note(
            Instrument::Kalimba,
            beat as f32 + offbeat,
            1.0,
            SCALE[place],
            loud,
        );
    }
    if !night {
        // A bird, three times round the loop.
        for bar in [5.0, 13.0, 21.0] {
            for (beat, held, pitch) in [(0.0, 0.22, 83.0), (0.33, 0.22, 86.0), (0.66, 0.5, 88.0)] {
                score.note(Instrument::Flute, bar * 4.0 + beat, held, pitch, 0.14);
            }
        }
    }
    let room = Shape {
        size: if night { 0.92 } else { 0.88 },
        damping: 0.35,
        wet: if night { 0.5 } else { 0.45 },
    };
    score.score(
        if night { 52.0 } else { 60.0 },
        room,
        if night { 0.38 } else { 0.5 },
    )
}

/// The Hilltop: open chords, wide apart, a harp sweeping up through each, and a horn calling over
/// the valley.
fn hilltop(night: bool) -> Score {
    // Each two bars' bass note and chord.
    const CHORDS: [(f32, [f32; 4]); 8] = [
        (38.0, [50.0, 57.0, 64.0, 66.0]), // D, with a ninth
        (43.0, [55.0, 62.0, 64.0, 69.0]), // G, six and nine
        (47.0, [54.0, 57.0, 62.0, 64.0]), // B minor seventh
        (45.0, [52.0, 57.0, 62.0, 64.0]), // A, suspended
        (43.0, [55.0, 59.0, 62.0, 66.0]), // G major seventh
        (42.0, [54.0, 57.0, 62.0, 69.0]), // D over F sharp
        (40.0, [55.0, 59.0, 62.0, 64.0]), // E minor seventh
        (45.0, [52.0, 57.0, 61.0, 64.0]), // A, and home to D
    ];
    // The horn: bar, beat, beats held, pitch.
    const CALL: [(f32, f32, f32, f32); 18] = [
        (2.0, 0.0, 2.0, 74.0),
        (2.0, 2.0, 2.0, 76.0),
        (3.0, 0.0, 4.0, 71.0),
        (4.0, 0.0, 2.0, 69.0),
        (4.0, 2.0, 2.0, 74.0),
        (5.0, 0.0, 4.0, 73.0),
        (6.0, 0.0, 2.0, 74.0),
        (6.0, 2.0, 2.0, 69.0),
        (7.0, 0.0, 4.0, 76.0),
        (10.0, 0.0, 2.0, 78.0),
        (10.0, 2.0, 2.0, 81.0),
        (11.0, 0.0, 2.0, 74.0),
        (11.0, 2.0, 2.0, 76.0),
        (12.0, 0.0, 4.0, 79.0),
        (13.0, 0.0, 2.0, 76.0),
        (13.0, 2.0, 2.0, 74.0),
        (14.0, 0.0, 4.0, 73.0),
        (15.0, 0.0, 2.0, 76.0),
    ];
    const LAST: (f32, f32, f32, f32) = (15.0, 2.0, 2.0, 69.0);
    let mut score = Writer::new(64.0, 0x4111);
    for (index, &(bass, chord)) in CHORDS.iter().enumerate() {
        let at = index as f32 * 8.0;
        score.chord(
            Instrument::Pad,
            at,
            8.0,
            &chord,
            if night { 0.17 } else { 0.2 },
        );
        score.played(Instrument::Bass, at, 7.0, bass, 0.38);
        // A harp sweeping up through the chord and on into the next octave; by night, only
        // every other time.
        if !night || index % 2 == 0 {
            let [a, b, c, d] = chord;
            let sweep = [a, b, c, d, a + 12.0, b + 12.0, c + 12.0, d + 12.0];
            for (step, pitch) in sweep.into_iter().enumerate() {
                let loud = (0.2 + 0.02 * step as f32) * if night { 0.75 } else { 1.0 };
                score.played(Instrument::Pluck, at + step as f32 * 0.5, 3.0, pitch, loud);
            }
        }
    }
    if !night {
        for (bar, beat, held, pitch) in CALL.into_iter().chain([LAST]) {
            score.played(Instrument::Horn, bar * 4.0 + beat, held * 0.95, pitch, 0.3);
        }
    }
    let room = Shape {
        size: 0.9,
        damping: 0.3,
        wet: if night { 0.48 } else { 0.42 },
    };
    score.score(
        if night { 60.0 } else { 72.0 },
        room,
        if night { 0.27 } else { 0.3 },
    )
}

/// The clearing: strings trembling in D minor, a kettle drum, and a brass call answered higher.
fn secret() -> Score {
    const DM: (f32, [f32; 3]) = (38.0, [62.0, 65.0, 69.0]);
    const BB: (f32, [f32; 3]) = (46.0, [62.0, 65.0, 70.0]);
    const GM: (f32, [f32; 3]) = (43.0, [62.0, 67.0, 70.0]);
    const A: (f32, [f32; 3]) = (45.0, [61.0, 64.0, 69.0]);
    const C: (f32, [f32; 3]) = (48.0, [60.0, 64.0, 67.0]);
    const BARS: [(f32, [f32; 3]); 16] = [DM, DM, BB, BB, GM, GM, A, A, DM, DM, BB, C, GM, A, DM, A];
    // The brass: bar, beat, beats held, pitch.
    const CALL: [(f32, f32, f32, f32); 30] = [
        (0.0, 0.0, 1.5, 62.0),
        (0.0, 1.5, 0.5, 65.0),
        (0.0, 2.0, 2.0, 69.0),
        (1.0, 0.0, 1.0, 70.0),
        (1.0, 1.0, 1.0, 69.0),
        (1.0, 2.0, 2.0, 65.0),
        (2.0, 0.0, 1.5, 70.0),
        (2.0, 1.5, 0.5, 69.0),
        (2.0, 2.0, 2.0, 65.0),
        (3.0, 0.0, 4.0, 62.0),
        (4.0, 0.0, 1.5, 67.0),
        (4.0, 1.5, 0.5, 70.0),
        (4.0, 2.0, 2.0, 74.0),
        (5.0, 0.0, 1.0, 72.0),
        (5.0, 1.0, 1.0, 70.0),
        (5.0, 2.0, 2.0, 67.0),
        (6.0, 0.0, 4.0, 69.0),
        (7.0, 0.0, 2.0, 73.0),
        (7.0, 2.0, 2.0, 76.0),
        (8.0, 0.0, 1.5, 74.0),
        (8.0, 1.5, 0.5, 77.0),
        (8.0, 2.0, 2.0, 81.0),
        (9.0, 0.0, 2.0, 77.0),
        (9.0, 2.0, 2.0, 74.0),
        (10.0, 0.0, 2.0, 74.0),
        (10.0, 2.0, 2.0, 77.0),
        (11.0, 0.0, 2.0, 76.0),
        (11.0, 2.0, 2.0, 79.0),
        (12.0, 0.0, 2.0, 74.0),
        (12.0, 2.0, 2.0, 70.0),
    ];
    const ANSWER: [(f32, f32, f32, f32); 5] = [
        (13.0, 0.0, 4.0, 73.0),
        (14.0, 0.0, 2.0, 74.0),
        (14.0, 2.0, 2.0, 69.0),
        (15.0, 0.0, 2.0, 73.0),
        (15.0, 2.0, 2.0, 64.0),
    ];
    let mut score = Writer::new(64.0, 0x5ec7);
    for (bar, &(root, chord)) in BARS.iter().enumerate() {
        let at = bar as f32 * 4.0;
        score.chord(Instrument::Strings, at, 3.9, &chord, 0.2);
        score.played(Instrument::Bass, at, 1.8, root + 12.0, 0.4);
        score.played(Instrument::Timpani, at, 1.0, root, 0.55);
        score.played(Instrument::Timpani, at + 2.5, 1.0, root, 0.3);
        // A roll into the turn of each half.
        if bar % 8 == 7 {
            for step in 0..4 {
                let loud = 0.18 + 0.08 * step as f32;
                score.played(
                    Instrument::Timpani,
                    at + 2.0 + step as f32 * 0.5,
                    0.5,
                    45.0,
                    loud,
                );
            }
        }
    }
    for (bar, beat, held, pitch) in CALL.into_iter().chain(ANSWER) {
        score.played(Instrument::Horn, bar * 4.0 + beat, held * 0.92, pitch, 0.34);
    }
    let room = Shape {
        size: 0.6,
        damping: 0.4,
        wet: 0.3,
    };
    score.score(92.0, room, 0.28)
}
