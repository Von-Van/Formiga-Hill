//! A leg of an expedition: a short go at a stop's own activity, on the expedition's one light and
//! with its one basket. Each is the activity itself, started partway through the expedition's
//! day with what is in the basket (see each one's `partway`), cut short: a few searches in the
//! glade, a cast or two at the pool, one bug in the meadow, one patch of the hedgerow, what the
//! Far Falls bring down, a rest on the log. It hands back the light left and the basket as it
//! now is, and anything caught and let go.
//!
//! Whoever is best at what is done at a stop leads there, read from who each is, with the next
//! best beside it (and anyone who opens something there first); anyone else in the party comes
//! along and is free to potter about. At the falls and on the log, everyone joins in.
//!
//! A new kind of stop joins here as a `Play`, with how it starts, ends, is played and drawn.

use super::map::Stop;
use super::picnic::{self, Picnic};
use crate::cast::{Cast, Id};
use crate::character::Character;
use crate::daylight::Daylight;
use crate::falls::{self, wading::Wading};
use crate::finds::Kind;
use crate::fishing::{self, angling::Angling};
use crate::hedgerow::{self, foraging::Foray};
use crate::hilltop::Arrangement;
use crate::meadow::{self, catching::Hunt};
use crate::playground::{Playground, distance};
use crate::woods::rummage::{self, Rummage};
use crate::woods::{self, Influence};
use formiga_art::Canvas;
use formiga_core::Habit;

/// How many spots a leg in the glade searches, and how many casts a leg at the pool makes.
pub const SEARCHES: u32 = 3;
pub const CASTS: u32 = 2;

/// What is going on at a stop.
pub enum Play {
    Rummage { rummage: Rummage, searched: u32 },
    Fish { angling: Angling, casts: u32 },
    Bugs { hunt: Hunt, caught: bool },
    Forage { foray: Foray },
    Rest { picnic: Picnic },
    Falls { wading: Wading },
}

/// Something that happened in a leg, as the activity tells it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegEvent {
    Rummage(rummage::Event),
    Fish(fishing::angling::Event),
    Bugs(meadow::catching::Event),
    Forage(hedgerow::foraging::Event),
    Rest(picnic::Event),
    Falls(falls::wading::Event),
}

/// Something caught and let go: what kind, how big, and who caught it.
pub type Caught = (&'static str, f32, Id);

/// What a leg is played with: what the person is holding down, and for how long since the last
/// frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// The pointer or Space held: the catching marker turned with reduced motion, or a line
    /// reeled in.
    pub holding: bool,
    /// The pointer, ↑ or W held: creeping up on a bug.
    pub creeping: bool,
    pub dt: f32,
}

/// What a leg starts with: who goes, the expedition's day and basket, and what the colony
/// knows already.
pub struct Start<'a> {
    pub place: usize,
    pub stop: Stop,
    /// The whole party, and who plays the leg, in order.
    pub everyone: &'a [Id],
    pub players: Vec<Id>,
    pub close_pair: bool,
    pub light: f32,
    pub full: f32,
    pub basket: Vec<&'static str>,
    pub influence: &'a Influence,
    pub droughts: super::Droughts,
    pub found: &'a dyn Fn(&str) -> bool,
    pub fish: &'a dyn Fn(&str) -> bool,
    pub bugs: &'a dyn Fn(&str) -> bool,
    pub hilltop: &'a Arrangement,
    /// The map as it is painted, for the picnic's picture to be pinned over.
    pub map: &'a Canvas,
    pub daylight: Daylight,
    pub seed: u64,
}

pub struct Leg {
    pub place: usize,
    /// Who leads it: whoever finds anything here found it.
    pub leader: Id,
    pub ground: Playground,
    pub play: Play,
    /// The basket as it was carried in, and the light then: a rest takes only what it costs,
    /// which the expedition has taken already.
    carried_in: Vec<&'static str>,
    light_in: f32,
}

impl Leg {
    pub fn start(cast: &Cast, start: Start, now: f32) -> Option<Self> {
        let leader = *start.players.first()?;
        let influence = start.influence.clone();
        let (players, seed) = (start.players.clone(), start.seed);
        let (light, full, basket) = (start.light, start.full, start.basket.clone());
        let (ground, play) = match start.stop {
            Stop::Rummage => {
                let mut ground = woods::open(cast, start.everyone, now);
                let outset = rummage::Outset {
                    party: players,
                    drought: start.droughts.rummage,
                    close_pair: start.close_pair,
                    influence,
                    // The glint only ever shows to an ordinary outing.
                    beckons: false,
                    seed,
                };
                let rummage = Rummage::new(&mut ground, outset, start.found, now)
                    .partway(light, full, basket);
                (
                    ground,
                    Play::Rummage {
                        rummage,
                        searched: 0,
                    },
                )
            }
            Stop::Fish => {
                let mut ground = fishing::open(cast, start.everyone, now);
                let outset = fishing::angling::Outset {
                    party: players,
                    drought: start.droughts.fish,
                    influence,
                    seed,
                };
                let angling = Angling::new(&mut ground, outset, start.fish, start.found, now)
                    .partway(light, full);
                (ground, Play::Fish { angling, casts: 0 })
            }
            Stop::Bugs => {
                let mut ground = meadow::open(cast, start.everyone, now, start.hilltop);
                let outset = meadow::catching::Outset {
                    party: players,
                    drought: start.droughts.bugs,
                    close_pair: start.close_pair,
                    influence,
                    seed,
                };
                let hunt = Hunt::new(&mut ground, outset, start.bugs, start.found, now)
                    .partway(light, full);
                (
                    ground,
                    Play::Bugs {
                        hunt,
                        caught: false,
                    },
                )
            }
            Stop::Forage => {
                let mut ground = hedgerow::open(cast, start.everyone, now, start.hilltop);
                let outset = hedgerow::foraging::Outset {
                    party: players,
                    drought: start.droughts.forage,
                    close_pair: start.close_pair,
                    influence,
                    seed,
                };
                let foray =
                    Foray::new(&mut ground, outset, start.found, now).partway(light, full, basket);
                (ground, Play::Forage { foray })
            }
            Stop::Falls => {
                let mut ground = falls::open(cast, start.everyone, now);
                let outset = falls::wading::Outset {
                    party: players,
                    drought: start.droughts.far,
                    close_pair: start.close_pair,
                    influence,
                    seed,
                };
                let wading =
                    Wading::new(&mut ground, outset, start.found, now).partway(light, full, basket);
                (ground, Play::Falls { wading })
            }
            Stop::Rest => {
                let mut ground = picnic::open(cast, start.everyone, start.map, now);
                let picnic = Picnic::new(&mut ground, cast, &players, seed, now);
                (ground, Play::Rest { picnic })
            }
            Stop::Edge | Stop::Fork => return None,
        };
        let mut leg = Self {
            place: start.place,
            leader,
            ground,
            play,
            carried_in: start.basket,
            light_in: start.light,
        };
        leg.set_daylight(start.daylight);
        Some(leg)
    }

    /// Plays the leg on, and says what happened.
    pub fn tick(&mut self, cast: &Cast, input: Input, now: f32) -> Vec<LegEvent> {
        let ground = &mut self.ground;
        ground.tick(cast, now);
        match &mut self.play {
            Play::Rummage { rummage, searched } => {
                rummage.hold(ground, input.holding, input.dt, now);
                rummage.tick(ground, now);
                let events = rummage.take_events();
                *searched += events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event,
                            rummage::Event::Got { .. } | rummage::Event::Nothing { .. }
                        )
                    })
                    .count() as u32;
                if *searched >= SEARCHES && rummage.phase() == rummage::Phase::Exploring {
                    rummage.head_home(ground, now);
                }
                events.into_iter().map(LegEvent::Rummage).collect()
            }
            Play::Fish { angling, casts } => {
                angling.hold(input.holding);
                angling.tick(ground, now);
                if *casts >= CASTS && angling.phase() == fishing::angling::Phase::Ready {
                    angling.head_home(ground, now);
                }
                angling
                    .take_events()
                    .into_iter()
                    .map(LegEvent::Fish)
                    .collect()
            }
            Play::Bugs { hunt, caught } => {
                hunt.hold(input.creeping);
                hunt.tick(ground, now);
                let events = hunt.take_events();
                *caught |= events
                    .iter()
                    .any(|event| matches!(event, meadow::catching::Event::Caught { .. }));
                if *caught && hunt.phase() == meadow::catching::Phase::Hunting {
                    hunt.head_home(ground, now);
                }
                events.into_iter().map(LegEvent::Bugs).collect()
            }
            Play::Forage { foray } => {
                foray.tick(ground, now);
                foray
                    .take_events()
                    .into_iter()
                    .map(LegEvent::Forage)
                    .collect()
            }
            Play::Rest { picnic } => {
                picnic.tick(now);
                picnic
                    .take_events()
                    .into_iter()
                    .map(LegEvent::Rest)
                    .collect()
            }
            Play::Falls { wading } => {
                wading.hold(ground, input.holding, input.dt, now);
                wading.tick(ground, now);
                wading
                    .take_events()
                    .into_iter()
                    .map(LegEvent::Falls)
                    .collect()
            }
        }
    }

    /// Whether the leg is over, and the party back on the map.
    pub fn over(&self) -> bool {
        match &self.play {
            Play::Rummage { rummage, .. } => rummage.phase() == rummage::Phase::Over,
            Play::Fish { angling, .. } => angling.phase() == fishing::angling::Phase::Over,
            Play::Bugs { hunt, .. } => hunt.phase() == meadow::catching::Phase::Over,
            Play::Forage { foray } => foray.phase() == hedgerow::foraging::Phase::Over,
            Play::Rest { picnic } => picnic.over(),
            Play::Falls { wading } => wading.phase() == falls::wading::Phase::Over,
        }
    }

    /// The light left, on the expedition's scale.
    pub fn light(&self) -> f32 {
        match &self.play {
            Play::Rummage { rummage, .. } => rummage.light(),
            Play::Fish { angling, .. } => angling.light(),
            Play::Bugs { hunt, .. } => hunt.light(),
            Play::Forage { foray } => foray.light(),
            Play::Rest { .. } => self.light_in,
            Play::Falls { wading } => wading.light(),
        }
    }

    /// The basket as it is now: what was carried in, less anything put back, and what this leg
    /// added.
    pub fn basket(&self) -> Vec<&'static str> {
        match &self.play {
            Play::Rummage { rummage, .. } => rummage.basket().to_vec(),
            Play::Forage { foray } => foray.basket().to_vec(),
            Play::Falls { wading } => wading.basket().to_vec(),
            Play::Fish { angling, .. } => [self.carried_in.as_slice(), angling.basket()].concat(),
            Play::Bugs { hunt, .. } => [self.carried_in.as_slice(), hunt.basket()].concat(),
            Play::Rest { .. } => self.carried_in.clone(),
        }
    }

    /// What was caught and let go: fish landed, or bugs netted, each with its size and who
    /// caught it.
    pub fn caught(&self) -> (Vec<Caught>, Vec<Caught>) {
        match &self.play {
            Play::Fish { angling, .. } => (
                angling
                    .creel()
                    .iter()
                    .map(|(id, length)| (*id, *length, self.leader))
                    .collect(),
                Vec::new(),
            ),
            Play::Bugs { hunt, .. } => (
                Vec::new(),
                hunt.jar()
                    .iter()
                    .map(|(id, size)| (*id, *size, self.leader))
                    .collect(),
            ),
            _ => (Vec::new(), Vec::new()),
        }
    }

    /// A click in the leg's scene, as the activity takes it, kept to the leg: at the pool a cast
    /// or two, at the hedgerow one patch.
    pub fn click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let ground = &mut self.ground;
        let reduce_motion = ground.reduce_motion();
        match &mut self.play {
            Play::Rummage { rummage, .. } => {
                let spot = pointer.and_then(|(x, y)| rummage.spot_at(x, y));
                match rummage.phase() {
                    rummage::Phase::Catching(catch)
                        if spot.is_none_or(|spot| spot == catch.spot) =>
                    {
                        if !reduce_motion {
                            rummage.strike(ground, now);
                        }
                    }
                    rummage::Phase::Exploring | rummage::Phase::Catching(_) => {
                        if let Some(spot) = spot {
                            rummage.choose(ground, spot, now);
                        }
                    }
                    _ => {}
                }
            }
            Play::Fish { angling, casts } => match angling.phase() {
                fishing::angling::Phase::Ready if *casts < CASTS => {
                    if let Some(at) = pointer {
                        angling.cast(ground, at, now);
                        if angling.phase() != fishing::angling::Phase::Ready {
                            *casts += 1;
                        }
                    }
                }
                fishing::angling::Phase::Waiting { .. } => angling.strike(ground, now),
                _ => {}
            },
            Play::Bugs { hunt, .. } => {
                let Some((x, y)) = pointer else {
                    return;
                };
                let chosen = hunt.target().map(|(_, at)| at);
                let on_chosen = chosen.is_some_and(|at| distance(at, (x, y)) <= 9.0);
                if on_chosen && hunt.in_reach(ground) {
                    hunt.swing(ground, now);
                } else {
                    hunt.choose(ground, x, y, now);
                }
            }
            Play::Forage { foray } => {
                let Some((x, y)) = pointer else {
                    return;
                };
                if let Some(slot) = foray.basket_slot_at(x, y, ground.backdrop().width()) {
                    foray.put_back(slot);
                    return;
                }
                if let Some(item) = foray.item_at(x, y)
                    && let Some((_, patch, _)) = foray.item(item)
                    && foray.at() == Some(patch)
                {
                    foray.pick(ground, item, now);
                    return;
                }
                // One patch a stop: somewhere to go, until the party is at one.
                if foray.at().is_none()
                    && let Some(patch) = foray.patch_at(x, y)
                {
                    foray.choose(ground, patch, now);
                }
            }
            Play::Rest { .. } => {}
            Play::Falls { wading } => {
                if !reduce_motion {
                    wading.strike(ground, now);
                }
            }
        }
    }

    /// The key for trying, striking or swinging, for anyone not using the pointer.
    pub fn strike(&mut self, now: f32) {
        let ground = &mut self.ground;
        let reduce_motion = ground.reduce_motion();
        match &mut self.play {
            Play::Rummage { rummage, .. } => {
                if matches!(rummage.phase(), rummage::Phase::Catching(_)) && !reduce_motion {
                    rummage.strike(ground, now);
                }
            }
            Play::Fish { angling, .. } => {
                if matches!(angling.phase(), fishing::angling::Phase::Waiting { .. }) {
                    angling.strike(ground, now);
                }
            }
            Play::Bugs { hunt, .. } => hunt.swing(ground, now),
            Play::Falls { wading } => {
                if !reduce_motion {
                    wading.strike(ground, now);
                }
            }
            Play::Forage { .. } | Play::Rest { .. } => {}
        }
    }

    /// The person would rather move on: the party finishes up here and goes back to the map
    /// with whatever is in the basket.
    pub fn move_on(&mut self, now: f32) {
        let ground = &mut self.ground;
        match &mut self.play {
            Play::Rummage { rummage, .. } => rummage.head_home(ground, now),
            Play::Fish { angling, .. } => angling.head_home(ground, now),
            Play::Bugs { hunt, .. } => hunt.head_home(ground, now),
            Play::Forage { foray } => foray.head_home(ground, now),
            Play::Rest { picnic } => picnic.move_on(),
            Play::Falls { wading } => wading.head_home(ground, now),
        }
    }

    /// Whether the party is already on its way back to the map.
    pub fn leaving(&self) -> bool {
        match &self.play {
            Play::Rummage { rummage, .. } => matches!(
                rummage.phase(),
                rummage::Phase::Leaving { .. } | rummage::Phase::Over
            ),
            Play::Fish { angling, .. } => matches!(
                angling.phase(),
                fishing::angling::Phase::Leaving { .. } | fishing::angling::Phase::Over
            ),
            Play::Bugs { hunt, .. } => matches!(
                hunt.phase(),
                meadow::catching::Phase::Leaving { .. } | meadow::catching::Phase::Over
            ),
            Play::Forage { foray } => matches!(
                foray.phase(),
                hedgerow::foraging::Phase::Leaving { .. } | hedgerow::foraging::Phase::Over
            ),
            Play::Rest { picnic } => picnic.over(),
            Play::Falls { wading } => matches!(
                wading.phase(),
                falls::wading::Phase::Leaving { .. } | falls::wading::Phase::Over
            ),
        }
    }

    /// What a steady hand would do now, for the renders and the tests: searches the glade's
    /// spots in turn and catches each moment on the gold; casts round the pool's haunts, strikes
    /// on the bite and reels in only while the fish isn't pulling; creeps up on the nearest bug
    /// while it is calm and swings when it is open; goes where something is ripening and picks
    /// it ripe; and catches what the falls bring down as it comes past the stone. Says what it
    /// is holding down.
    pub fn steady(&mut self, now: f32) -> Input {
        let ground = &mut self.ground;
        let mut input = Input {
            dt: 1.0 / 30.0,
            ..Input::default()
        };
        match &mut self.play {
            Play::Rummage { rummage, searched } => match rummage.phase() {
                rummage::Phase::Exploring if *searched < SEARCHES => {
                    if let Some(spot) =
                        (0..rummage.spot_count()).find(|spot| !rummage.searched(*spot))
                    {
                        rummage.choose(ground, spot, now);
                    }
                }
                rummage::Phase::Catching(_) if rummage.on_the_gold(now) => {
                    rummage.strike(ground, now);
                }
                _ => {}
            },
            Play::Fish { angling, casts } => match angling.phase() {
                fishing::angling::Phase::Ready if *casts < CASTS => {
                    let haunts = fishing::Haunt::ALL;
                    let ((x, y), r) = haunts[*casts as usize % haunts.len()].area();
                    angling.cast(ground, (x + r * 0.5, y), now);
                    if angling.phase() != fishing::angling::Phase::Ready {
                        *casts += 1;
                    }
                }
                fishing::angling::Phase::Waiting { .. } if angling.biting() => {
                    angling.strike(ground, now);
                }
                fishing::angling::Phase::Waiting { since, .. } if now - since > 12.0 => {
                    angling.strike(ground, now);
                }
                fishing::angling::Phase::Fighting(fight) => {
                    input.holding = !fight.pulling(now) && fight.strain < 0.6;
                }
                _ => {}
            },
            Play::Bugs { hunt, .. } => {
                if hunt.phase() == meadow::catching::Phase::Hunting {
                    if hunt.target().is_none() {
                        let netter = hunt
                            .party()
                            .first()
                            .and_then(|id| ground.position(*id))
                            .unwrap_or((192.0, 180.0));
                        let nearest = (60..210)
                            .step_by(3)
                            .flat_map(|y| (0..384).step_by(3).map(move |x| (x as f32, y as f32)))
                            .filter(|(x, y)| hunt.bug_at(*x, *y).is_some())
                            .min_by(|a, b| distance(*a, netter).total_cmp(&distance(*b, netter)));
                        if let Some((x, y)) = nearest {
                            hunt.choose(ground, x, y, now);
                        }
                    }
                    input.creeping = !hunt.in_reach(ground) && hunt.nerve() < 0.55;
                    if hunt.open_now(ground, now) {
                        hunt.swing(ground, now);
                    }
                }
            }
            Play::Forage { foray } => {
                use hedgerow::foraging::{Move, Phase};
                match (foray.canny(ground), foray.phase()) {
                    (Move::Pick(item), _) => foray.pick(ground, item, now),
                    (Move::PutBack(slot), _) => foray.put_back(slot),
                    (Move::Go(patch), Phase::Choosing) => foray.choose(ground, patch, now),
                    // One patch a stop: once nothing more is worth waiting for here, on.
                    (Move::Go(_), Phase::At { .. }) => foray.head_home(ground, now),
                    _ => {}
                }
            }
            Play::Rest { .. } => {}
            Play::Falls { wading } => {
                if wading.at_the_stone(now) {
                    wading.strike(ground, now);
                }
            }
        }
        input
    }

    /// The light of the hour, on the leg's place and its own dusk.
    pub fn set_daylight(&mut self, daylight: Daylight) {
        self.ground.set_daylight(daylight);
        let dark = daylight.darkness();
        match &mut self.play {
            Play::Rummage { rummage, .. } => rummage.set_hour_dark(dark),
            Play::Fish { angling, .. } => angling.set_hour_dark(dark),
            Play::Bugs { hunt, .. } => hunt.set_hour_dark(dark),
            Play::Forage { foray } => foray.set_hour_dark(dark),
            Play::Falls { wading } => wading.set_hour_dark(dark),
            Play::Rest { .. } => {}
        }
    }

    /// The leg's scene as it is now; `pointer` marks whatever is under it along the hedgerow.
    pub fn compose(&mut self, now: f32, pointer: Option<(f32, f32)>) -> Canvas {
        let ground = &mut self.ground;
        match &mut self.play {
            Play::Rummage { rummage, .. } => {
                let mut scene = ground.compose(now);
                rummage.draw(&mut scene, now);
                scene
            }
            Play::Fish { angling, .. } => {
                let mut scene = ground.compose(now);
                angling.draw(&mut scene, now);
                super::draw_carried(&mut scene, &self.carried_in, angling.basket());
                scene
            }
            Play::Bugs { hunt, .. } => {
                ground.set_fliers(hunt.fliers(now));
                let mut scene = ground.compose(now);
                hunt.draw(&mut scene, ground, now);
                super::draw_carried(&mut scene, &self.carried_in, hunt.basket());
                scene
            }
            Play::Forage { foray } => {
                ground.set_fliers(foray.produce(now));
                let mut scene = ground.compose(now);
                let hovered = pointer.and_then(|(x, y)| foray.item_at(x, y));
                foray.draw(&mut scene, hovered, now);
                scene
            }
            Play::Rest { .. } => ground.compose(now),
            Play::Falls { wading } => {
                let mut scene = ground.compose(now);
                wading.draw(&mut scene, now);
                scene
            }
        }
    }
}

/// Who plays a leg at `stop`, in order, from the party: at the falls and the log everyone, the
/// one best with water wading in first; anywhere else whoever is best at what is done there leads,
/// and beside it whoever opens something there, or else the next best.
pub fn players(stop: Stop, party: &[(Id, Character)]) -> Vec<Id> {
    let score = |character: &Character| -> f32 {
        let a = character.axes;
        match stop {
            Stop::Rummage => Kind::ALL
                .into_iter()
                .map(|kind| rummage::knack(character, kind))
                .fold(0.0, f32::max),
            Stop::Fish => 1.0 - a.impulsiveness,
            Stop::Bugs => 0.5 * (1.0 - a.energy) + 0.5 * (1.0 - a.impulsiveness),
            Stop::Forage => {
                1.0 - a.impulsiveness
                    + if character.habits.contains(&Habit::LooksFoodOver) {
                        0.2
                    } else {
                        0.0
                    }
            }
            Stop::Falls => rummage::knack(character, Kind::Scoop),
            Stop::Rest | Stop::Edge | Stop::Fork => 0.0,
        }
    };
    // Something only they open here: a little one's crevice or tucked-in places, an explorer's
    // badger sett, a bold one's high branches.
    let opens = |character: &Character| match stop {
        Stop::Rummage => {
            character.parent.is_some()
                || character.kind == formiga_core::TemperamentKind::Explorer
                || character.axes.curiosity >= 0.8
        }
        Stop::Forage => character.parent.is_some() || character.axes.boldness >= 0.65,
        _ => false,
    };
    let mut ranked: Vec<&(Id, Character)> = party.iter().collect();
    ranked.sort_by(|a, b| score(&b.1).total_cmp(&score(&a.1)));
    if matches!(stop, Stop::Falls | Stop::Rest) {
        return ranked.into_iter().map(|(id, _)| *id).collect();
    }
    let mut chosen: Vec<Id> = ranked.first().map(|(id, _)| *id).into_iter().collect();
    let second = ranked
        .iter()
        .skip(1)
        .find(|(_, character)| opens(character))
        .or_else(|| ranked.get(1));
    chosen.extend(second.map(|(id, _)| *id));
    chosen
}
