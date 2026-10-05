//! Sound at the Hill: a soft piece of music for each place, and a sound for what happens there.
//! Every one is a placeholder made in code (`synth`, `cues`, `music`), until real audio replaces
//! it: there are no files to ship, license or download.
//!
//! Sound is optional and never needed to play. Nothing opens until something is to be heard; with
//! nowhere to play it, the Hill says so once and plays on quietly; and muted, or turned right
//! down, nothing plays at all. The levels are the person's own, kept in `settings.json`.
//!
//! The window holds a `Sound` and asks it for cues and music as things happen. The mixer that
//! makes them runs on the speaker's own thread, so the window never waits on it.

mod cues;
mod mixer;
mod music;
mod settings;
mod speaker;
mod synth;
mod wav;

pub use cues::Cue;
pub use music::{Music, Track};
pub use settings::Levels;
pub use wav::render_sounds;

use mixer::Command;
use settings::Settings;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

/// Opens somewhere to play and plays what it is sent until the second channel closes: the
/// device, for the window, and stand-ins for the tests, which never need one.
type Speak = fn(Receiver<Command>, Receiver<()>) -> Result<(), String>;

/// Where the sound goes.
enum Output {
    /// Nothing audible has been asked for yet, so nothing has been opened.
    Unopened,
    /// The speaker is open, or opening, on a thread of its own.
    Open {
        commands: Sender<Command>,
        /// Let go when the window closes, which lets the speaker go too.
        _hush: Sender<()>,
    },
    /// There is nothing to play on. Nothing more is tried.
    Silent,
}

pub struct Sound {
    settings: Settings,
    output: Output,
    /// The piece wanted now, heard or not.
    music: Option<Music>,
    speak: Speak,
    /// Set by the speaker's thread if it found nothing to play on.
    failed: Arc<AtomicBool>,
}

impl Sound {
    /// The Hill's sound, set as it was last time in `data`.
    pub fn open(data: Option<&Path>) -> Self {
        Self::with(data, speaker::speak)
    }

    fn with(data: Option<&Path>, speak: Speak) -> Self {
        Self {
            settings: Settings::open(data),
            output: Output::Unopened,
            music: None,
            speak,
            failed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn levels(&self) -> Levels {
        self.settings.levels()
    }

    /// New levels, heard at once; `keep` writes them down.
    pub fn set_levels(&mut self, levels: Levels) {
        self.settings.set(levels);
        let levels = self.levels();
        self.send(
            Command::Levels {
                music: levels.music_gain(),
                sounds: levels.sounds_gain(),
            },
            false,
        );
        // Music wanted while it could not be heard may be heard now.
        if self.music.is_some() && levels.music_gain() > 0.0 {
            self.send(Command::Music(self.music), true);
        }
    }

    /// Writes the levels down, if they have changed.
    pub fn keep(&mut self) {
        self.settings.keep();
    }

    /// Mutes everything, or brings it back, and remembers which. Says whether it is now muted.
    pub fn toggle_mute(&mut self) -> bool {
        let mut levels = self.levels();
        levels.muted = !levels.muted;
        self.set_levels(levels);
        self.keep();
        levels.muted
    }

    pub fn play(&mut self, cue: Cue) {
        self.play_softly(cue, 1.0);
    }

    /// A cue, `loudness` as loud as it is made.
    pub fn play_softly(&mut self, cue: Cue, loudness: f32) {
        if self.levels().sounds_gain() > 0.0 && loudness > 0.0 {
            self.send(Command::Play { cue, loudness }, true);
        }
    }

    /// The piece for wherever the person is now, or quiet; crossfaded from whatever was playing.
    pub fn music(&mut self, music: Option<Music>) {
        if music == self.music {
            return;
        }
        self.music = music;
        let audible = music.is_some() && self.levels().music_gain() > 0.0;
        self.send(Command::Music(music), audible);
    }

    /// Whether there turned out to be nothing to play sound on.
    pub fn silent(&self) -> bool {
        matches!(self.output, Output::Silent) || self.failed.load(Ordering::Relaxed)
    }

    /// Sends a command to the speaker, opening it first if this is the first thing to be heard.
    /// Before then, the levels and the music are only remembered, and sent once it opens.
    fn send(&mut self, command: Command, audible: bool) {
        if matches!(self.output, Output::Unopened) {
            if !audible {
                return;
            }
            self.output = self.start();
            let levels = self.levels();
            self.post(Command::Levels {
                music: levels.music_gain(),
                sounds: levels.sounds_gain(),
            });
            self.post(Command::Music(self.music));
        }
        self.post(command);
    }

    fn post(&mut self, command: Command) {
        if let Output::Open { commands, .. } = &self.output
            && commands.send(command).is_err()
        {
            // The speaker has gone: there was nothing to play on.
            self.output = Output::Silent;
        }
    }

    /// Opens the speaker on a thread of its own, once and for all.
    fn start(&self) -> Output {
        let (commands, heard) = mpsc::channel();
        let (hush, hushed) = mpsc::channel();
        let speak = self.speak;
        let failed = Arc::clone(&self.failed);
        let spawned = std::thread::Builder::new()
            .name("formiga-hill sound".to_owned())
            .spawn(move || {
                if let Err(problem) = speak(heard, hushed) {
                    failed.store(true, Ordering::Relaxed);
                    eprintln!(
                        "formiga-hill: there is nothing to play sound on ({problem}), so the \
                         Hill is quiet"
                    );
                }
            });
        match spawned {
            Ok(_) => Output::Open {
                commands,
                _hush: hush,
            },
            Err(_) => Output::Silent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    static TRIED: AtomicUsize = AtomicUsize::new(0);

    fn no_device(_: Receiver<Command>, _: Receiver<()>) -> Result<(), String> {
        TRIED.fetch_add(1, Ordering::SeqCst);
        Err("no device here".to_owned())
    }

    #[test]
    fn with_nothing_to_play_on_the_hill_plays_on_quietly_and_tries_only_once() {
        let mut sound = Sound::with(None, no_device);
        let pastoral = Music::new(Track::Pastoral, false);
        let started = Instant::now();
        while !sound.silent() {
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "it never gave up"
            );
            sound.play(Cue::Bell);
            sound.music(Some(pastoral));
            sound.music(None);
            std::thread::sleep(Duration::from_millis(1));
        }
        for cue in Cue::ALL {
            sound.play(cue);
        }
        sound.music(Some(Music::new(Track::Woods, true)));
        sound.toggle_mute();
        sound.toggle_mute();
        assert_eq!(TRIED.load(Ordering::SeqCst), 1);
    }

    static OPENED: AtomicUsize = AtomicUsize::new(0);

    fn counted(_: Receiver<Command>, hush: Receiver<()>) -> Result<(), String> {
        OPENED.fetch_add(1, Ordering::SeqCst);
        let _ = hush.recv();
        Ok(())
    }

    #[test]
    fn nothing_is_opened_until_something_can_be_heard() {
        let mut sound = Sound::with(None, counted);
        let muted = Levels {
            muted: true,
            ..Levels::default()
        };
        sound.set_levels(muted);
        sound.play(Cue::Whistle);
        sound.music(Some(Music::new(Track::Pastoral, false)));
        assert!(
            matches!(sound.output, Output::Unopened),
            "muted, yet opened"
        );
        sound.set_levels(Levels {
            music: 0.0,
            sounds: 0.0,
            ..Levels::default()
        });
        sound.play(Cue::Whistle);
        assert!(
            matches!(sound.output, Output::Unopened),
            "at nothing, yet opened"
        );
        // Unmuted, with music wanted: now there is something to hear.
        sound.set_levels(Levels::default());
        assert!(matches!(sound.output, Output::Open { .. }));
        sound.play(Cue::Bell);
        drop(sound);
        let started = Instant::now();
        while OPENED.load(Ordering::SeqCst) == 0 {
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(OPENED.load(Ordering::SeqCst), 1);
    }
}
