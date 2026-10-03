//! Formiga Hill: somewhere the colony goes to spend time together.

mod actor;
mod app;
mod cast;
mod character;
mod clearing;
mod clubhouse;
mod cues;
mod daylight;
mod dice;
mod fairground;
mod finds;
mod fishing;
mod font;
mod green;
mod hilltop;
mod hosting;
mod keepsake_art;
mod kit;
mod materials;
mod meadow;
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
  --render-clubhouse <PNG> Draw the Clubhouse to a PNG and exit without opening a window
  --render-fairground <PNG>
                           Draw the Fairground to a PNG and exit without opening a window
  --render-hide-and-seek <PNG>
                           Draw a game of hide-and-seek at the Fairground, --at seconds into
                           the search
  --render-woods <PNG>     Draw a rummage in the Woods, --at seconds into it
  --render-hilltop <PNG>   Draw the Hilltop with a sample of finds placed on it
  --render-finds <PNG>     Draw every find's icon and Hilltop piece on one sheet, for review
  --render-fishing <PNG>   Draw a fishing trip at the pool, --at seconds into it
  --render-fish <PNG>      Draw every fish as it is held up, and its icon, for review
  --render-meadow <PNG>    Draw the meadow at the Woods' edge, every bug settled in its haunt
  --render-bugs <PNG>      Draw every bug in each of its poses, close up, and its icon, for review
  --render-sovereign <PNG> Draw the secret encounter --at seconds in, played through on its own
  --render-reactions <PNG> Draw everyone answering a pat, a snack and a toy, for review
  --render-story <PNG>     Draw a story in the Clubhouse --at seconds after it starts, reading each
                           line for 2.5 seconds and taking the first choice
  --package <FOLDER>       Load a story package beside Hill's own (for authors); repeatable
  --check-package <FOLDER> Check a story package and say what is wrong, without opening a window
  --packages-folder        Say where to put story packages for Hill to find, and what is there
  --sample-hilltop         Draw the station's skyline with a sample of finds on the Hilltop
  --hour <HOUR>            Draw at that hour of the day, from 0 to 24, rather than at midday;
                           with a window, hold the Hill at that hour rather than the clock's
  --at <SECONDS>           Draw that far into the arrival, or into free play on the green, in
                           the Clubhouse or at the Fairground
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
    Clubhouse,
    Fairground,
    /// The Fairground, a game of hide-and-seek under way.
    HideAndSeek,
    Woods,
    Hilltop,
    /// Not an area: the review sheet of every find.
    Finds,
    /// The pool in the Woods, a fishing trip under way.
    Fishing,
    /// Not an area: the review sheet of every fish.
    Fish,
    /// The meadow at the Woods' edge, every bug settled in its haunt.
    Meadow,
    /// Not an area: the review sheet of every bug.
    Bugs,
    /// The clearing that isn't on any map, the Sovereign met.
    Sovereign,
    /// Not an area: the review sheet of everyone's reactions.
    Reactions,
    /// The Clubhouse, a story under way in it.
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
    /// Say where the packages folder is, and what is in it, without opening a window.
    show_folder: bool,
    /// Draw the station with a sample of finds on the Hilltop, rather than the colony's own.
    sample_hilltop: bool,
    /// The hour to draw at, rather than midday.
    hour: Option<f32>,
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
        let daylight = daylight::Daylight::at_hour(args.hour.unwrap_or(12.0));
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
                if args.sample_hilltop {
                    station.show_hilltop(&sample_arrangement());
                } else {
                    station.show_hilltop(&memories.colony().hilltop);
                }
                station.set_daylight(daylight);
                station.compose(now)
            }
            Area::Green => {
                // Free play, run forward as the window would run it.
                let until = args.at.unwrap_or(20.0);
                let mut green = green::open(&arrival.cast, 0.0);
                green::show_hilltop(&mut green, &sample_arrangement());
                green.set_daylight(daylight);
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    green.tick(&arrival.cast, now);
                }
                green.compose(now)
            }
            Area::Clubhouse => {
                let until = args.at.unwrap_or(20.0);
                let library = story::Library::load(None, &args.packages);
                let pinned = library
                    .stories()
                    .enumerate()
                    .map(|(index, _)| index == 0)
                    .collect();
                let mut room =
                    clubhouse::Clubhouse::open(&arrival.cast, 0.0, &sample_arrangement(), pinned);
                room.set_daylight(daylight);
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    room.tick(&arrival.cast, now);
                }
                room.compose(now)
            }
            Area::Fairground => {
                let until = args.at.unwrap_or(20.0);
                let (mut fairground, _) = fairground::open(&arrival.cast, 0.0);
                fairground.set_daylight(daylight);
                fairground::show_hilltop(&mut fairground, &sample_arrangement());
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    fairground.tick(&arrival.cast, now);
                }
                fairground.compose(now)
            }
            Area::HideAndSeek => hiding_moment(&arrival.cast, args.at.unwrap_or(4.0), daylight),
            Area::Woods => woods_moment(&arrival.cast, args.at.unwrap_or(12.0), daylight),
            Area::Hilltop => {
                let until = args.at.unwrap_or(20.0);
                let mut hilltop = hilltop::open(&arrival.cast, 0.0, &sample_arrangement());
                hilltop.set_daylight(daylight);
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    hilltop.tick(&arrival.cast, now);
                }
                hilltop.compose(now)
            }
            Area::Finds => finds_sheet(),
            Area::Fishing => fishing_moment(&arrival.cast, args.at.unwrap_or(12.0), daylight),
            Area::Fish => fish_sheet(),
            Area::Meadow => meadow_moment(&arrival.cast, args.at.unwrap_or(12.0), daylight),
            Area::Bugs => bug_sheet(),
            Area::Sovereign => sovereign_moment(&arrival.cast, args.at.unwrap_or(10.0)),
            Area::Reactions => sheet::reactions(&arrival.cast),
            Area::Story => {
                let library = story::Library::load(None, &args.packages);
                story_moment(
                    &arrival.cast,
                    story_to_draw(&library)?,
                    args.at.unwrap_or(12.0),
                    daylight,
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
            Ok(Box::new(HillApp::new(cc, arrival, library, clock)))
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
    let mut show_folder = false;
    let mut sample_hilltop = false;
    let mut hour = None;
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
            Some("--render-woods") => render = Some((Area::Woods, value("--render-woods")?)),
            Some("--render-hilltop") => {
                render = Some((Area::Hilltop, value("--render-hilltop")?));
            }
            Some("--render-finds") => render = Some((Area::Finds, value("--render-finds")?)),
            Some("--render-fishing") => {
                render = Some((Area::Fishing, value("--render-fishing")?));
            }
            Some("--render-fish") => render = Some((Area::Fish, value("--render-fish")?)),
            Some("--render-meadow") => {
                render = Some((Area::Meadow, value("--render-meadow")?));
            }
            Some("--render-bugs") => render = Some((Area::Bugs, value("--render-bugs")?)),
            Some("--render-sovereign") => {
                render = Some((Area::Sovereign, value("--render-sovereign")?));
            }
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
    if at.is_some() && render.is_none() {
        bail!("--at only goes with one of the --render options");
    }
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
        at,
        packages,
        check,
        show_folder,
        sample_hilltop,
        hour,
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
fn story_moment(
    cast: &Cast,
    chosen: &story::Story,
    at: f32,
    daylight: daylight::Daylight,
) -> Result<Canvas> {
    // The colony comes in and settles first, as it would before anyone opens a story.
    const SETTLE: f32 = 12.0;
    let mut room = clubhouse::Clubhouse::open(cast, 0.0, &sample_arrangement(), vec![false]);
    room.set_daylight(daylight);
    let mut now = 0.0;
    while now < SETTLE {
        now += 1.0 / 30.0;
        room.tick(cast, now);
    }
    let mut director = story::Director::new(chosen.clone(), cast, 1).map_err(anyhow::Error::msg)?;
    room.ground().reserve(director.players());
    let at = SETTLE + at;
    let mut shown_since = None;
    while now < at && !director.finished() {
        now += 1.0 / 30.0;
        room.tick(cast, now);
        director.run(room.ground(), cast, now);
        let speaker = director
            .shown()
            .and_then(|shown| shown.speaker.as_ref().map(|(id, _)| *id));
        room.ground().set_speaker(speaker);
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
    Ok(room.compose(now))
}

/// A game of hide-and-seek `at` seconds into the search, once the colony has settled at the
/// Fairground and hidden.
fn hiding_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    const SETTLE: f32 = 12.0;
    let (mut ground, mut game) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
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

/// An outing to the Woods, `at` seconds in, played by a steady hand: it searches the spots in
/// turn and catches each moment as the marker crosses the gold. A parent and its little one go if
/// the colony has them, so the spots only some company opens show; otherwise the first two.
fn woods_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    let family = cast
        .members
        .iter()
        .find_map(|member| member.parent().map(|parent| vec![parent, member.id]));
    let party: Vec<cast::Id> = family.unwrap_or_else(|| cast.ids().take(2).collect());
    let close_pair = cast
        .bond(party[0], party[party.len() - 1])
        .is_some_and(|bond| bond.warmth >= formiga_travel::Band::High);
    let mut glade = woods::open(cast, &party, 0.0);
    let outset = woods::rummage::Outset {
        party,
        drought: 0,
        close_pair,
        influence: woods::influence(&sample_arrangement()),
        beckons: false,
        seed: 5,
    };
    glade.set_daylight(daylight);
    let mut outing = woods::rummage::Rummage::new(&mut glade, outset, |_| false, 0.0);
    outing.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    let mut next = 0;
    while now < at {
        now += 1.0 / 30.0;
        glade.tick(cast, now);
        outing.tick(&mut glade, now);
        match outing.phase() {
            woods::rummage::Phase::Exploring if next < outing.spot_count() => {
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

/// A fishing trip at the pool, `at` seconds in, played by a steady hand: it casts at each part of
/// the pool in turn, strikes on the bite, reels in only while the fish isn't pulling, and tries
/// somewhere else when nothing comes.
fn fishing_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use fishing::angling::{Angling, Outset, Phase};
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut pool = fishing::open(cast, &party, 0.0);
    let outset = Outset {
        party,
        drought: 0,
        influence: woods::influence(&sample_arrangement()),
        seed: 7,
    };
    pool.set_daylight(daylight);
    let mut trip = Angling::new(&mut pool, outset, |_| false, |_| false, 0.0);
    trip.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    let mut aim = 0;
    while now < at {
        now += 1.0 / 30.0;
        match trip.phase() {
            Phase::Ready => {
                // Around the pool's haunts in turn, a little way off each.
                let haunts = fishing::Haunt::ALL;
                let ((x, y), r) = haunts[aim % haunts.len()].area();
                aim += 1;
                trip.cast(&mut pool, (x + r * 0.5, y), now);
            }
            Phase::Waiting { .. } if trip.biting() => trip.strike(&mut pool, now),
            // Nothing coming: reel in and try somewhere else.
            Phase::Waiting { since, .. } if now - since > 10.0 => trip.strike(&mut pool, now),
            Phase::Fighting(fight) => trip.hold(!fight.pulling(now) && fight.strain < 0.6),
            _ => {}
        }
        pool.tick(cast, now);
        trip.tick(&mut pool, now);
        for event in trip.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    let mut scene = pool.compose(now);
    trip.draw(&mut scene, now);
    scene
}

/// The Sovereign met by the first two travellers, `at` seconds in: each line read after two and a
/// half seconds, and the attacks on offer chosen in turn.
fn sovereign_moment(cast: &Cast, at: f32) -> Canvas {
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut ground = clearing::open(cast, &party, 0.0);
    let mut sovereign = clearing::sovereign::Sovereign::new(&mut ground, cast, &party, true, 0.0);
    let mut now = 0.0;
    let mut line_since = None;
    let mut turn = 0;
    while now < at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        sovereign.tick(&mut ground, cast, now);
        if sovereign.stage.line.is_some() {
            let since = *line_since.get_or_insert(now);
            if now - since > 2.5 && now + 1.0 / 30.0 < at {
                sovereign.stage.read_on();
                line_since = None;
            }
        } else {
            line_since = None;
        }
        let offered = sovereign.choices().len();
        if offered > 0 {
            println!(
                "{now:6.1}s  chose {}",
                sovereign.choices()[turn % offered].name
            );
            sovereign.choose(turn % offered);
            turn += 1;
        }
        for event in sovereign.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    if let Some((speaker, line)) = &sovereign.stage.line {
        println!("On show: {speaker:?}: {line}");
    }
    clearing::compose(&mut ground, &sovereign, now)
}

/// Every fish, held up as it is when landed, with its icon in the corner: a row each.
fn fish_sheet() -> Canvas {
    const ROW: i32 = 40;
    let mut sheet = Canvas::new(384, (ROW * fishing::fish::CATALOGUE.len() as i32) as u32);
    for (row, fish) in fishing::fish::CATALOGUE.iter().enumerate() {
        let top = row as i32 * ROW;
        let shade = if row % 2 == 0 { 0xdcecf0 } else { 0xcfe2e8 };
        sheet.fill_rect(0, top, 384, ROW, paint::rgb(shade));
        paint::blit(&mut sheet, &fishing::art::icon(fish.id), 4, top + 4);
        let catch = fishing::art::catch(fish.id);
        let at = (24, top + (ROW - catch.height() as i32) / 2);
        paint::blit(&mut sheet, &catch, at.0, at.1);
    }
    sheet
}

/// The meadow `at` seconds after the first two travellers walk out into it, with every bug in
/// the catalogue settled on a perch in its haunt, for seeing the place and its bugs together.
fn meadow_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut ground = meadow::open(cast, &party, 0.0, &sample_arrangement());
    ground.set_daylight(daylight);
    let mut now = 0.0;
    while now < at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
    }
    let mut scene = ground.compose(now);
    for (index, bug) in meadow::bugs::CATALOGUE.iter().enumerate() {
        let perches = bug.haunt.perches();
        let (x, y) = perches[index % perches.len()];
        let sprite = meadow::art::sprite(bug.id, meadow::art::Pose::Settled, 0);
        let (ax, ay) = meadow::art::anchor(bug.id, meadow::art::Pose::Settled);
        paint::blit(&mut scene, &sprite, x as i32 - ax, y as i32 - ay);
    }
    scene
}

/// Every bug, a row each: its icon, each frame of flying, settled and crawling on a strip of
/// meadow, and close up as it is held in the net.
fn bug_sheet() -> Canvas {
    use meadow::art::{Pose, close_up, frames, icon, sprite};
    const ROW: i32 = 36;
    let mut sheet = Canvas::new(384, (ROW * meadow::bugs::CATALOGUE.len() as i32) as u32);
    for (row, bug) in meadow::bugs::CATALOGUE.iter().enumerate() {
        let top = row as i32 * ROW;
        let shade = if row % 2 == 0 { 0xcfe3bf } else { 0xc2d9b2 };
        sheet.fill_rect(0, top, 384, ROW, paint::rgb(shade));
        paint::blit(&mut sheet, &icon(bug.id), 4, top + 4);
        let mut x = 20;
        for pose in [Pose::Flying, Pose::Settled, Pose::Crawling] {
            for frame in 0..frames(bug.id, pose) {
                let picture = sprite(bug.id, pose, frame);
                paint::blit(
                    &mut sheet,
                    &picture,
                    x,
                    top + (ROW - picture.height() as i32) / 2,
                );
                x += picture.width() as i32 + 4;
            }
            x += 8;
        }
        let close = close_up(bug.id);
        paint::blit(
            &mut sheet,
            &close,
            384 - close.width() as i32 - 4,
            top + (ROW - close.height() as i32) / 2,
        );
    }
    sheet
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
    let mut sheet = Canvas::new((CELL.0 * columns) as u32, (CELL.1 * 5) as u32);
    // A row for each kind, and a last row for the relics.
    let rows: Vec<Vec<&finds::Find>> = finds::Kind::ALL
        .into_iter()
        .map(|kind| {
            finds::CATALOGUE
                .iter()
                .filter(|find| find.kind == kind)
                .collect()
        })
        .chain(std::iter::once(finds::RELICS.iter().collect()))
        .collect();
    for (row, of_kind) in rows.into_iter().enumerate() {
        for (column, find) in of_kind.into_iter().enumerate() {
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
        let library = story::Library::load(None, &[PathBuf::from("/no/such/package.formiga-hill")]);
        let problem = story_to_draw(&library).unwrap_err().to_string();
        assert!(problem.contains("package.formiga-hill"), "{problem}");
        assert!(story_to_draw(&story::Library::load(None, &[])).is_ok());
    }

    #[test]
    fn a_story_is_drawn_at_the_hour_asked_for() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let library = story::Library::load(None, &[]);
        let chosen = story_to_draw(&library).unwrap();
        let at =
            |hour| story_moment(&cast, chosen, 1.0, daylight::Daylight::at_hour(hour)).unwrap();
        assert_ne!(at(12.0), at(0.0), "midnight looks like noon");
    }

    #[test]
    fn two_sources_or_a_missing_path_are_refused() {
        assert!(parse(&["--sample", "--formiga-travel", "/trips/ab"]).is_err());
        assert!(parse(&["--formiga-travel"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--help"]).unwrap().is_none());
    }
}
