//! The album: photos taken at the Hill, kept for each colony beside Hill's memories, as small
//! PNGs at the scene's own scale. Nothing here goes to Desktop. A copy can be saved anywhere the
//! person chooses, made bigger by whole pixels so it stays crisp.

use anyhow::{Context, Result, bail};
use formiga_art::{Canvas, Rgba};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::macros::format_description;

/// How many photos one colony's album holds; past this, taking another says the album is full.
pub const MOST: usize = 240;
/// The biggest photo there can be: the whole scene. Anything bigger in the folder isn't one.
const MAX_SIDE: u32 = 384;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

pub struct Photo {
    /// Its file's name, which is when it was taken.
    pub name: String,
    pub taken: OffsetDateTime,
    pub picture: Canvas,
}

pub struct Album {
    folder: Option<PathBuf>,
    pub photos: Vec<Photo>,
}

impl Album {
    /// The colony's album, kept under `data`, oldest first. With nowhere to keep it, an album
    /// that lasts only the visit.
    pub fn open(data: Option<&Path>, colony: &str) -> Self {
        let folder = data.map(|data| data.join("photos").join(safe(colony)));
        let mut photos: Vec<Photo> = folder
            .as_deref()
            .and_then(|folder| fs::read_dir(folder).ok())
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?.to_str()?.to_owned();
                let taken = from_stamp(&name)?;
                let picture = read(&path).ok()?;
                Some(Photo {
                    name,
                    taken,
                    picture,
                })
            })
            .collect();
        photos.sort_by(|a, b| a.name.cmp(&b.name));
        Self { folder, photos }
    }

    pub fn is_full(&self) -> bool {
        self.photos.len() >= MOST
    }

    /// Keeps a photo taken `at`.
    pub fn keep(&mut self, picture: Canvas, at: OffsetDateTime) -> Result<()> {
        if self.is_full() {
            bail!("the album is full");
        }
        let at = at.to_offset(time::UtcOffset::UTC);
        let mut name = stamp(at);
        // Two photos in the same second: the second waits its turn.
        let mut extra = 1;
        while self.photos.iter().any(|photo| photo.name == name) {
            name = format!("{}-{extra}", stamp(at));
            extra += 1;
        }
        if let Some(folder) = &self.folder {
            fs::create_dir_all(folder)?;
            write(&folder.join(format!("{name}.png")), &picture, 1)?;
        }
        self.photos.push(Photo {
            name,
            taken: at,
            picture,
        });
        Ok(())
    }

    /// Takes a photo off the disk, and out of the album. The disk goes first: a photo that cannot
    /// be deleted stays in the album, rather than vanishing now and coming back next visit.
    pub fn remove(&mut self, index: usize) -> Result<()> {
        let Some(photo) = self.photos.get(index) else {
            return Ok(());
        };
        if let Some(folder) = &self.folder {
            let path = folder.join(format!("{}.png", photo.name));
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        self.photos.remove(index);
        Ok(())
    }
}

/// A copy of a photo at `path`, `scale` times as big each way.
pub fn save_copy(picture: &Canvas, path: &Path, scale: u32) -> Result<()> {
    write(path, picture, scale)
}

fn stamp(at: OffsetDateTime) -> String {
    at.format(format_description!(
        "[year][month][day]T[hour][minute][second]Z"
    ))
    .unwrap_or_else(|_| "photo".into())
}

fn from_stamp(name: &str) -> Option<OffsetDateTime> {
    let stamp = name.split('-').next()?;
    let parsed = time::PrimitiveDateTime::parse(
        stamp.strip_suffix('Z')?,
        format_description!("[year][month][day]T[hour][minute][second]"),
    )
    .ok()?;
    Some(parsed.assume_utc())
}

/// A colony's id, made fit to be a folder's name.
fn safe(colony: &str) -> String {
    let cleaned: String = colony
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if cleaned.is_empty() {
        "colony".into()
    } else {
        cleaned
    }
}

/// `picture` as PNG bytes, `scale` times as big each way.
pub fn png(picture: &Canvas, scale: u32) -> Vec<u8> {
    let (width, height) = (picture.width() * scale, picture.height() * scale);
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let pixel = picture.get((x / scale) as i32, (y / scale) as i32);
            data.extend([pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    // Writing into memory cannot fail but for a mismatch in size, which this never makes.
    if let Ok(mut writer) = encoder.write_header() {
        let _ = writer.write_image_data(&data);
    }
    bytes
}

/// `picture` saved at `path` as a PNG, `scale` times as big each way.
pub fn write(path: &Path, picture: &Canvas, scale: u32) -> Result<()> {
    fs::write(path, png(picture, scale))
        .with_context(|| format!("could not create {}", path.display()))
}

/// A photo read back, refusing anything that isn't one of Hill's own: too big, or not plain
/// eight-bit RGBA.
fn read(path: &Path) -> Result<Canvas> {
    if fs::metadata(path)?.len() > MAX_FILE_BYTES {
        bail!("too big to be a photo");
    }
    let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(path)?));
    let mut reader = decoder.read_info()?;
    let info = reader.info();
    let (width, height) = (info.width, info.height);
    if width > MAX_SIDE
        || height > MAX_SIDE
        || info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
    {
        bail!("not one of Hill's photos");
    }
    let mut buffer = vec![0; reader.output_buffer_size().context("no size")?];
    reader.next_frame(&mut buffer)?;
    let mut picture = Canvas::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let at = ((y * width + x) * 4) as usize;
            picture.set(
                x as i32,
                y as i32,
                Rgba::new(buffer[at], buffer[at + 1], buffer[at + 2], buffer[at + 3]),
            );
        }
    }
    Ok(picture)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-photos-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn picture() -> Canvas {
        let mut picture = Canvas::new(40, 24);
        picture.fill_rect(0, 0, 40, 24, crate::paint::rgb(0x86bb7c));
        picture.fill_rect(10, 5, 8, 8, crate::paint::rgb(0xd0574a));
        picture
    }

    #[test]
    fn a_photo_kept_is_there_next_visit_and_stays_with_its_colony() {
        let data = scratch("kept");
        let at = OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap();
        let mut album = Album::open(Some(&data), "colony-a");
        album.keep(picture(), at).unwrap();
        album.keep(picture(), at).unwrap();
        let again = Album::open(Some(&data), "colony-a");
        assert_eq!(again.photos.len(), 2, "two in one second are both kept");
        assert_eq!(again.photos[0].picture, picture());
        assert_eq!(again.photos[0].taken, at);
        assert!(Album::open(Some(&data), "colony-b").photos.is_empty());
    }

    #[test]
    fn a_photo_taken_out_is_gone() {
        let data = scratch("removed");
        let at = OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap();
        let mut album = Album::open(Some(&data), "c");
        album.keep(picture(), at).unwrap();
        album.remove(0).unwrap();
        assert!(Album::open(Some(&data), "c").photos.is_empty());
    }

    #[test]
    fn a_photo_that_cannot_be_deleted_stays_in_the_album() {
        let data = scratch("stuck");
        let at = OffsetDateTime::from_unix_timestamp(1_790_000_000).unwrap();
        let mut album = Album::open(Some(&data), "c");
        album.keep(picture(), at).unwrap();
        // Something in the way of deleting it: a folder, not empty, where its file was.
        let folder = album.folder.clone().unwrap();
        let path = folder.join(format!("{}.png", album.photos[0].name));
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("in-the-way"), b"").unwrap();
        assert!(album.remove(0).is_err());
        assert_eq!(album.photos.len(), 1, "it is still there, to try again");
    }

    #[test]
    fn a_copy_is_bigger_by_whole_pixels() {
        let data = scratch("copy");
        fs::create_dir_all(&data).unwrap();
        let path = data.join("copy.png");
        save_copy(&picture(), &path, 4).unwrap();
        let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(&path).unwrap()));
        let reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (160, 96));
    }

    #[test]
    fn anything_in_the_folder_that_isnt_a_photo_is_left_alone() {
        let data = scratch("junk");
        let folder = data.join("photos").join("c");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("20260101T000000Z.png"), b"not a png").unwrap();
        fs::write(folder.join("notes.txt"), b"hello").unwrap();
        assert!(Album::open(Some(&data), "c").photos.is_empty());
    }
}
