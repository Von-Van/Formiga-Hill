//! Formiga Hill: somewhere the colony goes to spend time together.

mod actor;
mod app;
mod cast;
mod character;
mod cues;
mod dice;
mod font;
mod green;
mod kit;
mod materials;
mod paint;
mod sheet;
mod station;
mod trip;

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
                    [--render-station <PNG> | --render-green <PNG>] [--at <SECONDS>]

  --sample                 Arrive with Desktop's made-up sample colony (the default)
  --formiga-travel <DIR>   How Desktop starts Hill: the trip it wrote, answered on the way home
  --from-save <FILE>       Development only: board a Desktop colony file, which is only ever read
  --render-station <PNG>   Draw the station to a PNG and exit without opening a window
  --render-green <PNG>     Draw the Village Green to a PNG and exit without opening a window
  --render-reactions <PNG> Draw everyone answering a pat, a snack and a toy, for review
  --at <SECONDS>           Draw that far into the arrival, or into free play on the green
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
    /// Not an area: the review sheet of everyone's reactions.
    Reactions,
}

struct Args {
    source: Source,
    render: Option<(Area, PathBuf)>,
    at: Option<f32>,
}

fn main() -> Result<()> {
    let Some(args) = parse_args(std::env::args_os().skip(1))? else {
        print!("{USAGE}");
        return Ok(());
    };
    let arrival = arrive(args.source)?;

    if let Some((area, path)) = args.render {
        let canvas = match area {
            Area::Station => {
                let (journey, now) = match args.at {
                    Some(seconds) => (station::Journey::Arriving { since: 0.0 }, seconds),
                    None => (station::Journey::Here, 0.0),
                };
                station::Station::new(&arrival.cast, journey).compose(now)
            }
            Area::Green => {
                // Free play, run forward as the window would run it.
                let until = args.at.unwrap_or(20.0);
                let mut green = green::Green::new(&arrival.cast, 0.0);
                let mut now = 0.0;
                while now < until {
                    now += 1.0 / 30.0;
                    green.tick(&arrival.cast, now);
                }
                green.compose(now)
            }
            Area::Reactions => sheet::reactions(&arrival.cast),
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
        Box::new(|cc| Ok(Box::new(HillApp::new(cc, arrival)))),
    )
    .map_err(|error| anyhow::anyhow!("the Hill window could not open: {error}"))
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Option<Args>> {
    let mut source = None;
    let mut render = None;
    let mut at = None;
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
            Some("-h" | "--help") => return Ok(None),
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        }
    }
    if at.is_some() && render.is_none() {
        bail!("--at only goes with --render-station or --render-green");
    }
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
        at,
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
    fn two_sources_or_a_missing_path_are_refused() {
        assert!(parse(&["--sample", "--formiga-travel", "/trips/ab"]).is_err());
        assert!(parse(&["--formiga-travel"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--help"]).unwrap().is_none());
    }
}
