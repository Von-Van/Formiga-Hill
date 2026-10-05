//! The music: a soft placeholder loop for each place, made as it plays rather than stored, so
//! nothing big is made at start-up and nothing waits on it.
//!
//! A piece is a score, notes on a loop of whole bars (in `scores`), played by a little band (in
//! `band`) in a room of the place's own shape. Each note plays on from where it was struck,
//! ringing over the end of the loop into the next time round, so the loop joins without a seam;
//! and from the second time round on, every time round is the same.

mod band;
mod scores;

pub use band::Instrument;

use super::synth::{DcBlock, Room, Shape, hz};
use band::Sounding;

/// A place's piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Track {
    /// The station and the green.
    Pastoral,
    /// The Clubhouse, by the fire.
    Fireside,
    /// The Fairground's waltz.
    Waltz,
    Woods,
    Hilltop,
    /// The clearing.
    Secret,
}

impl Track {
    pub const ALL: [Track; 6] = [
        Track::Pastoral,
        Track::Fireside,
        Track::Waltz,
        Track::Woods,
        Track::Hilltop,
        Track::Secret,
    ];
}

/// A piece of music: a place's track, by day or, once the lamps are lit, by night.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Music {
    pub track: Track,
    pub night: bool,
}

impl Music {
    /// The clearing keeps its own hours, and has no night of its own.
    pub fn new(track: Track, night: bool) -> Self {
        Self {
            track,
            night: night && track != Track::Secret,
        }
    }

    /// Every piece there is, by day and by night.
    pub fn all() -> Vec<Music> {
        Track::ALL
            .into_iter()
            .flat_map(|track| [Music::new(track, false), Music::new(track, true)])
            .fold(Vec::new(), |mut all, music| {
                if !all.contains(&music) {
                    all.push(music);
                }
                all
            })
    }

    /// Its name, for a file of it: "waltz-night".
    pub fn name(self) -> String {
        let track = match self.track {
            Track::Pastoral => "pastoral",
            Track::Fireside => "fireside",
            Track::Waltz => "waltz",
            Track::Woods => "woods",
            Track::Hilltop => "hilltop",
            Track::Secret => return "secret".to_owned(),
        };
        format!("{track}-{}", if self.night { "night" } else { "day" })
    }

    pub fn score(self) -> Score {
        scores::write(self)
    }
}

/// A note in a score, timed in beats.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub at: f32,
    pub held: f32,
    /// A MIDI note: 60 is middle C.
    pub pitch: f32,
    pub loud: f32,
    pub instrument: Instrument,
}

/// A piece written out: its tempo, how many beats it runs before it comes round again, its notes,
/// the room it is played in, and how loud it is as a whole.
#[derive(Clone, Debug)]
pub struct Score {
    pub bpm: f32,
    pub beats: f32,
    pub notes: Vec<Note>,
    pub room: Shape,
    pub level: f32,
}

/// A note waiting for its moment in the loop.
#[derive(Clone, Copy, Debug)]
struct Planned {
    /// The sample of the loop it starts on.
    start: usize,
    hz: f32,
    held: f32,
    loud: f32,
    instrument: Instrument,
    seed: u32,
}

/// A piece playing, a sample at a time.
pub struct Player {
    rate: f32,
    planned: Vec<Planned>,
    /// Samples in one time round, and where in it the player is.
    length: usize,
    at: usize,
    next: usize,
    sounding: Vec<Sounding>,
    room: Room,
    level: f32,
    dc: DcBlock,
}

impl Player {
    pub fn new(music: Music, rate: f32) -> Self {
        Self::of(&music.score(), rate)
    }

    pub fn of(score: &Score, rate: f32) -> Self {
        let beat = 60.0 / score.bpm * rate;
        let length = (score.beats * beat).round().max(1.0) as usize;
        let mut planned: Vec<Planned> = score
            .notes
            .iter()
            .enumerate()
            .map(|(index, note)| Planned {
                start: ((note.at * beat).round() as usize).min(length - 1),
                hz: hz(note.pitch),
                held: note.held * 60.0 / score.bpm,
                loud: note.loud,
                instrument: note.instrument,
                seed: (index as u32).wrapping_mul(0x2545_f491) ^ 0x5bd1_e995,
            })
            .collect();
        planned.sort_by_key(|note| note.start);
        Self {
            rate,
            planned,
            length,
            at: 0,
            next: 0,
            sounding: Vec::with_capacity(64),
            room: Room::new(score.room, rate),
            level: score.level,
            dc: DcBlock::new(rate),
        }
    }

    /// How many samples one time round the loop takes.
    pub fn loop_length(&self) -> usize {
        self.length
    }

    /// The next sample.
    pub fn next(&mut self) -> f32 {
        while let Some(note) = self.planned.get(self.next)
            && note.start == self.at
        {
            self.sounding.push(Sounding::new(
                note.instrument,
                note.hz,
                note.held,
                note.loud,
                note.seed,
                self.rate,
            ));
            self.next += 1;
        }
        let mut sum = 0.0;
        // In the order they were struck, so every time round adds them up the same way.
        self.sounding.retain_mut(|note| {
            sum += note.next();
            !note.done()
        });
        self.at += 1;
        if self.at >= self.length {
            self.at = 0;
            self.next = 0;
        }
        self.dc.run(self.room.run(sum) * self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synth::RATE;

    /// A low rate, so whole loops can be listened to quickly. Every note is a function of its own
    /// time since it was struck, so a loop comes round the same way at any rate.
    const LISTENING: f32 = 2000.0;

    fn heard(player: &mut Player, samples: usize) -> Vec<f32> {
        (0..samples).map(|_| player.next()).collect()
    }

    /// Every piece, each listened to on a thread of its own: `listen` is given the player and
    /// how long its loop is.
    fn each_piece<T: Send>(listen: impl Fn(&mut Player, usize) -> T + Sync) -> Vec<(Music, T)> {
        std::thread::scope(|scope| {
            let listening: Vec<_> = Music::all()
                .into_iter()
                .map(|music| {
                    let listen = &listen;
                    scope.spawn(move || {
                        let mut player = Player::new(music, LISTENING);
                        let length = player.loop_length();
                        (music, listen(&mut player, length))
                    })
                })
                .collect();
            listening
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect()
        })
    }

    #[test]
    fn a_note_struck_before_the_seam_rings_on_over_it() {
        // A one-bar loop with a chime struck just before its end.
        let score = Score {
            bpm: 60.0,
            beats: 1.0,
            notes: vec![Note {
                at: 0.9,
                held: 0.5,
                pitch: 72.0,
                loud: 1.0,
                instrument: Instrument::Chime,
            }],
            room: Shape {
                size: 0.0,
                damping: 0.0,
                wet: 0.0,
            },
            level: 1.0,
        };
        let mut player = Player::of(&score, RATE);
        let length = player.loop_length();
        let samples = heard(&mut player, length * 2);
        let level = |from: usize| {
            samples[from..from + 400]
                .iter()
                .fold(0.0f32, |most, x| most.max(x.abs()))
        };
        assert!(level(0) == 0.0, "nothing rings before anything is struck");
        assert!(level(length) > 0.1, "the chime was cut off at the seam");
        // It carries straight on: no step across the seam bigger than its own.
        let steepest = samples[length - 400..length]
            .windows(2)
            .fold(0.0f32, |most, pair| most.max((pair[1] - pair[0]).abs()));
        assert!((samples[length] - samples[length - 1]).abs() <= steepest * 1.01);
    }

    #[test]
    fn every_piece_comes_round_the_same_every_time_round() {
        // Twice round, and a little more: enough for whatever rings over each seam.
        let after = (8.0 * LISTENING) as usize;
        for (music, (length, samples)) in
            each_piece(|player, length| (length, heard(player, length * 2 + after)))
        {
            let differs = samples[length..length + after]
                .iter()
                .zip(&samples[length * 2..])
                .fold(0.0f32, |most, (a, b)| most.max((a - b).abs()));
            assert!(differs < 1e-4, "{} drifts by {differs}", music.name());
            assert!(
                samples[..after]
                    .iter()
                    .zip(&samples[length..])
                    .any(|(a, b)| a != b),
                "{} has nothing ringing over its seam",
                music.name()
            );
        }
    }

    #[test]
    fn music_is_made_the_same_way_every_time() {
        for music in Music::all() {
            let mut one = Player::new(music, LISTENING);
            let mut two = Player::new(music, LISTENING);
            let (one, two) = (heard(&mut one, 8000), heard(&mut two, 8000));
            assert_eq!(one, two, "{}", music.name());
        }
    }

    #[test]
    fn every_piece_stays_well_within_full_scale() {
        for (music, loudest) in each_piece(|player, length| {
            heard(player, length)
                .into_iter()
                .fold(0.0f32, |most, sample| most.max(sample.abs()))
        }) {
            assert!(loudest < 0.9, "{} reaches {loudest}", music.name());
            assert!(loudest > 0.1, "{} can barely be heard", music.name());
        }
    }

    #[test]
    fn night_is_slower_and_quieter_than_day() {
        let loudness = each_piece(|player, length| {
            let samples = heard(player, length);
            (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
        });
        let of = |music: Music| {
            loudness
                .iter()
                .find(|(piece, _)| *piece == music)
                .map(|(_, loudness)| *loudness)
                .unwrap()
        };
        for track in Track::ALL
            .into_iter()
            .filter(|track| *track != Track::Secret)
        {
            let (day, night) = (Music::new(track, false), Music::new(track, true));
            assert!(night.score().bpm < day.score().bpm, "{track:?}");
            assert!(of(night) < of(day), "{track:?} is louder by night");
        }
        assert_eq!(
            Music::new(Track::Secret, true),
            Music::new(Track::Secret, false)
        );
    }

    #[test]
    fn every_piece_is_long_enough_not_to_be_learnt_and_its_notes_fit_it() {
        for music in Music::all() {
            let score = music.score();
            let seconds = score.beats * 60.0 / score.bpm;
            assert!(
                seconds >= 40.0,
                "{} loops every {seconds:.0}s",
                music.name()
            );
            for note in &score.notes {
                assert!((0.0..score.beats).contains(&note.at), "{}", music.name());
                // Nothing rings so long it reaches round to itself again.
                assert!(
                    note.held * 60.0 / score.bpm < seconds / 2.0,
                    "{}",
                    music.name()
                );
            }
        }
    }
}
