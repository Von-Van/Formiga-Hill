//! The whole Hill without a window: built on egui's own context, which draws nothing, with a
//! scratch folder for its data, so a visit can be walked through as the person would.

use super::rummaging::Activity;
use super::*;
use crate::audio::Sound;
use crate::memories::ColonyMemories;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A scratch data folder of its own for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hill-headless-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The sample colony on a real trip, as Desktop leaves one, so whatever happens is kept in
/// `data`: at `hour` by a clock that keeps to it.
fn visit(data: &Path, hour: f32) -> HillApp {
    let snapshot = formiga_travel::sample::snapshot();
    let dir = data.join("trip").join(snapshot.session_id.as_str());
    std::fs::create_dir_all(&dir).unwrap();
    formiga_travel::write_document(&dir.join(formiga_travel::SNAPSHOT_FILE), &snapshot).unwrap();
    let (trip, cast) = crate::trip::arrive(&dir, false).unwrap();
    let arrival = Arrival {
        cast,
        visit: Visit::Trip(trip),
    };
    HillApp::begin(
        arrival,
        Library::load(None, &[]),
        Clock::Held(hour),
        None,
        Some(data.to_owned()),
        Sound::quiet(),
    )
}

/// One frame of the window, at `time` seconds in, with these events.
fn frame(app: &mut HillApp, ctx: &egui::Context, time: f64, events: Vec<egui::Event>) {
    let input = egui::RawInput {
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1152.0, 720.0),
        )),
        events,
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_ui(input, |ui| app.frame(ui, time as f32));
    // Nothing is painted, so the pictures it would have uploaded are let go of.
    output.textures_delta.clear();
}

/// The memories as written down in `data`, for the sample colony.
fn kept(data: &Path) -> ColonyMemories {
    let colony = formiga_travel::sample::snapshot().colony_id;
    Memories::look(Some(data), &colony).colony().clone()
}

/// The adventure being made on the map editor.
fn making(app: &HillApp) -> &super::mapmaking::Making {
    app.woods.making.as_ref().unwrap()
}

/// Every kind of outing there is to set off on, with the one an adventure is played as.
const OUTINGS: [Activity; 7] = [
    Activity::Rummage,
    Activity::Fish,
    Activity::Bugs,
    Activity::Forage,
    Activity::Scavenge,
    Activity::Treasure,
    Activity::Expedition,
];

/// How many outings of this kind the colony has been on, as its memories count them.
fn been_on(colony: &ColonyMemories, activity: Activity) -> u32 {
    match activity {
        Activity::Rummage => colony.outings,
        Activity::Fish => colony.fishing_trips,
        Activity::Bugs => colony.bug_hunts,
        Activity::Forage => colony.forays,
        Activity::Scavenge => colony.scavenges,
        Activity::Treasure => colony.treasure_hunts,
        Activity::Expedition | Activity::Adventure => colony.expeditions,
    }
}

/// Into the Woods and off on `activity`, with a torn map to follow first if it needs one.
fn set_off(app: &mut HillApp, activity: Activity, now: f32) {
    app.go_to(Area::Woods, now);
    if activity == Activity::Treasure {
        let leader = app.woods.party[0];
        assert!(app.memories.found_a_map(leader));
    }
    app.woods.activity = activity;
    app.set_off_on(activity, now);
    assert!(
        app.woods.outing.is_some() || app.woods.expedition.is_some(),
        "never set off on {activity:?}"
    );
}

#[test]
fn leaving_the_woods_brings_every_outing_home_and_keeps_it_once() {
    for activity in OUTINGS {
        let data = scratch(&format!("home-{activity:?}"));
        let mut app = visit(&data, 12.0);
        set_off(&mut app, activity, 1.0);
        let before = been_on(&kept(&data), activity);
        app.go_to(Area::Station, 2.0);
        assert!(app.woods.outing.is_none() && app.woods.expedition.is_none());
        assert_eq!(
            been_on(&kept(&data), activity),
            before + 1,
            "{activity:?} wasn't kept"
        );
        // Walking in and out again, and the visit ending, keep nothing more.
        app.go_to(Area::Woods, 3.0);
        app.go_to(Area::Station, 4.0);
        app.depart();
        assert_eq!(
            been_on(&kept(&data), activity),
            before + 1,
            "{activity:?} was kept twice"
        );
        std::fs::remove_dir_all(&data).unwrap();
    }
}

#[test]
fn every_ground_in_the_woods_is_dressed_and_lit_whatever_is_under_way() {
    let data = scratch("dressed");
    for activity in OUTINGS {
        let mut app = visit(&data, 21.0);
        set_off(&mut app, activity, 1.0);
        let id = app.woods.party[0];
        app.costumes.insert(id, "wizard_hat");
        app.keep_hours(2.0);
        app.dress_everyone();
        let daylight = app.daylight;
        assert_ne!(
            daylight,
            Daylight::default(),
            "the clock's hour is the evening's"
        );
        let mut met = 0;
        for ground in app.woods.grounds_mut() {
            assert_eq!(
                ground.daylight(),
                daylight,
                "a ground left unlit on {activity:?}"
            );
            if !ground.ids().contains(&id) {
                continue;
            }
            met += 1;
            let hatted = ground.bounds(id, 2.0).unwrap();
            ground.dress(&HashMap::new(), &HashSet::new());
            let bare = ground.bounds(id, 2.0).unwrap();
            assert!(hatted.1 < bare.1, "no hat on {activity:?}");
        }
        assert!(met > 0, "nobody out on {activity:?}");
        app.go_to(Area::Station, 3.0);
    }
    std::fs::remove_dir_all(&data).unwrap();
}

#[test]
fn the_map_editor_puts_joins_takes_off_and_undoes_without_a_hitch() {
    use super::mapmaking::Tool;
    use crate::expedition::map::Site;
    let data = scratch("making");
    let mut app = visit(&data, 12.0);
    let ctx = egui::Context::default();
    let mut time = 0.0;
    let mut frames = |app: &mut HillApp, seconds: f64, events: Vec<egui::Event>| {
        frame(app, &ctx, time, events);
        let end = time + seconds;
        while time < end {
            time += 0.25;
            frame(app, &ctx, time, Vec::new());
        }
    };
    frames(&mut app, 1.0, Vec::new());
    app.go_to(Area::Woods, 1.0);
    app.woods.activity = Activity::Adventure;
    app.begin_adventure();
    frames(&mut app, 1.0, Vec::new());
    let way_in = making(&app).draft.places[0].at;
    // A glade put down and joined to the way in, then a pool joined to the glade.
    app.making_click(Some((150.0, 150.0)), 2.0);
    app.making_click(Some(way_in), 2.0);
    let glade = making(&app).draft.places[1].at;
    app.woods.making.as_mut().unwrap().tool = Tool::Put(Site::Pool);
    app.making_click(Some((290.0, 170.0)), 2.0);
    app.making_click(Some(glade), 2.0);
    assert_eq!(making(&app).draft.places.len(), 3);
    assert_eq!(making(&app).draft.paths.len(), 2);
    frames(&mut app, 2.0, Vec::new());
    // The pool taken off the map again, with its path.
    assert!(app.woods.making.as_mut().unwrap().draft.remove_place(2));
    frames(&mut app, 2.0, Vec::new());
    assert_eq!(making(&app).draft.places.len(), 2);
    if let Tool::Join(Some(from)) = making(&app).tool {
        assert!(
            making(&app).draft.index(from).is_some(),
            "joined from a place gone"
        );
    }
    // Undone: the pool is back.
    let undo = egui::Event::Key {
        key: egui::Key::Z,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    };
    frames(&mut app, 1.0, vec![undo]);
    assert_eq!(making(&app).draft.places.len(), 3);
    assert_eq!(making(&app).draft.paths.len(), 2);
    std::fs::remove_dir_all(&data).unwrap();
}
