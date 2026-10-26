// No console window behind Hill's own on Windows, as with Desktop. Anything run for its output,
// as the packaging script runs --travel-version, has its output caught rather than shown.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

//! Formiga Hill: somewhere the colony goes to spend time together.

mod actor;
mod adventure;
mod app;
mod audio;
mod cast;
mod character;
mod clearing;
mod clubhouse;
mod costume;
mod cues;
mod daylight;
mod dice;
mod expedition;
mod fairground;
mod falls;
mod finds;
mod fishing;
mod font;
mod green;
mod hedgerow;
mod hilltop;
mod hosting;
mod icon;
mod kit;
mod lettering;
mod materials;
mod meadow;
mod memories;
mod paint;
mod photos;
mod playground;
mod render;
mod sharing;
mod sheet;
mod station;
mod story;
mod track;
mod trees;
mod trip;
mod woods;

use anyhow::{Context, Result, bail};
use app::{Arrival, HillApp, Visit};
use cast::Cast;
use formiga_travel::{LAUNCH_ARGUMENT, SessionId, project_colony};
use render::Area;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const USAGE: &str = "\
Formiga Hill

Usage: formiga-hill [--sample | --formiga-travel <TRIP DIRECTORY> | --from-save <FILE>]
                    [--render-station <PNG> | --render-green <PNG> | …] [--at <SECONDS>]

  --sample                 Arrive with Desktop's made-up sample colony, straight away. With no
                           arguments at all it arrives too, behind a card saying that a colony
                           of your own comes by train from Formiga Desktop
  --formiga-travel <DIR>   How Desktop starts Hill: the trip it wrote, answered on the way home
  --from-save <FILE>       Development only: board a Desktop colony file, which is only ever read
  --render-station <PNG>   Draw the station to a PNG and exit without opening a window
  --render-green <PNG>     Draw the Village Green to a PNG and exit without opening a window
  --render-clubhouse <PNG> Draw the Clubhouse to a PNG and exit without opening a window
  --render-fairground <PNG>
                           Draw the Fairground to a PNG and exit without opening a window
  --render-hide-and-seek <PNG>
                           Draw a game of hide-and-seek at the Fairground, --at seconds into
                           the search
  --render-sack-race <PNG> Draw a sack race at the Fairground, --at seconds after the off
  --render-high-striker <PNG>
                           Draw the colony at the high striker, --at seconds into the game
  --render-hoopla <PNG>    Draw the colony throwing rings at the hoopla stall, --at seconds in
  --render-tug-of-war <PNG>
                           Draw the colony at the tug-of-war, --at seconds into the bout
  --render-woods <PNG>     Draw a rummage in the Woods, --at seconds into it
  --render-hilltop <PNG>   Draw the Hilltop with a sample of finds placed on it
  --render-finds <PNG>     Draw every find's icon and Hilltop piece on one sheet, for review
  --render-growing <PNG>   Draw everything that grows on the Hilltop at each of its stages
  --render-plans <PNG>     Draw everything the colony can build on the Hilltop, and from what
  --render-building <PNG>  Draw the colony building a wishing well on the Hilltop, --at seconds in
  --render-fishing <PNG>   Draw a fishing trip at the pool, --at seconds into it
  --render-fish <PNG>      Draw every fish as it is held up, and its icon, for review
  --render-meadow <PNG>    Draw the meadow at the Woods' edge, every bug settled in its haunt
  --render-bugs <PNG>      Draw every bug in each of its poses, close up, and its icon, for review
  --render-bug-hunt <PNG>  Draw a bug hunt in the meadow, --at seconds in, played by a patient hand
  --render-hedgerow <PNG>  Draw a foray along the hedgerow, --at seconds in, played by a canny hand
  --render-expedition <PNG>
                           Draw an expedition --at seconds in, played by a steady hand: the map
                           of the Woods, and the stops along the way
  --render-falls <PNG>     Draw a visit to the Far Falls, --at seconds in, played by a steady hand
  --render-track <PNG>     Draw a scavenge along the old track, --at seconds in, played by a
                           careful hand
  --render-treasure <PNG>  Draw a treasure hunt off the old track, --at seconds in, following the
                           map as a careful reader would
  --render-landmarks <PNG> Draw every landmark a map can name, near and far off, for review
  --render-produce <PNG>   Draw everything that grows on the hedgerow at each stage, for review
  --render-sovereign <PNG> Draw the secret encounter --at seconds in, played through on its own
  --render-reactions <PNG> Draw everyone answering a pat, a snack and a toy, for review
  --render-costumes <PNG>  Draw everyone wearing every piece in the dress-up box, for review
  --render-story <PNG>     Draw a story in the Clubhouse --at seconds after it starts, reading each
                           line for 2.5 seconds and taking the first choice
  --render-adventure <PNG> Draw a Woods adventure --at seconds in, played by a steady hand along
                           the shortest way to an ending: the last loaded, so an author's own
  --render-sounds <FOLDER> Write every sound and a minute of every piece of music to WAV files in
                           the folder, to listen to without a window
  --package <FOLDER>       Load a story package beside Hill's own (for authors); repeatable
  --check-package <FOLDER> Check a story package, or a share file, and say what is wrong,
                           without opening a window
  --share-package <FOLDER> <FILE>
                           Write a story package to one share file, to pass on, once it loads
  --packages-folder        Say where to put story packages for Hill to find, and what is there
  --sample-hilltop         Draw the station's skyline with a sample of finds on the Hilltop
  --hour <HOUR>            Draw at that hour of the day, from 0 to 24, rather than at midday;
                           with a window, hold the Hill at that hour rather than the clock's
  --at <SECONDS>           Draw that far into the arrival, or into free play on the green, in
                           the Clubhouse or at the Fairground; with --snap, when to take it
  --snap <PNG>             For review: open the window, picture it after --at seconds (3 if not
                           given) and close, with nothing the keys or pointer do let in
  --place <PLACE>          With --snap: open at the station, green, clubhouse, fairground, woods
                           or hilltop rather than the station
  --card <CARD>            With --snap: open the welcome, notices, departures, board, shelf,
                           writing, dress-up, journal, plans, album, sound, making, competitions
                           or new-competition card, at its own place if none is given
  --story                  With --snap in the Clubhouse: begin the first story on the shelves
  --chapter <N>            With --snap --card journal: open the Nth chapter of its contents;
                           with --card writing, the Nth scene of a sample story; with --card
                           making, the Nth place (then path) of a sample adventure picked out
";

/// Each `--render-*` option, and what it draws.
const RENDERS: [(&str, Area); 32] = [
    ("--render-station", Area::Station),
    ("--render-green", Area::Green),
    ("--render-clubhouse", Area::Clubhouse),
    ("--render-fairground", Area::Fairground),
    ("--render-hide-and-seek", Area::HideAndSeek),
    ("--render-sack-race", Area::SackRace),
    ("--render-high-striker", Area::HighStriker),
    ("--render-hoopla", Area::Hoopla),
    ("--render-tug-of-war", Area::TugOfWar),
    ("--render-woods", Area::Woods),
    ("--render-hilltop", Area::Hilltop),
    ("--render-finds", Area::Finds),
    ("--render-growing", Area::Growing),
    ("--render-plans", Area::Plans),
    ("--render-building", Area::Building),
    ("--render-fishing", Area::Fishing),
    ("--render-fish", Area::Fish),
    ("--render-meadow", Area::Meadow),
    ("--render-bugs", Area::Bugs),
    ("--render-bug-hunt", Area::BugHunt),
    ("--render-hedgerow", Area::Hedgerow),
    ("--render-produce", Area::Produce),
    ("--render-expedition", Area::Expedition),
    ("--render-falls", Area::Falls),
    ("--render-track", Area::Track),
    ("--render-treasure", Area::Treasure),
    ("--render-landmarks", Area::Landmarks),
    ("--render-sovereign", Area::Sovereign),
    ("--render-story", Area::Story),
    ("--render-adventure", Area::Adventure),
    ("--render-reactions", Area::Reactions),
    ("--render-costumes", Area::Costumes),
];

enum Source {
    Sample,
    Trip(PathBuf),
    Save(PathBuf),
}

struct Args {
    source: Source,
    render: Option<(Area, PathBuf)>,
    at: Option<f32>,
    /// Package folders to load beside Hill's own, for authors.
    packages: Vec<PathBuf>,
    /// Check these package folders and report, without opening a window.
    check: Vec<PathBuf>,
    /// Say where the packages folder is, and what is in it, without opening a window.
    show_folder: bool,
    /// Write a package folder to a share file, without opening a window.
    share: Option<(PathBuf, PathBuf)>,
    /// Draw the station with a sample of finds on the Hilltop, rather than the colony's own.
    sample_hilltop: bool,
    /// The hour to draw at, rather than midday.
    hour: Option<f32>,
    /// Write every sound to WAV files in this folder, without opening a window.
    sounds: Option<PathBuf>,
    /// Picture the window for review, and close.
    snap: Option<app::Snap>,
    /// Hill was started with no arguments at all, as a person opening it from Applications or
    /// the Start menu starts it, rather than by Desktop or for development: the sample colony
    /// arrives behind a card that says where the person's own colony comes from.
    welcome: bool,
}

fn main() -> Result<()> {
    match packaging(std::env::args_os().skip(1)) {
        Some(Packaging::TravelVersion) => {
            println!("{}", formiga_travel::TRAVEL_FORMAT_VERSION);
            return Ok(());
        }
        Some(Packaging::Icon(folder)) => return write_icon(folder),
        None => {}
    }
    // Before anything starts a thread: see `daylight::Clock`.
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let Some(args) = parse_args(std::env::args_os().skip(1))? else {
        print!("{USAGE}");
        return Ok(());
    };
    if args.show_folder {
        return show_packages_folder();
    }
    if !args.check.is_empty() {
        return check_packages(&args.check);
    }
    if let Some((folder, file)) = &args.share {
        return share_package(folder, file);
    }
    if let Some(folder) = &args.sounds {
        for line in audio::render_sounds(folder)? {
            println!("{line}");
        }
        println!("Wrote every sound to {}", folder.display());
        return Ok(());
    }
    // A window hosts one colony at a time: see `hosting`. Drawing to a file needs no window.
    let hosting = if args.render.is_none() {
        memories::Memories::folder().map(|data| hosting::take(&data))
    } else {
        None
    };
    let busy = matches!(hosting, Some(Err(hosting::Busy)));
    if busy && !matches!(args.source, Source::Trip(_)) {
        bail!("Formiga Hill is already open, with a colony visiting");
    }
    let arrival = arrive(args.source, busy)?;

    if let Some((area, path)) = args.render {
        let canvas = render::draw(
            area,
            &arrival.cast,
            args.at,
            args.hour,
            args.sample_hilltop,
            &args.packages,
        )?;
        photos::write(&path, &canvas, 3)?;
        println!("Drew the {area:?} to {}", path.display());
        return Ok(());
    }

    let scale = 3.0;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Formiga Hill")
            .with_inner_size([
                station::SCENE_WIDTH as f32 * scale,
                station::SCENE_HEIGHT as f32 * scale,
            ])
            .with_min_inner_size([station::SCENE_WIDTH as f32, station::SCENE_HEIGHT as f32])
            .with_icon({
                let picture = icon::at(256);
                let mut rgba = Vec::with_capacity(256 * 256 * 4);
                for y in 0..256 {
                    for x in 0..256 {
                        let pixel = picture.get(x, y);
                        rgba.extend([pixel.r, pixel.g, pixel.b, pixel.a]);
                    }
                }
                eframe::egui::IconData {
                    rgba,
                    width: 256,
                    height: 256,
                }
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Formiga Hill",
        options,
        Box::new(|cc| {
            // Community stories go in the packages folder, made ready so it is there to find.
            let folder = memories::Memories::folder().map(|data| story::shelf::folder(&data));
            if let Some(folder) = &folder {
                let _ = std::fs::create_dir_all(folder);
            }
            let library = story::Library::load(folder.as_deref(), &args.packages);
            let clock = match args.hour {
                Some(hour) => daylight::Clock::Held(hour),
                None => daylight::Clock::Local(offset),
            };
            let app = HillApp::new(cc, arrival, library, clock, args.snap);
            Ok(Box::new(if args.welcome { app.welcome() } else { app }))
        }),
    )
    .map_err(|error| anyhow::anyhow!("the Hill window could not open: {error}"))?;
    if let Some(why) = app::snap_failed() {
        bail!("{why}");
    }
    Ok(())
}

/// What the packaging scripts ask of Hill, ahead of every other option.
#[derive(Debug, PartialEq)]
enum Packaging {
    /// The newest travel version this Hill reads, which goes in the macOS bundle and the Windows
    /// registry for Desktop to find.
    TravelVersion,
    /// Hill's icon, for the bundle and the installer, written into the folder given, if one is.
    Icon(Option<PathBuf>),
}

/// What the packaging scripts ask for, if they do: only ever as the first argument, so the
/// same words anywhere else are not taken for them.
fn packaging(mut args: impl Iterator<Item = OsString>) -> Option<Packaging> {
    let first = args.next()?;
    if first == "--travel-version" {
        Some(Packaging::TravelVersion)
    } else if first == "--icon" {
        Some(Packaging::Icon(args.next().map(PathBuf::from)))
    } else {
        None
    }
}

/// Writes Hill's icon into `folder` for the packaging scripts, in every size and format they
/// need.
fn write_icon(folder: Option<PathBuf>) -> Result<()> {
    let folder = folder.context("--icon needs a folder to write the icon into")?;
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("FormigaHill.icns"), icon::icns())?;
    std::fs::write(folder.join("FormigaHill.ico"), icon::ico())?;
    std::fs::write(
        folder.join("FormigaHill.png"),
        photos::png(&icon::at(1024), 1),
    )?;
    println!("Wrote Hill's icon to {}", folder.display());
    Ok(())
}

fn parse_args(args: impl Iterator<Item = OsString>) -> Result<Option<Args>> {
    let mut args = args.peekable();
    // Desktop always says how it starts Hill, and so does every development and review option,
    // so only someone opening the app itself starts it with nothing at all.
    let welcome = args.peek().is_none();
    let mut source = None;
    let mut render = None;
    let mut at = None;
    let mut packages = Vec::new();
    let mut check = Vec::new();
    let mut show_folder = false;
    let mut share = None;
    let mut sample_hilltop = false;
    let mut hour = None;
    let mut sounds = None;
    let mut snap = None;
    let mut place = None;
    let mut card = None;
    let mut story = false;
    let mut chapter = None;
    let mut set_source = |next: Source| {
        if source.replace(next).is_some() {
            bail!("choose one of --sample, {LAUNCH_ARGUMENT}, or --from-save");
        }
        Ok(())
    };
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .map(PathBuf::from)
                .with_context(|| format!("{flag} needs a value"))
        };
        if let Some(&(flag, area)) = RENDERS.iter().find(|(flag, _)| arg == *flag) {
            render = Some((area, value(flag)?));
            continue;
        }
        match arg.to_str() {
            Some("--sample") => set_source(Source::Sample)?,
            Some(LAUNCH_ARGUMENT) => set_source(Source::Trip(value(LAUNCH_ARGUMENT)?))?,
            Some("--from-save") => set_source(Source::Save(value("--from-save")?))?,
            Some("--render-sounds") => sounds = Some(value("--render-sounds")?),
            Some("--snap") => snap = Some(value("--snap")?),
            Some("--place") => {
                let named = value("--place")?.to_string_lossy().into_owned();
                if !app::SNAP_PLACES.contains(&named.as_str()) {
                    bail!("--place needs one of {}", app::SNAP_PLACES.join(", "));
                }
                place = Some(named);
            }
            Some("--card") => {
                let named = value("--card")?.to_string_lossy().into_owned();
                if !app::SNAP_CARDS.contains(&named.as_str()) {
                    bail!("--card needs one of {}", app::SNAP_CARDS.join(", "));
                }
                card = Some(named);
            }
            Some("--story") => story = true,
            Some("--chapter") => {
                chapter = Some(
                    value("--chapter")?
                        .to_str()
                        .and_then(|text| text.parse::<usize>().ok())
                        .filter(|chapter| *chapter >= 1)
                        .context("--chapter is a number, from 1")?,
                );
            }
            Some("--at") => {
                let seconds = value("--at")?;
                at = Some(
                    seconds
                        .to_str()
                        .and_then(|text| text.parse::<f32>().ok())
                        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
                        .with_context(|| format!("--at needs seconds, not {seconds:?}"))?,
                );
            }
            Some("--sample-hilltop") => sample_hilltop = true,
            Some("--hour") => {
                let given = value("--hour")?;
                hour = Some(
                    given
                        .to_str()
                        .and_then(|text| text.parse::<f32>().ok())
                        .filter(|hour| (0.0..=24.0).contains(hour))
                        .with_context(|| {
                            format!("--hour needs an hour from 0 to 24, not {given:?}")
                        })?,
                );
            }
            Some("--package") => packages.push(value("--package")?),
            Some("--check-package") => check.push(value("--check-package")?),
            Some("--packages-folder") => show_folder = true,
            Some("--share-package") => {
                let folder = value("--share-package")?;
                share = Some((folder, value("--share-package")?));
            }
            Some("-h" | "--help") => return Ok(None),
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        }
    }
    if at.is_some() && render.is_none() && snap.is_none() {
        bail!("--at only goes with --snap or one of the --render options");
    }
    if chapter.is_some() && !matches!(card.as_deref(), Some("journal" | "writing" | "making")) {
        bail!("--chapter only goes with --card journal, --card writing or --card making");
    }
    if (place.is_some() || card.is_some() || story) && snap.is_none() {
        bail!("--place, --card and --story only go with --snap");
    }
    if snap.is_some() && render.is_some() {
        bail!("choose --snap or a --render option, not both");
    }
    // A card is pictured where it can be open: at its own place if none is named, and a place
    // it is never open at is refused rather than pictured without it.
    if let Some(card) = &card {
        let places = app::snap_card_places(card);
        match place.as_deref() {
            _ if places.is_empty() => {}
            None => place = Some(places[0].to_owned()),
            Some(named) if !places.contains(&named) => {
                bail!(
                    "the {card} card is only open at the {}",
                    places.join(" or the ")
                )
            }
            Some(_) => {}
        }
    }
    if story && place.as_deref() != Some("clubhouse") {
        bail!("--story only goes with --place clubhouse");
    }
    if story && card.as_deref() == Some("board") {
        bail!("the board is put away while a story is told");
    }
    let snap = snap.map(|path| app::Snap {
        path,
        at: at.unwrap_or(3.0),
        place,
        card,
        story,
        chapter,
    });
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
        at,
        packages,
        check,
        show_folder,
        share,
        sample_hilltop,
        hour,
        sounds,
        snap,
        welcome,
    }))
}

fn arrive(source: Source, busy: bool) -> Result<Arrival> {
    Ok(match source {
        Source::Sample => Arrival {
            cast: Cast::new(formiga_travel::sample::snapshot())?,
            visit: Visit::Rehearsal("Desktop's sample colony".to_owned()),
        },
        Source::Trip(dir) => {
            let (trip, cast) = trip::arrive(&dir, busy)?;
            Arrival {
                cast,
                visit: Visit::Trip(trip),
            }
        }
        Source::Save(path) => {
            let snapshot = project_save(
                &path,
                SessionId::generate().context("no randomness for a session id")?,
                OffsetDateTime::now_utc(),
            )?;
            Arrival {
                cast: Cast::new(snapshot)?,
                visit: Visit::Rehearsal("from a colony file".to_owned()),
            }
        }
    })
}

/// The colony file at `path` as it would travel: read only, and projected exactly as Desktop
/// projects a colony for a real trip. The session and the time are given, so a test can hold
/// them still.
fn project_save(
    path: &Path,
    session: SessionId,
    at: OffsetDateTime,
) -> Result<formiga_travel::TravelSnapshot> {
    let save = formiga_core::SaveStore::read_snapshot(path)
        .with_context(|| format!("could not read the colony file {}", path.display()))?;
    Ok(project_colony(&save, session, at, "a colony file")?)
}

/// Where community packages go, and how each one there fares.
fn show_packages_folder() -> Result<()> {
    let folder = memories::Memories::folder()
        .map(|data| story::shelf::folder(&data))
        .context("there is nowhere to keep packages on this computer")?;
    println!("Put story packages in:\n  {}", folder.display());
    let library = story::Library::load(Some(&folder), &[]);
    let found: Vec<_> = library
        .packages
        .iter()
        .zip(&library.origins)
        .filter(|(_, origin)| **origin == story::Origin::Folder)
        .collect();
    if found.is_empty() && library.problems.is_empty() {
        println!("Nothing there yet.");
    }
    for (package, _) in found {
        println!(
            "{} {} (\u{201c}{}\u{201d} by {}) loads",
            package.id, package.version, package.title, package.author
        );
    }
    for problem in &library.problems {
        println!("{problem}");
    }
    Ok(())
}

/// Checks package folders, or share files, as Hill would load them, and says what is wrong in
/// each.
fn check_packages(folders: &[PathBuf]) -> Result<()> {
    let mut failed = 0;
    for folder in folders {
        let label = folder.display().to_string();
        let files = if folder.is_file() {
            sharing::read(folder)
        } else {
            story::read_folder(folder)
        };
        match files.and_then(|files| story::load(&files, &label)) {
            Ok(package) => {
                println!(
                    "{} {} (\u{201c}{}\u{201d} by {}) loads:",
                    package.id, package.version, package.title, package.author
                );
                for story in &package.stories {
                    println!(
                        "  \u{201c}{}\u{201d}, for {} or more travellers",
                        story.title, story.min_cast
                    );
                    // A story written for the green is played in the Clubhouse; say how it is read.
                    if story.api == 1 {
                        println!("    written for content API 1, so played in the Clubhouse");
                        for (green, clubhouse) in &story.read_as {
                            println!("    \"{green}\" is read as \"{clubhouse}\"");
                        }
                    }
                }
                for adventure in &package.adventures {
                    println!("  \u{201c}{}\u{201d}, a Woods adventure", adventure.title);
                }
                for format in &package.competitions {
                    println!(
                        "  \u{201c}{}\u{201d}, a competition for the Fairground",
                        format.title
                    );
                }
            }
            Err(problem) => {
                failed += 1;
                println!("{problem}");
            }
        }
    }
    if failed > 0 {
        bail!("{failed} of {} packages would not load", folders.len());
    }
    Ok(())
}

/// Writes a package folder to one share file, once it loads as Hill would load it.
fn share_package(folder: &Path, file: &Path) -> Result<()> {
    let files = story::read_folder(folder)?;
    let inspection = sharing::inspect(files, &folder.display().to_string())?;
    sharing::export(&inspection.files, file)?;
    println!(
        "Wrote {} {} (\u{201c}{}\u{201d} by {}) to {}",
        inspection.summary.id,
        inspection.summary.version,
        inspection.summary.title,
        inspection.summary.author,
        file.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Desktop finds an installed Hill by what `formiga_travel::discovery` names, so the bundle
    /// and the installer must say exactly that.
    #[test]
    fn the_packaging_says_what_desktop_looks_for() {
        use formiga_travel::discovery::*;
        let packaging = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging");
        // Read as Git checks them out, which on Windows is with carriage returns.
        let read = |file: &str| {
            std::fs::read_to_string(packaging.join(file))
                .unwrap()
                .replace("\r\n", "\n")
        };
        let plist = read("macos/Info.plist");
        assert!(plist.contains(&format!(
            "<key>CFBundleIdentifier</key><string>{MACOS_BUNDLE_ID}</string>"
        )));
        assert!(plist.contains(&format!("<key>{MACOS_TRAVEL_VERSION_KEY}</key><integer>")));
        assert!(plist.contains("<key>CFBundleIconFile</key><string>FormigaHill</string>"));
        let installer = read("windows/FormigaHill.wxs");
        for value in [
            WINDOWS_PATH_VALUE,
            WINDOWS_VERSION_VALUE,
            WINDOWS_TRAVEL_VERSION_VALUE,
        ] {
            assert!(
                installer.contains(&format!(
                    "Key=\"{WINDOWS_REGISTRY_KEY}\"\n              Name=\"{value}\""
                )),
                "the installer does not write {value}"
            );
        }
    }

    fn parse(args: &[&str]) -> Result<Option<Args>> {
        parse_args(args.iter().map(OsString::from))
    }

    #[test]
    fn no_arguments_means_the_sample_colony() {
        let args = parse(&[]).unwrap().unwrap();
        assert!(matches!(args.source, Source::Sample));
        assert!(args.render.is_none());
    }

    #[test]
    fn only_hill_opened_on_its_own_says_where_a_colony_comes_from() {
        let welcome = |args: &[&str]| parse(args).unwrap().unwrap().welcome;
        assert!(welcome(&[]), "opened from Applications or the Start menu");
        assert!(!welcome(&["--sample"]), "the sample asked for by name");
        assert!(
            !welcome(&["--formiga-travel", "/trips/ab"]),
            "a trip from Desktop"
        );
        assert!(!welcome(&["--from-save", "colony.json"]));
        assert!(!welcome(&["--render-station", "out.png"]));
        assert!(!welcome(&["--snap", "a.png"]));
        assert!(!welcome(&["--check-package", "story"]));
        assert!(!welcome(&["--package", "story"]));
        assert!(!welcome(&["--packages-folder"]));
        assert!(!welcome(&["--render-sounds", "sounds"]));
        let args = parse(&["--sample"]).unwrap().unwrap();
        assert!(matches!(args.source, Source::Sample));
    }

    #[test]
    fn the_welcome_card_can_be_pictured_at_the_station() {
        let snap = parse(&["--snap", "a.png", "--card", "welcome"])
            .unwrap()
            .unwrap()
            .snap
            .unwrap();
        assert_eq!(snap.card.as_deref(), Some("welcome"));
        assert_eq!(snap.place.as_deref(), Some("station"));
        assert!(parse(&["--snap", "a.png", "--card", "welcome", "--place", "green"]).is_err());
    }

    #[test]
    fn a_trip_and_a_render_can_be_combined() {
        let args = parse(&[
            "--formiga-travel",
            "/trips/ab",
            "--render-station",
            "out.png",
        ])
        .unwrap()
        .unwrap();
        assert!(matches!(args.source, Source::Trip(ref path) if path == Path::new("/trips/ab")));
        assert_eq!(args.render, Some((Area::Station, PathBuf::from("out.png"))));
    }

    #[test]
    fn each_fairground_game_can_be_drawn_partway_through() {
        let args = parse(&["--render-sack-race", "race.png", "--at", "4"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.render,
            Some((Area::SackRace, PathBuf::from("race.png")))
        );
        assert_eq!(args.at, Some(4.0));
        let args = parse(&["--render-high-striker", "striker.png"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.render,
            Some((Area::HighStriker, PathBuf::from("striker.png")))
        );
        let args = parse(&["--render-hoopla", "hoopla.png", "--at", "20"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.render,
            Some((Area::Hoopla, PathBuf::from("hoopla.png")))
        );
        assert_eq!(args.at, Some(20.0));
        let args = parse(&["--render-tug-of-war", "tug.png", "--at", "8"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.render,
            Some((Area::TugOfWar, PathBuf::from("tug.png")))
        );
        assert_eq!(args.at, Some(8.0));
    }

    #[test]
    fn every_render_has_its_own_option_and_is_in_the_help() {
        for (i, (flag, area)) in RENDERS.iter().enumerate() {
            assert!(
                USAGE.contains(&format!("  {flag} <PNG>")),
                "{flag} is not in the help"
            );
            assert!(
                RENDERS[..i].iter().all(|(f, a)| f != flag && a != area),
                "{flag} is named twice"
            );
            let args = parse(&[flag, "out.png"]).unwrap().unwrap();
            assert_eq!(args.render, Some((*area, PathBuf::from("out.png"))));
        }
    }

    #[test]
    fn the_sounds_can_be_written_out_to_listen_to() {
        let args = parse(&["--render-sounds", "sounds"]).unwrap().unwrap();
        assert_eq!(args.sounds, Some(PathBuf::from("sounds")));
        assert!(args.render.is_none());
        assert!(parse(&["--render-sounds"]).is_err());
    }

    #[test]
    fn a_render_can_be_taken_partway_through_the_arrival() {
        let args = parse(&["--render-station", "out.png", "--at", "2.5"])
            .unwrap()
            .unwrap();
        assert_eq!(args.at, Some(2.5));
        assert!(parse(&["--at", "2.5"]).is_err());
        assert!(parse(&["--render-station", "out.png", "--at", "soon"]).is_err());
    }

    #[test]
    fn the_window_can_be_pictured_anywhere_with_any_card_open() {
        let args = parse(&[
            "--snap",
            "board.png",
            "--place",
            "clubhouse",
            "--card",
            "board",
            "--at",
            "5",
        ])
        .unwrap()
        .unwrap();
        let snap = args.snap.unwrap();
        assert_eq!(snap.path, PathBuf::from("board.png"));
        assert_eq!(snap.place.as_deref(), Some("clubhouse"));
        assert_eq!(snap.card.as_deref(), Some("board"));
        assert_eq!(snap.at, 5.0);
        assert_eq!(
            parse(&["--snap", "a.png"])
                .unwrap()
                .unwrap()
                .snap
                .unwrap()
                .at,
            3.0
        );
        assert!(parse(&["--snap", "a.png", "--place", "the moon"]).is_err());
        assert!(parse(&["--snap", "a.png", "--card", "menu"]).is_err());
        assert!(parse(&["--place", "green"]).is_err(), "only with --snap");
        assert!(parse(&["--snap", "a.png", "--render-green", "b.png"]).is_err());
    }

    #[test]
    fn a_card_is_pictured_where_it_can_be_open_and_nowhere_else() {
        let place = |args: &[&str]| parse(args).unwrap().unwrap().snap.unwrap().place;
        let board = place(&["--snap", "a.png", "--card", "board"]);
        assert_eq!(
            board.as_deref(),
            Some("clubhouse"),
            "its own place if none is named"
        );
        let journal = place(&["--snap", "a.png", "--card", "journal", "--place", "hilltop"]);
        assert_eq!(journal.as_deref(), Some("hilltop"));
        let album = place(&["--snap", "a.png", "--card", "album", "--place", "woods"]);
        assert_eq!(
            album.as_deref(),
            Some("woods"),
            "the album is open anywhere"
        );
        assert!(parse(&["--snap", "a.png", "--card", "plans", "--place", "green"]).is_err());
        assert!(
            parse(&["--snap", "a.png", "--story"]).is_err(),
            "a story is in the Clubhouse"
        );
        let both = [
            "--snap",
            "a.png",
            "--place",
            "clubhouse",
            "--story",
            "--card",
            "board",
        ];
        assert!(parse(&both).is_err(), "the board is away during a story");
    }

    #[test]
    fn two_sources_or_a_missing_path_are_refused() {
        assert!(parse(&["--sample", "--formiga-travel", "/trips/ab"]).is_err());
        assert!(parse(&["--formiga-travel"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--help"]).unwrap().is_none());
    }

    /// A scratch folder of the test's own, emptied first.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hill-main-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Why `args` were refused, which they must be.
    fn refused(args: &[&str]) -> String {
        parse(args)
            .err()
            .map(|error| error.to_string())
            .expect("refused")
    }

    #[test]
    fn desktops_launch_argument_yields_the_trip_it_names() {
        let args = parse(&[LAUNCH_ARGUMENT, "/trips/5eed"]).unwrap().unwrap();
        assert!(matches!(args.source, Source::Trip(ref dir) if dir == Path::new("/trips/5eed")));
        assert!(args.render.is_none() && args.snap.is_none() && !args.welcome);
        assert_eq!(LAUNCH_ARGUMENT, "--formiga-travel");
    }

    #[test]
    fn a_colony_file_and_package_checks_are_read_from_the_command_line() {
        let args = parse(&["--from-save", "colony.json"]).unwrap().unwrap();
        assert!(matches!(args.source, Source::Save(ref path) if path == Path::new("colony.json")));
        assert!(parse(&["--from-save"]).is_err());
        let args = parse(&["--check-package", "one", "--check-package", "two.share"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.check,
            [PathBuf::from("one"), PathBuf::from("two.share")]
        );
        assert!(parse(&["--check-package"]).is_err());
    }

    #[test]
    fn a_render_can_be_drawn_at_any_hour_of_the_day_and_no_other() {
        let args = parse(&["--render-woods", "woods.png", "--at", "12", "--hour", "21"])
            .unwrap()
            .unwrap();
        assert_eq!(args.render, Some((Area::Woods, PathBuf::from("woods.png"))));
        assert_eq!((args.at, args.hour), (Some(12.0), Some(21.0)));
        for hour in ["0", "24", "6.5"] {
            assert!(parse(&["--hour", hour]).is_ok(), "{hour} is an hour");
        }
        for hour in ["25", "-1", "dusk", "NaN"] {
            assert!(parse(&["--hour", hour]).is_err(), "{hour} is not an hour");
        }
        assert!(parse(&["--hour"]).is_err());
        for at in ["-1", "inf", "soon"] {
            let why = refused(&["--render-woods", "woods.png", "--at", at]);
            assert!(why.contains("--at needs seconds"), "{why}");
        }
    }

    #[test]
    fn an_unknown_option_is_refused_with_the_help() {
        let why = refused(&["--wat"]);
        assert!(why.contains("\"--wat\""), "{why}");
        assert!(why.ends_with(USAGE), "{why}");
        assert!(refused(&["--sample", "stray"]).ends_with(USAGE));
    }

    #[test]
    fn the_packaging_scripts_are_answered_only_from_the_first_argument() {
        let ask = |args: &[&str]| packaging(args.iter().map(OsString::from));
        assert_eq!(ask(&["--travel-version"]), Some(Packaging::TravelVersion));
        assert_eq!(
            ask(&["--travel-version", "--icon"]),
            Some(Packaging::TravelVersion)
        );
        assert_eq!(
            ask(&["--icon", "out"]),
            Some(Packaging::Icon(Some(PathBuf::from("out"))))
        );
        assert_eq!(ask(&["--icon"]), Some(Packaging::Icon(None)));
        assert_eq!(ask(&[]), None);
        assert_eq!(ask(&["--sample", "--travel-version"]), None);
        assert_eq!(ask(&["--sample", "--icon", "out"]), None);
        // Anywhere but first they are nothing the rest of Hill knows.
        assert!(refused(&["--sample", "--travel-version"]).ends_with(USAGE));
        assert!(refused(&["--sample", "--icon", "out"]).ends_with(USAGE));
        assert!(write_icon(None).is_err(), "a folder is needed");
    }

    #[test]
    fn a_colony_file_travels_exactly_as_desktop_would_send_it() {
        use formiga_travel::sample;
        let dir = scratch("from-save");
        let path = dir.join("colony.json");
        formiga_core::SaveStore::new(&path)
            .save(&sample::colony(3))
            .unwrap();
        let session = SessionId::parse(sample::SESSION).unwrap();
        let projected = project_save(&path, session, sample::MADE).unwrap();
        // Desktop reads a colony file as it restores one, taking off anything worn that the
        // colony's scrapbook hasn't unlocked; the sample colony dresses two companions without
        // unlocking what they wear, so those come off, and nothing else changes.
        let mut expected = formiga_travel::TravelSnapshot {
            desktop_version: "a colony file".to_owned(),
            ..sample::snapshot()
        };
        let worn = expected
            .travelers
            .iter()
            .filter(|traveler| traveler.accessory.is_some())
            .count();
        assert_eq!(worn, 2, "the sample has someone dressed and someone pinned");
        for traveler in &mut expected.travelers {
            traveler.accessory = None;
        }
        assert_eq!(projected, expected);
        assert!(Cast::new(projected).is_ok());
        assert!(
            project_save(
                &dir.join("missing.json"),
                SessionId::generate().unwrap(),
                sample::MADE
            )
            .is_err()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A trip as travel version 5 wrote it, cut down from `formiga-travel`'s own
    /// `snapshot-v5.json` to two travellers: one with a recipe and something worn, and one from
    /// before recipes. Frozen here, so a change to how Hill reads an older trip shows.
    const SNAPSHOT_V5: &str = r#"{
  "format": "formiga.travel.snapshot",
  "version": 5,
  "min_reader_version": 1,
  "session_id": "5eed5eed5eed5eed5eed5eed5eed5eed",
  "colony_id": "601f2179d0b45400",
  "created_at_utc": "2026-10-02T09:30:00Z",
  "desktop_version": "0.66.1",
  "capabilities": ["visit_record", "souvenirs"],
  "accepts_souvenirs": [
    "picnic_ribbon", "pressed_daisy", "well_penny", "oak_acorn", "swing_feather",
    "chest_marble", "fair_ticket", "race_rosette", "striker_bell", "hoopla_teddy", "tug_rope"
  ],
  "travelers": [
    {
      "id": "876a55d6a17221ef",
      "name": "Mochi",
      "role": {"kind": "adult"},
      "born_at_utc": "2026-10-02T09:30:00Z",
      "appearance": {
        "design": {"parts": "0300000b0a0903030000bcd18f7ad2870000007c"},
        "family": "blob",
        "logical_size": 37,
        "body_width": 25,
        "body_height": 13,
        "head_ratio": 0.67133456,
        "roundness": 0.78652555,
        "leg_length": 5,
        "foot_size": 5,
        "head_appendages": {"style": "droop", "size": 3},
        "tail": {"style": "none", "length": 5},
        "face": {
          "eye_shape": "round",
          "eye_size": 2,
          "eye_spacing": 5,
          "vertical_offset": 0,
          "pupil": "wide",
          "highlight": "double",
          "brow": "soft",
          "mouth": "beak",
          "cheek": "none"
        },
        "forelimbs": {
          "style": "soft_nub",
          "length": 4,
          "thickness": 1,
          "tip": "round",
          "rest_pose": "together"
        },
        "effect_motif": "leaf",
        "palette_index": 5,
        "pattern": "patches",
        "pattern_density": 0.6479941,
        "marking_seed": "465568b953d40634",
        "gait_bob": 0.7812724,
        "face_signature": 458422880
      },
      "stature_percent": 97,
      "scale_percent": 100,
      "character": {
        "temperament": "wallflower",
        "axes": {
          "social": 0.16553476,
          "energy": 0.24074684,
          "boldness": 0.21814881,
          "playfulness": 0.5044405,
          "curiosity": 0.3669005,
          "feistiness": 0.15188114,
          "impulsiveness": 0.3082601,
          "suspicion": 0.7024214,
          "affection": 0.5133249
        },
        "traits": ["Shy", "Wary", "Mellow"],
        "trait_ids": ["shy", "wary", "mellow"],
        "phrase": "A shy wallflower"
      },
      "motion": {
        "activity": 0.38056013,
        "playfulness": 0.5535524,
        "boldness": 0.2854265,
        "celebration": "hop"
      },
      "habits": ["waves_hello", "circles_before_naps"],
      "accessory": {
        "item": {"kind": "worn", "item": "flower_crown"},
        "colors": {
          "outline": [42, 36, 51],
          "deep": [49, 18, 151],
          "body": [75, 27, 228],
          "light": [172, 156, 221],
          "accent": [240, 60, 246]
        }
      }
    },
    {
      "id": "66cd447617ba3991",
      "name": "Button",
      "role": {"kind": "adult"},
      "born_at_utc": "2026-10-02T09:30:00Z",
      "appearance": {
        "family": "hopper",
        "logical_size": 38,
        "body_width": 22,
        "body_height": 24,
        "head_ratio": 0.7581961,
        "roundness": 0.85026443,
        "leg_length": 3,
        "foot_size": 5,
        "head_appendages": {"style": "leaf", "size": 5},
        "tail": {"style": "none", "length": 8},
        "face": {
          "eye_shape": "soft_square",
          "eye_size": 2,
          "eye_spacing": 4,
          "vertical_offset": 0,
          "pupil": "spark",
          "highlight": "diagonal",
          "brow": "bold",
          "mouth": "cat",
          "cheek": "none"
        },
        "forelimbs": {
          "style": "mitten_arm",
          "length": 7,
          "thickness": 1,
          "tip": "mitten",
          "rest_pose": "folded"
        },
        "effect_motif": "dot",
        "palette_index": 11,
        "pattern": "patches",
        "pattern_density": 0.6628492,
        "marking_seed": "3c4238ab9cada22a",
        "gait_bob": 0.61300963,
        "face_signature": 2024248359
      },
      "stature_percent": 101,
      "scale_percent": 100,
      "character": {
        "temperament": "explorer",
        "axes": {
          "social": 0.63182133,
          "energy": 0.7012922,
          "boldness": 0.82865894,
          "playfulness": 0.4540892,
          "curiosity": 1.0,
          "feistiness": 0.5,
          "impulsiveness": 0.5,
          "suspicion": 0.5,
          "affection": 0.5
        },
        "traits": ["Curious", "Brave", "Energetic"],
        "trait_ids": ["curious", "brave", "energetic"],
        "phrase": "A curious explorer"
      },
      "motion": {
        "activity": 0.72596914,
        "playfulness": 0.51327133,
        "boldness": 0.8043601,
        "celebration": "twirl"
      }
    }
  ],
  "relationships": [
    {
      "a": "66cd447617ba3991",
      "b": "876a55d6a17221ef",
      "affinity": "high",
      "familiarity": "high",
      "playfulness": "low",
      "avoidance": "low"
    }
  ],
  "presentation": {"reduce_motion": false, "theme": "system", "text_scale_percent": 100}
}"#;

    /// Hill's book as version 1 kept it, for the colony in `SNAPSHOT_V5`: before anything on the
    /// Hilltop grew.
    const MEMORIES_V1: &str = r#"{
  "version": 1,
  "colonies": {
    "601f2179d0b45400": {
      "stories": ["hill/picnic"],
      "souvenirs": ["picnic_ribbon"],
      "visits": 4,
      "satchel": {"bluebell_bulb": 2},
      "hilltop": {"3": "bluebell_bulb", "9": "acorn_stash", "12": "geode"},
      "finds": {"geode": {"count": 1, "first_by": "7"}}
    }
  }
}"#;

    #[test]
    fn a_trip_from_an_older_desktop_finds_an_older_hills_book_as_it_was_left() {
        let root = scratch("older-files");
        let trip = root.join("5eed5eed5eed5eed5eed5eed5eed5eed");
        std::fs::create_dir_all(&trip).unwrap();
        std::fs::write(trip.join(formiga_travel::SNAPSHOT_FILE), SNAPSHOT_V5).unwrap();
        let data = root.join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("memories.json"), MEMORIES_V1).unwrap();

        let arrival = arrive(Source::Trip(trip.clone()), false).unwrap();
        assert!(matches!(arrival.visit, Visit::Trip(_)));
        let snapshot = &arrival.cast.snapshot;
        assert_eq!(snapshot.version, 5);
        let names: Vec<_> = arrival
            .cast
            .members
            .iter()
            .map(|member| member.name.as_str())
            .collect();
        assert_eq!(names, ["Mochi", "Button"]);
        assert!(
            trip.join(formiga_travel::ACK_FILE).exists(),
            "the trip is answered"
        );

        let book = memories::Memories::open(Some(&data), &snapshot.colony_id);
        assert_eq!(book.trouble(), None);
        let colony = book.colony();
        assert_eq!(colony.visits, 4);
        assert!(colony.stories.contains("hill/picnic"));
        assert_eq!(colony.souvenirs, ["picnic_ribbon"]);
        assert_eq!(colony.satchel["bluebell_bulb"], 2);
        assert_eq!(colony.hilltop.len(), 3);
        assert_eq!(colony.hilltop[&3], hilltop::Standing::from("bluebell_bulb"));
        assert!(colony.hilltop.values().all(|standing| !standing.growing()));
        assert_eq!(colony.finds["geode"].count, 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
