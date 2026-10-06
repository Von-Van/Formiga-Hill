//! The map of the Woods an expedition is planned on and walked across: the places it can go, the
//! paths between them and how long each is, and the ways only some company takes.
//!
//! Every place can be reached by anyone on their own, by the open paths. The rest are extras,
//! read from who came: an explorer knows a deer track, a little one squeezes through a gap in the
//! hedge, a close pair crosses the stepping stones together, a bold one takes the steep way up
//! the crag, and one who reads old things makes out the old way on the signpost. None of them
//! goes anywhere the open paths don't; they only get there for less of the day.
//!
//! A new place joins the map as a `Place` with its `Stop`, its paths in `PATHS`, and its picture
//! in `scenery`; a new kind of stop needs its leg in `legs` too.

use crate::cast::Id;
use crate::character::Character;
use crate::finds::Leaning;
use formiga_core::TemperamentKind;

/// What there is to do at a place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stop {
    /// Where an expedition sets out from: the lane in from the Hill.
    Edge,
    /// A short leg of one of the Woods' own activities.
    Rummage,
    Fish,
    Bugs,
    Forage,
    Scavenge,
    /// A picnic on the fallen log.
    Rest,
    /// Only a fork in the way, where the old signpost stands.
    Fork,
    /// The Far Falls, which only an expedition reaches.
    Falls,
}

impl Stop {
    /// Whether the party can stop and do something here.
    pub fn played(self) -> bool {
        !matches!(self, Self::Edge | Self::Fork)
    }
}

/// A place on the map.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    /// What memories call it.
    pub id: &'static str,
    /// What the person calls it.
    pub name: &'static str,
    pub stop: Stop,
    /// Where on the map the party stands there.
    pub at: (f32, f32),
    /// How far into the Woods it is: the edge is nought.
    pub depth: u8,
    /// Out of sight in the mist until it is found, seen from the Hilltop, or read of.
    pub hidden: bool,
}

/// Who opens a way only some company takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Opener {
    /// An explorer, or anyone very curious, knows a deer track.
    Explorer,
    /// Only a little one squeezes through the gap in the hedge.
    LittleOne,
    /// Two close friends cross the stepping stones together, steadying each other.
    ClosePair,
    /// Someone bold takes the steep way up the crag.
    Bold,
    /// Someone who reads old things makes out the old way on the signpost, once there.
    Reader,
}

impl Opener {
    pub const ALL: [Self; 5] = [
        Self::Explorer,
        Self::LittleOne,
        Self::ClosePair,
        Self::Bold,
        Self::Reader,
    ];
}

/// How a path is walked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// Anyone can walk it.
    Open,
    /// Only some company takes it: what it is called, and who opens it.
    Shortcut(&'static str, Opener),
}

/// A path between two places.
#[derive(Clone, Copy, Debug)]
pub struct Path {
    pub ends: (usize, usize),
    /// How much of the day walking it takes, at a middling pace.
    pub length: f32,
    pub way: Way,
    /// The bends it takes on the way from its first end to its second, as drawn.
    pub bends: &'static [(f32, f32)],
}

impl Path {
    /// The other end, from one end; `None` from anywhere else.
    pub fn from(&self, place: usize) -> Option<usize> {
        match self.ends {
            (a, b) if a == place => Some(b),
            (a, b) if b == place => Some(a),
            _ => None,
        }
    }

    /// Every point along it, from `place` to its other end, as walked.
    pub fn walk_from(&self, place: usize) -> Vec<(f32, f32)> {
        let (a, b) = self.ends;
        let mut points = vec![PLACES[a].at];
        points.extend_from_slice(self.bends);
        points.push(PLACES[b].at);
        if place == b {
            points.reverse();
        }
        points
    }
}

pub const EDGE: usize = 0;
pub const HEDGEROW: usize = 1;
pub const MEADOW: usize = 2;
pub const GLADE: usize = 3;
pub const LOG: usize = 4;
pub const SIGNPOST: usize = 5;
pub const POOL: usize = 6;
pub const FAR_FALLS: usize = 7;
pub const OLD_TRACK: usize = 8;

/// The places, the edge of the Woods first. The map is painted round these.
pub const PLACES: [Place; 9] = [
    Place {
        id: "edge",
        name: "the edge of the Woods",
        stop: Stop::Edge,
        at: (46.0, 186.0),
        depth: 0,
        hidden: false,
    },
    Place {
        id: "hedgerow",
        name: "the hedgerow",
        stop: Stop::Forage,
        at: (104.0, 200.0),
        depth: 1,
        hidden: false,
    },
    Place {
        id: "meadow",
        name: "the meadow",
        stop: Stop::Bugs,
        at: (64.0, 126.0),
        depth: 1,
        hidden: false,
    },
    Place {
        id: "glade",
        name: "the glade",
        stop: Stop::Rummage,
        at: (172.0, 152.0),
        depth: 2,
        hidden: false,
    },
    Place {
        id: "log",
        name: "the fallen log",
        stop: Stop::Rest,
        at: (124.0, 86.0),
        depth: 2,
        hidden: false,
    },
    Place {
        id: "signpost",
        name: "the old signpost",
        stop: Stop::Fork,
        at: (226.0, 106.0),
        depth: 3,
        hidden: false,
    },
    Place {
        id: "pool",
        name: "the pool",
        stop: Stop::Fish,
        at: (290.0, 182.0),
        depth: 3,
        hidden: false,
    },
    Place {
        id: "far_falls",
        name: "the Far Falls",
        stop: Stop::Falls,
        at: (318.0, 84.0),
        depth: 4,
        hidden: true,
    },
    Place {
        id: "old_track",
        name: "the old track",
        stop: Stop::Scavenge,
        at: (236.0, 172.0),
        depth: 3,
        hidden: false,
    },
];

/// The paths, the open ones first. The map is painted along these.
pub const PATHS: [Path; 17] = [
    Path {
        ends: (EDGE, HEDGEROW),
        length: 6.0,
        way: Way::Open,
        // Along the lane.
        bends: &[(72.0, 198.0)],
    },
    Path {
        ends: (EDGE, MEADOW),
        length: 8.0,
        way: Way::Open,
        bends: &[(40.0, 164.0), (48.0, 142.0)],
    },
    Path {
        ends: (EDGE, GLADE),
        length: 10.0,
        way: Way::Open,
        bends: &[(82.0, 174.0), (126.0, 164.0)],
    },
    Path {
        ends: (MEADOW, LOG),
        length: 8.0,
        way: Way::Open,
        bends: &[(84.0, 102.0)],
    },
    Path {
        ends: (MEADOW, GLADE),
        length: 9.0,
        way: Way::Open,
        bends: &[(104.0, 140.0), (140.0, 148.0)],
    },
    Path {
        ends: (GLADE, LOG),
        length: 7.0,
        way: Way::Open,
        bends: &[(146.0, 116.0)],
    },
    Path {
        ends: (GLADE, SIGNPOST),
        length: 7.0,
        way: Way::Open,
        bends: &[(198.0, 132.0)],
    },
    Path {
        ends: (LOG, SIGNPOST),
        length: 8.0,
        way: Way::Open,
        bends: &[(166.0, 88.0), (198.0, 96.0)],
    },
    Path {
        ends: (SIGNPOST, POOL),
        length: 8.0,
        way: Way::Open,
        bends: &[(244.0, 128.0), (262.0, 152.0), (278.0, 172.0)],
    },
    Path {
        ends: (SIGNPOST, FAR_FALLS),
        length: 16.0,
        way: Way::Open,
        // The long way round, winding up under the crag.
        bends: &[(232.0, 78.0), (260.0, 62.0), (292.0, 62.0)],
    },
    Path {
        ends: (GLADE, OLD_TRACK),
        length: 7.0,
        way: Way::Open,
        // On from the glade, where the old cart track starts.
        bends: &[(198.0, 170.0)],
    },
    Path {
        ends: (OLD_TRACK, POOL),
        length: 6.0,
        way: Way::Open,
        bends: &[(262.0, 175.0)],
    },
    Path {
        ends: (HEDGEROW, POOL),
        length: 13.0,
        way: Way::Shortcut("the deer track", Opener::Explorer),
        // Along the bottom of the Woods.
        bends: &[(152.0, 208.0), (214.0, 208.0), (262.0, 198.0)],
    },
    Path {
        ends: (HEDGEROW, GLADE),
        length: 5.0,
        way: Way::Shortcut("the gap in the hedge", Opener::LittleOne),
        bends: &[(114.0, 188.0), (144.0, 168.0)],
    },
    Path {
        ends: (POOL, FAR_FALLS),
        length: 8.0,
        way: Way::Shortcut("the stepping stones", Opener::ClosePair),
        // Up the stream, stone to stone.
        bends: &[
            (284.0, 166.0),
            (304.0, 152.0),
            (318.0, 144.0),
            (332.0, 122.0),
            (338.0, 100.0),
        ],
    },
    Path {
        ends: (LOG, FAR_FALLS),
        length: 10.0,
        way: Way::Shortcut("the steep way up the crag", Opener::Bold),
        bends: &[(148.0, 58.0), (196.0, 40.0), (252.0, 36.0), (296.0, 50.0)],
    },
    Path {
        ends: (SIGNPOST, FAR_FALLS),
        length: 9.0,
        way: Way::Shortcut("the old way", Opener::Reader),
        bends: &[(262.0, 96.0), (294.0, 90.0)],
    },
];

/// The place with this id, if the map has one.
pub fn place(id: &str) -> Option<usize> {
    PLACES.iter().position(|place| place.id == id)
}

/// Who in a party opens each way only some company takes, read from who each is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Company {
    pub explorer: Option<Id>,
    pub little_one: Option<Id>,
    pub close_pair: Option<(Id, Id)>,
    pub bold: Option<Id>,
    pub reader: Option<Id>,
}

/// Whether a companion knows its way about the Woods: an explorer, or anyone very curious.
pub fn explorer(character: &Character) -> bool {
    character.kind == TemperamentKind::Explorer || character.axes.curiosity >= 0.8
}

/// Whether a companion takes the steep way.
pub fn bold(character: &Character) -> bool {
    character.axes.boldness >= 0.65
}

/// Whether a companion reads old things: a scholar, or anyone with as much of an eye for them.
pub fn reader(character: &Character) -> bool {
    Leaning::Old.pull(character) >= 0.65
}

impl Company {
    /// Who opens what among `party`, the first of each kind leading; `close` says whether two of
    /// them are close friends.
    pub fn of(party: &[(Id, Character)], close: impl Fn(Id, Id) -> bool) -> Self {
        let first = |test: &dyn Fn(&Character) -> bool| {
            party
                .iter()
                .find(|(_, character)| test(character))
                .map(|(id, _)| *id)
        };
        let close_pair = party.iter().enumerate().find_map(|(index, (a, _))| {
            party[index + 1..]
                .iter()
                .find(|(b, _)| close(*a, *b))
                .map(|(b, _)| (*a, *b))
        });
        Self {
            explorer: first(&explorer),
            little_one: first(&|character| character.parent.is_some()),
            close_pair,
            bold: first(&bold),
            reader: first(&reader),
        }
    }

    /// Whether this company opens a way.
    pub fn opens(&self, opener: Opener) -> bool {
        !self.who(opener).is_empty()
    }

    /// Who opens a way: one, or the pair.
    pub fn who(&self, opener: Opener) -> Vec<Id> {
        match opener {
            Opener::Explorer => self.explorer.into_iter().collect(),
            Opener::LittleOne => self.little_one.into_iter().collect(),
            Opener::ClosePair => self.close_pair.map(|(a, b)| vec![a, b]).unwrap_or_default(),
            Opener::Bold => self.bold.into_iter().collect(),
            Opener::Reader => self.reader.into_iter().collect(),
        }
    }
}

/// Whether a path can be walked by this company: an open one by anyone, a shortcut by those who
/// open it, and the old way only once the signpost has been read.
pub fn walkable(path: &Path, company: &Company, read: bool) -> bool {
    match path.way {
        Way::Open => true,
        Way::Shortcut(_, Opener::Reader) => read && company.opens(Opener::Reader),
        Way::Shortcut(_, opener) => company.opens(opener),
    }
}

/// How much more or less of the day walking takes this party: the slowest sets the pace, and a
/// lively party gets about for less.
pub fn pace(party: &[&Character]) -> f32 {
    let liveliness = party
        .iter()
        .map(|character| character.axes.energy)
        .fold(1.0, f32::min);
    1.0 / (0.8 + 0.4 * liveliness)
}

/// The places this company can reach from the edge, walking only what it can walk.
#[cfg(test)]
pub fn reachable(company: &Company, read: bool) -> std::collections::BTreeSet<usize> {
    let mut reached = std::collections::BTreeSet::from([EDGE]);
    let mut frontier = vec![EDGE];
    while let Some(here) = frontier.pop() {
        for path in PATHS.iter().filter(|path| walkable(path, company, read)) {
            if let Some(there) = path.from(here)
                && reached.insert(there)
            {
                frontier.push(there);
            }
        }
    }
    reached
}

/// The least of the day it takes this company to get from the edge to each place, at a middling
/// pace: `None` for anywhere it cannot get to.
#[cfg(test)]
pub fn nearest(company: &Company, read: bool) -> [Option<f32>; PLACES.len()] {
    let mut best: [Option<f32>; PLACES.len()] = [None; PLACES.len()];
    best[EDGE] = Some(0.0);
    let mut settled = [false; PLACES.len()];
    loop {
        let next = (0..PLACES.len())
            .filter(|place| !settled[*place])
            .filter_map(|place| best[place].map(|cost| (place, cost)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((here, cost)) = next else {
            break;
        };
        settled[here] = true;
        for path in PATHS.iter().filter(|path| walkable(path, company, read)) {
            if let Some(there) = path.from(here) {
                let through = cost + path.length;
                if best[there].is_none_or(|known| through < known) {
                    best[there] = Some(through);
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use formiga_travel::Band;
    use std::collections::BTreeSet;

    fn sample() -> (Cast, Vec<(Id, Character)>) {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let party = cast
            .members
            .iter()
            .map(|member| (member.id, Character::of(member)))
            .collect();
        (cast, party)
    }

    fn close(cast: &Cast) -> impl Fn(Id, Id) -> bool + '_ {
        |a, b| {
            cast.bond(a, b)
                .is_some_and(|bond| bond.warmth >= Band::High)
        }
    }

    #[test]
    fn every_place_is_reachable_by_any_single_sample_companion() {
        let (cast, everyone) = sample();
        for one in &everyone {
            let company = Company::of(std::slice::from_ref(one), close(&cast));
            for read in [false, true] {
                assert_eq!(
                    reachable(&company, read).len(),
                    PLACES.len(),
                    "{:?} on their own can't get everywhere",
                    one.1.kind
                );
            }
        }
        // Nobody at all opens anything, and still everywhere is reachable.
        assert_eq!(
            reachable(&Company::default(), false).len(),
            PLACES.len(),
            "somewhere is only reached by a shortcut"
        );
    }

    #[test]
    fn the_old_track_is_on_the_map_and_anyone_on_their_own_gets_there() {
        let (cast, everyone) = sample();
        let track = &PLACES[OLD_TRACK];
        assert_eq!(track.stop, Stop::Scavenge);
        assert!(track.stop.played());
        assert!(!track.hidden, "the old track is no secret");
        assert!(track.depth > PLACES[GLADE].depth, "it is further in");
        // On from the glade by an open path, and on to the pool by another.
        for (from, to) in [(GLADE, OLD_TRACK), (OLD_TRACK, POOL)] {
            assert!(
                PATHS
                    .iter()
                    .any(|path| path.way == Way::Open && path.from(from) == Some(to)),
                "no open path from {} to {}",
                PLACES[from].name,
                PLACES[to].name
            );
        }
        for one in &everyone {
            let company = Company::of(std::slice::from_ref(one), close(&cast));
            assert!(
                reachable(&company, false).contains(&OLD_TRACK),
                "{:?} on their own can't get to the old track",
                one.1.kind
            );
        }
        assert!(nearest(&Company::default(), false)[OLD_TRACK].is_some());
    }

    #[test]
    fn shortcuts_and_company_only_add() {
        let (cast, everyone) = sample();
        let bare = nearest(&Company::default(), false);
        // Every party of one, two or three from the sample.
        let mut parties: Vec<Vec<(Id, Character)>> = Vec::new();
        for (a, first) in everyone.iter().enumerate() {
            parties.push(vec![first.clone()]);
            for (b, second) in everyone.iter().enumerate().skip(a + 1) {
                parties.push(vec![first.clone(), second.clone()]);
                for third in everyone.iter().skip(b + 1) {
                    parties.push(vec![first.clone(), second.clone(), third.clone()]);
                }
            }
        }
        for party in &parties {
            let company = Company::of(party, close(&cast));
            let opened = nearest(&company, true);
            for place in 0..PLACES.len() {
                let (with, without) = (opened[place].unwrap(), bare[place].unwrap());
                assert!(
                    with <= without,
                    "{} is further for company than for nobody",
                    PLACES[place].name
                );
            }
            // Bringing someone else along never shuts a way.
            for (index, _) in party.iter().enumerate() {
                let fewer: Vec<(Id, Character)> = party
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                    .map(|(_, member)| member.clone())
                    .collect();
                let less = Company::of(&fewer, close(&cast));
                for path in &PATHS {
                    if walkable(path, &less, true) {
                        assert!(walkable(path, &company, true), "{path:?} shut by company");
                    }
                }
            }
        }
        // And each shortcut does save something for whoever opens it.
        for path in PATHS
            .iter()
            .filter(|path| matches!(path.way, Way::Shortcut(..)))
        {
            let (a, b) = path.ends;
            let around = {
                let mut costs = [f32::MAX; PLACES.len()];
                costs[a] = 0.0;
                for _ in 0..PLACES.len() {
                    for open in PATHS.iter().filter(|p| p.way == Way::Open) {
                        let (x, y) = open.ends;
                        costs[y] = costs[y].min(costs[x] + open.length);
                        costs[x] = costs[x].min(costs[y] + open.length);
                    }
                }
                costs[b]
            };
            assert!(
                path.length < around,
                "{:?} is no shorter than the open way round",
                path.way
            );
        }
    }

    #[test]
    fn who_opens_what_is_read_from_who_they_are() {
        let (cast, everyone) = sample();
        let named = |name: &str| {
            everyone
                .iter()
                .find(|(id, _)| cast.member(*id).unwrap().name == name)
                .unwrap()
                .clone()
        };
        let pip = Company::of(&[named("Pip")], close(&cast));
        assert!(pip.opens(Opener::LittleOne) && !pip.opens(Opener::Bold));
        let tansy = Company::of(&[named("Tansy")], close(&cast));
        assert!(tansy.opens(Opener::Explorer) && tansy.opens(Opener::Bold));
        let mochi = Company::of(&[named("Mochi")], close(&cast));
        assert_eq!(mochi, Company::default(), "Mochi alone opens nothing");
        let pair = Company::of(&[named("Mochi"), named("Button")], close(&cast));
        assert!(pair.opens(Opener::ClosePair));
        let strangers = Company::of(&[named("Mochi"), named("Fig")], close(&cast));
        assert!(!strangers.opens(Opener::ClosePair));
        // Nobody in the sample reads old signs, but a scholar does.
        let mut scholar = named("Mochi");
        scholar.1.kind = TemperamentKind::Scholar;
        assert!(Company::of(&[scholar], close(&cast)).opens(Opener::Reader));
    }

    #[test]
    fn the_map_is_well_formed() {
        for (index, path) in PATHS.iter().enumerate() {
            let (a, b) = path.ends;
            assert!(
                a < PLACES.len() && b < PLACES.len() && a != b,
                "path {index}"
            );
            assert!(path.length > 0.0);
            let walked = path.walk_from(b);
            assert_eq!(walked.first(), Some(&PLACES[b].at));
            assert_eq!(walked.last(), Some(&PLACES[a].at));
        }
        let ids: BTreeSet<&str> = PLACES.iter().map(|place| place.id).collect();
        assert_eq!(ids.len(), PLACES.len());
        for (index, place) in PLACES.iter().enumerate() {
            assert_eq!(super::place(place.id), Some(index));
            let (x, y) = place.at;
            assert!((8.0..376.0).contains(&x) && (24.0..212.0).contains(&y));
        }
        assert_eq!(PLACES[EDGE].stop, Stop::Edge);
        assert!(
            PLACES
                .iter()
                .any(|place| place.stop == Stop::Falls && place.hidden)
        );
    }
}
