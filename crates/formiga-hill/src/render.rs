//! The headless `--render-*` commands, for review: an area drawn some way into a visit, or a
//! sheet of everything of one kind, to a PNG without opening a window.

use crate::cast::{self, Cast};
use crate::{
    clearing, clubhouse, daylight, expedition, fairground, falls, finds, fishing, green, hedgerow,
    hilltop, meadow, memories, paint, playground, sheet, station, story, track, woods,
};
use anyhow::{Context, Result, bail};
use formiga_art::Canvas;
use std::path::PathBuf;

/// What a `--render-*` command draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    Station,
    Green,
    Clubhouse,
    Fairground,
    /// The Fairground, a game of hide-and-seek under way.
    HideAndSeek,
    /// The Fairground, a sack race under way.
    SackRace,
    /// The Fairground, the colony taking turns at the high striker.
    HighStriker,
    /// The Fairground, the colony taking turns at the hoopla stall.
    Hoopla,
    /// The Fairground, the colony at the tug-of-war.
    TugOfWar,
    Woods,
    Hilltop,
    /// Not an area: the review sheet of every find.
    Finds,
    /// Not an area: the review sheet of everything that grows, at every stage.
    Growing,
    /// Not an area: the review sheet of everything that can be built.
    Plans,
    /// The Hilltop, the colony building something on it.
    Building,
    /// The pool in the Woods, a fishing trip under way.
    Fishing,
    /// Not an area: the review sheet of every fish.
    Fish,
    /// The meadow at the Woods' edge, every bug settled in its haunt.
    Meadow,
    /// Not an area: the review sheet of every bug.
    Bugs,
    /// The meadow, a bug hunt under way.
    BugHunt,
    /// The hedgerow, a foray under way.
    Hedgerow,
    /// Not an area: the review sheet of everything the hedgerow grows, at every stage.
    Produce,
    /// An expedition under way: the map, or a stop along the way.
    Expedition,
    /// The Far Falls, a visit under way.
    Falls,
    /// The old track, a scavenge under way.
    Track,
    /// Off the old track, a treasure hunt under way.
    Treasure,
    /// Not an area: the review sheet of the landmarks a map can name.
    Landmarks,
    /// The clearing that isn't on any map, the Sovereign met.
    Sovereign,
    /// Not an area: the review sheet of everyone's reactions.
    Reactions,
    /// Not an area: the review sheet of everyone in every costume.
    Costumes,
    /// The Clubhouse, a story under way in it.
    Story,
}

/// The picture `--render-*` asks for: `area` drawn `at` seconds in at `hour` (at midday if none
/// is given), or its review sheet.
pub fn draw(
    area: Area,
    cast: &Cast,
    at: Option<f32>,
    hour: Option<f32>,
    sample_hilltop: bool,
    packages: &[PathBuf],
) -> Result<Canvas> {
    let daylight = daylight::Daylight::at_hour(hour.unwrap_or(12.0));
    Ok(match area {
        Area::Station => {
            let (journey, now) = match at {
                Some(seconds) => (station::Journey::Arriving { since: 0.0 }, seconds),
                None => (station::Journey::Here, 0.0),
            };
            // The display case shows what this colony has kept, as Hill remembers it.
            let memories = memories::Memories::open(
                memories::Memories::folder().as_deref(),
                &cast.snapshot.colony_id,
            );
            let mut station = station::Station::new(cast, journey, &memories.colony().souvenirs);
            if sample_hilltop {
                station.show_hilltop(&sample_arrangement());
            } else {
                station.show_hilltop(&memories.colony().hilltop);
            }
            // A few notes on the board, as on a return visit with something to tell.
            station.show_notices(4);
            station.set_daylight(daylight);
            station.compose(now)
        }
        Area::Green => {
            // Free play, run forward as the window would run it.
            let until = at.unwrap_or(20.0);
            let mut green = green::open(cast, 0.0);
            green::show_hilltop(&mut green, &sample_arrangement());
            green.set_daylight(daylight);
            let mut now = 0.0;
            while now < until {
                now += 1.0 / 30.0;
                green.tick(cast, now);
            }
            green.compose(now)
        }
        Area::Clubhouse => {
            let until = at.unwrap_or(20.0);
            let library = story::Library::load(None, packages);
            let pinned = library
                .stories()
                .enumerate()
                .map(|(index, _)| index == 0)
                .collect();
            let mut room = clubhouse::Clubhouse::open(cast, 0.0, &sample_arrangement(), pinned);
            room.set_daylight(daylight);
            let mut now = 0.0;
            while now < until {
                now += 1.0 / 30.0;
                room.tick(cast, now);
            }
            room.compose(now)
        }
        Area::Fairground => {
            let until = at.unwrap_or(20.0);
            let (mut fairground, _) = fairground::open(cast, 0.0);
            fairground.set_daylight(daylight);
            fairground::show_hilltop(&mut fairground, &sample_arrangement());
            let mut now = 0.0;
            while now < until {
                now += 1.0 / 30.0;
                fairground.tick(cast, now);
            }
            fairground.compose(now)
        }
        Area::HideAndSeek => hiding_moment(cast, at.unwrap_or(4.0), daylight),
        Area::SackRace => race_moment(cast, at.unwrap_or(5.0), daylight),
        Area::HighStriker => striker_moment(cast, at.unwrap_or(9.0), daylight),
        Area::Hoopla => hoopla_moment(cast, at.unwrap_or(12.0), daylight),
        Area::TugOfWar => tug_moment(cast, at.unwrap_or(10.0), daylight),
        Area::Woods => woods_moment(cast, at.unwrap_or(12.0), daylight),
        Area::Hilltop => {
            let until = at.unwrap_or(20.0);
            let mut hilltop = hilltop::open(cast, 0.0, &sample_arrangement());
            hilltop.set_daylight(daylight);
            let mut now = 0.0;
            while now < until {
                now += 1.0 / 30.0;
                hilltop.tick(cast, now);
            }
            hilltop.compose(now)
        }
        Area::Finds => finds_sheet(),
        Area::Growing => growing_sheet(),
        Area::Plans => plans_sheet(),
        Area::Building => building_moment(cast, at.unwrap_or(5.0), daylight),
        Area::Fishing => fishing_moment(cast, at.unwrap_or(12.0), daylight),
        Area::Fish => fish_sheet(),
        Area::Meadow => meadow_moment(cast, at.unwrap_or(12.0), daylight),
        Area::Bugs => bug_sheet(),
        Area::BugHunt => bug_hunt_moment(cast, at.unwrap_or(30.0), daylight),
        Area::Hedgerow => foray_moment(cast, at.unwrap_or(30.0), daylight),
        Area::Produce => produce_sheet(),
        Area::Expedition => expedition_moment(cast, at.unwrap_or(8.0), daylight),
        Area::Falls => falls_moment(cast, at.unwrap_or(14.0), daylight),
        Area::Track => track_moment(cast, at.unwrap_or(30.0), daylight),
        Area::Treasure => treasure_moment(cast, at.unwrap_or(20.0), daylight),
        Area::Landmarks => track::landmarks_sheet(),
        Area::Sovereign => sovereign_moment(cast, at.unwrap_or(10.0)),
        Area::Reactions => sheet::reactions(cast),
        Area::Costumes => sheet::costumes(cast),
        Area::Story => {
            let library = story::Library::load(None, packages);
            story_moment(cast, story_to_draw(&library)?, at.unwrap_or(12.0), daylight)?
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
    let (mut ground, mut games) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    let mut seeking_since = None;
    while seeking_since.is_none_or(|since| now < since + at) && now < 300.0 {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        games.tick(&mut ground, now);
        if now >= SETTLE && games.playing().is_none() && seeking_since.is_none() {
            let game = fairground::Game::HideAndSeek;
            games.start(game, &mut ground, cast, &[], now);
        }
        if let fairground::Phase::Seeking { since } = games.hide_and_seek.phase() {
            seeking_since.get_or_insert(since);
        }
        for event in games.hide_and_seek.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.compose(now)
}

/// A sack race at the Fairground with everyone running, `at` seconds after "go" (or as it ends,
/// if it is over by then).
fn race_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use fairground::sack_race::Phase;
    const SETTLE: f32 = 6.0;
    let (mut ground, mut games) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    let mut go = None;
    let mut started = false;
    while go.is_none_or(|since| now < since + at) && now < 400.0 {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        games.tick(&mut ground, now);
        if now >= SETTLE && !started {
            games.start(fairground::Game::SackRace, &mut ground, cast, &[], now);
            started = true;
        }
        if let Phase::Racing { since } = games.sack_race.phase() {
            go.get_or_insert(since);
        }
        for event in games.sack_race.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
        if started && games.playing().is_none() {
            break;
        }
    }
    ground.compose(now)
}

/// The colony taking turns at the high striker, `at` seconds after the game starts.
fn striker_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    const SETTLE: f32 = 6.0;
    let (mut ground, mut games) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    games.tick(&mut ground, now);
    while now < SETTLE + at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        games.tick(&mut ground, now);
        if now >= SETTLE && now - 1.0 / 30.0 < SETTLE {
            games.start(fairground::Game::HighStriker, &mut ground, cast, &[], now);
        }
        for event in games.high_striker.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.compose(now)
}

/// The colony taking turns at the hoopla stall, `at` seconds after the game starts.
fn hoopla_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    const SETTLE: f32 = 6.0;
    let (mut ground, mut games) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    while now < SETTLE + at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        games.tick(&mut ground, now);
        if now >= SETTLE && now - 1.0 / 30.0 < SETTLE {
            games.start(fairground::Game::Hoopla, &mut ground, cast, &[], now);
        }
        for event in games.hoopla.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.compose(now)
}

/// A tug-of-war at the Fairground, the colony sorting itself into sides, `at` seconds after the
/// bout starts.
fn tug_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    const SETTLE: f32 = 6.0;
    let (mut ground, mut games) = fairground::open(cast, 0.0);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    while now < SETTLE + at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        games.tick(&mut ground, now);
        if now >= SETTLE && now - 1.0 / 30.0 < SETTLE {
            games.start(fairground::Game::TugOfWar, &mut ground, cast, &[], now);
        }
        for event in games.tug_of_war.take_events() {
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

/// A bug hunt `at` seconds in, played by a patient hand: it goes after the nearest bug, creeps
/// up while it is out of reach, and swings only when the bug is open to the net.
fn bug_hunt_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use meadow::catching::{Hunt, Outset, Phase};
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut ground = meadow::open(cast, &party, 0.0, &sample_arrangement());
    ground.set_daylight(daylight);
    let outset = Outset {
        party: party.clone(),
        drought: 0,
        close_pair: false,
        influence: woods::influence(&sample_arrangement()),
        seed: 11,
    };
    let mut hunt = Hunt::new(&mut ground, outset, |_| false, |_| false, 0.0);
    hunt.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    while now < at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        if hunt.phase() == Phase::Hunting {
            if hunt.target().is_none() {
                let netter = ground.position(party[0]).unwrap_or((192.0, 180.0));
                let mut best: Option<(f32, f32)> = None;
                for y in (60..210).step_by(2) {
                    for x in (0..384).step_by(2) {
                        if hunt.bug_at(x as f32, y as f32).is_some() {
                            let here = (x as f32, y as f32);
                            if best.is_none_or(|b| {
                                playground::distance(b, netter) > playground::distance(here, netter)
                            }) {
                                best = Some(here);
                            }
                        }
                    }
                }
                if let Some((x, y)) = best {
                    hunt.choose(&mut ground, x, y, now);
                }
            }
            // Creeps while it is out of reach, and keeps still while the bug is jittery.
            hunt.hold(!hunt.in_reach(&ground) && hunt.nerve() < 0.55);
            if hunt.open_now(&ground, now) && now + 0.5 < at {
                hunt.swing(&mut ground, now);
            }
        }
        hunt.tick(&mut ground, now);
        for event in hunt.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.set_fliers(hunt.fliers(now));
    let mut scene = ground.compose(now);
    hunt.draw(&mut scene, &ground, now);
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

/// A foray along the hedgerow `at` seconds in, played by a canny hand: it goes wherever something
/// worth having is ripe, or soonest to be, picks only what is ripe, and puts back the least thing
/// in the basket to make room for something better.
fn foray_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use hedgerow::foraging::{Foray, Move, Outset};
    let family = cast
        .members
        .iter()
        .find_map(|member| member.parent().map(|parent| vec![parent, member.id]));
    let party: Vec<cast::Id> = family.unwrap_or_else(|| cast.ids().take(2).collect());
    let close_pair = cast
        .bond(party[0], party[party.len() - 1])
        .is_some_and(|bond| bond.warmth >= formiga_travel::Band::High);
    let mut lane = hedgerow::open(cast, &party, 0.0, &sample_arrangement());
    lane.set_daylight(daylight);
    let outset = Outset {
        party,
        drought: 0,
        close_pair,
        influence: woods::influence(&sample_arrangement()),
        seed: 9,
    };
    let mut foray = Foray::new(&mut lane, outset, |_| false, 0.0);
    foray.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    let mut hovered = None;
    while now < at {
        now += 1.0 / 30.0;
        lane.tick(cast, now);
        foray.tick(&mut lane, now);
        match foray.canny(&lane) {
            Move::Pick(item) => {
                hovered = Some(item);
                foray.pick(&mut lane, item, now);
            }
            Move::PutBack(slot) => foray.put_back(slot),
            Move::Go(patch) => foray.choose(&mut lane, patch, now),
            Move::Wait => {}
        }
        for event in foray.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    lane.set_fliers(foray.produce(now));
    let mut scene = lane.compose(now);
    foray.draw(&mut scene, hovered, now);
    scene
}

/// A scavenge along the old track `at` seconds in, played by a careful hand: it watches for
/// glints, works its way to them from the top down or eases things out, peeks and squeezes in
/// where the party can, and puts back the least thing in a full basket for something better. A
/// parent and its little one go if the colony has them, so squeezing in shows; otherwise the
/// first two.
fn track_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use track::scavenging::{Move, Outset, Scavenge};
    let family = cast
        .members
        .iter()
        .find_map(|member| member.parent().map(|parent| vec![parent, member.id]));
    let party: Vec<cast::Id> = family.unwrap_or_else(|| cast.ids().take(2).collect());
    let close_pair = cast
        .bond(party[0], party[party.len() - 1])
        .is_some_and(|bond| bond.warmth >= formiga_travel::Band::High);
    let mut ground = track::open(cast, &party, 0.0, &sample_arrangement());
    ground.set_daylight(daylight);
    let outset = Outset {
        party,
        drought: 0,
        close_pair,
        influence: woods::influence(&sample_arrangement()),
        maps_held: 0,
        seed: 13,
    };
    let mut scavenge = Scavenge::new(&mut ground, outset, |_| false, 0.0);
    scavenge.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    let mut hovered = None;
    while now < at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        scavenge.tick(&mut ground, now);
        match scavenge.canny() {
            Move::Go(heap) => scavenge.choose(&mut ground, heap, now),
            Move::Act { hand, item } => {
                hovered = scavenge.at().map(|heap| (heap, item));
                scavenge.set_hand(hand);
                scavenge.act(&mut ground, item, now);
            }
            Move::Take(hidden) => {
                if let Some(heap) = scavenge.at() {
                    scavenge.take(&mut ground, heap, hidden, now);
                }
            }
            Move::PutBack(slot) => scavenge.put_back(slot),
            Move::Home if now + 1.0 < at => scavenge.head_home(&mut ground, now),
            Move::Home | Move::Wait => {}
        }
        for event in scavenge.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.set_fliers(scavenge.props(now));
    let mut scene = ground.compose(now);
    scavenge.draw(&mut scene, hovered, now);
    scene
}

/// A treasure hunt `at` seconds in, the first two travellers following a map as a careful reader
/// would: the way the line names as far as it can be read, and an explorer's hunch among the ways
/// it might be; and in any nook a wrong way leads to, scavenging its heap a while before going
/// back.
fn treasure_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use track::scavenging::Move as Heap;
    use track::treasure::{Hunt, Move, Outset};
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut ground = track::hunt_ground(cast, &party, 0.0);
    ground.set_daylight(daylight);
    let outset = Outset {
        party,
        close_pair: false,
        influence: woods::influence(&sample_arrangement()),
        map: 3,
        drought: 0,
        seed: 17,
    };
    let mut hunt = Hunt::new(&mut ground, outset, |_| false, 0.0);
    hunt.set_hour_dark(daylight.darkness());
    for line in hunt.route().lines(false) {
        println!("map:     {line}");
    }
    let mut now = 0.0;
    let mut in_nook = None;
    while now < at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        hunt.tick(&mut ground, now);
        match hunt.canny() {
            Move::Back if now - *in_nook.get_or_insert(now) < 8.0 => {
                if let Some(nook) = hunt.nook_mut() {
                    match nook.canny() {
                        Heap::Act { hand, item } => {
                            nook.set_hand(hand);
                            nook.act(&mut ground, item, now);
                        }
                        Heap::Take(hidden) => nook.take(&mut ground, 0, hidden, now),
                        Heap::PutBack(slot) => nook.put_back(slot),
                        Heap::Go(_) | Heap::Home | Heap::Wait => {}
                    }
                }
            }
            Move::Back => {
                in_nook = None;
                hunt.back(&mut ground, now);
            }
            Move::Way(way) => hunt.choose(&mut ground, way, now),
            Move::Dig(spot) => hunt.dig(&mut ground, spot, now),
            Move::Home | Move::Wait => {}
        }
        for event in hunt.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    ground.set_fliers(hunt.props(now));
    let mut scene = ground.compose(now);
    hunt.draw(&mut scene, None, now);
    scene
}

/// An expedition `at` seconds in, played by a steady hand. A parent and its little one go with
/// whoever is boldest, if the colony has them, so the ways only some company opens show on the
/// map. From the edge of the Woods it rummages in the glade, scavenges a heap or two on the old
/// track, fishes at the pool, crosses the stepping stones up to the Far Falls and wades in there,
/// and comes down the steep way to rest on the fallen log. The Hilltop has no telescope, so the
/// falls are found out of the mist.
fn expedition_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use expedition::map::{FAR_FALLS, GLADE, LOG, OLD_TRACK, POOL};
    use expedition::{Expedition, Known, Outset, Phase};
    let family = cast
        .members
        .iter()
        .find_map(|member| member.parent().map(|parent| vec![parent, member.id]));
    let mut party: Vec<cast::Id> = family.unwrap_or_else(|| cast.ids().take(1).collect());
    let boldest = cast
        .members
        .iter()
        .filter(|member| !party.contains(&member.id))
        .max_by(|a, b| a.axes().boldness.total_cmp(&b.axes().boldness))
        .map(|member| member.id);
    party.insert(0, boldest.unwrap_or(party[0]));
    party.dedup();
    let mut hilltop = sample_arrangement();
    hilltop.retain(|_, standing| !standing.finds().contains(&"brass_lens"));
    let known = Known {
        hilltop: hilltop.clone(),
        ..Known::default()
    };
    let outset = Outset {
        party,
        influence: woods::influence(&hilltop),
        seed: 3,
    };
    let mut trip = Expedition::new(cast, outset, known, 0.0);
    trip.set_daylight(daylight);
    let route = [GLADE, OLD_TRACK, POOL, FAR_FALLS, LOG];
    let mut next = 0;
    let mut now = 0.0;
    while now < at {
        now += 1.0 / 30.0;
        let input = trip
            .leg_mut()
            .map(|leg| leg.steady(now))
            .unwrap_or_default();
        trip.tick(cast, input, now);
        match trip.phase() {
            Phase::Choosing if next < route.len() => {
                if trip.at() == route[next] {
                    next += 1;
                    trip.stop(cast, now);
                } else {
                    trip.go(route[next], now);
                }
            }
            Phase::Choosing => trip.head_home(now),
            Phase::MakingRoom => {
                trip.leave_behind(0, now);
            }
            _ => {}
        }
        for event in trip.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    trip.compose(now, None)
}

/// A visit to the Far Falls `at` seconds in, played by a steady hand: it catches whatever comes
/// down as the eddy brings it past the wading stone.
fn falls_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    use falls::wading::{Outset, Wading};
    let party: Vec<cast::Id> = cast.ids().take(2).collect();
    let mut pool = falls::open(cast, &party, 0.0);
    pool.set_daylight(daylight);
    let outset = Outset {
        party,
        drought: 0,
        close_pair: false,
        influence: woods::influence(&sample_arrangement()),
        seed: 7,
    };
    let mut visit = Wading::new(&mut pool, outset, |_| false, 0.0);
    visit.set_hour_dark(daylight.darkness());
    let mut now = 0.0;
    while now < at {
        now += 1.0 / 30.0;
        pool.tick(cast, now);
        visit.tick(&mut pool, now);
        if visit.at_the_stone(now) && now + 0.5 < at {
            visit.strike(&mut pool, now);
        }
        for event in visit.take_events() {
            println!("{now:6.1}s  {event:?}");
        }
    }
    let mut scene = pool.compose(now);
    visit.draw(&mut scene, now);
    scene
}

/// Everything the hedgerow grows, a row each, at each stage of ripeness: unripe, turning, ripe and
/// over, on a strip of hedge green and close up.
fn produce_sheet() -> Canvas {
    use hedgerow::Plant;
    use hedgerow::produce::{Stage, sprite};
    const ROW: i32 = 17;
    let mut sheet = Canvas::new(384, (ROW * Plant::ALL.len() as i32) as u32);
    for (row, plant) in Plant::ALL.into_iter().enumerate() {
        let top = row as i32 * ROW;
        let shade = if row % 2 == 0 { 0x4c6e3a } else { 0x557a40 };
        sheet.fill_rect(0, top, 384, ROW, paint::rgb(shade));
        paint::blit(&mut sheet, &finds::art::icon(plant.find().id), 4, top + 4);
        for (column, stage) in Stage::ALL.into_iter().enumerate() {
            let x = 20 + column as i32 * 18;
            paint::blit(&mut sheet, &sprite(plant, stage, false), x, top + 2);
            // And on pale ground, to see its own outline.
            let pale = 110 + column as i32 * 18;
            sheet.fill_rect(pale - 1, top + 1, 15, 15, paint::rgb(0xe8e0c8));
            paint::blit(
                &mut sheet,
                &sprite(plant, stage, stage == Stage::Ripe),
                pale,
                top + 2,
            );
        }
    }
    sheet
}

/// A spread of finds over the Hilltop, some of them still growing and some built into something
/// grander, with a spot or two left open, for seeing it lived in.
fn sample_arrangement() -> hilltop::Arrangement {
    let picks = [
        (0, "weathervane"),
        (2, "brass_lens"),
        (4, "sun_coin"),
        (5, "acorn_stash"),
        (6, "wild_berries"),
        (9, "fallen_star"),
        (13, "wooden_duck"),
        (15, "tangled_kite"),
    ];
    let growing = [
        (3, "bluebell_bulb", 1),
        (10, "sycamore_key", 2),
        (12, "dandelion_clock", 0),
        (16, "silk_cocoon", 1),
    ];
    let built = [
        (1, "bandstand"),
        (8, "picnic_table"),
        (11, "burrow_house"),
        (14, "grand_cairn"),
        (17, "lantern_tree"),
    ];
    picks
        .into_iter()
        .map(|(spot, id)| (spot, hilltop::Standing::from(id)))
        .chain(growing.into_iter().map(|(spot, id, stage)| {
            let planted = hilltop::Standing::Planted {
                planted: id.to_owned(),
                stage,
            };
            (spot, planted)
        }))
        .chain(
            built
                .into_iter()
                .map(|(spot, plan)| (spot, hilltop::Standing::built(plan))),
        )
        .collect()
}

/// Every find on one sheet: its icon in the corner, its Hilltop piece standing on a patch of
/// summit grass, a row for each kind.
fn finds_sheet() -> Canvas {
    const CELL: (i32, i32) = (56, 70);
    let columns = 7;
    // A row for each kind, a row for the relics, the hedgerow's finds, the old track's, and the
    // finds from afar.
    let rows: Vec<Vec<&finds::Find>> = finds::Kind::ALL
        .into_iter()
        .map(|kind| {
            finds::CATALOGUE
                .iter()
                .filter(|find| find.kind == kind)
                .collect()
        })
        .chain(std::iter::once(finds::RELICS.iter().collect()))
        .chain(
            finds::FORAGED
                .chunks(columns as usize)
                .map(|row| row.iter().collect()),
        )
        .chain(
            finds::SCAVENGED
                .chunks(columns as usize)
                .map(|row| row.iter().collect()),
        )
        .chain(std::iter::once(finds::AFAR.iter().collect()))
        .collect();
    let mut sheet = Canvas::new(
        (CELL.0 * columns) as u32,
        (CELL.1 * rows.len() as i32) as u32,
    );
    for (row, of_kind) in rows.into_iter().enumerate() {
        for (column, find) in of_kind.into_iter().enumerate() {
            let (left, top) = (column as i32 * CELL.0, row as i32 * CELL.1);
            on_the_grass(&mut sheet, (left, top), CELL, &finds::art::piece(find.id));
            paint::blit(&mut sheet, &finds::art::icon(find.id), left + 2, top + 2);
        }
    }
    sheet
}

/// A piece standing on a patch of summit grass under a pale sky, in a cell of a review sheet.
fn on_the_grass(
    sheet: &mut Canvas,
    (left, top): (i32, i32),
    cell: (i32, i32),
    piece: &finds::art::Piece,
) {
    for y in 0..cell.1 {
        let t = y as f32 / cell.1 as f32;
        let color = if y < cell.1 - 12 {
            paint::mix(paint::rgb(0xd6ecf2), paint::rgb(0xf6e8cf), t)
        } else {
            paint::mix(paint::rgb(0x86bb7c), paint::rgb(0x639858), t)
        };
        sheet.fill_rect(left, top + y, cell.0, 1, color);
    }
    sheet.fill_rect(left + cell.0 - 1, top, 1, cell.1, paint::rgb(0x9ab0a0));
    sheet.fill_rect(left, top + cell.1 - 1, cell.0, 1, paint::rgb(0x9ab0a0));
    let base = (left + cell.0 / 2, top + cell.1 - 6);
    paint::blit(
        sheet,
        &piece.sprite,
        base.0 - piece.anchor.0,
        base.1 - piece.anchor.1,
    );
}

/// The colony building a wishing well on the Hilltop, `at` seconds after it starts, once everyone
/// has come up and settled: gathering round, at work, the puff it appears in, and the cheer.
fn building_moment(cast: &Cast, at: f32, daylight: daylight::Daylight) -> Canvas {
    const SETTLE: f32 = 12.0;
    const SPOT: u8 = 7;
    let mut arrangement = sample_arrangement();
    arrangement.remove(&SPOT);
    let mut ground = hilltop::open(cast, 0.0, &arrangement);
    ground.set_daylight(daylight);
    let mut now = 0.0;
    while now < SETTLE {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
    }
    let mut building = hilltop::building::Building::begin(&mut ground, SPOT, now);
    while now < SETTLE + at {
        now += 1.0 / 30.0;
        ground.tick(cast, now);
        if let Some(moment) = building.tick(&mut ground, now) {
            println!("{:6.1}s  {moment:?}", now - SETTLE);
            if moment == hilltop::building::Moment::Appeared {
                arrangement.insert(SPOT, hilltop::Standing::built("wishing_well"));
                ground.set_props(hilltop::props(&arrangement));
            }
        }
    }
    let mut scene = ground.compose(now);
    building.draw(&mut scene, now);
    scene
}

/// Every plan built, five to a row on the summit's grass, with the finds it takes along the top.
fn plans_sheet() -> Canvas {
    const CELL: (i32, i32) = (62, 76);
    let plans = &finds::plans::PLANS;
    let columns = 5;
    let rows = plans.len().div_ceil(columns) as i32;
    let mut sheet = Canvas::new((CELL.0 * columns as i32) as u32, (CELL.1 * rows) as u32);
    for (index, plan) in plans.iter().enumerate() {
        let left = (index % columns) as i32 * CELL.0;
        let top = (index / columns) as i32 * CELL.1;
        on_the_grass(&mut sheet, (left, top), CELL, &finds::art::built(plan.id));
        for (at, id) in plan.finds().into_iter().enumerate() {
            let icon = finds::art::icon(id);
            paint::blit(&mut sheet, &icon, left + 2 + at as i32 * 10, top + 2);
        }
    }
    sheet
}

/// Everything that grows, two to a row: each stage from just planted to its full piece, left to
/// right, with the find it was planted from in the corner of the first.
fn growing_sheet() -> Canvas {
    const CELL: (i32, i32) = (56, 70);
    let growing = finds::growing::GROWING;
    let most = growing
        .iter()
        .map(|growth| growth.stages.len() + 1)
        .max()
        .unwrap_or(1) as i32;
    let rows = growing.len().div_ceil(2) as i32;
    let mut sheet = Canvas::new((CELL.0 * most * 2 + 4) as u32, (CELL.1 * rows) as u32);
    sheet.fill_rect(CELL.0 * most, 0, 4, CELL.1 * rows, paint::rgb(0x4a6a50));
    for (index, growth) in growing.iter().enumerate() {
        let left = (index % 2) as i32 * (CELL.0 * most + 4);
        let top = (index / 2) as i32 * CELL.1;
        let stages = (0..growth.stages.len() as u8)
            .map(|stage| finds::art::stage(growth.id, stage))
            .chain(std::iter::once(finds::art::piece(growth.id)));
        for (column, piece) in stages.enumerate() {
            on_the_grass(
                &mut sheet,
                (left + column as i32 * CELL.0, top),
                CELL,
                &piece,
            );
        }
        paint::blit(&mut sheet, &finds::art::icon(growth.id), left + 2, top + 2);
    }
    sheet
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
