//! How loud the Hill is, as the person has set it, kept in `settings.json` in Hill's own data
//! folder: one setting for everyone who visits, not one per colony. Read as `memories.rs` reads
//! its book: anything missing takes its default, anything too big is not Hill's, and a file that
//! cannot be read is set aside rather than lost.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const FILE: &str = "settings.json";
/// A handful of numbers; anything bigger is not Hill's.
const MAX_BYTES: u64 = 64 * 1024;

/// The person's sound settings, each level from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Levels {
    /// Everything at once.
    pub master: f32,
    pub music: f32,
    /// The effects: the train, the games, the camera and the rest.
    pub sounds: f32,
    pub muted: bool,
}

impl Default for Levels {
    /// Music soft under everything else, as background music should be.
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.4,
            sounds: 0.7,
            muted: false,
        }
    }
}

impl Levels {
    /// Each level made one the Hill can use: from 0 to 1, and its default if it is not a number.
    pub fn clamped(self) -> Self {
        let defaults = Self::default();
        let level = |value: f32, default: f32| {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                default
            }
        };
        Self {
            master: level(self.master, defaults.master),
            music: level(self.music, defaults.music),
            sounds: level(self.sounds, defaults.sounds),
            muted: self.muted,
        }
    }

    /// How loud the music plays: the master level times its own, and nothing when muted.
    pub fn music_gain(&self) -> f32 {
        self.gain(self.music)
    }

    /// How loud the sounds play, the same way.
    pub fn sounds_gain(&self) -> f32 {
        self.gain(self.sounds)
    }

    fn gain(&self, own: f32) -> f32 {
        let levels = self.clamped();
        if levels.muted {
            0.0
        } else {
            levels.master * own.clamp(0.0, 1.0)
        }
    }
}

/// What the file holds: the sound's levels, and anything a later Hill keeps beside them, which is
/// written back as it was found.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Contents {
    #[serde(default)]
    sound: Levels,
    #[serde(flatten)]
    others: serde_json::Map<String, serde_json::Value>,
}

/// The settings, and where they are kept.
#[derive(Debug)]
pub struct Settings {
    path: Option<PathBuf>,
    contents: Contents,
    /// The levels as last written, so they are only written again once they change.
    kept: Levels,
}

impl Settings {
    /// The settings kept in `data`; the defaults if there are none, or nowhere to keep them.
    pub fn open(data: Option<&Path>) -> Self {
        let path = data.map(|data| data.join(FILE));
        let mut contents = path.as_deref().map(read).unwrap_or_default();
        contents.sound = contents.sound.clamped();
        Self {
            kept: contents.sound,
            path,
            contents,
        }
    }

    pub fn levels(&self) -> Levels {
        self.contents.sound
    }

    /// Takes new levels at once; `keep` writes them down.
    pub fn set(&mut self, levels: Levels) {
        self.contents.sound = levels.clamped();
    }

    /// Writes the levels down, if they have changed since they last were. A failure is reported
    /// and otherwise ignored: the Hill sounds as set, and only forgets it next time.
    pub fn keep(&mut self) {
        if self.contents.sound == self.kept {
            return;
        }
        self.kept = self.contents.sound;
        let Some(path) = &self.path else {
            return;
        };
        if let Err(error) = write(path, &self.contents) {
            eprintln!("formiga-hill: could not keep the sound settings: {error}");
        }
    }
}

fn read(path: &Path) -> Contents {
    let Ok(metadata) = fs::metadata(path) else {
        return Contents::default();
    };
    let parsed = (metadata.len() <= MAX_BYTES)
        .then(|| fs::read(path).ok())
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<Contents>(&bytes).ok());
    parsed.unwrap_or_else(|| {
        // Keep what was there for whoever wants to look at it, and start again.
        let mut aside = path.as_os_str().to_owned();
        aside.push(".unreadable");
        let _ = fs::rename(path, PathBuf::from(aside));
        Contents::default()
    })
}

fn write(path: &Path, contents: &Contents) -> std::io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let mut file = fs::File::create(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(contents)?)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_music_is_soft_under_the_sounds_by_default() {
        let levels = Settings::open(None).levels();
        assert_eq!(levels, Levels::default());
        assert_eq!(levels.master, 1.0);
        assert!((levels.music - 0.4).abs() < 1e-6);
        assert!((levels.sounds - 0.7).abs() < 1e-6);
        assert!(!levels.muted);
        let fresh = scratch("fresh");
        assert_eq!(Settings::open(Some(&fresh)).levels(), Levels::default());
    }

    #[test]
    fn each_kind_plays_at_the_master_level_times_its_own() {
        let levels = Levels {
            master: 0.5,
            music: 0.4,
            sounds: 0.8,
            muted: false,
        };
        assert!((levels.music_gain() - 0.2).abs() < 1e-6);
        assert!((levels.sounds_gain() - 0.4).abs() < 1e-6);
        let muted = Levels {
            muted: true,
            ..levels
        };
        assert_eq!((muted.music_gain(), muted.sounds_gain()), (0.0, 0.0));
        let silent = Levels {
            master: 0.0,
            ..levels
        };
        assert_eq!((silent.music_gain(), silent.sounds_gain()), (0.0, 0.0));
        let no_music = Levels {
            music: 0.0,
            ..levels
        };
        assert_eq!(no_music.music_gain(), 0.0);
        assert!(
            no_music.sounds_gain() > 0.0,
            "the sounds go on without the music"
        );
    }

    #[test]
    fn levels_out_of_range_are_brought_back_into_it() {
        let wild = Levels {
            master: 3.0,
            music: -1.0,
            sounds: f32::NAN,
            muted: false,
        }
        .clamped();
        assert_eq!(wild.master, 1.0);
        assert_eq!(wild.music, 0.0);
        assert_eq!(wild.sounds, Levels::default().sounds);
        let dir = scratch("clamped");
        fs::write(
            dir.join(FILE),
            br#"{"sound": {"master": 7, "music": -0.5, "sounds": 0.25}}"#,
        )
        .unwrap();
        let read = Settings::open(Some(&dir)).levels();
        assert_eq!((read.master, read.music, read.sounds), (1.0, 0.0, 0.25));
        let mut settings = Settings::open(Some(&dir));
        settings.set(Levels {
            master: 2.0,
            ..Levels::default()
        });
        assert_eq!(settings.levels().master, 1.0);
    }

    #[test]
    fn settings_kept_are_there_next_time_with_anything_else_in_the_file() {
        let dir = scratch("kept");
        fs::write(
            dir.join(FILE),
            br#"{"sound": {"music": 0.1}, "theme": "dusk"}"#,
        )
        .unwrap();
        let mut settings = Settings::open(Some(&dir));
        assert_eq!(settings.levels().music, 0.1);
        assert_eq!(
            settings.levels().sounds,
            0.7,
            "a missing level takes its default"
        );
        let changed = Levels {
            master: 0.8,
            music: 0.3,
            sounds: 0.55,
            muted: true,
        };
        settings.set(changed);
        settings.keep();
        assert_eq!(Settings::open(Some(&dir)).levels(), changed);
        let written = fs::read_to_string(dir.join(FILE)).unwrap();
        assert!(written.contains("\"theme\": \"dusk\""), "{written}");
    }

    #[test]
    fn nothing_is_written_until_something_changes() {
        let dir = scratch("unchanged");
        let mut settings = Settings::open(Some(&dir));
        settings.keep();
        assert!(!dir.join(FILE).exists());
        settings.set(Levels::default());
        settings.keep();
        assert!(!dir.join(FILE).exists());
    }

    #[test]
    fn an_unreadable_file_is_set_aside_and_the_defaults_used() {
        let dir = scratch("corrupt");
        fs::write(dir.join(FILE), b"{ this is not json").unwrap();
        assert_eq!(Settings::open(Some(&dir)).levels(), Levels::default());
        assert!(dir.join("settings.json.unreadable").exists());
        assert!(!dir.join(FILE).exists());
        // Too big to be Hill's is the same.
        fs::write(dir.join(FILE), vec![b' '; MAX_BYTES as usize + 1]).unwrap();
        assert_eq!(Settings::open(Some(&dir)).levels(), Levels::default());
    }
}
