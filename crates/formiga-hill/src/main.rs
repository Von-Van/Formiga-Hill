//! Formiga Hill: somewhere the colony goes to spend time together.

mod app;
mod station;

use anyhow::{Context, Result, bail};
use app::{Arrival, HillApp};
use formiga_art::Canvas;
use formiga_travel::{ExportOptions, SessionId, TravelFiles, export_snapshot, read_snapshot};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const USAGE: &str = "\
Formiga Hill

Usage: formiga-hill [--sample | --snapshot <FILE> | --from-save <FILE>] [--render-station <PNG>]

  --sample                 Arrive with a made-up colony (the default)
  --snapshot <FILE>        Arrive on a trip Desktop started, and write its receipt on the way home
  --from-save <FILE>       Development only: board a Desktop colony file, which is only ever read
  --render-station <PNG>   Draw the station to a PNG and exit without opening a window
";

enum Source {
    Sample,
    Snapshot(PathBuf),
    Save(PathBuf),
}

struct Args {
    source: Source,
    render: Option<PathBuf>,
}

fn main() -> Result<()> {
    let Some(args) = parse_args(std::env::args_os().skip(1))? else {
        print!("{USAGE}");
        return Ok(());
    };
    let arrival = arrive(args.source)?;

    if let Some(path) = args.render {
        let station = station::Station::new(arrival.snapshot());
        write_png(&path, &station.compose(0.0), 3)?;
        println!("Drew the station to {}", path.display());
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
    let mut set_source = |next: Source| {
        if source.replace(next).is_some() {
            bail!("choose one of --sample, --snapshot, or --from-save");
        }
        Ok(())
    };
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .map(PathBuf::from)
                .with_context(|| format!("{flag} needs a path"))
        };
        match arg.to_str() {
            Some("--sample") => set_source(Source::Sample)?,
            Some("--snapshot") => set_source(Source::Snapshot(value("--snapshot")?))?,
            Some("--from-save") => set_source(Source::Save(value("--from-save")?))?,
            Some("--render-station") => render = Some(value("--render-station")?),
            Some("-h" | "--help") => return Ok(None),
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        }
    }
    Ok(Some(Args {
        source: source.unwrap_or(Source::Sample),
        render,
    }))
}

fn arrive(source: Source) -> Result<Arrival> {
    Ok(match source {
        Source::Sample => Arrival::Visit {
            snapshot: formiga_travel::sample::snapshot(),
            label: "sample colony".to_owned(),
        },
        Source::Snapshot(path) => {
            let snapshot = read_snapshot(&path)?;
            let receipt = TravelFiles::beside(&path, snapshot.session_id).receipt;
            Arrival::Trip { snapshot, receipt }
        }
        Source::Save(path) => {
            // Read-only: this stands in for Desktop's exporter until Desktop has one.
            let save = formiga_core::SaveStore::read_snapshot(&path)
                .with_context(|| format!("could not read the colony file {}", path.display()))?;
            let snapshot = export_snapshot(
                &save,
                ExportOptions {
                    session_id: SessionId::random().context("no randomness for a session id")?,
                    created_at_utc: OffsetDateTime::now_utc(),
                    desktop_version: format!("colony file, save v{}", save.save_version),
                },
            );
            snapshot.validate()?;
            Arrival::Visit {
                snapshot,
                label: "from a colony file".to_owned(),
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
        let args = parse(&["--snapshot", "trip.json", "--render-station", "out.png"])
            .unwrap()
            .unwrap();
        assert!(
            matches!(args.source, Source::Snapshot(ref path) if path == Path::new("trip.json"))
        );
        assert_eq!(args.render.as_deref(), Some(Path::new("out.png")));
    }

    #[test]
    fn two_sources_or_a_missing_path_are_refused() {
        assert!(parse(&["--sample", "--snapshot", "trip.json"]).is_err());
        assert!(parse(&["--snapshot"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--help"]).unwrap().is_none());
    }
}
