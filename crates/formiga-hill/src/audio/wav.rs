//! Every sound written to a WAV file, for listening to them without the window: each cue, and a
//! minute of each piece of music, as the person would hear it with everything at full volume.
//! The header is written by hand: sixteen-bit mono PCM needs nothing more.

use super::cues::Cue;
use super::music::{Music, Player};
use super::synth::{RATE, soften};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

/// How much of each piece of music is written.
const MUSIC_SECS: f32 = 60.0;

/// A sixteen-bit mono WAV of `samples` at `rate`, each clamped to full scale.
pub fn encode(samples: &[f32], rate: u32) -> Vec<u8> {
    let data = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data as usize);
    bytes.extend(b"RIFF");
    bytes.extend((36 + data).to_le_bytes());
    bytes.extend(b"WAVE");
    bytes.extend(b"fmt ");
    bytes.extend(16u32.to_le_bytes());
    // Plain PCM, one channel, two bytes a sample.
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data.to_le_bytes());
    for sample in samples {
        let level = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        bytes.extend(level.to_le_bytes());
    }
    bytes
}

/// Writes every cue and a minute of every piece of music into `folder`, and says what it wrote.
pub fn render_sounds(folder: &Path) -> Result<Vec<String>> {
    fs::create_dir_all(folder)
        .with_context(|| format!("could not make the folder {}", folder.display()))?;
    let mut written = Vec::new();
    for cue in Cue::ALL {
        let samples: Vec<f32> = cue.voice(0, RATE).map(soften).collect();
        let name = format!("cue-{}.wav", cue.name());
        save(&folder.join(&name), &samples)?;
        written.push(format!("{name}: {:.2}s", samples.len() as f32 / RATE));
    }
    for music in Music::all() {
        let mut player = Player::new(music, RATE);
        let length = (MUSIC_SECS * RATE) as usize;
        // Let go of at the end over a moment, rather than cut off mid-note.
        let fade = (0.05 * RATE) as usize;
        let samples: Vec<f32> = (0..length)
            .map(|at| soften(player.next()) * ((length - at) as f32 / fade as f32).min(1.0))
            .collect();
        let name = format!("music-{}.wav", music.name());
        save(&folder.join(&name), &samples)?;
        written.push(format!(
            "{name}: {:.0}s, coming round every {:.2}s",
            MUSIC_SECS,
            player.loop_length() as f32 / RATE
        ));
    }
    Ok(written)
}

fn save(path: &Path, samples: &[f32]) -> Result<()> {
    let mut file =
        fs::File::create(path).with_context(|| format!("could not create {}", path.display()))?;
    file.write_all(&encode(samples, RATE as u32))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wav_says_what_it_holds_and_holds_it() {
        let bytes = encode(&[0.0, 0.5, -1.0, 2.0], 44_100);
        assert_eq!(bytes.len(), 44 + 8);
        let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let half = |at: usize| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(word(4), 36 + 8);
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!((word(16), half(20), half(22)), (16, 1, 1));
        assert_eq!(
            (word(24), word(28), half(32), half(34)),
            (44_100, 88_200, 2, 16)
        );
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(word(40), 8);
        let samples: Vec<i16> = bytes[44..]
            .chunks(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        assert_eq!(
            samples,
            [0, 16384, -32767, 32767],
            "too loud is held at full scale"
        );
    }
}
