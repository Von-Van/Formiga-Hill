//! Formiga Hill: somewhere the colony goes to spend time together.

mod actor;
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
mod sheet;
mod station;
mod story;
mod track;
mod trip;
mod woods;

use anyhow::{Context, Result, bail};
use app::{Arrival, HillApp, Visit};
use cast::Cast;
use formiga_travel::{LAUNCH_ARGUMENT, SessionId, project_colony};
use render::Area;
use std::ffi::OsString;
use std::path::PathBuf;
use time::OffsetDateTime;

const USAGE: &str = "\
Formiga Hill

Usage: formiga-hill [--sample | --formiga-travel <TRIP DIRECTORY> | --from-save <FILE>]
                    [--render-station <PNG> | --render-green <PNG> | …] [--at <SECONDS>]

  --sample                 Arrive with Desktop's made-up sample colony (the default)
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
  --render-sounds <FOLDER> Write every sound and a minute of every piece of music to WAV files in
                           the folder, to listen to without a window
  --package <FOLDER>       Load a story package beside Hill's own (for authors); repeatable
  --check-package <FOLDER> Check a story package and say what is wrong, without opening a window
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
  --card <CARD>            With --snap: open the notices, departures, board, shelf, dress-up,
                           journal, plans, album or sound card, at its own place if none is given
  --story                  With --snap in the Clubhouse: begin the first story on the shelves
";

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
    /// Draw the station with a sample of finds on the Hilltop, rather than the colony's own.
    sample_hilltop: bool,
    /// The hour to draw at, rather than midday.
    hour: Option<f32>,
    /// Write every sound to WAV files in this folder, without opening a window.
    sounds: Option<PathBuf>,
    /// Picture the window for review, and close.
    snap: Option<app::Snap>,
}

fn main() -> Result<()> {
    // For the packaging scripts: the newest travel version this Hill reads, which goes in the
    // macOS bundle and the Windows registry for Desktop to find.
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--travel-version")
    {
        println!("{}", formiga_travel::TRAVEL_FORMAT_VERSION);
        return Ok(());
    }
    // For the packaging scripts too: Hill's icon, for the bundle and the installer.
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--icon")
    {
        let folder = std::env::args_os()
            .nth(2)
            .map(PathBuf::from)
            .context("--icon needs a folder to write the icon into")?;
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("FormigaHill.icns"), icon::icns())?;
        std::fs::write(folder.join("FormigaHill.ico"), icon::ico())?;
        std::fs::write(
            folder.join("FormigaHill.png"),
            photos::png(&icon::at(1024), 1),
        )?;
        println!("Wrote Hill's icon to {}", folder.display());
        return Ok(());
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
            let app = HillApp::new(cc, arrival, library, clock);
            Ok(Box::new(match args.snap {
                Some(snap) => app.snap(snap),
                None => app,
            }))
        }),
    )
    .map_err(|error| anyhow::anyhow!("the Hill window could not open: {error}"))?;
    if let Some(why) = app::snap_failed() {
        bail!("{why}");
    }
    Ok(())
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>> {
    let mut source = None;
    let mut render = None;
    let mut at = None;
    let mut packages = Vec::new();
    let mut check = Vec::new();
    let mut show_folder = false;
    let mut sample_hilltop = false;
    let mut hour = None;
    let mut sounds = None;
    let mut snap = None;
    let mut place = None;
    let mut card = None;
    let mut story = false;
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
        match arg.to_str() {
            Some("--sample") => set_source(Source::Sample)?,
            Some(LAUNCH_ARGUMENT) => set_source(Source::Trip(value(LAUNCH_ARGUMENT)?))?,
            Some("--from-save") => set_source(Source::Save(value("--from-save")?))?,
            Some("--render-station") => {
                render = Some((Area::Station, value("--render-station")?));
            }
            Some("--render-green") => render = Some((Area::Green, value("--render-green")?)),
            Some("--render-clubhouse") => {
                render = Some((Area::Clubhouse, value("--render-clubhouse")?));
            }
            Some("--render-fairground") => {
                render = Some((Area::Fairground, value("--render-fairground")?));
            }
            Some("--render-hide-and-seek") => {
                render = Some((Area::HideAndSeek, value("--render-hide-and-seek")?));
            }
            Some("--render-sack-race") => {
                render = Some((Area::SackRace, value("--render-sack-race")?));
            }
            Some("--render-high-striker") => {
                render = Some((Area::HighStriker, value("--render-high-striker")?));
            }
            Some("--render-hoopla") => render = Some((Area::Hoopla, value("--render-hoopla")?)),
            Some("--render-tug-of-war") => {
                render = Some((Area::TugOfWar, value("--render-tug-of-war")?));
            }
            Some("--render-woods") => render = Some((Area::Woods, value("--render-woods")?)),
            Some("--render-hilltop") => {
                render = Some((Area::Hilltop, value("--render-hilltop")?));
            }
            Some("--render-finds") => render = Some((Area::Finds, value("--render-finds")?)),
            Some("--render-growing") => {
                render = Some((Area::Growing, value("--render-growing")?));
            }
            Some("--render-plans") => render = Some((Area::Plans, value("--render-plans")?)),
            Some("--render-building") => {
                render = Some((Area::Building, value("--render-building")?));
            }
            Some("--render-fishing") => {
                render = Some((Area::Fishing, value("--render-fishing")?));
            }
            Some("--render-fish") => render = Some((Area::Fish, value("--render-fish")?)),
            Some("--render-meadow") => {
                render = Some((Area::Meadow, value("--render-meadow")?));
            }
            Some("--render-bugs") => render = Some((Area::Bugs, value("--render-bugs")?)),
            Some("--render-bug-hunt") => {
                render = Some((Area::BugHunt, value("--render-bug-hunt")?));
            }
            Some("--render-hedgerow") => {
                render = Some((Area::Hedgerow, value("--render-hedgerow")?));
            }
            Some("--render-produce") => {
                render = Some((Area::Produce, value("--render-produce")?));
            }
            Some("--render-expedition") => {
                render = Some((Area::Expedition, value("--render-expedition")?));
            }
            Some("--render-falls") => render = Some((Area::Falls, value("--render-falls")?)),
            Some("--render-track") => render = Some((Area::Track, value("--render-track")?)),
            Some("--render-treasure") => {
                render = Some((Area::Treasure, value("--render-treasure")?));
            }
            Some("--render-landmarks") => {
                render = Some((Area::Landmarks, value("--render-landmarks")?));
            }
            Some("--render-sovereign") => {
                render = Some((Area::Sovereign, value("--render-sovereign")?));
            }
            Some("--render-story") => render = Some((Area::Story, value("--render-story")?)),
            Some("--render-reactions") => {
                render = Some((Area::Reactions, value("--render-reactions")?));
            }
            Some("--render-costumes") => {
                render = Some((Area::Costumes, value("--render-costumes")?));
            }
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
            Some("-h" | "--help") => return Ok(None),
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        }
    }
    if at.is_some() && render.is_none() && snap.is_none() {
        bail!("--at only goes with --snap or one of the --render options");
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
    });
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
        at,
        packages,
        check,
        show_folder,
        sample_hilltop,
        hour,
        sounds,
        snap,
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
            // Read-only, and projected exactly as Desktop projects a colony for a real trip.
            let save = formiga_core::SaveStore::read_snapshot(&path)
                .with_context(|| format!("could not read the colony file {}", path.display()))?;
            let snapshot = project_colony(
                &save,
                SessionId::generate().context("no randomness for a session id")?,
                OffsetDateTime::now_utc(),
                "a colony file",
            )?;
            Arrival {
                cast: Cast::new(snapshot)?,
                visit: Visit::Rehearsal("from a colony file".to_owned()),
            }
        }
    })
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

/// Checks package folders as Hill would load them, and says what is wrong in each.
fn check_packages(folders: &[PathBuf]) -> Result<()> {
    let mut failed = 0;
    for folder in folders {
        let label = folder.display().to_string();
        match story::read_folder(folder).and_then(|files| story::load(&files, &label)) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
}
