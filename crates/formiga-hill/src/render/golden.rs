//! Every picture Hill paints, checked pixel for pixel against how it was painted before: each
//! place's still layers, every find, bug and fish, and whole scenes composed and lit at several
//! hours. A change meant only to make drawing faster must leave all of them exactly as they were.
//!
//! Each picture is kept as a hash of its pixels, in the `golden` folder beside this file. When a
//! picture is changed on purpose, run the tests with `FORMIGA_HILL_BLESS=1` to write the new
//! hashes, and say in the commit which pictures changed and why.
//!
//! The hashes are only checked on macOS, where they were taken: some scenery is shaped with
//! sines and square roots, and another system's maths library may round one of those a hair
//! differently and move a pixel. Everywhere else the pictures are still painted, so nothing panics.

use super::*;
use crate::finds::art as find_art;
use crate::meadow::art::{self as bug_art, Pose};
use std::fmt::Write as _;
use std::path::Path;

/// A picture's pixels as one number: FNV-1a over its size and every channel of every pixel.
fn hash(canvas: &Canvas) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut eat = |byte: u8| {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    };
    for byte in canvas
        .width()
        .to_le_bytes()
        .into_iter()
        .chain(canvas.height().to_le_bytes())
    {
        eat(byte);
    }
    for pixel in canvas.pixels() {
        for byte in [pixel.r, pixel.g, pixel.b, pixel.a] {
            eat(byte);
        }
    }
    hash
}

/// The pictures of one kind, by name.
struct Pictures(Vec<(String, u64)>);

impl Pictures {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, name: impl Into<String>, canvas: &Canvas) {
        self.0.push((name.into(), hash(canvas)));
    }

    /// Every still layer of a playground, under `place`.
    fn place(&mut self, place: &str, ground: &playground::Playground) {
        for (layer, canvas) in ground.layers() {
            self.add(format!("{place}.{layer}"), &canvas);
        }
    }

    /// Checks these pictures against the hashes kept in `golden/<kind>.txt`, saying which have
    /// changed, which are new and which have gone; or writes them there, if asked to.
    fn check(self, kind: &str) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/render/golden")
            .join(format!("{kind}.txt"));
        let mut table = String::new();
        for (name, hash) in &self.0 {
            writeln!(table, "{name} {hash:016x}").unwrap();
        }
        if std::env::var_os("FORMIGA_HILL_BLESS").is_some_and(|value| !value.is_empty()) {
            std::fs::write(&path, &table).unwrap();
            return;
        }
        if !cfg!(target_os = "macos") {
            return;
        }
        let kept = std::fs::read_to_string(&path).unwrap_or_default();
        let kept: Vec<(&str, &str)> = kept
            .lines()
            .filter_map(|line| line.trim().split_once(' '))
            .collect();
        let mut differences = Vec::new();
        for (name, hash) in &self.0 {
            let now = format!("{hash:016x}");
            match kept.iter().find(|(kept_name, _)| kept_name == name) {
                Some((_, before)) if *before == now => {}
                Some((_, before)) => {
                    differences.push(format!("  changed  {name}: {before} -> {now}"))
                }
                None => differences.push(format!("  new      {name}: {now}")),
            }
        }
        for (name, before) in &kept {
            if !self.0.iter().any(|(now, _)| now == name) {
                differences.push(format!("  gone     {name}: {before}"));
            }
        }
        assert!(
            differences.is_empty(),
            "{} of the {kind} pictures are not painted as they were:\n{}\n\n\
             If that was meant, run the tests with FORMIGA_HILL_BLESS=1 to keep the new hashes.",
            differences.len(),
            differences.join("\n"),
        );
    }
}

/// Some of the sample colony: who comes along to the places only some go to.
fn party(cast: &Cast) -> Vec<cast::Id> {
    cast.ids().take(4).collect()
}

#[test]
fn every_place_is_painted_as_it_was() {
    let cast = cast::sample();
    let party = party(&cast);
    let hilltop = sample_arrangement();
    let mut pictures = Pictures::new();

    let mut ground = green::open(&cast, 0.0);
    pictures.place("green", &ground);
    green::show_hilltop(&mut ground, &hilltop);
    pictures.add("green.hilltop.backdrop", ground.backdrop());

    let (mut ground, _) = fairground::open(&cast, 0.0);
    pictures.place("fairground", &ground);
    fairground::show_hilltop(&mut ground, &hilltop);
    pictures.add("fairground.hilltop.backdrop", ground.backdrop());

    pictures.place("hilltop", &hilltop::open(&cast, 0.0, &hilltop));
    pictures.place(
        "hilltop.bare",
        &hilltop::open(&cast, 0.0, &hilltop::Arrangement::new()),
    );

    let mut room = clubhouse::Clubhouse::open(&cast, 0.0, &hilltop, vec![true, false]);
    pictures.place("clubhouse", room.ground());
    room.set_daylight(daylight::Daylight::at_hour(22.0));
    pictures.add("clubhouse.night.backdrop", room.ground().backdrop());

    pictures.place("meadow", &meadow::open(&cast, &party, 0.0, &hilltop));
    pictures.place("hedgerow", &hedgerow::open(&cast, &party, 0.0, &hilltop));
    pictures.place("track", &track::open(&cast, &party, 0.0, &hilltop));
    pictures.place("track.hunt", &track::hunt_ground(&cast, &party, 0.0));
    pictures.place("fishing", &fishing::open(&cast, &party, 0.0));
    pictures.place("woods", &woods::open(&cast, &party, 0.0));
    pictures.place("falls", &falls::open(&cast, &party, 0.0));
    pictures.place("clearing", &clearing::open(&cast, &party, 0.0));

    let mut station = station::Station::new(&cast, station::Journey::Here, &[]);
    station.show_hilltop(&hilltop);
    station.show_notices(4);
    for hour in [12.0, 6.5, 19.75, 0.0] {
        station.set_daylight(daylight::Daylight::at_hour(hour));
        pictures.add(format!("station.at.{hour}"), &station.compose(0.0));
    }

    pictures.check("places");
}

#[test]
fn every_find_is_painted_as_it_was() {
    let mut pictures = Pictures::new();
    let every = finds::CATALOGUE
        .iter()
        .chain(&finds::RELICS)
        .chain(&finds::FORAGED)
        .chain(&finds::SCAVENGED)
        .chain(&finds::AFAR);
    for find in every {
        pictures.add(format!("icon.{}", find.id), &find_art::icon(find.id));
        let piece = find_art::piece(find.id);
        pictures.add(
            format!("piece.{}.{},{}", find.id, piece.anchor.0, piece.anchor.1),
            &piece.sprite,
        );
    }
    pictures.add("icon.unknown", &find_art::icon("no_such_find"));
    for growth in finds::growing::GROWING {
        for stage in 0..growth.stages.len() as u8 {
            let piece = find_art::stage(growth.id, stage);
            pictures.add(
                format!(
                    "stage.{}.{stage}.{},{}",
                    growth.id, piece.anchor.0, piece.anchor.1
                ),
                &piece.sprite,
            );
        }
    }
    for plan in &finds::plans::PLANS {
        let piece = find_art::built(plan.id);
        pictures.add(
            format!("built.{}.{},{}", plan.id, piece.anchor.0, piece.anchor.1),
            &piece.sprite,
        );
    }
    pictures.check("finds");
}

#[test]
fn every_bug_and_fish_is_painted_as_it_was() {
    let mut pictures = Pictures::new();
    for bug in &meadow::bugs::CATALOGUE {
        for (pose, name) in [
            (Pose::Flying, "flying"),
            (Pose::Settled, "settled"),
            (Pose::Crawling, "crawling"),
        ] {
            for frame in 0..bug_art::frames(bug.id, pose) {
                pictures.add(
                    format!("bug.{}.{name}.{frame}", bug.id),
                    &bug_art::sprite(bug.id, pose, frame),
                );
            }
        }
        pictures.add(
            format!("bug.{}.close_up", bug.id),
            &bug_art::close_up(bug.id),
        );
        pictures.add(format!("bug.{}.icon", bug.id), &bug_art::icon(bug.id));
    }
    for fish in &fishing::fish::CATALOGUE {
        pictures.add(format!("fish.{}", fish.id), &fishing::art::catch(fish.id));
        pictures.add(
            format!("fish.{}.icon", fish.id),
            &fishing::art::icon(fish.id),
        );
    }
    pictures.check("creatures");
}

/// Whole scenes as the window shows them: the colony at play or at a game, composed back to
/// front and lit for the hour, by night as well as by day.
#[test]
fn every_scene_is_composed_as_it_was() {
    use Area::*;
    let cast = cast::sample();
    let scenes: [(Area, f32, f32); 30] = [
        (Green, 20.0, 12.0),
        (Green, 20.0, 0.0),
        (Green, 8.0, 19.75),
        (Clubhouse, 10.0, 23.0),
        (Fairground, 10.0, 22.0),
        (HideAndSeek, 4.0, 12.0),
        (SackRace, 5.0, 12.0),
        (HighStriker, 9.0, 6.5),
        (Hoopla, 12.0, 21.0),
        (TugOfWar, 10.0, 12.0),
        (Woods, 12.0, 12.0),
        (Woods, 12.0, 2.0),
        (Hilltop, 10.0, 12.0),
        (Hilltop, 10.0, 1.0),
        (Building, 5.0, 12.0),
        (Fishing, 12.0, 12.0),
        (Meadow, 12.0, 12.0),
        (BugHunt, 30.0, 20.5),
        (Hedgerow, 30.0, 12.0),
        (Expedition, 8.0, 12.0),
        (Falls, 14.0, 3.0),
        (Track, 30.0, 0.0),
        (Treasure, 20.0, 12.0),
        (Sovereign, 10.0, 12.0),
        (Story, 12.0, 12.0),
        (Adventure, 10.0, 12.0),
        (Costumes, 0.0, 12.0),
        (Reactions, 0.0, 12.0),
        (Landmarks, 0.0, 12.0),
        (Produce, 0.0, 12.0),
    ];
    let mut pictures = Pictures::new();
    for (area, at, hour) in scenes {
        let scene = draw(area, &cast, Some(at), Some(hour), true, &[]).unwrap();
        pictures.add(format!("{area:?}.{at}s.at.{hour}"), &scene);
    }
    pictures.check("scenes");
}
