//! Formiga Hill: somewhere the colony goes to spend time together.

mod actor;
mod app;
mod cast;
mod character;
mod cues;
mod dice;
mod fairground;
mod finds;
mod font;
mod green;
mod hilltop;
mod keepsake_art;
mod kit;
mod materials;
mod memories;
mod paint;
mod playground;
mod sheet;
mod station;
mod story;
mod trip;
mod woods;

use anyhow::{Context, Result, bail};
use app::{Arrival, HillApp, Visit};
use cast::Cast;
use formiga_art::Canvas;
use formiga_travel::{LAUNCH_ARGUMENT, SessionId, project_colony};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
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
  --render-fairground <PNG>
                           Draw the Fairground to a PNG and exit without opening a window
  --render-hide-and-seek <PNG>
                           Draw a game of hide-and-seek at the Fairground, --at seconds into
                           the search
  --render-woods <PNG>     Draw a rummage in the Woods, --at seconds into it
  --render-hilltop <PNG>   Draw the Hilltop with a sample of finds placed on it
  --render-finds <PNG>     Draw every find's icon and Hilltop piece on one sheet, for review
  --render-reactions <PNG> Draw everyone answering a pat, a snack and a toy, for review
  --render-story <PNG>     Draw a story on the green --at seconds after it starts, reading each
                           line for 2.5 seconds and taking the first choice
  --package <FOLDER>       Load a story package beside Hill's own (for authors); repeatable
  --check-package <FOLDER> Check a story package and say what is wrong, without opening a window
  --at <SECONDS>           Draw that far into the arrival, or into free play on the green or
                           at the Fairground
";

enum Source {
    Sample,
    Trip(PathBuf),
    Save(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Area {
    Station,
    Green,
    Fairground,
    /// The Fairground, a game of hide-and-seek under way.
    HideAndSeek,
    Woods,
    Hilltop,
    /// Not an area: the review sheet of every find.
    Finds,
    /// Not an area: the review sheet of everyone's reactions.
    Reactions,
    /// The green, a story under way on it.
    Story,
}

struct Args {
    source: Source,
    render: Option<(Area, PathBuf)>,
    at: Option<f32>,
    /// Package folders to load beside Hill's own, for authors.
    packages: Vec<PathBuf>,
    /// Check these package folders and report, without opening a window.
    check: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let Some(args) = parse_args(std::env::args_os().skip(1))? else {
        print!("{USAGE}");
        return Ok(());
    };
    if !args.check.is_empty() {
        return check_packages(&args.check);
    }
    let arrival = arrive(args.source)?;

    if let Some((area, path)) = args.render {
        let canvas = match area {
            Area::Station => {
                let (journey, now) = match args.at {
                    Some(seconds) => (station::Journey::Arriving { since: 0.0 }, seconds),
                    None => (station::Journey::Here, 0.0),
                };
                // The display case shows what this colony has kept, as Hill remembers it.
                let memories = memories::Memories::open(
                    memories::Memories::folder().as_deref(),
                    &arrival.cast.snapshot.colony_id,
                );
                let mut station =
                    station::Station::new(&arrival.cast, journey, &memories.colony().souvenirs);
                station.show_hilltop(&memories.colony().hilltop);
                station.compose(now)
            }
            Area::Green => {
                // Free play, run forward as the window would run it.
                let until = args.at.unwrap_or(20.0);
                let mut green = green::open(&arrival.cast, 0.0);
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    green.tick(&arrival.cast, now);
                }
                green.compose(now)
            }
            Area::Fairground => {
                let until = args.at.unwrap_or(20.0);
                let (mut fairground, _) = fairground::open(&arrival.cast, 0.0);
                fairground::show_hilltop(&mut fairground, &sample_arrangement());
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    fairground.tick(&arrival.cast, now);
                }
                fairground.compose(now)
            }
            Area::HideAndSeek => hiding_moment(&arrival.cast, args.at.unwrap_or(4.0)),
            Area::Woods => woods_moment(&arrival.cast, args.at.unwrap_or(12.0)),
            Area::Hilltop => {
                let until = args.at.unwrap_or(20.0);
                let mut hilltop = hilltop::open(&arrival.cast, 0.0, &sample_arrangement());
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    hilltop.tick(&arrival.cast, now);
                }
                hilltop.compose(now)
            }
            Area::Finds => finds_sheet(),
            Area::Reactions => sheet::reactions(&arrival.cast),
            Area::Story => {
                let library = story::Library::load(&args.packages);
                story_moment(
                    &arrival.cast,
                    story_to_draw(&library)?,
                    args.at.unwrap_or(12.0),
                )?
            }
        };
        write_png(&path, &canvas, 3)?;
        println!("Drew the {area:?} to {}", path.display());
        return Ok(());
    }

    let scale = 3.0;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Formiga Hill")
            .with_inner_size([
                station::SCENE_WIDTH as f32 * scale,
                station::SCENE_HEIGHT as f32 * scale + 40.0,
            ])
            .with_min_inner_size([station::SCENE_WIDTH as f32, station::SCENE_HEIGHT as f32]),
        ..Default::default()
    };
    eframe::run_native(
        "Formiga Hill",
        options,
        Box::new(|cc| {
            let library = story::Library::load(&args.packages);
            Ok(Box::new(HillApp::new(cc, arrival, library)))
        }),
    )
    .map_err(|error| anyhow::anyhow!("the Hill window could not open: {error}"))
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>> {
    let mut source = None;
    let mut render = None;
    let mut at = None;
    let mut packages = Vec::new();
    let mut check = Vec::new();
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
            Some("--render-fairground") => {
                render = Some((Area::Fairground, value("--render-fairground")?));
            }
            Some("--render-hide-and-seek") => {
                render = Some((Area::HideAndSeek, value("--render-hide-and-seek")?));
            }
            Some("--render-woods") => render = Some((Area::Woods, value("--render-woods")?)),
            Some("--render-hilltop") => {
                render = Some((Area::Hilltop, value("--render-hilltop")?));
            }
            Some("--render-finds") => render = Some((Area::Finds, value("--render-finds")?)),
            Some("--render-story") => render = Some((Area::Story, value("--render-story")?)),
            Some("--render-reactions") => {
                render = Some((Area::Reactions, value("--render-reactions")?));
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
            Some("--package") => packages.push(value("--package")?),
            Some("--check-package") => check.push(value("--check-package")?),
            Some("-h" | "--help") => return Ok(None),
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        }
    }
    if at.is_some() && render.is_none() {
        bail!("--at only goes with one of the --render options");
    }
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
        at,
        packages,
        check,
    }))
}

fn arrive(source: Source) -> Result<Arrival> {
    Ok(match source {
        Source::Sample => Arrival {
            cast: Cast::new(formiga_travel::sample::snapshot())?,
            visit: Visit::Rehearsal("Desktop's sample colony".to_owned()),
        },
        Source::Trip(dir) => {
            let (trip, cast) = trip::arrive(&dir)?;
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

/// The last story loaded, which is the author's own when they name a package. A package that
/// would not load is reported, rather than drawing one of Hill's own stories in its place.
fn story_to_draw(library: &story::Library) -> Result<&story::Story> {
    if !library.problems.is_empty() {
        let problems: Vec<String> = library.problems.iter().map(ToString::to_string).collect();
        bail!("the story would not load:\n{}", problems.join("\n"));
    }
    library
        .stories()
        .last()
        .map(|(_, story)| story)
        .context("there is no story to play")
}

/// A story `at` seconds in, with each line read after two and a half seconds and the first choice
/// always taken, for seeing how it is staged.
fn story_moment(cast: &Cast, chosen: &story::Story, at: f32) -> Result<Canvas> {
    // The colony arrives and settles on the green first, as it would before anyone opens a story.
    const SETTLE: f32 = 12.0;
    let mut green = green::open(cast, 0.0);
    let mut now = 0.0;
    while now < SETTLE {
        now += 1.0 / 30.0;
        green.tick(cast, now);
    }
    let mut director = story::Director::new(chosen.clone(), cast, 1).map_err(anyhow::Error::msg)?;
    green.reserve(director.players());
    let at = SETTLE + at;
    let mut shown_since = None;
    while now < at && !director.finished() {
        now += 1.0 / 30.0;
        green.tick(cast, now);
        director.run(&mut green, cast, now);
        let speaker = director
            .shown()
            .and_then(|shown| shown.speaker.as_ref().map(|(id, _)| *id));
        green.set_speaker(speaker);
        if director.shown().is_some() {
            let since = *shown_since.get_or_insert(now);
            if now - since > 2.5 && now + 1.0 / 30.0 < at {
                director.read_on();
                shown_since = None;
            }
        }
        if !director.choices().is_empty() {
            director.choose(0);
        }
    }
    if let Some(shown) = director.shown() {
        let speaker = shown.speaker.as_ref().map_or("", |(_, name)| name.as_str());
        println!(
            "On show: {speaker}{}{}",
            if speaker.is_empty() { "" } else { ": " },
            shown.text
        );
    }
    Ok(green.compose(now))
}

/// A game of hide-and-seek `at` seconds into the search, once the colony has settled at the
/// Fairground and hidden.
fn hiding_moment(cast: &Cast, at: f32) -> Canvas {
    const SETTLE: f32 = 12.0;
    let (mut ground, mut game) = fairground::open(cast, 0.0);
    let mut now = 0.0;
    let mut seeking_since = None;
    while seeking_since.is_none_or(|since| now < since + at) && now < 300.0 {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        game.tick(&mut ground, now);
        if now >= SETTLE && game.phase() == fairground::Phase::Ready && seeking_since.is_none() {
            game.start(&mut ground, None, now);
        }
        if let fairground::Phase::Seeking { since } = game.phase() {
            seeking_since.get_or_insert(since);
        }
        for event in game.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.compose(now)
}

/// An outing to the Woods with the first two travellers, `at` seconds in, played by a steady
/// hand: it searches the spots in turn and catches each moment as the marker crosses the gold.
fn woods_moment(cast: &Cast, at: f32) -> Canvas {
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut glade = woods::open(cast, &party, 0.0);
    let outset = woods::rummage::Outset {
        party,
        drought: 0,
        close_pair: false,
        influence: woods::influence(&sample_arrangement()),
        seed: 5,
    };
    let mut outing = woods::rummage::Rummage::new(&mut glade, outset, |_| false, 0.0);
    let mut now = 0.0;
    let mut next = 0;
    while now < at {
        now += 1.0 / 30.0;
        glade.tick(cast, now);
        outing.tick(&mut glade, now);
        match outing.phase() {
            woods::rummage::Phase::Exploring if next < woods::SPOTS.len() => {
                outing.choose(&mut glade, next, now);
                next += 1;
            }
            woods::rummage::Phase::Catching(_) if outing.on_the_gold(now) && now + 0.5 < at => {
                outing.strike(&mut glade, now);
            }
            _ => {}
        }
        for event in outing.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    let mut scene = glade.compose(now);
    outing.draw(&mut scene, now);
    scene
}

/// A spread of finds over the Hilltop, with some spots left open, for seeing it lived in.
fn sample_arrangement() -> hilltop::Arrangement {
    let picks = [
        (0, "weathervane"),
        (2, "brass_lens"),
        (4, "sun_coin"),
        (5, "acorn_stash"),
        (6, "wild_berries"),
        (8, "skimming_stone"),
        (9, "fallen_star"),
        (11, "old_nest"),
        (13, "wooden_duck"),
        (14, "smooth_pebble"),
        (15, "tangled_kite"),
        (17, "lost_lantern"),
    ];
    picks
        .into_iter()
        .map(|(spot, id)| (spot, id.to_owned()))
        .collect()
}

/// Every find on one sheet: its icon in the corner, its Hilltop piece standing on a patch of
/// summit grass, a row for each kind.
fn finds_sheet() -> Canvas {
    const CELL: (i32, i32) = (56, 70);
    let columns = 7;
    let mut sheet = Canvas::new((CELL.0 * columns) as u32, (CELL.1 * 4) as u32);
    for (row, kind) in finds::Kind::ALL.into_iter().enumerate() {
        let of_kind = finds::CATALOGUE.iter().filter(|find| find.kind == kind);
        for (column, find) in of_kind.enumerate() {
            let (left, top) = (column as i32 * CELL.0, row as i32 * CELL.1);
            for y in 0..CELL.1 {
                let t = y as f32 / CELL.1 as f32;
                let color = if y < CELL.1 - 12 {
                    paint::mix(paint::rgb(0xd6ecf2), paint::rgb(0xf6e8cf), t)
                } else {
                    paint::mix(paint::rgb(0x86bb7c), paint::rgb(0x639858), t)
                };
                sheet.fill_rect(left, top + y, CELL.0, 1, color);
            }
            sheet.fill_rect(left + CELL.0 - 1, top, 1, CELL.1, paint::rgb(0x9ab0a0));
            sheet.fill_rect(left, top + CELL.1 - 1, CELL.0, 1, paint::rgb(0x9ab0a0));
            let piece = finds::art::piece(find.id);
            let base = (left + CELL.0 / 2, top + CELL.1 - 6);
            paint::blit(
                &mut sheet,
                &piece.sprite,
                base.0 - piece.anchor.0,
                base.1 - piece.anchor.1,
            );
            paint::blit(&mut sheet, &finds::art::icon(find.id), left + 2, top + 2);
        }
    }
    sheet
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

/// Writes the canvas scaled up by whole pixels.
fn write_png(path: &Path, canvas: &Canvas, scale: u32) -> Result<()> {
    let (width, height) = (canvas.width() * scale, canvas.height() * scale);
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let pixel = canvas.get((x / scale) as i32, (y / scale) as i32);
            data.extend([pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }
    let file =
        File::create(path).with_context(|| format!("could not create {}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_render_can_be_taken_partway_through_the_arrival() {
        let args = parse(&["--render-station", "out.png", "--at", "2.5"])
            .unwrap()
            .unwrap();
        assert_eq!(args.at, Some(2.5));
        assert!(parse(&["--at", "2.5"]).is_err());
        assert!(parse(&["--render-station", "out.png", "--at", "soon"]).is_err());
    }

    #[test]
    fn a_package_that_would_not_load_is_reported_rather_than_drawn_around() {
        let library = story::Library::load(&[PathBuf::from("/no/such/package.formiga-hill")]);
        let problem = story_to_draw(&library).unwrap_err().to_string();
        assert!(problem.contains("package.formiga-hill"), "{problem}");
        assert!(story_to_draw(&story::Library::load(&[])).is_ok());
    }

    #[test]
    fn two_sources_or_a_missing_path_are_refused() {
        assert!(parse(&["--sample", "--formiga-travel", "/trips/ab"]).is_err());
        assert!(parse(&["--formiga-travel"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--help"]).unwrap().is_none());
    }
}
